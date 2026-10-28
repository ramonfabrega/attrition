//! **The Spanish at a held Modern age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3718): East Indies, `MAP_STYLE 18`, human
//! Nubians (4) against the **Spanish (9)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(18, 9, 4)`, is new: the Spanish hold this map at 3
//! (`spanish_east_word`) and 5 (`spanish_gunpowder_pair`), Great Lakes at 4
//! (`spanish_lakes_word`) and 5 (`spanish_industrial_pair`), and Great
//! Sahara at 3 (`spanish_word`) and 5 (`spanish_classical_pair`); the
//! battery fields no Spanish. Captures: run1394 (a `DUMP_ALL` start) and
//! run1395 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §365.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const SPANISH_MODERN: Pair = Pair {
    name: "EastIndiesSpanishModern",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1394-eastindies-spanish-modern-start.txt",
    long: (
        "gamelog-run1395-eastindies-spanish-modern-trace.txt",
        "rontrace-run1395.log",
    ),
    // The game ended itself on 4081 (closing block 4082).
    length: 4081,
    // Item 3718, first try: 3770 of 4081 by the count and the sequence —
    // ours 3212 draws against 3216, index 0 ours' `Guy::set_anim+0x97a <
    // Guy::do_turn+0x4a < Unit::move_step+0x389` against theirs'
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d` (`docs/AI.md` §365).
    count: 3770,
    sequence: 3770,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_MODERN_PAIR_WORD: i64 =
    if SPANISH_MODERN.count < SPANISH_MODERN.sequence {
        SPANISH_MODERN.count
    } else {
        SPANISH_MODERN.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn spanish_modern_pair_captures_say_their_lobby() {
    let p = &SPANISH_MODERN;
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
        "the Spanish Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3718; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Spanish — and no other Spanish lobby's: `spanish_east_word`'s
/// `(18, 9, 3)` and `spanish_gunpowder_pair`'s `(18, 9, 5)`, this map at
/// other difficulties, `spanish_lakes_word`'s `(14, 9, 4)`,
/// `spanish_industrial_pair`'s `(14, 9, 5)`, `spanish_word`'s `(7, 9, 3)`
/// and `spanish_classical_pair`'s `(7, 9, 5)`, other maps (item 3718).
#[test]
fn spanish_modern_pair_is_no_other_lobby() {
    let p = &SPANISH_MODERN;
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
        "the battery fields the Spanish"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Spanish lobby on this map at this difficulty"
        );
    }
    let w = &super::spanish_word::SPANISH;
    let k = &super::spanish_lakes_word::SPANISH_LAKES;
    let e = &super::spanish_east_word::SPANISH_EAST;
    let c = &super::spanish_classical_pair::SPANISH_CLASSICAL;
    let g = &super::spanish_gunpowder_pair::SPANISH_GUNPOWDER;
    let i = &super::spanish_industrial_pair::SPANISH_INDUSTRIAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Spanish");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}
