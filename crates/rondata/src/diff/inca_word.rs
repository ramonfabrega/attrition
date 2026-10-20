//! **The battery's Inca word** (DECISIONS 72, 83, 84; item 3360): the
//! held-out battery's lowest open game on the measure of `3eb2c76c`,
//! promoted by item 3335's recipe. East Indies (`MAP_STYLE 18`), human
//! Nubians (4) against **the Inca (2)** at Difficulty 3, seed 12345 —
//! minted by item 3335 (a nation no dump held, a tier the map's battery
//! lacked), scored by its first try and promoted out of the battery here,
//! so it is a scored lobby from this item (`battery::SCORED_LOBBIES`) and
//! no longer held out. Captures: run969 (a `DUMP_ALL` start) and run969b
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks
//! only); the game does not end itself (`docs/AI.md` §281).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA: Lobby = Lobby {
    name: "EastIndiesIncaTier3",
    map_style: 18,
    ai_tribe: 2,
    difficulty: 3,
    start: "gamelog-run969-eastindies-inca-diff3-start.txt",
    long: (
        "gamelog-run969b-eastindies-inca-diff3-24k-trace.txt",
        "rontrace-run969b.log",
    ),
    length: 24000,
    // Item 3335's first try, held by the battery's measure to `3eb2c76c`:
    // count 7782, sequence 7782 (ours 3218 draws, theirs 3219, at 1: ours
    // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`). Item
    // 3360 promoted it to the AI word (`docs/AI.md` §281), and built
    // `calc_gather`'s Inca arm (a mine's wealth per miner): 7782 → 16516,
    // ours 12 draws, theirs 11, at 3: ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::set_anim+0x97a < Unit::do_idle+0x7d`.
    // Value diff, run987 block 7776: who=1's `income[2:wealth]` ours 752,
    // theirs 912 → 912/912. Then `Guy::move`'s moving arm put back on the
    // walk a moving figure that holds a turn slot (the listing
    // `5d9565`–`5d958a` reads nothing of `cur_anim`): 16516 → **24000**,
    // the long's last frame. Value diff, run987b block 16512: the
    // Trebuchet `1/118`'s `g.cur_anim[0]` ours 22, theirs 8 → 8/8.
    count: 24000,
    sequence: 24000,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INCA_WORD: i64 = if INCA.count < INCA.sequence {
    INCA.count
} else {
    INCA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn inca_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INCA.start, INCA.long) else {
        return;
    };
    eprintln!(
        "INCA {}: count {} sequence {} of {}; {}",
        INCA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INCA.difficulty, "the lobby read back");
    pin_eq!(w.last, INCA.length, "the game's last frame");
    pin_eq!(w.count, INCA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INCA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn inca_word_is_no_other_lobby() {
    let p = &INCA;
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

/// run987: the word 7782's window, blocks 7776..8032 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3360). run969 dumps blocks 0..36 and
/// run969b its setup and closing blocks only, so no dump on disk held the
/// leader's ledger on the word's block.
pub(crate) const RUN987: &str = "gamelog-run987-eastindies-inca-diff3-window-7776-8033-leaders.txt";

/// The window: block 7776 through 8032; the word 7782's own block is 7783.
pub(crate) const WIDENING_INCA_FRAME_7782: (i64, i64) = (7776, 8032);

pub(crate) fn inca_frame_7782_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA.start],
        true,
        INCA.long,
        "run987",
        &[(RUN987, WIDENING_INCA_FRAME_7782.0)],
        WIDENING_INCA_FRAME_7782,
        1,
        &[7783],
        true,
    )
}

#[test]
fn run987_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_frame_7782_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3360: 533 keys parted on the first try, the window opening apart
    // on who=1's wealth — `income[2:wealth]` ours 752, theirs 912, `rate`
    // 47/57, `bucket` 22/37 on block 7776 — sixteen miners at
    // `INCA_WEALTH_PER_MINER` (10), the arm `BuildTypeData::calc_gather`'s
    // mine branch adds beside `german_miners` and this crate did not carry.
    // Built: 533 → 49. On 7782 the wealth then covers less of timber's
    // need, so `use_market` spends its second sell pass, as theirs does.
    // What stands: the standing family alone (`SITE[·].reg` 65 against 0,
    // `aggression`, `form` −1 against 0/9, the army `pool:n` and
    // `group:n.held` with no `GROUPDATA` printed, the newborn `g.angle[0]`).
    pin_eq!(w.firsts.len(), 49, "keys that part in the window");
}

/// run987b: the word 16516's window, blocks 16510..16766 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3360). run987 closes on 8032, before
/// the word.
pub(crate) const RUN987B: &str =
    "gamelog-run987b-eastindies-inca-diff3-window-16510-16767-leaders.txt";

/// The window: block 16510 through 16766; the word 16516's own block is
/// 16517.
pub(crate) const WIDENING_INCA_FRAME_16516: (i64, i64) = (16510, 16766);

pub(crate) fn inca_frame_16516_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA.start],
        true,
        INCA.long,
        "run987b",
        &[(RUN987B, WIDENING_INCA_FRAME_16516.0)],
        WIDENING_INCA_FRAME_16516,
        1,
        &[16517],
        true,
    )
}

#[test]
fn run987b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_frame_16516_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3360: 1994 keys on the first try. The first after the standing
    // family: the Trebuchet `1/118` (TI 266), orderless and drifting, takes
    // its turn animation (22) on 16512 in ours, where theirs keeps 8 until
    // 16517 — pushed by `1/74` every frame, its push turn asked for by
    // `do_turn` and put back on `CHAR_WALK` the same frame by
    // `Guy::move`'s moving arm, which spares no turn slot. Built: 1994 →
    // 48, the word 16516 → 24000. What stands: the standing family (47:
    // `SITE[·].reg`, `aggression[1]`, `form`, the army `pool:n` and
    // `group:n.held` with no `GROUPDATA` printed) and `1/46`'s
    // `dest_angle` on 16714 (ours 1073741824, theirs −1334181888), which
    // the draw stream never feels.
    pin_eq!(w.firsts.len(), 48, "keys that part in the window");
}

/// The closing state's window: run969b's closing block alone, the long's
/// last frame 24000 writing it (item 3360).
pub(crate) const WIDENING_INCA_CLOSING: (i64, i64) = (23999, 24002);

/// **The lobby's closing state** (item 3360, DECISIONS 55): run969b — the
/// draw stream's own capture, `cover=0` — prints the closing block whole.
/// The game does not end itself; the long's `!quit` at 24000 closes it.
#[test]
fn run969b_inca_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(INCA.long.0), dump(INCA.start), trace(INCA.long.1))
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
        "Inca closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 177, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    // Item 3360, the first walk (the word at 24000): 177 units compared on
    // block 24001, none apart, no building field-row; one closing field,
    // city `0/2000`'s `peasant_dist`, ours 2 against theirs 1 — the human's
    // city, which the draw stream never feels.
}
