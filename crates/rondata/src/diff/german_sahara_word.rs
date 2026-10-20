//! **The battery's German word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3378): the held-out battery's lowest open game on the measure of
//! `56f23ba5`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Germans (12)** at Difficulty 3, seed
//! 12345 — minted by item 3375 (entry 72's rule (2): a nation held only by a
//! closed game on another map), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::german_word`, which is the closed East Indies game at the same
//! difficulty. Captures: run1000 (a `DUMP_ALL` start) and run1000b (the draw
//! stream to the game's end, `cover=0`, its setup and closing blocks only);
//! the game ends itself on 19483 (`docs/AI.md` §289).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GERMAN_SAHARA: Lobby = Lobby {
    name: "GreatSaharaGermanTier3",
    map_style: 7,
    ai_tribe: 12,
    difficulty: 3,
    start: "gamelog-run1000-greatsahara-german-diff3-start.txt",
    long: (
        "gamelog-run1000b-greatsahara-german-diff3-24k-trace.txt",
        "rontrace-run1000b.log",
    ),
    length: 19483,
    // Item 3375's first try, held by the battery's measure to `56f23ba5`:
    // count 13585, sequence 13582 (ours 64 draws, theirs 64, at 0: ours
    // `Leader::use_market+0x1ed`, theirs
    // `Leader::produce_building+0xc99`). Item 3378 promoted it to the AI
    // word (`docs/AI.md` §289), and moved it 13582 -> 19483 (+5901), the
    // game's end: `GERMAN_INDUSTRY_DISCOUNT` on `get_cost`'s industry lines
    // (run1003's block 13579: who=1's Carpentry (609) at `MAKE[1].val`
    // 30000 ours, 120000 theirs; agreeing after).
    count: 19483,
    sequence: 19483,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const GERMAN_SAHARA_WORD: i64 = if GERMAN_SAHARA.count < GERMAN_SAHARA.sequence {
    GERMAN_SAHARA.count
} else {
    GERMAN_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn german_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(GERMAN_SAHARA.start, GERMAN_SAHARA.long) else {
        return;
    };
    eprintln!(
        "GERMAN_SAHARA {}: count {} sequence {} of {}; {}",
        GERMAN_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        GERMAN_SAHARA.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, GERMAN_SAHARA.length, "the game's last frame");
    pin_eq!(
        w.count,
        GERMAN_SAHARA.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        GERMAN_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn german_sahara_word_is_no_other_lobby() {
    let p = &GERMAN_SAHARA;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
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
    }
}

/// run1003: the word 13582's window, blocks 13576..13832 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3378). run1000 dumps blocks 0..36 and
/// run1000b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN1003: &str = "gamelog-run1003-greatsahara-german-diff3-window-13576-13833.txt";

/// The window: block 13576 through 13832; the word 13582's own block is
/// 13583.
pub(crate) const WIDENING_GERMAN_SAHARA_FRAME_13582: (i64, i64) = (13576, 13832);

pub(crate) fn german_sahara_frame_13582_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_SAHARA.start],
        true,
        GERMAN_SAHARA.long,
        "run1003",
        &[(RUN1003, WIDENING_GERMAN_SAHARA_FRAME_13582.0)],
        WIDENING_GERMAN_SAHARA_FRAME_13582,
        1,
        &[13583],
        true,
    )
}

#[test]
fn run1003_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_sahara_frame_13582_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3378: 1039 keys on the first try. The window opens apart on the
    // standing family on 13576 (`form`, the leaders' `SITE[·].reg`,
    // `aggression[1]`, the army pools and `group:N.held` with no
    // `GROUPDATA` printed); then, on block 13579, who=1's make list: slot 1,
    // Carpentry (609, cat 8, escrowed), `val` 30000 ours, 120000 theirs —
    // ours dropped it under the Crossbowmen (172) on 13581, theirs bought it
    // and a Farm (417) on 13582. Built (`GERMAN_INDUSTRY_DISCOUNT` on
    // `get_cost`'s industry lines): 1039 -> 26, the standing family alone.
    pin_eq!(w.firsts.len(), 26, "keys that part in the window");
}

/// The closing state's window: run1000b's closing block alone, the long's
/// last frame 19483 writing it (item 3378).
pub(crate) const WIDENING_GERMAN_SAHARA_CLOSING: (i64, i64) = (19482, 19485);

/// **The lobby's closing state** (item 3378, DECISIONS 55): run1000b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 19483.
#[test]
fn run1000b_german_sahara_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        crate::testenv::dump(GERMAN_SAHARA.long.0),
        crate::testenv::dump(GERMAN_SAHARA.start),
        trace(GERMAN_SAHARA.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
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
        "German Sahara closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 19484, "the closing block");
    pin_eq!(e.compared, 148, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3378, the first walk (`GERMAN_INDUSTRY_DISCOUNT`, the word at
    // 19483): 148 units compared on block 19484, none apart, no building
    // or city field-row, no closing field.
}
