//! **The French at a held Medieval age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3717): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **French (10)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 10, 4)`, is new: the French hold this map at 5 (the third pair),
//! East Indies at 4 (`french_enlightenment_pair`) and 5 (the third pair),
//! and Great Sahara at 4 (`french_word`) and 5 (`french_classical_pair`);
//! the battery fields no French. Captures: run1392 (a `DUMP_ALL` start)
//! and run1393 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §364.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const FRENCH_MEDIEVAL: Pair = Pair {
    name: "GreatLakesFrenchMedieval",
    map_style: 14,
    ai_tribe: 10,
    difficulty: 4,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1392-greatlakes-french-medieval-start.txt",
    long: (
        "gamelog-run1393-greatlakes-french-medieval-trace.txt",
        "rontrace-run1393.log",
    ),
    // The game ended itself on 4664 (closing block 4665).
    length: 4664,
    count: 4664,
    sequence: 4664,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const FRENCH_MEDIEVAL_PAIR_WORD: i64 =
    if FRENCH_MEDIEVAL.count < FRENCH_MEDIEVAL.sequence {
        FRENCH_MEDIEVAL.count
    } else {
        FRENCH_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn french_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &FRENCH_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "FRENCH_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn french_medieval_pair_captures_say_their_lobby() {
    let p = &FRENCH_MEDIEVAL;
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
        "the French Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3717; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no French — and no other French lobby's: the third pair's
/// `(14, 10, 5)`, this map at another difficulty, and `(18, 10, 5)`, the
/// Enlightenment pair's `(18, 10, 4)`, the Classical pair's `(7, 10, 5)`
/// and the AI word `GreatSaharaFrenchTier4`'s `(7, 10, 4)`, other maps
/// (item 3717).
#[test]
fn french_medieval_pair_is_no_other_lobby() {
    let p = &FRENCH_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the French"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another French lobby on this map at this difficulty"
        );
    }
    let w = &super::french_word::FRENCH;
    let c = &super::french_classical_pair::FRENCH_CLASSICAL;
    let e = &super::french_enlightenment_pair::FRENCH_ENLIGHTENMENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        ("the third pair, East Indies", (18, 10, 5)),
        ("the third pair, Great Lakes", (14, 10, 5)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the French");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}
