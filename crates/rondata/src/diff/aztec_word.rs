//! **The battery's Aztec word** (DECISIONS 72, 83, 84; item 3361): the
//! held-out battery's next open game once item 3359 closed the French and
//! the Inca went to item 3360, promoted by item 3335's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Aztecs (0)** at
//! Difficulty 2, seed 12345 — minted by item 2975 (DECISIONS 77 (ii), a
//! nation no dump held), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run895 (a
//! `DUMP_ALL` start) and run895b (the draw stream to the game's end,
//! `cover=0`, its setup and closing blocks only); the game ends itself on
//! 17554 (`docs/AI.md` §282).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AZTEC: Lobby = Lobby {
    name: "GreatSaharaAztecTier2",
    map_style: 7,
    ai_tribe: 0,
    difficulty: 2,
    start: "gamelog-run895-greatsahara-aztec-diff2-start.txt",
    long: (
        "gamelog-run895b-greatsahara-aztec-diff2-24k-trace.txt",
        "rontrace-run895b.log",
    ),
    length: 17554,
    // Item 2975's lobby, held by the battery's measure to `3b6c4a2a`:
    // count 9130, sequence 9130 (ours 90 draws, theirs 87, at 23: ours
    // `Unit::think_scout+0x436`, theirs `Unit::think_scout+0x64c`). Item
    // 3361 promoted it to the AI word (`docs/AI.md` §282).
    count: 9130,
    sequence: 9130,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AZTEC_WORD: i64 = if AZTEC.count < AZTEC.sequence {
    AZTEC.count
} else {
    AZTEC.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn aztec_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AZTEC.start, AZTEC.long) else {
        return;
    };
    eprintln!(
        "AZTEC {}: count {} sequence {} of {}; {}",
        AZTEC.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, AZTEC.difficulty, "the lobby read back");
    pin_eq!(w.last, AZTEC.length, "the game's last frame");
    pin_eq!(w.count, AZTEC.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        AZTEC.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn aztec_word_is_no_other_lobby() {
    let p = &AZTEC;
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
