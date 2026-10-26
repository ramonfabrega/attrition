//! **The Iroquois at a held Enlightenment age, East Indies** (DECISIONS 90,
//! 91, 93, 94, item 3629): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Iroquois (18)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4` —
//! `british_enlightenment_pair`'s lobby with the nation moved. Its triple,
//! `(18, 18, 5)`, is new: the battery fields no Iroquois (both its Tier3s
//! left it for AI words: `iroquois_word`'s `(14, 18, 3)` and
//! `iroquois_sahara_word`'s `(7, 18, 3)`), and the Ancient-start pair is
//! Great Sahara's `(7, 18, 5)`. Captures: run1290 (a `DUMP_ALL` start) and
//! run1291 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §337; whole since item
//! 3655 (the Bomb Ketch fires from its pivots), the closing state compared.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const IROQUOIS_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesIroquoisEnlightenment",
    map_style: 18,
    ai_tribe: 18,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1290-eastindies-iroquois-enlightenment-start.txt",
    long: (
        "gamelog-run1291-eastindies-iroquois-enlightenment-trace.txt",
        "rontrace-run1291.log",
    ),
    // The game ended itself on 5128 (closing block 5129).
    length: 5128,
    // Item 3629, first try: frame 3962, by the count and the sequence —
    // ours 11 draws against 14, parting at index 6, ours
    // `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`
    // (`docs/AI.md` §337). Item 3655: **5128 of 5128, the pair whole** —
    // the Bomb Ketch's piece 297 releases through its pivots
    // (`sim::pivot::RELEASES`), so `1/82`'s round leaves (4013, 12593, 133)
    // and lands on 3962 on both sides.
    count: 5128,
    sequence: 5128,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const IROQUOIS_ENLIGHTENMENT_PAIR_WORD: i64 =
    if IROQUOIS_ENLIGHTENMENT.count < IROQUOIS_ENLIGHTENMENT.sequence {
        IROQUOIS_ENLIGHTENMENT.count
    } else {
        IROQUOIS_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn iroquois_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &IROQUOIS_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "IROQUOIS_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4, and the long is scoreable.
#[test]
fn iroquois_enlightenment_pair_captures_say_their_lobby() {
    let p = &IROQUOIS_ENLIGHTENMENT;
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
        "the Iroquois Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3629; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no
/// Iroquois; and no other pinned Iroquois lobby's: `iroquois_word`'s
/// `(14, 18, 3)`, `iroquois_sahara_word`'s `(7, 18, 3)` and the
/// Ancient-start pair's `(7, 18, 5)` (item 3629).
#[test]
fn iroquois_enlightenment_pair_is_no_other_lobby() {
    let p = &IROQUOIS_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Iroquois"
    );
    let word = &super::iroquois_word::IROQUOIS;
    let sahara = &super::iroquois_sahara_word::IROQUOIS_SAHARA;
    let ancient = &super::iroquois_ancient_pair::IROQUOIS_ANCIENT;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
        (
            ancient.name,
            (ancient.map_style, ancient.ai_tribe, ancient.difficulty),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Iroquois");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3962** (item 3629):
/// only run1291's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_IROQUOIS_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 3901);

pub(crate) fn iroquois_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS_ENLIGHTENMENT.start],
        true,
        IROQUOIS_ENLIGHTENMENT.long,
        "run1291 stride",
        &[(
            IROQUOIS_ENLIGHTENMENT.long.0,
            WIDENING_IROQUOIS_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_IROQUOIS_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn iroquois_enlightenment_pair_the_long_s_stride_walks_back_from_3962() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 39, "every stride block in the window");
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
    // Item 3629: 155 keys, every one standing: 47 buildings' `build:extra`
    // (which the stride does not print), `form` on 65 units, who=1's 20
    // groups' `held` and pools as each is minted (65 on 101 through 83 on
    // 3501), and `blown` on three of who=1's units (`1/0` from 101, `1/18` and `1/19`
    // from 1101; parked 3114). No unit's position, orders or hits part
    // through block 3901.
    pin_eq!(w.firsts.len(), 155, "the stride's baseline");
}

/// run1291b: the word 3962's window, blocks 3956..4212 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 3963 and 250 after (item 3629); the stride's blocks are 3901 and
/// 4001.
pub(crate) const RUN1291B: &str =
    "gamelog-run1291b-eastindies-iroquois-enlightenment-window-3956-4212.txt";

/// The window: block 3956 through 4212; the word 3962's own block is 3963.
pub(crate) const WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_3962: (i64, i64) = (3956, 4212);

/// **The word 3962's window**: run1291b walked from run1290's start with
/// the recorder on and every round compared (item 3629).
pub(crate) fn iroquois_enlightenment_frame_3962_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[IROQUOIS_ENLIGHTENMENT.start],
        true,
        IROQUOIS_ENLIGHTENMENT.long,
        "run1291b",
        &[(RUN1291B, WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_3962.0)],
        WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_3962,
        1,
        &[3963],
        true,
    )
}

#[test]
fn iroquois_enlightenment_pair_run1291b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_enlightenment_frame_3962_window() else {
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
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3629, the first walk: 756 keys. Block 3956 carries the standing
    // families (`form`, `SITE[i].reg`, who=0's `aggression[1]`, who=1's
    // groups' `held` and pools, `blown`). **The state first parts on block
    // 3963, the word's own**: theirs' round from the Bomb Ketch `1/82`
    // (TypeIndex 347, sea, stance Raid, attacking `0/4`) lands on 3962 and
    // ours' is still flying — the human's Citizen `0/4` (TypeIndex 50)
    // `hits_left` ours 65 theirs 47, `damage_frame` 0 against 3962, its
    // order ours `GATHERORDER` (7) against `FLEEORDER` (4) to (6600,
    // 7608); the splash on `0/2002` (damage 4), `0/2003` (15) and `0/2004`
    // (1); the city `0/2000`'s `raid_stamp` 0 against 3962 and
    // `city_flags` 0x2/0x4/0x8; and treaty bit 1 in both leaders
    // (`treaties[1]` on who=0, `treaties[0]` on who=1, ours 1 theirs 3).
    // The draw on 3962: ours 11 against 14, index 6, ours
    // `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`
    // (`docs/AI.md` §337).
    // Item 3655: **756 → 121**, every one standing — block 3956's 110
    // (`SITE[i].reg`, `form`, who=1's groups' `held` and pools, who=0's
    // `aggression[1]`) and eleven later: `form` on `1/150`, `1/154` and
    // `1/157`, and groups 68, 82, 84 and 85's `held` and pools as each is
    // minted. No unit's position, orders or hits part.
    pin_eq!(w.firsts.len(), 121, "run1291b baseline");
    // Every round, both directions: 301 round-blocks theirs, 228 ours. The
    // first to part is `1/82`'s, on the window's first block, fired in the
    // undumped gap 3902..3955 on both sides: ours leaves from (3912, 12792,
    // 0) and flies 43, theirs from (4013, 12593, 133) and flies 41 — the
    // Bomb Ketch has no launch row (`sim::launch`'s `BAYS`). Every round
    // that parts is a Bomb Ketch's (`1/64`, `1/65`, `1/80`, `1/82`, all
    // TypeIndex 347), from the same launch point (`sz` 0 against 130–133).
    // Item 3655: with piece 297's pivot rows every round agrees — 301 a
    // side, none parting; `1/82`'s leaves (4013, 12593, 133) and flies 41.
    pin_eq!(w.rounds_read, (301, 301), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1320: the closing window, blocks 4873..5129 at run1291b's detail,
/// `cover=0` (item 3655) — 256 blocks through the closing block, so the
/// word 5128, the game's last frame, sits inside a window.
pub(crate) const RUN1320: &str =
    "gamelog-run1320-eastindies-iroquois-enlightenment-window-4873-5129.txt";

/// The window: block 4873 through the closing block 5129.
pub(crate) const WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_5128: (i64, i64) = (4873, 5129);

/// **The closing window**: run1320 walked from run1290's start with the
/// recorder on, the rounds compared (item 3655).
pub(crate) fn iroquois_enlightenment_frame_5128_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[IROQUOIS_ENLIGHTENMENT.start],
        true,
        IROQUOIS_ENLIGHTENMENT.long,
        "run1320",
        &[(RUN1320, WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_5128.0)],
        WIDENING_IROQUOIS_ENLIGHTENMENT_FRAME_5128,
        1,
        &[5129],
        true,
    )
}

#[test]
fn iroquois_enlightenment_pair_run1320_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_enlightenment_frame_5128_window() else {
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
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3655, the pair whole: 131 keys. Past the standing families —
    // `form` (65), who=1's groups' `held` (24) and pools (24), both
    // leaders' `SITE[i].reg` (14) and who=0's `aggression[1]` — **the
    // state parts on block 4983, in who=1's `1/2005` (type 435) queue**:
    // the third entry is TypeIndex 555 on both sides, and ours prices it
    // 500 where theirs prices it 250, so who=1's wealth bucket reads 15568
    // against 15818; the same price parts again on 5064 at `queue[1]`. No
    // unit's field parts through the closing block, and no draw.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((_, o, _), _)| *o >= 0)
            .filter(|((_, _, k), _)| k != "form")
            .map(|((who, o, k), (f, _))| format!("{f} {who}/{o} {k}"))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        [
            "4983 1/2005 queue:queue[2].cost[0]",
            "5064 1/2005 queue:queue[1].cost[0]",
        ],
        "the only object rows: a price"
    );
    pin_eq!(w.firsts.len(), 131, "run1320 baseline");
    // **Every round, both directions**: 940 round-blocks a side, and none
    // parts — the Musketeers' (`1/73`..`1/75`, piece 1457) among them,
    // which left the figure's own point before item 3655's bays.
    pin_eq!(w.rounds_read, (940, 940), "the rounds read");
    pin!(w.rounds.is_empty(), "no round parts");
}

/// **The closing state** (item 3655): run1291's closing block 5129, the
/// whole map's units, walked from run1290's start.
#[test]
fn iroquois_enlightenment_pair_run1291_s_closing_state() {
    let _pins = Pins::hold();
    let p = &IROQUOIS_ENLIGHTENMENT;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Iroquois Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    for &(who, o) in &e.extra {
        if let Some(u) = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
        {
            let name = u.ty.and_then(|ty| loaded.unit_names.get(ty).cloned());
            eprintln!("    extra {who}/{o} {name:?} at {:?}", u.pos);
        }
    }
    pin_eq!(e.frame, 5129, "the closing block");
    // Item 3655, the pair whole: 199 units, none off, torn, unlinked or
    // extra, `Closing fields: none`. Before piece 1457's bays the human's
    // Explorer `0/0` (−11, 32) and Citizen `0/3` (−22, 12) were off, each
    // struck by a Musketeer's round from the wrong point.
    pin_eq!(e.compared, 199, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
