//! **The battery's Bantu word** (DECISIONS 72, 83; item 3343): the held-out
//! battery's lowest open game once item 3338 closed the Maya, promoted by
//! item 2904's, 3107's, 3314's, 3318's, 3326's, 3331's, 3332's and 3338's
//! recipe. Great Sahara (`MAP_STYLE 7`), human Nubians (4) against **the
//! Bantu (3)** at Difficulty 4, seed 12345 — the lobby item 3338 minted
//! (`docs/AI.md` §125, §273), scored by its first try at 4 of 23,627 and
//! promoted out of the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run972 (a
//! `DUMP_ALL` start) and run972b (the draw stream to the game's end on
//! 23627, `cover=0`, its setup and closing blocks only); the word's window
//! is `docs/AI.md` §276's.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BANTU: Lobby = Lobby {
    name: "GreatSaharaBantuTier4",
    map_style: 7,
    ai_tribe: 3,
    difficulty: 4,
    start: "gamelog-run972-greatsahara-bantu-diff4-start.txt",
    long: (
        "gamelog-run972b-greatsahara-bantu-diff4-24k-trace.txt",
        "rontrace-run972b.log",
    ),
    length: 23627,
    // Item 3338, the battery's first try, count and sequence both at 4:
    // ours 8 draws, theirs 6, at 0 ours `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`, theirs `Farms::inc_time+0x1ae`. Item 3343
    // promoted it to the AI word.
    count: 4,
    sequence: 4,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const BANTU_WORD: i64 = if BANTU.count < BANTU.sequence {
    BANTU.count
} else {
    BANTU.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn bantu_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(BANTU.start, BANTU.long) else {
        return;
    };
    eprintln!(
        "BANTU {}: count {} sequence {} of {}; {}",
        BANTU.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, BANTU.difficulty, "the lobby read back");
    pin_eq!(w.last, BANTU.length, "the game's last frame");
    pin_eq!(w.count, BANTU.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        BANTU.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn bantu_word_is_no_other_lobby() {
    let p = &BANTU;
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
