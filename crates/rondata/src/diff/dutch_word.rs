//! **The battery's fifth word** (DECISIONS 72, 83; item 3329): the
//! held-out battery's lowest game after the Greeks (item 3326's), promoted
//! by item 3314's and 3318's recipe. East Indies (`MAP_STYLE 18`), human
//! Nubians (4) against **the Dutch (22)** at Difficulty 4, seed 12345 —
//! the lobby item 3107 minted (`docs/AI.md` §246), scored by its first try
//! at 1567 of 24,000 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run915 (a `DUMP_ALL` start, blocks 0..36) and run917
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks
//! only); the game does not end itself. The word's windows are run963's
//! (`docs/AI.md` §267).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const DUTCH: Lobby = Lobby {
    name: "EastIndiesDutchTier4",
    map_style: 18,
    ai_tribe: 22,
    difficulty: 4,
    start: "gamelog-run915-eastindies-dutch-diff4-start.txt",
    long: (
        "gamelog-run917-eastindies-dutch-diff4-24k-trace.txt",
        "rontrace-run917.log",
    ),
    length: 24000,
    // Item 3107, the battery's first try: count and sequence both at 1567
    // (ours 6 draws, theirs 5, at index 2: ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Farms::inc_time+0x1ae`). Item 3329
    // promoted it to the AI word (`docs/AI.md` §267).
    count: 1567,
    sequence: 1567,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const DUTCH_WORD: i64 = if DUTCH.count < DUTCH.sequence {
    DUTCH.count
} else {
    DUTCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn dutch_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(DUTCH.start, DUTCH.long) else {
        return;
    };
    eprintln!(
        "DUTCH {}: count {} sequence {} of {}; {}",
        DUTCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, DUTCH.difficulty, "the lobby read back");
    pin_eq!(w.last, DUTCH.length, "the game's last frame");
    pin_eq!(w.count, DUTCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        DUTCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn dutch_word_is_no_other_lobby() {
    let p = &DUTCH;
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
