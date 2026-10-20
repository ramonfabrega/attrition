//! **The battery's American word at Great Lakes** (DECISIONS 72, 83, 84;
//! item 3384): the held-out battery's lowest open game on the measure of
//! `689ddb67` (the Indians being item 3383's), promoted by item 3335's
//! recipe. Great Lakes (`MAP_STYLE 14`), human Nubians (4) against **the
//! Americans (20)** at Difficulty 4, seed 12345 — minted by item 3380 under
//! entry 72's rule (2), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. The name
//! `american_word` is the closed East Indies game's (item 3374). Captures:
//! run1004 (a `DUMP_ALL` start) and run1004b (the draw stream to the game's
//! end, `cover=0`, its setup and closing blocks only); the game ends itself
//! on 6152 (`docs/AI.md` §292).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN_LAKES: Lobby = Lobby {
    name: "GreatLakesAmericanTier4",
    map_style: 14,
    ai_tribe: 20,
    difficulty: 4,
    start: "gamelog-run1004-greatlakes-american-diff4-start.txt",
    long: (
        "gamelog-run1004b-greatlakes-american-diff4-24k-trace.txt",
        "rontrace-run1004b.log",
    ),
    length: 6152,
    // Item 3380's lobby, its first try: count 3176, sequence 3176 (ours 31
    // draws, theirs 36, at 0: ours `Animal::think_bird+0x82`, theirs
    // `Leader::compute_sites+0x4ac`). Item 3384 promoted it to the AI word
    // (`docs/AI.md` §292).
    count: 3176,
    sequence: 3176,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AMERICAN_LAKES_WORD: i64 = if AMERICAN_LAKES.count < AMERICAN_LAKES.sequence {
    AMERICAN_LAKES.count
} else {
    AMERICAN_LAKES.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn american_lakes_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AMERICAN_LAKES.start, AMERICAN_LAKES.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_LAKES {}: count {} sequence {} of {}; {}",
        AMERICAN_LAKES.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, AMERICAN_LAKES.difficulty, "the lobby read back");
    pin_eq!(w.last, AMERICAN_LAKES.length, "the game's last frame");
    pin_eq!(w.count, AMERICAN_LAKES.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        AMERICAN_LAKES.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn american_lakes_word_is_no_other_lobby() {
    let p = &AMERICAN_LAKES;
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
