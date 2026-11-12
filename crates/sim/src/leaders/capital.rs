//! **The lost capital** — item 4185, step V of the leaders line
//! (`docs/LEADERS.md` §11): `lost_capital_stamp`, `lost_capital_timer` and
//! `lost_capital_modifier`, held in [`crate::leaders::endgame::EndGame`].
//!
//! A **loss** (`Leader::lost_capital@006dc870`, [`Sim::lost_capital`]) sets
//! the timer, writes the 8.8 modifier and turns the stamp into a start
//! time. A **stop** (`Leader::recapture_capital@006d1ba0` and
//! `Build::activate@00623e20`'s Forbidden City arm,
//! [`Sim::stop_capital_clock`]) clears the timer and turns the stamp back
//! into a debt. A **refresh** inlined in `Leaders::process_all@006ed2a0`
//! pays the debt down one frame in `TIMER_REFRESH_RATIO` while no clock runs
//! ([`Sim::refresh_capital_clock`]). The **countdown**,
//! `Leader::process_elimination@006b8a20` ([`Sim::process_elimination`]),
//! defeats a loser whose clock runs out under elimination 1.
//!
//! The callers sit where the original's do: the loss and the recapture in
//! `Sim::capture_city`, the Forbidden City's close in `Sim::close_building`
//! and its activation in `Sim::activate` (`crate::cities::city`).

use crate::leaders::endgame::{DEFEAT_ELIMINATION, scaled_timer};
use crate::{Player, Sim};

/// The Forbidden City's `TypeIndex` (`0x213`, `has_wonder(O, 0x213)`; §11).
pub const FORBIDDEN_CITY: usize = 0x213;

/// The modifier's clamp before the city rescale, `[0x80, 0x200]` (§11).
pub const MODIFIER_FLOOR: i64 = 128;
/// The clamp's ceiling (§11).
pub const MODIFIER_CAP: i64 = 512;
/// The city rescale's ceiling, `0x300` (§11).
pub const MODIFIER_CITY_CAP: i64 = 768;

impl Sim {
    /// `LeaderData::get_team_terr@006d62e0`: `who`'s `territory` (`+0x9d8`,
    /// the border pass's count, [`crate::economy::Holdings::territory`])
    /// plus every live ally's — the sum `ai_census`'s `team_terr` makes.
    fn team_terr_of(&self, who: Player) -> i64 {
        let w = usize::from(who);
        (0..self.territory.players.len())
            .filter(|&i| i == w || (!self.leaders.defeated[i] && self.leaders.allied[w][i]))
            .map(|i| i64::from(self.economy.holdings[i].territory))
            .sum()
    }

    /// Whether `who` holds a live city flagged capital (`0x10`) other than
    /// `skip`: `LeaderData::find_capital@006eb930`'s first walk, the one
    /// that answers `who` itself.
    pub(crate) fn holds_own_capital(&self, who: Player, skip: Option<usize>) -> bool {
        self.cities
            .cities
            .iter()
            .enumerate()
            .any(|(k, c)| Some(k) != skip && c.alive && c.owner == who && c.capital)
    }

    /// `LeaderData::find_capital(−1, −1)@006eb930`'s holder: `who` while it
    /// holds a live capital of its own, else the first other slot holding a
    /// live city whose `was_capital` (`+0x68`) carries `who`'s bit, else
    /// `who`.
    pub(crate) fn capital_holder(&self, who: Player) -> Player {
        if self.holds_own_capital(who, None) {
            return who;
        }
        let bit = 1u64 << u32::from(who);
        (0..self.territory.players.len() as Player)
            .filter(|&j| j != who)
            .find(|&j| {
                self.cities
                    .cities
                    .iter()
                    .any(|c| c.alive && c.owner == j && c.was_capital & bit != 0)
            })
            .unwrap_or(who)
    }

    /// Whether `who` holds an active Forbidden City: `has_wonder(who, 0x213)`
    /// is 1 (2 is Conquer the World's bit alone).
    pub(crate) fn holds_forbidden_city(&self, who: Player) -> bool {
        self.buildings.iter().any(|b| {
            b.alive
                && b.active
                && b.owner == who
                && b.ty
                    .is_some_and(|r| self.tables.build_types[r].tree == Some(FORBIDDEN_CITY))
        })
    }

    /// Whether building `b` is a Forbidden City (`is(0x213)`).
    pub(crate) fn is_forbidden_city(&self, b: usize) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|r| self.tables.build_types[r].tree == Some(FORBIDDEN_CITY))
    }

    /// **The loss, `Leader::lost_capital@006dc870`** (§11), for loser `o`
    /// and other leader `a`: nothing while the timer runs; else the timer
    /// set, the modifier
    ///
    /// ```text
    /// w = land_size / num_nations
    /// m = clamp((To + w)·256 / (Ta + w), 128, 512), 256 on a zero side
    /// m = min(m·(O.city_num + 2) / (A.city_num + 2), 768) when A has fewer
    /// ```
    ///
    /// and the stamp `frame + stamp·256 / m`, every division truncating.
    /// Both `city_num`s count the city changing hands: the caller runs this
    /// after the taker's new city is recorded and before the old one closes.
    pub(crate) fn lost_capital(&mut self, o: Player, a: Player) {
        let ow = usize::from(o);
        if ow >= self.leaders.endgame.len() || self.leaders.endgame[ow].lost_capital_timer != 0 {
            return;
        }
        let to = self.team_terr_of(o);
        let ta = self.team_terr_of(a);
        let nations = self.territory.players.len() as i64;
        let w = if nations == 0 {
            0
        } else {
            i64::from(self.land_size()) / nations
        };
        let mut m = if to + w == 0 || ta + w == 0 {
            256
        } else {
            ((to + w) * 256 / (ta + w)).clamp(MODIFIER_FLOOR, MODIFIER_CAP)
        };
        let (oc, ac) = (i64::from(self.city_num(o)), i64::from(self.city_num(a)));
        if oc != 0 && ac != 0 && ac < oc {
            m = (m * (oc + 2) / (ac + 2)).min(MODIFIER_CITY_CAP);
        }
        let frame = self.frame;
        let e = &mut self.leaders.endgame[ow];
        e.lost_capital_timer = 1;
        e.lost_capital_modifier = m as i32;
        e.lost_capital_stamp = frame + e.lost_capital_stamp * 256 / m;
    }

    /// **The stop**, `Leader::recapture_capital@006d1ba0` and
    /// `Build::activate@00623e20`'s Forbidden City arm (§11): nothing unless
    /// the timer runs; else the timer cleared and the stamp the debt
    /// `(stamp − frame)·m / 256`, truncating toward zero. The modifier is
    /// kept.
    pub(crate) fn stop_capital_clock(&mut self, who: Player) {
        let frame = self.frame;
        let Some(e) = self.endgame_mut(who) else {
            return;
        };
        if e.lost_capital_timer != 1 {
            return;
        }
        e.lost_capital_timer = 0;
        e.lost_capital_stamp =
            (e.lost_capital_stamp - frame) * i64::from(e.lost_capital_modifier) / 256;
    }

    /// **The refresh**, inlined in `Leaders::process_all@006ed2a0` after the
    /// slot's `process_elimination` (§11): every `TIMER_REFRESH_RATIO` frames
    /// a stopped clock's debt gains one.
    pub(crate) fn refresh_capital_clock(&mut self, who: usize) {
        let r = i64::from(self.tables.tuning.timer_refresh_ratio);
        if r == 0 || self.frame % r != 0 {
            return;
        }
        if let Some(e) = self.leaders.endgame.get_mut(who)
            && e.lost_capital_stamp < 0
            && e.lost_capital_timer == 0
        {
            e.lost_capital_stamp += 1;
        }
    }

    /// `Game::retake_capital@00594530` for `who`: [`scaled_timer`] of
    /// `RETAKE_CAPITAL`, times the modifier over 256, truncating toward
    /// zero.
    ///
    /// SEAM: the two arms under `semaphore[2] & 2` (Conquer the World's
    /// Forbidden City doubling and the slot-0/1 halving) are not carried.
    pub fn retake_capital(&self, who: Player) -> i64 {
        let base = scaled_timer(self.tables.tuning.retake_capital, self.world.width());
        base * i64::from(self.leader_endgame(who).lost_capital_modifier) / 256
    }

    /// **`Cities::capture_city`'s two capital calls** (§11), after the
    /// taker's new city `newcity` is recorded and before the old `c`
    /// closes, so both `city_num`s count the city changing hands. E1,
    /// `Leader::lost_capital` at `735092`: `c` was the loser's capital and
    /// the loser holds no active Forbidden City and no other live capital
    /// of its own. E3, `Leader::recapture_capital` at `7350e2`: the new city
    /// is flagged capital and the taker holds no other.
    pub(crate) fn capture_capital_calls(
        &mut self,
        o: Player,
        a: Player,
        c: usize,
        was_capital: bool,
        newcity: Option<usize>,
    ) {
        if was_capital && !self.holds_forbidden_city(o) && !self.holds_own_capital(o, Some(c)) {
            self.lost_capital(o, a);
        }
        if let Some(nc) = newcity
            && self.cities.cities[nc].capital
            && !self.holds_own_capital(a, Some(nc))
        {
            self.stop_capital_clock(a);
        }
    }

    /// E4, `Build::activate@00623e20`'s Forbidden City arm (`625f64`, §11):
    /// the owner's running capital clock stops, a captured one's too.
    pub(crate) fn forbidden_city_activate(&mut self, b: usize) {
        if self.is_forbidden_city(b) {
            let who = self.buildings[b].owner;
            self.stop_capital_clock(who);
        }
    }

    /// E2, `Build::close@00628980`'s Forbidden City arm (`62912e`, §11): an
    /// active one closing while another leader holds the owner's former
    /// capital is the capital's loss, then `lost_a_city(O, 1, 1, A)`.
    ///
    /// SEAM: tribe bonus `0x17`'s second `find_capital` and
    /// `semaphore[1] & 8` are not carried.
    pub(crate) fn forbidden_city_close(&mut self, b: usize) {
        let who = self.buildings[b].owner;
        if !self.buildings[b].active
            || !self.is_forbidden_city(b)
            || self.ai.lobby.elimination == 3
            || self.capital_holder(who) == who
        {
            return;
        }
        let holder = self.capital_holder(who);
        self.lost_capital(who, holder);
        self.lost_a_city(who, true, Some(holder));
    }

    /// **The countdown, `Leader::process_elimination@006b8a20`** (§11):
    /// under elimination 1, a live leader whose clock runs and whose
    /// `retake_capital − (frame − stamp)` has fallen to 0 or below is
    /// defeated, type 1, by the holder `find_capital(−1, −1)` names (itself
    /// when no one holds its capital).
    pub(crate) fn process_elimination(&mut self, who: usize) {
        if self.ai.lobby.elimination != 1 {
            return;
        }
        let p = who as Player;
        let e = self.leader_endgame(p);
        if e.lost_capital_timer == 0 {
            return;
        }
        if self.retake_capital(p) - (self.frame - e.lost_capital_stamp) <= 0 {
            let by = self.capital_holder(p);
            self.defeat(p, DEFEAT_ELIMINATION, Some(by));
        }
    }
}

#[cfg(test)]
mod tests;
