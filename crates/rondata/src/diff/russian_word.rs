//! **The battery's Russian word** (DECISIONS 72, 83; item 3342): the
//! held-out battery's lowest open game once item 3340 closed the Indians
//! (the Maya being item 3338's), promoted by item 3335's recipe. Great
//! Lakes (`MAP_STYLE 14`), human Nubians (4) against **the Russians (13)**
//! at Difficulty 2, seed 12345 — one of item 1465's three first lobbies
//! (`docs/AI.md` §125), scored by its first try and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run647 (a
//! `DUMP_ALL` start, blocks 0..36) and run644 (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the game
//! ends itself on 7303. The word's window is run976's (`docs/AI.md` §275).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const RUSSIAN: Lobby = Lobby {
    name: "GreatLakesRussianTier2",
    map_style: 14,
    ai_tribe: 13,
    difficulty: 2,
    start: "gamelog-run647-greatlakes-russian-diff2-start.txt",
    long: (
        "gamelog-run644-greatlakes-russian-diff2-24k-trace.txt",
        "rontrace-run644.log",
    ),
    length: 7303,
    // Item 1465's first try, held by the battery's measure to `826e0b6c`:
    // count 6566, sequence 6563 (ours 11 draws, theirs 11, at 4: ours
    // `Guy::set_anim+0xf2f < Guy::move+0x166`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`). Item 3342 promoted
    // it to the AI word (`docs/AI.md` §275).
    count: 6566,
    sequence: 6563,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const RUSSIAN_WORD: i64 = if RUSSIAN.count < RUSSIAN.sequence {
    RUSSIAN.count
} else {
    RUSSIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn russian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(RUSSIAN.start, RUSSIAN.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN {}: count {} sequence {} of {}; {}",
        RUSSIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, RUSSIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, RUSSIAN.length, "the game's last frame");
    pin_eq!(w.count, RUSSIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        RUSSIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn russian_word_is_no_other_lobby() {
    let p = &RUSSIAN;
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
