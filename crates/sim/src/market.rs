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
        let rate = self.tuning.market_cycle_rate;
        if !(frame == 0 || rate < 2 || frame % i64::from(rate) == 0) {
            return;
        }
        let t = self.market.tick;
        for i in 0..GOODS {
            let ti = t + i as i32;
            if !(t == 0 || ti & 7 == 0) {
                continue;
            }
            // The price step: one good a tick, each good every 256 ticks
            // (the test is on the low byte of `tick + i`).
            if ti & 0xff == 0 {
                let eq = self.tuning.market_equilibrium;
                let p = self.market.price[i];
                if p > eq {
                    self.market.price[i] = if eq == 0 { p - 1 } else { p - p / eq };
                } else if p < eq {
                    let up = p + 1;
                    self.market.price[i] = if up < self.tuning.market_basement {
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
                self.market.flux_length[i] -= 1;
                self.market.flux_length[i] < 1
            };
            if fresh {
                self.calc_market(i);
            }
            self.market.flux[i] += self.market.delta_flux[i];
        }
        self.market.tick += 1;
    }

    /// `GameDaemon::calc_market(i)`: three draws — two for the next flux,
    /// one for the trend's length — and the per-tick step.
    fn calc_market(&mut self, i: usize) {
        let min_variance = self.tuning.market_min_variance;
        let mut v = self.market.price[i] / 2;
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
        self.market.next_flux[i] = (b - (v + 1)) + a;
        let range = self.tuning.market_trend_range;
        let spread = if range <= 1 {
            0
        } else {
            self.mark(SITE_LENGTH);
            self.rng.roll() % range
        };
        let len = self.tuning.market_min_trend + spread;
        let diff = self.market.next_flux[i] - self.market.flux[i];
        let sign = if diff < 1 { diff >> 31 } else { 1 };
        self.market.flux_length[i] = len;
        self.market.delta_flux[i] = ((len - 1) * sign + diff) / len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Rng;
    use crate::tuning::Tuning;
    use crate::world::World;

    /// A sim on a blank world, its stream set to `seed`.
    fn sim_with(seed: u32) -> Sim {
        let mut s = Sim::new(Tuning::RON, World::new(8, 8), 2);
        s.rng = Rng::new(seed);
        s
    }

    /// The word run12's dump prints entering frame 0 (`docs/ORACLE.md`), two
    /// draws on — the AI's sweep is the frame's first two draws.
    fn after_sweep() -> u32 {
        let mut r = Rng::new(0x3bd3_9ae9);
        r.roll();
        r.roll();
        r.seed
    }

    /// Run12, frame 0: the `end_frame` dump's `GAME` block, six goods each.
    /// Good 0's price stepped first (`market 50 → 51`), then every good drew
    /// its flux and trend, and the flux stepped once. These eighteen draws
    /// sit at offsets 2–19 of the frame and nowhere else (`docs/SYNC.md`
    /// §4).
    #[test]
    fn run12_s_frame_0_market_from_the_traced_word() {
        let mut s = sim_with(after_sweep());
        s.calc_markets(0);
        assert_eq!(s.market.price, [51, 50, 50, 50, 50, 50]);
        assert_eq!(s.market.next_flux, [-13, -9, -11, -3, -1, -9]);
        assert_eq!(s.market.flux_length, [15, 8, 21, 22, 11, 20]);
        assert_eq!(s.market.delta_flux, [-2, -3, -2, -1, -2, -2]);
        assert_eq!(s.market.flux, [13, 12, 13, 14, 13, 13]);
        assert_eq!(s.market.tick, 1);
        let mut r = Rng::new(after_sweep());
        for _ in 0..18 {
            r.roll();
        }
        assert_eq!(s.rng.seed, r.seed, "eighteen draws, no more");
    }

    /// After the first tick good `i` is visited on the ticks where
    /// `(tick + i) & 7 == 0` — good 1 at 7, 15, 23 …, good 0 at 8, 16 … —
    /// and draws only when its trend has run out: frames 1–3 draw nothing
    /// (run12), and good 1's eight-tick trend renews on its eighth visit,
    /// tick 63, the first redraw of the game.
    #[test]
    fn the_rota_visits_one_good_in_eight_and_renews_on_the_count() {
        let mut s = sim_with(after_sweep());
        s.calc_markets(0);
        let seed = s.rng.seed;
        for f in 1..=7 {
            s.calc_markets(f);
        }
        assert_eq!(s.rng.seed, seed, "ticks 1–7 draw nothing");
        assert_eq!(s.market.tick, 8);
        assert_eq!(s.market.flux_length[1], 7, "good 1 was visited at tick 7");
        assert_eq!(s.market.flux[1], 12 - 3, "and its flux stepped by delta");
        assert_eq!(s.market.flux_length[0], 15, "good 0 not yet");
        s.calc_markets(8);
        assert_eq!(s.market.flux_length[0], 14);
        assert_eq!(s.market.flux[0], 13 - 2);
        assert_eq!(s.rng.seed, seed);
        for f in 9..=62 {
            s.calc_markets(f);
        }
        assert_eq!(s.market.flux_length[1], 1);
        assert_eq!(s.rng.seed, seed, "no trend has run out before tick 63");
        s.calc_markets(63);
        assert_ne!(s.rng.seed, seed, "good 1 renewed its trend");
        assert!(s.market.flux_length[1] >= 8);
        let mut r = Rng::new(seed);
        for _ in 0..3 {
            r.roll();
        }
        assert_eq!(s.rng.seed, r.seed, "three draws");
    }

    /// The price walks toward the equilibrium one step per 256 ticks, two
    /// steps while under the basement, and down by a 65th above it. On tick
    /// 0 only good 0 steps (`(tick + i) & 0xff == 0`).
    #[test]
    fn the_price_step() {
        for (start, expect, why) in [
            (5, 7, "under the basement: +2"),
            (100, 100 - 100 / 65, "above: −p/65"),
            (65, 65, "at equilibrium: still"),
            (50, 51, "under: +1"),
        ] {
            let mut s = sim_with(1);
            s.market.price = [start, 100, 5, 50, 50, 50];
            s.calc_markets(0);
            assert_eq!(s.market.price[0], expect, "{why}");
            assert_eq!(s.market.price[1], 100, "good 1 steps on tick 255");
            assert_eq!(s.market.price[2], 5);
        }
    }

    /// `delta_flux` rounds away from zero: `(len − 1)·sign` before the
    /// division.
    #[test]
    fn delta_rounds_away_from_zero() {
        let mut s = sim_with(1);
        s.tuning.market_trend_range = 1; // no spread: len = MIN_TREND = 8
        s.market.flux = [15; GOODS];
        s.calc_markets(0);
        for i in 0..GOODS {
            let diff = s.market.next_flux[i] - 15;
            let expect = if diff >= 0 {
                (diff + 7) / 8
            } else {
                (diff - 7) / 8
            };
            assert_eq!(s.market.delta_flux[i], expect);
            assert_eq!(s.market.flux_length[i], 8);
        }
    }
}
