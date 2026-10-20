//! **The battery's Persian word** (DECISIONS 72, 83, 84; item 3355): the
//! held-out battery's lowest open game once item 3345 closed the Lakota,
//! promoted by item 3335's recipe. Great Lakes (`MAP_STYLE 14`), human
//! Nubians (4) against **the Persians (23)** at Difficulty 2, seed 12345 —
//! minted by item 3345 under entry 72's rule (2) as amended by 84, scored
//! by its first try and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Not the coverage pair's lobbies (`diff::coverage_pair`, East
//! Indies, and `diff::sahara_coverage`, Great Sahara, both at Toughest with
//! every tech): the nation is the same, the game is not. Captures: run979
//! (a `DUMP_ALL` start, blocks 0..36) and run979b (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the game
//! ends itself on 7105 (`docs/AI.md` §279).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const PERSIAN: Lobby = Lobby {
    name: "GreatLakesPersianTier2",
    map_style: 14,
    ai_tribe: 23,
    difficulty: 2,
    start: "gamelog-run979-greatlakes-persian-diff2-start.txt",
    long: (
        "gamelog-run979b-greatlakes-persian-diff2-24k-trace.txt",
        "rontrace-run979b.log",
    ),
    length: 7105,
    // Item 3345's first try, held by the battery's measure to `a98d672b`:
    // count 6046, sequence 6046 (ours 5 draws, theirs 4, at 1: ours
    // `Unit::fight+0x9b0`, theirs `Farms::inc_time+0x1ae`). Item 3355
    // promoted it to the AI word (`docs/AI.md` §279).
    count: 6046,
    sequence: 6046,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const PERSIAN_WORD: i64 = if PERSIAN.count < PERSIAN.sequence {
    PERSIAN.count
} else {
    PERSIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn persian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(PERSIAN.start, PERSIAN.long) else {
        return;
    };
    eprintln!(
        "PERSIAN {}: count {} sequence {} of {}; {}",
        PERSIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, PERSIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, PERSIAN.length, "the game's last frame");
    pin_eq!(w.count, PERSIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        PERSIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn persian_word_is_no_other_lobby() {
    let p = &PERSIAN;
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
