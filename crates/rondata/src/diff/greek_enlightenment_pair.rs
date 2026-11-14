//! **The Greeks at a held Enlightenment age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, item 3674): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **Greeks (5)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4`. Its triple,
//! `(14, 5, 5)`, is new: the Greek lobbies are `greek_word`'s
//! `GreatSaharaGreekTier1`, `(7, 5, 1)`, which left the battery for the AI
//! word, `greek_classical_pair`'s `(18, 5, 5)` and `greek_gunpowder_pair`'s
//! `(7, 5, 5)`, other maps; no battery game fields the Greeks. Captures:
//! run1347 (a `DUMP_ALL` start) and run1348 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §353.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const GREEK_ENLIGHTENMENT: Pair = Pair {
    name: "GreatLakesGreekEnlightenment",
    map_style: 14,
    ai_tribe: 5,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1347-greatlakes-greek-enlightenment-start.txt",
    long: (
        "gamelog-run1348-greatlakes-greek-enlightenment-trace.txt",
        "rontrace-run1348.log",
    ),
    // The game ended itself on 4705 (closing block 4706).
    length: 4705,
    // Item 3674, first try: whole, 4705 of 4705 by the count and the
    // sequence (`docs/AI.md` §353).
    count: 4705,
    sequence: 4705,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GREEK_ENLIGHTENMENT_PAIR_WORD: i64 =
    if GREEK_ENLIGHTENMENT.count < GREEK_ENLIGHTENMENT.sequence {
        GREEK_ENLIGHTENMENT.count
    } else {
        GREEK_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn greek_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GREEK_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GREEK_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn greek_enlightenment_pair_captures_say_their_lobby() {
    let p = &GREEK_ENLIGHTENMENT;
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
        "the Greek Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3674; the battery's identity
/// test refuses it), once, and none of the battery's: no battery game
/// fields the Greeks; and no other Greek lobby's: the AI word
/// `GreatSaharaGreekTier1`'s `(7, 5, 1)`, the Classical pair's
/// `(18, 5, 5)` and the Gunpowder pair's `(7, 5, 5)` (item 3674).
#[test]
fn greek_enlightenment_pair_is_no_other_lobby() {
    let p = &GREEK_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Greeks"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Greek lobby on this map at this difficulty"
        );
    }
    let w = &super::greek_word::GREEK;
    let c = &super::greek_classical_pair::GREEK_CLASSICAL;
    let g = &super::greek_gunpowder_pair::GREEK_GUNPOWDER;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Greeks");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The pair's closing state** (item 3674): run1348's closing block 4706,
/// the whole map's units, walked from run1347's start.
#[test]
fn greek_enlightenment_pair_run1348_s_closing_state() {
    let _pins = Pins::hold();
    let p = &GREEK_ENLIGHTENMENT;
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
        "Greek Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4706, "the closing block");
    // Item 3674, the pair whole on its first try: 237 units compared, none
    // off, torn, unlinked or extra, `Closing fields: none`.
    pin_eq!(e.compared, 237, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1348b: the pair's last blocks, 4450..4706 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — the word 4705 is the game's last frame and
/// 4706 its closing block (item 3674). run1348 prints units only every
/// hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, so no dump held the last blocks.
pub(crate) const RUN1348B: &str =
    "gamelog-run1348b-greatlakes-greek-enlightenment-window-4450-4706.txt";

/// The window: block 4450 through the closing block 4706.
pub(crate) const WIDENING_GREEK_ENLIGHTENMENT_CLOSING: (i64, i64) = (4450, 4706);

/// **The pair's last blocks**: run1348b walked from run1347's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3674).
pub(crate) fn greek_enlightenment_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GREEK_ENLIGHTENMENT.start],
        true,
        GREEK_ENLIGHTENMENT.long,
        "run1348b",
        &[(RUN1348B, WIDENING_GREEK_ENLIGHTENMENT_CLOSING.0)],
        WIDENING_GREEK_ENLIGHTENMENT_CLOSING,
        1,
        &[4706],
        true,
    )
}

#[test]
fn greek_enlightenment_pair_run1348b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_enlightenment_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3674, the first walk: **74** keys. Block 4450 carries the
    // standing families (`form` on 27 units, both leaders' `SITE[i].reg`,
    // who=0's `aggression[1]` ours 0 against theirs 1, who=1's groups 64..67,
    // 70 and 71's `held` and pools; 68's on 4503, 69's on 4533) and **one
    // family that is not standing**: who=1's four Barracks (type 427)
    // `1/2014`, `1/2017`, `1/2026` and `1/2028` hold their later queue
    // slots dearer in ours — Musketeers (99) 140/120 against theirs'
    // 125/108, Fusiliers (136) 120/99 against 108/89, Explorers (71) 44/44
    // against 35/35, the first two slots agreeing — and leader 1's four
    // buckets lower with them (food 2750 against 2804, timber 2202/2226,
    // wealth 15598/15607, metal 12198/12227); the same slots part again on
    // 4533, 4616 and 4698 as each moves up (parked 3674a). The draw stream
    // and the closing block 4706 agree.
    pin_eq!(w.firsts.len(), 74, "run1348b baseline");
    // Every round, both directions: 40 round-blocks a side, none parting.
    pin_eq!(w.rounds_read, (40, 40), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}
