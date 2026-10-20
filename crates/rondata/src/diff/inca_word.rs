//! **The battery's Inca word** (DECISIONS 72, 83, 84; item 3360): the
//! held-out battery's lowest open game on the measure of `3eb2c76c`,
//! promoted by item 3335's recipe. East Indies (`MAP_STYLE 18`), human
//! Nubians (4) against **the Inca (2)** at Difficulty 3, seed 12345 —
//! minted by item 3335 (a nation no dump held, a tier the map's battery
//! lacked), scored by its first try and promoted out of the battery here,
//! so it is a scored lobby from this item (`battery::SCORED_LOBBIES`) and
//! no longer held out. Captures: run969 (a `DUMP_ALL` start) and run969b
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks
//! only); the game does not end itself (`docs/AI.md` §281).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA: Lobby = Lobby {
    name: "EastIndiesIncaTier3",
    map_style: 18,
    ai_tribe: 2,
    difficulty: 3,
    start: "gamelog-run969-eastindies-inca-diff3-start.txt",
    long: (
        "gamelog-run969b-eastindies-inca-diff3-24k-trace.txt",
        "rontrace-run969b.log",
    ),
    length: 24000,
    // Item 3335's first try, held by the battery's measure to `3eb2c76c`:
    // count 7782, sequence 7782 (ours 3218 draws, theirs 3219, at 1: ours
    // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`). Item
    // 3360 promoted it to the AI word (`docs/AI.md` §281).
    count: 7782,
    sequence: 7782,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INCA_WORD: i64 = if INCA.count < INCA.sequence {
    INCA.count
} else {
    INCA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn inca_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INCA.start, INCA.long) else {
        return;
    };
    eprintln!(
        "INCA {}: count {} sequence {} of {}; {}",
        INCA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INCA.difficulty, "the lobby read back");
    pin_eq!(w.last, INCA.length, "the game's last frame");
    pin_eq!(w.count, INCA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INCA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn inca_word_is_no_other_lobby() {
    let p = &INCA;
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
