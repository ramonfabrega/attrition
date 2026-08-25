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
//! a group is given the **same** destination.

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
/// | `Form::compute`'s slot table | §6.4, where in the formation each member stands | every member takes the group's own destination; the group arrives as a heap |
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

/// `GroupData::form`'s "no formation" value (§6.3).
pub const FORM_NONE: i32 = 9;

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

    /// `Group::add(o, who, 0, 0)` (§4.1) for a unit: a non-captain adds its
    /// captain instead, and a member is added once. The simulation's units
    /// are all captains (no squads are modelled), so the recursion is the
    /// identity.
    pub fn group_add(&self, g: &mut Group, u: usize) {
        if !self.units[u].alive() {
            return;
        }
        if self.units[u].owner != g.who && !g.list.is_empty() {
            return;
        }
        if g.list.contains(&u) || g.list.len() >= 128 {
            return;
        }
        g.who = self.units[u].owner;
        g.list.push(u);
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
    /// `type_cat` is unread (`docs/GROUPS.md` §13), so the simulation takes
    /// the first that qualifies.
    pub fn group_find_leader(&self, g: &Group) -> Option<usize> {
        for on_map_only in [true, false] {
            for &u in &g.list {
                if !self.units[u].alive() {
                    continue;
                }
                if !self.units[u].captain {
                    continue;
                }
                if on_map_only && !self.units[u].on_map {
                    continue;
                }
                return Some(u);
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

    /// `ObjectData` vslot `+0x108` — which stance panel a unit shows
    /// (`docs/GROUPS.md` §13: read from its uses).
    fn unit_stance_type(&self, u: usize) -> StanceType {
        let Some(t) = self.units[u].ty else {
            return StanceType::None;
        };
        let c = &self.unit_types[t];
        if c.cols.flag2(uflags2::CASTER) {
            return StanceType::Caster;
        }
        if c.combat.packs {
            return StanceType::Packer;
        }
        if self.attack_of(Obj::Unit(u)) != 0 {
            return StanceType::Combat;
        }
        if c.cols.is(crate::ai_load::role::CITIZEN) {
            return StanceType::Worker;
        }
        StanceType::None
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

    fn group_domain(&self, u: usize) -> Domain {
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

    /// Whether a member takes an order at all: active, on the map, and not
    /// an airborne plane. Every action's inner loop opens with this.
    fn group_member_orderable(&self, u: usize) -> bool {
        self.units[u].alive() && self.units[u].on_map && self.group_domain(u) != Domain::Air
    }

    // ------------------------------------------------------------------
    // The actions
    // ------------------------------------------------------------------

    /// `Group::action_halt(mask)` (§7).
    pub fn group_action_halt(&mut self, g: &Group, mask: i32) {
        for &u in &g.list {
            if !self.group_member_orderable(u) || self.ignored(u, mask) {
                continue;
            }
            self.units[u].form = -1;
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
        let cur = self
            .group_find_leader(g)
            .map_or(0, |u| stance_option(self.units[u].combat.stance));
        let s = if s >= 0 {
            s
        } else if s == -2 {
            if cur - 1 < 0 { n - 1 } else { cur - 1 }
        } else {
            (cur + 1) % n
        };
        for &u in &g.list {
            if !self.units[u].alive() || self.unit_stance_type(u) != ty {
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

        // §6.5: the AI branch. `hurry` is the army's, so a group with no
        // army never takes it.
        let hurry_city = self.group_hurry_city(g, to);

        for &u in &g.list {
            if !self.group_member_orderable(u) {
                continue;
            }
            if g.army.is_some() {
                match hurry_city {
                    None => {
                        // No hurry: a siege unit already shooting is left
                        // entirely alone — orders, move and path.
                        if self.is_siege_unit(u) && self.order_type(u) == index::ATTACK {
                            continue;
                        }
                    }
                    Some(c) => {
                        if self.is_siege_unit(u) || self.is_supply_unit(u) || self.is_hero_unit(u) {
                            self.stable_in_city(u, c, angle);
                            continue;
                        }
                    }
                }
            }
            // §6.6 step 1: the formation index, on every member but the
            // four citizen/scholar ids.
            if !self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::CITIZEN))
            {
                self.units[u].form = form as i8;
            }
            self.add_move_order(u, to, kind, queue, action);
        }
        self.bump_order_num(g);
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
                if !self.units[u].alive() {
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
                // of range, with fewer than three orders queued: keep it.
                if self.units[u].combat.mandatory
                    && self.units[u].combat.target == Some(target)
                    && self.order_type(u) == index::ATTACK
                    && self.units[u].orders.len() < 3
                    && !self.is_in_range(Obj::Unit(u), target)
                {
                    continue;
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

    /// §6.5's hurry city: the nearest friendly city to the destination,
    /// when this is an AI army's group and the army is hurrying.
    fn group_hurry_city(&self, g: &Group, to: Pos) -> Option<usize> {
        let slot = g.army?;
        let w = g.who as usize;
        if self.nation.get(w).is_none_or(|n| n.human) {
            return None;
        }
        if self.armies[w].list[slot].hurry == 0 {
            return None;
        }
        self.city_near(g.who, to)
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

    /// `WorldData::restrict` on a destination (§6.1).
    fn restrict_pos(&self, p: Pos) -> Pos {
        Pos::new(
            p.x.clamp(0, self.world.width() * crate::world::UNITS_PER_CELL - 1),
            p.y.clamp(0, self.world.height() * crate::world::UNITS_PER_CELL - 1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn fighter(sim: &mut Sim) -> usize {
        sim.add_unit_type(UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        })
    }

    fn siege_type(sim: &mut Sim) -> usize {
        sim.add_unit_type(UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 40,
                uber_size: 1,
                siege: true,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        })
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
}
