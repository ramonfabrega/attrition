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
    // sequence. On 4501 ours spent 156 draws against theirs' 157, parting
    // at index 153 — ours `Farms::inc_time+0x1ae`, theirs
    // `Object::take_damage+0xe1` (`docs/AI.md` section 327). Item 3616:
    // 4501 → 5357, the Dromon's rounds leaving from piece 274's bays
    // (`sim::launch`); on 5357 ours spends 149 draws against theirs' 151,
    // parting at index 146 — ours `Farms::inc_time+0x1ae`, theirs
    // `Ammo::do_damage+0xc59`.
    count: 5357,
    sequence: 5357,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INCA_CLASSICAL_PAIR_WORD: i64 = if INCA_CLASSICAL.count < INCA_CLASSICAL.sequence {
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

/// run1260b: the word 4501's window, blocks 4495..4751 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 4502 and 250 after (item 3606); the long's stride prints units
/// every hundredth block (4501 and 4601) and no `LEADERS` past the start.
pub(crate) const RUN1260B: &str = "gamelog-run1260b-eastindies-inca-classical-window-4495-4751.txt";

/// The window: block 4495 through 4751; the word 4501's own block is 4502.
pub(crate) const WIDENING_INCA_CLASSICAL_FRAME_4501: (i64, i64) = (4495, 4751);

/// **The word 4501's window**: run1260b walked from run1259's start with
/// the recorder on (item 3606).
pub(crate) fn inca_classical_frame_4501_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_CLASSICAL.start],
        true,
        INCA_CLASSICAL.long,
        "run1260b",
        &[(RUN1260B, WIDENING_INCA_CLASSICAL_FRAME_4501.0)],
        WIDENING_INCA_CLASSICAL_FRAME_4501,
        1,
        &[4502],
        true,
    )
}

#[test]
fn inca_classical_pair_run1260b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_classical_frame_4501_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3606, the first walk: 754 keys. Block 4495 holds only the
    // standing families (every unit's `form`, who=1's groups' `held` and
    // pools, both leaders' `SITE[i].reg` 65 against 0 and who=0's
    // `aggression[1]`), and nothing else parts until the word's block
    // 4502. There the Farm `0/2004` (TypeIndex 417) takes its first wound
    // in theirs alone — `damage` 1 and `damage_frac` 5 against 0 — and its
    // city `0/2000` is raided (`raid_stamp` 4501, `city_flags` 0x2, 0x4
    // and 0x8), with both leaders' `treaties` 3 against ours' 1. The round
    // is the Dromon `1/24`'s (TypeIndex 324, sea, uid 54, on an
    // `EXPLORETOORDER`): theirs flies 12 frames from (2808, 6381, 52) to
    // (4319, 5864) and lands on 4501; ours flies 13 from (2808, 6568) to
    // (4327, 5881) (`RON_DEBUG_AMMO`), the same launch frame. Its next
    // round flies 12 on both sides, from (2809, 6079) in theirs and
    // (2808, 6088) in ours. From 4503 who=1's units' orders part (`1/15`,
    // `1/30`, `1/28`). Item 3616: 754 → 133. The Dromon's rounds leave
    // from piece 274's bays, its first from (2808, 6381) on both sides and
    // 12 frames long, so the Farm's wound on block 4502 agrees — `damage`
    // 1, `damage_frac` 5 — and with it the city's raid and the treaties.
    // What stands is the standing families alone: block 4495's 120 (every
    // unit's `form`, who=1's 23 groups' `held` and pools, `SITE[i].reg`
    // and `aggression[1]`), and later births' `form` and new groups'
    // `held` and pools.
    pin_eq!(w.firsts.len(), 133, "run1260b baseline");
}

/// run1269: the word 5357's window, blocks 5351..5607 at run1260b's
/// detail — six blocks before the word's block 5358 and 250 after (item
/// 3616); the long's stride prints units on 5301 and 5401 and no `AMMO`
/// past the start.
pub(crate) const RUN1269: &str = "gamelog-run1269-eastindies-inca-classical-window-5351-5607.txt";

/// The window: block 5351 through 5607; the word 5357's own block is 5358.
pub(crate) const WIDENING_INCA_CLASSICAL_FRAME_5357: (i64, i64) = (5351, 5607);

/// **The word 5357's window**: run1269 walked from run1259's start with
/// the recorder on (item 3616).
pub(crate) fn inca_classical_frame_5357_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_CLASSICAL.start],
        true,
        INCA_CLASSICAL.long,
        "run1269",
        &[(RUN1269, WIDENING_INCA_CLASSICAL_FRAME_5357.0)],
        WIDENING_INCA_CLASSICAL_FRAME_5357,
        1,
        &[5358],
        true,
    )
}

#[test]
fn inca_classical_pair_run1269_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_classical_frame_5357_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3616, the first walk: 1443 keys. Block 5351 already parts past
    // the standing families: the Scout `0/0` (TypeIndex 69, uid 7) a step
    // behind in ours — `pos` (8556, 7356) against (8578, 7330),
    // `damage_frame` 5344 against 5343 — and who=1's `free_peasants` and
    // `gatherers`, `1/58`'s facing. In `AMMO` the Archers `1/74`
    // (TypeIndex 171, piece 825)'s round at `0/0` leaves (7140, 8347) and
    // flies 15 frames in theirs, ending on 5357; ours leaves the figure's
    // own (7080, 8376) and flies 16 (`RON_DEBUG_AMMO`). On block 5357
    // `0/0`'s `g.angle[1]` parts (482541568 against 466354176), and on the
    // word's block 5358 `1/74`'s facing (638255104 against 635502592).
    // From 5371 who=1's army's orders part.
    pin_eq!(w.firsts.len(), 1443, "run1269 baseline");
}

/// **The long's stride between the two windows** (item 3616): run1260's
/// stride blocks 4801..5301, after run1260b's window and before run1269's.
pub(crate) const WIDENING_INCA_CLASSICAL_STRIDE_5357: (i64, i64) = (4801, 5301);

pub(crate) fn inca_classical_stride_5357_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_CLASSICAL.start],
        true,
        INCA_CLASSICAL.long,
        "run1260 stride to 5357",
        &[(INCA_CLASSICAL.long.0, WIDENING_INCA_CLASSICAL_STRIDE_5357.0)],
        WIDENING_INCA_CLASSICAL_STRIDE_5357,
        1,
        &[],
        true,
    )
}

#[test]
fn inca_classical_pair_the_long_s_stride_walks_back_from_5357() {
    let _pins = Pins::hold();
    let Some(w) = inca_classical_stride_5357_window() else {
        return;
    };
    pin_eq!(w.blocks, 6, "every stride block in the window");
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
    // Item 3616: 178 keys, the standing families alone (`form`,
    // `build:extra`, who=1's groups' `held` and pools); no unit row parts
    // through 5301. The state first parts in the gap 5302..5350, which no
    // dump holds: run1269's first block, 5351, already has the Scout `0/0`
    // a step behind.
    pin_eq!(w.firsts.len(), 178, "the stride's baseline");
}
