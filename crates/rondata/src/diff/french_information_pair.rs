//! **The French at a held Information age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, 97, 98, 99, 101, 102, item 3773): Great Sahara,
//! `MAP_STYLE 7`, human Nubians (4) against the **French (10)** at
//! **Moderate (3)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Information: `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple,
//! `(7, 10, 3)`, is new (`tools/refill.py 7 10 3` reads free): the French
//! hold this map at 4 (`french_word`) and 5 (`french_classical_pair`),
//! Great Lakes at 4 (`french_medieval_pair`) and 5 (the third pair), and
//! East Indies at 4 (`french_enlightenment_pair`) and 5 (the third pair);
//! the battery fields no French. Captures: run1447 (a `DUMP_ALL` start)
//! and run1448 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §379.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const FRENCH_INFORMATION: Pair = Pair {
    name: "GreatSaharaFrenchInformation",
    map_style: 7,
    ai_tribe: 10,
    difficulty: 3,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1447-greatsahara-french-information-start.txt",
    long: (
        "gamelog-run1448-greatsahara-french-information-trace.txt",
        "rontrace-run1448.log",
    ),
    // The game ended itself on 3191 (closing block 3192).
    length: 3191,
    // Item 3773, first try: whole — count 3191, sequence 3191 of 3191 (the
    // game's last frame), walked from run1447's start (`docs/AI.md` §379).
    count: 3191,
    sequence: 3191,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const FRENCH_INFORMATION_PAIR_WORD: i64 =
    if FRENCH_INFORMATION.count < FRENCH_INFORMATION.sequence {
        FRENCH_INFORMATION.count
    } else {
        FRENCH_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn french_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &FRENCH_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "FRENCH_INFORMATION {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 7 and the difficulty at 3, and the long is
/// scoreable.
#[test]
fn french_information_pair_captures_say_their_lobby() {
    let p = &FRENCH_INFORMATION;
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
        "the French Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3773; the battery's identity
/// test refuses it), once; the battery fields no French — and no other
/// French lobby's: the AI word's `(7, 10, 4)` and the Classical pair's
/// `(7, 10, 5)`, this map at other difficulties; the Medieval pair's
/// `(14, 10, 4)`, the Enlightenment pair's `(18, 10, 4)` and the third
/// pair's `(14, 10, 5)` and `(18, 10, 5)`, other maps (item 3773).
#[test]
fn french_information_pair_is_no_other_lobby() {
    let p = &FRENCH_INFORMATION;
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
        "the battery fields the French"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another French lobby on this map at this difficulty"
        );
    }
    let w = &super::french_word::FRENCH;
    let c = &super::french_classical_pair::FRENCH_CLASSICAL;
    let m = &super::french_medieval_pair::FRENCH_MEDIEVAL;
    let e = &super::french_enlightenment_pair::FRENCH_ENLIGHTENMENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        ("the third pair, East Indies", (18, 10, 5)),
        ("the third pair, Great Lakes", (14, 10, 5)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the French");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, the whole game** (item 3773): the pair is whole on
/// its first try, so the stride walks every block run1448 prints, 101
/// through 3101, the last before the closing block 3192.
pub(crate) const WIDENING_FRENCH_INFORMATION_STRIDE: (i64, i64) = (101, 3101);

pub(crate) fn french_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH_INFORMATION.start],
        true,
        FRENCH_INFORMATION.long,
        "run1448 stride",
        &[(
            FRENCH_INFORMATION.long.0,
            WIDENING_FRENCH_INFORMATION_STRIDE.0,
        )],
        WIDENING_FRENCH_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn french_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = french_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 31, "every stride block in the window");
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
    // Item 3773, whole on the first try: 87 keys over 31 blocks, the
    // standing families alone — the buildings' `build:extra` (47, which the
    // stride does not print), `form` on 27 units, who=1's groups 64..68'
    // `held` and pools (5 each), and `mirror` on `1/87` from 3001 and on
    // `1/106` and `1/123` from 3101 (ours 0, theirs 1). No draw parts.
    pin_eq!(w.firsts.len(), 87, "the stride's baseline");
}

/// run1448b: the pair's last blocks, 2935..3192 at the long's detail with
/// `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5`, through the closing block
/// (item 3773). The pair is whole on its first try, so its word is the
/// game's last frame; run1448 prints units only every hundred frames, with
/// no leader, building, city or round record past the start.
pub(crate) const RUN1448B: &str =
    "gamelog-run1448b-greatsahara-french-information-closing-window-2935-3192.txt";

/// The window: block 2935 through the closing block 3192.
pub(crate) const WIDENING_FRENCH_INFORMATION_CLOSING: (i64, i64) = (2935, 3192);

/// **The pair's last blocks**: run1448b walked from run1447's start with
/// the recorder on and every round compared (item 3773).
pub(crate) fn french_information_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[FRENCH_INFORMATION.start],
        true,
        FRENCH_INFORMATION.long,
        "run1448b",
        &[(RUN1448B, WIDENING_FRENCH_INFORMATION_CLOSING.0)],
        WIDENING_FRENCH_INFORMATION_CLOSING,
        1,
        &[3192],
        true,
    )
}

#[test]
fn french_information_pair_run1448b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_information_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
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
    // Item 3773, whole on the first try: **55** keys, the standing families
    // alone — `form` (27), who=1's groups 64..69' `held` and pools (6
    // each), who=0's `SITE[0..9].reg` (1 against 0) and `aggression[1]` (0
    // against 1), `mirror` on `1/87`, `1/106` and `1/123` (ours 0, theirs
    // 1), and two guards' `dest_angle` — the TOW `1/71` (type 141) and the
    // Abrams `1/83` (type 254), group 65's. No draw parts to 3191.
    pin_eq!(w.firsts.len(), 55, "run1448b baseline");
    // Every round, both directions: 56 against 56, none parting.
    pin_eq!(w.rounds_read, (56, 56), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3773): run1448's closing block,
/// the whole map's units, walked from run1447's start.
#[test]
fn french_information_pair_run1448_s_closing_state() {
    let _pins = Pins::hold();
    let p = &FRENCH_INFORMATION;
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
        "French Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3192, "the closing block");
    // Item 3773, whole on the first try: 127 units compared, every count
    // zero, nothing torn, unlinked or extra (`docs/AI.md` §379).
    pin_eq!(e.compared, 127, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
