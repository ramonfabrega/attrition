//! **Great Sahara in the coverage pair's lobby** (DECISIONS 56 §1, item
//! 1538): the closed map's successor once Toughest's closing state agreed.
//! `MAP_STYLE 7`, human Nubians (4) against Persians (23) at Toughest,
//! `STARTING_TECHNOLOGY 8` and `STARTING_RESOURCES 7`, seed 12345 — run651's
//! and run652's recipes with the map moved and nothing else. The captures
//! are run675 (the `DUMP_ALL` start, the lobby's own sibling) and run676 (the
//! draw stream to the game's end, `cover=0`). The first parting is the third
//! map's word in the newest pair's lobby; `docs/AI.md` §147 has it.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SAHARA_COVERAGE: Pair = Pair {
    name: "GreatSaharaPersianAllTech",
    map_style: 7,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run675-greatsahara-persian-alltech-start.txt",
    long: (
        "gamelog-run676-greatsahara-persian-alltech-24k-trace.txt",
        "rontrace-run676.log",
    ),
    length: 4340,
    // Item 1538 moved it 12 → 718 (the tied oil patches); item 1544's
    // `total_units` tally, merged beside it, moved it 718 → 720: ours
    // `Guy::set_anim+0x97a < Guy::move+0x19f` against the original's
    // `Animal::think_bird+0x82`, 25 draws against 24. Item 1549's build (the
    // oil well's stand is on `CHAR_FARM`, `docs/AI.md` §150) moved it
    // 720 → **1182**: ours 146 draws against 84 at index 2, ours
    // `Leader::produce_building+0xc99`, theirs `Leader::make_stuff+0x63d`.
    //
    // Item 1546's second-capital arm (`Build::activate`'s 0x17, merged beside
    // this build) moved it again, 1182 → **1197**: ours 4 draws against 3 at
    // index 0, ours `Unit::do_move+0xe84`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`.
    //
    // Item 1555's build (the border pass's rare arm, `docs/AI.md` §152: the
    // Persian Merchant `1/38` picks the Wool at (40992, 1824) the pass meets
    // and `reveal_fog`'s tile gate never did) moved it again, 1197 →
    // **1250**: ours 10 draws against 9 at index 0, ours `Guy::set_anim+0x97a
    // < Unit::move_step+0x823`, theirs `Animal::do_idle+0x83`.
    //
    // Item 1561's build (a walker blocked by a walking animal repaths rather
    // than waits, `docs/AI.md` §154: step 5's `is_enemy` reads a gaia owner
    // as an enemy) moved it again, 1250 → **1582**: ours 28 draws against 29
    // at index 8, ours `Leader::make_stuff+0x63d`, theirs
    // `Leader::produce_building+0x1805`.
    //
    // Item 1565's build (an enhancer stays in the city it is bought for:
    // `produce_building`'s `get_town(cand) != city` skip, `docs/AI.md`
    // §156 — the Refinery `1/2045` had taken a cell of city 1's) moved it
    // again, 1582 → **1818**: ours 3215 draws against 3218 at index 3201,
    // ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `PathFinder::calc_road_cost+0x46` (a caravan road's A*).
    //
    // Item 1576's build (a route reset by another's road clears the reset
    // when it restarts: `Caravan::build_road`'s `0073dbfc`, `docs/AI.md`
    // §157 — caravan 7's search started over on 1818 where the original
    // resumed it) moved it again, 1818 → **1830**: ours 3218 draws against
    // 3219 at index 2, ours `PathFinder::calc_road_cost+0x46`, theirs
    // `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`.
    //
    // Item 1577's build (`Army::normalize` runs `Group::normalize` whole, its
    // speed tail included, so an army's 128-frame turn resets its march's cap
    // to the leader's own speed, `docs/AI.md` §158 — the Supply Wagon `1/67`
    // stepped 41 where the original steps 43 on tick 1562) moved it again:
    // the sequence 1830 → **1985**, ours 16 draws against 16 at index 4, ours
    // `Leader::produce_building+0x1805`, theirs `Leader::make_stuff+0x63d`;
    // the count 1830 → **2048**.
    //
    // Item 1591's build (the silo's arm of the computer's sortie,
    // `docs/AI.md` §164), measured on this map by item 1579: silo `1/2017`
    // orders its ICBM `1/96` at Napata on 1951 and launches it on 1982
    // (`docs/AI.md` §160). It holds the word at 1985 and brings the count
    // down to it, 2048 → **1985**: ours 18 draws against 16 at index 6, ours
    // `Leader::make_stuff+0x63d`, theirs `Leader::produce_building+0x1805`.
    count: 1985,
    sequence: 1985,
};

/// The lobby's word as the handoff's `Third map:` line and `AI_WORDS` carry
/// it: the lower of the two partings.
pub(crate) const SAHARA_COVERAGE_WORD: i64 = if SAHARA_COVERAGE.count < SAHARA_COVERAGE.sequence {
    SAHARA_COVERAGE.count
} else {
    SAHARA_COVERAGE.sequence
};

/// **The third map's first parting in the coverage pair's lobby**, walked
/// from its own start: the frame the draw count first parts on, the frame
/// the draw sequence first parts on, and the lobby read back.
#[test]
fn sahara_coverage_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SAHARA_COVERAGE.start, SAHARA_COVERAGE.long) else {
        return;
    };
    eprintln!(
        "SAHARA_COVERAGE {}: count {} sequence {} of {}; {}",
        SAHARA_COVERAGE.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        SAHARA_COVERAGE.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, SAHARA_COVERAGE.length, "the game's last frame");
    pin_eq!(
        w.count,
        SAHARA_COVERAGE.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SAHARA_COVERAGE.sequence,
        "the draw sequence's first parting"
    );
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — map, difficulty, the two settings the lobby exists for, nations
/// and seed — and the long is scoreable. A setting that did not take reads
/// back as its default here.
#[test]
fn sahara_coverage_captures_say_their_lobby() {
    let p = &SAHARA_COVERAGE;
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
        assert_eq!(
            info("STARTING_TECHNOLOGY"),
            Some(p.starting_technology),
            "{name}"
        );
        assert_eq!(
            info("STARTING_RESOURCES"),
            Some(p.starting_resources),
            "{name}"
        );
        assert_eq!(info("GAME_RULES"), Some(1), "{name}");
        let tribes: Vec<_> = init
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe))
            .collect();
        assert_eq!(tribes, [(0, 4), (1, i64::from(p.ai_tribe))], "{name}");
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Sahara coverage long is not scoreable"
    );
}

/// The lobby is a scored one and none of the battery's.
#[test]
fn sahara_coverage_is_no_other_lobby() {
    let p = &SAHARA_COVERAGE;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run677, item 1538: the lobby's blocks 6..262 at the long's detail — the
/// word 12's block 13 with seven before it and 249 after.
pub(crate) const RUN677: &str = "gamelog-run677-greatsahara-persian-alltech-window-6-262.txt";

/// The window: block 6 through 262; the word 12's own block is 13.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_12: (i64, i64) = (6, 262);

/// **The coverage lobby's frame-12 window**: run677 walked from run675's
/// start with the recorder on. It was the `AI_WORDS` window until item
/// 1538's build moved the word past it, to run680's.
pub(crate) fn sahara_coverage_frame_12_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run677",
        &[(RUN677, WIDENING_SAHARA_COVERAGE_FRAME_12.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_12,
        1,
        &[13],
        true,
    )
}

#[test]
fn run677_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_12_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **395** on the tree at base 0edf81c8, before the wholesale oil claim
    // walked a land region's cells in order (`docs/AI.md` §147): 67 stand on
    // block 6 (the control's set: both leaders' `form`, the blank `SITE`
    // slots' `reg`, the city fields, the pools, `scouts`), and what the
    // build left is leader 1's `known_rares` on block 8 and leader 0's
    // `production_step` on 201, the control's too. **69.**
    pin_eq!(w.firsts.len(), 69, "initial run677 baseline");
    // **The word's value diff, block 12** (item 1538): the Oil Well `1/2013`
    // (`TypeIndex` 421, `OILWELL`) stood at (34944, 10368) in ours — the
    // patch at cell (45, 13) — and at (44928, 8832) in theirs, cell (58, 11):
    // two patches tied at score 108 from the anchor (53, 22), the list's
    // order deciding which. It agrees in every compared field, and so do
    // the builders' orders and paths (`1/1`, `1/2`, `1/6`).
    assert!(
        w.firsts.keys().all(|(who, b, _)| (*who, *b) != (1, 2013)),
        "1/2013 agrees in every compared field"
    );
    for o in [1, 2, 6] {
        assert!(
            w.firsts
                .keys()
                .all(|(who, u, f)| (*who, *u) != (1, o) || !f.starts_with("order")),
            "1/{o}'s order stack agrees on every block"
        );
    }
    // **What parts first past the standing block**: leader 1's `known_rares`
    // on block 8 (ours 0, theirs 1), read by `civilian_value`.
    let first = w.firsts.values().map(|(f, _)| *f).filter(|f| *f > 6).min();
    pin_eq!(first, Some(8), "the first parting past the standing block");
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:known_rares".to_string()))
            .map(|(f, _)| *f),
        Some(8),
        "leader 1's known rares part on block 8"
    );
}

/// run680, item 1549: the lobby's blocks 713..969 at the long's detail —
/// the word 720's block 721 with eight before it and 248 after.
pub(crate) const RUN680: &str = "gamelog-run680-greatsahara-persian-alltech-window-713-969.txt";

/// The window: block 713 through 969; the word 720's own block is 721.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_720: (i64, i64) = (713, 969);

/// **The coverage lobby's frame-720 window**: run680 walked from run675's
/// start with the recorder on. It was the `AI_WORDS` window until item 1549's
/// build moved the word past it, to run681's.
pub(crate) fn sahara_coverage_frame_720_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run680",
        &[(RUN680, WIDENING_SAHARA_COVERAGE_FRAME_720.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_720,
        1,
        &[721],
        true,
    )
}

#[test]
fn run680_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_720_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **582** on the tree at base 9b962c3c, before the oil well's stand took
    // the farm animation (`docs/AI.md` §150), **111** after: 83 stand from
    // the window's first block (the control's set: both leaders' `SITE`
    // `reg`, `form`, the pools, `scouts`, the `ally_mask`/territory counts),
    // and 28 part later — pools 66, 67 and 68 and their groups' `held`, the
    // Persians' `SITE` table on 776, the two queues' prices and leader 1's
    // two buckets on 783, leader 0's `production_step` on 801.
    // Item 1578: 111 → 74, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 74, "initial run680 baseline");
    // **The word's value diff, block 721** (item 1549): the dump prints no
    // animation (`GUYS=2`'s `GUY` record is type, position and angle), so the
    // value on the word's frame is the draw record — frame 720, ours 25
    // against 24 at index 0, ours `Guy::set_anim+0x97a < Guy::move+0x19f`
    // (the arrival stand of the Peasant `1/1`, `TypeIndex` 50, land) and
    // theirs `Animal::think_bird+0x82`. `1/1` finished the Oil Well `1/2013`
    // on 718 and stood at its (45216, 8832) from 719 in both; ours put it on
    // `CHAR_WALK` (anim 8) through the turn arm and rolled the stand on 720,
    // and the original's `do_gather` puts the guy on `CHAR_FARM` first. It
    // agrees in every compared field through the window's last block.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| (*who, *o) == (1, 1) && *f > 713)
            .count(),
        0,
        "1/1 agrees in every compared field past its standing `form`"
    );
    // **What parts first past the standing block**: pool 66 and its group's
    // `held` on 742 (ours [17], theirs none).
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 713)
        .min();
    pin_eq!(
        first,
        Some(742),
        "the first parting past the standing block"
    );
    pin_eq!(
        w.firsts
            .get(&(1, -2, "pool:66".to_string()))
            .map(|(f, _)| *f),
        Some(742),
        "leader 1's pool 66 parts on block 742"
    );
}

/// run681, item 1555: the lobby's blocks 1191..1447 at the long's detail —
/// the word 1197's block 1198 with seven before it and 249 after.
pub(crate) const RUN681: &str = "gamelog-run681-greatsahara-persian-alltech-window-1191-1447.txt";

/// The window: block 1191 through 1447; the word 1197's own block is 1198.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1197: (i64, i64) = (1191, 1447);

/// run681 walked from run675's start with the recorder on: the word 1197's
/// block 1198 and the word 1250's block 1251. It was the `AI_WORDS` window
/// until item 1561's build moved the word past it, to run683's.
pub(crate) fn sahara_coverage_frame_1250_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run681",
        &[(RUN681, WIDENING_SAHARA_COVERAGE_FRAME_1197.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_1197,
        1,
        &[1198, 1251],
        true,
    )
}

#[test]
fn run681_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_1250_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **1138** on the tree at base a792e271 (the word 1197), **885** after
    // the border pass met the Wool (`docs/AI.md` §152), **185** after item
    // 1561's build (a walker blocked by a walking animal repaths rather than
    // waits, `docs/AI.md` §154): 140 stand on block 1191 (the control's set:
    // both leaders' `SITE` `reg`, `form`, the city and pool fields, `scouts`,
    // the territory counts) and the rest part later. The 700 that went part
    // downstream of `1/35`'s stall on 1247 (not itemised row by row).
    // Item 1578: 185 → 128, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 128, "initial run681 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1191).count(),
        96,
        "the keys standing on the window's first block"
    );
    // **The word 1197's value diff, block 1198** (item 1555): the Persian
    // Merchant `1/38` (`TypeIndex` 61, land), born on 1195 at (39048,
    // 17112), is given its walk on 1197 — to (31032, 21048) in ours, the
    // Dye at (31008, 21024), and to (41016, 1848) in theirs, the Wool at
    // (40992, 1824) — and ours spends the grid draw `Unit::do_move+0xe84`
    // on a far walk the original's pick does not need. Leader 1's
    // `new_rares` is the cause: [10, 22, 12, 13] in ours (the Peacocks,
    // Uranium, Dye, Wine), and the original's list holds the Wool in
    // fourth place from the cell's first pass at 439. It agrees in every
    // compared field since the build.
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 38)),
        "1/38 agrees in every compared field"
    );
    // **The word 1250's value diff, block 1251** (item 1561): `1/35`, a
    // Persian soldier (`TypeIndex` 104, land) on an `AttackTo` to (36456,
    // 19080), proposes (38371, 20297) on 1247 and meets the gaia animal
    // `8/2` (`TypeIndex` 413) walking ahead of it: ours waits (step 5 of
    // `resolve_unit_collision`) and stands where it is through 1250, the
    // original repaths — its path's fifth and sixth nodes and its position
    // part on 1248 — and steps. `8/2`'s own step on 1250 then differs by
    // one `Unit::move_step+0x823` stand roll, which is the word. It agrees
    // in every compared field since the build.
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 35)),
        "1/35 agrees in every compared field"
    );
    // **What parts first past the standing block**: leader 0's
    // `production_step` on 1201 (ours 0, theirs 1).
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1191)
        .min();
    pin_eq!(
        first,
        Some(1192),
        "the first parting past the standing block"
    );
}

/// run683, item 1561: the lobby's blocks 1577..1833 at the long's detail —
/// the word 1582's block 1583 with six before it and 250 after, and the word
/// 1818's block 1819 (item 1565) with 242 before it and fourteen after, and
/// the word 1830's block 1831 (item 1576) with two after.
pub(crate) const RUN683: &str = "gamelog-run683-greatsahara-persian-alltech-window-1577-1833.txt";

/// The window: block 1577 through 1833; the word 1582's own block is 1583.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1582: (i64, i64) = (1577, 1833);

/// run683 walked from run675's start with the recorder on — the word 1582's
/// block 1583, the word 1818's block 1819 and the word 1830's block 1831. It
/// was the `AI_WORDS` window until item 1577's build moved the word past
/// it, to run693's.
pub(crate) fn sahara_coverage_frame_1830_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run683",
        &[(RUN683, WIDENING_SAHARA_COVERAGE_FRAME_1582.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_1582,
        1,
        &[1583, 1819, 1831],
        true,
    )
}

#[test]
fn run683_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_1830_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **1183** on the tree at base e02e4b0c (the word 1582), **863** after
    // item 1565's build (an enhancer stays in the city it is bought for,
    // `docs/AI.md` §156): the 320 that went part downstream of the
    // Refinery's site and its four jitter draws (not itemised row by row).
    // **845** after item 1576's build (a reset route's restart clears the
    // reset, `docs/AI.md` §157): the eighteen that went part on 1818..1829
    // downstream of caravan 7's search, which drew 3201 rolls where the
    // original's resumed one drew 3204.
    // **185** after item 1577's build (an army's 128-frame normalize resets
    // its group's speed, `docs/AI.md` §158): the soldiers' guard orders and
    // half steps that parted from 1719 agree; 151 stand on 1577. (Item
    // 1586 landed the same tail on the integration branch first, for the
    // coverage pair's army `1/0`, and pinned 540 there on its own tree.)
    // Item 1578: 185 → 126, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 126, "initial run683 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1577).count(),
        110,
        "the keys standing on the window's first block"
    );
    // **The word 1582's value diff, block 1583** (item 1561): the Persians'
    // `REFINERY` (`orig_type` 426) `1/2045`, bought for city 2 (`1/2006`, at
    // (27744, 19296)) by a `MAKE` row both sides print alike, stands at
    // (30432, 19872) in the original — `city` 2, chained after `1/2006`
    // (`city_down` 2045) — and at (30432, 22176) in ours, `city` 1, chained
    // after `1/2041`. The original spends four `produce_building+0x1805`
    // jitter draws (a 2×2, every sub-position unblocked) and ours three, so
    // every roll after it sits a place off: the human's `0/5` and `1/65`
    // re-target on the same block from the shifted stream (their
    // `orders_x/y`, `dest_angle`), and are no cause.
    //
    // The cause (item 1565): `produce_building`'s `is_gather_enhancer` arm
    // skips a candidate whose tile `get_town` gives to another city than
    // the one the enhancer is bought for (`006e20b3`–`006e20f3`); this
    // crate skipped only "no city at all", and the spiral's last tie, at
    // (30432, 22176), was city 1's. It agrees in every compared field since
    // the build, and so does its city's chain.
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 2045)),
        "1/2045 agrees in every compared field"
    );
    for b in [2006, 2041] {
        pin!(
            !w.firsts
                .contains_key(&(1, b, "build:city_down".to_string())),
            "1/{b}'s city chain agrees"
        );
    }
    // **What parts first past the standing block** is the word's own block:
    // the human's `0/5`, the Persians' pool 65 and the queues' prices,
    // which stood beside the Refinery before the build and are no part of
    // it. **The word 1818's block 1819** (item 1565's move): the draw
    // stream's delta is three `PathFinder::calc_road_cost+0x46` draws of a
    // caravan road's A* the original spends and ours does not; the first
    // rows on the frame are the Persian soldiers' guard and path offsets
    // (`1/39`, `1/71`) downstream of the army's re-targets since 1789. The
    // draw stream on 1818 is a road search's, and the dump prints no
    // caravan slot: its value diff is the search's own first node, ours
    // (40800, 17184) from (40800, 16992) — a fresh start at caravan 7's
    // ends — against the original's (31008, 16800) from (31008, 16608), the
    // parked open list's best; since item 1576's build the two agree node
    // for node, 3204 each ([`run676_s_road_searches_hold_node_for_node`]).
    // **The word 1830's block 1831**: the original's third draw is a
    // guarding unit's stand (`Unit::do_guard+0x7f4`), and the rows on 1828
    // and 1829 are the Persian soldiers' guard orders (`1/29`, `1/69`,
    // `1/71` holding a second order the original does not) — landing
    // 1577's to read.
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1577)
        .min();
    pin_eq!(
        first,
        Some(1583),
        "the first parting past the standing block"
    );
}

/// The last frame [`run676_s_road_searches_hold_node_for_node`] walks: the
/// frame before the word 1830, whose own road search prices its third node
/// with a roll the original spent on `Unit::do_guard` (item 1576).
pub(crate) const SAHARA_COVERAGE_ROAD_FRAME: i64 = 1829;

/// **Every road search to the word 1830, node for node** (item 1576).
/// run676's trace proxies `valid_roadcoord` and `calc_road_cost` over the
/// whole game, so each search's priced nodes — the tile, its parent, the
/// wheel index and the answer — are comparable with ours from frame 0.
/// It parted on the word 1818 at node 0: caravan 0's road (`1/2000` to
/// `1/2018`) was laid on 1817 and `reset_paths` told caravan 7's parked
/// search (`1/2000` to `1/2028`) to start over — both sides did, on 1817 —
/// but this crate left `reset_road` set, so caravan 7 started over again on
/// 1818 where the original resumed (`Caravan::build_road`'s `0073dbfc`,
/// `docs/AI.md` §157).
#[test]
fn run676_s_road_searches_hold_node_for_node() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let p = &SAHARA_COVERAGE;
    let (Some(path), Some(sib), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1)) else {
        eprintln!("skipping: no run675/run676 (set RON_GAMELOG_DIR)");
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    built.sim.trace_costs = true;
    let mut searched = Vec::new();
    let mut parted = Vec::new();
    while built.sim.frame <= SAHARA_COVERAGE_ROAD_FRAME {
        let f = built.sim.frame;
        built.sim.road_marks.clear();
        built.tick();
        let ours = std::mem::take(&mut built.sim.road_marks);
        let theirs = tr.road_nodes(f);
        if ours.is_empty() && theirs.is_empty() {
            continue;
        }
        searched.push((f, ours.len(), theirs.len()));
        if let Some(i) = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i)) {
            let line = format!(
                "{f} at {i}: ours {:?} theirs {:?} ({} against {})",
                ours.get(i),
                theirs.get(i),
                ours.len(),
                theirs.len()
            );
            eprintln!("  road {line}");
            if std::env::var("RON_ROADS").is_ok() {
                for k in i.saturating_sub(12)..(i + 6).min(ours.len().max(theirs.len())) {
                    eprintln!("    {k}: ours {:?} theirs {:?}", ours.get(k), theirs.get(k));
                }
            }
            parted.push(line);
        }
    }
    eprintln!("road frames {}: {:?}", searched.len(), searched.last());
    pin!(parted.is_empty(), "a road search parted: {parted:?}");
    pin_eq!(searched.len(), 181, "the road-search frames to 1829");
    pin_eq!(
        searched.iter().find(|s| s.0 == 1818).copied(),
        Some((1818, 3204, 3204)),
        "1818's search resumes caravan 7's parked plan"
    );
}

/// run693, item 1577: the lobby's blocks 1979..2235 at the long's detail —
/// the word 1985's block 1986 with seven before it and 249 after, and the
/// draw count's own parting 2048's block 2049.
pub(crate) const RUN693: &str = "gamelog-run693-greatsahara-persian-alltech-window-1979-2235.txt";

/// The window: block 1979 through 2235; the word 1985's own block is 1986.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1985: (i64, i64) = (1979, 2235);

/// **The coverage lobby's word's window** (`AI_WORDS`' `Third map` row for
/// `GreatSaharaPersianAllTech`): run693 walked from run675's start with the
/// recorder on — the word 1985's block 1986 and the count's 2049.
pub(crate) fn sahara_coverage_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run693",
        &[(RUN693, WIDENING_SAHARA_COVERAGE_FRAME_1985.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_1985,
        1,
        &[1986, 2049],
        true,
    )
}

#[test]
fn run693_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **1401** on the tree at item 1577's build (an army's normalize resets
    // its group's speed, `docs/AI.md` §158): 157 stand from the window's
    // first block (the control's set, as run683's), and the rest part
    // from 1980.
    // Item 1578: 1401 → 1328, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    // Item 1579: 1328 → 1316, the ICBM `1/96`'s order (1979) and its launch (1982) agree (`docs/AI.md` §160);
    // 1316 → 1313 on the tree item 1591 landed (its nuke ring and pasture builds, not attributed key by key).
    pin_eq!(w.firsts.len(), 1313, "initial run693 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1979).count(),
        120,
        "the keys standing on the window's first block"
    );
    // **The word 1985's block 1986** (item 1577's move): ours spends a
    // `produce_building+0x1805` jitter draw later than the original, ours
    // 18 draws against 16 since item 1591's. Item 1578's build (the 1%) and
    // item 1591's (the silo's arm: the ICBM `1/96` ordered on 1951 and
    // launched on 1982 as the original's) bring the Persians' make list to
    // agree but two rows on 1984: `MAKE[5]` a Peasant (type 50) offer in
    // ours, val 7210, where the original's slot is empty, and the Spy's
    // `val` ours 1083333 theirs 1129432 (`check_income`'s `fac` 235 against
    // 245: the wealth price over income/16 one notch apart). **What parts
    // first past the standing block** is city `1/2018`'s `bordering` on
    // 1980 (ours 0, theirs 3).
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:MAKE[5].t".to_string()))
            .map(|(f, _)| *f),
        Some(1984),
        "the make list's extra Peasant parts on block 1984"
    );
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 96)),
        "the ICBM 1/96 agrees in every compared field"
    );
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1979)
        .min();
    pin_eq!(
        first,
        Some(1980),
        "the first parting past the standing block"
    );
}

/// run690, item 1577: the lobby's blocks 1521..1590 at the long's detail —
/// the gap between run681 and run683 where the Supply Wagon `1/67` was born
/// and parted (`docs/AI.md` §158).
pub(crate) const RUN690: &str = "gamelog-run690-greatsahara-persian-alltech-window-1521-1590.txt";

/// The window: block 1521 through 1590.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1521: (i64, i64) = (1521, 1590);

/// run690 walked from run675's start with the recorder on.
pub(crate) fn sahara_coverage_frame_1521_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run690",
        &[(RUN690, WIDENING_SAHARA_COVERAGE_FRAME_1521.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_1521,
        1,
        &[1563],
        true,
    )
}

#[test]
fn run690_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_1521_window() else {
        return;
    };
    pin_eq!(w.blocks, 70, "every captured block");
    // **126** on item 1578's tree: the Supply Wagon `1/67` parts on 1563
    // (item 1577's walk, `docs/AI.md` §158, now agreeing) and leader 1's
    // queue prices, which item 1578's 1% brought into line, stood here
    // before it.
    pin_eq!(w.firsts.len(), 126, "initial run690 baseline");
}
