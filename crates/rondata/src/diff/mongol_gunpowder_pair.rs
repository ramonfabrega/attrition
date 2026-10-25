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
    count: 1973,
    sequence: 1973,
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
    pin_eq!(w.firsts.len(), 1038, "run1279b baseline");
}
