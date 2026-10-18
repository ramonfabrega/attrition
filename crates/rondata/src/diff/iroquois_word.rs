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
    // `Animal::think_bird+0x82`. **Item 3117 moved it 1120 → 1496** with
    // `astar_path`'s `iroquois` mode taken from the unit (`unit_masks2 &
    // 0x4000`, `docs/PATHFINDER.md` §4.1; a SEAM, false, before): run918b's
    // block 894, the Scout's ten-leg explore path to (39672, 22776), legs
    // 6..8 one cell east in ours ((43512, 22008) against (42744, 22008) for
    // leg 6), agrees whole, and with it the Scout's `pos` on every block to
    // 1370 (`docs/AI.md` §247). At 1496 ours 37 draws, theirs 36, index 30:
    // ours `Unit::do_non_flat_gather+0xcc3`, theirs `Farms::inc_time+0x1ae`.
    // **Item 3140 moved it 1496 → 1772** with `action_swarm_around`'s
    // builder mark (`Unit::add_build_order@005e5210`'s `unit_masks |=
    // 0x400`, which this crate's `swarm_around` never set): 1496's extra
    // draw was the woodcutter `1/2`'s chop reroll, `all_gathering` false
    // in ours because `1/7` was walking to the camp 1/2001; run921's
    // block 1378 has `1/7` taking the re-placed camp 1/2008's
    // `BUILDORDER` (uid 20) behind an `EXPLORETO` (40248, 23160) in the
    // original, a `GATHERORDER` on 2001 in ours, and agrees whole with
    // the mark (`docs/AI.md` §248). At 1772 ours 3 draws, theirs 4,
    // index 0: ours `Guy::set_anim+0x104b`, theirs
    // `Unit::do_non_flat_gather+0x54b`. **Item 3156 widened 1772 on
    // run922** (1620..2022): theirs' extra draw is who=1's `1/6` taking
    // the camp 1/2008's gather on frame 1770, which ours' commerce cap
    // refuses — Barter, queued by `defensive.bhs` step 15 on 1576 in the
    // original and unaffordable in ours (food 46 against the 54 it
    // costs), the food row standing from block 38 (`docs/AI.md` §249).
    count: 1772,
    sequence: 1772,
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
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496's pattern): the animals' clocks.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 37, "initial run918 baseline");
    // Item 3117, on base 808fe40c: the word 1120's window (ours 36 draws,
    // theirs 33, index 0: ours `Unit::explore_goody+0x27c` ×3, theirs the
    // same three on 1121 — who=1's Scout `1/0` entering its goody cell a
    // frame early). Block 1114 already held the Scout parted: `pos` ours
    // (41445, 25153) theirs (41483, 25113), `heading` −1767309312 against
    // −1757216768, `dest_angle` −1750335488 against −1749155840 on one
    // `orders_x`/`orders_y`, and its walk on to 1133 (`orders`, `path`,
    // `idle`, the next explore's `x`/`y` 38904/22776 against 41112/25752).
    //
    // **Item 3117: the word 1120 → 1496**, with `astar_path`'s `iroquois`
    // mode (`unit_masks2 & 0x4000`, `docs/PATHFINDER.md` §4.1) taken from
    // the unit — before, a SEAM, false. The Scout agrees on every block of
    // the window but one bit. What stands (37): the control's 32
    // (`SITE[n].reg`, `aggression`, `form`, groups' `held`, the pool), the
    // Scout's `blown` 1/0 (`unit_masks & 0x1000`: theirs' `0xc080` cloak bit
    // skips `Unit::work`'s stealth arm, parked 3114), and who=1's food from
    // 1114 — `income`/`resources` 800 against 896, `bucket` 31/45,
    // `leftover` 640/4320 — standing from run918b's block 38.
}

/// run918b: the gap 37..1114 at a stride of 4 (item 3117, Loop 3085's
/// `--stride`): blocks 38, 42, …, 1114, at run918's detail.
pub(crate) const RUN918B: &str = "gamelog-run918b-greatlakes-iroquois-diff3-stride4-38-1114.txt";

/// The window: block 38 through 1114, one in four.
pub(crate) const WIDENING_IROQUOIS_GAP_918B: (i64, i64) = (38, 1114);

pub(crate) fn iroquois_gap_918b_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run918b",
        &[(RUN918B, WIDENING_IROQUOIS_GAP_918B.0)],
        WIDENING_IROQUOIS_GAP_918B,
        1,
        &[1114],
        true,
    )
}

#[test]
fn run918b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_gap_918b_window() else {
        return;
    };
    pin_eq!(w.blocks, 270, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496's pattern): the animals' clocks.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 43, "initial run918b baseline");
    // Item 3117, on base 808fe40c: 60 keys part over the gap. The first
    // that is the word's: block 894, who=1's Scout `1/0` reaching
    // (45792, 18912) and taking `EXPLORETO` (39672, 22776) on both sides,
    // with a ten-leg path whose legs 6..8 are ours (43512, 22008),
    // (44280, 21240), (45048, 20472) against theirs (42744, 22008),
    // (43512, 21240), (44280, 20472): from cell (58, 25) to (55, 29) ours
    // steps south first and theirs diagonally first, and from 918 its `pos`
    // (45204, 19537) against (45202, 19535) — the parting that put ours a
    // frame ahead at 1120.
    //
    // **Item 3117: 60 → 43**, with `astar_path`'s `iroquois` mode from the
    // unit (`docs/PATHFINDER.md` §4.1; a SEAM, false, before): the path's
    // terrain term reads the signed `WData.solid` for a forest walker, and
    // the ten legs agree. What stands (43): the control's 37 from 38, the
    // Scout's `blown` 1/0 from 38 (parked 3114's cloak bit), and who=1's
    // food from block 38 — `income`/`resources` 640 against 672, `bucket`
    // 4/5, `leftover` 6560/352 — with `known_rares` 0 against 1: theirs
    // knows a rare on 38 and ours none (run897's block 37 holds no such row).
}

/// run921: the word 1496's window, blocks 1371..1626 at run918's detail
/// (item 3140) — run918's last block is 1370, so it starts there, and
/// it holds the frame the state parted on (block 1378) as well as the
/// word's own (1497).
pub(crate) const RUN921: &str = "gamelog-run921-greatlakes-iroquois-diff3-window-1371-1627.txt";

/// The window: block 1371 through 1626.
pub(crate) const WIDENING_IROQUOIS_FRAME_1496: (i64, i64) = (1371, 1626);

pub(crate) fn iroquois_frame_1496_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run921",
        &[(RUN921, WIDENING_IROQUOIS_FRAME_1496.0)],
        WIDENING_IROQUOIS_FRAME_1496,
        1,
        &[1378, 1497],
        true,
    )
}

#[test]
fn run921_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_1496_window() else {
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
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496's pattern): the animals' clocks.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 40, "initial run921 baseline");
    // Item 3140, on base ca8f3d8a: 448 keys part over the window. The
    // first that is the word's: block 1378, who=1's citizen `1/7`, its
    // build on the Woodcutter's Camp 1/2008 (uid 18, placed and closed on
    // 1176) killed on 1377 when the script re-placed the camp under the
    // same number on 1376 (uid 20) — on both sides — and then, idle,
    // `1/7` takes in the original the new site's `BUILDORDER` behind an
    // `EXPLORETO` (40248, 23160) (`order:kind` 3, `orders.len` 2,
    // `orders_x`/`orders_y` 40248/23160) and in ours a `GATHERORDER` on
    // the camp 1/2001 (kind 7, one order, 40220/22597). `think_peasant`'s
    // build arm reads `unit_masks & 0x400`, which ours' `swarm_around`
    // never set.
    //
    // **Item 3140: 448 → 40**, with the mark (`action_swarm_around@
    // 0070fbe0` queues through `add_build_order@005e5210`, `orl $0x400`
    // at `5e5247`): `1/7` agrees on every block, and with it the camp
    // 2001's chain and `1/2`'s chop reroll on 1496. What stands (40): the
    // control's 30 from 1371 (`SITE[n].reg`, `aggression`, `form`,
    // groups 65/66 `held`, pool 66), who=1's food from 1371 —
    // `bucket` 89 against 107, `income`/`resources` 800 against 928,
    // `leftover` 4640 against 5600, standing from run918b's block 38 —
    // the Scout's `blown` 1/0 from 1537 (parked 3114's cloak bit), and
    // who=1 on 1577: `script_step` 15 against 18, `tech_frame` and
    // `tech_cat_frame[2]` 1 against 1576, the Library 1/2005's `queued`
    // 0/1 and timber `bucket` 133 against 79 — the original queues a
    // tech on 1576 that ours does not.
}

/// run922: the word 1772's window, blocks 1620..2022 at run918's detail
/// (item 3156) — from six blocks before run921's last (1626, Loop 3155),
/// so the two windows overlap, through 250 past the word's own block 1773.
pub(crate) const RUN922: &str = "gamelog-run922-greatlakes-iroquois-diff3-window-1620-2023.txt";

/// The window: block 1620 through 2022.
pub(crate) const WIDENING_IROQUOIS_FRAME_1772: (i64, i64) = (1620, 2022);

pub(crate) fn iroquois_frame_1772_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run922",
        &[(RUN922, WIDENING_IROQUOIS_FRAME_1772.0)],
        WIDENING_IROQUOIS_FRAME_1772,
        1,
        &[1773],
        true,
    )
}

#[test]
fn run922_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_1772_window() else {
        return;
    };
    pin_eq!(w.blocks, 403, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496's pattern): the animals' clocks.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 408, "initial run922 baseline");
    // Item 3156, on base b0758479: 408 keys part over the window. The
    // word's own record is block 1771 (frame 1770): who=1's citizen `1/6`
    // (TI 50, the builder of the Woodcutter's Camp 1/2008, idle beside it
    // at (40488, 23160)) reaches `idle` 12, `think_peasant`'s `(idle − 2)
    // % 5` gate, and in the original takes a `GATHERORDER` on 2008 (uid
    // 20, `build_type` 418, `orders.len` 1 against 0; the camp's
    // `gather_down` 6 against 7); in ours `find_gather_spot` finds
    // nothing. Theirs' `+0x54b` on 1772 is that order's tile pick (block
    // 1773's `tx`/`ty` 206/123), and the camp's tile list and `1/6`'s walk
    // part from there (1773..1777). Walked back: the spot is scored on the
    // headroom under the commerce cap, and who=1's timber `income` stands
    // at 1120 on both sides, which is ours' cap and under theirs' —
    // `resource_cap` 1120 against 1600 from block 1758, `epoch[2]` 0
    // against 1 from 1757: **Barter** completing in the original. It was
    // queued on 1576 (`tech_frame` 1 against 1576, the Library 1/2005's
    // `queued` 0/1, `script_step` 15 against 18, standing from 1620):
    // `defensive.bhs` step 15's `research_tech_with_cost(who, "Barter")`,
    // whose `6f/6t` the original pays (food `bucket` 67 → 13, timber 133
    // → 79) and ours cannot — food 46. The food row stands from run918b's
    // block 38 (parked 3138; `bucket` 51/20, `income`/`resources` 960
    // against 1120, `leftover` 6240/1440 on 1620). The rest: the control's
    // 30 from 1620, the Scout's `blown` 1/0 (parked 3114), and what Barter
    // moves downstream (`gather_stamp`, `known_rares` on 1760, the
    // gatherers from 1776, the City Center's queue from 1777).
}

/// run923: the lobby's first forty blocks dense, at run918's detail (item
/// 3170) — the gap run897 leaves between its start dump's block 1 and its
/// blocks 36 and 37, where the food row parts.
pub(crate) const RUN923: &str = "gamelog-run923-greatlakes-iroquois-diff3-window-1-41.txt";

/// The window: block 1 through 40.
pub(crate) const WIDENING_IROQUOIS_FOOD_923: (i64, i64) = (1, 40);

pub(crate) fn iroquois_food_923_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run923",
        &[(RUN923, WIDENING_IROQUOIS_FOOD_923.0)],
        WIDENING_IROQUOIS_FOOD_923,
        1,
        &[1773],
        true,
    )
}

#[test]
fn run923_s_food_row_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_food_923_window() else {
        return;
    };
    pin_eq!(w.blocks, 40, "every captured block");
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
    pin_eq!(w.firsts.len(), 0, "initial run923 baseline");
}

/// PROBE 3170 (reverted whole): who=1's holdings at the frame-7 recompute.
#[test]
fn probe_3170_food() {
    if std::env::var("RON_PROBE_3170").is_err() {
        return;
    }
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
    for _ in 0..8 {
        built.tick();
    }
    eprintln!("{:#?}", built.sim.holdings[1]);
    for u in 0..built.sim.units.len() {
        let x = &built.sim.units[u];
        if x.owner == 1 {
            eprintln!("unit {u} ti {} pos {:?} order {:?}", x.type_index, x.pos, built.sim.current_order(u).map(|o| o.body));
        }
    }
}
