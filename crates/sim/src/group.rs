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

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `Form::compute`'s slot table | §6.4, where in the formation each member stands | every member takes the group's own destination; the group arrives as a heap. Diffable: `GROUPDATA` logs `off_x`/`off_y`/`curr_x`/`curr_y`/`angles`/`form_num` per member |
/// | the group pool | §3, 64 slots a leader and `get_open_slot`'s recycling | numbered since item 518 (§19) and reset by [`Sim::groups_process`]; `get_open_slot`'s fallbacks and `equals_group`'s normalize are not modelled |
/// | `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `Move` — `docs/ORDERS.md` §8.4's verdict |
/// | `action_guard` | §9's escort half | with siege *and* a matching area the escort keeps its orders; no traced army has siege |
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
            return false;
        }
        let pool = self.pool_slot_for(g.who, &g.list);
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
        self.pushed[slot] = crate::group::Pushed {
            who: g.who,
            list: g.list.clone(),
            state: GroupState {
                pool: Some(pool),
                ..GroupState::default()
            },
        };
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
    /// alone), so it no longer counts for the slot it left.
    fn pool_members(&self, who: Player, s: u8) -> Vec<usize> {
        let w = who as usize;
        if let Some(a) = self.armies[w]
            .list
            .iter()
            .find(|a| a.group.pool == Some(s) && a.units.iter().any(|&u| self.units[u].alive()))
        {
            return a
                .units
                .iter()
                .copied()
                .filter(|&u| self.units[u].alive())
                .collect();
        }
        self.pushed
            .iter()
            .filter(|p| p.who == who && p.state.pool == Some(s))
            .map(|p| {
                p.list
                    .iter()
                    .copied()
                    .filter(|&u| self.units[u].alive() && self.army_of(u).is_none())
                    .collect::<Vec<_>>()
            })
            .find(|l| !l.is_empty())
            .unwrap_or_default()
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
            (0..46u8)
                .find(|&s| s != last && self.pool_members(who, s).is_empty())
                .unwrap_or(45)
        };
        self.last_group[w] = s;
        s
    }

    /// The pool slot `u` points at, as the dump prints it — `who·64 + s`,
    /// or −1 for a unit in no group. For a probe and the diff harness.
    pub fn pool_group_of(&self, u: usize) -> i64 {
        let who = i64::from(self.units[u].owner);
        let s = match self.army_of(u) {
            Some(a) => self.armies[self.units[u].owner as usize].list[a].group.pool,
            None => self
                .pushed
                .iter()
                .find(|p| p.list.contains(&u))
                .and_then(|p| p.state.pool),
        };
        s.map_or(-1, |s| who * 64 + i64::from(s))
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
    /// SEAM: the prune is applied to a pushed seat's list only. An army's
    /// member list is also its membership here (`docs/ARMY.md` §3.2), and
    /// [`Sim::army_normalize`] is what drops its dead; the leader is chosen
    /// among live members either way. `find_role` is not modelled — no
    /// consumer of `GroupData::role` is.
    pub(crate) fn groups_process(&mut self, frame: i64) {
        let s = u8::try_from(frame.rem_euclid(64)).expect("under 64");
        for w in 0..self.armies.len() {
            let who = w as Player;
            for a in 0..self.armies[w].list.len() {
                if self.armies[w].list[a].group.pool != Some(s) {
                    continue;
                }
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
                    .filter(|&u| self.units[u].alive() && self.army_of(u).is_none())
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
        if let Some(slot) = self.army_of(u) {
            return self.armies[self.units[u].owner as usize].list[slot]
                .group
                .speed;
        }
        self.pushed
            .iter()
            .find(|x| x.list.contains(&u))
            .map_or(0, |x| x.state.speed)
    }

    /// The cap and its accumulator — `GroupData::speed` and `new_speed` —
    /// of the seat `u` sits in, army or pool slot, for a probe's line.
    /// `None` for a unit with no seat. Read-only; nothing in the
    /// simulation calls it.
    pub fn group_speed_pair_of(&self, u: usize) -> Option<(i32, i32)> {
        let st = match self.army_of(u) {
            Some(slot) => &self.armies[self.units[u].owner as usize].list[slot].group,
            None => &self.pushed.iter().find(|x| x.list.contains(&u))?.state,
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
    fn unit_stance_type(&self, u: usize) -> StanceType {
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
    fn group_loc_to(&self, g: &Group) -> Option<Pos> {
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
    fn group_o(&self, g: &Group) -> Pos {
        self.gstate(g).map_or(Pos::new(-1, -1), |st| st.o)
    }

    // ------------------------------------------------------------------
    // The type tests the actions ask (§6.5, §7, §9, §10)
    // ------------------------------------------------------------------

    fn is_siege_unit(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].combat.siege || self.unit_types[t].cols.flag(uflags::SIEGE)
        })
    }

    fn is_hero_unit(&self, u: usize) -> bool {
        self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].cols.flag2(uflags2::GENERAL))
    }

    fn is_supply_unit(&self, u: usize) -> bool {
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
            self.group_action_halt(g, 0);
            self.group_action_move_to(g, to, QueuePos::New, set_angle, angle, kind, action);
            for o in saved {
                match o.body {
                    Body::Move(m) => self.group_action_move_to(
                        g,
                        m.dest,
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
                    // SEAM: `finish_insert`'s other twenty cases —
                    // gather, garrison, board, follow, guard, patrol,
                    // trade, spell, the two swarms. No capture reaches a
                    // group `QUEUE_FIRST` carrying one.
                    _ => {}
                }
            }
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
        let plan: Vec<Member> = g
            .list
            .iter()
            .map(|&u| {
                if !self.group_member_orderable(u) {
                    return Member::Skip;
                }
                if !ai {
                    return Member::Move;
                }
                let shooting_siege = self.is_siege_unit(u) && self.order_type(u) == index::ATTACK;
                match hurry_city {
                    Some(c)
                        if self.is_siege_unit(u)
                            || self.is_supply_unit(u)
                            || self.is_hero_unit(u) =>
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
            })
            .collect();

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
            if !self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::CITIZEN))
            {
                self.units[u].form = form as i8;
                self.units[u].form_width = width as i8;
            }
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
                );
            } else {
                self.add_move_facing_order(
                    u,
                    slot,
                    kind,
                    queue,
                    action,
                    order_angle,
                    Some(reverse),
                    true,
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
        self.group_action_move_to(
            &sub,
            to,
            QueuePos::New,
            true,
            angle,
            MoveKind::AttackTo,
            true,
        );
        // `action_guard(anchor, who, QUEUE_NEW, 1)` on the rest is the
        // seam: GUARD is not an order the simulation has.
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

    fn group_o_angle(&self, g: &Group) -> Angle {
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
    /// [`Pushed`] slot. They are exclusive by construction — `push_group`
    /// takes its members out of the army and out of every other slot — so
    /// the order they are asked in only decides which answer a bug would
    /// give, not which one is right.
    pub(crate) fn group_of(&self, u: usize) -> Option<Group> {
        if let Some(s) = self.army_of(u) {
            return Some(self.army_group(self.units[u].owner, s));
        }
        let at = self.pushed.iter().position(|x| x.list.contains(&u))?;
        self.pushed_group_at(at)
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
    /// The original also mirrors the result into `unit_masks & 2`, and
    /// skips the whole branch for an order type outside
    /// `{1, 2, 3, 4, 0x12, 0x13, 0x15}` — the move family, which is
    /// [`Body::Move`] here.
    pub(crate) fn hand_back_facing(&mut self, u: usize, order_facing: bool, order_angle: Angle) {
        let turned = reversing(Angle(
            self.units[u].movement.heading.0.wrapping_sub(order_angle.0),
        ));
        let f = order_facing != turned;
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
        assert_eq!(
            s.order_type(a),
            index::NONE,
            "the escort would guard the anchor; GUARD is the seam"
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
