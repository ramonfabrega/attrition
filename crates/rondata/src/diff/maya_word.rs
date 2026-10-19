//! **The battery's Maya word** (DECISIONS 72, 83; item 3338): the held-out
//! battery's lowest open game once item 3336 closed the Egyptians, promoted
//! by item 2904's, 3107's, 3314's, 3318's, 3326's, 3331's and 3332's
//! recipe. Great Sahara (`MAP_STYLE 7`), human Nubians (4) against **the
//! Maya (1)** at Difficulty 4, seed 12345 — the lobby item 3332 minted
//! (`docs/AI.md` §125, §269), scored by its first try at 776 of 13,086 and
//! promoted out of the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run966 (a
//! `DUMP_ALL` start) and run966b (the draw stream to the game's end on
//! 13086, `cover=0`, its setup and closing blocks only); the word's window
//! is `docs/AI.md` §273's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const MAYA: Lobby = Lobby {
    name: "GreatSaharaMayaTier4",
    map_style: 7,
    ai_tribe: 1,
    difficulty: 4,
    start: "gamelog-run966-greatsahara-maya-diff4-start.txt",
    long: (
        "gamelog-run966b-greatsahara-maya-diff4-24k-trace.txt",
        "rontrace-run966b.log",
    ),
    length: 13086,
    // Item 3332: the battery's first try, count and sequence both at 776:
    // ours 34 draws, theirs 75, at 0 ours `Animal::think_bird+0x82`, theirs
    // `Leader::produce_building+0xc99`. Item 3338 promoted it to the AI word.
    count: 776,
    sequence: 776,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const MAYA_WORD: i64 = if MAYA.count < MAYA.sequence {
    MAYA.count
} else {
    MAYA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn maya_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(MAYA.start, MAYA.long) else {
        return;
    };
    eprintln!(
        "MAYA {}: count {} sequence {} of {}; {}",
        MAYA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, MAYA.difficulty, "the lobby read back");
    pin_eq!(w.last, MAYA.length, "the game's last frame");
    pin_eq!(w.count, MAYA.count, "the draw count's first parting");
    pin_eq!(w.sequence, MAYA.sequence, "the draw sequence's first parting");
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn maya_word_is_no_other_lobby() {
    let p = &MAYA;
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
