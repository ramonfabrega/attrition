//! **The Greeks at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, item 3703): East Indies, `MAP_STYLE 18`, human Nubians
//! (4) against the **Greeks (5)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(18, 5, 4)`, is new: the Greeks hold this map at 5
//! (`greek_classical_pair`), Great Sahara at 1 (`greek_word`) and 5
//! (`greek_gunpowder_pair`) and Great Lakes at 5
//! (`greek_enlightenment_pair`); the battery fields no Greeks.
//! Captures: run1379 (a `DUMP_ALL` start) and run1380 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §363.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GREEK_MEDIEVAL: Pair = Pair {
    name: "EastIndiesGreekMedieval",
    map_style: 18,
    ai_tribe: 5,
    difficulty: 4,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1379-eastindies-greek-medieval-start.txt",
    long: (
        "gamelog-run1380-eastindies-greek-medieval-trace.txt",
        "rontrace-run1380.log",
    ),
    // The game ended itself on 5942 (closing block 5943).
    length: 5942,
    // Item 3703, first try: 5626 of 5942 by the count and the sequence —
    // ours 29 draws against 28, index 9: ours `Unit::do_move+0xe84`,
    // theirs `Guy::set_anim+0x97a < Guy::move+0x19f` (`docs/AI.md` §363).
    count: 5626,
    sequence: 5626,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GREEK_MEDIEVAL_PAIR_WORD: i64 = if GREEK_MEDIEVAL.count < GREEK_MEDIEVAL.sequence {
    GREEK_MEDIEVAL.count
} else {
    GREEK_MEDIEVAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn greek_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GREEK_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GREEK_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn greek_medieval_pair_captures_say_their_lobby() {
    let p = &GREEK_MEDIEVAL;
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
        "the Greek Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3703; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Greeks — and no other Greek lobby's: the Classical pair's
/// `(18, 5, 5)`, this map at another difficulty, nor `greek_word`'s
/// `(7, 5, 1)`, the Gunpowder pair's `(7, 5, 5)` or the Enlightenment
/// pair's `(14, 5, 5)`, other maps.
#[test]
fn greek_medieval_pair_is_no_other_lobby() {
    let p = &GREEK_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the Greeks"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Greek lobby on this map at this difficulty"
        );
    }
    let c = &super::greek_classical_pair::GREEK_CLASSICAL;
    let g = &super::greek_gunpowder_pair::GREEK_GUNPOWDER;
    let e = &super::greek_enlightenment_pair::GREEK_ENLIGHTENMENT;
    let w = &super::greek_word::GREEK;
    for (name, l) in [
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Greeks");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3703, from
/// 5626): only run1380's stride blocks are on disk before the word's
/// window, 101 through 5601.
pub(crate) const WIDENING_GREEK_MEDIEVAL_STRIDE: (i64, i64) = (101, 5601);

pub(crate) fn greek_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK_MEDIEVAL.start],
        true,
        GREEK_MEDIEVAL.long,
        "run1380 stride",
        &[(GREEK_MEDIEVAL.long.0, WIDENING_GREEK_MEDIEVAL_STRIDE.0)],
        WIDENING_GREEK_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn greek_medieval_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = greek_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 56, "every stride block in the window");
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
    // Item 3703: 253 keys over 56 blocks, every one a standing family —
    // `form` and the buildings' `build:extra` (which the stride does not
    // print) on 101 and as each is born, and who=1's groups 64.. with
    // their pools as each is minted. No unit row parts before the word's
    // window.
    pin_eq!(w.firsts.len(), 253, "the stride's baseline");
}

/// run1380b: the word 5626's window, blocks 5595..5877 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 5601 and 250 past the word's own block 5627 (item 3703).
pub(crate) const RUN1380B: &str = "gamelog-run1380b-eastindies-greek-medieval-window-5595-5877.txt";

/// The window: block 5595 through 5877; the word 5626's own block is 5627.
pub(crate) const WIDENING_GREEK_MEDIEVAL_FRAME_5626: (i64, i64) = (5595, 5877);

/// **The word 5626's window**: run1380b walked from run1379's start with
/// the recorder on and every round compared (item 3703).
pub(crate) fn greek_medieval_frame_5626_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GREEK_MEDIEVAL.start],
        true,
        GREEK_MEDIEVAL.long,
        "run1380b",
        &[(RUN1380B, WIDENING_GREEK_MEDIEVAL_FRAME_5626.0)],
        WIDENING_GREEK_MEDIEVAL_FRAME_5626,
        1,
        &[5627],
        true,
    )
}

#[test]
fn greek_medieval_pair_run1380b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_medieval_frame_5626_window() else {
        return;
    };
    pin_eq!(w.blocks, 282, "every captured block");
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
    // Item 3703, the first walk: **3660** keys. Block 5595 carries the
    // standing families (`form`, `build:extra`, who=1's groups and pools,
    // who=0's `SITE[i].reg` and `aggression[1]`); no bucket parts. **The
    // word**: the Heavy Fire Raft `1/70` (TI 335, TY 285, sea), idle and
    // orderless on both sides through 5626, spends ours' eighth
    // `Unit::do_move+0xe84` draw on 5626 (index 9) and sets `avoid` to its
    // own square; theirs spends none and gives it a move to (46056, 38088).
    // So on 5627 its `orders.len` reads 0 against 1, `orders_x/y` 45675,
    // 37688 against 46056, 38088 and `heading` 145883136 against
    // 1153564672, and on 5628 its `pos` parts.
    pin_eq!(w.firsts.len(), 3660, "run1380b baseline");
    // Every round, both directions: 1205 against 1199. The first to part
    // are launch points a unit or two off — the Heavy Horse Archer
    // `1/151`'s `ammo[4]` on 5601 (`sx` 6566 against 6565) and the
    // Crossbowmen `1/157`'s `ammo[0]` on 5612 — before the word, and the
    // draws never feel them (parked 3703a).
    pin_eq!(w.rounds_read, (1205, 1199), "the rounds read");
    pin_eq!(w.rounds.len(), 243, "the rounds that part");
}

/// **The pair's closing state** (item 3703): run1380's closing block,
/// the whole map's units, walked from run1379's start.
#[test]
fn greek_medieval_pair_run1380_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GREEK_MEDIEVAL;
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
        "Greek Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5943, "the closing block");
    // Item 3703, downstream of the word 5626: 319 units compared, counts
    // `[119, 3, 1, 7, 6, 1, 0]`, 3 unlinked and 1 extra, who=1's army off
    // and two of its buildings apart (parked 3703b); 1 inexact ground
    // corner (`GROUND_INEXACT`).
    pin_eq!(e.compared, 319, "units compared");
    pin_eq!(e.counts(), [119, 3, 1, 7, 6, 1, 0], "the closing counts");
    pin_eq!(e.extra.len(), 1, "the closing extras");
}
