//! **`ScenarioFuncSet::add_unit@009e2220`** (item 3917, `docs/GOLDEN.md`
//! §164): what the script's `create_unit(who, x, y, type, num)` does —
//! `ScenarioFuncSet::create_unit@009f4c50` is `add_unit(…, 0)` and nothing
//! else — and the two group actions it ends on, `Group::action_transport`
//! and `Group::dbg_jump_to_action`.
//!
//! ```text
//! who 1-based, alive and playing; num <= 2000; the type a unit type
//! type = get_graft(type); a TRANSPORTBARGE is refused
//! point = (x * 0xc0 + 0x60, y * 0xc0 + 0x60): x, y are quarter cells
//! a land type on an is_ocean cell, its leader able to transport:
//!     num × init_unit at the point, each added to a group;
//!     push_group(0); action_transport(1)            # each squad in a barge
//! a sea type on the ocean, a land type on land, an air type anywhere:
//!     num × find_nearby_spot(0, 0xc00, bearing 0x55555555), init_unit there
//!     an air type: return (its strafe, and a carrier's planes: SEAMs)
//!     push_group(0)
//! a sea type on land, a land type on the ocean with no transport: return
//! action_form(0, 0, QUEUE_NEW); dbg_jump_to_action
//! ```
//!
//! SEAMS: the script's named groups (`clear_group`, `add_to_group`, and the
//! host swap after the transport), which are state this crate does not
//! keep; the air arm's `STRAFEORDER` and height; the Aircraft Carrier's
//! planes (`num_aircraft_limit`); `action_transport`'s scenario
//! `ignore_orders` sweep and its `is_busy` (a unit born on this call is
//! never busy); `dbg_jump_to_action`'s `semaphore & 4` (clear in a game)
//! and its two action arms, a `GARRISON` (`insert_inside`, `repath`) and a
//! `GATHER` (`set_new_location` at the target), which a unit born here never
//! holds.

use crate::group::Group;
use crate::movement::Angle;
use crate::orders::{Body, QueuePos, index};
use crate::transport::ty::{MERCHANTFLEET, TRANSPORTBARGE};
use crate::{Player, Pos, Sim};

/// The bearing every placement here sweeps from (`0x55555555`).
const SWEEP: Angle = Angle::INITIAL;
/// `add_unit`'s placement radius, and `dbg_jump_to_action`'s ring.
const ADD_RADIUS: i32 = 0xc00;
const JUMP_MIN: i32 = 0xc0;
const JUMP_MAX: i32 = 0x600;

impl Sim {
    /// The unit-type index a tree id names.
    fn unit_type_of_tree(&self, id: usize) -> Option<usize> {
        self.tables
            .unit_types
            .iter()
            .position(|t| t.tree == Some(id))
    }

    /// `ScenarioFuncSet::add_unit(who + 1, x, y, ty, num, 0)`: `ty` a unit
    /// type, `(x, y)` in quarter cells. The units it made, in birth order
    /// (heads of squads), or `None` for the original's `-1`.
    pub fn scenario_add_unit(
        &mut self,
        who: Player,
        ty: usize,
        (x, y): (i32, i32),
        num: i32,
    ) -> Option<Vec<usize>> {
        if who as usize >= self.players.len() || !(0..=2000).contains(&num) {
            return None;
        }
        // `LeaderData::get_graft`: the leader's own version of the type.
        let ty = match (self.tables.unit_types[ty].tree, self.tech.get(who as usize)) {
            (Some(t), Some(p)) if t < self.tables.tech_tree.types.len() => self
                .tables
                .tech_tree
                .get_graft(&self.tables.setup, p, Some(t))
                .and_then(|g| self.unit_type_of_tree(g))
                .unwrap_or(ty),
            _ => ty,
        };
        if self.tables.unit_types[ty].tree.is_some_and(|t| {
            t == TRANSPORTBARGE || self.tables.tech_tree.is(t, TRANSPORTBARGE, false)
        }) {
            return None;
        }
        let at = Pos::new(x * 0xc0 + 0x60, y * 0xc0 + 0x60);
        // `WorldData::valid` on the cell.
        let c = at.cell();
        if !(0..self.world.width()).contains(&c.x) || !(0..self.world.height()).contains(&c.y) {
            return None;
        }
        let ocean = self.world.is_ocean(at.cell());
        let domain = self.tables.unit_types[ty].combat.domain;
        use crate::attrition::Domain;
        let mut g = Group::stack(who);
        let mut made = Vec::new();
        match (domain, ocean) {
            (Domain::Land, true) => {
                if self.transport_level(who) == crate::transport::TransportType::None {
                    return Some(made);
                }
                for _ in 0..num {
                    let u = self.init_unit(who, ty, at);
                    self.group_add(&mut g, u);
                    made.push(u);
                }
                if g.list.is_empty() {
                    return Some(made);
                }
                self.push_group(&mut g, false);
                self.group_action_transport(&g, true);
                // `009e2611`..`009e2650`: each member short of the local
                // group (`+0x8cc`) is overwritten with its host's index
                // (`+0x82`) when it has one — so the group formed and
                // jumped below is the barge, once per passenger.
                for m in &mut g.list {
                    if let Some(host) = self.units[*m].inside_unit {
                        *m = host;
                    }
                }
            }
            (Domain::Sea, false) => return Some(made),
            _ => {
                for _ in 0..num {
                    let Some(spot) = self.find_nearby_spot_type(ty, at, 0, ADD_RADIUS, 0, SWEEP)
                    else {
                        continue;
                    };
                    let u = self.init_unit(who, ty, spot);
                    self.group_add(&mut g, u);
                    made.push(u);
                }
                if g.list.is_empty() || domain == Domain::Air {
                    return Some(made);
                }
                self.push_group(&mut g, false);
            }
        }
        // `push_group` copied the group into a pool slot; what is formed
        // and jumped is the **local** group (`local_9e8`), so the slot's
        // record never takes the formation (run1511's slot 0 on 641:
        // `form` −1, `form_num` 0, `ox`/`oy` 0).
        let local = Group {
            army: None,
            pushed: None,
            ..g
        };
        self.group_action_form(&local, 0, 0, QueuePos::New);
        self.group_dbg_jump_to_action(&local);
        Some(made)
    }

    /// `Group::action_transport(kill_on_fail)@00702620`: each active
    /// captain on the map, of the land, is put in a boat of its leader's
    /// `TRANSPORTBARGE` (a caravan's `MERCHANTFLEET`) born at its point and
    /// set at the nearest water within `unit_board_distance`. When there is
    /// none, the boat is set at the unit's own point all the same with the
    /// argument 1, and dies with 0.
    pub fn group_action_transport(&mut self, g: &Group, keep: bool) {
        for &u in &g.list {
            if !self.units[u].alive() || !self.units[u].on_map || !self.is_captain(u) {
                continue;
            }
            if self.unit_domain_of(u) != crate::attrition::Domain::Land {
                continue;
            }
            let base = if self.unit_is(u, crate::transport::ty::CARAVAN) {
                MERCHANTFLEET
            } else {
                TRANSPORTBARGE
            };
            let who = self.units[u].owner;
            let Some(ty) = self.unit_type_of_tree(self.current_upgrade_id(who as usize, base))
            else {
                continue;
            };
            let from = self.units[u].pos;
            let Some(b) = self.transport_boat(who, ty, from) else {
                continue;
            };
            let spot = self.find_nearby_spot_type_for(
                ty,
                b,
                from,
                0,
                self.tables.tuning.unit_board_distance,
                SWEEP,
            );
            if spot.is_none() && !keep {
                self.die_with(b, 0);
                continue;
            }
            self.set_new_location(b, spot.unwrap_or(from), true);
            self.clear_orders(u);
            self.board(u, b);
        }
    }

    /// `Group::dbg_jump_to_action@00707a00`: every active member on the map
    /// is snapped to where its action is going.
    pub fn group_dbg_jump_to_action(&mut self, g: &Group) {
        for &u in &g.list {
            if self.units[u].alive() && self.units[u].on_map {
                self.unit_dbg_jump_to_action(u);
            }
        }
    }

    /// `Unit::dbg_jump_to_action@00602ea0`'s move arm: with a move-family
    /// order current, every leading order of types 1–4, `CHANGE_FORM`,
    /// `GROUP_MOVE` and `GROUP_ATTACK_TO` is popped, the unit taking each
    /// one's angle, and the unit is set at the last one's point — or at
    /// `find_nearby_spot(0xc0, 0x600)` round it when the point is invalid,
    /// or where it stands when that finds nothing.
    pub fn unit_dbg_jump_to_action(&mut self, u: usize) {
        let jumps = index::is_move_family;
        if !jumps(self.order_type(u)) {
            return;
        }
        let mut to = self.units[u].pos;
        while let Some(o) = self.current_order(u) {
            let Body::Move(m) = o.body else { break };
            if !jumps(o.index()) {
                break;
            }
            to = m.dest;
            self.unit_set_angle(u, m.angle);
            self.kill_current_order(u);
        }
        let tile = Pos::new(to.x / 0xc0, to.y / 0xc0);
        if self.invalid_loc(u, tile, false, false, false, false, false) != 0 {
            let near = self.units[u].ty.and_then(|ty| {
                self.find_nearby_spot_type_for(ty, u, to, JUMP_MIN, JUMP_MAX, SWEEP)
            });
            to = near.unwrap_or(self.units[u].pos);
        }
        self.set_new_location(u, to, true);
    }
}

#[cfg(test)]
mod tests;
