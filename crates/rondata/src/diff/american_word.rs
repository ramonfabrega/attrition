//! **The battery's American word** (DECISIONS 72, 83, 84; item 3374): the
//! held-out battery's lowest open game on the measure of `13bebf18` once
//! item 3370 closed the East Indies Spanish and the Dutch went to item
//! 3369, promoted by item 3335's recipe. East Indies (`MAP_STYLE 18`),
//! human Nubians (4) against **the Americans (20)** at Difficulty 1, seed
//! 12345 — minted by item 3314 (DECISIONS 72, a nation no dump held, a tier
//! no lobby on the map had), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run955 (a
//! `DUMP_ALL` start) and run955b (the draw stream to 24000, `cover=0`, its
//! setup and closing blocks only); the game does not end itself
//! (`docs/AI.md` §287).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN: Lobby = Lobby {
    name: "EastIndiesAmericanTier1",
    map_style: 18,
    ai_tribe: 20,
    difficulty: 1,
    start: "gamelog-run955-eastindies-american-diff1-start.txt",
    long: (
        "gamelog-run955b-eastindies-american-diff1-24k-trace.txt",
        "rontrace-run955b.log",
    ),
    length: 24000,
    // Item 3314's lobby, held by the battery's measure to `13bebf18`:
    // count 9382, sequence 9382 (ours 14 draws, theirs 12, at 1: ours
    // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`).
    // Item 3374 promoted it to the AI word (`docs/AI.md` §287).
    count: 9382,
    sequence: 9382,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const AMERICAN_WORD: i64 = if AMERICAN.count < AMERICAN.sequence {
    AMERICAN.count
} else {
    AMERICAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn american_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(AMERICAN.start, AMERICAN.long) else {
        return;
    };
    eprintln!(
        "AMERICAN {}: count {} sequence {} of {}; {}",
        AMERICAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, AMERICAN.difficulty, "the lobby read back");
    pin_eq!(w.last, AMERICAN.length, "the game's last frame");
    pin_eq!(w.count, AMERICAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        AMERICAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn american_word_is_no_other_lobby() {
    let p = &AMERICAN;
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
