//! **The Spanish at a held Information age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, 99, 101, 102, item 3866): Great Lakes,
//! `MAP_STYLE 14`, human Nubians (4) against the **Spanish (9)** at
//! **Moderate (3)**, seed 12345, `STARTING_RESOURCES 7`, the age held at
//! Information: `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple,
//! `(14, 9, 3)`, is new (`tools/refill.py 14 9 3` read free): the Spanish
//! hold this map at 4 (`spanish_lakes_word`) and 5
//! (`spanish_industrial_pair`), Great Sahara at 3 (`spanish_word`) and 5
//! (`spanish_classical_pair`), and East Indies at 3 (`spanish_east_word`),
//! 4 (`spanish_modern_pair`) and 5 (`spanish_gunpowder_pair`); the battery
//! fields no Spanish. The lobby tests `docs/GOLDEN.md` §154's claim that a
//! start owns its own age's upgrades: the Spanish seat stands two
//! `Elite Special Forces[ELITESPECIALSPANISH]` (78) on frame 1. Captures:
//! run1488 (a `DUMP_ALL` start) and run1489 (the draw stream to the
//! game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked
//! from the start's initial state). The first try is in `docs/AI.md` §380.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const SPANISH_INFORMATION: Pair = Pair {
    name: "GreatLakesSpanishInformation",
    map_style: 14,
    ai_tribe: 9,
    difficulty: 3,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1488-greatlakes-spanish-information-start.txt",
    long: (
        "gamelog-run1489-greatlakes-spanish-information-trace.txt",
        "rontrace-run1489.log",
    ),
    // The game ended itself on 3500 (closing block 3501).
    length: 3500,
    // Item 3866, first try: whole — count 3500, sequence 3500 of 3500 (the
    // game's last frame), walked from run1488's start (`docs/AI.md` §380).
    count: 3500,
    sequence: 3500,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_INFORMATION_PAIR_WORD: i64 =
    if SPANISH_INFORMATION.count < SPANISH_INFORMATION.sequence {
        SPANISH_INFORMATION.count
    } else {
        SPANISH_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_INFORMATION {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 7 and the difficulty at 3, and the long is
/// scoreable.
#[test]
fn spanish_information_pair_captures_say_their_lobby() {
    let p = &SPANISH_INFORMATION;
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
        "the Spanish Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3866; the battery's identity
/// test refuses it), once; the battery fields no Spanish — and no other
/// Spanish lobby's: the Lakes word's `(14, 9, 4)` and the Industrial pair's
/// `(14, 9, 5)`, this map at other difficulties; the word's `(7, 9, 3)`,
/// the Classical pair's `(7, 9, 5)`, the East word's `(18, 9, 3)`, the
/// Modern pair's `(18, 9, 4)` and the Gunpowder pair's `(18, 9, 5)`, other
/// maps (item 3866).
#[test]
fn spanish_information_pair_is_no_other_lobby() {
    let p = &SPANISH_INFORMATION;
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
    let i = &super::spanish_industrial_pair::SPANISH_INDUSTRIAL;
    let m = &super::spanish_modern_pair::SPANISH_MODERN;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Spanish");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}
/// **A start owns its own age's upgrades** (`docs/GOLDEN.md` §154, item
/// 3866): the Spanish seat stands two `Elite Special Forces
/// [ELITESPECIALSPANISH]` (78) on the start dump's first frame — the
/// scout `Setup::build_units` places as `current_upgrade(SCOUT)`, and the
/// second, `SPANISH_EXTRA_SCOUT` (parked 3865c) — and the Nubian seat
/// stands the generic Elite Special Forces (77). The claim's killer: a
/// start whose Spanish seat holds no 78.
#[test]
fn spanish_information_pair_s_start_stands_the_spanish_elite_special_forces() {
    let p = &SPANISH_INFORMATION;
    let Some(path) = dump(p.start) else {
        return;
    };
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = log.initial().expect("an initial state");
    let of = |who: i64, kind: i64| {
        let mut os: Vec<i64> = init
            .units
            .iter()
            .filter(|u| u.who == who && u.guys.first().and_then(|g| g.kind) == Some(kind))
            .map(|u| u.o)
            .collect();
        os.sort_unstable();
        os
    };
    assert_eq!(of(1, 78), [0, 1], "the Spanish seat's 78s");
    assert_eq!(of(1, 77), Vec::<i64>::new(), "the Spanish seat's 77s");
    assert!(!of(0, 77).is_empty(), "the Nubian seat stands no 77");
    assert_eq!(of(0, 78), Vec::<i64>::new(), "the Nubian seat's 78s");
}

/// **The long's stride, the whole game** (item 3866): the pair is whole on
/// its first try, so the stride walks every block run1489 prints, 101
/// through 3401, the last before the closing block 3501.
pub(crate) const WIDENING_SPANISH_INFORMATION_STRIDE: (i64, i64) = (101, 3401);

pub(crate) fn spanish_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_INFORMATION.start],
        true,
        SPANISH_INFORMATION.long,
        "run1489 stride",
        &[(
            SPANISH_INFORMATION.long.0,
            WIDENING_SPANISH_INFORMATION_STRIDE.0,
        )],
        WIDENING_SPANISH_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = spanish_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 34, "every stride block in the window");
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
    // Item 3866, whole on the first try: 102 keys over 34 blocks — the
    // standing families, the buildings' `build:extra` (53, which the stride
    // does not print), `form` on 27 units, who=1's groups 64..70' `held`
    // and pools (7 each), and `mirror` on `1/63`, `1/75`, `1/87`, `1/102`
    // and `1/109` from 3101 and on `1/115` from 3401 (ours 0, theirs 1);
    // and one value row, the Assault Infantry `1/18`'s (type 104, land, at
    // its full 180 hits) `damage_frac`, ours 13 against theirs 12 from 3101
    // (parked 3866b). No draw parts. 102 → 100 (item 3986): the two
    // `damage_frac` rows of `1/18` agree once `UnitData::armor@00610160`'s
    // patriot terms are carried (`docs/GOLDEN.md` §179).
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1489b: the pair's last blocks, 3244..3501 at the long's detail with
/// `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5`, through the closing block
/// (item 3866). The pair is whole on its first try, so its word is the
/// game's last frame; run1489 prints units only every hundred frames, with
/// no leader, building, city or round record past the start.
pub(crate) const RUN1489B: &str =
    "gamelog-run1489b-greatlakes-spanish-information-closing-window-3244-3501.txt";

/// The window: block 3244 through the closing block 3501.
pub(crate) const WIDENING_SPANISH_INFORMATION_CLOSING: (i64, i64) = (3244, 3501);

/// **The pair's last blocks**: run1489b walked from run1488's start with
/// the recorder on and every round compared (item 3866).
pub(crate) fn spanish_information_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[SPANISH_INFORMATION.start],
        true,
        SPANISH_INFORMATION.long,
        "run1489b",
        &[(RUN1489B, WIDENING_SPANISH_INFORMATION_CLOSING.0)],
        WIDENING_SPANISH_INFORMATION_CLOSING,
        1,
        &[3501],
        true,
    )
}

#[test]
fn spanish_information_pair_run1489b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_information_closing_window() else {
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
    // Item 3866, whole on the first try: **47** keys — the standing
    // families, `form` (14), who=1's groups 64..70' `held` and pools (7
    // each), who=0's `SITE[0..9].reg` (1 against 0) and `aggression[1]` (0
    // against 1), `mirror` on `1/63`, `1/75`, `1/87`, `1/102`, `1/109` and
    // (from 3324) `1/115` (ours 0, theirs 1) — and the one value row the
    // stride carries, the Assault Infantry `1/18`'s `damage_frac`, ours 13
    // against theirs 12 (parked 3866b). No draw parts to 3500.
    pin_eq!(w.firsts.len(), 47, "run1489b baseline");
    // Every round, both directions: 85 against 85, none parting.
    pin_eq!(w.rounds_read, (85, 85), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3866): run1489's closing block,
/// the whole map's units, walked from run1488's start.
#[test]
fn spanish_information_pair_run1489_s_closing_state() {
    let _pins = Pins::hold();
    let p = &SPANISH_INFORMATION;
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
        "Spanish Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3501, "the closing block");
    // Item 3866, the pair whole on its first try: 114 units compared, none
    // off, torn, unlinked or extra, no building row; **one closing field**,
    // who=1's city `1/2046`'s `assimilation_timer`, ours 3500 against
    // theirs 3497 — 3699a's, 3742c's and 3760d's family on a fourth lobby
    // (parked 3866a). The draw stream never feels it (`docs/AI.md` §380).
    pin_eq!(e.compared, 114, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 1], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
