//! **The Americans at a held Modern age, Great Sahara** (DECISIONS 90,
//! item 3509): the land map's held-age pair. The Aztec pair's lobby
//! (Great Sahara, `MAP_STYLE 7`, human Nubians (4), Difficulty 3, seed
//! 12345, `STARTING_RESOURCES 7`) with the nation varied to the
//! **Americans (20)** and the age held at Modern: `STARTING_TECHNOLOGY 6`,
//! `ENDING_TECHNOLOGY 6`. Captures: run1150 (a `DUMP_ALL` start) and
//! run1151 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §302.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const AMERICAN_MODERN: Pair = Pair {
    name: "GreatSaharaAmericanModern",
    map_style: 7,
    ai_tribe: 20,
    difficulty: 3,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1150-greatsahara-american-modern-start.txt",
    long: (
        "gamelog-run1151-greatsahara-american-modern-trace.txt",
        "rontrace-run1151.log",
    ),
    // The game ended itself on 4548 (closing block 4549).
    length: 4548,
    // Item 3509, first try: frame 1995 by the count and the sequence, ours
    // 6 draws against 7, index 1 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Guy::init_real+0x52`: theirs makes a
    // birth's draw ours does not (`docs/AI.md` section 302): the Airbase
    // `1/2022`'s free Bomber `1/45`, which ours' `MODERN_AGE` gate refused.
    // Item 3509: 1995 -> 4488, the gate asks the tree's Modern Age rather
    // than the original's `0x225` (the Information Age in the tree's ids):
    // ours 44 draws against 43, index 29 — ours `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`, theirs `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`.
    count: 4488,
    sequence: 4488,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AMERICAN_MODERN_PAIR_WORD: i64 =
    if AMERICAN_MODERN.count < AMERICAN_MODERN.sequence {
        AMERICAN_MODERN.count
    } else {
        AMERICAN_MODERN.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn american_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AMERICAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AMERICAN_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — `ENDING_TECHNOLOGY` beside `STARTING_TECHNOLOGY` — and the long
/// is scoreable.
#[test]
fn american_modern_pair_captures_say_their_lobby() {
    let p = &AMERICAN_MODERN;
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
        "the American pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn american_modern_pair_is_no_other_lobby() {
    let p = &AMERICAN_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// **The American Airbase pays a Bomber in the Modern age** (item 3509):
/// `Build::activate`'s `AIRBASE` arm, `AMERICANS_FREE_BOMBER` behind
/// `has_tech(MODERN_AGE)`, on run1150's own lobby — the AI American, the
/// age held at Modern. The original pays it on 1995 (run1151b's `1/45`,
/// a `BOMBER` inside the Airbase `1/2022`). The tree lays the techs out at
/// `543 + i` where the original's `TypeIndex` is `0x220 + i`, and asking
/// for the original's `0x225` asked for the Information Age, so no Bomber
/// came and the pair parted on 1995.
///
/// Made to fail on purpose first: with `0x225` back as the gate, no unit is
/// trained here.
#[test]
fn american_modern_pair_an_american_airbase_pays_a_bomber_in_the_modern_age() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some(path) = dump(AMERICAN_MODERN.start) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = replay_initial(&log).unwrap();
    let mut built = build_sim(&loaded, &init, sim::Tuning::RON);
    let sim = &mut built.sim;
    let modern = sim.tech_tree.ages[5].expect("a sixth age");
    assert_eq!(sim.tech_tree.types[modern].name, "Modern Age");
    assert_eq!(
        loaded.type_index(modern),
        0x225,
        "MODERN_AGE in the original"
    );
    let airbase = loaded.build_named("Airbase").expect("an Airbase type");
    let bomber = loaded.unit_named("Bomber").expect("a Bomber type");
    let bombers = |s: &sim::Sim, who: sim::Player| {
        s.units
            .iter()
            .filter(|u| u.alive() && u.owner == who && u.ty == Some(bomber))
            .count()
    };
    let before = bombers(sim, 1);
    let b = sim.init_build(1, airbase, sim::Pos::new(35040, 15456), false);
    sim.activate(b, false, true);
    assert_eq!(
        bombers(sim, 1),
        before + 1,
        "the American Airbase pays one Bomber"
    );
    // The human is Nubian: the arm is the Americans' alone.
    let h = bombers(sim, 0);
    let b0 = sim.init_build(0, airbase, sim::Pos::new(8000, 8000), false);
    sim.activate(b0, false, true);
    assert_eq!(bombers(sim, 0), h, "the human is not American");
}

/// run1151b: the word 1995's window, blocks 1990..2246 at the long's
/// detail — six blocks before the word's block 1996 and 250 after (item
/// 3509).
pub(crate) const RUN1151B: &str =
    "gamelog-run1151b-greatsahara-american-modern-window-1990-2246.txt";

/// The window: block 1990 through 2246; the word 1995's own block is 1996.
pub(crate) const WIDENING_AMERICAN_MODERN_FRAME_1995: (i64, i64) = (1990, 2246);

/// **The word 1995's window**: run1151b walked from run1150's start with
/// the recorder on (item 3509).
pub(crate) fn american_modern_frame_1995_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_MODERN.start],
        true,
        AMERICAN_MODERN.long,
        "run1151b",
        &[(RUN1151B, WIDENING_AMERICAN_MODERN_FRAME_1995.0)],
        WIDENING_AMERICAN_MODERN_FRAME_1995,
        1,
        &[1996],
        true,
    )
}

#[test]
fn american_modern_pair_run1151b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_modern_frame_1995_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3509, the first walk: 713 keys, the first not standing on the
    // word's block 1996 — who=1's `num_units[254]` ours 0, theirs 1, and
    // `1/45` (uid 77, a `BOMBER`, inside the Airbase `1/2022`) theirs'
    // alone: the Airbase completes on both sides and theirs pays the
    // American free Bomber. 713 -> 72 with the gate reading the tree's
    // Modern Age: block 1990's standing rows (both leaders' `SITE[i].reg`
    // and `aggression`, every unit's `form`, groups 64 and 66's `held` and
    // pool, `1/14`'s `g.angle[0]`), three `dest_angle` rows (`1/31` on
    // 2092, `1/20` on 2118, `1/40` on 2196), groups 65 and 67 ours alone
    // (2112, 2204), and the make list's `MAKE[2].val` on 2184 (ours 653062,
    // theirs 710507) before the Airbases `1/2022` and `1/2025` queue at
    // ours' dearer price on 2186 (`queue[0].cost[0]` ours 89, theirs 71;
    // `docs/AI.md` section 302).
    pin_eq!(w.firsts.len(), 72, "run1151b baseline");
}

/// run1151c: the word 4488's window, blocks 4482..4549 — six blocks before
/// the word's block 4489 and on to the game's closing block (item 3509).
pub(crate) const RUN1151C: &str =
    "gamelog-run1151c-greatsahara-american-modern-window-4482-4549.txt";

/// The window: block 4482 through the closing block 4549; the word 4488's
/// own block is 4489.
pub(crate) const WIDENING_AMERICAN_MODERN_FRAME_4488: (i64, i64) = (4482, 4549);

/// **The word 4488's window**: run1151c walked from run1150's start with
/// the recorder on (item 3509).
pub(crate) fn american_modern_frame_4488_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_MODERN.start],
        true,
        AMERICAN_MODERN.long,
        "run1151c",
        &[(RUN1151C, WIDENING_AMERICAN_MODERN_FRAME_4488.0)],
        WIDENING_AMERICAN_MODERN_FRAME_4488,
        1,
        &[4489],
        true,
    )
}

#[test]
fn american_modern_pair_run1151c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_modern_frame_4488_window() else {
        return;
    };
    pin_eq!(w.blocks, 67, "every captured block");
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
    // Item 3509, the first walk: 2419 keys. Block 4482 carries 188
    // standing rows — the window opens 2236 blocks past run1151b with only
    // the long's stride between — among them the Bazooka squad `1/21`,
    // `1/41`, `1/65` (TypeIndex 139, land) on guard orders that already
    // differ (`1/21`'s `guard_x`/`guard_y` ours (9192, 30936), theirs (8808,
    // 33096)). The first row after them is `1/41`'s path on 4487 (length
    // ours 17, theirs 18, `path[12].to` ours (9720, 31848), theirs (9576,
    // 31752)), then `1/21`'s `collide_o` on the word's block 4489 (ours
    // 65, theirs -1; `docs/AI.md` section 302).
    pin_eq!(w.firsts.len(), 2419, "run1151c baseline");
}

/// **The long's stride, walked back from the word 4488** (item 3517): run1151
/// dumps `UNITS=3,GUYS=2` every hundredth block, and run1151c's window opens
/// on 188 standing rows, so the state parted somewhere in 2247..4481 and only
/// the stride's blocks are on disk there.
pub(crate) const WIDENING_AMERICAN_MODERN_STRIDE: (i64, i64) = (2201, 4481);

pub(crate) fn american_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_MODERN.start],
        true,
        AMERICAN_MODERN.long,
        "run1151 stride",
        &[(AMERICAN_MODERN.long.0, WIDENING_AMERICAN_MODERN_STRIDE.0)],
        WIDENING_AMERICAN_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn american_modern_pair_the_long_s_stride_walks_back_from_4488() {
    let _pins = Pins::hold();
    let Some(w) = american_modern_stride_window() else {
        return;
    };
    eprintln!("stride blocks {}", w.blocks);
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    eprintln!("stride firsts {}", w.firsts.len());
}

/// run1163: the walk back from the word 4488, blocks 4396..4482 at
/// run1151c's detail (item 3517) — the long's stride has the Bazooka squad
/// agreeing on 4401 and run1151c opens with its guard orders apart.
pub(crate) const RUN1163: &str = "gamelog-run1163-greatsahara-american-modern-window-4396-4482.txt";

/// The window: block 4396 through 4482.
pub(crate) const WIDENING_AMERICAN_MODERN_GAP_1163: (i64, i64) = (4396, 4482);

pub(crate) fn american_modern_gap_1163_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AMERICAN_MODERN.start],
        true,
        AMERICAN_MODERN.long,
        "run1163",
        &[(RUN1163, WIDENING_AMERICAN_MODERN_GAP_1163.0)],
        WIDENING_AMERICAN_MODERN_GAP_1163,
        1,
        &[],
        true,
    )
}

#[test]
fn american_modern_pair_run1163_s_walk_back_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = american_modern_gap_1163_window() else {
        return;
    };
    eprintln!("run1163 blocks {}", w.blocks);
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    eprintln!("run1163 missing {:?}", w.missing);
    eprintln!("run1163 firsts {}", w.firsts.len());
}
