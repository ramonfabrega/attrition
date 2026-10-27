//! **The Chinese at a held Enlightenment age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, item 3675): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Chinese (14)** at **Toughest (5)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its triple,
//! `(14, 14, 5)`, is new: the Chinese lobbies are `chinese_medieval_pair`'s
//! `(7, 14, 5)`, `chinese_industrial_pair`'s `(7, 14, 3)` and
//! `chinese_gunpowder_pair`'s `(18, 14, 5)`, other maps, and the held-out
//! battery's `GreatLakesChineseTier1`, `(14, 14, 1)`, another difficulty.
//! Captures: run1349 (a `DUMP_ALL` start) and run1350 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §354; item 3704 closes it whole, 3836 of 3836 (§354.2),
//! widened on run1381b through the closing block.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const CHINESE_ENLIGHTENMENT: Pair = Pair {
    name: "GreatLakesChineseEnlightenment",
    map_style: 14,
    ai_tribe: 14,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1349-greatlakes-chinese-enlightenment-start.txt",
    long: (
        "gamelog-run1350-greatlakes-chinese-enlightenment-trace.txt",
        "rontrace-run1350.log",
    ),
    // The game ended itself on 3836 (closing block 3837).
    length: 3836,
    // Item 3675, first try: 3263 by the count and the sequence, ours 6
    // draws against theirs 7, at index 3 ours `Farms::inc_time+0x1ae`,
    // theirs `Object::take_damage+0xe1` (`docs/AI.md` §354). Item 3704
    // closes it, 3263 → 3836, the game's last frame (§354.2): pieces 66
    // (the Manchu Musketeers, TI 116) and 1142 (the Fusiliers, TI 136) take
    // three bays each (`sim::launch`, off run1381's packet), so the
    // Musketeer `1/42`'s round of 3261 leaves (5094, 31444, 678), the
    // dump's, and flies 4 frames, where it left from the figure's own
    // (5208, 31512, 464) and flew 5; it strikes the city centre `0/2000`
    // on the word, as theirs does.
    count: 3836,
    sequence: 3836,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const CHINESE_ENLIGHTENMENT_PAIR_WORD: i64 =
    if CHINESE_ENLIGHTENMENT.count < CHINESE_ENLIGHTENMENT.sequence {
        CHINESE_ENLIGHTENMENT.count
    } else {
        CHINESE_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn chinese_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &CHINESE_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "CHINESE_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn chinese_enlightenment_pair_captures_say_their_lobby() {
    let p = &CHINESE_ENLIGHTENMENT;
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
        "the Chinese Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3675; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields
/// the Chinese only as `GreatLakesChineseTier1`, `(14, 14, 1)`, another
/// difficulty; and no other Chinese lobby's: the Medieval pair's
/// `(7, 14, 5)`, the Industrial pair's `(7, 14, 3)` and the Gunpowder
/// pair's `(18, 14, 5)`, other maps (item 3675).
#[test]
fn chinese_enlightenment_pair_is_no_other_lobby() {
    let p = &CHINESE_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let fielded: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        fielded,
        [("GreatLakesChineseTier1", (14, 14, 1))],
        "the battery's Chinese games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Chinese lobby on this map at this difficulty"
        );
    }
    let m = &super::chinese_medieval_pair::CHINESE_MEDIEVAL;
    let i = &super::chinese_industrial_pair::CHINESE_INDUSTRIAL;
    let g = &super::chinese_gunpowder_pair::CHINESE_GUNPOWDER;
    for (name, l) in [
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Chinese");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3263** (item 3675):
/// only run1350's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_CHINESE_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 3201);

pub(crate) fn chinese_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_ENLIGHTENMENT.start],
        true,
        CHINESE_ENLIGHTENMENT.long,
        "run1350 stride",
        &[(
            CHINESE_ENLIGHTENMENT.long.0,
            WIDENING_CHINESE_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_CHINESE_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn chinese_enlightenment_pair_the_long_s_stride_walks_back_from_3263() {
    let _pins = Pins::hold();
    let Some(w) = chinese_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 32, "every stride block in the window");
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
    // Item 3675: 92 keys, every one a standing family — `form` on 27
    // units, 51 buildings' `build:extra` (which the stride does not print)
    // and who=1's groups 64..70's `held` and pools as each is minted. No
    // unit row parts on a stride block through 3201.
    pin_eq!(w.firsts.len(), 92, "the stride's baseline");
}

/// run1350b: the word 3263's window, blocks 3195..3514 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3201 and 250 past the word's own block 3264 (item 3675).
pub(crate) const RUN1350B: &str =
    "gamelog-run1350b-greatlakes-chinese-enlightenment-window-3195-3514.txt";

/// The window: block 3195 through 3514; the word 3263's own block is 3264.
pub(crate) const WIDENING_CHINESE_ENLIGHTENMENT_FRAME_3263: (i64, i64) = (3195, 3514);

/// **The word 3263's window**: run1350b walked from run1349's start with
/// the recorder on and every round compared (item 3675).
pub(crate) fn chinese_enlightenment_frame_3263_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[CHINESE_ENLIGHTENMENT.start],
        true,
        CHINESE_ENLIGHTENMENT.long,
        "run1350b",
        &[(RUN1350B, WIDENING_CHINESE_ENLIGHTENMENT_FRAME_3263.0)],
        WIDENING_CHINESE_ENLIGHTENMENT_FRAME_3263,
        1,
        &[3264],
        true,
    )
}

#[test]
fn chinese_enlightenment_pair_run1350b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_enlightenment_frame_3263_window() else {
        return;
    };
    pin_eq!(w.blocks, 319, "every captured block");
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
    // Item 3675, the first walk: **1575** keys. Block 3195 carries the
    // standing families (`form`, the human's `SITE[i].reg` and
    // `aggression[1]`, who=1's `SITE[3]`/`SITE[6]` ranks and `SITE[6]`'s
    // value and square, groups and pools; 69's on 3228). **Who=1's Manchu
    // Musketeers (TI 116) and Fusiliers (TI 136) loose every round from
    // their own square in ours**: the Fusilier `1/52`'s on 3195 from
    // (6216, 29448, 492) against theirs' (6117, 29386, 609), and so on
    // through the window. The human's Citizen `0/6` (TY 50) is wounded on
    // 3245 in theirs and 3246 in ours (`hits:damage` 10 on block 3246, ours
    // 0). On the word 3263 the Musketeer `1/42`'s round at the human's
    // city centre `0/2000`, loosed on 3261 — ours (5208, 31512, 464) in 5
    // frames, theirs (5094, 31444, 678) in 4 — lands in theirs
    // (`Object::take_damage+0xe1`, index 3) and ours holds it to 3264:
    // `0/2000`'s `build:damage` 4 in theirs on block 3264, ours 0. Item
    // 3704, pieces 66 and 1142's bays: 1575 → 58 keys, nothing parting on
    // 3246 or 3264; what stands is the families block 3195 carries (and
    // 69's on 3228), with who=1's `SITE[8].rank` on 3376 (ours 6, theirs
    // 7; 3675a's family).
    pin_eq!(w.firsts.len(), 58, "run1350b baseline");
    // Every round, both directions: 263 round-blocks theirs, 305 ours (each
    // of ours flies a frame longer), 209 parting, the first on the
    // window's first block 3195 (item 3675). Item 3704: 263 against 263,
    // 2 parting — the Manchu Musketeer `1/48`'s `ammo[1]` on 3286, `whom`
    // and `ox` (ours −1, theirs `0/8`), its launch and landing agreeing
    // (parked 3704a).
    pin_eq!(w.rounds_read, (263, 263), "the rounds read");
    pin_eq!(w.rounds.len(), 2, "the rounds that part");
}

/// run1381b: the pair's last blocks, 3580..3837 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — item 3704 closes the word at the game's last
/// frame 3836, and 3837 is its closing block. run1350 prints units only
/// every hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, and run1350b ends on 3514, so no dump held the
/// last blocks.
pub(crate) const RUN1381B: &str =
    "gamelog-run1381b-greatlakes-chinese-enlightenment-closing-window-3580-3837.txt";

/// The window: block 3580 through the closing block 3837.
pub(crate) const WIDENING_CHINESE_ENLIGHTENMENT_CLOSING: (i64, i64) = (3580, 3837);

/// **The pair's last blocks**: run1381b walked from run1349's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3704).
pub(crate) fn chinese_enlightenment_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[CHINESE_ENLIGHTENMENT.start],
        true,
        CHINESE_ENLIGHTENMENT.long,
        "run1381b",
        &[(RUN1381B, WIDENING_CHINESE_ENLIGHTENMENT_CLOSING.0)],
        WIDENING_CHINESE_ENLIGHTENMENT_CLOSING,
        1,
        &[3837],
        true,
    )
}

#[test]
fn chinese_enlightenment_pair_run1381b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_enlightenment_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
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
    // Item 3704, the first walk: **62** keys, every one a standing family
    // on block 3580 — `form`, who=1's groups and pools 64..70 (69's on
    // 3799), the human's `SITE[i].reg` and `aggression[1]` (ours 0, theirs
    // 1), and who=1's `SITE[0..6]` ranks, values and squares (3675a's
    // family; `SITE[3].reg` on 3776). No unit, building or city field and
    // no round parts, and the closing block 3837 agrees.
    pin_eq!(w.firsts.len(), 62, "run1381b baseline");
    // Every round, both directions: 231 round-blocks each side, none
    // parting.
    pin_eq!(w.rounds_read, (231, 231), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3675): run1350's closing block 3837,
/// the whole map's units, walked from run1349's start.
#[test]
fn chinese_enlightenment_pair_run1350_s_closing_state() {
    let _pins = Pins::hold();
    let p = &CHINESE_ENLIGHTENMENT;
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
        "Chinese Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3837, "the closing block");
    // Item 3675, the first try: the pair parts at 3263, and the closing
    // block 3837 compares 151 units — who=1's army off (`1/0`, `1/3`..`1/148`),
    // `1/44` torn, `1/149`..`1/153` unlinked, the human's `0/10` off and
    // `0/9` extra; who=1's buildings 2043..2051 and cities 2000, 2021 and
    // 2046 part in the closing fields. Item 3704, the pair whole: 156
    // units, none off, torn, unlinked or extra, `Closing fields: none` —
    // the game ends itself on 3836.
    pin_eq!(e.compared, 156, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
