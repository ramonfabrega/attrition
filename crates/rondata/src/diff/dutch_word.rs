//! **The battery's fifth word** (DECISIONS 72, 83; item 3329): the
//! held-out battery's lowest game after the Greeks (item 3326's), promoted
//! by item 3314's and 3318's recipe. East Indies (`MAP_STYLE 18`), human
//! Nubians (4) against **the Dutch (22)** at Difficulty 4, seed 12345 —
//! the lobby item 3107 minted (`docs/AI.md` §246), scored by its first try
//! at 1567 of 24,000 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run915 (a `DUMP_ALL` start, blocks 0..36) and run917
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks
//! only); the game does not end itself. The word's windows are run963's
//! (`docs/AI.md` §267).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const DUTCH: Lobby = Lobby {
    name: "EastIndiesDutchTier4",
    map_style: 18,
    ai_tribe: 22,
    difficulty: 4,
    start: "gamelog-run915-eastindies-dutch-diff4-start.txt",
    long: (
        "gamelog-run917-eastindies-dutch-diff4-24k-trace.txt",
        "rontrace-run917.log",
    ),
    length: 24000,
    // Item 3107, the battery's first try: count and sequence both at 1567
    // (ours 6 draws, theirs 5, at index 2: ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Farms::inc_time+0x1ae`). Item 3329
    // promoted it to the AI word (`docs/AI.md` §267). **Item 3329 moved it
    // 1567 → 1769** with the Dutch merchant's leg of `Unit::think`'s
    // auto-attack arm (`think@005f6e40:145`, `docs/MERCHANT.md` §4): the
    // packed `1/2` (`MERCHANTDUTCH`) asks `think_merchant` one frame in
    // thirty-two, so on 1566 it took its `MOVE_TO` (31800, 37176) in theirs
    // and stood idle in ours, whose only call was the tail's one in 128
    // (run963's block 1567: `orders.len` 0 against 1). **And 1769 → 5583**
    // with the unit piece table keyed without case
    // (`rondata::artdata::graph_types`): the Dutch caravan `1/14`
    // (`CARADUTCH`, `GRAPH` `ArmedCaravan`), born on 1764, played no piece
    // in ours — `gpiece` 0 against theirs' 10 and 12682 on block 1765, its
    // clock 33 against 31 — and on 1769 its idle draw fell a frame apart.
    // At 5583 ours 10 draws, theirs 9, at 4: ours `Leader::make_stuff+0x63d`,
    // theirs `GameDaemon::calc_market+0x54`, past run963's last block.
    // **And 5583 → 5782** with the Dutch interest in `economy::pay`
    // (`Leader::do_gather@006ce450:116`–`147`): run963b's block 5577 had
    // who=1's `bucket` (food 145 ours against 149, timber 199/218, wealth
    // 63/128, metal 154/155) and `income` (timber 1440/1552, wealth
    // 880/976, metal 480/512) apart, all agreeing after. **And 5782 →
    // 22935** with the Dutch ship discount taken out of the Military
    // block (`get_cost:532`–`535`): on block 5783 the dock `1/2011`'s
    // `DROMON` research `queue[0].cost[0..1]` 60 ours against 54, and
    // who=1's `bucket` food 57/63, timber 69/75; both 54 after. At 22935
    // ours 13 draws, theirs 11, at 2: ours `Guy::init_real+0x52`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`. **And 22935 → 23782**
    // with the library's slots read before its recursion (`do_queue:100`,
    // `:283`–`290`, `:344`): run963c's Library `1/2005` held the
    // Industrial Age (slot 0, 95300) and Electricity (slot 1, 50000); on
    // 22935 Electricity completes in both, and theirs advances the age to
    // 95400 under the Science-4 target where ours, reading it after the
    // epoch landed, took 90000 and finished the age — block 22936's
    // `queued` 0 ours against 1, `ages` 5 against 4, `pop` 17 against 13;
    // both finish it on 22936 after. At 23782 ours 17 draws, theirs 24, at
    // 3: ours `Leader::make_stuff+0x221`, theirs `Leader::use_market
    // +0x1ed`, past run963c's last block. **And 23782 → 23985** with the
    // Colossus in the population cap (`calc_pop_cap@006dc490:89`, called
    // from `Build::activate:1795` for the Colossus), built by this item and
    // by item 3326 in parallel and kept as 3326's at the merge: run963d's
    // block 23776, who=1's `pop_cap` 150 ours against 200, and its make
    // list topped by `NATION_IN_ARMS` at 9999999 from 23779; both 200
    // after. At 23985 ours 13 draws, theirs 15, at 0: ours
    // `Leader::produce_building+0x1805`, theirs `Leader::use_market+0x1ed`.
    // **Item 3333 moved it 23985 → 24000, the long's last frame**, with
    // an Oil Platform priced by the leader's Oil Wells too
    // (`get_cost@00664090`, `0066598f`..`006659be`): on 23982 both sides
    // place the Platform `1/2079` from the make list's Oil Well entry, and
    // ours, counting no Platform, charged it 100 timber and 50 metal where
    // theirs, counting the Well `1/2078`, charged 120 and 70 — run963d's
    // block 23983, who=1's `bucket` timber 199 ours against 179, metal 136
    // against 116; on 23985 theirs could not pay the next Well and went to
    // the market. run917's closing block 24001 is compared whole
    // (`run917_dutch_closing_state`).
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const DUTCH_WORD: i64 = if DUTCH.count < DUTCH.sequence {
    DUTCH.count
} else {
    DUTCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn dutch_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(DUTCH.start, DUTCH.long) else {
        return;
    };
    eprintln!(
        "DUTCH {}: count {} sequence {} of {}; {}",
        DUTCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, DUTCH.difficulty, "the lobby read back");
    pin_eq!(w.last, DUTCH.length, "the game's last frame");
    pin_eq!(w.count, DUTCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        DUTCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn dutch_word_is_no_other_lobby() {
    let p = &DUTCH;
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

/// run963: the word 1567's window, blocks 1561..1817 at `GUYS=4` (item
/// 3329, run958e's recipe); run915 dumps blocks 0..36 and run917 its setup
/// and closing blocks only.
pub(crate) const RUN963: &str = "gamelog-run963-eastindies-dutch-diff4-window-1561-1818-guys4.txt";

/// The window: block 1561 through 1817; the word 1567's own block is 1568.
pub(crate) const WIDENING_DUTCH_FRAME_1567: (i64, i64) = (1561, 1817);

pub(crate) fn dutch_frame_1567_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963",
        &[(RUN963, WIDENING_DUTCH_FRAME_1567.0)],
        WIDENING_DUTCH_FRAME_1567,
        1,
        &[1568],
        true,
    )
}

#[test]
fn run963_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_1567_window() else {
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
    pin_eq!(w.firsts.len(), 38, "keys that part in the window");
    // Item 3329: 531 keys part on blocks 1561..1817, the word's own block
    // being 1568. Standing from 1561 (36): the control's set —
    // `SITE[n].reg`, `form`, the groups' `held`, the pool and both
    // leaders' `aggression`. The first that is not: block 1567, who=1's
    // Dutch merchant `1/2` (`TypeIndex` 62, land) holds a `MOVEORDER` to
    // (31800, 37176) in theirs and nothing in ours (`orders.len` 0/1,
    // `orders_x` 40344/31800), then its path, idle and figures.
    //
    // **531 → 239** with the Dutch merchant's leg of the auto-attack arm:
    // the order is on both sides on 1567. The first that parted next was
    // block 1765, the newborn caravan `1/14`'s `g.gpiece[0]` 0/10 and
    // `g.gpiece[1]` 0/12682, `g.end_time[0]` 33/31. **239 → 38** with the
    // piece table keyed without case: both read 10 and 12682. What stands
    // is the control's set and its two late rows, who=1's `SITE[6].reg` on
    // 1576 and `group:64.held` on 1577.
}

/// run963b: the word 5583's window, blocks 5577..5833 at run963's detail
/// (item 3329); run963 ends on block 1817 and run917 dumps its setup and
/// closing blocks only.
pub(crate) const RUN963B: &str =
    "gamelog-run963b-eastindies-dutch-diff4-window-5577-5834-guys4.txt";

/// The window: block 5577 through 5833; the word 5583's own block is 5584.
pub(crate) const WIDENING_DUTCH_FRAME_5583: (i64, i64) = (5577, 5833);

pub(crate) fn dutch_frame_5583_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963b",
        &[(RUN963B, WIDENING_DUTCH_FRAME_5583.0)],
        WIDENING_DUTCH_FRAME_5583,
        1,
        &[5584],
        true,
    )
}

#[test]
fn run963b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_5583_window() else {
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
    // 674 at the clear (the economy rows, `myhits` and the standing set);
    // 662 on the Dutch hit-point term, 233 on the interest (the word then
    // at 5782, its block 5783 in the window), 63 on the ship discount:
    // the standing set alone — `SITE[n].reg`, `form`, `group:n.held`,
    // `pool:n`, `aggression` and the newborn `g.angle[0]`, 53 on 5577 and
    // ten later as new members join those kinds (`1/27`'s `form` on 5601
    // through `1/36`'s `g.angle[0]` on 5811).
    pin_eq!(w.firsts.len(), 63, "keys that part in the window");
}

/// run963c: the word 22935's window, blocks 22929..23185 at run963's
/// detail (item 3329); run963b ends on block 5833.
pub(crate) const RUN963C: &str =
    "gamelog-run963c-eastindies-dutch-diff4-window-22929-23186-guys4.txt";

/// The window: block 22929 through 23185; the word 22935's own block is
/// 22936.
pub(crate) const WIDENING_DUTCH_FRAME_22935: (i64, i64) = (22929, 23185);

pub(crate) fn dutch_frame_22935_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963c",
        &[(RUN963C, WIDENING_DUTCH_FRAME_22935.0)],
        WIDENING_DUTCH_FRAME_22935,
        1,
        &[22936],
        true,
    )
}

#[test]
fn run963c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_22935_window() else {
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
    // 2191 on the first build (the Industrial Age a frame early on
    // 22935); **116** with the library's slots read before its recursion:
    // the standing set and 101 keys already apart on 22929, which no dump
    // between 5834 and 22928 can place — the Dutch merchants' `1/1`,
    // `1/2`, `1/96`, `1/141` figure pieces (`g.gpiece` 2124/14796/27468
    // ours against 50689 for all three figures theirs) and their second
    // figure's offsets, who=1's `num_units[270]` −1 and `[271]` 1 against
    // 0 and 0 (`ARTILLERYGERMAN`, `HOWITZER`), and `1/201`'s `myhits` 172
    // against 212.
    // Item 3336: −4, `Wall::set_type` leaving the hits to the next `calc_wall_stats` (`docs/AI.md` §272).
    // Item 3369: 112 -> 74, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 74, "keys that part in the window");
}

/// run963d: the word 23782's window to the trace's last frame, blocks
/// 23776..24000 at run963's detail (item 3329); run963c ends on block
/// 23185.
pub(crate) const RUN963D: &str =
    "gamelog-run963d-eastindies-dutch-diff4-window-23776-24001-guys4.txt";

/// The window: block 23776 through 24000; the word 23782's own block is
/// 23783.
pub(crate) const WIDENING_DUTCH_FRAME_23782: (i64, i64) = (23776, 24000);

pub(crate) fn dutch_frame_23782_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963d",
        &[(RUN963D, WIDENING_DUTCH_FRAME_23782.0)],
        WIDENING_DUTCH_FRAME_23782,
        1,
        &[23783],
        true,
    )
}

#[test]
fn run963d_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_23782_window() else {
        return;
    };
    pin_eq!(w.blocks, 225, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // 1819 on build 4 (who=1's `pop_cap` 150 against 200 from 23776, the
    // Colossus); **197** with it: run963c's 101 keys that stand from before
    // its window, the standing set, and the word 23985's own rows from 23983.
    // **94** with the Oil Platform priced by the Oil Wells (item 3333): the
    // word's rows — who=1's `bucket` timber 199/179 and metal 136/116 on
    // 23983, the make list on 23984, and the units that followed it from
    // 23987 — close; what stands is run963c's rows from before the window
    // (`num_units[270]`/`[271]`, the merchants' second figures) and the
    // standing set, the last on 23839.
    // Item 3369: 94 -> 92, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 92, "keys that part in the window");
}

/// The closing state's window: run917's closing block alone, the long's
/// last frame 24000 writing it (item 3333).
pub(crate) const WIDENING_DUTCH_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3333, DECISIONS 55): run917 — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run917_dutch_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(DUTCH.long.0), dump(DUTCH.start), trace(DUTCH.long.1))
    else {
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
        "Dutch closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 209, "units compared");
    // Item 3333, the first walk (the word at 24000): 209 units compared on
    // block 24001, none apart, no building or city field-row, no closing
    // field.
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}
