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
    // **Item 3170 moved it 1772 → 5144** with run923, the lobby's blocks
    // 1..40 dense: the food row parts on block 8, the first `calc_gather`
    // recompute — ours 640, theirs 672, the Iroquois woodcutter's
    // `IROQUOIS_FOOD × 16` (`BuildTypeData::calc_gather@00639e40`'s
    // `has_tribe_bonus(0x12)` food arm, not carried) — and `known_rares`
    // 0/1 beside it, a setup border fix ours left pending into frame 0's
    // `check_borders`. With both, Barter is paid on 1576 and 1/6's gather
    // on 1770 agrees (`docs/AI.md` §250). At 5144 ours 36 draws, theirs 37,
    // index 0: ours `Animal::think_bird+0x82`, theirs
    // `Unit::think_spellcaster+0x413`. **Item 3186 moved it 5144 → 5576**
    // with `Army::use_scouts@006f49a0` (and `use_spies`), the army's
    // spellcaster turn this crate left a seam: on 5144 who=1's army 1
    // holds `1/22` (TY 29 `takohs`, TI 79, land, the Scout's lineage), so
    // `Army::process`'s `count(COUNT_NON_DECOY_TYPE, 0x45)` is non-zero and
    // its special arm throws the coin at `+0x413`. run926 widens 5145: the
    // coin writes nothing, so the block is quiet; without the turn the
    // shifted stream first parts on 5156, `1/4`'s move order (41016,
    // 17016) against (40824, 17592) (`docs/AI.md` §251). At 5576 ours 35 draws, theirs 37, index 30:
    // ours `Guy::set_anim+0x104b`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`. **Item 3202 widened 5576** on run928
    // (5570..5826, whose first block already parts) and run930 (5556..5578
    // at `GUYS=4`): the state first parts on block 5564, who=1's Slinger
    // squad `1/25`..`1/27` taking its `ATTACKTO` — the followers' offsets
    // (0, ±144) off the captain in ours against (−48, ±96), and the
    // figures' speed 28 against 35 — and theirs' extra draw is `1/26`'s
    // figure, stopped on 5574, rolling its idle on 5576 (`docs/AI.md`
    // §252). **Item 3211 moved it 5576 → 5843** with
    // `UnitData::is_modern_infantry@00607b40`'s tribe arm: an Iroquois
    // player (`has_tribe_bonus(0x12)`) asks the type's `0x100` alone, so
    // the Slingers (TI 82) take `do_move`'s × 5 / 4 — 28 → 35 — and
    // `Form::compute_dests`' follower scatter, `1/26` (408, 552) and
    // `1/27` (408, 360) on block 5564 on both sides. At 5843 ours 2
    // draws, theirs 3, index 0: ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` — the second squad's `1/28` (TI 166
    // `IRQSPEAR`) stopping on its collision with `1/9` — widened on run932
    // over 5826..6094. **The same item moved it 5843 → 6982** with
    // `UnitData::speed`'s Iroquois spear arm (`IROQUOIS_SPEAR_SPEED`, +1
    // on allied ground for the Spearmen line): that squad, born on 5809,
    // stepped one point short from 5810. At 6982 ours 10 draws, theirs 15,
    // index 5: ours `Guy::set_anim+0x97a < Animal::do_idle+0x19`, theirs
    // `Leader::produce_building+0x1805`; past run932's last block, its
    // widening owed (`docs/AI.md` §253). **Item 3231 widened it on run934
    // and moved it 6982 → 7033 → 7982** (`docs/AI.md` §254): block 6982's
    // `MAKE[3]` was ours `t` 171 (val 62976) against theirs 438, the
    // Senate (val 80000, city 1) — ours valued the Senate at 80000 too and
    // `check_income` zeroed it, because `get_cost`'s Iroquois arm (the
    // first Senate is free) was not carried. With it the word was 7033
    // (theirs 208 draws, `PathFinder::calc_road_cost`, the Senate's
    // capital move on 7025): block 6983's `1/2023` `construct_hits` ours 1
    // against theirs 1200, `construct_time`'s twin arm (the first Senate
    // is instant). With both, 7982: ours 12 draws, theirs 14, index 5,
    // ours `Animal::do_idle+0x83`, theirs `Leader::make_stuff+0x63d`;
    // past run934's last block (7232), its widening owed. **Item 3242
    // widened it on run937 and moved it 7982 → 8415** (`docs/AI.md` §255):
    // block 7980's `MAKE[7].t` was ours 167 (`IRQMOSPEAR`) against theirs
    // 83 (`JAVELINEERS`) — ours offered the Javelineers' upgrade at a
    // quarter of theirs' value (62976 against 251904, `aff` 0x40), because
    // ours priced its research 80/80 against a timber bucket of 74 and
    // theirs 68/68: `get_cost`'s Despotism arm on a Barracks unit
    // (`DESPOTISM_MILITARY_CHEAPER`, 15%) was carried for library techs
    // only. With it, 8415: ours 8 draws, theirs 14, index 3, ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::init_real+0x52`; past run937's last block (8232), its
    // widening owed. **Item 3248 widened it on run940 and moved it 8415 →
    // 10695** (`docs/AI.md` §256): theirs' Barracks `1/2016` finished its
    // Javelineers on 8415 at a clock of 16200, ours' target 18750 — who=1
    // gained Cotton that frame (block 8416's rare mask `080000` →
    // `081000`, the Merchant `1/24`'s cast), and `train_time`'s Cotton arm
    // on a Barracks, Stable or Dock unit (`COTTON_CREATION_SPEED`, 25%:
    // 15000) was not carried. With it, 10695: ours 12 draws, theirs 11,
    // index 10, ours `Guy::set_anim+0x97a < Guy::inc_time+0x1ed`, theirs
    // `Farms::inc_time+0x1ae`; past run940's last block (8665), its
    // widening owed. **Item 3255 widened it on run943 and moved it 10695
    // → 10852** (`docs/AI.md` §257): the Scout `0/0` was struck a frame
    // late (10692 against 10691) by the Archers `1/32`'s arrow, which
    // ours loosed from the figure's own square; with the bays of pieces
    // 1529 and 1441 and the Horse Archers' (1545) pivot releases, 10852:
    // ours 25 draws, theirs 30, index 3, ours `Unit::close+0xcb6`, theirs
    // `605fe5`; run943's state first parts on 10843 (`1/32`'s
    // `dest_angle`, ours −786038784 against theirs −792788992).
    // **Item 3279 moved it 10852 → 10917** (`docs/AI.md` §259): the four
    // draws are The Despot's `Unit::plunder+0x4e5` on the Citizen `0/3`'s
    // kill by `1/58` (food, timber, wealth, metal; oil not yet available),
    // and who=1's metal on block 10853 is 59 on both sides (ours 44 before).
    // 10917: ours 285 draws, theirs 296, index 272, ours `Guy::set_anim
    // +0x97a < Guy::inc_time+0x271`, theirs `PathFinder::calc_road_cost+0x46`.
    // **Item 3287 moved it 10917 → 11585** (`docs/AI.md` §260): `1/2027`'s
    // road replan priced node 5, (45216, 29472), at 139 against theirs' 27
    // — ours' caravan diagonal (235, 153) had been swept on 10620, because
    // `set_diags`' corner tail (C4 references, `docs/ROADS.md` §12) was not
    // carried; with it, node 5 is 27 and the search 283 nodes on both sides,
    // and run946's packet's twelve elements agree on 10842. 11585: ours 16
    // draws, theirs 15, index 4, ours `Leader::make_stuff+0x221`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`; past run943's last block
    // (10945), its widening owed. **Item 3297 widened it on run952 and
    // run952b and moved it 11585 → 12156** (`docs/AI.md` §261): ours'
    // second `+0x221` was a second Farm offer in the make list, behind
    // who=1's food 69 against 89 on block 11259 — `0/2002` razed beside
    // The Despot `1/46`, whose cut (`Build::plunder`'s `find_hero`) ours
    // never paid; with it, 89 on both sides. The Despot's Ambush now holds
    // its craft (`mana_burn` 1000 through run943, 392 on 11579 on both).
    // 12156: ours 29 draws, theirs 35, index 1, ours `Unit::fight+0x824`,
    // theirs `Guy::set_anim+0x97a < Unit::fight+0x169`; past run952's last
    // block (11835), its widening owed. **Item 3306 widened it on run954
    // and run954b and moved it 12156 → 12658, the game's last frame**
    // (`docs/AI.md` §262): block 12156's `order:target` on the escorts
    // `1/25`, `1/48`, `1/51` and The Despot's decoys `1/77`, `1/78`,
    // `1/81`, `1/84`, `1/85` was ours `0/2000` (the human's city) against
    // theirs `0/5` (a Citizen gathering at 2004) — decoy `1/88`'s
    // `think_attack` hands `0/5` to its group through
    // `Group::target_opportunity`, and ours refused it: a held building
    // yields without the range tests (`target_opportunity@005fffc0:202`–
    // `209`), and a yield skips the flee arm and the on-duty gate
    // (`LAB_006004e7` → `LAB_00600877`). The close is compared whole on
    // run897b's own closing block 12659 by `run897b_iroquois_closing_state`.
    count: 12658,
    sequence: 12658,
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
    pin_eq!(w.firsts.len(), 33, "initial run918 baseline");
    // Item 3170: 37 → 33. who=1's food row (`income`, `resources`,
    // `bucket`, `leftover`) leaves with `IROQUOIS_FOOD`, the woodcutter's
    // two food (`docs/AI.md` §250). What stands is the control's set and the
    // Scout's `blown`.
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
    pin_eq!(w.firsts.len(), 38, "initial run918b baseline");
    // Item 3170: 43 → 38. who=1's food row and `known_rares` leave
    // (`IROQUOIS_FOOD`; setup's border fixes no longer zero frame 0's
    // census count, `docs/AI.md` §250).
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
    pin_eq!(w.firsts.len(), 32, "initial run921 baseline");
    // Item 3170: 40 → 32. The food row, and with it Barter's queue on
    // 1576 (`tech_frame`, the Library's `queued`, `script_step`): ours now
    // pays its 54 food (`docs/AI.md` §250).
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
    pin_eq!(w.firsts.len(), 32, "initial run922 baseline");
    // **Item 3170: 408 → 32**, the word's own frame closed. The value diff
    // on block 1771 (frame 1770): who=1's `1/6` takes the `GATHERORDER` on
    // the camp 1/2008 on both sides (`orders.len` 1 and 1, the camp's
    // `gather_down` 6 and 6), because Barter completes on 1757 on both
    // (`epoch[2]` 1 and 1, `resource_cap` 1600 and 1600 from 1758). It was
    // the food row: from block 8 ours paid 640 food against 672, the Iroquois
    // woodcutter's `IROQUOIS_FOOD × 16` (`BuildTypeData::calc_gather@00639e40`'s
    // `has_tribe_bonus(0x12)` food arm), and on 1576 ours held 46 food
    // against Barter's 54. What stands: the control's 30, the groups'
    // `held`, the pools, the Scout's `blown` (parked 3114).
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
    // Item 3170, on base 12c51ebe: 41 keys over blocks 1..40 before. Two
    // rows part past the control's set, both on block 8 — the first
    // `calc_gather` recompute (`gather_stamp` 0 → 7): who=1's food `income`
    // 640 against 672 (`resources`, `leftover` with it; `bucket` 3/4 from
    // 27) and `known_rares` 0 against 1. The first is `IROQUOIS_FOOD`,
    // which this crate did not carry; the second is frame 0's census count
    // of the Silk at (40320, 14208), which ours' `check_borders` zeroed on
    // frame 0 for a fix `build_sim`'s stand-up `activate`s raised — the
    // original's setup ends in `World::compute_all_territory`, so no pass
    // is pending (`docs/AI.md` §250). Both closed: 41 → 36, the control's
    // set, the groups' `held`, the pool, and the Scout's `blown`.
    pin_eq!(w.firsts.len(), 36, "initial run923 baseline");
}

/// run926: the word 5144's window, blocks 5138..5394 at run918's detail
/// (item 3186) — six blocks before the word's own block 5145, 250 past it;
/// run922 ends at 2022 and run897b is `cover=0`.
pub(crate) const RUN926: &str = "gamelog-run926-greatlakes-iroquois-diff3-window-5138-5395.txt";

/// The window: block 5138 through 5394.
pub(crate) const WIDENING_IROQUOIS_FRAME_5144: (i64, i64) = (5138, 5394);

pub(crate) fn iroquois_frame_5144_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run926",
        &[(RUN926, WIDENING_IROQUOIS_FRAME_5144.0)],
        WIDENING_IROQUOIS_FRAME_5144,
        1,
        &[5145],
        true,
    )
}

#[test]
fn run926_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_5144_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3186, on base 954b4359 with `Army::use_scouts` built: 32 keys,
    // the standing set alone — both leaders' `SITE[n].reg`, `aggression`,
    // `form`, the groups' `held`, the pools, `1/22`'s `blown` (parked
    // 3114). The word's block 5145 parts nothing on either side: theirs'
    // extra draw on 5144 is `1/22`'s special coin (`Unit::think_spellcaster
    // +0x413` under `Army::use_scouts`), which writes no state. With the
    // scouts' turn left out (a probe, reverted whole) the window parts 425
    // keys, the first on block 5156: `1/4`'s move order `x`/`y` 41016/17016
    // against 40824/17592 (`off_x`/`off_y` 312/120 against 120/696) — the
    // stream one draw behind. The word moved 5144 → 5576, past the window.
    pin_eq!(w.firsts.len(), 32, "initial run926 baseline");
}

/// run928: the word 5576's window, blocks 5570..5826 at run918's detail
/// (item 3202) — six blocks before the word's own block 5577, 250 past it;
/// run926 ends at 5394 and run897b is `cover=0`.
pub(crate) const RUN928: &str = "gamelog-run928-greatlakes-iroquois-diff3-window-5570-5827.txt";

/// The window: block 5570 through 5826.
pub(crate) const WIDENING_IROQUOIS_FRAME_5576: (i64, i64) = (5570, 5826);

pub(crate) fn iroquois_frame_5576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run928",
        &[(RUN928, WIDENING_IROQUOIS_FRAME_5576.0)],
        WIDENING_IROQUOIS_FRAME_5576,
        1,
        &[5577],
        true,
    )
}

#[test]
fn run928_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_5576_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3202, on base 128293bb: 622 keys. The standing set (both
    // leaders' `SITE[n].reg`, `aggression`, `form`, the groups' `held`,
    // the pools) and, from the window's first block 5570, who=1's Slinger
    // squad `1/25`..`1/27` (TI 82, born on 5563 at the Barracks, an
    // `ATTACKTO` to (36552, 21960) in group 64): `1/26`'s `off_x`/`off_y`
    // ours 456/600 against theirs 408/552, `1/27`'s 456/312 against
    // 408/360 — the followers one `guy_spacing` 144 off the captain in
    // ours, (−48, ±96) in theirs — and every squad member's `pos` one
    // walking frame behind. The word's own block 5577 parts nothing new:
    // theirs' extra index-30 draw is an idle roll (`Guy::set_anim+0x97a`)
    // whose guy the capture's `GUYS=2` cannot name (`docs/AI.md` §252).
    // Item 3211, on base 9e0fa5ee: 622 → 52 with the Iroquois arm of
    // `is_modern_infantry`, the squad quiet; then the second squad
    // `1/28`..`1/30` (TI 166, born on 5809) parted one point on 5810
    // (`1/29` ours (45117, 22713) theirs (45116, 22712)), and 52 → 35 with
    // `UnitData::speed`'s spear arm. Past the standing set what remains
    // first parts on 5627 (`1/22`'s `blown`, ours 1 theirs 0) and 5673
    // (`1/23`'s figure 1, ours (40682, 14692) theirs (40512, 14592)),
    // neither a draw (`docs/AI.md` §253).
    // Item 3369: 35 -> 33, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 33, "initial run928 baseline");
}

/// run932: the word 5843's window, blocks 5826..6094 at run918's detail
/// with `GUYS=4` (item 3211) — from run928's last block, 250 past the
/// word's own block 5844, with the figure clocks the word's
/// `Guy::set_anim+0x97a` rolls need.
pub(crate) const RUN932: &str =
    "gamelog-run932-greatlakes-iroquois-diff3-window-5826-6095-guys4.txt";

/// The window: block 5826 through 6094.
pub(crate) const WIDENING_IROQUOIS_FRAME_5843: (i64, i64) = (5826, 6094);

pub(crate) fn iroquois_frame_5843_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run932",
        &[(RUN932, WIDENING_IROQUOIS_FRAME_5843.0)],
        WIDENING_IROQUOIS_FRAME_5843,
        1,
        &[5844],
        true,
    )
}

#[test]
fn run932_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_5843_window() else {
        return;
    };
    pin_eq!(w.blocks, 269, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3211, on base 9e0fa5ee, with `is_modern_infantry`'s Iroquois
    // arm: 734 keys, the word 5843's block 5844 holding `1/28` (TI 166)
    // colliding with `1/9` in theirs (`collide_o` 9, `stopped` 1 on 5845)
    // after the squad's start parted one point on 5810. With the spear
    // arm, 41: the standing set, `1/22`'s `blown` (5883), the camp pool
    // (5896) and `SITE[3].reg` (5976). The word moved past the window to
    // 6982 (`docs/AI.md` §253). The move's value diff is this block.
    // Item 3369: 41 -> 32, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 32, "initial run932 baseline");
}

/// run934: the word 6982's window, blocks 6976..7232 at run932's detail
/// (`LEADERS=9` for the make list, `GUYS=4` for the figure clocks; item
/// 3231) — six blocks before the word's own block 6983, 250 after; no
/// earlier dump of this lobby holds 6094..12658.
pub(crate) const RUN934: &str =
    "gamelog-run934-greatlakes-iroquois-diff3-window-6976-7233-guys4.txt";

/// The window: block 6976 through 7232.
pub(crate) const WIDENING_IROQUOIS_FRAME_6982: (i64, i64) = (6976, 7232);

pub(crate) fn iroquois_frame_6982_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run934",
        &[(RUN934, WIDENING_IROQUOIS_FRAME_6982.0)],
        WIDENING_IROQUOIS_FRAME_6982,
        1,
        &[6983],
        true,
    )
}

#[test]
fn run934_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_6982_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3231, on base 5b18de8c: 1105 keys, the first non-standing row
    // block 6982's `leader:MAKE[3]` (ours `t` 171 val 62976 escrow 1 cat 7,
    // theirs 438 val 80000 escrow 0 city 1 cat 8: the Senate). With
    // `get_cost`'s Iroquois Senate arm, 986, the first block 6983's
    // `1/2023` `construct_hits` (ours 1, theirs 1200); with
    // `construct_time`'s, 41: run932's standing set (the `SITE` regs, the
    // forms, pools 64 and 66, `1/22`'s `blown`, `1/23`'s second figure's
    // nine), the newborn figures' `g.angle[0]` on `1/35`, `1/36` and
    // `1/37` (`Unit::init`'s 0x55555555 against 0, a family standing in
    // other lobbies) and `SITE[1].reg` on 7176. The move's value diff is these two blocks.
    // Item 3369: 41 -> 32, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 32, "initial run934 baseline");
}

/// run937: the word 7982's window, blocks 7976..8232 at run934's detail
/// (`LEADERS=9` for the make list, `GUYS=4` for the figure clocks; item
/// 3242) — six blocks before the word's own block 7983, 250 after; no
/// earlier dump of this lobby holds 7232..12658.
pub(crate) const RUN937: &str =
    "gamelog-run937-greatlakes-iroquois-diff3-window-7976-8233-guys4.txt";

/// The window: block 7976 through 8232.
pub(crate) const WIDENING_IROQUOIS_FRAME_7982: (i64, i64) = (7976, 8232);

pub(crate) fn iroquois_frame_7982_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run937",
        &[(RUN937, WIDENING_IROQUOIS_FRAME_7982.0)],
        WIDENING_IROQUOIS_FRAME_7982,
        1,
        &[7983],
        true,
    )
}

#[test]
fn run937_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_7982_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3242, on base 160f3033: 631 keys. The first non-standing row is
    // block 7980's `leader:MAKE[7].t`, ours 167 (`IRQMOSPEAR`) against
    // theirs 83 (`JAVELINEERS`), both val 251904, cat 7: ours offered the
    // Javelineers at 62976, unaffordable at 80/80 against 74 timber, and
    // block 7983 is theirs' buy (`MAKE[7].val` 2519, `bucket[0]` 19 and
    // `bucket[1]` 7, ours 87 and 75: 68/68). With `get_cost`'s Despotism
    // arm on a Barracks unit, 51: the standing set on 7976, pools 65 (7994)
    // and 66 (8031), `SITE[2].reg` on 8176. The word moved past the window
    // to 8415 (`docs/AI.md` §255). The move's value diff is blocks 7980 and
    // 7983.
    // Item 3369: 51 -> 42, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 42, "initial run937 baseline");
}

/// run940: the word 8415's window, blocks 8409..8665 at run937's detail
/// (`LEADERS=9`, `GUYS=4`; item 3248) — six blocks before the word's own
/// block 8416, 250 after; no earlier dump of this lobby holds 8232..12658.
pub(crate) const RUN940: &str =
    "gamelog-run940-greatlakes-iroquois-diff3-window-8409-8666-guys4.txt";

/// The window: block 8409 through 8665.
pub(crate) const WIDENING_IROQUOIS_FRAME_8415: (i64, i64) = (8409, 8665);

pub(crate) fn iroquois_frame_8415_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run940",
        &[(RUN940, WIDENING_IROQUOIS_FRAME_8415.0)],
        WIDENING_IROQUOIS_FRAME_8415,
        1,
        &[8416],
        true,
    )
}

#[test]
fn run940_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_8415_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3248, on base bfd657c1: 909 keys. The first rows past the
    // standing set (8409) are block 8415's `1/24` second figure (theirs'
    // Merchant snaps it onto figure 0 at its cast's end, ours walks it on:
    // `g.x[1]` 1622 against 1536) and block 8416's birth: theirs' Barracks
    // `1/2016` empties (`queue:queued` ours 2, theirs 1), the three
    // Javelineers `1/48`..`1/50` the dump holds alone, `num_units[33]` ours
    // 1 against theirs 2. With `train_time`'s Cotton arm, 61: the standing
    // set on 8409 (the `SITE` regs, the forms, pools 64 and 66, `1/22`'s
    // `blown`, `1/23`'s second figure, the newborn figures' `g.angle[0]`,
    // `0/-1`'s `aggression[1]`), `1/24`'s second figure on 8415 and its
    // `g.end_time[1]` on 8416, pools 65 (8416) and 67 (8497), and three
    // `dest_angle`s (8453, 8463). The word moved past the window to 10695
    // (`docs/AI.md` §256). The move's value diff is block 8416.
    // Item 3369: 61 -> 43, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 43, "initial run940 baseline");
}

/// run943: the word 10695's window, blocks 10689..10945 at run940's detail
/// (`LEADERS=9`, `GUYS=4`; item 3255) — six blocks before the word's own
/// block 10696, 250 after; no earlier dump of this lobby holds 8666..12658.
pub(crate) const RUN943: &str =
    "gamelog-run943-greatlakes-iroquois-diff3-window-10689-10946-guys4.txt";

/// The window: block 10689 through 10945.
pub(crate) const WIDENING_IROQUOIS_FRAME_10695: (i64, i64) = (10689, 10945);

pub(crate) fn iroquois_frame_10695_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run943",
        &[(RUN943, WIDENING_IROQUOIS_FRAME_10695.0)],
        WIDENING_IROQUOIS_FRAME_10695,
        1,
        &[10696],
        true,
    )
}

#[test]
fn run943_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_10695_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3255, on base 37884575: 2032 keys, 47 of them the standing set
    // on 10689 (the forms, the `SITE` regs, pools 64 and 66, `0/-1`'s
    // `aggression[1]`, `1/23`'s and `1/24`'s second figures). **The state
    // first parts on block 10692**: the Scout `0/0` is struck on frame
    // 10691 in theirs (`damage_frame` 10691, `hits_left` 46 against ours'
    // 50, its `FLEE_TO` to (1992, 32280) on the block) and on 10692 in
    // ours — the Archers `1/32`'s arrow, fired on 10674 on both sides,
    // flies 19 frames in ours and 18 in theirs, and every row after is
    // that frame's lag (`docs/AI.md` §257). **With the release rows, 456
    // keys**: the 47 standing, and the state first parts on 10843 (`1/32`'s
    // `dest_angle`), the word's own block 10853 after it. (The widening's
    // block; the word's delta is in `IROQUOIS`'s comment.) **Item 3269,
    // 455**: 10843 closes — a follower's `check_target` reads its squad
    // captain's action through `update_action`, which copies `1/32`'s new
    // heading into its `dest_angle` the frame it turns (run946's packet,
    // `docs/AI.md` §258); the state now first parts on the word's own
    // block 10853 (`0/3`'s `death:extra`, who=1's metal 44 against 59:
    // The Despot's `Unit::plunder`, not modelled). **Item 3279, 174**:
    // the plunder is built (who=1's `bucket[4:metal]` 59 on both sides on
    // 10853), and `death:extra` is not a parting on a capture that logs no
    // `DEATHS` (run943 prints no `DEATH_OBJS`). The state first parts on
    // 10853 still: `1/25`, upgraded Javelineers → Peltasts on 10852 on both
    // sides, `g.cur_anim[0]` ours 1 against theirs 2 and `g.end_time[0]` 96
    // against 36. **Item 3287, 47**, the standing set alone through
    // 10945: `Guy::init_real` zeroes `guy_flags`, so `1/25`'s re-typed
    // figure rolls `IDLE2` (`g.cur_anim[0]` 2, `g.end_time[0]` 36 on both
    // sides on 10853); the Peltasts' bays (piece 1442) bring `0/2000`'s
    // `build:damage` to 44 on both on 10870; `set_diags`' corner tail keeps
    // the caravan diagonal (235, 153) (`docs/AI.md` §260).
    // Item 3369: 47 -> 29, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 29, "run943 with set_diags' corner tail");
}

/// run952: the word 11585's window, blocks 11579..11835 at run943's
/// detail (`LEADERS=9`, `GUYS=4`; item 3297) — six blocks before the
/// word's own block 11586, 250 after; run943 ends on 10945.
pub(crate) const RUN952: &str =
    "gamelog-run952-greatlakes-iroquois-diff3-window-11579-11836-guys4.txt";

/// The window: block 11579 through 11835.
pub(crate) const WIDENING_IROQUOIS_FRAME_11585: (i64, i64) = (11579, 11835);

pub(crate) fn iroquois_frame_11585_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run952",
        &[(RUN952, WIDENING_IROQUOIS_FRAME_11585.0)],
        WIDENING_IROQUOIS_FRAME_11585,
        1,
        &[11586],
        true,
    )
}

#[test]
fn run952_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_11585_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3297: 786 keys on its first build, opening already parted on
    // 11579 (who=1's food 78 against 98, The Despot `1/46`'s Create Decoy
    // in ours alone, the make list from 11580). **60** with Ambush's hold
    // and the building plunder's Despot cut: the standing set, `1/53`'s
    // `damage_frac`, the Scholars `1/71`/`1/73`'s `g.angle[0]` and `1/66`'s
    // second figure. (The widening's block; the word's delta is in
    // `IROQUOIS`'s comment.)
    // Item 3369: 60 -> 33, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 33, "run952 with the Despot's cut");
}

/// run954: the word 12156's window, blocks 12150..12406 at run952's
/// detail (`LEADERS=9`, `GUYS=4`; item 3306) — six blocks before the
/// word's own block 12157, 250 after; run952 ends on 11835.
pub(crate) const RUN954: &str =
    "gamelog-run954-greatlakes-iroquois-diff3-window-12150-12407-guys4.txt";

/// The window: block 12150 through 12406.
pub(crate) const WIDENING_IROQUOIS_FRAME_12156: (i64, i64) = (12150, 12406);

pub(crate) fn iroquois_frame_12156_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run954",
        &[(RUN954, WIDENING_IROQUOIS_FRAME_12156.0)],
        WIDENING_IROQUOIS_FRAME_12156,
        1,
        &[12157],
        true,
    )
}

#[test]
fn run954_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_frame_12156_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3369: 61 -> 34, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    // Item 3375: 34 -> 33, the Cavalry Archers' piece 1546 in
    // `sim::pivot::RELEASES`: block 12231's `0/2000` `build:damage`, ours
    // 902 against theirs 904, agrees.
    pin_eq!(w.firsts.len(), 33, "run954 on its first build");
    // Item 3306: 1118 on the first build, 1088 with the held building's
    // yield (the parting index 1 → 10), and 61 with the yield's skip of
    // the flee arm and the on-duty gate. Block 12150 holds the standing
    // set (58 rows: `form`, `SITE.reg`, pool 64/66, the Scholars'
    // `g.angle[0]`, `1/53`'s `damage_frac`); past it only `0/2000`'s
    // `build:damage` (12231, ours 902 against theirs 904), who=1's
    // `SITE[1].reg` (12376) and the newborn `1/98`'s `g.angle[0]` (12381)
    // part, and no position or order parts in the window.
}

/// The closing state's window: run897b's closing block alone, the game's
/// last frame 12658 writing it (item 3306).
pub(crate) const WIDENING_IROQUOIS_CLOSING: (i64, i64) = (12657, 12660);

/// **The lobby's closing state** (item 3306, DECISIONS 55): the draws
/// agree to the game's last frame, 12658, and run897b — the draw stream's
/// own capture, `cover=0` — prints the closing block whole. Walked to it
/// and compared whole, as `run643_german_closing_state` is.
#[test]
fn run897b_iroquois_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(IROQUOIS.long.0),
        dump(IROQUOIS.start),
        trace(IROQUOIS.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
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
        "Iroquois closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 12659, "the closing block");
    pin_eq!(e.compared, 104, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3306, the first walk: 104 units compared on block 12659, none
    // apart, no building or city field-row, and no closing field parted.
}

/// run952b: the walk-back behind run952's first block, blocks
/// 10940..11579 at run952's detail (item 3297) — run943 ends on 10945 and
/// run952 opens on 11579 already parted (who=1's food 78 against 98).
pub(crate) const RUN952B: &str =
    "gamelog-run952b-greatlakes-iroquois-diff3-window-10940-11580-guys4.txt";

/// The window: block 10940 through 11579.
pub(crate) const WIDENING_IROQUOIS_GAP_952B: (i64, i64) = (10940, 11579);

pub(crate) fn iroquois_gap_952b_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run952b",
        &[(RUN952B, WIDENING_IROQUOIS_GAP_952B.0)],
        WIDENING_IROQUOIS_GAP_952B,
        1,
        &[11579],
        true,
    )
}

#[test]
fn run952b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_gap_952b_window() else {
        return;
    };
    pin_eq!(w.blocks, 640, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3297: 63 keys, the state first parting past the standing set on
    // block 11259 (who=1's `bucket[0:food]` 69 against 89: `0/2002` razed
    // beside The Despot, `Build::plunder`'s hero cut). **59** with the cut,
    // 89 on both sides (`docs/AI.md` §261).
    // Item 3369: 59 -> 32, an unpacked merchant's "over time" piece
    // (`sim::anim::stall_piece`): its crew on one piece at its point.
    pin_eq!(w.firsts.len(), 32, "run952b with the Despot's cut");
}

/// **The Despot's Ambush holds its craft** (item 3297, `docs/AI.md`
/// §261): `SpellType::cast_ambush@00671610` sets `unit_masks |= 0x2800`,
/// and `Unit::process`'s caster arm takes no craft back while `0x2000`
/// stands. The original's `1/46` (TI 352, The Despot) casts on the move on
/// 10521 and its `mana_burn` reads 1000 on every block of run943
/// (10689..10945, `unit_masks` `0x42008`) and 392 on run952's 11579
/// (`0x40008`), one a frame from the `ActiveSpell`'s end. `mana_burn` is
/// on the coverage driver's unread list, so no widening compared it.
#[test]
fn the_despot_s_ambush_holds_its_craft() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(IROQUOIS.long.0),
        dump(IROQUOIS.start),
        trace(IROQUOIS.long.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", IROQUOIS.long.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let (log, sib_log) = (Log::parse(&text), Log::parse(&sib_text));
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_log.initial().expect("a start dump")]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    // (block, theirs' `mana_burn`): run943's first and last, run952's
    // first and the word's own.
    let theirs = [(10689, 1000), (10945, 1000), (11579, 392), (11586, 385)];
    let mut parted = Vec::new();
    for (block, want) in theirs {
        while built.sim.frame < block {
            built.tick();
        }
        let ours = built
            .sim
            .unit_by_o(1, 46)
            .map(|u| i64::from(built.sim.units[u].mana_burn));
        if ours != Some(want) {
            parted.push(format!("block {block}: ours {ours:?} theirs {want}"));
        }
    }
    assert!(parted.is_empty(), "the Despot's craft: {parted:?}");
}

/// run930: the walk-back behind run928's first block, blocks 5556..5578 at
/// run918's detail with `GUYS=4` (item 3202) — the Slinger squad's birth on
/// 5563 and the figures' clocks on the word's block 5577, which run928's
/// `GUYS=2` does not print.
pub(crate) const RUN930: &str =
    "gamelog-run930-greatlakes-iroquois-diff3-window-5556-5579-guys4.txt";

/// The window: block 5556 through 5578.
pub(crate) const WIDENING_IROQUOIS_SQUAD_930: (i64, i64) = (5556, 5578);

pub(crate) fn iroquois_squad_930_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS.start],
        true,
        IROQUOIS.long,
        "run930",
        &[(RUN930, WIDENING_IROQUOIS_SQUAD_930.0)],
        WIDENING_IROQUOIS_SQUAD_930,
        1,
        &[5577],
        true,
    )
}

#[test]
fn run930_s_squad_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_squad_930_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3202, on base 128293bb: 115 keys, 29 of them the standing set
    // on the window's first block. **The state first parts on block
    // 5564**, the frame the Slinger squad born on 5563 (`1/25` captain,
    // `1/26`, `1/27` down its `o_down` chain, TI 82, group 64) takes its
    // `ATTACKTO` to (36552, 21960): the followers' `off_x`/`off_y` ours
    // 456/600 and 456/312 against theirs 408/552 and 408/360, and their
    // figures' `last_speed` ours 28 (`myspeed`) against theirs 35, the
    // captain's on 5565. At 35 theirs' figures jog (`cur_anim` 9, from
    // 5574); `1/26`'s stops on frame 5574 (`stopped` 1, `cur_anim` 23,
    // `end_time` 3) and wraps on 5576 into the idle roll — block 5577's
    // `cur_anim` 0, `end_time` 75 — which is the word's index-30
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`; theirs' third wing
    // beat is a bird's re-throw on the shifted stream (`docs/AI.md`
    // §252). Item 3211, on base 9e0fa5ee: 115 → 31, the standing set
    // alone. With `is_modern_infantry`'s Iroquois arm the squad agrees on
    // 5564 — `1/26` `off_x`/`off_y` 408/552, `1/27` 408/360, every
    // figure's `last_speed` 35 on both sides — and through the window's
    // end (`docs/AI.md` §253). The word's block in this widening.
    pin_eq!(w.firsts.len(), 31, "initial run930 baseline");
}

/// **run946's road elements at logger frame 10842, against ours** (item
/// 3287, `docs/ROADS.md` §12). The packet
/// (`~/ron-golden/run946/map-14/frame-snapshot.bin`) holds the twelve tiles
/// around the caravan diagonal (235, 153) that `1/2027`'s ring reached on
/// 10485. Each row is the element's connection byte, `support_codes`, the
/// two counts and whether `element_num` is the C4 piece (55), read through
/// `terrain +0x6ab0`. Theirs' (236, 153) is a slot with no counts, which
/// this crate holds as no element. The two counts are compared apart —
/// the caravan's tiles hold camel steps, the ring's references — except at
/// (235, 154), which agrees by their sum alone: theirs ref 2 / camel 0, ours
/// ref 1 / camel 1 (item 3287's parked row).
#[test]
fn run946_s_road_elements_at_10842_are_ours() {
    use sim::mesh::dir::{E, N, NE, S, SW, W};
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(IROQUOIS.long.0),
        dump(IROQUOIS.start),
        trace(IROQUOIS.long.1),
    ) else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", IROQUOIS.long.0);
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let (log, sib_log) = (Log::parse(&text), Log::parse(&sib_text));
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_log.initial().expect("a start dump")]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    while built.sim.frame < 10_843 {
        built.tick();
    }
    // (x, y, connections, support, (ref, camel), C4)
    type Row = (i32, i32, u32, u16, (u8, u8), bool);
    let theirs: [Row; 12] = [
        (234, 152, E | W, 0, (1, 0), false),
        (235, 152, E | S | W, 0, (1, 0), false),
        (236, 152, E | W, 0, (0, 1), false),
        (237, 152, N | W, 0, (1, 0), false),
        (234, 153, 0, 0, (1, 0), true),
        (236, 153, 0, 0, (0, 0), false),
        (235, 153, N | SW, 0xc400, (0, 1), false),
        (234, 154, NE | SW, 0xc200, (0, 1), false),
        (233, 155, NE | W, 0xda00, (0, 1), false),
        (233, 154, 0, 0, (2, 0), true),
        (235, 154, 0, 0, (2, 0), true),
        (232, 155, N | E | S, 0, (1, 0), false),
    ];
    let mut parted = Vec::new();
    for (x, y, flags, support, refs, c4) in theirs {
        let e = built
            .sim
            .mesh
            .elem(&built.sim.world, Pos::new(x, y))
            .unwrap_or_default();
        let mut counts = (e.refs, e.refs2);
        if (x, y) == (235, 154) {
            // By the sum alone (item 3287's parked row).
            counts = (counts.0 + counts.1, 0);
        }
        let ours = (x, y, e.flags & 0xff00_0000, e.support, counts, e.c4);
        if ours != (x, y, flags, support, refs, c4) || e.made {
            parted.push(format!(
                "({x}, {y}) ours {:#x} {:#x} {:?} {} made {} theirs {flags:#x} {support:#x} {refs:?} {c4}",
                ours.2, ours.3, ours.4, ours.5, e.made
            ));
        }
    }
    assert_eq!(
        parted,
        Vec::<String>::new(),
        "a road element parts on 10842"
    );
}
