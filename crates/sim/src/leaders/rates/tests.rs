use super::{BATTLE_SENTINEL, DAMAGE, DEATHS, HITS, KILLS, RATE_LEAVES, Rates, fold_average};

/// The empty fold decays by 7/8 and a full one halves toward the count
/// ×100 (`docs/LEADERS.md` §3.2 step 3). From the battle's sentinel, five
/// empty folds reach 33099 — run1075's who 0 on its first block, 2034,
/// `frame_battle` 1750 — and the lock-out's 36 reach 525.
#[test]
fn the_averages_decay_by_seven_eighths_and_halve_toward_the_count() {
    let mut v = BATTLE_SENTINEL;
    for _ in 0..5 {
        v = fold_average(v, 0);
    }
    assert_eq!(v, 33099);
    for _ in 5..36 {
        v = fold_average(v, 0);
    }
    assert_eq!(v, 525);
    assert_eq!(fold_average(0, 300), 150);
    assert_eq!(fold_average(u16::MAX, u16::MAX), u16::MAX);
}

/// Steps 1, 2 and 6: the raw count goes to `_fifteen_seconds`, the window
/// ×100 (mod 2^16) feeds the average, and the window starts again.
#[test]
fn a_fold_totals_the_window_and_clears_it() {
    let mut r = Rates::default();
    r.add(HITS, 5);
    r.add(DAMAGE, 1);
    r.add(DAMAGE, 1);
    r.fold(50, 0);
    assert_eq!(r.fifteen[HITS], 5);
    assert_eq!(r.fifteen[DAMAGE], 2);
    assert_eq!(r.average[HITS], 250);
    assert_eq!(r.average[DAMAGE], 100);
    assert_eq!(r.current, [0; 4]);
    // 656 × 100 is 65600, 64 mod 2^16: the zero test is on the product.
    r.current[HITS] = 656;
    r.fold(100, 0);
    assert_eq!(r.average[HITS], (64 + 250) >> 1);
    assert_eq!(r.fifteen[HITS], 661);
}

/// The battle test: at age 0, three deaths and no kills in one window give
/// `D = 150`, past the size test's 125 and the lead's 20, a defeat —
/// the sentinel and the frame. Inside the lock-out nothing fires, and a
/// tie writes nothing.
#[test]
fn the_battle_event_writes_the_sentinel_and_the_frame() {
    let mut r = Rates::default();
    r.current[DEATHS] = 3;
    r.fold(500, 0);
    assert_eq!(r.average[DEATHS], BATTLE_SENTINEL);
    assert_eq!(r.average[KILLS], BATTLE_SENTINEL);
    assert_eq!(r.frame_battle, 500);
    // 1750 frames later, still locked: 7/8 decay only.
    r.fold(2250, 0);
    assert_eq!(r.frame_battle, 500);
    let mut t = Rates::default();
    t.current[DEATHS] = 3;
    t.current[KILLS] = 3;
    t.fold(50, 0);
    assert_eq!((t.average[DEATHS], t.average[KILLS]), (150, 150));
    assert_eq!(t.frame_battle, 0, "a tie is no event");
}

/// From the decayed 525, past the lock-out, three deaths and no kills
/// halve `D` toward 300 while `K` keeps 7/8: a **victory** at age 2
/// (`docs/LEADERS.md` §3.2, the arithmetic's own consequence).
#[test]
fn deaths_alone_can_record_a_victory_past_the_lock_out() {
    let mut r = Rates {
        frame_battle: 50,
        average: [525, 525, 0, 0],
        ..Rates::default()
    };
    r.current[DEATHS] = 3;
    r.fold(1850, 2);
    assert_eq!(r.frame_battle, 1850);
    assert_eq!(r.average[DEATHS], BATTLE_SENTINEL);
}

/// Every leaf is named once.
#[test]
fn every_leaf_is_named_once() {
    let mut names = RATE_LEAVES.to_vec();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 13);
}
