//! Orders: the per-unit order list, the three enqueue modes, one order stepped
//! per frame, and the lifecycle every other mechanic hangs off.
//! `docs/ORDERS.md`.
//!
//! The original keeps a circular ring of `UnitOrder`s per unit whose current
//! order is `head->prev`, the oldest; `QUEUE_LAST` appends, `QUEUE_FIRST`
//! rotates the new order to the front, `QUEUE_NEW` kills everything first.
//! Here that is a [`VecDeque`] whose **front is the current order**. The step
//! is `Unit::work` → `do_job`: one dispatch on the front order per unit per
//! frame, after attrition and before the body follows (`docs/ORDERS.md` §2).
//! An order a `do_*` inserts in front (`QUEUE_FIRST`) runs from the next
//! frame; an order applied from outside the step runs in the same frame.
//!
//! What is modelled and what is an input is §13 of the document. The
//! pathfinder is a named seam: [`Sim::find_path`] is the straight-line
//! verifier, and `find_wpath` is the stub that leaves `[goal]` on the stack —
//! exact within two world cells on open ground, and stated wrong beyond.

use crate::build::{self, Ident, flags as bflags};
use crate::combat::{self, Obj};
use crate::garrison::GarrisonRefused;
use crate::movement::{self, Angle, find_angle};
use crate::world::vector_dist;
use crate::{Player, Pos, Sim};

/// `OrderIndex` — the value the gamelog's `type` line carries.
pub mod index {
    pub const NONE: u8 = 0;
    pub const MOVE_TO: u8 = 1;
    pub const ATTACK_TO: u8 = 2;
    pub const EXPLORE_TO: u8 = 3;
    pub const FLEE_TO: u8 = 4;
    pub const BUILD_AT: u8 = 6;
    pub const GATHER: u8 = 7;
    pub const ATTACK: u8 = 10;
    pub const REPAIR: u8 = 13;
    pub const GARRISON: u8 = 26;
    pub const THINK: u8 = 27;
}

/// `UnitOrder::flags` bits (`docs/ORDERS.md` §1.3).
pub mod flag {
    /// The top segment of the unit's path stack is this move's.
    pub const PATHED: u8 = 0x1;
    pub const FLEEING: u8 = 0x2;
    /// This order is an intent, not a transit leg: `get_action` stops on it.
    pub const ACTION: u8 = 0x4;
    /// A DEFENSIVE unit's "this move is my post".
    pub const POST: u8 = 0x8;
    /// An attack order's "re-target requested".
    pub const RETARGET: u8 = 0x10;
}

/// `QueuePos` — where an order goes (§1.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueuePos {
    /// Ctrl-click: the new order runs now, the old current resumes after it.
    First,
    /// Shift-click: after everything queued.
    Last,
    /// Plain click: everything is killed first.
    New,
}

/// `MoveOrder`'s kind: which `do_job` case steps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveKind {
    MoveTo,
    AttackTo,
    ExploreTo,
    FleeTo,
}

/// The fields of `MoveOrder` that are live on open ground (§4.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOrder {
    pub kind: MoveKind,
    /// The destination, snapped to its 48-unit cell centre.
    pub dest: Pos,
    /// The facing to apply on arrival — the direction from the unit to the
    /// target at order time.
    pub angle: Angle,
    /// "I have a current waypoint."
    pub has_waypoint: bool,
    /// The current waypoint — what the step walks toward.
    pub waypoint: Pos,
    /// Where the last straight-line plan was made.
    pub last: Option<Pos>,
    pub pause: i32,
    pub timer: i32,
}

/// One entry of the unit's path stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathData {
    pub to: Pos,
    pub tolerance: i32,
    pub flags: u8,
}

/// `PathData::flags`.
pub mod path_flag {
    /// The final waypoint of an order's segment — the goal.
    pub const FINAL: u8 = 0x1;
    pub const SIDESTEP: u8 = 0x2;
    /// Turn in place before walking to this one.
    pub const TURN_FIRST: u8 = 0x4;
}

/// The fields of `GatherOrder` (§1.1, §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherOrder {
    pub building: usize,
    /// The resource tile a woodcutter or miner is working, in tiles.
    pub tile: Option<Pos>,
    pub wait: i32,
    /// Non-flat: heading to or at the camp (true) vs out at the tile.
    pub goto_build: bool,
    pub dist_mod: i32,
    /// The unit has arrived; the economy counts only these.
    pub been_there: bool,
}

/// The fields of `AttackOrder` the order keeps; the target and `mandatory`
/// stay in [`combat::State`], which `fight.rs` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackOrder {
    pub defensive: bool,
    pub def: Option<Pos>,
    pub in_range: bool,
    pub ever_in_range: bool,
    /// "Has not struck yet" — 1 at creation, cleared by the first strike.
    pub new_ord: bool,
}

/// The order kinds this crate implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Move(MoveOrder),
    Build(usize),
    Repair(usize),
    Garrison { building: usize, search: bool },
    Gather(GatherOrder),
    Attack(AttackOrder),
    Think,
}

/// One order in a unit's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub flags: u8,
    pub body: Body,
}

impl Order {
    /// `get_type()` — the `OrderIndex` value.
    pub const fn index(&self) -> u8 {
        match self.body {
            Body::Move(m) => match m.kind {
                MoveKind::MoveTo => index::MOVE_TO,
                MoveKind::AttackTo => index::ATTACK_TO,
                MoveKind::ExploreTo => index::EXPLORE_TO,
                MoveKind::FleeTo => index::FLEE_TO,
            },
            Body::Build(_) => index::BUILD_AT,
            Body::Repair(_) => index::REPAIR,
            Body::Garrison { .. } => index::GARRISON,
            Body::Gather(_) => index::GATHER,
            Body::Attack(_) => index::ATTACK,
            Body::Think => index::THINK,
        }
    }

    pub const fn is_move(&self) -> bool {
        matches!(self.body, Body::Move(_))
    }

    pub const fn has(&self, f: u8) -> bool {
        self.flags & f != 0
    }

    /// A plain transit move: a move without the action bit — what
    /// `get_action` walks past.
    pub const fn is_transit(&self) -> bool {
        self.is_move() && !self.has(flag::ACTION)
    }

    const fn move_mut(&mut self) -> Option<&mut MoveOrder> {
        match &mut self.body {
            Body::Move(m) => Some(m),
            _ => None,
        }
    }
}

/// What `Unit::do_move` reports back to `work`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Did {
    Nothing,
    Something,
}

/// `ACCEL_CONSTRUCT`-independent literals of the mechanic, position units.
const SNAP: i32 = 0x30;
const SNAP_CENTRE: i32 = 0x18;
const HALF_TILE: i32 = 0x60;
const TILE: i32 = 0xc0;
const CELL: i32 = 0x300;
/// `find_path`'s "far and reachable: the pathfinder's job" rule, world cells.
const STRAIGHT_LINE_CELLS: i32 = 4;
/// The working-the-tile radius of a woodcutter or miner.
const AT_TILE: i32 = 0x140;
/// The oil-well stance offset.
const OILWELL_OFFSET: i32 = 0x120;
/// Adjacency: `attack_dist < 0x60`.
const ADJACENT: i32 = 0x60;
/// How many `Farms::grow` calls flip a wheat tile — the float accumulator's
/// `0.005f` to `1.0f`, pinned as a count. 200 or 201 is a behavioural check.
pub const FARM_GROWS: i32 = 200;

/// The 31 bearings of one ring of `find_nearby_spot`, as multiples of a
/// sixteenth of a turn from the base angle; `|k| >= 8` adds a thirty-second.
const BEARINGS: [i32; 31] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8, 9, -9, 10, -10, 11, -11, 12, -12,
    13, -13, 14, -14, 15, -15,
];

/// Which worker kind a unit type is, for the gather chain and `think_peasant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Worker {
    #[default]
    None,
    Citizen,
    Scholar,
}

impl Sim {
    // ------------------------------------------------------------------
    // The list
    // ------------------------------------------------------------------

    /// The current order — `Unit::update_order`, `head->prev`.
    pub fn current_order(&self, u: usize) -> Option<&Order> {
        self.units[u].orders.front()
    }

    /// `UnitData::order_type`: the current order's `OrderIndex`, `NONE` for
    /// an empty list.
    pub fn order_type(&self, u: usize) -> u8 {
        self.current_order(u).map_or(index::NONE, Order::index)
    }

    /// `UnitData::get_action`: the position in the list of the first order
    /// that is not a transit move — the intent under the pathing legs.
    pub fn action_of(&self, u: usize) -> Option<usize> {
        self.units[u].orders.iter().position(|o| !o.is_transit())
    }

    /// `Unit::update_action`: `orders_x/y` = the final destination of the
    /// leading run of transit moves, `dest_angle` = the last one's angle;
    /// returns the action's position (§3.3).
    pub fn update_action(&mut self, u: usize) -> Option<usize> {
        let unit = &mut self.units[u];
        unit.orders_pos = unit.pos;
        unit.movement.des_angle = unit.movement.facing;
        let mut action = None;
        for (i, o) in unit.orders.iter().enumerate() {
            match o.body {
                Body::Move(m) if !o.has(flag::ACTION) => {
                    unit.orders_pos = m.dest;
                    unit.movement.des_angle = m.angle;
                }
                Body::Move(m) => {
                    // A move that is itself the action: `update_action`
                    // copies it once more and stops.
                    unit.orders_pos = m.dest;
                    unit.movement.des_angle = m.angle;
                    action = Some(i);
                    break;
                }
                _ => {
                    action = Some(i);
                    break;
                }
            }
        }
        action
    }

    /// The generic enqueue (§3.1): `QUEUE_NEW` clears first, then the order
    /// is appended (`Last`, `New`) or rotated to the front (`First`).
    fn enqueue(&mut self, u: usize, order: Order, pos: QueuePos) {
        match pos {
            QueuePos::New => {
                self.units[u].path.clear();
                self.close_orders(u);
                self.clear_partial_path(u);
                self.update_action(u);
                self.units[u].orders.push_back(order);
            }
            QueuePos::Last => self.units[u].orders.push_back(order),
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
            }
        }
        self.update_action(u);
    }

    /// `clear_partial_path`: there is no suspended search here; the scratch
    /// it would clear is the verified-line bit.
    fn clear_partial_path(&mut self, u: usize) {
        self.units[u].line_ok = false;
    }

    /// `Unit::kill_current_order(0)` (§3.2): the per-kind teardown, then the
    /// pop, the path segment, `update_action`.
    pub fn kill_current_order(&mut self, u: usize) {
        let Some(order) = self.units[u].orders.front().copied() else {
            return;
        };
        match order.body {
            Body::Gather(g) => {
                let who = self.units[u].owner;
                self.ledgers[who as usize].dirty = true;
                if self.buildings.get(g.building).is_some_and(|b| b.alive)
                    && self.buildings[g.building].owner == who
                {
                    self.remove_gatherer(g.building, u);
                }
            }
            Body::Attack(_) => {
                // The order is the target's home; dropping it drops the target.
                let unit = &mut self.units[u];
                unit.combat.target = None;
                unit.combat.mandatory = false;
            }
            _ => {}
        }
        self.units[u].orders.pop_front();
        if order.is_move() && order.has(flag::PATHED) {
            self.kill_current_path(u);
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// Remove the order the cursor sits on, `i` places behind the current
    /// one. Only `check_build_order` walks a cursor, and only onto a
    /// `BUILD_AT`, which carries no per-kind teardown — so this is
    /// [`Sim::kill_current_order`]'s generic tail applied at an index.
    fn remove_order_at(&mut self, u: usize, i: usize) {
        if self.units[u].orders.remove(i).is_none() {
            return;
        }
        self.units[u].movement.dest = None;
        self.clear_partial_path(u);
        self.update_action(u);
    }

    /// `kill_current_path`: pop the stack until an entry with the final flag
    /// is popped.
    fn kill_current_path(&mut self, u: usize) {
        let path = &mut self.units[u].path;
        while let Some(p) = path.pop() {
            if p.flags & path_flag::FINAL != 0 {
                break;
            }
        }
    }

    /// `Unit::close_orders`: kill everything, oldest first.
    pub fn close_orders(&mut self, u: usize) {
        while !self.units[u].orders.is_empty() {
            self.kill_current_order(u);
        }
    }

    /// `Unit::clear_orders`: the `QUEUE_NEW` clear without the add.
    pub fn clear_orders(&mut self, u: usize) {
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
    }

    // ------------------------------------------------------------------
    // The adders
    // ------------------------------------------------------------------

    /// `Unit::add_move_order` → `add_move_facing_order` (§4.3): the 48-unit
    /// snap, the angle at order time, the action bit from the caller.
    pub fn add_move_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
    ) {
        let dest = Pos::new(
            to.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
            to.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        );
        let here = self.units[u].pos;
        let angle = find_angle(dest.x - here.x, dest.y - here.y);
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Move(MoveOrder {
                kind,
                dest,
                angle,
                has_waypoint: false,
                waypoint: dest,
                last: None,
                pause: 0,
                timer: 0,
            }),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_build_order`.
    pub fn add_build_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Build(b),
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_repair_order`.
    pub fn add_repair_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.units[u].was_builder = true;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Repair(b),
        };
        // The one adder that is **not** §3.1's common shape: `add_repair_
        // order@005e4ff0` has no `QUEUE_FIRST` tail at all — after the list
        // add at `0x5e51f0` come only `update_action` and `ret 0x10`, where
        // `add_build_order` has an explicit `cmp [ebp+0x10],0` →
        // `clear_partial_path; head = head->next`. No caller passes
        // `QUEUE_FIRST`, so this is fidelity, not a live bug.
        // (`docs/audit/2026-08-21-orders.md` R3 P1.)
        debug_assert!(
            pos != QueuePos::First,
            "add_repair_order has no QUEUE_FIRST branch in the original"
        );
        let pos = if pos == QueuePos::First {
            QueuePos::Last
        } else {
            pos
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_garrison_order(o, who, search, pos, action)`.
    pub fn add_garrison_order(
        &mut self,
        u: usize,
        b: usize,
        search: bool,
        pos: QueuePos,
        action: bool,
    ) {
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Garrison {
                building: b,
                search,
            },
        };
        self.enqueue(u, order, pos);
    }

    /// `Unit::add_gather_order(o, pos, action)` (§6.2): the unit joins the
    /// building's chain **now**, before it has moved, if the type is a
    /// gatherer; the non-flat fields are set for wood and ore.
    pub fn add_gather_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        if pos == QueuePos::New {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        let mut dist_mod = 0;
        if self.is_gather_type(b) {
            self.add_gatherer(b, u);
            if !self.is_flat(b) && self.building_ident(b) != Ident::University {
                dist_mod = if self.building_ident(b) == Ident::Mine {
                    10
                } else {
                    4
                };
            }
        }
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Gather(GatherOrder {
                building: b,
                tile: None,
                wait: 0,
                goto_build: true,
                dist_mod,
                been_there: false,
            }),
        };
        self.units[u].orders.push_back(order);
        if pos == QueuePos::First {
            let o = self.units[u].orders.pop_back().expect("just pushed");
            self.clear_partial_path(u);
            self.units[u].orders.push_front(o);
        }
        self.update_action(u);
    }

    /// `Unit::add_attack_order(o, who, pos, mandatory, action)` (§7.1). The
    /// target and `mandatory` live in [`combat::State`]; the order carries
    /// the rest.
    pub fn add_attack_order(
        &mut self,
        u: usize,
        target: Obj,
        pos: QueuePos,
        mandatory: bool,
        action: bool,
    ) {
        if pos == QueuePos::New {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        let unit = &self.units[u];
        let defensive = unit.combat.stance == combat::Stance::Defensive && !action;
        let def = if defensive {
            Some(self.find_def_pos(u))
        } else {
            None
        };
        self.bump_targeted_pub(target, 1);
        let unit = &mut self.units[u];
        unit.combat.target = Some(target);
        unit.combat.mandatory = mandatory;
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body: Body::Attack(AttackOrder {
                defensive,
                def,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        };
        match pos {
            QueuePos::First => {
                self.clear_partial_path(u);
                self.units[u].orders.push_front(order);
            }
            _ => self.units[u].orders.push_back(order),
        }
        self.update_action(u);
    }

    /// `UnitData::find_def_pos`: an existing DEFENSIVE attack's post, else a
    /// head move carrying the post flag, else the unit's own quarter-tile
    /// centre.
    fn find_def_pos(&self, u: usize) -> Pos {
        for o in &self.units[u].orders {
            match o.body {
                Body::Attack(a) if a.defensive => {
                    if let Some(p) = a.def {
                        return p;
                    }
                }
                Body::Move(m) if o.has(flag::POST) => return m.dest,
                _ => {}
            }
        }
        let p = self.units[u].pos;
        Pos::new(
            p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
            p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        )
    }

    /// `Unit::add_think_order`: the argument is ignored — it is always
    /// rotated to the front, with the action bit.
    pub fn add_think_order(&mut self, u: usize) {
        let order = Order {
            flags: flag::ACTION,
            body: Body::Think,
        };
        self.units[u].orders.push_front(order);
        self.clear_partial_path(u);
        self.update_action(u);
    }

    // ------------------------------------------------------------------
    // The step — `Unit::work` → `do_job`
    // ------------------------------------------------------------------

    /// `Unit::work` (§2.3): the liveness test on the action's target, the
    /// idle reset, then one `do_job` on the front order.
    pub(crate) fn work(&mut self, u: usize, frame: i64) {
        // The recharging-melee gate: a recharging melee unit steps only an
        // order carrying the action bit, and never an ATTACK through here
        // (ATTACK's own entry gate is `fight`'s).
        if let Some(o) = self.current_order(u).copied()
            && o.index() != index::ATTACK
            && self.units[u].combat.recharging != 0
            && self.max_range_of(Obj::Unit(u)) == 0
            && !o.has(flag::ACTION)
        {
            return;
        }
        // Target liveness: a build, repair, garrison or gather whose target
        // died (the `uid` test — indices are never reused here, so it is the
        // alive bit) dies before its `do_*` ever runs.
        if let Some(a) = self.action_of(u) {
            let dead = match self.units[u].orders[a].body {
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    !self.buildings.get(b).is_some_and(|bd| bd.alive)
                }
                Body::Gather(g) => !self.buildings.get(g.building).is_some_and(|bd| bd.alive),
                _ => false,
            };
            if dead {
                self.repath(u);
                self.kill_current_order(u);
                return;
            }
        }
        if !self.units[u].orders.is_empty() {
            self.units[u].idle = 0;
        }
        match self.current_order(u).map(|o| o.body) {
            None => self.do_idle(u, frame),
            Some(Body::Move(_)) => {
                self.do_move(u, frame);
            }
            Some(Body::Build(_)) => self.do_build(u, frame),
            Some(Body::Repair(_)) => self.do_repair(u, frame),
            Some(Body::Garrison { .. }) => self.do_garrison_order(u),
            Some(Body::Gather(_)) => self.do_gather(u, frame),
            Some(Body::Attack(_)) => self.do_attack(u, frame),
            Some(Body::Think) => self.do_think_order(u, frame),
        }
    }

    /// `Unit::repath`: pop the leading transit legs so the target order
    /// re-issues them.
    fn repath(&mut self, u: usize) {
        while self.current_order(u).is_some_and(Order::is_transit) {
            self.kill_current_order(u);
        }
    }

    // ------------------------------------------------------------------
    // Idle — `do_idle`, `check_idle`, `think`
    // ------------------------------------------------------------------

    /// `Unit::do_idle`: `collide = 0`, `check_idle`, `think`.
    fn do_idle(&mut self, u: usize, frame: i64) {
        self.check_idle(u, frame);
        self.think(u, frame);
    }

    /// `Unit::check_idle`: 0 → 1 → 2 on consecutive frames, then +1 every
    /// 16 frames phased by `o`, wrapping to 1.
    fn check_idle(&mut self, u: usize, frame: i64) {
        let unit = &mut self.units[u];
        if unit.idle == 0xff {
            unit.idle = 1;
            return;
        }
        if unit.idle > 1 && (frame + i64::from(unit.index)) & 15 != 0 {
            return;
        }
        unit.idle += 1;
    }

    /// `Unit::think` (§2.4): the auto-attack on the first idle frame and
    /// every 32 after, phased by `o`; a worker's `think_peasant(0)`.
    fn think(&mut self, u: usize, frame: i64) {
        let unit = &self.units[u];
        let phase = frame + i64::from(unit.index);
        // The global cadence gate (`Unit::think@005f6e40:87`, found by the
        // second reading — `docs/audit/2026-08-21-orders.md` R1): once a unit
        // has been idle for more than two frames it thinks only one frame in
        // sixteen, phased by `o`. The "could not reach" bit exempts it, so a
        // unit that has just failed to reach something keeps searching every
        // frame.
        if !unit.cant_reach && unit.idle > 2 && phase % 16 != 0 {
            return;
        }
        let me = Obj::Unit(u);
        if self.attack_of(me) != 0
            && (unit.idle == 1 || phase & 0x1f == 0)
            && unit.combat.stance != combat::Stance::HoldFire
            && unit.combat.target.is_none()
            && let Some(t) = self.find_melee_target(u, -1)
        {
            self.add_attack_order(u, t, QueuePos::New, false, false);
            return;
        }
        if self.worker_of(u) != Worker::None {
            self.think_peasant(u, false);
        }
    }

    /// `Unit::think_peasant(forced)` (§5.9): the idle gate, then the job
    /// search — here `find_gather_spot`; `find_build_spot`/`find_repair_spot`
    /// need the object searches and are not yet modelled.
    fn think_peasant(&mut self, u: usize, forced: bool) -> bool {
        let unit = &self.units[u];
        if !forced {
            let t = i32::from(unit.idle_threshold);
            let idle = i32::from(unit.idle);
            if idle < t {
                return false;
            }
            if idle != t && (idle - 2).rem_euclid(5) != 0 {
                return false;
            }
        }
        let stance = unit.stance;
        if stance <= 1 && self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE) {
            return true;
        }
        self.units[u].was_builder = false;
        false
    }

    /// `Unit::do_think_order`: consume the THINK; if nothing follows, a
    /// worker runs the forced search.
    fn do_think_order(&mut self, u: usize, _frame: i64) {
        self.kill_current_order(u);
        if self.order_type(u) != index::NONE {
            return;
        }
        if self.worker_of(u) != Worker::None {
            self.think_peasant(u, true);
        }
    }

    // ------------------------------------------------------------------
    // The move order — `Unit::do_move`
    // ------------------------------------------------------------------

    /// `Unit::do_move` (§4.4) on open ground: the planning half, the
    /// waypoint take with both arrival tests, the straight-line check, the
    /// step.
    fn do_move(&mut self, u: usize, frame: i64) -> Did {
        let Some(Order {
            body: Body::Move(mut mo),
            flags,
        }) = self.current_order(u).copied()
        else {
            return Did::Nothing;
        };
        let mut flags = flags;

        // `timer`: a self-destruct.
        if mo.timer > 0 {
            if mo.timer == 1 {
                self.kill_current_order(u);
                self.work(u, frame);
                return Did::Nothing;
            }
            mo.timer -= 1;
            self.store_move(u, mo, flags);
        }

        // The action under the move: an ATTACK whose target is in range has
        // no further use for the chase; one whose target is gone re-paths.
        if let Some(a) = self.action_of(u)
            && let Body::Attack(_) = self.units[u].orders[a].body
        {
            let me = Obj::Unit(u);
            match self.units[u].combat.target {
                Some(t) if self.valid_target(me, t) => {
                    if self.is_in_range(me, t) {
                        self.kill_current_order(u);
                        return Did::Something;
                    }
                }
                _ => {
                    self.repath(u);
                    return Did::Nothing;
                }
            }
        }

        // Planning: the first time, or with an empty stack.
        if flags & flag::PATHED == 0 || self.units[u].path.is_empty() {
            self.units[u].path.push(PathData {
                to: mo.dest,
                tolerance: 0,
                flags: path_flag::FINAL,
            });
            self.find_wpath(u);
            flags |= flag::PATHED;
            self.store_move(u, mo, flags);
            if self.units[u].path.len() > 10 {
                return Did::Something;
            }
        }

        // Taking a waypoint.
        if !mo.has_waypoint {
            let Some(top) = self.units[u].path.last().copied() else {
                // Nothing to walk: the goal was already here.
                self.kill_current_order(u);
                return Did::Nothing;
            };
            mo.has_waypoint = true;
            mo.waypoint = top.to;
            self.units[u].line_ok = false;
            self.units[u].tolerance = top.tolerance;
            let here = self.units[u].pos;
            // Already there: the Euclidean test before the step.
            if vector_dist(mo.waypoint.x - here.x, mo.waypoint.y - here.y)
                <= self.units[u].tolerance
            {
                mo.has_waypoint = false;
                let popped = self.units[u].path.pop();
                self.store_move(u, mo, flags);
                if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                    return Did::Something;
                }
                self.arrive(u, mo);
                return Did::Nothing;
            }
            self.store_move(u, mo, flags);
        }

        // The speed, and the straight-line check.
        let speed = self.units[u].movement.speed;
        if !self.units[u].line_ok {
            self.units[u].path_recursion = 0;
            let r = self.find_path(u, mo.waypoint);
            if r == 0 {
                let top = self.units[u].path.last().copied();
                match top {
                    Some(t) if t.to != self.units[u].pos => {
                        self.units[u].line_ok = true;
                        mo.has_waypoint = true;
                        mo.waypoint = t.to;
                        self.units[u].tolerance = t.tolerance;
                        self.store_move(u, mo, flags);
                    }
                    _ => self.units[u].line_ok = false,
                }
            }
            if !self.units[u].line_ok {
                // The straight line is not enough: the pathfinder's job —
                // and the draw that chooses the grid happens whether or not
                // a pathfinder answers.
                let here = self.units[u].pos;
                let n = self.rng.roll();
                let _thr = match n % 5 {
                    2 => 2 * CELL,
                    0 => 8 * CELL,
                    _ => 5 * CELL,
                };
                let _far = (mo.waypoint.x - here.x).abs() + (mo.waypoint.y - here.y).abs() > _thr;
                // The stub: every grid leaves `[goal]` (§4.6 — exact within
                // two cells, stated wrong beyond). The original then takes
                // the top — a near cell centre — and verifies the line to it
                // the same frame; with no chain to take, the straight line to
                // the goal is walked as the stand-in, and the step is on this
                // frame as it would be.
                self.find_wpath(u);
                let Some(top) = self.units[u].path.last().copied() else {
                    self.kill_current_order(u);
                    return Did::Something;
                };
                mo.last = None;
                mo.has_waypoint = true;
                mo.waypoint = top.to;
                self.units[u].tolerance = top.tolerance;
                self.units[u].path_recursion = 0;
                let _ = self.find_path(u, top.to);
                self.units[u].line_ok = true;
                self.store_move(u, mo, flags);
            }
        }

        // Stepping.
        if mo.pause != 0 {
            mo.pause -= 1;
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        self.unit_step(u, mo, speed)
    }

    /// Writes a move order's fields back to the front of the list.
    fn store_move(&mut self, u: usize, mo: MoveOrder, flags: u8) {
        if let Some(front) = self.units[u].orders.front_mut() {
            front.flags = flags;
            if let Some(m) = front.move_mut() {
                *m = mo;
            }
        }
    }

    /// The stub for `PathFinder::find_wpath`: the goal stays on the stack and
    /// the length is returned — exact for a goal within two world cells on
    /// open ground, and the stated stand-in beyond.
    fn find_wpath(&mut self, u: usize) -> i32 {
        self.units[u].path.len() as i32
    }

    /// `Unit::find_path` (§4.6): 0 = walk straight, 1 = plan, 2 = abort.
    /// On open ground the march never meets an invalid tile, so the answer
    /// is the four-cell rule.
    fn find_path(&mut self, u: usize, goal: Pos) -> u8 {
        let here = self.units[u].pos;
        if goal == here {
            return 0;
        }
        let cells = (goal.cell().x - here.cell().x).abs() + (goal.cell().y - here.cell().y).abs();
        if cells > STRAIGHT_LINE_CELLS && self.world.accepts(goal) {
            return 1;
        }
        let unit = &mut self.units[u];
        unit.path_recursion = unit.path_recursion.saturating_add(1);
        // The march: every tile the straight line crosses must be valid.
        let speed = unit.movement.speed.max(3);
        let ang = find_angle(goal.x - here.x, goal.y - here.y);
        let sx = movement::sin_component(ang, speed);
        let cy = movement::cos_component(ang, speed);
        let mut at = here;
        let mut last_manh = i32::MAX;
        loop {
            let (dx, dy) = (goal.x - at.x, goal.y - at.y);
            if dx.abs() <= speed && dy.abs() <= speed {
                return 0;
            }
            let manh = dx.abs() + dy.abs();
            if manh > last_manh {
                return 0;
            }
            last_manh = manh;
            let step_x = if sx.abs() > dx.abs() { dx } else { sx };
            let step_y = if cy.abs() > dy.abs() { -dy } else { cy };
            let next = Pos::new(at.x + step_x, at.y - step_y);
            if next.tile() != at.tile() && !self.world.accepts(next) {
                // Blocked, and `go_around_building` is not modelled: plan.
                return 1;
            }
            at = next;
        }
    }

    /// The unit step and its arrival (§4.5): `move_step` through
    /// `docs/MOVEMENT.md`, then the Manhattan test against the waypoint's
    /// tolerance, the pop, the facing on the final waypoint, the kill.
    fn unit_step(&mut self, u: usize, mut mo: MoveOrder, speed: i32) -> Did {
        let unit = &self.units[u];
        let m = unit.movement;
        let from = unit.pos;
        let rate = movement::turn_speed(
            &self.tuning,
            &m.turning,
            m.body.last_speed,
            m.body.avg_speed,
            movement::TurnMode::Unit,
        );
        let step = movement::move_step(from, m.facing, mo.waypoint, speed, &m.turning, rate);
        let unit = &mut self.units[u];
        unit.movement.facing = step.facing;
        unit.movement.des_angle = step.heading;
        let mut arrived = false;
        if self.world.accepts(step.pos) {
            self.units[u].pos = step.pos;
            if step.arrived {
                arrived = true;
            } else {
                let (dx, dy) = (mo.waypoint.x - step.pos.x, mo.waypoint.y - step.pos.y);
                if dx.abs() + dy.abs() <= self.units[u].tolerance {
                    arrived = true;
                }
            }
        } else {
            // A step the world refuses: re-plan next frame.
            self.units[u].line_ok = false;
            self.units[u].movement.dest = None;
            return Did::Nothing;
        }
        self.units[u].movement.dest = Some(mo.waypoint);
        if !arrived {
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        mo.has_waypoint = false;
        let popped = self.units[u].path.pop();
        let flags = self.current_order(u).map_or(0, |o| o.flags);
        self.store_move(u, mo, flags);
        self.units[u].movement.dest = None;
        if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
            // Next frame takes the next waypoint and re-checks the line.
            self.units[u].line_ok = false;
            return Did::Something;
        }
        self.arrive(u, mo);
        Did::Something
    }

    /// The final waypoint: `set_angle(mo->angle)` when the move is the only
    /// order or the action is a gather; clear the pathed bit; kill.
    fn arrive(&mut self, u: usize, mo: MoveOrder) {
        let only = self.units[u].orders.len() == 1;
        let gather_beneath = self
            .action_of(u)
            .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Gather(_)));
        if only || gather_beneath {
            self.units[u].movement.set_facing(mo.angle);
        }
        if let Some(front) = self.units[u].orders.front_mut() {
            front.flags &= !flag::PATHED;
        }
        self.kill_current_order(u);
    }

    // ------------------------------------------------------------------
    // Build, repair, garrison — §5
    // ------------------------------------------------------------------

    /// `Object::adjacent_to`: `attack_dist < 0x60`, the footprint's extent
    /// for the building and `block_radius + 0x18` for the unit.
    pub fn adjacent_to(&self, u: usize, b: usize) -> bool {
        let bd = &self.buildings[b];
        let t_ext = match bd.ty {
            Some(ty) => (
                self.build_types[ty].x_size * HALF_TILE,
                self.build_types[ty].y_size * HALF_TILE,
            ),
            None => combat::extent(&bd.combat.unwrap_or_default(), true),
        };
        let a_ext = combat::extent(&self.profile(Obj::Unit(u)), false);
        combat::attack_dist(self.units[u].pos, bd.pos, a_ext, t_ext, false) < ADJACENT
    }

    /// `WallData::covers_tile`: the tile is inside the footprint.
    pub fn covers_tile(&self, b: usize, tile: Pos) -> bool {
        let bd = &self.buildings[b];
        let Some(ty) = bd.ty else {
            return bd.pos.tile() == tile;
        };
        let corner = self.tile_corner(ty, bd.pos);
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        tile.x >= corner.x && tile.x < corner.x + xs && tile.y >= corner.y && tile.y < corner.y + ys
    }

    /// `UnitType::find_nearby_spot(x, y, min, max, step, angle, …)`
    /// (`docs/ORDERS.md` §10): rings of radius `min, min + step, …, max`
    /// around the centre, each swept over 31 bearings in the order `0, ±1,
    /// ±2, … ±7` sixteenths of a turn from `angle`, then the odd
    /// thirty-seconds (`17/32` twice, `1/32` never, the direct opposite
    /// never); each projected point snapped to its quarter-tile centre; the
    /// first candidate on the map, not on a blocked tile, not on
    /// `footprint_of`'s footprint (a farm excepted for a citizen) and of the
    /// unit's terrain class wins. `max <= 0` and `step <= 0` take the
    /// defaults. No RNG. The collision half — other units' positions and
    /// ordered positions — is not modelled: on open ground with nothing in
    /// the way the first candidate is free, which is the stated assumption.
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
    ) -> Option<Pos> {
        let p = self.profile(Obj::Unit(u));
        let mut max = max;
        if (min > 0 && max == 0) || max < 0 {
            max = if p.big_radius == 0 {
                min + 0x240
            } else {
                min + 4 * p.big_radius
            };
        }
        if max < min {
            max = min;
        }
        let step = if step <= 0 {
            ((max - min) / 8).max(1)
        } else {
            step
        };
        let farm_ok = footprint_of.is_some_and(|b| self.building_ident(b) == Ident::Farm)
            && self.worker_of(u) == Worker::Citizen;
        let air = matches!(p.domain, crate::attrition::Domain::Air);
        let mut r = min;
        loop {
            let ks: &[i32] = if r == 0 { &[0] } else { &BEARINGS };
            for &k in ks {
                let cand = if r == 0 {
                    centre
                } else {
                    let half = if !(-7..=7).contains(&k) {
                        0x800_0000
                    } else {
                        0
                    };
                    let a = Angle(
                        angle
                            .0
                            .wrapping_add(k.wrapping_mul(0x1000_0000))
                            .wrapping_add(half),
                    );
                    Pos::new(
                        centre.x + movement::sin_component(a, r),
                        centre.y - movement::cos_component(a, r),
                    )
                };
                let c = Pos::new(
                    cand.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
                    cand.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
                );
                if !self.world.tile_in_bounds(c.tile()) {
                    continue;
                }
                if !air && self.world.tile_mask(c.tile()) & crate::world::tile::BLOCKED != 0 {
                    continue;
                }
                if let Some(b) = footprint_of
                    && !farm_ok
                    && self.covers_tile(b, c.tile())
                {
                    continue;
                }
                if !air && !self.world.accepts(c) {
                    continue;
                }
                return Some(c);
            }
            if max <= r {
                return None;
            }
            r = (r + step).min(max);
        }
    }

    /// The swarm ring (§5.4): `min(x_size, y_size) × 0x60 + 0x30`, halved
    /// for a farm under a build, nudged `0x30` outward.
    fn swarm_spot(&self, u: usize, b: usize, building: bool) -> Option<Pos> {
        let bd = &self.buildings[b];
        let (xs, ys) = bd.ty.map_or((1, 1), |t| {
            (self.build_types[t].x_size, self.build_types[t].y_size)
        });
        let mut r = xs.min(ys) * HALF_TILE + SNAP;
        if building && self.building_ident(b) == Ident::Farm {
            r /= 2;
        }
        let here = self.units[u].pos;
        let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
        let spot = self.find_nearby_spot(u, bd.pos, r, 0, -1, angle, Some(b))?;
        if !building {
            return Some(spot);
        }
        // The `+0x30` nudge away from the site on each axis, re-validated
        // as a one-candidate ring.
        let nudge = |v: i32, c: i32| {
            if v > c {
                v + SNAP
            } else if v < c {
                v - SNAP
            } else {
                v
            }
        };
        let nudged = Pos::new(nudge(spot.x, bd.pos.x), nudge(spot.y, bd.pos.y));
        Some(
            self.find_nearby_spot(u, nudged, 0, 0, 0, angle, Some(b))
                .unwrap_or(spot),
        )
    }

    /// `Group::action_swarm_around` for one unit: the approach move in
    /// front, then the order re-queued with the same action bit.
    fn swarm_around(&mut self, u: usize, b: usize, body: Body, action: bool) {
        let building = matches!(body, Body::Build(_));
        if let Some(spot) = self.swarm_spot(u, b, building) {
            self.add_move_order(u, spot, MoveKind::ExploreTo, QueuePos::First, false);
        }
        let order = Order {
            flags: if action { flag::ACTION } else { 0 },
            body,
        };
        // The build order goes behind the approach move and ahead of
        // everything that was queued.
        let at = usize::from(self.current_order(u).is_some_and(Order::is_transit));
        self.units[u].orders.insert(at, order);
        self.update_action(u);
    }

    /// `Unit::do_build` (§5.2).
    fn do_build(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Build(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let who = self.units[u].owner;
        let action = flags & flag::ACTION != 0;
        if !self.buildings.get(b).is_some_and(|bd| bd.alive) {
            self.kill_current_order(u);
            if self.order_type(u) != index::NONE {
                return;
            }
            self.build_done(u, Some(b));
            return;
        }
        if self.buildings[b].active {
            self.kill_current_order(u);
            if self.order_type(u) != index::NONE {
                self.check_build_order(u);
                return;
            }
            if self.building_ident(b) != Ident::OilPlatform && self.units[u].stance > 1 {
                self.build_done(u, Some(b));
                return;
            }
            if self.buildings[b].owner != who
                || !self.is_gather_type(b)
                || self.building_ident(b) == Ident::University
            {
                self.build_done(u, Some(b));
                return;
            }
            self.add_gather_order(u, b, QueuePos::New, false);
            return;
        }
        let inside =
            self.covers_tile(b, self.units[u].pos.tile()) && self.building_ident(b) != Ident::Farm;
        if !self.adjacent_to(u, b) || inside {
            self.kill_current_order(u);
            self.swarm_around(u, b, Body::Build(b), action);
            return;
        }
        let bpos = self.buildings[b].pos;
        let here = self.units[u].pos;
        self.units[u]
            .movement
            .set_facing(find_angle(bpos.x - here.x, bpos.y - here.y));
        let amount = build::builder_amount(
            &self.tuning,
            self.buildings[b].is_under_attack(),
            self.nation[who as usize].koreans,
        );
        if !self.do_construct(b, amount) {
            return;
        }
        let _ = frame;
        self.kill_current_order(u);
        let rest = self.order_type(u);
        if rest != index::NONE {
            self.check_build_order(u);
            return;
        }
        let gather = (self.building_ident(b) == Ident::OilPlatform || self.units[u].stance <= 1)
            && self.buildings[b].owner == who
            && self.is_gather_type(b)
            && self.building_ident(b) != Ident::University;
        if gather {
            self.add_gather_order(u, b, QueuePos::New, false);
            return;
        }
        self.build_done(u, Some(b));
    }

    /// `Unit::check_build_order` (§5.3): pop the queued build orders, re-add
    /// the surviving sites least-crowded first, all with the action bit.
    fn check_build_order(&mut self, u: usize) {
        let mut sites = Vec::new();
        // A **cursor** walk, not a run of kills (the second reading's R3 C2 —
        // the first reading had this hedged as a stale decompile local; the
        // listing at `0x6036fd` is a plain `current_node = current_node->prev`
        // with no call between the flag test and the advance). Three
        // consequences the earlier code got wrong:
        //
        //  * an intervening transit move is **stepped over, not removed**;
        //  * only a `MOVE_TO` is stepped over — the loop's own gate is
        //    `get_type() == BUILD_AT || get_type() == MOVE_TO`, so a swarm's
        //    `EXPLORE_TO` ends the scan;
        //  * the site the scan **stops** on keeps its order (R3 C5, a finding
        //    neither reading made): the `break` jumps out before the
        //    `repath; kill`, so that site ends the pass with a duplicate,
        //    which the next `do_build` disposes of through its
        //    "already finished" branch.
        //
        // After a kill the original re-tails the list and resumes from the
        // front, which is what resetting the cursor to 0 reproduces.
        let mut i = 0usize;
        while let Some(o) = self.units[u].orders.get(i).copied() {
            match o.body {
                Body::Build(b) => {
                    let live = self
                        .buildings
                        .get(b)
                        .is_some_and(|bd| bd.alive && !bd.active);
                    if live && self.building_is_city(b) {
                        sites.push(b);
                        break;
                    }
                    self.repath(u);
                    self.remove_order_at(u, i);
                    if live {
                        sites.push(b);
                    }
                    i = 0;
                }
                Body::Move(m) if m.kind == MoveKind::MoveTo && !o.has(flag::ACTION) => {
                    if i + 1 >= self.units[u].orders.len() {
                        break;
                    }
                    i += 1;
                }
                _ => break,
            }
        }
        if sites.is_empty() {
            return;
        }
        if sites.len() > 1 {
            let who = self.units[u].owner;
            // The count skips the unit doing the counting — `check_build_
            // order@00603470:132` guards the whole body on
            // `this_00->o != this->o`. (`find_build_spot`'s otherwise
            // identical count does *not*; see the audit's R3 F2.)
            let me = self.units[u].index;
            let crowd = |sim: &Sim, b: usize| {
                sim.units
                    .iter()
                    .filter(|x| x.owner == who && x.alive() && x.index != me)
                    .filter(|x| {
                        x.orders
                            .iter()
                            .find(|o| !o.is_transit())
                            .is_some_and(|o| matches!(o.body, Body::Build(s) if s == b))
                    })
                    .count()
            };
            let least = (0..sites.len())
                .min_by_key(|&i| crowd(self, sites[i]))
                .expect("non-empty");
            sites.swap(0, least);
        }
        for &b in sites.iter().rev() {
            self.add_build_order(u, b, QueuePos::First, true);
        }
    }

    /// `Unit::build_done` (§5.5): what a builder does next.
    fn build_done(&mut self, u: usize, site: Option<usize>) {
        if self.units[u].orders.len() > 1 {
            return;
        }
        let stance = self.units[u].stance;
        if stance <= 1 && self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE) {
            return;
        }
        let who = self.units[u].owner;
        if let Some(b) = site
            && self
                .buildings
                .get(b)
                .is_some_and(|bd| bd.alive && bd.owner == who)
            && self.is_gather_type(b)
            && !matches!(
                self.building_ident(b),
                Ident::University | Ident::OilPlatform
            )
            && self.gather_room(b)
        {
            self.add_gather_order(u, b, QueuePos::New, false);
        }
    }

    /// `Unit::do_repair` (§5.6), on the existing repair step.
    fn do_repair(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Repair(b),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let who = self.units[u].owner;
        let action = flags & flag::ACTION != 0;
        let gather_after = |sim: &mut Sim| {
            if sim.order_type(u) == index::NONE
                && sim.buildings[b].owner == who
                && sim.units[u].stance < 2
                && sim.is_gather_type(b)
                && sim.building_ident(b) != Ident::University
            {
                sim.add_gather_order(u, b, QueuePos::New, false);
            }
        };
        let bd = &self.buildings[b];
        if bd.damage == 0 {
            self.kill_current_order(u);
            gather_after(self);
            return;
        }
        if bd.owner != who && !self.is_ally(who, bd.owner) {
            self.kill_current_order(u);
            gather_after(self);
            return;
        }
        if !bd.active || (bd.is_under_attack() && !action) {
            self.kill_current_order(u);
            return;
        }
        let bpos = bd.pos;
        if matches!(self.world.owner_at(bpos), crate::Owner::Player(p) if self.is_enemy(who, p)) {
            self.kill_current_order(u);
            return;
        }
        if !self.adjacent_to(u, b) {
            self.kill_current_order(u);
            self.swarm_around(u, b, Body::Repair(b), action);
            return;
        }
        self.repair_step(u, b, frame);
    }

    /// `Unit::do_garrison`'s order side (§5.7): the gates are the garrison
    /// mechanic's; not adjacent → a move **in front**, the order stays.
    fn do_garrison_order(&mut self, u: usize) {
        let Some(Order {
            body:
                Body::Garrison {
                    building: b,
                    search,
                },
            // The redirected order's action bit is a constant 0, so the
            // original order's flags are not read here (R3 G6).
            flags: _,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        match self.garrison(u, b) {
            Ok(()) => self.kill_garrison_order(u),
            Err(GarrisonRefused::NotAdjacent) => {
                let bd = &self.buildings[b];
                let (xs, ys) = bd.ty.map_or((1, 1), |t| {
                    (self.build_types[t].x_size, self.build_types[t].y_size)
                });
                let r = xs.min(ys) * HALF_TILE + SNAP;
                let here = self.units[u].pos;
                let angle = find_angle(here.x - bd.pos.x, here.y - bd.pos.y);
                let bpos = bd.pos;
                match self.find_nearby_spot(u, bpos, r, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
            }
            Err(GarrisonRefused::Full | GarrisonRefused::NoCapacity) if search => {
                let who = self.units[u].owner;
                let city = self.buildings[b].city;
                let next = self.find_garrison_build(u, city, who);
                self.kill_current_order(u);
                if let Some(n) = next {
                    // The action bit of the redirected order is **not** the
                    // original's: `do_garrison@005e6b80:288` reads it through
                    // vtable slot `+0x2c` on the order's `UnitOrder`
                    // sub-object, and `GarrisonOrder`'s entry there
                    // (`0xb48ee8+0x2c` → `0x41bff0`) is the folded
                    // `xor eax,eax; ret`. So it is always 0.
                    // (`docs/audit/2026-08-21-orders.md` R3 G6.)
                    self.add_garrison_order(u, n, true, QueuePos::First, false);
                }
            }
            Err(_) => self.kill_current_order(u),
        }
    }

    /// `Unit::kill_garrison_order`: every GARRISON action down the squad is
    /// killed; here the unit's own.
    fn kill_garrison_order(&mut self, u: usize) {
        while let Some(a) = self.action_of(u) {
            if !matches!(self.units[u].orders[a].body, Body::Garrison { .. }) {
                break;
            }
            self.repath(u);
            self.kill_current_order(u);
        }
    }

    /// `Unit::find_garrison_build`: the city's nearest active building with
    /// room that the unit may enter.
    fn find_garrison_build(&self, u: usize, city: Option<usize>, who: Player) -> Option<usize> {
        let city = city?;
        let here = self.units[u].pos;
        let uty = self.units[u].ty?;
        self.buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.alive && b.active && b.owner == who && b.city == Some(city))
            .filter(|(i, b)| {
                b.ty.is_some_and(|t| self.can_garrison(uty, t))
                    && self.num_inside(*i) < self.garrison_limit(*i)
            })
            .min_by_key(|(_, b)| vector_dist(b.pos.x - here.x, b.pos.y - here.y))
            .map(|(i, _)| i)
    }

    // ------------------------------------------------------------------
    // The gather chain — §6.1
    // ------------------------------------------------------------------

    /// `BuildType` vslot `0x90`: `build_flags & 0x40`.
    pub fn is_gather_type(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.build_types[t].has(bflags::GATHER))
    }

    /// `build_flags & FLAT`.
    pub fn is_flat(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| self.build_types[t].has(bflags::FLAT))
    }

    /// The unit type's worker kind.
    pub fn worker_of(&self, u: usize) -> Worker {
        self.units[u]
            .ty
            .map_or(Worker::None, |t| self.unit_types[t].worker)
    }

    /// `BuildData::gather_max` against the chain: room for one more.
    fn gather_room(&self, b: usize) -> bool {
        self.buildings[b]
            .gather_max
            .is_none_or(|max| self.num_gatherers(b, false, false) < max)
    }

    /// `Build::add_gatherer` (§6.1): same owner, room, a worker on the map,
    /// not already in the chain → prune, then push-front.
    pub fn add_gatherer(&mut self, b: usize, u: usize) -> bool {
        let unit = &self.units[u];
        if self.buildings[b].owner != unit.owner
            || !self.gather_room(b)
            || !unit.alive()
            || !unit.on_map
            || self.worker_of(u) == Worker::None
            || self.buildings[b].gatherers.contains(&u)
        {
            return false;
        }
        self.check_gatherers(b);
        self.buildings[b].gatherers.insert(0, u);
        true
    }

    /// `Build::remove_gatherer`.
    pub fn remove_gatherer(&mut self, b: usize, u: usize) {
        self.buildings[b].gatherers.retain(|&x| x != u);
    }

    /// `Build::check_gatherers`: prune the dead, the re-tasked, the off-map.
    pub fn check_gatherers(&mut self, b: usize) {
        let keep: Vec<usize> = self.buildings[b]
            .gatherers
            .iter()
            .copied()
            .filter(|&u| {
                self.units[u].alive() && self.units[u].on_map && self.is_gathering_at(u, b, false)
            })
            .collect();
        self.buildings[b].gatherers = keep;
    }

    /// `BuildData::is_gathered_by`.
    pub fn is_gathered_by(&self, b: usize, u: usize) -> bool {
        self.buildings[b].gatherers.contains(&u)
    }

    /// `UnitData::is_gathering_at(o, who, strict)`: a worker whose first
    /// order is a GATHER on this building — and, strictly, arrived. A scholar
    /// inside the building counts as gathering there.
    pub fn is_gathering_at(&self, u: usize, b: usize, strict: bool) -> bool {
        let unit = &self.units[u];
        if !unit.on_map {
            return unit.inside == Some(b);
        }
        if self.worker_of(u) == Worker::None {
            return false;
        }
        match unit.orders.front().map(|o| o.body) {
            Some(Body::Gather(g)) if g.building == b => !strict || g.been_there,
            _ => false,
        }
    }

    /// `BuildData::num_gatherers(arrived, skip_decoys)`: the garrisoned
    /// gatherers of a university or platform, plus the chain members that
    /// are gathering here. `calc_gather` calls it `(1, 1)`.
    pub fn num_gatherers(&self, b: usize, arrived: bool, skip_decoys: bool) -> i32 {
        let ident = self.building_ident(b);
        let mut n = 0;
        if matches!(ident, Ident::University | Ident::OilPlatform) {
            let want = if ident == Ident::University {
                Worker::Scholar
            } else {
                Worker::Citizen
            };
            n += self.buildings[b]
                .garrison
                .iter()
                .filter(|&&u| self.worker_of(u) == want)
                .count() as i32;
        }
        n += self.buildings[b]
            .gatherers
            .iter()
            .filter(|&&u| self.is_gathering_at(u, b, arrived))
            .filter(|&&u| !(skip_decoys && self.units[u].decoy))
            .count() as i32;
        n
    }

    /// `Build::all_gathering`: every chain member is out at its tile.
    fn all_gathering(&self, b: usize) -> bool {
        self.buildings[b].gatherers.iter().all(|&u| {
            matches!(self.units[u].orders.front().map(|o| o.body),
                Some(Body::Gather(g)) if !g.goto_build && g.wait >= 0)
        })
    }

    // ------------------------------------------------------------------
    // `do_gather` — §6.3
    // ------------------------------------------------------------------

    /// `Unit::do_gather`: the walk, the arrival, the stand.
    fn do_gather(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Gather(mut g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let b = g.building;
        let who = self.units[u].owner;
        let bd = &self.buildings[b];
        if bd.owner != who || !bd.alive || !bd.active {
            self.kill_current_order(u);
            return;
        }
        let ident = self.building_ident(b);
        let fail = |sim: &mut Sim| {
            sim.kill_current_order(u);
            sim.add_think_order(u);
            if sim.is_flat(b) && !sim.covers_tile(b, sim.units[u].pos.tile()) {
                let p = sim.buildings[b].pos;
                sim.add_move_order(u, p, MoveKind::MoveTo, QueuePos::First, false);
            }
        };
        if !g.been_there && !self.is_gathered_by(b, u) {
            // The retry for a unit refused at issue.
            if !self.gather_room(b) || !self.add_gatherer(b, u) {
                fail(self);
                return;
            }
        }
        if ident == Ident::Farm && self.buildings[b].city.is_none() {
            fail(self);
            return;
        }
        if matches!(ident, Ident::Mine | Ident::Woodcutter) {
            let phase = frame + i64::from(self.units[u].index) * 4;
            if phase & 127 == 0
                && self.buildings[b]
                    .gather_max
                    .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.add_think_order(u);
                return;
            }
            self.do_non_flat_gather(u, g);
            return;
        }
        let here = self.units[u].pos;
        let bpos = self.buildings[b].pos;
        if ident != Ident::Farm {
            if !self.adjacent_to(u, b) {
                let (xs, ys) = self.buildings[b].ty.map_or((1, 1), |t| {
                    (self.build_types[t].x_size, self.build_types[t].y_size)
                });
                let d = xs.min(ys) * HALF_TILE + SNAP;
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                let Some(spot) = self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) else {
                    self.kill_current_order(u);
                    self.units[u].cant_reach = true;
                    return;
                };
                if !self.is_gathered_by(b, u) && ident != Ident::OilPlatform {
                    g.been_there = false;
                    self.store_gather(u, g);
                    return;
                }
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                return;
            }
        } else if !self.covers_tile(b, here.tile()) {
            if self.buildings[b]
                .gather_max
                .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.units[u].cant_reach = true;
                return;
            }
            if !self.is_gathered_by(b, u) {
                g.been_there = false;
                self.store_gather(u, g);
                return;
            }
            self.add_move_order(u, bpos, MoveKind::MoveTo, QueuePos::First, false);
            return;
        }
        // Arrived.
        if !g.been_there {
            if self.buildings[b]
                .gather_max
                .is_some_and(|max| self.num_gatherers(b, false, false) > max)
            {
                self.kill_current_order(u);
                self.units[u].cant_reach = true;
                return;
            }
            g.been_there = true;
            self.store_gather(u, g);
            self.ledgers[who as usize].dirty = true;
            if self.worker_of(u) == Worker::Scholar || ident == Ident::OilPlatform {
                self.go_inside(u, b);
                self.check_gatherers(b);
                self.kill_current_order(u);
                return;
            }
            self.check_gatherers(b);
        }
        if ident != Ident::Farm {
            // The oil well: one worker a frame bumps the building's counter;
            // stand at `b.x + 0x120`.
            if !self.buildings[b].gather_bumped {
                self.buildings[b].recharging += 1;
                self.buildings[b].gather_bumped = true;
            }
            self.units[u].movement.set_facing(Angle(0x4000_0000));
            let stand = Pos::new(bpos.x + OILWELL_OFFSET, bpos.y);
            if self.world.accepts(stand) {
                self.units[u].pos = stand;
                self.units[u].movement.body.pos = stand;
            }
            return;
        }
        self.do_farm(u, b, g, frame);
    }

    fn store_gather(&mut self, u: usize, g: GatherOrder) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Gather(x) = &mut front.body
        {
            *x = g;
        }
    }

    /// `Unit::do_non_flat_gather` (§6.4): wood and ore, given the building's
    /// resource tiles. Without a tile list the worker stays at the camp with
    /// `been_there` set — an input.
    fn do_non_flat_gather(&mut self, u: usize, mut g: GatherOrder) {
        let b = g.building;
        let who = self.units[u].owner;
        let wood = self.building_ident(b) == Ident::Woodcutter;
        if g.been_there && !self.buildings[b].gather_bumped {
            self.buildings[b].recharging += 1;
            self.buildings[b].gather_bumped = true;
        }
        let bpos = self.buildings[b].pos;
        let here = self.units[u].pos;
        let (xs, ys) = self.buildings[b].ty.map_or((1, 1), |t| {
            (self.build_types[t].x_size, self.build_types[t].y_size)
        });
        let d = xs.min(ys) * HALF_TILE + SNAP;
        if !g.goto_build {
            if !g.been_there {
                g.been_there = true;
                self.ledgers[who as usize].dirty = true;
            }
            if g.wait < 0 {
                // Return to the camp.
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                match self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                    None => {
                        g.wait = 20;
                    }
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                        g.goto_build = true;
                        g.wait = 32;
                    }
                }
                self.store_gather(u, g);
                return;
            }
            let Some(t) = g.tile else {
                // A tile was never chosen: nothing to walk to.
                self.store_gather(u, g);
                return;
            };
            let centre = Pos::new(t.x * TILE + HALF_TILE, t.y * TILE + HALF_TILE);
            if vector_dist(centre.x - here.x, centre.y - here.y) < AT_TILE {
                g.wait -= 1;
                if g.wait == 0 {
                    g.wait = if self.all_gathering(b) {
                        -1
                    } else {
                        100 + self.rng.roll() % 50
                    };
                }
                self.units[u]
                    .movement
                    .set_facing(find_angle(centre.x - here.x, centre.y - here.y));
                self.store_gather(u, g);
                return;
            }
            let angle = find_angle(here.x - centre.x, here.y - centre.y);
            match self.find_nearby_spot(u, centre, TILE, 0x100, 2, angle, None) {
                Some(spot) if spot != here => {
                    if !self.is_gathered_by(b, u) {
                        g.been_there = false;
                        self.store_gather(u, g);
                        return;
                    }
                    self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                }
                _ => {
                    g.tile = None;
                    g.goto_build = true;
                    g.wait = -1;
                    if g.dist_mod > 0 {
                        g.dist_mod -= 1;
                    }
                }
            }
            self.store_gather(u, g);
            return;
        }
        // Heading to, or at, the camp.
        if g.wait >= 0 {
            if self.adjacent_to(u, b) {
                g.wait -= 1;
                if !g.been_there {
                    g.been_there = true;
                    self.ledgers[who as usize].dirty = true;
                }
                if g.wait >= 0 {
                    self.units[u]
                        .movement
                        .set_facing(find_angle(bpos.x - here.x, bpos.y - here.y));
                }
                self.store_gather(u, g);
                return;
            }
            if !self.is_gathered_by(b, u) {
                g.been_there = false;
                self.store_gather(u, g);
                return;
            }
            let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
            if let Some(spot) = self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                self.store_gather(u, g);
                return;
            }
            if !g.been_there && vector_dist(bpos.x - here.x, bpos.y - here.y) > 0x600 {
                match self.find_nearby_spot(u, bpos, 0x600, -1, 0, angle, None) {
                    Some(spot) => {
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false)
                    }
                    None => self.kill_current_order(u),
                }
                return;
            }
            g.wait = -1;
            self.store_gather(u, g);
            return;
        }
        // Choose a tile.
        if !g.been_there {
            g.been_there = true;
            self.ledgers[who as usize].dirty = true;
        }
        if g.tile.is_none() {
            let from = &self.buildings[b].gather_from;
            if from.is_empty() {
                // No tile list: an input. The worker keeps the camp.
                self.store_gather(u, g);
                return;
            }
            let btile = bpos.tile();
            let dm = g.dist_mod;
            let mut best = (i32::MAX, 0usize);
            for (i, t) in from.iter().enumerate() {
                let score = vector_dist((t.x - btile.x).abs(), (t.y - btile.y).abs()).max(3) * dm
                    + (i as i32 >> 2);
                if score < best.0 {
                    best = (score, i);
                }
            }
            let pick = self.buildings[b].gather_from.remove(best.1);
            self.buildings[b].gather_from.push(pick);
            g.tile = Some(pick);
        }
        g.wait = 400 + self.rng.roll() % 200;
        g.goto_build = false;
        if !wood {
            g.wait = 1_000_000;
        }
        self.store_gather(u, g);
    }

    /// The farm stand (§6.5): the tile under the farmer grows for
    /// [`FARM_GROWS`] frames, is reaped, and then the farmer re-targets a
    /// tile with two sync-stream draws.
    fn do_farm(&mut self, u: usize, b: usize, _g: GatherOrder, _frame: i64) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        let here = self.units[u].pos.tile();
        let (mut dx, mut dy) = (here.x - corner.x, here.y - corner.y);
        if dx < 0 || dy < 0 || dx >= xs || dy >= ys {
            dx = 1;
            dy = 1;
        }
        let idx = (dy * 4 + dx) as usize;
        let farm = &mut self.buildings[b].farm;
        let state = farm.state[idx];
        match state {
            // 0: sowing anim not yet on → sow and grow (as 1); the "new tile"
            // branch needs the animation byte, which is not modelled.
            0 | 1 => {
                farm.state[idx] = 1;
                farm.percent[idx] += 1;
                if farm.percent[idx] >= FARM_GROWS {
                    farm.state[idx] = 2;
                }
            }
            2 => {
                // Reap: snip, 2 → 3.
                farm.state[idx] = 3;
            }
            _ => {}
        }
    }

    /// `Unit::find_gather_spot` (§6.6): the nearest-best gather building of
    /// the owner's with room; the per-good rate term is an input (taken as 1).
    fn find_gather_spot(&mut self, u: usize, range: i32) -> bool {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let scholar = self.worker_of(u) == Worker::Scholar;
        let mut best: Option<(i32, usize)> = None;
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.owner != who || !self.is_gather_type(b) {
                continue;
            }
            if (self.building_ident(b) == Ident::University) != scholar {
                continue;
            }
            if !self.gather_room(b) && !self.is_gathered_by(b, u) {
                continue;
            }
            let dist = vector_dist(bd.pos.x - here.x, bd.pos.y - here.y);
            if range > 0 && dist > range {
                continue;
            }
            let score = 500 / (dist / TILE + 2);
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, b));
            }
        }
        let Some((_, b)) = best else {
            return false;
        };
        if !self.is_gathering_at(u, b, false) {
            self.add_gather_order(u, b, QueuePos::Last, false);
        }
        true
    }

    // ------------------------------------------------------------------
    // The attack order — §7.2
    // ------------------------------------------------------------------

    /// `Unit::do_attack` → `fight`'s order-management half, on the existing
    /// attack step: the target check, the reload gate with `new_ord`, the
    /// 1/5 re-search, range → strike, out of range → the chase inserted in
    /// front.
    fn do_attack(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Attack(mut a),
            flags,
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let me = Obj::Unit(u);
        let Some(target) = self.units[u].combat.target else {
            self.kill_current_order(u);
            return;
        };
        if self.attack_of(me) == 0 {
            self.kill_current_order(u);
            return;
        }
        let state = self.units[u].combat;
        if !self.valid_target(me, target) || flags & flag::RETARGET != 0 {
            // `find_new_target`: the idle search with the order dropped.
            self.kill_current_order(u);
            if let Some(t) = self.find_melee_target(u, -1) {
                self.add_attack_order(u, t, QueuePos::First, false, false);
            }
            return;
        }
        if state.stance == combat::Stance::HoldFire {
            return;
        }
        // The reload gate: a recharging unit returns at once unless this is
        // the first frame of a fresh order, which turns and then returns.
        if state.recharging != 0 {
            if a.new_ord {
                let (from, to) = (self.units[u].pos, self.pos_of(target));
                self.units[u]
                    .movement
                    .set_facing(find_angle(to.x - from.x, to.y - from.y));
            }
            return;
        }
        // The one-in-five re-search (`docs/COMBAT.md` §8.2 step 0).
        if !state.mandatory
            && state.captain == i32::from(self.units[u].index)
            && let Obj::Unit(t) = target
        {
            let roll = self.rng.roll();
            if !self.profile(Obj::Unit(t)).combat_role
                && roll % 5 != 0
                && let Some(f) = self.find_melee_target(u, -1)
                && f != target
            {
                self.retarget_attack(u, f);
                return;
            }
        }
        let in_range = self.is_in_range(me, target);
        a.in_range = in_range;
        if in_range {
            a.ever_in_range = true;
            a.new_ord = false;
            self.store_attack(u, a);
            self.fight_pub(u, target, frame);
            return;
        }
        self.store_attack(u, a);
        if state.stance == combat::Stance::StandGround {
            return;
        }
        // The chase: `find_attack_pos` is the target's position here; the
        // move goes in front and runs from the next frame.
        let dest = self.pos_of(target);
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
    }

    fn store_attack(&mut self, u: usize, a: AttackOrder) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Attack(x) = &mut front.body
        {
            *x = a;
        }
    }

    /// A found better target rewrites the order's target in place.
    fn retarget_attack(&mut self, u: usize, t: Obj) {
        self.bump_targeted_pub(t, 1);
        let unit = &mut self.units[u];
        unit.combat.target = Some(t);
        unit.combat.mandatory = false;
    }
}
