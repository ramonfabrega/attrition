//! The gather point, the player's rally point on a building
//! (`docs/PRODUCTION.md`, "The gather point"; `docs/GOLDEN.md` §39).
//!
//! `BuildData::gather` (`+0xb8`, a `PtrLinkListAbstract<GatherPoint>`, the
//! count at `+0xc8`) holds the points in order; `Build::add_gather_point@
//! 00622e70` and `Build::clear_gather@00623180` are its only writers.

use crate::world::Pos;

/// `GatherPoint`: `x`, `y` and `action`, the record the dump prints as
/// `GATHERPOINT`. `action` 0 is a point on the ground, 1 a friendly
/// object's point, 2 an enemy's, 3 an object named by `(o, who)` in
/// `(x, y)`; `(−1, −1)` with an action other than 3 is "inside"
/// (`GatherPoint::is_inside@00730400`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherPoint {
    pub pos: Pos,
    pub action: u8,
}

use crate::Sim;
use crate::build::{self, Ident};
use crate::movement::{Angle, find_angle};
use crate::orders::{MoveKind, QueuePos};
use crate::world::Player;

/// `find_nearby_spot`'s neutral bias, `0x55555555`, which the gather
/// block and the squad's target both pass.
const NEUTRAL: Angle = Angle(0x5555_5555);

/// The radius `Group::action_gather_point` asks for a City's Woodcutter
/// or Mine, and the one `come_out` searches around the head point: `0x600`.
const REACH: i32 = 0x600;

impl Sim {
    /// A member `action_gather_point` walks: alive, a building (always,
    /// here), and neither a University nor a Missile Silo (`is(0x1a4)`,
    /// `is(0x208)` at `6ff2a9` and `6ff2f7`).
    fn gather_member(&self, b: usize) -> bool {
        let bd = &self.buildings[b];
        bd.alive
            && !matches!(
                self.building_ident(b),
                Ident::University | Ident::MissileSilo
            )
    }

    /// `Group::action_gather_point@006ff1b0(x, y, action, add_to_end)` on a
    /// group of `who`'s buildings (`docs/PRODUCTION.md`, "The gather
    /// point"; run312). Under the emulator and on the capture alike:
    ///
    /// - `x < 0 || y < 0` is the Clear: `clear_gather` on every member;
    /// - the point is clamped to the world;
    /// - a member whose head point is "inside" turns `add_to_end` off;
    /// - a group of City centres alone clicking a forest (`TData & 0x30 ==
    ///   0x30`) or a mountain (`& 3 == 2`) takes its nearest friendly
    ///   Woodcutter's or Mine's point within `0x600`, the action unchanged;
    /// - a click on a building tile a member covers is the "inside" point
    ///   (−1, −1, 0) for every member — and on a Senate the first member's
    ///   list is cleared instead;
    /// - each member that trains (`build_flags & 0x80000000`), has a
    ///   garrison limit, or is a Senate takes the point, its list cleared
    ///   first unless `add_to_end`.
    ///
    /// SEAM: `action` 3 (an Airbase's strike through `action_flight`, and
    /// `add_gather_point`'s re-ordering of the base's planes under
    /// `build_masks & 8`); the Terracotta Army and the Kremlin beside the
    /// Senate (wonders with no ident here); `find_building`'s own metric
    /// (the nearest by `vector_dist` here). No capture reaches them.
    pub fn action_gather_point(
        &mut self,
        who: Player,
        members: &[usize],
        at: crate::Pos,
        action: i32,
        add_to_end: bool,
    ) {
        if at.x < 0 || at.y < 0 {
            for &b in members {
                if self.gather_member(b) {
                    self.clear_gather(b);
                }
            }
            return;
        }
        let mut at = self.restrict_pos(at);
        let inside_now = members
            .iter()
            .any(|&b| self.gather_member(b) && self.gather_inside(b));
        let append = add_to_end && !inside_now;
        let action = action as u8;
        let tile = at.tile();
        let mask = self.world.tile_mask(tile);
        // The City centres' snap (`6ff36c`..`6ff4a6`).
        if action != 3
            && !members.is_empty()
            && members.iter().all(|&b| {
                self.buildings[b]
                    .ty
                    .is_some_and(|t| build::is(&self.build_types, t, Ident::Village))
            })
        {
            let want = if mask & 0x30 == 0x30 {
                Some(Ident::Woodcutter)
            } else if mask & 3 == 2 {
                Some(Ident::Mine)
            } else {
                None
            };
            if let Some(ident) = want
                && let Some(w) = self.nearest_friendly_building(who, at, ident, REACH)
            {
                at = self.buildings[w].pos;
            }
        }
        // A click on a member's own footprint (`6ff4a6`..`6ff750`).
        let mut inside = false;
        if mask & 3 == 3 && !append {
            for &b in members {
                if !self.gather_member(b) || !self.covers_tile(b, tile) {
                    continue;
                }
                inside = true;
                if self.building_ident(b) == Ident::Senate {
                    if let Some(&first) = members.iter().find(|&&m| self.gather_member(m)) {
                        self.clear_gather(first);
                    }
                    return;
                }
                break;
            }
        }
        for &b in members {
            if !self.gather_member(b) || (action == 3 && self.building_ident(b) == Ident::Airbase) {
                continue;
            }
            let Some(t) = self.buildings[b].ty else {
                continue;
            };
            let takes = build::is_training_building(&self.build_types, t)
                || self.garrison_limit(b) != 0
                || self.building_ident(b) == Ident::Senate;
            if !takes {
                continue;
            }
            let (p, new) = if inside {
                (
                    GatherPoint {
                        pos: crate::Pos::new(-1, -1),
                        action: 0,
                    },
                    true,
                )
            } else {
                (GatherPoint { pos: at, action }, !append)
            };
            self.add_gather_point(b, p, new);
        }
    }

    /// `Build::add_gather_point@00622e70(x, y, action, pos)`: `QUEUE_NEW`
    /// clears the list first, and the point goes on its end.
    pub fn add_gather_point(&mut self, b: usize, p: GatherPoint, new: bool) {
        if new {
            self.clear_gather(b);
        }
        self.buildings[b].gather.push(p);
    }

    /// `Build::clear_gather@00623180`: the list emptied. (Its Airbase
    /// half, the base's planes re-ordered, is `action_gather_point`'s
    /// SEAM.)
    pub fn clear_gather(&mut self, b: usize) {
        self.buildings[b].gather.clear();
    }

    /// `BuildData::gather_inside@0046f180`: the head point is (−1, −1)
    /// with any action but 3.
    pub fn gather_inside(&self, b: usize) -> bool {
        self.buildings[b]
            .gather
            .first()
            .is_some_and(|p| p.action != 3 && (p.pos.x < 0 || p.pos.y < 0))
    }

    /// `ObjectsData::find_building(x, y, SEARCH_FRIENDLY, who, radius, …,
    /// FILTER_TYPE, ident)` as `action_gather_point` asks it: the nearest
    /// live building of `who` or an ally of that lineage within `radius`.
    fn nearest_friendly_building(
        &self,
        who: Player,
        at: crate::Pos,
        ident: Ident,
        radius: i32,
    ) -> Option<usize> {
        (0..self.buildings.len())
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive
                    && (bd.owner == who || self.is_ally(who, bd.owner))
                    && bd
                        .ty
                        .is_some_and(|t| build::is(&self.build_types, t, ident))
            })
            .map(|b| {
                let p = self.buildings[b].pos;
                (crate::world::vector_dist(p.x - at.x, p.y - at.y), b)
            })
            .filter(|&(d, _)| d <= radius)
            .min()
            .map(|(_, b)| b)
    }

    /// `ObjectsData::find_any_building_at(tx, ty, …, FILTER_SEEN, who)` as
    /// `come_out`'s routing asks it: a live building whose footprint
    /// covers the tile. SEAM: the seen filter.
    fn building_at_tile(&self, tile: crate::Pos) -> Option<usize> {
        (0..self.buildings.len()).find(|&b| self.buildings[b].alive && self.covers_tile(b, tile))
    }

    /// `come_out`'s **gather-point block** (`6181a3`..`618377`): a captain
    /// leaving a building with a point, not "inside", is given the free
    /// spot nearest its head point (`UnitType::find_nearby_spot(point, 0,
    /// 0x600, 0x55555555)`, `FILTER_NOT_ME` for a type with a
    /// `block_radius`), and `find_angle(spot − building)` becomes its
    /// `angle` (`+0x50`) and the exit sweep's bias in place of south.
    /// Returns the spot and the bearing; `None` leaves the exit as it was.
    pub(crate) fn gather_exit(&mut self, captain: usize, b: usize) -> Option<(crate::Pos, Angle)> {
        let head = *self.buildings[b].gather.first()?;
        if self.gather_inside(b) || head.action == 3 {
            // SEAM: action 3's point is an object's (`HotKeyGroups::copy`
            // and the object's own point); no capture sets one.
            return None;
        }
        let spot = if self.profile(crate::combat::Obj::Unit(captain)).block_radius == 0 {
            self.find_nearby_spot_coll(
                captain,
                head.pos,
                0,
                REACH,
                0,
                NEUTRAL,
                None,
                crate::orders::Coll::None,
            )
        } else {
            self.find_nearby_spot(captain, head.pos, 0, REACH, 0, NEUTRAL, None)
        }?;
        let from = self.buildings[b].pos;
        let bearing = find_angle(spot.x - from.x, spot.y - from.y);
        self.units[captain].movement.heading = bearing;
        Some((spot, bearing))
    }

    /// `come_out`'s two **re-seats**: the captain swept again round its
    /// trainer's **land** exit ring (`(x_size + y_size) × 0x30 +
    /// UNIT_TRAIN_DISTANCE` out to `… + UNIT_TRAIN_MAX_DISTANCE`, step 0,
    /// no boat arm and no dying-building zero) from `bearing`, under
    /// `FILTER_ALL` ([`Coll::All`](crate::orders::Coll::All)), and
    /// `set_new_location(·, spot, 1, 1)` puts it there if the sweep finds a
    /// spot. Its members stay where the exit put them, round its first
    /// point.
    ///
    /// - **At a building point** (`61923e`..`619381`), the first thing the
    ///   routing does when `find_any_building_at` finds a building at the
    ///   last point, whatever it goes on to order: the bearing is
    ///   **trainer → that building's own point** (`find_angle(tb − b)`,
    ///   the register pair at `6192e4`/`61930b`).
    /// - **In the lone move arm** (`619ea2`..`619f86`): the bearing is
    ///   **trainer → the gather block's spot** (`[esp+0x44]`/`[esp+0x40]`
    ///   at `619efa`/`619ee5`), which is the exit's own bearing.
    ///
    /// run312 has both on the unit they move (`docs/GOLDEN.md` §39):
    /// - the Bowmen `0/17`, whose point is on 2008. The exit's bearing is
    ///   the free spot south-west of 2008, `0x4a590000`, and puts the
    ///   captain at (3336, 14376), with its members seated round it. The
    ///   re-seat's bearing is 2008 − 2007, due east, and its first
    ///   candidate is (3384, 14232), where the original has it on 1060;
    /// - the Citizen `0/10`, whose point is on the Woodcutter. The first
    ///   re-seat puts it at (3576, 29928), and the lone arm's puts it back
    ///   on its exit point (3576, 29976), where it stands there on 760.
    fn gather_reseat(&mut self, captain: usize, b: usize, bearing: Angle) {
        let Some(t) = self.buildings[b].ty else {
            return;
        };
        let (xs, ys) = (self.build_types[t].x_size, self.build_types[t].y_size);
        let near = self.tuning.unit_train_distance;
        let ring = (xs + ys) * 0x30 + near;
        let max = ring + (self.tuning.unit_train_max_distance - near);
        let host = self.buildings[b].pos;
        if let Some(spot) = self.find_nearby_spot_coll(
            captain,
            host,
            ring,
            max,
            0,
            bearing,
            None,
            crate::orders::Coll::All,
        ) {
            self.set_new_location(captain, spot, true);
        }
    }

    /// `come_out`'s **routing** (`618b22`..`619fe2`), for a captain with no
    /// order out of a building whose list is not "inside"; `spot` is the
    /// gather block's, `exit` where the captain now stands, `group` the
    /// squad's pushed group (882's push). One point is the only case a
    /// capture holds, and it is the last:
    ///
    /// - a building at the point: a citizen's build, repair or gather
    ///   (`action` ≠ 0 for the gather) on its own building; then a friendly
    ///   building with room the type can garrison, `action` ≠ 0: a
    ///   `GARRISONORDER` down the squad, `QUEUE_LAST`, and nothing more;
    /// - otherwise a move: `ATTACK_TO` for an armed unit (`+0x1e8`) whose
    ///   stance is not 5 and which is made at a Barracks, Stable or Dock
    ///   (`TypeData::where`), `MOVE_TO` for anything else; a squad with a
    ///   group goes through `Group::action_move_to(QUEUE_LAST, set_angle,
    ///   find_angle(target − exit))` to the free spot nearest `spot`, a lone
    ///   unit through `add_move_facing_order` to `spot`.
    ///
    /// SEAM: more than one point (the waypoints before the last); an
    /// enemy at the point (the attack arms) and a caravan's trade arm;
    /// `find_unit_with_radius`'s re-seat and the lone arm's second ring
    /// sweep, which put the unit back where the exit put it on every
    /// capture that reaches them.
    pub(crate) fn gather_route(
        &mut self,
        captain: usize,
        b: usize,
        spot: crate::Pos,
        exit: crate::Pos,
        group: Option<&crate::group::Group>,
    ) {
        if self.gather_inside(b) || !self.units[captain].orders.is_empty() {
            return;
        }
        let Some(&last) = self.buildings[b].gather.last() else {
            return;
        };
        if last.action == 3 {
            return;
        }
        let who = self.units[captain].owner;
        let Some(ty) = self.units[captain].ty else {
            return;
        };
        let mut open = true;
        if let Some(tb) = self.building_at_tile(last.pos.tile()) {
            let (host, there) = (self.buildings[b].pos, self.buildings[tb].pos);
            self.gather_reseat(captain, b, find_angle(there.x - host.x, there.y - host.y));
            let citizen = self.unit_types[ty].type_index;
            let owner = self.buildings[tb].owner;
            if matches!(citizen, 0x32 | 0x33) {
                if !self.buildings[tb].active && owner == who {
                    self.add_build_order(captain, tb, QueuePos::Last, false);
                    open = false;
                } else if self.buildings[tb].damage == 0 {
                    let gathers = self.buildings[tb]
                        .ty
                        .is_some_and(|t| self.build_types[t].has(build::flags::GATHER));
                    if last.action != 0
                        && self.buildings[tb].alive
                        && gathers
                        && self.building_ident(tb) != Ident::University
                        && owner == who
                    {
                        self.add_gather_order(captain, tb, QueuePos::Last, true);
                        open = false;
                    }
                } else {
                    self.add_repair_order(captain, tb, QueuePos::Last, false);
                    open = false;
                }
            }
            if open
                && self.buildings[tb].alive
                && self.buildings[tb].active
                && (owner == who || self.is_ally(who, owner))
                && self.garrison_limit(tb) != 0
                && self.buildings[tb]
                    .ty
                    .is_some_and(|bt| self.can_garrison(ty, bt))
                && last.action != 0
            {
                for f in self.squad_of(captain) {
                    self.add_garrison_order(f, tb, false, QueuePos::Last, false);
                }
                return;
            }
            if !open {
                return;
            }
        }
        let armed = self.profile(crate::combat::Obj::Unit(captain)).attack != 0;
        let from_trainer = self.unit_types[ty].garrison.trained_at.is_some_and(|w| {
            [Ident::Barracks, Ident::Stable, Ident::Dock]
                .iter()
                .any(|&i| build::is(&self.build_types, w, i))
        });
        let kind = if armed && self.units[captain].stance != 5 && from_trainer {
            MoveKind::AttackTo
        } else {
            MoveKind::MoveTo
        };
        let squad = self.unit_types[ty].combat.uber_size > 1;
        match group {
            Some(g) if squad => {
                let to = self
                    .find_nearby_spot(captain, spot, 0, -1, 0, NEUTRAL, None)
                    .unwrap_or(spot);
                let angle = find_angle(to.x - exit.x, to.y - exit.y);
                self.group_action_move_to(g, to, QueuePos::Last, true, angle, kind, true);
            }
            _ => {
                let host = self.buildings[b].pos;
                self.gather_reseat(captain, b, find_angle(spot.x - host.x, spot.y - host.y));
                let here = self.units[captain].pos;
                let angle = find_angle(spot.x - here.x, spot.y - here.y);
                self.add_move_facing_order(
                    captain,
                    spot,
                    kind,
                    QueuePos::Last,
                    true,
                    angle,
                    None,
                    false,
                );
            }
        }
    }
}
