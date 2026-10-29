//! **The Romans at a held Modern age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, 102, item 3771): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **Romans (6)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(18, 6, 4)`, is new (`tools/refill.py 18 6 4` reads free): the Romans
//! hold East Indies at 2 (`roman_word`) and 5 (`roman_medieval_pair`),
//! Great Lakes at 5 (`roman_gunpowder_pair`) and Great Sahara at 5
//! (`roman_classical_pair`); the battery's one Roman game is the
//! held-out `GreatLakesRomanTier2`, `(14, 6, 2)`.
//! Captures: run1443 (a `DUMP_ALL` start) and run1444 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §377.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const ROMAN_MODERN: Pair = Pair {
    name: "EastIndiesRomanModern",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1443-eastindies-roman-modern-start.txt",
    long: (
        "gamelog-run1444-eastindies-roman-modern-trace.txt",
        "rontrace-run1444.log",
    ),
    // The game ended itself on 4130 (closing block 4131).
    length: 4130,
    // Item 3751, first try: whole — count 4130, sequence 4130 of 4130; no
    // draw parts to the game's last frame (`docs/AI.md` §377).
    count: 4130,
    sequence: 4130,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const ROMAN_MODERN_PAIR_WORD: i64 = if ROMAN_MODERN.count < ROMAN_MODERN.sequence {
    ROMAN_MODERN.count
} else {
    ROMAN_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn roman_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &ROMAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "ROMAN_MODERN {}: count {} sequence {} of {}; {}",
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
fn roman_modern_pair_captures_say_their_lobby() {
    let p = &ROMAN_MODERN;
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
        "the Roman Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3771; the battery's identity
/// test refuses it), once, and none of the battery's — its one Roman game
/// is the held-out `GreatLakesRomanTier2`, `(14, 6, 2)` — and no other
/// pinned Roman lobby's: the AI word's `(18, 6, 2)` and the Medieval
/// pair's `(18, 6, 5)`, East Indies at other difficulties, the Gunpowder
/// pair's `(14, 6, 5)`, Great Lakes, and the Classical pair's
/// `(7, 6, 5)`, Great Sahara (item 3771).
#[test]
fn roman_modern_pair_is_no_other_lobby() {
    let p = &ROMAN_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let roman: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        roman,
        [("GreatLakesRomanTier2", (14, 6, 2))],
        "the battery's Roman games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Roman lobby on this map at this difficulty"
        );
    }
    let w = &super::roman_word::ROMAN;
    let c = &super::roman_classical_pair::ROMAN_CLASSICAL;
    let m = &super::roman_medieval_pair::ROMAN_MEDIEVAL;
    let g = &super::roman_gunpowder_pair::ROMAN_GUNPOWDER;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Romans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride** (item 3771): the pair is whole, so the window ends
/// where the stride does — run1444's stride blocks 101 through 5201.
pub(crate) const WIDENING_ROMAN_MODERN_STRIDE: (i64, i64) = (101, 5201);

pub(crate) fn roman_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MODERN.start],
        true,
        ROMAN_MODERN.long,
        "run1444 stride",
        &[(ROMAN_MODERN.long.0, WIDENING_ROMAN_MODERN_STRIDE.0)],
        WIDENING_ROMAN_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn roman_modern_pair_the_long_s_stride_is_widened() {
    let _pins = Pins::hold();
    let Some(w) = roman_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 52, "every stride block in the window");
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
    // Item 3751: 207 keys over 41 blocks — the standing families (`form`
    // on 69 units, `build:extra` on 64 buildings, which the stride does not
    // print, who=1's groups' `held` and pools) and, from 3901, the
    // Submarine `1/139` (type 338, group 71) on its `ATTACKTOORDER`: the
    // order's point (45288, 37992) against theirs' (45000, 37608), its
    // formation offset (744, 360) against (456, 744), and the path with it.
    // No draw parts to the game's end (parked 3751a).
    pin_eq!(w.firsts.len(), 207, "the stride's baseline");
}

/// **The pair's closing state** (item 3771): run1444's closing block 5235,
/// the whole map's units, walked from run1443's start.
#[test]
fn roman_modern_pair_run1444_s_closing_state() {
    let _pins = Pins::hold();
    let p = &ROMAN_MODERN;
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
        "Roman Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5235, "the closing block");
    // Item 3751, whole on its first try: 180 units compared, counts
    // [1, 0, 0, 0, 0, 0, 0] — the Submarine `1/139` off by (70, -591), its
    // re-chosen formation offset (parked 3751a); nothing torn, unlinked
    // or extra. A floor, not a score.
    pin_eq!(e.compared, 180, "units compared");
    pin_eq!(e.counts(), [1, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
