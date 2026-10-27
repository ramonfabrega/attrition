//! **The Russians at a held Gunpowder age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, item 3670): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **Russians (13)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder:
//! `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`. Its triple,
//! `(14, 13, 5)`, is new: the Russian lobbies are `russian_modern_pair`'s
//! `(18, 13, 5)`, `russian_medieval_pair`'s `(7, 13, 5)`, the AI word
//! `russian_word`'s `GreatLakesRussianTier2`, `(14, 13, 2)`, another
//! difficulty, and the held-out battery's `GreatSaharaRussianTier4`,
//! `(7, 13, 4)`. Captures: run1339 (a `DUMP_ALL` start) and run1340 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §349.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const RUSSIAN_GUNPOWDER: Pair = Pair {
    name: "GreatLakesRussianGunpowder",
    map_style: 14,
    ai_tribe: 13,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1339-greatlakes-russian-gunpowder-start.txt",
    long: (
        "gamelog-run1340-greatlakes-russian-gunpowder-trace.txt",
        "rontrace-run1340.log",
    ),
    // The game ended itself on 5341 (closing block 5342).
    length: 5341,
    count: 0,
    sequence: 0,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const RUSSIAN_GUNPOWDER_PAIR_WORD: i64 =
    if RUSSIAN_GUNPOWDER.count < RUSSIAN_GUNPOWDER.sequence {
        RUSSIAN_GUNPOWDER.count
    } else {
        RUSSIAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn russian_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3, and the long is scoreable.
#[test]
fn russian_gunpowder_pair_captures_say_their_lobby() {
    let p = &RUSSIAN_GUNPOWDER;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Russian Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3670; the battery's identity
/// test refuses it), once, and none of the battery's: the held-out
/// `GreatSaharaRussianTier4` is `(7, 13, 4)`, another map and difficulty;
/// and no other Russian lobby's: `russian_word`'s `(14, 13, 2)` is this
/// map at another difficulty, and the Modern and Medieval pairs' `(18, 13,
/// 5)` and `(7, 13, 5)` other maps (item 3670).
#[test]
fn russian_gunpowder_pair_is_no_other_lobby() {
    let p = &RUSSIAN_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let russians: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        russians,
        [("GreatSaharaRussianTier4", 7, 4)],
        "the battery's Russian games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Russian lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::russian_modern_pair::RUSSIAN_MODERN,
        &super::russian_medieval_pair::RUSSIAN_MEDIEVAL,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    let w = &super::russian_word::RUSSIAN;
    assert_eq!(w.ai_tribe, p.ai_tribe);
    assert_ne!((w.map_style, w.ai_tribe, w.difficulty), triple);
}
