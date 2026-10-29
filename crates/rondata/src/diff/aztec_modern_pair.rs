//! **The Aztecs at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, item 3750): Great Lakes, `MAP_STYLE 14`,
//! human Nubians (4) against the **Aztecs (0)** at **Toughest (5)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 0, 5)`, is new: the Aztecs hold Great Sahara at 2 (`aztec_word`),
//! 3 (`aztec_alltech_pair`) and 5 (`aztec_medieval_pair`) and East Indies
//! at 5 (`aztec_ancient_pair`); the battery's one Aztec game is
//! `EastIndiesAztecTier1`, `(18, 0, 1)`.
//! Captures: run1428 (a `DUMP_ALL` start) and run1429 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §374.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AZTEC_MODERN: Pair = Pair {
    name: "GreatLakesAztecModern",
    map_style: 14,
    ai_tribe: 0,
    difficulty: 5,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1428-greatlakes-aztec-modern-start.txt",
    long: (
        "gamelog-run1429-greatlakes-aztec-modern-trace.txt",
        "rontrace-run1429.log",
    ),
    // The game ended itself on 3389 (closing block 3390).
    length: 3389,
    // Item 3750, first try: count 3313, sequence 3313 of 3389 — on 3313
    // ours 2 draws against theirs' 4, index 2 theirs'
    // `Ammo::do_damage+0xc59` (`docs/AI.md` §374).
    count: 3313,
    sequence: 3313,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AZTEC_MODERN_PAIR_WORD: i64 = if AZTEC_MODERN.count < AZTEC_MODERN.sequence {
    AZTEC_MODERN.count
} else {
    AZTEC_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn aztec_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AZTEC_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AZTEC_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn aztec_modern_pair_captures_say_their_lobby() {
    let p = &AZTEC_MODERN;
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
        "the Aztec Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3750; the battery's identity
/// test refuses it), once, and none of the battery's — its one Aztec game
/// is the held-out `EastIndiesAztecTier1`, `(18, 0, 1)` — and no other
/// pinned Aztec lobby's: `aztec_word`'s `(7, 0, 2)`, the AllTech pair's
/// `(7, 0, 3)` and the Medieval pair's `(7, 0, 5)`, Great Sahara, and the
/// Ancient-start pair's `(18, 0, 5)`, East Indies (item 3750).
#[test]
fn aztec_modern_pair_is_no_other_lobby() {
    let p = &AZTEC_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let aztecs: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        aztecs,
        [("EastIndiesAztecTier1", (18, 0, 1))],
        "the battery's Aztec games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Aztec lobby on this map at this difficulty"
        );
    }
    let w = &super::aztec_word::AZTEC;
    let t = &super::aztec_alltech_pair::AZTEC_ALLTECH;
    let m = &super::aztec_medieval_pair::AZTEC_MEDIEVAL;
    let a = &super::aztec_ancient_pair::AZTEC_ANCIENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (t.name, (t.map_style, t.ai_tribe, t.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Aztecs");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}
