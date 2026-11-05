//! **The Germans at a held Industrial age, Great Lakes, at Moderate**
//! (DECISIONS 90, 91, 93, 94, 97, 98, 99, 101, 102, item 3867): Great
//! Lakes, `MAP_STYLE 14`, human Nubians (4) against the **Germans (12)** at
//! **Moderate (3)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Industrial: `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5` —
//! `german_industrial_pair`'s lobby at another difficulty, so its name
//! carries the difficulty as `GreatSaharaToughest` does. Its triple,
//! `(14, 12, 3)`, is new (`tools/refill.py 14 12 3` read free): the
//! Germans hold this map at 4 (`german_modern_pair`) and 5
//! (`german_industrial_pair`), Great Sahara at 3 (`german_sahara_word`) and
//! 5 (`german_ancient_pair`), and East Indies at 3 (`german_word`) and 5
//! (`german_alltech_pair`); the battery fields no Germans. The lobby tests
//! `docs/GOLDEN.md` §154.1 3: the German computer fields
//! `Machine Gun[MACHINEGUNGERMAN]` (124), never the generic 123, from an
//! Industrial start. Captures: run1490 (a `DUMP_ALL` start) and run1491
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §381.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GERMAN_INDUSTRIAL_MODERATE: Pair = Pair {
    name: "GreatLakesGermanIndustrialModerate",
    map_style: 14,
    ai_tribe: 12,
    difficulty: 3,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1490-greatlakes-german-industrial-moderate-start.txt",
    long: (
        "gamelog-run1491-greatlakes-german-industrial-moderate-trace.txt",
        "rontrace-run1491.log",
    ),
    // The game ended itself on 4784 (closing block 4785).
    length: 4784,
    // Item 3867, first try: whole — count 4784, sequence 4784 of 4784 (the
    // game's last frame), walked from run1490's start (`docs/AI.md` §381).
    count: 4784,
    sequence: 4784,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_INDUSTRIAL_MODERATE_PAIR_WORD: i64 =
    if GERMAN_INDUSTRIAL_MODERATE.count < GERMAN_INDUSTRIAL_MODERATE.sequence {
        GERMAN_INDUSTRIAL_MODERATE.count
    } else {
        GERMAN_INDUSTRIAL_MODERATE.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn german_industrial_moderate_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_INDUSTRIAL_MODERATE;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_INDUSTRIAL_MODERATE {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5 and the difficulty at 3, and the long is
/// scoreable.
#[test]
fn german_industrial_moderate_pair_captures_say_their_lobby() {
    let p = &GERMAN_INDUSTRIAL_MODERATE;
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
        "the German Industrial Moderate pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3867; the battery's identity
/// test refuses it), once; the battery fields no Germans — and no other
/// German lobby's: the Modern pair's `(14, 12, 4)` and the Industrial
/// pair's `(14, 12, 5)`, this map at other difficulties; the Sahara word's
/// `(7, 12, 3)`, the Ancient pair's `(7, 12, 5)`, the word's `(18, 12, 3)`
/// and the All Technologies pair's `(18, 12, 5)`, other maps (item 3867).
#[test]
fn german_industrial_moderate_pair_is_no_other_lobby() {
    let p = &GERMAN_INDUSTRIAL_MODERATE;
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
    let m = &super::german_modern_pair::GERMAN_MODERN;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (s.name, (s.map_style, s.ai_tribe, s.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
        (n.name, (n.map_style, n.ai_tribe, n.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Germans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The Germans' machine gun is their own** (`docs/GOLDEN.md` §154.1 1
/// and 3, item 3867): no machine gun stands at the start; run1491's first
/// `Machine Gun[MACHINEGUNGERMAN]` (124) stands on block 4601, on the
/// German seat, and no block holds the generic Machine Gun (123). The
/// claim's killers: a 123 on any block (§154.1 1), or no 124 by the game's
/// end. The dump prints a guy's `type` on the stride blocks too, so this
/// reads the text rather than the typed frames.
#[test]
fn german_industrial_moderate_pair_s_computer_fields_the_german_machine_gun() {
    let p = &GERMAN_INDUSTRIAL_MODERATE;
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    let mut frame = 0i64;
    let mut seen: Vec<(i64, i64)> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(n) = t.strip_prefix("BEGIN FRAME ") {
            frame = n.trim().parse().unwrap_or(frame);
        } else if let Some(k) = t.strip_prefix("type ") {
            if let Ok(k @ (123 | 124)) = k.trim().parse::<i64>() {
                if seen.last() != Some(&(frame, k)) {
                    seen.push((frame, k));
                }
            }
        }
    }
    assert_eq!(
        seen,
        [(4601, 124), (4701, 124), (4785, 124)],
        "the blocks that hold a machine gun, and which"
    );
}

/// **The long's stride, the whole game** (item 3867): the pair is whole on
/// its first try, so the stride walks every block run1491 prints, 101
/// through 4701, the last before the closing block 4785.
pub(crate) const WIDENING_GERMAN_INDUSTRIAL_MODERATE_STRIDE: (i64, i64) = (101, 4701);

pub(crate) fn german_industrial_moderate_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_INDUSTRIAL_MODERATE.start],
        true,
        GERMAN_INDUSTRIAL_MODERATE.long,
        "run1491 stride",
        &[(
            GERMAN_INDUSTRIAL_MODERATE.long.0,
            WIDENING_GERMAN_INDUSTRIAL_MODERATE_STRIDE.0,
        )],
        WIDENING_GERMAN_INDUSTRIAL_MODERATE_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn german_industrial_moderate_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = german_industrial_moderate_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 47, "every stride block in the window");
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
    // Item 3867, whole on the first try: 107 keys over 47 blocks, every
    // one a standing family — the buildings' `build:extra` (64, which the
    // stride does not print), `form` on 27 units, and who=1's groups
    // 64..71' `held` and pools (8 each). No value row; no draw parts.
    pin_eq!(w.firsts.len(), 107, "the stride's baseline");
}

/// run1491b: the pair's last blocks, 4528..4785 at the long's detail with
/// `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5`, through the closing block
/// (item 3867). The pair is whole on its first try, so its word is the
/// game's last frame; run1491 prints units only every hundred frames, with
/// no leader, building, city or round record past the start.
pub(crate) const RUN1491B: &str =
    "gamelog-run1491b-greatlakes-german-industrial-moderate-closing-window-4528-4785.txt";

/// The window: block 4528 through the closing block 4785.
pub(crate) const WIDENING_GERMAN_INDUSTRIAL_MODERATE_CLOSING: (i64, i64) = (4528, 4785);

/// **The pair's last blocks**: run1491b walked from run1490's start with
/// the recorder on and every round compared (item 3867).
pub(crate) fn german_industrial_moderate_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_INDUSTRIAL_MODERATE.start],
        true,
        GERMAN_INDUSTRIAL_MODERATE.long,
        "run1491b",
        &[(RUN1491B, WIDENING_GERMAN_INDUSTRIAL_MODERATE_CLOSING.0)],
        WIDENING_GERMAN_INDUSTRIAL_MODERATE_CLOSING,
        1,
        &[4785],
        true,
    )
}

#[test]
fn german_industrial_moderate_pair_run1491b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_industrial_moderate_closing_window() else {
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
    // Item 3867, whole on the first try: **52** keys, every one a standing
    // family — `form` (23), who=1's groups' `held` and pools (9 each),
    // who=0's `SITE[0..9].reg` (1 against 0) and `aggression[1]` (0 against
    // 1). No draw parts to 4784.
    pin_eq!(w.firsts.len(), 52, "run1491b baseline");
    // Every round, both directions: 1015 against 1015, and one parts, on
    // two fields: the Armored Car `1/35`'s (TI 216, land) `ammo[3]` on 4564
    // flies at the Citizen `0/12`, which died on 4563 — theirs keeps `ox`
    // 12 and `whom` 0, ours reads -1 for both. 3714a's family on a second
    // lobby (parked 3867a); it lands nowhere a draw feels.
    pin_eq!(w.rounds_read, (1015, 1015), "the rounds read");
    pin_eq!(w.rounds.len(), 2, "the rounds that part");
}

/// **The pair's closing state** (item 3867): run1491's closing block,
/// the whole map's units, walked from run1490's start.
#[test]
fn german_industrial_moderate_pair_run1491_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GERMAN_INDUSTRIAL_MODERATE;
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
        "German Industrial Moderate pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4785, "the closing block");
    // Item 3867, the pair whole on its first try: 148 units compared, none
    // off, torn, unlinked or extra, no building row and no closing field
    // (`docs/AI.md` §381).
    pin_eq!(e.compared, 148, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
