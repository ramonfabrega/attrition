//! **The `cant_reach` tally** (item 3962, `docs/AI.md` §406): the rows of
//! `SubObjectData::flags & 0x10` against [`sim::Unit::cant_reach`], kept
//! apart from a widening's `firsts` the way [`Widened::rounds`] is, and
//! pinned per test.
//!
//! The bit is the one `Unit::think`'s one-in-sixteen idle gate reads.
//! `Group::action_stance` raises it on every member it presses, and that
//! writer is built; the original has at least two more that this crate
//! does not — a General's decoy squads' non-captain members are born with
//! it (parked 3962a) and `Object::take_damage` raises it on a struck
//! squad's captain (parked 3962b) — so on the day the row landed every
//! walk but the stance press's own read it ours 0 / theirs 1, 3,257 rows
//! over 377 tests. Folded into `firsts`, those would have re-pinned 398
//! sites and six floors for two unported writers; kept here, each walk's
//! count is still a pin, so a writer that lands, or a regression, moves a
//! number. An item that ports a writer folds its rows into `firsts`.
//!
//! **Item 3983** (`docs/AI.md` §415) ported both: `check_idle`'s latch is
//! `0x18`, so every unit raises the bit on its first idle frame and a
//! non-captain keeps it (the decoys' members were that), and
//! `take_damage` raises it on the captain of a unit that survives a blow,
//! attrition included. 373 tests and 2,398 units fell to 20 and 77, every
//! one named: 59 are the human's units on a quit's closing block, idle in
//! theirs because `Leader::defeat` cleared their orders (parked 3841c,
//! 3983a); 18 ride a field that already parts on the same unit — 14 an
//! attrition that starts earlier in ours, three an order, one a blow
//! theirs lands and ours does not (3983b).
//!
//! [`widen_block`](crate::diff::harness::widen_block) notes each parting
//! unit once, at its first parting block, on the test's own thread;
//! [`Pins`](crate::diff::testkit::pins::Pins) clears the tally when it is
//! taken and checks it against [`PINNED`] when it drops, so every test
//! that holds the guard and walks a block is held to its count. A test
//! [`PINNED`] does not name is held to 0. `RON_FIRSTS` prints the rows.
//!
//! [`Widened::rounds`]: crate::diff::harness::tests::Widened

use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static TALLY: RefCell<BTreeMap<(i64, i64), (i64, String)>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// One parting `cant_reach` row: unit `who/o` on block `n`.
pub(crate) fn note(who: i64, o: i64, n: i64, row: String) {
    TALLY.with(|t| {
        t.borrow_mut().entry((who, o)).or_insert((n, row));
    });
}

/// Forget what this thread has noted.
pub(crate) fn clear() {
    TALLY.with(|t| t.borrow_mut().clear());
}

/// This thread's test name, as libtest names the thread.
fn test_name() -> String {
    std::thread::current().name().unwrap_or("").to_string()
}

/// The count [`PINNED`] holds for `test`, 0 where it names none.
fn pinned(test: &str) -> usize {
    PINNED
        .iter()
        .find(|(name, _)| *name == test)
        .map_or(0, |(_, n)| *n)
}

/// Check this thread's tally against [`PINNED`] and clear it: `None` when
/// it holds, else the line the guard reports.
pub(crate) fn check() -> Option<String> {
    let tally = TALLY.with(|t| std::mem::take(&mut *t.borrow_mut()));
    let test = test_name();
    if std::env::var_os("RON_FIRSTS").is_some() {
        for ((who, o), (n, row)) in &tally {
            eprintln!("  cant_reach {n} {who}/{o}: {row}");
        }
    }
    let want = pinned(&test);
    if !tally.is_empty() || want != 0 {
        eprintln!("  cant_reach tally {test} {}", tally.len());
    }
    (tally.len() != want).then(|| {
        format!(
            "  crates/rondata/src/diff/cant_reach.rs: the cant_reach tally of {test}\n    got  {}\n    want {want}",
            tally.len()
        )
    })
}

/// Each test's count of units whose `cant_reach` parts (item 3962; item
/// 3983's writers took it from 2,398 units to 77, §415 of `docs/AI.md`). `tools/cant_reach_pins.py <log>
/// --write` rewrites it from a suite run's `cant_reach tally` lines.
pub(crate) const PINNED: &[(&str, usize)] = &[
    (
        "diff::aztec_alltech_pair::aztec_alltech_pair_run1147b_s_word_frame_is_widened_whole",
        2,
    ),
    (
        "diff::british_modern_pair::british_modern_pair_run1433b_s_closing_frame_is_widened_whole",
        1,
    ),
    (
        "diff::coverage_pair::run1075_s_garrison_window_is_widened_whole",
        73,
    ),
    (
        "diff::golden::chapter_thirty_eight_s_word_frame_is_widened_whole",
        1,
    ),
    (
        "diff::harness::tests::run96_s_word_frame_is_widened_whole",
        5,
    ),
    ("diff::lakota_pair::run881_s_word_frame_is_widened_whole", 1),
    ("diff::lakota_pair::run882_s_word_frame_is_widened_whole", 1),
    (
        "diff::lakota_pair::run883_s_closing_frame_is_widened_whole",
        1,
    ),
    (
        "diff::mongol_pair::run844_s_closing_frame_is_widened_whole",
        3,
    ),
    (
        "diff::sahara_coverage::run790_s_word_frame_is_widened_whole",
        4,
    ),
    (
        "diff::sahara_coverage::run817_s_word_frame_is_widened_whole",
        2,
    ),
    (
        "diff::sahara_mongol::run834_s_word_frame_is_widened_whole",
        2,
    ),
    // Item 4204 (second lane): the six melees' word windows 7 -> 0 and
    // run80's 5 -> 0 (the rows leave): the head quit's `clear_orders` stops
    // the human's citizens on the closing block (2601; run80's 24001), so
    // `cant_reach` reads theirs' 1 (ours was 0 with the order held).
    // Item 4204: run609's closing walk 2 -> 0 (the row leaves), and
    // run667's 5 -> 0: `Sim::resign` carries the defeat's `clear_orders`
    // (`docs/LEADERS.md` §10.2), so the human's orders are gone on the quit.
    // Item 4185: run667's closing walk 5 -> 0 (the row leaves): ours wins the
    // territory on 17379 and the cascade's `defeat(5)` clears the human's
    // orders, as the original's quit on that frame does (`docs/LEADERS.md`
    // §10.5). Item 4189: 0 -> 5, the row back: the quit on 17379's head is
    // replayed, its conquest victory bars the territory win and so the
    // cascade, and `Sim::resign` does not carry the defeat's `clear_orders`
    // (GOLDEN §152.15, parked 4189a). Item 4204: 5 -> 0, above.
];
