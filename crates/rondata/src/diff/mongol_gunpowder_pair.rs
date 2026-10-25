//! **The Mongols at a held Gunpowder age, East Indies** (DECISIONS 90,
//! 91, 93, 94, item 3623): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Mongols (17)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `dutch_medieval_pair`'s lobby with the
//! nation and the age moved. Its triple, `(18, 17, 5)`, is new: the
//! Mongols' other lobbies are the successor pair `mongol_pair`'s
//! `EastIndiesMongolAllTech`, `(18, 17, 3)` — another difficulty —
//! `sahara_mongol`'s `GreatSaharaMongolAllTech`, `(7, 17, 3)`, and
//! `mongol_classical_pair`'s `GreatSaharaMongolClassical`, `(7, 17, 5)` —
//! another map; the battery fields no Mongols. Captures: run1278 (a
//! `DUMP_ALL` start) and run1279 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §331.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const MONGOL_GUNPOWDER: Pair = Pair {
    name: "EastIndiesMongolGunpowder",
    map_style: 18,
    ai_tribe: 17,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1278-eastindies-mongol-gunpowder-start.txt",
    long: (
        "gamelog-run1279-eastindies-mongol-gunpowder-trace.txt",
        "rontrace-run1279.log",
    ),
    // The game ended itself on 5399 (closing block 5400).
    length: 5399,
    // Item 3623, first try: frame 1973 of 5399 by the count and the
    // sequence. On 1973 ours spends 7 draws against theirs' 5, parting at
    // index 0 — ours `Guy::set_anim+0x97a < Unit::do_idle+0x7d` (twice,
    // `1/17`'s), theirs `Guy::init_real+0x52` (`docs/AI.md` section 331).
    // Item 3630: 1973 → 4499. A supplier's pool is `MANA ×
    // (get_supply_upgrade + 1)`, so the Monarch `1/17` casts Create Decoy
    // on 1946 under one supply upgrade (`mana` 2000 against `mana_burn`
    // 767 + 1000). On 4499 ours spends 11 draws against 10, parting at
    // index 2 — ours `Guy::set_anim+0x97a < Unit::move_step+0x823`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`.
    count: 4499,
    sequence: 4499,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MONGOL_GUNPOWDER_PAIR_WORD: i64 =
    if MONGOL_GUNPOWDER.count < MONGOL_GUNPOWDER.sequence {
        MONGOL_GUNPOWDER.count
    } else {
        MONGOL_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn mongol_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MONGOL_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MONGOL_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn mongol_gunpowder_pair_captures_say_their_lobby() {
    let p = &MONGOL_GUNPOWDER;
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
        "the Mongol Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3623; the battery's identity
/// test refuses it) and none of the battery's — the battery fields no
/// Mongols — nor any other Mongol lobby's: the successor pair
/// `EastIndiesMongolAllTech` is `(18, 17, 3)`, another difficulty, and
/// `GreatSaharaMongolAllTech` `(7, 17, 3)` and `GreatSaharaMongolClassical`
/// `(7, 17, 5)` another map.
#[test]
fn mongol_gunpowder_pair_is_no_other_lobby() {
    let p = &MONGOL_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Mongols"
    );
    let others = [
        &super::mongol_pair::MONGOL,
        &super::sahara_mongol::SAHARA_MONGOL,
        &super::mongol_classical_pair::MONGOL_CLASSICAL,
    ];
    let seen: Vec<_> = others
        .iter()
        .map(|o| (o.name, o.map_style, o.ai_tribe, o.difficulty))
        .collect();
    assert_eq!(
        seen,
        [
            ("EastIndiesMongolAllTech", 18, 17, 3),
            ("GreatSaharaMongolAllTech", 7, 17, 3),
            ("GreatSaharaMongolClassical", 7, 17, 5),
        ],
        "the Mongols' other lobbies"
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

/// **The long's stride, walked back from the word 1973** (item 3623):
/// only run1279's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_STRIDE: (i64, i64) = (101, 1901);

pub(crate) fn mongol_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1279 stride",
        &[(MONGOL_GUNPOWDER.long.0, WIDENING_MONGOL_GUNPOWDER_STRIDE.0)],
        WIDENING_MONGOL_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_the_long_s_stride_walks_back_from_1973() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 19, "every stride block in the window");
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
    // Item 3623: 81 keys, every one standing — the start's buildings'
    // `build:extra` (which the stride does not print) and who=1's groups'
    // `held` and pools as each group is minted (65 on 101 through 70 and 71
    // on 1901). No unit's position, orders or spell state parts through
    // block 1901: the Despot `1/17`'s (TypeIndex 352) Create Decoy cast of
    // 1401 (`CASTORDER`, spell 634) agrees on both sides.
    pin_eq!(w.firsts.len(), 81, "the stride's baseline");
}

/// run1279b: the word 1973's window, blocks 1968..2224 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 1974 and 250 after (item 3623); the stride's blocks are 1901 and
/// 2001.
pub(crate) const RUN1279B: &str =
    "gamelog-run1279b-eastindies-mongol-gunpowder-window-1968-2224.txt";

/// The window: block 1968 through 2224; the word 1973's own block is 1974.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_FRAME_1973: (i64, i64) = (1968, 2224);

/// **The word 1973's window**: run1279b walked from run1278's start with
/// the recorder on (item 3623).
pub(crate) fn mongol_gunpowder_frame_1973_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1279b",
        &[(RUN1279B, WIDENING_MONGOL_GUNPOWDER_FRAME_1973.0)],
        WIDENING_MONGOL_GUNPOWDER_FRAME_1973,
        1,
        &[1974],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_run1279b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_frame_1973_window() else {
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
    // Item 3623, the first walk: 1038 keys. Block 1968 holds the standing
    // families (every unit's `form`, `SITE[i].reg`, both leaders'
    // `aggression`, who=1's groups' `held` and pools, the make queue's
    // costs) and **the Monarch `1/17`** (TypeIndex 354, land, uid 37;
    // The Despot, 352, through 1901), standing at (34008, 37921): theirs
    // holds a `CASTORDER` (spell 634, Create Decoy, `paid` 1) with
    // `spell_time` 22 and `mana_burn` 1745, ours no order, `idle` 14,
    // `spell_time` 0 and `mana_burn` 745 — the cast began in the gap
    // 1902..1967 that no dump prints. **The word 1973**, ours 7 draws
    // against 5, index 0: ours spends `1/17`'s two `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, and the other five agree in order (the birth's
    // two `Guy::init_real`, two `Guy::inc_time`, one farm) — ours acts
    // (`docs/AI.md` section 331). Nothing parts on the word's block 1974;
    // the first rows past it are who=1's `leader:attack` 210 against 218
    // on 1976 and the make list from 1983.
    // Item 3630: 1038 → 90. The Monarch casts Create Decoy on 1946 as
    // theirs does (a supplier's pool, `mana` 2000 under one supply
    // upgrade), so `1/17`'s rows leave the window; what stays is the
    // standing families and who=1's `leader:attack` 210 against 218 on
    // 1976. The word left this window for 4499.
    pin_eq!(w.firsts.len(), 90, "run1279b baseline");
}

/// run1292: the gap before the word's window, blocks 1895..1975 at
/// run1279b's detail, `cover=1` (item 3630) — the stride's block 1901 and
/// the gap 1902..1967 that no earlier dump printed.
pub(crate) const RUN1292: &str = "gamelog-run1292-eastindies-mongol-gunpowder-window-1895-1975.txt";

/// The window: block 1895 through 1975; the cast's own block is 1947.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_GAP: (i64, i64) = (1895, 1975);

/// **The gap's window**: run1292 walked from run1278's start with the
/// recorder on (item 3630).
pub(crate) fn mongol_gunpowder_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1292",
        &[(RUN1292, WIDENING_MONGOL_GUNPOWDER_GAP.0)],
        WIDENING_MONGOL_GUNPOWDER_GAP,
        1,
        &[1947],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_run1292_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 80, "every captured block");
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
    // Item 3630, the gap's first walk: 85 keys, and the one unit row was
    // the Monarch `1/17` (TypeIndex 354) on block 1947 — theirs `orders.len`
    // 1 (`CASTORDER`, Create Decoy), `spell_time` 1, `mana_burn` 1766,
    // `idle` 0; ours 0, 0, 766 and 12. Ours reached the hero arm on the
    // same frame 1946 (`Army::use_generals`), the Monarch standing with no
    // decoy, and refused it on the mana: 1000 against `mana_burn` 767 +
    // Create Decoy's 1000, where the original's `UnitData::mana` gives a
    // supplier `MANA × (get_supply_upgrade + 1)` = 2000 (the leader held
    // one step from 1818..1946). Built: 85 → 80, every row standing (both
    // leaders' `SITE[i].reg`, who=1's stockpile and queue costs, the
    // groups' `held` and pools).
    pin_eq!(w.firsts.len(), 80, "run1292 baseline");
}

/// run1292b: the word 4499's window, blocks 4493..4749 at run1279b's
/// detail — six blocks before the word's block 4500 and 250 after (item
/// 3630); the stride's blocks are 4401 and 4501.
pub(crate) const RUN1292B: &str =
    "gamelog-run1292b-eastindies-mongol-gunpowder-window-4493-4749.txt";

/// The window: block 4493 through 4749; the word 4499's own block is 4500.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_FRAME_4499: (i64, i64) = (4493, 4749);

/// **The word 4499's window**: run1292b walked from run1278's start with
/// the recorder on (item 3630).
pub(crate) fn mongol_gunpowder_frame_4499_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1292b",
        &[(RUN1292B, WIDENING_MONGOL_GUNPOWDER_FRAME_4499.0)],
        WIDENING_MONGOL_GUNPOWDER_FRAME_4499,
        1,
        &[4500],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_run1292b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_frame_4499_window() else {
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
    // Item 3630, the word 4499's widening: 5030 keys, the state parted
    // before the window (run1292c's block 4475). **The word 4499**, ours 11
    // draws against 10, index 2: ours `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`, theirs `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`.
    pin_eq!(w.firsts.len(), 5030, "run1292b baseline");
}

/// **The long's stride between the windows** (item 3630): run1279's
/// stride blocks after run1292's window and up to the word 4499's.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_LATE_STRIDE: (i64, i64) = (2001, 4401);

pub(crate) fn mongol_gunpowder_late_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1279 late stride",
        &[(
            MONGOL_GUNPOWDER.long.0,
            WIDENING_MONGOL_GUNPOWDER_LATE_STRIDE.0,
        )],
        WIDENING_MONGOL_GUNPOWDER_LATE_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_the_long_s_stride_walks_back_from_4499() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_late_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 25, "every stride block in the window");
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
    // Item 3630: 187 keys, every one standing — the start's buildings'
    // and new buildings' `build:extra`, who=1's groups' `held` and pools as
    // each is minted, and new units' `form`. No unit's position or orders
    // part through block 4401.
    pin_eq!(w.firsts.len(), 187, "the late stride's baseline");
}

/// run1292c: the walk back from the word 4499, blocks 4395..4495 at
/// run1279b's detail (item 3630) — the stride agrees through 4401 and
/// run1292b's first block 4493 already parts.
pub(crate) const RUN1292C: &str =
    "gamelog-run1292c-eastindies-mongol-gunpowder-window-4395-4495.txt";

/// The window: block 4395 through 4495.
pub(crate) const WIDENING_MONGOL_GUNPOWDER_BEFORE_4499: (i64, i64) = (4395, 4495);

/// **The walk back from 4499**: run1292c walked from run1278's start with
/// the recorder on (item 3630).
pub(crate) fn mongol_gunpowder_before_4499_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_GUNPOWDER.start],
        true,
        MONGOL_GUNPOWDER.long,
        "run1292c",
        &[(RUN1292C, WIDENING_MONGOL_GUNPOWDER_BEFORE_4499.0)],
        WIDENING_MONGOL_GUNPOWDER_BEFORE_4499,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_gunpowder_pair_run1292c_walks_back_from_4499() {
    let _pins = Pins::hold();
    let Some(w) = mongol_gunpowder_before_4499_window() else {
        return;
    };
    pin_eq!(w.blocks, 100, "every captured block");
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
    // Item 3630: 404 keys. **The state first parts on block 4475** (frame
    // 4474): both leaders' `treaties` — `0`'s `treaties[1]` and `1`'s
    // `treaties[0]` — read 3 in theirs and 1 in ours, and the human's `0/5`
    // takes 26 in theirs (`hits_left` 24 against 50, `damage_frame` and its
    // city's `raid_stamp` 4474) with a second order (kind 4 over ours' 7).
    // From 4476 `0/5`'s position parts, and from 4479 who=1's move
    // destinations; no draw parts until 4499.
    pin_eq!(w.firsts.len(), 404, "run1292c baseline");
}
