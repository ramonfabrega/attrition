//! **The battery's East Indies Spanish word** (DECISIONS 72, 83, 84; item
//! 3367): the held-out battery's lowest open game on the measure of
//! `9e63ec11`, promoted by item 3335's recipe. East Indies (`MAP_STYLE
//! 18`), human Nubians (4) against **the Spanish (9)** at Difficulty 3,
//! seed 12345 — minted by item 3360 under entry 72's rule (2), scored by
//! its first try and promoted out of the battery here, so it is a scored
//! lobby from this item (`battery::SCORED_LOBBIES`) and no longer held
//! out. Captures: run986 (a `DUMP_ALL` start) and run986b (the draw
//! stream to 24000, `cover=0`, its setup and closing blocks only); the
//! game does not end itself (`docs/AI.md` §284). `diff::spanish_word` is
//! the nation's other game, the closed Great Sahara one; the two are kept
//! apart.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_EAST: Lobby = Lobby {
    name: "EastIndiesSpanishTier3",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 3,
    start: "gamelog-run986-eastindies-spanish-diff3-start.txt",
    long: (
        "gamelog-run986b-eastindies-spanish-diff3-24k-trace.txt",
        "rontrace-run986b.log",
    ),
    length: 24000,
    // Item 3360's first try, held by the battery's measure to `9e63ec11`:
    // count 9028, sequence 9028 (ours 2 draws, theirs 12, at 0: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`). Item 3367 promoted it to
    // the AI word (`docs/AI.md` §284).
    count: 9028,
    sequence: 9028,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_EAST_WORD: i64 = if SPANISH_EAST.count < SPANISH_EAST.sequence {
    SPANISH_EAST.count
} else {
    SPANISH_EAST.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_east_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH_EAST.start, SPANISH_EAST.long) else {
        return;
    };
    eprintln!(
        "SPANISH_EAST {}: count {} sequence {} of {}; {}",
        SPANISH_EAST.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SPANISH_EAST.difficulty, "the lobby read back");
    pin_eq!(w.last, SPANISH_EAST.length, "the game's last frame");
    pin_eq!(
        w.count,
        SPANISH_EAST.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SPANISH_EAST.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_east_word_is_no_other_lobby() {
    let p = &SPANISH_EAST;
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
