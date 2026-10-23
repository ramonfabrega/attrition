//! **The British at a held Enlightenment age, East Indies** (DECISIONS 90,
//! 91, item 3512): the coverage pair's lobby (run652's: East Indies,
//! `MAP_STYLE 18`, human Nubians (4), Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`) with the nation varied to the **British (11)**
//! and the age held at Enlightenment: `STARTING_TECHNOLOGY 4`,
//! `ENDING_TECHNOLOGY 4`. Captures: run1156 (a `DUMP_ALL` start) and
//! run1157 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §306.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BRITISH_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesBritishEnlightenment",
    map_style: 18,
    ai_tribe: 11,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1156-eastindies-british-enlightenment-start.txt",
    long: (
        "gamelog-run1157-eastindies-british-enlightenment-trace.txt",
        "rontrace-run1157.log",
    ),
    // The game ended itself on 5522 (closing block 5523).
    length: 5522,
    // Item 3512, first try: frame 4587 by the count and the sequence, ours
    // 26 draws against 25, index 22 — ours `Unit::close+0xcb6`, theirs
    // `Farms::inc_time+0x1ae` (`docs/AI.md` section 306). **Item 3542:
    // whole at 5522**, the game's last frame, with two pieces' launch rows
    // (`sim::launch`'s `BAYS`, off run1185's packet): the Sloop's piece 277,
    // whose round at `0/13` left the hull and flew 8 frames against 9, and
    // the Highlanders' piece 64 — 4774 between them, its state parted on
    // 4735, where `1/91`'s round struck `0/10` a frame late in ours.
    count: 5522,
    sequence: 5522,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_ENLIGHTENMENT_PAIR_WORD: i64 =
    if BRITISH_ENLIGHTENMENT.count < BRITISH_ENLIGHTENMENT.sequence {
        BRITISH_ENLIGHTENMENT.count
    } else {
        BRITISH_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — `ENDING_TECHNOLOGY` beside `STARTING_TECHNOLOGY` — and the long
/// is scoreable.
#[test]
fn british_enlightenment_pair_captures_say_their_lobby() {
    let p = &BRITISH_ENLIGHTENMENT;
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
        "the British pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn british_enlightenment_pair_is_no_other_lobby() {
    let p = &BRITISH_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// **The pair's closing state**: run1157's closing dump, block 5523,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3542).
#[test]
fn run1157_british_enlightenment_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &BRITISH_ENLIGHTENMENT;
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
        "British pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5523, "the closing block");
    // Item 3542, the pair whole: 176 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 176, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1157b: the word 4587's window, blocks 4581..4837 at the long's
/// detail — six blocks before the word's block 4588 and 250 after (item
/// 3512).
pub(crate) const RUN1157B: &str =
    "gamelog-run1157b-eastindies-british-enlightenment-window-4581-4837.txt";

/// The window: block 4581 through 4837; the word 4587's own block is 4588.
pub(crate) const WIDENING_BRITISH_ENLIGHTENMENT_FRAME_4587: (i64, i64) = (4581, 4837);

/// **The word 4587's window**: run1157b walked from run1156's start with
/// the recorder on (item 3512).
pub(crate) fn british_enlightenment_frame_4587_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_ENLIGHTENMENT.start],
        true,
        BRITISH_ENLIGHTENMENT.long,
        "run1157b",
        &[(RUN1157B, WIDENING_BRITISH_ENLIGHTENMENT_FRAME_4587.0)],
        WIDENING_BRITISH_ENLIGHTENMENT_FRAME_4587,
        1,
        &[4588],
        true,
    )
}

#[test]
fn british_enlightenment_pair_run1157b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_enlightenment_frame_4587_window() else {
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
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3512, the first walk: 2773 keys. Block 4581 carries 96
    // standing rows, all of the standing families — `form`, `SITE[i].reg`,
    // `aggression`, who=1's groups' `held` and pools — and no other field.
    // **The state parts on the word's frame**: on block 4588 the human's
    // Citizen `0/13` (TypeIndex 50, land), fleeing, is dead in ours
    // (`damage` 72, `hold_frames` 30) and alive in theirs at 63, and city
    // `0/2000`'s `raid_stamp` reads ours 4587 against theirs 4560; the
    // human's `active`, `control`, `num_units[0]` and `peasants` read ours
    // 11, 11, 10, 10 against 12, 12, 11, 11. The blow is the Sloop `1/61`'s
    // (TypeIndex 327, sea) round of 4580, `total_time` 8 in ours, which
    // lands on 4587 where theirs' kills on 4588 (`docs/AI.md` section 306).
    // **Item 3542: 2773 → 693 → 108** with the Sloop's launch rows (the
    // round leaves (3315, 6861) on run1185's packet, 9 frames, and block
    // 4588 agrees: `0/13` alive at 63, `raid_stamp` 4560, the leader's
    // counts 12, 12, 11, 11 on both sides), then the Highlanders' (block
    // 4735's `0/10` struck on 4734 on both, `hits_left` 44). What stands
    // is the standing families alone: block 4581's 96 rows, the new
    // units' `form` and groups 76's and 79's `held`/pool, and one
    // `SITE[8].reg` on 4776.
    pin_eq!(w.firsts.len(), 108, "run1157b baseline");
}

/// run1185b: the pair's last blocks, 5266..5523 at the window's detail —
/// the word 5522 is the game's last frame and 5523 its closing block (item
/// 3542).
pub(crate) const RUN1185B: &str =
    "gamelog-run1185b-eastindies-british-enlightenment-window-5266-5523.txt";

/// The window: block 5266 through the closing block 5523.
pub(crate) const WIDENING_BRITISH_ENLIGHTENMENT_CLOSING: (i64, i64) = (5266, 5523);

/// **The pair's last blocks**: run1185b walked from run1156's start with
/// the recorder on. The word is the game's last frame (item 3542).
pub(crate) fn british_enlightenment_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_ENLIGHTENMENT.start],
        true,
        BRITISH_ENLIGHTENMENT.long,
        "run1185b",
        &[(RUN1185B, WIDENING_BRITISH_ENLIGHTENMENT_CLOSING.0)],
        WIDENING_BRITISH_ENLIGHTENMENT_CLOSING,
        1,
        &[5523],
        true,
    )
}

#[test]
fn british_enlightenment_pair_run1185b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_enlightenment_closing_window() else {
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
    // Item 3542, the first walk: **110** keys. Block 5266 carries the
    // standing families alone (`SITE[i].reg`, `aggression[1]`, `form`,
    // who=1's groups' `held`/pool), and later blocks the new units' `form`
    // and group 76's `held`/pool. **One row is not a standing family**: on
    // block 5276 city `0/2000`'s `build:damage` reads ours 604 and 6/16
    // against theirs 610 and 0 (parked 3542a); the closing block agrees.
    pin_eq!(w.firsts.len(), 110, "run1185b baseline");
}
