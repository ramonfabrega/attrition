//! **The Germans at All Technologies, East Indies** (DECISIONS 90, item
//! 3506): the first of the late-age pairs. The coverage pair's lobby
//! (run652's: East Indies, `MAP_STYLE 18`, human Nubians (4), Difficulty 5,
//! seed 12345, `STARTING_TECHNOLOGY 8`, `ENDING_TECHNOLOGY 7`,
//! `STARTING_RESOURCES 7`) with the nation varied to the **Germans (12)**,
//! for the age-7 line on an island map. Captures: run1144 (a `DUMP_ALL`
//! start), run1145b (the draw stream to the game's end, `cover=0`, run831's
//! detail lines) and run1145 (the same game, `end:UNITS=3,GUYS=2` on a
//! stride of 100 for the rubric; a strided long carries no initial state,
//! so the walk reads run1145b). **Whole on its first try**: the draw
//! stream agrees to 4175, the game's last frame, and the closing block
//! compares 298 units whole (`docs/AI.md` §300).

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GERMAN_ALLTECH: Pair = Pair {
    name: "EastIndiesGermanAllTech",
    map_style: 18,
    ai_tribe: 12,
    difficulty: 5,
    starting_technology: 8,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1144-eastindies-german-alltech-start.txt",
    long: (
        "gamelog-run1145b-eastindies-german-alltech-trace.txt",
        "rontrace-run1145b.log",
    ),
    length: 4175,
    // Item 3506, first try: whole. The draw count and the draw sequence
    // agree on every frame to 4175, the game's last (the idle human is
    // defeated there), and the closing block 4176 compares whole.
    count: 4175,
    sequence: 4175,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_ALLTECH_PAIR_WORD: i64 = if GERMAN_ALLTECH.count < GERMAN_ALLTECH.sequence {
    GERMAN_ALLTECH.count
} else {
    GERMAN_ALLTECH.sequence
};

/// **The pair's first parting**, walked from its own start, first try.
#[test]
fn german_alltech_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_ALLTECH;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_ALLTECH {}: count {} sequence {} of {}; {}",
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
fn german_alltech_pair_captures_say_their_lobby() {
    let p = &GERMAN_ALLTECH;
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
        "the German pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn german_alltech_pair_is_no_other_lobby() {
    let p = &GERMAN_ALLTECH;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// **The pair's closing state**: run1145b's closing dump, block 4176,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099).
#[test]
fn run1145b_german_alltech_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &GERMAN_ALLTECH;
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
        "German pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4176, "the closing block");
    // Item 3506, first try: 298 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 298, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1145c: the pair's last blocks, 4169..4176 at the long's detail — six
/// blocks before the word 4175's block 4176, which is the game's closing
/// block (item 3506).
pub(crate) const RUN1145C: &str = "gamelog-run1145c-eastindies-german-alltech-window-4169-4176.txt";

/// The window: block 4169 through the closing block 4176, the word 4175's
/// own block.
pub(crate) const WIDENING_GERMAN_ALLTECH_CLOSING: (i64, i64) = (4169, 4176);

/// **The pair's last blocks**: run1145c walked from run1144's start with
/// the recorder on. The word is the game's last frame (item 3506).
pub(crate) fn german_alltech_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_ALLTECH.start],
        true,
        GERMAN_ALLTECH.long,
        "run1145c",
        &[(RUN1145C, WIDENING_GERMAN_ALLTECH_CLOSING.0)],
        WIDENING_GERMAN_ALLTECH_CLOSING,
        1,
        &[4176],
        true,
    )
}

#[test]
fn run1145c_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_alltech_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 7, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // Item 3506, the first walk: 185 keys, 183 standing on 4169 — who=1's
    // `form` (ours -1 against theirs 9 or 0) and `mirror` (ours 0 against
    // 1) on most of its units, group and pool `held` lists theirs does not
    // print at this detail, the leaders' `SITE[i].reg` (ours 65 against
    // 0), who=0's `aggression[1]` (ours 0 against 1), and six who=1 units'
    // `attrition` (ours 6 against 24) with their wounds (`1/20`, `1/21`,
    // `1/22`, `1/49`, `1/91`, `1/92`); group 93's `held` and pool on 4170.
    // The word 4175's block 4176, the game's closing block, parts no key
    // of its own; the draw stream agrees to 4175 (`docs/AI.md` §300).
    pin_eq!(w.firsts.len(), 183, "run1145c baseline");
}
