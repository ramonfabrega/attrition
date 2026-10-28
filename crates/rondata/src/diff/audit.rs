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

use super::coverage_pair::Pair;
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

fn audit_pair(p: &Pair, want: &Expect) {
    audit_and_pin(p.name, &[p.start], p.long, p.length, want);
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

/// One test per whole pair, its audit pinned: `(seed, clock, gaia)`
/// attempted, the re-seats that changed and the first [`NAMED`] of them.
macro_rules! audited {
    ($($test:ident: $pair:path => $att:expr, $n:expr, [$($k:expr),* $(,)?];)*) => {$(
        #[test]
        fn $test() {
            audit_pair(
                &$pair,
                &Expect {
                    attempted: $att,
                    reseats_changed: $n,
                    first_reseats: &[$($k),*],
                },
            );
        }
    )*};
}

audited! {
    american_modern_pair_corrections_change_nothing: crate::diff::american_modern_pair::AMERICAN_MODERN => (1, 35, 8), 0, [];
    russian_modern_pair_corrections_change_nothing: crate::diff::russian_modern_pair::RUSSIAN_MODERN => (1, 131, 104), 0, [];
    american_enlightenment_pair_corrections_change_nothing: crate::diff::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT => (1, 132, 104), 0, [];
    american_industrial_pair_corrections_change_nothing: crate::diff::american_industrial_pair::AMERICAN_INDUSTRIAL => (1, 35, 8), 0, [];
    aztec_alltech_pair_corrections_change_nothing: crate::diff::aztec_alltech_pair::AZTEC_ALLTECH => (1, 35, 8), 0, [];
    aztec_ancient_pair_corrections_change_nothing: crate::diff::aztec_ancient_pair::AZTEC_ANCIENT => (1, 132, 104), 0, [];
    aztec_medieval_pair_corrections_change_nothing: crate::diff::aztec_medieval_pair::AZTEC_MEDIEVAL => (1, 36, 8), 0, [];
    bantu_ancient_pair_corrections_change_nothing: crate::diff::bantu_ancient_pair::BANTU_ANCIENT => (1, 132, 104), 0, [];
    bantu_medieval_pair_corrections_change_nothing: crate::diff::bantu_medieval_pair::BANTU_MEDIEVAL => (1, 36, 8), 0, [];
    british_classical_pair_corrections_change_nothing: crate::diff::british_classical_pair::BRITISH_CLASSICAL => (1, 132, 104), 0, [];
    british_enlightenment_pair_corrections_change_nothing: crate::diff::british_enlightenment_pair::BRITISH_ENLIGHTENMENT => (1, 132, 104), 0, [];
    british_industrial_pair_corrections_change_nothing: crate::diff::british_industrial_pair::BRITISH_INDUSTRIAL => (1, 35, 8), 0, [];
    british_medieval_pair_corrections_change_nothing: crate::diff::british_medieval_pair::BRITISH_MEDIEVAL => (1, 68, 40), 0, [];
    chinese_enlightenment_pair_corrections_change_nothing: crate::diff::chinese_enlightenment_pair::CHINESE_ENLIGHTENMENT => (1, 68, 40), 0, [];
    chinese_gunpowder_pair_corrections_change_nothing: crate::diff::chinese_gunpowder_pair::CHINESE_GUNPOWDER => (1, 132, 104), 0, [];
    chinese_industrial_pair_corrections_change_nothing: crate::diff::chinese_industrial_pair::CHINESE_INDUSTRIAL => (1, 35, 8), 0, [];
    chinese_medieval_pair_corrections_change_nothing: crate::diff::chinese_medieval_pair::CHINESE_MEDIEVAL => (1, 36, 8), 0, [];
    coverage_pair_corrections_change_nothing: crate::diff::coverage_pair::COVERAGE => (1, 131, 104), 0, [];
    dutch_medieval_pair_corrections_change_nothing: crate::diff::dutch_medieval_pair::DUTCH_MEDIEVAL => (1, 138, 104), 0, [];
    egyptian_classical_pair_corrections_change_nothing: crate::diff::egyptian_classical_pair::EGYPTIAN_CLASSICAL => (1, 132, 104), 0, [];
    egyptian_gunpowder_pair_corrections_change_nothing: crate::diff::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER => (1, 36, 8), 0, [];
    egyptian_industrial_pair_corrections_change_nothing: crate::diff::egyptian_industrial_pair::EGYPTIAN_INDUSTRIAL => (1, 131, 104), 0, [];
    french_classical_pair_corrections_change_nothing: crate::diff::french_classical_pair::FRENCH_CLASSICAL => (1, 36, 8), 0, [];
    french_enlightenment_pair_corrections_change_nothing: crate::diff::french_enlightenment_pair::FRENCH_ENLIGHTENMENT => (1, 132, 104), 0, [];
    german_alltech_pair_corrections_change_nothing: crate::diff::german_alltech_pair::GERMAN_ALLTECH => (1, 131, 104), 0, [];
    german_ancient_pair_corrections_change_nothing: crate::diff::german_ancient_pair::GERMAN_ANCIENT => (1, 36, 8), 0, [];
    german_industrial_pair_corrections_change_nothing: crate::diff::german_industrial_pair::GERMAN_INDUSTRIAL => (1, 67, 40), 0, [];
    greek_classical_pair_corrections_change_nothing: crate::diff::greek_classical_pair::GREEK_CLASSICAL => (1, 132, 104), 0, [];
    greek_enlightenment_pair_corrections_change_nothing: crate::diff::greek_enlightenment_pair::GREEK_ENLIGHTENMENT => (1, 68, 40), 0, [];
    greek_gunpowder_pair_corrections_change_nothing: crate::diff::greek_gunpowder_pair::GREEK_GUNPOWDER => (1, 36, 8), 0, [];
    greek_medieval_pair_corrections_change_nothing: crate::diff::greek_medieval_pair::GREEK_MEDIEVAL => (1, 132, 104), 0, [];
    inca_classical_pair_corrections_change_nothing: crate::diff::inca_classical_pair::INCA_CLASSICAL => (1, 132, 104), 0, [];
    inca_enlightenment_pair_corrections_change_nothing: crate::diff::inca_enlightenment_pair::INCA_ENLIGHTENMENT => (1, 36, 8), 0, [];
    inca_industrial_pair_corrections_change_nothing: crate::diff::inca_industrial_pair::INCA_INDUSTRIAL => (1, 67, 40), 0, [];
    indian_enlightenment_pair_corrections_change_nothing: crate::diff::indian_enlightenment_pair::INDIAN_ENLIGHTENMENT => (1, 132, 104), 0, [];
    iroquois_ancient_pair_corrections_change_nothing: crate::diff::iroquois_ancient_pair::IROQUOIS_ANCIENT => (1, 36, 8), 0, [];
    iroquois_enlightenment_pair_corrections_change_nothing: crate::diff::iroquois_enlightenment_pair::IROQUOIS_ENLIGHTENMENT => (1, 132, 104), 0, [];
    japanese_ancient_pair_corrections_change_nothing: crate::diff::japanese_ancient_pair::JAPANESE_ANCIENT => (1, 132, 104), 0, [];
    japanese_enlightenment_pair_corrections_change_nothing: crate::diff::japanese_enlightenment_pair::JAPANESE_ENLIGHTENMENT => (1, 132, 104), 0, [];
    japanese_gunpowder_pair_corrections_change_nothing: crate::diff::japanese_gunpowder_pair::JAPANESE_GUNPOWDER => (1, 36, 8), 0, [];
    japanese_industrial_pair_corrections_change_nothing: crate::diff::japanese_industrial_pair::JAPANESE_INDUSTRIAL => (1, 131, 104), 0, [];
    japanese_medieval_pair_corrections_change_nothing: crate::diff::japanese_medieval_pair::JAPANESE_MEDIEVAL => (1, 68, 40), 0, [];
    korean_ancient_pair_corrections_change_nothing: crate::diff::korean_ancient_pair::KOREAN_ANCIENT => (1, 133, 104), 0, [];
    korean_classical_pair_corrections_change_nothing: crate::diff::korean_classical_pair::KOREAN_CLASSICAL => (1, 37, 8), 0, [];
    korean_medieval_pair_corrections_change_nothing: crate::diff::korean_medieval_pair::KOREAN_MEDIEVAL => (1, 69, 40), 0, [];
    lakota_enlightenment_pair_corrections_change_nothing: crate::diff::lakota_enlightenment_pair::LAKOTA_ENLIGHTENMENT => (1, 65, 40), 0, [];
    lakota_gunpowder_pair_corrections_change_nothing: crate::diff::lakota_gunpowder_pair::LAKOTA_GUNPOWDER => (1, 33, 8), 0, [];
    lakota_medieval_pair_corrections_change_nothing: crate::diff::lakota_medieval_pair::LAKOTA_MEDIEVAL => (1, 129, 104), 0, [];
    lakota_pair_corrections_change_nothing: crate::diff::lakota_pair::LAKOTA => (1, 128, 104), 0, [];
    maya_ancient_pair_corrections_change_nothing: crate::diff::maya_ancient_pair::MAYA_ANCIENT => (1, 132, 104), 0, [];
    mongol_classical_pair_corrections_change_nothing: crate::diff::mongol_classical_pair::MONGOL_CLASSICAL => (1, 36, 8), 0, [];
    mongol_enlightenment_pair_corrections_change_nothing: crate::diff::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT => (1, 68, 40), 0, [];
    mongol_gunpowder_pair_corrections_change_nothing: crate::diff::mongol_gunpowder_pair::MONGOL_GUNPOWDER => (1, 132, 104), 0, [];
    mongol_pair_corrections_change_nothing: crate::diff::mongol_pair::MONGOL => (1, 131, 104), 0, [];
    persian_classical_pair_corrections_change_nothing: crate::diff::persian_classical_pair::PERSIAN_CLASSICAL => (1, 68, 40), 0, [];
    persian_gunpowder_pair_corrections_change_nothing: crate::diff::persian_gunpowder_pair::PERSIAN_GUNPOWDER => (1, 132, 104), 0, [];
    persian_medieval_pair_corrections_change_nothing: crate::diff::persian_medieval_pair::PERSIAN_MEDIEVAL => (1, 132, 104), 0, [];
    roman_classical_pair_corrections_change_nothing: crate::diff::roman_classical_pair::ROMAN_CLASSICAL => (1, 36, 8), 0, [];
    roman_gunpowder_pair_corrections_change_nothing: crate::diff::roman_gunpowder_pair::ROMAN_GUNPOWDER => (1, 68, 40), 0, [];
    roman_medieval_pair_corrections_change_nothing: crate::diff::roman_medieval_pair::ROMAN_MEDIEVAL => (1, 132, 104), 0, [];
    russian_gunpowder_pair_corrections_change_nothing: crate::diff::russian_gunpowder_pair::RUSSIAN_GUNPOWDER => (1, 68, 40), 0, [];
    russian_medieval_pair_corrections_change_nothing: crate::diff::russian_medieval_pair::RUSSIAN_MEDIEVAL => (1, 36, 8), 0, [];
    sahara_coverage_corrections_change_nothing: crate::diff::sahara_coverage::SAHARA_COVERAGE => (1, 35, 8), 0, [];
    sahara_mongol_corrections_change_nothing: crate::diff::sahara_mongol::SAHARA_MONGOL => (1, 35, 8), 0, [];
    spanish_classical_pair_corrections_change_nothing: crate::diff::spanish_classical_pair::SPANISH_CLASSICAL => (1, 36, 8), 0, [];
    spanish_gunpowder_pair_corrections_change_nothing: crate::diff::spanish_gunpowder_pair::SPANISH_GUNPOWDER => (1, 132, 104), 0, [];
    spanish_industrial_pair_corrections_change_nothing: crate::diff::spanish_industrial_pair::SPANISH_INDUSTRIAL => (1, 67, 40), 0, [];
    turkish_enlightenment_pair_corrections_change_nothing: crate::diff::turkish_enlightenment_pair::TURKISH_ENLIGHTENMENT => (1, 36, 8), 0, [];
}
