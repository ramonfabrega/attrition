//! **The battery's Egyptian word** (DECISIONS 72, 83; item 3332): the
//! held-out battery's lowest open game on the measure of `987c4c33`,
//! promoted by item 2904's, 3107's, 3314's, 3318's, 3326's and 3331's
//! recipe. Great Sahara (`MAP_STYLE 7`), human Nubians (4) against **the
//! Egyptians (7)** at Difficulty 4, seed 12345 — one of item 1465's first
//! three lobbies (`docs/AI.md` §125), scored by its first try at 2974 of
//! 20,380 and promoted out of the battery here, so it is a scored lobby from
//! this item (`battery::SCORED_LOBBIES`) and no longer held out. Captures:
//! run648 (a `DUMP_ALL` start) and run645 (the draw stream to the game's
//! end on 20380, `cover=0`, its setup and closing blocks only); the word's
//! window is run967's (`docs/AI.md` §269).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const EGYPTIAN: Lobby = Lobby {
    name: "GreatSaharaEgyptianTier4",
    map_style: 7,
    ai_tribe: 7,
    difficulty: 4,
    start: "gamelog-run648-greatsahara-egyptian-diff4-start.txt",
    long: (
        "gamelog-run645-greatsahara-egyptian-diff4-24k-trace.txt",
        "rontrace-run645.log",
    ),
    length: 20380,
    // Item 1465: the battery's first try; the measure on `987c4c33`, count
    // and sequence both at 2974: ours 9 draws, theirs 12, at 2 ours
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389`,
    // theirs `Guy::set_anim+0x104b`. Item 3332 promoted it to the AI word.
    count: 2974,
    sequence: 2974,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const EGYPTIAN_WORD: i64 = if EGYPTIAN.count < EGYPTIAN.sequence {
    EGYPTIAN.count
} else {
    EGYPTIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn egyptian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(EGYPTIAN.start, EGYPTIAN.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN {}: count {} sequence {} of {}; {}",
        EGYPTIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, EGYPTIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, EGYPTIAN.length, "the game's last frame");
    pin_eq!(w.count, EGYPTIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        EGYPTIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn egyptian_word_is_no_other_lobby() {
    let p = &EGYPTIAN;
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
