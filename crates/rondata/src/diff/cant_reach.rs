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

/// Each test's count of units whose `cant_reach` parts, ours 0 against
/// theirs 1 on every row (item 3962). `tools/cant_reach_pins.py <log>
/// --write` rewrites it from a suite run's `cant_reach tally` lines.
pub(crate) const PINNED: &[(&str, usize)] = &[];
