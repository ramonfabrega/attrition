//! **The Chinese at a held Gunpowder age, East Indies** (DECISIONS 90,
//! 91, 93, 94, item 3625): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Chinese (14)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `chinese_gunpowder_pair`'s lobby with the
//! nation moved. Its triple, `(18, 14, 5)`, is new: the Chinese's other
//! lobbies are `chinese_industrial_pair`'s `GreatSaharaChineseIndustrial`,
//! `(7, 14, 3)`, and `chinese_medieval_pair`'s
//! `GreatSaharaChineseMedieval`, `(7, 14, 5)` — another map — and the
//! battery's one Chinese game is the held-out `GreatLakesChineseTier1`,
//! `(14, 14, 1)`, another map. Captures: run1282 (a `DUMP_ALL` start) and
//! run1283 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §333.
use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const CHINESE_GUNPOWDER: Pair = Pair {
    name: "EastIndiesChineseGunpowder",
    map_style: 18,
    ai_tribe: 14,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1282-eastindies-chinese-gunpowder-start.txt",
    long: (
        "gamelog-run1283-eastindies-chinese-gunpowder-trace.txt",
        "rontrace-run1283.log",
    ),
    // The game ended itself on 5568 (closing block 5569).
    length: 5568,
    // Item 3625, first try: frame 4884 of 5568 by the count and the
    // sequence. On 4884 ours spends 341 draws against theirs' 357, parting
    // at index 6 — ours `PathFinder::calc_road_cost+0x46`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` (`docs/AI.md` section
    // 333).
    //
    // Item 3637: **4884 → 5196**. `do_move` keeps the failed unit-grid
    // search's `retry` roll (the Heavy Fire Lance `1/215`, 6 on block 4862
    // on both sides now). On 5196 ours spends 19 draws against theirs' 20,
    // parting at index 1 — ours `Guy::set_anim+0x97a < Unit::do_guard+
    // 0x7f4` (`1/126`), theirs a second `< Unit::do_idle+0x7d`, a who=1
    // unit between `1/115` and `1/126`; the state parts first on block
    // 5133 at the human's `0/10`, wounded in theirs on 5132 (`docs/AI.md`
    // section 333).
    count: 5196,
    sequence: 5196,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const CHINESE_GUNPOWDER_PAIR_WORD: i64 =
    if CHINESE_GUNPOWDER.count < CHINESE_GUNPOWDER.sequence {
        CHINESE_GUNPOWDER.count
    } else {
        CHINESE_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn chinese_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &CHINESE_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "CHINESE_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn chinese_gunpowder_pair_captures_say_their_lobby() {
    let p = &CHINESE_GUNPOWDER;
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
        "the Chinese Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3625; the battery's identity
/// test refuses it) and none of the battery's: `(18, 14, 5)` is no held-out
/// game's — the battery's one Chinese game is `GreatLakesChineseTier1`,
/// `(14, 14, 1)` — and no other pinned pair's: the Chinese's other pairs
/// are Great Sahara's, `chinese_industrial_pair`'s `(7, 14, 3)` and
/// `chinese_medieval_pair`'s `(7, 14, 5)`, another map.
#[test]
fn chinese_gunpowder_pair_is_no_other_lobby() {
    let p = &CHINESE_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let chinese: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        chinese,
        [("GreatLakesChineseTier1", 14, 1)],
        "the battery's Chinese games"
    );
    let others = [
        &super::chinese_industrial_pair::CHINESE_INDUSTRIAL,
        &super::chinese_medieval_pair::CHINESE_MEDIEVAL,
    ];
    let seen: Vec<_> = others
        .iter()
        .map(|o| (o.name, o.map_style, o.ai_tribe, o.difficulty))
        .collect();
    assert_eq!(
        seen,
        [
            ("GreatSaharaChineseIndustrial", 7, 14, 3),
            ("GreatSaharaChineseMedieval", 7, 14, 5),
        ],
        "the Chinese's other lobbies"
    );
    for o in others {
        assert_ne!(
            (o.map_style, o.ai_tribe, o.difficulty),
            triple,
            "{} is this lobby",
            o.name
        );
    }
}

/// **The long's stride, walked back from the word 4884** (item 3625):
/// only run1283's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_CHINESE_GUNPOWDER_STRIDE: (i64, i64) = (101, 4801);

pub(crate) fn chinese_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_GUNPOWDER.start],
        true,
        CHINESE_GUNPOWDER.long,
        "run1283 stride",
        &[(
            CHINESE_GUNPOWDER.long.0,
            WIDENING_CHINESE_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_CHINESE_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn chinese_gunpowder_pair_the_long_s_stride_walks_back_from_4884() {
    let _pins = Pins::hold();
    let Some(w) = chinese_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 48, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3625: 177 keys, every one standing — `form` on every start unit
    // (ours -1 against theirs 9; 0 on `0/0`) and on each unit as it is
    // born (ours -1 against theirs 0; parked 3386i),
    // the buildings' `build:extra` (which the stride does not print), and
    // who=1's groups' `held` and pools as each group is minted. No unit's
    // position or orders part through block 4801.
    pin_eq!(w.firsts.len(), 177, "the stride's baseline");
}

/// run1283b: the word 4884's window, blocks 4879..5135 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 4885 and 250 after (item 3625); the stride's blocks are 4801 and
/// 4901.
pub(crate) const RUN1283B: &str =
    "gamelog-run1283b-eastindies-chinese-gunpowder-window-4879-5135.txt";

/// The window: block 4879 through 5135; the word 4884's own block is 4885.
pub(crate) const WIDENING_CHINESE_GUNPOWDER_FRAME_4884: (i64, i64) = (4879, 5135);

/// **The word 4884's window**: run1283b walked from run1282's start with
/// the recorder on (item 3625).
pub(crate) fn chinese_gunpowder_frame_4884_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_GUNPOWDER.start],
        true,
        CHINESE_GUNPOWDER.long,
        "run1283b",
        &[(RUN1283B, WIDENING_CHINESE_GUNPOWDER_FRAME_4884.0)],
        WIDENING_CHINESE_GUNPOWDER_FRAME_4884,
        1,
        &[4885],
        true,
    )
}

#[test]
fn chinese_gunpowder_pair_run1283b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_gunpowder_frame_4884_window() else {
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
    // Item 3625, the first walk: 3727 keys. Block 4879 holds the standing
    // families (every unit's `form`, both leaders' `SITE[i].reg`, who=0's
    // `aggression[1]`, who=1's groups' `held` and pools, `build:extra`)
    // and **the Heavy Fire Lance `1/215`** (TypeIndex 97, uid 300) at x
    // 40327 against theirs' 40105, y 36732 on both — six frames of its
    // 37-a-frame walk east ahead, from the gap's block 4863
    // (`chinese_gunpowder_pair_run1283c_s_gap_is_widened_whole`). On 4884
    // `1/196`'s `collide_o` reads 215 against 213. **The word 4884**, ours
    // 341 draws against 357, index 6: theirs spends a `Unit::move_step`,
    // a `PathFinder::astar_path` under `find_upath` and another
    // `move_step` (indexes 6..8) before the caravan road's
    // `calc_road_cost` run that ours reaches at 6. On the word's block 4885 the units round
    // `1/215` part: `1/185`, `1/186`, `1/197` and `1/213`'s positions,
    // `collide` and paths.
    //
    // Item 3637: **3727 → 167**. With `retry` kept, `1/215` agrees, and the
    // window is the standing families until block 5133: the human's
    // Citizen `0/10` (type 50, uid 17) reads `hits:damage` 26, `hits_left`
    // 24 and `damage_frame` 5132 in theirs against 0, 50 and 0 in ours,
    // and flees; on 5134 its walk, path and `idle` part.
    pin_eq!(w.firsts.len(), 167, "run1283b baseline");
}

/// run1299: the word 5196's window, blocks 5190..5446 at run1283b's detail
/// (item 3637) — six blocks before the word's own block 5197 and 250
/// after; the long's stride prints 5101 and 5201 only.
pub(crate) const RUN1299: &str =
    "gamelog-run1299-eastindies-chinese-gunpowder-window-5190-5446.txt";

/// The window: block 5190 through 5446; the word 5196's own block is 5197.
pub(crate) const WIDENING_CHINESE_GUNPOWDER_FRAME_5196: (i64, i64) = (5190, 5446);

/// **The word 5196's window**: run1299 walked from run1282's start with
/// the recorder on (item 3637).
pub(crate) fn chinese_gunpowder_frame_5196_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_GUNPOWDER.start],
        true,
        CHINESE_GUNPOWDER.long,
        "run1299",
        &[(RUN1299, WIDENING_CHINESE_GUNPOWDER_FRAME_5196.0)],
        WIDENING_CHINESE_GUNPOWDER_FRAME_5196,
        1,
        &[5197],
        true,
    )
}

#[test]
fn chinese_gunpowder_pair_run1299_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_gunpowder_frame_5196_window() else {
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
    // Item 3637, the first walk: 4356 keys. Block 5190 holds the standing
    // families and two units already apart: the human's Citizen `0/10`,
    // wounded on 5132 in theirs and 5133 in ours (run1283b's block 5133),
    // and who=1's `1/39`, whose move aims at (7272, 8664) against theirs'
    // (7224, 8664); on 5202 its attack names `0/12` against `0/10`. **The
    // word 5196**, ours 19 draws against 20, index 1: theirs spends a
    // second `Unit::do_idle+0x7d` before the `do_guard+0x7f4` ours spends
    // at 1 on `1/126` (`docs/AI.md` section 333.1).
    pin_eq!(w.firsts.len(), 4356, "run1299 baseline");
}

/// run1283c: the gap before the word's window, blocks 4795..4880 at
/// run1283b's detail (item 3625) — the stride's block 4801 and the gap
/// 4802..4878 that no other dump prints, where the Heavy Fire Lance
/// `1/215` is born and parts.
pub(crate) const RUN1283C: &str =
    "gamelog-run1283c-eastindies-chinese-gunpowder-window-4795-4880.txt";

/// The window: block 4795 through 4880.
pub(crate) const WIDENING_CHINESE_GUNPOWDER_GAP: (i64, i64) = (4795, 4880);

/// **The gap's window**: run1283c walked from run1282's start with the
/// recorder on (item 3625).
pub(crate) fn chinese_gunpowder_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_GUNPOWDER.start],
        true,
        CHINESE_GUNPOWDER.long,
        "run1283c",
        &[(RUN1283C, WIDENING_CHINESE_GUNPOWDER_GAP.0)],
        WIDENING_CHINESE_GUNPOWDER_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn chinese_gunpowder_pair_run1283c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_gunpowder_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 85, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3625, the gap's first walk: 144 keys. Block 4795 holds the
    // standing families (every unit's `form`, both leaders' `SITE[i].reg`,
    // who=1's groups' `held` and pools, `build:extra`), and **the state
    // first parts on block 4863, at the Heavy Fire Lance `1/215`**
    // (TypeIndex 97, land, uid 300; born 4797): theirs stands at (39905,
    // 36456) with `move.dest` 0, `last_x` -1, `tolerance` 384 and
    // `path_recursion` 10, ours walks to (39923, 36488) with `dest` 1,
    // `last_x` 39905, `tolerance` 0 and `path_recursion` 3, and `path`
    // 12 against 13. Walked back one block: on 4862 theirs' attack-move
    // holds `retry` 6 (rolled on frame 4861 by `PathFinder::astar_path+
    // 0x1697`, a spent unit-grid search), ours 0 — a field the move
    // order's row does not compare (`order.rs`). Theirs stands out the six
    // frames and re-paths on 4869 to (40068, 36732), ours on 4863, so ours
    // walks six frames (222) ahead (`docs/AI.md` section 333).
    //
    // Item 3637: **144 → 130**. The move row compares `retry` and
    // `attempts`, and `do_move` keeps the roll: `1/215`'s rows leave, and
    // what remains is the standing families (`docs/AI.md` section 333.1).
    pin_eq!(w.firsts.len(), 130, "run1283c baseline");
}
