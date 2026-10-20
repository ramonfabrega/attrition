//! **The battery's Inca word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3385): the held-out battery's lowest open game on the measure of
//! `60438302`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Inca (2)** at Difficulty 3, seed 12345 —
//! minted by item 3383 (entry 72's rule (2): a nation held only by a closed
//! game on another map), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::inca_word`, which is the closed East Indies game. Captures:
//! run1006 (a `DUMP_ALL` start) and run1006b (the draw stream to the game's
//! end, `cover=0`, its setup and closing blocks only); the game ends itself
//! on 15632 (`docs/AI.md` §293).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA_SAHARA: Lobby = Lobby {
    name: "GreatSaharaIncaTier3",
    map_style: 7,
    ai_tribe: 2,
    difficulty: 3,
    start: "gamelog-run1006-greatsahara-inca-diff3-start.txt",
    long: (
        "gamelog-run1006b-greatsahara-inca-diff3-24k-trace.txt",
        "rontrace-run1006b.log",
    ),
    length: 15632,
    // Item 3383's first try, held by the battery's measure to `60438302`:
    // count 13127, sequence 13126 (ours 10 draws, theirs 10, at 8: ours
    // `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`). Item
    // 3385 promoted it to the AI word (`docs/AI.md` §293).
    count: 13127,
    sequence: 13126,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INCA_SAHARA_WORD: i64 = if INCA_SAHARA.count < INCA_SAHARA.sequence {
    INCA_SAHARA.count
} else {
    INCA_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn inca_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INCA_SAHARA.start, INCA_SAHARA.long) else {
        return;
    };
    eprintln!(
        "INCA_SAHARA {}: count {} sequence {} of {}; {}",
        INCA_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INCA_SAHARA.difficulty, "the lobby read back");
    pin_eq!(w.last, INCA_SAHARA.length, "the game's last frame");
    pin_eq!(w.count, INCA_SAHARA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INCA_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn inca_sahara_word_is_no_other_lobby() {
    let p = &INCA_SAHARA;
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
