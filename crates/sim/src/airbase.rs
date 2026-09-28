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
const HELICOPTER: crate::tech::TypeId = 0x136;

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
    /// SEAM: a helicopter's move in place of the patrol (`703919`..
    /// `7039f0`), and the console's feedback; no capture holds a
    /// helicopter, and the feedback writes no state.
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
                self.add_air_patrol_order(u, at, Some(b), true);
            }
        }
        if all {
            return;
        }
        if let Some((_, b, u)) = best {
            self.add_air_patrol_order(u, at, Some(b), true);
        }
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
    /// SEAM: the `NUCLEARMISSILE` and `V2ROCKET` arms (a missile inside
    /// the base narrows the choice to missiles and asks `valid_target`),
    /// the rush rules' early refusal, and the `MOVE_TO` arm to another
    /// base, which takes the **farthest** plane that `can_carry` admits
    /// (`6fc797`..`6fc802`); no capture holds a missile, a rush rule or a
    /// second base.
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
        if kind != Flight::Strike || !live {
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
            let p = self.buildings[b].pos;
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
}
