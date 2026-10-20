//! **The battery's French word** (DECISIONS 72, 83, 84; item 3347): the
//! held-out battery's second open game once item 3343 closed the Bantu,
//! promoted by item 3343's recipe (`diff::french_word`). Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the French (10)** at
//! Difficulty 4, seed 12345 — the lobby item 3343 minted under entry 72's
//! rule (2) (`docs/AI.md` §125, §276), scored by its first try at 13422 of
//! 16,764 and promoted out of the battery here, so it is a scored lobby
//! from this item (`battery::SCORED_LOBBIES`) and no longer held out.
//! Captures: run977 (a `DUMP_ALL` start) and run977b (the draw stream to the
//! game's end on 16764, `cover=0`, its setup and closing blocks only); the
//! word's window is `docs/AI.md` §278's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const FRENCH: Lobby = Lobby {
    name: "GreatSaharaFrenchTier4",
    map_style: 7,
    ai_tribe: 10,
    difficulty: 4,
    start: "gamelog-run977-greatsahara-french-diff4-start.txt",
    long: (
        "gamelog-run977b-greatsahara-french-diff4-24k-trace.txt",
        "rontrace-run977b.log",
    ),
    length: 16764,
    // Item 3343, the battery's first try, count and sequence both at
    // 13422: ours 14 draws, theirs 19, at 6 ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::init_real+0x52`. Item 3347
    // promoted it to the AI word.
    count: 13422,
    sequence: 13422,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const FRENCH_WORD: i64 = if FRENCH.count < FRENCH.sequence {
    FRENCH.count
} else {
    FRENCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn french_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(FRENCH.start, FRENCH.long) else {
        return;
    };
    eprintln!(
        "FRENCH {}: count {} sequence {} of {}; {}",
        FRENCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, FRENCH.difficulty, "the lobby read back");
    pin_eq!(w.last, FRENCH.length, "the game's last frame");
    pin_eq!(w.count, FRENCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        FRENCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn french_word_is_no_other_lobby() {
    let p = &FRENCH;
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
