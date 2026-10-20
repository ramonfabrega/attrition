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
    // word (`docs/AI.md` §289).
    count: 13585,
    sequence: 13582,
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
