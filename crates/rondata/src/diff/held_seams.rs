//! **The seams the held-out games enter** (item 4580, DECISIONS 153 §10):
//! each battery's walks played again with the sim's `seam!` hooks counting
//! (`sim::core::seam`), and for each game every hook entered with its first state,
//! beside the game's first parting. A hook entered on or before the first
//! parting is a candidate cause; one entered after it can move only later
//! columns (item 4574's 3,673 fallback rounds, every one after its row's
//! first parting).
//!
//! Compiled only under `rondata`'s `seam-count` feature:
//!
//! ```text
//! zsh tools/check.sh test --release -p rondata --features seam-count -- held_seams
//! ```
//!
//! Each battery writes its table to `$RON_SEAMS_DIR/seams-<battery>.txt`
//! (outside git) and prints it; with the variable unset the directory is
//! item 4580's, `~/ron-data/lab-experiments/2026-11-23-item-4580`. **A pass
//! sets it to its own directory** (`docs/LOOP.md`, the pass's tenth act):
//! the pass of 2026-11-23 ran this unset and wrote the tip's tables over the
//! item's.
//!
//! **No gate builds the feature, so nothing here is held between passes.**
//! Eighteen landings after it landed the fallback's pin read 21 where the
//! tip fired 16, and the same run named row 133's first parting:
//! `sim::orders::jobs::computer_repair_explore` entered once, on state 1906,
//! where the units line parts on `order.type` 3 against 1.

use crate::diff::held_out::tests::paths;
use crate::diff::held_out::{Battery, FIFTH, FIRST, FOURTH, SECOND, SIXTH, THIRD};

/// `launch_point`'s fallback, the one seam counted at run time before this
/// item (items 4559, 4574): every key the held-out walks fired from the
/// figure's square has its row, so no walk enters it.
const LAUNCH_FALLBACK: &str = "sim::data::launch::fallback";

/// One game's walk: its first parting (the earliest state any line parts
/// on, with the lines parting there) and every hook it entered.
struct Walk {
    row: usize,
    run: usize,
    states: u32,
    first: Option<(i32, Vec<&'static str>)>,
    entered: Vec<(&'static str, sim::core::seam::Entered)>,
}

fn walk_battery(b: &Battery) -> Option<Vec<Walk>> {
    let inst = crate::testenv::install()?;
    let mut out = Vec::new();
    for (k, held) in b.table.iter().enumerate() {
        let (Some(rcx), Some(take)) = paths(b, k) else {
            eprintln!(
                "skipping: row {} has no file or no run{} take",
                held.row,
                b.run + k
            );
            continue;
        };
        let until = b.scores[k].as_ref().and_then(|p| p.last);
        sim::core::seam::start();
        let score = crate::checksum::playback::compare_until(inst.root(), &rcx, &take, until);
        let entered = sim::core::seam::take();
        let score = score.unwrap_or_else(|e| panic!("row {} run{}: {e}", held.row, b.run + k));
        let earliest = score.first.iter().flatten().map(|f| f.0).min();
        let first = earliest.map(|s| {
            let lines = crate::checksum::CHECKSUM_LINES
                .iter()
                .zip(&score.first)
                .filter(|(_, f)| f.is_some_and(|f| f.0 == s))
                .map(|(l, _)| l.0)
                .collect();
            (s, lines)
        });
        out.push(Walk {
            row: held.row,
            run: b.run + k,
            states: score.states,
            first,
            entered,
        });
    }
    Some(out)
}

/// The table: per game, its first parting, then each hook by first state,
/// marked `<` when entered before the first parting, `=` on it, `>` after.
fn table(label: &str, walks: &[Walk]) -> String {
    let mut s = format!("# {label}: the seams each held-out walk entered (item 4580)\n");
    for w in walks {
        let parting = match &w.first {
            Some((st, lines)) => format!("first parting state {st} ({})", lines.join(", ")),
            None => "whole: no parting".to_string(),
        };
        let before = w
            .entered
            .iter()
            .filter(|(_, e)| w.first.as_ref().is_none_or(|(st, _)| e.first <= *st))
            .count();
        s += &format!(
            "\nrow {} run{}: {} states, {parting}; {} hooks entered, {before} on or before it\n",
            w.row,
            w.run,
            w.states,
            w.entered.len()
        );
        for (name, e) in &w.entered {
            let mark = match &w.first {
                Some((st, _)) if e.first < *st => '<',
                Some((st, _)) if e.first == *st => '=',
                Some(_) => '>',
                None => '<',
            };
            s += &format!("  {mark} {:>6} {:>10}  {name}\n", e.first, e.count);
        }
    }
    s
}

fn count_battery(b: &Battery, file: &str) -> Option<Vec<Walk>> {
    let walks = walk_battery(b)?;
    let text = table(b.label, &walks);
    eprintln!("{text}");
    let dir = std::env::var_os("RON_SEAMS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/lab-experiments/2026-11-23-item-4580")
        });
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = std::fs::write(dir.join(format!("seams-{file}.txt")), &text);
    }
    Some(walks)
}

/// The fallback rounds a battery's walks fired, per row that fired any.
fn fallback_rows(walks: &[Walk]) -> Vec<(usize, u64)> {
    walks
        .iter()
        .filter_map(|w| {
            w.entered
                .iter()
                .find(|(n, _)| *n == LAUNCH_FALLBACK)
                .map(|(_, e)| (w.row, e.count))
        })
        .collect()
}

/// **The comparison that can fail**: `launch_point`'s fallback rounds per
/// row, pinned. Item 4574 gave the sixth's 102 keys their rows and item
/// 4559 the first five's 53. The pass's probe on the same arm
/// (`~/ron-data/lab-experiments/2026-11-22-pass-third/fallback-by-row.txt`)
/// listed 3,673 rounds on the sixth before 4574: rows 33 (27), 73 (296),
/// 93 (3,165), 113 (53) and 133 (132). On the tree with 4574's rows
/// (`data/launch.rs` and `data/pivot.rs` as `93932dfd0` held them, the
/// hook laid on that `launch_point`) this counter reads the same five.
/// After them, one row still enters it: row 133, 21 rounds from state
/// 4370, past its first parting (956): keys no walk fired before 4574's
/// rows moved its later states (parked 4580a). **16 on `de60a62ee`** (the
/// pass of 2026-11-23): items 4585 and 4594 moved row 133's game past its
/// first parting, and the keys it fires there with it.
const FALLBACK: [(&str, &[(usize, u64)]); 6] = [
    ("first", &[]),
    ("second", &[]),
    ("third", &[]),
    ("fourth", &[]),
    ("fifth", &[]),
    ("sixth", &[(133, 16)]),
];

fn fallback_as_pinned(file: &str, walks: &[Walk]) {
    let want = FALLBACK.iter().find(|(f, _)| *f == file).map(|(_, w)| *w);
    let got = fallback_rows(walks);
    assert_eq!(
        Some(got.as_slice()),
        want,
        "{file}: the rows that enter `launch_point`'s fallback (row, rounds)"
    );
}

#[test]
fn the_first_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FIRST, "first") {
        fallback_as_pinned("first", &w);
    }
}

#[test]
fn the_second_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&SECOND, "second") {
        fallback_as_pinned("second", &w);
    }
}

#[test]
fn the_third_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&THIRD, "third") {
        fallback_as_pinned("third", &w);
    }
}

#[test]
fn the_fourth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FOURTH, "fourth") {
        fallback_as_pinned("fourth", &w);
    }
}

#[test]
fn the_fifth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FIFTH, "fifth") {
        fallback_as_pinned("fifth", &w);
    }
}

#[test]
fn the_sixth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&SIXTH, "sixth") {
        fallback_as_pinned("sixth", &w);
    }
}
