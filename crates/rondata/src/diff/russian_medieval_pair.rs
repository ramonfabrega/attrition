//! **The Russians at a held Medieval age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, item 3605): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Russians (13)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval: `STARTING_TECHNOLOGY
//! 2`, `ENDING_TECHNOLOGY 2` — `chinese_medieval_pair`'s lobby with the
//! nation moved. Its triple, `(7, 13, 5)`, is new: the held-out
//! `GreatSaharaRussianTier4` is `(7, 13, 4)`, another difficulty, and the
//! Russians' other lobbies are other maps — `russian_modern_pair`'s
//! `(18, 13, 5)` and `russian_word`'s `GreatLakesRussianTier2`, `(14, 13,
//! 2)`. Captures: run1257 (a `DUMP_ALL` start) and run1258 (the draw
//! stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride
//! of 100, walked from the start's initial state). The first try is in
//! `docs/AI.md` §326.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const RUSSIAN_MEDIEVAL: Pair = Pair {
    name: "GreatSaharaRussianMedieval",
    map_style: 7,
    ai_tribe: 13,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1257-greatsahara-russian-medieval-start.txt",
    long: (
        "gamelog-run1258-greatsahara-russian-medieval-trace.txt",
        "rontrace-run1258.log",
    ),
    // The game ended itself on 4541 (closing block 4542).
    length: 4541,
    // Item 3605, first try: frame 3149 by the sequence (3164 by the
    // count), 11 draws a side, parting at index 3 — ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Ammo::init+0xcd9` (`docs/AI.md` section 326): the Supply Wagon born
    // into the Peltasts' freed `1/43` in ours, `1/132` in theirs. On the
    // tree item 3614 landed (`find_free`'s `o_up` term holds a dead tail),
    // the wagon is `1/132` on both sides and the first try reads 3349 by
    // the count and the sequence, ours 10 draws against 9, index 1 — ours
    // `Guy::set_anim+0xf2f < Unit::set_anim+0x56` (the Elite Javelineers
    // `1/27`, TypeIndex 84), theirs `Guy::set_anim+0x97a <
    // Unit::do_guard+0x7f4`.
    count: 3349,
    sequence: 3349,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const RUSSIAN_MEDIEVAL_PAIR_WORD: i64 =
    if RUSSIAN_MEDIEVAL.count < RUSSIAN_MEDIEVAL.sequence {
        RUSSIAN_MEDIEVAL.count
    } else {
        RUSSIAN_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn russian_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2, and the long is scoreable.
#[test]
fn russian_medieval_pair_captures_say_their_lobby() {
    let p = &RUSSIAN_MEDIEVAL;
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
        "the Russian Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3605; the battery's identity
/// test refuses it) and none of the battery's: the held-out
/// `GreatSaharaRussianTier4` is `(7, 13, 4)`, another difficulty, and the
/// Russians' other pinned lobbies are other maps (item 3605).
#[test]
fn russian_medieval_pair_is_no_other_lobby() {
    let p = &RUSSIAN_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let held_out = battery::BATTERY
        .iter()
        .find(|l| l.name == "GreatSaharaRussianTier4")
        .expect("the held-out Russian game");
    assert_eq!(
        (held_out.map_style, held_out.ai_tribe, held_out.difficulty),
        (7, 13, 4),
        "the held-out Russian game is another difficulty"
    );
    for (name, other) in [
        (
            super::russian_word::RUSSIAN.name,
            (
                super::russian_word::RUSSIAN.map_style,
                super::russian_word::RUSSIAN.ai_tribe,
                super::russian_word::RUSSIAN.difficulty,
            ),
        ),
        (
            super::russian_modern_pair::RUSSIAN_MODERN.name,
            (
                super::russian_modern_pair::RUSSIAN_MODERN.map_style,
                super::russian_modern_pair::RUSSIAN_MODERN.ai_tribe,
                super::russian_modern_pair::RUSSIAN_MODERN.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// run1258b: blocks 3143..3399 at the long's detail with `AMMO=5`, booked
/// for the first try's 3149 — six blocks before its block 3150 and 250
/// after (item 3605); the long's stride prints units every hundredth block
/// (3101 and 3201) and no `LEADERS` past the start. On 3614's tree the word
/// is 3349, whose block 3350 the window holds with 49 after it.
pub(crate) const RUN1258B: &str =
    "gamelog-run1258b-greatsahara-russian-medieval-window-3143-3399.txt";

/// The window: block 3143 through 3399; the word 3349's own block is 3350.
pub(crate) const WIDENING_RUSSIAN_MEDIEVAL_RUN1258B: (i64, i64) = (3143, 3399);

/// **The word 3349's window**: run1258b walked from run1257's start with
/// the recorder on (item 3605).
pub(crate) fn russian_medieval_run1258b_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_MEDIEVAL.start],
        true,
        RUSSIAN_MEDIEVAL.long,
        "run1258b",
        &[(RUN1258B, WIDENING_RUSSIAN_MEDIEVAL_RUN1258B.0)],
        WIDENING_RUSSIAN_MEDIEVAL_RUN1258B,
        1,
        &[3350],
        true,
    )
}

#[test]
fn russian_medieval_pair_run1258b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_medieval_run1258b_window() else {
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
    // Item 3605, the first walk: 2299 keys. The window opens apart on
    // 3143, past the stride's 3101, which agrees: beside the standing
    // families (every unit's `form`, who=0's `SITE[i].reg` and
    // `aggression[1]`, who=1's groups' `held`), who=1's unit numbers part
    // from 1/132 up. Ours' Supply Wagon (TypeIndex 63, three figures, at
    // (40536, 22776), born on 3106 in ours) took `1/43`, the slot ours'
    // Peltasts `1/43` freed on 3053; theirs' is `1/132`, and every later
    // birth sits one number higher in theirs (`1/133` holds ours' `1/132`,
    // ...). So on the word 3149 ours' wagon spends its `Guy::inc_time`
    // draw ahead of the Heavy Horse Archer `1/54`'s round (TypeIndex 188,
    // `Ammo::init+0xcd9`), and theirs behind it: the same eleven draws,
    // parted at index 3. On the tree item 3614 landed: 337 keys — the
    // wagon is `1/132` on both sides, and past the standing families the
    // first row is the Elite Javelineers `1/27`'s (TypeIndex 84) target on
    // block 3348, building `0/2005` in ours against `0/2006` in theirs;
    // its heading parts on 3350, the word 3349's block, where ours spends
    // a `Unit::set_anim` draw theirs does not. Then `1/42`'s wound on 3351
    // (`hits_left` 91 against 90) and `1/41`, ours' alone.
    pin_eq!(w.firsts.len(), 337, "run1258b baseline");
}

/// **The long's stride, walked back from the window** (item 3605):
/// only run1258's stride blocks are on disk before run1258b's window.
pub(crate) const WIDENING_RUSSIAN_MEDIEVAL_STRIDE: (i64, i64) = (101, 3101);

pub(crate) fn russian_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_MEDIEVAL.start],
        true,
        RUSSIAN_MEDIEVAL.long,
        "run1258 stride",
        &[(RUSSIAN_MEDIEVAL.long.0, WIDENING_RUSSIAN_MEDIEVAL_STRIDE.0)],
        WIDENING_RUSSIAN_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_medieval_pair_the_long_s_stride_walks_back_from_3149() {
    let _pins = Pins::hold();
    let Some(w) = russian_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 31, "every stride block in the window");
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
    // Item 3605: 87 keys, the standing families alone — every unit's
    // `form` and `build:extra`, which the stride does not print, and
    // who=1's pools and groups' `held`. The Peltasts `1/43` stand on 3001
    // and are gone on 3101 on both sides; the wagon's birth, and the
    // numbering, fall in the gap 3102..3142.
    pin_eq!(w.firsts.len(), 87, "the stride's baseline");
}

/// **The Elite Javelineers' two buildings, valued as run1268's packet values
/// them** (item 3615, `docs/AI.md` §326): on block 3347 — the state the
/// packet holds, before the tick whose guard roll drops `1/27`'s attack and
/// searches again — `Object::compare_target(o, 0, 0, 0)` returns **27** for
/// the Library `0/2005` (`240 × 11`, `(2640 + 99) / 100`) and **71** for the
/// Market `0/2006` (`320 × 2 × 11`: the computer searcher's class arm takes
/// a training building twice, `0064e9e2`). Without the arm the Market is 36
/// and the two score 9 apiece, the Library first. The city `0/2000` is the
/// packet's 2787 (the class arm's `× 5` and the armed weight's empty-city
/// gate together).
#[test]
fn russian_medieval_pair_s_guard_search_values_the_market_as_a_trainer() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let p = &RUSSIAN_MEDIEVAL;
    let (Some(path), Some(sib), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1)) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    // Block 3347 is the state after the tick this crate numbers 3346.
    while built.sim.frame < 3347 {
        built.tick();
    }
    let s = &built.sim;
    let me = (0..s.units.len())
        .find(|&u| s.units[u].owner == 1 && s.units[u].index == 27)
        .expect("the Elite Javelineers 1/27");
    let building = |o: i16| {
        (0..s.buildings.len())
            .find(|&b| s.buildings[b].owner == 0 && s.buildings[b].index == o)
            .expect("who=0's building")
    };
    let ai = s.search_ai(1);
    let value = |o: i16| {
        s.compare_target(
            sim::combat::Obj::Unit(me),
            sim::combat::Obj::Building(building(o)),
            false,
            ai,
        )
    };
    pin_eq!(
        ai,
        false,
        "the search's `ai`, as the packet's call passes it"
    );
    pin_eq!(
        (value(2005), value(2006)),
        (27, 71),
        "the Library's and the Market's worth to 1/27, as run1268's packet computes them"
    );
    // The city `0/2000`, empty: the class arm's `× 5` with the armed
    // weight's `num_inside` gate (`64f171`) is the packet's 2787; the arm
    // alone read 13935, the gate alone 2785 less the arm's factor.
    pin_eq!(
        value(2000),
        2787,
        "the empty city's worth to 1/27, as the packet computes it"
    );
}
