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
pub(crate) const BIPLANE: crate::tech::TypeId = 0x11f;

/// `ObjectData::is(0x136)` — the Helicopter line.
pub(crate) const HELICOPTER: crate::tech::TypeId = 0x136;

/// `V2ROCKET` (0x139), the line `action_launch_flight`'s `count_inside`
/// asks for a V2 (`6fc15d`).
const V2ROCKET: crate::tech::TypeId = 0x139;

/// `NUCLEARMISSILE` (0x13b), the line it asks for a nuke (`6fc12e`).
pub(crate) const NUCLEARMISSILE: crate::tech::TypeId = 0x13b;
/// `CRUISEMISSILE` (0x13a): `Ammo::init`'s `is(0x13a)`, the fourth argument
/// of `Spline::calc_nuke_spline@00913ad0` (item 3806, `docs/GOLDEN.md` §152.4).
pub(crate) const CRUISEMISSILE: crate::tech::TypeId = 0x13a;

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

    pub(crate) fn is_air_unit(&self, u: usize) -> bool {
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
    /// not busy ([`Sim::is_busy`]: a plane inside holds no order) and not
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

    /// **`Group::action_recall@006fa7e0`'s building arm** (item 3912,
    /// `docs/GOLDEN.md` §159): `OPTION_AIRBASE_RECALL` on a selection of
    /// buildings, `CommandPackage::process_recall@00947bd0` (byte 0x23).
    /// A group whose leader is domain 2 takes `action_return` instead, the
    /// unit arm (item 3911's).
    ///
    /// - Each member that is active (`+0xc`, `SubObjectData::is_active`) and
    ///   a building (`+0x20`, `Buffer::is_pending_load`, 1) is cleared:
    ///   [`Sim::clear_gather`] (`6fa8f0`..`6fa96f`), whose hangar half
    ///   already sends every homed plane on the map home and empties every
    ///   one inside.
    /// - Then every live unit of the player, in index order, that is a
    ///   fixed-wing plane (`is_plane`, no `+0x2b4 & 0x20`) and no missile
    ///   (`type +0x1e4 & 0x8000000`): **on the map**, when its front air
    ///   order's home is a member, the order's `returning` is set, its
    ///   `cruising_alt` and `sharp_turn` kept, and the plane is given
    ///   `add_strafe_order(−1, −1, home, 0, QUEUE_NEW, 0)` with those two
    ///   heights; **inside** a member, `clear_orders` and out of the host's
    ///   `launching` (`+0x44`).
    ///
    /// The walk repeats what `clear_gather` has just done for any plane both
    /// reach, so on an Airbase the recall is its Clear's hangar half.
    ///
    /// SEAM: `home_base`'s order-`0x19` arm — the walk reads the order behind
    /// a head of type `0x19`; no plane here holds one.
    pub fn group_action_recall_buildings(&mut self, who: Player, buildings: &[usize]) {
        for &b in buildings {
            if self.buildings[b].alive && self.buildings[b].active {
                self.clear_gather(b);
            }
        }
        let mut units: Vec<usize> = (0..self.units.len())
            .filter(|&u| self.units[u].owner == who)
            .collect();
        units.sort_by_key(|&u| self.units[u].index);
        for u in units {
            if !self.units[u].alive()
                || !self.is_air_unit(u)
                || self.is_helicopter(u)
                || self.is_missile(u)
            {
                continue;
            }
            match self.units[u].inside {
                None => {
                    let Some(home) = self.current_air(u).and_then(|a| a.home) else {
                        continue;
                    };
                    if !buildings.contains(&home) {
                        continue;
                    }
                    let heights = self.air_order_heights(u);
                    self.add_strafe_order(u, None, Some(home), false, QueuePos::New, false);
                    if let Some((alt, sharp)) = heights {
                        self.set_cruising_alt(u, alt);
                        self.set_sharp_turn(u, sharp);
                    }
                }
                Some(host) if buildings.contains(&host) => {
                    self.clear_orders(u);
                    self.buildings[host].launching.retain(|&x| x != u);
                }
                Some(_) => {}
            }
        }
    }

    /// **`Group::action_scramble@007111c0`** (item 3912, `docs/GOLDEN.md`
    /// §159): `OPTION_AIRBASE_SCRAMBLE`, `CommandPackage::process_scramble@
    /// 00947ae0` (byte 0x24). Down each member's `inside_down` chain
    /// (`+0x28`/`+0x3e`), every passenger that is a unit (`+0x18`), domain
    /// 2, not busy (`UnitData::is_busy@0060a370`), no missile and **full**
    /// (`mana_burn`, `+0x96`, 0) goes up over its host's own point:
    ///
    /// - a fixed-wing plane, `add_air_patrol_order(host.x, host.y, host, 1)`
    ///   (`7114dc`..`7114fa`) — the action bit; the queue argument the
    ///   listing pushes is the host's `y`, which the fixed-wing arm never
    ///   reads;
    /// - a helicopter (`+0x2b4 & 0x20`), a `MOVE_TO` built in place to the
    ///   host's cell centre with the action bit: [`Sim::launch_one`]'s
    ///   helicopter arm. Read only: no capture holds a helicopter here.
    ///
    /// SEAM: [`Sim::is_busy`] is not called, as in
    /// [`Sim::group_action_launch_patrol`] — a plane inside holds no pack
    /// or unpack and is never entering or exiting here.
    pub fn group_action_scramble(&mut self, buildings: &[usize]) {
        for (b, u) in self.planes_inside(buildings) {
            if !self.is_air_unit(u) || self.units[u].mana_burn != 0 || self.is_missile(u) {
                continue;
            }
            let at = self.buildings[b].pos;
            self.launch_one(u, at, b);
        }
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
    pub fn has_nuke(&self, b: usize) -> bool {
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

    /// `ObjectData::num_aircraft_here@00645330` with its argument 0, on a
    /// base that is no carrier: every live aircraft of the base's owner whose
    /// `UnitData::home_base` is the base, standing in it or flying (item
    /// 1714). [`Self::aircraft_here`] counts the garrison alone.
    pub(crate) fn aircraft_homed(&self, base: usize) -> usize {
        let who = self.buildings[base].owner;
        (0..self.units.len())
            .filter(|&u| {
                self.units[u].alive()
                    && self.units[u].owner == who
                    && self.is_air_unit(u)
                    && self.home_base(u) == Some(base)
            })
            .count()
    }

    /// `Build::finished@00628490`'s aircraft arm (`6285c0`..): an Airbase
    /// training a type that does not fly like a helicopter
    /// (`+0x2b4 & 0x20` clear) answers -1 while
    /// `num_aircraft_here(0) >= num_aircraft_limit`, which at an Airbase is
    /// `MAX_AIRCRAFT_PER_AIRBASE` (10); the entry stays full and the handover
    /// repeats (item 1714).
    pub(crate) fn airbase_pad_full(&self, at: usize, ty: usize) -> bool {
        self.building_ident(at) == crate::build::Ident::Airbase
            && self.unit_types[ty].cols.unit_flags & crate::ai_load::uflags::HELICOPTER == 0
            && self.aircraft_homed(at) >= 10
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
    pub(crate) fn base_can_carry(&self, base: usize, u: usize) -> bool {
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
        // `num_aircraft_here(0) < num_aircraft_limit` (`006483c0:137`):
        // every plane homed here, standing or flying (item 3991).
        self.aircraft_homed(base) < 10
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
    /// (`6ffbcc`). The points are laid by the steps
    /// ([`crate::gather_view::action_gather_point_steps`]), member by member;
    /// this is the flight alone.
    pub(crate) fn gather_flight(&mut self, who: Player, airbases: &[usize], at: Pos) {
        if airbases.is_empty() {
            return;
        }
        if let Some(target) = self.gather_object(at) {
            self.group_action_launch_flight(who, airbases, target, Flight::Strike, Keys::default());
        }
    }
}

#[cfg(test)]
mod tests;
