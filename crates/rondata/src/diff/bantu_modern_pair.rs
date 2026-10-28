//! **The Bantu at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3723): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Bantu (3)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 3, 5)`, is new: the Bantu hold East Indies at 5
//! (`bantu_ancient_pair`) and Great Sahara at 4 (`bantu_word`) and 5
//! (`bantu_medieval_pair`); the battery fields no Bantu.
//! Captures: run1404 (a `DUMP_ALL` start) and run1405 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §370.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BANTU_MODERN: Pair = Pair {
    name: "GreatLakesBantuModern",
    map_style: 14,
    ai_tribe: 3,
    difficulty: 5,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1404-greatlakes-bantu-modern-start.txt",
    long: (
        "gamelog-run1405-greatlakes-bantu-modern-trace.txt",
        "rontrace-run1405.log",
    ),
    // The game ended itself on 4395 (closing block 4396).
    length: 4395,
    // Item 3723, first try: count 3410, sequence 3410 of 4395 — on 3410
    // ours 12 draws against theirs' 11, index 0 ours'
    // `Guy::set_anim+0x97a` under `Unit::move_step+0x823` where theirs'
    // is under `Guy::inc_time+0x271` (`docs/AI.md` §370).
    count: 3410,
    sequence: 3410,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BANTU_MODERN_PAIR_WORD: i64 = if BANTU_MODERN.count < BANTU_MODERN.sequence {
    BANTU_MODERN.count
} else {
    BANTU_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn bantu_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BANTU_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BANTU_MODERN {}: count {} sequence {} of {}; {}",
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
fn bantu_modern_pair_captures_say_their_lobby() {
    let p = &BANTU_MODERN;
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
        "the Bantu Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3723; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Bantu — and no other Bantu lobby's: `bantu_word`'s `(7, 3, 4)` and
/// `bantu_medieval_pair`'s `(7, 3, 5)`, Great Sahara, and
/// `bantu_ancient_pair`'s `(18, 3, 5)`, East Indies (item 3723).
#[test]
fn bantu_modern_pair_is_no_other_lobby() {
    let p = &BANTU_MODERN;
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
        "the battery fields the Bantu"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Bantu lobby on this map at this difficulty"
        );
    }
    let w = &super::bantu_word::BANTU;
    let m = &super::bantu_medieval_pair::BANTU_MEDIEVAL;
    let a = &super::bantu_ancient_pair::BANTU_ANCIENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Bantu");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3723, from
/// 3410): only run1405's stride blocks are on disk before the word's
/// window, 101 through 3301.
pub(crate) const WIDENING_BANTU_MODERN_STRIDE: (i64, i64) = (101, 3301);

pub(crate) fn bantu_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_MODERN.start],
        true,
        BANTU_MODERN.long,
        "run1405 stride",
        &[(BANTU_MODERN.long.0, WIDENING_BANTU_MODERN_STRIDE.0)],
        WIDENING_BANTU_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn bantu_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = bantu_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every stride block in the window");
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
    // Item 3723: 94 keys over 33 blocks, the standing families alone —
    // `form` on 27 units, the buildings' `build:extra` (which the stride
    // does not print), and who=1's groups 64..69 with their pools.
    pin_eq!(w.firsts.len(), 94, "the stride's baseline");
}

/// run1405b: the word 3410's window, blocks 3395..3661 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3401 and 250 past the word's own block 3411 (item 3723).
/// run1405 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1405B: &str = "gamelog-run1405b-greatlakes-bantu-modern-window-3395-3661.txt";

/// The window: block 3395 through 3661.
pub(crate) const WIDENING_BANTU_MODERN_FRAME_3410: (i64, i64) = (3395, 3661);

/// **The word 3410's window**: run1405b walked from run1404's start with
/// the recorder on and every round compared (item 3723).
pub(crate) fn bantu_modern_frame_3410_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BANTU_MODERN.start],
        true,
        BANTU_MODERN.long,
        "run1405b",
        &[(RUN1405B, WIDENING_BANTU_MODERN_FRAME_3410.0)],
        WIDENING_BANTU_MODERN_FRAME_3410,
        1,
        &[3411],
        true,
    )
}

#[test]
fn bantu_modern_pair_run1405b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_modern_frame_3410_window() else {
        return;
    };
    pin_eq!(w.blocks, 266, "every captured block");
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
    // Item 3723, the first walk: PENDING.
    pin_eq!(w.firsts.len(), 1418, "run1405b baseline");
    // Every round, both directions: 195 against 195, none parting.
    pin_eq!(w.rounds_read, (195, 195), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3723): run1405's closing block 4396,
/// the whole map's units, walked from run1404's start.
#[test]
fn bantu_modern_pair_run1405_s_closing_state() {
    let _pins = Pins::hold();
    let p = &BANTU_MODERN;
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
        "Bantu Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4396, "the closing block");
    // Item 3723, first try: 986 frames past the word 3410 the game has
    // parted widely — 184 units compared, counts [140, 0, 1, 2, 93, 1, 35],
    // nothing torn or unlinked, who=1's `1/151` extra in ours. A floor,
    // not a score.
    pin_eq!(e.compared, 184, "units compared");
    pin_eq!(e.counts(), [140, 0, 1, 2, 93, 1, 35], "the closing counts");
    pin_eq!(e.extra.clone(), vec![(1, 151)], "the closing extras");
}
