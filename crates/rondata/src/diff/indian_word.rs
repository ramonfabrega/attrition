//! **The battery's Indian word** (DECISIONS 72, 83; item 3335): the
//! held-out battery's lowest game once item 3333 closed the Dutch, promoted
//! by item 3314's, 3318's, 3326's, 3329's and 3331's recipe. East Indies
//! (`MAP_STYLE 18`), human Nubians (4) against **the Indians (21)** at
//! Difficulty 2, seed 12345 — the lobby item 3329 minted (`docs/AI.md`
//! §267), scored by its first try at 221 of 24,000 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run962 (a
//! `DUMP_ALL` start, blocks 0..36) and run962b (the draw stream to 24000,
//! `cover=0`, its setup and closing blocks only); the game does not end
//! itself. The word's window is run970's (`docs/AI.md` §271).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INDIAN: Lobby = Lobby {
    name: "EastIndiesIndianTier2",
    map_style: 18,
    ai_tribe: 21,
    difficulty: 2,
    start: "gamelog-run962-eastindies-indian-diff2-start.txt",
    long: (
        "gamelog-run962b-eastindies-indian-diff2-24k-trace.txt",
        "rontrace-run962b.log",
    ),
    length: 24000,
    // Item 3329, the battery's first try: count and sequence both at 221
    // (ours 5 draws, theirs 17, at 0: ours `Farms::inc_time+0x1ae`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`). Item 3335 promoted it to
    // the AI word (`docs/AI.md` §271) and moved it 221 → 1176 (a goody
    // box's own `ever_seen`, `reveal_fog`'s goody arm) → 1177 (`get_cost`'s
    // Indian ramp arm) → 9985 (a closed building's number held thirty
    // frames, run970b's 1176) → 14594 (every attacking type counted in
    // its trainer's group, run970c's 9983 Horse Archer) → 14631
    // (`find_gather_spot`'s region is `get_tregion`'s coastal-refined one,
    // run970e's barge `1/77` beside the Dock): ours 12 draws, theirs 11,
    // at 10: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Farms::inc_time+0x1ae`. Item 3340 moved it 14631 → 24000, the
    // long's last frame (`think_carry_civilian`'s boat region is
    // `get_tregion`'s, run970e's barge `1/77` on 14595); the close is
    // compared whole by `run962b_indian_closing_state`.
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INDIAN_WORD: i64 = if INDIAN.count < INDIAN.sequence {
    INDIAN.count
} else {
    INDIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn indian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INDIAN.start, INDIAN.long) else {
        return;
    };
    eprintln!(
        "INDIAN {}: count {} sequence {} of {}; {}",
        INDIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INDIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, INDIAN.length, "the game's last frame");
    pin_eq!(w.count, INDIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INDIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn indian_word_is_no_other_lobby() {
    let p = &INDIAN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
    }
}

/// run970: the word 221's window, blocks 37..472 at `GUYS=4` (item 3335,
/// run965's recipe from the start dump's last block, so the gap and the
/// word are one capture); run962 dumps blocks 0..36 and run962b its setup
/// and closing blocks only.
pub(crate) const RUN970: &str = "gamelog-run970-eastindies-indian-diff2-window-37-473-guys4.txt";

/// The window: block 37 through 472; the word 221's own block is 222.
pub(crate) const WIDENING_INDIAN_FRAME_221: (i64, i64) = (37, 472);

pub(crate) fn indian_frame_221_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970",
        &[(RUN970, WIDENING_INDIAN_FRAME_221.0)],
        WIDENING_INDIAN_FRAME_221,
        1,
        &[222],
        true,
    )
}

#[test]
fn run970_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_frame_221_window() else {
        return;
    };
    pin_eq!(w.blocks, 435, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3335: 496 keys part on blocks 37..472, the word's own block
    // being 222. The first that is not the standing set: block 180,
    // ours' scout `1/0` (TI 69) sweeping the goody box `o 1` at (34944,
    // 38016) and re-targeting where theirs walks on — the item's
    // `ever_seen` is its own byte, 0 in the dump while the cells' `seen2`
    // read 2. **496 → 33** with the item's byte (the word 221 → 1176),
    // **33 → 32** with `get_cost`'s Indian ramp arm (who=1's timber 29
    // against 41 from block 37; the word 1176 → 1177, past the window).
    // What stands is the control's set — `SITE[n].reg`, `aggression`,
    // `form`, the groups' `held` and the pool — 30 on block 37 and two
    // late rows, who=1's `group:66.held` on 177 and `pool:64` on 222.
    pin_eq!(w.firsts.len(), 32, "keys that part in the window");
}

/// run970b: the word 1177's window, blocks 1171..1428 at run970's detail
/// (item 3335); run970 ends on block 472 and run962b dumps its setup and
/// closing blocks only.
pub(crate) const RUN970B: &str =
    "gamelog-run970b-eastindies-indian-diff2-window-1171-1428-guys4.txt";

/// The window: block 1171 through 1428; the word 1177's own block is 1178.
pub(crate) const WIDENING_INDIAN_FRAME_1177: (i64, i64) = (1171, 1428);

pub(crate) fn indian_frame_1177_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970b",
        &[(RUN970B, WIDENING_INDIAN_FRAME_1177.0)],
        WIDENING_INDIAN_FRAME_1177,
        1,
        &[1178],
        true,
    )
}

#[test]
fn run970b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_frame_1177_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3335: 400 keys part on blocks 1171..1428, the word's own block
    // being 1178. The first that is not the standing set: block 1177,
    // who=1's citizen `1/7` holds a `BUILDORDER` on `1/2009` (uid 19) and
    // its walk to (38184, 36744) in theirs and nothing in ours
    // (`orders.len` 0/2, `orders_x` 40248/38184), and the Woodcutter's
    // Camp ours numbers 2009 is theirs' `1/2010` (the Farm `1/2008`'s
    // `city_down` 2009/2010). The script places a Camp at the queued
    // Small City 2007, reads `max_workers_at_building` 4, destroys it and
    // places another on the same frame (1176): theirs' 2009 is held thirty
    // frames by its close, ours was reused at once. **400 → 35** with the
    // close's hold (the word 1177 → 9985): both read 2010 and `1/7`'s two
    // orders. What stands is the control's set — 32 on 1171 and three
    // late rows, who=1's `SITE[5].reg` on 1176, `SITE[6].reg` on 1376 and
    // `group:66.held` on 1377.
    pin_eq!(w.firsts.len(), 35, "keys that part in the window");
}

/// run970c: the word 9985's window, blocks 9979..10236 at run970's detail
/// (item 3335); run970b ends on block 1428.
pub(crate) const RUN970C: &str =
    "gamelog-run970c-eastindies-indian-diff2-window-9979-10236-guys4.txt";

/// The window: block 9979 through 10236; the word 9985's own block is 9986.
pub(crate) const WIDENING_INDIAN_FRAME_9985: (i64, i64) = (9979, 10236);

pub(crate) fn indian_frame_9985_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970c",
        &[(RUN970C, WIDENING_INDIAN_FRAME_9985.0)],
        WIDENING_INDIAN_FRAME_9985,
        1,
        &[9986],
        true,
    )
}

#[test]
fn run970c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_frame_9985_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3335: 586 keys part on blocks 9979..10236, the word's own
    // block being 9986, and the state has parted before the window
    // opens: on 9979 who=1's `bucket[0:food]` 74 against 89 and
    // `bucket[2:wealth]` 56 against 65, and twelve of who=1's units'
    // `g.angle[0]` 1431655765 against 0. Then the make list on 9981
    // (`MAKE[3].num` 5/1) and 9982 (`MAKE[3].t` 438/261), the queue's
    // price on 9983 and the empty head on 9986 — the word's. The gap
    // 1428..9979 is run970d's.
    //
    // **586 → 569** with the Indian elephant arm (the food and wealth rows
    // close; run970d's 9589), and **569 → 50** with every attacking type
    // counted in its trainer's group: block 9983's Horse Archer (TI 187,
    // `PROGRESSION 3`) at the Stable `1/2019` cost 51/34 ours against
    // 52/35, theirs' `stable_units` 1 — the War Elephant, `PROGRESSION 2`,
    // which this crate kept out of the group. Both read 52/35, and the
    // word moved 9985 → 14594. What stands is the control's set and the
    // newborns' `g.angle[0]`.
    pin_eq!(w.firsts.len(), 50, "keys that part in the window");
}

/// run970d: the gap 1428..9979 at a stride of 32 (item 3335): blocks
/// 1429, 1461, …, 9941, at run970's detail; run970b ends on block 1428
/// and run970c opens on 9979 already parted.
pub(crate) const RUN970D: &str = "gamelog-run970d-eastindies-indian-diff2-stride32-1428-9979.txt";

/// The window: block 1429 through 9941, one in thirty-two.
pub(crate) const WIDENING_INDIAN_GAP_970D: (i64, i64) = (1429, 9941);

pub(crate) fn indian_gap_970d_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970d",
        &[(RUN970D, WIDENING_INDIAN_GAP_970D.0)],
        WIDENING_INDIAN_GAP_970D,
        1,
        &[9941],
        true,
    )
}

#[test]
fn run970d_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_gap_970d_window() else {
        return;
    };
    pin_eq!(w.blocks, 267, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3335: 67 keys part over the gap. Apart from the control's set
    // and twelve newborns' `g.angle[0]` (1431655765 against 0, each on the
    // first block that prints the unit, 6485 to 8981): block 9589, who=1's
    // `bucket[0:food]` 53 against 68 and `bucket[2:wealth]` 51 against 60,
    // the War Elephant (TI 261) queued at the Stable `1/2019` costing
    // 96/56 ours against 81/47 — fifteen percent, `get_cost`'s Indian
    // elephant arm. **67 → 62** with the arm: the standing set alone.
    pin_eq!(w.firsts.len(), 62, "keys that part in the gap");
}

/// run970e: the word 14594's window, blocks 14588..14845 at run970's
/// detail (item 3335); run970c ends on block 10236.
pub(crate) const RUN970E: &str =
    "gamelog-run970e-eastindies-indian-diff2-window-14588-14845-guys4.txt";

/// The window: block 14588 through 14845; the word 14594's own block is
/// 14595.
pub(crate) const WIDENING_INDIAN_FRAME_14594: (i64, i64) = (14588, 14845);

pub(crate) fn indian_frame_14594_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970e",
        &[(RUN970E, WIDENING_INDIAN_FRAME_14594.0)],
        WIDENING_INDIAN_FRAME_14594,
        1,
        &[14595],
        true,
    )
}

#[test]
fn run970e_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_frame_14594_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3335: 1125 keys part on blocks 14588..14845, the word's own
    // block being 14595: 54 on 14588, the control's set and the newborns'
    // `g.angle[0]`. The first that is not: block 14594, who=1's citizen
    // `1/77` holds one order in ours and none in theirs (`orders.len`
    // 1/0). It finished the Dock `1/2030` (TI 432) on frame 14593 (theirs'
    // `job_counter` 37700 → 0, `flags` 3 → 7, on both sides), and
    // `build_done`'s AI arm (`unit_masks` 0x840408 holds 0x40000) sent
    // ours to gather at the Woodcutter's Camp `1/2026` — 2 of 4, 3097
    // away, timber headroom 832 — where theirs' found no spot, left it
    // idle, and on 14595 the AI walks it to (21912, 32664). The leader's
    // caps and incomes, the camp's gatherers and the unit's masks agree;
    // The input was the region: `1/77` is a barge (TI 320) on a coastal cell's
    // water tile, `tregion` 12 and `get_tregion` (`tregion_alt`) 0, and no
    // camp is in region 0. With it, 1125 → 929 and the word 14594 →
    // 14631. Walked back from 14631, the first parting is block 14595:
    // theirs' AI gives the idle `1/77` a move to (21912, 32664)
    // (`orders.len` 0/1, `orders_x` 38952/21912), ours leaves it standing
    // — `think_peasant`'s tail, a hypothesis.
    //
    // Item 3340: the arm is `think@005f6e40`'s carry arm, not the peasant's
    // — the barge carries the citizen `1/42` (TI 50), so `think_carry` asks
    // `think_carry_civilian(1)`, whose shore search starts from
    // `get_tregion` of the boat's tile (`005f37c0`, `local_64`): sea region
    // 0 on the coastal cell's ocean tile, where ours read the cell's plain
    // `tregion`, land region 12, which no land region coasts. Value diff,
    // block 14595: `1/77`'s `orders.len` 0/1 → 1/1, `orders_x/_y` (38952,
    // 33048)/(21912, 32664) → agreeing. **929 → 56**: the 54 standing on
    // 14588 and who=1's `group:71.held` and `pool:71` on 14640, the
    // control's late rows as run970's 177 and 222. The word 14631 → 24000.
    pin_eq!(w.firsts.len(), 56, "keys that part in the window");
}

/// The closing state's window: run962b's closing block alone, the long's
/// last frame 24000 writing it (item 3340).
pub(crate) const WIDENING_INDIAN_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3340, DECISIONS 55): run962b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run962b_indian_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(INDIAN.long.0),
        dump(INDIAN.start),
        trace(INDIAN.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
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
        "Indian closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 24001, "the closing block");
    pin_eq!(e.compared, 113, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3340, the first walk (with `think_carry_civilian`'s region from
    // `get_tregion`, the word at 24000): 113 units compared on block
    // 24001, none apart, no building or city field-row, no closing field.
}
