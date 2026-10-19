//! **The battery's Egyptian word** (DECISIONS 72, 83; item 3332): the
//! held-out battery's lowest open game on the measure of `987c4c33`,
//! promoted by item 2904's, 3107's, 3314's, 3318's, 3326's and 3331's
//! recipe. Great Sahara (`MAP_STYLE 7`), human Nubians (4) against **the
//! Egyptians (7)** at Difficulty 4, seed 12345 — one of item 1465's first
//! three lobbies (`docs/AI.md` §125), scored by its first try at 2974 of
//! 20,380 and promoted out of the battery here, so it is a scored lobby from
//! this item (`battery::SCORED_LOBBIES`) and no longer held out. Captures:
//! run648 (a `DUMP_ALL` start) and run645 (the draw stream to the game's
//! end on 20380, `cover=0`, its setup and closing blocks only); the word's
//! window is run967's (`docs/AI.md` §269).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const EGYPTIAN: Lobby = Lobby {
    name: "GreatSaharaEgyptianTier4",
    map_style: 7,
    ai_tribe: 7,
    difficulty: 4,
    start: "gamelog-run648-greatsahara-egyptian-diff4-start.txt",
    long: (
        "gamelog-run645-greatsahara-egyptian-diff4-24k-trace.txt",
        "rontrace-run645.log",
    ),
    length: 20380,
    // Item 1465: the battery's first try; the measure on `987c4c33`, count
    // and sequence both at 2974: ours 9 draws, theirs 12, at 2 ours
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389`,
    // theirs `Guy::set_anim+0x104b`. Item 3332 promoted it to the AI word.
    // run967 (2968..3225) parts who=1's economy on its first block, walked
    // back through run967b's stride to block 54's wealth income, 0 against
    // 96: the Egyptian Farm's wealth (`calc_gather:997`–`1005`,
    // `EGYPTIAN_FARM_WEALTH`). With it the word's row is the Merchant
    // `1/17`'s destination on 2973, good 12 against good 13: the good's
    // `ever_seen` bit `reveal_fog` sets (`:87`–`88`). The word moved 2974
    // → 15782: ours 62 draws, theirs 63, at 1 ours
    // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`.
    // run967c (15776..16033) parts on who=1's Metal Alloys, 54000 against
    // 216000 on the make list: `check_income` 64 against 256 (run967d's
    // `RON_LEADER_PROBE`), the price 200 against 68 (run967e's
    // `get_cost`) — the Hanging Gardens' industry discount
    // (`get_cost:1270`–`1276`, 66%). The word moved 15782 → 15982: ours
    // 495 draws, theirs 450, at 4 ours `Leader::produce_building+0xc99`,
    // theirs `Build::find_gather_tiles+0x10a`. There who=1 offered a Farm
    // theirs refused at `gather_value`'s three-quarters gate: the census's
    // `gather_slots[0]`, 19 against 20, a second copy of the ledger's
    // field (`LeaderData +0x8a4`) that the Farm finishing on 15976 never
    // reached. With the one field, the word moved 15982 → 20031: ours 12
    // draws, theirs 15, at 4 ours `Animal::do_idle+0x83`, theirs
    // `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4`. Item 3336 walked
    // it back on run967g to who=1's Fusilier `1/162` on tick 20029: theirs
    // sets `collide_frame` to 20029 and drops the move, ours counts
    // `collide` 4 → 5 — `do_move`'s `ATTACK` retarget, a non-mandatory
    // attack's `find_new_target(NULL, 1)` four frames after the collision
    // (`005f7b30`). The word moved 20031 → 20186: ours 11 draws, theirs 12,
    // at 1 ours `Unit::do_move+0xe84`, theirs `Unit::fight+0x9b0`: the
    // Royal Mameluke `1/68`, a `GROUP_ATTACK_TO` follower, re-searching
    // from the Citizen `0/4` it held — `do_group_move`'s own fire on the
    // move (`005e79a0:392`–`406`). The word moved 20186 → 20235: ours 15
    // draws, theirs 16, at 4 ours `Guy::set_anim+0x97a < Guy::move+0x19f`,
    // theirs `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`. Walked back to
    // block 20219: the army's escort round the Cannon `1/125` on tick 20218
    // read it moving in ours, standing in theirs — theirs keeps a packer's
    // pack cast at the head through the group move's `QUEUE_NEW` clear
    // (`action_move_near@00704990:410`–`440`, `:727`–`752`). With that keep
    // the draw stream agrees to the game's end.
    count: 20380,
    sequence: 20380,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const EGYPTIAN_WORD: i64 = if EGYPTIAN.count < EGYPTIAN.sequence {
    EGYPTIAN.count
} else {
    EGYPTIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn egyptian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(EGYPTIAN.start, EGYPTIAN.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN {}: count {} sequence {} of {}; {}",
        EGYPTIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, EGYPTIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, EGYPTIAN.length, "the game's last frame");
    pin_eq!(w.count, EGYPTIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        EGYPTIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn egyptian_word_is_no_other_lobby() {
    let p = &EGYPTIAN;
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

/// run967: the word 2974's window, blocks 2968..3225 at run965's detail
/// (`LEADERS=9`, `GUYS=4`; item 3332); run648 dumps blocks 0..2 and run645
/// its setup and closing blocks only.
pub(crate) const RUN967: &str =
    "gamelog-run967-greatsahara-egyptian-diff4-window-2968-3225-guys4.txt";

/// The window: block 2968 through 3225; the word 2974's own block is 2975.
pub(crate) const WIDENING_EGYPTIAN_FRAME_2974: (i64, i64) = (2968, 3225);

pub(crate) fn egyptian_frame_2974_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN.start],
        true,
        EGYPTIAN.long,
        "run967",
        &[(RUN967, WIDENING_EGYPTIAN_FRAME_2974.0)],
        WIDENING_EGYPTIAN_FRAME_2974,
        1,
        &[2975],
        true,
    )
}

#[test]
fn run967_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_frame_2974_window() else {
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
    // Item 3332. On the first try 736 keys part, the first who=1's
    // economy on block 2968 past the standing set — `ages` 0 against 1,
    // `tech_frame` 2776 against 2576, `discovered` 1 against 3,
    // `income[2:wealth]` 160 against 384 — walked back through run967b to
    // the Egyptian Farm's wealth (`calc_gather:997`–`1005`); with it the
    // economy closes and **the word's own row is the Merchant `1/17`'s
    // order on 2973**: `MOVEORDER` (31032, 21048) against (34968, 25752),
    // the good 12 ours took where theirs took good 13, the head of
    // who=1's `new_rares`, met through its footprint on 2114 and
    // unmarked in ours (`reveal_fog:87`–`88`, the good's `ever_seen`).
    // With both: **33** keys, the standing set (both leaders'
    // `SITE[n].reg` and `aggression`, every unit's `form`, who=1's groups'
    // `held` and pools) and `1/16`'s `g.gpiece` from 3056 (ours 11 against
    // 363), and the word moved 2974 → 15782.
    pin_eq!(
        w.firsts.len(),
        33,
        "run967 with the Egyptian farm wealth and the good's ever_seen"
    );
}

/// run967b: the gap from run648's last block to run967's first, one block
/// in fifty (`--stride 50`): blocks 4, 54, …, 2954, at run967's detail with
/// `GUYS=2` (item 3332) — where who=1's age, standing apart on run967's
/// block 2968, was walked back to.
pub(crate) const RUN967B: &str =
    "gamelog-run967b-greatsahara-egyptian-diff4-gap-3-2967-stride50.txt";

/// The window: block 4 through 2954, one in fifty.
pub(crate) const WIDENING_EGYPTIAN_GAP_967B: (i64, i64) = (4, 2954);

pub(crate) fn egyptian_gap_967b_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN.start],
        true,
        EGYPTIAN.long,
        "run967b",
        &[(RUN967B, WIDENING_EGYPTIAN_GAP_967B.0)],
        WIDENING_EGYPTIAN_GAP_967B,
        1,
        &[],
        true,
    )
}

#[test]
fn run967b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_gap_967b_window() else {
        return;
    };
    pin_eq!(w.blocks, 60, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496's pattern): the animals' clocks.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3332. On the first try 55 keys part, the first past the
    // standing set on block 54: who=1's `income[2:wealth]` 0 against 96
    // (`leftover` 0 against 5640), three worked Egyptian Farms at
    // `EGYPTIAN_FARM_WEALTH` 2 each (`calc_gather:997`–`1005`); behind it
    // the food bucket from 1704 and the age's queue on 2604 (`tech_frame`
    // 1576 against 2576). With the arm, **39**: the standing set and
    // who=0's `other_team_terr` 441 against 388 from 1404.
    pin_eq!(w.firsts.len(), 39, "run967b with the Egyptian farm wealth");
}

/// run967c: the word 15782's window, blocks 15776..16033 at run967b's
/// detail (`GUYS=2`; item 3332); run645 prints its setup and closing blocks
/// only.
pub(crate) const RUN967C: &str =
    "gamelog-run967c-greatsahara-egyptian-diff4-window-15776-16033.txt";

/// The window: block 15776 through 16033; the word 15782's own block is
/// 15783.
pub(crate) const WIDENING_EGYPTIAN_FRAME_15782: (i64, i64) = (15776, 16033);

pub(crate) fn egyptian_frame_15782_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN.start],
        true,
        EGYPTIAN.long,
        "run967c",
        &[(RUN967C, WIDENING_EGYPTIAN_FRAME_15782.0)],
        WIDENING_EGYPTIAN_FRAME_15782,
        1,
        &[15783],
        true,
    )
}

#[test]
fn run967c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_frame_15782_window() else {
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
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3332. On the first try 820 keys part. Past the standing set the
    // first are on block 15776, `1/126`..`1/129`'s `g.angle[0]` (ours
    // `0x55555555`, `Angle::INITIAL`, against theirs 0), and on 15779 the
    // word's own row: who=1's `MAKE[2].val`, Metal Alloys (dump type 620),
    // ours 54000 against theirs 216000 — `research_techs`' pass on 15778
    // (ours: weights 5 × base 3600, the gather-line ×2, recency ×6, and
    // `check_income`'s 64, unaffordable at timber 149 of 200). Theirs
    // keeps it, ours sinks it below the Crossbowmen on 15781, and theirs
    // goes to the market on 15782 where ours makes. With the Hanging
    // Gardens' industry discount (68 food and 68 timber, not 200) **276**,
    // and the word moved 15782 → 15982: on 15982 who=1's make list carries
    // ours' building 417 (cat 4, city 4, 312960) where theirs' holds 418
    // and Metal Alloys, which theirs researches on 15982 (`tech_frame`)
    // while ours places the building in city 4. With the census writing
    // the ledger's `gather_slots` (`plan_strategy:1162`) **36**: the
    // standing set, the newborns' `g.angle[0]` and the pool and group
    // lists, and nothing on 15982 — the Farm offers and the make list
    // agree whole, and the word moved 15982 → 20031, past the window.
    pin_eq!(
        w.firsts.len(),
        36,
        "run967c with the census's gather slots the ledger's"
    );
}

/// run967g: the word 20031's window, blocks 20025..20282 at run967c's
/// detail (item 3332); run967c ends on 16033.
pub(crate) const RUN967G: &str =
    "gamelog-run967g-greatsahara-egyptian-diff4-window-20025-20282.txt";

/// The window: block 20025 through 20282; the word 20031's own block is
/// 20032.
pub(crate) const WIDENING_EGYPTIAN_FRAME_20031: (i64, i64) = (20025, 20282);

pub(crate) fn egyptian_frame_20031_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN.start],
        true,
        EGYPTIAN.long,
        "run967g",
        &[(RUN967G, WIDENING_EGYPTIAN_FRAME_20031.0)],
        WIDENING_EGYPTIAN_FRAME_20031,
        1,
        &[20032],
        true,
    )
}

#[test]
fn run967g_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_frame_20031_window() else {
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
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3332. On the first try **3286** keys part. Past the standing
    // set the first are on block 20025, `1/92` and `1/160`'s
    // `attack[1].mandatory` (ours 0, theirs 1) — apart before the window
    // opens — and on 20030 the word's unit, who=1's `1/162`: ours two
    // orders and a three-node path where theirs holds none, `orders_x/y`
    // (7608, 28584) against (7080, 26280), `collide` 5 against 4 and
    // `collide_frame` 20025 against 20029. On 20031 `1/162`'s order kind
    // is 1 against theirs' 10 and it is idle on theirs only, and `1/163`'s
    // path is 1 node against 17 (`start_dist` 1344 against 672); the
    // positions part on 20032. The word's draws: theirs
    // `Unit::find_attack_pos+0xea9 < Unit::fight+0xcb4` where ours idles
    // an animal. The next item's frame; not a mechanism yet.
    pin_eq!(w.firsts.len(), 3286, "run967g on its first try");
}

/// The closing state's window: run645's closing block alone, the long's
/// last frame 20380 writing it (item 3336).
pub(crate) const WIDENING_EGYPTIAN_CLOSING: (i64, i64) = (20379, 20382);

/// **The lobby's closing state** (item 3336, DECISIONS 55): run645 — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
#[test]
fn run645_egyptian_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(EGYPTIAN.long.0),
        dump(EGYPTIAN.start),
        trace(EGYPTIAN.long.1),
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
        "Egyptian closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 20381, "the closing block");
    pin_eq!(e.compared, 180, "units compared");
    // Item 3336, the first walk (the word at 20380): 180 units compared on
    // block 20381, one apart — the human's Citizen `0/1`, (13, 21) short
    // of theirs, one step of its `FLEE_TO` — no building or city
    // field-row, no closing field.
    pin_eq!(e.counts(), [1, 0, 0, 0, 0, 0, 0], "the closing counts");
}
