//! **The British at a held Modern age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, item 3752): Great Lakes, `MAP_STYLE 14`,
//! human Nubians (4) against the **British (11)** at **Toughest (5)**,
//! seed 12345, `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 11, 5)`, is the second pair's (an unheld age) and no held-age
//! pair's: the British held ages sit on Great Lakes at 4
//! (`british_medieval_pair`), East Indies at 4 (`british_classical_pair`)
//! and 5 (`british_enlightenment_pair`, which shares the second pair's
//! `(18, 11, 5)` the same way) and Great Sahara at 4
//! (`british_industrial_pair`); the battery fields no British.
//! Captures: run1432 (a `DUMP_ALL` start) and run1433 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §376.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BRITISH_MODERN: Pair = Pair {
    name: "GreatLakesBritishModern",
    map_style: 14,
    ai_tribe: 11,
    difficulty: 5,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1432-greatlakes-british-modern-start.txt",
    long: (
        "gamelog-run1433-greatlakes-british-modern-trace.txt",
        "rontrace-run1433.log",
    ),
    // The game ended itself on 3793 (closing block 3794).
    length: 3793,
    // Item 3752, first try: whole — count 3793, sequence 3793 of 3793; no
    // draw parts to the game's last frame (`docs/AI.md` §376).
    count: 3793,
    sequence: 3793,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_MODERN_PAIR_WORD: i64 = if BRITISH_MODERN.count < BRITISH_MODERN.sequence {
    BRITISH_MODERN.count
} else {
    BRITISH_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_MODERN {}: count {} sequence {} of {}; {}",
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
fn british_modern_pair_captures_say_their_lobby() {
    let p = &BRITISH_MODERN;
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
        "the British Modern pair's long is not scoreable"
    );
}

/// The pair's triple is a scored lobby, the second pair's `(14, 11, 5)`
/// (item 3752: entered there, not minted — as the Enlightenment pair shares
/// the second pair's `(18, 11, 5)`), and none of the battery's, which
/// fields no British; and no other held-age British pair's: the Medieval
/// pair's `(14, 11, 4)`, this map at Hard, the Classical pair's
/// `(18, 11, 4)`, the Enlightenment pair's `(18, 11, 5)`, East Indies, and
/// the Industrial pair's `(7, 11, 4)`, Great Sahara.
#[test]
fn british_modern_pair_is_no_other_lobby() {
    let p = &BRITISH_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields a British game"
    );
    let m = &super::british_medieval_pair::BRITISH_MEDIEVAL;
    let c = &super::british_classical_pair::BRITISH_CLASSICAL;
    let e = &super::british_enlightenment_pair::BRITISH_ENLIGHTENMENT;
    let i = &super::british_industrial_pair::BRITISH_INDUSTRIAL;
    for (name, l) in [
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the British");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}
/// **The long's stride** (item 3752): the pair is whole, so the window ends
/// where the stride does — run1433's stride blocks 101 through 3701.
pub(crate) const WIDENING_BRITISH_MODERN_STRIDE: (i64, i64) = (101, 3701);

pub(crate) fn british_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_MODERN.start],
        true,
        BRITISH_MODERN.long,
        "run1433 stride",
        &[(BRITISH_MODERN.long.0, WIDENING_BRITISH_MODERN_STRIDE.0)],
        WIDENING_BRITISH_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn british_modern_pair_the_long_s_stride_is_widened() {
    let _pins = Pins::hold();
    let Some(w) = british_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 37, "every stride block in the window");
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
    // Item 3752: 105 keys over 37 blocks — the standing families
    // (`build:extra` on 57 buildings, which the stride does not print,
    // `form` on 27 units, who=1's groups' `held` and pools) and `mirror`
    // on five of who=1's aircraft from 3201: the Lancaster Bombers `1/100`,
    // `1/108` and `1/112` (type 307) and the Helicopters `1/111` and
    // `1/128` (type 310), 0 against 1. No draw parts to the game's end.
    pin_eq!(w.firsts.len(), 105, "the stride's baseline");
}

/// run1433b: the pair's closing window, blocks 3538..3794 at the long's
/// detail with `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5` — 256 blocks to
/// the closing block 3794 (item 3752). The pair is whole on its first try;
/// run1433's stride prints units only every hundred frames, no `AMMO` and
/// no `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1433B: &str =
    "gamelog-run1433b-greatlakes-british-modern-closing-window-3538-3794.txt";

/// The window: block 3538 through the closing block 3794.
pub(crate) const WIDENING_BRITISH_MODERN_CLOSING: (i64, i64) = (3538, 3794);

/// **The pair's last blocks**: run1433b walked from run1432's start with
/// the recorder on and every round compared (item 3752).
pub(crate) fn british_modern_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_MODERN.start],
        true,
        BRITISH_MODERN.long,
        "run1433b",
        &[(RUN1433B, WIDENING_BRITISH_MODERN_CLOSING.0)],
        WIDENING_BRITISH_MODERN_CLOSING,
        1,
        &[3794],
        true,
    )
}

#[test]
fn british_modern_pair_run1433b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_modern_closing_window() else {
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
    // Item 3752, the pair whole on its first try: **61** keys. Block 3538
    // carries the standing families — the leaders' `SITE[n].reg` (1
    // against 0), who=0's `aggression[1]` (0 against 1), `form`, who=1's
    // groups' `held` and pools — and `mirror` on the Lancaster Bombers
    // `1/100`, `1/108`, `1/112` and the Helicopters `1/111`, `1/124`
    // (`1/128` from 3695); group 71's `held` from 3763; `0/4`'s
    // `damage_frac` 3 against 10 on 3776 and `0/2000`'s 8 against 15 on
    // 3790, the bombs' landings. No draw parts to 3793.
    pin_eq!(w.firsts.len(), 61, "run1433b baseline");
    pin_eq!(w.rounds_read, (243, 243), "the rounds read");
    // Every round, both directions: 243 a side, 75 parting — every one a
    // Lancaster Bomber's (TI 307, `LANCASTERBOMBER`), `1/100` on 62 rows
    // and `1/108` on 13: each bomb leaves at `sz` 2160 against 2130 and
    // off its `sx, sy` by up to (88, 155) — `1/100`'s `ammo[0]` on 3760 at
    // (3710, 30659, 2160) against (3698, 30814, 2130), the guy's own
    // position in ours — its `v1z` −6385656 against −4620950, and the
    // first five bombs' `accuracy` 0 against 300 or 252 (parked 3752a).
    pin_eq!(w.rounds.len(), 75, "the rounds that part");
}

/// **The pair's closing state** (item 3752): run1433's closing block 3794,
/// the whole map's units, walked from run1432's start.
#[test]
fn british_modern_pair_run1433_s_closing_state() {
    let _pins = Pins::hold();
    let p = &BRITISH_MODERN;
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
        "British Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3794, "the closing block");
    // Item 3752, whole on its first try: 146 units compared, counts
    // [0, 0, 3, 0, 0, 0, 1] — three who=1 units, `1/136`..`1/138`, born
    // in ours on the closing step alone (the traced walk holds none of
    // them through 3793; theirs' 3794 holds none, parked 3752b), and the
    // city `1/2050`'s `assimilation_timer` 3793 against 3790 (3760d's
    // family); nothing off, torn or unlinked. A floor, not a score.
    pin_eq!(e.compared, 146, "units compared");
    pin_eq!(e.counts(), [0, 0, 3, 0, 0, 0, 1], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        vec![(1, 136), (1, 137), (1, 138)],
        "the closing extras"
    );
}
