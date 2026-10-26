//! **The Persians at a held Classical age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, item 3652): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Persians (23)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical:
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`. Its triple,
//! `(14, 23, 5)`, is new: the Persian lobbies are the coverage pair's and
//! `persian_medieval_pair`'s `(18, 23, 5)`, Great Sahara's `(7, 23, 5)`
//! and the AI word `GreatLakesPersianTier2`'s `(14, 23, 2)`, and the
//! battery fields no Persians. The first stand-up on Great Lakes on the
//! `Late ages` line. Captures: run1317 (a `DUMP_ALL` start) and run1318
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §343.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const PERSIAN_CLASSICAL: Pair = Pair {
    name: "GreatLakesPersianClassical",
    map_style: 14,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1317-greatlakes-persian-classical-start.txt",
    long: (
        "gamelog-run1318-greatlakes-persian-classical-trace.txt",
        "rontrace-run1318.log",
    ),
    // The game ended itself on 11633 (closing block 11634).
    length: 11633,
    // Item 3652, first try: frame 10937, by the count and the sequence —
    // ours 34 draws against 36, parting at index 28, ours
    // `Farms::inc_time+0x1ae`, theirs `Ammo::do_damage+0xc59`
    // (`docs/AI.md` §343).
    count: 10937,
    sequence: 10937,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_CLASSICAL_PAIR_WORD: i64 =
    if PERSIAN_CLASSICAL.count < PERSIAN_CLASSICAL.sequence {
        PERSIAN_CLASSICAL.count
    } else {
        PERSIAN_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn persian_classical_pair_captures_say_their_lobby() {
    let p = &PERSIAN_CLASSICAL;
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
        "the Persian Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3652; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Persians; and no other Persian lobby's: the coverage and Medieval
/// pairs' `(18, 23, 5)`, Great Sahara's `(7, 23, 5)` and the AI word
/// `GreatLakesPersianTier2`'s `(14, 23, 2)` are another map or another
/// difficulty (item 3652).
#[test]
fn persian_classical_pair_is_no_other_lobby() {
    let p = &PERSIAN_CLASSICAL;
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
        "the battery fields the Persians"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Persian lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::persian_medieval_pair::PERSIAN_MEDIEVAL,
        &super::coverage_pair::COVERAGE,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
}

/// **The long's stride, walked back from the word 10937** (item 3652):
/// only run1318's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_PERSIAN_CLASSICAL_STRIDE: (i64, i64) = (101, 10901);

pub(crate) fn persian_classical_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_CLASSICAL.start],
        true,
        PERSIAN_CLASSICAL.long,
        "run1318 stride",
        &[(
            PERSIAN_CLASSICAL.long.0,
            WIDENING_PERSIAN_CLASSICAL_STRIDE.0,
        )],
        WIDENING_PERSIAN_CLASSICAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn persian_classical_pair_the_long_s_stride_walks_back_from_10937() {
    let _pins = Pins::hold();
    let Some(w) = persian_classical_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 109, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3652.
    pin_eq!(w.firsts.len(), 143, "the stride's baseline");
}
