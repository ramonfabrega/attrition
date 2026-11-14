//! **The correction audit, pinned at zero on whole pairs** (Loop 3728).
//!
//! The harness keeps the replay on the original's draws with three
//! corrections, each installed where a capture holds the value
//! (`setup::Built::tick`): the sync word at a `FULL DUMP` frame end, each
//! linked figure's animation clock on that frame, and gaia's animals put
//! back where the dump has them. On run69's 3,001 frames the audit
//! (`corrections::CorrectionAudit`) found that none of them changed the
//! state the simulation had already reached — RNG installs 0 of 14, clocks
//! 0 of 748, and seven re-seats that moved one animal — so that game ran on
//! its own. This module holds the same claim on every whole pair: the
//! pair's long capture is replayed from its start sibling and its trace's
//! pasture with the audit on, to the capture's last record, and **an
//! installed word or clock that differs from the one the simulation already
//! held fails the test**. The re-seats that moved an animal are pinned.
//!
//! `attempted` is printed beside `changed` for every counter: the
//! corrections run only on the frames a sibling's `DUMP_ALL` carries, so a
//! counter that attempted nothing asserts nothing, and the pin says so
//! ([`Expect::attempted`]).

use super::testkit::*;
use super::*;
use crate::capture::indexed::IndexedCapture;
use crate::testenv::{dump, install};

/// One audited replay: the audit, the records pushed and where the
/// simulation stood when the capture ran out.
struct Audited {
    /// Held so its ground guard (`Built`'s drop) fires after the report.
    _session: ReplaySession,
    audit: CorrectionAudit,
    records: usize,
    last_record: i64,
    /// Records with any unit position parted, and the first such frame —
    /// the replay's own lockstep, printed beside the audit.
    parted: (usize, Option<i64>),
    /// The last record's units compared and parted.
    closing: (usize, usize),
    sim_frame: i64,
}

/// What a pair's audit is pinned to: each counter's `attempted` for
/// (seed, clock, gaia reseat) — so a counter that went blind is a moved pin
/// — and the re-seats that changed an animal, their count and the first
/// few named `f<frame> <who>/<o>`.
struct Expect {
    attempted: (u64, u64, u64),
    reseats_changed: u64,
    first_reseats: &'static [&'static str],
}

/// The re-seats [`Expect::first_reseats`] names.
const NAMED: usize = 4;

/// Replay `gamelog` from `siblings`' heads and `tracelog`'s pasture with the
/// audit on, every record the capture holds. `None` when the captures are
/// not on this machine.
fn audit_walk(siblings: &[&str], (gamelog, tracelog): (&str, &str)) -> Option<Audited> {
    let inst = install()?;
    let (Some(path), Some(tr)) = (dump(gamelog), trace(tracelog)) else {
        eprintln!("skipping: no {gamelog}/{tracelog} (set RON_GAMELOG_DIR)");
        return None;
    };
    if siblings.iter().any(|s| dump(s).is_none()) {
        eprintln!("skipping: a sibling of {gamelog} is missing: {siblings:?}");
        return None;
    }
    let loaded = crate::load::load(&inst).unwrap();
    let mut source = IndexedCapture::open(&path).unwrap();
    let mut session = with_named_sibling_initials(siblings, |refs| {
        source
            .with_replay_initial(|init| {
                let mut init = init;
                borrow_from_siblings(&mut init, refs);
                borrow_pasture(&mut init, &tr);
                ReplaySession::new(&loaded, &init, Tuning::RON, None)
            })
            .unwrap()
    });
    session.enable_correction_audit();
    let records = source.frames().len();
    let (mut last_record, mut parted, mut closing) = (0, (0, None), (0, 0));
    for i in 0..records {
        let frame = source.frame_state(i).unwrap();
        last_record = frame.n;
        let r = session.push(i, &frame).unwrap();
        if !r.diverged.is_empty() {
            parted.0 += 1;
            parted.1.get_or_insert(r.frame);
        }
        closing = (r.compared, r.diverged.len());
    }
    source.validate().unwrap();
    let built = session.built();
    let (audit, sim_frame) = (
        built
            .correction_audit
            .clone()
            .expect("the audit was enabled"),
        built.sim.frame,
    );
    Some(Audited {
        _session: session,
        audit,
        records,
        last_record,
        parted,
        closing,
        sim_frame,
    })
}

fn line(name: &str, c: &CorrectionCounter) -> String {
    format!(
        "{name}: attempted {} changed {} first {:?} {}",
        c.attempted,
        c.changed,
        c.first_changed_frame,
        c.first_witness.as_deref().unwrap_or("")
    )
}

/// Replay, print, assert the word and the clocks never changed, and pin
/// the rest. `length` is the game's last frame: the replay must reach it.
fn audit_and_pin(name: &str, siblings: &[&str], long: (&str, &str), length: i64, want: &Expect) {
    let _pins = Pins::hold();
    let start = std::time::Instant::now();
    let Some(a) = audit_walk(siblings, long) else {
        return;
    };
    let au = &a.audit;
    let named: Vec<String> = au
        .changed_reseats
        .iter()
        .take(NAMED)
        .map(|k| format!("f{} {}/{}", k.frame, k.who, k.o))
        .collect();
    eprintln!(
        "AUDIT {name}: {} records to frame {} (sim {}, game {length}) in {:?}",
        a.records,
        a.last_record,
        a.sim_frame,
        start.elapsed()
    );
    eprintln!(
        "  lockstep: {} records parted, first {:?}; last record {} compared, {} parted",
        a.parted.0, a.parted.1, a.closing.0, a.closing.1
    );
    for (n, c) in [
        ("seed", &au.seed),
        ("clock", &au.clock),
        ("gaia_reseat", &au.gaia_reseat),
    ] {
        eprintln!("  {}", line(n, c));
    }
    eprintln!(
        "  reseats changed {:?} (+{} omitted); unlinked {}, predicate-skipped {}",
        named, au.omitted_reseats, au.unlinked_units, au.predicate_skipped_units
    );
    for (n, c) in [("seed", &au.seed), ("clock", &au.clock)] {
        if c.attempted == 0 {
            eprintln!("  {name}: the {n} counter attempted nothing and asserts nothing");
        }
    }
    assert!(
        a.sim_frame >= length,
        "{name}: the replay stopped at {} short of the game's {length}",
        a.sim_frame
    );
    // The two the brief holds at zero: plain asserts, not pins — a word or
    // a clock the harness had to change is a game that did not run on its own.
    assert_eq!(au.seed.changed, 0, "{name}: {}", line("seed", &au.seed));
    assert_eq!(au.clock.changed, 0, "{name}: {}", line("clock", &au.clock));
    pin_eq!(
        (
            au.seed.attempted,
            au.clock.attempted,
            au.gaia_reseat.attempted
        ),
        want.attempted,
        "{name}: (seed, clock, gaia_reseat) attempted"
    );
    pin_eq!(
        au.gaia_reseat.changed,
        want.reseats_changed,
        "{name}: gaia re-seats that changed an animal"
    );
    pin_eq!(
        named,
        want.first_reseats
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "{name}: the first changed re-seats"
    );
}

/// **East Indies' long word** (run54, 24,000 frames), stood up from run38's
/// start as its widenings are (`harness::tests::widen_east_indies`).
#[test]
fn east_indies_long_word_corrections_change_nothing() {
    audit_and_pin(
        "LONG_WORD_EAST_INDIES",
        &["gamelog-run38-islands-start.txt"],
        ("gamelog-run54-islands-24k-trace.txt", "rontrace-run54.log"),
        LONG_WORD_EAST_INDIES,
        &Expect {
            attempted: (1, 116, 104),
            reseats_changed: 0,
            first_reseats: &[],
        },
    );
}

/// **Great Lakes' long word** (run53, 24,000 frames), stood up from the
/// siblings' heads as its widenings are (`widen_great_lakes`).
#[test]
fn great_lakes_long_word_corrections_change_nothing() {
    audit_and_pin(
        "LONG_WORD_GREAT_LAKES",
        SIBLING_DUMPS,
        (
            "gamelog-run53-greatlakes-24k-trace.txt",
            "rontrace-run53.log",
        ),
        LONG_WORD_GREAT_LAKES,
        // run69's seven (`checkpoint::run69_single_reseat_interventions_
        // follow_the_remaining_capture`): the same sibling frames, 94..=100,
        // the sheep 8/0 put back on its own cell with its path dropped.
        &Expect {
            attempted: (14, 748, 560),
            reseats_changed: 7,
            first_reseats: &["f94 8/0", "f95 8/0", "f96 8/0", "f97 8/0"],
        },
    );
}

/// **Great Sahara's long word** (run383, 24,000 frames), the third map's
/// (Loop 3801), stood up from run381's start as `third::walk_sahara` is.
#[test]
fn great_sahara_long_word_corrections_change_nothing() {
    audit_and_pin(
        "LONG_WORD_GREAT_SAHARA",
        &[crate::diff::third::SAHARA_START],
        crate::diff::third::SAHARA_LONG,
        LONG_WORD_GREAT_SAHARA,
        &Expect {
            attempted: (1, 20, 8),
            reseats_changed: 0,
            first_reseats: &[],
        },
    );
}

/// **Great Sahara at Toughest** (run470, 15,432 frames), the third map in
/// the second pair's lobby (Loop 3801), stood up from run468's start as
/// its walks are.
#[test]
fn great_sahara_toughest_corrections_change_nothing() {
    audit_and_pin(
        "THIRD_WORD_GREAT_SAHARA_TOUGHEST",
        &[crate::diff::sahara_toughest::TOUGHEST_START],
        crate::diff::sahara_toughest::TOUGHEST_LONG,
        THIRD_WORD_GREAT_SAHARA_TOUGHEST,
        &Expect {
            attempted: (1, 20, 8),
            reseats_changed: 0,
            first_reseats: &[],
        },
    );
}

/// **The third pair's East Indies word** (run600, 17,379 frames), stood
/// up from run595's start as `third_pair`'s walk is (Loop 3801).
#[test]
fn third_pair_east_indies_corrections_change_nothing() {
    audit_and_pin(
        "THIRD_PAIR_WORD_EAST_INDIES",
        &[crate::diff::third_pair::EAST_START],
        crate::diff::third_pair::EAST_LONG,
        THIRD_PAIR_WORD_EAST_INDIES,
        &Expect {
            attempted: (1, 116, 104),
            reseats_changed: 0,
            first_reseats: &[],
        },
    );
}

/// **The third pair's Great Lakes word** (run598, 5,638 frames), stood up from
/// run597's start as `third_pair`'s walk is (Loop 3801).
#[test]
fn third_pair_great_lakes_corrections_change_nothing() {
    audit_and_pin(
        "THIRD_PAIR_WORD_GREAT_LAKES",
        &[crate::diff::third_pair::LAKES_START],
        crate::diff::third_pair::LAKES_LONG,
        THIRD_PAIR_WORD_GREAT_LAKES,
        &Expect {
            attempted: (1, 52, 40),
            reseats_changed: 0,
            first_reseats: &[],
        },
    );
}

/// One test per whole game, its audit pinned: `(seed, clock, gaia)`
/// attempted, the re-seats that changed and the first [`NAMED`] of them.
/// A row is any const with a `name`, a `start` sibling, a `long` capture
/// and a `length` — a `coverage_pair::Pair` or a `battery::Lobby` (item
/// 3801) — and its name joins [`AUDITED_ROWS`], which the scoreboard's
/// guard reads.
macro_rules! audited {
    ($($test:ident: $game:path => $att:expr, $n:expr, [$($k:expr),* $(,)?];)*) => {
        $(
            #[test]
            fn $test() {
                audit_and_pin(
                    $game.name,
                    &[$game.start],
                    $game.long,
                    $game.length,
                    &Expect {
                        attempted: $att,
                        reseats_changed: $n,
                        first_reseats: &[$($k),*],
                    },
                );
            }
        )*
        /// Every `audited!` row's game, as the handoff's lines name it.
        const AUDITED_ROWS: &[&str] = &[$($game.name),*];
    };
}

/// The audits written out by hand, `(line, word)` as the handoff names
/// them: a bare map name is on more than one line (`EastIndies` is a long
/// capture and the second pair), so these carry their line.
const HAND_AUDITED: &[(&str, &str)] = &[
    ("Long captures", "EastIndies"),
    ("Long captures", "GreatLakes"),
    ("Third map", "GreatSahara"),
    ("Third map", "GreatSaharaToughest"),
    ("Third pair", "EastIndiesFrench"),
    ("Third pair", "GreatLakesFrench"),
];

/// Whole words on an audited line that have no audit yet, `(line, word,
/// why)` — the guard holds this list to exactly the words missing, so a
/// word that gains its row fails until it is struck here, and a word that
/// closes without one fails until it is given a row or a line here (Loop
/// 3801 found these beside the third map's; which of them the audit owes
/// is the pass's).
const OWED: &[(&str, &str, &str)] = &[
    (
        "Second pair",
        "EastIndies",
        "stood up by `second::walk_second`, not one start sibling",
    ),
    (
        "Second pair",
        "GreatLakes",
        "stood up by `second::walk_second`, not one start sibling",
    ),
    (
        "Melee",
        "GreatSaharaGunpowderMelee",
        "a staged game: `audit_walk` stages no script",
    ),
    (
        "Melee",
        "GreatSaharaClassicalMelee",
        "a staged game: `audit_walk` stages no script",
    ),
    (
        "Melee",
        "GreatSaharaInformationMelee",
        "a staged game: `audit_walk` stages no script",
    ),
    (
        "Melee",
        "EastIndiesInformationMelee",
        "a staged game: `audit_walk` stages no script",
    ),
    (
        "Melee",
        "EastIndiesAirbaseMelee",
        "a staged game: `audit_walk` stages no script",
    ),
    (
        "Melee",
        "GreatSaharaCorpseMelee",
        "a staged game: `audit_walk` stages no script",
    ),
];

/// The handoff's lines whose whole words the audit owes (Loop 3801, the
/// sixty-fourth pass's corrected ruling): every line of `<word> w<N> of
/// <length>` parts — `Battery word:` among them, its lobbies promoted out
/// of the held-out battery and debugged against — but `Scoreboard:`, whose
/// parts are the score runs' ticks and orders and never read `of <length>`
/// (its maps' whole games are the `Long captures:` and `Third map:` lines).
/// The held-out battery itself (`Held-out:`, `battery::BATTERY`) is no
/// `w<N> of` line and is not audited: it is scored by its first try.
const AUDITED_LINES: &[&str] = &[
    "Long captures",
    "Battery word",
    "Second pair",
    "Third pair",
    "Coverage pair",
    "Successor pair",
    "Peace pair",
    "Late ages",
    "Third map",
    "Melee",
];

/// The line the guard leaves out, with its reason above.
const UNAUDITED_LINES: &[&str] = &["Scoreboard"];

/// **Every whole word the handoff names is audited** (Loop 3801). A word is
/// whole when its line reads `w<N> of <N>`; each one on [`AUDITED_LINES`]
/// must be an `audited!` row or a [`HAND_AUDITED`] test, so a game that
/// closes without its row is named here rather than passing unseen. The
/// queue's lines are read with the floors guards' own parser
/// (`floors::ai_line_parts`), and a line of `w<N> of` parts that neither
/// list names fails too, so a new line cannot slip past the audit. **A
/// line in the closed form** (`every word whole · <n>`, the pass of
/// 2026-11-14, R1; `floors::ai_line_whole`) names no word: its whole words
/// are its pinned rows', and the same recogniser finds an unlisted one.
#[test]
fn every_whole_scoreboard_word_is_audited() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
    let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
    for prefix in AUDITED_LINES {
        let head = format!("{prefix}:");
        assert!(
            q.lines().any(|l| l.starts_with(&head)),
            "docs/QUEUE.md has no `{head}` line"
        );
    }
    let (missing, stale) = audit_gaps(&q, AUDITED_LINES, &pinned_rows(), OWED);
    let unlisted = unlisted_lines(&q);
    assert!(
        unlisted.is_empty(),
        "docs/QUEUE.md carries `w<N> of <length>` lines the audit's guard does not \
         name: {unlisted:?}; add each to AUDITED_LINES or, with its reason, to \
         UNAUDITED_LINES"
    );
    assert!(
        missing.is_empty(),
        "whole words on the handoff with no correction audit: {missing:?}; give \
         each an `audited!` row (or a hand-written audit in HAND_AUDITED), its \
         tuple measured by the replay, and strike an audited word from OWED"
    );
    assert!(
        stale.is_empty(),
        "OWED names words that are not whole on the handoff: {stale:?}"
    );
}

/// Every pinned row an audited line is read against: `AI_WORDS`, and the
/// `Long captures:` line's `LONG_CAPTURES` (`floors::ai_line_rows`).
fn pinned_rows() -> Vec<AiWord> {
    AI_WORDS.iter().chain(LONG_CAPTURES).copied().collect()
}

/// The queue's lines that read as a scoreboard line — `w<N> of` parts the
/// floors guards' parser reads, or the closed form its recogniser does —
/// and that neither [`AUDITED_LINES`] nor [`UNAUDITED_LINES`] names.
fn unlisted_lines(q: &str) -> Vec<&str> {
    q.lines()
        .filter_map(|l| l.split_once(':').map(|(p, _)| (p, l)))
        .filter(|(p, _)| !AUDITED_LINES.contains(p) && !UNAUDITED_LINES.contains(p))
        .filter(|(p, l)| {
            super::floors::ai_line_parts(l, p).is_ok()
                || super::floors::ai_line_whole(l, p).is_some()
        })
        .map(|(p, _)| p)
        .collect()
}

/// The words whole on `prefix`'s line: the handoff's `w<N> of <N>` parts,
/// and every `pinned` row of that line whose word has reached its length
/// (Loop 3998). The line is the commander's, written at the merge; the pin
/// is the lane's own, so a word its lane closes is owed its audit there. A
/// line in the closed form names no part, and its words are the pins'
/// alone (the floors guards hold the form to every row whole).
fn whole_words(q: &str, prefix: &str, pinned: &[AiWord]) -> Vec<String> {
    let line = q.lines().find(|l| l.starts_with(&format!("{prefix}:")));
    let parts = line.map(|l| match super::floors::ai_line_whole(l, prefix) {
        Some(Ok(_)) => Vec::new(),
        Some(Err(e)) => panic!("{e}"),
        None => super::floors::ai_line_parts(l, prefix).unwrap_or_else(|e| panic!("{e}")),
    });
    let mut out: Vec<String> = parts
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, w, length)| w >= length)
        .map(|(word, _, _)| word)
        .collect();
    for row in pinned.iter().filter(|r| r.line == prefix && r.closed()) {
        if !out.iter().any(|w| w == row.map) {
            out.push(row.map.to_string());
        }
    }
    out
}

/// `(missing, stale)`: the whole words on `lines` with neither an audit nor
/// an `owed` line (or with both), and the `owed` lines whose word is not
/// whole.
fn audit_gaps(
    q: &str,
    lines: &[&str],
    pinned: &[AiWord],
    owed: &[(&str, &str, &str)],
) -> (Vec<String>, Vec<String>) {
    let mut missing = Vec::new();
    for prefix in lines {
        for word in whole_words(q, prefix, pinned) {
            let audited = AUDITED_ROWS.contains(&word.as_str())
                || HAND_AUDITED.contains(&(*prefix, word.as_str()));
            let is_owed = owed.iter().any(|(l, w, _)| l == prefix && *w == word);
            if audited == is_owed {
                missing.push(format!(
                    "{prefix}: {word} ({})",
                    if is_owed {
                        "audited and still OWED"
                    } else {
                        "no audit"
                    }
                ));
            }
        }
    }
    let stale = owed
        .iter()
        .filter(|(l, w, _)| !whole_words(q, l, pinned).iter().any(|m| m == w))
        .map(|(l, w, _)| format!("{l}: {w}"))
        .collect();
    (missing, stale)
}

/// **A word the pin closes is owed its audit in the lane that closes it**
/// (Loop 3998): 3975 re-pinned `GreatSaharaCorpseMelee` to w2600 of 2,600
/// while the queue's `Melee:` line — the commander's, written at the
/// merge — still read w744, so the guard above, in its lane gate's filters
/// since Loop 3880, was green there and red only at the drain's whole gate.
/// The guard reads the pinned `AI_WORDS` row beside the handoff's line.
#[test]
fn a_word_the_pin_closes_is_owed_before_the_handoff_says_so() {
    let row = |word| AiWord {
        line: "Melee",
        map: "GreatSaharaCorpseMelee",
        named: "",
        word,
        length: 2_600,
        endpoint: None,
        window: None,
    };
    let q = "Melee: GreatSaharaCorpseMelee w744 of 2,600 420 off\n";
    let staged = [("Melee", "GreatSaharaCorpseMelee", "a staged game")];
    // 3975's tip: the pin closed, the handoff not yet, no OWED line.
    let (missing, stale) = audit_gaps(q, &["Melee"], &[row(2_600)], &[]);
    assert_eq!(missing, ["Melee: GreatSaharaCorpseMelee (no audit)"]);
    assert!(stale.is_empty());
    // The lane's own fix, the OWED line, is green before the handoff moves.
    assert_eq!(
        audit_gaps(q, &["Melee"], &[row(2_600)], &staged),
        (vec![], vec![])
    );
    // An open pin under an open line owes nothing, and an OWED line on it
    // is stale.
    assert_eq!(
        audit_gaps(q, &["Melee"], &[row(744)], &[]),
        (vec![], vec![])
    );
    assert_eq!(
        audit_gaps(q, &["Melee"], &[row(744)], &staged).1,
        ["Melee: GreatSaharaCorpseMelee"]
    );
}

/// **The audit reads the handoff in the closed form** (the pass of
/// 2026-11-14, R1): the queue as it stands, with every line whose rows are
/// all whole rewritten as `every word whole · <n>`, owes the same audits
/// as the per-row queue — none lost where the line names no word — and a
/// new line written in the closed form is found unlisted.
#[test]
fn the_closed_form_owes_the_same_audits() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/QUEUE.md");
    let q = std::fs::read_to_string(path).expect("docs/QUEUE.md");
    let mut lines = super::floors::AI_LINES.to_vec();
    let mut audited = AUDITED_LINES.to_vec();
    lines.sort_unstable();
    audited.sort_unstable();
    assert_eq!(lines, audited, "the lines the closed form is written on");
    let closed = super::floors::closed_form_queue(&q);
    assert!(
        super::floors::closed_form_took(&closed),
        "a line's rows are all whole and no line took the closed form"
    );
    let pinned = pinned_rows();
    for row in &pinned {
        assert!(
            AUDITED_LINES.contains(&row.line),
            "{} names no audited line",
            row.line
        );
    }
    for prefix in AUDITED_LINES {
        let mut per_row = whole_words(&q, prefix, &pinned);
        let mut whole = whole_words(&closed, prefix, &pinned);
        per_row.sort();
        whole.sort();
        assert_eq!(per_row, whole, "{prefix}: the closed form's whole words");
    }
    assert_eq!(
        audit_gaps(&q, AUDITED_LINES, &pinned, OWED),
        audit_gaps(&closed, AUDITED_LINES, &pinned, OWED)
    );
    assert!(unlisted_lines(&closed).is_empty());
    let new = format!("{closed}Fourth map: every word whole \u{b7} 3\n");
    assert_eq!(unlisted_lines(&new), ["Fourth map"]);
}

audited! {
    american_enlightenment_pair_corrections_change_nothing: crate::diff::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT => (1, 132, 104), 0, [];
    american_industrial_pair_corrections_change_nothing: crate::diff::american_industrial_pair::AMERICAN_INDUSTRIAL => (1, 35, 8), 0, [];
    american_information_pair_corrections_change_nothing: crate::diff::american_information_pair::AMERICAN_INFORMATION => (1, 67, 40), 0, [];
    american_lakes_word_corrections_change_nothing: crate::diff::american_lakes_word::AMERICAN_LAKES => (1, 52, 40), 0, [];
    american_modern_pair_corrections_change_nothing: crate::diff::american_modern_pair::AMERICAN_MODERN => (1, 35, 8), 0, [];
    american_word_corrections_change_nothing: crate::diff::american_word::AMERICAN => (1, 116, 104), 0, [];
    aztec_alltech_pair_corrections_change_nothing: crate::diff::aztec_alltech_pair::AZTEC_ALLTECH => (1, 35, 8), 0, [];
    aztec_ancient_pair_corrections_change_nothing: crate::diff::aztec_ancient_pair::AZTEC_ANCIENT => (1, 132, 104), 0, [];
    aztec_medieval_pair_corrections_change_nothing: crate::diff::aztec_medieval_pair::AZTEC_MEDIEVAL => (1, 36, 8), 0, [];
    aztec_modern_pair_corrections_change_nothing: crate::diff::aztec_modern_pair::AZTEC_MODERN => (1, 67, 40), 0, [];
    aztec_word_corrections_change_nothing: crate::diff::aztec_word::AZTEC => (1, 20, 8), 0, [];
    bantu_ancient_pair_corrections_change_nothing: crate::diff::bantu_ancient_pair::BANTU_ANCIENT => (1, 132, 104), 0, [];
    bantu_medieval_pair_corrections_change_nothing: crate::diff::bantu_medieval_pair::BANTU_MEDIEVAL => (1, 36, 8), 0, [];
    bantu_modern_pair_corrections_change_nothing: crate::diff::bantu_modern_pair::BANTU_MODERN => (1, 67, 40), 0, [];
    bantu_word_corrections_change_nothing: crate::diff::bantu_word::BANTU => (1, 20, 8), 0, [];
    british_classical_pair_corrections_change_nothing: crate::diff::british_classical_pair::BRITISH_CLASSICAL => (1, 132, 104), 0, [];
    british_enlightenment_pair_corrections_change_nothing: crate::diff::british_enlightenment_pair::BRITISH_ENLIGHTENMENT => (1, 132, 104), 0, [];
    british_industrial_pair_corrections_change_nothing: crate::diff::british_industrial_pair::BRITISH_INDUSTRIAL => (1, 35, 8), 0, [];
    british_medieval_pair_corrections_change_nothing: crate::diff::british_medieval_pair::BRITISH_MEDIEVAL => (1, 68, 40), 0, [];
    british_modern_pair_corrections_change_nothing: crate::diff::british_modern_pair::BRITISH_MODERN => (1, 67, 40), 0, [];
    chinese_enlightenment_pair_corrections_change_nothing: crate::diff::chinese_enlightenment_pair::CHINESE_ENLIGHTENMENT => (1, 68, 40), 0, [];
    chinese_gunpowder_pair_corrections_change_nothing: crate::diff::chinese_gunpowder_pair::CHINESE_GUNPOWDER => (1, 132, 104), 0, [];
    chinese_industrial_pair_corrections_change_nothing: crate::diff::chinese_industrial_pair::CHINESE_INDUSTRIAL => (1, 35, 8), 0, [];
    chinese_medieval_pair_corrections_change_nothing: crate::diff::chinese_medieval_pair::CHINESE_MEDIEVAL => (1, 36, 8), 0, [];
    chinese_modern_pair_corrections_change_nothing: crate::diff::chinese_modern_pair::CHINESE_MODERN => (1, 131, 104), 0, [];
    coverage_pair_corrections_change_nothing: crate::diff::coverage_pair::COVERAGE => (1, 131, 104), 0, [];
    dutch_medieval_pair_corrections_change_nothing: crate::diff::dutch_medieval_pair::DUTCH_MEDIEVAL => (1, 138, 104), 0, [];
    dutch_sahara_word_corrections_change_nothing: crate::diff::dutch_sahara_word::DUTCH_SAHARA => (1, 23, 8), 0, [];
    dutch_word_corrections_change_nothing: crate::diff::dutch_word::DUTCH => (1, 122, 104), 0, [];
    egyptian_classical_pair_corrections_change_nothing: crate::diff::egyptian_classical_pair::EGYPTIAN_CLASSICAL => (1, 132, 104), 0, [];
    egyptian_gunpowder_pair_corrections_change_nothing: crate::diff::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER => (1, 36, 8), 0, [];
    egyptian_industrial_pair_corrections_change_nothing: crate::diff::egyptian_industrial_pair::EGYPTIAN_INDUSTRIAL => (1, 131, 104), 0, [];
    egyptian_information_pair_corrections_change_nothing: crate::diff::egyptian_information_pair::EGYPTIAN_INFORMATION => (1, 67, 40), 0, [];
    egyptian_modern_pair_corrections_change_nothing: crate::diff::egyptian_modern_pair::EGYPTIAN_MODERN => (1, 67, 40), 0, [];
    egyptian_word_corrections_change_nothing: crate::diff::egyptian_word::EGYPTIAN => (1, 20, 8), 0, [];
    french_classical_pair_corrections_change_nothing: crate::diff::french_classical_pair::FRENCH_CLASSICAL => (1, 36, 8), 0, [];
    french_enlightenment_pair_corrections_change_nothing: crate::diff::french_enlightenment_pair::FRENCH_ENLIGHTENMENT => (1, 132, 104), 0, [];
    french_information_pair_corrections_change_nothing: crate::diff::french_information_pair::FRENCH_INFORMATION => (1, 35, 8), 0, [];
    french_medieval_pair_corrections_change_nothing: crate::diff::french_medieval_pair::FRENCH_MEDIEVAL => (1, 68, 40), 0, [];
    french_word_corrections_change_nothing: crate::diff::french_word::FRENCH => (1, 20, 8), 0, [];
    german_industrial_moderate_pair_corrections_change_nothing: crate::diff::german_industrial_moderate_pair::GERMAN_INDUSTRIAL_MODERATE => (1, 67, 40), 0, [];
    german_alltech_pair_corrections_change_nothing: crate::diff::german_alltech_pair::GERMAN_ALLTECH => (1, 131, 104), 0, [];
    german_ancient_pair_corrections_change_nothing: crate::diff::german_ancient_pair::GERMAN_ANCIENT => (1, 36, 8), 0, [];
    german_industrial_pair_corrections_change_nothing: crate::diff::german_industrial_pair::GERMAN_INDUSTRIAL => (1, 67, 40), 0, [];
    german_modern_pair_corrections_change_nothing: crate::diff::german_modern_pair::GERMAN_MODERN => (1, 67, 40), 0, [];
    german_sahara_word_corrections_change_nothing: crate::diff::german_sahara_word::GERMAN_SAHARA => (1, 20, 8), 0, [];
    german_word_corrections_change_nothing: crate::diff::german_word::GERMAN => (1, 116, 104), 0, [];
    greek_classical_pair_corrections_change_nothing: crate::diff::greek_classical_pair::GREEK_CLASSICAL => (1, 132, 104), 0, [];
    greek_enlightenment_pair_corrections_change_nothing: crate::diff::greek_enlightenment_pair::GREEK_ENLIGHTENMENT => (1, 68, 40), 0, [];
    greek_gunpowder_pair_corrections_change_nothing: crate::diff::greek_gunpowder_pair::GREEK_GUNPOWDER => (1, 36, 8), 0, [];
    greek_medieval_pair_corrections_change_nothing: crate::diff::greek_medieval_pair::GREEK_MEDIEVAL => (1, 132, 104), 0, [];
    greek_word_corrections_change_nothing: crate::diff::greek_word::GREEK => (1, 20, 8), 0, [];
    inca_classical_pair_corrections_change_nothing: crate::diff::inca_classical_pair::INCA_CLASSICAL => (1, 132, 104), 0, [];
    inca_enlightenment_pair_corrections_change_nothing: crate::diff::inca_enlightenment_pair::INCA_ENLIGHTENMENT => (1, 36, 8), 0, [];
    inca_industrial_pair_corrections_change_nothing: crate::diff::inca_industrial_pair::INCA_INDUSTRIAL => (1, 67, 40), 0, [];
    inca_sahara_word_corrections_change_nothing: crate::diff::inca_sahara_word::INCA_SAHARA => (1, 20, 8), 0, [];
    inca_word_corrections_change_nothing: crate::diff::inca_word::INCA => (1, 116, 104), 0, [];
    indian_enlightenment_pair_corrections_change_nothing: crate::diff::indian_enlightenment_pair::INDIAN_ENLIGHTENMENT => (1, 132, 104), 0, [];
    indian_sahara_word_corrections_change_nothing: crate::diff::indian_sahara_word::INDIAN_SAHARA => (1, 20, 8), 0, [];
    indian_word_corrections_change_nothing: crate::diff::indian_word::INDIAN => (1, 116, 104), 0, [];
    iroquois_ancient_pair_corrections_change_nothing: crate::diff::iroquois_ancient_pair::IROQUOIS_ANCIENT => (1, 36, 8), 0, [];
    iroquois_enlightenment_pair_corrections_change_nothing: crate::diff::iroquois_enlightenment_pair::IROQUOIS_ENLIGHTENMENT => (1, 132, 104), 0, [];
    iroquois_sahara_word_corrections_change_nothing: crate::diff::iroquois_sahara_word::IROQUOIS_SAHARA => (1, 20, 8), 0, [];
    iroquois_word_corrections_change_nothing: crate::diff::iroquois_word::IROQUOIS => (1, 52, 40), 0, [];
    japanese_ancient_pair_corrections_change_nothing: crate::diff::japanese_ancient_pair::JAPANESE_ANCIENT => (1, 132, 104), 0, [];
    japanese_enlightenment_pair_corrections_change_nothing: crate::diff::japanese_enlightenment_pair::JAPANESE_ENLIGHTENMENT => (1, 132, 104), 0, [];
    japanese_gunpowder_pair_corrections_change_nothing: crate::diff::japanese_gunpowder_pair::JAPANESE_GUNPOWDER => (1, 36, 8), 0, [];
    japanese_industrial_pair_corrections_change_nothing: crate::diff::japanese_industrial_pair::JAPANESE_INDUSTRIAL => (1, 131, 104), 0, [];
    japanese_medieval_pair_corrections_change_nothing: crate::diff::japanese_medieval_pair::JAPANESE_MEDIEVAL => (1, 68, 40), 0, [];
    japanese_modern_pair_corrections_change_nothing: crate::diff::japanese_modern_pair::JAPANESE_MODERN => (1, 67, 40), 0, [];
    japanese_word_corrections_change_nothing: crate::diff::japanese_word::JAPANESE => (1, 20, 8), 0, [];
    korean_ancient_pair_corrections_change_nothing: crate::diff::korean_ancient_pair::KOREAN_ANCIENT => (1, 133, 104), 0, [];
    korean_classical_pair_corrections_change_nothing: crate::diff::korean_classical_pair::KOREAN_CLASSICAL => (1, 37, 8), 0, [];
    korean_medieval_pair_corrections_change_nothing: crate::diff::korean_medieval_pair::KOREAN_MEDIEVAL => (1, 69, 40), 0, [];
    korean_word_corrections_change_nothing: crate::diff::korean_word::KOREAN => (1, 21, 8), 0, [];
    lakota_enlightenment_pair_corrections_change_nothing: crate::diff::lakota_enlightenment_pair::LAKOTA_ENLIGHTENMENT => (1, 65, 40), 0, [];
    lakota_gunpowder_pair_corrections_change_nothing: crate::diff::lakota_gunpowder_pair::LAKOTA_GUNPOWDER => (1, 33, 8), 0, [];
    lakota_medieval_pair_corrections_change_nothing: crate::diff::lakota_medieval_pair::LAKOTA_MEDIEVAL => (1, 129, 104), 0, [];
    lakota_pair_corrections_change_nothing: crate::diff::lakota_pair::LAKOTA => (1, 128, 104), 0, [];
    lakota_word_corrections_change_nothing: crate::diff::lakota_word::LAKOTA => (1, 49, 40), 0, [];
    maya_ancient_pair_corrections_change_nothing: crate::diff::maya_ancient_pair::MAYA_ANCIENT => (1, 132, 104), 0, [];
    maya_information_pair_corrections_change_nothing: crate::diff::maya_information_pair::MAYA_INFORMATION => (1, 67, 40), 0, [];
    maya_word_corrections_change_nothing: crate::diff::maya_word::MAYA => (1, 20, 8), 0, [];
    mongol_classical_pair_corrections_change_nothing: crate::diff::mongol_classical_pair::MONGOL_CLASSICAL => (1, 36, 8), 0, [];
    mongol_enlightenment_pair_corrections_change_nothing: crate::diff::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT => (1, 68, 40), 0, [];
    mongol_gunpowder_pair_corrections_change_nothing: crate::diff::mongol_gunpowder_pair::MONGOL_GUNPOWDER => (1, 132, 104), 0, [];
    mongol_medieval_pair_corrections_change_nothing: crate::diff::mongol_medieval_pair::MONGOL_MEDIEVAL => (1, 132, 104), 0, [];
    mongol_pair_corrections_change_nothing: crate::diff::mongol_pair::MONGOL => (1, 131, 104), 0, [];
    persian_classical_pair_corrections_change_nothing: crate::diff::persian_classical_pair::PERSIAN_CLASSICAL => (1, 68, 40), 0, [];
    persian_gunpowder_pair_corrections_change_nothing: crate::diff::persian_gunpowder_pair::PERSIAN_GUNPOWDER => (1, 132, 104), 0, [];
    persian_medieval_pair_corrections_change_nothing: crate::diff::persian_medieval_pair::PERSIAN_MEDIEVAL => (1, 132, 104), 0, [];
    persian_word_corrections_change_nothing: crate::diff::persian_word::PERSIAN => (1, 52, 40), 0, [];
    roman_classical_pair_corrections_change_nothing: crate::diff::roman_classical_pair::ROMAN_CLASSICAL => (1, 36, 8), 0, [];
    roman_gunpowder_pair_corrections_change_nothing: crate::diff::roman_gunpowder_pair::ROMAN_GUNPOWDER => (1, 68, 40), 0, [];
    roman_medieval_pair_corrections_change_nothing: crate::diff::roman_medieval_pair::ROMAN_MEDIEVAL => (1, 132, 104), 0, [];
    roman_modern_pair_corrections_change_nothing: crate::diff::roman_modern_pair::ROMAN_MODERN => (1, 131, 104), 0, [];
    roman_word_corrections_change_nothing: crate::diff::roman_word::ROMAN => (1, 116, 104), 0, [];
    russian_gunpowder_pair_corrections_change_nothing: crate::diff::russian_gunpowder_pair::RUSSIAN_GUNPOWDER => (1, 68, 40), 0, [];
    russian_information_pair_corrections_change_nothing: crate::diff::russian_information_pair::RUSSIAN_INFORMATION => (1, 131, 104), 0, [];
    russian_medieval_pair_corrections_change_nothing: crate::diff::russian_medieval_pair::RUSSIAN_MEDIEVAL => (1, 36, 8), 0, [];
    russian_modern_pair_corrections_change_nothing: crate::diff::russian_modern_pair::RUSSIAN_MODERN => (1, 131, 104), 0, [];
    russian_word_corrections_change_nothing: crate::diff::russian_word::RUSSIAN => (1, 52, 40), 0, [];
    sahara_coverage_corrections_change_nothing: crate::diff::sahara_coverage::SAHARA_COVERAGE => (1, 35, 8), 0, [];
    sahara_mongol_corrections_change_nothing: crate::diff::sahara_mongol::SAHARA_MONGOL => (1, 35, 8), 0, [];
    spanish_classical_pair_corrections_change_nothing: crate::diff::spanish_classical_pair::SPANISH_CLASSICAL => (1, 36, 8), 0, [];
    spanish_east_word_corrections_change_nothing: crate::diff::spanish_east_word::SPANISH_EAST => (1, 116, 104), 0, [];
    spanish_gunpowder_pair_corrections_change_nothing: crate::diff::spanish_gunpowder_pair::SPANISH_GUNPOWDER => (1, 132, 104), 0, [];
    spanish_industrial_pair_corrections_change_nothing: crate::diff::spanish_industrial_pair::SPANISH_INDUSTRIAL => (1, 67, 40), 0, [];
    spanish_information_pair_corrections_change_nothing: crate::diff::spanish_information_pair::SPANISH_INFORMATION => (1, 67, 40), 0, [];
    spanish_lakes_word_corrections_change_nothing: crate::diff::spanish_lakes_word::SPANISH_LAKES => (1, 52, 40), 0, [];
    spanish_modern_pair_corrections_change_nothing: crate::diff::spanish_modern_pair::SPANISH_MODERN => (1, 131, 104), 0, [];
    spanish_word_corrections_change_nothing: crate::diff::spanish_word::SPANISH => (1, 20, 8), 0, [];
    turkish_enlightenment_pair_corrections_change_nothing: crate::diff::turkish_enlightenment_pair::TURKISH_ENLIGHTENMENT => (1, 36, 8), 0, [];
    turkish_information_pair_corrections_change_nothing: crate::diff::turkish_information_pair::TURKISH_INFORMATION => (1, 131, 104), 0, [];
    turkish_word_corrections_change_nothing: crate::diff::turkish_word::TURKISH => (1, 52, 40), 0, [];
}
