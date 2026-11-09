//! The market's price cycle — `GameDaemon::calc_markets@00732180` and
//! `GameDaemon::calc_market@00732270`, `docs/SYNC.md` §3.1.
//!
//! Six goods, each with a price that drifts toward `MARKET_EQUILIBRIUM` one
//! step every 256 ticks, and a *flux* that walks from its current value to
//! a freshly drawn `next_flux` over a freshly drawn `flux_length` ticks.
//! The three draws a good takes when its trend runs out are on the sync
//! stream, and on frame 0 every good takes them: eighteen draws between the
//! AI's sweep and the first unit, which is where run12's dump puts them.
//! What the flux *means* to trade — `docs/ECONOMY.md`'s market — is not
//! wired here; this module keeps the stream and the state the trade reads.

use crate::Sim;

/// The three draw sites, under the original's own offsets from
/// `GameDaemon::calc_market@00732270`. [`Sim::mark`] writes them into
/// [`Sim::phase_marks`], so this mechanic's draws line up against
/// `rondata::trace`'s **site by site** rather than by a count
/// (`docs/SYNC.md` §5).
pub const SITE_A: &str = "GameDaemon::calc_market+0x54";
pub const SITE_B: &str = "GameDaemon::calc_market+0x7e";
pub const SITE_LENGTH: &str = "GameDaemon::calc_market+0xbe";

/// The six tradeable goods, in `Game::market[6]` order.
pub const GOODS: usize = 6;

/// `Game`'s market arrays and `market_tick`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Market {
    /// `market[i]`: the price.
    pub price: [i32; GOODS],
    /// `market_flux[i]`: the current flux, stepped by `delta_flux` a tick.
    pub flux: [i32; GOODS],
    /// `next_flux[i]`: where the flux is walking to.
    pub next_flux: [i32; GOODS],
    /// `delta_flux[i]`: the per-tick step, `(next − flux)` over the length,
    /// rounded away from zero.
    pub delta_flux: [i32; GOODS],
    /// `flux_length[i]`: ticks left on the trend.
    pub flux_length: [i32; GOODS],
    /// `market_tick`.
    pub tick: i32,
}

impl Default for Market {
    /// The start-of-game values as every dump prints them (`GAME`: `market
    /// 50`, `market_flux 15`, `next_flux 15`, `delta_flux 0`, `flux_length
    /// 1`); the writer in `Game::init` was not read.
    fn default() -> Market {
        Market {
            price: [50; GOODS],
            flux: [15; GOODS],
            next_flux: [15; GOODS],
            delta_flux: [0; GOODS],
            flux_length: [1; GOODS],
            tick: 0,
        }
    }
}

impl Sim {
    /// `GameDaemon::calc_markets`, once a frame from `GameDaemon::
    /// process_all` — between `Leaders::strategy_all` and the objects.
    pub fn calc_markets(&mut self, frame: i64) {
        let rate = self.tables.tuning.market_cycle_rate;
        if !(frame == 0 || rate < 2 || frame % i64::from(rate) == 0) {
            return;
        }
        let t = self.economy.market.tick;
        for i in 0..GOODS {
            let ti = t + i as i32;
            if !(t == 0 || ti & 7 == 0) {
                continue;
            }
            // The price step: one good a tick, each good every 256 ticks
            // (the test is on the low byte of `tick + i`).
            if ti & 0xff == 0 {
                let eq = self.tables.tuning.market_equilibrium;
                let p = self.economy.market.price[i];
                if p > eq {
                    self.economy.market.price[i] = if eq == 0 { p - 1 } else { p - p / eq };
                } else if p < eq {
                    let up = p + 1;
                    self.economy.market.price[i] = if up < self.tables.tuning.market_basement {
                        p + 2
                    } else {
                        up
                    };
                }
            }
            // A fresh trend on the first tick, and whenever the running one
            // has been counted down to nothing.
            let fresh = if t == 0 {
                true
            } else {
                self.economy.market.flux_length[i] -= 1;
                self.economy.market.flux_length[i] < 1
            };
            if fresh {
                self.calc_market(i);
            }
            self.economy.market.flux[i] += self.economy.market.delta_flux[i];
        }
        self.economy.market.tick += 1;
    }

    /// `GameDaemon::calc_market(i)`: three draws — two for the next flux,
    /// one for the trend's length — and the per-tick step.
    fn calc_market(&mut self, i: usize) {
        let min_variance = self.tables.tuning.market_min_variance;
        let mut v = self.economy.market.price[i] / 2;
        if v < min_variance {
            v = min_variance;
        }
        v = (v + 1) / 2;
        let (a, b) = if v < 1 {
            (0, 0)
        } else {
            self.mark(SITE_A);
            let a = self.rng.roll() % (v + 1);
            self.mark(SITE_B);
            let b = self.rng.roll() % (v + 1);
            (a, b)
        };
        self.economy.market.next_flux[i] = (b - (v + 1)) + a;
        let range = self.tables.tuning.market_trend_range;
        let spread = if range <= 1 {
            0
        } else {
            self.mark(SITE_LENGTH);
            self.rng.roll() % range
        };
        let len = self.tables.tuning.market_min_trend + spread;
        let diff = self.economy.market.next_flux[i] - self.economy.market.flux[i];
        let sign = if diff < 1 { diff >> 31 } else { 1 };
        self.economy.market.flux_length[i] = len;
        self.economy.market.delta_flux[i] = ((len - 1) * sign + diff) / len;
    }
}

#[cfg(test)]
mod tests;
