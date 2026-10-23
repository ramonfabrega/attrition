//! **The Aztecs at All Technologies, Great Sahara** (DECISIONS 90, item
//! 3507): one of the fifty-second tranche's late-age pairs, the land line at
//! age 7 on the land map. `MAP_STYLE 7`, human Nubians (4) against **Aztecs
//! (0)** at **Difficulty 3**, `STARTING_TECHNOLOGY 8`, `ENDING_TECHNOLOGY 7`
//! and `STARTING_RESOURCES 7`, seed 12345 — the Great Sahara Mongol pair's
//! lobby (`sahara_mongol.rs`) with the nation varied. The captures are
//! run1146 (the `DUMP_ALL` start) and run1147 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` one block in a hundred). The first
//! parting is in `docs/AI.md` §301.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AZTEC_ALLTECH: Pair = Pair {
    name: "GreatSaharaAztecAllTech",
    map_style: 7,
    ai_tribe: 0,
    difficulty: 3,
    starting_technology: 8,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1146-greatsahara-aztec-alltech-start.txt",
    long: (
        "gamelog-run1147-greatsahara-aztec-alltech-trace.txt",
        "rontrace-run1147.log",
    ),
    // The game ended itself on 4565 at Difficulty 3 (closing block 4566).
    length: 4565,
    // Item 3507, first try: frame 4382 by the count and the sequence, ours
    // 607 draws against 567, index 9: ours `Leader::produce_building+0x1805`,
    // theirs `Leader::make_stuff+0x63d` (`docs/AI.md` section 301).
    // Item 3514: whole, 4382 -> 4565. `get_cost`'s domain arm prices the
    // Silo `1/2010`'s Nuclear ICBM at 872/942 with the Space Program held
    // (ours charged 1224/1366 and escrowed it), and the draw stream agrees
    // to the game's last frame; the closing block 4566 compares whole.
    count: 4565,
    sequence: 4565,
};

/// The pair's word as the handoff's line and `AI_WORDS` carry it.
pub(crate) const AZTEC_ALLTECH_WORD: i64 = if AZTEC_ALLTECH.count < AZTEC_ALLTECH.sequence {
    AZTEC_ALLTECH.count
} else {
    AZTEC_ALLTECH.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn aztec_alltech_pair_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AZTEC_ALLTECH.start, AZTEC_ALLTECH.long) else {
        return;
    };
    eprintln!(
        "AZTEC_ALLTECH {}: count {} sequence {} of {}; {}",
        AZTEC_ALLTECH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        AZTEC_ALLTECH.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, AZTEC_ALLTECH.length, "the game's last frame");
    pin_eq!(
        w.count,
        AZTEC_ALLTECH.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        AZTEC_ALLTECH.sequence,
        "the draw sequence's first parting"
    );
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — map, difficulty, the age range, resources and nations — and the
/// long is scoreable. A setting that did not take reads back as its default.
#[test]
fn aztec_alltech_pair_captures_say_their_lobby() {
    let p = &AZTEC_ALLTECH;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Aztec All Technologies long is not scoreable"
    );
}

/// **The pair's closing state**: run1147's closing dump, block 4566,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3514).
#[test]
fn run1147_aztec_alltech_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &AZTEC_ALLTECH;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Aztec pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 4566, "the closing block");
    // Item 3514, the pair whole: 263 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 263, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1147b: the word 4382's window, blocks 4376..4566 at the long's detail
/// — six blocks before the word's block 4383, and on to the game's closing
/// block 4566 (item 3507).
pub(crate) const RUN1147B: &str = "gamelog-run1147b-greatsahara-aztec-alltech-window-4376-4566.txt";

/// The window: block 4376 through the closing block 4566; the word 4382's
/// own block is 4383.
pub(crate) const WIDENING_AZTEC_ALLTECH_FRAME_4382: (i64, i64) = (4376, 4566);

/// **The word 4382's window**: run1147b walked from run1146's start with
/// the recorder on (item 3507).
pub(crate) fn aztec_alltech_frame_4382_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_ALLTECH.start],
        true,
        AZTEC_ALLTECH.long,
        "run1147b",
        &[(RUN1147B, WIDENING_AZTEC_ALLTECH_FRAME_4382.0)],
        WIDENING_AZTEC_ALLTECH_FRAME_4382,
        1,
        &[4383],
        true,
    )
}

#[test]
fn aztec_alltech_pair_run1147b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_alltech_frame_4382_window() else {
        return;
    };
    pin_eq!(w.blocks, 190, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // The word 4382's block is 4383. Item 3507, first try: 2415.
    // Item 3514: 2415 -> 308, the Silo's ICBM queued on 4383 on both sides
    // (`num_queued[316]` 1, `bucket[3:knowledge]` 228, `bucket[5:oil]`
    // 8064) and the word whole at 4565, whose block 4566 closes the window.
    // 92 keys stand on 4376 as before; the first later row is `1/93`'s
    // `attrition` on 4388 (ours 6 against 24), the family parked 3506b.
    pin_eq!(w.firsts.len(), 308, "run1147b baseline");
}
