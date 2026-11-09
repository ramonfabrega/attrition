//! Groups — the order layer between an army and its units
//! (`docs/GROUPS.md`).
//!
//! Nothing in the original gives an order to a unit directly: a click, an
//! AI army's tick and a unit's own `go_to` all build a `Group` and call a
//! `Group::action_*` on it, and that walks the members and calls
//! `Unit::add_*_order` on each. This module is that layer — the five
//! actions `docs/ARMY.md` cites, the membership queries they lean on, and
//! `Groups::push_group`'s one live rule.
//!
//! The simulation collapses an army to a single group (`docs/ARMY.md`
//! §3.2) and has no player selection, so a [`Group`] here is a **value**:
//! the member list plus the two fields an action reads, built from an army
//! ([`Sim::army_group`]) or on the stack (§9's siege sub-group). The
//! persistent half of the record lives on the army as [`GroupState`].
//!
//! What is modelled and what stands in is `docs/GROUPS.md` §12; the
//! largest seam is `Form::compute`'s slot table (§6.4), so every member of
//! a group is given the **same** destination. That seam stands on **cost
//! alone**: the second reading (`docs/audit/2026-08-25-groups.md`) killed
//! both of the reasons the first gave for it — there is no float barrier,
//! and `GroupData::log_data` dumps the table's whole output every frame.
//!
//! Two substitutions worth naming here rather than only in the document:
//!
//! - §6.6 step 1 exempts exactly `TypeIndex` `0x32..=0x35` from the
//!   per-member `+0xaa` write. This module tests `role::CITIZEN`, and for
//!   types loaded from the install the two are the **same set**:
//!   `UnitType::determine_roles@0061c320` opens by setting `role = 0x200`
//!   for those four ids and nothing else ever sets that bit
//!   (`ai_load::RoleFacts::citizen_id`, by identity). For a hand-built test
//!   type they are whatever the test says.
//! - The simulation keeps an attack's target on the unit
//!   (`combat.target`/`combat.mandatory`) rather than on the order, so
//!   §10's `ox`/`whom`/`+0x1c` comparison is made against those.

use crate::ai_load::{uflags, uflags2};
use crate::attrition::Domain;
use crate::combat::{Obj, Stance};
use crate::movement::{Angle, find_angle};
use crate::orders::{Body, MoveKind, Order, QueuePos, flag, index};
use crate::world::{Pos, vector_dist};

/// `GroupData::get_loc`'s two gates, `70e128` and `70e1b7`. The compare is
/// `cmpl $0x180` followed by `jg`, so the bound is **inclusive** and the
/// decompiler's `< 0x181` names the same set.
const GROUP_LOC_NEAR: i32 = 0x180;
use crate::{Player, Sim};

/// The flight command's `orders` (`docs/ORDERS.md` §32): `MOVE_TO` (1),
/// home to one's own base, or `ATTACK` (10), at an enemy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flight {
    Home,
    Strike,
}

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `Form::compute`'s slot table | §6.4, where in the formation each member stands | every member takes the group's own destination; the group arrives as a heap. Diffable: `GROUPDATA` logs `off_x`/`off_y`/`curr_x`/`curr_y`/`angles`/`form_num` per member |
/// | the group pool | §3, 64 slots a leader and `get_open_slot`'s recycling | numbered since item 518 (§19) and reset by [`Sim::groups_process`]; `get_open_slot`'s fallbacks are not modelled (`equals_group`'s normalize is, §36.2) |
/// | `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `Move` — `docs/ORDERS.md` §8.4's verdict |
/// | `action_guard`'s building arm, human sweep and `QUEUE_FIRST` | §9's escort half, `docs/ORDERS.md` §24.7 | no building is ever guarded; the unit arm is built (item 567) |
/// | the order-time path plan | §6.7 | the sim plans on the first step, in `do_move`; with a zero slot offset the plan is the same one |
/// | `find_nearby_spot`'s collision, `invalid_loc` on a slot | §6.6 step 4's same-region arm | no slot is ever invalid in the same region, so no member is re-slotted there; the different-region arm is built (§37.1) |
/// | `QUEUE_FIRST`'s insert dance (`set_up_insert` / `action_halt` / recurse / `finish_insert`) | §6.2, §10 | `charge`'s `QUEUE_FIRST` is a plain `push_front` on each member |
/// | the scenario `ignore_orders` filter | §5 | never set outside a scenario |
/// | `is_modern_infantry`, the strafe | §6.6, §10 | no modern infantry, no planes (`is_packing` is built: [`Sim::clear_keeping_pack`], item 3336) |
/// | the area-id table (`world +0x134`) | §6.7, §9's anchor test | the sim compares tile regions instead |
pub mod seams {}

/// Formation 9 — **Mob**, the tenth and last of `rules.xml`'s formations
/// (§6.4). It is a real index, not a sentinel: `GroupData::form`'s "no
/// formation" value is **−1**, and run29 carries live groups at both. What
/// 9 does mean is "lay nobody out": `action_move_near` treats it as one of
/// the disqualifiers for a `GroupMoveOrder` (§6.6 step 6) and for a
/// follower's intermediate waypoints (§6.7).
pub const FORM_MOB: i32 = 9;

/// What a group move does to one member (§6.5, §6.6).
///
/// The original decides it twice — the `QUEUE_NEW` clear loop at `70524f`
/// and the order loop at `7054c7` ask overlapping questions of the same
/// member — and the two loops straddle `compute_form`. Deciding once, up
/// front, is what lets the clear run in its own pass ahead of the layout
/// without asking the second loop to read state the first has wiped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Member {
    /// Left entirely alone: not orderable, or a shooting siege unit under a
    /// move that is not hurrying.
    Skip,
    /// Stabled in this city instead of marching (§6.5).
    Stable(usize),
    /// Cleared, then given its slot.
    Move,
}

/// What [`Sim::group_set_up_insert`] copies aside and
/// [`Sim::group_finish_insert`] re-issues (§6.2, §17).
pub(crate) struct Insert {
    saved: Vec<Order>,
    aim: Option<(Obj, bool)>,
}

impl Insert {
    /// Nothing was copied: `action_form`'s `QUEUE_NEW` arm.
    pub(crate) fn is_empty(&self) -> bool {
        self.saved.is_empty()
    }
}

/// [`crate::finish_insert_view::FinishInsertView`] over an [`Insert`]: the
/// copies are typed [`Order`]s, not the original's raw records, so a record's
/// words are what this crate stores of the order and an action reads the
/// current order's body where the record names a slot or a pair this crate
/// holds as one index (`docs/GROUPS.md` §12).
///
/// SEAM: `finish_insert`'s other cases — gather, garrison, board, follow,
/// trade, spell, form, attack-ground — reach no action here. No capture
/// reaches a group `QUEUE_FIRST` carrying one.
struct SimInsert<'a> {
    sim: &'a mut Sim,
    g: &'a Group,
    saved: std::collections::VecDeque<Order>,
    cur: Option<Order>,
    aim: Option<(Obj, bool)>,
}

impl crate::finish_insert_view::FinishInsertView for SimInsert<'_> {
    fn reseat(&mut self) {}

    fn pending(&mut self) -> bool {
        !self.saved.is_empty()
    }

    fn take(&mut self) -> bool {
        self.cur = self.saved.pop_front();
        self.cur.is_some()
    }

    fn kind(&mut self) -> i32 {
        let Some(o) = self.cur else { return 0 };
        match o.body {
            Body::Move(m) => match (m.kind, m.group.is_some()) {
                (MoveKind::MoveTo, false) => 1,
                (MoveKind::AttackTo, false) => 2,
                (MoveKind::ExploreTo, _) => 3,
                (MoveKind::FleeTo, _) => 4,
                (MoveKind::MoveTo, true) => 0x13,
                (MoveKind::AttackTo, true) => 0x15,
            },
            Body::Build(_) => 6,
            Body::Gather(_) => 7,
            Body::Attack(_) => 0xa,
            Body::Follow(_) => 0xb,
            Body::Guard(_) => 0xc,
            Body::Repair(_) => 0xd,
            Body::Cast(_) => 0xe,
            Body::Trade(_) => 0xf,
            Body::Patrol(_) => 0x16,
            Body::AttackGround(_) => 0x17,
            Body::Garrison { .. } => 0x1a,
            Body::Strafe(_) | Body::AirPatrol(_) | Body::AirAttackGround(_) | Body::Think => 0,
        }
    }

    fn word(&mut self, _slot: u32, off: u32) -> i32 {
        match self.cur.map(|o| o.body) {
            Some(Body::Move(m)) => {
                let orig = m.group.map(|gm| gm.orig).or(m.orig);
                match off {
                    4 => m.dest.x,
                    8 => m.dest.y,
                    0xc => m.angle.0,
                    0x44 => orig.map_or(-1, |p| p.x),
                    0x48 => orig.map_or(-1, |p| p.y),
                    _ => 0,
                }
            }
            Some(Body::Build(b) | Body::Repair(b)) => {
                let bd = &self.sim.buildings[b];
                match off {
                    8 => i32::from(bd.owner),
                    0xc => i32::from(bd.index),
                    _ => 0,
                }
            }
            _ => 0,
        }
    }

    fn record(&mut self, _slot: u32) -> i32 {
        0
    }

    fn flags4(&mut self, _slot: u32) -> i32 {
        match self.cur {
            Some(o) if o.has(flag::ACTION) => 4,
            _ => 0,
        }
    }

    fn left_ecx(&mut self) -> i32 {
        0
    }

    fn give_obj(&mut self) {
        self.cur = None;
    }

    fn action_move_near(&mut self, a: [i32; 11]) {
        // `a[2]` (`+0x14` of a group move's record) has no field here.
        let kind = match a[6] {
            2 => MoveKind::AttackTo,
            3 => MoveKind::ExploreTo,
            4 => MoveKind::FleeTo,
            _ => MoveKind::MoveTo,
        };
        self.sim.group_action_move_to(
            self.g,
            Pos { x: a[0], y: a[1] },
            QueuePos::Last,
            a[4] != 0,
            Angle(a[5]),
            kind,
            a[7] != 0,
        );
    }

    // Cases 6 and `0xd`: the copy names a slot, not a building — the case
    // passes the order's `ox` and `whom` and no uid, and
    // `action_swarm_around` asks only that the slot's object be alive
    // (`docs/AI.md` §215); this crate holds the pair as the building index
    // and re-looks it up by its owner and number.
    fn action_swarm_around(&mut self, a: [i32; 5]) {
        let Some(Body::Build(b) | Body::Repair(b)) = self.cur.map(|o| o.body) else {
            return;
        };
        let bd = &self.sim.buildings[b];
        let Some(now) = self.sim.building_by_o(bd.owner, bd.index) else {
            return;
        };
        let body = if a[3] == crate::orders::index::REPAIR as i32 {
            Body::Repair(now)
        } else {
            Body::Build(now)
        };
        self.sim
            .group_action_swarm_around(self.g, now, QueuePos::Last, body, a[4] != 0);
    }

    fn action_gather(&mut self, _a: [i32; 2]) {}

    fn action_board_ship(&mut self, _a: [i32; 2]) {}

    fn action_attack(&mut self, _a: [i32; 5]) {
        // The target is the unit's, not the order's (`docs/GROUPS.md` §12).
        if let Some((t, mandatory)) = self.aim {
            self.sim
                .group_action_attack(self.g, t, mandatory, QueuePos::Last, 0);
        }
    }

    fn action_follow(&mut self, _a: [i32; 3]) {}

    fn action_guard(&mut self, _a: [i32; 4]) {
        if let Some(Body::Guard(gd)) = self.cur.map(|o| o.body) {
            self.sim
                .group_action_guard(self.g, gd.target, QueuePos::Last, false);
        }
    }

    fn action_spell(&mut self, _a: [i32; 5]) {}

    fn action_trade(&mut self, _a: [i32; 5]) {}

    fn action_form(&mut self, a: [i32; 5]) {
        // `a[3]` is the leftover `ecx` (parked 3372f): the listing never reads it.
        let mut v = crate::form_view::SimActionForm::new(&mut *self.sim, self.g);
        crate::form_view::action_form_steps(&mut v, a[0], a[1], a[2], a[4]);
    }

    fn redo_patrol_order(&mut self, _a: [i32; 2]) {
        if let Some(Body::Patrol(p)) = self.cur.map(|o| o.body) {
            self.sim.group_redo_patrol_order(self.g, p);
        }
    }

    fn action_attack_ground(&mut self, _a: [i32; 3]) {}

    fn action_garrison(&mut self, _a: [i32; 4]) {}
}

/// `reversing@0092cf20` — "is this angular difference a turn-around?"
///
/// Nine lines in the original and the engine's own name for the test three
/// separate mirror decisions share: `compute_form`'s toggle (§6.3),
/// `Unit::set_angle`'s (§4.1), and the inversion
/// `Unit::kill_current_order` applies to a dying order's own flag. The
/// compare is unsigned on the wrapped difference — `d < 0x40000000` is
/// false and `d < 0xc0000001` is true — so **both bounds are inclusive**:
/// exactly 90° reverses and so does exactly 270°.
pub fn reversing(d: Angle) -> bool {
    (0x4000_0000..=0xc000_0000).contains(&(d.0 as u32))
}

/// The persistent half of an army's one `GroupData` (`docs/GROUPS.md` §1).
///
/// Nothing in the simulation reads these — they are the record, kept so a
/// later `GROUPDATA` diff has something to compare — except [`Self::form`],
/// which `get_form` would read if the formation were modelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupState {
    /// `+0x10`: the formation index the last move applied, −1 none.
    pub form: i32,
    /// `+0x2c`: bumped by every action that issues orders.
    pub order_num: i32,
    /// `+0x18`, `+0x1c`: where the last move was ordered from.
    pub o: Pos,
    /// `+0x24`: the formation angle of the last move.
    pub o_angle: Angle,
    /// `+0x20`: `Form::compute`'s leftover distance.
    pub o_dist: i32,
    /// `+0x48`: the formation's mirror flag.
    pub facing: bool,
    /// `+0x14`: the frame membership last changed.
    pub stamp: i64,
    /// `+0x44`: how many slots `Form::compute` last laid out —
    /// `compute_dests` opens by writing the group's `num` here, so it is
    /// the membership *at the time of the last move* and `update_positions`
    /// walks this many, not `num`.
    pub form_num: i32,
    /// `+0x4c`, `+0x24c`: per member, its formation offset **quantised by
    /// the floor divide by 48** (`docs/GROUPS.md` §6.4).
    pub off: Vec<(i32, i32)>,
    /// `+0x44c`, `+0x64c`: `Group::update_positions`' rotation of them by
    /// the leader's heading — what `do_group_move` adds to the leader's
    /// position each frame.
    pub curr: Vec<Pos>,
    /// `+0x84c`: per member, the eighth-turn its slot faces off the
    /// formation's own bearing.
    pub angles: Vec<i8>,
    /// `+0x40`: **the speed cap `UnitData::get_speed` applies to a member
    /// with no action order** (`docs/GROUPS.md` §14). A one-pass-lagged
    /// minimum, not the leader's speed and not this pass's minimum: see
    /// [`Sim::group_leader_report_speed`] and [`Sim::group_report_speed`].
    /// Zero is "not set" and caps nothing.
    pub speed: i32,
    /// `+0x3c`: the accumulator the **next** pass's [`Self::speed`] comes
    /// from. The leader's step moves this into `speed` and restarts it at
    /// its own uncapped speed, and every follower that reports lower drives
    /// both down.
    pub new_speed: i32,
    /// `+0x4b`: cleared by the leader's own report every frame it steps, and
    /// set only by the forced-march arm below it
    /// ([`Sim::group_set_march`]). It gates whether a follower reports its
    /// speed at all.
    pub march: bool,
    /// **Which of its player's 64 pool slots this record is** —
    /// `GroupData::id − who·64`, the number every member's `UnitData +0x80`
    /// holds and the dump prints as `group` (`docs/GROUPS.md` §19).
    /// `None` for a seat that never took one. It is what
    /// [`Sim::groups_process`]'s cursor selects on, so an army and a
    /// pushed group are reset on the frames the original resets them.
    pub pool: Option<u8>,
    /// `+0x38`: the frame `Group::target_opportunity` last ran on this
    /// group, which it gates at fifteen frames. `Group::clear` zeroes it
    /// and `copy_group` is taken to leave the previous occupant's.
    pub opportunity: i64,
    /// `+0x8 army` written −1 by `Group::clear(-1)@00713e80`: the record
    /// no longer names its army, so `Army::remove_group@006f8b50` (which
    /// asks `group.army == army.id`) cannot take it off the army's list,
    /// and `Army::add_unit`'s join goes through it rather than pushing a
    /// fresh one ([`Sim::army_add_unit`], item 4000). `add_group` writes
    /// the link on every push, so a pushed or copied record starts linked.
    pub unlinked: bool,
}

impl GroupState {
    /// `Groups::copy_group@006fa690` of a fresh stack group into pool slot
    /// `pool`, whose record was `prev`: the stack group's own `(ox, oy)`,
    /// `o_dist`, `o_angle`, `speed` and offset arrays, `stamp = frame`, and
    /// everything else — `form`, `order_num`, `facing`, `form_num`,
    /// `new_speed`, `march` — the previous occupant's (§28, §30). The
    /// arrays are the stack group's `n` entries, each zeroed by its
    /// `Group::add` (`00714350:71–75`) and copied whole (`copy_group`'s
    /// `memcpy`s): run1555's pushes 66 and 67 on 642 print three zero
    /// slots each (item 4000).
    pub fn copied(prev: GroupState, pool: u8, frame: i64, n: usize) -> GroupState {
        GroupState {
            pool: Some(pool),
            stamp: frame,
            // The stack group's own: `Group::Group@007140c0`'s
            // `Group::clear(-1)@00713e80` zeroes `ox`/`oy` (`+0x18`,
            // `+0x1c`) and `Group::add` never writes them, so the copy is
            // (0, 0) — not the record default's (−1, −1). French East
            // Indies' come-out push on 10765 reads 0/0 on run630's 10766,
            // and the standing `group:73.ox/oy` on every French window
            // from 9772 (item 1457, `docs/GROUPS.md` §36.1).
            o: Pos::new(0, 0),
            o_dist: 0,
            o_angle: Angle(0),
            speed: 0,
            off: vec![(0, 0); n],
            curr: vec![Pos::new(0, 0); n],
            angles: vec![0; n],
            unlinked: false,
            ..prev
        }
    }

    /// `Group::clear(-1)@00713e80` on a pool record, for the fields this
    /// record carries: `form −1`, `order_num`, `facing`, `form_num`,
    /// `new_speed`, `speed`, `march`, `(ox, oy)`, `o_dist` and `o_angle`
    /// zeroed, and `stamp = game->frame`. The pool index stays. It is what
    /// `Group::kill@00714110` runs when its list empties (`docs/GROUPS.md`
    /// §30), so the next `copy_group` into the slot inherits zeros, not the
    /// last occupant's counts.
    pub fn clear(&mut self, frame: i64) {
        *self = GroupState {
            o: Pos::new(0, 0),
            stamp: frame,
            pool: self.pool,
            unlinked: true,
            ..GroupState::default()
        };
    }

    /// `Group::kill@00714110`'s tail on the member that empties a seated
    /// list: `Group::clear(-1)@00713e80` on the record itself. It sets
    /// `form` −1 and zeroes `ox`/`oy`, `o_dist`, `o_angle`, `order_num`,
    /// `think_frame`, both speeds, `form_num`, the march bit and
    /// **`facing`** (`+0x48`, a short with `buildings`), and stamps the
    /// frame. The slot arrays are `kill`'s own shift's, and the pool slot
    /// is the seat's, so both stand. Item 4000 (`docs/GOLDEN.md` §190):
    /// `Group::sort` kills a follower-first squad whole, so the mirror the
    /// move's layout reads a line later is 0 (run1555's slot 64 on 765:
    /// `form` −1, `facing` 0).
    pub fn emptied(&mut self, frame: i64) {
        self.form = -1;
        self.form_num = 0;
        self.o = Pos::new(0, 0);
        self.o_dist = 0;
        self.o_angle = Angle(0);
        self.order_num = 0;
        self.opportunity = 0;
        self.speed = 0;
        self.new_speed = 0;
        self.march = false;
        self.facing = false;
        self.unlinked = true;
        self.stamp = frame;
    }

    /// `Group::refresh_group_order@00713a50`'s middle third — **slide the
    /// whole table so that member `i` sits on the origin** (`docs/GROUPS.md`
    /// §6.8).
    ///
    /// This is the second thing that writes `off`, and until run31 nobody
    /// had looked for one: `compute_dests` puts the anchor at `(0, 0)`, and
    /// then, whenever the unit the group order names can no longer serve as
    /// the anchor, the member that notices re-origins the block onto
    /// **itself** and rewrites everyone's order. The subtraction is over the
    /// **quantised** offsets — the record's own numbers — and it runs over
    /// `form_num` entries, not `num`.
    pub fn reorigin(&mut self, i: usize) {
        let Some(&(dx, dy)) = self.off.get(i) else {
            return;
        };
        let n = (self.form_num.max(0) as usize).min(self.off.len());
        for slot in &mut self.off[..n] {
            slot.0 -= dx;
            slot.1 -= dy;
        }
    }
}

impl Default for GroupState {
    fn default() -> GroupState {
        GroupState {
            form: -1,
            // `Group::clear`'s own initialiser, and [`Sim::group_o`]
            // reads it as "this group has never moved" (§6.3): either
            // coordinate negative and `get_loc` hands back the leader's
            // position rather than the record's point.
            order_num: 0,
            o: Pos::new(-1, -1),
            o_angle: Angle(0),
            o_dist: 0,
            facing: false,
            stamp: 0,
            form_num: 0,
            off: Vec::new(),
            curr: Vec::new(),
            angles: Vec::new(),
            // `Group::clear@00713e80` zeroes both, and zero is what
            // `UnitData::get_speed`'s cap reads as "no cap".
            speed: 0,
            new_speed: 0,
            march: false,
            pool: None,
            opportunity: 0,
            unlinked: false,
        }
    }
}

/// An installed stack group — one slot of the original's `Groups` pool
/// (§3), which until item 465 this crate had exactly one of and kept no
/// record for.
///
/// The pool matters because a **pushed group outlives the army** its
/// members came from: Great Lakes' probe (`docs/ARMY.md` §12) pushes six
/// raiders out of army 1 on frame 8186 and the dump carries them on
/// `group 65` for the next two thousand frames, holding the
/// `GroupMoveOrder` that group issued. With the group gone the moment the
/// next `push_group` ran, `do_group_move` could not run on them at all
/// (`docs/ORDERS.md` §16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pushed {
    /// `GroupData::who`.
    pub who: Player,
    /// `GroupData::list`, in join order.
    pub list: Vec<usize>,
    /// The record half, the same one an army carries.
    pub state: GroupState,
    /// A **building group**'s members, as indices into [`Sim::buildings`]
    /// (`GroupData::buildings`, `+0x49`): the AI scripts' `train_unit*`
    /// and `research_tech_with_cost` push the producing building before
    /// `Group::action_queue_up` (§28.4). Empty for a group of units.
    pub builds: Vec<usize>,
}

/// Where a group's record lives: an army's slot, or an entry of
/// [`Sim::pushed`] — [`Sim::seat_of`]'s answer for a unit's `+0x80`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    /// The army of `who` in slot `.1`.
    Army(Player, usize),
    Pushed(usize),
}

/// One `Group` as an action sees it: the members, the owner, and which
/// seat holds its record — an army's (`GroupData::army`, the switch §6.5
/// turns on) or a pool slot's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub who: Player,
    /// `GroupData::army`, `None` for a group on the stack or a player's
    /// selection. The simulation only ever builds the first and the third.
    pub army: Option<usize>,
    /// The [`Pushed`] slot this group was installed in, if any. A group
    /// has at most one seat: `push_group` takes its members out of the
    /// army and out of any other slot, which is the original's
    /// `UnitData +0x80` pointing at one place.
    pub pushed: Option<usize>,
    /// `GroupData::list`, in join order.
    pub list: Vec<usize>,
}

impl Group {
    /// A group on the stack — `Group::clear(-1)` then `add` (§4.1).
    pub const fn stack(who: Player) -> Group {
        Group {
            who,
            army: None,
            pushed: None,
            list: Vec::new(),
        }
    }

    /// `GroupData::num`.
    pub fn num(&self) -> i32 {
        self.list.len() as i32
    }
}

/// `StanceTypes` — how many options a stance has (§8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StanceType {
    None,
    Combat,
    Worker,
    Caster,
    Packer,
}

impl StanceType {
    /// The option count the cycle wraps at.
    const fn options(self) -> i32 {
        match self {
            StanceType::Combat => 6,
            StanceType::Worker => 4,
            StanceType::Caster | StanceType::Packer => 2,
            StanceType::None => 0,
        }
    }
}

/// The six combat stances, in the option order `action_stance` writes
/// (`docs/COMBAT.md`).
const COMBAT_STANCES: [Stance; 6] = [
    Stance::Aggressive,
    Stance::Defensive,
    Stance::StandGround,
    Stance::Raid,
    Stance::Raze,
    Stance::HoldFire,
];

/// `add_group_move_order`'s `id` (`705f10`):
/// `(group.id + frame × 10) × 100 + group.order_num`, shared by every
/// member's order and the key `do_group_move` and `ungroup_move_order`
/// match on.
pub fn group_move_id(group_id: i32, frame: i64, order_num: i32) -> i64 {
    (i64::from(group_id) + frame * 10) * 100 + i64::from(order_num)
}

/// The combat stance an option byte `+0xb1` reads as —
/// `get_combat_stance`'s answer, `None` past the six options.
pub(crate) fn combat_stance_of(option: u8) -> Option<Stance> {
    COMBAT_STANCES.get(option as usize).copied()
}

/// The option index of a stance — `action_stance`'s `s`.
pub fn stance_option(s: Stance) -> i32 {
    COMBAT_STANCES
        .iter()
        .position(|&x| x == s)
        .expect("every Stance is in COMBAT_STANCES") as i32
}

/// `Groups::push_group@0070f9e0(who, g, force)`'s own steps (§3.2), the
/// callees answered by the caller: with `force == 0` a group of fewer than
/// two (`+0xc`, signed) is not installed — every member `in_use` is `seat`ed
/// at −1 and the answer is −1; else `slot` (`equals_group` against
/// `last_group[who]`, else `get_open_slot`, `copy_group` and the
/// `last_group` write) names the slot, and every member `in_use` whose
/// `+0x80` (`seat_of`) names another group (`>= 0`, `!=` the slot) is
/// `kill`ed from it (`(*old->vtbl+0x10)(o, who, 0, 0)`) before it is
/// `seat`ed at the slot. Answers the slot.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_group_steps<C: ?Sized>(
    cx: &mut C,
    n: i32,
    force: bool,
    mut in_use: impl FnMut(&mut C, usize) -> bool,
    mut slot: impl FnMut(&mut C) -> i32,
    mut seat_of: impl FnMut(&mut C, usize) -> i16,
    mut kill: impl FnMut(&mut C, i16, usize),
    mut seat: impl FnMut(&mut C, usize, i32),
) -> i32 {
    if !force && n < 2 {
        for k in 0..n.max(0) as usize {
            if in_use(cx, k) {
                seat(cx, k, -1);
            }
        }
        return -1;
    }
    let s = slot(cx);
    for k in 0..n.max(0) as usize {
        if in_use(cx, k) {
            let old = seat_of(cx, k);
            if old >= 0 && i32::from(old) != s {
                kill(cx, old, k);
            }
            seat(cx, k, s);
        }
    }
    s
}

impl Sim {
    // ------------------------------------------------------------------
    // Building a group (§3, §4)
    // ------------------------------------------------------------------

    /// The army's one group (`docs/ARMY.md` §3.2).
    pub fn army_group(&self, who: Player, slot: usize) -> Group {
        Group {
            who,
            army: Some(slot),
            pushed: None,
            list: self.armies[who as usize].list[slot].units.clone(),
        }
    }

    /// One installed [`Pushed`] slot as a [`Group`].
    pub(crate) fn pushed_group_at(&self, slot: usize) -> Option<Group> {
        let x = self.pushed.get(slot)?;
        Some(Group {
            who: x.who,
            army: None,
            pushed: Some(slot),
            list: x.list.clone(),
        })
    }

    /// The record half of a seated group — the army's, or the pool
    /// slot's. `None` is a group with no seat at all: a stack group that
    /// was never pushed, or a player's selection, which is the original's
    /// `UnitData::group == -1`.
    pub(crate) fn gstate(&self, g: &Group) -> Option<&GroupState> {
        match (g.army, g.pushed) {
            (Some(s), _) => Some(&self.armies[g.who as usize].list[s].group),
            (None, Some(i)) => self.pushed.get(i).map(|x| &x.state),
            (None, None) => None,
        }
    }

    /// [`Self::gstate`], to write.
    pub(crate) fn gstate_mut(&mut self, g: &Group) -> Option<&mut GroupState> {
        match (g.army, g.pushed) {
            (Some(s), _) => Some(&mut self.armies[g.who as usize].list[s].group),
            (None, Some(i)) => self.pushed.get_mut(i).map(|x| &mut x.state),
            (None, None) => None,
        }
    }

    /// `Group::add(o, who, 0, 0)` (§4.1) — the entry point every caller in
    /// the simulation uses. A non-captain is refused and **its captain is
    /// added instead**, so asking for a figure gets you its squad.
    pub fn group_add(&self, g: &mut Group, u: usize) {
        self.group_add_keeping(g, u, false);
    }

    /// `Group::add(o, who, keep_captain, const)` (§4.1) whole.
    ///
    /// `keep_captain` is the original's `param_3`, and it flips the walk:
    /// with 0 a **non-captain** is replaced by its captain, and with 1 a
    /// **captain** is first killed out of the group so that re-adding it
    /// moves it to the end. The one caller that passes 1 is this function
    /// itself, on the subordinate chain — which is why a group built from
    /// captains still holds every figure, in `captain, o_down, o_down's
    /// o_down` order. A player's selection is exactly that, and it is why
    /// run31's twelve squads are **36** members and not 12
    /// (`docs/GROUPS.md` §12.2).
    pub fn group_add_keeping(&self, g: &mut Group, u: usize, keep_captain: bool) {
        if !self.units[u].alive() {
            return;
        }
        if self.units[u].owner != g.who && !g.list.is_empty() {
            return;
        }
        if !keep_captain && !self.units[u].captain {
            // The tail call: `add(unit +0x8e, who, 0, const)`.
            if let Some(cap) = self.units[u].o_up {
                self.group_add_keeping(g, cap, false);
            }
            return;
        }
        if keep_captain && self.units[u].captain {
            // `kill(o, who, 0, 0)` first, so the re-add appends.
            g.list.retain(|&m| m != u);
        }
        if g.list.contains(&u) || g.list.len() >= 128 {
            return;
        }
        g.who = self.units[u].owner;
        g.list.push(u);
        // `+0x90 >= 0` and active: the next figure joins behind its captain,
        // and with `keep_captain = 1` so that it is not swapped back for one.
        if let Some(sub) = self.units[u].o_down
            && self.units[sub].alive()
        {
            self.group_add_keeping(g, sub, true);
        }
    }

    /// `Groups::push_group(who, g, force)` (§3.2), two rules of which are
    /// live without a pool: with `force == 0` a group of fewer than two is
    /// **not** installed and every member is left group-less; and an
    /// installed group **takes its members out of the group they were
    /// in** (§3.3). Returns whether the group took a slot.
    pub fn push_group(&mut self, g: &mut Group, force: bool) -> bool {
        // The original's own steps, [`push_group_steps`] (swept under the
        // emulator, `sweep::groups_push_group`): the gate's arm writes
        // `+0x80 = −1` on every active member and touches no list — the unit
        // stays wherever it was listed, naming nothing (§23); past it the
        // slot is chosen and every active member pointed at it.
        let n = g.num();
        let mut cx = (self, g);
        let s = push_group_steps(
            &mut cx,
            n,
            force,
            |(s, g), k| s.units[g.list[k]].alive(),
            |(s, g)| i32::from(s.push_group_slot(g)),
            |(s, g), k| s.units[g.list[k]].group_ptr.map_or(-1, i16::from),
            // The port asks each member's named group before the copy, in
            // `unseat_group` ([`Sim::kill_from_named`], its whole squad),
            // where the original asks it here, after; so this walk's kill
            // is already done (and on the equal arm, none is owed).
            |_, _, _| {},
            |(s, g), k, slot| {
                s.units[g.list[k]].group_ptr = u8::try_from(slot).ok();
            },
        );
        s >= 0
    }

    /// `push_group`'s slot (`70fa86`..`70fac6`): `equals_group` against the
    /// last pushed slot, else `get_open_slot` and `copy_group` into it —
    /// with the port's own bookkeeping of the record it lands in.
    fn push_group_slot(&mut self, g: &mut Group) -> u8 {
        let last = self.last_group[g.who as usize];
        // `Group::equals_group@00708000` opens by normalizing each side
        // whose `id` is not −1 — so the last pushed slot is pruned and its
        // speed reset to its leader's (`normalize`'s tail) on every push,
        // equal or not. French East Indies' go-to push on 10766 reads
        // slot 71's speed 28 on run630's 10767, 0 here before item 1457
        // (`docs/GROUPS.md` §36.2).
        if let Some(seat) = self.pool_seat(g.who, last) {
            self.seat_normalize(seat);
        }
        let pool = self.pool_slot_for(g.who, &g.list);
        // `70fa3b`: `equals_group` against the player's **last pushed**
        // slot — the same owner and the same members in the same order —
        // and an equal group is **not copied**: `copy_group` runs only on
        // the fresh slot `get_open_slot` hands back. So a selection pushed
        // again keeps its record, `(ox, oy)`, `o_angle`, `facing` and the
        // slot bytes, which the next layout reads (`docs/GOLDEN.md` §22:
        // run210's `process_group, repeat` on 701 and 741). The second
        // walk then finds every member already pointing at the slot and
        // kills none.
        if pool == last
            && let Some(i) = self
                .pushed
                .iter()
                .position(|x| x.who == g.who && x.state.pool == Some(pool) && x.list == g.list)
        {
            g.army = None;
            g.pushed = Some(i);
            return pool;
        }
        // **Parked, twenty-fourth pass, group 15 row 9 (A9 row 9).** The
        // listing has the equal arm skip `get_open_slot` and `copy_group`
        // whoever holds `last_group`'s slot, so an equal push onto an army's
        // group would be that one record. Built, it parted golden chapter 41
        // at 732 from its 1100 (frame 732: ours 6 draws against 5, index 0
        // `Guy::set_anim+0x97a < Unit::move_step+0x823` against
        // `Farms::inc_time+0x1ae`), so it is not built: the capture's push
        // is one an army-backed record does not take.
        self.unseat_group(g, pool);
        // `Groups::copy_group@006fa690` writes the stack group **into the
        // slot's own record** (§28): `who`, `num`, `ox`/`oy`, `o_dist`,
        // `o_angle`, `buildings`, `speed`, the list and its four offset
        // arrays and the angle bytes, and `stamp = game->frame` (run215's
        // slot 1 prints `stamp 621`, `docs/GOLDEN.md` §23). Everything
        // else — `form`, `order_num`, `facing`, `form_num`, `new_speed`,
        // `march` — is the previous occupant's. So the pool index is the
        // record's identity: an entry of [`Sim::pushed`] is the one slot
        // it names, and a push onto that index reuses it rather than any
        // entry whose members happen to be gone. East Indies' tick 20000
        // pushes `1/64`..`1/66` onto the slot army 1's closed group last
        // held, and the layout reads that group's `facing` (§28).
        let (slot, prev) = self.pool_record(g.who, pool);
        let state = GroupState::copied(prev, pool, self.frame, g.list.len());
        let entry = crate::group::Pushed {
            who: g.who,
            list: g.list.clone(),
            state,
            builds: Vec::new(),
        };
        let slot = match slot {
            Some(i) => {
                self.pushed[i] = entry;
                i
            }
            None => {
                self.pushed.push(entry);
                self.pushed.len() - 1
            }
        };
        g.army = None;
        g.pushed = Some(slot);
        // `copy_group` wrote the **army's own record** when the slot is one a valid
        // army owns and has emptied but not recounted (`num_units` still set,
        // list empty — `get_open_slot` hands such a slot out): the army's
        // list now names the push, so the army's tick commands it. Golden
        // chapter ninety-four's Scout `1/0` joins army 0 this way on 846
        // and gets its `ATTACK_TO` to the muster on 1021 (`docs/GOLDEN.md`
        // §120). Only the empty case: the equal arm above is parked row 9's.
        if let Some(a) = self.armies[g.who as usize].list.iter_mut().find(|a| {
            a.valid && a.num_units != 0 && a.group.pool == Some(pool) && a.units.is_empty()
        }) {
            a.units = g.list.clone();
        }
        pool
    }

    /// `push_group`'s second walk (`0070f9e0`), which the first reading
    /// left out: before a member's `+0x80` is pointed at the new slot,
    /// **the group it was in is asked to kill it** — `(*old->vtbl+0x10)
    /// (unit.o, unit.who, 0, 0)`, `Group::kill@00714110` through the
    /// vtable (`vtables.txt`) — whenever that group is not the slot being
    /// written. `Group::kill(o, who, 0, 0)` walks the squad chain exactly
    /// as `Group::add` does (§4.1): a non-captain is replaced by its
    /// captain, and a captain's `o_down` chain goes with it. So the unit
    /// that leaves is always a **whole squad**.
    ///
    /// An army's one group is its [`Army::units`] list (`docs/ARMY.md`
    /// §3.2), so a unit the simulation pushes into a stack group **leaves
    /// the army** — it stops being `Army::member`, it stops being counted
    /// by `Army::normalize`, and every later `Group::action_*` the army
    /// issues goes out without it. That is how Great Lakes' AI army is
    /// nine units and not fifteen from frame 8186 on: §12's probe pushes
    /// its pair, `Group::add` brings both squads, and the six leave
    /// (`docs/ARMY.md` §3.4). The group each member's `+0x80` names is
    /// asked first, and may be cleared ([`Sim::kill_from_named`]).
    fn unseat_group(&mut self, g: &Group, pool: u8) {
        let w = g.who as usize;
        let mut touched = self.kill_from_named(g.who, &g.list, pool);
        // `Group::kill`'s own walk: up to the captain, then down `o_down`.
        let leaving = self.leaving_squads(&g.list);
        // A pool slot is a group too, and `(*old->vtbl+0x10)` is asked of
        // whichever one holds the member — so a unit pushed twice leaves
        // the first slot rather than sitting in two groups at once.
        for i in 0..self.pushed.len() {
            self.seat_drop(Seat::Pushed(i), &leaving);
        }
        for slot in 0..self.armies[w].list.len() {
            if self.seat_drop(Seat::Army(g.who, slot), &leaving) && !touched.contains(&slot) {
                touched.push(slot);
            }
        }
        for slot in touched {
            self.army_normalize(g.who, slot);
        }
    }

    /// Every figure of each listed member's squad, `Group::kill`'s own
    /// walk: up to the captain, then down `o_down`.
    fn leaving_squads(&self, list: &[usize]) -> Vec<usize> {
        let mut leaving: Vec<usize> = Vec::new();
        for &m in list {
            if !self.units[m].alive() {
                continue;
            }
            for f in self.squad_of(self.captain_of(m)) {
                if !leaving.contains(&f) {
                    leaving.push(f);
                }
            }
        }
        leaving
    }

    /// `push_group`'s second walk (`0070f9e0`), the kill half, **on the
    /// group each member's `+0x80` names** when that is not `pool`, the
    /// slot being written (`docs/GROUPS.md` §30). `Group::kill@00714110`
    /// takes the member's squad out of that record's list and writes its
    /// `stamp = frame` — or, when the list empties, runs `Group::clear(-1)`
    /// on it, so the slot's next occupant inherits `order_num 0`, `facing
    /// 0` and `form −1` rather than the last one's counts. run45's who=1
    /// slot 65 reads exactly that on block 239, when `think_scout` repushes
    /// `1/0` onto 64, and run233's slot 64 on 17157, when `1/60` leaves
    /// army 0's orphan for army 1's fresh group on 71. Returns the army
    /// slots of `who` it took members from, for their `normalize`.
    pub(crate) fn kill_from_named(&mut self, who: Player, list: &[usize], pool: u8) -> Vec<usize> {
        // **Per member, through the seat's own `Group::kill`** (item 1171,
        // `docs/GROUPS.md` §30). `kill@00714110` shifts all six parallel
        // arrays down over the leaving index — `list +0x8cc`, `angles
        // +0x84c`, and `off_x`/`off_y`/`curr_x`/`curr_y` from `+0x4c` in
        // steps of `0x200` — so a member that stays keeps its own offset
        // at its new index. A retain on the list alone left the offsets
        // where they were: Great Sahara's army group 64 lost `1/28`–`1/30`
        // and `1/65`–`1/67` to group 68 on frame 14586, and `1/52`, now
        // listed fifth, walked to the fifth *old* slot, `1/47`'s.
        let frame = self.frame;
        let mut touched = Vec::new();
        for &m in list {
            let Some(s) = self.units[m].group_ptr else {
                continue;
            };
            if !self.units[m].alive() || s == pool {
                continue;
            }
            let Some(seat) = self.pool_seat(who, s) else {
                continue;
            };
            let before = self.seat_list(seat).len();
            self.seat_kill(seat, m, false, false);
            let (l, st) = self.seat_parts(seat);
            if l.len() < before {
                // `num == 0` runs `clear(-1)` where a survivor stamps.
                if l.is_empty() {
                    st.clear(frame);
                }
                if let Seat::Army(_, a) = seat
                    && !touched.contains(&a)
                {
                    touched.push(a);
                }
            }
        }
        touched
    }

    /// Drop every member `leaving` names from a seat's list, shifting the
    /// per-member arrays with it ([`Self::seat_remove_at`]), last to first.
    fn seat_drop(&mut self, seat: Seat, leaving: &[usize]) -> bool {
        let (l, st) = self.seat_parts(seat);
        let mut dropped = false;
        for i in (0..l.len()).rev() {
            if leaving.contains(&l[i]) {
                Self::seat_remove_at(l, st, i);
                dropped = true;
            }
        }
        dropped
    }

    // ------------------------------------------------------------------
    // The pool's numbering and its per-frame pass (§19)
    // ------------------------------------------------------------------

    /// A closed army's group, kept in the pool on its own index
    /// (`Army::close@006f8ea0` writes only its `army` to −1; `docs/ARMY.md`
    /// §22). It takes a [`Pushed`] entry whose members are all gone, as
    /// `push_group` does, else a new one; its record and list are the
    /// army's, and [`Sim::groups_process`] prunes it from then on.
    pub(crate) fn seat_orphan(&mut self, who: Player, list: Vec<usize>, state: GroupState) {
        // The entry is the pool index's own record (§28): the orphan
        // replaces whatever stale entry named its index, and never an entry
        // that names another — that entry is another slot's record, which
        // a later `copy_group` into that slot reads.
        let at = self
            .pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == state.pool);
        let entry = crate::group::Pushed {
            who,
            list,
            state,
            builds: Vec::new(),
        };
        match at {
            Some(i) => self.pushed[i] = entry,
            None => self.pushed.push(entry),
        }
    }

    /// The record pool slot `s` of `who` holds now — the army whose group
    /// sits on it, else the [`Pushed`] entry that names it — as what
    /// `copy_group` leaves behind (§28), with the entry to overwrite.
    pub(crate) fn pool_record(&self, who: Player, s: u8) -> (Option<usize>, GroupState) {
        let entry = self
            .pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == Some(s));
        let army = self.armies.get(who as usize).and_then(|x| {
            x.list
                .iter()
                .find(|a| a.valid && a.group.pool == Some(s))
                .map(|a| a.group.clone())
        });
        let prev = army
            .or_else(|| entry.map(|i| self.pushed[i].state.clone()))
            .unwrap_or_default();
        (entry, prev)
    }

    /// Whether the pool slot a pushed group sits in still names a live army
    /// in its `army` field (`GroupData +0x8`).
    ///
    /// `copy_group@006fa690` never writes that field (§28): the slot keeps
    /// its previous occupant's. When `get_open_slot` hands back the slot of
    /// an army that has lost every member, the army is still `valid` and its
    /// slot still reads `army ≥ 0`, so `Group::action_move_near@00704990`
    /// (`706178`..`70619f`) sets the pathfinder's `army` mode for whoever
    /// was pushed there. Golden chapter ninety-four's Scout `1/0` on 846:
    /// army 0's group 64 empties with the Artillery (824), the Scout's lone
    /// group is pushed onto 64, and its explore search prices every
    /// `NEARBLOCK` cell at `base << 5` (`docs/GOLDEN.md` §119).
    ///
    /// **Only while the army's `num_units` is still non-zero** — the count
    /// the army's own recount (`army_normalize`) clears. Chapter ninety-four's
    /// army 0 still reads 2 on 846 (the Artillery died on 824, no recount
    /// since); chapter sixty-two's same slot on 846 reads 0 and its Scout
    /// keeps the plain search, which a diff pins (`ch62` word frame). That
    /// the recount is what resets the link in the original is an assumption
    /// this discriminator stands on, not a reading (`docs/GOLDEN.md` §119).
    pub(crate) fn slot_keeps_army_link(&self, g: &Group) -> bool {
        let Some(i) = g.pushed else { return false };
        let Some(pool) = self.pushed.get(i).and_then(|e| e.state.pool) else {
            return false;
        };
        self.armies.get(g.who as usize).is_some_and(|x| {
            x.list
                .iter()
                .any(|a| a.valid && a.num_units != 0 && a.group.pool == Some(pool))
        })
    }

    /// The seat whose record pool slot `s` of `who` is — the army whose
    /// group sits on it, else the [`Pushed`] entry that names it.
    fn pool_seat(&self, who: Player, s: u8) -> Option<Seat> {
        let w = who as usize;
        if let Some(a) = self.armies.get(w).and_then(|x| {
            x.list
                .iter()
                .position(|a| a.valid && a.group.pool == Some(s))
        }) {
            return Some(Seat::Army(who, a));
        }
        self.pushed
            .iter()
            .position(|x| x.who == who && x.state.pool == Some(s))
            .map(Seat::Pushed)
    }

    /// The live members of whichever seat of `who` holds pool slot `s` —
    /// an army's group or a pushed one — in join order.
    ///
    /// "Live" is what `Group::get_num` answers after the `normalize` it
    /// opens with: alive, and still pointing at this slot. A unit that has
    /// joined an army since it was pushed points at the army's slot
    /// (`Unit::set_group@00605220` writes `+0x80` and leaves the old list
    /// alone), so it no longer counts for the slot it left — and since
    /// item 557 the test is the pointer itself ([`Unit::group_ptr`]), for
    /// an army's list as for a pushed one.
    pub(crate) fn pool_members(&self, who: Player, s: u8) -> Vec<usize> {
        let live = |l: &[usize]| -> Vec<usize> {
            l.iter()
                .copied()
                .filter(|&u| self.units[u].alive() && self.units[u].group_ptr == Some(s))
                .collect()
        };
        let w = who as usize;
        self.armies[w]
            .list
            .iter()
            .filter(|a| a.valid && a.group.pool == Some(s))
            .map(|a| live(&a.units))
            .chain(
                self.pushed
                    .iter()
                    .filter(|p| p.who == who && p.state.pool == Some(s))
                    .map(|p| live(&p.list)),
            )
            .find(|l| !l.is_empty())
            .unwrap_or_default()
    }

    /// The seat `u`'s back-pointer names — what the original reaches as
    /// `groups[unit->+0x80]`, and so what every reader of a unit's group
    /// asks (§23).
    ///
    /// The original has one record per pool index; this crate keeps an
    /// army's record on the army and a pushed one in [`Sim::pushed`], and
    /// may hold a stale seat beside a live one on the same index. So the
    /// seat that **lists** `u` is preferred, army first; a unit its own
    /// seat does not list — `1/62` and `1/63` on Great Lakes between
    /// 11424 and 11512 — is answered by the seat on that index that still
    /// has a live member, which is the one the original's record is.
    pub(crate) fn seat_of(&self, u: usize) -> Option<Seat> {
        let s = self.units[u].group_ptr?;
        let who = self.units[u].owner;
        let w = who as usize;
        let armies = || {
            self.armies
                .get(w)
                .into_iter()
                .flat_map(|x| x.list.iter().enumerate())
                .filter(|(_, a)| a.valid && a.group.pool == Some(s))
        };
        let pushed = || {
            self.pushed
                .iter()
                .enumerate()
                .filter(|(_, p)| p.who == who && p.state.pool == Some(s))
        };
        if let Some((a, _)) = armies().find(|(_, a)| a.units.contains(&u)) {
            return Some(Seat::Army(who, a));
        }
        if let Some((i, _)) = pushed().find(|(_, p)| p.list.contains(&u)) {
            return Some(Seat::Pushed(i));
        }
        let live = |l: &[usize]| l.iter().any(|&m| self.units[m].alive());
        if let Some((a, _)) = armies().find(|(_, a)| live(&a.units)) {
            return Some(Seat::Army(who, a));
        }
        pushed()
            .find(|(_, p)| live(&p.list))
            .map(|(i, _)| Seat::Pushed(i))
    }

    // ------------------------------------------------------------------
    // A seated group's own list, as `Group::add`/`kill`/`normalize`/`sort`
    // leave it (§4, §23)
    // ------------------------------------------------------------------

    /// The seat a [`Group`] value was built from, if it has one.
    pub(crate) fn seat_of_group(g: &Group) -> Option<Seat> {
        match (g.army, g.pushed) {
            (Some(a), _) => Some(Seat::Army(g.who, a)),
            (None, Some(i)) => Some(Seat::Pushed(i)),
            (None, None) => None,
        }
    }

    pub(crate) fn seat_list(&self, seat: Seat) -> &Vec<usize> {
        match seat {
            Seat::Army(w, a) => &self.armies[w as usize].list[a].units,
            Seat::Pushed(i) => &self.pushed[i].list,
        }
    }

    fn seat_who(&self, seat: Seat) -> Player {
        match seat {
            Seat::Army(w, _) => w,
            Seat::Pushed(i) => self.pushed[i].who,
        }
    }

    pub(crate) fn seat_parts(&mut self, seat: Seat) -> (&mut Vec<usize>, &mut GroupState) {
        match seat {
            Seat::Army(w, a) => {
                let x = &mut self.armies[w as usize].list[a];
                (&mut x.units, &mut x.group)
            }
            Seat::Pushed(i) => {
                let x = &mut self.pushed[i];
                (&mut x.list, &mut x.state)
            }
        }
    }

    /// The seat as a [`Group`] value, for the queries that take one.
    fn seat_group(&self, seat: Seat) -> Group {
        let who = self.seat_who(seat);
        match seat {
            Seat::Army(_, a) => self.army_group(who, a),
            Seat::Pushed(i) => Group {
                who,
                army: None,
                pushed: Some(i),
                list: self.pushed[i].list.clone(),
            },
        }
    }

    /// Drop list index `i` and the per-member arrays' entry with it — the
    /// shift `Group::kill` and `normalize` both do over all five arrays.
    fn seat_remove_at(list: &mut Vec<usize>, st: &mut GroupState, i: usize) {
        list.remove(i);
        if i < st.off.len() {
            st.off.remove(i);
        }
        if i < st.curr.len() {
            st.curr.remove(i);
        }
        if i < st.angles.len() {
            st.angles.remove(i);
        }
    }

    /// `speed = new_speed = UnitData::speed(find_leader)`, or 0 — the tail
    /// `compute_speed`, `kill` and `normalize` share (§18.1).
    pub(crate) fn seat_set_speed(&mut self, seat: Seat) {
        let g = self.seat_group(seat);
        let v = self.group_compute_speed(&g);
        let (_, st) = self.seat_parts(seat);
        st.speed = v;
        st.new_speed = v;
    }

    /// `Group::normalize@00711540` on a seated group (§4.3): last to first,
    /// drop a member that is dead or whose `+0x80` does not name this slot
    /// (`priority` is 0 and `id >= 0` for every seat here), then the speed
    /// tail.
    pub(crate) fn seat_normalize(&mut self, seat: Seat) {
        let (list, st) = self.seat_parts(seat);
        let pool = st.pool;
        let list = list.clone();
        for i in (0..list.len()).rev() {
            let u = list[i];
            if !self.units[u].alive() || self.units[u].group_ptr != pool {
                let (l, st) = self.seat_parts(seat);
                Self::seat_remove_at(l, st, i);
            }
        }
        self.seat_set_speed(seat);
    }

    /// `Group::get_num` (vslot `+0x4`): with **fewer than four** members
    /// and an `id`, it is `normalize` whole; otherwise only the inactive
    /// are dropped. It is the first thing `Group::add` does, which is why
    /// a squad joining a small group can lose its own head (§23).
    ///
    /// **An empty group is tested first** (`Group::get_num@00714700`:
    /// `num < 1` → `num = 0; oy = 0; ox = 0; return`, twenty-fourth pass,
    /// group 14; A9 row 12): no `normalize`, so its speed pair stands, and
    /// its origin is zeroed.
    fn seat_get_num(&mut self, seat: Seat) -> usize {
        if self.seat_list(seat).is_empty() {
            let (_, st) = self.seat_parts(seat);
            st.o = Pos::new(0, 0);
            return 0;
        }
        if self.seat_list(seat).len() < 4 {
            self.seat_normalize(seat);
        } else {
            let list = self.seat_list(seat).clone();
            for i in (0..list.len()).rev() {
                if !self.units[list[i]].alive() {
                    let (l, st) = self.seat_parts(seat);
                    Self::seat_remove_at(l, st, i);
                }
            }
        }
        self.seat_list(seat).len()
    }

    /// `Group::get_num_cap@007145c0` on a seat: an empty group answers zero
    /// untouched; one of fewer than four members is `normalize`d whole, a
    /// larger one only loses its dead; the answer is the captains standing.
    fn seat_get_num_cap(&mut self, seat: Seat) -> usize {
        if self.seat_list(seat).is_empty() {
            return 0;
        }
        self.seat_get_num(seat);
        self.seat_list(seat)
            .iter()
            .filter(|&&u| self.units[u].alive() && self.is_captain(u))
            .count()
    }

    /// `Group::add(o, who, keep_captain, const)@00714350` on a seated group,
    /// whole (§4.1, §23). Unlike [`Sim::group_add_keeping`], which builds a
    /// stack group (`id −1`, never normalized), every step here opens with
    /// `get_num` unless `konst` — so a figure whose `+0x80` does not name
    /// this group is dropped by the very recursion that adds the next one.
    /// It writes no back-pointer.
    pub(crate) fn seat_add(&mut self, seat: Seat, o: usize, keep_captain: bool, konst: bool) {
        if !konst {
            self.seat_get_num(seat);
        }
        if !self.units[o].alive() {
            return;
        }
        if !keep_captain && !self.units[o].captain {
            // The tail call: `add(unit +0x8e, who, 0, const)`.
            if let Some(up) = self.units[o].o_up {
                self.seat_add(seat, up, false, konst);
            }
            return;
        }
        if keep_captain && self.units[o].captain {
            self.seat_kill(seat, o, false, false);
        }
        let frame = self.frame;
        let (list, st) = self.seat_parts(seat);
        if list.contains(&o) || list.len() >= 128 {
            return;
        }
        // The four offset arrays and the angle byte are zeroed at the new
        // index — where the arrays reach it.
        let n = list.len();
        if st.off.len() == n {
            st.off.push((0, 0));
        }
        if st.curr.len() == n {
            st.curr.push(Pos::new(0, 0));
        }
        if st.angles.len() == n {
            st.angles.push(0);
        }
        list.push(o);
        st.stamp = frame;
        if let Some(d) = self.units[o].o_down
            && self.units[d].alive()
        {
            self.seat_add(seat, d, true, konst);
        }
        self.seat_set_speed(seat);
    }

    /// `Group::kill(o, who, keep_captain, const)@00714110` on a seated group
    /// (§4.2): a non-captain kills its captain instead and returns; a
    /// captain's subordinate goes first; then `o` leaves the list, and its
    /// `+0x80` is cleared **only if it names this group**.
    pub(crate) fn seat_kill(&mut self, seat: Seat, o: usize, keep_captain: bool, konst: bool) {
        if !keep_captain && !self.units[o].captain {
            if let Some(up) = self.units[o].o_up {
                self.seat_kill(seat, up, false, konst);
            }
            return;
        }
        if let Some(d) = self.units[o].o_down
            && (konst || self.units[d].alive())
        {
            self.seat_kill(seat, d, true, konst);
        }
        let frame = self.frame;
        let (list, st) = self.seat_parts(seat);
        let Some(i) = list.iter().position(|&m| m == o) else {
            return;
        };
        let pool = st.pool;
        Self::seat_remove_at(list, st, i);
        st.stamp = frame;
        if list.is_empty() {
            st.emptied(frame);
        }
        if self.units[o].group_ptr == pool {
            self.units[o].group_ptr = None;
        }
        self.seat_set_speed(seat);
    }

    /// `Group::sort@00708090` — `Form::categorize`'s first statement, so it
    /// runs on every formation a seated group lays out (§4.1, §23).
    ///
    /// Walk the list keeping the last captain seen; a follower whose own
    /// top captain (`UnitData::get_captain`, up `o_up` to the root) is not
    /// that one — or that comes before any captain — is `kill`ed (which
    /// takes its whole squad out, clearing each `+0x80` that names this
    /// group), the group is `normalize`d, and the follower is re-`add`ed
    /// with `const = 1`, which brings the squad back whole at the end of
    /// the list **and writes no pointer**. Repeat until a pass is clean.
    ///
    /// On Great Lakes 11512 this is the whole of `1/64`'s exit: the army's
    /// list holds `1/64` without its captain (§23), the sort kills and
    /// re-adds the squad, and `1/64` comes back listed and naming nothing.
    ///
    /// The original loops until clean; a re-add the list refuses would
    /// loop it forever, so this stops after 256 passes, twice the most a
    /// list can hold.
    pub(crate) fn seat_sort(&mut self, seat: Seat) {
        for _ in 0..256 {
            let list = self.seat_list(seat).clone();
            let mut last: Option<usize> = None;
            let mut bad = None;
            for &m in &list {
                if self.units[m].captain {
                    last = Some(m);
                    continue;
                }
                match last {
                    Some(l) if self.top_captain(m) == l => {}
                    _ => {
                        bad = Some(m);
                        break;
                    }
                }
            }
            let Some(m) = bad else { return };
            self.seat_kill(seat, m, false, false);
            self.seat_normalize(seat);
            self.seat_add(seat, m, false, true);
        }
    }

    /// `Group::sort@00708090` on a group **on the stack** (§27): the same
    /// walk as [`Self::seat_sort`], over the value's own list. The `kill`
    /// takes the follower's whole squad out and the `add` brings it back
    /// whole at the end.
    ///
    /// SEAM: the stack group carries its parent's `id`, so the original's
    /// `kill` clears `+0x80` on a squad member that names it, and its
    /// `normalize` prunes by that id. Neither is written here. No capture
    /// has a sub-group that needs sorting: every siege arm on file has one
    /// anchor, and one member sorts to itself.
    pub(crate) fn stack_sort(&self, g: &mut Group) {
        for _ in 0..256 {
            let mut last: Option<usize> = None;
            let mut bad = None;
            for &m in &g.list {
                if self.units[m].captain {
                    last = Some(m);
                    continue;
                }
                match last {
                    Some(l) if self.top_captain(m) == l => {}
                    _ => {
                        bad = Some(m);
                        break;
                    }
                }
            }
            let Some(m) = bad else { return };
            let cap = self.top_captain(m);
            let before = g.list.len();
            g.list.retain(|&x| self.top_captain(x) != cap);
            self.group_add(g, m);
            if g.list.len() < before {
                // A re-add the list refused: stop rather than loop.
                return;
            }
        }
    }

    /// `UnitData::get_captain` (vslot `+0xe4`): up `o_up` (`+0x8e`) to the
    /// figure that has none.
    pub(crate) fn top_captain(&self, u: usize) -> usize {
        let mut f = u;
        for _ in 0..self.units.len() {
            match self.units[f].o_up {
                Some(up) => f = up,
                None => break,
            }
        }
        f
    }

    /// `Unit::set_group(unit, g, 0)@00605220`: up the squad chain to its
    /// captain, then `+0x80 = g` on the captain and every **active**
    /// figure down its `o_down` chain. `Army::add_unit`'s last act, and
    /// the only writer that points a unit **at** an army's group.
    pub(crate) fn set_group(&mut self, u: usize, s: Option<u8>) {
        let units = &self.units;
        let walked = crate::squad_walk(
            u,
            units.len(),
            |i| units[i].o_up,
            |i| units[i].o_down.filter(|&d| units[d].alive()),
        );
        for f in walked {
            self.units[f].group_ptr = s;
        }
    }

    /// The slot `Groups::push_group@0070f9e0` hands a group of `who`, and
    /// the `last_group` it leaves behind (§3.1, §3.2):
    ///
    /// - the group **equals** the seat `last_group[who]` names — same
    ///   members in the same order — and that slot is reused;
    /// - otherwise `Groups::get_open_slot@006fa460`'s first rule: the
    ///   lowest of `0..46` whose seat is empty and which is not
    ///   `last_group[who]`.
    ///
    /// `last_group` starts at each player's slot 0 (`Groups::clear@00713f20`
    /// writes `who·64`), which is why a player's first push lands in slot 1.
    ///
    /// SEAM: `get_open_slot`'s two fallbacks — the oldest one-captain
    /// group, and past it the "UH OH, NEED MORE GROUPS!" pass — are not
    /// modelled; they need 46 live groups at once, and no capture has more
    /// than four. A full pool answers slot 45.
    pub(crate) fn pool_slot_for(&mut self, who: Player, list: &[usize]) -> u8 {
        let w = who as usize;
        let last = self.last_group[w];
        let s = if !list.is_empty() && self.pool_members(who, last) == list {
            last
        } else {
            self.open_slot(who)
        };
        self.last_group[w] = s;
        s
    }

    /// `Groups::get_open_slot@006fa460` (§3.1, §28): the slot a group that
    /// does not equal `last_group` is copied into, with its tail's
    /// eviction. `last_group` is the caller's to write.
    fn open_slot(&mut self, who: Player) -> u8 {
        let last = self.last_group[who as usize];
        // **One pass, as `Groups::get_open_slot@006fa460` walks it**
        // (twenty-fourth pass, group 13; A9 row 15): `get_num` on every slot
        // it reaches — each normalises a slot of one to three members — and
        // an open slot (`get_num == 0`, or a building group) that is not
        // `last_group` ends the walk; past a non-open slot the stamp arm
        // calls `get_num_cap`, which normalises it again, while its stamp
        // is no later than the best so far, and takes it when it holds
        // exactly one captain. The last of equal stamps wins; failing all of
        // them the last slot not `last_group`.
        let mut best: Option<u8> = None;
        let mut stamp = self.frame;
        let mut open = None;
        for s in 0..46u8 {
            let seat = self.pool_seat(who, s);
            let n = seat.map_or(0, |seat| self.seat_get_num(seat));
            if (n == 0 || self.pool_is_building_group(who, s)) && s != last {
                open = Some(s);
                break;
            }
            if let Some(seat) = seat
                && self.pool_record(who, s).1.stamp <= stamp
                && self.seat_get_num_cap(seat) == 1
                && s != last
            {
                stamp = self.pool_record(who, s).1.stamp;
                best = Some(s);
            }
        }
        let s = open.or(best).unwrap_or(if last == 45 { 44 } else { 45 });
        // `get_open_slot`'s tail (§3.1): unless the slot holds a
        // building group, every unit of `who` whose `+0x80` names it
        // is cleared before the new group moves in. What this reaches
        // is a pointer its list dropped — the stale half §23 is about.
        if !self.pool_is_building_group(who, s) {
            for x in &mut self.units {
                if x.owner == who && x.group_ptr == Some(s) {
                    x.group_ptr = None;
                }
            }
        }
        s
    }

    /// Does slot `s` of `who` hold a building group now?
    fn pool_is_building_group(&self, who: Player, s: u8) -> bool {
        let army = self
            .armies
            .get(who as usize)
            .is_some_and(|x| x.list.iter().any(|a| a.valid && a.group.pool == Some(s)));
        !army
            && self
                .pushed
                .iter()
                .any(|x| x.who == who && x.state.pool == Some(s) && !x.builds.is_empty())
    }

    /// `Groups::push_group(who, g, 1)` for the one-building group the AI
    /// scripts' `train_unit*` and `research_tech_with_cost` build before
    /// `Group::action_queue_up` (§28.4): [`Sim::push_buildings_group`] of
    /// one building.
    pub(crate) fn push_building_group(&mut self, who: Player, b: usize) {
        self.push_buildings_group(who, &[b]);
    }

    /// `Groups::push_group@0070f9e0(who, g, 1)` for a group of buildings
    /// (`docs/PRODUCTION.md`, "The command on a selection of buildings"):
    /// `list` in the order `Group::add@00714350` took them. A group equal to `last_group`'s record —
    /// `Group::equals_group@00708000`: the same members **in the same
    /// order** — reuses it and writes nothing; otherwise `get_open_slot`
    /// and `copy_group` seat it, and it takes every later push's slot
    /// first. A building's own `+0x80` is not carried: the member walk
    /// writes a unit's alone (vslot 8).
    pub(crate) fn push_buildings_group(&mut self, who: Player, list: &[usize]) {
        let w = who as usize;
        let last = self.last_group[w];
        let equal = self.pushed.iter().any(|x| {
            x.who == who && x.state.pool == Some(last) && x.builds == list && x.list.is_empty()
        }) && !self.armies[w]
            .list
            .iter()
            .any(|a| a.valid && a.group.pool == Some(last));
        if equal {
            return;
        }
        let pool = self.open_slot(who);
        self.last_group[w] = pool;
        let (slot, prev) = self.pool_record(who, pool);
        let state = GroupState::copied(prev, pool, self.frame, list.len());
        let entry = crate::group::Pushed {
            who,
            list: Vec::new(),
            state,
            builds: list.to_vec(),
        };
        match slot {
            Some(i) => self.pushed[i] = entry,
            None => self.pushed.push(entry),
        }
    }

    /// The pool slot `u` points at, as the dump prints it — `who·64 + s`,
    /// or −1 for a unit in no group. For a probe and the diff harness.
    pub fn pool_group_of(&self, u: usize) -> i64 {
        let who = i64::from(self.units[u].owner);
        self.units[u]
            .group_ptr
            .map_or(-1, |s| who * 64 + i64::from(s))
    }

    /// The record half of `who`'s pool slot `s` — an army's or a pushed
    /// group's, whichever [`Sim::pool_list`] would list — for a widening
    /// that compares the dump's `GROUPDATA` whole (`docs/GOLDEN.md` §23).
    pub fn pool_state(&self, who: Player, s: u8) -> Option<&GroupState> {
        let w = who as usize;
        let army = self
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .find(|a| a.valid && a.group.pool == Some(s))
            .map(|a| &a.group);
        army.or_else(|| {
            self.pushed
                .iter()
                .find(|p| p.who == who && p.state.pool == Some(s) && !p.list.is_empty())
                .map(|p| &p.state)
        })
    }

    /// Every record this crate holds on `who`'s pool slot `s` — an army's
    /// group first, then each [`Pushed`] entry naming it, stale or not —
    /// with its members as object numbers. The original has one record per
    /// index; this crate can hold a closed army's orphan and a later push
    /// on the same one (`docs/GROUPS.md` §28). For a widening.
    pub fn pool_records(&self, who: Player, s: u8) -> Vec<(&GroupState, Vec<i16>)> {
        let names = |l: &[usize]| l.iter().map(|&u| self.units[u].index).collect();
        let w = who as usize;
        let armies = self
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .filter(|a| a.valid && a.group.pool == Some(s))
            .map(|a| (&a.group, names(&a.units)));
        let pushed = self
            .pushed
            .iter()
            .filter(|p| p.who == who && p.state.pool == Some(s))
            .map(|p| (&p.state, names(&p.list)));
        armies.chain(pushed).collect()
    }

    /// The member list of the seat on `who`'s pool slot `s`, as object
    /// numbers in list order — what the dump's `GROUPDATA` prints under
    /// `id who·64 + s`. Empty for a slot nothing holds. For the diff
    /// harness; nothing in the simulation calls it.
    pub fn pool_list(&self, who: Player, s: u8) -> Vec<i16> {
        let w = who as usize;
        let army = self
            .armies
            .get(w)
            .into_iter()
            .flat_map(|x| x.list.iter())
            .find(|a| a.valid && a.group.pool == Some(s))
            .map(|a| a.units.clone());
        let list = army.or_else(|| {
            self.pushed
                .iter()
                .find(|p| p.who == who && p.state.pool == Some(s) && !p.list.is_empty())
                .map(|p| p.list.clone())
        });
        list.unwrap_or_default()
            .into_iter()
            .map(|u| self.units[u].index)
            .collect()
    }

    /// `Groups::process@006fa210` — once a frame, from
    /// `GameDaemon::process_all` after the markets and before
    /// `Armies::process_all` and the objects: for **one** slot of each
    /// player, the cursor `proc_group`, the same prune `Group::normalize`
    /// runs and then `speed = new_speed = UnitData::speed(find_leader)`, or
    /// 0 for a group with no leader (§3.3, §19).
    ///
    /// The cursor starts at 0 (`Groups::Groups`, `Groups::clear`) and steps
    /// once a call, wrapping at 64, so on frame `f` it is `f mod 64` — and
    /// every group's cap goes back to its leader's own speed once every 64
    /// frames whatever its members have reported. It is the writer
    /// `docs/GROUPS.md` §18 did not count: Great Lakes' raid walks at its
    /// slow squad's 25 from frame 10241 because 10241 is `65 mod 64`'s
    /// frame, with nobody in the group able to report.
    ///
    /// The prune is the back-pointer's (§23): a member is kept while it is
    /// alive and its `+0x80` names this slot, on an army's list and a
    /// pushed one alike. `find_role` is not modelled — no consumer of
    /// `GroupData::role` is.
    pub(crate) fn groups_process(&mut self, frame: i64) {
        let s = u8::try_from(frame.rem_euclid(64)).expect("under 64");
        for w in 0..self.armies.len() {
            let who = w as Player;
            for a in 0..self.armies[w].list.len() {
                if self.armies[w].list[a].group.pool != Some(s) {
                    continue;
                }
                // `normalize`'s prune, on the army's list too (§23): a
                // member whose `+0x80` no longer names this slot is
                // dropped. The army's own counts are `Army::normalize`'s
                // and wait for it.
                let keep: Vec<usize> = self.armies[w].list[a]
                    .units
                    .iter()
                    .copied()
                    .filter(|&u| self.units[u].alive() && self.units[u].group_ptr == Some(s))
                    .collect();
                self.army_keep_units(w, a, keep);
                let g = self.army_group(who, a);
                let v = self.group_compute_speed(&g);
                let st = &mut self.armies[w].list[a].group;
                st.speed = v;
                st.new_speed = v;
            }
            for i in 0..self.pushed.len() {
                if self.pushed[i].who != who || self.pushed[i].state.pool != Some(s) {
                    continue;
                }
                // The shift is over all five arrays (`normalize@00711540`'s
                // inner loop, item 1776): the member's `off`, `curr` and
                // `angles` entries leave with it.
                for k in (0..self.pushed[i].list.len()).rev() {
                    let u = self.pushed[i].list[k];
                    if self.units[u].alive() && self.units[u].group_ptr == Some(s) {
                        continue;
                    }
                    let p = &mut self.pushed[i];
                    Self::seat_remove_at(&mut p.list, &mut p.state, k);
                }
                // A building group's members take the same test's first
                // half (`(+8 & 1) == 0`, the object no longer in use): a
                // building closed under the group leaves it on the slot's
                // turn, and an emptied slot is the dump's no longer.
                // run1512's `[2007]`, disbanded on 832, leaves slot 1 on
                // 833, `833 mod 64` (item 3918, `docs/GOLDEN.md` §165).
                let buildings = &self.buildings;
                self.pushed[i].builds.retain(|&b| buildings[b].alive);
                let g = crate::group::Group {
                    who,
                    army: None,
                    pushed: Some(i),
                    list: self.pushed[i].list.clone(),
                };
                let v = self.group_compute_speed(&g);
                let st = &mut self.pushed[i].state;
                st.speed = v;
                st.new_speed = v;
            }
        }
    }

    // ------------------------------------------------------------------
    // The queries (§4.3, §4.4)
    // ------------------------------------------------------------------

    /// `GroupData::find_leader` (§4.4): the active captain with the lowest
    /// `FormData::type_cat`, on the map if any is, ties to the first.
    ///
    /// The key is live as of 2026-08-26 — `sim::form::type_cat` was built
    /// for `Form::compute` and this is its second reader. The listing at
    /// `0070ccb0` gives the winner test as `local_8 < 0 || cat < best`, so
    /// the **first** qualifying member always takes the lead and only a
    /// **strictly** lower category displaces it; a tie goes to the first.
    /// Two passes, and only the first tests `is_on_map`.
    ///
    /// A typeless unit — a harness fixture, never an install's — keys at
    /// `NUM_FORM_CAT`, so it leads only when it is the first candidate and
    /// loses to any typed one that follows.
    pub fn group_find_leader(&self, g: &Group) -> Option<usize> {
        self.group_find_leader_slot(g).map(|(u, _)| u)
    }

    /// [`Self::group_find_leader`] with the out-parameter the original also
    /// fills: the winner's **index in `list`**, which `compute_form` uses to
    /// read the leader's own `angles` byte (§6.3).
    pub fn group_find_leader_slot(&self, g: &Group) -> Option<(usize, usize)> {
        let human = self.nation[g.who as usize].human;
        for on_map_only in [true, false] {
            let mut best: Option<(usize, usize, usize)> = None;
            for (i, &u) in g.list.iter().enumerate() {
                if !self.units[u].alive() || !self.units[u].captain {
                    continue;
                }
                if on_map_only && !self.units[u].on_map {
                    continue;
                }
                let key = self.units[u].ty.map_or(crate::form::cat::NUM, |t| {
                    crate::form::type_cat(&self.unit_types[t], t, human)
                });
                if best.is_none_or(|(k, _, _)| key < k) {
                    best = Some((key, u, i));
                }
            }
            if let Some((_, u, i)) = best {
                return Some((u, i));
            }
        }
        None
    }

    /// `GroupData::is_on_map`: at least one active on-map captain.
    pub fn group_is_on_map(&self, g: &Group) -> bool {
        !g.list.is_empty()
            && g.list
                .iter()
                .any(|&u| self.units[u].alive() && self.units[u].captain && self.units[u].on_map)
    }

    /// `GroupData::num_valid`: members whose object is valid.
    pub fn group_num_valid(&self, g: &Group) -> i32 {
        g.list.iter().filter(|&&u| self.units[u].alive()).count() as i32
    }

    /// `Group::normalize@00711540` (§4.3): drop the dead, last to first,
    /// and then **recompute the group's speed from its leader**.
    ///
    /// The tail is the second half of §18: `find_role`, then
    /// [`Sim::group_compute_speed`] written into both halves of the pair.
    /// It is what primes a freshly pushed group — `Army::find_target`'s
    /// probe normalizes at the end of its own build
    /// ([`Sim::find_target_probe`](Self::find_target_probe)) — so the
    /// first frame of the first march is already capped, rather than
    /// running at the leader's own speed until its first report.
    pub fn group_normalize(&mut self, g: &mut Group) {
        g.list
            .retain(|&u| u < self.units.len() && self.units[u].alive());
        self.group_set_speed(g);
    }

    /// `Group::normalize`'s speed tail on its own, for the sites that
    /// normalize a **seated** group without a local [`Group`] to prune:
    /// `Group::kill_group_move@007123f0` and
    /// `Group::refresh_group_order@00713a50` both open with `normalize`.
    ///
    /// SEAM: the prune itself. The seat's list is not walked here, because
    /// membership is maintained elsewhere in this crate and dropping a
    /// dead member from a pool slot mid-frame is a different change.
    pub(crate) fn group_set_speed(&mut self, g: &Group) {
        let s = self.group_compute_speed(g);
        if let Some(st) = self.gstate_mut(g) {
            st.speed = s;
            st.new_speed = s;
        }
    }

    // ------------------------------------------------------------------
    // The speed cap (§14)
    // ------------------------------------------------------------------

    /// The cap `UnitData::get_speed@00608720`'s last arm reads —
    /// `groups[unit->group].speed` — **0 for a unit with no seat**, which
    /// is what the original's `group >= 0` test answers in the same
    /// breath: a zero caps nothing either way.
    ///
    /// Written without building a [`Group`], because this runs once per
    /// moving unit per frame and [`Sim::group_of`] clones the list.
    pub(crate) fn group_speed_of(&self, u: usize) -> i32 {
        match self.seat_of(u) {
            Some(Seat::Army(w, a)) => self.armies[w as usize].list[a].group.speed,
            Some(Seat::Pushed(i)) => self.pushed[i].state.speed,
            None => 0,
        }
    }

    /// The cap and its accumulator — `GroupData::speed` and `new_speed` —
    /// of the seat `u` sits in, army or pool slot, for a probe's line.
    /// `None` for a unit with no seat. Read-only; nothing in the
    /// simulation calls it.
    pub fn group_speed_pair_of(&self, u: usize) -> Option<(i32, i32)> {
        let st = match self.seat_of(u)? {
            Seat::Army(w, a) => &self.armies[w as usize].list[a].group,
            Seat::Pushed(i) => &self.pushed[i].state,
        };
        Some((st.speed, st.new_speed))
    }

    /// `Group::report_speed@00713bb0`, which `Unit::do_group_move` inlines
    /// at `5e8336` — a **follower** in formation drives the group's speed
    /// down to its own, never up.
    ///
    /// The value reported is `UnitData::speed`, layer two, not
    /// `get_speed` — so the ground the follower is standing on and the
    /// order it carries do not enter the cap the rest of the group walks
    /// at.
    pub(crate) fn group_report_speed(&mut self, g: &Group, speed: i32) {
        let Some(st) = self.gstate_mut(g) else { return };
        if speed < st.speed {
            st.speed = speed;
            st.new_speed = speed;
        }
    }

    /// `Group::leader_report_speed@007137f0`, which `Unit::do_group_move`
    /// inlines at `5e7a9a` — the **leader**, on every frame its own step
    /// succeeds, publishes the pass.
    ///
    /// `speed` takes what the accumulator held, and the accumulator
    /// restarts at the leader's own **uncapped** speed
    /// (`get_speed(x, y, 1)`). So the cap a member walks at is the
    /// *previous* pass's minimum, one frame stale, and a group whose slow
    /// members stop reporting drifts back up to the leader's own speed
    /// over two frames rather than at once.
    ///
    /// Neither function has a caller in the 48k-function export: both are
    /// inlined at their one call site, which is why a grep for callers
    /// finds nothing and reads like dead code.
    pub(crate) fn group_leader_report_speed(&mut self, g: &Group, speed: i32) {
        let Some(st) = self.gstate_mut(g) else { return };
        st.speed = st.new_speed;
        st.new_speed = speed;
        st.march = false;
    }

    /// `do_group_move`'s forced-march arm under the leader's report
    /// (`groups[+0x80] +0x4b = 1`): the leader's leader flags carry
    /// `0x8000` and `has_general(0x8000, -1)` found a marching hero.
    pub(crate) fn group_set_march(&mut self, g: &Group) {
        if let Some(st) = self.gstate_mut(g) {
            st.march = true;
        }
    }

    /// `Group::compute_speed@00707f80` — the group's speed as the four
    /// membership sites set it: the **leader's** `UnitData::speed`, or 0
    /// when the group holds buildings, holds nothing, or has no leader.
    ///
    /// The original recomputes at `Group::add`, `Group::kill`,
    /// `Group::normalize` and `Group::clear`, and the group machinery
    /// reaches all four constantly: a dozen `Group::is_*` queries
    /// normalize, `Group::kill_group_move` and
    /// `Group::refresh_group_order` open with one, and `Form::categorize`
    /// calls `Group::sort`, which kills a member and re-adds it.
    /// SEAM: `Group::add` and `Group::kill` recompute too, and neither is
    /// a site this crate has — `Form::categorize`'s call to `Group::sort`,
    /// which kills a member out of category order and re-adds it, is not
    /// modelled. What is modelled is [`Sim::group_normalize`] and the two
    /// sites that open with it, [`Sim::seat_normalize`], and
    /// `Army::normalize`'s own pass over its group
    /// ([`Sim::army_normalize`], item 1586).
    pub(crate) fn group_compute_speed(&self, g: &Group) -> i32 {
        if g.list.is_empty() {
            return 0;
        }
        self.group_find_leader(g).map_or(0, |u| self.unit_speed(u))
    }

    /// `GroupData::get_stance_type` (§4.4): the leader's, or the first
    /// member's that is neither `None` nor `Caster`.
    pub fn group_stance_type(&self, g: &Group) -> StanceType {
        let Some(leader) = self.group_find_leader(g) else {
            return StanceType::None;
        };
        let s = self.unit_stance_type(leader);
        if s != StanceType::None && s != StanceType::Caster {
            return s;
        }
        let mut caster = false;
        for &u in &g.list {
            let s = self.unit_stance_type(u);
            if s == StanceType::None {
                continue;
            }
            if s != StanceType::Caster {
                return s;
            }
            caster = true;
        }
        if caster { StanceType::Caster } else { s }
    }

    /// `ObjectData` vslot `+0x108` → the type's `UnitTypeData::get_stance_type@0061d350`
    /// (the PDB names the slot; `docs/GROUPS.md` §4.4). The order of the
    /// tests is the original's and it is load-bearing:
    ///
    /// 1. `role & MILITARY` → `PACKER` if `unit_flags2 & 4`, else
    ///    `COMBAT` — so a military caster is *combat*;
    /// 2. the four citizen ids (`role::CITIZEN`, set for exactly those) →
    ///    `WORKER`;
    /// 3. `(unit_flags2 & 6) == 2` — a caster that does not pack → `CASTER`;
    /// 4. else `NONE`. Having an attack decides nothing on its own.
    ///
    /// `Profile::packs` is `unit_flags2 & 4` for a type from the install.
    pub(crate) fn unit_stance_type(&self, u: usize) -> StanceType {
        let Some(t) = self.units[u].ty else {
            return StanceType::None;
        };
        let c = &self.unit_types[t];
        if c.cols.is(crate::ai_load::role::MILITARY) {
            return if c.combat.packs {
                StanceType::Packer
            } else {
                StanceType::Combat
            };
        }
        if c.cols.is(crate::ai_load::role::CITIZEN) {
            return StanceType::Worker;
        }
        if c.cols.flag2(uflags2::CASTER) && !c.combat.packs {
            return StanceType::Caster;
        }
        StanceType::None
    }

    /// `GroupData::get_stance_option@0070bab0`: the **modal** stance option
    /// over the members — a histogram of each valid member's `+0xb1` that
    /// answers `has_stance_type` (vslot `+0x104`) for the group's own
    /// stance type, argmax on a **strict `<`** so a tie goes to the lowest
    /// index, and 0 when nothing counts. This, not the leader's own stance,
    /// is what `action_stance`'s negative cycle steps from (§8).
    fn group_stance_option(&self, g: &Group, ty: StanceType) -> i32 {
        let n = ty.options();
        if n == 0 {
            return 0;
        }
        // Six is the largest option count (`STANCE_COMBAT`).
        let mut counts = [0i32; 6];
        for &u in &g.list {
            if !self.units[u].alive() || self.unit_stance_type(u) != ty {
                continue;
            }
            let opt = if ty == StanceType::Combat {
                stance_option(self.units[u].combat.stance)
            } else {
                i32::from(self.units[u].stance)
            };
            if let Some(c) = counts.get_mut(opt as usize) {
                *c += 1;
            }
        }
        let mut best = 0usize;
        for i in 0..n as usize {
            if counts[i] > 0 && counts[best] < counts[i] {
                best = i;
            }
        }
        best as i32
    }

    /// `GroupData::get_form`: the `unit +0xaa` shared by every active
    /// on-map member, −1 as soon as two differ.
    pub fn group_get_form(&self, g: &Group) -> i32 {
        let mut form = -1;
        let mut seen = false;
        for &u in &g.list {
            if !self.units[u].alive() || !self.units[u].on_map {
                continue;
            }
            let f = i32::from(self.units[u].form);
            if seen && f != form {
                return -1;
            }
            form = f;
            seen = true;
        }
        form
    }

    /// `GroupData::get_loc@0070e030`: the leader's position, **and the two
    /// `(ox, oy)` substitutions on top of it** (§6.3, §12.5).
    ///
    /// Both gates are a `vector_dist` whose operands the decompiler loses
    /// to `unaff_EDI`/`unaff_ESI`; the listing at `70e109` and `70e190`
    /// supplies them, and `jg 0x180` makes both bounds **inclusive**:
    ///
    /// - the leader within `0x180` of the group's own `(ox, oy)` returns
    ///   `(ox, oy)` outright;
    /// - otherwise, the leader's **current move order's destination**
    ///   within `0x180` of `(ox, oy)` returns the leader's position
    ///   translated by the difference, `pos - order.dest + (ox, oy)` —
    ///   "where the group will be".
    ///
    /// The second arm's pair is `MoveOrder +0x4`/`+0x8`, which the type
    /// record names **`x`/`y`** — the order's destination. It is *not*
    /// `orig_x`/`orig_y`, which sit at `+0x44`/`+0x48`; the surrounding
    /// code reads the same in both spellings and only the layout settles
    /// it.
    ///
    /// This matters because [`Self::group_move`] feeds the answer to both
    /// the formation angle **and** `group_plan_path`'s start, so a
    /// substitution here moves the cell the leader's route is planned
    /// from.
    fn group_loc(&self, g: &Group) -> Option<Pos> {
        let u = self.group_find_leader(g)?;
        let pos = self.units[u].pos;
        let o = self.group_o(g);
        // `70e0f8`/`70e103`: either coordinate negative and the record is
        // not a point at all — a group that has never moved.
        if o.x < 0 || o.y < 0 {
            return Some(pos);
        }
        if vector_dist(pos.x - o.x, pos.y - o.y) <= GROUP_LOC_NEAR {
            return Some(o);
        }
        // `70e14b`/`70e162`: the object must be a unit and `is_moving` —
        // which is `orderlist` head, `get_type`, non-zero — and then
        // `get_move_order` must hand back a `MoveOrder`. All three
        // collapse here to "the current order is a move".
        let Some(dest) = self.current_order(u).and_then(Order::move_dest) else {
            return Some(pos);
        };
        if vector_dist(dest.x - o.x, dest.y - o.y) > GROUP_LOC_NEAR {
            return Some(pos);
        }
        Some(Pos::new(pos.x - dest.x + o.x, pos.y - dest.y + o.y))
    }

    /// `GroupData::get_loc_to@0070c5d0` — the group's location for a
    /// **`QUEUE_LAST`** move, which is not where the leader stands but
    /// where it will *end up*: `UnitData::get_final_loc` walks the orders
    /// already on its list and hands back the first point one of them
    /// names.
    ///
    /// The `(ox, oy)` override is the same `0x180` window
    /// [`Self::group_loc`] uses, measured from the **unit** like it — not
    /// from the final point (`0070c6cc`; the decompile's operands are
    /// `unaff_` and the listing settles it, item 1528).
    ///
    /// SEAM: the `buildings` seat (`list[0]` rather than `find_leader`) and
    /// the `get_inside` hop for a garrisoned leader, neither of which any
    /// group this simulation builds reaches.
    pub(crate) fn group_loc_to(&self, g: &Group) -> Option<Pos> {
        let u = self.group_find_leader(g)?;
        let p = self.unit_final_loc(u);
        let o = self.group_o(g);
        // **The window is measured from the leader's own position**
        // (`70c6cc`–`70c702`: `objects[who][leader] +0x10/+0x14` less
        // `(ox, oy)`, `get_final_loc`'s answer already written to the
        // caller's pair), the decompile's `vector_dist(unaff_EBX,
        // unaff_EDI)` hiding the operands. Item 1528: a leader still
        // 3,400 units from the point it was ordered to leaves the window
        // shut, the answer the *final* location, and a formation re-issued
        // at `QUEUE_LAST` to a point behind that location is laid out
        // reversed (`docs/AI.md` §142).
        let pos = self.units[u].pos;
        if o.x >= 0 && o.y >= 0 && vector_dist(pos.x - o.x, pos.y - o.y) <= GROUP_LOC_NEAR {
            return Some(o);
        }
        Some(p)
    }

    /// `UnitData::get_final_loc@00608040` — where this unit's order list
    /// leaves it.
    ///
    /// **The walk starts at the bottom of the ring** — the order issued
    /// last at `QUEUE_LAST`, the one that runs last — and climbs toward the
    /// current order, stopping at the first that names a point: the
    /// *final* location is the deepest point on the list (item 2055,
    /// `docs/AI.md` §206). The ring's anchor (`orderlist +0x14`) is where
    /// `get_final_loc` starts and `OrderList::log_data` prints first, and
    /// the current order is the node before it (`kill_current_order` reads
    /// `+0x14 → +4`), so the dump prints the bottom first and the current
    /// order last. ~~From the head~~ until item 2055: a re-issued charge
    /// read the charge's own point, (6216, 29784), where the original reads
    /// the copy beneath it, (6072, 29832). A move order hands back its
    /// destination (vslot `+0x14`, then `+0xb8`'s `(+4, +8)` — the order's
    /// `x`/`y`, not its live waypoint), and an order carrying a
    /// `TargetOrder` hands back its target **object's** position, provided
    /// that object is still `flags & 1`. An order naming neither is walked
    /// past.
    ///
    /// Then the answer is checked with `invalid_loc(unit, tile, 1, …)` —
    /// buildings ignored, which is why a farm's own cell is an answer at all
    /// — and a point that fails it is replaced by the unit's own position,
    /// as is a list that named nothing.
    ///
    /// **The target lives on the unit here, not on the order**
    /// (`docs/GROUPS.md` §12), so the target arm reads
    /// [`combat::State::target`] for an attack and the order's own building
    /// for the three that carry one. SEAM: a gather order out at a resource
    /// tile, and a queued attack behind another unit's — this crate keeps
    /// one target per unit, so a list holding two attacks answers with the
    /// live one for both.
    ///
    /// **A guard is a `TargetOrder` too**, and it answers the escorted
    /// unit's own position, never its post (item 2055, `docs/AI.md` §206):
    /// Great Sahara's leader `1/17` holds a `GUARDORDER` on the MLRS `1/144`
    /// at its head on block 4265, and the original's `get_loc_to` on run821's
    /// packet answers (14743, 25626), the MLRS's point. Walked past, the
    /// answer was the charge's (6216, 29784), and `compute_form`'s reverse
    /// on the last re-issue never fired. **And so is a follow** (item 2131,
    /// `docs/GOLDEN.md` §78): `FollowOrder`'s block carries `TargetOrder`'s
    /// `is_targeted` and `get_target_order` thunks, and run827's two
    /// shift-moves on following squads lay their formation out from the
    /// leader's point — under a leg, too, since the walk starts at the
    /// bottom.
    pub(crate) fn unit_final_loc(&self, u: usize) -> Pos {
        let here = self.units[u].pos;
        let found = self.units[u]
            .orders
            .iter()
            .rev()
            .find_map(|o| match o.body {
                Body::Move(m) => Some(m.dest),
                Body::Attack(_) => self.units[u]
                    .combat
                    .target
                    .and_then(|t| self.obj_alive_pos(t)),
                Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                    self.obj_alive_pos(crate::combat::Obj::Building(b))
                }
                Body::Gather(gt) => self.obj_alive_pos(crate::combat::Obj::Building(gt.building)),
                Body::Guard(gd) => self.obj_alive_pos(crate::combat::Obj::Unit(gd.target)),
                Body::Follow(f) => self.obj_alive_pos(crate::combat::Obj::Unit(f.target)),
                _ => None,
            });
        let Some(p) = found else { return here };
        if self.invalid_loc(u, p.tile(), true, false, false, false, false) != 0 {
            return here;
        }
        p
    }

    /// `flags & 1` alone — `get_final_loc`'s own test on a target object,
    /// which unlike [`Sim::active`] asks nothing about the map or combat.
    fn obj_alive_pos(&self, o: crate::combat::Obj) -> Option<Pos> {
        let alive = match o {
            crate::combat::Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive()),
            crate::combat::Obj::Building(b) => self.buildings.get(b).is_some_and(|b| b.alive),
        };
        alive.then(|| self.pos_of(o))
    }

    /// `GroupData +0x18`/`+0x1c` — the point the last move was ordered
    /// **to**, before any slot offset. A group with no army has no record
    /// to read, and the original's `(-1, -1)` initialiser is what a
    /// never-moved group holds.
    pub(crate) fn group_o(&self, g: &Group) -> Pos {
        self.gstate(g).map_or(Pos::new(-1, -1), |st| st.o)
    }

    // ------------------------------------------------------------------
    // The type tests the actions ask (§6.5, §7, §9, §10)
    // ------------------------------------------------------------------

    pub(crate) fn is_siege_unit(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].combat.siege || self.unit_types[t].cols.flag(uflags::SIEGE)
        })
    }

    pub(crate) fn is_hero_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::GENERAL))
    }

    /// `UnitData::is_supply@0046ce80`, the base every unit vtable's `+0xcc`
    /// names: `unit_flags2 & 0x40`, the raw bit (swept by
    /// `sweep::unit_is_supply`). A hero that carries it — The Citizen, TY
    /// 356 — answers yes, so `action_siege_attack_to`'s supply arm takes
    /// it before a later Supply Wagon (item 3682, `docs/AI.md` §346.2).
    pub(crate) fn is_supply_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO))
    }

    pub(crate) fn is_special_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SCOUT))
    }

    pub(crate) fn group_domain(&self, u: usize) -> Domain {
        self.units[u]
            .ty
            .map_or(Domain::Land, |t| self.unit_types[t].combat.domain)
    }

    /// `UnitData::is_plane@0046ce40` — `domain == 2 && !(unit_flags & 0x20)`.
    /// There is no altitude test: a plane is skipped whether it is in the
    /// air or on a runway (§7). Bit `f` is `unitrules.xml`'s "flies like a
    /// helicopter", carried by exactly three types — `Helicopter` and the
    /// two `Attack Helicopter`s, all of them `<DOMAIN>Air` — so a
    /// **helicopter is not a plane** and is ordered like a ground unit.
    pub fn is_plane(&self, u: usize) -> bool {
        self.group_domain(u) == Domain::Air
            && !self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.flag(uflags::HELICOPTER))
    }

    /// `Group::is_attacking_to@0070ea70` (item 1012, `docs/COMBAT.md`
    /// §66): after `normalize`, every member that is alive and on the map
    /// and has a head order is walking an attack-move — `ATTACK_TO` (2)
    /// or `GROUP_ATTACK_TO` (`0x15`). A member with no order passes; one
    /// member fighting, or walking to where it will fight, is a no.
    pub(crate) fn group_is_attacking_to(&mut self, g: &Group) -> bool {
        let mut g = g.clone();
        self.group_normalize(&mut g);
        g.list.iter().all(|&u| {
            !(self.units[u].alive() && self.units[u].on_map)
                || self
                    .current_order(u)
                    .is_none_or(|o| matches!(o.index(), index::ATTACK_TO | index::GROUP_ATTACK_TO))
        })
    }

    /// Whether a member takes an order at all: active, on the map, and not
    /// a plane. Every action's inner loop opens with this.
    fn group_member_orderable(&self, u: usize) -> bool {
        self.units[u].alive() && self.units[u].on_map && !self.is_plane(u)
    }

    // ------------------------------------------------------------------
    // The actions
    // ------------------------------------------------------------------

    /// `Group::action_swarm_around@0070fbe0` at **`QUEUE_NEW`** or
    /// **`QUEUE_LAST`** — the position a player's drop issues a build at
    /// (`Group::action_build@00707510`, `docs/GOLDEN.md` §26) and the one
    /// `finish_insert` re-issues a build or a repair at (§24). The two
    /// are one arm of the original: the member's approach is
    /// `add_move_facing_order` at the swarm's own position (`QVar16 =
    /// param_3`), so `QUEUE_NEW` clears each member's list first, and the
    /// order goes in behind it at `QUEUE_LAST` either way.
    ///
    /// The members are walked twice, land (`domain` 0) first and sea
    /// second; an air member is never taken. A citizen (`0x32`/`0x33`)
    /// gets the approach and the order ([`Sim::swarm_around_last`]).
    /// The approach is an `EXPLORE_TO` for a building under a computer
    /// (`local_40 = ~(leader_flags >> 1) & 2 | 1`) and a `MOVE_TO`
    /// otherwise.
    ///
    /// A member that can cast `0x293` (a Militia) is a builder too, and
    /// takes the Civilian between its approach and its order
    /// ([`Sim::swarm_around_last`], item 1167, `docs/GOLDEN.md` §49).
    ///
    /// SEAM, none reached by a capture on file: the non-builders, which
    /// the original gathers into a scratch group and sends `MOVE_TO` the
    /// site at `QUEUE_NEW`; a builder already inside a building
    /// (`count_inside`), which goes with them;
    /// the gather filter (`local_30`, the group's idle citizens, against a
    /// member whose action is a gather); and `is_busy`, a member mid-cast
    /// or boarding. `finish_insert`'s one reach on file is a goody box's
    /// one-member group whose member was just halted. **`QUEUE_FIRST`**
    /// is the original's re-entry (`set_up_insert`, `action_halt`, this
    /// arm at `QUEUE_NEW`, `finish_insert`), which a group command on file
    /// never passes; it is taken here member by member through
    /// [`Sim::swarm_around`], the shape `do_build`'s re-entry has.
    pub(crate) fn group_action_swarm_around(
        &mut self,
        g: &Group,
        b: usize,
        pos: QueuePos,
        body: Body,
        action: bool,
    ) {
        let Some(who) = self.buildings.get(b).map(|bd| i32::from(bd.owner)) else {
            return;
        };
        let order = if matches!(body, Body::Repair(_)) {
            crate::orders::index::REPAIR as i32
        } else {
            crate::orders::index::BUILD_AT as i32
        };
        let queue = match pos {
            QueuePos::First => 0,
            QueuePos::Last => 1,
            QueuePos::New => 2,
        };
        crate::swarm_around_view::action_swarm_around_steps(
            &mut crate::swarm_around_view::SimSwarm::new(self, g),
            b as i32,
            who,
            queue,
            order,
            i32::from(action),
        );
    }

    /// `Group::action_build@00707510` (`docs/GOLDEN.md` §26): a player's
    /// drop. The group must be on the map; the type must be available
    /// (`LeaderData::type_avail(t, 1) == 4`); then the site, the city
    /// limit and the price are [`Sim::place_building`]'s, in the
    /// original's order of refusal — `validate_build`'s snap,
    /// `blocked_site` and `can_afford`, the city limit, and the loop's
    /// snap, `blocked_site`, price and `Objects::init_build`. The loop
    /// runs once: `action_build` pushes one `PathData`, the first point,
    /// and nothing else pushes. A placed site is swarmed at the command's
    /// position with `BUILD_AT` and the action bit
    /// ([`Self::group_action_swarm_around`]). Returns the site.
    ///
    /// SEAM: the scenario's free-building semaphore, the feedback and
    /// the sounds, and `ignore_orders`; none is reached by a capture on
    /// file.
    pub fn group_action_build(
        &mut self,
        g: &Group,
        at: Pos,
        ty: usize,
        pos: QueuePos,
    ) -> Option<usize> {
        if !self.group_is_on_map(g) || !g.list.iter().any(|&u| self.units[u].alive()) {
            return None;
        }
        let tree = self.build_types[ty].tree?;
        if self.type_avail(g.who, tree) != crate::tech::AVAILABLE {
            return None;
        }
        let b = self.place_building(g.who, ty, at).ok()?;
        self.group_action_swarm_around(g, b, pos, Body::Build(b), true);
        Some(b)
    }

    /// `Group::set_up_insert@0070e520` (§6.2, §17): the **leader's**
    /// action-flagged orders, copied aside ahead of a halt, and the target
    /// an attack copy needs — this crate keeps it on the unit rather than on
    /// the order (`docs/GROUPS.md` §12), and the halt clears it.
    pub(crate) fn group_set_up_insert(&self, g: &Group) -> Insert {
        let leader = self.group_find_leader(g);
        let saved: Vec<Order> = leader
            .map(|l| {
                self.units[l]
                    .orders
                    .iter()
                    .filter(|o| o.has(flag::ACTION))
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        // The target goes with the copies: this crate keeps it on the
        // unit rather than on the attack order (`docs/GROUPS.md` §12),
        // and the halt below clears it.
        let aim = leader.and_then(|l| {
            let unit = &self.units[l];
            unit.combat.target.map(|t| (t, unit.combat.mandatory))
        });
        Insert { saved, aim }
    }

    /// `Group::finish_insert@0070e620` (§6.2, §17): each copy
    /// [`Self::group_set_up_insert`] saved, re-issued as a **group**
    /// action at `QUEUE_LAST`. The dispatch is
    /// [`crate::finish_insert_view::finish_insert`], swept against the
    /// original (item 3372); this crate's view over the copies is
    /// [`SimInsert`].
    pub(crate) fn group_finish_insert(&mut self, g: &Group, insert: Insert) {
        let Insert { saved, aim } = insert;
        let mut v = SimInsert {
            sim: self,
            g,
            saved: saved.into(),
            cur: None,
            aim,
        };
        crate::finish_insert_view::finish_insert(&mut v);
    }

    /// `Group::action_halt(mask)` (§7): the steps of [`crate::halt_view`]
    /// over this simulation.
    pub fn group_action_halt(&mut self, g: &Group, mask: i32) {
        let mut view = crate::halt_view::SimHalt { sim: self, g };
        crate::halt_view::action_halt_steps(&mut view, mask);
    }

    /// `Group::action_stance(s)` (§8). A negative `s` cycles: −1 forward,
    /// −2 back.
    pub fn group_action_stance(&mut self, g: &Group, s: i32) {
        if !self.group_is_on_map(g) {
            return;
        }
        let ty = self.group_stance_type(g);
        let n = ty.options();
        if n == 0 {
            return;
        }
        let cur = self.group_stance_option(g, ty);
        let s = if s >= 0 {
            s
        } else if s == -2 {
            if cur - 1 < 0 { n - 1 } else { cur - 1 }
        } else {
            (cur + 1) % n
        };
        for &u in &g.list {
            // `0070d440:95`: vslot `+0xc0` is `is_plane`, so a plane never
            // receives a stance. (The building arm's gate is `is_build` —
            // 1 for a Build and 0 for a Wall, so a wall never receives one
            // either; the simulation has no buildings group to reach it.)
            if !self.units[u].alive() || self.is_plane(u) || self.unit_stance_type(u) != ty {
                continue;
            }
            // `0070d440:117`: `+0xb1 = option`, one byte for **every**
            // panel, the combat stance included. This crate keeps the
            // combat half again as a [`Stance`] below, and wrote only that
            // half until item 530 — so a hoplite the army set to
            // `action_stance(1)` read `stance 0` against the dump's 1.
            self.units[u].stance = s.clamp(0, 255) as u8;
            // `0070d440:100`: `SubObjectData::flags |= 0x10` on every
            // member set, whatever the panel — the bit `Unit::think`'s
            // idle gate reads, so the member thinks on its next frame
            // (item 3962, `docs/AI.md` §406).
            self.units[u].cant_reach = true;
            if ty != StanceType::Combat {
                // The worker/caster/packer stances live elsewhere in the
                // simulation (`Unit::stance`); only the write is modelled.
                continue;
            }
            let Some(&st) = COMBAT_STANCES.get(s as usize) else {
                // Unreachable for a combat stance: it has six options.
                self.clear_orders(u);
                continue;
            };
            self.units[u].combat.stance = st;
            match s {
                0 | 3 | 4 => self.units[u].combat.mandatory = false,
                // 1, 2, 5 kill the current order — for a **human** leader
                // only (`leaders.list[who] & 4`, `docs/GROUPS.md` §8),
                // when it lacks the action bit and the action order does
                // too: `Unit::repath`, then `kill_current_order(0)`. No
                // army's leader is human; a player's stance button is
                // (item 2165, `docs/GOLDEN.md` §79): run835's `0/7` drops
                // the attack it found on the fishing boat on 681.
                1 | 2 | 5 => {
                    let human = self.nation.get(g.who as usize).is_some_and(|n| n.human);
                    if human
                        && self
                            .current_order(u)
                            .is_some_and(|o| !o.has(crate::orders::flag::ACTION))
                        && self.update_action(u).is_none_or(|a| {
                            !self.units[u].orders[a].has(crate::orders::flag::ACTION)
                        })
                    {
                        self.repath(u);
                        self.kill_current_order(u);
                    }
                }
                _ => {}
            }
        }
    }

    /// `Group::action_move_to` → `action_move_near` (§6). `set_angle` and
    /// `disembark` are the caller's; `form`/`width` are −1 on every army
    /// call and the formation is a seam, so every member is given the
    /// group's own destination.
    #[allow(clippy::too_many_arguments)]
    pub fn group_action_move_to(
        &mut self,
        g: &Group,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    ) {
        if !self.group_is_on_map(g) || g.list.is_empty() {
            return;
        }

        // §17 — **a group's `QUEUE_FIRST` is not a member's.** The arm at
        // `00704bfe` does not hand `QUEUE_FIRST` down to
        // `add_move_facing_order` at all: it copies the **leader's**
        // action-flagged orders aside (`set_up_insert`), `action_halt`s
        // every member, calls itself with `QUEUE_NEW`, and re-issues the
        // copies as group actions at `QUEUE_LAST` (`finish_insert`).
        //
        // So the walk a member was on is *dropped*, not stacked behind —
        // which is what run39's frame 825 shows: the scout's plain
        // `EXPLORE_TO` carries no action bit, so nothing is saved, and its
        // order list comes out of `Unit::get_goody_box` holding the new
        // explore alone (`docs/GOODY.md` §7.3).
        //
        // **And the re-issue reads the group as the move left it** (item
        // 3517): the original's `this` is the army's own record, and the
        // move's `Form::categorize` has just `Group::sort`ed it — a squad
        // listed by its follower killed and re-added whole at the end — so
        // `finish_insert`'s `action_guard` lays the escort out over the
        // sorted list. Great Sahara's Americans, frame 4478: army 1/0 listed
        // `1/21` and `1/24` (the tails of the squads `1/19` and `1/22`)
        // second and third, and the escort on `1/99` took its slots in the
        // stale order — every member's `GUARDORDER` `dx`/`dy` parted.
        if queue == QueuePos::First {
            let stack = Self::seat_of_group(g).is_some_and(|seat| *self.seat_list(seat) != g.list);
            let insert = self.group_set_up_insert(g);
            self.group_action_halt(g, 0);
            self.group_action_move_to(g, to, QueuePos::New, set_angle, angle, kind, action);
            let now = match Self::seat_of_group(g) {
                Some(seat) if !stack => Group {
                    list: self.seat_list(seat).clone(),
                    ..g.clone()
                },
                _ => g.clone(),
            };
            self.group_finish_insert(&now, insert);
            return;
        }

        let to = self.restrict_pos(to);
        // A value that names a seat but holds another list is a group on
        // the stack carrying the seat's record — `action_siege_attack_to`'s
        // sub-group (§26, §27). Read before the clear, which can move the
        // seat's list.
        let stack = Self::seat_of_group(g).is_some_and(|seat| *self.seat_list(seat) != g.list);

        // §6.3: the formation angle. With `set_angle` the caller's stands;
        // without it, the direction from the group's own location to the
        // destination.
        //
        // **And which location depends on the queue position**
        // (`00704990:361`–`364`): a `QUEUE_LAST` move asks
        // [`Self::group_loc_to`] — where the leader will *end up* once the
        // orders already on its list are done — and every other position
        // asks [`Self::group_loc`], where it stands now. The two come apart
        // exactly when something is queued ahead, which is the case §12's
        // probe makes on every game: it queues an attack on a farm at
        // `QUEUE_NEW` and the walk home behind it at `QUEUE_LAST`, to the
        // leader's **own** position — so read from where the leader stands
        // the delta is zero and the formation has no bearing at all, and
        // read from the farm it is the length of the map (§17).
        let from = if queue == QueuePos::Last {
            self.group_loc_to(g)
        } else {
            self.group_loc(g)
        };
        let angle = if set_angle {
            angle
        } else {
            match from {
                Some(p) if p != to => find_angle(to.x - p.x, to.y - p.y),
                // The zero-delta arm, and it has two halves
                // (`707e3d`–`707e6d`): with the location different from the
                // group's own `(ox, oy)` — which is every group that has not
                // already been ordered to this very point — the angle is the
                // **leader's own heading** less its packed slot byte, the
                // same quantity [`Self::group_leader_faces_away`] measures.
                // Only a group standing where it was last sent falls through
                // to the record's `o_angle`.
                Some(p) if p != self.group_o(g) => self
                    .group_leader_bearing(g)
                    .unwrap_or_else(|| self.group_o_angle(g)),
                _ => self.group_o_angle(g),
            }
        };

        // §6.3: the formation index. Every simulation caller passes −1,
        // which takes the group's own (`get_form`), clamped up to 0 — so
        // an army's first move settles on form 0 and writes it to every
        // member, which is what `get_form` reads back next time.
        let form = self.group_get_form(g).max(0);
        // …and the same for the width: `param_10 == −1` takes
        // `get_form_mod_option`, which is 50 for an army and stays 50.
        let width = self.group_form_mod_option(g);
        // `action_move_near` writes `form` to the record here, ahead of
        // `compute_form` (`00704990:354` against `:460`), so a sort that
        // kills the list whole and `Group::clear`s the record leaves it −1
        // (run1555's slot 64 on 765, item 4000); `ox`/`oy` and `o_angle`
        // come after it ([`Self::record_move`]).
        if let Some(st) = self.gstate_mut(g) {
            st.form = form;
        }

        // §6.5: the AI branch. `hurry` is the army's, so a group with no
        // army never takes it — and "hurrying" and "found a city" are two
        // different facts, because the clear loop and the order loop do not
        // gate on the same one.
        let ai = self.ai_army_group(g);
        let hurry = ai && self.armies[g.who as usize].list[g.army.unwrap_or(0)].hurry != 0;
        let hurry_city = if hurry {
            self.city_near(g.who, to)
        } else {
            None
        };

        // What the move does to each member, decided **before** the clear.
        // The original decides the same thing twice — once in the clear
        // loop at `70524f` and once in the order loop at `7054c7` — and the
        // second reading sees state the first has already wiped, which is
        // why a shooting siege unit that was cleared falls through and
        // takes the move like everyone else.
        let decide = |this: &Sim, u: usize| -> Member {
            if !this.group_member_orderable(u) {
                return Member::Skip;
            }
            if !ai {
                return Member::Move;
            }
            let shooting_siege = this.is_siege_unit(u) && this.order_type(u) == index::ATTACK;
            match hurry_city {
                Some(c)
                    if this.is_siege_unit(u) || this.is_supply_unit(u) || this.is_hero_unit(u) =>
                {
                    Member::Stable(c)
                }
                // The `QUEUE_NEW` clear at `70524f` gates on `hurry`
                // **alone**; the order loop at `7054c7` on `hurry && a
                // city was found`. So a siege unit that is already
                // shooting is left entirely alone by a move that is not
                // hurrying. A hurrying army that found no friendly city
                // clears its orders — and the order loop then re-reads
                // `order_type()`, which `Unit::close_orders` has left at
                // `NONE`, so the unit falls through and takes the move
                // like every other member. Only `QUEUE_NEW` clears; any
                // other queue position leaves it shooting and skipped.
                _ if shooting_siege && !(hurry && queue == QueuePos::New) => Member::Skip,
                _ => Member::Move,
            }
        };
        let plan: Vec<Member> = g.list.iter().map(|&u| decide(self, u)).collect();

        // §6.6's `QUEUE_NEW` clear, and **it runs before the layout**: the
        // loop at `70524f` clears every member's orders, and its
        // `Unit::clear_orders` at `70538d` is ahead of `compute_form` at
        // `7053ec` in the listing. That ordering is load-bearing rather
        // than incidental — the leader's dying move hands its own mirror
        // back to the group on the way out (§6.3), so the flag
        // `compute_form` reads a line later is the *last layout's* answer
        // and not whatever the march left behind.
        //
        // **A packer mid-pack keeps its pack** (`70524f`–`7053a0`, item
        // 3336): a member whose type packs (`+0x2b8 & 4`) and whose head is
        // a pack cast (`is_packing`) has that order lifted off the list,
        // the rest cleared, and the cast put back
        // ([`Self::clear_keeping_pack`]).
        if queue == QueuePos::New {
            for (&u, p) in g.list.iter().zip(&plan) {
                if *p == Member::Move && !self.clear_keeping_pack(u) {
                    self.clear_orders(u);
                }
            }
        }

        // §6.4: the slot table. `facing` is the group's own mirror flag, and
        // `compute_form` **toggles it around the call** when the leader is
        // pointing more than 90° from the formation's bearing, toggling it
        // back after — so the mirror the layout uses is not the `facing` the
        // record keeps. run31 shows the two coming apart: two of its moves
        // carry `facing 1` and lay out unmirrored, the third carries
        // `facing 1` and mirrors (§6.3, §12.3).
        //
        // **The leader is found before the sort, the flag read after it**
        // (`compute_form@00707c80:37` finds the leader and its slot;
        // `Form::categorize`'s `Group::sort` runs at `:72`; the mirror test
        // reads `GroupData::facing` at `:103`). A sort that kills a
        // follower-first squad whole empties the list, and `Group::kill`'s
        // `Group::clear(-1)` zeroes the flag before it is read (item 4000,
        // `docs/GOLDEN.md` §190: chapter 128's `1/6..1/8` on 764).
        let away = self.group_leader_faces_away(g, angle);
        // `Form::compute` → `Form::categorize`, whose first statement is
        // `Group::sort@00708090` (§4.1, §23) — after the clear, before the
        // mirror test, the layout and the order loop, which
        // therefore walk the **sorted** list. A member the sort brought in
        // was not in the list the clear walked, so it is decided now, as
        // the order loop's own second reading would.
        //
        // **A stack group sorts its own list** (§27): the siege arm's
        // sub-group carries the parent's `army` so that it reads and writes
        // a record, but `Group::sort` runs on the stack copy and the army's
        // list is never touched. Sorting the seat here handed a one-anchor
        // sub-group the whole army whenever the army's list held a squad
        // follower ahead of its captain — East Indies 17402, army 1's
        // `[60, 69]`.
        let resorted = match Self::seat_of_group(g) {
            Some(_) if stack => {
                let mut ng = g.clone();
                self.stack_sort(&mut ng);
                (ng.list != g.list).then_some(ng)
            }
            Some(seat) => {
                let before = self.seat_list(seat).clone();
                self.seat_sort(seat);
                (*self.seat_list(seat) != before).then(|| self.seat_group(seat))
            }
            None => None,
        };
        let (g, plan) = match &resorted {
            Some(ng) => {
                let p: Vec<Member> = ng
                    .list
                    .iter()
                    .map(|&u| match g.list.iter().position(|&m| m == u) {
                        Some(i) => plan[i],
                        None => decide(self, u),
                    })
                    .collect();
                (ng, p)
            }
            None => (g, plan),
        };
        let facing = self.group_facing(g);
        let reverse = facing != away;
        let mut slots = self.form_compute(
            g,
            to,
            angle,
            form,
            width,
            reverse,
            false,
            &self.group_angles(g),
        );
        // §6.3's **second** test, and it is not a mirror: with the angle
        // supplied — every army call — `compute_form` measures its own
        // `find_angle(dest − group_loc)` against it, and when the two are
        // `90°` or more apart the tail negates every member's offsets on
        // both the `Form` and the group, leaving the destinations alone.
        // A group laid out this way walks to the slots it was given and
        // carries the opposite offsets, so `update_positions` puts every
        // follower on the far side of its leader.
        //
        // SEAM: the scenario-editor bit (`semaphore[1] & 8`) forces this
        // false with `facing`; nothing in this simulation sets it.
        if set_angle
            && let Some(p) = from
            && reversing(Angle(
                find_angle(to.x - p.x, to.y - p.y).0.wrapping_sub(angle.0),
            ))
        {
            slots.flipped = true;
            for off in &mut slots.off {
                *off = (-off.0, -off.1);
            }
        }

        for (i, &u) in g.list.iter().enumerate() {
            match plan[i] {
                Member::Skip => continue,
                Member::Stable(c) => {
                    self.stable_in_city(u, c, angle);
                    continue;
                }
                Member::Move => {}
            }
            // §6.6 step 1: the formation index and its width twin, on
            // every member but the four citizen/scholar ids.
            // The exemption guards `form` (`+0xaa`) alone: the width's
            // store at `00705749` (`+0xab`) is outside the test, so a
            // citizen carries the group's `form_mod` like every other
            // member (run157's `1/1` on 990, §24).
            if !self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::CITIZEN))
            {
                self.units[u].form = form as i8;
            }
            self.units[u].form_width = width as i8;
            // §6.6 step 2, and it only means anything now that §6.7
            // plans: a `QUEUE_LAST` move turns the member's existing stack
            // over before the group's new legs are pushed on top, so that
            // the single invert at the end of §6.7 leaves the old segment
            // the right way up underneath the new one.
            if queue == QueuePos::Last {
                self.units[u].path.reverse();
            }
            // §6.6 step 3: the member's **own** slot, clamped into the
            // world — not the group's destination.
            //
            // **The clamp is written back into the table**
            // (`action_move_near@00704990:521`–`536`: the four stores are
            // to `param_9 + 0x514`/`+0x714` themselves), so §6.7's plan
            // — the leader's base, a member's translation and
            // `group_plan_each`'s own goal — reads the clamped slot. Greek
            // East Indies' Heavy Fire Raft `1/70` on 5626: its slot sat
            // at x 46259 on a map 46080 wide, and the raw one went on its
            // stack, so `find_path` pulled back off the map's edge onto
            // its own square where theirs, from (46079, 38109), stopped
            // at (45683, 37689) (item 3711, `docs/AI.md` §363.1).
            let mut slot = self.restrict_pos(slots.to[i]);
            slots.to[i] = slot;
            // §6.6 step 4's **different-region** arm (`705a2a`..`705c9b`):
            // a member past the first whose slot's `get_tregion` — the
            // coastal-water `region2` arm, [`crate::world::World::tregion_alt`]
            // — is not slot 0's is re-placed by `find_nearby_spot` around
            // **slot 0's** point (`min 0, max −1`, step 0, bias
            // `0x55555555`, `FILTER_NOT_ME`), and failing that takes slot
            // 0's point itself. French East Indies' Hoplite `1/81` on
            // run630's 10766: its slot's tile is coastal water (region2 0)
            // and slot 0's is region 12, and its order aims at the group's
            // own point there (item 1458, `docs/GROUPS.md` §37).
            //
            // **The sweep asks a barge where slot 0 will not hold the
            // member** (`705c0f`–`705c9b`, item 3522): when `invalid_loc(1,
            // 1, 0, 0, 0)` refuses slot 0's tile to a land member and its
            // leader `can_transport`, the type swept is the leader's
            // `current_upgrade(TRANSPORTBARGE)`, so the ring's first
            // candidate — slot 0's own ocean point — stands. Russian East
            // Indies' army 1 on 4090: slot 0 is the Spy's, in the sea off
            // (8010, 2873), and a T34's own ring reached the shore at
            // (8664, 1368) where the original's whole army aims at
            // (8568, 984) (run1168's block 4091, `docs/AI.md` §304).
            // SEAM: a loaded boat's passenger type (a sea member with
            // `num_inside`); and the same-region `invalid_loc` arm, which a
            // computer's unit (`unit_masks & 0x40000`) skips.
            if i > 0 {
                let s0 = self.restrict_pos(slots.to[0]);
                if self.world.tregion_alt(slot.tile()) != self.world.tregion_alt(s0.tile()) {
                    // The type's sweep with `not_o`/`not_who` −1: nothing
                    // is exempt, the member itself included.
                    slot = self
                        .region_spot_type(u, s0)
                        .and_then(|t| {
                            self.find_nearby_spot_type(t, s0, 0, -1, 0, Angle(0x5555_5555))
                        })
                        .unwrap_or(s0);
                    // Written back into the table (`param_9 + 0x514`,
                    // `+0x714`), which §6.7's translation reads next.
                    slots.to[i] = slot;
                    // **The member names this group afterwards**: the arm
                    // writes `+0x80 = −1` before the search and
                    // `+0x80 = this->id` after it (`705a2a..705c9b`,
                    // decompile 653-655 and 716-718; twenty-fourth pass,
                    // group 15; A9 row 34), so a stale back-pointer is
                    // re-pointed. A group with no seat has no id to write.
                    self.repoint_member(g, u);
                }
            }
            // §6.6 step 6: the order's angle is the formation's, **plus**
            // this slot's packed byte — an addition of a signed byte
            // shifted into the top of the word (`705f42`–`705f4d`), where
            // `compute_form` subtracts the same product back off the
            // leader's heading. And its `facing` is the mirror this layout
            // used, which is what the order hands back when it dies.
            let order_angle = Angle(angle.0.wrapping_add(i32::from(slots.angles[i]) << 24));
            // **The order loop's own keep** (`705e5d`–`705ed8`, item 3336):
            // under `QUEUE_NEW` a packer still packing keeps its cast at
            // the head once more, and its move is added `QUEUE_LAST`,
            // beneath it — Great Sahara's Cannon `1/125` on tick 20218
            // (run967g's block 20219: the `GROUPATTACKTOORDER` under the
            // `CASTORDER` `spell 651`, `spell_time` running on), so the
            // escort its army posts round it that tick reads it standing.
            let queue = if queue == QueuePos::New && self.clear_keeping_pack(u) {
                QueuePos::Last
            } else {
                queue
            };
            // §6.6 step 6's own gate (`705f00`–`705f61`): a **land
            // formation of two or more** walking a `MOVE_TO` or an
            // `ATTACK_TO` gets a `GroupMoveOrder`, and everything else a
            // plain move. The exemptions are modern infantry, a
            // `role & 0x10` type that is not AI-driven (a scout on land, a
            // bark at sea), a group of fewer than two, `unit_masks & 4`
            // ([`sim::Unit::in_danger`], carried since item 465 because
            // it is what the army line below was standing in for), a sea
            // type, and form 9.
            //
            // A group with **no seat** is exempt too, and that is this
            // crate's own line rather than the original's: a `Group` here
            // is a value a caller builds, where the original's is always
            // a pool slot, so the stand-in for `this->group == -1` —
            // which `do_group_move` ungroups on its first line — is "the
            // record has somewhere to live". A player's selection is
            // still N independent moves (`docs/ORDERS.md` §8.4); an
            // army's group and a **pushed** one are not.
            //
            // ~~`g.army.is_some()`~~ until item 465, which cost Great
            // Lakes' probe its `GroupMoveOrder` for four months: the six
            // raiders it pushes out of the army on 8186 are `group 65` in
            // the dump and held no group at all here (`docs/ORDERS.md`
            // §16).
            let grouped = self.gstate(g).is_some()
                && matches!(kind, MoveKind::MoveTo | MoveKind::AttackTo)
                && !self.is_modern_infantry(u)
                && !(self.units[u]
                    .ty
                    .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::SCOUT))
                    && !self.ai_driven(g.who))
                && g.num() >= 2
                && !self.units[u].in_danger
                && self.group_domain(u) != Domain::Sea
                && form != 9;
            if let (true, Some(leader)) = (grouped, self.group_find_leader(g)) {
                let gm = crate::orders::GroupMove {
                    leader,
                    id: group_move_id(self.group_id(g), self.frame, self.group_order_num(g)),
                    form_id: i,
                    group_angle: order_angle,
                    in_group: false,
                    orig: to,
                };
                self.add_move_facing_order_grouped(
                    u,
                    slot,
                    kind,
                    queue,
                    action,
                    order_angle,
                    Some(reverse),
                    true,
                    Some(gm),
                    Some(to),
                );
            } else {
                // `705f61`–`705f9b`: the plain arm hands
                // `add_move_facing_order` the click as its `orig`
                // (`param_1`, `param_2`), which `finish_insert` replays
                // to (`docs/GOLDEN.md` §24).
                self.add_move_facing_order_grouped(
                    u,
                    slot,
                    kind,
                    queue,
                    action,
                    order_angle,
                    Some(reverse),
                    true,
                    None,
                    Some(to),
                );
            }
            // **Either arm takes the builder's mark off** (`705fe3`: `andl
            // $0xfffffbff, 0x68(%eax)`, after the member's order): a citizen
            // an army or a raid moves off its site is no longer "has been a
            // builder", and its next idle think does not search for a
            // build. run1370's `1/95` leaves its `BUILDORDER` for group 80's
            // move between 4001 and 4101 and its `unit_masks` loses 0x400;
            // ours kept it to 4841 and sent it to the unbuilt `1/2052`
            // (`docs/AI.md` §358.4, item 3710).
            self.units[u].was_builder = false;
        }
        // §6.7: the group's own path, planned once off the leader's slot
        // and handed to every member translated. It sits exactly here in
        // the original — after the order loop, before `order_num` — and it
        // is why the orders above are born `PATHED`.
        //
        // `from` is `get_loc`'s answer and is `None` only for a group with
        // no leader at all, which is the same condition that sends the plan
        // down its own no-leader arm; the destination stands in so that the
        // arm has a start to fall back on.
        self.group_plan_path(g, &slots, &plan, kind, form, from.unwrap_or(to));
        self.bump_order_num(g);
        // The order matters and run31 settles it: `Form::compute`'s tail
        // writes `o_angle` and `o_dist` from the leader's slot, and then
        // `action_move_near` writes `o_angle` again — with the **formation
        // angle** — for a `QUEUE_NEW`/`QUEUE_LAST` move (`00704990:464`).
        // So the record's `o_angle` is the formation's bearing and its
        // `o_dist` is the leader's offset within the block, which is how
        // run31 measures §6.4's displacement.
        self.record_form(g, &slots, to);
        self.record_move(g, to, angle, queue);
    }

    /// The type `action_move_near`'s different-region arm sweeps for member
    /// `u` round slot 0's point `s0` (`705c0f`–`705c9b`): the member's own,
    /// unless slot 0's tile is `invalid_loc(1, 1, 0, 0, 0)` for it and its
    /// type is of the land domain (`+0x218 == 0`) under a leader that
    /// `can_transport` — then the leader's `current_upgrade(TRANSPORTBARGE)`.
    pub(crate) fn region_spot_type(&self, u: usize, s0: Pos) -> Option<usize> {
        let own = self.units[u].ty?;
        let who = self.units[u].owner;
        if self.unit_types[own].combat.domain == Domain::Land
            && self.invalid_loc(u, s0.tile(), true, true, false, false, false) != 0
            && self.transport_level(who) != crate::transport::TransportType::None
        {
            return self.barge_type(who).or(Some(own));
        }
        Some(own)
    }

    /// `Group::action_move_near@00704990`'s keep (`:410`–`440`, `:727`–
    /// `752`, item 3336): a member whose type packs and whose head order is
    /// a pack cast (`UnitData::is_packing`) has that order taken off the
    /// list (`LinkListBase::remove_current` — not killed), the rest
    /// cleared (`Unit::clear_orders`), and the cast added back, then
    /// `update_gpiece`. False, and nothing done, for any other member.
    pub(crate) fn clear_keeping_pack(&mut self, u: usize) -> bool {
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.packs);
        if !packs || !self.is_packing(u) {
            return false;
        }
        let Some(cast) = self.units[u].orders.pop_front() else {
            return false;
        };
        self.clear_orders(u);
        self.units[u].orders.push_front(cast);
        self.update_gpiece(u);
        true
    }

    /// `Group::action_siege_attack_to(x, y, ·, ·, angle)` (§9).
    pub fn group_action_siege_attack_to(&mut self, g: &Group, to: Pos, angle: Angle) {
        if g.list.is_empty() {
            return;
        }
        // The sub-group: every siege member, else the first supply wagon,
        // else the first hero.
        let mut sub = Group {
            who: g.who,
            army: g.army,
            pushed: g.pushed,
            list: Vec::new(),
        };
        for &u in &g.list {
            if self.units[u].alive() && self.units[u].on_map && self.is_siege_unit(u) {
                self.group_add(&mut sub, u);
            }
        }
        // A **human** leader skips the widening and the anchor scoring and
        // takes the sub-group's own leader (`0070dabd`). No army's leader
        // is human, so this arm is the player's siege-attack command.
        if self.nation.get(g.who as usize).is_some_and(|n| n.human) {
            // The anchor is only read by `action_guard`, which is the seam;
            // whether the sub-group has a leader is what chooses the arm.
            let escort = self.group_find_leader(&sub).is_some();
            let it = if escort { &sub } else { g };
            self.group_action_move_to(it, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        }
        if sub.list.is_empty()
            && let Some(&u) = g
                .list
                .iter()
                .find(|&&u| self.units[u].alive() && self.units[u].on_map && self.is_supply_unit(u))
        {
            self.group_add(&mut sub, u);
        }
        if sub.list.is_empty()
            && let Some(&u) = g
                .list
                .iter()
                .find(|&&u| self.units[u].alive() && self.units[u].on_map && self.is_hero_unit(u))
        {
            self.group_add(&mut sub, u);
        }

        // The anchor: the sub-group member with the smallest total
        // Manhattan distance to the whole group, in 1024-unit steps.
        let anchor = self.siege_anchor(g, &sub);
        let Some(anchor) = anchor else {
            self.group_action_move_to(g, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        };
        // The area test: the anchor and the destination must share a
        // region (the sim's stand-in for §9's area id).
        let same =
            self.world.tregion(self.units[anchor].pos.tile()) == self.world.tregion(to.tile());
        if !same {
            self.group_action_move_to(g, to, QueuePos::New, true, angle, MoveKind::AttackTo, true);
            return;
        }
        // **The sub-group's record is its own** (§26): `Group::clear(-1)` on
        // the stack, then the parent's `id` and `army` and `stamp = 0`
        // (`0070d830:60–66`). Its layout reads that record's `facing`, 0,
        // and writes that record, which the original discards on return.
        // This crate keys a group's record by its seat, so the parent's is
        // set aside while the sub-group moves and put back after it.
        let stack = GroupState {
            stamp: 0,
            pool: self.gstate(&sub).and_then(|st| st.pool),
            ..GroupState::default()
        };
        let parent = self.gstate_mut(&sub).map(|st| std::mem::replace(st, stack));
        // The parent is set aside where a member's own writes can find it:
        // the clear inside the layout kills each member's head order, and a
        // dying move's hand-back goes to the unit's real group (`docs/AI.md`
        // §212) — run820's Supply Wagon `1/205`, the army's leader, wrote its
        // 1 into the stack record on 4082 and the layout read it.
        self.set_aside = parent.map(|p| (sub.who, sub.army, sub.pushed, p));
        self.group_action_move_to(
            &sub,
            to,
            QueuePos::New,
            true,
            angle,
            MoveKind::AttackTo,
            true,
        );
        let parent = self.set_aside.take().map(|(.., p)| p);
        // The sub-group's own list as its layout walked it: `Group::sort`
        // runs on the stack record in place, and the record's tables are
        // indexed by the sorted list.
        let mut walked = sub.clone();
        self.stack_sort(&mut walked);
        let mine = match (parent, self.gstate_mut(&sub)) {
            (Some(p), Some(st)) => Some(std::mem::replace(st, p)),
            _ => None,
        };
        // **The copy-back** (`0070de..0070dfa0`): for each member of the
        // parent that the sub-group holds, the sub-group's slot — `off`,
        // `curr`, `angles` at the member's index in the sub-group — is
        // written into the parent's table **at the member's index in the
        // parent**, and `Unit::replace_form_id@005fd420` re-points every
        // group order the member holds at that index. So a siege unit past
        // the parent's `form_num` carries its slot in the parent's tail,
        // where `refresh_group_order` reads it (item 1493, `docs/AI.md`
        // §128: 1/141's `off` (23, 0) at index 62).
        if let Some(mine) = mine {
            for (k, &m) in g.list.iter().enumerate() {
                if !(self.units[m].alive() && self.units[m].on_map) {
                    continue;
                }
                let Some(j) = walked.list.iter().position(|&x| x == m) else {
                    continue;
                };
                if let Some(st) = self.gstate_mut(&sub) {
                    // The arrays are the member count long once a member
                    // has been added through the seat; a record built
                    // before that is brought up to the list's length.
                    let n = g.list.len();
                    if st.off.len() < n {
                        st.off.resize(n, (0, 0));
                    }
                    if st.curr.len() < n {
                        st.curr.resize(n, Pos::new(0, 0));
                    }
                    if st.angles.len() < n {
                        st.angles.resize(n, 0);
                    }
                    if let Some(&o) = mine.off.get(j) {
                        st.off[k] = o;
                    }
                    if let Some(&o) = mine.curr.get(j) {
                        st.curr[k] = o;
                    }
                    if let Some(&o) = mine.angles.get(j) {
                        st.angles[k] = o;
                    }
                }
                for o in &mut self.units[m].orders {
                    if let Some(gm) = o.move_mut().and_then(|mv| mv.group.as_mut()) {
                        gm.form_id = k;
                    }
                }
            }
        }
        // `action_guard(anchor, who, QUEUE_NEW, 1)` on the parent: the
        // rest escort the anchor while it walks in (`docs/ORDERS.md` §24).
        // Golden chapter four's word, 1277: an AI army with a Supply Wagon
        // and no siege, whose three hoplites take a `GUARDORDER` on the
        // wagon on the army's own tick.
        self.group_action_guard(g, anchor, QueuePos::New, true);
    }

    /// `Group::action_guard(o, who, queue, no_siege)@006fcd30`
    /// (`docs/ORDERS.md` §24), for a **unit** target.
    ///
    /// `action_begin`, and the group's `form` goes to −1. The target is
    /// replaced by its squad's **captain** (`get_captain`, vslot `+0xe4`;
    /// the building test at vslot `+0x18` answers 1 for every unit). A
    /// stack group then gathers the members that can escort it: active, on
    /// the map, not a plane, of the target's domain — or a helicopter, or
    /// a sea transport beside a land target — not siege when `no_siege`,
    /// and not the target itself. On patch versions above 3 a target that
    /// is itself guarding one of the members has its orders cleared, so
    /// two units never guard each other. The escort is laid out by
    /// `compute_form` **at the target**, on bearing 0, width `0x32`, with
    /// the guard flag set — which reserves an artillery rank for the target
    /// and keeps the reverse negation off — and each member is given its
    /// slot's `off_x/off_y` as the guard offset: rewritten on a `GUARD`
    /// it already holds on this target, else a fresh `add_guard_order`.
    ///
    /// SEAM, none reached by a capture on file: `QUEUE_FIRST`'s insert
    /// dance (taken as `QUEUE_NEW`); the building-target arm (every member
    /// `add_guard_order(o, who, −1, −1)`; this returns instead); the human
    /// leader's sweep over the player's other units already guarding the
    /// target; and `Group::sort` inside `Form::compute`, a no-op on a list
    /// [`Self::group_add`] has just built captain-first.
    pub fn group_action_guard(
        &mut self,
        g: &Group,
        target: usize,
        queue: QueuePos,
        no_siege: bool,
    ) {
        if g.list.is_empty() {
            return;
        }
        if !(self.units[target].alive() && self.units[target].on_map) {
            return;
        }
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let whom = self.units[target].owner;
        if !self.is_ally(g.who, whom) || self.is_plane(target) {
            return;
        }
        let cap = self.top_captain(target);
        let dom = self.group_domain(cap);
        let mut local = Group {
            who: g.who,
            army: None,
            pushed: None,
            list: Vec::new(),
        };
        for &m in &g.list {
            if !(self.units[m].alive() && self.units[m].on_map) || self.is_plane(m) {
                continue;
            }
            let flag = |f: u32| {
                self.units[m]
                    .ty
                    .is_some_and(|t| self.unit_types[t].cols.flag(f))
            };
            let admitted = flag(uflags::HELICOPTER)
                || self.group_domain(m) == dom
                || (flag(uflags::TRANSPORT) && dom == Domain::Land);
            if !admitted || (no_siege && self.is_siege_unit(m)) {
                continue;
            }
            if m == cap && g.who == whom {
                continue;
            }
            self.group_add(&mut local, m);
        }
        // `Game::get_patch_version() > 3`: break a guard cycle.
        if self.order_type(cap) != index::NONE
            && g.list
                .iter()
                .any(|&m| self.update_guard_order(cap, m).is_some())
        {
            self.clear_orders(cap);
        }
        // `get_num_cap`: the captains still standing.
        if !local
            .list
            .iter()
            .any(|&m| self.units[m].alive() && self.units[m].captain)
        {
            return;
        }
        if self.group_find_leader(&local).is_none() {
            return;
        }
        // `compute_form(local, target, leader_flags >> 2 & 1, 0x32, ·, 1,
        // &0, loc, 1)`: formation 0 for an AI, the caller's angle of zero
        // standing, and the guard flag. The stack group's `facing` is the
        // `clear`ed zero, so the mirror is the leader toggle alone; the
        // reverse negation is skipped because the guard flag is set.
        let human = self.nation.get(g.who as usize).is_some_and(|n| n.human);
        let form = i32::from(human);
        let tp = self.units[cap].pos;
        let reverse = self.group_leader_faces_away(&local, Angle(0));
        let slots = self.form_compute(&local, tp, Angle(0), form, 0x32, reverse, true, &[]);
        for (k, &m) in local.list.iter().enumerate() {
            if m == cap || self.top_captain(m) == cap {
                continue;
            }
            let (dx, dy) = slots.off.get(k).copied().unwrap_or((0, 0));
            match self.update_guard_order(m, cap) {
                Some(i) => {
                    if let Body::Guard(x) = &mut self.units[m].orders[i].body {
                        x.dx = dx;
                        x.dy = dy;
                    }
                }
                None => self.add_guard_order(m, cap, dx, dy, queue),
            }
        }
    }

    /// `Group::action_follow(ox, whom, queue)@006fd510` (`docs/ORDERS.md`
    /// §28), reached from `CommandPackage::process_follow@009479c0` with
    /// the command's leader and queue position.
    ///
    /// `action_begin`, then a group of buildings returns. The group's
    /// `form` goes to −1. The leader must be active, on the map and not a
    /// plane (vslots `+0x8`, `+0xbc`, `+0xc0`). Every member that is the
    /// same, and is not the leader's own captain of the same player
    /// (`get_captain`, vslot `+0xe4`), gets one
    /// [`Sim::add_follow_order`]. **There is no `is_ally` test**: a player
    /// may follow anyone's unit.
    ///
    /// SEAM, none reached by a capture on file: `QUEUE_FIRST`'s insert
    /// dance (`set_up_insert`, `action_halt`, the follow at `QUEUE_NEW`,
    /// `finish_insert`), taken here as `QUEUE_NEW`; the scenario's
    /// `ignore_orders` sweep; a buildings group, which the command's
    /// `process_group` cannot build from units.
    pub fn group_action_follow(&mut self, g: &Group, target: usize, queue: QueuePos) {
        let queue = if queue == QueuePos::First {
            QueuePos::New
        } else {
            queue
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        if !self.group_member_orderable(target) {
            return;
        }
        let whom = self.units[target].owner;
        let cap = self.top_captain(target);
        for &m in &g.list {
            if !self.group_member_orderable(m) || (m == cap && g.who == whom) {
                continue;
            }
            self.add_follow_order(m, target, queue);
        }
    }

    /// `Group::action_garrison(ox, whom, queue, search)@00700490`
    /// (`docs/ORDERS.md` §29), reached from
    /// `CommandPackage::process_garrison@00948760` with the command's
    /// building and queue position, and `search` 0.
    ///
    /// `action_begin`; the group's `form` goes to −1. The building must be
    /// the group's player's or a mutual ally's, alive, with a garrison
    /// limit, and not an unassimilated city. Then every member that is
    /// active, on the map and not a plane gets one
    /// [`Sim::add_garrison_order`] with the action bit — **if its type
    /// `can_garrison` the building's**; a member that cannot gets nothing.
    /// A member whose action is already a GARRISON is `repath`ed and has
    /// its head killed first.
    ///
    /// The walk is [`crate::garrison_view::action_garrison_steps`], swept
    /// against the original (item 3376); the SEAMs are
    /// [`crate::garrison_view::SimGarrison`]'s, none reached by a capture on
    /// file.
    pub fn group_action_garrison(&mut self, g: &Group, b: usize, queue: QueuePos) {
        self.group_action_garrison_search(g, b, queue, false);
    }

    /// [`Self::group_action_garrison`] with its fifth argument, `search`,
    /// which the City's alarm passes as 1 (item 1167, `docs/GOLDEN.md`
    /// §49): each member is sent to `Unit::find_garrison_build@00605040`'s
    /// building of the city when it finds one, the building `b` otherwise,
    /// and the order carries `search`. At `QUEUE_NEW` a **worker**
    /// (`ObjectData::is_worker@0046fa10`: `0x32`..`0x35`) takes its order
    /// at `QUEUE_FIRST`, in front of what it holds, with the same `search`
    /// (`00700a05`..`00700a13`: `edx` is still `param_4` from `007008c4`
    /// or `00700909`).
    pub fn group_action_garrison_search(
        &mut self,
        g: &Group,
        b: usize,
        queue: QueuePos,
        search: bool,
    ) {
        let queue = match queue {
            QueuePos::First => 0,
            QueuePos::Last => 1,
            QueuePos::New => 2,
        };
        let who = i32::from(self.buildings[b].owner);
        let mut view = crate::garrison_view::SimGarrison { sim: self, g };
        crate::garrison_view::action_garrison_steps(
            &mut view,
            b as i32,
            who,
            queue,
            i32::from(search),
            0,
        );
    }

    /// `ObjectData::is_worker@0046fa10`: the type is `0x32`..`0x35` — the
    /// Citizen, its tribe's, and the two Scholars.
    pub(crate) fn is_worker(&self, u: usize) -> bool {
        (0x32..=0x35).contains(&self.units[u].type_index)
    }

    /// `Group::action_eject_all(back_to_work, who, eject_o, eject_who)
    /// @00710b40` (`docs/ORDERS.md` §29), reached from
    /// `CommandPackage::process_eject_all@00947fe0` on a group of
    /// buildings. With `who < 0`, or `eject_who < 0` and `who` the group's,
    /// every building of the group, last first, that is alive, holds a
    /// squad and is no hangar gets `Object::eject_contents(0, −1, 0,
    /// back_to_work == 0)`, which for a building on the map defers:
    /// [`Sim::eject_contents`], one squad a frame from the head.
    ///
    /// SEAMS: `back_to_work`'s type filter (`0x32`, the citizens) and the
    /// immediate path it takes; the `eject_o`/`eject_who` arm that pulls one
    /// player's units out of another's building. A group of one boat, with
    /// the two types (`0x140`, `0x13e`) that die when emptied, is
    /// [`Sim::boat_eject_all`] (item 3521).
    pub fn action_eject_all(
        &mut self,
        who: Player,
        buildings: &[usize],
        eject_who: i32,
        cmd_who: i32,
    ) {
        if !eject_all_own_arm(i32::from(who), eject_who, cmd_who) {
            return;
        }
        for &b in buildings.iter().rev() {
            let bd = &self.buildings[b];
            if !eject_all_wants(
                bd.alive,
                || !bd.garrison.is_empty(),
                || bd.ty.is_some_and(|t| self.is_hangar(t)),
            ) {
                continue;
            }
            self.eject_contents(b, false);
        }
    }

    /// `Group::action_patrol@007030c0` (`docs/ORDERS.md` §27), reached from
    /// `CommandPackage::process_patrol@00949380` with the command's point
    /// and queue position.
    ///
    /// The point is clamped into the world and a `QUEUE_FIRST` taken as
    /// `QUEUE_NEW`; the group's `form` goes to −1. The patrol's first point
    /// is the group's location ([`Self::group_loc`], the leader's position
    /// unless the group stands on its own order point), and every member
    /// that is on the map, not a plane and not busy gets a
    /// `GroupPatrolOrder` from it to the click, with the group's id, its
    /// own index and the leader. `order_num` steps once after the loop.
    ///
    /// SEAMS, none of which a player's command on land units reaches:
    /// a plane leader or any plane member hands the whole call to
    /// `action_air_patrol`, which returns here; the `QUEUE_LAST` arm reads
    /// [`Self::group_loc_to`] and appends the click to each member's
    /// existing patrol (`update_patrol_order`) instead of issuing one —
    /// here it issues; a member off the map with type flag `0x20` gets a
    /// plain move; the scenario's `ignore_orders` filter; a buildings
    /// group. `UnitData::is_busy@0060a370` is [`Self::is_busy`]: a head pack
    /// or unpack, never another cast — and a lone member's alone, since a
    /// group's leader halts every member in `do_patrol`'s `QUEUE_FIRST`
    /// move the same frame (item 3988, `docs/GOLDEN.md` §181).
    pub fn group_action_patrol(&mut self, g: &Group, to: Pos, queue: QueuePos) {
        let to = self.restrict_pos(to);
        let queue = if queue == QueuePos::First {
            QueuePos::New
        } else {
            queue
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let Some(leader) = self.group_find_leader(g) else {
            return;
        };
        // `count(COUNT_DOMAIN, 4)`: param 4 counts vslot `+0xc0`, `is_plane`,
        // over the alive members — a Helicopter is not counted, so a group
        // with one and no plane takes the land arm (item 3967, `docs/GOLDEN.md`
        // §173).
        if self.is_plane(leader)
            || g.list
                .iter()
                .any(|&m| self.units[m].alive() && self.is_plane(m))
        {
            self.group_action_air_patrol(g, to, queue);
            return;
        }
        let from = if queue == QueuePos::Last {
            self.group_loc_to(g)
        } else {
            self.group_loc(g)
        };
        let Some(from) = from else {
            return;
        };
        let id = group_move_id(self.group_id(g), self.frame, self.group_order_num(g));
        for (i, &m) in g.list.iter().enumerate() {
            if !self.group_member_orderable(m) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy {
                continue;
            }
            self.add_patrol_order(m, (from, to), id, i, leader, queue);
        }
        self.bump_order_num(g);
    }

    /// **`Group::action_air_patrol(x, y, queue)@007029d0`** (item 3911,
    /// `docs/GOLDEN.md` §158), which [`Self::group_action_patrol`] hands a
    /// group with a plane leader or an air member. The point is clamped
    /// into the world, and every member that is a plane (`is_plane`: domain
    /// 2, not a helicopter), not busy and not a missile (`type +0x1e4 &
    /// 0x8000000`) takes `add_air_patrol_order(x, y, home, 1)`: the home is
    /// the front air order's (a `STRAFE`, `AIR_PATROL` or
    /// `AIR_ATTACK_GROUND`, `+4`/`+8`), else `get_inside`, and a plane with
    /// neither takes nothing. The adder's queue argument is the constant 1,
    /// not the command's. run1505's Fighter `0/6`, flying home on its strafe:
    /// one `AIRPATROLORDER` over (7680, 11520), home `0/2007`, on 682.
    ///
    /// SEAMS, none of which run1505 reaches: a group with members that are
    /// not planes is split, the planes recurse and the rest take
    /// `action_move_to(MOVE_TO)`, here the others take nothing; a plane
    /// already on an `AIR_PATROL` under `QUEUE_LAST` appends the point to
    /// its arrays (see [`crate::orders::PATROL_POINTS`]), here it is given
    /// a new patrol; the empty-tank feedback is the interface's alone; the
    /// scenario's `ignore_orders` filter.
    pub fn group_action_air_patrol(&mut self, g: &Group, to: Pos, _queue: QueuePos) {
        let to = self.restrict_pos(to);
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy || self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            let home = match self.current_air(m) {
                Some(a) => a.home,
                None => self.units[m].inside,
            };
            if home.is_some() {
                self.add_air_patrol_order(m, to, home, true);
            }
        }
    }

    /// **`GroupData::is_in_air_range(o, whom)@0070b6b0`** (item 3911,
    /// `docs/GOLDEN.md` §158): `Console::execute_at_cursor@007c6630:3116`'s
    /// test for a right-click on an enemy with a selection that carries
    /// planes. Whether the target object is within `mana × get_speed(·, 1)`
    /// of a plane, the listing's arithmetic (`0070b6b0`–`0070b9d0`):
    ///
    /// - **a building selection** (`+0x49`): each live member that
    ///   `can_carry(AIR)` measures `vector_dist(target − member)` once, and
    ///   any plane in its `inside_down` chain whose `UnitData::mana` times
    ///   its speed at its own point (vslot `+0x17c`, flag 1) reaches it
    ///   answers 1;
    /// - **a unit selection**: each live member that is off the map and a
    ///   fixed-wing plane measures from the host `get_inside` names.
    ///
    /// run1505b's Airbase `0/2007` with the Fighter `0/6` inside, on
    /// `1/2006`: 1, and the click issued the launch on 807.
    ///
    /// SEAM: `can_carry(AIR)` is [`Self::is_hangar`] here (an Airbase or a
    /// Missile Silo); a carrier selected is the unit arm's host, not
    /// reached.
    pub fn is_in_air_range(&self, buildings: &[usize], units: &[usize], target: Obj) -> bool {
        let at = self.pos_of(target);
        let reach = |u: usize| self.unit_mana(u) * self.get_speed(u, 1);
        for &b in buildings {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.ty.is_some_and(|t| self.is_hangar(t)) {
                continue;
            }
            let d = crate::world::vector_dist(at.x - bd.pos.x, at.y - bd.pos.y);
            if bd.garrison.iter().any(|&u| d <= reach(u)) {
                return true;
            }
        }
        for &u in units {
            let un = &self.units[u];
            if !un.alive() || un.on_map || !self.is_plane(u) {
                continue;
            }
            let Some(host) = un.inside else {
                continue;
            };
            let h = self.buildings[host].pos;
            if crate::world::vector_dist(at.x - h.x, at.y - h.y) <= reach(u) {
                return true;
            }
        }
        false
    }

    /// **`Group::action_air_attack_ground(x, y, queue)@00703d80`** (item
    /// 3911, `docs/GOLDEN.md` §158), which `Group::action_attack_ground@
    /// 00704520:119-168` hands a group whose leader is a plane or carries
    /// `AIR`. The point is clamped into the world, and each live member that
    /// is a plane and not busy takes `add_air_attack_ground_order(x, y,
    /// home, queue, 1)`: the home is the front air order's (a `STRAFE`,
    /// `AIR_PATROL` or `AIR_ATTACK_GROUND`), else `get_inside`, and a plane
    /// with neither takes nothing. run1505b's Bomber `0/7`, on its patrol
    /// over the Barracks' point: one `AIRATTACKGROUNDORDER` there, home
    /// `0/2007`, on 682.
    ///
    /// SEAMS, none of which run1505b reaches: the missile arms — a missile
    /// still in its silo's `launching` list is skipped, the first missile
    /// ends the search for others (`local_28`), and a missile's strike asks
    /// `war_allowed` and the target tile's ocean; a group holding type
    /// `0x13b` asks each member its type's slot `+0x60`; the empty-tank
    /// feedback is the interface's; the scenario's `ignore_orders` filter.
    pub fn group_action_air_attack_ground(&mut self, g: &Group, at: Pos, queue: QueuePos) {
        let at = self.restrict_pos(at);
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            if self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            let busy = self.is_busy(m);
            if busy {
                continue;
            }
            let home = match self.current_air(m) {
                Some(a) => a.home,
                None => self.units[m].inside,
            };
            if home.is_some() {
                self.add_air_attack_ground_order(m, at, home, queue, true);
            }
        }
    }

    /// **`Group::action_recall@006fa7e0`**, the `recall` command's (byte
    /// 0x23, `CommandPackage::process_recall@00947bd0`) group action: a
    /// leader whose type is domain 2 hands the call to
    /// [`Self::group_action_return`] (item 3911, `docs/GOLDEN.md` §158).
    ///
    /// The building arm is [`Self::group_action_recall_buildings`] (item
    /// 3912), which the harness reaches on a building selection.
    pub fn group_action_recall(&mut self, g: &Group) {
        let Some(leader) = self.group_find_leader(g) else {
            return;
        };
        if self.group_domain(leader) == Domain::Air {
            self.group_action_return(g);
        }
    }

    /// **`Group::action_return@006fad40`** (item 3911, `docs/GOLDEN.md`
    /// §158): each member that is not a missile and stands on the map takes
    /// its front air order's home: the order's `cruising_alt` and
    /// `sharp_turn` are kept, its orders are closed (`unit_masks &=
    /// ~0x4000000`, the path emptied, `close_orders`, `clear_partial_path`,
    /// `update_action`), and `add_strafe_order(-1, -1, home, 0, QUEUE_NEW,
    /// 0)` lays a strafe home, which takes `returning` 1 for having no
    /// target; the two kept fields are written back into it. run1505's
    /// Bomber `0/8`, on its strafe at `1/2006`: on 854 one `STRAFEORDER`,
    /// target −1, `mandatory 0`, home `0/2007`, `cruising_alt` 1600,
    /// `returning 1`, the patrol behind it gone.
    ///
    /// SEAMS, none of which run1505 reaches: a helicopter (`+0x2b4 & 0x20`)
    /// on the map is closed and given a strafe with no home, flying home;
    /// a plane inside a host is closed and cut from the host's
    /// `launching` list (`+0x44`), which this crate does not carry; a front
    /// `SPECIALANIMORDER` (a landing) gives its own air order; the
    /// scenario's filter of units that ignore orders.
    pub fn group_action_return(&mut self, g: &Group) {
        for &m in &g.list {
            if !self.units[m].alive() || !self.is_plane(m) {
                continue;
            }
            if self.profile(Obj::Unit(m)).has(crate::combat::mask::MISSILE) {
                continue;
            }
            if self.units[m].inside.is_some() {
                continue;
            }
            let Some(air) = self.current_air(m) else {
                continue;
            };
            self.units[m].path.clear();
            self.close_orders(m);
            self.clear_partial_path(m);
            self.update_action(m);
            self.add_strafe_order(m, None, air.home, false, QueuePos::New, false);
            self.set_cruising_alt(m, air.cruising_alt);
            self.set_sharp_turn(m, air.sharp_turn);
        }
    }

    /// `Group::redo_patrol_order@00706d90`, `finish_insert`'s case `0x16`
    /// (`docs/ORDERS.md` §27): after a group `QUEUE_FIRST` has halted every
    /// member, each one that is on the map, not a plane and not a missile
    /// (`obj_masks & 0x8000000`) is given the saved patrol again at
    /// `QUEUE_LAST` — its two points, its id, its leader and **its
    /// `form_id`, the leader's** — and then the saved `waypoint` is
    /// written into the new order (`update_patrol_order(id)`). run184's
    /// squad carries `form_id 0` on all three patrols from its first leg.
    ///
    /// SEAM: the copy of points past the second; see
    /// [`crate::orders::PatrolOrder`].
    pub(crate) fn group_redo_patrol_order(&mut self, g: &Group, p: crate::orders::PatrolOrder) {
        let mut v = crate::redo_patrol_view::SimRedoPatrol::new(self, g, p);
        crate::redo_patrol_view::redo_patrol_order_steps(&mut v);
    }

    /// §9's anchor score.
    fn siege_anchor(&self, g: &Group, sub: &Group) -> Option<usize> {
        let mut best: Option<(i32, usize)> = None;
        for &c in &sub.list {
            let p = self.units[c].pos;
            let mut total = 0;
            for &u in &g.list {
                // `0070d830:194` gates every summed member on
                // `is_valid_unit() && is_on_map()`, so a garrisoned or
                // carried member does not drag the anchor towards itself.
                if !self.units[u].alive() || !self.units[u].on_map {
                    continue;
                }
                let q = self.units[u].pos;
                total += ((p.x - q.x).abs() + (p.y - q.y).abs()) >> 10;
            }
            if best.is_none_or(|(bt, _)| total < bt) {
                best = Some((total, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// `Group::action_flight(ox, whom, orders, shift, ctrl, alt)@006fb260`
    /// (`docs/ORDERS.md` §32): the flight command's one call, and the
    /// only maker of a player's `StrafeOrder`. `kind` is the command's
    /// `orders`: [`Flight::Home`] for `MOVE_TO`, a right-click on one's
    /// own base, and [`Flight::Strike`] for `ATTACK`, one on an enemy.
    ///
    /// The target must be live (`+8 & 1`); a flight home must name one of
    /// the group's own Airbases (`can_carry(AIR)`). Then, per member
    /// (the loop's bound is `group.num`):
    /// - **a member already on a `STRAFE`** has that order re-pointed and
    ///   gets no new one. A strike writes the target and its point, then
    ///   `returning 0`, `mandatory 1` and the action bit; a flight home
    ///   writes the base as its home, no target, `returning 1`,
    ///   `mandatory 1` and the action bit;
    /// - **any other member**'s "inside" is its `inside_up`. A strike
    ///   needs it; a flight home gives `add_strafe_order(−1, −1, base,
    ///   who, 1, QUEUE_NEW, 1)` unless the member already stands in that
    ///   base.
    ///
    /// So **an aircraft on the ground outside a base takes no strike at
    /// all**, and no feedback beyond the console's: run223's block 622.
    ///
    /// **A strike on a mixed group splits first**: the planes are flown by
    /// this call on a group of their own, and the rest take
    /// `action_attack(o, whom, mandatory 1, QUEUE_NEW)` (item 3991,
    /// `docs/GOLDEN.md` §184). **A flight home asks `can_carry` of each
    /// member**, which counts the base's homed planes in the air too.
    ///
    /// SEAMs, none reached by run223: the `NUCLEARMISSILE` arm, the missile arm of each gate, the empty-tank
    /// arms (this crate carries no fuel, so a tank is never empty), the
    /// `FIGHTERBOMBER` `home_base` gate, a carrier as the base, the
    /// `AIR_PATROL`/`AIR_ATTACK_GROUND` home as the "inside" (no member
    /// holds either), and the war test (always allowed). A strike from
    /// inside a base is [`Sim::strike_from_inside`]: a missile's three
    /// gates, a plane's none (item 3968).
    pub fn group_action_flight(&mut self, g: &Group, target: Obj, kind: Flight) {
        let live = match target {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive()),
            Obj::Building(b) => self.buildings.get(b).is_some_and(|b| b.alive),
        };
        if !live {
            return;
        }
        let base = match (kind, target) {
            (Flight::Home, Obj::Building(b))
                if self.buildings[b].owner == g.who
                    && self.buildings[b].ty.is_some_and(|t| {
                        crate::build::is(&self.build_types, t, crate::build::Ident::Airbase)
                    }) =>
            {
                Some(b)
            }
            (Flight::Home, _) => return,
            (Flight::Strike, _) => None,
        };
        // **The split** (`6fb3f9`..`6fb53f`, item 3991, `docs/GOLDEN.md`
        // §184): a strike whose live members are not all planes
        // (`UnitData::is_plane`, vslot `+0xc0`: a helicopter is no plane)
        // hands the planes to this call again on a group of their own, and
        // the rest to `action_attack(o, whom, 1, QUEUE_NEW, 0)` — or
        // `action_guard` on an ally's target — and returns.
        if kind == Flight::Strike {
            let mut air = Group::stack(g.who);
            let mut rest = Group::stack(g.who);
            for &u in &g.list {
                if !self.units[u].alive() {
                    continue;
                }
                if self.is_plane(u) {
                    self.group_add(&mut air, u);
                } else {
                    self.group_add(&mut rest, u);
                }
            }
            if !rest.list.is_empty() {
                self.group_action_flight(&air, target, Flight::Strike);
                let whom = self.owner_of(target);
                if whom < crate::world::PLAYER_SLOTS && self.is_ally(g.who, whom) {
                    // SEAM: `action_guard` on an ally's building, which
                    // this crate's guard does not take.
                    if let Obj::Unit(t) = target {
                        self.group_action_guard(&rest, t, QueuePos::New, false);
                    }
                } else {
                    self.group_action_attack(&rest, target, true, QueuePos::New, 0);
                }
                return;
            }
        }
        let point = self.pos_of(target);
        for &u in &g.list {
            if !self.units[u].alive() {
                continue;
            }
            // `MOVE_TO`'s gate (`6fb5f0`..`6fb67b`): `ObjectData::can_carry`
            // on each member, whose room counts every plane homed on the
            // base, flying or standing (`num_aircraft_here(0)`, item 3991).
            if let Some(b) = base
                && self.is_air_unit(u)
                && !self.base_can_carry(b, u)
            {
                continue;
            }
            if let Some(o) = self.units[u].orders.front_mut()
                && let Body::Strafe(ref mut sf) = o.body
            {
                match kind {
                    Flight::Strike => {
                        sf.target = Some(target);
                        sf.at = Some(point);
                        sf.returning = false;
                    }
                    Flight::Home => {
                        sf.home = base;
                        sf.target = None;
                        sf.at = None;
                        sf.returning = true;
                    }
                }
                sf.mandatory = true;
                o.flags |= flag::ACTION;
                continue;
            }
            let inside = self.units[u].inside;
            match kind {
                // A strike from inside a base: `crate::air`'s inside arm
                // (`docs/ORDERS.md` §38.2, item 836).
                Flight::Strike => {
                    if let Some(b) = inside {
                        self.strike_from_inside(u, b, target);
                    }
                }
                Flight::Home => {
                    if inside != base {
                        self.add_strafe_order(u, None, base, true, QueuePos::New, true);
                    }
                }
            }
        }
    }

    /// `Group::action_attack(o, whom, mandatory, queue, ignore)` (§10):
    /// three passes by domain, the `ignore` mask, and the `mandatory == 0`
    /// melee retarget — **as the steps of [`crate::attack_view`]** (item
    /// 3362), which the sweep holds to the original call for call; this is
    /// the crate's view over them ([`crate::attack_view::SimAttack`]).
    ///
    /// The one line that is the crate's own is the target's liveness: the
    /// original asks only `whom >= 0` (its caller never hands a dead one).
    pub fn group_action_attack(
        &mut self,
        g: &Group,
        target: Obj,
        mandatory: bool,
        queue: QueuePos,
        ignore: i32,
    ) {
        if !self.active(target) {
            return;
        }
        let (whom, wwho) = crate::attack_view::obj_key(self, target);
        let q = match queue {
            QueuePos::First => crate::attack_view::QUEUE_FIRST,
            QueuePos::Last => crate::attack_view::QUEUE_LAST,
            QueuePos::New => crate::attack_view::QUEUE_NEW,
        };
        let mut v = crate::attack_view::SimAttack {
            sim: self,
            g,
            target,
            insert: None,
        };
        crate::attack_view::action_attack_steps(
            &mut v,
            whom,
            wwho,
            i32::from(mandatory),
            q,
            ignore,
        );
    }

    // ------------------------------------------------------------------
    // The small pieces the actions share
    // ------------------------------------------------------------------

    /// §6.5's `ai_group`: `!(leaders[who].flags & 4) && group.army >= 0`.
    fn ai_army_group(&self, g: &Group) -> bool {
        g.army.is_some() && self.nation.get(g.who as usize).is_some_and(|n| !n.human)
    }

    /// `ObjectsData::find_city(SEARCH_FRIENDLY, who, 0x200, FILTER_ALL)`
    /// around a point.
    fn city_near(&self, who: Player, p: Pos) -> Option<usize> {
        let reg = self.world.tregion(p.tile());
        let mut best: Option<(i32, usize)> = None;
        for c in self.cities_of(who) {
            if self.cities[c].reg != reg {
                continue;
            }
            let q = self.cities[c].pos;
            let d = crate::world::vector_dist((p.x - q.x).abs(), (p.y - q.y).abs());
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, c));
            }
        }
        best.map(|(_, c)| c)
    }

    /// §6.5's stable: `can_garrison` → a garrison order, else a move to a
    /// spot around the city between `0x300` and `0x600`.
    fn stable_in_city(&mut self, u: usize, c: usize, angle: Angle) {
        let b = self.cities[c].building;
        let p = self.cities[c].pos;
        if self.units[u]
            .ty
            .zip(self.buildings[b].ty)
            .is_some_and(|(ut, bt)| self.can_garrison(ut, bt))
        {
            self.add_garrison_order(u, b, false, QueuePos::New, false);
            return;
        }
        let spot = self
            .find_nearby_spot(u, p, 0x300, 0x600, 0, angle, None)
            .unwrap_or(p);
        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::New, false);
    }

    pub(crate) fn group_o_angle(&self, g: &Group) -> Angle {
        self.gstate(g).map_or(Angle(0), |st| st.o_angle)
    }

    /// The group's own id (`GroupData +0x4`), the pool index: 64 slots a
    /// player, so a group's is `who × 64 +` the pool slot its record sits in
    /// (item 1926: an army's `group.pool`, a [`Pushed`] group's `state.pool`,
    /// which `get_open_slot` hands out after the AI scripts' building groups;
    /// item 1863 had the army's own number, which parted by one to four slots).
    /// A record with no slot yet falls back to its own number.
    fn group_id(&self, g: &Group) -> i32 {
        let who = i32::from(g.who);
        match (g.army, g.pushed) {
            (Some(s), _) => {
                let slot = self
                    .armies
                    .get(g.who as usize)
                    .and_then(|a| a.list.get(s))
                    .and_then(|a| a.group.pool)
                    .map_or(s as i32, i32::from);
                who * 64 + slot
            }
            (None, Some(i)) => who * 64 + self.pushed[i].state.pool.map_or(i as i32, i32::from),
            (None, None) => -1,
        }
    }

    fn group_order_num(&self, g: &Group) -> i32 {
        self.gstate(g).map_or(0, |st| st.order_num)
    }

    /// `Group::update_positions@00713810(o, who)` — the slot table
    /// rotated once a frame from `do_group_move`'s leader arm. It is what
    /// makes a formation bend around a corner.
    ///
    /// **And the angle is not the leader's heading.** `713844` loads
    /// `UnitData::angle` as the default, and then `7138e1`–`71390f`
    /// replaces it with `find_angle(order.waypoint − leader.pos)` whenever
    /// the leader's head order is a move that **has** a waypoint
    /// (`MoveOrder +0x10 dest`). So the block points where the leader is
    /// *going*, not where it is *facing*, and the two come apart on every
    /// frame of a turn — which is exactly where a follower would otherwise
    /// be walking into its leader.
    pub(crate) fn group_update_positions(&mut self, g: &Group, leader: usize) {
        if self.gstate(g).is_none() {
            return;
        }
        let mut v = SimUpdatePositions {
            sim: self,
            g,
            u: leader,
        };
        crate::update_positions_view::update_positions_steps(&mut v);
    }

    /// A follower's target point this frame: the leader's **current**
    /// position plus slot `i`'s rotated offset.
    pub(crate) fn group_slot_point(&self, g: &Group, leader: usize, i: usize) -> Option<Pos> {
        let off = *self.gstate(g)?.curr.get(i)?;
        let p = self.units[leader].pos;
        Some(Pos::new(p.x + off.x, p.y + off.y))
    }

    /// After a refresh, every member's group order names the member that
    /// took the formation over (`refresh_group_order`'s last third —
    /// `modify_group_order` per member).
    pub(crate) fn group_rewrite_leader(&mut self, g: &Group, leader: usize, id: i64) {
        for i in 0..g.list.len() {
            let m = g.list[i];
            for o in &mut self.units[m].orders {
                if let Some(mv) = o.move_mut()
                    && let Some(gm) = mv.group.as_mut()
                    && gm.id == id
                {
                    gm.leader = leader;
                }
            }
        }
    }

    pub(crate) fn bump_order_num(&mut self, g: &Group) {
        if let Some(st) = self.gstate_mut(g) {
            st.order_num += 1;
        }
    }

    /// `action_move_near`'s writes to the record after `compute_form`: the
    /// origin and its angle, only for `QUEUE_NEW` and `QUEUE_LAST` (§6.3) —
    /// so `charge`'s `QUEUE_FIRST` leaves them. `form` (`70535a`) is written
    /// before the layout, in [`Self::group_action_move_to`].
    fn record_move(&mut self, g: &Group, to: Pos, angle: Angle, queue: QueuePos) {
        let Some(st) = self.gstate_mut(g) else { return };
        if queue != QueuePos::First {
            st.o = to;
            st.o_angle = angle;
        }
    }

    /// The different-region arm's `+0x80 = this->id`: the member names the
    /// group. A group with no seat has no id to write.
    fn repoint_member(&mut self, g: &Group, u: usize) {
        if let Some(pool) = self.gstate(g).and_then(|st| st.pool) {
            self.units[u].group_ptr = Some(pool);
        }
    }

    /// `Group::refresh_group_order@00713a50` (§6.8) — hand the group's
    /// formation over to `member`, which becomes the block's origin and the
    /// object every member's group order names.
    ///
    /// `Unit::do_group_move@005e79a0` calls it when the unit named by the
    /// order's `oxx` is no longer usable — dead, off the map, in another
    /// group, or no longer holding a matching group order — and the mover is
    /// still more than `0x5ff` away. The trigger is order machinery the
    /// simulation stands in for (§12's third seam); the **effect** is this,
    /// and it is what run31's record actually shows: the table it dumps is
    /// `compute_dests`' output re-origined nought or once.
    ///
    /// Returns whether the group had a slot for `member`.
    pub fn group_refresh_order(&mut self, g: &Group, member: usize) -> bool {
        // `00713a5a`: `normalize(this)` opens it, and its tail is the
        // group's speed (§18).
        self.group_set_speed(g);
        let Some(i) = g.list.iter().position(|&u| u == member) else {
            return false;
        };
        let Some(st) = self.gstate_mut(g) else {
            return false;
        };
        st.reorigin(i);
        // `713b94`: the tail is `call 0x713810` — the whole of
        // `update_positions` off the taking member, **waypoint arm
        // included**, not a turn by its heading (Great Sahara 17493, item
        // 1206). `modify_group_order` before it writes only the group
        // order's `oxx` and `whose`, so the waypoint it reads is the
        // member's own.
        self.group_update_positions(g, member);
        true
    }

    /// `Group::compute_form`'s own reverse test (§6.3): is the leader
    /// pointing more than 90° away from the formation's bearing?
    ///
    /// `compute_form@00707c80` flips `facing` when it holds, calls
    /// `Form::compute` with the flipped value, and flips it back — so the
    /// mirror the layout uses is `facing XOR this`, and the record only ever
    /// shows `facing`. The heading is the **leader's** own, offset by its
    /// slot's angle byte, and the compare is unsigned on the wrapped
    /// difference so both bounds are inclusive.
    ///
    /// The byte's **sign** in that sum is the listing's, not the
    /// decompiler's paraphrase: `707e95`–`707ea6` is `movsbl angles[slot]`,
    /// `shll $0x18`, `sub` — so `compute_form` **subtracts** the packed
    /// byte, and `705f42`–`705f4d` **adds** the same product onto the angle
    /// the order carries (§6.6 step 6). No capture separates the two, since
    /// every group any run has dumped lays out in Line and carries `angles`
    /// of all zero; a formation that leans is still what would show it.
    fn group_leader_faces_away(&self, g: &Group, angle: Angle) -> bool {
        let Some(heading) = self.group_leader_bearing(g) else {
            return false;
        };
        reversing(Angle(heading.0.wrapping_sub(angle.0)))
    }

    /// The quantity both halves of §6.3 are built on: the leader's own
    /// heading less its packed slot byte, `%esi` at `707ea8`.
    ///
    /// `compute_form` computes it once and uses it twice — the zero-delta
    /// arm **assigns** it as the formation angle (`707e3d`–`707e6d`) and the
    /// mirror test compares it against whatever angle was chosen — so it is
    /// one function here rather than two readings of the same listing.
    fn group_leader_bearing(&self, g: &Group) -> Option<Angle> {
        let (u, slot) = self.group_find_leader_slot(g)?;
        let byte = self.group_angles(g).get(slot).copied().unwrap_or(0);
        Some(Angle(
            self.units[u]
                .movement
                .heading
                .0
                .wrapping_sub(i32::from(byte) << 24),
        ))
    }

    /// `GroupData::facing` — the mirror flag the layout reads.
    fn group_facing(&self, g: &Group) -> bool {
        self.gstate(g).is_some_and(|st| st.facing)
    }

    /// The group this unit belongs to, if any — the original's
    /// `unit +0x80` read straight into the pool.
    ///
    /// Two seats hold a group here: an army's (`docs/ARMY.md` §3.2) and a
    /// [`Pushed`] slot, and the pointer ([`Unit::group_ptr`]) names one of
    /// them through [`Sim::seat_of`]. Until item 557 this asked the lists,
    /// which cannot tell a unit its group has dropped from one it never
    /// left (§23).
    pub(crate) fn group_of(&self, u: usize) -> Option<Group> {
        match self.seat_of(u)? {
            Seat::Army(w, a) => Some(self.army_group(w, a)),
            Seat::Pushed(i) => self.pushed_group_at(i),
        }
    }

    /// Is this unit the leader of its own group?
    ///
    /// `Unit::set_angle` and `Unit::kill_current_order` both ask it the same
    /// way and both only act when the answer is yes: `find_leader` for a
    /// unit group, `list[0]` for a buildings group, and `-1` — nobody —
    /// while the group holds fewer than one member.
    fn is_group_leader(&self, u: usize, g: &Group) -> bool {
        g.num() >= 1 && self.group_find_leader(g) == Some(u)
    }

    /// `Unit::set_angle@00605400`, the second writer of `GroupData::facing`
    /// (§4.1) and the ordinary source of a live group's `facing 1`.
    ///
    /// Setting a unit's heading to something 90° or more from the one it
    /// has **toggles its group's mirror flag**, if that unit is the group's
    /// leader. The original also flips the unit's own `unit_masks & 2`,
    /// which nothing this simulation models reads.
    ///
    /// The field is `UnitData::angle` — [`Movement::heading`], not the
    /// facing. `set_angle` compares against it, writes it, and passes the
    /// same value on to guy 0 as its `des_angle`; guy 0's own `angle`, the
    /// one the step is taken along, is turned by `Guy::do_turn` and is not
    /// touched here.
    ///
    /// `Unit::move_step`'s call is modelled — the one a marching unit
    /// makes every frame — and `Unit::do_build`'s turn to its site (item
    /// 1330). `come_out`'s turn to a boat's heading reaches the group arm
    /// through [`Self::toggle_leader_facing`] (item 2193). The other
    /// callers (`do_gather`, `fight`, …) are turns this simulation makes
    /// through `Movement::set_heading` or not at all, and each is a place a
    /// group's flag would move that this one leaves still.
    ///
    /// **And the tail of it is `Guy::set_angle(guy 0, angle, 0)`**, whose
    /// crew loop rewrites every tracked figure's `des_angle` and `des`
    /// from guy 0's own point and *this* angle — the **heading**, which is
    /// the bearing `find_angle` has just returned and not the facing the
    /// step is taken along ([`Sim::crew_des`],
    /// `docs/MOVEMENT.md`, "Who writes it, and when", third row). It
    /// fires at the **top** of `move_step`, ahead of the collision block,
    /// so a crew figure that walked exactly onto its destination last
    /// frame is one unit off it again before the blocked stand's
    /// `set_anim` asks: `Guy::set_anim`'s walking-guy early return then
    /// takes it, and it does not roll. That was East Indies 6571 — the
    /// crew's second `Unit::move_step+0x823` draw, and the difference is
    /// 720,896 of a turn, one unit on each axis of a (−48, −192) track.
    pub fn unit_set_angle(&mut self, u: usize, angle: Angle) {
        self.unit_set_angle_as(u, angle, false);
    }

    /// `Unit::set_angle(angle, target, param_3)` with its third argument
    /// set — `Unit::do_cast@005ebfe0`'s turn to its target (`5ec619`
    /// pushes 1; item 3920, `docs/GOLDEN.md` §167). `Guy::set_angle@005d9010`
    /// hands `param_3` to its crew loop, which then **puts** each tracked
    /// figure on its point (`set_angle(crew, des_angle, 1)`,
    /// `set_new_location(crew, des, 1)`) rather than telling it the point:
    /// run1514c's Scout crew stands on its rotated track the cast's first
    /// frame, `stopped` 1, and plays the cast's `ATTACK2` with guy 0.
    pub(crate) fn unit_set_angle_as(&mut self, u: usize, angle: Angle, snap: bool) {
        let turned = reversing(Angle(
            angle.0.wrapping_sub(self.units[u].movement.heading.0),
        ));
        self.units[u].movement.heading = angle;
        self.units[u].movement.crew_des = Some(self.frame);
        // `605424`: the unit's own mirror flips before the leader test.
        if turned {
            self.units[u].movement.mirror = !self.units[u].movement.mirror;
        }
        let from = self.units[u].movement.body.pos;
        self.crew_des(u, from, angle, snap);
        if turned {
            self.toggle_leader_facing(u);
        }
    }

    /// `Unit::set_angle@00605400`'s group arm on its own: a turn of 90° or
    /// more by the group's **leader** (`find_leader`, through the unit's
    /// `+0x80`) toggles `GroupData::facing`. Its two callers are
    /// [`Self::unit_set_angle`] and `come_out`'s turn to the host's angle
    /// (`crate::transport`'s `land_tail`, item 2193), which writes the
    /// heading through [`crate::Movement::set_facing`] and so reaches
    /// here only for the flag.
    pub(crate) fn toggle_leader_facing(&mut self, u: usize) {
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(st) = self.facing_record_mut(&g)
        {
            st.facing = !st.facing;
        }
    }

    /// `Unit::kill_current_order@005e2cb0`'s move branch, the third writer
    /// of `GroupData::facing` (§4.1, §6.3) — and the one that makes a
    /// group's *second* click behave.
    ///
    /// A dying move order carries the mirror its formation was laid out
    /// with. When the unit is its group's leader, that mirror is written
    /// back onto the group — **inverted if the unit has since turned
    /// around**, by the same `reversing` window against the order's own
    /// angle. So `compute_form` never sees the flag the leader's marching
    /// left behind; it sees the last layout's own answer.
    ///
    /// The original also writes the result into `unit_masks & 2`
    /// ([`crate::Movement::mirror`]) for every unit, before the leader
    /// test, and skips the whole branch for an order type outside
    /// `{1, 2, 3, 4, 0x12, 0x13, 0x15}` — the move family, which is
    /// [`Body::Move`] here.
    pub(crate) fn hand_back_facing(&mut self, u: usize, order_facing: bool, order_angle: Angle) {
        let turned = reversing(Angle(
            self.units[u].movement.heading.0.wrapping_sub(order_angle.0),
        ));
        let f = order_facing != turned;
        // `5e3087`/`5e308d`: `unit_masks & 2` takes it, leader or not.
        self.units[u].movement.mirror = f;
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(st) = self.facing_record_mut(&g)
        {
            st.facing = f;
        }
    }

    /// The record a member's own write of `GroupData::facing` lands on —
    /// `kill_current_order`'s hand-back and `set_angle`'s toggle, both
    /// through the unit's `+0x80`. That is the seat's record, except while
    /// `action_siege_attack_to`'s stack sub-group holds the seat: then it
    /// is the parent's, set aside in [`Sim::set_aside`] (`docs/AI.md` §212).
    fn facing_record_mut(&mut self, g: &Group) -> Option<&mut GroupState> {
        if self
            .set_aside
            .as_ref()
            .is_some_and(|(who, army, pushed, _)| {
                (*who, *army, *pushed) == (g.who, g.army, g.pushed)
            })
        {
            return self.set_aside.as_mut().map(|(.., st)| st);
        }
        self.gstate_mut(g)
    }

    /// `GroupData::angles` — carried in because `Form::compute` does not
    /// clear it and Column and Mob never write it (`form::Form::new`).
    fn group_angles(&self, g: &Group) -> Vec<i8> {
        self.gstate(g)
            .map(|st| st.angles.clone())
            .unwrap_or_default()
    }

    /// `Form::compute`'s writes to the record: `form_num`, the per-member
    /// offsets quantised by the floor divide by 48 (§6.4), the group's
    /// `o_angle`/`o_dist` from the **leader's** slot rather than the
    /// order's point, and `update_positions`' `curr`.
    ///
    /// The last of those is really `Unit::do_group_move`'s, once a frame
    /// off the leader's own heading; it is written here from the same
    /// heading so the record has a value the moment the move is issued,
    /// which is what run29's window compares against.
    fn record_form(&mut self, g: &Group, f: &crate::form::Form, to: Pos) {
        if self.gstate(g).is_none() {
            return;
        }
        // The group's table is quantised from the `Form`'s **before**
        // `compute_form`'s tail negates either (§6.3): the tail negates two
        // tables that have already been written, and `quantise` is a floor
        // divide, so negating the raw value first would round the other
        // way.
        let q = |v: i32| {
            if f.flipped {
                -crate::form::Form::quantise(-v)
            } else {
                crate::form::Form::quantise(v)
            }
        };
        let off: Vec<(i32, i32)> = f.off.iter().map(|&(x, y)| (q(x), q(y))).collect();
        let theta = f.o.map_or(Angle(0), |u| self.units[u].movement.heading);
        let curr = Sim::form_update_positions(&off, theta);
        let (o_angle, o_dist) = Sim::form_leader_offset(f, to);
        let num = g.num();
        let angles = f.angles.clone();
        let Some(st) = self.gstate_mut(g) else { return };
        st.form_num = num;
        st.off = off;
        st.curr = curr;
        st.angles = angles;
        st.o_angle = o_angle;
        st.o_dist = o_dist;
    }

    /// `WorldData::restrict` on a destination (§6.1).
    pub(crate) fn restrict_pos(&self, p: Pos) -> Pos {
        Pos::new(
            p.x.clamp(0, self.world.width() * crate::world::UNITS_PER_CELL - 1),
            p.y.clamp(0, self.world.height() * crate::world::UNITS_PER_CELL - 1),
        )
    }
}

/// The arm of `Group::action_eject_all@00710b40` that empties the group's own
/// buildings: `cmd_who` (the original's `param_2`) is negative, or the command
/// names no other player (`eject_who`, `param_4`, negative) and `cmd_who` is
/// the group's player. Plain data, so `sweep::group_action_eject_all` drives it.
pub(crate) fn eject_all_own_arm(group_player: i32, eject_who: i32, cmd_who: i32) -> bool {
    (eject_who < 0 && cmd_who == group_player) || cmd_who < 0
}

/// One building of that arm: alive, then holding a squad, then no hangar —
/// the original asks `num_inside` and `can_carry(AIR)` only as far as the
/// answer needs, so the two are closures.
pub(crate) fn eject_all_wants(
    alive: bool,
    holds_squad: impl FnOnce() -> bool,
    is_hangar: impl FnOnce() -> bool,
) -> bool {
    alive && holds_squad() && !is_hangar()
}

/// `Group::update_positions`' view over the sim: the leader unit and the
/// group's slot table (`update_positions_view.rs`).
struct SimUpdatePositions<'a> {
    sim: &'a mut Sim,
    g: &'a Group,
    u: usize,
}

impl crate::update_positions_view::UpdatePositionsView for SimUpdatePositions<'_> {
    fn heading(&mut self) -> i32 {
        self.sim.units[self.u].movement.heading.0
    }
    fn waypoint(&mut self) -> Option<(i32, i32)> {
        match self.sim.current_move(self.u) {
            Some(m) if m.has_waypoint => Some((m.waypoint.x, m.waypoint.y)),
            _ => None,
        }
    }
    fn leader_point(&mut self) -> (i32, i32) {
        let p = self.sim.units[self.u].pos;
        (p.x, p.y)
    }
    /// `update_positions@00713810` loops over `form_num` slots, not `num`: a
    /// member past it keeps the `curr` it was given — a siege unit's, copied
    /// back from the sub-group (item 1493). The table's own length bounds it.
    fn form_num(&mut self) -> i32 {
        self.sim
            .gstate(self.g)
            .map_or(0, |st| st.form_num.max(0).min(st.off.len() as i32))
    }
    fn slot_offset(&mut self, i: usize) -> (i32, i32) {
        self.sim.gstate(self.g).map_or((0, 0), |st| st.off[i])
    }
    fn set_curr(&mut self, i: usize, p: Pos) {
        if let Some(st) = self.sim.gstate_mut(self.g) {
            if st.curr.len() <= i {
                st.curr.resize(i + 1, Pos::new(0, 0));
            }
            st.curr[i] = p;
        }
    }
}

#[cfg(test)]
mod tests;
