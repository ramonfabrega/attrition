//! **The shutdown dump**: the whole-map state `GameLog::end_game` writes on
//! the way out of a game, and what it is worth as ground truth.
//!
//! [`crate::gamelog::Log::final_state`] is the reader. This module is the
//! comparison — and the finding that makes the comparison safe.
//!
//! # The two shapes, and only one of them needs this reader
//!
//! A closing dump is labelled with the frame the game was on when the player
//! quit, and it lands in one of two places (`docs/ORACLE.md`, "The shutdown
//! dump"):
//!
//! - **A sibling** of the last `FRAME n` block, at `FRAME`'s own indent. The
//!   frame walk cannot reach it, `Log::dumps` catches it only when a
//!   `DUMP_ALL` run wraps it in a `FULL DUMP`, and on an ordinary windowed
//!   capture nothing read it at all until [`crate::gamelog::Log::final_state`]
//!   existed. **65 of this machine's 95 archives are this shape.**
//! - **A child** of that block, written before the log's indent was popped.
//!   [`crate::gamelog::Log::frame_states`] already reads it as the frame's own
//!   body, so `final_state` answers `None` and is right to — 25 archives.
//!   (Of the remaining five, four are `DUMP_ALL` captures whose trailing
//!   state is a `FULL DUMP` that [`crate::gamelog::Log::dumps`] already has,
//!   and one — run1 — wrote no closing state at all.)
//!
//! # It is frame `n`, and it tears by one tick on at most one unit
//!
//! The label is not a guess and the state is not off by a frame: on all 36
//! archives that carry *both* a closing dump and an ordinary `FRAME n` body
//! at the same `n`, the two records agree on **every unit's position and
//! flags** (`a_closing_dump_and_its_own_block_are_one_state`). That is
//! consistency rather than accuracy, though, because both are written at the
//! same instant — so the accuracy is settled across captures instead, against
//! a run that reached `n` in the ordinary way. run72 quit at 4811 and run71
//! is the same game running past it: 27 of the 28 units agree, and the
//! twenty-eighth, `0/3`, sits on run71's **4810** position
//! (`the_closing_dump_lags_one_tick_on_one_unit`).
//!
//! So a closing dump is frame `n` for almost everything and one tick behind
//! for a unit whose update had not run when the quit fired. run68 saw the
//! same thing from the other side — `0/5`, one unit, one tick — and concluded
//! the harness could score no position in a closing block. It can: the lag is
//! *decidable*, because a torn unit is on the simulation's own `n − 1`
//! position. [`compare_shutdown`] is that decision, and it is what lets a
//! closing dump be diffed like any other block.

use super::*;

use std::collections::{BTreeMap, BTreeSet};

/// One closing dump compared against the simulation, with the dump's own
/// one-tick tear separated out from the divergences that are ours.
#[derive(Debug, Default, Clone)]
pub struct ShutdownResult {
    pub frame: i64,
    /// Units of a real player the dump holds and the simulation links.
    pub compared: usize,
    /// The `(who, o)` the dump has and the simulation does not.
    pub unlinked: Vec<(i64, i64)>,
    /// Units the dump holds at the simulation's **`n − 1`** position: the
    /// closing dump's own tear, and not a divergence. Sorted.
    pub torn: Vec<(i64, i64)>,
    /// Units the dump agrees with at neither `n − 1` nor `n`, with
    /// `ours − theirs` at `n`. These are ours to explain.
    pub off: BTreeMap<(i64, i64), (i32, i32)>,
}

/// Compare a closing dump against a simulation standing at **`fin.n − 1`**,
/// ticking it the last frame here.
///
/// The caller steps to `n − 1` rather than `n` because that is what makes the
/// tear decidable: a unit the dump disagrees with at `n` but *agreed* with at
/// `n − 1` is one the quit caught before its update, not one this crate moved
/// wrongly. Everything else lands in [`ShutdownResult::off`].
pub fn compare_shutdown(built: &mut Built, fin: &Frame, players: usize) -> ShutdownResult {
    let before = compare(built, fin, players);
    let agreed_before: BTreeSet<(i64, i64)> = {
        let bad: BTreeSet<(i64, i64)> = before
            .diverged
            .iter()
            .map(|d| (d.who, d.o))
            .chain(before.unlinked_units.iter().copied())
            .collect();
        fin.units
            .iter()
            .filter(|u| (0..players as i64).contains(&u.who))
            .map(|u| (u.who, u.o))
            .filter(|k| !bad.contains(k))
            .collect()
    };
    built.tick();
    let after = compare(built, fin, players);
    let mut torn = Vec::new();
    let mut off = BTreeMap::new();
    for d in &after.diverged {
        let key = (d.who, d.o);
        if agreed_before.contains(&key) {
            torn.push(key);
        } else {
            off.insert(key, (d.ours.x - d.theirs.x, d.ours.y - d.theirs.y));
        }
    }
    torn.sort_unstable();
    ShutdownResult {
        frame: fin.n,
        compared: after.compared,
        unlinked: after.unlinked_units.clone(),
        torn,
        off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::{sibling_texts, trace};
    use crate::gamelog::Log;
    use crate::testenv::{dump, install};

    /// **The census: which of this machine's archives carry a closing dump,
    /// and what it holds.**
    ///
    /// `Ok((frame, units, builds, leaders, cities, paired))` is what
    /// [`crate::gamelog::Log::final_state`] answers; `paired` says the file
    /// also carries an ordinary `FRAME n` body at the same `n`, so the two
    /// records can be checked against each other
    /// (`a_closing_dump_and_its_own_block_are_one_state`). `Err` is a
    /// `None`, with the reason: `fulldump` — a `DUMP_ALL` capture whose
    /// trailing state [`crate::gamelog::Log::dumps`] already reads;
    /// `nested` — the closing dump is the last `FRAME`'s own child and
    /// [`crate::gamelog::Log::frame_states`] already reads it; `none` — the
    /// run wrote no closing state at all.
    ///
    /// A capture that lands later is not in this table and does not fail it;
    /// the test names it instead, and the row is added when the row is
    /// looked at. What it *cannot* do is go quietly green — the two counts
    /// below are asserted against the table itself, on every machine.
    /// What [`crate::gamelog::Log::final_state`] answers for one archive:
    /// `Ok((frame, units, builds, leaders, cities, paired))`, or the reason
    /// it answered `None`.
    type Closing = Result<(i64, usize, usize, usize, usize, bool), &'static str>;

    /// One archive's expected closing dump, and the run's own name.
    type Row = (&'static str, Closing);

    /// One capture in a map's closing-dump table: name, the frame, the
    /// units compared, and the dump's own torn units.
    type Want = (&'static str, i64, usize, &'static [(i64, i64)]);

    const CENSUS: &[Row] = &[
        ("fuzz-424242-early", Err("nested")),
        ("fuzz-424242-heights", Ok((5, 136, 15, 4, 0, false))),
        ("run1-fulldump", Err("none")),
        ("run10-world6-long", Ok((1772, 57, 16, 4, 3, true))),
        ("run11-checksum", Ok((144, 53, 14, 4, 2, true))),
        ("run12-dumpall-seeds", Err("fulldump")),
        ("run13-window-95-105", Ok((104, 53, 14, 4, 0, true))),
        ("run14-trace", Ok((285, 54, 14, 4, 2, true))),
        ("run16-attrition", Ok((6872, 77, 20, 4, 3, true))),
        ("run16b-cmd", Ok((2401, 60, 16, 4, 3, true))),
        ("run17-combat", Ok((2601, 71, 15, 4, 2, true))),
        ("run18a-startonly", Ok((24001, 127, 32, 4, 3, false))),
        ("run18b-window-6374-6590", Err("nested")),
        ("run19-window-8174-8192", Err("nested")),
        ("run2-units", Ok((1730, 57, 15, 4, 2, true))),
        ("run20-islands-dumpall", Ok((5, 116, 14, 4, 0, false))),
        ("run21-islands-long", Ok((24001, 193, 40, 4, 3, false))),
        (
            "run22-islands-dock-window",
            Ok((3584, 124, 19, 4, 0, false)),
        ),
        ("run23-islands-war", Ok((24001, 193, 40, 4, 3, false))),
        ("run24-islands-raid", Ok((16489, 181, 27, 4, 4, false))),
        (
            "run25-islands-emergency-window",
            Ok((16007, 178, 28, 4, 0, false)),
        ),
        (
            "run26-islands-findtarget-window",
            Ok((16007, 178, 28, 4, 0, false)),
        ),
        (
            "run27-islands-defending-window",
            Ok((16007, 178, 28, 4, 0, false)),
        ),
        (
            "run28-islands-engagement",
            Ok((15401, 175, 28, 4, 3, false)),
        ),
        (
            "run29-islands-engagement-window",
            Ok((15105, 185, 27, 4, 0, false)),
        ),
        ("run3-fulldump-types", Err("fulldump")),
        ("run30-humangroup-nogroups", Err("nested")),
        ("run31-humangroup", Err("nested")),
        ("run32-roadpath", Ok((111, 53, 16, 4, 0, false))),
        ("run33-longtrace", Ok((1851, 57, 16, 4, 3, true))),
        (
            "run34-greatlakes-dumpall-start",
            Ok((6, 52, 14, 4, 0, false)),
        ),
        ("run36-islands-start", Ok((6, 52, 14, 4, 0, false))),
        ("run38-islands-start", Ok((6, 116, 14, 4, 0, false))),
        ("run39-islands-longtrace", Ok((1851, 121, 16, 4, 3, true))),
        ("run4-gunpowder-nubian-leaders9", Err("nested")),
        ("run40-census", Err("nested")),
        ("run41-census", Err("nested")),
        ("run42-islands-goodybucket", Ok((901, 120, 14, 4, 2, true))),
        ("run43-roadterraform", Ok((111, 53, 16, 4, 0, false))),
        ("run44-islands-turners", Ok((701, 128, 14, 4, 2, true))),
        ("run45-islands-groups", Ok((901, 120, 14, 4, 2, true))),
        (
            "run46-greatlakes-fourthclick",
            Ok((901, 79, 15, 4, 2, true)),
        ),
        (
            "run47-greatlakes-selectprobe",
            Ok((301, 72, 14, 4, 2, true)),
        ),
        ("run48-greatlakes-formprobe", Ok((301, 72, 14, 4, 2, true))),
        ("run49-greatlakes-cardprobe", Ok((321, 73, 14, 4, 2, true))),
        ("run5-gunpowder-nubian-fulldump", Err("fulldump")),
        ("run50-greatlakes-echelon", Ok((901, 79, 15, 4, 2, true))),
        ("run51-greatlakes-tooltip", Ok((251, 63, 14, 4, 2, true))),
        ("run52-greatlakes-keypress", Ok((261, 63, 14, 4, 2, true))),
        (
            "run53-greatlakes-24k-trace",
            Ok((24001, 127, 32, 4, 3, false)),
        ),
        ("run54-islands-24k-trace", Ok((24001, 193, 40, 4, 3, false))),
        ("run55-islands-calccost", Ok((1501, 120, 15, 4, 2, true))),
        ("run56-islands-3k", Ok((3001, 122, 18, 4, 3, true))),
        ("run57-islands-4k", Ok((4001, 125, 20, 4, 3, true))),
        ("run58-islands-5k2", Ok((5201, 127, 20, 4, 3, true))),
        ("run59-islands-census-5150", Err("nested")),
        ("run6-ancient-nubian-builds7", Ok((432, 55, 14, 4, 2, true))),
        ("run6-copy", Ok((432, 55, 14, 4, 2, true))),
        (
            "run60-islands-census-thin",
            Ok((5401, 129, 21, 4, 3, false)),
        ),
        ("run61-islands-air", Ok((5401, 129, 21, 4, 3, false))),
        ("run62-roadpath", Ok((111, 53, 16, 4, 0, false))),
        ("run63-islands-scoutwalk", Err("nested")),
        (
            "run64-islands-caravanroad",
            Ok((6181, 129, 21, 4, 0, false)),
        ),
        (
            "run65-islands-caravanturn",
            Ok((6221, 129, 21, 4, 0, false)),
        ),
        ("run66-islands-merchantwalk", Err("nested")),
        ("run67-islands-crewclocks", Err("nested")),
        ("run68-islands-citizenword", Err("nested")),
        ("run69-greatlakes-3k", Ok((3001, 60, 19, 4, 3, true))),
        ("run7-ancient-nubian-orders", Ok((1732, 60, 17, 4, 3, true))),
        ("run70-greatlakes-astarwin", Ok((2001, 57, 17, 4, 3, false))),
        ("run71-greatlakes-5k", Ok((5001, 69, 23, 4, 3, true))),
        (
            "run72-greatlakes-marketroad",
            Ok((4811, 68, 23, 4, 0, false)),
        ),
        (
            "run73-greatlakes-caravanstart",
            Ok((5591, 70, 23, 4, 0, false)),
        ),
        ("run74-greatlakes-merchantturn", Err("nested")),
        ("run75-greatlakes-scoutwalk", Err("nested")),
        ("run76-greatlakes-archermarch", Err("nested")),
        ("run77-eastindies-squadbirth", Err("nested")),
        ("run78-eastindies-threeborn", Err("nested")),
        ("run79-greatlakes-secondsquad", Err("nested")),
        ("run8-personality", Ok((60, 52, 14, 4, 2, true))),
        ("run80-greatlakes-latecensus", Err("nested")),
        (
            "run81-islands-merchantunpack",
            Ok((6816, 131, 21, 4, 3, false)),
        ),
        (
            "run82-islands-merchantcast",
            Ok((6946, 132, 21, 4, 3, false)),
        ),
        ("run83-greatlakes-wordwindow", Err("nested")),
        ("run84-greatlakes-makelist", Err("nested")),
        ("run85-eastindies-blockedstand", Err("nested")),
        ("run86-eastindies-transportride", Err("nested")),
        ("run9-world6", Ok((36, 52, 14, 4, 2, true))),
        ("run900-control", Ok((61, 52, 14, 4, 2, true))),
        ("run901-controlB", Ok((61, 52, 14, 4, 2, true))),
        ("run902-controlC", Err("fulldump")),
        ("run903-wineproof", Ok((401, 55, 14, 4, 2, true))),
        ("run904-repathproof", Ok((401, 55, 14, 4, 2, true))),
        ("run905-coverfixproof", Ok((401, 55, 14, 4, 2, true))),
        ("scratch-2026-08-20-2219", Err("nested")),
    ];

    #[test]
    fn the_census_is_the_corpus() {
        // Static first, so a machine with no archives still checks the
        // ledger's own arithmetic rather than skipping into green.
        assert_eq!(CENSUS.len(), 95, "the archives this census was taken over");
        let readable = CENSUS.iter().filter(|r| r.1.is_ok()).count();
        let paired = CENSUS
            .iter()
            .filter(|r| matches!(r.1, Ok((_, _, _, _, _, true))))
            .count();
        let mut why: BTreeMap<&str, usize> = BTreeMap::new();
        for r in CENSUS {
            if let Err(w) = r.1 {
                *why.entry(w).or_default() += 1;
            }
        }
        assert_eq!(
            (readable, paired, why),
            (
                65,
                36,
                [("fulldump", 4), ("nested", 25), ("none", 1)]
                    .into_iter()
                    .collect()
            ),
            "the census: 65 archives carry a closing dump `final_state` reads, \
             36 of them beside an ordinary block at the same frame"
        );

        let Some(one) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping the disk half: no archives (set RON_GAMELOG_DIR)");
            return;
        };
        let dir = std::path::Path::new(&one).parent().unwrap().to_owned();
        let mut seen = 0usize;
        for (name, want) in CENSUS {
            let path = dir.join(format!("gamelog-{name}.txt"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            seen += 1;
            let log = Log::parse(&text);
            let states = log.frame_states();
            let got: Result<(i64, usize, usize, usize, usize, bool), &str> = match log.final_state()
            {
                Some(f) => Ok((
                    f.n,
                    f.units.len(),
                    f.builds.len(),
                    f.leaders.len(),
                    f.cities.len(),
                    states.iter().any(|s| s.n == f.n && !s.units.is_empty()),
                )),
                None => {
                    let last = log.frames().last().map(|f| f.0);
                    Err(if log.dumps().iter().any(|(n, _)| Some(*n) == last) {
                        "fulldump"
                    } else if states
                        .iter()
                        .any(|s| Some(s.n) == last && !s.units.is_empty())
                    {
                        "nested"
                    } else {
                        "none"
                    })
                }
            };
            assert_eq!(&got, want, "gamelog-{name}.txt's closing dump");
        }
        eprintln!(
            "the census: {seen} of {} archives on this disk",
            CENSUS.len()
        );

        // A capture that landed since is named, not failed on.
        let known: BTreeSet<String> = CENSUS
            .iter()
            .map(|r| format!("gamelog-{}.txt", r.0))
            .collect();
        let mut fresh: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with("gamelog-") && n.ends_with(".txt"))
            .filter(|n| !known.contains(n))
            .collect();
        fresh.sort();
        if !fresh.is_empty() {
            eprintln!("archives not in the census yet: {fresh:?}");
        }
    }

    /// **A closing dump and the ordinary block at its own frame are one
    /// state**, on every archive that carries both — every unit's position
    /// and every unit's flags.
    ///
    /// This is what says the label is not a guess. It is *consistency*
    /// rather than accuracy, because both records are written at the same
    /// instant on the way out; the accuracy is
    /// `the_closing_dump_lags_one_tick_on_one_unit`, across captures.
    #[test]
    fn a_closing_dump_and_its_own_block_are_one_state() {
        let Some(one) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: no archives (set RON_GAMELOG_DIR)");
            return;
        };
        let dir = std::path::Path::new(&one).parent().unwrap().to_owned();
        let mut pairs = 0usize;
        let mut moving = 0usize;
        let mut bad: Vec<String> = Vec::new();
        for (name, want) in CENSUS {
            if !matches!(want, Ok((_, _, _, _, _, true))) {
                continue;
            }
            let path = dir.join(format!("gamelog-{name}.txt"));
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let log = Log::parse(&text);
            let fin = log.final_state().expect("the census says it has one");
            let states = log.frame_states();
            let st = states
                .iter()
                .find(|s| s.n == fin.n && !s.units.is_empty())
                .expect("the census says it is paired");
            pairs += 1;
            // How many units the frame moved, so a run of standing armies
            // cannot pass this by holding still.
            if let Some(prev) = states
                .iter()
                .rev()
                .find(|s| s.n < fin.n && !s.units.is_empty())
            {
                moving += st
                    .units
                    .iter()
                    .filter(|u| {
                        prev.units
                            .iter()
                            .find(|v| v.who == u.who && v.o == u.o)
                            .is_some_and(|v| v.pos != u.pos)
                    })
                    .count();
            }
            for u in &fin.units {
                match st.units.iter().find(|v| v.who == u.who && v.o == u.o) {
                    Some(v) => {
                        if v.pos != u.pos {
                            bad.push(format!(
                                "{name} {}/{} pos {:?} vs {:?}",
                                u.who, u.o, u.pos, v.pos
                            ));
                        }
                        if v.flags != u.flags {
                            bad.push(format!(
                                "{name} {}/{} flags {} vs {}",
                                u.who, u.o, u.flags, v.flags
                            ));
                        }
                    }
                    None => bad.push(format!(
                        "{name} {}/{} absent from its own block",
                        u.who, u.o
                    )),
                }
            }
        }
        eprintln!("{pairs} closing/ordinary pairs, {moving} unit-moves inside them");
        assert!(
            bad.is_empty(),
            "a closing dump parted from its own block: {bad:?}"
        );
        assert_eq!(pairs, 36, "the census's paired archives, all of them");
        assert!(
            moving >= 100,
            "the pairs must contain moving units: {moving}"
        );
    }

    /// **The closing dump lags one tick, on one unit** — measured across
    /// captures, which is the only way it can be measured at all.
    ///
    /// run72 quit at 4811 with its `DUMP_ALL` window long closed, so its
    /// closing dump is the only record of that frame it has. run71 is the
    /// same Great Lakes game run past it, dumping every frame. 27 of the 28
    /// units agree; `0/3` sits on run71's **4810** position and on run71's
    /// 4811 it has already stepped (−18, +18) away.
    ///
    /// This is run68's `0/5` seen from the other side, and it is why
    /// [`compare_shutdown`] exists rather than an exception list.
    #[test]
    fn the_closing_dump_lags_one_tick_on_one_unit() {
        let (Some(a), Some(b)) = (
            dump("gamelog-run72-greatlakes-marketroad.txt"),
            dump("gamelog-run71-greatlakes-5k.txt"),
        ) else {
            eprintln!("skipping: no run71/run72 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let at = std::fs::read_to_string(&a).unwrap();
        let bt = std::fs::read_to_string(&b).unwrap();
        let closing = Log::parse(&at).final_state().expect("run72 quit");
        let long = Log::parse(&bt);
        let ordinary = long.frame_states();
        assert_eq!(closing.n, 4_811, "run72's closing dump");
        let differ = |n: i64| -> Vec<(i64, i64)> {
            let st = ordinary
                .iter()
                .find(|s| s.n == n && !s.units.is_empty())
                .unwrap_or_else(|| panic!("run71 has no ordinary block at {n}"));
            closing
                .units
                .iter()
                .filter(|u| {
                    st.units
                        .iter()
                        .find(|v| v.who == u.who && v.o == u.o)
                        .is_some_and(|v| v.pos != u.pos)
                })
                .map(|u| (u.who, u.o))
                .collect()
        };
        assert_eq!(
            differ(4_811),
            vec![(0, 3)],
            "run72's closing 4811 against run71's ordinary 4811: one unit, and it is `0/3`"
        );
        assert!(
            !differ(4_810).contains(&(0, 3)),
            "and `0/3` is exactly where run71 had it a frame earlier"
        );
    }

    /// **Great Lakes' closing dumps, diffed whole** — the scored map's five
    /// readable ones, each a whole-map state stepped to from the game's own
    /// start block.
    ///
    /// Two of them are frames no capture on this disk had ever printed:
    /// run72's **4811** is six frames past its window's last block and
    /// run73's **5591** is twelve past its own. Both reproduce, unit for
    /// unit — run72's `0/3` as the dump's tear, run73's thirty units
    /// outright. The other three sit at a frame their capture also dumps
    /// ordinarily, so they are a second reading of a block already scored,
    /// and they are here because a table that skipped them would not notice
    /// the reader breaking on them.
    ///
    /// **No word moves on this.** Great Lakes' long word is 7176 and the
    /// furthest of these is 5591, so what the sweep bought is coverage
    /// under the word rather than past it — which is worth having precisely
    /// because a draw stream that matches is not a position that matches
    /// (item 250 bought 182 frames of matching draws on a destination two
    /// tiles wrong).
    #[test]
    fn great_lakes_closing_dumps_are_the_original_s() {
        let Some(inst) = install() else { return };
        let want: &[Want] = &[
            ("run70-greatlakes-astarwin", 2_001, 17, &[]),
            ("run69-greatlakes-3k", 3_001, 20, &[]),
            ("run72-greatlakes-marketroad", 4_811, 28, &[(0, 3)]),
            ("run71-greatlakes-5k", 5_001, 29, &[]),
            ("run73-greatlakes-caravanstart", 5_591, 30, &[]),
        ];
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut ran = 0usize;
        for (name, frame, compared, torn) in want {
            let run = name.split('-').next().unwrap();
            let (Some(path), Some(tr)) = (
                dump(&format!("gamelog-{name}.txt")),
                trace(&format!("rontrace-{run}.log")),
            ) else {
                eprintln!("skipping {name}: set RON_GAMELOG_DIR");
                continue;
            };
            ran += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let fin = log.final_state().expect("the census says it has one");
            assert_eq!(fin.n, *frame, "{name}'s closing dump");
            let mut init = log.initial().expect("a capture carries its start block");
            borrow_from_siblings(&mut init, &refs);
            borrow_pasture(&mut init, &tr);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            for _ in 0..fin.n - 1 {
                built.tick();
            }
            let r = compare_shutdown(&mut built, &fin, 8);
            eprintln!(
                "{name} {}: {} compared, {} unlinked, torn {:?}, off {:?}",
                r.frame,
                r.compared,
                r.unlinked.len(),
                r.torn,
                r.off
            );
            assert_eq!(
                (r.compared, r.unlinked.len(), r.torn.as_slice(), r.off.len()),
                (*compared, 0, *torn, 0),
                "{name}'s closing dump, every unit of it"
            );
        }
        assert!(
            ran == 5 || ran == 0,
            "{ran} of Great Lakes' five closing dumps ran"
        );
    }

    /// **East Indies' closing dumps, diffed whole** — nine of them, from
    /// 1501 to 6816, every unit of every record.
    ///
    /// The furthest, run81's **6816**, is seventeen frames past that
    /// capture's last block and reproduces outright: 27 units, none
    /// unlinked, none off, and not even a tear. run82's **6946** is the
    /// tenth and lives in its own test
    /// (`run82_s_window_is_the_east_indies_ride_s_run_up`) because the
    /// window under it is what item 241 bought.
    ///
    /// **No word moves on this either.** East Indies' long word is 7529 and
    /// nothing on this disk closes above it — run77, run78, run85 and run86
    /// all quit *into* their last frame, so their state is that block's own
    /// child and [`crate::gamelog::Log::frame_states`] has had it all along.
    ///
    /// The four rows that read `(0, N)` torn are the dump's one-tick lag on
    /// one unit, and they are the argument for [`compare_shutdown`] over an
    /// exception list: it is a different unit every time — `0/3`, `0/4`,
    /// `0/5` — and it is whichever one the quit caught mid-update.
    #[test]
    fn east_indies_closing_dumps_are_the_original_s() {
        let Some(inst) = install() else { return };
        // name, frame, units compared, the dump's own torn units.
        let want: &[Want] = &[
            ("run55-islands-calccost", 1_501, 16, &[]),
            ("run56-islands-3k", 3_001, 18, &[(0, 5)]),
            ("run22-islands-dock-window", 3_584, 20, &[]),
            ("run57-islands-4k", 4_001, 21, &[(0, 5)]),
            ("run58-islands-5k2", 5_201, 23, &[]),
            ("run60-islands-census-thin", 5_401, 25, &[(0, 3)]),
            ("run64-islands-caravanroad", 6_181, 25, &[(0, 4)]),
            ("run65-islands-caravanturn", 6_221, 25, &[]),
            ("run81-islands-merchantunpack", 6_816, 27, &[]),
        ];
        let (Some(sib), Some(_)) = (
            dump("gamelog-run38-islands-start.txt"),
            dump("gamelog-run81-islands-merchantunpack.txt"),
        ) else {
            eprintln!("skipping: no East Indies captures (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut ran = 0usize;
        for (name, frame, compared, torn) in want {
            let run = name.split('-').next().unwrap();
            let (Some(path), Some(tr)) = (
                dump(&format!("gamelog-{name}.txt")),
                trace(&format!("rontrace-{run}.log")),
            ) else {
                eprintln!("skipping {name}: set RON_GAMELOG_DIR");
                continue;
            };
            ran += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let fin = log.final_state().expect("the census says it has one");
            assert_eq!(fin.n, *frame, "{name}'s closing dump");
            let mut init = log.initial().expect("a capture carries its start block");
            borrow_from_siblings(&mut init, &refs);
            borrow_pasture(&mut init, &tr);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            for _ in 0..fin.n - 1 {
                built.tick();
            }
            let r = compare_shutdown(&mut built, &fin, 8);
            eprintln!(
                "{name} {}: {} compared, {} unlinked, torn {:?}, off {:?}",
                r.frame,
                r.compared,
                r.unlinked.len(),
                r.torn,
                r.off
            );
            assert_eq!(
                (r.compared, r.unlinked.len(), r.torn.as_slice(), r.off.len()),
                (*compared, 0, *torn, 0),
                "{name}'s closing dump, every unit of it"
            );
        }
        assert!(
            ran == want.len() || ran == 0,
            "{ran} of East Indies' {} closing dumps ran",
            want.len()
        );
    }
}
