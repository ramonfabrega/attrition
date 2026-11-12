//! **A unit's own training queue: the Aircraft Carrier's planes** (item
//! 3981, `docs/AI.md` §413).
//!
//! Three writers of `UnitData +0xa0 num_queued` and `+0x64 queue_time`, the
//! only three the export holds:
//!
//! ```text
//! Group::action_spell@006fe1a0:171, a unit type whose +0x218 is 2 (air):
//!   for each member, active and on the map:
//!     is_castable(spelltypes[BUILDAIR 0x295], o, who, 0)  else next
//!     num_queued < 10 and num_aircraft_here(0) < num_aircraft_limit
//!                                                           else next
//!     t = current_upgrade(FIGHTERBOMBER)                  # whatever was pressed
//!     can_pay_cost(who, -1, -1, 1)                          else return
//!     pay_cost(who, -1, -1, 1, 0)                           # escrowed
//!     num_queued += 1; leader num_queued[t] += 1; the trainer's tally
//!
//! Unit::process@00610bc0:288, after the attrition, before the heal:
//!   num_queued != 0 and leader_flags & 2:
//!     t = current_upgrade(FIGHTERBOMBER)
//!     queue_time += ACCEL_TRAIN
//!     queue_time >= train_time(t):
//!       pop_cap < pop(t) + control: queue_time -= ACCEL_TRAIN  # held
//!       else num_queued -= 1; queue_time = 0; the tallies down;
//!            init_unit(who, t, here); go_inside(this)
//!
//! Unit::action_unqueue@005e1f20(p), the cancel (`process_unqueue`'s unit
//! arm): num_queued -= 1 (queue_time = 0 only at zero); the tallies down;
//!   p != 0: unpay_cost(who, -1, -1)
//! ```
//!
//! run1066 holds all three on the carrier `0/14`: the two presses on 612 and
//! 614 (`num_queued` 1 on 614, 2 on 616), the cancel on 620 (1 on 622, the
//! clock kept at 900), and the plane out of the clock on 817 (20400 against
//! the Persians' Fighter-Bomber line's `train_time`).
//!
//! SEAMS: the arm's two refusals' messages (the console's feedback, no
//! state); `ai_speed` above 1 (a lobby setting); the pop
//! cap's message flag (`player.flags & 0x800`); `unpay_cost`'s
//! `get_cost(…, 1, 1, -1)`, whose fourth argument counts one fewer in the
//! ramp, priced here at the count as it stands; `go_inside`'s selection
//! and the trainer's President (`train_time`'s reach clause, read off a
//! building).

use crate::{Player, Sim, cities::unit_unqueue_view, territory::attrition};

/// `LeaderData::current_upgrade`'s argument, the Fighter-Bomber line.
const FIGHTERBOMBER: i32 = unit_unqueue_view::FIGHTERBOMBER;
/// `BUILDAIR`, the carrier's craft (`craftrules.xml`'s 33rd record).
const BUILDAIR: i32 = 0x295;
/// The Aircraft Carrier's lineage.
const AIRCRAFTCARRIER: usize = 0x15f;
/// `num_queued < 10`.
const MAX_QUEUED: i16 = 10;

impl Sim {
    /// `Group::action_spell`'s carrier arm: `ti` a unit `TypeIndex` whose
    /// domain is air. `None` when the arm is not taken (the caller runs the
    /// spell arms), else the planes queued.
    pub(crate) fn carrier_action_spell(&mut self, members: &[usize], ti: i32) -> Option<usize> {
        if !(0x32..0x19e).contains(&ti) {
            return None;
        }
        let rec = self.unit_rec_of_index(ti)?;
        if self.tables.unit_types[rec].combat.domain != attrition::Domain::Air {
            return None;
        }
        let mut queued = 0;
        for &u in members {
            let unit = &self.units[u];
            if !unit.alive() || !unit.on_map {
                continue;
            }
            if !self.spell_castable(BUILDAIR, u) {
                continue;
            }
            if self.units[u].num_queued >= MAX_QUEUED
                || self.aircraft_aboard(u) >= self.unit_aircraft_limit(u)
            {
                continue;
            }
            let who = self.units[u].owner;
            let Some(t) = self.upgrade_record(who, FIGHTERBOMBER) else {
                return Some(queued);
            };
            let charges = self.price_of(who, t);
            let available = self.economy.holdings[who as usize].available;
            if crate::economy::cost::affordable(
                &charges,
                &self.economy.ledgers[who as usize],
                &available,
                true,
            ) < 1
            {
                return Some(queued);
            }
            crate::economy::cost::pay(
                &charges,
                &mut self.economy.ledgers[who as usize],
                &available,
                true,
            );
            self.units[u].num_queued += 1;
            self.roster.muster[who as usize].queued_by_type[t] += 1;
            self.track_tree_queued(who, t, 1);
            queued += 1;
        }
        Some(queued)
    }

    /// `ObjectData::num_aircraft_here(0)` on a unit: the planes riding it.
    fn aircraft_aboard(&self, carrier: usize) -> i32 {
        self.units
            .iter()
            .filter(|p| {
                p.alive()
                    && p.inside_unit == Some(carrier)
                    && p.ty.is_some_and(|t| {
                        self.tables.unit_types[t].combat.domain == attrition::Domain::Air
                    })
            })
            .count() as i32
    }

    /// `ObjectData::num_aircraft_limit@006454a0` on a unit: a carrier's
    /// `MAX_AIRCRAFT_PER_CARRIER`. The Airbase and Missile Silo arms are a
    /// building's ([`Sim::airbase_pad_full`]).
    fn unit_aircraft_limit(&self, u: usize) -> i32 {
        if self.unit_line_is(u, AIRCRAFTCARRIER) {
            self.tables.tuning.max_aircraft_per_carrier
        } else {
            0
        }
    }

    /// `Unit::process@00610bc0:288`–`:420`: the queue's clock, one call.
    pub(crate) fn carrier_queue_tick(&mut self, u: usize) {
        if self.units[u].num_queued == 0 {
            return;
        }
        let who = self.units[u].owner;
        if !self.leader_in_play(who) {
            return;
        }
        let Some(t) = self.upgrade_record(who, FIGHTERBOMBER) else {
            return;
        };
        let target = self.unit_train_target(who, t, false);
        let accel = self.tables.tuning.accel_train;
        self.units[u].queue_time += accel;
        if self.units[u].queue_time < target {
            return;
        }
        let muster = &self.roster.muster[who as usize];
        if muster.cap < self.tables.unit_types[t].price.pop + muster.control {
            self.units[u].queue_time -= accel;
            // The capped launch counts, every frame the cap holds
            // (`Unit::process`, `docs/LEADERS.md` §12.2).
            if let Some(n) = self.leaders.pop_issues.get_mut(who as usize) {
                *n += 1;
            }
            return;
        }
        self.units[u].num_queued -= 1;
        self.units[u].queue_time = 0;
        self.untrack_queued(who, t);
        let plane = self.init_unit(who, t, self.units[u].pos);
        self.board(plane, u);
    }

    /// `Unit::action_unqueue@005e1f20(refund)`: one off the unit's queue.
    pub fn unit_action_unqueue(&mut self, u: usize, refund: bool) {
        if self.units[u].num_queued == 0 {
            return;
        }
        self.units[u].num_queued -= 1;
        if self.units[u].num_queued == 0 {
            self.units[u].queue_time = 0;
        }
        let who = self.units[u].owner;
        let Some(t) = self.upgrade_record(who, FIGHTERBOMBER) else {
            return;
        };
        self.untrack_queued(who, t);
        if refund {
            let charges = self.price_of(who, t);
            let ledger = &mut self.economy.ledgers[who as usize];
            for (b, c) in ledger.bucket.iter_mut().zip(charges) {
                *b += c;
            }
        }
    }

    /// `leader_flags & 2`, the leader in play: a live city or a live unit,
    /// the standing answer ([`Sim::scout_leader_in_play`]'s).
    fn leader_in_play(&self, who: Player) -> bool {
        who < 8
            && (self.city_num(who) > 0 || self.units.iter().any(|u| u.owner == who && u.alive()))
    }
}
