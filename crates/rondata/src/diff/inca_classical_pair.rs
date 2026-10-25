//! **The Inca at a held Classical age, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3606): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Inca (2)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical: `STARTING_TECHNOLOGY
//! 1`, `ENDING_TECHNOLOGY 1` — the Egyptian Classical pair's lobby with
//! the nation moved. Its triple, `(18, 2, 5)`, is new: the Inca's other
//! lobbies are the AI words `inca_word`'s `EastIndiesIncaTier3`, `(18, 2,
//! 3)` — another difficulty — and `inca_sahara_word`'s
//! `GreatSaharaIncaTier3`, `(7, 2, 3)` — another map and difficulty — and
//! no battery game fields the Inca. Captures: run1259 (a `DUMP_ALL` start)
//! and run1260 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §327.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA_CLASSICAL: Pair = Pair {
    name: "EastIndiesIncaClassical",
    map_style: 18,
    ai_tribe: 2,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1259-eastindies-inca-classical-start.txt",
    long: (
        "gamelog-run1260-eastindies-inca-classical-trace.txt",
        "rontrace-run1260.log",
    ),
    // The game ended itself on 6073 (closing block 6074).
    length: 6073,
    // Item 3606, first try: frame 4501 of 6073 by the count and the
    // sequence. On 4501 ours spends 156 draws against theirs' 157, parting
    // at index 153 — ours `Farms::inc_time+0x1ae`, theirs
    // `Object::take_damage+0xe1` (`docs/AI.md` section 327).
    count: 4501,
    sequence: 4501,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INCA_CLASSICAL_PAIR_WORD: i64 = if INCA_CLASSICAL.count < INCA_CLASSICAL.sequence
{
    INCA_CLASSICAL.count
} else {
    INCA_CLASSICAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn inca_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &INCA_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "INCA_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn inca_classical_pair_captures_say_their_lobby() {
    let p = &INCA_CLASSICAL;
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
        "the Inca Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3606; the battery's identity
/// test refuses it) and none of the battery's — no battery game fields
/// the Inca — nor either Inca AI word's: `EastIndiesIncaTier3` is `(18, 2,
/// 3)`, another difficulty, and `GreatSaharaIncaTier3` `(7, 2, 3)`.
#[test]
fn inca_classical_pair_is_no_other_lobby() {
    let p = &INCA_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "a battery game fields the Inca"
    );
    let east = &super::inca_word::INCA;
    assert_eq!(
        (east.name, east.map_style, east.ai_tribe, east.difficulty),
        ("EastIndiesIncaTier3", 18, 2, 3),
        "the East Indies Inca word is another difficulty"
    );
    let sahara = &super::inca_sahara_word::INCA_SAHARA;
    for (name, other) in [
        (east.name, (east.map_style, east.ai_tribe, east.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 4501** (item 3606):
/// only run1260's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_INCA_CLASSICAL_STRIDE: (i64, i64) = (101, 4501);

pub(crate) fn inca_classical_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_CLASSICAL.start],
        true,
        INCA_CLASSICAL.long,
        "run1260 stride",
        &[(INCA_CLASSICAL.long.0, WIDENING_INCA_CLASSICAL_STRIDE.0)],
        WIDENING_INCA_CLASSICAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn inca_classical_pair_the_long_s_stride_walks_back_from_4501() {
    let _pins = Pins::hold();
    let Some(w) = inca_classical_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 45, "every stride block in the window");
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
    // Item 3606: 172 keys, the standing families alone — every unit's
    // `form` (71) and `build:extra` (53), which the stride does not print,
    // and who=1's groups' `held` and pools (24 each). No unit row parts
    // through block 4501, the last before the word's block 4502.
    pin_eq!(w.firsts.len(), 172, "the stride's baseline");
}
