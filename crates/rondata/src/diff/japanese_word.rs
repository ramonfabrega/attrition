//! **The battery's Japanese word** (DECISIONS 72, 83; item 3331): the
//! held-out battery's lowest game once item 3326 closed the Greeks, promoted
//! by item 2904's, 3107's, 3314's, 3318's and 3326's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Japanese (15)** at
//! Difficulty 1, seed 12345 — the lobby item 3326 minted (`docs/AI.md`
//! §266), scored by its first try at 576 of 24,000 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run960 (a
//! `DUMP_ALL` start, blocks 0..36) and run960b (the draw stream to 24000,
//! `cover=0`, its setup and closing blocks only); the word's window is
//! run965's (`docs/AI.md` §268).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const JAPANESE: Lobby = Lobby {
    name: "GreatSaharaJapaneseTier1",
    map_style: 7,
    ai_tribe: 15,
    difficulty: 1,
    start: "gamelog-run960-greatsahara-japanese-diff1-start.txt",
    long: (
        "gamelog-run960b-greatsahara-japanese-diff1-24k-trace.txt",
        "rontrace-run960b.log",
    ),
    length: 24000,
    // Item 3326: the battery's first try, count and sequence both at 576;
    // ours 56 draws, theirs 35, at 4 ours `Leader::compute_sites+0x4ac`,
    // theirs `Animal::think_bird+0x82`. Item 3331 promoted it to the AI word.
    count: 576,
    sequence: 576,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const JAPANESE_WORD: i64 = if JAPANESE.count < JAPANESE.sequence {
    JAPANESE.count
} else {
    JAPANESE.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn japanese_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(JAPANESE.start, JAPANESE.long) else {
        return;
    };
    eprintln!(
        "JAPANESE {}: count {} sequence {} of {}; {}",
        JAPANESE.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, JAPANESE.difficulty, "the lobby read back");
    pin_eq!(w.last, JAPANESE.length, "the game's last frame");
    pin_eq!(w.count, JAPANESE.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        JAPANESE.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn japanese_word_is_no_other_lobby() {
    let p = &JAPANESE;
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
