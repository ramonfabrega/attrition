//! **The French at a held Classical age, Great Sahara** (DECISIONS 90, 91,
//! 93, 94, item 3607): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **French (10)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical: `STARTING_TECHNOLOGY
//! 1`, `ENDING_TECHNOLOGY 1` — the Korean Classical pair's lobby with the
//! nation moved. The French's other lobbies are `french_word`'s
//! `GreatSaharaFrenchTier4`, `(7, 10, 4)`, and the third pair's East Indies
//! and Great Lakes, `(18, 10, 5)` and `(14, 10, 5)`, at the default age —
//! another difficulty, or another map. Captures: run1261 (a `DUMP_ALL`
//! start) and run1262 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §328.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const FRENCH_CLASSICAL: Pair = Pair {
    name: "GreatSaharaFrenchClassical",
    map_style: 7,
    ai_tribe: 10,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1261-greatsahara-french-classical-start.txt",
    long: (
        "gamelog-run1262-greatsahara-french-classical-trace.txt",
        "rontrace-run1262.log",
    ),
    // The game ended itself on 6106 (closing block 6107).
    length: 6106,
    // Item 3607, first try: whole — frame 6106, the game's last, by the
    // count and the sequence (`docs/AI.md` section 328).
    count: 6106,
    sequence: 6106,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const FRENCH_CLASSICAL_PAIR_WORD: i64 =
    if FRENCH_CLASSICAL.count < FRENCH_CLASSICAL.sequence {
        FRENCH_CLASSICAL.count
    } else {
        FRENCH_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn french_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &FRENCH_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "FRENCH_CLASSICAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 1, and the long is scoreable.
#[test]
fn french_classical_pair_captures_say_their_lobby() {
    let p = &FRENCH_CLASSICAL;
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
        "the French Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the AI word `GreatSaharaFrenchTier4` is
/// `(7, 10, 4)` — this map at another difficulty — and the third pair's
/// French lobbies `(18, 10, 5)` and `(14, 10, 5)` other maps (item 3607).
#[test]
fn french_classical_pair_is_no_other_lobby() {
    let p = &FRENCH_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let f = &super::french_word::FRENCH;
    for l in [
        (f.map_style, f.ai_tribe, f.difficulty),
        (18, 10, 5),
        (14, 10, 5),
    ] {
        assert_ne!(l, triple);
    }
}

/// **The pair's closing state**: run1262's closing dump, block 6107,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3607).
#[test]
fn french_classical_pair_run1262_s_closing_state() {
    let _pins = Pins::hold();
    let p = &FRENCH_CLASSICAL;
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
        "French Classical pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 6107, "the closing block");
    // Item 3607, whole on its first try: 254 units compared, none torn,
    // off or unlinked, no building or city field-row.
    pin_eq!(e.compared, 254, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// **The long, from the disk** (item 3607): the word is the game's last
/// frame, 6106, and run1262 itself prints the stride's blocks 101..6101 and
/// the closing block 6107 at its `endgame:` detail, so the widening that
/// holds the word walks the whole long rather than a new window.
pub(crate) const WIDENING_FRENCH_CLASSICAL_CLOSING: (i64, i64) = (101, 6107);

pub(crate) fn french_classical_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH_CLASSICAL.start],
        true,
        FRENCH_CLASSICAL.long,
        "run1262 closing",
        &[(FRENCH_CLASSICAL.long.0, WIDENING_FRENCH_CLASSICAL_CLOSING.0)],
        WIDENING_FRENCH_CLASSICAL_CLOSING,
        1,
        &[6107],
        true,
    )
}

#[test]
fn french_classical_pair_run1262_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_classical_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 62, "every stride block and the closing block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
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
    // Item 3607, the pair whole on its first try: **154** keys. The
    // standing families from block 101 (`form`, `build:extra`, who=1's
    // groups' `held` and pools), and one unit row: the Light Horse `1/50`
    // (TypeIndex 209, land, uid 82) holds `hits_left` 68 and `damage` 5
    // in ours against 73 and 0 in theirs from block 5601 (agreeing on
    // 5501, when it took an `ATTACKORDER` on `0/12`), standing to the
    // close with no draw parting (parked 3607a). `1/37`'s `dest_angle` on
    // 6101 (3574b). The closing block 6107 adds the human's defeat on
    // 6106, as on 3578c's and 3609's: who=0's `SITE[i].reg`,
    // `aggression[1]` and `leader:defense`, who=1's `num_queued` and six
    // `queue:queued` rows, ours 1..6 against 0, `bucket[3:knowledge]` ours
    // 4739 against 4239 (3609a's shape), two idle Phalanxes' (`1/89`,
    // `1/221`, TypeIndex 133) `g.cur_anim`/`g.end_time`, and the Light
    // Horse `1/182`'s `guard.retry` 7 against 6.
    pin_eq!(w.firsts.len(), 154, "the closing walk's baseline");
}
