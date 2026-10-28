//! **The Americans at a held Information age, Great Lakes** (DECISIONS
//! 90, 91, 93, 94, 97, 98, 99, 101, item 3725): Great Lakes, `MAP_STYLE
//! 14`, human Nubians (4) against the **Americans (20)** at difficulty 5,
//! seed 12345, `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple,
//! `(14, 20, 5)`, is new: the Americans hold this map at 4
//! (`american_lakes_word`), East Indies at 1 (`american_word`) and 5
//! (`american_enlightenment_pair`), and Great Sahara at 3
//! (`american_modern_pair`) and 5 (`american_industrial_pair`); the
//! battery fields no Americans. Captures: run1408 (a `DUMP_ALL` start)
//! and run1409 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §372.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AMERICAN_INFORMATION: Pair = Pair {
    name: "GreatLakesAmericanInformation",
    map_style: 14,
    ai_tribe: 20,
    difficulty: 5,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1408-greatlakes-american-information-start.txt",
    long: (
        "gamelog-run1409-greatlakes-american-information-trace.txt",
        "rontrace-run1409.log",
    ),
    // The game ended itself on 4320 (closing block 4321).
    length: 4320,
    // Item 3725, first try: count 2385, sequence 2385 of 4320 — on 2385
    // ours 9 draws against theirs' 10, index 6 ours'
    // `Guy::set_anim+0x97a` against theirs' sixth
    // `Leader::make_stuff+0x63d` (`SITE_EXPIRE_SLOT`): who=1's make list
    // holds one more slot to expire in theirs (`docs/AI.md` §372).
    count: 2385,
    sequence: 2385,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AMERICAN_INFORMATION_PAIR_WORD: i64 =
    if AMERICAN_INFORMATION.count < AMERICAN_INFORMATION.sequence {
        AMERICAN_INFORMATION.count
    } else {
        AMERICAN_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn american_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AMERICAN_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_INFORMATION {}: count {} sequence {} of {}; {}",
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
fn american_information_pair_captures_say_their_lobby() {
    let p = &AMERICAN_INFORMATION;
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
        "the American Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3725; the battery's identity
/// test refuses it), once, and none of the battery's — which fields no
/// Americans — and no other American lobby's: `american_lakes_word`'s
/// `(14, 20, 4)`, this map at another difficulty, `american_word`'s
/// `(18, 20, 1)`, `american_enlightenment_pair`'s `(18, 20, 5)`,
/// `american_modern_pair`'s `(7, 20, 3)` and `american_industrial_pair`'s
/// `(7, 20, 5)`, other maps (item 3725).
#[test]
fn american_information_pair_is_no_other_lobby() {
    let p = &AMERICAN_INFORMATION;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert_eq!(
        battery::BATTERY
            .iter()
            .filter(|l| l.ai_tribe == p.ai_tribe)
            .count(),
        0,
        "the battery fields the Americans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another American lobby on this map at this difficulty"
        );
    }
    let k = &super::american_lakes_word::AMERICAN_LAKES;
    let w = &super::american_word::AMERICAN;
    let e = &super::american_enlightenment_pair::AMERICAN_ENLIGHTENMENT;
    let m = &super::american_modern_pair::AMERICAN_MODERN;
    let i = &super::american_industrial_pair::AMERICAN_INDUSTRIAL;
    for (name, l) in [
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Americans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3725, from
/// 2385): only run1409's stride blocks are on disk before the word's
/// window, 101 through 2301.
pub(crate) const WIDENING_AMERICAN_INFORMATION_STRIDE: (i64, i64) = (101, 2301);

pub(crate) fn american_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1409 stride",
        &[(
            AMERICAN_INFORMATION.long.0,
            WIDENING_AMERICAN_INFORMATION_STRIDE.0,
        )],
        WIDENING_AMERICAN_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn american_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = american_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every stride block in the window");
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
    // Item 3725: 78 keys over 23 blocks, the standing families alone —
    // the buildings' `build:extra` (40, which the stride does not print),
    // `form` on 27 units, who=1's groups 64..68 with their pools (10), and
    // `1/14`'s `g.angle[0]` from 401. No position, order or hit parts.
    pin_eq!(w.firsts.len(), 78, "the stride's baseline");
}

/// run1409b: the word 2385's window, blocks 2295..2635 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2301 and 250 past the word's own block 2386 (item 3725).
/// run1409 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1409B: &str =
    "gamelog-run1409b-greatlakes-american-information-window-2295-2635.txt";

/// The window: block 2295 through 2635.
pub(crate) const WIDENING_AMERICAN_INFORMATION_FRAME_2385: (i64, i64) = (2295, 2635);

/// **The word 2385's window**: run1409b walked from run1408's start with
/// the recorder on and every round compared (item 3725).
pub(crate) fn american_information_frame_2385_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[AMERICAN_INFORMATION.start],
        true,
        AMERICAN_INFORMATION.long,
        "run1409b",
        &[(RUN1409B, WIDENING_AMERICAN_INFORMATION_FRAME_2385.0)],
        WIDENING_AMERICAN_INFORMATION_FRAME_2385,
        1,
        &[2386],
        true,
    )
}

#[test]
fn american_information_pair_run1409b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_information_frame_2385_window() else {
        return;
    };
    pin_eq!(w.blocks, 340, "every captured block");
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
    // Item 3725, the first walk: **1392** keys. Block 2295 carries 63
    // standing rows: `form` (27), who=0's and who=1's `aggression` (0
    // against 1), who=1's `SITE[i]` (14), groups 64..67 and their pools,
    // `1/14`'s `g.angle[0]`, **who=1's wealth and oil buckets** (17727
    // against 17799, 17415 against 17487) and `MAKE[3].val` (2112 against
    // 2188), and **the Airbase `1/2021`'s four queued Jet Fighters (TI
    // 295)** at `cost[0]` and `cost[1]` 89, 91, 93, 95 in ours against 71,
    // 73, 75, 77 — 4 × 18 = 72, the buckets' gap. `get_cost`'s American
    // aircraft arm (`AMERICANS_AIRCRAFT`, 20 %) is not carried (parked
    // 3669a). **The make list parts on 2384**: `MAKE[3]` reads `t` −1,
    // `num` 1, `escrow` 1 in ours against the Jet Fighter's `t` 295, `num`
    // 10, `escrow` 0, and `MAKE[2].val` 180836 against 191072; on 2385
    // theirs expires one more slot (the word). No round flies in the window
    // on either side (`docs/AI.md` §372).
    pin_eq!(w.firsts.len(), 1392, "run1409b baseline");
    // Every round, both directions: `AMMO=5` was staged and none prints
    // on either side over 2295..2635.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3725): run1409's closing block 4321,
/// the whole map's units, walked from run1408's start.
#[test]
fn american_information_pair_run1409_s_closing_state() {
    let _pins = Pins::hold();
    let p = &AMERICAN_INFORMATION;
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
        "American Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4321, "the closing block");
    // Item 3725, first try: 1935 frames past the word 2385 the game has
    // parted — 136 units compared, counts [118, 0, 7, 5, 145, 0, 63],
    // nothing torn or unlinked, ours' `1/128`..`1/134` extra. A floor, not
    // a score.
    pin_eq!(e.compared, 136, "units compared");
    pin_eq!(e.counts(), [118, 0, 7, 5, 145, 0, 63], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        vec![
            (1, 128),
            (1, 129),
            (1, 130),
            (1, 131),
            (1, 132),
            (1, 133),
            (1, 134)
        ],
        "the closing extras"
    );
}
