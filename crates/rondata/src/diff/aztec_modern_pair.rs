//! **The Aztecs at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, item 3750): Great Lakes, `MAP_STYLE 14`,
//! human Nubians (4) against the **Aztecs (0)** at **Toughest (5)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 0, 5)`, is new: the Aztecs hold Great Sahara at 2 (`aztec_word`),
//! 3 (`aztec_alltech_pair`) and 5 (`aztec_medieval_pair`) and East Indies
//! at 5 (`aztec_ancient_pair`); the battery's one Aztec game is
//! `EastIndiesAztecTier1`, `(18, 0, 1)`.
//! Captures: run1428 (a `DUMP_ALL` start) and run1429 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §374.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AZTEC_MODERN: Pair = Pair {
    name: "GreatLakesAztecModern",
    map_style: 14,
    ai_tribe: 0,
    difficulty: 5,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1428-greatlakes-aztec-modern-start.txt",
    long: (
        "gamelog-run1429-greatlakes-aztec-modern-trace.txt",
        "rontrace-run1429.log",
    ),
    // The game ended itself on 3389 (closing block 3390).
    length: 3389,
    // Item 3750, first try: count 3313, sequence 3313 of 3389 — on 3313
    // ours 2 draws against theirs' 4, index 2 theirs'
    // `Ammo::do_damage+0xc59` (`docs/AI.md` §374). Item 3766: **whole,
    // 3389 of 3389** — piece 60's five bays (the Jaguar Infantry, run1441's
    // packet) and piece 196's pivot rows (the Sherman, run1164's packet);
    // `1/24`'s round of 3309 leaves (5625, 30777, 592) on both sides and
    // explodes on 3313 on both (`docs/AI.md` §374.1).
    count: 3389,
    sequence: 3389,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AZTEC_MODERN_PAIR_WORD: i64 = if AZTEC_MODERN.count < AZTEC_MODERN.sequence {
    AZTEC_MODERN.count
} else {
    AZTEC_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn aztec_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AZTEC_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AZTEC_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn aztec_modern_pair_captures_say_their_lobby() {
    let p = &AZTEC_MODERN;
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
        "the Aztec Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3750; the battery's identity
/// test refuses it), once, and none of the battery's — its one Aztec game
/// is the held-out `EastIndiesAztecTier1`, `(18, 0, 1)` — and no other
/// pinned Aztec lobby's: `aztec_word`'s `(7, 0, 2)`, the AllTech pair's
/// `(7, 0, 3)` and the Medieval pair's `(7, 0, 5)`, Great Sahara, and the
/// Ancient-start pair's `(18, 0, 5)`, East Indies (item 3750).
#[test]
fn aztec_modern_pair_is_no_other_lobby() {
    let p = &AZTEC_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let aztecs: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        aztecs,
        [("EastIndiesAztecTier1", (18, 0, 1))],
        "the battery's Aztec games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Aztec lobby on this map at this difficulty"
        );
    }
    let w = &super::aztec_word::AZTEC;
    let t = &super::aztec_alltech_pair::AZTEC_ALLTECH;
    let m = &super::aztec_medieval_pair::AZTEC_MEDIEVAL;
    let a = &super::aztec_ancient_pair::AZTEC_ANCIENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (t.name, (t.map_style, t.ai_tribe, t.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Aztecs");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3750, from
/// 3313): run1429's stride blocks before the word's window, 101 through
/// 3201.
pub(crate) const WIDENING_AZTEC_MODERN_STRIDE: (i64, i64) = (101, 3201);

pub(crate) fn aztec_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_MODERN.start],
        true,
        AZTEC_MODERN.long,
        "run1429 stride",
        &[(AZTEC_MODERN.long.0, WIDENING_AZTEC_MODERN_STRIDE.0)],
        WIDENING_AZTEC_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn aztec_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = aztec_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 32, "every stride block in the window");
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
    // Item 3750: 93 keys over 32 blocks, the standing families alone —
    // `form`, the buildings' `build:extra` (which the stride does not
    // print), both leaders' `SITE[i].reg`, and who=1's groups and pools.
    pin_eq!(w.firsts.len(), 93, "the stride's baseline");
}

/// run1429b: the word 3313's window, blocks 3295..3389 at the long's
/// detail with `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5`, to the
/// game's own end — six blocks before the stride's block 3301 (item
/// 3750). run1429 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1429B: &str = "gamelog-run1429b-greatlakes-aztec-modern-window-3295-3389.txt";

/// The window: block 3295 through the closing block 3390.
pub(crate) const WIDENING_AZTEC_MODERN_FRAME_3313: (i64, i64) = (3295, 3390);

/// **The word 3313's window**: run1429b walked from run1428's start with
/// the recorder on and every round compared (item 3750).
pub(crate) fn aztec_modern_frame_3313_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AZTEC_MODERN.start],
        true,
        AZTEC_MODERN.long,
        "run1429b",
        &[(RUN1429B, WIDENING_AZTEC_MODERN_FRAME_3313.0)],
        WIDENING_AZTEC_MODERN_FRAME_3313,
        1,
        &[3314],
        true,
    )
}

#[test]
fn aztec_modern_pair_run1429b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_modern_frame_3313_window() else {
        return;
    };
    pin_eq!(w.blocks, 95, "every captured block");
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
    // Item 3750, the first walk: **895** keys. Block 3295 carries the
    // standing families (`form`, both leaders' `SITE[i].reg`, who=0's
    // `aggression[1]`, who=1's groups 64..69 and their pools, `1/98` and
    // `1/99` `mirror`); the first unit row is 3302's `0/7` (`hits:damage`
    // ours 77 theirs 82, `hits_left` 8 against 3), the round `1/22` loosed
    // on 3296 landing a frame apart. **The rounds part from the window's
    // first block**: who=1's Jaguar Infantry (TI 110, piece 60,
    // `JAGINFANTRY-DEFAULT-AGE0`) and Shermans (TI 246, piece 196,
    // `SHERMAN-DEFAULT-AGE0`) loose every round from their own square in
    // ours — `1/22`'s of 3296 from (6024, 30792, 426) against theirs'
    // (5891, 30618, 607), `total_time` 6 against 5. The word 3313 is
    // `1/24`'s round of 3309, from (5640, 30840, 474) against theirs'
    // (5625, 30777, 592), 6 frames against 5: theirs sets `flags&8` and
    // explodes it on 3313, `Ammo::do_damage+0xc59`/`+0xc7e`, and ours
    // holds it a frame on (`docs/AI.md` §374).
    // Item 3766, with pieces 60's and 196's rows: **53** keys, the
    // standing families alone (`form`, both leaders' `SITE[i].reg`, who=0's
    // `aggression[1]`, who=1's groups and pools, five `mirror`s); `0/7`'s
    // 3302 damage and the city's and building's rows agree.
    pin_eq!(w.firsts.len(), 53, "run1429b baseline");
    // Every round, both directions: 86 against 86 (item 3750: 86 against
    // 104). The six rows left are `1/34`'s and `1/35`'s three rounds on
    // 3313 whose target, the Citizen `0/7` dead that frame, ours drops
    // (`ox`, `whom` −1 against 7, 0; 3766a).
    pin_eq!(w.rounds_read, (86, 86), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// **The pair's closing state** (item 3750): run1429's closing block 3390,
/// the whole map's units, walked from run1428's start.
#[test]
fn aztec_modern_pair_run1429_s_closing_state() {
    let _pins = Pins::hold();
    let p = &AZTEC_MODERN;
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
        "Aztec Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3390, "the closing block");
    // Item 3750, first try: 76 frames past the word 3313 — 117 units
    // compared, counts [33, 0, 0, 0, 6, 0, 5], nothing torn, unlinked or
    // extra; who=1's Jaguar Infantry and Shermans still loosing from their
    // own squares. A floor, not a score.
    pin_eq!(e.compared, 117, "units compared");
    // Item 3766, the pair whole: counts all zero, no closing field.
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
