//! **The British at a held Medieval age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, item 3701): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **British (11)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 11, 4)`, is new: the British hold this map at 0 (the first pair)
//! and 5 (the second), and difficulty 4 on East Indies
//! (`british_classical_pair`'s `(18, 11, 4)`) and Great Sahara
//! (`british_industrial_pair`'s `(7, 11, 4)`); the battery fields no
//! British. Captures: run1375 (a `DUMP_ALL` start) and run1376 (the draw
//! stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of
//! 100, walked from the start's initial state). The first try is in
//! `docs/AI.md` §361.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BRITISH_MEDIEVAL: Pair = Pair {
    name: "GreatLakesBritishMedieval",
    map_style: 14,
    ai_tribe: 11,
    difficulty: 4,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1375-greatlakes-british-medieval-start.txt",
    long: (
        "gamelog-run1376-greatlakes-british-medieval-trace.txt",
        "rontrace-run1376.log",
    ),
    // The game ended itself on 5282 (closing block 5283).
    length: 5282,
    // Item 3701, first try: whole, 5282 of 5282 by the count and the
    // sequence (`docs/AI.md` §361).
    count: 5282,
    sequence: 5282,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_MEDIEVAL_PAIR_WORD: i64 =
    if BRITISH_MEDIEVAL.count < BRITISH_MEDIEVAL.sequence {
        BRITISH_MEDIEVAL.count
    } else {
        BRITISH_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn british_medieval_pair_captures_say_their_lobby() {
    let p = &BRITISH_MEDIEVAL;
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
        "the British Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3701; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no British — and no other British pair's: the Classical pair's
/// `(18, 11, 4)`, the Enlightenment pair's `(18, 11, 5)` and the
/// Industrial pair's `(7, 11, 4)`, other maps (item 3701).
#[test]
fn british_medieval_pair_is_no_other_lobby() {
    let p = &BRITISH_MEDIEVAL;
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
        "the battery fields the British"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another British lobby on this map at this difficulty"
        );
    }
    let c = &super::british_classical_pair::BRITISH_CLASSICAL;
    let e = &super::british_enlightenment_pair::BRITISH_ENLIGHTENMENT;
    let i = &super::british_industrial_pair::BRITISH_INDUSTRIAL;
    for (name, l) in [
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the British");
        assert_ne!(l.0, p.map_style, "{name} is on this map");
    }
}

/// run1376b: the pair's last blocks, 5027..5283 at the long's detail with
/// `LEADERS=9` and `AMMO=5` — the word 5282 is the game's last frame and
/// 5283 its closing block (item 3701). run1376 prints units only every
/// hundredth block, no `AMMO` and no `LEADERS`, `BUILDS` or `CITIES`
/// record past the start, so no dump held the last blocks.
pub(crate) const RUN1376B: &str =
    "gamelog-run1376b-greatlakes-british-medieval-window-5027-5283.txt";

/// The window: block 5027 through the closing block 5283.
pub(crate) const WIDENING_BRITISH_MEDIEVAL_CLOSING: (i64, i64) = (5027, 5283);

/// **The pair's last blocks**: run1376b walked from run1375's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3701).
pub(crate) fn british_medieval_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_MEDIEVAL.start],
        true,
        BRITISH_MEDIEVAL.long,
        "run1376b",
        &[(RUN1376B, WIDENING_BRITISH_MEDIEVAL_CLOSING.0)],
        WIDENING_BRITISH_MEDIEVAL_CLOSING,
        1,
        &[5283],
        true,
    )
}

#[test]
fn british_medieval_pair_run1376b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_medieval_closing_window() else {
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
    // Item 3701, the first walk: **56** keys. Block 5027 carries the
    // standing families (`form` on 27 units, both leaders' `SITE[i].reg`
    // on 10 rows, who=1's groups 64..71's `held` and pools, who=0's
    // `aggression[1]`); after it, who=1's groups 69 (5088, ours [241]
    // against none) and 72 (5160, ours [248, 249, 250] against none), the
    // same family. The draw stream and the closing block 5283 agree.
    pin_eq!(w.firsts.len(), 56, "run1376b baseline");
    // Every round, both directions: 296 round-blocks each side. **10 rows
    // part, every one a launch** of a Trebuchet (TI 266, TY 216, packed,
    // the lead figure on **piece 216**, slot 12 `CHAR_ATTACK2`, which
    // `launch::BAYS` has no row for): `1/142`'s `ammo[1]` on 5027 leaves
    // (6840, 30264, 301) in ours — the figure's own square — against
    // theirs' (7138, 30223, 994), `v1z` 205446716 against 187209869;
    // `1/60`'s `ammo[0]` on 5029 (6888, 31752, 159) against (7182, 31820,
    // 852), `v1z` 209183548 against 190946716; each with its `angle`.
    pin_eq!(w.rounds_read, (296, 296), "the rounds read");
    pin_eq!(w.rounds.len(), 10, "the rounds that part");
}

/// **The pair's closing state** (item 3701): run1376's closing block 5283,
/// the whole map's units, walked from run1375's start.
#[test]
fn british_medieval_pair_run1376_s_closing_state() {
    let _pins = Pins::hold();
    let p = &BRITISH_MEDIEVAL;
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
        "British Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5283, "the closing block");
    // Item 3701, the pair whole on its first try: 274 units compared, none
    // off, torn, unlinked or extra, no building row and no closing field.
    pin_eq!(e.compared, 274, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
