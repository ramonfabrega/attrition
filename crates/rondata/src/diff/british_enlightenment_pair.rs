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
use crate::testenv::dump;

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
    // `Farms::inc_time+0x1ae` (`docs/AI.md` section 306).
    count: 4587,
    sequence: 4587,
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
    pin_eq!(w.firsts.len(), 2773, "run1157b baseline");
}
