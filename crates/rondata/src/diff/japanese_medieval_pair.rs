//! **The Japanese at a held Medieval age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, item 3664): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Japanese (15)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 15, 5)`, is new: the Japanese lobbies are the Industrial and
//! Ancient pairs' `(18, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and the
//! AI word `GreatSaharaJapaneseTier1`'s `(7, 15, 1)`, and the battery
//! fields no Japanese. Captures: run1328 (a `DUMP_ALL` start) and run1329
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §344.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const JAPANESE_MEDIEVAL: Pair = Pair {
    name: "GreatLakesJapaneseMedieval",
    map_style: 14,
    ai_tribe: 15,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1328-greatlakes-japanese-medieval-start.txt",
    long: (
        "gamelog-run1329-greatlakes-japanese-medieval-trace.txt",
        "rontrace-run1329.log",
    ),
    // The game ended itself on 5140 (closing block 5141).
    length: 5140,
    // Item 3652, first try: frame 10937, by the count and the sequence —
    // ours 34 draws against 36, parting at index 28, ours
    // `Farms::inc_time+0x1ae`, theirs `Ammo::do_damage+0xc59`
    // (`docs/AI.md` §343).
    count: 10937,
    sequence: 10937,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_MEDIEVAL_PAIR_WORD: i64 =
    if JAPANESE_MEDIEVAL.count < JAPANESE_MEDIEVAL.sequence {
        JAPANESE_MEDIEVAL.count
    } else {
        JAPANESE_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 1, and the long is scoreable.
#[test]
fn japanese_medieval_pair_captures_say_their_lobby() {
    let p = &JAPANESE_MEDIEVAL;
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
        "the Japanese Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3664; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Japanese; and no other Japanese lobby's: the Industrial and Ancient
/// pairs' `(18, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and the AI word
/// `GreatSaharaJapaneseTier1`'s `(7, 15, 1)` are another map (item 3664).
#[test]
fn japanese_medieval_pair_is_no_other_lobby() {
    let p = &JAPANESE_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Japanese"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Japanese lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL,
        &super::japanese_ancient_pair::JAPANESE_ANCIENT,
        &super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    let w = &super::japanese_word::JAPANESE;
    assert_eq!(w.ai_tribe, p.ai_tribe);
    assert_ne!((w.map_style, w.ai_tribe, w.difficulty), triple);
}
