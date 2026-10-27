//! **The Mongols at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, item 3702): East Indies, `MAP_STYLE 18`, human Nubians
//! (4) against the **Mongols (17)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(18, 17, 4)`, is new: the Mongols hold this map at 3 (the successor
//! pair, `mongol_pair`) and 5 (`mongol_gunpowder_pair`), Great Sahara at 3
//! (`sahara_mongol`) and 5 (`mongol_classical_pair`) and Great Lakes at 5
//! (`mongol_enlightenment_pair`); the battery fields no Mongols.
//! Captures: run1377 (a `DUMP_ALL` start), run1378 (the draw stream to the
//! game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked
//! from the start's initial state) and run1378b (the word's window). The
//! first try is in `docs/AI.md` §362.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const MONGOL_MEDIEVAL: Pair = Pair {
    name: "EastIndiesMongolMedieval",
    map_style: 18,
    ai_tribe: 17,
    difficulty: 4,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1377-eastindies-mongol-medieval-start.txt",
    long: (
        "gamelog-run1378-eastindies-mongol-medieval-trace.txt",
        "rontrace-run1378.log",
    ),
    // The game ended itself on 5592 (closing block 5593).
    length: 5592,
    // Item 3702, first try: 2595 of 5592 by the count and the sequence —
    // ours 55 draws against 54, index 53: ours `Unit::think_fish+0x27a`,
    // theirs `Farms::inc_time+0x1ae` (`docs/AI.md` §362).
    count: 2595,
    sequence: 2595,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MONGOL_MEDIEVAL_PAIR_WORD: i64 =
    if MONGOL_MEDIEVAL.count < MONGOL_MEDIEVAL.sequence {
        MONGOL_MEDIEVAL.count
    } else {
        MONGOL_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn mongol_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MONGOL_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MONGOL_MEDIEVAL {}: count {} sequence {} of {}; {}",
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
fn mongol_medieval_pair_captures_say_their_lobby() {
    let p = &MONGOL_MEDIEVAL;
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
        "the Mongol Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3702; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Mongols — and no other Mongol pair's: the successor pair's
/// `(18, 17, 3)` and the Gunpowder pair's `(18, 17, 5)`, this map at other
/// difficulties, nor the Classical pair's `(7, 17, 5)`, Great Sahara's
/// `(7, 17, 3)` or the Enlightenment pair's `(14, 17, 5)`, other maps.
#[test]
fn mongol_medieval_pair_is_no_other_lobby() {
    let p = &MONGOL_MEDIEVAL;
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
        "the battery fields the Mongols"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Mongol lobby on this map at this difficulty"
        );
    }
    let s = &super::mongol_pair::MONGOL;
    let g = &super::mongol_gunpowder_pair::MONGOL_GUNPOWDER;
    let c = &super::mongol_classical_pair::MONGOL_CLASSICAL;
    let e = &super::mongol_enlightenment_pair::MONGOL_ENLIGHTENMENT;
    let h = &super::sahara_mongol::SAHARA_MONGOL;
    for (name, l) in [
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (h.name, (h.map_style, h.ai_tribe, h.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Mongols");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3702, from
/// 2595): only run1378's stride blocks are on disk before the word's
/// window, 101 through 2501.
pub(crate) const WIDENING_MONGOL_MEDIEVAL_STRIDE: (i64, i64) = (101, 2501);

pub(crate) fn mongol_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_MEDIEVAL.start],
        true,
        MONGOL_MEDIEVAL.long,
        "run1378 stride",
        &[(MONGOL_MEDIEVAL.long.0, WIDENING_MONGOL_MEDIEVAL_STRIDE.0)],
        WIDENING_MONGOL_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_medieval_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = mongol_medieval_stride_window() else {
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
    // Item 3702: 100 keys over 25 blocks, every one a standing family —
    // `form` and the leaders on 101, the buildings' `build:extra` (which
    // the stride does not print), who=1's groups 64..78's `held` and pools
    // as each is minted, and `form` on 1/56 and 1/60 from 2201. No unit
    // row parts before the word's window.
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1378b: the word 2595's window, blocks 2495..2846 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2501 and 250 past the word's own block 2596 (item 3702).
pub(crate) const RUN1378B: &str =
    "gamelog-run1378b-eastindies-mongol-medieval-window-2495-2846.txt";

/// The window: block 2495 through 2846; the word 2595's own block is 2596.
pub(crate) const WIDENING_MONGOL_MEDIEVAL_FRAME_2595: (i64, i64) = (2495, 2846);

/// **The word 2595's window**: run1378b walked from run1377's start with
/// the recorder on and every round compared (item 3702).
pub(crate) fn mongol_medieval_frame_2595_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[MONGOL_MEDIEVAL.start],
        true,
        MONGOL_MEDIEVAL.long,
        "run1378b",
        &[(RUN1378B, WIDENING_MONGOL_MEDIEVAL_FRAME_2595.0)],
        WIDENING_MONGOL_MEDIEVAL_FRAME_2595,
        1,
        &[2596],
        true,
    )
}

#[test]
fn mongol_medieval_pair_run1378b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_medieval_frame_2595_window() else {
        return;
    };
    pin_eq!(w.blocks, 351, "every captured block");
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
    // Item 3702, the first walk: **910** keys. Block 2495 carries the
    // standing families (`form`, both leaders' `SITE[i].reg` and who=0's
    // `aggression[1]`, who=1's groups 64..78 and pools) and **who=1's
    // buckets**, already apart: food 17182 against 17188, timber 14828
    // against 14832, wealth 18222 against 18256, metal 16561 against 16602
    // — no leader record is on disk between the start and 2495. On 2586
    // three buildings' queued costs part (132/88 against 126/84). **The
    // word**: the Fishermen `1/85` (TI 317, TY 267, sea), born on 2595,
    // runs `Unit::think_fish` on its first frame and spends 52
    // `think_fish+0x27a` draws in ours against 51 in theirs — one more
    // accepted cell — so on 2596 its first `order:move.x` reads 34968 in
    // ours against 30360 in theirs (`dest_angle` -1170669568 against
    // -1099563008), and on 2597 its `pos` parts.
    pin_eq!(w.firsts.len(), 910, "run1378b baseline");
    // Every round, both directions: none fly in the window.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3702): run1378's closing block 5593,
/// the whole map's units, walked from run1377's start past the word.
#[test]
fn mongol_medieval_pair_run1378_s_closing_state() {
    let _pins = Pins::hold();
    let p = &MONGOL_MEDIEVAL;
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
        "Mongol Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5593, "the closing block");
    // Item 3702, downstream of the word 2595: 157 units compared, counts
    // `[131, 5, 16, 9, 102, 1, 55]`, 5 unlinked and 16 extra, who=1's
    // cities and buildings apart; 2 inexact ground corners
    // (`GROUND_INEXACT`).
    pin_eq!(e.compared, 157, "units compared");
    pin_eq!(
        e.counts(),
        [131, 5, 16, 9, 102, 1, 55],
        "the closing counts"
    );
    pin_eq!(e.extra.len(), 16, "the closing extras");
}
