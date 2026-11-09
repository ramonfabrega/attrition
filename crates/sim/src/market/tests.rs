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
