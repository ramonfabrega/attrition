//! **The Chinese at a held Modern age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, item 3751): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **Chinese (14)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(18, 14, 4)`, is new: the Chinese hold Great Sahara at 3
//! (`chinese_industrial_pair`) and 5 (`chinese_medieval_pair`), East
//! Indies at 5 (`chinese_gunpowder_pair`) and Great Lakes at 5
//! (`chinese_enlightenment_pair`); the battery's one Chinese game is
//! `GreatLakesChineseTier1`, `(14, 14, 1)`.
//! Captures: run1430 (a `DUMP_ALL` start) and run1431 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §375.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const CHINESE_MODERN: Pair = Pair {
    name: "EastIndiesChineseModern",
    map_style: 18,
    ai_tribe: 14,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1430-eastindies-chinese-modern-start.txt",
    long: (
        "gamelog-run1431-eastindies-chinese-modern-trace.txt",
        "rontrace-run1431.log",
    ),
    // The game ended itself on 4130 (closing block 4131).
    length: 4130,
    // Item 3751, first try: whole — count 4130, sequence 4130 of 4130; no
    // draw parts to the game's last frame (`docs/AI.md` §375).
    count: 4130,
    sequence: 4130,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const CHINESE_MODERN_PAIR_WORD: i64 = if CHINESE_MODERN.count < CHINESE_MODERN.sequence {
    CHINESE_MODERN.count
} else {
    CHINESE_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn chinese_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &CHINESE_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "CHINESE_MODERN {}: count {} sequence {} of {}; {}",
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
fn chinese_modern_pair_captures_say_their_lobby() {
    let p = &CHINESE_MODERN;
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
        "the Chinese Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3751; the battery's identity
/// test refuses it), once, and none of the battery's — its one Chinese
/// game is the held-out `GreatLakesChineseTier1`, `(14, 14, 1)` — and no
/// other pinned Chinese lobby's: the Industrial pair's `(7, 14, 3)` and
/// the Medieval pair's `(7, 14, 5)`, Great Sahara, the Gunpowder pair's
/// `(18, 14, 5)`, East Indies at another difficulty, and the Enlightenment
/// pair's `(14, 14, 5)`, Great Lakes (item 3751).
#[test]
fn chinese_modern_pair_is_no_other_lobby() {
    let p = &CHINESE_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    let chinese: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, (l.map_style, l.ai_tribe, l.difficulty)))
        .collect();
    assert_eq!(
        chinese,
        [("GreatLakesChineseTier1", (14, 14, 1))],
        "the battery's Chinese games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Chinese lobby on this map at this difficulty"
        );
    }
    let i = &super::chinese_industrial_pair::CHINESE_INDUSTRIAL;
    let m = &super::chinese_medieval_pair::CHINESE_MEDIEVAL;
    let g = &super::chinese_gunpowder_pair::CHINESE_GUNPOWDER;
    let e = &super::chinese_enlightenment_pair::CHINESE_ENLIGHTENMENT;
    for (name, l) in [
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Chinese");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride** (item 3751): the pair is whole, so the window ends
/// where the stride does — run1431's stride blocks 101 through 4101.
pub(crate) const WIDENING_CHINESE_MODERN_STRIDE: (i64, i64) = (101, 4101);

pub(crate) fn chinese_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_MODERN.start],
        true,
        CHINESE_MODERN.long,
        "run1431 stride",
        &[(CHINESE_MODERN.long.0, WIDENING_CHINESE_MODERN_STRIDE.0)],
        WIDENING_CHINESE_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn chinese_modern_pair_the_long_s_stride_is_widened() {
    let _pins = Pins::hold();
    let Some(w) = chinese_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 41, "every stride block in the window");
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
    // Item 3751: 207 keys over 41 blocks — the standing families (`form`
    // on 69 units, `build:extra` on 64 buildings, which the stride does not
    // print, who=1's groups' `held` and pools) and, from 3901, the
    // Submarine `1/139` (type 338, group 71) on its `ATTACKTOORDER`: the
    // order's point (45288, 37992) against theirs' (45000, 37608), its
    // formation offset (744, 360) against (456, 744), and the path with it.
    // No draw parts to the game's end (parked 3751a).
    // Item 3782, 207 → 177: the Submarine's re-chosen slot agrees —
    // `find_unit_with_radius` asks the disc's cells, so its block-0 re-place
    // no longer sees a unit across a cell line (`docs/AI.md` §378.2).
    pin_eq!(w.firsts.len(), 177, "the stride's baseline");
}

/// run1431b: the pair's closing window, blocks 3875..4131 at the long's
/// detail with `BUILDS`, `CITIES`, `LEADERS=9` and `AMMO=5` — 256 blocks to
/// the closing block 4131 (item 3751). The pair is whole on its first try;
/// run1431's stride prints units only every hundred frames, no `AMMO` and
/// no `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1431B: &str =
    "gamelog-run1431b-eastindies-chinese-modern-closing-window-3875-4131.txt";

/// The window: block 3875 through the closing block 4131.
pub(crate) const WIDENING_CHINESE_MODERN_CLOSING: (i64, i64) = (3875, 4131);

/// **The pair's last blocks**: run1431b walked from run1430's start with
/// the recorder on and every round compared (item 3751).
pub(crate) fn chinese_modern_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[CHINESE_MODERN.start],
        true,
        CHINESE_MODERN.long,
        "run1431b",
        &[(RUN1431B, WIDENING_CHINESE_MODERN_CLOSING.0)],
        WIDENING_CHINESE_MODERN_CLOSING,
        1,
        &[4131],
        true,
    )
}

#[test]
fn chinese_modern_pair_run1431b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_modern_closing_window() else {
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
    // Item 3751, the pair whole on its first try: **164** keys. Block 3875
    // carries the standing families — who=0's `aggression[1]` (0 against
    // 1), `form`, who=1's groups' `held` and pools — and the Submarine
    // `1/139` already parted: at (31843, 31369) against (31968, 31483),
    // its `ATTACKTOORDER`'s point (45288, 37992) against (45000, 37608),
    // offset (744, 360) against (456, 744), its path 19 nodes against 17
    // (run1431's 3801 agrees, offset (408, 216) both sides; parked 3751a).
    // No draw parts to 4130.
    // Item 3782, 164 → 125: `1/139`'s rows go — its block-0 re-place asks
    // the disc's cells (`docs/AI.md` §378.2).
    pin_eq!(w.firsts.len(), 125, "run1431b baseline");
    pin_eq!(w.rounds_read, (71, 73), "the rounds read");
    // Every round, both directions: 71 in ours against 73, 52 parting —
    // all three who=1's Manchu Infantry (TI 118, TY 68,
    // `MANCHUINFANTRY-DEFAULT-AGE0`) `1/26`, `1/27`, `1/28`, loosed from
    // their own square in ours: `1/26`'s `ammo[0]` on 4012 at `sz` 17
    // against 184, `1/28`'s pair on 4003 at 21 against 139. The piece has
    // no `launch::BAYS` row (parked 3751b).
    pin_eq!(w.rounds.len(), 52, "the rounds that part");
}

/// **The pair's closing state** (item 3751): run1431's closing block 4131,
/// the whole map's units, walked from run1430's start.
#[test]
fn chinese_modern_pair_run1431_s_closing_state() {
    let _pins = Pins::hold();
    let p = &CHINESE_MODERN;
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
        "Chinese Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4131, "the closing block");
    // Item 3751, whole on its first try: 180 units compared, counts
    // [1, 0, 0, 0, 0, 0, 0] — the Submarine `1/139` off by (70, -591), its
    // re-chosen formation offset (parked 3751a); nothing torn, unlinked
    // or extra. A floor, not a score.
    // Item 3782: every count zero — the Submarine's offset agrees once
    // `find_unit_with_radius` asks the disc's cells (`docs/AI.md` §378.2).
    pin_eq!(e.compared, 180, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
