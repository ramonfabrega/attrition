//! **The Spanish at a held Industrial age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, item 3697): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Spanish (9)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple,
//! `(14, 9, 5)`, is new: the Spanish lobbies are `spanish_word`'s
//! `(7, 9, 3)`, `spanish_classical_pair`'s `(7, 9, 5)`,
//! `spanish_lakes_word`'s `(14, 9, 4)` — this map at another difficulty —
//! `spanish_east_word`'s `(18, 9, 3)` and `spanish_gunpowder_pair`'s
//! `(18, 9, 5)`; the battery fields no Spanish. Captures: run1367 (a
//! `DUMP_ALL` start) and run1368 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §357.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const SPANISH_INDUSTRIAL: Pair = Pair {
    name: "GreatLakesSpanishIndustrial",
    map_style: 14,
    ai_tribe: 9,
    difficulty: 5,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1367-greatlakes-spanish-industrial-start.txt",
    long: (
        "gamelog-run1368-greatlakes-spanish-industrial-trace.txt",
        "rontrace-run1368.log",
    ),
    // The game ended itself on 4200 (closing block 4201).
    length: 4200,
    // Item 3697, first try: 3229 by the count and the sequence, ours 22
    // draws against theirs 23, at index 19 ours `Farms::inc_time+0x1ae`,
    // theirs `Object::take_damage+0xe1` (`docs/AI.md` §357).
    count: 3229,
    sequence: 3229,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_INDUSTRIAL_PAIR_WORD: i64 =
    if SPANISH_INDUSTRIAL.count < SPANISH_INDUSTRIAL.sequence {
        SPANISH_INDUSTRIAL.count
    } else {
        SPANISH_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn spanish_industrial_pair_captures_say_their_lobby() {
    let p = &SPANISH_INDUSTRIAL;
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
        "the Spanish Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3697; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Spanish — and no other Spanish lobby's: `spanish_lakes_word`'s
/// `(14, 9, 4)`, this map at another difficulty, `spanish_word`'s
/// `(7, 9, 3)`, `spanish_classical_pair`'s `(7, 9, 5)`,
/// `spanish_east_word`'s `(18, 9, 3)` and `spanish_gunpowder_pair`'s
/// `(18, 9, 5)`, other maps (item 3697).
#[test]
fn spanish_industrial_pair_is_no_other_lobby() {
    let p = &SPANISH_INDUSTRIAL;
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
        "the battery fields the Spanish"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Spanish lobby on this map at this difficulty"
        );
    }
    let w = &super::spanish_word::SPANISH;
    let k = &super::spanish_lakes_word::SPANISH_LAKES;
    let e = &super::spanish_east_word::SPANISH_EAST;
    let c = &super::spanish_classical_pair::SPANISH_CLASSICAL;
    let g = &super::spanish_gunpowder_pair::SPANISH_GUNPOWDER;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Spanish");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3229** (item 3697):
/// only run1368's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_SPANISH_INDUSTRIAL_STRIDE: (i64, i64) = (101, 3201);

pub(crate) fn spanish_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_INDUSTRIAL.start],
        true,
        SPANISH_INDUSTRIAL.long,
        "run1368 stride",
        &[(
            SPANISH_INDUSTRIAL.long.0,
            WIDENING_SPANISH_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_SPANISH_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_industrial_pair_the_long_s_stride_walks_back_from_3229() {
    let _pins = Pins::hold();
    let Some(w) = spanish_industrial_stride_window() else {
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
    // Item 3697: 99 keys, all but six a standing family — `form` on 27
    // units, the buildings' `build:extra` (which the stride does not
    // print) and who=1's groups 64..66's `held` and pools as each is
    // minted. **The first unit row is on the stride's last block 3201**:
    // the human's Citizen `0/4` (TY 50) wounded on 3189 in ours against
    // 3188 in theirs (`damage_frame`), and its position after it.
    pin_eq!(w.firsts.len(), 99, "the stride's baseline");
}

/// run1368b: the word 3229's window, blocks 3195..3480 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3201 and 250 past the word's own block 3230 (item 3697).
pub(crate) const RUN1368B: &str =
    "gamelog-run1368b-greatlakes-spanish-industrial-window-3195-3480.txt";

/// The window: block 3195 through 3480; the word 3229's own block is 3230.
pub(crate) const WIDENING_SPANISH_INDUSTRIAL_FRAME_3229: (i64, i64) = (3195, 3480);

/// **The word 3229's window**: run1368b walked from run1367's start with
/// the recorder on and every round compared (item 3697).
pub(crate) fn spanish_industrial_frame_3229_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[SPANISH_INDUSTRIAL.start],
        true,
        SPANISH_INDUSTRIAL.long,
        "run1368b",
        &[(RUN1368B, WIDENING_SPANISH_INDUSTRIAL_FRAME_3229.0)],
        WIDENING_SPANISH_INDUSTRIAL_FRAME_3229,
        1,
        &[3230],
        true,
    )
}

#[test]
fn spanish_industrial_pair_run1368b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_industrial_frame_3229_window() else {
        return;
    };
    pin_eq!(w.blocks, 285, "every captured block");
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
    // Item 3697, the first walk: **776** keys. Block 3195 carries the
    // standing families and the Citizen `0/4`'s `damage_frame` (ours 3189
    // against theirs 3188, the city `0/2000`'s `raid_stamp` with it).
    // **Who=1's group 64 Riflemen (TY 100 `RIFLEMAN`) `1/53`, `1/54` and
    // `1/55` loose every round from their own square in ours**: `1/54`'s
    // on 3198 from (4344, 33768, 480) against theirs' (4315, 33650, 648),
    // `total_time` 3 against 2. On the word 3229 `1/53`'s round at the
    // human's building `0/2003`, first printed on block 3229 — ours
    // (4392, 33912, 466) in 3 frames, theirs (4328, 33795, 588) in 2 —
    // lands in theirs
    // (`Object::take_damage+0xe1`, index 19) and ours holds it to 3230:
    // `0/2003`'s `build:damage` 5 in theirs on block 3230, ours 0.
    pin_eq!(w.firsts.len(), 776, "run1368b baseline");
    // Every round, both directions: 98 round-blocks theirs, 105 ours (each
    // of ours flies a frame longer), 55 parting, the first on 3198
    // (item 3697).
    pin_eq!(w.rounds_read, (98, 105), "the rounds read");
    pin_eq!(w.rounds.len(), 55, "the rounds that part");
}

/// **The pair's closing state** (item 3697): run1368's closing block 4201,
/// the whole map's units, walked from run1367's start.
#[test]
fn spanish_industrial_pair_run1368_s_closing_state() {
    let _pins = Pins::hold();
    let p = &SPANISH_INDUSTRIAL;
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
        "Spanish Industrial pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4201, "the closing block");
    // Item 3697, the first try: the pair parts at 3229, and the closing
    // block 4201 compares 176 units — who=1's army off, `1/170` torn and
    // `1/174` extra; who=1's buildings part in the closing fields
    // (`myhits` a tenth low in ours on every one) and cities 2000..2054.
    pin_eq!(e.compared, 176, "units compared");
    pin_eq!(e.counts(), [114, 0, 1, 4, 112, 0, 50], "the closing counts");
    pin_eq!(e.extra.clone(), vec![(1, 174)], "the closing extras");
}
