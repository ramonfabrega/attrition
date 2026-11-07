//! **The Egyptians at a held Modern age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, 98, 99, item 3724): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Egyptians (7)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(14, 7, 4)`, is new: the Egyptians hold this map at 5
//! (`egyptian_information_pair`), East Indies at 4
//! (`egyptian_industrial_pair`) and 5 (`egyptian_classical_pair`), and
//! Great Sahara at 4 (`egyptian_word`) and 5 (`egyptian_gunpowder_pair`);
//! the battery's one Egyptian game is the held-out
//! `EastIndiesEgyptianTier3`, `(18, 7, 3)`. Captures: run1406 (a
//! `DUMP_ALL` start) and run1407 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §371.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const EGYPTIAN_MODERN: Pair = Pair {
    name: "GreatLakesEgyptianModern",
    map_style: 14,
    ai_tribe: 7,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1406-greatlakes-egyptian-modern-start.txt",
    long: (
        "gamelog-run1407-greatlakes-egyptian-modern-trace.txt",
        "rontrace-run1407.log",
    ),
    // The game ended itself on 3878 (closing block 3879).
    length: 3878,
    // Item 3724, first try: count 3451, sequence 3451 of 3878 — on 3451
    // ours 20 draws against theirs' 21, index 17 ours'
    // `Farms::inc_time+0x1ae` against theirs' `Unit::close+0xcb6`
    // (`docs/AI.md` §371). Item 3747: whole, 3878 of 3878 — 3451 → 3748
    // on piece 195's `pivot::SAME_AS` row (the Churchills' rounds), 3748 →
    // 3853 on piece 52's `launch::BAYS` (the Infantry's), 3853 → 3878 on
    // piece 89's (the Bazookas'), `docs/AI.md` §371.1.
    count: 3878,
    sequence: 3878,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const EGYPTIAN_MODERN_PAIR_WORD: i64 =
    if EGYPTIAN_MODERN.count < EGYPTIAN_MODERN.sequence {
        EGYPTIAN_MODERN.count
    } else {
        EGYPTIAN_MODERN.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn egyptian_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN_MODERN {}: count {} sequence {} of {}; {}",
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
fn egyptian_modern_pair_captures_say_their_lobby() {
    let p = &EGYPTIAN_MODERN;
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
        "the Egyptian Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3724; the battery's identity
/// test refuses it), once, and none of the battery's — its one Egyptian
/// game, the held-out `EastIndiesEgyptianTier3` `(18, 7, 3)`, is another
/// map — and no other Egyptian lobby's: `egyptian_information_pair`'s
/// `(14, 7, 5)`, this map at another difficulty,
/// `egyptian_industrial_pair`'s `(18, 7, 4)`, `egyptian_classical_pair`'s
/// `(18, 7, 5)`, `egyptian_word`'s `(7, 7, 4)` and
/// `egyptian_gunpowder_pair`'s `(7, 7, 5)`, other maps (item 3724).
#[test]
fn egyptian_modern_pair_is_no_other_lobby() {
    let p = &EGYPTIAN_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    for l in battery::BATTERY.iter().filter(|l| l.ai_tribe == p.ai_tribe) {
        assert_ne!(
            (l.map_style, l.ai_tribe, l.difficulty),
            triple,
            "{} is this lobby",
            l.name
        );
        assert_ne!(
            l.map_style, p.map_style,
            "{} fields the Egyptians here",
            l.name
        );
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Egyptian lobby on this map at this difficulty"
        );
    }
    let n = &super::egyptian_information_pair::EGYPTIAN_INFORMATION;
    let c = &super::egyptian_classical_pair::EGYPTIAN_CLASSICAL;
    let i = &super::egyptian_industrial_pair::EGYPTIAN_INDUSTRIAL;
    let g = &super::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER;
    let w = &super::egyptian_word::EGYPTIAN;
    for (name, l) in [
        (n.name, (n.map_style, n.ai_tribe, n.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Egyptians");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3724, from
/// 3451): only run1407's stride blocks are on disk before the word's
/// window, 101 through 3001.
pub(crate) const WIDENING_EGYPTIAN_MODERN_STRIDE: (i64, i64) = (101, 3301);

pub(crate) fn egyptian_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_MODERN.start],
        true,
        EGYPTIAN_MODERN.long,
        "run1407 stride",
        &[(EGYPTIAN_MODERN.long.0, WIDENING_EGYPTIAN_MODERN_STRIDE.0)],
        WIDENING_EGYPTIAN_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn egyptian_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every stride block in the window");
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
    // Item 3724: 97 keys over 33 blocks, the standing families alone —
    // the buildings' `build:extra` (58, which the stride does not print),
    // `form` on 27 units, and who=1's groups with their pools (12). No
    // unit row parts.
    pin_eq!(w.firsts.len(), 97, "the stride's baseline");
}

/// run1407b: the word 3451's window, blocks 3395..3702 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3401 and 250 past the word's own block 3452 (item 3724).
/// run1407 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1407B: &str =
    "gamelog-run1407b-greatlakes-egyptian-modern-window-3395-3702.txt";

/// The window: block 3395 through 3702.
pub(crate) const WIDENING_EGYPTIAN_MODERN_FRAME_3451: (i64, i64) = (3395, 3702);

/// **The word 3451's window**: run1407b walked from run1406's start with
/// the recorder on and every round compared (item 3724).
pub(crate) fn egyptian_modern_frame_3451_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_MODERN.start],
        true,
        EGYPTIAN_MODERN.long,
        "run1407b",
        &[(RUN1407B, WIDENING_EGYPTIAN_MODERN_FRAME_3451.0)],
        WIDENING_EGYPTIAN_MODERN_FRAME_3451,
        1,
        &[3452],
        true,
    )
}

#[test]
fn egyptian_modern_pair_run1407b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_modern_frame_3451_window() else {
        return;
    };
    pin_eq!(w.blocks, 307, "every captured block");
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
    // Item 3724, the first walk: **1287** keys. Past the standing
    // families, the first unit parting is **the Special Forces `0/0`**
    // (TI 75, land, who=0's) on 3442: `hits:damage` ours 52 against theirs
    // 93, `hits_left` 58 against 17 — struck harder and earlier in theirs
    // by group 64's Churchills (TI 245, `CHURCHILLDES`, piece 195, which
    // neither `launch` nor `pivot` holds). Theirs' `0/0` dies on 3451
    // (`Unit::close+0xcb6` under `Object::die`, the word's index 17); on
    // block 3452 ours alone holds it, and who=0's `num_units[25]` reads 1
    // against 0 (`docs/AI.md` §371).
    // Item 3747: 1287 → **46**, the standing families alone (`form` on 23
    // units, who=0's ten `SITE[i].reg` and `aggression[1]`, who=1's two
    // `SITE[i].reg` and five groups with their pools) once piece 195 reads
    // piece 194's rows (`pivot::SAME_AS`, `docs/AI.md` §371.1).
    pin_eq!(w.firsts.len(), 46, "run1407b baseline");
    // Every round, both directions: 11 in ours against 18, 27 rows
    // parting, all the Churchills' (`1/27`, `1/36`, `1/37`, `1/47`) from
    // 3438 — `1/47`'s `ammo[0]` leaves the figure's own square (3720,
    // 33960, z 582) in ours and flies six frames, (3698, 33775, z 728) in
    // theirs and flies five: the launch point.
    // Item 3747: (11, 18) with 27 parting → (11, 11) with none: `1/47`'s
    // round of 3438 leaves (3698, 33775, z 728) both sides, the killer.
    pin_eq!(w.rounds_read, (11, 11), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1426: the word 3748's window, blocks 3695..3878 at run1407b's
/// detail — six blocks before the stride's last agreeing block 3701, to
/// the game's last frame (item 3747). run1407 prints units only on 3701
/// and 3801, no `AMMO`, and its 3701 parts no unit row.
pub(crate) const RUN1426: &str = "gamelog-run1426-greatlakes-egyptian-modern-window-3695-3878.txt";

/// The window: block 3695 through the closing block 3879.
pub(crate) const WIDENING_EGYPTIAN_MODERN_FRAME_3748: (i64, i64) = (3695, 3879);

/// **The word 3748's window**: run1426 walked from run1406's start with
/// the recorder on and every round compared (item 3747).
pub(crate) fn egyptian_modern_frame_3748_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_MODERN.start],
        true,
        EGYPTIAN_MODERN.long,
        "run1426",
        &[(RUN1426, WIDENING_EGYPTIAN_MODERN_FRAME_3748.0)],
        WIDENING_EGYPTIAN_MODERN_FRAME_3748,
        1,
        &[3749],
        true,
    )
}

#[test]
fn egyptian_modern_pair_run1426_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_modern_frame_3748_window() else {
        return;
    };
    pin_eq!(w.blocks, 183, "every captured block");
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
    // Item 3747, with pieces 195, 52 and 89 built: **51** keys, the
    // standing families alone — `form` on 22 units, who=0's ten
    // `SITE[i].reg` and `aggression[1]`, who=1's two `SITE[i].reg` and six
    // groups with their pools, `mirror` on four craft minted in the window
    // (3747a). The first walk, before piece 52's and 89's bays: 1123 keys,
    // rounds (222, 191) with 216 parting; the first unit parting the
    // Infantry `1/19`'s round of 3723 leaving (6456, 31752, z 221) against
    // (6417, 31701, z 341), and the Citizen `0/12` struck harder from 3739
    // and dying on 3748 in theirs alone.
    pin_eq!(w.firsts.len(), 51, "run1426 baseline");
    pin_eq!(w.rounds_read, (222, 222), "the rounds read");
    // 16 rows part, every one a round's `whom`/`ox` read after its target
    // died (`-1` in ours against the dead unit's number in theirs, 3749,
    // 3848, 3875); none reaches a draw (3747b).
    pin_eq!(w.rounds.len(), 16, "the rounds that part");
}

/// **The pair's closing state** (item 3724): run1407's closing block 3879,
/// the whole map's units, walked from run1406's start.
#[test]
fn egyptian_modern_pair_run1407_s_closing_state() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_MODERN;
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
        "Egyptian Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3879, "the closing block");
    // Item 3724, first try: 427 frames past the word 3451 the game has
    // parted — 130 units compared, counts [68, 0, 2, 0, 6, 0, 7], nothing
    // torn or unlinked, ours' `0/7` and `0/10` extra. A floor, not a score.
    pin_eq!(e.compared, 130, "units compared");
    // Item 3747: whole — 130 units, counts [0, 0, 0, 0, 0, 0, 0], nothing
    // torn, unlinked or extra, no closing field (from [68, 0, 2, 0, 6, 0,
    // 7] with ours' `0/7` and `0/10` extra).
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
