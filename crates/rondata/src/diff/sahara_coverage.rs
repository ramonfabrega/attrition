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
use crate::testenv::{dump, install};

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
    // Item 1565's build (a gather enhancer is seated only in the city it is
    // placed for, `docs/AI.md` §156: `produce_building`'s
    // `get_town(cand) == near`) moved it again, 1582 → **1818**: ours 3215
    // draws against 3218 at index 3201, ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `PathFinder::calc_road_cost+0x46`.
    //
    // Item 1581's build (a restarted caravan road search consumes the reset
    // flag, `docs/AI.md` §157: slot 7's search was restarted every frame
    // from 1817 where the original resumes it) moved it again, 1818 →
    // **1830**: ours 3218 draws against 3219 at index 2, ours
    // `PathFinder::calc_road_cost+0x46`, theirs `Guy::set_anim+0x97a <
    // Unit::do_guard+0x7f4`.
    //
    // Merged with item 1586's build (an army's normalize puts its group's cap
    // back to its leader's speed, `docs/AI.md` §161 — the writer of the cap
    // reset item 1581 parked as 1582) the word moves again, 1830 → **1985**
    // by the draw sequence and 2048 by the count: ours 16 draws against 16 at
    // index 4, ours `Leader::produce_building+0x1805`, theirs
    // `Leader::make_stuff+0x63d`.
    //
    // Landing 3 of the race on item 1565 (`docs/AI.md` §158): the mechanisms
    // the merged tree's word 1985 stood on — a bomber's cap, a modern unit's
    // level in an all-tech game, the nuke count, the silo's own launch, the
    // Spy's discount and `reg_free_peasants` read unsigned — move it again,
    // 1985 → **2048** by the count and the sequence alike: ours 44 draws
    // against 45, index 29, ours `Guy::set_anim+0x97a < Unit::move_step+0x823`,
    // theirs `Unit::do_move+0xe84`.
    //
    // Landing 4 (`docs/AI.md` §159): an upgraded building weighs its basic
    // type's trainer bit in the danger map, and a Spy trains in half the
    // time under Tactics — 2048 → **2077** → **2118** (count and sequence):
    // ours 22 draws against 20, index 5, ours `Guy::set_anim+0x97a <
    // Unit::do_move+0x11cf`, theirs `Object::take_damage+0xe1`.
    //
    // Landing 5 (`docs/AI.md` §160): a Spy's hit points, sight and speed
    // climb with the Spy upgrades — 2118 → **2185** (count and sequence):
    // ours 30 draws against 32, index 12, ours `Leader::make_stuff+0x63d`,
    // theirs `Leader::produce_building+0x1805`.
    //
    // Landing 6 (`docs/AI.md` §167): a city a rival's border touches
    // (`CityData.bordering`) is worth ten times its tower — the Persians'
    // second ICBM is priced for Uranium and the spent nuke, and their
    // Bunker is offered at 80000 where ours' Library led — 2185 →
    // **2206** (count and sequence): ours 14 draws against 16, index 3, ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x97a
    // < Unit::move_step+0x823`.
    //
    // Landing 7 (`docs/AI.md` §171): the quick collision `find_merchant_spot`
    // asks (the Merchant `1/38`'s walk) and the founding capital's border
    // bonus (the territory table agrees on every dumped block) — 2206 →
    // **2273** (count and sequence): ours 99 draws against 91, index 66,
    // ours `Unit::think_scout+0x64c`, theirs `Unit::think_scout+0x436`; the
    // word's first parting by index was 0 (the scout `1/0`'s own idle).
    count: 2273,
    sequence: 2273,
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
    pin_eq!(w.firsts.len(), 47, "initial run677 baseline");
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
    pin_eq!(w.firsts.len(), 52, "initial run680 baseline");
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
    pin_eq!(w.firsts.len(), 63, "initial run681 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1191).count(),
        47,
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
        Some(1203),
        "the first parting past the standing block"
    );
}

/// run683, item 1561: the lobby's blocks 1577..1833 at the long's detail —
/// the word 1582's block 1583 with six before it and 250 after, and item
/// 1565's word 1818 inside it (block 1819).
pub(crate) const RUN683: &str = "gamelog-run683-greatsahara-persian-alltech-window-1577-1833.txt";

/// The window: block 1577 through 1833; the word 1582's own block is 1583.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1582: (i64, i64) = (1577, 1833);

/// run683 walked from run675's start with the recorder on — the word
/// 1582's block 1583, the word 1818's block 1819 and the word 1830's block
/// 1831. It was the `AI_WORDS` window until the word moved past it, to
/// run702's (item 1581, merged with item 1586's build).
pub(crate) fn sahara_coverage_frame_1582_window() -> Option<harness::tests::Widened> {
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
    let Some(w) = sahara_coverage_frame_1582_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **1183** on the tree at item 1561's build, **863** after item 1565's
    // (`docs/AI.md` §156): the Refinery's site and city, and what the
    // shifted stream moved after them, agree. **842** after item 1581's
    // (`docs/AI.md` §157): the Persian supply wagon `1/67`'s `myhits` (150,
    // the type's 90 and `SUPPLY_HP_UPGRADE[3]`'s 60 — three standing keys:
    // `myhits`, `hits_left` and `hits:myhits`) and the caravan search that
    // restarted each frame from 1817 (the other 18). Item 1586 (on a tree
    // without 1565's and 1581's): 1183 → 540, 159 → 151 standing — an army's
    // normalize puts its group's cap back to its leader's speed
    // (`docs/AI.md` §161); the merged tree's count is the union, measured:
    // **182**, 148 standing.
    pin_eq!(w.firsts.len(), 57, "initial run683 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1577).count(),
        48,
        "the keys standing on the window's first block"
    );
    // **The word 1582's value diff, block 1583** (items 1561 and 1565): the
    // Persians' `REFINERY` (`orig_type` 426) `1/2045`, bought for city 2
    // (`1/2006`, at (27744, 19296)) by a `MAKE` row both sides print alike,
    // stands at (30432, 19872) in the original — `city` 2, chained after
    // `1/2006` (`city_down` 2045) — and stood at (30432, 22176) in ours,
    // `city` 1, chained after `1/2041`: the spiral's candidate loop dropped
    // no site outside the city the Refinery was placed for, and the
    // original's `get_town(cand) == near` (`docs/AI.md` §156) drops them.
    // The original's four `produce_building+0x1805` jitter draws (a 2×2,
    // every sub-position unblocked) were three in ours. It agrees in every
    // compared field since the build, and so do the human's `0/5` and `1/65`
    // whose targets sat a place off the shifted stream.
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 2045)),
        "the Refinery 1/2045 agrees in every compared field"
    );
    // **What parts first past the standing block** is the word's own block.
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

/// **Every road search to 1829, node for node** (item 1581). run683's trace
/// proxies `valid_roadcoord` and `calc_road_cost` over the game, so each
/// search's priced nodes are comparable with ours from frame 0, as run483's
/// were at Toughest. It parted on 1818, at node 0: caravan slot 7's search
/// (`1/2000` to `1/2028`) restarted where the original resumed it, the
/// reset flag `Caravans::reset_paths` raised on 1817 never consumed
/// (`docs/AI.md` §157); the window from 1577 holds 23 more searches that
/// agree.
#[test]
fn run683_s_road_searches_hold_node_for_node() {
    let _pins = Pins::hold();
    let Some(inst) = install() else { return };
    let (Some(path), Some(sib), Some(t683)) = (
        dump(SAHARA_COVERAGE.long.0),
        dump(SAHARA_COVERAGE.start),
        trace("rontrace-run683.log"),
    ) else {
        eprintln!("skipping: no run676/run675/run683 (set RON_GAMELOG_DIR)");
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
    if let Some(t) = trace(SAHARA_COVERAGE.long.1) {
        borrow_pasture(&mut init, &t);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    built.sim.trace_costs = true;
    let mut parted = Vec::new();
    let mut searched = 0;
    while built.sim.frame < 1830 {
        let f = built.sim.frame;
        built.sim.road_marks.clear();
        built.tick();
        let ours = std::mem::take(&mut built.sim.road_marks);
        if f < 1577 {
            continue;
        }
        let theirs = t683.road_nodes(f);
        if ours.is_empty() && theirs.is_empty() {
            continue;
        }
        searched += 1;
        if let Some(i) = (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i)) {
            parted.push(format!(
                "{f} at {i}: ours {:?} theirs {:?} ({} against {})",
                ours.get(i),
                theirs.get(i),
                ours.len(),
                theirs.len()
            ));
        }
    }
    pin_eq!(searched, 35, "the road-search frames from 1577 to 1829");
    pin!(parted.is_empty(), "a road search parted: {parted:?}");
}

/// run702, item 1581: the lobby's blocks 1979..2235 at the long's detail —
/// the word 1985's block 1986 with six before it and 249 after, taken on the
/// tree that merged item 1586's army normalize.
pub(crate) const RUN702: &str = "gamelog-run702-greatsahara-persian-alltech-window-1979-2235.txt";

/// The window: block 1979 through 2235; the word 1985's own block is 1986.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_1985: (i64, i64) = (1979, 2235);

/// run702 walked from run675's start with the recorder on — the word 1985's
/// block 1986, the word 2048's and the word 2185's and 2206's blocks. It was
/// the `AI_WORDS` window until the word moved past it, to run718's (item
/// 1611).
pub(crate) fn sahara_coverage_frame_1985_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run702",
        &[(RUN702, WIDENING_SAHARA_COVERAGE_FRAME_1985.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_1985,
        1,
        &[2207],
        true,
    )
}

#[test]
fn run702_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_coverage_frame_1985_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // 259 since item 1608 compares `spell_time`: `1/38` 1 against 0 on
    // 2207, the word 2206's own block — ours has a cast's first frame
    // there that the original has not.
    // **116** on the merged tree: 258 (259 with item 1608's `spell_time`
    // row) before item 1611's two builds — the founding capital's border
    // bonus took 142 — and 51 of them stand on block 1979 (95 before).
    pin_eq!(w.firsts.len(), 116, "initial run702 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 1979).count(),
        51,
        "the keys standing on the window's first block"
    );
    // **The word 1985's value diff** (item 1581, on the tree that merged
    // item 1586), and what it became (item 1583, `docs/AI.md` §158). The
    // draws agreed to index 3 on 1985 and the fifth was where the original
    // rolled `Leader::make_stuff+0x63d` and ours the placement jitter
    // `Leader::produce_building+0x1805`; the make list's rows 1 and 2 had
    // parted on 1981 (the Supply Wagon against the MLRS). With the landing's
    // six mechanisms the make list agrees to the end of the window but for
    // the Persian army's march, and **the word is 2048**: unit `1/43`'s
    // path parts on **2046** (its heading, `pos`, a path of four nodes
    // against two) — the draw on 2048 is the move step it takes. What parts
    // first past the standing block is Persian city `1/2018`'s `bordering` on
    // **1980** (ours 0, theirs 3, the bit of each leader whose ground touches
    // the city's), and then, of the compared leader keys, only
    // `production_step` on 2001.
    pin_eq!(
        w.firsts
            .get(&(1, 2018, "city:bordering".to_string()))
            .map(|(f, _)| *f),
        None,
        "Persian city 1/2018's bordering agrees (item 1600: the border pass writes it)"
    );
    // Items 1584 and 1585 (`docs/AI.md` §159, §160): the unit `1/43` agrees
    // for the whole window now (the danger map weighs an upgraded trainer at
    // fifty, so its search is priced node for node), and the Spy `1/103` of
    // Persian city `1/2030`, born on 2077 (a Spy trains in half the time under
    // Tactics), agrees in every compared field (its hit points, sight and
    // speed climb with the Spy upgrades); the word is **2185**.
    for (o, key) in [
        (43, "pos"),
        (103, "myhits"),
        (103, "mylos"),
        (103, "myspeed"),
    ] {
        pin_eq!(
            w.firsts.get(&(1, o, key.to_string())).map(|(f, _)| *f),
            None,
            "a unit the earlier words moved agrees in the window"
        );
    }
    // **The word 2185's value diff** (item 1585): the draws agree to index 11
    // and the twelfth is where the original places a building
    // (`Leader::produce_building+0x1805`, 32 draws) where ours rolls
    // `Leader::make_stuff+0x63d` (30). The dump parts first on **2183** at
    // building `1/2034`'s queue (`queue[0].cost[0]` ours 816 theirs 927,
    // `cost[1]` 965 against 1069: the same entry, priced 12 % dearer in the
    // original), with the knowledge and oil buckets 111 and 104 apart; on
    // **2185** the make list's row 8 (`t` 435 against 442, `val` 39981 against
    // 80000 with `escrow` 1, `city` 5 against 1).
    for key in ["queue:queue[0].cost[0]", "queue:queue[0].cost[1]"] {
        pin_eq!(
            w.firsts.get(&(1, 2034, key.to_string())).map(|(f, _)| *f),
            None,
            "building 1/2034's queued entry agrees (item 1600: Uranium and the spent nuke)"
        );
    }
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:MAKE[8].t".to_string()))
            .map(|(f, _)| *f),
        None,
        "leader 1's make row 8 agrees (item 1600: the Bunker, bordering ×10)"
    );
    // **The word 2206's value diff** (item 1600, `docs/AI.md` §167): the
    // draws agree to index 2 on 2206 and the fourth is where ours idles
    // Persian unit `1/38` (`Unit::do_idle+0x7d`) and the original steps it
    // (`Unit::move_step+0x823`). The unit's order parts first, on **2205**:
    // its move's destination (`x` 40920 against 40728, `y` 1944 against
    // 1752, the offsets 216/408 against 24/216); on 2206 its order kind (14
    // against 1), its order count (1 against 2) and its path (0 nodes
    // against 1). **Item 1611's value diff** (`docs/AI.md` §171): the unit
    // is a Merchant standing on the rare `8/6`; `find_merchant_spot`'s ring
    // walk took tile (0, 0) here and the original's third test, the quick
    // `detect_unit_collision` at the tile's corner, refused it and took
    // (−1, −1) — the destination (40728, 1752) and the offsets 24/216. Every
    // key of `1/38` agrees on the window now.
    for key in ["order:move.x", "order:move.y", "orders_x", "orders_y"] {
        pin_eq!(
            w.firsts.get(&(1, 38, key.to_string())).map(|(f, _)| *f),
            None,
            "unit 1/38's move destination agrees (item 1611: the spot's quick collision)"
        );
    }
    for key in ["order:kind", "path:length", "heading"] {
        pin_eq!(
            w.firsts.get(&(1, 38, key.to_string())).map(|(f, _)| *f),
            None,
            "unit 1/38's order, path and heading agree (item 1611)"
        );
    }
    // **What parts first past the standing block.**
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1979)
        .min();
    pin_eq!(
        first,
        Some(1991),
        "the first parting past the standing block"
    );
}

/// run718, item 1611: the lobby's blocks 2267..2523 at the long's detail —
/// the word 2273's block 2274 with six before it and 249 after.
pub(crate) const RUN718: &str = "gamelog-run718-greatsahara-persian-alltech-window-2267-2524.txt";

/// The window: block 2267 through 2523; the word 2273's own block is 2274.
pub(crate) const WIDENING_SAHARA_COVERAGE_FRAME_2273: (i64, i64) = (2267, 2523);

/// **The coverage lobby's word's window** (`AI_WORDS`' `Third map` row for
/// `GreatSaharaPersianAllTech`): run718 walked from run675's start with the
/// recorder on — the word 2273's block 2274.
pub(crate) fn sahara_coverage_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_COVERAGE.start],
        true,
        SAHARA_COVERAGE.long,
        "run718",
        &[(RUN718, WIDENING_SAHARA_COVERAGE_FRAME_2273.0)],
        WIDENING_SAHARA_COVERAGE_FRAME_2273,
        1,
        &[2274],
        true,
    )
}

#[test]
fn run718_s_word_frame_is_widened_whole() {
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
    pin_eq!(w.firsts.len(), 737, "initial run718 baseline");
    pin_eq!(
        w.firsts.values().filter(|(f, _)| *f == 2267).count(),
        66,
        "the keys standing on the window's first block"
    );
    // **The word 2273's value diff** (item 1611, `docs/AI.md` §171). The
    // Persian scout `1/0` thinks on 2273 at (22, 34): the draws agree to
    // index 65 (the two rings of its own city, then Napata's three) and
    // the 66th is where ours rolls the cell jitter `Unit::think_scout+0x64c`
    // of a Napata ring cell the original has refused (99 draws against 91,
    // the same target — `1/0` agrees in every key). The refusal is the
    // fog's (`seen` and the cell not the Persians') and the fog's cause
    // stands in the dump from **2102**: the original's `treaties` between
    // the two leaders read 3 and every human building's `ever_seen` 255 from
    // 2105 (ours 1 and 1) — Napata was struck on 2101 (`city_flags`
    // `0x2|0x4|0x8`, `reduce_stamp` 2101 on both sides) — and by 2267 the
    // original's human holds no unit (`control` 0) where ours holds 14.
    for key in ["build:ever_seen", "build:ever_seen_completed"] {
        pin_eq!(
            w.firsts.get(&(0, 2000, key.to_string())).map(|(f, _)| *f),
            Some(2267),
            "Napata's buildings stand at 255 in the original and 1 here"
        );
    }
    pin_eq!(
        w.firsts
            .get(&(0, -1, "leader:treaties[1]".to_string()))
            .map(|(f, _)| *f),
        Some(2267),
        "the human's treaty bits stand at 3 in the original and 1 here"
    );
    pin_eq!(
        w.firsts
            .get(&(0, -1, "leader:control".to_string()))
            .map(|(f, _)| *f),
        Some(2267),
        "the human's units are all dead in the original on the window's first block"
    );
    pin_eq!(
        w.firsts
            .get(&(1, 0, "order:move.x".to_string()))
            .map(|(f, _)| *f),
        None,
        "the scout `1/0`'s target agrees (item 1611: the territory table)"
    );
    // **What parts first past the standing block**: the order lists of the
    // Persian units `1/18`, `1/34`, `1/58`, `1/81`, `1/97` and `1/116` (kind
    // 17 against 16, one order against two; the first on 2271) — the item
    // after's, with the fog's nuke aftermath above.
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 2267)
        .min();
    pin_eq!(
        first,
        Some(2271),
        "the first parting past the standing block"
    );
}
