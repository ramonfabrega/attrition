//! **The battery's third word** (DECISIONS 72, 83; item 3314): the
//! held-out battery's lowest game, promoted by item 2904's and 3107's
//! recipe. East Indies (`MAP_STYLE 18`), human Nubians (4) against **the
//! Romans (6)** at Difficulty 2, seed 12345 — `docs/AI.md` §243's fifth
//! lobby, scored by its first try at 576 of 18,620 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run899 (a
//! `DUMP_ALL` start, blocks 0..36) and run899b (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the word's
//! window is run956's (`docs/AI.md` §263). `GreatSaharaSpanishTier3` parts
//! on 576 too and stays held out: it is never debugged against.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const ROMAN: Lobby = Lobby {
    name: "EastIndiesRomanTier2",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 2,
    start: "gamelog-run899-eastindies-roman-diff2-start.txt",
    long: (
        "gamelog-run899b-eastindies-roman-diff2-24k-trace.txt",
        "rontrace-run899b.log",
    ),
    length: 18620,
    // Item 2975: the battery's first try, count and sequence both at 576
    // (ours 28 draws, theirs 89, index 0: ours `Animal::think_bird+0x82`,
    // theirs `Leader::produce_building+0xc99`). Item 3314 promoted it to
    // the AI word (`docs/AI.md` §263).
    count: 576,
    sequence: 576,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const ROMAN_WORD: i64 = if ROMAN.count < ROMAN.sequence {
    ROMAN.count
} else {
    ROMAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn roman_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(ROMAN.start, ROMAN.long) else {
        return;
    };
    eprintln!(
        "ROMAN {}: count {} sequence {} of {}; {}",
        ROMAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, ROMAN.difficulty, "the lobby read back");
    pin_eq!(w.last, ROMAN.length, "the game's last frame");
    pin_eq!(w.count, ROMAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        ROMAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn roman_word_is_no_other_lobby() {
    let p = &ROMAN;
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
