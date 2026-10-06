//! **Great Sahara in the coverage pair's lobby** (DECISIONS 56 §1, item
//! 1538): the closed map's successor once Toughest's closing state agreed.
//! `MAP_STYLE 7`, human Nubians (4) against Persians (23) at Toughest,
//! `STARTING_TECHNOLOGY 8` and `STARTING_RESOURCES 7`, seed 12345 — run651's
//! and run652's recipes with the map moved and nothing else. The captures
//! are run675 (the `DUMP_ALL` start, the lobby's own sibling) and run676 (the
//! draw stream to the game's end, `cover=0`). The first parting is the third
//! map's word in the newest pair's lobby; `docs/AI.md` §147 has it.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SAHARA_COVERAGE: Pair = Pair {
    name: "GreatSaharaPersianAllTech",
    map_style: 7,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run675-greatsahara-persian-alltech-start.txt",
    long: (
        "gamelog-run676-greatsahara-persian-alltech-24k-trace.txt",
        "rontrace-run676.log",
    ),
    length: 4340,
    // Item 1538 moved it 12 → 718 (the tied oil patches); item 1544's
    // `total_units` tally, merged beside it, moved it 718 → 720: ours
    // `Guy::set_anim+0x97a < Guy::move+0x19f` against the original's
    // `Animal::think_bird+0x82`, 25 draws against 24.
    count: 720,
    sequence: 720,
};

/// The lobby's word as the handoff's `Third map:` line and `AI_WORDS` carry
/// it: the lower of the two partings.
pub(crate) const SAHARA_COVERAGE_WORD: i64 = if SAHARA_COVERAGE.count < SAHARA_COVERAGE.sequence {
    SAHARA_COVERAGE.count
} else {
    SAHARA_COVERAGE.sequence
};

/// **The third map's first parting in the coverage pair's lobby**, walked
/// from its own start: the frame the draw count first parts on, the frame
/// the draw sequence first parts on, and the lobby read back.
#[test]
fn sahara_coverage_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SAHARA_COVERAGE.start, SAHARA_COVERAGE.long) else {
        return;
    };
    eprintln!(
        "SAHARA_COVERAGE {}: count {} sequence {} of {}; {}",
        SAHARA_COVERAGE.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        SAHARA_COVERAGE.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, SAHARA_COVERAGE.length, "the game's last frame");
    pin_eq!(
        w.count,
        SAHARA_COVERAGE.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SAHARA_COVERAGE.sequence,
        "the draw sequence's first parting"
    );
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — map, difficulty, the two settings the lobby exists for, nations
/// and seed — and the long is scoreable. A setting that did not take reads
/// back as its default here.
#[test]
fn sahara_coverage_captures_say_their_lobby() {
    let p = &SAHARA_COVERAGE;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
        assert_eq!(
            info("STARTING_TECHNOLOGY"),
            Some(p.starting_technology),
            "{name}"
        );
        assert_eq!(
            info("STARTING_RESOURCES"),
            Some(p.starting_resources),
            "{name}"
        );
        assert_eq!(info("GAME_RULES"), Some(1), "{name}");
        let tribes: Vec<_> = init
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe))
            .collect();
        assert_eq!(tribes, [(0, 4), (1, i64::from(p.ai_tribe))], "{name}");
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Sahara coverage long is not scoreable"
    );
}

/// The lobby is a scored one and none of the battery's.
#[test]
fn sahara_coverage_is_no_other_lobby() {
    let p = &SAHARA_COVERAGE;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run677, item 1538: the lobby's blocks 6..262 at the long's detail — the
/// word 12's block 13 with seven before it and 249 after.
pub(crate) const RUN677: &str = "gamelog-run677-greatsahara-persian-alltech-window-6-262.txt";

/// The window: block 6 through 262; the word 12's own block is 13.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_12: (i64, i64) = (6, 262);

/// **The coverage lobby's frame-12 window**: run677 walked from run675's
/// start with the recorder on. It was the `AI_WORDS` window until item
/// 1538's build moved the word past it, to run680's.
pub(crate) fn sahara_coverage_frame_12_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run677",
        &[(RUN677, WIDENING_SAHARA_COVERAGE_FRAME_12.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_12,
        1,
        &[13],
        true,
    )
}

#[test]
fn run677_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_12_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **395** on the tree at base 0edf81c8, before the wholesale oil claim
    // walked a land region's cells in order (`docs/AI.md` §147): 67 stand on
    // block 6 (the control's set: both leaders' `form`, the blank `SITE`
    // slots' `reg`, the city fields, the pools, `scouts`), and what the
    // build left is leader 1's `known_rares` on block 8 and leader 0's
    // `production_step` on 201, the control's too. **69.**
    pin_eq!(w.firsts.len(), 69, "initial run677 baseline");
    // **The word's value diff, block 12** (item 1538): the Oil Well `1/2013`
    // (`TypeIndex` 421, `OILWELL`) stood at (34944, 10368) in ours — the
    // patch at cell (45, 13) — and at (44928, 8832) in theirs, cell (58, 11):
    // two patches tied at score 108 from the anchor (53, 22), the list's
    // order deciding which. It agrees in every compared field, and so do
    // the builders' orders and paths (`1/1`, `1/2`, `1/6`).
    assert!(
        w.firsts.keys().all(|(who, b, _)| (*who, *b) != (1, 2013)),
        "1/2013 agrees in every compared field"
    );
    for o in [1, 2, 6] {
        assert!(
            w.firsts
                .keys()
                .all(|(who, u, f)| (*who, *u) != (1, o) || !f.starts_with("order")),
            "1/{o}'s order stack agrees on every block"
        );
    }
    // **What parts first past the standing block**: leader 1's `known_rares`
    // on block 8 (ours 0, theirs 1), read by `civilian_value`.
    let first = w.firsts.values().map(|(f, _)| *f).filter(|f| *f > 6).min();
    pin_eq!(first, Some(8), "the first parting past the standing block");
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:known_rares".to_string()))
            .map(|(f, _)| *f),
        Some(8),
        "leader 1's known rares part on block 8"
    );
}

/// run680, item 1549: the lobby's blocks 713..969 at the long's detail —
/// the word 720's block 721 with eight before it and 248 after.
pub(crate) const RUN680: &str = "gamelog-run680-greatsahara-persian-alltech-window-713-969.txt";

/// The window: block 713 through 969; the word 720's own block is 721.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_720: (i64, i64) = (713, 969);

/// **The coverage lobby's word's window** (`AI_WORDS`' `Third map` row for
/// `GreatSaharaPersianAllTech`): run680 walked from run675's start with the
/// recorder on.
pub(crate) fn sahara_coverage_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run680",
        &[(RUN680, WIDENING_SAHARA_COVERAGE_FRAME_720.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_720,
        1,
        &[721],
        true,
    )
}

#[test]
fn run680_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    eprintln!("RUN680 firsts {}", w.firsts.len());
}
