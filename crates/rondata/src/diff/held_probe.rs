//! **The held-leaf probe** (item 4159's eighteenth residue, parked
//! 4159r17a): one switch that makes every leader-record comparison site
//! report the cells of the leaves named, where ours parts from theirs
//! (`HELD`) and where theirs is non-zero and agrees (`SEEN`).
//!
//! Residues 10, 16 and 17 each placed this probe by hand at the sixteen
//! sites and removed it before committing. Now every site calls [`rows`]
//! or [`map`] once per record, and the probe is switched on by name:
//!
//! ```text
//! RON_HELD_PROBE=hire_who,good_deeds RON_HELD_PROBE_FILE=/tmp/probe.txt \
//!   zsh tools/check.sh test -p rondata --release -- diff::
//! ```
//!
//! A cell is a record key equal to a named leaf or starting `leaf[`. Off (no
//! `RON_HELD_PROBE`), a call reads one initialised `OnceLock` and returns.
//!
//! **The probe sees only what the record diff compares.** A leaf that
//! [`crate::diff::leader::rows`] or [`crate::diff::leader::theirs`] does not
//! carry gives no row at all, which looks like a clean leaf. Residue 18
//! measured `good_deeds`, `hire_stamp`, `hire_who`, `tribute_demanded` and
//! `tributes` with both sides carrying them, on a commit that is not kept
//! (parked 4159r18a, `docs/journal/2026-11-16-item-4159-residue-18.md`).
//!
//! Each `HELD` row is written as it is found, one line:
//! `HELD <test> <run> <site> block <n> who <w> <cell> ours <v> theirs <y>`.
//! `SEEN` rows are counted per thread and written when the thread ends (a
//! libtest test thread does): `SEEN <test> <run> <site> who <w> <cell>
//! theirs <y> rows <count> blocks <first>..<last>`, one line per value. A
//! thread that never ends (the main thread under `--test-threads 1`) writes
//! no `SEEN` lines; `HELD` lines never wait. `<test>` is the thread's name,
//! `<run>` what the site passed (empty where the test's name carries it),
//! `<site>` the caller's file and line. With no `RON_HELD_PROBE_FILE` the
//! lines go to stderr.
//!
//! `RON_HELD_PROBE_BEND=<cell>` adds 1 to ours for that one cell *inside
//! the probe only* — the comparison the site makes is untouched — so a run
//! can see the instrument turn a cell `HELD` (CLAUDE.md: a comparison must
//! be able to fail).

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// The switch, read once: the leaves named, the output, the bent cell.
struct Probe {
    leaves: Vec<String>,
    file: Option<String>,
    bend: Option<String>,
}

fn probe() -> Option<&'static Probe> {
    static PROBE: OnceLock<Option<Probe>> = OnceLock::new();
    PROBE
        .get_or_init(|| {
            let names = std::env::var("RON_HELD_PROBE").ok()?;
            let leaves: Vec<String> = names
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            (!leaves.is_empty()).then(|| Probe {
                leaves,
                file: std::env::var("RON_HELD_PROBE_FILE").ok(),
                bend: std::env::var("RON_HELD_PROBE_BEND").ok(),
            })
        })
        .as_ref()
}

/// Whether `key` is a cell of one of `leaves`: the leaf itself or `leaf[…`.
fn is_cell(leaves: &[String], key: &str) -> bool {
    leaves.iter().any(|l| {
        key.strip_prefix(l.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('['))
    })
}

/// One finding, before it is spelled.
#[derive(Debug, PartialEq, Eq)]
enum Row {
    Held {
        cell: String,
        ours: i64,
        theirs: i64,
    },
    Seen {
        cell: String,
        theirs: i64,
    },
}

/// The probe's judgement on one record: every cell of `leaves` the record
/// compares, `HELD` where ours (bent by one on `bend`) parts, `SEEN` where
/// theirs is non-zero and agrees. A cell theirs does not print is neither:
/// the site counts it unprinted.
fn judge<'a>(
    leaves: &[String],
    bend: Option<&str>,
    mine: impl Iterator<Item = (&'a str, i64)>,
    theirs: &BTreeMap<String, i64>,
) -> Vec<Row> {
    let mut out = Vec::new();
    for (k, v) in mine {
        if !is_cell(leaves, k) {
            continue;
        }
        let Some(&y) = theirs.get(k) else { continue };
        let v = if bend == Some(k) { v + 1 } else { v };
        if v != y {
            out.push(Row::Held {
                cell: k.to_string(),
                ours: v,
                theirs: y,
            });
        } else if y != 0 {
            out.push(Row::Seen {
                cell: k.to_string(),
                theirs: y,
            });
        }
    }
    out
}

/// Where the lines go, one writer at a time.
fn emit(p: &Probe, text: &str) {
    static OUT: Mutex<()> = Mutex::new(());
    let _g = OUT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match &p.file {
        Some(path) => {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap_or_else(|e| panic!("RON_HELD_PROBE_FILE {path}: {e}"));
            f.write_all(text.as_bytes()).unwrap();
        }
        None => eprint!("{text}"),
    }
}

/// `SEEN` counts by `(test, run, site, who, cell, theirs)`: rows, first and
/// last block. Written when the thread ends.
#[derive(Default)]
struct Seen(BTreeMap<SeenKey, (usize, i64, i64)>);

/// `(test, run, site, who, cell, theirs)`.
type SeenKey = (String, String, String, usize, String, i64);

impl Drop for Seen {
    fn drop(&mut self) {
        let (Some(p), false) = (probe(), self.0.is_empty()) else {
            return;
        };
        let mut text = String::new();
        for ((test, run, site, who, cell, y), (c, a, b)) in &self.0 {
            text.push_str(&format!(
                "SEEN {test} {run} {site} who {who} {cell} theirs {y} rows {c} blocks {a}..{b}\n"
            ));
        }
        emit(p, &text);
    }
}

thread_local! {
    static SEEN: std::cell::RefCell<Seen> = std::cell::RefCell::new(Seen::default());
}

fn record<'a>(
    site: &std::panic::Location<'_>,
    run: &str,
    n: i64,
    who: usize,
    mine: impl Iterator<Item = (&'a str, i64)>,
    theirs: &BTreeMap<String, i64>,
) {
    let Some(p) = probe() else { return };
    let rows = judge(&p.leaves, p.bend.as_deref(), mine, theirs);
    if rows.is_empty() {
        return;
    }
    let test = std::thread::current()
        .name()
        .unwrap_or("?")
        .rsplit("::")
        .next()
        .unwrap_or("?")
        .to_string();
    let run = if run.is_empty() { "-" } else { run };
    let file = site.file().rsplit('/').next().unwrap_or("?");
    let site = format!("{file}:{}", site.line());
    let mut held = String::new();
    for r in rows {
        match r {
            Row::Held { cell, ours, theirs } => held.push_str(&format!(
                "HELD {test} {run} {site} block {n} who {who} {cell} ours {ours} theirs {theirs}\n"
            )),
            Row::Seen { cell, theirs } => SEEN.with(|s| {
                let mut s = s.borrow_mut();
                let e =
                    s.0.entry((
                        test.clone(),
                        run.to_string(),
                        site.clone(),
                        who,
                        cell,
                        theirs,
                    ))
                    .or_insert((0, n, n));
                e.0 += 1;
                e.1 = e.1.min(n);
                e.2 = e.2.max(n);
            }),
        }
    }
    if !held.is_empty() {
        emit(p, &held);
    }
}

/// One leader record at a comparison site: `mine` as
/// [`crate::diff::leader::rows`] builds it, `theirs` as
/// [`crate::diff::leader::theirs`] parses it. `run` names the capture where
/// the site knows it, else empty.
#[track_caller]
pub(crate) fn rows(
    run: &str,
    n: i64,
    who: usize,
    mine: &[(String, i64)],
    theirs: &BTreeMap<String, i64>,
) {
    if probe().is_none() {
        return;
    }
    let site = std::panic::Location::caller();
    record(
        site,
        run,
        n,
        who,
        mine.iter().map(|(k, v)| (k.as_str(), *v)),
        theirs,
    );
}

/// [`rows`] for a site that keys ours by name.
#[track_caller]
pub(crate) fn map(
    run: &str,
    n: i64,
    who: usize,
    mine: &BTreeMap<String, i64>,
    theirs: &BTreeMap<String, i64>,
) {
    if probe().is_none() {
        return;
    }
    let site = std::panic::Location::caller();
    record(
        site,
        run,
        n,
        who,
        mine.iter().map(|(k, v)| (k.as_str(), *v)),
        theirs,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_of(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    /// **The probe can fail**: a record agreeing on every cell is `SEEN`
    /// where theirs is non-zero and nowhere `HELD`; the same record with one
    /// cell bent by the switch's own bend is `HELD` on that cell alone.
    #[test]
    fn a_bent_cell_is_held() {
        let leaves = vec!["hire_who".to_string(), "tributes".to_string()];
        let theirs = record_of(&[
            ("hire_who[0]", -1),
            ("hire_who[1]", -1),
            ("hire_whoever", 5),
            ("tributes[0]", 0),
            ("score", 219),
        ]);
        let mine = [
            ("hire_who[0]", -1),
            ("hire_who[1]", -1),
            ("hire_whoever", 6),
            ("tributes[0]", 0),
            ("tributes[1]", 3),
            ("score", 220),
        ];
        let agree = judge(&leaves, None, mine.iter().copied(), &theirs);
        assert_eq!(
            agree,
            [
                Row::Seen {
                    cell: "hire_who[0]".into(),
                    theirs: -1
                },
                Row::Seen {
                    cell: "hire_who[1]".into(),
                    theirs: -1
                },
            ],
            "a leaf's own cells only; `tributes[1]` unprinted, 0 = 0 unseen"
        );
        let bent = judge(&leaves, Some("hire_who[1]"), mine.iter().copied(), &theirs);
        assert_eq!(
            bent,
            [
                Row::Seen {
                    cell: "hire_who[0]".into(),
                    theirs: -1
                },
                Row::Held {
                    cell: "hire_who[1]".into(),
                    ours: 0,
                    theirs: -1
                },
            ]
        );
    }

    /// Off, the probe is off: no `RON_HELD_PROBE` in the suite's own
    /// environment, so every site's call returns at the switch.
    #[test]
    fn the_probe_is_off_unless_named() {
        if std::env::var_os("RON_HELD_PROBE").is_none() {
            assert!(probe().is_none());
        }
    }
}
