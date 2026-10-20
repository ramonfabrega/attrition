//! **The battery's Turkish word** (DECISIONS 72, 83, 84; item 3380): the
//! held-out battery's second open game on the measure of `56f23ba5` (the
//! Germans being item 3378's), promoted by item 3335's recipe. Great Lakes
//! (`MAP_STYLE 14`), human Nubians (4) against **the Turks (8)** at
//! Difficulty 4, seed 12345 — minted by item 2975, scored by its first try
//! and promoted out of the battery here, so it is a scored lobby from this
//! item (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run898
//! (a `DUMP_ALL` start) and run898b (the draw stream to the game's end,
//! `cover=0`, its setup and closing blocks only); the game ends itself on
//! 15503 (`docs/AI.md` §290).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const TURKISH: Lobby = Lobby {
    name: "GreatLakesTurkishTier4",
    map_style: 14,
    ai_tribe: 8,
    difficulty: 4,
    start: "gamelog-run898-greatlakes-turkish-diff4-start.txt",
    long: (
        "gamelog-run898b-greatlakes-turkish-diff4-24k-trace.txt",
        "rontrace-run898b.log",
    ),
    length: 15503,
    // Item 2975's lobby: 1440 -> 6982 on item 3160 (`get_cost`'s
    // `TURK_CITIZEN` and `WOOL_CITIZENS_COST` arms), 6982 -> 14991 on the
    // forty-fifth tranche's fixes made for other words, re-pinned by the
    // forty-sixth pass and held by the battery's measure to `56f23ba5`:
    // count 14991, sequence 14991 (ours 20 draws, theirs 23, at 12: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`). Item 3380
    // promoted it to the AI word (`docs/AI.md` §290).
    count: 14991,
    sequence: 14991,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const TURKISH_WORD: i64 = if TURKISH.count < TURKISH.sequence {
    TURKISH.count
} else {
    TURKISH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn turkish_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(TURKISH.start, TURKISH.long) else {
        return;
    };
    eprintln!(
        "TURKISH {}: count {} sequence {} of {}; {}",
        TURKISH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, TURKISH.difficulty, "the lobby read back");
    pin_eq!(w.last, TURKISH.length, "the game's last frame");
    pin_eq!(w.count, TURKISH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        TURKISH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn turkish_word_is_no_other_lobby() {
    let p = &TURKISH;
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
