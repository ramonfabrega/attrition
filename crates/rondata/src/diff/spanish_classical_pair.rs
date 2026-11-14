//! **The Spanish at a held Classical age, Great Sahara** (DECISIONS 90,
//! 91, 93, item 3586): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Spanish (9)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical: `STARTING_TECHNOLOGY
//! 1`, `ENDING_TECHNOLOGY 1` — the Roman Classical pair's lobby with the
//! nation moved. The Spanish's other lobbies are `spanish_word`'s
//! `GreatSaharaSpanishTier3`, `(7, 9, 3)`, `spanish_east_word`'s
//! `EastIndiesSpanishTier3`, `(18, 9, 3)`, and `spanish_lakes_word`'s
//! `GreatLakesSpanishTier4`, `(14, 9, 4)` — another difficulty, or another
//! map. Captures: run1231 (a `DUMP_ALL` start) and run1232 (the draw stream
//! to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §319.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const SPANISH_CLASSICAL: Pair = Pair {
    name: "GreatSaharaSpanishClassical",
    map_style: 7,
    ai_tribe: 9,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1231-greatsahara-spanish-classical-start.txt",
    long: (
        "gamelog-run1232-greatsahara-spanish-classical-trace.txt",
        "rontrace-run1232.log",
    ),
    // The game ended itself on 8565 (closing block 8566).
    length: 8565,
    // Item 3586, first try: whole — frame 8565 of 8565 by the count and
    // the sequence (`docs/AI.md` section 319).
    count: 8565,
    sequence: 8565,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_CLASSICAL_PAIR_WORD: i64 =
    if SPANISH_CLASSICAL.count < SPANISH_CLASSICAL.sequence {
        SPANISH_CLASSICAL.count
    } else {
        SPANISH_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn spanish_classical_pair_captures_say_their_lobby() {
    let p = &SPANISH_CLASSICAL;
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
        "the Spanish Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery holds no Spanish game, and the
/// Spanish's three scored words are `(7, 9, 3)` — this map at another
/// difficulty — `(18, 9, 3)` and `(14, 9, 4)` (item 3586).
#[test]
fn spanish_classical_pair_is_no_other_lobby() {
    let p = &SPANISH_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for l in [
        &super::spanish_word::SPANISH,
        &super::spanish_east_word::SPANISH_EAST,
        &super::spanish_lakes_word::SPANISH_LAKES,
    ] {
        assert_ne!((l.map_style, l.ai_tribe, l.difficulty), triple);
    }
}

/// **The pair's closing state**: run1232's closing dump, block 8566,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3586).
#[test]
fn spanish_classical_pair_run1232_s_closing_state() {
    let _pins = Pins::hold();
    let p = &SPANISH_CLASSICAL;
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
        "Spanish Classical pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 8566, "the closing block");
    // Item 3586, the pair whole on its first try: 395 units compared, none
    // off, torn or unlinked, and no building or city field-row.
    pin_eq!(e.compared, 395, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1232b: the pair's last blocks, 8310..8566 at the long's detail —
/// the word 8565 is the game's last frame and 8566 its closing block (item
/// 3586). run1232 prints units only every hundredth block and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start, so no dump held
/// the last blocks.
pub(crate) const RUN1232B: &str =
    "gamelog-run1232b-greatsahara-spanish-classical-window-8310-8566.txt";

/// The window: block 8310 through the closing block 8566.
pub(crate) const WINDOW_SPANISH_CLASSICAL_CLOSING: (i64, i64) = (8310, 8566);

/// **The pair's last blocks**: run1232b walked from run1231's start with
/// the recorder on. The word is the game's last frame (item 3586).
pub(crate) fn spanish_classical_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_CLASSICAL.start],
        true,
        SPANISH_CLASSICAL.long,
        "run1232b",
        &[(RUN1232B, WINDOW_SPANISH_CLASSICAL_CLOSING.0)],
        WINDOW_SPANISH_CLASSICAL_CLOSING,
        1,
        &[8566],
        true,
    )
}

#[test]
fn spanish_classical_pair_run1232b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_classical_closing_window() else {
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
    // Item 3586, the first walk: **82** keys. Block 8310 carries the
    // standing families (`form`, `SITE[i].reg`, `aggression[1]`, who=1's
    // groups' `held`/`pool`, more groups on 8331 and 8539) and both
    // leaders' `territory`, one tile apart — who=0 ours 241 against
    // theirs 242, who=1 2746 against 2745, `my_team_terr`,
    // `other_team_terr` and `min_other_team_terr` with it (3574a's shape,
    // parked 3586a). `g.angle[0]` on seven of who=1's Scholars (TypeIndex
    // 52), garrisoned (one `inside` chain, `inside_up` 2006 at its head),
    // ours 1431655765 against theirs 0, from
    // 8310 (`1/206`, `1/208`, `1/212`, `1/365`, `1/368`), 8377 (`1/373`)
    // and 8500 (`1/380`); and twelve `dest_angle` rows from 8358 (`1/79`
    // first; 3567a's shape, parked 3580b). Positions, orders and hits
    // agree to the closing block.
    // Item 3595: 82 -> 74, the eight-slot territory tie (`docs/AI.md`
    // section 318.2; parked 3586a answered): both leaders' `territory`
    // and team sums on 8310 agree.
    pin_eq!(w.firsts.len(), 74, "run1232b baseline");
}
