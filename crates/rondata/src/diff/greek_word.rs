//! **The battery's Greek word** (DECISIONS 72, 83; item 3326): the held-out
//! battery's lowest game once item 3318 closed the Spanish, promoted by item
//! 2904's, 3107's, 3314's and 3318's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Greeks (5)** at Difficulty 1, seed 12345
//! — the lobby item 3318 minted (`docs/AI.md` §264), scored by its first
//! try at 176 of 24,000 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run957 (a `DUMP_ALL` start, blocks 0..36) and run957b
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks only);
//! the word's window is run961's (`docs/AI.md` §266).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GREEK: Lobby = Lobby {
    name: "GreatSaharaGreekTier1",
    map_style: 7,
    ai_tribe: 5,
    difficulty: 1,
    start: "gamelog-run957-greatsahara-greek-diff1-start.txt",
    long: (
        "gamelog-run957b-greatsahara-greek-diff1-24k-trace.txt",
        "rontrace-run957b.log",
    ),
    length: 24000,
    // Item 3318: the battery's first try, count and sequence both at 176;
    // ours 15 draws, theirs 36, at 0 ours `Animal::think_bird+0x82`, theirs
    // `Leader::compute_sites+0x4ac`. Item 3326 promoted it to the AI word.
    // The Greek research-time arm (`train_time:453`: an age or an epoch at
    // `t × 100 / (GREEK_RESEARCH_SPEED + 100)`) moved it 176 → 1976: theirs
    // holds City State on 176 and places its second city five times
    // (run961, block 177). The Greek research cost (`get_cost:1254`, a tenth
    // off an age's or an epoch's price but its knowledge) left 1976 at ours
    // 145 draws, theirs 147: run961c's block 37 queues City State at food
    // 108 on both sides (ours was 120). The Greek Library arm (`get_cost:970`,
    // `GREEK_LIBRARY_COST` off a Library or a University) moved it 1976 →
    // 19182: who=1's University `1/2013` placed on 1976 at timber 27, wealth
    // 17 on both sides (run961b, block 1977). On 19182 ours draws 8, theirs
    // 9; index 2 is ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Leader::make_stuff+0x63d`. The Colossus' cap (`calc_pop_cap`'s
    // `COLOSSUS_POP_CAP`, never set before) moved it 19182 → 24000, the
    // long's last frame: who=1's `pop_cap` 110 against 160 from run961d's
    // first block, and theirs makes a second item at its capital on 19182.
    // run957b's closing block 24001 is compared whole
    // (`run957b_greek_closing_state`).
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const GREEK_WORD: i64 = if GREEK.count < GREEK.sequence {
    GREEK.count
} else {
    GREEK.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn greek_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(GREEK.start, GREEK.long) else {
        return;
    };
    eprintln!(
        "GREEK {}: count {} sequence {} of {}; {}",
        GREEK.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, GREEK.difficulty, "the lobby read back");
    pin_eq!(w.last, GREEK.length, "the game's last frame");
    pin_eq!(w.count, GREEK.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        GREEK.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn greek_word_is_no_other_lobby() {
    let p = &GREEK;
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

/// run961: the word 176's window, blocks 170..426 at run958d's detail
/// (`LEADERS=9`, `GUYS=4`; item 3326); run957 dumps blocks 0..36 and run957b
/// its setup and closing blocks only.
pub(crate) const RUN961: &str = "gamelog-run961-greatsahara-greek-diff1-window-170-427-guys4.txt";

/// The window: block 170 through 426; the word 176's own block is 177.
pub(crate) const WIDENING_GREEK_FRAME_176: (i64, i64) = (170, 426);

pub(crate) fn greek_frame_176_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK.start],
        true,
        GREEK.long,
        "run961",
        &[(RUN961, WIDENING_GREEK_FRAME_176.0)],
        WIDENING_GREEK_FRAME_176,
        1,
        &[177],
        true,
    )
}

#[test]
fn run961_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_frame_176_window() else {
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
    // Item 3326. On the first build the word's block 177 parts: theirs'
    // who=1 places its second city five times (`economic.bhs` step 10's
    // `city_placement`, City State held), ours still researches City State.
    // With the Greek research-time arm (`train_time:453`, an age or an
    // epoch in half the time) the word moved 176 → 1976 and the window
    // parted 36 keys: the standing set and who=1's `bucket[0:food]`, ours
    // 92 against theirs 104 from block 170 (walked back on run961c to
    // block 37). With the Greek research cost the window parts **the
    // standing set alone**.
    pin_eq!(w.firsts.len(), 35, "run961 with the Greek research arms");
}

/// run961b: the word 1976's window, blocks 1970..2226 at run961's detail
/// (item 3326) — the Greek research arm moved the word past run961's last
/// block, and run957b dumps no block between.
pub(crate) const RUN961B: &str =
    "gamelog-run961b-greatsahara-greek-diff1-window-1970-2228-guys4.txt";

/// The window: block 1970 through 2226; the word 1976's own block is 1977.
pub(crate) const WIDENING_GREEK_FRAME_1976: (i64, i64) = (1970, 2226);

pub(crate) fn greek_frame_1976_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK.start],
        true,
        GREEK.long,
        "run961b",
        &[(RUN961B, WIDENING_GREEK_FRAME_1976.0)],
        WIDENING_GREEK_FRAME_1976,
        1,
        &[1977],
        true,
    )
}

#[test]
fn run961b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_frame_1976_window() else {
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
    // Item 3326. With the research arms, past the standing set block 1977
    // parts who=1's `bucket[1:timber]` 40 against 13 and `bucket[2:wealth]`
    // 100 against 83: theirs places the University `1/2013` on 1976 at
    // timber 27 and wealth 17 (`produce_building+0x1805`), a third of 80
    // and 50 — `get_cost`'s Greek Library arm. With it the window parts
    // **the standing set alone**, and the word moved 1976 → 19182.
    pin_eq!(w.firsts.len(), 30, "run961b with the Greek Library arm");
}

/// run961c: the gap from run957's last block to run961's first, blocks
/// 37..170 at run961's detail with `GUYS=2` (item 3326) — where the food
/// standing apart on run961's block 170 was walked back to.
pub(crate) const RUN961C: &str = "gamelog-run961c-greatsahara-greek-diff1-window-37-171.txt";

/// The window: block 37 through 170.
pub(crate) const WIDENING_GREEK_GAP_961C: (i64, i64) = (37, 170);

pub(crate) fn greek_gap_961c_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK.start],
        true,
        GREEK.long,
        "run961c",
        &[(RUN961C, WIDENING_GREEK_GAP_961C.0)],
        WIDENING_GREEK_GAP_961C,
        1,
        &[],
        true,
    )
}

#[test]
fn run961c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_gap_961c_window() else {
        return;
    };
    pin_eq!(w.blocks, 134, "every captured block");
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
    // Item 3326, with the research-time arm: past the standing set, block
    // 37 parts who=1's `bucket[0:food]`, ours 82 against theirs 94, and the
    // Library `1/2005`'s queue head (City State) `cost[0]` 120 against 108
    // — `get_cost`'s Greek research arm, a tenth off every good but
    // knowledge, refunded on the queue's price. With it the gap parts **the
    // standing set alone**.
    pin_eq!(w.firsts.len(), 35, "run961c with the Greek research cost");
}

/// run961d: the word 19182's window, blocks 19176..19432 at run961's
/// detail (item 3326) — the Greek Library arm moved the word past run961b's
/// last block, and run957b dumps no block between.
pub(crate) const RUN961D: &str =
    "gamelog-run961d-greatsahara-greek-diff1-window-19176-19433-guys4.txt";

/// The window: block 19176 through 19432; the word 19182's own block is 19183.
pub(crate) const WIDENING_GREEK_FRAME_19182: (i64, i64) = (19176, 19432);

pub(crate) fn greek_frame_19182_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK.start],
        true,
        GREEK.long,
        "run961d",
        &[(RUN961D, WIDENING_GREEK_FRAME_19182.0)],
        WIDENING_GREEK_FRAME_19182,
        1,
        &[19183],
        true,
    )
}

#[test]
fn run961d_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_frame_19182_window() else {
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
    // Item 3326. Past the standing set, block 19176 parts who=1's
    // `pop_cap`, ours 110 against theirs 160: 100 at Military 3, Peacocks'
    // tenth, and theirs adds `COLOSSUS_POP_CAP`'s 50 (`calc_pop_cap`'s last
    // term, `has_wonder(0x20f)`). On 19181 theirs' `MAKE[0].val` is 888888
    // (ours 8461) and on 19182 theirs makes a second item at its capital;
    // ours, capped, did not. With the Colossus term the window parts **the
    // standing set alone**, and the word moved 19182 → 24000.
    pin_eq!(w.firsts.len(), 41, "run961d with the Colossus' cap");
}

/// The closing state's window: run957b's closing block alone, the long's
/// last frame 24000 writing it (item 3326).
pub(crate) const WIDENING_GREEK_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3326, DECISIONS 55): run957b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run957b_greek_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(GREEK.long.0), dump(GREEK.start), trace(GREEK.long.1))
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
        "Greek closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 114, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3326, the first walk (with the Colossus' cap, the word at
    // 24000): 114 units compared on block 24001, none apart, no building or
    // city field-row, no closing field.
}
