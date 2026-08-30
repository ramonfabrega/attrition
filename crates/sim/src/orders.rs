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
//! pathfinder is no longer a stub: [`Sim::find_path`] is the straight-line
//! verifier, and the three grid planners live in `path.rs`
//! (`docs/PATHFINDER.md`).

use crate::anim;
use crate::build::{self, Ident, flags as bflags};
use crate::combat::{self, Obj};
use crate::economy;
use crate::garrison::GarrisonRefused;
use crate::movement::{self, Angle, find_angle};
use crate::world::{tile, vector_dist};
use crate::{FarmAnim, Player, Pos, Sim, farms};

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

/// How far above the commerce cap `find_gather_spot` believes the Dutch
/// interest bonus (`has_tribe_bonus(0x16)`) can carry a good's income, in
/// sixteenths — `0x640` at `find_gather_spot@005f5170:191`.
///
/// `Leader::do_gather` spells the same headroom `dutch_interest_cap × 16`
/// out of `GameAccess::constants`; the search hard-codes it, so it is a
/// literal here too rather than a [`crate::tuning::Tuning`] slot. Nothing in
/// this simulation pays the bonus yet, so this term only ever moves the
/// *choice* of building, never the income.
const DUTCH_INTEREST_HEADROOM: i32 = 0x640;

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
    /// target at order time. A group move overrides it with the formation's
    /// own bearing plus the slot's packed byte (`docs/GROUPS.md` §6.6 step
    /// 6).
    pub angle: Angle,
    /// `MoveOrder +0x28 facing` — the mirror the **formation** this order
    /// belongs to was laid out with, or `None` for the −1 every plain move
    /// carries (`Unit::add_move_order@00616ed0` passes it literally).
    ///
    /// It is not read while the order runs. It is read when the order
    /// *dies*: `Unit::kill_current_order` writes it back onto the group as
    /// `GroupData::facing` when the unit is the group's leader, so the next
    /// formation starts from the mirror the last one used
    /// (`docs/GROUPS.md` §6.3).
    pub facing: Option<bool>,
    /// "I have a current waypoint."
    pub has_waypoint: bool,
    /// The current waypoint — what the step walks toward.
    pub waypoint: Pos,
    /// Where the last straight-line plan was made.
    pub last: Option<Pos>,
    pub pause: i32,
    pub timer: i32,
    /// `MoveOrder +0x3c/+0x40 coll_x/coll_y` — the point the last
    /// collision refused, which `resolve_unit_collision` sidesteps from
    /// (`docs/COLLISION.md` §4.3, §6 step 4). The dump prints the pair.
    pub coll: Option<Pos>,
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
    /// The same bit, as the pathfinder writes it: this waypoint boards a
    /// transport (`docs/PATHFINDER.md` §7). One bit, two producers.
    pub const TRANSPORT: u8 = 0x4;
    /// A `go_around_building` mid-detour point — the turn-in corner between
    /// the lane the unit is in and the one it is stepping into. It
    /// suppresses the collision test entirely (`docs/ORDERS.md` §4.1).
    pub const DETOUR: u8 = 0x8;
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

    /// A move order's destination — the `+0x4`/`+0x8` pair
    /// `Group::action_attack` reads off the current order (§10).
    pub const fn move_dest(&self) -> Option<Pos> {
        match self.body {
            Body::Move(m) => Some(m.dest),
            _ => None,
        }
    }

    pub const fn has(&self, f: u8) -> bool {
        self.flags & f != 0
    }

    /// A plain transit move: a move without the action bit — what
    /// `get_action` walks past.
    pub const fn is_transit(&self) -> bool {
        self.is_move() && !self.has(flag::ACTION)
    }

    pub(crate) const fn move_mut(&mut self) -> Option<&mut MoveOrder> {
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
/// The 48-unit snap, `div_3_table[v >> 4] * 0x30 + 0x18`.
///
/// Two callers in the original and they are the same arithmetic: every move
/// order's destination (`Unit::add_move_facing_order`, §4.3) and **every
/// unit's starting position** — `Unit::init@00612100:69` snaps both
/// coordinates before it hands them to `Object::init` and to
/// `set_new_location`, so an object is never born off the grid however
/// unrounded the point its maker computed. `div_3_table` is `floor(i / 3)`
/// and the coordinates are non-negative, so `div_euclid` is the same table.
pub(crate) const fn snapped(p: Pos) -> Pos {
    Pos::new(
        p.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
        p.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
    )
}

/// `find_path`'s "far and reachable: the pathfinder's job" rule, world cells.
const STRAIGHT_LINE_CELLS: i32 = 4;
/// The working-the-tile radius of a woodcutter or miner.
const AT_TILE: i32 = 0x140;
/// The oil-well stance offset.
const OILWELL_OFFSET: i32 = 0x120;
/// Adjacency: `attack_dist < 0x60`.
const ADJACENT: i32 = 0x60;
/// The farm's tile extent, `GameAccess::rnd`'s modulus on a re-target
/// (§6.5). The decompiler lost the register; the first reading guessed 3
/// from the geometry, and run13 measured **4**: all six farmers' twelve
/// draws on sim-frame 101 land on the dump's new tiles under `% 4` and
/// under no other modulus (`docs/SYNC.md` §4) — a 4 × 4 farm has sixteen
/// cells, and the farmer may be sent to any of them.
const FARM_SPAN: i32 = 4;

/// `unit_masks & 0x78000000` — the four carrying-walk bits
/// ([`crate::Unit::carry`]), named as `Guy::set_anim`'s walk arm tests
/// them and written only by `Unit::do_non_flat_gather` (§6.4).
pub(crate) const CARRY_WITH_WOOD: u32 = 0x0800_0000;
pub(crate) const CARRY_TO_WOOD: u32 = 0x1000_0000;
pub(crate) const CARRY_WITH_ORE: u32 = 0x2000_0000;
pub(crate) const CARRY_TO_ORE: u32 = 0x4000_0000;
/// The nibble `& 0x87ffffff` clears whole.
const CARRY_ANY: u32 = CARRY_WITH_WOOD | CARRY_TO_WOOD | CARRY_WITH_ORE | CARRY_TO_ORE;

/// The wood machine's two direct draw sites, under the original's own
/// offsets from `Unit::do_non_flat_gather@005f0170` — the tile-choice wait
/// (`% 200 + 400`) and the at-work wait (`% 50 + 100`), §6.4. [`Sim::mark`]
/// writes them into [`Sim::phase_marks`], which is what keeps them from
/// being read as the stand that precedes them (`docs/SYNC.md` §5).
pub const SITE_TILE_WAIT: &str = "Unit::do_non_flat_gather+0x54b";
pub const SITE_WORK_WAIT: &str = "Unit::do_non_flat_gather+0xcc3";

/// The farmer's cell re-pick — `Unit::do_gather@005ef2a0`'s two
/// `GameAccess::rnd(4)` calls, the pair that follows the cell-state switch
/// when the state and the animation disagree (§6.5).
///
/// Both draws are the **same** address: `GameAccess::rnd` is a frameless
/// helper whose `Random::get` call returns to `+0x20` whoever asked, so the
/// trace names the pair once and the `ebp` walk skips straight past both it
/// and `do_gather` to `Unit::do_job+0x67`. One mark therefore carries the
/// two draws (`docs/SYNC.md` §3.10).
pub const SITE_FARM_CELL: &str = "GameAccess::rnd+0x20 < Unit::do_job+0x67";

/// `Unit::do_move@005f7b30:599`'s grid draw — the call is at `005f89af`,
/// so the site is `+0xe84` (§4.4, `docs/SYNC.md` §6's frame-3 item). The
/// sim reaches it where the original does not, and naming it is what turns
/// that from an unattributed unit-loop draw into a row.
pub const SITE_MOVE_GRID: &str = "Unit::do_move+0xe84";

/// The 31 bearings of one ring of `find_nearby_spot`, as multiples of a
/// sixteenth of a turn from the base angle; `|k| >= 8` adds a thirty-second.
const BEARINGS: [i32; 31] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8, 9, -9, 10, -10, 11, -11, 12, -12,
    13, -13, 14, -14, 15, -15,
];

/// Which of `find_nearby_spot`'s two collision halves a call site asks for
/// (`docs/ORDERS.md` §10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coll {
    /// `FILTER_NOT_ME` / `FILTER_CAN_COLLIDE` with a real `(o, who)` and no
    /// squad — every build, repair, gather, garrison, idle and stable site:
    /// `Objects::find_collision` and then `find_ordered_collision`.
    Pairwise,
    /// Accept any passable candidate. The original's `nocoll != 0`, and —
    /// as a stated seam — its **general** path too: `FILTER_ALL` and a
    /// squad placement go through `ObjectsData::find_unit_with_radius`,
    /// whose `big_radius + r_coll` circle this crate does not model.
    None,
}

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

    /// `get_speed(x, y, 0)` — the virtual at slot `+0x17c` that both the
    /// unit step (`do_move@005f7b30`) and the pull-back
    /// (`find_path@005fb910`) call for the step length.
    ///
    /// For a player's unit this is `UnitData::get_speed@00608720`, whose
    /// three layers `docs/MOVEMENT.md`, "The speed pipeline", describes and
    /// which this crate takes as the cached `Movement::speed`.
    ///
    /// **An animal replaces it whole.** `AnimalData::get_speed@005d8380`
    /// occupies the same slot on `Animal`'s and `AnimalData`'s vtables, and
    /// it does not call `UnitData::get_speed` at all — it calls
    /// `UnitData::speed` (the aura layer) and then, on a land or sea
    /// animal:
    ///
    /// - if the current order `is_move` (slot `+0x14`) and the animal is
    ///   more than `0x180` from that order's **goal** — `get_move_order`
    ///   (slot `+0xb8`), then `MoveOrder +0x4/+0x8`, the ordered point and
    ///   not the current waypoint — the speed becomes `speed * 3 / 2`,
    ///   truncated toward zero;
    /// - and the result is floored at 3.
    ///
    /// An **air** animal — a bird, `type +0x218 == 2` — returns before
    /// both: no hurry and no floor.
    ///
    /// So none of `UnitData::get_speed`'s own layers reach an animal: not
    /// the order scale, not the `unit_masks & 0x10` halving, not the
    /// `0x800` tile, not the group cap. The two things `do_move` applies
    /// *after* the virtual — `ai_speed` and the modern-infantry `5/4` —
    /// still do.
    ///
    /// The `0x180` is the same threshold `Animal::do_idle` measures its
    /// wander with ([`crate::anim::WANDER_NEAR`] is `0x181`, the strict
    /// `<`), but it is a **different distance**: `do_idle` measures from
    /// the herd's centre, this from the order's goal. `docs/MOVEMENT.md`,
    /// "The animal's own `get_speed`".
    ///
    /// SEAM: the original decides this by class — a unit is an `Animal` or
    /// it is not — and this crate has no class, so it asks
    /// [`crate::Unit::is_gaia`], the same stand-in
    /// [`Sim::do_idle`](Self::do_idle) uses to reach `Animal::do_idle`.
    pub fn get_speed(&self, u: usize) -> i32 {
        let unit = &self.units[u];
        let speed = unit.movement.speed;
        if !unit.is_gaia() {
            return speed;
        }
        if unit.kind.domain == crate::attrition::Domain::Air {
            return speed;
        }
        let far = self
            .current_order(u)
            .and_then(Order::move_dest)
            .is_some_and(|to| vector_dist(unit.pos.x - to.x, unit.pos.y - to.y) > 0x180);
        let speed = if far { speed * 3 / 2 } else { speed };
        speed.max(3)
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
        unit.movement.des_angle = unit.movement.heading;
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
    pub(crate) fn clear_partial_path(&mut self, u: usize) {
        self.units[u].line_ok = false;
    }

    /// `Unit::kill_current_order(0)` (§3.2): the per-kind teardown, then the
    /// pop, the path segment, `update_action`.
    pub fn kill_current_order(&mut self, u: usize) {
        let Some(order) = self.units[u].orders.front().copied() else {
            return;
        };
        // Before the teardown: a dying **move** carries the mirror its
        // formation was laid out with, and the group's leader hands it back
        // (`docs/GROUPS.md` §6.3). This is the write that makes a group's
        // second right-click start from the first click's mirror instead of
        // from whatever the leader's turning left behind.
        if let Body::Move(m) = order.body
            && let Some(f) = m.facing
        {
            self.hand_back_facing(u, f, m.angle);
        }
        match order.body {
            Body::Gather(g) => {
                let who = self.units[u].owner;
                self.ledgers[who as usize].dirty = true;
                if self.buildings.get(g.building).is_some_and(|b| b.alive)
                    && self.buildings[g.building].owner == who
                {
                    self.remove_gatherer(g.building, u);
                }
                // `kill_current_order:107`, inside the same arm and after
                // the chain: the carrying walk goes with the job.
                self.units[u].carry &= !CARRY_ANY;
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
    ///
    /// **The angle is taken to the caller's point, not to the snapped one**
    /// (§4.3). The listing at `00616ed0` computes the two `find_angle`
    /// arguments in registers from the *arguments* — `ecx = x − (this->
    /// field_0x10 ^ 0x63637)`, `edx = y − (field_0x14 ^ 0x63637)` — and only
    /// then indexes `div_3_table` for the pair it pushes. So a caller that
    /// hands over an unsnapped point gets the bearing to that point and a
    /// destination 48-snapped away from it, and the two differ by up to
    /// twenty-four units on each axis. A farmer's re-picked cell is
    /// `t · 0xc0 + 0x60` — the centre of a 192-unit tile, which is never a
    /// 48-unit cell centre — so every farm walk in the game ends facing
    /// somewhere the snapped bearing does not name (`docs/ORDERS.md` §4.3,
    /// `docs/SYNC.md` §3.12).
    pub fn add_move_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
    ) {
        let here = self.units[u].pos;
        let angle = find_angle(to.x - here.x, to.y - here.y);
        self.add_move_facing_order(u, to, kind, pos, action, angle, None, false);
    }

    /// `Unit::add_move_facing_order@005e55c0` (§4.3), and
    /// `add_group_move_order@005e4710` where they agree: the same snap and
    /// the same record, with the **caller's** angle and the formation's own
    /// `facing` instead of the bearing `add_move_order` derives.
    ///
    /// `add_move_order` is that call with `angle = find_angle(dest − here)`
    /// and `facing = −1`, which is exactly what the listing at `616ed0`
    /// pushes. The two arguments only ever come from a group move
    /// (`docs/GROUPS.md` §6.6 step 6), which is why the ordinary adder does
    /// not take them.
    ///
    /// `pathed` is the original's `param_5`, and it writes [`flag::PATHED`]
    /// straight into the order at birth (`005e55c0`: `*pbVar1 |= 1`). Only
    /// the group move passes it: `action_move_near` plans the whole chain
    /// itself a few lines later (`docs/GROUPS.md` §6.7), so the order it
    /// hands out must not send `do_move` off to plan again.
    #[allow(clippy::too_many_arguments)] // the original's twelve, minus the seven this does not model
    pub fn add_move_facing_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
        angle: Angle,
        facing: Option<bool>,
        pathed: bool,
    ) {
        let dest = snapped(to);
        let order = Order {
            flags: if action { flag::ACTION } else { 0 } | if pathed { flag::PATHED } else { 0 },
            body: Body::Move(MoveOrder {
                kind,
                dest,
                angle,
                facing,
                has_waypoint: false,
                waypoint: dest,
                last: None,
                pause: 0,
                timer: 0,
                coll: None,
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
        // `Unit::work@0060d180:378`, before the dispatch: the collision
        // cooldown a failed 48-grid search bought counts down
        // (`docs/COLLISION.md` §4.1).
        if self.units[u].safe != 0 {
            self.units[u].safe -= 1;
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

    /// `Unit::do_idle`: `set_anim(CHAR_DEFAULT, 0, 1)`, `collide = 0`,
    /// `check_idle`, `think`. An animal's is `Animal::do_idle`, which
    /// replaces the whole of it (`anim.rs`).
    fn do_idle(&mut self, u: usize, frame: i64) {
        if self.units[u].is_gaia() {
            // A bird is never idle in the original: it carries the
            // `AirOrder` `Objects::process_all` gave it at birth, and
            // `do_job` runs `Unit::do_air_patrol` on it every frame —
            // first the `+0x180` virtual, `Animal::think_bird`, then
            // `Unit::do_air_physics`, which ends in `set_anim(CHAR_WALK,
            // 0, 1)` on every frame the bird is not a flock's
            // (`field_0xae`, only ever set for `FLOCKBIRD`). SEAM: this
            // crate has no `AirOrder`, so the bird stands here instead and
            // the *flight* is unmodelled — but the two things that reach
            // the stream, the think and that walk request, are both here
            // (`docs/SYNC.md` §3.9).
            if self.units[u].ty == self.bird_type() {
                self.think_bird(u, frame);
                self.set_anim(u, crate::anim::WALK, false, true);
                return;
            }
            self.animal_idle(u);
            return;
        }
        self.mark(crate::anim::SITE_IDLE_UNIT);
        self.set_default_anim(u);
        // `Unit::do_idle`'s own `collide = 0` (`docs/COLLISION.md` §6).
        self.units[u].collide = 0;
        self.check_idle(u, frame);
        self.think(u, frame);
    }

    /// The carrying walk a gatherer plays — `unit_masks & 0x78000000`, in
    /// `Guy::set_anim`'s own order (`005db61f`–`005db665`): `WALK_TO_WOOD`
    /// first, then `WALK_WITH_WOOD`, `WALK_TO_ORE`, `WALK_WITH_ORE`.
    ///
    /// **It is the mask, not the order's `goto_build`.** Reading the order
    /// made every walk of a gather a carrying walk, including the first
    /// walk to the camp — which `do_non_flat_gather` has never run for, so
    /// the original plays it as the plain `CHAR_WALK` and takes the
    /// arrival stand `Guy::move+0x19f` at the end of it. run33's `1/7`
    /// carries `unit_masks 262146` on the frame it arrives, and this
    /// crate had it on `WALK_WITH_WOOD` (`docs/ANIM.md` §4.4).
    pub(crate) fn gather_walk(&self, u: usize) -> Option<i8> {
        let m = self.units[u].carry;
        if m & CARRY_ANY == 0 {
            return None;
        }
        Some(if m & CARRY_TO_WOOD != 0 {
            crate::anim::WALK_TO_WOOD
        } else if m & CARRY_WITH_WOOD != 0 {
            crate::anim::WALK_WITH_WOOD
        } else if m & CARRY_TO_ORE != 0 {
            crate::anim::WALK_TO_ORE
        } else {
            crate::anim::WALK_WITH_ORE
        })
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
        // `think:82` — a **citizen** (`TypeIndex` 0x32 or 0x33) drops its
        // carrying walk here, and the leader takes `0x80000`. In the
        // original this sits after the auto-attack arm and before the
        // cadence gate; here the gate stands first, so a citizen that
        // takes an auto-attack keeps the nibble the original would have
        // cleared. SEAM: the two orderings differ only for an armed
        // citizen, and no capture has one.
        if matches!(self.units[u].type_index, 0x32 | 0x33) {
            self.units[u].carry &= !CARRY_ANY;
        }
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
        // `005f7195`: a worker whose `think_peasant` **found something**
        // ends the think there — the listing's `goto LAB_005f761a`, which
        // is the function's own exit. Ignoring the return value put a
        // citizen that had just been given a gather job through the tail
        // below on the same frame.
        if self.worker_of(u) != Worker::None && self.think_peasant(u, false) {
            return;
        }
        // The tail (`docs/SCOUT.md` §2): a scout or a spy not in an army
        // takes `think_scout`; a supply wagon or a hero takes
        // `add_to_army` (`docs/ARMY.md` §4). The two are exclusive in the
        // original and are kept so — and nothing else joins anything.
        if self.scout_thinks(u) {
            self.think_scout(u);
            return;
        }
        self.think_join_army(u);
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

        // Planning: the first time, or with an empty stack. Every fresh
        // move calls `find_wpath` exactly once; a refusal re-pushes the
        // goal and falls back to the tile grid (§4.4).
        if flags & flag::PATHED == 0 || self.units[u].path.is_empty() {
            let goal = PathData {
                to: mo.dest,
                tolerance: 0,
                flags: path_flag::FINAL,
            };
            self.units[u].path.push(goal);
            let r = self.find_wpath(u);
            if r < 1 {
                if self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
                let r2 = self.find_tpath(u);
                if r2 < 1 && self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
            }
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
            self.store_move(u, mo, flags);

            // **The waypoint's own collision test** (§4.4), the one call of
            // `detect_unit_collision` that is not `move_step`'s or
            // `resolve_unit_collision`'s. It runs once per leg — only on
            // the frame the waypoint is taken — and asks whether somebody
            // is already standing where this leg ends.
            //
            // A **final** waypoint under a `GATHER`, `ATTACK` or
            // `BUILD_AT` action is then not worth walking at all: the walk
            // dies here and the action picks somewhere else next frame.
            // That is how a farmer whose re-picked cell a sibling already
            // works stays where it is (`docs/COLLISION.md` §8).
            //
            // Otherwise a **parked** collider — one whose own current order
            // is not a move, so it is not going to get out of the way —
            // widens the tolerance to three of its `big_radius`
            // (`ObjectType +0x244`): give up short of it rather than walk
            // into it.
            //
            // SEAM: the original also spells `TRADE_ROUTE` in the kill's
            // action set and runs a region check just above (a
            // turn-in-place before a leg that ends in another terrain
            // region); neither is modelled — `docs/ORDERS.md` §4.4.
            if let Some(other) = self.detect_unit_collision(u, mo.waypoint) {
                let action = self.action_of(u).map(|a| self.units[u].orders[a].index());
                if top.flags & path_flag::FINAL != 0
                    && matches!(
                        action,
                        Some(index::GATHER | index::ATTACK | index::BUILD_AT)
                    )
                {
                    self.kill_current_order(u);
                    return Did::Nothing;
                }
                if !self.current_order(other).is_some_and(Order::is_move) {
                    let t = self.profile(Obj::Unit(other)).big_radius * 3;
                    if self.units[u].tolerance < t {
                        self.units[u].tolerance = t;
                        self.units[u].path.pop();
                        self.units[u].path.push(PathData {
                            tolerance: t,
                            ..top
                        });
                    }
                }
            }
            // `detect_unit_collision` writes `coll_x`/`coll_y` into the
            // order in place; take the copy back so the stores below keep
            // them.
            if let Some(m) = self.current_move(u) {
                mo = m;
            }

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
                // `false`: `do_move`'s own arrival (`005f8844`) tests
                // `orderlist.length == 1` and nothing else — the gather
                // clause belongs to `move_step`'s snap arm alone.
                self.arrive(u, mo, false);
                return Did::Nothing;
            }
            self.store_move(u, mo, flags);
        }

        // The speed, and the straight-line check. `do_move` takes it from
        // the `+0x17c` virtual, which an animal overrides
        // ([`Sim::get_speed`]).
        let speed = self.get_speed(u);
        if !self.units[u].line_ok {
            self.units[u].path_recursion = 0;
            let goal = mo.waypoint;
            let r = self.find_path(u, &mut mo, goal);
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
                // The straight line is not enough: the pathfinder's job
                // (§4.4). An unreachable goal with more orders queued kills
                // this one and the next.
                let here = self.units[u].pos;
                if self.invalid_loc(u, mo.dest.tile(), true, false, false, false, true) != 0
                    && self.units[u].orders.len() > 1
                {
                    self.kill_current_order(u);
                    self.kill_current_order(u);
                    return Did::Something;
                }
                // The draw that chooses the grid — one `Random::get` off
                // the sync stream on the first `do_move` of any move
                // `find_path` refuses.
                self.mark(SITE_MOVE_GRID);
                let n = self.rng.roll();
                let thr = match n % 5 {
                    2 => 2 * CELL,
                    0 => 8 * CELL,
                    _ => 5 * CELL,
                };
                let far = (mo.waypoint.x - here.x).abs() + (mo.waypoint.y - here.y).abs() > thr;
                let len_before = self.units[u].path.len();
                // `wflag` (which grid planned) only matters to the
                // resolve-block branch below, which is a seam; kept for the
                // shape.
                let (r, _wflag) = if far {
                    (self.find_wpath(u), true)
                } else {
                    // A non-final top equal to `last` is stale: pop it.
                    if let Some(t) = self.units[u].path.last().copied()
                        && t.flags & path_flag::FINAL == 0
                        && Some(t.to) == mo.last
                    {
                        self.units[u].path.pop();
                    }
                    // Not colliding: drop loose near waypoints, then plan
                    // on tiles. (The `collide != 0` arm of `do_move`'s own
                    // branch — a re-probe of `coll_x/coll_y` every other
                    // frame — is still unmodelled; the recovery path is
                    // `move_step`'s, `docs/COLLISION.md` §5.)
                    while let Some(t) = self.units[u].path.last().copied() {
                        if t.flags & (path_flag::FINAL | 0x20) == 0 && t.tolerance < 0x60 {
                            self.units[u].path.pop();
                        } else {
                            break;
                        }
                    }
                    (self.find_tpath(u), false)
                };
                // A positive return with the stack length unchanged counts
                // as a refusal.
                let r = if r > 0 && self.units[u].path.len() == len_before {
                    0
                } else {
                    r
                };
                if r == 0 && !self.units[u].path.is_empty() {
                    let top = self.units[u].path.last().copied().expect("non-empty");
                    if top.flags & path_flag::FINAL == 0 {
                        self.units[u].path.pop();
                        mo.has_waypoint = false;
                        self.store_move(u, mo, flags);
                        return Did::Something;
                    }
                    self.units[u].path.pop();
                    flags &= !flag::PATHED;
                    self.store_move(u, mo, flags);
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                } else if r == -1 {
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                }
                // TAKE: the new top is the target, and the straight line to
                // it is verified the same frame.
                let Some(top) = self.units[u].path.last().copied() else {
                    self.kill_current_order(u);
                    return Did::Something;
                };
                mo.last = None;
                mo.has_waypoint = true;
                mo.waypoint = top.to;
                self.units[u].tolerance = top.tolerance;
                self.units[u].path_recursion = 0;
                let goal = mo.waypoint;
                let r2 = self.find_path(u, &mut mo, goal);
                if r2 == 0 {
                    self.units[u].line_ok = true;
                }
                if !self.units[u].line_ok {
                    // The line to the new top failed too: with a world-grid
                    // plan, wait for the next frame; the tile-grid case
                    // falls into collision resolution (SEAM: `do_move`'s own
                    // arm, `docs/COLLISION.md` §9 — the step's is modelled).
                    self.store_move(u, mo, flags);
                    return Did::Something;
                }
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

    /// `Unit::find_path` (§4.6): 0 = walk straight, 1 = plan. The written
    /// `2` ("abort") is dead code in the original — nothing produces it at
    /// the base case — so it is not modelled.
    ///
    /// The march, per the settled third reading (`docs/PATHFINDER.md` §9):
    /// the angle is recomputed from the current remainder every iteration,
    /// each component is clamped to its axis' remainder, and the exits are
    /// `manh <= speed` (the loop condition), a grown remainder, and a
    /// remainder within one *actual clamped step* on both axes (`<=`). A
    /// no-progress step is unreachable with `speed >= 3`, so the interim
    /// guard of 2026-08-22 is retired; the soak that found the hang stands
    /// guard over this rewrite.
    fn find_path(&mut self, u: usize, mo: &mut MoveOrder, goal: Pos) -> u8 {
        let here = self.units[u].pos;
        let mut goal = goal;
        if goal == here {
            return 0;
        }
        let cells = (goal.cell().x - here.cell().x).abs() + (goal.cell().y - here.cell().y).abs();
        if cells > STRAIGHT_LINE_CELLS
            && self.invalid_loc(u, goal.tile(), true, true, false, false, false) == 0
        {
            return 1;
        }
        self.units[u].path_recursion = self.units[u].path_recursion.saturating_add(1);
        // `find_path`'s own `(*+0x17c)(x, y, 0)` and its `< 4 → 3` floor,
        // the same virtual the step takes ([`Sim::get_speed`]).
        let speed = self.get_speed(u).max(3);
        // THE PULL-BACK (§4.6, `005fbaa6`-`005fbb56`), and it is what item 28
        // was: a goal whose own tile refuses is walked *back toward us* one
        // step at a time until it does not, and the waypoint and the path's
        // top follow it wherever they are the goal. So a woodcutter sent at
        // a forest tile is really sent at the last open point short of it,
        // and the march that follows never enters the tile that would have
        // asked `go_around_building` for a detour. The sim had no pull-back,
        // marched into the forest, and paid `do_move`'s grid draw for a
        // detour the original never needed.
        //
        // The step is `sinx/cosx` of the bearing to the goal, taken once and
        // then only ever clamped down — never re-aimed, unlike the march's.
        let ang = find_angle(goal.x - here.x, goal.y - here.y);
        let mut back_x = movement::sin_component(ang, speed);
        let mut back_y = movement::cos_component(ang, speed);
        while self.invalid_loc(u, goal.tile(), false, false, false, false, false) != 0 {
            let (dx, dy) = (goal.x - here.x, goal.y - here.y);
            if dx.abs() < back_x.abs() {
                back_x = dx;
            }
            if dy.abs() < back_y.abs() {
                back_y = -dy;
            }
            if mo.waypoint == goal {
                mo.waypoint = Pos::new(mo.waypoint.x - back_x, mo.waypoint.y + back_y);
            }
            if let Some(top) = self.units[u].path.last_mut()
                && top.to == goal
            {
                top.to = Pos::new(top.to.x - back_x, top.to.y + back_y);
            }
            goal = Pos::new(goal.x - back_x, goal.y + back_y);
            // `sinx` and `cosx` of a bearing are never both zero at speed
            // >= 3, so this only fires once both have been clamped to a
            // remainder of zero — the goal has arrived at us.
            if back_x == 0 && back_y == 0 {
                break;
            }
        }
        // The goal walked all the way back onto us: there is nowhere to go,
        // and the cell is marked so the pathfinder does not offer it again.
        if goal == here {
            self.units[u].avoid = Some(goal);
            return 1;
        }
        let mut at = here;
        let (mut dx, mut dy) = (goal.x - at.x, goal.y - at.y);
        let mut manh = dx.abs() + dy.abs();
        let mut prev = manh;
        while speed < manh {
            if prev < manh {
                return 0;
            }
            prev = manh;
            let ang = find_angle(dx, dy);
            let mut sx = movement::sin_component(ang, speed);
            let mut cy = movement::cos_component(ang, speed);
            if dx.abs() < sx.abs() {
                sx = dx;
            }
            if dy.abs() < cy.abs() {
                cy = -dy;
            }
            let next = Pos::new(at.x + sx, at.y - cy);
            if next.tile() != at.tile()
                && self.invalid_loc(u, next.tile(), false, false, false, false, false) != 0
            {
                // Blocked. `go_around_building` walks the blocking tile's
                // edge and pushes a detour; a recursive `find_path` on its
                // first point is what decides whether the detour counts.
                // Ten deep, the original stops asking.
                if self.units[u].path_recursion < 10 {
                    let n0 = self.units[u].path.len();
                    self.go_around_building(u, mo, at, ang, speed);
                    if self.units[u].path_recursion != 10 && self.detour_verified(u, mo, goal, n0) {
                        return 0;
                    }
                }
                return 1;
            }
            at = next;
            dx = goal.x - at.x;
            dy = goal.y - at.y;
            if dx.abs() <= sx.abs() && dy.abs() <= cy.abs() {
                return 0;
            }
            manh = dx.abs() + dy.abs();
        }
        0
    }

    /// `UnitData::invalid_loc` with the five flags `find_path` and
    /// `go_around_building` both pass clear — the plain "may this unit
    /// stand on this tile" question.
    fn tile_refuses(&self, u: usize, t: Pos) -> bool {
        self.invalid_loc(u, t, false, false, false, false, false) != 0
    }

    /// `Unit::go_around_building@005fc350` (§4.6) — the answer to a
    /// straight line that clips a *building*, which the pull-back cannot
    /// help with: the goal's own tile is clear, so there is nothing to pull
    /// back, and the obstacle sits somewhere in the middle of the march.
    ///
    /// It is a **tile-edge walk**, and its shape is the same in both axes.
    /// The step that failed crossed one tile edge, so the unit has a *lane*
    /// (the row or column it is still standing in) and a *blocked lane*
    /// (the one it tried to enter). The walk runs perpendicular to the
    /// crossing, one tile at a time in both directions from the blocked
    /// point, and stops when either the blocked lane opens (success, at
    /// that offset) or the unit's own lane closes (success only if the
    /// blocked lane happens to be open there). Off the map is a failure for
    /// that direction; both directions failing gives up.
    ///
    /// Of the two candidates the closer to `mo.waypoint` wins, with
    /// forward as the tie-break, and either is discarded if it resolves to
    /// the unit's own tile centre. A candidate more than `0x300` from the
    /// blocked point in either axis gives up — the detour would be longer
    /// than the pathfinder's own leg.
    ///
    /// Then one to three `PathData`s go on the stack, bottom first:
    /// **the turn-in point** in the blocked lane and a **`DETOUR`
    /// midpoint** between the lanes, both only when the unit's own lane is
    /// closed at the found offset; and always **the target**, in the unit's
    /// own lane, one tile back along the walk when the turn-in pair was
    /// pushed. Every one of them is placed `off % 0xc0 / 2` off its tile's
    /// low corner rather than at the centre, from the order's own
    /// `off_x/off_y` — so a formation's units do not all aim at the same
    /// point.
    ///
    /// Giving up is `mo.dest = 0; masks &= ~8; path_recursion = 10`, and
    /// the recursion counter is how the caller tells the two apart.
    fn go_around_building(
        &mut self,
        u: usize,
        mo: &mut MoveOrder,
        last: Pos,
        ang: Angle,
        spd: i32,
    ) {
        let lt = last.tile();
        // The blocked point, recomputed here from an *unclamped* step: the
        // march clamps each component to its remainder, this does not, so
        // the two need not agree. That is the original's, not a slip.
        let sx = movement::sin_component(ang, spd);
        let sy = -movement::cos_component(ang, spd);
        let blocked = Pos::new(last.x + sx, last.y + sy);
        let bt = blocked.tile();

        // Which lane is which. `cmp` is the column the diagonal case keeps;
        // `settled` is the original's `goto LAB_005fc4c0`.
        let mut cmp = bt.x;
        let mut settled = lt.x == bt.x;
        if !settled && lt.y != bt.y {
            // A diagonal entry. More than two tiles at once is not a step
            // this can reason about.
            if (lt.y - bt.y).abs() + (lt.x - bt.x).abs() > 2 {
                return self.detour_gave_up(u, mo);
            }
            if self.tile_refuses(u, Pos::new(lt.x, bt.y)) {
                if self.tile_refuses(u, Pos::new(bt.x, lt.y)) {
                    // Both corners of the diagonal refuse: shorten the step
                    // and ask again, which may land orthogonally instead.
                    cmp = bt.x;
                    if spd > 1 {
                        return self.go_around_building(u, mo, last, ang, spd - 1);
                    }
                } else {
                    cmp = lt.x;
                }
                settled = true;
            }
        }
        let vertical = !settled || lt.y == bt.y || (lt.x != cmp && sx.abs() < sy.abs());
        let step = if vertical {
            Pos::new(0, if sy > 0 { 1 } else { -1 })
        } else {
            Pos::new(if sx > 0 { 1 } else { -1 }, 0)
        };
        let back = Pos::new(-step.x, -step.y);
        let (fwd_ok, fwd) = self.edge_walk(u, blocked, step, lt);
        let (bwd_ok, bwd) = self.edge_walk(u, blocked, back, lt);

        let here = self.units[u].pos;
        let centre = |t: i32| t * 0xc0 + 0x60;
        let at_unit = |p: Pos| {
            let t = p.tile();
            here.x == centre(t.x) && here.y == centre(t.y)
        };
        let (pick, pstep) = if !fwd_ok {
            if !bwd_ok {
                return self.detour_gave_up(u, mo);
            }
            (bwd, back)
        } else if !bwd_ok {
            (fwd, step)
        } else if at_unit(fwd) {
            (bwd, back)
        } else if at_unit(bwd) {
            (fwd, step)
        } else {
            let reach = |p: Pos| (p.x - mo.waypoint.x).abs() + (p.y - mo.waypoint.y).abs();
            if reach(bwd) < reach(fwd) {
                (bwd, back)
            } else {
                (fwd, step)
            }
        };

        if (pick.x - last.x).abs() >= 0x301 || (pick.y - last.y).abs() >= 0x301 {
            return self.detour_gave_up(u, mo);
        }

        // The waypoints. `off` is the order's own destination offset inside
        // its world cell (§4.1); the halved tile part of it is what places
        // every detour point off its tile's centre.
        let off = Pos::new(mo.dest.x.rem_euclid(0x300), mo.dest.y.rem_euclid(0x300));
        let skew = |v: i32| (v % 0xc0) / 2;
        let ct = pick.tile();
        let mut pushes: Vec<PathData> = Vec::new();
        let target = if pstep.x == 0 {
            let mut ty = ct.y;
            if self.tile_refuses(u, Pos::new(lt.x, ty)) {
                let (cx, cy) = (centre(ct.x), centre(ty));
                pushes.push(PathData {
                    to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                    tolerance: 0,
                    flags: 0,
                });
                ty -= pstep.y;
                let (lane_cx, back_cy) = (centre(lt.x), centre(ty));
                pushes.push(PathData {
                    to: Self::detour_mid(cx, cy, lane_cx, back_cy),
                    tolerance: 0,
                    flags: path_flag::DETOUR,
                });
            }
            Pos::new(lt.x, ty)
        } else {
            let mut tx = ct.x;
            if self.tile_refuses(u, Pos::new(tx, lt.y)) {
                let (cx, cy) = (centre(tx), centre(ct.y));
                pushes.push(PathData {
                    to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                    tolerance: 0,
                    flags: 0,
                });
                tx -= pstep.x;
                let (back_cx, lane_cy) = (centre(tx), centre(lt.y));
                pushes.push(PathData {
                    to: Self::detour_mid(cx, cy, back_cx, lane_cy),
                    tolerance: 0,
                    flags: path_flag::DETOUR,
                });
            }
            Pos::new(tx, lt.y)
        };
        pushes.push(PathData {
            to: Pos::new(
                skew(off.x) + target.x * 0xc0 + 0x30,
                skew(off.y) + target.y * 0xc0 + 0x30,
            ),
            tolerance: 0,
            flags: 0,
        });
        for p in pushes {
            self.units[u].path.push(p);
        }
    }

    /// The `DETOUR` midpoint between the two lanes: the mean of the two
    /// tile centres on each axis, biased one unit *down* whenever the
    /// turn-in centre is the larger, so the point falls inside the tile the
    /// unit is leaving rather than on the edge between them.
    fn detour_mid(cx: i32, cy: i32, other_x: i32, other_y: i32) -> Pos {
        let mut m = Pos::new((other_x + cx) / 2, (other_y + cy) / 2);
        if cx != other_x && cx - other_x >= 0 {
            m.x -= 1;
        }
        if cy != other_y && cy - other_y >= 0 {
            m.y -= 1;
        }
        m
    }

    /// One direction of `go_around_building`'s edge walk. Returns whether
    /// the blocked lane was found open, and the point the walk stopped at.
    fn edge_walk(&self, u: usize, from: Pos, step: Pos, lt: Pos) -> (bool, Pos) {
        let w = self.world.width() * crate::world::TILES_PER_CELL * 0xc0;
        let h = self.world.height() * crate::world::TILES_PER_CELL * 0xc0;
        let mut at = from;
        loop {
            let next = Pos::new(at.x + step.x * 0xc0, at.y + step.y * 0xc0);
            if next.x < 0 || next.y < 0 || next.x >= w || next.y >= h {
                return (false, at);
            }
            at = next;
            let cur = at.tile();
            // The unit's own lane keeps the coordinate the walk does not
            // move; the blocked lane is the tile the walk is standing on.
            let lane = if step.x == 0 {
                Pos::new(lt.x, cur.y)
            } else {
                Pos::new(cur.x, lt.y)
            };
            if self.tile_refuses(u, lane) {
                return (!self.tile_refuses(u, cur), at);
            }
            if !self.tile_refuses(u, cur) {
                return (true, at);
            }
        }
    }

    /// `go_around_building`'s refusal: the waypoint is dropped, the
    /// verified-line bit is cleared, and `path_recursion` is pinned at ten
    /// so that neither this call nor any caller asks again.
    fn detour_gave_up(&mut self, u: usize, mo: &mut MoveOrder) {
        mo.has_waypoint = false;
        self.units[u].line_ok = false;
        self.units[u].path_recursion = 10;
    }

    /// `find_path`'s acceptance of a detour (§4.6), and the half of it that
    /// is not `go_around_building`'s.
    ///
    /// The pushed waypoints are lifted off one at a time — any that lands
    /// exactly on the unit is dropped and the next one down takes its place
    /// as the candidate — and the candidate is then verified by a
    /// **recursive `find_path`**. Six degenerate cases refuse before the
    /// recursion: the candidate or the entry below it being the unit's own
    /// position or the position the last plan was made at, a unit already
    /// standing where it last planned, and a candidate equal to the goal we
    /// could not reach.
    ///
    /// On a verified line the detour goes back on the stack in its original
    /// order, each entry's "turn in place" bit recomputed against the one
    /// that will sit above it — set when exactly one of the two is on
    /// water, so that a leg crossing the shore turns before it walks.
    fn detour_verified(&mut self, u: usize, mo: &mut MoveOrder, goal: Pos, n0: usize) -> bool {
        let here = self.units[u].pos;
        let Some(top) = self.units[u].path.last().copied() else {
            return false;
        };
        let n1 = self.units[u].path.len();
        // `w` is the candidate to verify; `below` the entry under it.
        let mut w = top.to;
        let mut below = top.to;
        let mut saved: Vec<PathData> = Vec::new();
        let mut i = n0;
        while i < n1 {
            let Some(e) = self.units[u].path.pop() else {
                break;
            };
            below = e.to;
            if w == here && i + 1 < n1 {
                if let Some(p) = self.units[u].path.last().copied() {
                    w = p.to;
                    below = p.to;
                }
            } else {
                saved.push(e);
            }
            i += 1;
        }
        let planned_at = mo.last.unwrap_or(Pos::new(-1, -1));
        if w == here
            || w == planned_at
            || below == here
            || below == planned_at
            || here == planned_at
            || goal == w
        {
            return false;
        }
        // The `2` arm here is the original's dead code (§4.6): nothing
        // produces it at the base case, so a non-zero return is "plan".
        if self.find_path(u, mo, w) != 0 {
            return false;
        }
        if !self.units[u].line_ok {
            while let Some(e) = saved.pop() {
                if let Some(f) = self.units[u].path.pop() {
                    let f = self.shore_flagged(f, e.to);
                    self.units[u].path.push(f);
                }
                self.units[u].path.push(e);
            }
            mo.last = Some(here);
            if let Some(t) = self.units[u].path.pop() {
                let t = self.shore_flagged(t, here);
                self.units[u].path.push(t);
            }
            self.units[u].line_ok = true;
        }
        true
    }

    /// `TURN_FIRST` set on `e` exactly when it and `other` are on opposite
    /// sides of the shore — the `(mask & 0x30) == 0x20` test, water against
    /// everything else.
    fn shore_flagged(&self, mut e: PathData, other: Pos) -> PathData {
        let wet = |p: Pos| {
            self.world.tile_mask(p.tile()) & crate::world::tile::SURFACE
                == crate::world::tile::SURFACE_OCEAN
        };
        if wet(e.to) == wet(other) {
            e.flags &= !path_flag::TURN_FIRST;
        } else {
            e.flags |= path_flag::TURN_FIRST;
        }
        e
    }

    /// The unit step and its arrival (§4.5): `move_step` through
    /// `docs/MOVEMENT.md`, then the Manhattan test against the waypoint's
    /// tolerance, the pop, the facing on the final waypoint, the kill.
    fn unit_step(&mut self, u: usize, mut mo: MoveOrder, speed: i32) -> Did {
        // STEP (§4.4): `avoid_x/y = −1,−1` before every step.
        self.units[u].avoid = None;
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
        // `Unit::move_step`'s own `set_angle`, which is the one call of the
        // eighteen this simulation makes — and it is where a marching
        // leader's turn-around flips its group's mirror flag
        // (`docs/GROUPS.md` §4.1, §6.3). It is passed the **heading**, before
        // and regardless of the turn: `move_step` calls it at the top, on the
        // bearing `find_angle` just returned. The facing that the step is
        // actually taken along is guy 0's, and only `Guy::do_turn` moves it.
        self.unit_set_angle(u, step.heading);
        let unit = &mut self.units[u];
        unit.movement.facing = step.facing;

        // **The collision block** (`docs/COLLISION.md` §5). The proposed
        // point is tested against the occupancy index; a blocked step
        // either snaps through onto a sidestep waypoint, waits out the
        // turn it still owes, gives up and calls the waypoint reached, or
        // goes to `resolve_unit_collision`.
        let top = self.units[u].path.last().copied();
        let mut target = step.pos;
        let hit = self.detect_unit_collision(u, target);
        if let Some(other) = hit {
            let (dx, dy) = (mo.waypoint.x - from.x, mo.waypoint.y - from.y);
            let through = top.is_some_and(|t| t.flags & path_flag::SIDESTEP != 0)
                && !self.detect_quick(u, top.expect("tested").to)
                && dx.abs() < 0x61
                && dy.abs() < 0x61;
            if through {
                // The final snap through a collision: walk onto the
                // waypoint itself and let the arrival test take it.
                target = mo.waypoint;
            } else {
                // The blocked stand — `move_step:281`, the call at
                // `005fb74e` and so the site `+0x823`
                // ([`crate::anim::SITE_BLOCKED`], `docs/COLLISION.md` §5).
                // It sits **before** all three give-up tests, so a unit
                // still owed a turn has re-rolled its idle by the time
                // `move_step` returns.
                self.mark(crate::anim::SITE_BLOCKED);
                self.set_default_anim(u);
                if step.owed != 0 {
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    self.store_move(u, mo, flags);
                    return Did::Something;
                }
                let manh = dx.abs() + dy.abs();
                let reach = self.profile(Obj::Unit(u)).big_radius
                    + self.profile(Obj::Unit(other)).big_radius;
                let give_up = reach * 3 <= manh
                    || self.units[u].collide < 0x1a
                    || (top.is_none_or(|t| t.tolerance == 0 || t.flags & path_flag::SIDESTEP != 0)
                        && top.is_none_or(|t| t.flags & path_flag::FINAL == 0));
                if give_up {
                    self.resolve_unit_collision(u);
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    if self.current_order(u).is_some_and(Order::is_move) {
                        let mo = self.current_move(u).expect("a move order");
                        self.store_move(u, mo, flags);
                    }
                    return Did::Something;
                }
                // Give up on reaching it exactly: the waypoint is close
                // enough now.
                self.units[u].tolerance = manh * 2;
            }
        }

        let mut arrived = false;
        let mut snapped_in = false;
        if self.world.accepts(target) {
            self.set_new_location(u, target, false);
            // `Unit::set_new_location`'s half-cell test and the reveal
            // behind it (`docs/VISION.md` §6). `move_step` is the caller
            // that passes `param_3 = 0`, so this is the **ring** pass.
            self.moved_to(u, from, true);
            if step.arrived && target == step.pos {
                arrived = true;
                snapped_in = step.snapped;
            } else {
                let (dx, dy) = (mo.waypoint.x - target.x, mo.waypoint.y - target.y);
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
        self.arrive(u, mo, snapped_in);
        Did::Something
    }

    /// The final waypoint: `set_angle(mo->angle)` when the move is the only
    /// order — **or, on the Manhattan-snap arm only, when the action beneath
    /// is a gather**; clear the pathed bit; kill.
    ///
    /// The two arms of `move_step@005faf30` end with two different tests, and
    /// the difference is one clause: the snap's is `orderlist.length == 1 ||
    /// get_action()->get_type() == GATHER` (`005fb562`), the partial step's
    /// is `orderlist.length == 1` alone (`005fb4a8`). A farmer that lands on
    /// its cell by a partial step — its last leg still *longer* in Manhattan
    /// than one step, so the snap does not fire, while the sine and cosine
    /// components reach it exactly — keeps the bearing of that step and never
    /// faces the order's angle. run10's AI farmers do
    /// exactly that on frame 116 and the human's do not on 110
    /// (`docs/ORDERS.md` §4.5, `docs/SYNC.md` §3.12).
    fn arrive(&mut self, u: usize, mo: MoveOrder, snapped_in: bool) {
        let only = self.units[u].orders.len() == 1;
        let gather_beneath = snapped_in
            && self
                .action_of(u)
                .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Gather(_)));
        if only || gather_beneath {
            // `set_angle(mo->angle, …, 0)` — the heading, and guy 0's
            // `des_angle` with it. The facing does not snap: the body turns
            // toward the order's angle while it stands there.
            self.unit_set_angle(u, mo.angle);
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
    /// (`docs/ORDERS.md` §10) under the **pairwise** filter — the
    /// `FILTER_NOT_ME` every build, repair, gather, garrison and stable
    /// call site passes.
    ///
    /// Rings of radius `min, min + step, …, max` around the centre, each
    /// swept over 31 bearings in the order `0, ±1, ±2, … ±7` sixteenths of
    /// a turn from `angle`, then the odd thirty-seconds (`17/32` twice,
    /// `1/32` never, the direct opposite never); each projected point
    /// snapped to its quarter-tile centre; the first candidate on the map,
    /// not on a blocked tile, not on `footprint_of`'s footprint (a farm
    /// excepted for a citizen), of the unit's terrain class and **free of
    /// other units** wins. `max <= 0` and `step <= 0` take the defaults.
    /// No RNG.
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
        self.find_nearby_spot_coll(
            u,
            centre,
            min,
            max,
            step,
            angle,
            footprint_of,
            Coll::Pairwise,
        )
    }

    /// The same sweep with the collision half named. `Coll::None` is the
    /// original's `nocoll != 0` — and, as a stated seam, its whole
    /// general path (`docs/ORDERS.md` §10, §14).
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot_coll(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
        coll: Coll,
    ) -> Option<Pos> {
        let p = self.profile(Obj::Unit(u));
        let mut max = max;
        if (min > 0 && max == 0) || max < 0 {
            // `find_nearby_spot@0061de70:45`: the default is
            // `min + 4 × big_radius`, overridden to `min + 0x240` only when
            // `big_radius == 0` **and** `!(unit_flags & 0x10)` — the
            // transport-capable flag, which this crate does not model, so the
            // second condition is assumed false. It can only differ for a
            // transport-capable type whose `big_radius` is 0, and no shipped
            // row is both. Stated in §14. The squad branch
            // (`min + 0xc0 + 4 × (((uber−1) × guy_spacing)/2 + big_radius)`)
            // is not modelled either — nothing here places a squad.
            // (`docs/audit/2026-08-21-orders.md` R7 N3.)
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
                // The collision half, last of all: `Objects::find_collision`
                // then `Objects::find_ordered_collision`, both against
                // `(u)` as "me" (`docs/COLLISION.md` §5.2). A spot another
                // unit is standing on — or has already been sent to — is
                // taken.
                if coll == Coll::Pairwise
                    && (self.find_collision(u, c) || self.find_ordered_collision(u, c))
                {
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
    pub(crate) fn swarm_around(&mut self, u: usize, b: usize, body: Body, action: bool) {
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
            // §5.2 step 2, and the same predicate as step 6: only an
            // **oil platform** overrides it, and the exemption is
            // "not AI-driven *and* a gathering stance". An AI builder
            // that finds its site already finished never adopts it — it
            // goes back through `build_done`'s own arm.
            if self.building_ident(b) != Ident::OilPlatform
                && (self.ai_driven(who) || self.units[u].stance > 1)
            {
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
        // §5.2 step 4's first half, and it is a draw the arrival stand
        // would otherwise spend: the builder is put on its **work**
        // animation before it is turned, so `Guy::move`'s arrival test
        // (`cur_anim == CHAR_WALK`) never sees a walk again and the idle
        // roll at `Guy::set_anim+0x97a < Guy::move+0x19f` never fires
        // (`docs/ANIM.md` §4.6). A farm sows; everything else builds.
        let work = if self.building_ident(b) == Ident::Farm {
            anim::SOW
        } else {
            anim::BUILD
        };
        self.set_anim(u, work, false, true);
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
        // §5.2 step 6 — **the builder keeps the building it just finished
        // only if it is not the AI's.** `(OILPLATFORM or (not
        // `unit_masks & 0x40000` and stance ∈ {0,1}))`; an AI citizen goes
        // to `build_done`, whose own arm searches afresh and may well pick
        // a different building.
        let gather = (self.building_ident(b) == Ident::OilPlatform
            || (!self.ai_driven(who) && self.units[u].stance <= 1))
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
    ///
    /// **The AI's arm is its own and stops there.** `unit_masks & 0x40000`
    /// takes `find_build_spot` → `find_repair_spot` → `find_gather_spot`
    /// with **no stance gate** and, crucially, **without the tail** that
    /// hands a human builder the site it has just finished. The two object
    /// searches are the unmodelled seams; the gather search is not.
    fn build_done(&mut self, u: usize, site: Option<usize>) {
        if self.units[u].orders.len() > 1 {
            return;
        }
        let who = self.units[u].owner;
        if self.ai_driven(who) {
            // `find_build_spot`/`find_repair_spot` — seams (§5.5).
            if !self.lobby.resources_unlimited() {
                self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE);
            }
            return;
        }
        let stance = self.units[u].stance;
        if stance <= 1 && self.find_gather_spot(u, self.tuning.unit_gather_respond_range * TILE) {
            return;
        }
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
        // §5.6's first line, and unlike `do_build`'s it is **ahead of every
        // gate**: a repairer is put on `CHAR_REPAIR` even on the frame the
        // order dies. Same reason as `do_build`'s (`docs/ANIM.md` §4.6).
        self.set_anim(u, anim::REPAIR, false, true);
        // §5.6: an AI repairer takes `find_repair_spot` (a seam) instead of
        // adopting the building it has just mended — the same `unit_masks &
        // 0x40000` split as `do_build`'s two.
        let gather_after = |sim: &mut Sim| {
            if !sim.ai_driven(who)
                && sim.order_type(u) == index::NONE
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

    /// `UnitData::is_gathering_at(o, who, strict)`: a worker whose **action**
    /// is a GATHER on this building — and, strictly, arrived.
    ///
    /// The match is on [`Sim::action_of`], not the front of the list
    /// (`is_gathering_at@00608880`: `pUVar3 = get_action(this)`, then
    /// `get_type() == 7`). That is load-bearing, not pedantry: `do_gather`
    /// and `do_non_flat_gather` insert their walks as `QUEUE_FIRST` moves
    /// *without* the action bit, so on every walk-out and walk-back leg the
    /// front order is a move and only the action walk still finds the
    /// GATHER. Reading the front instead made a woodcutter stop counting as
    /// a gatherer the moment it set off — and be pruned out of its own chain
    /// by `check_gatherers`. (`docs/audit/2026-08-21-orders.md` R4 G16.)
    ///
    /// The inside-the-building arm is **scholar-only**: the original gates it
    /// on `ptype[4] ∈ {SCHOLARS, SCHOLARSKOREAN}` (G17), so a garrisoned
    /// citizen matches nothing. A platform's peasants are counted by
    /// `num_gatherers` through `count_inside` instead.
    pub fn is_gathering_at(&self, u: usize, b: usize, strict: bool) -> bool {
        let unit = &self.units[u];
        if !unit.on_map {
            return self.worker_of(u) == Worker::Scholar && unit.inside == Some(b);
        }
        if self.worker_of(u) == Worker::None {
            return false;
        }
        match self
            .action_of(u)
            .and_then(|i| unit.orders.get(i))
            .map(|o| o.body)
        {
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
            // The retry for a unit refused at issue. An oil platform, and any
            // non-land unit, skip the chain entirely and go straight on to
            // the arrival path — `do_gather@005ef2a0:98`
            // `if (build_type == 0x1a6 || ptype[0x218] != 0) goto ARRIVED`.
            // Registering them gave the platform chain entries the original
            // never creates. (`docs/audit/2026-08-21-orders.md` R4 G19.)
            let skips_chain = ident == Ident::OilPlatform
                || self.units[u].kind.domain != crate::attrition::Domain::Land;
            if !skips_chain && (!self.gather_room(b) || !self.add_gatherer(b, u)) {
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
                let from = self.units[u].pos;
                self.set_new_location(u, stand, true);
                // `do_gather`'s `set_new_location(…, 1, 1)`: the whole disc,
                // not the ring (`docs/VISION.md` §6).
                self.moved_to(u, from, false);
            }
            return;
        }
        self.do_farm(u, b, g, frame);
    }

    /// Writes a gather order's fields back — **to the gather order, not to
    /// the front of the list**.
    ///
    /// The original holds a pointer to the order object (`go`) for the whole
    /// of `do_gather` and `do_non_flat_gather`, and every walk those
    /// functions issue goes in *front* of it: `add_move_order(…, 1, …)`
    /// allocates a new order and links it at the head, leaving `go` valid.
    /// So a branch that issues a walk and then writes `go->goto_build` is
    /// writing the same object it was handed.
    ///
    /// Writing the front instead dropped every field of the branches that
    /// walk — most visibly the return to camp, which sets `goto_build = 1`
    /// and `wait = 32` *after* `add_move_order` (`docs/ORDERS.md` §6.4).
    /// Without them a woodcutter that had finished its shift never left the
    /// `goto_build == 0, wait < 0` arm: it walked to the camp, arrived, and
    /// re-issued the same walk and the same `CHAR_DEFAULT` stand for the
    /// rest of the game, so it never unloaded and never took another tile.
    fn store_gather(&mut self, u: usize, g: GatherOrder) {
        if let Some(x) = self.units[u]
            .orders
            .iter_mut()
            .find_map(|o| match &mut o.body {
                Body::Gather(x) => Some(x),
                _ => None,
            })
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
            // `do_non_flat_gather:96`, the first statement of this half:
            // the carrying walk is dropped whole, and one of the four is
            // put back by whichever walk this frame issues.
            self.units[u].carry &= !CARRY_ANY;
            if !g.been_there {
                g.been_there = true;
                self.ledgers[who as usize].dirty = true;
            }
            if g.wait < 0 {
                // Return to the camp.
                let angle = find_angle(here.x - bpos.x, here.y - bpos.y);
                match self.find_nearby_spot(u, bpos, d, -1, 0, angle, None) {
                    None => {
                        // No spot: keep chopping (`do_non_flat_gather:130`).
                        let work = if wood {
                            crate::anim::CHOP_WOOD
                        } else {
                            crate::anim::MINE_ORE
                        };
                        self.set_anim(u, work, false, true);
                        g.wait = 20;
                    }
                    Some(spot) => {
                        self.mark(crate::anim::SITE_STAND_RETURN);
                        self.set_default_anim(u);
                        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                        g.goto_build = true;
                        g.wait = 32;
                        // `:150` / `:154` — the walk back is a *carrying*
                        // one, by the camp's resource.
                        self.units[u].carry |= if wood {
                            CARRY_WITH_WOOD
                        } else {
                            CARRY_WITH_ORE
                        };
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
                // At the tile: the work animation (`do_non_flat_gather:267`
                // and `:271`), looping and silent.
                let work = if wood {
                    crate::anim::CHOP_WOOD
                } else {
                    crate::anim::MINE_ORE
                };
                self.set_anim(u, work, false, true);
                g.wait -= 1;
                if g.wait == 0 {
                    g.wait = if self.all_gathering(b) {
                        -1
                    } else {
                        self.mark(SITE_WORK_WAIT);
                        100 + self.rng.roll() % 50
                    };
                }
                self.units[u]
                    .movement
                    .set_facing(find_angle(centre.x - here.x, centre.y - here.y));
                self.store_gather(u, g);
                return;
            }
            // The stand before the approach — `do_non_flat_gather:275`,
            // the call at `005f113f` and so the site `+0xfd4`. §6.4's
            // pseudocode has always carried it and the implementation did
            // not; the whole-frame sequence check is what made the gap
            // countable (`docs/SYNC.md` §5.1).
            self.mark(crate::anim::SITE_STAND_TILE);
            self.set_default_anim(u);
            let angle = find_angle(here.x - centre.x, here.y - centre.y);
            match self.find_nearby_spot(u, centre, TILE, 0x100, 2, angle, None) {
                Some(spot) if spot != here => {
                    if !self.is_gathered_by(b, u) {
                        g.been_there = false;
                        self.store_gather(u, g);
                        return;
                    }
                    self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
                    // `:376` and `LAB_005f13da` — the approach is a walk
                    // *to* the resource.
                    self.units[u].carry |= if wood { CARRY_TO_WOOD } else { CARRY_TO_ORE };
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
                // At the camp, and the unload — `do_non_flat_gather:398`
                // through `:416`, in the original's own order: the wait is
                // decremented, `been_there` set, and a wait that has run
                // out returns *before* the facing and the animation. There
                // is **no idle stand here**: the branch is a two-way
                // `CHAR_DUMP_WOOD` / `CHAR_DUMP_ORE` on the camp's
                // resource, `been_there` is written and never read, and
                // the listing at `5f0b5e`–`5f0b89` shows the two `set_anim`
                // calls and no third.
                //
                // A first arrival used to stand idle here, which is where
                // the sim's two extra frame-0 draws came from: run20's
                // trace has no `Unit::do_non_flat_gather` draw at frame 0
                // at all, and the sequence comparison in `rondata::diff`
                // named this call as the divergence (`docs/SYNC.md` §4.2,
                // §6 — the stand half of the stand/wrap swap).
                g.wait -= 1;
                if !g.been_there {
                    g.been_there = true;
                    self.ledgers[who as usize].dirty = true;
                }
                if g.wait >= 0 {
                    self.units[u]
                        .movement
                        .set_facing(find_angle(bpos.x - here.x, bpos.y - here.y));
                    let dump = if wood {
                        crate::anim::DUMP_WOOD
                    } else {
                        crate::anim::DUMP_ORE
                    };
                    self.set_anim(u, dump, false, true);
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
        // Choose a tile. The stand first (`do_non_flat_gather:512`, whose
        // call is at `005f027a` and whose return address is therefore
        // `+0x10f` — the listing's three `CHAR_DEFAULT` stands are
        // `+0x10f`, `+0xb99` and `+0xfd4`, and the decompile's own
        // `pTVar19 = (TCoord *)0x5f027f` names this one) — the
        // woodcutters' frame-0 draws on run12 are this call (`docs/ANIM.md`
        // §4).
        if !g.been_there {
            g.been_there = true;
            self.ledgers[who as usize].dirty = true;
        }
        self.mark(crate::anim::SITE_STAND_GATHER);
        self.set_default_anim(u);
        // `tx < 0 || ty < 0 || !has_gather_access(tx, ty, who, 1, 0)` — the
        // held tile is given up and chosen again when the walk into it has
        // closed (`005f0655`, the `else` arm's `goto LAB_005f02b2`).
        let stale = match g.tile {
            None => true,
            Some(t) => !self.has_gather_access(t, who),
        };
        if stale {
            g.tile = None;
            let btile = bpos.tile();
            let dm = g.dist_mod;
            let mut best: Option<(i32, usize)> = None;
            for i in 0..self.buildings[b].gather_from.len() {
                let t = self.buildings[b].gather_from[i];
                // **The candidate is filtered before it is scored**
                // (`005f0575`): the tile still carries `mask & 0x4000`, and
                // `has_gather_access` still holds for it. A tile ringed by
                // its own kind has no orthogonal neighbour to stand on, so
                // it is never chosen however near the camp it is — which is
                // exactly what separates player 1's `(213, 92)` from the
                // nearer `(214, 93)` on run10 (`docs/ORDERS.md` §6.4).
                if self.world.tile_mask(t) & tile::BLOCKED == 0 || !self.has_gather_access(t, who) {
                    continue;
                }
                let score = vector_dist((t.x - btile.x).abs(), (t.y - btile.y).abs()).max(3) * dm
                    + (i as i32 >> 2);
                if best.is_none_or(|(s, _)| score < s) {
                    best = Some((score, i));
                }
            }
            // Nothing eligible — an empty list, or every tile walled in.
            // The original returns here (`005f05b0`), before the wait is
            // rolled, so the worker keeps the camp and takes no draw.
            let Some((_, at)) = best else {
                self.store_gather(u, g);
                return;
            };
            let pick = self.buildings[b].gather_from.remove(at);
            self.buildings[b].gather_from.push(pick);
            g.tile = Some(pick);
        }
        self.mark(SITE_TILE_WAIT);
        g.wait = 400 + self.rng.roll() % 200;
        g.goto_build = false;
        if !wood {
            g.wait = 1_000_000;
        }
        // `:694` / `:700`, the function's last statement either way.
        self.units[u].carry |= if wood { CARRY_TO_WOOD } else { CARRY_TO_ORE };
        self.store_gather(u, g);
    }

    /// The farm stand (§6.5), `Unit::do_gather@005ef2a0`'s wheat branch on
    /// the cell under the farmer, by the cell's state and the guy's
    /// animation (`'#'` is the sow anim — index 35, the one every farmer
    /// shows — and `'$'` the reap):
    ///
    /// ```text
    /// 0: not reaping → as 1;                    else → new tile
    /// 1: set_anim(SOW); Farms::grow
    /// 2: not sowing  → set_anim(REAP); snip;    else → new tile
    /// 3: set_anim(REAP)
    /// ```
    ///
    /// The clock is `farms.rs`'s: the farmer's `grow` and `inc_time`'s add
    /// each frame ripen the cell on frame 100's `grow` — the farmer is still
    /// sowing, so the next frame is the "new tile": two draws
    /// (`GameAccess::rnd(4)` for x, then y) and a move to that tile's
    /// centre, in front of the gather order. That is the original's
    /// re-target on the log's frame 102 — sim-frame 101, where run13 shows
    /// all six farmers' `orders_x/y` change and the walk start on 102
    /// (`docs/SYNC.md` §4).
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
        // **`dx * 4 + dy`, not `dy * 4 + dx`.** `FarmStruct::status` is a
        // `uchar[4][4]` and the farmer's cell is `status[dx][dy]`: both
        // `do_gather`'s own switch (`005eff54`, `(dx + farm·0x30)·4 + 0xac +
        // dy`) and `Farms::grow@008d91c0`'s write (`(farm·0x30 + param_3)·4
        // + 0xac + param_2`, called `(dy, dx)`) land on the same byte, and
        // it is the transpose of what this had. Invisible for a hundred
        // frames — every starting farmer stands on `(2, 2)`, which is its
        // own transpose — and then wrong the moment one re-picks a cell
        // (`docs/ORDERS.md` §6.5).
        let idx = (dx * 4 + dy) as usize;
        let anim = self.units[u].farm_anim;
        let state = self.buildings[b].farm.state[idx];
        match state {
            s if s == farms::GROWING || (s == farms::EMPTY && anim != FarmAnim::Reap) => {
                self.units[u].farm_anim = FarmAnim::Sow;
                self.set_anim(u, crate::anim::SOW, false, true);
                self.buildings[b].farm.grow(idx);
                return;
            }
            farms::RIPE if anim != FarmAnim::Sow => {
                self.units[u].farm_anim = FarmAnim::Reap;
                self.set_anim(u, crate::anim::REAP, false, true);
                self.buildings[b].farm.snip(idx);
                return;
            }
            farms::CUT => {
                self.units[u].farm_anim = FarmAnim::Reap;
                self.set_anim(u, crate::anim::REAP, false, true);
                return;
            }
            // Empty under a reaper, or ripe under a sower: a new tile.
            _ => {}
        }
        self.mark(SITE_FARM_CELL);
        let rx = self.rng.roll() % FARM_SPAN;
        let ry = self.rng.roll() % FARM_SPAN;
        let dest = Pos::new(
            (corner.x + rx) * TILE + HALF_TILE,
            (corner.y + ry) * TILE + HALF_TILE,
        );
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
    }

    /// `Unit::find_gather_spot` (§6.6): the best gather building of the
    /// owner's with room, scored on the **headroom under the commerce cap**
    /// of the goods it gathers, divided by the distance.
    pub(crate) fn find_gather_spot(&mut self, u: usize, range: i32) -> bool {
        let who = self.units[u].owner;
        let here = self.units[u].pos;
        let scholar = self.worker_of(u) == Worker::Scholar;
        // A scholar searches without a range limit: `find_gather_spot@
        // 005f5170:44` keeps the caller's range only `if (!bVar2)`, where
        // `bVar2` is the `SCHOLARS`/`SCHOLARSKOREAN` test — otherwise it
        // stays −1. (`docs/audit/2026-08-21-orders.md` R4 G42.)
        let range = if scholar { -1 } else { range };
        let region = self.world.tregion(here.tile());
        // `find_city_at(x, y, who, −1, 0)` on the citizen's own position: the
        // city it counts as standing in, and the input to the crossing rule.
        let my_city = self.find_city_at(who, here, None);
        // `local_20 = 0` and a strict `<`: the first maximum wins, and a
        // building whose score is not **above** zero is never taken.
        let mut best: Option<(i32, usize)> = None;
        let mut best_score = 0;
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.owner != who || !self.is_gather_type(b) {
                continue;
            }
            if (self.building_ident(b) == Ident::University) != scholar {
                continue;
            }
            if !self.gather_room(b) && !self.is_gathering_at(u, b, false) {
                continue;
            }
            if self.world.tregion(self.buildings[b].pos.tile()) != region {
                continue;
            }
            if !scholar
                && let Some(mine) = my_city
                && !self.crosses_to(who, mine, self.buildings[b].city)
            {
                continue;
            }
            let dist = vector_dist(
                self.buildings[b].pos.x - here.x,
                self.buildings[b].pos.y - here.y,
            );
            if range > 0 && dist > range {
                continue;
            }
            let score = self.gather_spot_value(who, b) * 500 / (dist / TILE + 2);
            if score > best_score {
                best_score = score;
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

    /// `find_gather_spot@005f5170:108` — whether a citizen standing in city
    /// `mine` will walk to a building belonging to city `theirs`.
    ///
    /// It always will inside its own city, and never when its own city holds
    /// fewer than two citizens (`CityData +0x5a free` plus `+0x5c gatherers`,
    /// the census counters). Otherwise it crosses only to a building of no
    /// city at all, or to a city that is more than two citizens *less*
    /// crowded than its own.
    fn crosses_to(&self, who: Player, mine: usize, theirs: Option<usize>) -> bool {
        if theirs == Some(mine) {
            return true;
        }
        // `city_ai` only exists as far as the last census reached, and a
        // human leader never runs one — so an absent record is zero, which
        // is what the original's `CityData` holds for the same reason.
        let pop = |c: usize| {
            self.ai[who as usize]
                .city_ai
                .get(c)
                .map_or(0, |r| r.free + r.gatherers)
        };
        let mypop = pop(mine);
        if mypop < 2 {
            return false;
        }
        match theirs {
            Some(c) => mypop > pop(c) + 2,
            None => true,
        }
    }

    /// The numerator of `find_gather_spot`'s score: over the six goods, the
    /// ones this leader has and this building gathers, each contributing the
    /// **unused** part of its commerce cap — `resource_cap[g] − income[g]`,
    /// both in sixteenths — and skipped entirely while the good is already
    /// over its cap. `has_tribe_bonus(0x16)` — the Dutch interest — adds
    /// [`DUTCH_INTEREST_HEADROOM`] to every good but knowledge, because
    /// `Leader::do_gather` lets that bonus carry income that far above the
    /// commerce cap.
    ///
    /// This is what sends a citizen to a farm rather than to a nearer
    /// woodcutter's camp once the camp's timber is closer to the ceiling.
    fn gather_spot_value(&self, who: Player, b: usize) -> i32 {
        let Some(good) = crate::ai_place::gather_good(self.building_ident(b)) else {
            return 0;
        };
        let w = who as usize;
        if !self.holdings[w].available[good] {
            return 0;
        }
        let ledger = &self.ledgers[w];
        if ledger.over_cap[good] != economy::OverCap::Under {
            return 0;
        }
        let mut v = ledger.cap[good] - ledger.income[good];
        if good != economy::Resource::Knowledge.index()
            && self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], 0x16)
        {
            v += DUTCH_INTEREST_HEADROOM;
        }
        v
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
