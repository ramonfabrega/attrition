//! **The Germans at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3719): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Germans (12)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 12, 4)`, is new: the Germans hold this map at 5
//! (`german_industrial_pair`), East Indies at 3 (`german_word`) and 5
//! (`german_alltech_pair`), and Great Sahara at 3 (`german_sahara_word`)
//! and 5 (`german_ancient_pair`); the battery fields no Germans. Captures:
//! run1396 (a `DUMP_ALL` start) and run1397 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §366.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GERMAN_MODERN: Pair = Pair {
    name: "GreatLakesGermanModern",
    map_style: 14,
    ai_tribe: 12,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1396-greatlakes-german-modern-start.txt",
    long: (
        "gamelog-run1397-greatlakes-german-modern-trace.txt",
        "rontrace-run1397.log",
    ),
    // The game ended itself on 5304 (closing block 5305).
    length: 5304,
    // Item 3719, first try: count 4079, sequence 4077 of 5304 — on 4077
    // ours 370 draws against theirs' 370, index 0 ours' `Guy::set_anim+0x97a
    // < Unit::do_guard+0x7f4` against theirs' `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d` (`docs/AI.md` §366).
    count: 4079,
    sequence: 4077,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_MODERN_PAIR_WORD: i64 = if GERMAN_MODERN.count < GERMAN_MODERN.sequence {
    GERMAN_MODERN.count
} else {
    GERMAN_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn german_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn german_modern_pair_captures_say_their_lobby() {
    let p = &GERMAN_MODERN;
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
        "the German Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3719; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Germans — and no other German lobby's: `german_industrial_pair`'s
/// `(14, 12, 5)`, this map at another difficulty, `german_word`'s
/// `(18, 12, 3)`, `german_alltech_pair`'s `(18, 12, 5)`,
/// `german_sahara_word`'s `(7, 12, 3)` and `german_ancient_pair`'s
/// `(7, 12, 5)`, other maps (item 3719).
#[test]
fn german_modern_pair_is_no_other_lobby() {
    let p = &GERMAN_MODERN;
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
        "the battery fields the Germans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another German lobby on this map at this difficulty"
        );
    }
    let w = &super::german_word::GERMAN;
    let s = &super::german_sahara_word::GERMAN_SAHARA;
    let a = &super::german_alltech_pair::GERMAN_ALLTECH;
    let n = &super::german_ancient_pair::GERMAN_ANCIENT;
    let i = &super::german_industrial_pair::GERMAN_INDUSTRIAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
        (n.name, (n.map_style, n.ai_tribe, n.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Germans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3719, from
/// 4077): only run1397's stride blocks are on disk before the word's
/// window, 101 through 4001.
pub(crate) const WIDENING_GERMAN_MODERN_STRIDE: (i64, i64) = (101, 4001);

pub(crate) fn german_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1397 stride",
        &[(GERMAN_MODERN.long.0, WIDENING_GERMAN_MODERN_STRIDE.0)],
        WIDENING_GERMAN_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn german_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 40, "every stride block in the window");
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
    // Item 3719: 114 keys over 40 blocks. The standing families — `form`
    // on every unit, the buildings' `build:extra` (which the stride does
    // not print) and who=1's groups with their pools — and, from block
    // 3101, `myspeed` on who=1's units, theirs a quarter above ours (75
    // against 93, 60 against 75, 115 against 143) with no `pos` parting:
    // parked 3677a's and 3705b's family.
    pin_eq!(w.firsts.len(), 114, "the stride's baseline");
}

/// run1397b: the word 4077's window, blocks 3995..4328 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4001 and 250 past the word's own block 4078 (item 3719).
/// run1397 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1397B: &str = "gamelog-run1397b-greatlakes-german-modern-window-3995-4328.txt";

/// The window: block 3995 through 4328.
pub(crate) const WIDENING_GERMAN_MODERN_FRAME_4077: (i64, i64) = (3995, 4328);

/// **The word 4077's window**: run1397b walked from run1396's start with
/// the recorder on and every round compared (item 3719).
pub(crate) fn german_modern_frame_4077_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_MODERN.start],
        true,
        GERMAN_MODERN.long,
        "run1397b",
        &[(RUN1397B, WIDENING_GERMAN_MODERN_FRAME_4077.0)],
        WIDENING_GERMAN_MODERN_FRAME_4077,
        1,
        &[4078],
        true,
    )
}

#[test]
fn german_modern_pair_run1397b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_modern_frame_4077_window() else {
        return;
    };
    pin_eq!(w.blocks, 333, "every captured block");
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
    // Item 3719, the first walk: **1946** keys. Block 3995 carries the
    // standing families (both leaders' `SITE[i].reg`, `aggression[1]`,
    // `form`, groups and pools, the airports' `myspeed`). **The word**: on
    // 4017 theirs' Tiger Tank `1/38` (TI 258, piece 208, which neither
    // `launch` nor `pivot` holds) looses a round that is made and explodes
    // inside the frame and wounds the Citizen `0/12` (TI 50, hits 85 → 49);
    // ours' leaves from the guy's own square, (4392, 31416), and is still
    // in flight on 4018 (`t1/2`). Theirs' citizen flees on a `FLEETOORDER`
    // to (2136, 32616), arrives on 4077, drops the order and goes idle with
    // one `Guy::set_anim+0x97a < Unit::do_idle+0x7d` draw ours never
    // spends; the same frame theirs' Tiger `1/62` drops its attack on the
    // Citizen `0/13`, fled since its own round of 4038, and spends no
    // `Unit::fight+0x824` draw where ours does.
    pin_eq!(w.firsts.len(), 1946, "run1397b baseline");
    // Every round, both directions: 137 in ours against 176, the first
    // parting the Tiger `1/38`'s on 4018, which ours holds alone.
    pin_eq!(w.rounds_read, (137, 176), "the rounds read");
    pin_eq!(w.rounds.len(), 47, "the rounds that part");
}

/// **The pair's closing state** (item 3719): run1397's closing block 5305,
/// the whole map's units, walked from run1396's start.
#[test]
fn german_modern_pair_run1397_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GERMAN_MODERN;
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
        "German Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5305, "the closing block");
    // Item 3719, first try: 1227 frames past the word 4077 the game has
    // parted widely — 146 units compared, counts [127, 2, 3, 1, 51, 0, 55],
    // `0/13`, `1/36` and `1/111` extra in ours, `1/139` and `1/141`
    // unlinked. A floor, not a score.
    pin_eq!(e.compared, 146, "units compared");
    pin_eq!(e.counts(), [127, 2, 3, 1, 51, 0, 55], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
