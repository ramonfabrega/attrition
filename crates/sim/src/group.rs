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

/// `MAX_AIRCRAFT_PER_AIRBASE` (`rules.xml`, 10): `ObjectData::can_carry`'s
/// room for aircraft in an Airbase.
const MAX_AIRCRAFT_PER_AIRBASE: usize = 10;

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
/// | the group pool | §3, 64 slots a leader and `get_open_slot`'s recycling | numbered since item 518 (§19) and reset by [`Sim::groups_process`]; `get_open_slot`'s fallbacks and `equals_group`'s normalize are not modelled |
/// | `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `Move` — `docs/ORDERS.md` §8.4's verdict |
/// | `action_guard`'s building arm, human sweep and `QUEUE_FIRST` | §9's escort half, `docs/ORDERS.md` §24.7 | no building is ever guarded; the unit arm is built (item 567) |
/// | the order-time path plan | §6.7 | the sim plans on the first step, in `do_move`; with a zero slot offset the plan is the same one |
/// | `find_nearby_spot`'s collision, `invalid_loc` on a slot | §6.6 step 4 | `find_nearby_spot` still does not ask the occupancy index, so no slot is ever invalid and no member is re-slotted |
/// | `QUEUE_FIRST`'s insert dance (`set_up_insert` / `action_halt` / recurse / `finish_insert`) | §6.2, §10 | `charge`'s `QUEUE_FIRST` is a plain `push_front` on each member |
/// | the scenario `ignore_orders` filter | §5 | never set outside a scenario |
/// | `is_modern_infantry`, `is_packing`, the strafe | §6.6, §10 | no modern infantry, no packers in flight, no planes |
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
    /// set only by the forced-march arm below it. It gates whether a
    /// follower reports its speed at all.
    ///
    /// SEAM: nothing sets it here, because the arm that does is
    /// `has_general(0x8000, -1)` under `LeaderData & 0x8000` and no capture
    /// has a general.
    pub march: bool,
    /// **Which of its player's 64 pool slots this record is** —
    /// `GroupData::id − who·64`, the number every member's `UnitData +0x80`
    /// holds and the dump prints as `group` (`docs/GROUPS.md` §19).
    /// `None` for a seat that never took one. It is what
    /// [`Sim::groups_process`]'s cursor selects on, so an army and a
    /// pushed group are reset on the frames the original resets them.
    pub pool: Option<u8>,
}

impl GroupState {
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

/// The option index of a stance — `action_stance`'s `s`.
pub fn stance_option(s: Stance) -> i32 {
    COMBAT_STANCES
        .iter()
        .position(|&x| x == s)
        .expect("every Stance is in COMBAT_STANCES") as i32
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
        if !(force || g.num() >= 2) {
            // `+0x80 = −1` on every active member, and no list is
            // touched: the unit stays wherever it was listed, naming
            // nothing (§23).
            for &u in &g.list {
                if self.units[u].alive() {
                    self.units[u].group_ptr = None;
                }
            }
            return false;
        }
        let last = self.last_group[g.who as usize];
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
            for &u in &g.list {
                if self.units[u].alive() {
                    self.units[u].group_ptr = Some(pool);
                }
            }
            g.army = None;
            g.pushed = Some(i);
            return true;
        }
        self.unseat_group(g);
        // The pool slot every member's `+0x80` then points at.
        // `Groups::get_open_slot` recycles, so a slot whose members are
        // all gone is taken before a new one is appended — without that
        // the pool would grow without bound over a long game, and the
        // original's is 64 entries.
        let slot = self
            .pushed
            .iter()
            .position(|x| x.list.iter().all(|&u| !self.units[u].alive()))
            .unwrap_or_else(|| {
                self.pushed.push(crate::group::Pushed {
                    who: g.who,
                    list: Vec::new(),
                    state: GroupState::default(),
                });
                self.pushed.len() - 1
            });
        // `Groups::copy_group@006fa690` stamps the fresh slot with the
        // frame (`+0x14 = game->frame`), whatever the stack group held:
        // run215's slot 1 prints `stamp 621`, the frame its selection was
        // processed (`docs/GOLDEN.md` §23).
        self.pushed[slot] = crate::group::Pushed {
            who: g.who,
            list: g.list.clone(),
            state: GroupState {
                pool: Some(pool),
                stamp: self.frame,
                ..GroupState::default()
            },
        };
        // The second walk's tail: `+0x80 = slot` on every active member.
        for &u in &g.list {
            if self.units[u].alive() {
                self.units[u].group_ptr = Some(pool);
            }
        }
        g.army = None;
        g.pushed = Some(slot);
        true
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
    /// (`docs/ARMY.md` §3.4).
    fn unseat_group(&mut self, g: &Group) {
        let w = g.who as usize;
        // `Group::kill`'s own walk: up to the captain, then down `o_down`.
        let mut leaving: Vec<usize> = Vec::new();
        for &m in &g.list {
            if !self.units[m].alive() {
                continue;
            }
            for f in self.squad_of(self.captain_of(m)) {
                if !leaving.contains(&f) {
                    leaving.push(f);
                }
            }
        }
        // A pool slot is a group too, and `(*old->vtbl+0x10)` is asked of
        // whichever one holds the member — so a unit pushed twice leaves
        // the first slot rather than sitting in two groups at once.
        for slot in &mut self.pushed {
            slot.list.retain(|u| !leaving.contains(u));
        }
        if self.armies[w].list.iter().all(|a| a.units.is_empty()) {
            return;
        }
        let mut touched: Vec<usize> = Vec::new();
        for slot in 0..self.armies[w].list.len() {
            let a = &mut self.armies[w].list[slot];
            if a.units.iter().any(|u| leaving.contains(u)) {
                a.units.retain(|u| !leaving.contains(u));
                touched.push(slot);
            }
        }
        for slot in touched {
            self.army_normalize(g.who, slot);
        }
    }

    // ------------------------------------------------------------------
    // The pool's numbering and its per-frame pass (§19)
    // ------------------------------------------------------------------

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
    fn pool_members(&self, who: Player, s: u8) -> Vec<usize> {
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

    fn seat_parts(&mut self, seat: Seat) -> (&mut Vec<usize>, &mut GroupState) {
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
    fn seat_set_speed(&mut self, seat: Seat) {
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
    fn seat_get_num(&mut self, seat: Seat) -> usize {
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
        let mut f = self.top_captain(u);
        loop {
            self.units[f].group_ptr = s;
            match self.units[f].o_down {
                Some(d) if self.units[d].alive() => f = d,
                _ => return,
            }
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
            let s = (0..46u8)
                .find(|&s| s != last && self.pool_members(who, s).is_empty())
                .unwrap_or(45);
            // `get_open_slot`'s tail (§3.1): every unit of `who` whose
            // `+0x80` names the slot is cleared before the new group moves
            // in. A slot is chosen because nothing live both lists and
            // names it, so what this reaches is a pointer its list
            // dropped — the stale half §23 is about.
            for x in &mut self.units {
                if x.owner == who && x.group_ptr == Some(s) {
                    x.group_ptr = None;
                }
            }
            s
        };
        self.last_group[w] = s;
        s
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
                self.armies[w].list[a].units = keep;
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
                let keep: Vec<usize> = self.pushed[i]
                    .list
                    .iter()
                    .copied()
                    .filter(|&u| self.units[u].alive() && self.units[u].group_ptr == Some(s))
                    .collect();
                self.pushed[i].list = keep;
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
        // SEAM: the arm below it sets this instead, under
        // `LeaderData & 0x8000` and `has_general(0x8000, -1)`.
        st.march = false;
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
    /// sites that open with it.
    pub(crate) fn group_compute_speed(&self, g: &Group) -> i32 {
        if g.list.is_empty() {
            return 0;
        }
        self.group_find_leader(g)
            .map_or(0, |u| self.units[u].movement.speed)
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
    /// [`Self::group_loc`] uses, and it is measured here from the **final**
    /// point rather than from the unit (`0070c634`).
    ///
    /// SEAM: the `buildings` seat (`list[0]` rather than `find_leader`) and
    /// the `get_inside` hop for a garrisoned leader, neither of which any
    /// group this simulation builds reaches.
    pub(crate) fn group_loc_to(&self, g: &Group) -> Option<Pos> {
        let u = self.group_find_leader(g)?;
        let p = self.unit_final_loc(u);
        let o = self.group_o(g);
        if o.x >= 0 && o.y >= 0 && vector_dist(p.x - o.x, p.y - o.y) <= GROUP_LOC_NEAR {
            return Some(o);
        }
        Some(p)
    }

    /// `UnitData::get_final_loc@00608040` — where this unit's order list
    /// leaves it.
    ///
    /// The walk is over the list from the head and it stops at the **first**
    /// order that names a point, not the last: a move order hands back its
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
    pub(crate) fn unit_final_loc(&self, u: usize) -> Pos {
        let here = self.units[u].pos;
        let found = self.units[u].orders.iter().find_map(|o| match o.body {
            Body::Move(m) => Some(m.dest),
            Body::Attack(_) => self.units[u]
                .combat
                .target
                .and_then(|t| self.obj_alive_pos(t)),
            Body::Build(b) | Body::Repair(b) | Body::Garrison { building: b, .. } => {
                self.obj_alive_pos(crate::combat::Obj::Building(b))
            }
            Body::Gather(gt) => self.obj_alive_pos(crate::combat::Obj::Building(gt.building)),
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

    pub(crate) fn is_supply_unit(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            let c = self.unit_types[t].cols;
            c.flag2(uflags2::SUPPLY_OR_HERO) && !c.flag2(uflags2::GENERAL)
        })
    }

    fn is_special_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::SCOUT))
    }

    pub(crate) fn group_domain(&self, u: usize) -> Domain {
        self.units[u]
            .ty
            .map_or(Domain::Land, |t| self.unit_types[t].combat.domain)
    }

    /// `action_halt`'s and `action_attack`'s shared `ignore` mask (§7):
    /// `4` siege, `2` special, `1` spy. The simulation has no spy type, so
    /// bit 1 never matches.
    fn ignored(&self, u: usize, mask: i32) -> bool {
        (mask & 4 != 0 && self.is_siege_unit(u)) || (mask & 2 != 0 && self.is_special_unit(u))
    }

    /// `UnitData::is_plane@0046ce40` — `domain == 2 && !(unit_flags & 0x20)`.
    /// There is no altitude test: a plane is skipped whether it is in the
    /// air or on a runway (§7). Bit `f` is `unitrules.xml`'s "flies like a
    /// helicopter", carried by exactly three types — `Helicopter` and the
    /// two `Attack Helicopter`s, all of them `<DOMAIN>Air` — so a
    /// **helicopter is not a plane** and is ordered like a ground unit.
    pub(crate) fn is_plane(&self, u: usize) -> bool {
        self.group_domain(u) == Domain::Air
            && !self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.flag(uflags::HELICOPTER))
    }

    /// Whether a member takes an order at all: active, on the map, and not
    /// a plane. Every action's inner loop opens with this.
    fn group_member_orderable(&self, u: usize) -> bool {
        self.units[u].alive() && self.units[u].on_map && !self.is_plane(u)
    }

    // ------------------------------------------------------------------
    // The actions
    // ------------------------------------------------------------------

    /// `Group::action_swarm_around@0070fbe0` at **`QUEUE_LAST`** — the
    /// position `finish_insert` re-issues a build or a repair at (§24).
    ///
    /// The members are walked twice, land (`domain` 0) first and sea
    /// second; an air member is never taken. A citizen (`0x32`/`0x33`)
    /// gets the approach and the order ([`Sim::swarm_around_last`]).
    /// The approach is an `EXPLORE_TO` for a building under a computer
    /// (`local_40 = ~(leader_flags >> 1) & 2 | 1`) and a `MOVE_TO`
    /// otherwise.
    ///
    /// SEAM, none reached by a capture on file: the non-builders, which
    /// the original gathers into a scratch group and sends `MOVE_TO` the
    /// site at `QUEUE_NEW`; a builder already inside a building
    /// (`count_inside`) or able to cast `0x293`, which goes with them;
    /// the gather filter (`local_30`, the group's idle citizens, against a
    /// member whose action is a gather); and `is_busy`, a member mid-cast
    /// or boarding. `finish_insert`'s one reach on file is a goody box's
    /// one-member group whose member was just halted.
    fn group_action_swarm_around_last(&mut self, g: &Group, b: usize, body: Body, action: bool) {
        use crate::attrition::Domain;
        if !self.group_is_on_map(g) || !self.buildings.get(b).is_some_and(|bd| bd.alive) {
            return;
        }
        let kind = if matches!(body, Body::Build(_)) && !self.nation[g.who as usize].human {
            MoveKind::ExploreTo
        } else {
            MoveKind::MoveTo
        };
        for layer in [Domain::Land, Domain::Sea] {
            for &u in &g.list {
                if !self.group_member_orderable(u) {
                    continue;
                }
                let domain = self.units[u]
                    .ty
                    .map_or(Domain::Land, |t| self.unit_types[t].kind.domain);
                if domain != layer || self.worker_of(u) != crate::orders::Worker::Citizen {
                    continue;
                }
                self.swarm_around_last(u, b, body, action, kind);
            }
        }
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
    /// action at `QUEUE_LAST`.
    pub(crate) fn group_finish_insert(&mut self, g: &Group, insert: Insert) {
        let Insert { saved, aim } = insert;
        for o in saved {
            match o.body {
                // Cases 1–4, `0x13` and `0x15`: `action_move_near` to the
                // copy's `orig_x`/`orig_y` when both are non-negative — the
                // point the move was issued to, a group move's and a plain
                // one's alike — else its `x`/`y`, at `QUEUE_LAST`,
                // `set_angle 1` and the order's own angle, with the action
                // bit. run219's explorers replay to their click behind the
                // goody-box leg (`docs/GOLDEN.md` §24): the chariot's plan
                // ends on (2400, 17280), not its snap, and the squad lays
                // its slots out afresh round the click.
                Body::Move(m) => self.group_action_move_to(
                    g,
                    m.group
                        .map(|gm| gm.orig)
                        .or(m.orig)
                        .filter(|p| p.x >= 0 && p.y >= 0)
                        .unwrap_or(m.dest),
                    QueuePos::Last,
                    true,
                    m.angle,
                    m.kind,
                    true,
                ),
                Body::Attack(_) => {
                    if let Some((t, mandatory)) = aim {
                        self.group_action_attack(g, t, mandatory, QueuePos::Last, 0);
                    }
                }
                // Cases 6 and `0xd`: `action_swarm_around(o, who,
                // QUEUE_LAST, kind, flags & 4)`, the order's own
                // action bit. run157's `1/1` is a citizen on its way
                // to a site when the goody look halts it on 990, and
                // the original keeps the build behind the box's walk
                // (§24).
                Body::Build(b) | Body::Repair(b) => {
                    self.group_action_swarm_around_last(g, b, o.body, o.has(flag::ACTION));
                }
                // Case `0x16`: `redo_patrol_order(group, order,
                // QUEUE_LAST)` — every member's patrol rebuilt from
                // the leader's, its step included. It is how a
                // patrol's leg reaches the whole group (§27 of
                // `docs/ORDERS.md`, run184).
                Body::Patrol(p) => self.group_redo_patrol_order(g, p),
                // SEAM: `finish_insert`'s other seventeen cases —
                // gather, garrison, board, follow, guard, trade,
                // spell. No capture reaches a group `QUEUE_FIRST`
                // carrying one.
                _ => {}
            }
        }
    }

    /// `Group::action_halt(mask)` (§7).
    pub fn group_action_halt(&mut self, g: &Group, mask: i32) {
        // `0070d0c0:29`: the **group's** `form` is cleared once, before any
        // member is examined, and only for a unit group. No member's
        // `+0xaa` is touched — and `get_form` reads those bytes, so a halt
        // does not cost the group the formation its members still carry.
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        for &u in &g.list {
            if !self.group_member_orderable(u) || self.ignored(u, mask) {
                continue;
            }
            self.clear_orders(u);
        }
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
                // only, and no army's leader is human (`docs/GROUPS.md`
                // §8), so nothing else happens here.
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
        if queue == QueuePos::First {
            let insert = self.group_set_up_insert(g);
            self.group_action_halt(g, 0);
            self.group_action_move_to(g, to, QueuePos::New, set_angle, angle, kind, action);
            self.group_finish_insert(g, insert);
            return;
        }

        let to = self.restrict_pos(to);

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
        if queue == QueuePos::New {
            for (&u, p) in g.list.iter().zip(&plan) {
                if *p == Member::Move {
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
        let facing = self.group_facing(g);
        let reverse = facing != self.group_leader_faces_away(g, angle);
        // `Form::compute` → `Form::categorize`, whose first statement is
        // `Group::sort@00708090` (§4.1, §23) — after the clear and the
        // mirror above, before the layout and the order loop, which
        // therefore walk the **sorted** list. A member the sort brought in
        // was not in the list the clear walked, so it is decided now, as
        // the order loop's own second reading would.
        let resorted = Self::seat_of_group(g).and_then(|seat| {
            let before = self.seat_list(seat).clone();
            self.seat_sort(seat);
            (*self.seat_list(seat) != before).then(|| self.seat_group(seat))
        });
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
            let slot = self.restrict_pos(slots.to[i]);
            // §6.6 step 6: the order's angle is the formation's, **plus**
            // this slot's packed byte — an addition of a signed byte
            // shifted into the top of the word (`705f42`–`705f4d`), where
            // `compute_form` subtracts the same product back off the
            // leader's heading. And its `facing` is the mirror this layout
            // used, which is what the order hands back when it dies.
            let order_angle = Angle(angle.0.wrapping_add(i32::from(slots.angles[i]) << 24));
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
        self.record_move(g, to, angle, form, queue);
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
        self.group_action_move_to(
            &sub,
            to,
            QueuePos::New,
            true,
            angle,
            MoveKind::AttackTo,
            true,
        );
        if let (Some(p), Some(st)) = (parent, self.gstate_mut(&sub)) {
            *st = p;
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
    /// SEAMS, none reached by a capture on file: `QUEUE_FIRST`'s insert
    /// dance (`set_up_insert`, `action_halt`, the garrison at
    /// `QUEUE_NEW`, `finish_insert`), taken here as `QUEUE_NEW`; the
    /// scenario's `ignore_orders` sweep; the zero-limit arm's second test
    /// (a type vslot `+0x60`), taken as a refusal; `search`'s
    /// `find_garrison_build`, which the command never asks for; the
    /// editor's instant `go_inside` (`Game::semaphore` bit `0xb`); a
    /// worker's `QUEUE_FIRST` and a packing type's arms;
    /// `is_entering_or_exiting`.
    pub fn group_action_garrison(&mut self, g: &Group, b: usize, queue: QueuePos) {
        let queue = if queue == QueuePos::First {
            QueuePos::New
        } else {
            queue
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let whom = self.buildings[b].owner;
        if !self.is_ally(g.who, whom) || !self.buildings[b].alive {
            return;
        }
        let Some(bty) = self.buildings[b].ty else {
            return;
        };
        if self.garrison_limit(b) == 0 {
            return;
        }
        // `BuildData::is_unassimilated`: a city whose race is not its owner.
        if self.building_is_city(b)
            && let Some(c) = self.buildings[b].city
            && self.cities[c].alive
            && self.cities[c].race != Some(whom)
        {
            return;
        }
        for &m in &g.list {
            if !self.group_member_orderable(m) {
                continue;
            }
            if self
                .action_of(m)
                .is_some_and(|a| matches!(self.units[m].orders[a].body, Body::Garrison { .. }))
            {
                self.repath(m);
                self.kill_current_order(m);
            }
            if !self.units[m].ty.is_some_and(|t| self.can_garrison(t, bty)) {
                continue;
            }
            self.add_garrison_order(m, b, false, queue, true);
        }
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
    /// immediate path it takes; the two types (`0x140`, `0x13e`) that die
    /// when emptied; the `eject_o`/`eject_who` arm that pulls one player's
    /// units out of another's building.
    pub fn action_eject_all(
        &mut self,
        who: Player,
        buildings: &[usize],
        eject_who: i32,
        cmd_who: i32,
    ) {
        if !((eject_who < 0 && cmd_who == i32::from(who)) || cmd_who < 0) {
            return;
        }
        for &b in buildings.iter().rev() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.garrison.is_empty() || bd.ty.is_some_and(|t| self.is_hangar(t)) {
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
    /// a plane leader or any air member hands the whole call to
    /// `action_air_patrol`, which returns here; the `QUEUE_LAST` arm reads
    /// [`Self::group_loc_to`] and appends the click to each member's
    /// existing patrol (`update_patrol_order`) instead of issuing one —
    /// here it issues; a member off the map with type flag `0x20` gets a
    /// plain move; the scenario's `ignore_orders` filter; a buildings
    /// group. `UnitData::is_busy@0060a370` is a head `CastOrder` here — the
    /// original also asks the spell's type two questions, and whether the
    /// unit is entering or exiting a building.
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
        if self.is_plane(leader) || g.list.iter().any(|&m| self.group_domain(m) == Domain::Air) {
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
            let busy = matches!(self.current_order(m).map(|o| o.body), Some(Body::Cast(_)));
            if busy {
                continue;
            }
            self.add_patrol_order(m, (from, to), id, i, leader, queue);
        }
        self.bump_order_num(g);
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
        const MISSILE: u32 = 0x800_0000;
        for &m in &g.list {
            if !self.group_member_orderable(m) {
                continue;
            }
            let missile = self.units[m]
                .ty
                .is_some_and(|t| self.unit_types[t].combat.obj_masks & MISSILE != 0);
            if missile {
                continue;
            }
            self.add_patrol_order(
                m,
                (p.points[0], p.points[1]),
                p.id,
                p.form_id,
                p.leader,
                QueuePos::Last,
            );
            if let Some(i) = self.update_patrol_order(m, Some(p.id))
                && let Body::Patrol(x) = &mut self.units[m].orders[i].body
            {
                x.waypoint = p.waypoint;
            }
        }
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
    /// SEAMs, none reached by run223: the split of a strike's non-air
    /// members into `action_attack` (every member is air here), the
    /// `NUCLEARMISSILE` arm, the missile arm of each gate, the empty-tank
    /// arms (this crate carries no fuel, so a tank is never empty), the
    /// `FIGHTERBOMBER` `home_base` gate, a carrier as the base, the
    /// `AIR_PATROL`/`AIR_ATTACK_GROUND` home as the "inside" (no member
    /// holds either), and **a strike from inside a base**, whose
    /// `valid_target`, `MISSILE_DEFENSE_BONUS`, reach and war tests are
    /// not built: such a member takes no order here.
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
        let point = self.pos_of(target);
        for &u in &g.list {
            if !self.units[u].alive() {
                continue;
            }
            if let Some(b) = base
                && self.aircraft_inside(b) >= MAX_AIRCRAFT_PER_AIRBASE
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
                // SEAM: a strike from inside a base (above).
                Flight::Strike => {}
                Flight::Home => {
                    if inside != base {
                        self.add_strafe_order(u, None, base, true, QueuePos::New, true);
                    }
                }
            }
        }
    }

    /// `ObjectData::num_aircraft_here`: the planes standing in a base,
    /// its `inside_up` chain's aircraft.
    fn aircraft_inside(&self, b: usize) -> usize {
        self.buildings[b]
            .garrison
            .iter()
            .filter(|&&u| self.group_domain(u) == Domain::Air)
            .count()
    }

    /// `Group::action_attack(o, whom, mandatory, queue, ignore)` (§10):
    /// three passes by domain, the `ignore` mask, and the `mandatory == 0`
    /// melee retarget.
    pub fn group_action_attack(
        &mut self,
        g: &Group,
        target: Obj,
        mandatory: bool,
        queue: QueuePos,
        ignore: i32,
    ) {
        if !self.group_is_on_map(g) || !self.active(target) {
            return;
        }
        // **The leader asks first** (`action_attack@00712490:215`): one
        // `find_attack_pos` a group order, on the group's leader, and
        // only when `is_in_range` says the leader cannot already shoot
        // from where it stands. `docs/COMBAT.md` §17.1 step 3 — this is
        // the `Group::action_attack+0x41a` draw family, and it can never
        // run longer than one call's budget.
        if let Some(leader) = self.group_find_leader(g) {
            let at = self.units[leader].pos;
            if !self.is_in_range_at(Obj::Unit(leader), at, target) {
                self.find_attack_pos(leader, target, at, crate::fight::SITE_ATTACK_POS_GROUP);
            }
        }
        let respond = self.tuning.unit_respond_range * 0x240;
        for pass in [Domain::Land, Domain::Sea, Domain::Air] {
            for &u in &g.list {
                if !self.group_member_orderable(u) || self.group_domain(u) != pass {
                    continue;
                }
                if self.ignored(u, ignore) {
                    continue;
                }
                // Already carrying a mandatory ATTACK on this target, out
                // of range, with fewer than three orders queued — and then
                // two sub-arms decide (`00712490:390`–`424`). The outer
                // test is on `get_action`, the intent under the transit
                // legs, not on the current order's own type; the
                // simulation keeps the target on the unit rather than on
                // the attack order (`docs/GROUPS.md` §12), so
                // `combat.target`/`combat.mandatory` stand in for the
                // order's `ox`/`whom`/`+0x1c`.
                if self
                    .action_of(u)
                    .is_some_and(|i| self.units[u].orders[i].index() == index::ATTACK)
                    && self.units[u].combat.mandatory
                    && self.units[u].combat.target == Some(target)
                    && self.units[u].orders.len() < 3
                    && !self.is_in_range(Obj::Unit(u), target)
                {
                    // Nothing but the attack: keep it.
                    if self.units[u].orders.len() == 1 {
                        continue;
                    }
                    // Or a current *move* whose destination already puts
                    // the target in range: it is marching into the shot.
                    if let Some(o) = self.current_order(u)
                        && let Some(dest) = o.move_dest()
                        && self.is_in_range_at(Obj::Unit(u), dest, target)
                    {
                        continue;
                    }
                    // Otherwise it falls through and is re-ordered.
                }
                let t = if mandatory {
                    target
                } else {
                    let d = crate::world::vector_dist(
                        (self.units[u].pos.x - self.pos_of(target).x).abs(),
                        (self.units[u].pos.y - self.pos_of(target).y).abs(),
                    );
                    self.find_melee_target(u, (d + 0xc0).min(respond))
                        .unwrap_or(target)
                };
                self.add_attack_order(u, t, queue, mandatory, true);
            }
        }
        self.bump_order_num(g);
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
        if self.can_garrison(u, b) {
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

    /// The group's own id (`GroupData +0x4`). The seat stands in for the
    /// pool index the original keeps: an army group's is its army slot and
    /// a [`Pushed`] group's is `64 +` its slot, which is all
    /// [`group_move_id`] needs of it — distinct per owner, and stable for
    /// as long as the group lives. The numbers are **not** the dump's own
    /// (Great Lakes' probe is `group 65` there and this crate's armies do
    /// not sit in the same pool), so nothing compares them.
    fn group_id(&self, g: &Group) -> i32 {
        match (g.army, g.pushed) {
            (Some(s), _) => s as i32,
            (None, Some(i)) => 64 + i as i32,
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
        let Some(off) = self.gstate(g).map(|st| st.off.clone()) else {
            return;
        };
        let p = self.units[leader].pos;
        let theta = match self.current_move(leader) {
            Some(m) if m.has_waypoint => find_angle(m.waypoint.x - p.x, m.waypoint.y - p.y),
            _ => self.units[leader].movement.heading,
        };
        let curr = Sim::form_update_positions(&off, theta);
        if let Some(st) = self.gstate_mut(g) {
            st.curr = curr;
        }
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

    fn bump_order_num(&mut self, g: &Group) {
        if let Some(st) = self.gstate_mut(g) {
            st.order_num += 1;
        }
    }

    /// `action_move_near`'s writes to the record: `form` unconditionally
    /// (`70535a`), the origin and its angle only for `QUEUE_NEW` and
    /// `QUEUE_LAST` (§6.3) — so `charge`'s `QUEUE_FIRST` leaves them.
    fn record_move(&mut self, g: &Group, to: Pos, angle: Angle, form: i32, queue: QueuePos) {
        let Some(st) = self.gstate_mut(g) else { return };
        st.form = form;
        if queue != QueuePos::First {
            st.o = to;
            st.o_angle = angle;
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
        let off = st.off.clone();
        let theta = self.units[member].movement.heading;
        let curr = Sim::form_update_positions(&off, theta);
        if let Some(st) = self.gstate_mut(g) {
            st.curr = curr;
        }
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
    /// Only `Unit::move_step`'s call is modelled — the one a marching unit
    /// makes every frame. The other seventeen callers (`do_build`,
    /// `do_gather`, `fight`, `come_out`, …) are turns this simulation does
    /// not yet make, and each is a place a group's flag would move that
    /// this one leaves still.
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
        let turned = reversing(Angle(
            angle.0.wrapping_sub(self.units[u].movement.heading.0),
        ));
        self.units[u].movement.heading = angle;
        // `605424`: the unit's own mirror flips before the leader test.
        if turned {
            self.units[u].movement.mirror = !self.units[u].movement.mirror;
        }
        let from = self.units[u].movement.body.pos;
        self.crew_des(u, from, angle, false);
        if !turned {
            return;
        }
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(st) = self.gstate_mut(&g)
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
            && let Some(st) = self.gstate_mut(&g)
        {
            st.facing = f;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_load::role;
    use crate::combat;
    use crate::{Unit, UnitType};

    /// A two-player world with an AI leader 1 at war with the human 0.
    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        s.nation[0].human = true;
        s.nation[1].human = false;
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        s
    }

    /// Every hand-built combatant carries `role::MILITARY`, which is what
    /// `UnitTypeData::get_stance_type` keys on — for a type loaded from the
    /// install `ai_load` derives it from the same facts the original does.
    fn fighter(sim: &mut Sim) -> usize {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        t.cols.role |= role::MILITARY;
        sim.add_unit_type(t)
    }

    /// An air-domain type. Without `unit_flags` bit `f` it is a plane
    /// (`UnitData::is_plane`); with it, a helicopter — which is not.
    fn flier(sim: &mut Sim, helicopter: bool) -> usize {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                domain: Domain::Air,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        if helicopter {
            t.cols.unit_flags |= uflags::HELICOPTER;
        }
        t.cols.role |= role::MILITARY;
        sim.add_unit_type(t)
    }

    /// who=0's Airbase and who=1's other building, as chapter seventeen
    /// stages them (`docs/GOLDEN.md` §25).
    fn airbase_and_target(sim: &mut Sim) -> (usize, usize) {
        let airbase = sim.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::Airbase,
            ..crate::build::BuildType::default()
        });
        let base = sim.add_building(0, Pos::new(11616, 13920), 0);
        sim.buildings[base].ty = Some(airbase);
        let target = sim.add_building(1, Pos::new(21120, 16512), 0);
        (base, target)
    }

    /// **A flight home is one `StrafeOrder` with no target, going home**
    /// (`Group::action_flight@006fb260`'s `MOVE_TO` arm →
    /// `Unit::add_strafe_order(−1, −1, base, who, 1, QUEUE_NEW, 1)`,
    /// `docs/ORDERS.md` §32; run223's blocks 642 and 662).
    #[test]
    fn a_flight_home_is_one_strafe_with_no_target_going_home() {
        let mut s = sim();
        let t = flier(&mut s, false);
        let u = spawn(&mut s, 0, t, Pos::new(11616, 16224));
        let (base, _) = airbase_and_target(&mut s);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
        let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
        assert_eq!(o.len(), 1, "one order");
        assert_eq!(o[0].index(), index::STRAFE);
        assert!(o[0].has(flag::ACTION), "the action bit");
        let Body::Strafe(sf) = o[0].body else {
            panic!("a strafe")
        };
        assert_eq!(sf.target, None);
        assert_eq!(sf.at, None);
        assert!(sf.returning && sf.mandatory);
        assert_eq!(sf.home, Some(base));
        assert_eq!(sf.cruising_alt, 0x640);
    }

    /// **An aircraft on the ground outside a base takes no strike**: an
    /// `ATTACK` flight needs the member "inside" — its `inside_up`, or an
    /// air order's home — and `add` placed it outside (run223's block
    /// 622, where the original gave the pair nothing).
    #[test]
    fn an_unbased_aircraft_on_the_ground_takes_no_strike() {
        let mut s = sim();
        let t = flier(&mut s, false);
        let u = spawn(&mut s, 0, t, Pos::new(10104, 16248));
        let (_, target) = airbase_and_target(&mut s);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        assert!(
            s.units[u].orders.is_empty(),
            "no order: the member is skipped"
        );
    }

    /// **A strike re-points a flying strafe and adds nothing**: the STRAFE
    /// arm writes the target and its point, `returning 0`, and keeps the
    /// home (run223's block 665, the order the original then turns into
    /// an air patrol over the unseen target's point).
    #[test]
    fn a_strike_re_points_a_flying_strafe() {
        let mut s = sim();
        let t = flier(&mut s, false);
        let u = spawn(&mut s, 0, t, Pos::new(10104, 16248));
        let (base, target) = airbase_and_target(&mut s);
        let g = group_of(0, &[u]);
        s.group_action_flight(&g, Obj::Building(base), Flight::Home);
        s.group_action_flight(&g, Obj::Building(target), Flight::Strike);
        let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
        assert_eq!(o.len(), 1, "still one order");
        let Body::Strafe(sf) = o[0].body else {
            panic!("a strafe")
        };
        assert_eq!(sf.target, Some(Obj::Building(target)));
        assert_eq!(sf.at, Some(Pos::new(21120, 16512)));
        assert!(!sf.returning && sf.mandatory);
        assert_eq!(sf.home, Some(base));
    }

    /// A plane on run223's numbers: `MOVES 75`, air domain, its figure
    /// seated, pointed where the test wants it. `bomber` puts its type on
    /// the `0x130` line.
    fn plane(s: &mut Sim, at: Pos, heading: Angle, bomber: bool) -> usize {
        let t = UnitType {
            hits: 100,
            moves: 75,
            turn_speed: crate::movement::degrees_to_angle(10).0,
            kind: crate::attrition::UnitKind {
                domain: Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                domain: Domain::Air,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        let ty = s.add_unit_type(t);
        if bomber {
            // After the add, which indexes the leaders' tech bits by it: a
            // hand-built tree holds no `0x130`, and `is` answers the
            // equality alone.
            s.unit_types[ty].tree = Some(0x130);
        }
        let u = spawn(s, 0, ty, at);
        s.units[u].kind = s.unit_types[ty].kind;
        s.units[u].movement.speed = 75;
        s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
        s.units[u].movement.heading = heading;
        s.units[u].movement.facing = heading;
        s.init_guys(u, Some(ty));
        u
    }

    /// **The Fighter's first frame home is run223's block 642, field for
    /// field** (`Unit::do_air_physics@005e86d0`, `docs/ORDERS.md` §33).
    /// `check_fuel` aims at the landing point pushed half the distance
    /// back along the approach, (11424, 15089); the returning bank wants
    /// its double, clamps at 55 and steps 10, and turns twice by the
    /// half-degree floor; banked, the plane flies at three quarters, and
    /// within `0x600` of its point at half that, 28; the pitch climbs 2 and
    /// lifts the figure 2. Made to fail with each arm changed: without the
    /// second turn the heading is one floor short, without the halving the
    /// step is (48, 27).
    #[test]
    fn a_fighter_s_first_frame_home_is_run223_s_642() {
        let mut s = sim();
        let (base, _) = airbase_and_target(&mut s);
        let u = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
        let seed = s.rng.seed;
        // Frame 641: `(0 + 641) & 7 != 0`, no redraw.
        assert_eq!(s.plane_air_physics(u, None, 641), crate::air::Flew::On);
        assert_eq!(s.rng.seed, seed, "no draw off the eighth frame");
        assert_eq!(s.units[u].pos, Pos::new(11664, 16262), "run223's 642");
        assert_eq!(s.units[u].movement.heading, Angle(1_419_725_301));
        assert_eq!(s.units[u].path[0].to, Pos::new(11424, 15089));
        let af = s.units[u].airframe;
        assert_eq!(af.bank.bits(), 0x4120_0000, "bank 10, the guy's sign");
        assert_eq!(af.pitch.bits(), 0x4000_0000, "pitch 2");
        assert_eq!((af.z, af.last_z), (2, 0), "the climb's first step");
    }

    /// **The redraw is a non-bomber's, every eighth frame phased by `o`**
    /// (`Unit::do_air_physics+0xba`): `(r % 7 + 13) · 100`, and a Bomber
    /// holds `0x640` and throws nothing. Made to fail by dropping the
    /// bomber test (a draw appears) and by un-phasing the cadence (the
    /// Fighter draws on 648 and not on 647).
    #[test]
    fn only_a_non_bomber_redraws_its_altitude_and_only_on_its_eighth_frame() {
        let mut s = sim();
        let (base, _) = airbase_and_target(&mut s);
        let b = plane(&mut s, Pos::new(13176, 16248), Angle(0x5555_5555), true);
        let f = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
        assert_eq!((s.units[b].index, s.units[f].index), (0, 1));
        s.group_action_flight(&group_of(0, &[f, b]), Obj::Building(base), Flight::Home);
        let alt = |s: &Sim, u: usize| match s.units[u].orders.front().map(|o| o.body) {
            Some(Body::Strafe(sf)) => sf.cruising_alt,
            _ => panic!("a strafe"),
        };
        // 648: `(0 + 648) & 7 == 0` is the Bomber's, and it draws nothing.
        let seed = s.rng.seed;
        s.plane_air_physics(b, None, 648);
        assert_eq!(s.rng.seed, seed, "a Bomber throws no redraw");
        assert_eq!(alt(&s, b), 0x640);
        // 648 is not the Fighter's (`o` 1): nothing.
        s.plane_air_physics(f, None, 648);
        assert_eq!(s.rng.seed, seed, "off its eighth frame");
        // 647 is: `(1 + 647) & 7 == 0`.
        let mut probe = s.rng;
        let want = (probe.roll() % 7 + 13) * 100;
        s.plane_air_physics(f, None, 647);
        assert_eq!(s.rng, probe, "one draw");
        assert_eq!(alt(&s, f), want);
    }

    /// **The landing** (`Unit::land_plane@005e9950`): inside a step and a
    /// half of the landing point the plane lands — the strafe home
    /// cleared, the plane inside its base, off the map, on the point it
    /// reached, its attitude kept. Made to fail by testing the landing
    /// against a single step (it flies on).
    #[test]
    fn a_plane_inside_a_step_and_a_half_of_its_point_lands_in_its_base() {
        let mut s = sim();
        let (base, _) = airbase_and_target(&mut s);
        let at = Pos::new(11424, 13920 + 100);
        let u = plane(&mut s, at, Angle(0), false);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
        assert_eq!(s.plane_air_physics(u, None, 721), crate::air::Flew::Done);
        assert!(s.units[u].orders.is_empty(), "the strafe home is gone");
        assert!(s.units[u].path.is_empty());
        assert_eq!(s.units[u].inside, Some(base));
        assert!(!s.units[u].on_map);
        assert_eq!(s.units[u].pos, at, "it keeps its point");
    }

    /// **End to end: a flight home is flown and lands** — the flight
    /// command, then whole frames of `Sim::tick`, through `do_strafe`,
    /// the unit loop and the figure. The Fighter comes round from 120°
    /// onto the north-bound approach, lands inside its base, spends one
    /// redraw on each eighth frame it flew and nothing else, and its
    /// figure takes the standing arm on the landing frame alone. Made to
    /// fail with the landed plane's figure frame removed from
    /// `process_unit` (`stopped` stays false).
    #[test]
    fn a_flight_home_is_flown_to_its_base_and_lands() {
        let mut s = sim();
        s.trace_phases = true;
        let (base, _) = airbase_and_target(&mut s);
        let u = plane(&mut s, Pos::new(11640, 16248), Angle(0x5555_5555), false);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(base), Flight::Home);
        let mut flown = 0;
        let mut eighths = 0;
        for _ in 0..400 {
            if s.units[u].inside.is_some() {
                break;
            }
            if (i64::from(s.units[u].index) + s.frame) & 7 == 0 {
                eighths += 1;
            }
            s.phase_marks.clear();
            s.tick();
            flown += 1;
            let draws = s
                .phase_marks
                .iter()
                .filter(|(l, _)| l == crate::air::SITE_AIR_ALT)
                .count();
            assert!(draws <= 1, "one redraw a frame at most");
        }
        assert_eq!(s.units[u].inside, Some(base), "landed after {flown} frames");
        assert!(flown > 20, "it flew: {flown} frames");
        assert!(eighths >= 2);
        let g = s.units[u].guys[0];
        assert!(g.stopped, "the landing frame's standing arm");
        assert_eq!(s.units[u].movement.body.last_speed, 0);
        // The inside arm puts `last_z` onto `z` from the next frame on.
        s.tick();
        let af = s.units[u].airframe;
        assert_eq!(af.last_z, af.z);
    }

    /// A Bomber on run223's numbers (`unitrules.xml`'s `Bomber`): `MOVES
    /// 60`, `TURN_SPEED 3`, `RANGE 1-3`, `RECHARGE 30`, on the `0x130`
    /// line, its figure seated at `z`, banked `bank` (the guy's sign) and
    /// pitched `pitch`, all by their bits.
    fn bomber(s: &mut Sim, at: Pos, heading: Angle) -> usize {
        let t = UnitType {
            hits: 300,
            moves: 60,
            turn_speed: crate::movement::degrees_to_angle(3).0,
            kind: crate::attrition::UnitKind {
                domain: Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            combat: combat::Profile {
                attack: 43,
                uber_size: 1,
                domain: Domain::Air,
                min_range: 1,
                max_range: 3,
                recharge: 30,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        let ty = s.add_unit_type(t);
        s.unit_types[ty].tree = Some(0x130);
        let u = spawn(s, 0, ty, at);
        s.units[u].kind = s.unit_types[ty].kind;
        s.units[u].movement.speed = 60;
        s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
        s.units[u].movement.heading = heading;
        s.units[u].movement.facing = heading;
        s.init_guys(u, Some(ty));
        u
    }

    /// who=1's building made a target: started, active, a combat profile
    /// and run223's Barracks' 1200 hit points.
    fn live_barracks(s: &mut Sim, b: usize) {
        let bd = &mut s.buildings[b];
        bd.started = true;
        bd.active = true;
        bd.combat = Some(combat::Profile {
            armor: 1,
            ..combat::Profile::default()
        });
        bd.health = 1200;
    }

    /// **A strike it may not take is a patrol over the strike's point,
    /// flown in the same frame: run223's `0/7`, 665 → 666, field for
    /// field** (`Unit::do_strafe@005eab00`, `docs/ORDERS.md` §34). The
    /// strike is re-pointed at a target `valid_target` refuses (here: no
    /// war; on run223, unseen), so `do_strafe` kills it, adds an
    /// `AirPatrolOrder` over `xx/yy` with no action bit, and `work` flies
    /// it at once. Within 199 of the ground the patrol's bank wants none,
    /// so the 40° it came off the runway with steps to 30; the pitch's
    /// rate divides by `0x240 / speed`, 9, and climbs 8 → 10, lifting the
    /// figure 12. Made to fail with each arm changed: without `work`'s
    /// re-entry the plane stands; without the low-bank arm the heading
    /// and the bank part; with the distance as the divisor the pitch
    /// holds at 8 and the figure rises 10.
    #[test]
    fn a_strike_it_may_not_take_is_a_patrol_flown_the_same_frame() {
        let mut s = sim();
        let (base, target) = airbase_and_target(&mut s);
        s.at_war[0][1] = false;
        s.at_war[1][0] = false;
        let u = bomber(&mut s, Pos::new(10263, 16330), Angle(1_341_635_061));
        let g = group_of(0, &[u]);
        s.group_action_flight(&g, Obj::Building(base), Flight::Home);
        s.group_action_flight(&g, Obj::Building(target), Flight::Strike);
        {
            let af = &mut s.units[u].airframe;
            af.bank = crate::single::Single::from_bits(0x4220_0000);
            af.pitch = crate::single::Single::from_bits(0x4100_0000);
            af.z = 24;
        }
        s.work(u, 665);
        let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
        assert_eq!(o.len(), 1, "the patrol alone");
        assert_eq!(o[0].index(), index::AIR_PATROL);
        assert_eq!(o[0].flags, 0, "no action bit: slot +0x2c is xor eax, eax");
        let Body::AirPatrol(p) = o[0].body else {
            panic!("an air patrol")
        };
        assert_eq!(p.point, Pos::new(21120, 16512), "the strike's xx/yy");
        assert_eq!((p.home, p.waypoint, p.returning), (Some(base), 0, false));
        assert_eq!(p.cruising_alt, 0x640);
        assert_eq!(s.units[u].pos, Pos::new(10319, 16351), "run223's 666");
        assert_eq!(s.units[u].movement.heading, Angle(1_315_604_981));
        assert_eq!(s.units[u].path[0].to, Pos::new(21120, 16512));
        let af = s.units[u].airframe;
        assert_eq!(af.bank.bits(), 0x41f0_0000, "bank 40 → 30");
        assert_eq!(af.pitch.bits(), 0x4120_0000, "pitch 8 → 10");
        assert_eq!((af.z, af.last_z), (36, 24));
    }

    /// **The patrol's search pushes its strike first, and takes no
    /// `update_action`** (`Unit::do_air_patrol@005ea620`,
    /// `add_strafe_order(…, QUEUE_FIRST, 0)@005e48c0`, `docs/ORDERS.md`
    /// §34.5): on the sixteenth frame phased by `o` a Bomber asks
    /// `find_new_bomber_target` round the patrol's point, and pushes a
    /// strafe with `mandatory 0` and no action bit in front of the patrol;
    /// `orders_x/y` keep the point `work` wrote before the step (run223's
    /// 777).
    /// Made to fail with the generic enqueue's `update_action` put back
    /// (`orders_pos` is the new point) and with the phase dropped (the
    /// search runs on 783).
    #[test]
    fn the_patrol_s_search_pushes_its_strike_first_without_update_action() {
        let mut s = sim();
        let (base, target) = airbase_and_target(&mut s);
        live_barracks(&mut s, target);
        let u = bomber(&mut s, Pos::new(18803, 16729), Angle(1_014_693_888));
        s.units[u].airframe.z = 1656;
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), false);
        // `o` 0: its sixteenth frames are `frame & 15 == 0`, 784 where
        // run223's `0/8` searched on 776.
        assert_eq!(s.units[u].index, 0);
        s.work(u, 783);
        assert_eq!(s.units[u].orders.len(), 1, "783 is not its sixteenth frame");
        let before = s.units[u].pos;
        s.work(u, 784);
        assert_ne!(s.units[u].pos, before, "it flew first");
        let o: Vec<Order> = s.units[u].orders.iter().copied().collect();
        assert_eq!(o.len(), 2, "the strike in front of the patrol");
        assert_eq!(o[0].index(), index::STRAFE);
        assert_eq!(o[0].flags, 0, "no action bit");
        let Body::Strafe(sf) = o[0].body else {
            panic!("a strafe")
        };
        assert_eq!(sf.target, Some(Obj::Building(target)));
        assert!(!sf.mandatory && !sf.returning);
        assert_eq!(sf.home, Some(base));
        assert_eq!(o[1].index(), index::AIR_PATROL);
        // `work`'s own `update_action` ran before the step, so the point
        // is the frame's first; the push after the step leaves it.
        assert_eq!(
            s.units[u].orders_pos, before,
            "no update_action on QUEUE_FIRST"
        );
        assert_ne!(s.units[u].orders_pos, s.units[u].pos);
    }

    /// **A Bomber releases in range and within 15° of its nose, once, and
    /// reloads `recharge() + 1`** (`do_strafe+0x9d0`, `docs/ORDERS.md`
    /// §34.3): `CHAR_ATTACK2` on the figure and `recharging 31`, which is
    /// run223's `0/8` on 806; reloading, it flies on without a second.
    /// Twenty degrees off the nose, it holds its bomb. Made to fail with
    /// the angle test dropped (the off-line plane bombs) and with the
    /// `+ 1` dropped (`recharging 30`).
    #[test]
    fn a_bomber_releases_in_range_and_on_its_nose() {
        let mut s = sim();
        let (base, target) = airbase_and_target(&mut s);
        live_barracks(&mut s, target);
        for (anim, len) in [
            (crate::anim::WALK, 31),
            (crate::anim::SLOG, 31),
            (crate::anim::ATTACK2, 30),
        ] {
            s.art.lengths.insert((-1, anim), len);
        }
        let run = |s: &mut Sim, heading: Angle| {
            let u = bomber(s, Pos::new(21120 - 400, 16512), heading);
            s.units[u].airframe.z = 1632;
            s.add_strafe_order(
                u,
                Some(Obj::Building(target)),
                Some(base),
                true,
                crate::orders::QueuePos::New,
                false,
            );
            s.work(u, 805);
            u
        };
        let u = run(&mut s, Angle::EAST);
        assert_eq!(s.units[u].guys[0].anim, crate::anim::ATTACK2);
        assert_eq!(s.units[u].combat.recharging, 31, "recharge() + 1");
        s.work(u, 806);
        assert_eq!(s.units[u].combat.recharging, 31, "no second release");
        let off = Angle(Angle::EAST.0 + crate::movement::degrees_to_angle(20).0);
        let v = run(&mut s, off);
        assert_eq!(s.units[v].combat.recharging, 0, "20° off: no release");
        assert_ne!(s.units[v].guys[0].anim, crate::anim::ATTACK2);
    }

    /// **A strafe releases its bomb, and the bomb falls** (item 770,
    /// `docs/ORDERS.md` §35). The Bomber's release on its first
    /// `CHAR_ATTACK2` frame is a round under the strafe's own target; it
    /// leaves `node 0`'s bay 19 under the plane's altitude, lands one tile
    /// (192) ahead of the bay along the heading with no draw, and takes
    /// the fall's 17 frames from 1613 to the ground — run235's `0/8` on
    /// 806, `total_time 17`, `sz 1613`, `ex − sx` 191 at 83°. Made to fail
    /// with the strafe dropped from the release gate (no round), the
    /// ground's height for the plane's (`sz`), the scatter arm kept for a
    /// Bomber (the draws) and the fall for the flight (`total_time`).
    #[test]
    fn a_strafe_releases_a_bomb_that_falls_a_tile_ahead() {
        let mut s = sim();
        let (base, target) = airbase_and_target(&mut s);
        live_barracks(&mut s, target);
        // run223's Barracks is four tiles square: the bomb lands inside.
        if let Some(c) = s.buildings[target].combat.as_mut() {
            c.x_size = 4;
            c.y_size = 4;
        }
        for (anim, len) in [
            (crate::anim::WALK, 31),
            (crate::anim::SLOG, 31),
            (crate::anim::ATTACK2, 30),
        ] {
            s.art.lengths.insert((-1, anim), len);
        }
        let at = Pos::new(21120 - 400, 16512);
        let u = bomber(&mut s, at, Angle::EAST);
        s.units[u].airframe.z = 1632;
        let piece = s.units[u].guys[0].gpiece;
        s.art.releases.insert(
            piece,
            [(
                crate::anim::ATTACK2,
                vec![1, 2, 5, 8, 10, 13, 16, 18, 21, 24],
            )]
            .into_iter()
            .collect(),
        );
        s.add_strafe_order(
            u,
            Some(Obj::Building(target)),
            Some(base),
            true,
            crate::orders::QueuePos::New,
            false,
        );
        s.work(u, 805);
        assert_eq!(s.units[u].guys[0].anim, crate::anim::ATTACK2);
        let seed = s.rng.seed;
        s.guys_inc_time();
        assert_eq!(s.projectiles.len(), 1, "the first event's round");
        assert_eq!(s.rng.seed, seed, "a bomb spends no draw");
        let p = s.projectiles[0];
        assert_eq!(p.target, Some(Obj::Building(target)));
        let from = s.units[u].pos;
        assert_eq!(
            p.launch,
            crate::launch::launch_point(from, Angle::EAST, piece, crate::anim::ATTACK2, 1)
        );
        // The figure's altitude as the release reads it, after `work`'s
        // climb: not the ground's.
        assert_eq!(
            p.sz,
            s.units[u].airframe.z
                + crate::launch::release_dz(piece, crate::anim::ATTACK2, 1).unwrap_or(0)
        );
        assert!(p.sz > 1500);
        assert_eq!(p.landing, Pos::new(p.launch.x + 192, p.launch.y));
        assert_eq!(p.total_time, combat::fall_time(p.sz, p.ez).max(1));
        assert_eq!(p.total_time, 17);
        assert!(!p.rolling);
        for _ in 0..16 {
            s.process_projectiles(806);
        }
        assert_eq!(s.projectiles.len(), 1, "still falling after 16");
        s.process_projectiles(822);
        assert!(s.projectiles.is_empty(), "down on the seventeenth");
        assert!(
            s.hits.iter().any(|h| h.target == Obj::Building(target)),
            "and it strikes"
        );
    }

    /// **A plane lights the fog it flies over** — `Unit::set_new_location`'s
    /// half-cell test and `update_seen(param_3 == 0)`, the ring pass,
    /// which `do_air_physics`' step reaches like any other
    /// (`docs/ORDERS.md` §34.5). It is how run223's pair first see the
    /// Barracks between their searches on 760 and 776. Made to fail with
    /// the reveal dropped from the plane's step (no new cell).
    #[test]
    fn a_flying_plane_lights_the_fog_it_crosses() {
        let mut s = sim();
        assert!(s.world.set_fog(vec![0; 60 * 60 * 4]));
        let (base, _) = airbase_and_target(&mut s);
        let mut t = UnitType {
            hits: 300,
            moves: 60,
            los: 16,
            ..UnitType::default()
        };
        t.kind.domain = Domain::Air;
        t.combat.domain = Domain::Air;
        let ty = s.add_unit_type(t);
        let u = spawn(&mut s, 0, ty, Pos::new(10000, 16000));
        s.units[u].kind = s.unit_types[ty].kind;
        s.units[u].movement.speed = 60;
        s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
        s.units[u].movement.heading = Angle::EAST;
        s.init_guys(u, Some(ty));
        s.add_air_patrol_order(u, Pos::new(30000, 16000), Some(base), false);
        let lit = |s: &Sim| {
            (0..s.world.fog_ys())
                .flat_map(|y| (0..s.world.fog_xs()).map(move |x| (x, y)))
                .filter(|&(x, y)| s.world.seen(x, y).unwrap_or(0) & 1 != 0)
                .count()
        };
        let before = lit(&s);
        for f in 0..20 {
            s.work(u, 801 + f);
        }
        assert!(s.units[u].pos.x > 11000, "it flew east");
        assert!(lit(&s) > before, "the rim it crossed is lit");
    }

    fn siege_type(sim: &mut Sim) -> usize {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 40,
                uber_size: 1,
                siege: true,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        t.cols.role |= role::MILITARY;
        sim.add_unit_type(t)
    }

    fn spawn(sim: &mut Sim, who: Player, ty: usize, p: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.movement.speed = 25;
        sim.add_unit(u)
    }

    fn group_of(who: Player, list: &[usize]) -> Group {
        Group {
            who,
            army: None,
            pushed: None,
            list: list.to_vec(),
        }
    }

    /// An armed type in one `type_cat` category, by the mask that puts it
    /// there (`docs/GROUPS.md` §6.4, `sim::form::type_cat`).
    fn typed(sim: &mut Sim, obj_masks: u32, max_range: i32) -> usize {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                obj_masks,
                max_range,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        t.cols.role |= role::MILITARY;
        sim.add_unit_type(t)
    }

    /// `find_leader`'s key is `FormData::type_cat` and nothing else
    /// (`docs/GROUPS.md` §4.4, listing `0070ccb0`): the **strictly**
    /// lowest category wins, so a tie goes to the first, and the first
    /// qualifying member leads until something lower turns up.
    ///
    /// The old simulation took the first on-map captain outright, which is
    /// only right for a group of one category — and every group in every
    /// dump before run31 was exactly that, which is why no diff caught it.
    /// run31 is the capture and it agrees:
    /// `run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns`
    /// stands its twelve squads up in the harness and `find_leader` names
    /// the first **hoplite** captain, member 12 of 36, where the old rule
    /// would have named the slinger at member 0.
    #[test]
    fn find_leader_takes_the_lowest_type_cat_and_ties_to_the_first() {
        use crate::combat::mask;
        use crate::form::{cat, type_cat};
        let mut s = sim();
        let foot = typed(&mut s, mask::FOOT, 0);
        let archer = typed(&mut s, mask::FOOT, 0x600);
        let mounted = typed(&mut s, mask::MOUNTED, 0);
        let mech = typed(&mut s, mask::VEHICLE, 0);
        let human = s.nation[1].human;
        assert_eq!(type_cat(&s.unit_types[mech], mech, human), cat::MECH);
        assert_eq!(
            type_cat(&s.unit_types[mounted], mounted, human),
            cat::MOUNTED
        );
        assert_eq!(type_cat(&s.unit_types[foot], foot, human), cat::FOOT);
        assert_eq!(
            type_cat(&s.unit_types[archer], archer, human),
            cat::FOOT_RANGED
        );

        let at =
            |s: &mut Sim, t: usize, n: i32| spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
        let (a, b, c) = (
            at(&mut s, archer, 0),
            at(&mut s, foot, 1),
            at(&mut s, mounted, 2),
        );
        let d = at(&mut s, mech, 3);
        // Join order is archer, foot, mounted, mech — increasingly early
        // in the category order, so each addition displaces the last.
        assert_eq!(s.group_find_leader(&group_of(1, &[a])), Some(a));
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b])), Some(b));
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c])), Some(c));
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(d));
        // And the other way round nothing displaces the first.
        assert_eq!(s.group_find_leader(&group_of(1, &[d, c, b, a])), Some(d));

        // A tie goes to the first: `cat < best` is strict.
        let e = at(&mut s, foot, 4);
        assert_eq!(s.group_find_leader(&group_of(1, &[b, e])), Some(b));
        assert_eq!(s.group_find_leader(&group_of(1, &[e, b])), Some(e));

        // Two passes, and only the first tests `is_on_map`: the mech is
        // the lowest category but garrisoned, so the on-map pass takes the
        // mounted unit and the second pass never runs.
        s.units[d].on_map = false;
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(c));
        // With nobody on the map at all the second pass finds it.
        for u in [a, b, c] {
            s.units[u].on_map = false;
        }
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), Some(d));
        // A dead member is never a leader, in either pass.
        for u in [a, b, c, d] {
            s.units[u].health = 0;
        }
        assert_eq!(s.group_find_leader(&group_of(1, &[a, b, c, d])), None);
    }

    /// `Group::add`'s two recursions (§4.1): a **captain** drags its figures
    /// in behind it, and a **figure** is refused and its captain taken
    /// instead. That is why a player's selection of twelve squads is a
    /// group of thirty-six, and it is the shape run31 dumps.
    #[test]
    fn adding_a_captain_takes_its_figures_and_adding_a_figure_takes_its_captain() {
        let mut s = sim();
        let t = fighter(&mut s);
        let squad = |s: &mut Sim, n: i32| {
            let c = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
            let f1 = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
            let f2 = spawn(s, 1, t, Pos::new(0x1000 + n * 0x80, 0x1000));
            for f in [f1, f2] {
                s.units[f].captain = false;
                s.units[f].o_up = Some(c);
            }
            s.units[c].o_down = Some(f1);
            s.units[f1].o_down = Some(f2);
            (c, f1, f2)
        };
        let (c0, a0, b0) = squad(&mut s, 0);
        let (c1, a1, b1) = squad(&mut s, 1);

        let mut g = Group::stack(1);
        s.group_add(&mut g, c0);
        s.group_add(&mut g, c1);
        assert_eq!(
            g.list,
            vec![c0, a0, b0, c1, a1, b1],
            "each captain followed by its own figures, in `o_down` order"
        );

        // A figure is never a member in its own right: asking for one adds
        // its captain, and with the captain already in, nothing happens.
        let mut h = Group::stack(1);
        s.group_add(&mut h, b1);
        assert_eq!(h.list, vec![c1, a1, b1], "the figure's whole squad");
        let before = h.list.clone();
        s.group_add(&mut h, a1);
        assert_eq!(h.list, before, "and a second ask is a no-op");

        // A dead figure is left out, and its own subordinate with it —
        // `+0x90 >= 0` is tested on the object, not the chain.
        s.units[a0].health = 0;
        let mut k = Group::stack(1);
        s.group_add(&mut k, c0);
        assert_eq!(k.list, vec![c0], "the chain stops at the dead figure");
    }

    /// `Group::refresh_group_order` (§6.8) — the second writer of `off`,
    /// and the one that makes run31's frame 204 different from its frame
    /// 328 without a single number of the layout changing.
    ///
    /// The block is slid so that the taking member sits on the origin, over
    /// `form_num` entries; `curr` is then re-rotated by **that** member's
    /// heading rather than the old leader's.
    #[test]
    fn a_refresh_re_origins_the_table_onto_the_member_that_took_it_over() {
        let mut st = GroupState {
            form_num: 3,
            off: vec![(0, 0), (-9, 0), (9, -3), (99, 99)],
            ..GroupState::default()
        };
        st.reorigin(1);
        assert_eq!(
            st.off,
            vec![(9, 0), (0, 0), (18, -3), (99, 99)],
            "slid by member 1's own slot, and only over form_num of them"
        );
        // Idempotent on the member it is already origined at.
        let again = st.off.clone();
        st.reorigin(1);
        assert_eq!(st.off, again);
        // A slot past `form_num` is not an origin the walk can reach, but
        // asking for one still slides the block it does reach.
        st.reorigin(2);
        assert_eq!(st.off, vec![(-9, 3), (-18, 3), (0, 0), (99, 99)]);
    }

    /// §6.3's **tail negation** — `bVar8` at `707e09` and the loop at
    /// `707eb8` — which is not the mirror above it and which this crate
    /// went without until item 267.
    ///
    /// With the caller supplying the angle (`set_angle`, which is every
    /// army call) `compute_form` measures its own `find_angle(dest −
    /// group_loc)` against it, and when the two are `90°` or more apart it
    /// negates every member's `off_x/off_y` on **both** the `Form` and the
    /// group *after* `Form::compute` has written them — leaving `to`
    /// alone. So two groups given the same destination and the same angle
    /// from opposite sides march to the **same slots** and hold
    /// **opposite offsets**, and the offsets are where each follower
    /// stands relative to its leader on every later frame
    /// (`Group::update_positions` → `curr` → `do_group_move`).
    ///
    /// Angle 0 is north, which is also a fresh unit's heading, so the
    /// `facing` toggle above cannot fire and the tail is measured alone.
    ///
    /// Made to fail on purpose twice: with the negation dropped the two
    /// offset tables agree, and with `to` negated alongside them the two
    /// destination lists no longer do.
    #[test]
    fn compute_form_negates_the_offsets_when_the_bearing_opposes_the_angle() {
        const DEST: Pos = Pos::new(0x4000, 0x4000);
        let run = |from: Pos| {
            let mut s = sim();
            // A type with a formation footprint — `fighter`'s spacing is
            // zero, and a table of zeroes negates to itself.
            let t = {
                let mut t = UnitType {
                    hits: 100,
                    combat: combat::Profile {
                        attack: 15,
                        uber_size: 1,
                        x_spacing: 0xc0,
                        y_spacing: 0x180,
                        ..combat::Profile::default()
                    },
                    ..UnitType::default()
                };
                t.cols.role |= role::MILITARY;
                s.add_unit_type(t)
            };
            let slot = s.init_army(1, None);
            let a = spawn(&mut s, 1, t, from);
            let b = spawn(&mut s, 1, t, Pos::new(from.x + 0x100, from.y));
            s.army_add_unit(1, slot, a);
            s.army_add_unit(1, slot, b);
            let g = s.army_group(1, slot);
            s.group_action_move_to(
                &g,
                DEST,
                QueuePos::New,
                true,
                Angle(0),
                MoveKind::AttackTo,
                true,
            );
            let off = s.armies[1].list[slot].group.off.clone();
            let dests: Vec<Pos> = [a, b]
                .iter()
                .map(|&u| match s.current_order(u).expect("a move order").body {
                    Body::Move(m) => m.dest,
                    _ => unreachable!("a group move is a move"),
                })
                .collect();
            (off, dests)
        };
        // South of the destination: the bearing to it is north, which is
        // the angle, so the tail does not fire.
        let (aligned, aligned_dests) = run(Pos::new(DEST.x, DEST.y + 0x2000));
        // North of it: the bearing is due south, 180° from the angle.
        let (opposed, opposed_dests) = run(Pos::new(DEST.x, DEST.y - 0x2000));

        assert!(
            aligned.iter().any(|&o| o != (0, 0)),
            "a table of zeroes negates to itself and says nothing: {aligned:?}"
        );
        assert_eq!(
            aligned_dests, opposed_dests,
            "the tail leaves the destinations alone"
        );
        assert_eq!(
            opposed,
            aligned
                .iter()
                .map(|&(x, y)| (-x, -y))
                .collect::<Vec<(i32, i32)>>(),
            "the offsets are negated: {aligned:?} against {opposed:?}"
        );
    }

    /// **The group's cap is a one-pass-lagged minimum, not a minimum**
    /// (§18), and the lag is the whole of item 515's residue — a
    /// `compute_speed`-shaped model walks the original's own raid at 25
    /// on frames it walks at 26.
    ///
    /// Made to fail on purpose three ways: with
    /// [`Sim::group_leader_report_speed`] writing `speed` from its own
    /// argument rather than from the accumulator, the second row is 26
    /// and the sixth is 26; with [`Sim::group_report_speed`] dropping
    /// its `<` test, the third row is 25 and the fifth is 30; with it
    /// writing only `new_speed`, the fourth row is 26.
    #[test]
    fn the_group_s_cap_lags_one_pass_behind_what_its_members_report() {
        let mut s = sim();
        let t = fighter(&mut s);
        let fast = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let slow = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        s.units[fast].movement.speed = 26;
        s.units[slow].movement.speed = 25;
        let mut g = group_of(1, &[fast, slow]);
        assert!(s.push_group(&mut g, true), "the pair takes a slot");
        let cap = |s: &Sim| s.group_speed_of(fast);

        // A freshly seated group caps nothing until something reports.
        assert_eq!(cap(&s), 0, "zero is `Group::clear`'s, and caps nothing");

        // **A zero cap is a trap, and it is the original's.**
        // `report_speed` only ever *lowers*, so a follower cannot get a
        // group off zero: the first two frames of a march out of a
        // freshly seated group run uncapped whatever the followers say,
        // and it takes the leader's second publication to seed the pair.
        // That is why [`Sim::group_normalize`] priming the group matters
        // at all — without it, Great Lakes' raid walks its first frames
        // at the raider's own 26.
        s.group_leader_report_speed(&g, 26);
        assert_eq!(cap(&s), 0, "the accumulator was empty, so the cap still is");
        s.group_report_speed(&g, 25);
        assert_eq!(
            cap(&s),
            0,
            "25 is not less than 0, so the report is dropped"
        );

        // Frame 2: the leader publishes the accumulator it restarted at
        // its own speed, and only now does the group cap anything.
        s.group_leader_report_speed(&g, 26);
        assert_eq!(cap(&s), 26, "the leader's own pass, one frame late");

        // And now a follower can drive it down, both halves at once.
        s.group_report_speed(&g, 25);
        assert_eq!(cap(&s), 25, "a report lowers");
        s.group_report_speed(&g, 30);
        assert_eq!(cap(&s), 25, "and never raises: 30 is not less than 25");

        // Frame 3: the leader publishes the pass the follower reported
        // into, so the cap **stays** at 25 while the accumulator
        // restarts at 26.
        s.group_leader_report_speed(&g, 26);
        assert_eq!(cap(&s), 25, "the pass the follower reported into");

        // Frame 4, with the follower no longer reporting — it has
        // arrived, or it is fighting rather than marching. **Now** the
        // cap drifts back up, one frame late. This is the row a
        // `compute_speed`-shaped model gets wrong, and it is five frames
        // of Great Lakes' raid.
        s.group_leader_report_speed(&g, 26);
        assert_eq!(cap(&s), 26, "no report, so the leader's own speed again");
    }

    /// **`Group::normalize`'s tail puts the cap back on its leader**
    /// (§18) — the site that primes a group before its first march, and
    /// the reason the first frame of Great Lakes' raid is capped rather
    /// than running at the raider's own speed.
    ///
    /// The leader is `find_leader`'s, so the **slow** type leads here
    /// only because its `type_cat` is lower; made to fail by giving both
    /// types the same category, which hands the lead to the first member
    /// of the list and the cap to 26.
    #[test]
    fn normalizing_a_group_puts_its_cap_back_on_its_leader() {
        use crate::combat::mask;
        let mut s = sim();
        // Mounted outranks foot in `type_cat`, so the mounted type
        // leads whichever order the list is in.
        let foot = typed(&mut s, mask::FOOT, 0);
        let horse = typed(&mut s, mask::MOUNTED, 0);
        let fast = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let slow = spawn(&mut s, 1, horse, Pos::new(0x1100, 0x1000));
        s.units[fast].movement.speed = 26;
        s.units[slow].movement.speed = 25;
        let mut g = group_of(1, &[fast, slow]);
        assert!(s.push_group(&mut g, true));
        assert_eq!(
            s.group_find_leader(&g),
            Some(slow),
            "the mounted type leads on `type_cat`, not on list order"
        );
        // Walk the cap up the way a march does, then normalize.
        s.group_leader_report_speed(&g, 26);
        s.group_leader_report_speed(&g, 26);
        assert_eq!(s.group_speed_of(fast), 26);
        s.group_normalize(&mut g);
        assert_eq!(
            s.group_speed_of(fast),
            25,
            "`compute_speed` is the leader's own speed, and it is the slow one"
        );
        assert_eq!(
            s.gstate(&g).map(|st| st.new_speed),
            Some(25),
            "and it writes both halves of the pair"
        );
    }

    /// **`Groups::process` resets one pool slot a frame, and the slot is
    /// the frame mod 64** (§19). This is the writer §18 did not count:
    /// Great Lakes' raid is player 1's slot 1, `group 65`, and the frame
    /// its cap goes back to the slow squad's 25 is 10241 — `65 mod 64`'s —
    /// with nobody in the group able to report.
    ///
    /// Made to fail on purpose three ways: with the cursor one frame late
    /// (`frame − 1`) the cap is still 26 after 10241; with the pass
    /// removed from nowhere but here, the same; with the reset writing
    /// only `speed`, the last row's `new_speed` is 26.
    #[test]
    fn groups_process_resets_one_pool_slot_a_frame_on_the_frame_mod_64() {
        use crate::combat::mask;
        let mut s = sim();
        let foot = typed(&mut s, mask::FOOT, 0);
        let horse = typed(&mut s, mask::MOUNTED, 0);
        let fast = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let slow = spawn(&mut s, 1, horse, Pos::new(0x1100, 0x1000));
        s.units[fast].movement.speed = 26;
        s.units[slow].movement.speed = 25;
        let mut g = group_of(1, &[fast, slow]);
        assert!(s.push_group(&mut g, true));
        assert_eq!(
            s.pool_group_of(fast),
            65,
            "a player's first push skips `last_group`'s initial slot 0"
        );
        // A march's reports walk the cap up to the leader's own 26 …
        s.group_leader_report_speed(&g, 26);
        s.group_leader_report_speed(&g, 26);
        assert_eq!(s.group_speed_of(fast), 26);
        // … and a pass on any other slot leaves it there.
        s.groups_process(10_240);
        assert_eq!(s.group_speed_of(fast), 26, "10240 is slot 0's frame");
        // 10241 is slot 1's: the cap goes back to `find_leader`'s speed,
        // the slow squad's, both halves.
        s.groups_process(10_241);
        assert_eq!(s.group_speed_of(fast), 25, "10241 is `65 mod 64`'s frame");
        assert_eq!(s.gstate(&g).map(|st| st.new_speed), Some(25));
        // And the next frame for the slot is 64 later.
        s.group_leader_report_speed(&g, 26);
        s.group_leader_report_speed(&g, 26);
        for f in 10_242..10_305 {
            s.groups_process(f);
        }
        assert_eq!(s.group_speed_of(fast), 26, "no reset between");
        s.groups_process(10_305);
        assert_eq!(s.group_speed_of(fast), 25, "and one at 10305");
    }

    /// **`push_group`'s numbering** (§3.1, §3.2, §19): a group equal to
    /// `last_group`'s reuses it, anything else takes the lowest empty slot
    /// that is not `last_group` — which is how Great Lakes' scout walks
    /// 65 → 64 → 66 across its pushes while one slot is held elsewhere.
    ///
    /// Made to fail on purpose twice: without the `last_group` exclusion
    /// the first push is 64; without the equality reuse the second is 64.
    #[test]
    fn a_push_takes_the_lowest_empty_slot_that_is_not_the_last() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        let push = |s: &mut Sim, list: &[usize]| {
            let mut g = group_of(1, list);
            assert!(s.push_group(&mut g, true));
            s.pool_group_of(list[0])
        };
        assert_eq!(push(&mut s, &[a]), 65, "slot 0 is `last_group` at start");
        assert_eq!(push(&mut s, &[a]), 65, "the same group again reuses it");
        assert_eq!(push(&mut s, &[b, c]), 64, "slot 0 is free and not the last");
        assert_eq!(
            push(&mut s, &[a]),
            66,
            "65 is not the last now, but `a` still holds it — and it leaves"
        );
        assert_eq!(push(&mut s, &[b]), 65, "65 emptied when `a` left it");
    }

    #[test]
    fn push_group_refuses_a_singleton_unless_forced() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let mut one = group_of(1, &[a]);
        assert!(
            !s.push_group(&mut one, false),
            "a group of fewer than two takes no slot without force"
        );
        assert!(s.push_group(&mut one, true), "Army::add_unit forces it");
        assert_eq!(one.pushed, Some(0), "and the forced push names its slot");
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        assert!(s.push_group(&mut group_of(1, &[a, b]), false));
    }

    /// **A pushed group is still a group the next frame** — the pool
    /// slot item 465 gave `push_group`, and the reason Great Lakes'
    /// probe can march in formation at all.
    ///
    /// The probe (`docs/ARMY.md` §12) pushes six of the army's raiders
    /// out of army 1 on frame 8186 and the dump carries them on
    /// `group 65` for the next two thousand blocks. With one slot and no
    /// record the scout's next `go_to` overwrote them — eleven times
    /// before the word — so `do_group_move` could not run on them and
    /// the whole follower arm was dead code on the only capture that
    /// reaches it (`docs/ORDERS.md` §16).
    ///
    /// **Made to fail on purpose**: with `pushed` back to one slot the
    /// second push takes the first group's seat and `group_of` answers
    /// `None` for `a` and `b`.
    #[test]
    fn a_pushed_group_outlives_the_next_push() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let c = spawn(&mut s, 1, t, Pos::new(0x4000, 0x4000));
        let mut first = group_of(1, &[a, b]);
        assert!(s.push_group(&mut first, true));
        let mut second = group_of(1, &[c]);
        assert!(s.push_group(&mut second, true));
        assert_eq!(
            (first.pushed, second.pushed),
            (Some(0), Some(1)),
            "two live groups take two slots"
        );
        assert_eq!(
            s.group_of(a).map(|g| g.list),
            Some(vec![a, b]),
            "the first group is still the first group's"
        );
        assert_eq!(
            s.group_of(c).map(|g| g.list),
            Some(vec![c]),
            "and the second is its own"
        );
        // `Groups::get_open_slot`: a slot whose members are all gone is
        // taken before a third is appended.
        s.units[c].health = 0;
        let d = spawn(&mut s, 1, t, Pos::new(0x5000, 0x5000));
        let mut third = group_of(1, &[d]);
        assert!(s.push_group(&mut third, true));
        assert_eq!(third.pushed, Some(1), "the dead group's slot is recycled");
        assert_eq!(s.pushed.len(), 2, "and the pool does not grow");
    }

    /// **A seated group's move is a `GroupMoveOrder`, and an in-danger
    /// member's is not** — §6.6 step 6's gate as the original states it
    /// (`705f00`-`705f61`), which has no army test in it at all.
    ///
    /// This crate carried one — `g.army.is_some()` — for four months,
    /// and it was standing in for the wrong thing: what the original
    /// exempts a `Unit::go_to` group by is `unit_masks & 4`, which
    /// `go_to_unit@005f78c0` sets over the whole squad one line above
    /// the walk. Both halves are asserted here, because dropping the
    /// army test without carrying the bit collapses Great Lakes' word to
    /// **6994** — the second squad's own walk to the army, which is a
    /// `go_to` (`docs/ORDERS.md` §16).
    #[test]
    fn a_pushed_group_marches_unless_its_member_is_in_danger() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let mut g = group_of(1, &[a, b]);
        assert!(s.push_group(&mut g, true));
        s.group_action_move_to(
            &g,
            Pos::new(0x8000, 0x8000),
            QueuePos::New,
            false,
            Angle(0),
            MoveKind::MoveTo,
            false,
        );
        let grouped = |s: &Sim, u: usize| match s.current_order(u).expect("a move").body {
            Body::Move(m) => m.group.is_some(),
            _ => unreachable!("a move is a move"),
        };
        assert!(
            grouped(&s, a) && grouped(&s, b),
            "a seated group of two on a MOVE_TO marches in formation"
        );
        // And the same group with one member marked: only that member
        // falls back to a plain move.
        s.units[a].in_danger = true;
        s.group_action_move_to(
            &g,
            Pos::new(0x2000, 0x8000),
            QueuePos::New,
            false,
            Angle(0),
            MoveKind::MoveTo,
            false,
        );
        assert_eq!(
            (grouped(&s, a), grouped(&s, b)),
            (false, true),
            "`unit_masks & 4` is a per-member exemption, not the group's"
        );
    }

    /// **A pushed group takes its members out of the army** — §3.2's
    /// third bullet (`docs/ARMY.md` §3.4), which the first reading had
    /// right and no implementation carried until item 350.
    ///
    /// Great Lakes is the case: §12's probe pushes two of the army's
    /// units, `Group::add` brings both squads, and the original's army
    /// issues every later order to the nine that are left. Here the
    /// fixture is the same shape at two units — push one of them, and the
    /// army is one unit, one captain, one standard.
    #[test]
    fn a_pushed_group_takes_its_members_out_of_the_army() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        assert_eq!(
            (
                s.armies[1].list[slot].units.clone(),
                s.armies[1].list[slot].num_captains
            ),
            (vec![a, b], 2),
            "the army's one group holds both"
        );
        // `push_group(force = 1)`, the probe's own call.
        assert!(s.push_group(&mut group_of(1, &[a]), true));
        assert_eq!(
            s.armies[1].list[slot].units,
            vec![b],
            "the pushed member is killed out of the army's group"
        );
        assert_eq!(
            (
                s.armies[1].list[slot].num_units,
                s.armies[1].list[slot].num_captains,
                s.armies[1].list[slot].num_standard
            ),
            (1, 1, 1),
            "and `Army::normalize` recounts what is left"
        );
        // A group the army never held leaves it alone.
        let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        assert!(s.push_group(&mut group_of(1, &[c]), true));
        assert_eq!(
            s.armies[1].list[slot].units,
            vec![b],
            "a non-member's push is not the army's business"
        );
    }

    /// An army of two on land takes **`GroupMoveOrder`s** and a lone one
    /// does not (`docs/ORDERS.md` §8.2's gate), and a group with no army
    /// takes none at all — this crate's own line, since a player's
    /// selection has no persistent group for `do_group_move` to read.
    #[test]
    fn an_army_s_move_is_a_group_order_and_a_selection_s_is_not() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        let leader = s.group_find_leader(&g).expect("a leader");
        for u in [a, b] {
            let m = s.current_move(u).expect("a move order");
            let gm = m.group.expect("a group order");
            assert_eq!(gm.leader, leader, "every member names one leader");
            assert_eq!(gm.id, group_move_id(slot as i32, s.frame, 0));
            assert!(!gm.in_group, "`GroupMoveOrder::clear` leaves it 0");
        }
        assert_eq!(
            s.current_move(a).expect("a move").group.expect("g").form_id,
            0
        );
        assert_eq!(
            s.current_move(b).expect("a move").group.expect("g").form_id,
            1
        );

        // The same two as a plain selection: no army, so no group order.
        let plain = group_of(1, &[a, b]);
        s.group_action_move_to(
            &plain,
            Pos::new(0x5000, 0x5000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        for u in [a, b] {
            assert!(
                s.current_move(u).expect("a move order").group.is_none(),
                "a selection has no group to hang the order on"
            );
        }

        // And an army of one: `group.num < 2` is an exemption of the
        // original's own.
        let c = spawn(&mut s, 1, t, Pos::new(0x2000, 0x2000));
        let solo = s.init_army(1, None);
        s.army_add_unit(1, solo, c);
        let one = s.army_group(1, solo);
        s.group_action_move_to(
            &one,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        assert!(
            s.current_move(c).expect("a move order").group.is_none(),
            "fewer than two is a plain move"
        );
    }

    /// **A `QUEUE_LAST` move is laid out from where the leader ends up**
    /// (`docs/ORDERS.md` §17.2): `Group::action_move_near` asks
    /// `GroupData::get_loc_to` for that one queue position and `get_loc`
    /// for every other (`00704990:361`–`364`), and the two answers are the
    /// whole formation angle when something is queued ahead.
    ///
    /// The scenario is Great Lakes' probe in miniature: a pair standing at
    /// the point they are about to be sent to, holding an attack order on a
    /// building far to the west. Read from their feet the delta is zero and
    /// the block has no bearing; read from the building it points east, and
    /// the order carries that bearing.
    ///
    /// **Made to fail on purpose** by asking `group_loc` for both positions
    /// — which is what this crate did until item 471 — whereupon the angle
    /// is the zero-delta arm's and the two members lay out on an axis.
    #[test]
    fn a_queued_group_move_takes_its_bearing_from_where_the_leader_ends_up() {
        let mut s = sim();
        let t = fighter(&mut s);
        s.unit_types[t].combat.x_spacing = 0xc0;
        s.unit_types[t].combat.y_spacing = 0xc0;
        let here = Pos::new(0x8000, 0x4000);
        let a = spawn(&mut s, 1, t, here);
        let b = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4200));
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        let leader = s.group_find_leader(&g).expect("a leader");
        // The thing queued ahead, and it is the probe's: a target far to
        // the west, which is where `get_final_loc` says the pair will be.
        let away = Pos::new(0x1000, 0x6000);
        let prey = spawn(&mut s, 0, t, away);
        s.units[leader].combat.target = Some(crate::combat::Obj::Unit(prey));
        s.group_action_attack(&g, crate::combat::Obj::Unit(prey), true, QueuePos::New, 0);
        // …and the walk home behind it, to the leader's **own** point.
        s.group_action_move_to(
            &g,
            here,
            QueuePos::Last,
            false,
            Angle(0),
            MoveKind::MoveTo,
            false,
        );
        let want = find_angle(here.x - away.x, here.y - away.y);
        let m = s
            .units
            .iter()
            .find_map(|u| u.orders.iter().find(|o| o.move_dest().is_some()))
            .and_then(|o| match o.body {
                Body::Move(m) => Some(m),
                _ => None,
            })
            .expect("a group move on the pair");
        assert_eq!(
            m.angle, want,
            "the formation angle is `find_angle` from the queued target, not \
             from the leader's feet"
        );
        assert_ne!(want, Angle(0), "the scenario has to have a bearing at all");
    }

    /// **A move to where the group already stands takes the leader's own
    /// heading**, not the record's `o_angle` (`docs/ORDERS.md` §17.4).
    ///
    /// `Group::compute_form`'s zero-delta arm has two halves
    /// (`707e3d`–`707e6d`) and this crate had neither: with the location
    /// different from the group's own `(ox, oy)` — which is every group that
    /// has not already been sent to this exact point — the angle is
    /// `leader.angle − ((signed char)angles[slot] << 24)`, and only a group
    /// standing where it was last sent falls through to `o_angle`.
    ///
    /// **No capture on disk reaches it**: every `QUEUE_NEW` group move in
    /// the scored windows has a non-zero delta, so this is a built
    /// scenario and the claim is the listing's.
    ///
    /// **Made to fail on purpose** by dropping the first half — which is
    /// what this crate did until item 471 — whereupon the angle is nought
    /// on a group whose leader is plainly pointing somewhere.
    #[test]
    fn a_group_sent_to_its_own_feet_lays_out_on_the_leader_s_heading() {
        let mut s = sim();
        let t = fighter(&mut s);
        s.unit_types[t].combat.x_spacing = 0xc0;
        s.unit_types[t].combat.y_spacing = 0xc0;
        let here = Pos::new(0x8000, 0x4000);
        let a = spawn(&mut s, 1, t, here);
        let b = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4200));
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        let leader = s.group_find_leader(&g).expect("a leader");
        let facing = Angle(0x2000_0000);
        s.units[leader].movement.heading = facing;
        s.group_action_move_to(
            &g,
            s.units[leader].pos,
            QueuePos::New,
            false,
            Angle(0),
            MoveKind::MoveTo,
            false,
        );
        let m = s.current_move(leader).expect("a move order");
        // The slot byte is nought on a group that has not been laid out,
        // so the arm's whole answer is the heading.
        assert_eq!(
            m.angle, facing,
            "a zero delta takes the leader's own heading less its slot byte"
        );
        assert_ne!(
            m.angle,
            Angle(0),
            "and not the `o_angle` of a group that has never moved"
        );
    }

    /// **`UnitData::get_final_loc@00608040` stops at the first order that
    /// names a point**, not the last (`docs/ORDERS.md` §17.3) — and an
    /// order naming none is walked past.
    ///
    /// Three orders on one unit: a think with nothing to say, then an
    /// attack, then a move. The answer is the attack's target, because it
    /// comes first; put the move in front and the answer is the move's
    /// destination.
    ///
    /// **Made to fail on purpose** with a `rev()` on the walk, which reads
    /// the move under the attack and is the natural way to write "where the
    /// list ends".
    #[test]
    fn get_final_loc_stops_at_the_first_order_that_names_a_point() {
        let mut s = sim();
        let t = fighter(&mut s);
        let u = spawn(&mut s, 1, t, Pos::new(0x8000, 0x4000));
        let prey = spawn(&mut s, 0, t, Pos::new(0x1000, 0x6000));
        assert_eq!(
            s.unit_final_loc(u),
            s.units[u].pos,
            "an empty list answers with the unit's own position"
        );
        let far = Pos::new(0x7000, 0x2000);
        s.add_attack_order(
            u,
            crate::combat::Obj::Unit(prey),
            QueuePos::New,
            true,
            false,
        );
        s.add_move_order(u, far, MoveKind::MoveTo, QueuePos::Last, false);
        // A `Think` in front, which names no point and must be walked past
        // rather than answered with.
        s.units[u].orders.push_front(Order {
            flags: 0,
            body: Body::Think,
        });
        let prey_at = s.units[prey].pos;
        // The adder snaps its destination to the 48-unit cell centre
        // (§4.3), so the order's own `dest` is what `get_final_loc` hands
        // back and `far` is not.
        let snapped = s.units[u]
            .orders
            .iter()
            .find_map(Order::move_dest)
            .expect("the move went on");
        assert_eq!(
            s.unit_final_loc(u),
            prey_at,
            "the attack is ahead of the move, so its target is the answer"
        );
        s.units[u].orders.swap(1, 2);
        assert_eq!(
            s.unit_final_loc(u),
            snapped,
            "with the move ahead of it, the move's destination is"
        );
        // `flags & 1` alone, and nothing about the map: a dead target is
        // walked past like an order that names nothing.
        s.units[u].orders.swap(1, 2);
        s.units[prey].health = 0;
        assert_eq!(
            s.unit_final_loc(u),
            snapped,
            "a dead target is not a point, so the walk goes on to the move"
        );
    }

    /// **The follower tracks the leader's current position, not the
    /// destination** (§8.3) — and the leader tracks its own path. That
    /// difference is the whole mechanic: one `do_move` a frame, not N.
    #[test]
    fn a_group_move_follower_walks_to_the_leader_s_position_plus_its_slot() {
        let mut s = sim();
        let t = fighter(&mut s);
        // A real `X_SPACING`, so `Form::compute` lays the two out side by
        // side rather than both on the origin — the shipped data never
        // leaves it at zero and a hand-built type would.
        s.unit_types[t].combat.x_spacing = 0xc0;
        s.unit_types[t].combat.y_spacing = 0xc0;
        // The second stands where the Line's slot for it will be — south
        // of the leader, the move going east — so it walks straight to it
        // rather than crossing behind.
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1200));
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        let leader = s.group_find_leader(&g).expect("a leader");
        let follower = if leader == a { b } else { a };
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x1000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        let i = g.list.iter().position(|&u| u == follower).expect("a slot");
        for _ in 0..4 {
            s.tick();
        }
        let want = s
            .group_slot_point(&g, leader, i)
            .expect("the leader's point plus my offset");
        let m = s.current_move(follower).expect("a move order");
        assert!(
            m.group.is_some_and(|x| x.in_group),
            "a follower on its slot is `in_group`"
        );
        assert_eq!(
            m.waypoint, want,
            "the waypoint is the leader's own position plus the rotated slot"
        );
        assert_ne!(
            m.waypoint, m.dest,
            "and it is not the destination the order carries"
        );
    }

    /// A squad `cap → mid → tail` whose back-pointers name a pushed slot,
    /// joining an army already holding `a` and `b`: Great Lakes 11424's
    /// shape (`docs/GROUPS.md` §23).
    fn a_squad_joins_a_small_army() -> (Sim, usize, [usize; 5]) {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let cap = spawn(&mut s, 1, t, Pos::new(0x2000, 0x2000));
        let mid = spawn(&mut s, 1, t, Pos::new(0x2030, 0x2000));
        let tail = spawn(&mut s, 1, t, Pos::new(0x2060, 0x2000));
        for (f, up) in [(mid, cap), (tail, mid)] {
            s.units[f].captain = false;
            s.units[f].o_up = Some(up);
            s.units[up].o_down = Some(f);
        }
        let slot = s.init_army(1, None);
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        // The squad's `come_out` group: its own slot, which is what its
        // three pointers name when the army's `Group::add` walks them.
        let mut g = Group::stack(1);
        s.group_add(&mut g, cap);
        assert!(s.push_group(&mut g, true));
        s.army_add_unit(1, slot, tail);
        (s, slot, [a, b, cap, mid, tail])
    }

    /// **`Group::add` normalizes a small group at every step**, so the
    /// squad's head and middle are dropped by the recursion that adds the
    /// figure after them, and only the tail is left listed — while
    /// `Unit::set_group` points all three at the army (`docs/GROUPS.md`
    /// §23, `Group::get_num`'s `num < 4` arm). Made to fail on purpose by
    /// skipping the per-step `get_num`: the list is then the whole squad.
    #[test]
    fn a_squad_joining_a_small_seated_group_is_listed_by_its_tail_alone() {
        let (s, slot, [a, b, cap, mid, tail]) = a_squad_joins_a_small_army();
        assert_eq!(s.armies[1].list[slot].units, vec![a, b, tail]);
        let pool = s.armies[1].list[slot].group.pool;
        for u in [cap, mid, tail] {
            assert_eq!(
                s.units[u].group_ptr, pool,
                "set_group points the squad at the army"
            );
        }
        assert_eq!(s.army_of(tail), Some(slot));
        assert_eq!(
            s.army_of(cap),
            None,
            "named but not listed: `Object::get_army` answers no army"
        );
    }

    /// **`Group::sort` kills a stray follower and re-adds its squad whole**,
    /// at the end of the list and naming nothing: the kill clears the
    /// pointer that names this group, and the `const` re-add writes none.
    /// The tail is then listed with no group, and its first
    /// `do_group_move` ungroups it (`docs/GROUPS.md` §23). Made to fail on
    /// purpose by skipping the sort.
    #[test]
    fn the_sort_re_seats_a_stray_follower_s_squad_and_clears_its_pointer() {
        let (mut s, slot, [a, b, cap, mid, tail]) = a_squad_joins_a_small_army();
        let pool = s.armies[1].list[slot].group.pool;
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        assert_eq!(s.armies[1].list[slot].units, vec![a, b, cap, mid, tail]);
        assert_eq!(s.units[tail].group_ptr, None, "the kill cleared it");
        assert_eq!(s.units[cap].group_ptr, pool, "the re-add wrote nothing");
        assert_eq!(s.units[mid].group_ptr, pool);
        assert_eq!(s.army_of(tail), None);
        // And the army's own normalize then drops it, as the pool's
        // per-frame pass does on the slot's frame.
        s.army_normalize(1, slot);
        assert_eq!(s.armies[1].list[slot].units, vec![a, b, cap, mid]);
    }

    /// `ungroup_move_order` walks **up to the captain and back down every
    /// subordinate**, so one member's ungroup is the squad's.
    #[test]
    fn an_ungroup_converts_the_whole_squad_s_orders_to_plain_moves() {
        let mut s = sim();
        let t = fighter(&mut s);
        let cap = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let sub = spawn(&mut s, 1, t, Pos::new(0x1030, 0x1000));
        let far = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        s.units[sub].captain = false;
        s.units[sub].o_up = Some(cap);
        s.units[cap].o_down = Some(sub);
        let slot = s.init_army(1, None);
        for u in [cap, sub, far] {
            s.army_add_unit(1, slot, u);
        }
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        let id = s
            .current_move(sub)
            .expect("a move")
            .group
            .expect("a group order")
            .id;
        s.ungroup_move_order(sub, id);
        for u in [cap, sub] {
            assert!(
                s.current_move(u).expect("a move").group.is_none(),
                "the captain and its subordinate both ungroup"
            );
        }
        assert!(
            s.current_move(far).expect("a move").group.is_some(),
            "and nobody outside the squad does"
        );
    }

    /// **The plain move an ungroup makes is put at the *head* of the
    /// order list, not left where the group order stood** (item 487,
    /// `docs/ORDERS.md` §20).
    ///
    /// `Unit::ungroup_move_order@005fd140`'s `GROUP_MOVE` arm ends with
    /// `remove_current` on the order list and then
    /// `LinkListBase<UnitOrder *, …>::add@0046d5a0`, which **prepends** —
    /// `head_node = new` — so the conversion is also a promotion. The two
    /// readings are the same thing for a unit whose group move is already
    /// the head order, which is every unit any capture on disk reached
    /// until run100's `1/29`: an AI raider carrying an `ATTACK` above its
    /// group move, on a farm that died ten frames earlier.
    ///
    /// Made to fail on purpose by restoring the in-place rewrite, which
    /// leaves the attack in front and the move buried under it.
    #[test]
    fn an_ungroup_puts_the_plain_move_at_the_head_of_the_list() {
        let mut s = sim();
        let t = fighter(&mut s);
        let cap = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let sub = spawn(&mut s, 1, t, Pos::new(0x1030, 0x1000));
        let prey = spawn(&mut s, 0, t, Pos::new(0x1400, 0x1000));
        s.units[sub].captain = false;
        s.units[sub].o_up = Some(cap);
        s.units[cap].o_down = Some(sub);
        let slot = s.init_army(1, None);
        for u in [cap, sub] {
            s.army_add_unit(1, slot, u);
        }
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        let id = s
            .current_move(sub)
            .expect("a move")
            .group
            .expect("a group order")
            .id;
        // The attack goes in front of the group move, which is the shape
        // run100's `1/29` is in on block 10240.
        s.add_attack_order(sub, Obj::Unit(prey), QueuePos::First, false, true);
        assert!(
            matches!(
                s.current_order(sub).map(|o| o.body),
                Some(crate::orders::Body::Attack(_))
            ),
            "the attack is the head order before the ungroup"
        );
        s.ungroup_move_order(sub, id);
        assert_eq!(
            s.units[sub]
                .orders
                .iter()
                .map(crate::orders::Order::index)
                .collect::<Vec<_>>(),
            vec![crate::orders::index::MOVE_TO, crate::orders::index::ATTACK],
            "the ungrouped move is the head order and the attack is under it"
        );
        assert!(
            s.current_move(sub).expect("a move").group.is_none(),
            "and it is a plain move"
        );
    }

    /// `do_group_move`'s window is a **third** of a turn, not the quarter
    /// `reversing` uses (`5e8167`).
    #[test]
    fn the_group_move_window_is_a_third_of_a_turn() {
        use crate::orders::within_third;
        assert!(within_third(Angle(0), Angle(0)));
        assert!(
            within_third(Angle(0x5555_5554), Angle(0)),
            "a hair under 120°"
        );
        assert!(
            !within_third(Angle(0x5555_5555), Angle(0)),
            "exactly 120° is out"
        );
        // The fold is the original's `not`, not a negate, so the far side
        // is one unit wider: `!(-0x55555555) == 0x55555554`, which passes.
        assert!(
            within_third(Angle(-0x5555_5555), Angle(0)),
            "the `not` fold makes the other side inclusive"
        );
        assert!(
            !within_third(Angle(-0x5555_5556), Angle(0)),
            "and a unit past it is out"
        );
        assert!(!within_third(Angle(0), Angle(i32::MIN)), "dead astern");
    }

    /// **A formation command makes no order of its own** (`docs/GOLDEN.md`
    /// §22, `docs/ORDERS.md` §30). A group that stands takes the byte and
    /// re-forms on the spot — a move to the leader's own point, **without**
    /// the action bit; a group that walks is halted and the leader's move
    /// replayed to the same point, **with** it. Neither holds a
    /// `CHANGE_FORM`.
    #[test]
    fn action_form_writes_the_byte_and_moves_the_group_it_does_not_order_a_form() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
        let g = group_of(0, &[a, b]);

        // Standing: the byte, and one move each at the leader's point.
        s.group_action_form(&g, 2, 0, QueuePos::New);
        let leader = s.group_find_leader(&g).expect("a leader");
        for u in [a, b] {
            assert_eq!(s.units[u].form, 2, "the byte is written");
            assert_eq!(s.units[u].orders.len(), 1, "one move, no FormOrder");
            let o = *s.current_order(u).expect("a move order");
            assert_ne!(o.index(), index::CHANGE_FORM);
            assert!(o.move_dest().is_some());
            assert!(
                !o.has(crate::orders::flag::ACTION),
                "no action bit on the spot"
            );
        }
        assert!(
            s.current_order(leader)
                .and_then(Order::move_dest)
                .is_some_and(|d| vector_dist(
                    d.x - s.units[leader].pos.x,
                    d.y - s.units[leader].pos.y
                ) < 0x180),
            "the group re-forms round where its leader stands"
        );

        // Walking: an action-bit move, then Line; the move is replayed.
        let to = Pos::new(0x4000, 0x1000);
        s.group_action_move_to(
            &g,
            to,
            QueuePos::New,
            false,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        s.group_action_form(&g, 0, 0, QueuePos::New);
        for u in [a, b] {
            assert_eq!(s.units[u].form, 0);
            assert_eq!(s.units[u].orders.len(), 1, "halted, then one replay");
            let o = *s.current_order(u).expect("the replayed move");
            assert!(
                o.has(crate::orders::flag::ACTION),
                "the replay keeps the action bit"
            );
            let d = o.move_dest().expect("a move");
            assert!(
                vector_dist(d.x - to.x, d.y - to.y) < 0x400,
                "to the same point"
            );
        }
    }

    #[test]
    fn action_move_to_gives_every_member_the_destination() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        let g = group_of(1, &[a, b]);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        for u in [a, b] {
            let o = *s.current_order(u).expect("a move order");
            assert_eq!(o.index(), index::ATTACK_TO);
            assert!(o.has(crate::orders::flag::ACTION));
        }
    }

    #[test]
    fn a_hurrying_army_stables_its_siege_and_marches_the_rest() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let siege = siege_type(&mut s);
        let city_pos = Pos::new(0x4000, 0x4000);
        let bldg = s.add_building(1, city_pos, 4);
        let c = s.cities.len();
        s.cities.push(crate::city::City {
            alive: true,
            owner: 1,
            race: Some(1),
            founder: 1,
            building: bldg,
            members: Vec::new(),
            reg: None,
            pos: city_pos,
            capital: true,
            founding_capital: true,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            no_muster: false,
            was_capital: 0,
            capture_stamp: 0,
            assimilation_timer: 0,
            attack_stamp: 0,
            capture_strength: 0,
            pop: 1,
            has_citizen: false,
            source: None,
            trade_val: 0,
            traded_with: [0; 8],
        });
        let slot = s.init_army(1, Some(c));
        let f = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
        s.army_add_unit(1, slot, f);
        s.army_add_unit(1, slot, m);

        // No hurry: both march.
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            city_pos,
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        // A land formation of two, so both members hold the grouped
        // spelling of the kind (item 237; `docs/ORDERS.md` §1.2).
        assert_eq!(s.order_type(m), index::GROUP_ATTACK_TO);

        // Hurrying, with a city at the destination: the siege is stabled
        // and the footman still marches.
        s.armies[1].list[slot].hurry = 1;
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            city_pos,
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        assert_eq!(
            s.order_type(f),
            index::GROUP_ATTACK_TO,
            "the line still marches, and the group is still two"
        );
        assert!(
            !matches!(s.order_type(m), index::ATTACK_TO | index::GROUP_ATTACK_TO),
            "the siege is sent into the city instead, not at it: {}",
            s.order_type(m)
        );
    }

    #[test]
    fn a_shooting_siege_unit_is_left_alone_by_a_move_that_is_not_hurrying() {
        let mut s = sim();
        let siege = siege_type(&mut s);
        let slot = s.init_army(1, None);
        let m = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
        let foe = spawn(&mut s, 0, siege, Pos::new(0x1080, 0x1000));
        s.army_add_unit(1, slot, m);
        s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
        assert_eq!(s.order_type(m), index::ATTACK);
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x8000, 0x8000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        assert_eq!(
            s.order_type(m),
            index::ATTACK,
            "a siege unit already shooting keeps its order"
        );
    }

    /// `Group::action_follow@006fd510` (`docs/ORDERS.md` §28): **every**
    /// orderable member gets one `FOLLOW` on the leader with the action
    /// bit — run204's squad, three orders on `0/11` on block 642 — and a
    /// member that is the leader's own captain of the same player gets
    /// none.
    #[test]
    fn a_follow_is_one_follow_order_a_member_and_none_on_the_leader_itself() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 0, t, Pos::new(11640, 10872));
        let b = spawn(&mut s, 0, t, Pos::new(11784, 10872));
        let c = spawn(&mut s, 0, t, Pos::new(11688, 11016));
        let lead = spawn(&mut s, 0, t, Pos::new(11640, 11640));
        let mut g = group_of(0, &[a, b, c, lead]);
        assert!(s.push_group(&mut g, true));
        s.group_action_follow(&g, lead, QueuePos::New);
        for u in [a, b, c] {
            let [o] = s.units[u].orders.iter().copied().collect::<Vec<_>>()[..] else {
                panic!("one order on {u}: {:?}", s.units[u].orders);
            };
            assert_eq!(
                o.body,
                Body::Follow(crate::orders::FollowOrder { target: lead })
            );
            assert!(o.has(flag::ACTION));
        }
        assert!(s.units[lead].orders.is_empty(), "the leader follows no one");
    }

    /// `Unit::do_follow@005e65d0`'s standoff, on run204's squad: a hoplite
    /// (`los` 6, slower than its leader) 768 from a standing chariot is
    /// inside `s + 0xc0` = 1,575 and stands. Once the chariot walks, `k`
    /// doubles, the threshold is 653, and the hoplite takes a `MOVE_TO` leg
    /// without the action bit to the point 461 short of the leader — block
    /// 722's (11640, 11160).
    #[test]
    fn a_follower_stands_inside_its_standoff_and_walks_when_the_leader_does() {
        let mut s = sim();
        let t = fighter(&mut s);
        s.unit_types[t].los = 6;
        let lead = spawn(&mut s, 0, t, Pos::new(11640, 11640));
        let f = spawn(&mut s, 0, t, Pos::new(11640, 10872));
        s.units[lead].movement.speed = 29;
        s.units[f].movement.speed = 23;
        s.add_follow_order(f, lead, QueuePos::New);
        let o = i64::from(s.units[f].index);
        s.work(f, 1 - o);
        assert_eq!(s.units[f].orders.len(), 1, "no leg within the standoff");
        s.add_move_order(
            lead,
            Pos::new(11640, 20000),
            MoveKind::MoveTo,
            QueuePos::New,
            true,
        );
        s.work(f, 2 - o);
        let head = s.units[f].orders[0];
        let Body::Move(m) = head.body else {
            panic!("no leg: {:?}", s.units[f].orders);
        };
        assert_eq!((m.kind, m.dest), (MoveKind::MoveTo, Pos::new(11640, 11160)));
        assert!(!head.has(flag::ACTION));
        assert!(matches!(s.units[f].orders[1].body, Body::Follow(_)));
    }

    /// A pushed squad of three and its patrol, as `process_group` and
    /// `process_patrol` build it (`docs/ORDERS.md` §27).
    fn patrolling_squad() -> (Sim, [usize; 3], Pos) {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 0, t, Pos::new(10872, 7800));
        let b = spawn(&mut s, 0, t, Pos::new(11016, 7800));
        let c = spawn(&mut s, 0, t, Pos::new(10920, 7944));
        let mut g = group_of(0, &[a, b, c]);
        assert!(s.push_group(&mut g, true));
        let to = Pos::new(11136, 11904);
        s.group_action_patrol(&g, to, QueuePos::New);
        (s, [a, b, c], to)
    }

    fn patrol(s: &Sim, u: usize) -> Vec<crate::orders::PatrolOrder> {
        s.units[u]
            .orders
            .iter()
            .filter_map(|o| match o.body {
                Body::Patrol(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    /// `Group::action_patrol@007030c0`: **every** member gets one
    /// `GroupPatrolOrder` — no plain `PatrolOrder` exists — from the
    /// group's place to the click, both on the 48-unit grid, with one id,
    /// one leader and its own index; run184's block 642 has the points.
    #[test]
    fn a_patrol_is_one_group_patrol_order_a_member_from_the_group_s_place() {
        let (s, [a, b, c], _) = patrolling_squad();
        for (i, u) in [a, b, c].into_iter().enumerate() {
            let p = patrol(&s, u);
            assert_eq!(p.len(), 1, "one patrol on member {i}");
            let p = p[0];
            assert_eq!(s.order_type(u), index::GROUP_PATROL);
            assert_eq!(
                p.points,
                [Pos::new(10872, 7800), Pos::new(11160, 11928)],
                "the leader's place and the click, snapped"
            );
            assert_eq!((p.waypoint, p.leader, p.form_id), (0, a, i));
            assert_eq!(p.id, patrol(&s, a)[0].id);
        }
    }

    /// `Unit::do_patrol@005f1910` on the leader steps the waypoint and
    /// hands the group an attack-move `QUEUE_FIRST`; the group's
    /// `QUEUE_FIRST` halts the members and `redo_patrol_order@00706d90`
    /// rebuilds each patrol behind the leg with the **leader's** step and
    /// `form_id` — run184's squad carries `form_id 0` and `waypoint 1` on
    /// all three on 642. A follower's `do_patrol` issues nothing.
    #[test]
    fn the_leader_s_leg_rebuilds_every_patrol_with_its_step() {
        let (mut s, [a, b, c], _) = patrolling_squad();
        let Some(Body::Patrol(p)) = s.current_order(b).map(|o| o.body) else {
            panic!("the follower's head is its patrol");
        };
        s.do_patrol(b, p);
        assert_eq!(
            s.order_type(b),
            index::GROUP_PATROL,
            "a follower only idles"
        );
        let Some(Body::Patrol(p)) = s.current_order(a).map(|o| o.body) else {
            panic!("the leader's head is its patrol");
        };
        s.do_patrol(a, p);
        for u in [a, b, c] {
            assert_eq!(
                s.order_type(u),
                index::GROUP_ATTACK_TO,
                "the leg is at the head"
            );
            let ps = patrol(&s, u);
            assert_eq!(ps.len(), 1, "the patrol is rebuilt once, behind the leg");
            assert_eq!((ps[0].waypoint, ps[0].form_id), (1, 0));
            assert!(s.units[u].orders[1].has(flag::ACTION));
        }
        let Some(Body::Move(m)) = s.current_order(a).map(|o| o.body) else {
            panic!("a move");
        };
        assert_eq!(m.kind, MoveKind::AttackTo);
        assert!(
            !s.current_order(a).unwrap().has(flag::ACTION),
            "the leg carries no action bit: do_patrol passes action 0"
        );
    }

    #[test]
    fn action_halt_clears_the_orders_and_the_mask_exempts_siege() {
        let mut s = sim();
        let siege = siege_type(&mut s);
        let foot = fighter(&mut s);
        let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
        let g = group_of(1, &[a, m]);
        for u in [a, m] {
            s.add_move_order(
                u,
                Pos::new(0x4000, 0x4000),
                MoveKind::MoveTo,
                QueuePos::New,
                true,
            );
        }
        s.group_action_halt(&g, 4);
        assert!(s.current_order(a).is_none(), "the footman halts");
        assert!(
            s.current_order(m).is_some(),
            "the mask's 4 bit exempts a siege unit"
        );
        s.group_action_halt(&g, 0);
        assert!(s.current_order(m).is_none());
    }

    #[test]
    fn action_stance_writes_the_combat_stance_and_cycles_on_a_negative() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let g = group_of(1, &[a, b]);
        assert_eq!(s.group_stance_type(&g), StanceType::Combat);
        s.units[a].combat.mandatory = true;
        s.group_action_stance(&g, stance_option(Stance::Raid));
        assert_eq!(s.units[a].combat.stance, Stance::Raid);
        assert_eq!(s.units[b].combat.stance, Stance::Raid);
        assert!(!s.units[a].combat.mandatory, "3 clears mandatory");
        // −1 steps forward from the leader's current option, 3 → 4.
        s.group_action_stance(&g, -1);
        assert_eq!(s.units[a].combat.stance, Stance::Raze);
        // −2 steps back, 4 → 3.
        s.group_action_stance(&g, -2);
        assert_eq!(s.units[a].combat.stance, Stance::Raid);
        // And it wraps at the six combat options.
        s.group_action_stance(&g, stance_option(Stance::HoldFire));
        s.group_action_stance(&g, -1);
        assert_eq!(s.units[a].combat.stance, Stance::Aggressive);
    }

    #[test]
    fn siege_attack_to_is_an_attack_move_when_the_group_has_no_siege() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let g = group_of(1, &[a, b]);
        s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
        for u in [a, b] {
            assert_eq!(
                s.order_type(u),
                index::ATTACK_TO,
                "no siege, no wagon, no hero: the whole group attack-moves"
            );
        }
    }

    #[test]
    fn siege_attack_to_sends_the_siege_and_leaves_the_escort_behind() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let siege = siege_type(&mut s);
        let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let m = spawn(&mut s, 1, siege, Pos::new(0x1100, 0x1000));
        let g = group_of(1, &[a, m]);
        s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
        assert_eq!(s.order_type(m), index::ATTACK_TO, "the siege advances");
        assert_eq!(s.order_type(a), index::GUARD, "and the escort guards it");
        assert!(
            matches!(s.units[a].orders[0].body, Body::Guard(g) if g.target == m),
            "on the anchor: {:?}",
            s.units[a].orders
        );
    }

    fn wagon_type(sim: &mut Sim) -> usize {
        sim.add_unit_type(UnitType {
            hits: 100,
            cols: crate::ai_load::UnitCols {
                unit_flags2: uflags2::SUPPLY_OR_HERO,
                ..crate::ai_load::UnitCols::default()
            },
            ..UnitType::default()
        })
    }

    /// Golden chapter four's 1277 in miniature (`docs/ORDERS.md` §24): an
    /// AI army with a Supply Wagon and no siege sends the wagon on, and
    /// every other member takes a `GUARD` on it, at the head of its list,
    /// with its slot's offset from `compute_form` laid out at the wagon.
    #[test]
    fn a_siegeless_army_s_wagon_is_the_anchor_and_the_rest_guard_it() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, foot, Pos::new(0x1030, 0x1000));
        let c = spawn(&mut s, 1, foot, Pos::new(0x1060, 0x1000));
        let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1400));
        let g = group_of(1, &[a, b, c, w]);
        s.group_action_siege_attack_to(&g, Pos::new(0x4000, 0x4000), Angle(0));
        assert_eq!(s.order_type(w), index::ATTACK_TO, "the wagon walks in");
        let mut dy = None;
        for u in [a, b, c] {
            let Some(Body::Guard(o)) = s.units[u].orders.front().map(|o| o.body) else {
                panic!("{u} does not guard: {:?}", s.units[u].orders);
            };
            assert_eq!(o.target, w);
            assert_eq!(o.guard, s.units[w].pos, "the post starts at the target");
            assert_eq!((o.idle, o.retry), (0, 0));
            assert!(s.units[u].orders[0].has(flag::ACTION));
            // One rank, behind the phantom artillery rank reserved for
            // the wagon: every figure shares the rank's depth.
            assert_eq!(*dy.get_or_insert(o.dy), o.dy);
        }
        // A second call rewrites the offsets in place rather than
        // stacking a second guard (`update_guard_order`).
        s.group_action_guard(&g, w, QueuePos::New, true);
        assert_eq!(s.units[a].orders.len(), 1);
    }

    /// **The anchor's sub-group lays out on its own record** (§26, item
    /// 736): `action_siege_attack_to@0070d830` builds it on the stack with
    /// `Group::clear(-1)`, so its `facing` is 0 whatever the parent's is,
    /// and what the layout writes is discarded. Great Lakes' Despot took
    /// army 3's `facing 1` onto its `ATTACK_TO` on 15351 where the original
    /// carries 0, and that mirror came back into `unit_masks & 2` when the
    /// move died.
    #[test]
    fn the_anchor_s_sub_group_lays_out_on_its_own_cleared_facing() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let slot = s.init_army(1, None);
        let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let w = spawn(&mut s, 1, wagon, Pos::new(0x1400, 0x1000));
        for u in [a, w] {
            s.army_add_unit(1, slot, u);
            s.units[u].movement.set_facing(Angle::NORTH);
        }
        s.armies[1].list[slot].group.facing = true;
        let before = s.armies[1].list[slot].group.clone();
        let g = s.army_group(1, slot);
        s.group_action_siege_attack_to(&g, Pos::new(0x1400, 0x4000), Angle::NORTH);
        assert_eq!(s.order_type(w), index::ATTACK_TO, "the wagon walks in");
        let facing = s.units[w].orders.iter().find_map(|o| match o.body {
            Body::Move(m) => Some(m.facing),
            _ => None,
        });
        assert_eq!(
            facing,
            Some(Some(false)),
            "the stack record's 0, not the army's 1: {:?}",
            s.units[w].orders
        );
        let after = &s.armies[1].list[slot].group;
        assert!(after.facing, "the army's own flag is left alone");
        assert_eq!(
            (after.o, after.o_angle, after.order_num),
            (before.o, before.o_angle, before.order_num),
            "and the sub-group's layout is not written onto the army's record"
        );
    }

    /// `Unit::do_attack_to_pause@005f22a0` (`docs/ORDERS.md` §24.9),
    /// golden chapter four's 1415 in miniature: a Supply Wagon on an
    /// attack-move, seated in a group, looks on its own fifteen-frame phase
    /// at the group's armed captains within `0x600` of it, and waits
    /// fifteen frames when no fewer than half of them are fighting — which
    /// **one** near captain that is only walking satisfies, `1 / 2` being
    /// 0. Two near and neither fighting do not. Nor does a captain out of
    /// reach.
    #[test]
    fn an_unarmed_attack_mover_waits_on_its_phase_for_a_captain_at_its_heels() {
        let case = |near: &[i32]| -> i32 {
            let mut s = sim();
            let foot = fighter(&mut s);
            let wagon = wagon_type(&mut s);
            let at = Pos::new(0x2000, 0x2000);
            let w = spawn(&mut s, 1, wagon, at);
            let mut list = vec![w];
            for &dx in near {
                let a = spawn(&mut s, 1, foot, Pos::new(at.x + dx, at.y));
                s.add_move_order(
                    a,
                    Pos::new(0x6000, 0x2000),
                    MoveKind::AttackTo,
                    QueuePos::New,
                    true,
                );
                list.push(a);
            }
            let mut g = group_of(1, &list);
            assert!(s.push_group(&mut g, true));
            s.add_move_order(
                w,
                Pos::new(0x6000, 0x2000),
                MoveKind::AttackTo,
                QueuePos::New,
                true,
            );
            let dest = s.current_move(w).expect("the wagon's attack-move").dest;
            let o = i64::from(s.units[w].index);
            // Off the phase first: nothing looks.
            s.do_attack_to_tail(w, 16 - o, dest);
            assert_eq!(s.current_move(w).unwrap().pause, 0, "off the phase");
            s.do_attack_to_tail(w, 15 - o, dest);
            s.current_move(w).unwrap().pause
        };
        assert_eq!(case(&[0x300]), 15, "one walking captain near: 0 >= 1 / 2");
        assert_eq!(case(&[0x300, -0x300]), 0, "two near, none fighting");
        assert_eq!(case(&[0x700]), 0, "out of reach: 0x700 > 0x600");
    }

    /// `do_guard`'s reposition: a guard off its post beside a moving
    /// target is given an `ATTACK_TO` leg to the post, at the head,
    /// without the action bit, stepped the same frame — so its `timer`,
    /// thirty frames a world cell of Manhattan distance, reads one less.
    ///
    /// **Every number is run133's** (`docs/ORDERS.md` §24): the three
    /// hoplites' and the wagon's block-1276 positions and the wagon's
    /// heading, the offsets `action_guard` gave them, and the posts,
    /// angle and timers the original printed on block 1277. It is the
    /// arithmetic of the post — `sinx`/`cosx` of the target's heading on
    /// `dy` and `dx`, the 48-unit snap, the world-cell timer — checked
    /// against the original's own answer on the original's own input.
    #[test]
    fn a_guard_off_its_post_walks_to_it_on_a_timed_leg_the_same_frame() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let w = spawn(&mut s, 1, wagon, Pos::new(8211, 32341));
        s.units[w].movement.heading = Angle(535_429_120);
        s.add_move_order(
            w,
            Pos::new(20000, 20000),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        let cases = [
            // (block-1276 position, dx, the post, the timer on 1277)
            ((13438, 26412), 0, (8376, 32136), 419),
            ((13430, 26268), 144, (8520, 32232), 389),
            ((13445, 26555), -144, (8280, 32040), 419),
        ];
        for ((x, y), dx, (px, py), timer) in cases {
            let a = spawn(&mut s, 1, foot, Pos::new(x, y));
            s.add_guard_order(a, w, dx, 264, QueuePos::New);
            // A frame off both sixteen-frame phases and the review's.
            let o = i64::from(s.units[a].index);
            s.work(a, 1 - o);
            let post = Pos::new(px, py);
            let head = s.units[a].orders[0];
            let Body::Move(m) = head.body else {
                panic!("no leg: {:?}", s.units[a].orders);
            };
            assert_eq!((m.kind, m.dest), (MoveKind::AttackTo, post), "dx {dx}");
            assert_eq!(m.angle, Angle(535_429_120), "the moving target's heading");
            assert!(!head.has(flag::ACTION));
            assert_eq!(m.timer, timer, "dx {dx}");
            let Body::Guard(g) = s.units[a].orders[1].body else {
                panic!("the guard is not under the leg");
            };
            assert_eq!((g.guard, g.idle, g.retry), (post, 0, 0));
        }
    }

    /// **The target's own mirror, `unit_masks & 2`** (`docs/GROUPS.md`
    /// §25): `do_guard` negates `dx` at `5e5fed` when the guarded unit
    /// carries it. **Every number is run196's**: The Despot `1/79` on
    /// block 15094 at (42855, 22630), heading −14221312 and `unit_masks`
    /// 0x4000A, and the three Longbowmen's `action_guard` offsets. The posts
    /// are the ones the original printed on block 15095. `1/78`'s 22392
    /// comes from the heading's `sinx` on the negated `dx`. Without the
    /// mirror, `1/77` and `1/78` trade posts, which is what this crate
    /// printed before item 711.
    #[test]
    fn a_guard_s_offset_is_mirrored_by_its_target_s_own_flag() {
        let posts = |mirror: bool| {
            let mut s = sim();
            let foot = fighter(&mut s);
            let wagon = wagon_type(&mut s);
            let w = spawn(&mut s, 1, wagon, Pos::new(42855, 22630));
            s.units[w].movement.heading = Angle(-14_221_312);
            s.units[w].movement.mirror = mirror;
            s.add_move_order(
                w,
                Pos::new(39480, 20184),
                MoveKind::AttackTo,
                QueuePos::New,
                true,
            );
            [0, -144, 144].map(|dx| {
                let a = spawn(&mut s, 1, foot, Pos::new(42648, 20472));
                s.add_guard_order(a, w, dx, 264, QueuePos::New);
                let o = i64::from(s.units[a].index);
                s.work(a, 1 - o);
                let Body::Guard(g) = s.units[a].orders[1].body else {
                    panic!("no leg: {:?}", s.units[a].orders);
                };
                (g.guard.x, g.guard.y)
            })
        };
        assert_eq!(
            posts(true),
            [(42840, 22344), (42984, 22344), (42696, 22392)],
            "run196's 1/76, 1/77 and 1/78 on block 15095"
        );
        assert_eq!(
            posts(false),
            [(42840, 22344), (42696, 22392), (42984, 22344)],
            "unmirrored, the flanks trade posts"
        );
    }

    /// `unit_masks & 2`'s two writers (`docs/GROUPS.md` §25), for a unit
    /// that leads no group. `Unit::set_angle@00605400` flips it on a turn
    /// `reversing` admits (`605424`) and leaves it on a smaller one, in
    /// every port of `set_angle`. `Unit::kill_current_order`'s move branch
    /// writes the handed-back mirror into it (`5e3087`/`5e308d`). Both
    /// writes come **before** the leader test, so a lone unit takes them.
    #[test]
    fn a_unit_s_own_mirror_flips_on_a_reversing_turn_leader_or_not() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let a = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        s.units[a].movement.heading = Angle(0x5555_5555);
        assert!(
            !s.units[a].movement.mirror,
            "born clear, as run192's 0x40000"
        );
        // run192's Despot on 14985: 0x55555555 → −756678656, reversing.
        s.unit_set_angle(a, Angle(-756_678_656));
        assert!(s.units[a].movement.mirror, "a reversing turn sets it");
        s.unit_set_angle(a, Angle(-756_678_656 + 0x1000_0000));
        assert!(s.units[a].movement.mirror, "a smaller one leaves it");
        s.units[a].movement.set_heading(Angle(0x4000_0000));
        assert!(
            !s.units[a].movement.mirror,
            "the ordinary path flips it too"
        );
        s.hand_back_facing(a, true, Angle(0x4000_0000));
        assert!(
            s.units[a].movement.mirror,
            "a dying move hands its mirror back"
        );
        s.hand_back_facing(a, true, Angle(-0x4000_0000));
        assert!(
            !s.units[a].movement.mirror,
            "inverted when the unit has turned"
        );
    }

    /// On its post beside a target that stands, a guard faces **outward**
    /// — `find_angle(post − target)` — and counts `idle`; a dead target
    /// ends the order.
    #[test]
    fn a_guard_on_its_post_faces_out_and_a_dead_target_ends_it() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let w = spawn(&mut s, 1, wagon, Pos::new(0x1000, 0x1000));
        s.units[w].movement.heading = Angle(0);
        // `dy` 264 at heading 0 is 264 south of the target; its cell centre.
        let post = Pos::new(0x1000 / 48 * 48 + 24, (0x1000 - 264) / 48 * 48 + 24);
        let a = spawn(&mut s, 1, foot, post);
        s.add_guard_order(a, w, 0, 264, QueuePos::New);
        let frame = 1; // `o + frame` off both sixteen-frame phases
        s.work(a, frame);
        assert_eq!(s.order_type(a), index::GUARD, "no leg: already on the post");
        let Body::Guard(g) = s.units[a].orders[0].body else {
            unreachable!()
        };
        assert_eq!((g.guard, g.idle), (post, 1));
        assert_eq!(
            s.units[a].movement.heading,
            find_angle(post.x - 0x1000, post.y - 0x1000)
        );
        s.units[w].health = 0;
        s.work(a, frame + 1);
        assert_eq!(s.order_type(a), index::NONE);
    }

    /// The review's `GUARD` arm (`Unit::work@0060d4a0`): on the
    /// sixty-four-frame phase the reposition leg is dropped and `do_guard`
    /// issues a fresh one the same frame; off it the leg runs on.
    #[test]
    fn the_review_drops_a_guard_s_leg_on_the_sixty_four_frame_phase() {
        let mut s = sim();
        let foot = fighter(&mut s);
        let wagon = wagon_type(&mut s);
        let a = spawn(&mut s, 1, foot, Pos::new(0x3000, 0x3000));
        let w = spawn(&mut s, 1, wagon, Pos::new(0x1000, 0x1000));
        s.add_move_order(
            w,
            Pos::new(20000, 20000),
            MoveKind::AttackTo,
            QueuePos::New,
            true,
        );
        s.add_guard_order(a, w, 0, 264, QueuePos::New);
        let o = i64::from(s.units[a].index);
        s.work(a, 1 - o);
        let timer = |s: &Sim| match s.units[a].orders[0].body {
            Body::Move(m) => m.timer,
            _ => panic!("no leg"),
        };
        let first = timer(&s);
        s.work(a, 2 - o);
        assert_eq!(timer(&s), first - 1, "off the phase the leg runs on");
        s.work(a, 64 - o);
        assert_eq!(s.units[a].orders.len(), 2, "one leg over the guard");
        assert!(
            timer(&s) > first - 2,
            "on it the leg is re-issued: {}",
            timer(&s)
        );
    }

    // ------------------------------------------------------------------
    // The second reading's corrections (`docs/audit/2026-08-25-groups.md`)
    // ------------------------------------------------------------------

    #[test]
    fn halt_clears_the_group_s_form_and_leaves_every_member_s_own() {
        // `action_halt@0070d0c0:29`: `GroupData.form = −1` is written once,
        // on the **group**, before any member is examined. No unit's `+0xaa`
        // is touched — and `get_form` reads the unit bytes, so writing them
        // would make the group's next move inherit −1 instead of the form
        // the members still carry.
        let mut s = sim();
        let t = fighter(&mut s);
        let slot = s.init_army(1, None);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        assert_eq!(s.armies[1].list[slot].group.form, 0, "the move settled 0");
        assert_eq!(s.group_get_form(&g), 0, "and wrote it to both members");

        s.group_action_halt(&g, 0);
        assert_eq!(
            s.armies[1].list[slot].group.form, -1,
            "the halt clears the group's own form"
        );
        assert_eq!(
            s.group_get_form(&g),
            0,
            "and leaves every member's +0xaa where it was"
        );
    }

    #[test]
    fn stance_cycles_from_the_modal_option_not_the_leader_s() {
        // `action_stance@0070d440:38` seeds the cycle with
        // `get_stance_option(NULL)` — the modal option over the members —
        // not with the leader's own stance.
        let mut s = sim();
        let t = fighter(&mut s);
        let lead = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        let c = spawn(&mut s, 1, t, Pos::new(0x1200, 0x1000));
        let g = group_of(1, &[lead, b, c]);
        s.units[lead].combat.stance = Stance::Raid; // option 3
        s.units[b].combat.stance = Stance::Aggressive; // option 0
        s.units[c].combat.stance = Stance::Aggressive; // option 0
        s.group_action_stance(&g, -1);
        assert_eq!(
            s.units[lead].combat.stance,
            Stance::Defensive,
            "the modal option is 0, so −1 steps to 1 — not to 4 from the leader's 3"
        );
    }

    #[test]
    fn stance_skips_a_plane_but_not_a_helicopter() {
        // `action_stance@0070d440:95`: vslot `+0xc0` is `is_plane`, so a
        // plane never receives a stance. A helicopter carries `unit_flags`
        // bit `f` and is not a plane, so it does.
        let mut s = sim();
        let plane = flier(&mut s, false);
        let heli = flier(&mut s, true);
        let foot = fighter(&mut s);
        let f = spawn(&mut s, 1, foot, Pos::new(0x1000, 0x1000));
        let p = spawn(&mut s, 1, plane, Pos::new(0x1100, 0x1000));
        let h = spawn(&mut s, 1, heli, Pos::new(0x1200, 0x1000));
        let g = group_of(1, &[f, p, h]);
        s.group_action_stance(&g, stance_option(Stance::Raze));
        assert_eq!(s.units[f].combat.stance, Stance::Raze);
        assert_eq!(
            s.units[p].combat.stance,
            Stance::Aggressive,
            "a plane is skipped"
        );
        assert_eq!(
            s.units[h].combat.stance,
            Stance::Raze,
            "a helicopter is not a plane"
        );
    }

    #[test]
    fn a_helicopter_takes_a_group_move_where_a_plane_does_not() {
        // The same `is_plane` split at every action's inner loop
        // (`group_member_orderable`).
        let mut s = sim();
        let plane = flier(&mut s, false);
        let heli = flier(&mut s, true);
        let p = spawn(&mut s, 1, plane, Pos::new(0x1000, 0x1000));
        let h = spawn(&mut s, 1, heli, Pos::new(0x1100, 0x1000));
        let g = group_of(1, &[p, h]);
        s.group_action_move_to(
            &g,
            Pos::new(0x4000, 0x4000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            true,
        );
        assert_eq!(s.order_type(p), index::NONE, "a plane takes no group move");
        assert_eq!(s.order_type(h), index::MOVE_TO, "a helicopter does");
    }

    #[test]
    fn attack_keeps_a_lone_shot_and_a_march_into_range_and_re_orders_the_rest() {
        // `action_attack@00712490:390`–`424`: the "already attacking" skip
        // is conditional on two sub-arms. `orderlist.count == 1` keeps it;
        // so does a current *move* order whose destination puts the target
        // in range. Anything else falls through and is re-ordered.
        let mut s = sim();
        let t = fighter(&mut s);
        let foe = spawn(&mut s, 0, t, Pos::new(0x9000, 0x1000));
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1400));
        let c = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1800));
        for u in [a, b, c] {
            s.add_attack_order(u, Obj::Unit(foe), QueuePos::New, true, true);
            assert!(!s.is_in_range(Obj::Unit(u), Obj::Unit(foe)));
        }
        // `b` is walking somewhere that leaves the target out of range;
        // `c`'s leg ends beside it. Both are transit moves, so the *action*
        // under them is still the attack.
        s.add_move_order(
            b,
            Pos::new(0x2000, 0x1400),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        s.add_move_order(
            c,
            Pos::new(0x8fc0, 0x1000),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        assert_eq!(s.units[b].orders.len(), 2);
        assert_eq!(s.units[c].orders.len(), 2);

        let g = group_of(1, &[a, b, c]);
        s.group_action_attack(&g, Obj::Unit(foe), true, QueuePos::New, 0);
        assert_eq!(
            s.units[a].orders.len(),
            1,
            "a lone out-of-range mandatory attack on the same target is kept"
        );
        assert_eq!(
            s.units[b].orders.len(),
            1,
            "a march that does not reach into range falls through and is replaced"
        );
        assert_eq!(
            s.units[c].orders.len(),
            2,
            "a march that ends in range of the target is kept"
        );
    }

    #[test]
    fn the_siege_anchor_scores_only_members_that_are_on_the_map() {
        // `action_siege_attack_to@0070d830:194` gates every summed member on
        // `is_valid_unit() && is_on_map()`.
        let mut s = sim();
        let siege = siege_type(&mut s);
        let foot = fighter(&mut s);
        let near = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
        let far = spawn(&mut s, 1, siege, Pos::new(0x8000, 0x1000));
        // A garrisoned crowd out by `far`: off the map, so it must not pull
        // the anchor over to `far`.
        let mut hidden = Vec::new();
        for k in 0..6 {
            let u = spawn(&mut s, 1, foot, Pos::new(0x8000 + k * 0x40, 0x1000));
            s.units[u].on_map = false;
            hidden.push(u);
        }
        let escort = spawn(&mut s, 1, foot, Pos::new(0x1040, 0x1000));
        let mut list = vec![near, far, escort];
        list.extend_from_slice(&hidden);
        let g = group_of(1, &list);
        let sub = group_of(1, &[near, far]);
        assert_eq!(
            s.siege_anchor(&g, &sub),
            Some(near),
            "the off-map members do not count towards the score"
        );
    }

    #[test]
    fn a_hurrying_army_with_no_city_clears_and_marches_its_shooting_siege() {
        // §6.5's clear/order asymmetry: the `QUEUE_NEW` clear gates on
        // `hurry` alone (`70524f`), the order loop on `hurry && a city was
        // found` (`7054c7`). But the order loop re-reads `order_type()`
        // **after** the clear has emptied the list (`Unit::close_orders`
        // kills every order; `UnitData::order_type` returns `NONE` on an
        // empty list), so the "shooting siege" arm no longer matches and
        // the unit takes the move like everyone else. The third pass
        // overturned the second's "issues it nothing"
        // (`docs/audit/2026-08-25-groups.md`, "Third pass — verdicts").
        let mut s = sim();
        let siege = siege_type(&mut s);
        let slot = s.init_army(1, None);
        let m = spawn(&mut s, 1, siege, Pos::new(0x1000, 0x1000));
        let foe = spawn(&mut s, 0, siege, Pos::new(0x1080, 0x1000));
        s.army_add_unit(1, slot, m);
        s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
        s.armies[1].list[slot].hurry = 1; // hurrying, and there is no city
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x8000, 0x8000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        assert_eq!(
            s.order_type(m),
            index::ATTACK_TO,
            "the clear ran, and then the order loop no longer saw an ATTACK"
        );
        assert_eq!(s.units[m].orders.len(), 1);

        // Any other queue position skips the clear loop, so the unit is
        // still shooting when the order loop looks, and is left alone.
        s.add_attack_order(m, Obj::Unit(foe), QueuePos::New, true, true);
        assert_eq!(s.order_type(m), index::ATTACK);
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x8000, 0x8000),
            QueuePos::Last,
            true,
            Angle(0),
            MoveKind::AttackTo,
            true,
        );
        assert_eq!(s.order_type(m), index::ATTACK, "QUEUE_LAST clears nothing");
        assert_eq!(s.units[m].orders.len(), 1, "and appends nothing either");
    }

    #[test]
    fn the_stance_type_is_decided_by_the_military_role_first() {
        // `UnitTypeData::get_stance_type@0061d350`, with the PDB's
        // `StanceTypes` (`STANCE_COMBAT = 0`, `WORKER = 1`, `CASTER = 2`,
        // `PACKER = 3`, `NONE = −1`): `role & MILITARY` → PACKER if
        // `unit_flags2 & 4`, else COMBAT; then the four citizen ids →
        // WORKER; then `(unit_flags2 & 6) == 2` → CASTER; else NONE. The
        // order of the tests is the point — a military caster is COMBAT,
        // a civilian packer is NONE, and "has an attack" decides nothing.
        let mut s = sim();
        let base = |role: u32, flags2: u32, packs: bool, attack: i32| UnitType {
            hits: 100,
            combat: combat::Profile {
                attack,
                packs,
                uber_size: 1,
                ..combat::Profile::default()
            },
            cols: crate::ai_load::UnitCols {
                role,
                unit_flags2: flags2,
                ..crate::ai_load::UnitCols::default()
            },
            ..UnitType::default()
        };
        let cases = [
            (
                base(0, 0, false, 15),
                StanceType::None,
                "an attack alone is not military",
            ),
            (
                base(role::MILITARY, uflags2::CASTER, false, 15),
                StanceType::Combat,
                "a military caster is combat",
            ),
            (
                base(role::MILITARY, 0, true, 15),
                StanceType::Packer,
                "a military packer packs",
            ),
            (
                base(0, 0, true, 0),
                StanceType::None,
                "a civilian packer has no stance",
            ),
            (
                base(role::CITIZEN, 0, false, 0),
                StanceType::Worker,
                "a citizen works",
            ),
            (
                base(0, uflags2::CASTER, false, 0),
                StanceType::Caster,
                "a civilian caster casts",
            ),
            (
                base(0, uflags2::CASTER | uflags2::PACKS, true, 0),
                StanceType::None,
                "a caster that packs is neither",
            ),
        ];
        for (ty, want, why) in cases {
            let t = s.add_unit_type(ty);
            let u = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
            assert_eq!(s.unit_stance_type(u), want, "{why}");
        }
    }

    #[test]
    fn action_attack_without_mandatory_lets_each_member_take_what_is_nearest() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let far = spawn(&mut s, 0, t, Pos::new(0x9000, 0x1000));
        let near = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
        let g = group_of(1, &[a]);
        s.group_action_attack(&g, Obj::Unit(far), false, QueuePos::New, 0);
        assert_eq!(
            s.units[a].combat.target,
            Some(Obj::Unit(near)),
            "mandatory 0 retargets to whatever is nearest within the respond range"
        );
        s.group_action_attack(&g, Obj::Unit(far), true, QueuePos::New, 0);
        assert_eq!(
            s.units[a].combat.target,
            Some(Obj::Unit(far)),
            "mandatory 1 keeps the given target"
        );
    }

    /// **A fresh pool slot is stamped with the frame it was pushed on**,
    /// and an equal group pushed again keeps its record's stamp:
    /// `Groups::copy_group@006fa690` writes `+0x14 = game->frame`, and
    /// `push_group` calls it only on the slot `get_open_slot` hands back
    /// (`docs/GOLDEN.md` §23, run215's `stamp 621`).
    #[test]
    fn a_pushed_group_is_stamped_with_the_frame_it_took_its_slot() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 0, t, Pos::new(0x1000, 0x1000));
        let b = spawn(&mut s, 0, t, Pos::new(0x1200, 0x1000));
        s.frame = 621;
        let mut g = Group::stack(0);
        s.group_add(&mut g, a);
        s.group_add(&mut g, b);
        assert!(s.push_group(&mut g, true));
        let slot = s.units[a].group_ptr.expect("a slot");
        assert_eq!(s.pool_state(0, slot).map(|st| st.stamp), Some(621));
        s.frame = 701;
        let mut again = Group::stack(0);
        s.group_add(&mut again, a);
        s.group_add(&mut again, b);
        assert!(s.push_group(&mut again, true));
        assert_eq!(
            s.pool_state(0, slot).map(|st| st.stamp),
            Some(621),
            "an equal group is not copied, so its stamp stays"
        );
    }

    /// `reversing@0092cf20` is inclusive at **both** ends (§6.3): the
    /// unsigned compares are `d < 0x40000000` → no and `d < 0xc0000001` →
    /// yes, so exactly 90° reverses and so does exactly 270°.
    #[test]
    fn reversing_takes_both_bounds() {
        assert!(!reversing(Angle(0x3fff_ffff)), "a hair under 90°");
        assert!(reversing(Angle(0x4000_0000)), "exactly 90°");
        assert!(reversing(Angle(-0x4000_0000)), "exactly 270°, i.e. −90°");
        assert!(
            !reversing(Angle(-0x3fff_ffff)),
            "a hair under 270° from the other side"
        );
        assert!(reversing(Angle(i32::MIN)), "dead astern");
    }

    /// **The mirror flag's three writers, in one march** (§6.3).
    ///
    /// The record's own version of this is
    /// `run31_s_three_mirrors_come_out_of_facing_s_three_writers` in
    /// `rondata::diff`; this is the same machine driven by the simulation
    /// rather than replayed out of a dump, and it is what proves the
    /// writers are wired to the places that call them.
    #[test]
    fn a_group_s_mirror_is_the_last_layout_s_and_not_the_march_s() {
        /// Degrees as the engine's binary angle, exactly.
        fn deg(n: i64) -> Angle {
            Angle((n * (1 << 32) / 360) as i32)
        }
        let mut s = sim();
        let t = fighter(&mut s);
        let slot = s.init_army(1, None);
        let a = spawn(&mut s, 1, t, Pos::new(0x4000, 0x4000));
        let b = spawn(&mut s, 1, t, Pos::new(0x4100, 0x4000));
        s.army_add_unit(1, slot, a);
        s.army_add_unit(1, slot, b);
        let g = s.army_group(1, slot);
        assert_eq!(
            s.group_find_leader(&g),
            Some(a),
            "the first of one category"
        );
        assert!(!s.armies[1].list[slot].group.facing, "Group::clear");
        // Both face north to start. `Unit::init` would leave them on
        // `Angle::INITIAL` — 120°, which is inside the first order's window
        // too, but the march below is written in round numbers from north.
        for u in [a, b] {
            s.units[u].movement.set_facing(Angle::NORTH);
        }

        // A move at 80°. The leader faces north, which is inside the 90°
        // window, so the block is **not** mirrored — and the order carries
        // that answer out with it.
        let first = deg(80);
        let order_facing = |s: &Sim, u: usize| match s.current_order(u).map(|o| o.body) {
            Some(crate::orders::Body::Move(m)) => m.facing,
            _ => panic!("the unit took a move"),
        };
        s.group_action_move_to(
            &g,
            Pos::new(0x5000, 0x5000),
            QueuePos::New,
            true,
            first,
            MoveKind::MoveTo,
            false,
        );
        assert_eq!(order_facing(&s, a), Some(false), "an unmirrored block");
        assert!(
            !s.armies[1].list[slot].group.facing,
            "and the toggle around Form::compute put the flag back"
        );

        // The march. The leader overshoots to 100° — a 100° `set_angle`,
        // so the group's flag flips — and settles back onto 80°, which is
        // only 20° and flips nothing. The group now reads `facing 1` while
        // its live layout is unmirrored, which is precisely run31's
        // frame 327 and the state that made §6.3 look wrong.
        s.unit_set_angle(a, deg(100));
        assert!(
            s.armies[1].list[slot].group.facing,
            "set_angle toggles the leader's group"
        );
        s.unit_set_angle(a, first);
        s.unit_set_angle(b, deg(-100));
        assert!(
            s.armies[1].list[slot].group.facing,
            "a small turn does not, and a follower's never does"
        );

        // A second `QUEUE_NEW` move, at 150°. Taken at face value the
        // flag would mirror this block; the clear runs first, the leader's
        // dying order hands back the mirror **it** was laid out with, and
        // the answer is the unmirrored one again.
        let g = s.army_group(1, slot);
        s.group_action_move_to(
            &g,
            Pos::new(0x5000, 0x6000),
            QueuePos::New,
            true,
            deg(150),
            MoveKind::MoveTo,
            false,
        );
        assert_eq!(
            order_facing(&s, a),
            Some(false),
            "the hand-back discards the march's flag and restores the \
             last layout's — reading `facing` straight would mirror here"
        );
        assert!(
            !s.armies[1].list[slot].group.facing,
            "and the march's 1 is gone: the hand-back **assigns**, so a \
             record dumped after this click shows the last layout's flag \
             until the leader turns again"
        );
    }
}
