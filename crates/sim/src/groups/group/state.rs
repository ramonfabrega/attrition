//! The group types: `Flight`, `Member`, the insert, `GroupState`, `Pushed`, `Seat`, `Group`, the stance types and `push_group_steps` (STRUCTURE §5, move 7).

use super::*;

/// The flight command's `orders` (`docs/ORDERS.md` §32): `MOVE_TO` (1),
/// home to one's own base, or `ATTACK` (10), at an enemy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flight {
    Home,
    Strike,
}

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
    pub(super) saved: Vec<Order>,
    pub(super) aim: Option<(Obj, bool)>,
}

impl Insert {
    /// Nothing was copied: `action_form`'s `QUEUE_NEW` arm.
    pub(crate) fn is_empty(&self) -> bool {
        self.saved.is_empty()
    }
}

/// [`crate::groups::finish_insert_view::FinishInsertView`] over an [`Insert`]: the
/// copies are typed [`Order`]s, not the original's raw records, so a record's
/// words are what this crate stores of the order and an action reads the
/// current order's body where the record names a slot or a pair this crate
/// holds as one index (`docs/GROUPS.md` §12).
///
/// SEAM: `finish_insert`'s other cases — gather, garrison, board, follow,
/// trade, spell, form, attack-ground — reach no action here. No capture
/// reaches a group `QUEUE_FIRST` carrying one.
pub(super) struct SimInsert<'a> {
    pub(super) sim: &'a mut Sim,
    pub(super) g: &'a Group,
    pub(super) saved: std::collections::VecDeque<Order>,
    pub(super) cur: Option<Order>,
    pub(super) aim: Option<(Obj, bool)>,
}

impl crate::groups::finish_insert_view::FinishInsertView for SimInsert<'_> {
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
        let mut v = crate::groups::form_view::SimActionForm::new(&mut *self.sim, self.g);
        crate::groups::form_view::action_form_steps(&mut v, a[0], a[1], a[2], a[4]);
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
    /// `+0x34 role`: **`Group::find_role@007081f0`'s word** — zeroed, then,
    /// for a group of units (`buildings` clear), the OR of every member's
    /// type `role` (`UnitType +0x2c8`, [`crate::ai::ai_load::UnitCols::role`])
    /// that is still in use (`docs/GROUPS.md` §3.4). Its only writers are
    /// `find_role` — run by `Group::normalize@00711540` and by
    /// `Groups::process@006fa210` on the cursor's slot — and
    /// `Group::clear@00713e80`'s zero; `copy_group` leaves it, so a pushed
    /// group keeps its slot's last word until the slot's next normalize
    /// (item 4116: run 1562's slot 2, pushed on 90, still reads 0 on 91
    /// beside a Citizen's `0x40300`). Read by the checksum's groups line
    /// (`docs/CHECKSUM.md` §19) and by `Army::add_group`'s OR into the
    /// army's role, which this crate does not carry.
    pub role: u32,
}

impl GroupState {
    /// `Groups::copy_group@006fa690` of a fresh stack group into pool slot
    /// `pool`, whose record was `prev`: the stack group's own `(ox, oy)`,
    /// `o_dist`, `o_angle`, `speed` and offset arrays, `stamp = frame`, and
    /// everything else — `form`, `order_num`, `facing`, `form_num`,
    /// `new_speed`, `march`, `role` — the previous occupant's (§28, §30). The
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
    } // twin of Groups::copy_group

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
        self.role = 0;
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
            role: 0,
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
    pub(super) const fn options(self) -> i32 {
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
pub(super) const COMBAT_STANCES: [Stance; 6] = [
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
