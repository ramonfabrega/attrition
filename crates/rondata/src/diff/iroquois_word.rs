//! **The battery's second word** (DECISIONS 72, item 3107): the first
//! battery word, `EastIndiesGermanTier3`, closed at its game's end on item
//! 3086, and its successor is the held-out battery's lowest lobby. Great
//! Lakes (`MAP_STYLE 14`), human Nubians (4) against **the Iroquois (18)**
//! at Difficulty 3, seed 12345 — `docs/AI.md` §243's third lobby, scored by
//! its first try at 0 of 12,658 and promoted out of the battery here, so it
//! is a scored lobby from this item (`battery::SCORED_LOBBIES`) and no
//! longer held out. Captures: run897 (a `DUMP_ALL` start, blocks 1, 36 and
//! 37) and run897b (the draw stream to the game's end, `cover=0`, its setup
//! and closing blocks only); the word's window is run897's own block 1
//! (`docs/AI.md` §246).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const IROQUOIS: Lobby = Lobby {
    name: "GreatLakesIroquoisTier3",
    map_style: 14,
    ai_tribe: 18,
    difficulty: 3,
    start: "gamelog-run897-greatlakes-iroquois-diff3-start.txt",
    long: (
        "gamelog-run897b-greatlakes-iroquois-diff3-24k-trace.txt",
        "rontrace-run897b.log",
    ),
    length: 12658,
    // Item 2975: the battery's first try, count and sequence both at 0
    // (ours 121 draws, theirs 122, index 30: ours `Unit::think_scout+0x436`,
    // theirs `Unit::think_scout+0x64c`). Item 3107 promoted it to the AI
    // word unmoved (`docs/AI.md` §246). **Item 3107 moved it 0 → 1120**
    // with the Iroquois Scout's forest walker (`Unit::init@00612100:341`,
    // `unit_masks2 |= 0x4080`; `UnitData::invalid_loc` passes forest for
    // `& 0x4000`): on frame 0 who=1's Scout `1/0` (TI 69) draws a second
    // ring-5 cell, (58, 16), a forest cell ours refused (`invalid_loc` 2),
    // and goes there — block 1's `orders_y` 12792 on both sides (ours 19704
    // before), the path six legs on both. At 1120 ours 36 draws, theirs 33,
    // index 0: ours `Unit::explore_goody+0x27c`, theirs
    // `Animal::think_bird+0x82`.
    count: 1120,
    sequence: 1120,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const IROQUOIS_WORD: i64 = if IROQUOIS.count < IROQUOIS.sequence {
    IROQUOIS.count
} else {
    IROQUOIS.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn iroquois_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(IROQUOIS.start, IROQUOIS.long) else {
        return;
    };
    eprintln!(
        "IROQUOIS {}: count {} sequence {} of {}; {}",
        IROQUOIS.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, IROQUOIS.difficulty, "the lobby read back");
    pin_eq!(w.last, IROQUOIS.length, "the game's last frame");
    pin_eq!(w.count, IROQUOIS.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        IROQUOIS.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn iroquois_word_is_no_other_lobby() {
    let p = &IROQUOIS;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
        let tribes: Vec<_> = init
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe))
            .collect();
        assert_eq!(tribes, [(0, 4), (1, i64::from(p.ai_tribe))], "{name}");
    }
}

/// run897's own blocks, 0 and 1: the start dump's whole records, which is
/// where frame 0's word lives (item 3107; the coverage pair's
/// `WIDENING_COVERAGE_START` is the pattern).
pub(crate) const WIDENING_IROQUOIS_START: (i64, i64) = (0, 1);

/// The word's widening: every dumped record on block 1 (the state frame 0
/// writes), the start dump itself as the capture.
pub(crate) fn iroquois_start_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run897",
        &[(IROQUOIS.start, 0)],
        WIDENING_IROQUOIS_START,
        1,
        &[1],
        true,
    )
}

#[test]
fn run897_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_start_window() else {
        return;
    };
    pin_eq!(
        w.blocks,
        1,
        "every captured block: the start dump's block 1"
    );
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(w.firsts.len(), 31, "initial run897 baseline");
    // Item 3107, on base 8d08a485: 45 keys part on block 1 (frame 0: ours
    // 121 draws, theirs 122, index 30 — ours `Unit::think_scout+0x436`,
    // theirs `Unit::think_scout+0x64c`). The control's 31 (both leaders'
    // `SITE[n].reg` 1/0, `aggression` 0/1, and the starting units' `form`
    // -1/9) stand on every lobby's block 1. The 14 the lobby adds are all
    // who=1's Scout `1/0` and its group 65: `myhits`, `hits_left` and
    // `hits:myhits` 60 in ours against 63; its explore move's `y` 19704
    // against 12792 (`orders_y`, the path's three legs against six, its
    // `angle` 1913782272 against 120324096, `facing` 0/1), and group 65's
    // `oy` 19680/12768 and `o_angle` with it (`docs/AI.md` §246).
    //
    // **Item 3107: 45 → 42 → 31.** The Iroquois' `IROQUOIS_EXTRA_HITS`
    // (`Unit::update_hits@0060e930`'s `has_tribe_bonus(0x12)` arm, a
    // Barracks unit armed or a Scout) makes the Scout 63 on both (42); its
    // forest walker sends it to (58, 16) on both — `orders_y` 12792,
    // `dest_angle` 120324096, the path's six legs, group 65's `oy` 12768
    // (31). What stands is the control's 31 alone. The word moved to 1120,
    // past the start dump's blocks (1, 36, 37).
}

/// **run897's block 1 fog plane is ours** (item 3107): `seen2`, every
/// half-cell, after frame 0. `Unit::think_scout`'s cell filter reads it
/// (`was_really_seen`) before `invalid_loc`, so the frame-0 parting — the
/// Scout `1/0`'s ring-5 cell (58, 16), drawn in theirs and refused in ours —
/// was the forest test's and not the fog's.
#[test]
fn run897_s_block_1_fog_plane_is_ours() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(start), Some(tr)) = (dump(IROQUOIS.start), trace(IROQUOIS.long.1)) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let start_text = crate::capture::read(start);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.tick();
    let block = start_log
        .frames()
        .into_iter()
        .find(|(n, _)| *n == 1)
        .map(|(_, b)| b)
        .expect("block 1");
    let world = block
        .kid("FULL DUMP")
        .unwrap_or(block)
        .kid("WORLD")
        .expect("block 1 has no WORLD record");
    let theirs = crate::gamelog::world_fog(&world.fields().to_vec());
    let w = &built.sim.world;
    let (fw, fh) = (w.fog_xs(), w.fog_ys());
    pin_eq!(theirs.len(), (fw * fh) as usize, "block 1's fog plane");
    let parted = (0..fh)
        .flat_map(|y| (0..fw).map(move |x| (x, y)))
        .filter(|&(x, y)| w.seen2(x, y).unwrap_or(0) != theirs[(y * fw + x) as usize])
        .count();
    pin_eq!(parted, 0, "half-cells parted on block 1");
}

/// run918: the word 1120's window, blocks 1114..1370 at the long's detail
/// (item 3117); run897 dumps blocks 1, 36 and 37 and run897b is `cover=0`.
pub(crate) const RUN918: &str = "gamelog-run918-greatlakes-iroquois-diff3-window-1114-1371.txt";

/// The window: block 1114 through 1370; the word 1120's own block is 1121.
pub(crate) const WIDENING_IROQUOIS_FRAME_1120: (i64, i64) = (1114, 1370);

pub(crate) fn iroquois_frame_1120_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run918",
        &[(RUN918, WIDENING_IROQUOIS_FRAME_1120.0)],
        WIDENING_IROQUOIS_FRAME_1120,
        1,
        &[1121],
        true,
    )
}

#[test]
fn run918_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_1120_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    pin_eq!(w.firsts.len(), 0, "initial run918 baseline");
}
