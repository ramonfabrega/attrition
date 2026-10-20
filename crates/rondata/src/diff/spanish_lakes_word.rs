//! **The battery's Spanish word at Great Lakes** (DECISIONS 72, 83, 84;
//! item 3390): the held-out battery's lowest open game on the measure of
//! `02005991` (the Inca at Great Sahara being item 3385's), promoted by item
//! 3335's recipe. Great Lakes (`MAP_STYLE 14`), human Nubians (4) against
//! **the Spanish (9)** at Difficulty 4, seed 12345 — minted by item 3384
//! under entry 72's rule (2), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. The names
//! `spanish_word` and `spanish_east_word` are the closed Great Sahara and
//! East Indies games' (items 3318, 3367). Captures: run1008 (a `DUMP_ALL`
//! start) and run1008b (the draw stream to the game's end, `cover=0`, its
//! setup and closing blocks only); the game ends itself on 14068
//! (`docs/AI.md` §294).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_LAKES: Lobby = Lobby {
    name: "GreatLakesSpanishTier4",
    map_style: 14,
    ai_tribe: 9,
    difficulty: 4,
    start: "gamelog-run1008-greatlakes-spanish-diff4-start.txt",
    long: (
        "gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt",
        "rontrace-run1008b.log",
    ),
    length: 14068,
    // Item 3384's lobby, its first try: count 8384, sequence 8384 (ours
    // 3266 draws, theirs 3271, at 3262: ours `Herd::process+0x17`, theirs
    // `PathFinder::calc_road_cost+0x46`). Item 3390 promoted it to the AI
    // word (`docs/AI.md` §294).
    count: 8384,
    sequence: 8384,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_LAKES_WORD: i64 = if SPANISH_LAKES.count < SPANISH_LAKES.sequence {
    SPANISH_LAKES.count
} else {
    SPANISH_LAKES.sequence
};
/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_lakes_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH_LAKES.start, SPANISH_LAKES.long) else {
        return;
    };
    eprintln!(
        "SPANISH_LAKES {}: count {} sequence {} of {}; {}",
        SPANISH_LAKES.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        SPANISH_LAKES.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, SPANISH_LAKES.length, "the game's last frame");
    pin_eq!(
        w.count,
        SPANISH_LAKES.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SPANISH_LAKES.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_lakes_word_is_no_other_lobby() {
    let p = &SPANISH_LAKES;
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
