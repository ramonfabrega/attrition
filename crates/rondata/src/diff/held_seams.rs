//! **The seams the held-out games enter** (item 4580, DECISIONS 153 §10):
//! each battery's walks played again with the sim's `seam!` hooks counting
//! (`sim::seam`), and for each game every hook entered with its first state,
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
//! Each battery writes its table to
//! `~/ron-data/lab-experiments/2026-11-23-item-4580/seams-<battery>.txt`
//! (outside git) and prints it.

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
    entered: Vec<(&'static str, sim::seam::Entered)>,
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
        sim::seam::start();
        let score = crate::checksum::playback::compare_until(inst.root(), &rcx, &take, until);
        let entered = sim::seam::take();
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
    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let dir = home.join("ron-data/lab-experiments/2026-11-23-item-4580");
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

/// **The comparison that can fail**: item 4574 gave the sixth's last 102
/// keys their rows, and item 4559 the first five's 53, so no held-out walk
/// enters `launch_point`'s fallback. Red on the tree before 4574's rows
/// (`git show f8fd03379~1:crates/sim/src/data/launch.rs`): the sixth's
/// rows 73, 93, 113 and 133 enter it.
fn no_fallback(label: &str, walks: &[Walk]) {
    let rows = fallback_rows(walks);
    assert!(
        rows.is_empty(),
        "{label}: rows enter `launch_point`'s fallback (row, rounds): {rows:?}"
    );
}

#[test]
fn the_first_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FIRST, "first") {
        no_fallback(FIRST.label, &w);
    }
}

#[test]
fn the_second_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&SECOND, "second") {
        no_fallback(SECOND.label, &w);
    }
}

#[test]
fn the_third_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&THIRD, "third") {
        no_fallback(THIRD.label, &w);
    }
}

#[test]
fn the_fourth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FOURTH, "fourth") {
        no_fallback(FOURTH.label, &w);
    }
}

#[test]
fn the_fifth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&FIFTH, "fifth") {
        no_fallback(FIFTH.label, &w);
    }
}

#[test]
fn the_sixth_held_out_walks_list_the_seams_they_enter() {
    if let Some(w) = count_battery(&SIXTH, "sixth") {
        no_fallback(SIXTH.label, &w);
    }
}
