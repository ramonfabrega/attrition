//! An Airbase's own launch commands: the player's right-click with a base
//! selected (`docs/PRODUCTION.md`, "The launch commands"; `docs/GOLDEN.md`
//! §42, item 976).
//!
//! A selection of buildings that holds an Airbase turns the two orders a
//! player gives aircraft into orders for **the planes standing inside**:
//!
//! - **the ground**: `WorldMap::on_right_up@008c7050:240` and
//!   `Console::execute_at_cursor@007c6630:3230` issue
//!   `CommandManager::issue_launch_patrol@00941860(group, x, y, queue,
//!   shift, ctrl, alt)` — a 25-byte `launch_patrol` (type `0x0b`) under the
//!   emulator — in place of the gather point; `CommandPackage::
//!   process_launch_patrol@00949230` hands it to
//!   [`Sim::group_action_launch_patrol`];
//! - **an enemy in air range, or another base**: `execute_at_cursor`
//!   issues `CommandManager::issue_flight@00941d40` on the building group
//!   itself, and `Group::action_flight@006fb260`'s `buildings` arm is
//!   [`Sim::group_action_launch_flight`]. There is no `issue_launch_flight`
//!   in the executable.
//!
//! Both walk each member's `inside_down` chain — the planes in the order
//! they came in — and pick **one** plane unless the queue is `QUEUE_LAST`
//! or shift is held. Both were run unchanged under the emulator (item 976,
//! `docs/GOLDEN.md` §42 has the table), and the choice is decided by a
//! distance every plane in one base shares: the base's to the point or the
//! target, scaled by the plane's line and multiplied by 200 for a plane
//! with an order.

use crate::combat::Obj;
use crate::group::{Flight, Group};
use crate::world::{Player, Pos, vector_dist};
use crate::{Sim, orders::QueuePos};

/// `ObjectData::is(0x11f)` — the Biplane line, the fighters.
const BIPLANE: crate::tech::TypeId = 0x11f;

/// `ObjectData::is(0x136)` — the Helicopter line.
pub(crate) const HELICOPTER: crate::tech::TypeId = 0x136;

/// `V2ROCKET` (0x139), the line `action_launch_flight`'s `count_inside`
/// asks for a V2 (`6fc15d`).
const V2ROCKET: crate::tech::TypeId = 0x139;

/// `NUCLEARMISSILE` (0x13b), the line it asks for a nuke (`6fc12e`).
const NUCLEARMISSILE: crate::tech::TypeId = 0x13b;

/// The flight command's modifiers as `Console::execute_at_cursor` reads
/// them: shift (every plane), ctrl (bombers only), alt (fighters only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Sim {
    /// The planes standing in the group's buildings, each with the base it
    /// stands in: every member's `inside_down` chain (`+0x28`/`+0x3e`),
    /// member by member (`703634`..`703a7c`, `6fc240`..`6fc724`).
    fn planes_inside(&self, buildings: &[usize]) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for &b in buildings {
            for &u in &self.buildings[b].garrison {
                if self.units[u].inside == Some(b) && self.units[u].alive() {
                    v.push((b, u));
                }
            }
        }
        v
    }

    fn is_air_unit(&self, u: usize) -> bool {
        matches!(
            self.profile(Obj::Unit(u)).domain,
            crate::attrition::Domain::Air
        )
    }

    /// `type +0x1e4 & 0x8000000`, the missile flag both commands skip.
    fn is_missile(&self, u: usize) -> bool {
        self.profile(Obj::Unit(u)).has(crate::combat::mask::MISSILE)
    }

    /// `ObjectData::count_inside(COUNT_TYPE, line) != 0`: a unit of the
    /// line stands inside `b`.
    fn holds_line(&self, b: usize, line: crate::tech::TypeId) -> bool {
        self.buildings[b].garrison.iter().any(|&u| {
            self.units[u].inside == Some(b) && self.units[u].alive() && self.air_line_is(u, line)
        })
    }

    /// The ctrl and alt filters: alt keeps the Biplane line (`is(0x11f)`),
    /// ctrl the Bomber line (`is(0x130)`); alt is asked first.
    fn keys_admit(&self, u: usize, keys: Keys) -> bool {
        if keys.alt {
            self.air_line_is(u, BIPLANE)
        } else if keys.ctrl {
            self.is_bomber(u)
        } else {
            true
        }
    }

    /// **`Group::action_launch_patrol(x, y, queue, shift, ctrl, alt)@
    /// 00703580`** on a group of buildings: a right-click on the ground
    /// with an Airbase selected. Of each member's planes that are air,
    /// not busy (`UnitData::is_busy@0060a370`, a head cast here) and not
    /// missiles, and pass the ctrl/alt filter:
    ///
    /// - **without shift, a plane still refuelling** (`mana_burn != 0`,
    ///   `70374c`) is passed over — the console's "not ready" feedback;
    /// - its **distance** is `vector_dist` from its base to the point,
    ///   **÷ 10 for the Biplane line** (`7037f0`) or else ÷ 4 for the
    ///   Helicopter line (`703835`), and **× 200 for a plane with an order**
    ///   (`70388e`); the least wins, the first of equals (`70389b`);
    /// - **`QUEUE_LAST` or shift** gives every such plane
    ///   `add_air_patrol_order(x, y, base, 1)` — the action bit — on the
    ///   spot (`7038af`..`703c7a`), fuel or none;
    /// - **otherwise** the one plane chosen gets it, its base the building
    ///   `ObjectData::get_inside@00651a80` names.
    ///
    /// Under the emulator on chapter thirty-two's four (`docs/GOLDEN.md`
    /// §42): all fuelled, the Fighter `0/6`, not the Bomber `0/8` ahead of
    /// it; `0/8` and `0/7` refuelling, `0/6` still; every plane on a patrol,
    /// `0/6`; every plane refuelling, nothing; shift, all four.
    ///
    /// A type that flies like a helicopter takes a move in place of the
    /// patrol ([`Sim::launch_one`], item 1009, the emulator's row alone:
    /// no capture holds a helicopter). The console's feedback writes no
    /// state and is not modelled.
    pub fn group_action_launch_patrol(
        &mut self,
        buildings: &[usize],
        at: Pos,
        queue: QueuePos,
        keys: Keys,
    ) {
        let all = queue == QueuePos::Last || keys.shift;
        let mut best: Option<(i32, usize, usize)> = None;
        for (b, u) in self.planes_inside(buildings) {
            if !self.is_air_unit(u) || self.is_missile(u) || !self.keys_admit(u, keys) {
                continue;
            }
            if !keys.shift && self.units[u].mana_burn != 0 {
                continue;
            }
            let p = self.buildings[b].pos;
            let mut d = vector_dist(p.x - at.x, p.y - at.y);
            if self.air_line_is(u, BIPLANE) {
                d /= 10;
            } else if self.air_line_is(u, HELICOPTER) {
                d /= 4;
            }
            if !self.units[u].orders.is_empty() {
                d = d.wrapping_mul(200);
            }
            if !keys.shift && best.is_none_or(|(bd, _, _)| d < bd) {
                best = Some((d, b, u));
            }
            if all {
                self.launch_one(u, at, b);
            }
        }
        if all {
            return;
        }
        if let Some((_, b, u)) = best {
            self.launch_one(u, at, b);
        }
    }

    /// The launch patrol's order for the one plane (`7038bd`..`703c7a`,
    /// `703aeb`..`703c87`): `add_air_patrol_order(x, y, base, who, 1)`, or
    /// for a type that **flies like a helicopter** (`+0x2b4 & 0x20`,
    /// `7038d1`) a `MOVE_TO` built in place — `close_orders(0)`,
    /// `clear_partial_path`, `update_action`, then a `MoveOrder` to the
    /// point's 48-unit cell centre, facing `find_angle(point − here)`, with
    /// the action bit (`703a22`), appended.
    fn launch_one(&mut self, u: usize, at: Pos, b: usize) {
        if self.is_helicopter(u) {
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
            self.add_move_order(u, at, crate::orders::MoveKind::MoveTo, QueuePos::Last, true);
            return;
        }
        self.add_air_patrol_order(u, at, Some(b), true);
    }

    /// **`Build::train@0062f9b0`'s block at `62fc47`** (item 1019,
    /// `docs/PRODUCTION.md` "The Helicopter and the missile under a
    /// point"): a missile or a helicopter trained under a gather point reads
    /// the **first** point alone (`BuildData::get_first_gather@0046f140`):
    ///
    /// - off the world (`62fc6f`): a helicopter over the base's aircraft
    ///   limit comes out; nothing else;
    /// - on a building that is not the trainer (`find_building_at`,
    ///   `SEARCH_ALL`, `FILTER_ALL`): an enemy's is a strike,
    ///   `add_strafe_order(b, owner, this, who, 1, QUEUE_NEW, 1)`; a live
    ///   base of one's own that carries a non-missile is a flight home to
    ///   it, `add_strafe_order(−1, −1, b, owner, 1, QUEUE_NEW, 1)`;
    /// - else a non-missile takes `add_air_patrol_order(point, this, 1)` —
    ///   for a helicopter, the move to the point's cell — and a missile
    ///   nothing: it stays inside with no order.
    ///
    /// Under the emulator (`tools/emu/train_arm.py`) every arm above.
    ///
    /// A missile's strike is `add_air_attack_ground_order` at the target's
    /// point, through `Unit::add_strafe_order`'s head (item 1050).
    ///
    /// SEAM: an enemy with `MISSILE_DEFENSE_BONUS` refuses a missile's
    /// strike; no staging reaches it.
    pub(crate) fn train_first_point(
        &mut self,
        u: usize,
        base: usize,
        q: crate::rally::GatherPoint,
    ) {
        let missile = self.is_missile(u);
        if !self.in_world(q.pos) {
            if self.air_line_is(u, HELICOPTER) && self.aircraft_here(base) > 10 {
                self.come_out(u);
            }
            return;
        }
        let who = self.units[u].owner;
        let tile = q.pos.tile();
        let hit = (0..self.buildings.len())
            .find(|&b| self.buildings[b].alive && self.covers_tile(b, tile))
            .filter(|&b| b != base);
        if let Some(b) = hit {
            let owner = self.buildings[b].owner;
            if self.is_enemy(who, owner) {
                self.add_strafe_order(
                    u,
                    Some(Obj::Building(b)),
                    Some(base),
                    true,
                    QueuePos::New,
                    true,
                );
                return;
            }
            if !missile && self.base_can_carry(b, u) && self.active(Obj::Building(b)) {
                self.add_strafe_order(u, None, Some(b), true, QueuePos::New, true);
                return;
            }
        }
        if !missile {
            self.add_air_patrol_order(u, q.pos, Some(base), true);
        }
    }

    /// **`ObjectData::can_carry(type)@00645e00` at a Missile Silo** (item
    /// 1019): a missile type (`UnitTypeData::is_missile@0061d430`, the
    /// `obj_masks` bit) and a silo that `has_nuke` does not answer for.
    /// `Group::action_queue_up@006fdbb0` asks it of a silo before each
    /// `queue_up` (`6fdfa2`..`6fe025`) and skips the silo when it refuses.
    /// run371's `@queueup 0 313 2 2009`: one V2 queued and paid, the second
    /// refused (`queued 1` on 2227).
    pub(crate) fn silo_takes(&self, b: usize, ty: usize) -> bool {
        self.unit_types[ty].combat.obj_masks & crate::combat::mask::MISSILE != 0
            && !self.has_nuke(b)
    }

    /// **`ObjectData::has_nuke@00643d40`**: a silo holds one missile at a
    /// time. It answers 1 for an active building with a missile type in its
    /// queue, and for any live missile of its player on a `STRAFE` (order
    /// 16) homed at it or standing inside it.
    pub(crate) fn has_nuke(&self, b: usize) -> bool {
        let bd = &self.buildings[b];
        let missile =
            |t: usize| self.unit_types[t].combat.obj_masks & crate::combat::mask::MISSILE != 0;
        if bd.alive
            && bd.active
            && bd
                .queue
                .items
                .iter()
                .any(|i| i.tech.is_none() && missile(i.ty))
        {
            return true;
        }
        (0..self.units.len()).any(|u| {
            let un = &self.units[u];
            un.owner == bd.owner
                && un.alive()
                && un.ty.is_some_and(missile)
                && (un.inside == Some(b)
                    || matches!(un.orders.front().map(|o| o.body),
                        Some(crate::orders::Body::Strafe(sf)) if sf.home == Some(b)))
        })
    }

    /// `ObjectData::num_aircraft_here(0)`: the aircraft standing in a base.
    fn aircraft_here(&self, base: usize) -> usize {
        self.buildings[base]
            .garrison
            .iter()
            .filter(|&&p| self.is_air_unit(p))
            .count()
    }

    /// `ObjectData::can_carry(o, who)@006483c0` for a building and an
    /// aircraft: not a missile; a helicopter only into an Airbase, a plane
    /// not into a Missile Silo; then its own home, or room under
    /// `num_aircraft_limit`.
    fn base_can_carry(&self, base: usize, u: usize) -> bool {
        let Some(t) = self.buildings[base].ty else {
            return false;
        };
        let ident = self.build_types[t].ident;
        if self.is_missile(u) || !self.is_hangar(t) {
            return false;
        }
        if self.is_helicopter(u) && ident != crate::build::Ident::Airbase {
            return false;
        }
        if ident == crate::build::Ident::MissileSilo {
            return false;
        }
        if self.home_base(u) == Some(base) {
            return true;
        }
        let here = self.buildings[base]
            .garrison
            .iter()
            .filter(|&&p| self.is_air_unit(p))
            .count();
        here < 10
    }

    /// **`Group::action_launch_flight(ox, whom, orders, shift, ctrl, alt)@
    /// 006fbfb0`**, `Group::action_flight@006fb260`'s arm for a group of
    /// buildings (`6fb361`): an Airbase's right-click on an enemy
    /// (`ATTACK`) — or on another base (`MOVE_TO`, below). Of each member's
    /// planes that are air, not busy, and pass the ctrl/alt filter (asked
    /// only when the group holds an Airbase):
    ///
    /// - **without shift, a plane still refuelling** is passed over;
    /// - one that cannot reach — `get_speed(x, y, 1) · mana` under the
    ///   base-to-target `vector_dist`, both of the plane (`6fc53e`..
    ///   `6fc57e`) — is passed over;
    /// - its **distance** is the base's to the target, **÷ 4 for the line
    ///   the target calls for**: the Biplane line for a target of one's
    ///   own or a mutual ally, or for any unit; the Bomber line for an
    ///   enemy building (`6fc5b2`..`6fc661`); and **× 200 for a plane with
    ///   an order** (`6fc670`);
    /// - without shift the least wins, the first of equals; with shift
    ///   every such plane goes.
    ///
    /// The chosen go into a group of their own (`Group::clear`, `add`)
    /// and **`action_flight(o, whom, orders, 0, 0, 0)` runs on it**
    /// ([`Sim::group_action_flight`]): a plane already on a strike is
    /// re-pointed, any other takes the strike from inside its base.
    ///
    /// Under the emulator on chapter thirty-two's four (§42): all fuelled,
    /// the Bomber `0/8` on an enemy building and the Fighter `0/6` on a
    /// unit of one's own; `0/8` refuelling, `0/7`; both Bombers
    /// refuelling, `0/6`; shift, all four.
    ///
    /// The `MOVE_TO` arm, a right-click on another base, is
    /// [`Sim::launch_move`] (item 1009, run362).
    ///
    /// **The missile arm** (item 1078, `docs/PRODUCTION.md` "The
    /// missile's other arms"): a member holding a nuke (`count_inside` of
    /// `0x13b`) or a V2 (`0x139`) narrows the choice to missiles
    /// (`6fc0bd`..`6fc203`), and a group every member of which holds a
    /// nuke asks `is(0x13b)` of each. A missile skips the ctrl/alt and
    /// tank gates (`6fc3cb`); an invalid target passes it over when a V2
    /// narrowed the choice; the reach is a plane's; and **a missile whose
    /// current order answers a type is passed over** (`6fc681`..`6fc6ef`),
    /// so a second strike pressed while the first counts down in
    /// `launching` finds nothing. Its distance is the base's, with no ÷ 4
    /// and no × 200. Under the emulator (`tools/emu/launch_arm.py`): a V2
    /// or a nuke on its `AIR_ATTACK_GROUND` is not chosen, with or without
    /// shift.
    ///
    /// SEAM: `UnitData::is_busy`, and the rush rules' early refusal. No
    /// capture holds a rush rule.
    pub fn group_action_launch_flight(
        &mut self,
        who: Player,
        buildings: &[usize],
        target: Obj,
        kind: Flight,
        keys: Keys,
    ) {
        // `action_flight`'s first gate (`6fb2f5`): the target is live.
        let live = match target {
            Obj::Unit(i) => self.units.get(i).is_some_and(|u| u.alive()),
            Obj::Building(b) => self.buildings.get(b).is_some_and(|b| b.alive),
        };
        if !live {
            return;
        }
        if kind == Flight::Home {
            self.launch_move(who, buildings, target, keys);
            return;
        }
        let airbase = buildings
            .iter()
            .any(|&b| self.building_ident(b) == crate::build::Ident::Airbase);
        let t = self.pos_of(target);
        let whom = self.owner_of(target);
        let want = if self.is_ally(who, whom) || matches!(target, Obj::Unit(_)) {
            BIPLANE
        } else {
            crate::air::BOMBER
        };
        // `6fc0bd`..`6fc203`: what the members hold narrows the choice.
        let (mut nukes_only, mut any_nuke, mut any_v2) = (true, false, false);
        for &b in buildings {
            if !self.buildings[b].alive {
                continue;
            }
            if self.holds_line(b, NUCLEARMISSILE) {
                any_nuke = true;
            } else {
                nukes_only = false;
                any_v2 |= self.holds_line(b, V2ROCKET);
            }
        }
        let missiles = any_nuke || any_v2;
        let mut chosen = Group::stack(who);
        let mut best: Option<(i32, usize)> = None;
        for (b, u) in self.planes_inside(buildings) {
            if !self.is_air_unit(u) {
                continue;
            }
            if nukes_only && missiles && !self.air_line_is(u, NUCLEARMISSILE) {
                continue;
            }
            let p = self.buildings[b].pos;
            if missiles {
                // `6fc3bb`: a missile, or nothing; `6fc51c`: the target.
                if !self.is_missile(u) || (any_v2 && !self.valid_target(Obj::Unit(u), target)) {
                    continue;
                }
                let d = vector_dist(t.x - p.x, t.y - p.y);
                // `6fc681`: a missile on a live order is passed over.
                if self.unit_mana(u) * self.get_speed(u, 1) < d || !self.units[u].orders.is_empty()
                {
                    continue;
                }
                if keys.shift {
                    chosen.list.push(u);
                } else if best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, u));
                }
                continue;
            }
            if airbase && (keys.ctrl || keys.alt) && !self.keys_admit(u, keys) {
                continue;
            }
            if !keys.shift && self.units[u].mana_burn != 0 {
                continue;
            }
            let mut d = vector_dist(t.x - p.x, t.y - p.y);
            if self.unit_mana(u) * self.get_speed(u, 1) < d {
                continue;
            }
            if self.air_line_is(u, want) {
                d /= 4;
            }
            if self.order_type(u) != 0 {
                d = d.wrapping_mul(200);
            }
            if keys.shift {
                chosen.list.push(u);
            } else if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, u));
            }
        }
        if let Some((_, u)) = best {
            chosen.list.push(u);
        }
        if chosen.list.is_empty() {
            return;
        }
        self.group_action_flight(&chosen, target, kind);
    }

    /// **`action_launch_flight`'s `MOVE_TO` arm** (`6fc797`..`6fc80c`): a
    /// right-click on another base of one's own. The ctrl/alt and fuel
    /// gates as for a strike; then a plane that is not a missile and that
    /// the target `can_carry`s; the **farthest** — `dist > best` from −1
    /// (`6fc7f4`), the base's distance, so the first of one base — or
    /// every one under shift; and `action_flight(o, whom, MOVE_TO)` on
    /// them. A target that is a Missile Silo with anything inside narrows
    /// the choice to missiles (`6fc18c`..`6fc203`), which this arm then
    /// refuses: nothing goes.
    fn launch_move(&mut self, who: Player, buildings: &[usize], target: Obj, keys: Keys) {
        let Obj::Building(to) = target else {
            return;
        };
        if self.building_ident(to) == crate::build::Ident::MissileSilo
            && !self.buildings[to].garrison.is_empty()
        {
            return;
        }
        let airbase = buildings
            .iter()
            .any(|&b| self.building_ident(b) == crate::build::Ident::Airbase);
        let t = self.pos_of(target);
        let mut chosen = Group::stack(who);
        let mut best: Option<(i32, usize)> = None;
        for (b, u) in self.planes_inside(buildings) {
            if !self.is_air_unit(u) {
                continue;
            }
            if airbase && (keys.ctrl || keys.alt) && !self.keys_admit(u, keys) {
                continue;
            }
            if !keys.shift && self.units[u].mana_burn != 0 {
                continue;
            }
            if !self.base_can_carry(to, u) {
                continue;
            }
            let p = self.buildings[b].pos;
            let d = vector_dist(t.x - p.x, t.y - p.y);
            if keys.shift {
                chosen.list.push(u);
            } else if best.is_none_or(|(bd, _)| d > bd) {
                best = Some((d, u));
            }
        }
        if let Some((_, u)) = best {
            chosen.list.push(u);
        }
        if chosen.list.is_empty() {
            return;
        }
        self.group_action_flight(&chosen, target, Flight::Home);
    }

    /// **The action-3 arm of `Group::action_gather_point@006ff1b0`**
    /// (`6ff34d`..`6ffa71`): a gather point on a friendly unit, `(captain,
    /// who)` in place of the point, at a group holding an Airbase. Each
    /// member that is alive, finished and `is(0x1bf)` takes the point —
    /// `QUEUE_LAST` under shift, `QUEUE_NEW` otherwise — through
    /// `Build::add_gather_point`, whose hangar walk gives each homed plane
    /// its strike ([`Sim::add_gather_point`]); and then **the Airbases go
    /// into a group of their own and `action_flight(o, who, ATTACK, 0, 0,
    /// 0)` runs on it** (`6ffa47`..`6ffa71`): [`Sim::group_action_launch_flight`]
    /// picks one plane inside and re-points its strike with the action bit.
    /// The general loop after it skips every Airbase under action 3
    /// (`6ffbcc`).
    pub(crate) fn gather_launch(&mut self, who: Player, airbases: &[usize], at: Pos, append: bool) {
        let point = crate::rally::GatherPoint { pos: at, action: 3 };
        for &b in airbases {
            self.add_gather_point(b, point, !append);
        }
        if airbases.is_empty() {
            return;
        }
        if let Some(target) = self.gather_object(at) {
            self.group_action_launch_flight(who, airbases, target, Flight::Strike, Keys::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attrition::Domain;
    use crate::combat;
    use crate::{Unit, UnitType};

    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(128, 128),
            2,
        );
        s.nation[0].human = true;
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        s
    }

    /// A plane of `line` (the tree's type id) with a 450 tank, standing in
    /// `base` behind those already there.
    fn plane(s: &mut Sim, base: usize, line: crate::tech::TypeId) -> usize {
        let t = UnitType {
            hits: 100,
            moves: 75,
            mana: 450,
            kind: crate::attrition::UnitKind {
                domain: Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                max_range: 7 * 192,
                domain: Domain::Air,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        let ty = s.add_unit_type(t);
        // The line only: `air_line_is` reads the id itself before the tree.
        s.unit_types[ty].tree = Some(line);
        let index = i16::try_from(s.units.len()).unwrap();
        let mut u = Unit::new(0, index, s.buildings[base].pos, 100);
        u.ty = Some(ty);
        u.on_map = true;
        let u = s.add_unit(u);
        s.units[u].kind = s.unit_types[ty].kind;
        s.units[u].movement.speed = 75;
        s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
        s.init_guys(u, Some(ty));
        s.go_inside(u, base);
        u
    }

    /// Chapter thirty-two's hangar after its Clear: a who=0 Airbase with
    /// the Bomber `0/8`, the Bomber `0/7`, the Fighter `0/6` and the
    /// Biplane `0/9` in the order they landed, their tanks as given.
    fn hangar(burn: [i16; 4]) -> (Sim, usize, [usize; 4]) {
        let mut s = sim();
        let airbase = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::Airbase,
            flags: crate::build::flags::TRAINS,
            ..crate::build::BuildType::default()
        });
        let b = s.add_building(0, Pos::new(11616, 13920), 0);
        s.buildings[b].ty = Some(airbase);
        s.buildings[b].started = true;
        s.buildings[b].active = true;
        let planes = [
            plane(&mut s, b, crate::air::BOMBER),
            plane(&mut s, b, crate::air::BOMBER),
            plane(&mut s, b, BIPLANE),
            plane(&mut s, b, BIPLANE),
        ];
        for (i, &u) in planes.iter().enumerate() {
            s.units[u].mana_burn = burn[i];
        }
        (s, b, planes)
    }

    fn patrol_of(s: &Sim, u: usize) -> Option<(Pos, u8)> {
        match s.units[u].orders.front() {
            Some(o) => match o.body {
                crate::orders::Body::AirPatrol(ref ap) => Some((ap.live()[0], o.flags)),
                _ => None,
            },
            None => None,
        }
    }

    /// **The launch patrol's choice** (`docs/GOLDEN.md` §42, the
    /// emulator's rows): the Biplane line's ÷ 10 puts a fighter ahead of
    /// the bombers before it in the chain; a refuelling plane is passed
    /// over without shift; shift takes every plane, fuel or none.
    #[test]
    fn the_launch_patrol_takes_the_first_fuelled_fighter_and_shift_takes_every_plane() {
        let p = Pos::new(11520, 7680);
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
        s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
        assert_eq!(patrol_of(&s, f6), Some((p, crate::orders::flag::ACTION)));
        assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
        s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
        assert_eq!(
            patrol_of(&s, f9).map(|x| x.0),
            Some(p),
            "the fuelled fighter"
        );
        assert!([b8, b7, f6].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, planes) = hangar([40, 12, 0, 4]);
        let shift = Keys {
            shift: true,
            ..Keys::default()
        };
        s.group_action_launch_patrol(&[b], p, QueuePos::Last, shift);
        assert!(planes.iter().all(|&u| patrol_of(&s, u).is_some()));

        let (mut s, b, planes) = hangar([4, 4, 4, 4]);
        s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
        assert!(planes.iter().all(|&u| s.units[u].orders.is_empty()));
    }

    /// who=1's Barracks, finished and seen by who=0, at `at`.
    fn enemy_barracks(s: &mut Sim, at: Pos) -> usize {
        let t = s.add_building(1, at, 0);
        s.buildings[t].started = true;
        s.buildings[t].active = true;
        s.buildings[t].combat = Some(combat::Profile::default());
        s.buildings[t].health = 1200;
        s.buildings[t].ever_seen = 3;
        t
    }

    /// A who=0 unit standing on the ground at `at`.
    fn friend(s: &mut Sim, at: Pos) -> usize {
        let index = i16::try_from(s.units.len()).unwrap();
        let mut u = Unit::new(0, index, at, 40);
        u.on_map = true;
        s.add_unit(u)
    }

    fn strike_of(s: &Sim, u: usize) -> Option<(Option<Obj>, u8, bool)> {
        match s.units[u].orders.front() {
            Some(o) => match o.body {
                crate::orders::Body::Strafe(ref sf) => Some((sf.target, o.flags, sf.mandatory)),
                _ => None,
            },
            None => None,
        }
    }

    /// **The launch strike's choice** (`docs/GOLDEN.md` §42, the
    /// emulator's rows): on an enemy building the Bomber line's ÷ 4 puts
    /// the first Bomber ahead of the fighters; with both Bombers
    /// refuelling the first fuelled fighter goes; shift sends all four.
    /// One plane takes the strike from inside, the others nothing.
    #[test]
    fn the_launch_strike_takes_the_first_bomber_on_a_building() {
        let target = Pos::new(13920, 15072);
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
        let t = Obj::Building(enemy_barracks(&mut s, target));
        s.group_action_launch_flight(0, &[b], t, Flight::Strike, Keys::default());
        assert_eq!(
            strike_of(&s, b8),
            Some((Some(t), crate::orders::flag::ACTION, true))
        );
        assert!([b7, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, [b8, b7, f6, f9]) = hangar([40, 40, 0, 0]);
        let t = Obj::Building(enemy_barracks(&mut s, target));
        s.group_action_launch_flight(0, &[b], t, Flight::Strike, Keys::default());
        assert!(strike_of(&s, f6).is_some(), "the first fuelled fighter");
        assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, planes) = hangar([0, 0, 0, 0]);
        let t = Obj::Building(enemy_barracks(&mut s, target));
        let shift = Keys {
            shift: true,
            ..Keys::default()
        };
        s.group_action_launch_flight(0, &[b], t, Flight::Strike, shift);
        assert!(planes.iter().all(|&u| strike_of(&s, u).is_some()));
    }

    /// **Action 3 at an Airbase, and a point after it** (§42): every homed
    /// plane a `mandatory` strike on the friendly unit, flags 0, and the
    /// one `action_launch_flight` picks inside — the Biplane line's ÷ 4 on
    /// a unit of one's own — re-pointed with the action bit; then a
    /// ground point appended is a fresh patrol that ends every strike.
    #[test]
    fn action_three_at_an_airbase_strikes_every_plane_and_a_later_point_ends_it() {
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
        let c = friend(&mut s, Pos::new(4104, 28392));
        let at = Pos::new(i32::from(s.units[c].index), 0);
        s.action_gather_point(0, &[b], at, 3, false);
        let u = Obj::Unit(c);
        assert_eq!(
            strike_of(&s, f6),
            Some((Some(u), crate::orders::flag::ACTION, true))
        );
        for p in [b8, b7, f9] {
            assert_eq!(strike_of(&s, p), Some((Some(u), 0, true)), "plane {p}");
        }
        let p3 = Pos::new(9600, 7680);
        s.action_gather_point(0, &[b], p3, 0, true);
        assert_eq!(s.buildings[b].gather.len(), 2);
        for p in [b8, b7, f6, f9] {
            assert_eq!(s.units[p].orders.len(), 1, "plane {p}");
            assert_eq!(
                patrol_of(&s, p),
                Some((p3, crate::orders::flag::ACTION)),
                "plane {p}"
            );
        }
    }

    /// **`[P1, A3, P2]`** (item 1009, `docs/GOLDEN.md` §43, run362 on
    /// 2322, 2337 and 2352): a ground point is a patrol for every homed
    /// plane; A3 behind it a strike that closes the patrol; and P2 behind
    /// that goes into the walk's dead patrol, so the strike stands, alone.
    #[test]
    fn a_point_behind_an_action_three_point_behind_a_patrol_leaves_the_strike() {
        let (mut s, b, planes) = hangar([0, 0, 0, 0]);
        let c = friend(&mut s, Pos::new(4104, 28392));
        let p1 = Pos::new(11520, 7680);
        s.action_gather_point(0, &[b], p1, 0, false);
        assert!(
            planes
                .iter()
                .all(|&u| patrol_of(&s, u).map(|x| x.0) == Some(p1))
        );
        let a3 = Pos::new(i32::from(s.units[c].index), 0);
        s.action_gather_point(0, &[b], a3, 3, true);
        let p2 = Pos::new(7680, 11520);
        s.action_gather_point(0, &[b], p2, 0, true);
        assert_eq!(s.buildings[b].gather.len(), 3);
        for p in planes {
            assert_eq!(s.units[p].orders.len(), 1, "plane {p}");
            assert_eq!(
                strike_of(&s, p).map(|x| x.0),
                Some(Some(Obj::Unit(c))),
                "plane {p}"
            );
        }
    }

    /// **The escort** (`Unit::do_strafe@005eab00`'s ally arm, §42): a
    /// strike on one's own unit is flown at the unit, its `xx/yy`
    /// following it; this crate's `return` for an ally left the plane where
    /// it was and its point stale.
    #[test]
    fn a_strike_on_one_s_own_unit_is_flown_at_the_unit() {
        let (mut s, b, [_, _, f6, _]) = hangar([0, 0, 0, 0]);
        let c = friend(&mut s, Pos::new(4104, 28392));
        s.come_out(f6);
        s.add_strafe_order(f6, Some(Obj::Unit(c)), Some(b), true, QueuePos::New, true);
        s.units[c].pos = Pos::new(4296, 28392);
        // How far the heading is off the bearing to the unit.
        let off = |s: &Sim| {
            let p = s.units[f6].pos;
            let bearing = crate::movement::find_angle(4296 - p.x, 28392 - p.y);
            let d = (s.units[f6].movement.heading.0 as u32).wrapping_sub(bearing.0 as u32);
            if d > 0x8000_0000 { !d } else { d }
        };
        let off0 = off(&s);
        for f in 2308..2368 {
            s.work(f6, f);
        }
        let Some(crate::orders::Order {
            body: crate::orders::Body::Strafe(sf),
            ..
        }) = s.units[f6].orders.front().copied()
        else {
            panic!("the escort stands");
        };
        assert_eq!(sf.at, Some(Pos::new(4296, 28392)), "xx/yy follow the unit");
        assert!(
            off(&s) < off0,
            "turned at it: {:#x} from {off0:#x}",
            off(&s)
        );
    }

    /// **ctrl and alt** (item 1009, `docs/GOLDEN.md` §43, the emulator's
    /// rows): on the patrol ctrl keeps the Bomber line, so the first Bomber
    /// goes where the Fighter's ÷ 10 would have won; on the strike alt keeps
    /// the Biplane line, so the first fuelled fighter goes where the Bomber
    /// line's ÷ 4 would have; ctrl on the strike with only a fighter full
    /// sends nothing; ctrl and alt together are alt's.
    #[test]
    fn ctrl_keeps_the_bombers_and_alt_the_fighters_on_both_launch_commands() {
        let p = Pos::new(13440, 9600);
        let ctrl = Keys {
            ctrl: true,
            ..Keys::default()
        };
        let alt = Keys {
            alt: true,
            ..Keys::default()
        };
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
        s.group_action_launch_patrol(&[b], p, QueuePos::New, ctrl);
        assert_eq!(patrol_of(&s, b8), Some((p, crate::orders::flag::ACTION)));
        assert!([b7, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 0, 0]);
        s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys { ctrl: true, ..alt });
        assert!(patrol_of(&s, f6).is_some(), "alt is asked first");
        assert!([b8, b7, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let target = Pos::new(13920, 15072);
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 0]);
        let t = Obj::Building(enemy_barracks(&mut s, target));
        s.group_action_launch_flight(0, &[b], t, Flight::Strike, alt);
        assert!(strike_of(&s, f9).is_some(), "the fuelled fighter");
        assert!([b8, b7, f6].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, planes) = hangar([40, 40, 0, 40]);
        let t = Obj::Building(enemy_barracks(&mut s, target));
        s.group_action_launch_flight(0, &[b], t, Flight::Strike, ctrl);
        assert!(planes.iter().all(|&u| s.units[u].orders.is_empty()));
    }

    /// A second who=0 Airbase at `at`, finished.
    fn second_base(s: &mut Sim, first: usize, at: Pos) -> usize {
        let ty = s.buildings[first].ty;
        let b = s.add_building(0, at, 0);
        s.buildings[b].ty = ty;
        s.buildings[b].started = true;
        s.buildings[b].active = true;
        b
    }

    /// **`MOVE_TO` onto another base** (item 1009, §43): the first fuelled
    /// plane of equals — the farthest, strict — flies home to it; a
    /// refuelling plane is passed over; shift sends every one; a base
    /// with no room takes none — which `group_action_flight`'s own room
    /// test would refuse too, so `can_carry` dropped fails nothing here.
    #[test]
    fn a_right_click_on_another_base_sends_the_first_fuelled_plane_to_it() {
        let to = Pos::new(8544, 14688);
        let (mut s, b, [b8, b7, f6, f9]) = hangar([40, 0, 0, 0]);
        let b2 = second_base(&mut s, b, to);
        let t = Obj::Building(b2);
        s.group_action_launch_flight(0, &[b], t, Flight::Home, Keys::default());
        let home = |s: &Sim, u: usize| match s.units[u].orders.front() {
            Some(o) => match o.body {
                crate::orders::Body::Strafe(ref sf) => Some((sf.home, sf.returning, o.flags)),
                _ => None,
            },
            None => None,
        };
        assert_eq!(
            home(&s, b7),
            Some((Some(b2), true, crate::orders::flag::ACTION)),
            "0/7, the first full"
        );
        assert!([b8, f6, f9].iter().all(|&u| s.units[u].orders.is_empty()));

        let (mut s, b, planes) = hangar([40, 0, 0, 0]);
        let b2 = second_base(&mut s, b, to);
        let shift = Keys {
            shift: true,
            ..Keys::default()
        };
        s.group_action_launch_flight(0, &[b], Obj::Building(b2), Flight::Home, shift);
        assert!(planes.iter().all(|&u| home(&s, u).is_some()), "shift: all");

        let (mut s, b, planes) = hangar([0, 0, 0, 0]);
        let b2 = second_base(&mut s, b, to);
        for _ in 0..10 {
            let u = plane(&mut s, b2, BIPLANE);
            assert_eq!(s.units[u].inside, Some(b2));
        }
        s.group_action_launch_flight(0, &[b], Obj::Building(b2), Flight::Home, Keys::default());
        assert!(
            planes.iter().all(|&u| s.units[u].orders.is_empty()),
            "no room under num_aircraft_limit"
        );
    }

    /// **The Helicopter's move** (item 1009, §43, the emulator's row; no
    /// capture holds one): a type that flies like a helicopter takes a
    /// `MOVE_TO` to the point's cell centre, facing from itself to the
    /// point as clicked, with the action bit — and wins over the Bombers
    /// by the Helicopter line's ÷ 4.
    #[test]
    fn a_helicopter_launched_on_the_ground_takes_a_move_to_the_cell_centre() {
        let p = Pos::new(13440, 9600);
        let (mut s, b, [b8, b7, f6, f9]) = hangar([0, 0, 40, 40]);
        let h = plane(&mut s, b, HELICOPTER);
        let t = s.units[h].ty.unwrap();
        s.unit_types[t].cols.unit_flags |= crate::ai_load::uflags::HELICOPTER;
        let here = s.units[h].pos;
        s.group_action_launch_patrol(&[b], p, QueuePos::New, Keys::default());
        assert!(
            [b8, b7, f6, f9]
                .iter()
                .all(|&u| s.units[u].orders.is_empty())
        );
        let o = s.units[h].orders.front().copied().expect("the move");
        assert_eq!(
            o.flags & crate::orders::flag::ACTION,
            crate::orders::flag::ACTION
        );
        let crate::orders::Body::Move(m) = o.body else {
            panic!("a MOVE_TO, not {:?}", o.body);
        };
        assert_eq!(m.dest, Pos::new(13464, 9624));
        assert_eq!(
            m.angle,
            crate::movement::find_angle(p.x - here.x, p.y - here.y)
        );
    }

    /// A type of `line` made a Helicopter (`unit_flags & 0x20`) or a
    /// missile (`obj_masks & 0x8000000`), its template standing in `b`.
    fn air_type(
        s: &mut Sim,
        b: usize,
        line: crate::tech::TypeId,
        heli: bool,
        missile: bool,
    ) -> usize {
        // The tree holds the line's id, so `init_unit` can read its entry.
        if s.tech_tree.types.len() <= line {
            let unit = crate::tech::TypeDef::new("", crate::tech::Kind::Unit(Default::default()));
            let mut tree = s.tech_tree.clone();
            tree.types.resize(line + 1, unit);
            s.set_tech_tree(tree);
        }
        let p = plane(s, b, line);
        let t = s.units[p].ty.unwrap();
        if heli {
            s.unit_types[t].cols.unit_flags |= crate::ai_load::uflags::HELICOPTER;
        }
        if missile {
            s.unit_types[t].combat.obj_masks |= crate::combat::mask::MISSILE;
        }
        t
    }

    /// **A Helicopter trained with no gather point comes out at once, and
    /// a missile stays inside** (item 1019, `Build::train@0062f9b0`'s
    /// `62ff59`..`62ff8f`, the emulator's first row): no limit test on the
    /// empty list. Made to fail with the arm dropped (the Helicopter
    /// inside).
    #[test]
    fn a_helicopter_trained_with_no_point_comes_out_and_a_missile_stays() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        s.frame = 2445;
        let heli = air_type(&mut s, b, HELICOPTER, true, false);
        let v2 = air_type(&mut s, b, 0x139, false, true);
        let h = s.build_train(b, heli).unit;
        assert_eq!(s.units[h].inside, None, "the Helicopter out");
        assert!(s.units[h].orders.is_empty(), "with no order");
        let m = s.build_train(b, v2).unit;
        assert_eq!(s.units[m].inside, Some(b), "the missile inside");
        assert!(s.units[m].orders.is_empty(), "with no order");
    }

    /// **Under a point, a missile or a Helicopter reads the first point
    /// alone** (item 1019, the block at `62fc47`; the emulator's rows): on
    /// the ground a Helicopter takes the move `add_air_patrol_order` gives
    /// its line and a missile nothing; on an enemy building each a strike,
    /// mandatory, the action bit; on a base of one's own a Helicopter a
    /// flight home to it and a missile nothing. Made to fail with
    /// [`Sim::train_first_point`] dropped.
    #[test]
    fn a_missile_or_a_helicopter_under_a_point_reads_the_first_point_alone() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let heli = air_type(&mut s, b, HELICOPTER, true, false);
        let v2 = air_type(&mut s, b, 0x139, false, true);
        let ground = Pos::new(5760, 12288);
        let point = |pos| crate::rally::GatherPoint { pos, action: 0 };
        s.buildings[b].gather = vec![point(ground), point(Pos::new(7680, 11520))];
        let h = s.build_train(b, heli).unit;
        let o = s.units[h].orders.front().copied().expect("a move");
        let crate::orders::Body::Move(mv) = o.body else {
            panic!("an ATTACK_TO, not {:?}", o.body);
        };
        assert_eq!(mv.kind, crate::orders::MoveKind::AttackTo);
        assert_eq!(
            (mv.dest, o.flags & crate::orders::flag::ACTION),
            (Pos::new(5784, 12312), crate::orders::flag::ACTION)
        );
        assert_eq!(s.units[h].orders.len(), 1, "the first point alone");
        let m = s.build_train(b, v2).unit;
        assert!(
            s.units[m].orders.is_empty(),
            "a missile on the ground: nothing"
        );
        // An enemy building on the point: a strike, for either — and a
        // missile's strike is `add_strafe_order`'s head, an air attack on
        // the building's point (item 1050; the emulator hooked the adder
        // and never ran its head).
        let enemy = s.add_building(1, Pos::new(13824, 14976), 0);
        s.buildings[enemy].started = true;
        s.buildings[enemy].active = true;
        s.buildings[enemy].combat = Some(combat::Profile::default());
        s.buildings[enemy].health = 1200;
        s.buildings[enemy].ever_seen = 3;
        s.buildings[b].gather = vec![point(Pos::new(13824, 14976))];
        let u = s.build_train(b, heli).unit;
        let o = s.units[u].orders.front().copied().expect("a strike");
        let crate::orders::Body::Strafe(sf) = o.body else {
            panic!("a STRAFE, not {:?}", o.body);
        };
        assert_eq!(sf.target, Some(Obj::Building(enemy)));
        assert!(sf.mandatory && o.flags & crate::orders::flag::ACTION != 0);
        let u = s.build_train(b, v2).unit;
        let o = s.units[u].orders.front().copied().expect("a strike");
        let crate::orders::Body::AirAttackGround(g) = o.body else {
            panic!("an AIRATTACKGROUNDORDER, not {:?}", o.body);
        };
        assert_eq!(g.at, Pos::new(13824, 14976));
        assert_eq!(g.home, Some(b));
        assert!(o.flags & crate::orders::flag::ACTION != 0);
        // A base of one's own: a Helicopter flies home to it, a missile stays.
        let other = second_base(&mut s, b, Pos::new(8544, 14688));
        s.buildings[other].combat = Some(combat::Profile::default());
        s.buildings[other].health = 1200;
        // The Airbase's own 4-by-4 footprint, which the search reads.
        let t = s.buildings[other].ty.unwrap();
        (s.build_types[t].x_size, s.build_types[t].y_size) = (4, 4);
        s.buildings[b].gather = vec![point(Pos::new(8544, 14688))];
        let u = s.build_train(b, heli).unit;
        let o = s.units[u].orders.front().copied().expect("a flight home");
        let crate::orders::Body::Strafe(sf) = o.body else {
            panic!("a STRAFE home, not {:?}", o.body);
        };
        assert_eq!(
            (sf.target, sf.home, sf.returning),
            (None, Some(other), true)
        );
        let m = s.build_train(b, v2).unit;
        assert!(s.units[m].orders.is_empty(), "a missile to a base: nothing");
    }

    /// **A Helicopter's exit spends two draws** (item 1019,
    /// `Unit::do_spec_anim@005e5880`, `5e59d5`..`5e5a1c`): `x` less `197 −
    /// rand % 11`, `y` plus `rand % 11 − 5`, and 200 over the ground. Made
    /// to fail with the arm dropped (no draw, the plane's `x − 0xc0`).
    #[test]
    fn a_helicopter_s_exit_spends_two_draws_and_hovers_200_up() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        s.frame = 2445;
        air_type(&mut s, b, HELICOPTER, true, false);
        let h = *s.buildings[b].garrison.last().unwrap();
        let mut r = s.rng;
        let (dx, dy) = (r.roll() % 11 - 197, r.roll() % 11 - 5);
        assert!(s.come_out(h));
        let at = s.buildings[b].pos;
        assert_eq!(s.units[h].pos, Pos::new(at.x + dx, at.y + dy));
        let ground = s.ground_z(at);
        assert_eq!(s.units[h].airframe.z, ground + 200);
    }

    /// A Helicopter of its own type out of `b` and on the map at `at`, with
    /// its figure on it and no order.
    fn heli_at(s: &mut Sim, b: usize, at: Pos) -> usize {
        // The exit's arm, which faces it north, wants a frame past 0.
        s.frame = s.frame.max(1);
        air_type(s, b, HELICOPTER, true, false);
        let h = *s.buildings[b].garrison.last().unwrap();
        assert!(s.come_out(h));
        s.units[h].orders.clear();
        s.set_new_location(h, at, true);
        h
    }

    /// **A Helicopter's move is one straight waypoint** (item 1048: the
    /// planners' `+0x2b4 & 0x20` returns, `find_path@005fb910`'s `5fb96f`,
    /// `find_wpath`'s `689110`, `find_tpath`'s `68990d`): run371's `0/12`
    /// out at (8351, 14690) on its attack-move to P_h's cell takes the goal
    /// alone, no `path_recursion`, and steps 75 straight at it — the dump's
    /// (−55, −51). Made to fail with each of the three returns dropped.
    #[test]
    fn a_helicopter_flies_one_straight_waypoint() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let h = heli_at(&mut s, b, Pos::new(8351, 14690));
        let goal = Pos::new(5784, 12312);
        s.add_air_patrol_order(h, goal, None, true);
        // Faced along the bearing already: this fixture's type turns
        // slowly, where the install's Helicopter took it in one frame.
        let at = s.units[h].pos;
        let bearing = crate::movement::find_angle(goal.x - at.x, goal.y - at.y);
        s.units[h].movement.set_facing(bearing);
        s.frame = 2674;
        s.work(h, 2674);
        let path: Vec<Pos> = s.units[h].path.iter().map(|p| p.to).collect();
        assert_eq!(path, vec![goal], "the goal alone");
        assert_eq!(s.units[h].path_recursion, 0);
        assert_eq!(s.units[h].pos, Pos::new(8296, 14639), "75 straight at it");
    }

    /// **Two Helicopters are set 48 apart by the one working** (item 1048,
    /// `Unit::work@0060d180`'s tail, `60dadd`..`60dc9f`): run371's `0/12`
    /// at (8296, 14639) and `0/11` idle at (8355, 14690) on 2674 — the one
    /// working moves half the vector, the other the whole of it, each with
    /// its figure put on it: (8278, 14624) and (8392, 14721), block 2675's
    /// own. Nothing moves while the other is inside. Made to fail with the
    /// arm dropped.
    #[test]
    fn two_helicopters_are_set_48_apart_by_the_one_working() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let near = heli_at(&mut s, b, Pos::new(8355, 14690));
        s.go_inside(near, b);
        let me = heli_at(&mut s, b, Pos::new(8296, 14639));
        s.frame = 2674;
        s.work(me, 2674);
        assert_eq!(s.units[me].pos, Pos::new(8296, 14639), "the other inside");
        assert!(s.come_out(near));
        s.units[near].orders.clear();
        s.set_new_location(near, Pos::new(8355, 14690), true);
        s.work(me, 2674);
        assert_eq!(s.units[me].pos, Pos::new(8278, 14624));
        assert_eq!(s.units[near].pos, Pos::new(8392, 14721));
        for u in [me, near] {
            assert_eq!(s.units[u].movement.body.pos, s.units[u].pos, "put on it");
        }
    }

    /// **The tile planner leaves a Helicopter's goal as it is** (item 1048,
    /// `PathFinder::find_tpath@006897d0`'s `68990d`): a goal two cells off
    /// comes back alone, tolerance 0. `find_wpath` answers first on every
    /// fresh move, so this arm is reached only by a caller that asks the
    /// tile grid directly. Made to fail with the arm dropped.
    #[test]
    fn the_tile_planner_leaves_a_helicopter_s_goal_as_it_is() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let h = heli_at(&mut s, b, Pos::new(8351, 14690));
        let goal = Pos::new(5784, 12312);
        s.units[h].path.push(crate::orders::PathData {
            to: goal,
            tolerance: 384,
            flags: crate::orders::path_flag::FINAL,
        });
        assert_eq!(s.find_tpath(h), 1);
        let top = s.units[h].path[0];
        assert_eq!((top.to, top.tolerance), (goal, 0));
    }

    /// **A Helicopter on an air order is not set apart** (item 1048, the
    /// separation's `is_air` gate, `UnitOrder` vslot `+0x30` at `60db14`):
    /// the pair of run371's 2674, the working one on a strike. No capture
    /// has the case. Made to fail with the gate dropped.
    #[test]
    fn a_helicopter_on_an_air_order_is_not_set_apart() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let near = heli_at(&mut s, b, Pos::new(8355, 14690));
        let me = heli_at(&mut s, b, Pos::new(8296, 14639));
        s.add_strafe_order(
            me,
            None,
            Some(b),
            false,
            crate::orders::QueuePos::New,
            false,
        );
        s.helicopter_spread(me);
        assert_eq!(s.units[me].pos, Pos::new(8296, 14639));
        assert_eq!(s.units[near].pos, Pos::new(8355, 14690));
    }

    /// **A Helicopter's figure climbs thirty a move toward 1000 over the
    /// ground** (item 1048, `Guy::set_new_location@005d86f0`'s
    /// `5d880b`..`5d884a`): by a teleport, which leaves `last_z` on it, and
    /// by a step of the figure, which leaves `last_z` where it stood —
    /// run371's `0/12` on 2677, `z` 308 over `last_z` 278. Made to fail
    /// with either call dropped.
    #[test]
    fn a_helicopter_s_figure_climbs_thirty_a_move() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let at = Pos::new(8351, 14690);
        let h = heli_at(&mut s, b, at);
        let ground = s.ground_z(at);
        s.units[h].airframe.z = ground + 158;
        s.set_new_location(h, at, true);
        assert_eq!(s.units[h].airframe.z, ground + 188);
        assert_eq!(s.units[h].airframe.last_z, ground + 188);
        s.units[h].airframe.z = ground + 990;
        s.set_new_location(h, at, true);
        assert_eq!(s.units[h].airframe.z, ground + 1000, "the cap");
        // A step: the unit ahead of its figure, which follows it.
        s.units[h].airframe.z = ground + 278;
        s.units[h].pos = Pos::new(at.x - 55, at.y - 51);
        s.process_movement(h);
        let z = s.units[h].airframe;
        assert_eq!((z.z, z.last_z), (ground + 308, ground + 278));
    }

    /// **A Missile Silo holds one missile at a time** (item 1019,
    /// `Group::action_queue_up@006fdbb0`'s `can_carry(type)` at a silo,
    /// `ObjectData::has_nuke@00643d40`): of `@queueup 313 2`, one V2 is
    /// queued and paid and the second refused — run371's `queued 1` on
    /// 2227. A missile already inside refuses the first as well. Made to
    /// fail with the gate dropped (two queued).
    #[test]
    fn a_missile_silo_queues_one_missile_at_a_time() {
        let (mut s, b, _) = hangar([0, 0, 0, 0]);
        let silo_ty = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::MissileSilo,
            flags: crate::build::flags::TRAINS,
            ..crate::build::BuildType::default()
        });
        let silo = s.add_building(0, Pos::new(9984, 12288), 0);
        s.buildings[silo].ty = Some(silo_ty);
        s.buildings[silo].started = true;
        s.buildings[silo].active = true;
        let v2 = air_type(&mut s, b, 0x139, false, true);
        assert!(s.silo_takes(silo, v2), "an empty silo takes a missile");
        // Through the command: `@queueup 313 2` lays one entry.
        s.muster[0].researched[v2] = true;
        s.buildings[silo].queue.capacity = 20;
        for g in 0..crate::economy::RESOURCES {
            s.ledgers[0].bucket[g] = 100_000;
        }
        assert_eq!(s.action_queue_up(&[silo], v2, 2), 1, "one of two");
        assert_eq!(s.buildings[silo].queue.items.len(), 1);
        s.buildings[silo].queue.items.clear();
        s.buildings[silo].queue.items.push(crate::production::Item {
            job_counter: 0,
            ty: v2,
            tech: None,
            good: [-1; 3],
            cost: [0; 3],
        });
        assert!(!s.silo_takes(silo, v2), "one queued: the second is refused");
        s.buildings[silo].queue.items.clear();
        let m = s.build_train(silo, v2).unit;
        assert_eq!(s.units[m].inside, Some(silo));
        assert!(!s.silo_takes(silo, v2), "one inside: refused");
        let bomber = air_type(&mut s, b, crate::air::BOMBER, false, false);
        let empty = s.add_building(0, Pos::new(7680, 12288), 0);
        s.buildings[empty].ty = Some(silo_ty);
        s.buildings[empty].active = true;
        assert!(!s.silo_takes(empty, bomber), "not a missile: refused");
    }

    /// run371's silo `0/2009` with a V2 inside — `TO_HIT 300`, `ATTENUATE
    /// 3`, `RECHARGE 30` at the silo — and the Barracks `1/2006` it strikes.
    fn silo_with_a_v2() -> (Sim, usize, usize, usize) {
        let mut s = sim();
        let silo_ty = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::MissileSilo,
            flags: crate::build::flags::TRAINS,
            ..crate::build::BuildType::default()
        });
        let silo = s.add_building(0, Pos::new(9984, 12288), 0);
        s.buildings[silo].ty = Some(silo_ty);
        s.buildings[silo].started = true;
        s.buildings[silo].active = true;
        s.buildings[silo].combat = Some(combat::Profile {
            recharge: 30,
            ..combat::Profile::default()
        });
        let t = air_type(&mut s, silo, 0x139, false, true);
        s.unit_types[t].combat.to_hit = 300;
        s.unit_types[t].combat.attenuate = 3;
        let v2 = s.units.len() - 1;
        let enemy = enemy_barracks(&mut s, Pos::new(13824, 14976));
        (s, silo, v2, enemy)
    }

    /// **A missile's strike from its silo is an air attack on the point**
    /// (item 1050): the flight command's inside arm takes a missile as it
    /// takes a plane, and `add_strafe_order`'s head turns the strafe into
    /// `add_air_attack_ground_order` at the target's point, home the silo,
    /// with the action bit — run371's `0/10` on 2672. Made to fail with the
    /// head dropped (a `StrafeOrder`) and with the inside arm's missile
    /// refusal put back (no order).
    #[test]
    fn a_missile_s_strike_from_its_silo_is_an_air_attack_on_the_point() {
        let (mut s, silo, v2, enemy) = silo_with_a_v2();
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        assert_eq!(s.units[v2].orders.len(), 1, "one order");
        let o = s.units[v2].orders[0];
        assert_eq!(o.index(), crate::orders::index::AIR_ATTACK_GROUND);
        assert_eq!(o.flags, crate::orders::flag::ACTION, "flags 4");
        let crate::orders::Body::AirAttackGround(g) = o.body else {
            panic!("an air attack on the ground")
        };
        assert_eq!(
            g.at,
            Pos::new(13824, 14976),
            "att_x/att_y: the target's point"
        );
        assert_eq!(g.home, Some(silo), "oxx: the silo");
        assert_eq!(g.cruising_alt, crate::orders::CRUISING_ALT);
        assert!(!g.returning);
        assert_eq!(s.units[v2].inside, Some(silo), "the missile waits inside");
    }

    /// **The shield refuses a missile's order** (item 1078, `action_flight`'s
    /// `6fbbd7`..`6fbbfb`): a target owned by a player holding
    /// `MISSILE_DEFENSE_BONUS`'s prerequisite takes no missile — run390's
    /// V2c `0/15` on 3080, inside with no order on 3082 — and the same
    /// press before the shield orders it. Made to fail with the gate
    /// dropped (the order laid).
    #[test]
    fn the_shield_refuses_a_missile_s_order_on_its_holder() {
        let (mut s, silo, v2, enemy) = silo_with_a_v2();
        let mut tree = s.tech_tree.clone();
        let shield = tree.types.len();
        tree.types.push(crate::tech::TypeDef::new(
            "Missile Shield",
            crate::tech::Kind::Final,
        ));
        tree.roles.missile_defense_preq = Some(shield);
        s.set_tech_tree(tree);
        for p in &mut s.tech {
            p.tech.resize(shield + 1, false);
        }
        s.tech[1].tech[shield] = true;
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        assert!(
            s.units[v2].orders.is_empty(),
            "refused: who=1 holds the shield"
        );
        s.tech[1].tech[shield] = false;
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        assert_eq!(s.units[v2].orders.len(), 1, "ordered without it");
    }

    /// **A missile that fires leaves its type's count** (item 1078,
    /// `Unit::close@0060ee50`'s `track_unit_type(·, −1)`): the next V2 is
    /// priced as if none stood — run390's `0/2010` charged 100 and 100 on
    /// 2722, where this crate charged 120. Made to fail with the decrement
    /// dropped (the count stays 1).
    #[test]
    fn a_missile_that_fires_leaves_its_type_s_count() {
        let (mut s, silo, v2, enemy) = silo_with_a_v2();
        let ty = s.units[v2].ty.unwrap();
        s.unit_types[ty].price.pop = 1;
        let before = s.muster[0].by_type[ty];
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        s.frame = 2671;
        s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
        for _ in 0..31 {
            s.do_launch(silo);
            s.frame += 1;
        }
        assert!(!s.units[v2].alive(), "fired and gone");
        assert_eq!(s.muster[0].by_type[ty], before - 1, "out of num_units");
    }

    /// **A second strike pressed while the first counts down finds
    /// nothing** (item 1078, `action_launch_flight`'s `6fc681`..`6fc6ef`,
    /// `tools/emu/launch_arm.py`): the V2 on its `AIR_ATTACK_GROUND` is
    /// passed over, with or without shift, and keeps the first target's
    /// point — chapter thirty-six's `0/14` on 3029. Made to fail with the
    /// live-order test dropped (the V2 re-pointed to the second target).
    #[test]
    fn a_second_strike_during_the_countdown_passes_the_missile_over() {
        let (mut s, silo, v2, enemy) = silo_with_a_v2();
        let other = enemy_barracks(&mut s, Pos::new(35712, 18816));
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        s.frame = 2671;
        s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
        s.do_launch(silo);
        assert_eq!(s.buildings[silo].recharging, 30, "counting down");
        for shift in [false, true] {
            let keys = Keys {
                shift,
                ..Keys::default()
            };
            s.group_action_launch_flight(0, &[silo], Obj::Building(other), Flight::Strike, keys);
            assert_eq!(s.units[v2].orders.len(), 1, "one order still");
            let crate::orders::Body::AirAttackGround(g) = s.units[v2].orders[0].body else {
                panic!("the air attack on the ground")
            };
            assert_eq!(
                g.at,
                Pos::new(13824, 14976),
                "the first target's point, shift {shift}"
            );
        }
        assert_eq!(
            s.buildings[silo].launching,
            vec![v2],
            "one entry in launching"
        );
    }

    /// **The silo counts its missile out, and the missile fires and ends in
    /// the same call** (item 1050): the launch sets `recharging` to the
    /// silo's `RECHARGE`, 30, with `launch_frames` 0 and the missile in
    /// `launching`; each later call takes one off and does nothing else;
    /// at 0 the missile comes out on the silo's own point, flies one step,
    /// fires its round from its point plus [`crate::air::MISSILE_OFFSET`],
    /// and dies with its number held for the round's 120 frames and one.
    /// run371: 30 on 2672, 1 on 2701, the round on 2702. Made to fail with
    /// the countdown's head dropped (the missile never leaves), the
    /// missile's `recharging` dropped (the same), and the silo's point
    /// dropped from `come_out` (the ring's spot).
    #[test]
    fn the_silo_counts_its_missile_out_and_it_fires_on_the_thirtieth() {
        let (mut s, silo, v2, enemy) = silo_with_a_v2();
        s.group_action_launch_flight(
            0,
            &[silo],
            Obj::Building(enemy),
            Flight::Strike,
            Keys::default(),
        );
        s.frame = 2671;
        s.buildings[silo].launch_frames = crate::air::FRAMES_BETWEEN_LAUNCHES;
        s.do_launch(silo);
        assert_eq!(s.buildings[silo].recharging, 30);
        assert_eq!(s.buildings[silo].launch_frames, 0);
        assert_eq!(s.buildings[silo].launching, vec![v2]);
        for k in 1..30 {
            s.frame += 1;
            s.do_launch(silo);
            assert_eq!(s.buildings[silo].recharging, 30 - k);
            assert_eq!(s.buildings[silo].launch_frames, 0, "it stands");
            assert_eq!(s.units[v2].inside, Some(silo));
        }
        assert!(s.projectiles.is_empty());
        s.frame += 1;
        s.do_launch(silo);
        assert_eq!(s.buildings[silo].recharging, 0);
        assert!(s.buildings[silo].launching.is_empty());
        assert!(s.buildings[silo].garrison.is_empty());
        assert!(!s.units[v2].alive(), "the missile ends with its shot");
        let at = s.units[v2].pos;
        let from = s.buildings[silo].pos;
        let step = vector_dist(at.x - from.x, at.y - from.y);
        assert!(
            step > 0 && step <= s.get_speed(v2, 1) + 1,
            "one step from the silo's own point: {at:?}"
        );
        assert_eq!(s.projectiles.len(), 1, "one round");
        let p = s.projectiles[0];
        let (dx, dy, dz) = crate::air::MISSILE_OFFSET;
        assert_eq!(p.launch, Pos::new(at.x + dx, at.y + dy));
        assert_eq!(p.sz, dz, "the ground at 0 and the offset's height");
        assert_eq!(p.total_time, crate::air::MISSILE_FLIGHT);
        assert_eq!(p.target, None, "a round at the ground");
        assert_eq!(s.units[v2].hold_frames, 121, "held for its round");
    }

    /// **A missile's round, on run371's own numbers** (item 1050,
    /// `Ammo::init`'s missile arm): from (10083, 12346) at the point
    /// (13824, 14976), accuracy `300 − 3 · (4662 / 192)` = 228, the land
    /// formula's 11 **doubled** to 22, and the trace's own frame word
    /// `0x14e73b8f` — rolls 62369 and 56984, `% 22 − 11` — lands it on
    /// (13834, 14969) after 120 frames, as the dump's `ex`/`ey` read on 2702.
    /// Made to fail with the doubling dropped (`s` 11: (13826, 14976)) and
    /// with the time the flight formula's.
    #[test]
    fn a_missile_s_round_is_run371_s_to_the_unit() {
        let (mut s, silo, v2, _) = silo_with_a_v2();
        s.come_out(v2);
        s.units[v2].pos = Pos::new(10083, 12346);
        let _ = silo;
        s.rng.seed = 0x14e7_3b8f;
        s.missile_round(v2, Pos::new(13824, 14976));
        let p = s.projectiles[0];
        assert_eq!(p.launch, Pos::new(9974, 12347), "sx, sy");
        assert_eq!(p.accuracy, 228);
        assert_eq!(p.landing, Pos::new(13834, 14969), "ex, ey");
        assert_eq!(p.total_time, 120);
        assert_eq!(p.angle.0, 1_482_031_104, "the dump's angle");
    }
}
