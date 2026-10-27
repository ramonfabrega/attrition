//! **The Persians at a held Gunpowder age, East Indies, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97, 98, item 3698): East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Persians (23)** at
//! **Hard (4)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Gunpowder: `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`. Its triple,
//! `(18, 23, 4)`, is new: the Persian lobbies are the coverage pair's and
//! the Medieval pair's `(18, 23, 5)` — this map at another difficulty —
//! the Sahara coverage pair's `(7, 23, 5)`, the Classical pair's
//! `(14, 23, 5)` and `persian_word`'s `GreatLakesPersianTier2`,
//! `(14, 23, 2)`, which left the battery for the AI word; no battery game
//! fields the Persians. Captures: run1369 (a `DUMP_ALL` start) and run1370
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §358.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const PERSIAN_GUNPOWDER: Pair = Pair {
    name: "EastIndiesPersianGunpowder",
    map_style: 18,
    ai_tribe: 23,
    difficulty: 4,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1369-eastindies-persian-gunpowder-start.txt",
    long: (
        "gamelog-run1370-eastindies-persian-gunpowder-trace.txt",
        "rontrace-run1370.log",
    ),
    // The game ended itself on 5673 (closing block 5674).
    length: 5673,
    // Item 3698, first try: parts at 1180 by the count and the sequence,
    // ours 17 draws against theirs 14, index 7 — ours
    // `Leader::create_units+0x642`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (`docs/AI.md` §358).
    count: 1180,
    sequence: 1180,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_GUNPOWDER_PAIR_WORD: i64 =
    if PERSIAN_GUNPOWDER.count < PERSIAN_GUNPOWDER.sequence {
        PERSIAN_GUNPOWDER.count
    } else {
        PERSIAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn persian_gunpowder_pair_captures_say_their_lobby() {
    let p = &PERSIAN_GUNPOWDER;
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
        "the Persian Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3698; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Persians — and no other Persian lobby's: the coverage pair's and the
/// Medieval pair's `(18, 23, 5)`, this map at another difficulty, the
/// Sahara coverage pair's `(7, 23, 5)`, the Classical pair's `(14, 23, 5)`
/// and the AI word `GreatLakesPersianTier2`'s `(14, 23, 2)`, other maps
/// (item 3698).
#[test]
fn persian_gunpowder_pair_is_no_other_lobby() {
    let p = &PERSIAN_GUNPOWDER;
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
        "the battery fields the Persians"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Persian lobby on this map at this difficulty"
        );
    }
    let w = &super::persian_word::PERSIAN;
    let c = &super::coverage_pair::COVERAGE;
    let s = &super::sahara_coverage::SAHARA_COVERAGE;
    let k = &super::persian_classical_pair::PERSIAN_CLASSICAL;
    let m = &super::persian_medieval_pair::PERSIAN_MEDIEVAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Persians");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 1180** (item 3698):
/// only run1370's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_STRIDE: (i64, i64) = (101, 1101);

pub(crate) fn persian_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1370 stride",
        &[(
            PERSIAN_GUNPOWDER.long.0,
            WIDENING_PERSIAN_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_PERSIAN_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_the_long_s_stride_walks_back_from_1180() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 11, "every stride block in the window");
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
    // Item 3698.
    pin_eq!(w.firsts.len(), 61, "the stride's baseline");
}

/// run1370b: the word 1180's window, blocks 1095..1431 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 1101 and 250 past the word's own block 1181 (item 3698).
pub(crate) const RUN1370B: &str =
    "gamelog-run1370b-eastindies-persian-gunpowder-window-1095-1431.txt";

/// The window: block 1095 through 1431; the word 1180's own block is 1181.
pub(crate) const WIDENING_PERSIAN_GUNPOWDER_FRAME_1180: (i64, i64) = (1095, 1431);

/// **The word 1180's window**: run1370b walked from run1369's start with
/// the recorder on and every round compared (item 3698).
pub(crate) fn persian_gunpowder_frame_1180_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[PERSIAN_GUNPOWDER.start],
        true,
        PERSIAN_GUNPOWDER.long,
        "run1370b",
        &[(RUN1370B, WIDENING_PERSIAN_GUNPOWDER_FRAME_1180.0)],
        WIDENING_PERSIAN_GUNPOWDER_FRAME_1180,
        1,
        &[1181],
        true,
    )
}

#[test]
fn persian_gunpowder_pair_run1370b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_gunpowder_frame_1180_window() else {
        return;
    };
    pin_eq!(w.blocks, 336, "every captured block");
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
    // Item 3698, the first walk.
    pin_eq!(w.firsts.len(), 809, "run1370b baseline");
    // Every round, both directions.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3698): run1370's closing block 5674,
/// the whole map's units, walked from run1369's start.
#[test]
fn persian_gunpowder_pair_run1370_s_closing_state() {
    let _pins = Pins::hold();
    let p = &PERSIAN_GUNPOWDER;
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
        "Persian Gunpowder pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5674, "the closing block");
    // Item 3698, the first try: the pair parts at 1180.
    pin_eq!(e.compared, 238, "units compared");
    pin_eq!(
        e.counts(),
        [237, 19, 0, 4, 321, 1, 123],
        "the closing counts"
    );
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
