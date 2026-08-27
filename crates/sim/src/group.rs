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
use crate::orders::{MoveKind, QueuePos, index};
use crate::world::Pos;
use crate::{Player, Sim};

/// Where this module knowingly stands in for the original, in one list.
///
/// | seam | stands in for | what it costs |
/// | --- | --- | --- |
/// | `Form::compute`'s slot table | §6.4, where in the formation each member stands | every member takes the group's own destination; the group arrives as a heap. Diffable: `GROUPDATA` logs `off_x`/`off_y`/`curr_x`/`curr_y`/`angles`/`form_num` per member |
/// | the group pool | §3, 64 slots a leader and `get_open_slot`'s recycling | one group per army, never recycled |
/// | `GroupMoveOrder` | §6.6's per-frame formation | every member gets a plain `Move` — `docs/ORDERS.md` §8.4's verdict |
/// | `action_guard` | §9's escort half | with siege *and* a matching area the escort keeps its orders; no traced army has siege |
/// | the order-time path plan | §6.7 | the sim plans on the first step, in `do_move`; with a zero slot offset the plan is the same one |
/// | `find_nearby_spot`'s collision, `invalid_loc` on a slot | §6.6 step 4 | no slot is ever invalid, so no member is re-slotted |
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
            order_num: 0,
            o: Pos::new(0, 0),
            o_angle: Angle(0),
            o_dist: 0,
            facing: false,
            stamp: 0,
            form_num: 0,
            off: Vec::new(),
            curr: Vec::new(),
            angles: Vec::new(),
        }
    }
}

/// One `Group` as an action sees it: the members, the owner, and whether
/// an army owns it (`GroupData::army`, the switch §6.5 turns on).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub who: Player,
    /// `GroupData::army`, `None` for a group on the stack or a player's
    /// selection. The simulation only ever builds the first and the third.
    pub army: Option<usize>,
    /// `GroupData::list`, in join order.
    pub list: Vec<usize>,
}

impl Group {
    /// A group on the stack — `Group::clear(-1)` then `add` (§4.1).
    pub const fn stack(who: Player) -> Group {
        Group {
            who,
            army: None,
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
            list: self.armies[who as usize].list[slot].units.clone(),
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

    /// `Groups::push_group(who, g, force)` (§3.2), the one rule that is
    /// live without a pool: with `force == 0` a group of fewer than two is
    /// **not** installed and every member is left group-less. Returns
    /// whether the group took a slot.
    pub fn push_group(&self, g: &Group, force: bool) -> bool {
        force || g.num() >= 2
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

    /// `Group::normalize`'s prune (§4.3): drop the dead, last to first.
    pub fn group_normalize(&self, g: &mut Group) {
        g.list
            .retain(|&u| u < self.units.len() && self.units[u].alive());
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

    /// `GroupData::get_loc`: the leader's position (the `(ox, oy)`
    /// substitutions of §6.3 need a live move order's origin, which the
    /// simulation's `MoveOrder` does not keep — stated in `docs/GROUPS.md`
    /// §12).
    fn group_loc(&self, g: &Group) -> Option<Pos> {
        self.group_find_leader(g).map(|u| self.units[u].pos)
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
        if let Some(s) = g.army {
            self.armies[g.who as usize].list[s].group.form = -1;
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
            if ty != StanceType::Combat {
                // The worker/caster/packer stances live elsewhere in the
                // simulation (`Unit::stance`); only the write is modelled.
                self.units[u].stance = s.clamp(0, 255) as u8;
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
        let to = self.restrict_pos(to);

        // §6.3: the formation angle. With `set_angle` the caller's stands;
        // without it, the direction from the group to the destination.
        let from = self.group_loc(g);
        let angle = if set_angle {
            angle
        } else {
            match from {
                Some(p) if p != to => find_angle(to.x - p.x, to.y - p.y),
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
        let slots = self.form_compute(
            g,
            to,
            angle,
            form,
            width,
            reverse,
            false,
            &self.group_angles(g),
        );

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
        g.army.map_or(Angle(0), |s| {
            self.armies[g.who as usize].list[s].group.o_angle
        })
    }

    fn bump_order_num(&mut self, g: &Group) {
        if let Some(s) = g.army {
            self.armies[g.who as usize].list[s].group.order_num += 1;
        }
    }

    /// `action_move_near`'s writes to the record: `form` unconditionally
    /// (`70535a`), the origin and its angle only for `QUEUE_NEW` and
    /// `QUEUE_LAST` (§6.3) — so `charge`'s `QUEUE_FIRST` leaves them.
    fn record_move(&mut self, g: &Group, to: Pos, angle: Angle, form: i32, queue: QueuePos) {
        let Some(s) = g.army else { return };
        let a = &mut self.armies[g.who as usize].list[s];
        a.group.form = form;
        if queue != QueuePos::First {
            a.group.o = to;
            a.group.o_angle = angle;
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
        let (Some(s), Some(i)) = (g.army, g.list.iter().position(|&u| u == member)) else {
            return false;
        };
        self.armies[g.who as usize].list[s].group.reorigin(i);
        let theta = self.units[member].movement.heading;
        let off = self.armies[g.who as usize].list[s].group.off.clone();
        let curr = Sim::form_update_positions(&off, theta);
        self.armies[g.who as usize].list[s].group.curr = curr;
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
        let Some((u, slot)) = self.group_find_leader_slot(g) else {
            return false;
        };
        let byte = self.group_angles(g).get(slot).copied().unwrap_or(0);
        let heading = self.units[u]
            .movement
            .heading
            .0
            .wrapping_sub(i32::from(byte) << 24);
        reversing(Angle(heading.wrapping_sub(angle.0)))
    }

    /// `GroupData::facing` — the mirror flag the layout reads.
    fn group_facing(&self, g: &Group) -> bool {
        g.army
            .is_some_and(|s| self.armies[g.who as usize].list[s].group.facing)
    }

    /// The group this unit belongs to, if any — `Object::get_army` and the
    /// army's one group (`docs/ARMY.md` §3.2). The original reads
    /// `unit +0x80` straight into the pool; without a pool the army is the
    /// only thing that holds a group, so this is the same question asked of
    /// the army list.
    fn group_of(&self, u: usize) -> Option<Group> {
        let s = self.army_of(u)?;
        Some(self.army_group(self.units[u].owner, s))
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
    pub fn unit_set_angle(&mut self, u: usize, angle: Angle) {
        let turned = reversing(Angle(
            angle.0.wrapping_sub(self.units[u].movement.heading.0),
        ));
        self.units[u].movement.heading = angle;
        if !turned {
            return;
        }
        let Some(g) = self.group_of(u) else { return };
        if self.is_group_leader(u, &g)
            && let Some(s) = g.army
        {
            let f = &mut self.armies[g.who as usize].list[s].group.facing;
            *f = !*f;
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
            && let Some(s) = g.army
        {
            self.armies[g.who as usize].list[s].group.facing = f;
        }
    }

    /// `GroupData::angles` — carried in because `Form::compute` does not
    /// clear it and Column and Mob never write it (`form::Form::new`).
    fn group_angles(&self, g: &Group) -> Vec<i8> {
        g.army
            .map(|s| self.armies[g.who as usize].list[s].group.angles.clone())
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
        let Some(s) = g.army else { return };
        let off: Vec<(i32, i32)> = f
            .off
            .iter()
            .map(|&(x, y)| {
                (
                    crate::form::Form::quantise(x),
                    crate::form::Form::quantise(y),
                )
            })
            .collect();
        let theta = f.o.map_or(Angle(0), |u| self.units[u].movement.heading);
        let curr = Sim::form_update_positions(&off, theta);
        let (o_angle, o_dist) = Sim::form_leader_offset(f, to);
        let a = &mut self.armies[g.who as usize].list[s];
        a.group.form_num = g.num();
        a.group.off = off;
        a.group.curr = curr;
        a.group.angles = f.angles.clone();
        a.group.o_angle = o_angle;
        a.group.o_dist = o_dist;
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

    #[test]
    fn push_group_refuses_a_singleton_unless_forced() {
        let mut s = sim();
        let t = fighter(&mut s);
        let a = spawn(&mut s, 1, t, Pos::new(0x1000, 0x1000));
        let one = group_of(1, &[a]);
        assert!(
            !s.push_group(&one, false),
            "a group of fewer than two takes no slot without force"
        );
        assert!(s.push_group(&one, true), "Army::add_unit forces it");
        let b = spawn(&mut s, 1, t, Pos::new(0x1100, 0x1000));
        assert!(s.push_group(&group_of(1, &[a, b]), false));
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
        assert_eq!(s.order_type(m), index::ATTACK_TO);

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
        assert_eq!(s.order_type(f), index::ATTACK_TO, "the line still marches");
        assert_ne!(
            s.order_type(m),
            index::ATTACK_TO,
            "the siege is sent into the city instead"
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
