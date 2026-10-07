//! **The coverage pair** (DECISIONS 61 §6, item 1466): the lobby chosen for
//! the rows no trace has entered, not for adjacency. East Indies
//! (`MAP_STYLE 18`, the sea map), human Nubians (4) against **Persians
//! (23)** at Toughest, seed 12345, with the two settings that open every
//! age's content at frame 0: `STARTING_TECHNOLOGY 8` ("All Technologies")
//! and `STARTING_RESOURCES 7` ("Deathmatch") — the lobby `Leader::
//! research_techs`' own hard-coded branch tests for. `docs/AI.md` §127 has
//! the choice, the rows it should enter, and what the closing state of the
//! game says it did.
//!
//! The captures are run651 (a `DUMP_ALL` start, the lobby's own sibling) and
//! run652 (the draw stream to the game's end at 4730, `cover=0`); run650 is
//! the short capture that showed the click-free lane carries a numbered late
//! age (`STARTING_TECHNOLOGY 5`). The pair's first parting is
//! [`COVERAGE`]'s `count`/`sequence`, pinned here and not in `FLOORS`: the
//! handoff's scoreboard is the commander's to extend.

use super::testkit::*;
use super::*;
use crate::testenv::dump;

/// The coverage pair's lobby and captures.
pub(crate) struct Pair {
    pub name: &'static str,
    pub map_style: i32,
    pub ai_tribe: i32,
    pub difficulty: i32,
    /// `STARTING_TECHNOLOGY`: 8 is "All Technologies".
    pub starting_technology: i32,
    /// `STARTING_RESOURCES`: 7 is the "Deathmatch" row.
    pub starting_resources: i32,
    pub start: &'static str,
    pub long: (&'static str, &'static str),
    /// The trace's last frame: the frame the game ended on.
    pub length: i64,
    /// The first parting, first try: the frame the draw count parts on and
    /// the frame the draw sequence parts on. **Item 1496 moved it from frame
    /// 0 (195 draws ours against 198, index 26) to frame 8 (21 against 24,
    /// index 2: ours `Leader::make_stuff+0x63d`, theirs `Leader::
    /// produce_building+0x1805`)** — the original's Oil Well on the AI's
    /// make list, which this crate never placed (`docs/AI.md` §129).
    /// **Item 1505 moved it from frame 8 to frame 177 (9 draws ours
    /// against 8, index 2: ours `Leader::make_stuff+0x221`, theirs `Guy::
    /// set_anim+0x97a < Guy::inc_time+0x271`)**: the leader's oil patches
    /// are carried now (`docs/AI.md` §133), and 177's `make_me` fills
    /// three Village slots in ours against two in theirs.
    /// **Item 1511 moved it from frame 177 to 185 (the sequence; the count
    /// parts on 377)**: the All Technologies start owns Electronics, and
    /// `WorldData::was_seen`'s leader arm answers every cell seen for it,
    /// so leader 1's site list agrees from block 1 (`docs/AI.md` §141). At
    /// 185, 22 draws a side, index 8: ours `Leader::produce_building+0x1805`,
    /// theirs `Leader::make_stuff+0x63d`.
    /// **Item 1530 moved it from 185 to 583 (count and sequence)**:
    /// `find_friends`' trainer arm reads the derived flag, so the second
    /// Missile Silo `1/2021` stands where the original puts it on frame
    /// 182 (`docs/AI.md` §143). At 583, ours 198 game draws against 5,
    /// index 0: ours `PathFinder::calc_road_cost+0x46`, theirs `Farms::
    /// inc_time+0x1ae`.
    /// **Item 1532 moved it from 583 to 667 (count and sequence)**: a
    /// builder already holding a `BUILD_AT` is sent the new site through
    /// `action_swarm_around`'s `QUEUE_LAST` arm, an `EXPLORE_TO` and then
    /// the build, so `1/9` starts the Barracks `1/2022` on 274 as the
    /// original does, the Barracks finishes on 567, and its re-flag of the
    /// city's roads no longer replans the University `1/2009` on 583
    /// (`docs/AI.md` §144). At 667, ours 152 game draws against 155,
    /// index 143: ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    /// `Guy::init_real+0x52 < Unit::init` — a three-figure unit, `1/15`,
    /// born in theirs.
    /// **Item 1539 moved it from 667 to 727 (count and sequence)**: the
    /// Persians' Market trains a caravan on its own fifteen-frame slot
    /// (`Build::process@0061edf0:362–381`), so the Market `1/2018`,
    /// finished on 661, trains `1/15` on 667 as the original does
    /// (`docs/AI.md` §146). At 727, ours 8 game draws against 7, index 0:
    /// ours `Guy::set_anim+0x97a < Unit::move_step+0x823`, theirs `Guy::
    /// set_anim+0x97a < Guy::inc_time+0x271`.
    /// **Item 1544 moved it from 727 to 982 (count and sequence)**:
    /// `game->total_units` counts owners below nine, so `find_build_spot`'s
    /// builder tally walks the lists while the players' and the animals'
    /// units are under `circle_radius[6]`, and `1/8` goes to the Bunker
    /// `1/2020` on 690 as the original sends it (`docs/AI.md` §148). At
    /// 982, ours 562 game draws against 696, index 13: ours `Leader::
    /// produce_building+0xc99`, theirs `Leader::produce_building+0x1805`.
    /// **Item 1546 moved it from 982 to 1183 (count and sequence)**: a
    /// Persian's second city is a second capital (`Build::activate
    /// @00623e20`'s tribe-bonus-`0x17` arm, `city_flags |= 0x10`), so
    /// `1/2006`'s gather base is halved and its Mine offer no longer wraps
    /// negative: leader 1 lists the Mine (419, 6,300,000) on 981 as the
    /// original does (`docs/AI.md` §149). At 1183, ours 29 game draws
    /// against 28, index 21: ours `Guy::set_anim+0x97a < Guy::move+0x19f`,
    /// theirs `Guy::set_anim+0x97a < Guy::inc_time+0x271`.
    /// **Item 1552 moved it from 1183 to 1277 (count and sequence)**: a
    /// citizen swarming a water building is searched as the leader's
    /// barge once the leader transports civilians (`Group::
    /// action_swarm_around@0070fbe0`, `00710238`–`007102cb`), so `1/26`,
    /// sent to the Oil Platform `1/2031` at sea on 1182, takes the ring's
    /// first water point (42936, 34104) as the original does
    /// (`docs/AI.md` §151). At 1277, ours 10 game draws against 15,
    /// index 2: ours `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
    /// `Unit::explore_goody+0x27c`.
    /// **Item 1558 moved it from 1277 to 1408 (count and sequence)**: a goody
    /// item's `is_seen` falls through to `WorldData::is_seen`, whose
    /// territory arm sees a leader's own ground once it holds Computerization
    /// (`leader_flags & 0x2000`), so the scout `1/0` re-aims for the box at
    /// (38, 34) on frame 1200 as the original does (`docs/AI.md` §153). At
    /// 1408, ours 39 game draws against 40, index 25: ours
    /// `Objects::process_all+0x2df`, theirs `Guy::set_anim+0x97a <
    /// Unit::do_cast+0xc89`, then `Guy::init_real < Unit::init <
    /// Objects::init_unit`.
    /// **Item 1563 moved it from 1408 to 1532 (count and sequence)**:
    /// `find_build_spot`'s builder tally on the circle walk tests each
    /// unit's own tile region against the searcher's (`Objects::find_units
    /// @0065a620`), so the barge-borne builder `1/24`, on a coastal cell's
    /// water, is not counted on the Mine `1/2028`, and `1/8` goes there on
    /// 1340 as the original sends it (`docs/AI.md` §155). At 1532, ours 13
    /// game draws against 11, index 8: ours `Unit::do_move+0xe84`, theirs
    /// `Farms::inc_time+0x1ae` — theirs four `Unit::do_move+0xe84 <
    /// Unit::do_attack_to` draws, ours six.
    /// **Item 1586 moved it from 1532 to 1610 (count and sequence)**:
    /// `Army::normalize` runs `Group::normalize` on the army's group, whose
    /// tail puts the cap back to its leader's own speed, so army `1/0`'s
    /// periodic normalize on 1434 lifts the slow squad's 25 to `1/28`'s 47
    /// as the original does (`docs/AI.md` §161). At 1610, ours 589 game
    /// draws against 598, index 0: ours `Guy::set_anim+0x97a <
    /// Guy::move+0x19f`, theirs `Guy::set_anim+0x97a < Unit::move_step+0x823`
    /// — before 586 `PathFinder::calc_road_cost` draws in the original.
    /// **Item 1588 moved it from 1610 to 1696 (count and sequence)**:
    /// `Wall::process`'s oil-platform arm disbands a computer leader's
    /// unfinished Oil Platform that no unit of its owner holds as its
    /// action, every 128 frames by `o`, so `1/2037` goes on 1419 as the
    /// original removes it and the citizen `1/10` no longer walks to it
    /// (`docs/AI.md` §162). At 1696, ours 41 game draws against 40, index
    /// 29: ours `Unit::do_move+0xe84` (`1/52`), theirs
    /// `Objects::process_all+0x2df`.
    /// **Item 1589 moved it from 1696 to 1719 (count and sequence)**:
    /// `PathFinder::find_tpath`'s pull-back walk runs only for a unit that
    /// is not an aircraft and cannot transport, as `find_upath`'s does, so
    /// the Freighter `1/52` keeps its tile goal across the regions on 1685
    /// and searches to it as the original does, where ours walked the goal
    /// home, refused the plan and spent a second grid draw on 1696
    /// (`docs/AI.md` §163). At 1719, ours 15 game draws against 16, index
    /// 11: ours `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`.
    /// **Item 1579 moved it from 1719 to 1735 (count and sequence)**: a
    /// computer's Missile Silo, on its 128-frame turn, takes the missile at
    /// the head of its inside chain and lays an air strike on the enemy
    /// city it values most (`Object::do_launch`'s silo arm, `docs/AI.md`
    /// §160), so the Persians' missile `1/42` is ordered on 1570 and
    /// strikes the human's capital `0/2000` on 1719 as the original's does
    /// (its `take_damage` draw; block 1720's `build:damage` 1560 and
    /// `reduce_stamp` 1719 agree). At 1735, ours 6 game draws against 5,
    /// index 0: ours `Unit::close+0xcb6`, theirs `Guy::set_anim+0x97a <
    /// Guy::inc_time+0x271`.
    pub count: i64,
    pub sequence: i64,
}

pub(crate) const COVERAGE: Pair = Pair {
    name: "EastIndiesPersianAllTech",
    map_style: 18,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run651-eastindies-persian-alltech-start.txt",
    long: (
        "gamelog-run652-eastindies-persian-alltech-24k-trace.txt",
        "rontrace-run652.log",
    ),
    length: 4730,
    count: 1735,
    sequence: 1735,
};

/// The pair's word as the handoff's `Coverage pair:` line and `AI_WORDS`
/// carry it: the lower of the two partings.
pub(crate) const COVERAGE_PAIR_WORD: i64 = if COVERAGE.count < COVERAGE.sequence {
    COVERAGE.count
} else {
    COVERAGE.sequence
};

/// **The pair's first parting**, walked from its own start: the frame the
/// draw count first parts on, the frame the draw sequence first parts on,
/// and the lobby read back from the dump.
#[test]
fn coverage_pair_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(COVERAGE.start, COVERAGE.long) else {
        return;
    };
    eprintln!(
        "COVERAGE {}: count {} sequence {} of {}; {}",
        COVERAGE.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, COVERAGE.difficulty, "the lobby read back");
    pin_eq!(w.last, COVERAGE.length, "the game's last frame");
    pin_eq!(w.count, COVERAGE.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        COVERAGE.sequence,
        "the draw sequence's first parting"
    );
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — map, difficulty, the two settings the pair exists for, nations
/// and seed — and the long is scoreable. A lane that stopped carrying
/// `STARTING_TECHNOLOGY` would read back 0 here.
#[test]
fn coverage_pair_captures_say_their_lobby() {
    for name in [COVERAGE.long.0, COVERAGE.start] {
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
        assert_eq!(info("MAP_STYLE"), Some(COVERAGE.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(COVERAGE.difficulty), "{name}");
        assert_eq!(
            info("STARTING_TECHNOLOGY"),
            Some(COVERAGE.starting_technology),
            "{name}"
        );
        assert_eq!(
            info("STARTING_RESOURCES"),
            Some(COVERAGE.starting_resources),
            "{name}"
        );
        assert_eq!(info("GAME_RULES"), Some(1), "{name}");
        let tribes: Vec<_> = init
            .leaders
            .iter()
            .filter(|l| l.who < 2)
            .map(|l| (l.who, l.tribe))
            .collect();
        assert_eq!(
            tribes,
            [(0, 4), (1, i64::from(COVERAGE.ai_tribe))],
            "{name}"
        );
    }
    let Some(path) = dump(COVERAGE.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the coverage long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the pair exists for what no other lobby enters.
#[test]
fn coverage_pair_is_no_other_lobby() {
    let triple = (COVERAGE.map_style, COVERAGE.ai_tribe, COVERAGE.difficulty);
    // `SCORED_LOBBIES` carries the pair's own triple, so a battery lobby can
    // never be moved onto it, and the battery holds none of it.
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// **The lobby's age reaches the tech tree.** Until item 1466 no lobby on
/// file started anywhere but Ancient, and `Sim::setup` was never fed from
/// the dump's `GAME INFO`; run651 is the first. A leader built from its
/// start owns every age (`Setup::starting_age` 8 is "All Technologies",
/// `LeaderData::starting_age` clamps it to 7) and a tech a plain age start
/// would not (`docs/TECH.md`, "The starting position"). Without the three
/// lines in `Sim::sync_setup_from_lobby` it owns none.
#[test]
fn coverage_pair_start_owns_every_age() {
    let (Some(inst), Some(path)) = (crate::testenv::install(), dump(COVERAGE.start)) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = log.initial().expect("a start dump");
    let built = build_sim(&loaded, &init, Tuning::RON);
    assert_eq!(built.sim.lobby.starting_technology, 8);
    assert_eq!(built.sim.setup.starting_age, 8);
    for who in 0..2 {
        let t = &built.sim.tech[who];
        assert_eq!(t.ages, 7, "who {who} owns every age at frame 0");
        assert!(
            t.discovered > 0,
            "who {who}: all_techs grants the plain techs"
        );
    }
}

/// **The lobby's three settings reach the start** (item 1496, `docs/AI.md`
/// §129), each read off run651's own block 1 and none of them a Standard
/// start's: the `startingresources` row (`Deathmatch`, `100`/`100`) that
/// prices both leaders' stockpiles, the hit points a citizen is born with
/// under an all-technology start (85, the Militia line's, against the
/// type's 40), and `Setup::build_units`' steps 3 and 4 for the citizens
/// past the farm list — `starting_resources == 7` adds eight, and of the
/// thirteen two stand at the woodcutter, three at the farms, **four more at
/// the woodcutter until it holds its six**, and four are placed idle.
#[test]
fn coverage_pair_start_pays_its_lobby() {
    let (Some(inst), Some(path)) = (crate::testenv::install(), dump(COVERAGE.start)) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    assert_eq!(loaded.starting_resources.get(1), Some(&(1, 1)), "Standard");
    assert_eq!(
        loaded.starting_resources.get(7),
        Some(&(100, 100)),
        "Deathmatch"
    );
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = log.initial().expect("a start dump");
    let built = build_sim(&loaded, &init, Tuning::RON);
    assert_eq!(built.sim.lobby.starting_resources_row, (100, 100));
    // Nubians' food is the table's own; the Persians' is half as much again
    // (`PERSIANS_BONUS_FOOD`, `Leader::init`).
    assert_eq!(
        built.sim.ledgers[0].bucket,
        [20000, 20000, 20000, 10000, 20000, 20000]
    );
    assert_eq!(
        built.sim.ledgers[1].bucket,
        [30000, 20000, 20000, 10000, 20000, 20000]
    );
    for who in 0..2u8 {
        for o in 1..=13i16 {
            let u = built.sim.unit_by_o(who, o).expect("a starting citizen");
            let un = &built.sim.units[u];
            assert_eq!(
                un.max_health, 85,
                "who {who} citizen {o}: the Militia line's hits"
            );
            // Citizens 1..=9 are ordered (two, three, four at the woodcutter
            // and farms); the last four are `place_unit`'s idle ones.
            assert_eq!(
                !un.orders.is_empty(),
                o <= 9,
                "who {who} citizen {o}: a gather order only while the woodcutter has room"
            );
        }
    }
}

/// run651's own blocks, 0 and 1: the start dump's whole records, which is
/// where frame 0's word lives (`docs/AI.md` §127).
pub(crate) const WIDENING_COVERAGE_START: (i64, i64) = (0, 1);

/// The pair's first word's widening: every dumped record on block 1 (the
/// state frame 0 writes), the start dump itself as the capture.
pub(crate) fn coverage_start_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run651",
        &[(COVERAGE.start, 0)],
        WIDENING_COVERAGE_START,
        1,
        &[1],
        true,
    )
}

#[test]
fn run651_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_start_window() else {
        return;
    };
    pin_eq!(
        w.blocks,
        1,
        "every captured block: the start dump's block 1"
    );
    pin!(w.missing.is_empty(), "every key is read: {:?}", w.missing);
    // Item 1511 (`WorldData::was_seen`'s leader arm, `docs/AI.md` §141):
    // **66**, the SITE list's 21 gone — the control's 48 and 18 more.
    // Item 1496 (the lobby's stockpile row, the all-technology start's
    // hit points and §9.3's woodcutter fill): **87**, of which 48 are the
    // control's. Item 1466, on the tree at base 4b29c0d7 plus the lobby's age: **226**
    // keys part on block 1, against **48** for the scored French start's
    // own block 1 (run595, the control: its `form`, the leaders' `SITE`
    // `reg`s and city terrain, which every lobby parts on). The 178 the
    // lobby adds: both leaders' six stockpile buckets (ours 200 / 200 /
    // 100 / 100 / 100 / 100, theirs 20000 / 20000 / 20000 / 10000 / 20000
    // / 20000 — `STARTING_RESOURCES 7`), 26 who=1 units' `myhits`,
    // `hits_left` and `hits:myhits` (ours 40, theirs 85), their gather
    // order's `been_there`/`wait` and `idle`, and a move order and its
    // path. The word's own draw delta is §127's.
    pin_eq!(w.firsts.len(), 66, "initial run651 baseline");
}

/// run656, item 1496: the lobby's first 33 blocks at the long's detail
/// (`LEADERS=9`, `BUILDS=7`, `UNITS=3`), so frame 8's word has records.
pub(crate) const RUN656: &str = "gamelog-run656-eastindies-persian-alltech-window-1-33.txt";

/// The window: block 1 through 33; the word's own block is 9.
pub(crate) const WIDENING_COVERAGE_FRAME_8: (i64, i64) = (1, 33);

pub(crate) fn coverage_frame_8_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run656",
        &[(RUN656, 1)],
        WIDENING_COVERAGE_FRAME_8,
        1,
        &[9],
        true,
    )
}

#[test]
fn run656_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_8_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every captured block");
    // The capture's `GUYS=2` prints two gaia animation keys nothing reads
    // (item 1496): both are the animals' clocks, which agree on every draw
    // the walk compares.
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // 364 before item 1505 (the oil patches): block 9's 275 keys are gone and the
    // 89 left are block 1's (the control's 48 and the SITE list's). 68 since
    // item 1511: the SITE list agrees (`docs/AI.md` §141).
    pin_eq!(w.firsts.len(), 68, "initial run656 baseline");
}

/// run660, item 1505: the lobby's blocks 171..184 at the long's detail, so
/// frame 177's word has records (the make list, the buildings, the leader).
pub(crate) const RUN660: &str = "gamelog-run660-eastindies-persian-alltech-window-171-184.txt";

/// The window: block 171 through 184; the word's own block is 178.
pub(crate) const WIDENING_COVERAGE_FRAME_177: (i64, i64) = (171, 184);

pub(crate) fn coverage_frame_177_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run660",
        &[(RUN660, 171)],
        WIDENING_COVERAGE_FRAME_177,
        1,
        &[178],
        true,
    )
}

#[test]
fn run660_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_177_window() else {
        return;
    };
    pin_eq!(w.blocks, 14, "every captured block");
    // Block 171's standing rows (90: the control's and the SITE list's, parted since
    // block 1), the SITE ranks from 176 and the make list from 183: 132 before
    // item 1511. **97** since: the SITE list agrees on every block and frame
    // 177's three Village slots are two; what stands is the control's.
    // **74** since item 1530: the Silo `1/2021` placed on frame 182 stands
    // where the original puts it, and block 183's make list with it.
    // **71** since item 1532: `1/10`'s order stack agrees on 183.
    pin_eq!(w.firsts.len(), 71, "initial run660 baseline");
}

/// run669, item 1511: the lobby's blocks 180..436 at the long's detail —
/// the word 185's block 186 with six before it and 250 after.
pub(crate) const RUN669: &str = "gamelog-run669-eastindies-persian-alltech-window-180-436.txt";

/// The window: block 180 through 436; the word's own block is 186.
pub(crate) const WIDENING_COVERAGE_FRAME_185: (i64, i64) = (180, 436);

/// run669 walked from run651's start with the recorder on: the newest
/// pair's word's window from item 1511 to item 1532, which moved the word
/// to run672's ([`coverage_frame_583_window`]).
pub(crate) fn coverage_frame_185_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run669",
        &[(RUN669, 180)],
        WIDENING_COVERAGE_FRAME_185,
        1,
        &[186],
        true,
    )
}

#[test]
fn run669_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_185_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // 192 before item 1530; 100 since: `find_friends`' trainer arm
    // reads the derived flag, and the second Missile Silo agrees.
    // **73** since item 1532 — the control's standing set on block 180,
    // leader 0's `production_step` on 201 and a blank `SITE` slot's `reg`
    // on 376, nothing else: a busy builder's next build takes its own
    // approach.
    pin_eq!(w.firsts.len(), 73, "initial run669 baseline");
    // **The move's value diff, block 183** (item 1530, `docs/AI.md` §143):
    // the Silo `1/2021` stood at (36480, 41088) in ours against (41088,
    // 37248) and agrees in every field now; the three buildings placed on
    // 185 stand where the original's do (`1/2022`'s `job_counter` parts on
    // 275, downstream).
    assert!(
        w.firsts.keys().all(|(who, b, _)| (*who, *b) != (1, 2021)),
        "1/2021 agrees in every compared field"
    );
    for o in [2022, 2023, 2024] {
        assert!(
            w.firsts
                .keys()
                .all(|(who, b, f)| (*who, *b) != (1, o) || !f.ends_with("_internal")),
            "1/{o} stands where the original's does"
        );
    }
    assert!(
        w.firsts
            .keys()
            .all(|(who, _, field)| *who != 1 || !field.starts_with("leader:MAKE")),
        "leader 1's make list agrees on every block"
    );
    // **The move's value diff, item 1532** (`docs/AI.md` §144): `1/10`'s
    // order stack, four orders in ours against six on block 183, and
    // `1/2`, `1/4`, `1/9`'s on 186 (3/4, 3/4, 2/3), agree on every block;
    // `1/9` walks to the Barracks `1/2022`'s ring and starts it on block
    // 275 (`job_counter` 0 against 100 before).
    for o in [2, 4, 9, 10] {
        assert!(
            w.firsts
                .keys()
                .all(|(who, u, f)| (*who, *u) != (1, o) || !f.starts_with("order")),
            "1/{o}'s order stack agrees on every block"
        );
    }
    assert!(
        w.firsts.keys().all(|(who, b, _)| (*who, *b) != (1, 2022)),
        "1/2022 agrees in every compared field"
    );
}

/// run672, item 1532: the lobby's blocks 577..833 at the long's detail —
/// the word 583's block 584 with six before it and 250 after.
pub(crate) const RUN672: &str = "gamelog-run672-eastindies-persian-alltech-window-577-833.txt";

/// The window: block 577 through 833; the word 583's own block is 584.
pub(crate) const WIDENING_COVERAGE_FRAME_583: (i64, i64) = (577, 833);

/// run672 walked from run651's start with the recorder on: the word 583
/// (item 1532), block 584, the word 667 (item 1532), block 668, and the
/// word 727 (item 1539), block 728. It was the `AI_WORDS` window until
/// item 1544 moved the word past it, to run678's.
pub(crate) fn coverage_frame_583_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run672",
        &[(RUN672, 577)],
        WIDENING_COVERAGE_FRAME_583,
        1,
        &[584, 668, 728],
        true,
    )
}

#[test]
fn run672_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_583_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **107** on the tree item 1546 landed (109 on item 1544's, 247 on
    // item 1539's, 355 on item 1532's): `1/2006`'s capital bit, standing
    // from its founding on 612, agrees since item 1546. Block
    // 577's stand from the window's first block (the control's set, as on
    // run669's 180: the blank `SITE` slots' `reg`, `form`, the pools,
    // `scouts`).
    // Item 1578: 107 → 83, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 83, "initial run672 baseline");
    // **The move's value diff, item 1546** (`docs/AI.md` §149): the
    // Persians' second city `1/2006` carries `city_flags & 0x10` from its
    // founding on block 612 in both.
    pin_eq!(
        w.firsts
            .get(&(1, 2006, "city:city_flags[0x10]".to_string()))
            .map(|(f, _)| *f),
        None,
        "1/2006 is a capital in both"
    );
    // **The move's value diff** (`docs/AI.md` §144): the University
    // `1/2009` replanned its road on 583 in ours, flagged by the Barracks
    // `1/2022`'s activation on 568 where the original's came on 567, a
    // frame before the University's slot; it agrees now in every field
    // through the window's last block (its flag parted again on 779 until
    // item 1544 sent `1/8` to the Bunker).
    let university = w
        .firsts
        .iter()
        .filter(|((who, b, _), _)| (*who, *b) == (1, 2009))
        .map(|(_, (f, _))| *f)
        .min();
    pin_eq!(
        university,
        None,
        "1/2009 agrees in every compared field through the window"
    );
    // **What parts first past the standing block**: leader 1's make list
    // on block 581 (`MAKE[2].num`, `MAKE[3].num` 5 against 10), then the
    // queue prices and buckets on 583; and on **the word's block, 668**,
    // the original holds `1/15`, the three-figure unit `Guy::init_real`
    // made on 667, and ours does not.
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 577)
        .min();
    pin_eq!(
        first,
        Some(581),
        "the first parting past the standing block"
    );
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:MAKE[2].num".to_string()))
            .map(|(f, _)| *f),
        Some(581),
        "leader 1's make list parts on block 581"
    );
    // **The move's value diff** (item 1539, `docs/AI.md` §146): on 668
    // the original held the caravan `1/15` alone and leader 1's
    // `num_units[9]`, `caras`, `active` and `control` one above ours. The
    // Persians' Market `1/2018` trains it on 667 in ours too, and it agrees
    // in every compared field through the window's last block.
    pin_eq!(
        w.firsts
            .keys()
            .filter(|(who, o, _)| (*who, *o) == (1, 15))
            .count(),
        0,
        "1/15 agrees in every compared field"
    );
    for key in [
        "leader:caras",
        "leader:num_units[9]",
        "leader:active",
        "leader:control",
    ] {
        let at = w.firsts.get(&(1, -1, key.to_string())).map(|(f, _)| *f);
        pin!(
            at.is_none_or(|f| f > 728),
            "leader 1's {key} agrees through the word 727's block"
        );
    }
    // **The move's value diff** (item 1544, `docs/AI.md` §148): the word
    // 727's block, 728, had ours' citizen `1/8` (`PEASANTS`) blocked by
    // `1/6`, its approach (42072, 41928) against the original's (41160,
    // 40584) from block 691 — `find_build_spot` on 690 sent it to the
    // Lumber Mill `1/2012` in ours and the Bunker `1/2020` in the original.
    // Ours' builder tally walked the cell circle on 153 live units where
    // `total_units`, owners below nine, is 134 and walks the lists. It
    // agrees in every compared field through the window's last block.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| (*who, *o) == (1, 8) && *f > 577)
            .count(),
        0,
        "1/8 agrees in every compared field past its standing `form`"
    );
}

/// run678, item 1544: the lobby's blocks 977..1233 at the long's detail —
/// the word 982's block 983 with six before it and 250 after.
pub(crate) const RUN678: &str = "gamelog-run678-eastindies-persian-alltech-window-977-1233.txt";

/// The window: block 977 through 1233; the word 982's own block is 983.
pub(crate) const WIDENING_COVERAGE_FRAME_982: (i64, i64) = (977, 1233);

/// run678 walked from run651's start with the recorder on: the word 982
/// (item 1544), block 983, and the word 1183 (item 1546), block 1184. It
/// was the `AI_WORDS` window until item 1552 moved the word past it, to
/// run679's.
pub(crate) fn coverage_frame_982_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run678",
        &[(RUN678, 977)],
        WIDENING_COVERAGE_FRAME_982,
        1,
        &[983, 1184],
        true,
    )
}

#[test]
fn run678_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_982_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // **139** on the tree item 1558 landed (160 on item 1552's, 307 on item
    // 1546's, 1,378 on item 1544's); 98 stand from the window's first block
    // — `1/2006`'s capital bit no longer among them.
    // Item 1578: 139 → 95, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 95, "initial run678 baseline");
    // **The move's value diff, item 1546** (`docs/AI.md` §149): leader 1's
    // make list on 982 holds the Mine (419, 6,300,000) in `MAKE[3]`/`[4]`
    // in both — ours held the Farm (417, 2,520,000) — and the site
    // `1/2025` stands where the original's does; the list parts again on
    // 1186. `1/2006`'s capital bit (standing from 977) agrees.
    pin_eq!(
        w.firsts
            .get(&(1, 2006, "city:city_flags[0x10]".to_string()))
            .map(|(f, _)| *f),
        None,
        "1/2006 is a capital in both"
    );
    // **What parts first past the standing block**: the citizen `1/24`'s
    // `form` on 979, then the queue prices on 983 a unit under (`1/2022`,
    // `1/2024`: the bucket residue standing from 977).
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 977)
        .min();
    pin_eq!(
        first,
        Some(979),
        "the first parting past the standing block"
    );
    pin_eq!(
        w.firsts
            .get(&(1, -1, "leader:MAKE[3].t".to_string()))
            .map(|(f, _)| *f),
        None,
        "leader 1's make list agrees through the window (1186 before item 1552)"
    );
    pin_eq!(
        w.firsts
            .get(&(1, 2025, "build:x_internal".to_string()))
            .map(|(f, _)| *f),
        None,
        "the site 1/2025 stands where the original's does"
    );
    // **The move's value diff, item 1552** (`docs/AI.md` §151): the word
    // 1183's citizen `1/26` (`TypeIndex` 50, `PEASANTS`) held one order on
    // block 1183, the bare `Build` of the Oil Platform `1/2031` (kind 6, at
    // its own point (36478, 34118)), against the original's two — the
    // `ExploreTo` to (42936, 34104) with a path of 8, then the build — and
    // on 1184 stood at x 36478 against 36502. The swarm ring is searched
    // as the leader's barge, takes the water, and `1/26` agrees in every
    // compared field through the window's last block.
    pin_eq!(
        w.firsts
            .keys()
            .filter(|(who, o, _)| (*who, *o) == (1, 26))
            .count(),
        0,
        "1/26 agrees in every compared field"
    );
    // **The move's value diff, item 1558** (`docs/AI.md` §153): the scout
    // `1/0` (`TypeIndex` 77, `ELITESPECIALFORCES`) walked to (24312, 26616)
    // on both sides through block 1200, and on 1201 the original's was
    // bound for the goody cell (38, 34)'s centre, (29592, 26520), with a
    // path of 3, where ours kept (24312, 26616) and its path of 12. The box
    // is on leader 1's ground and its item's `is_seen` falls through to
    // `WorldData::is_seen`'s territory arm (Computerization held), so ours
    // re-aims on 1200 too, and `1/0` agrees through the window's last block.
    pin_eq!(
        w.firsts
            .keys()
            .filter(|(who, o, _)| (*who, *o) == (1, 0))
            .count(),
        0,
        "the scout 1/0 turns for the goody on block 1201 in both"
    );
}

/// run679, item 1552: the lobby's blocks 1272..1528 at the long's detail —
/// the word 1277's block 1278 with six before it and 250 after.
pub(crate) const RUN679: &str = "gamelog-run679-eastindies-persian-alltech-window-1272-1528.txt";

/// The window: block 1272 through 1528; the word 1277's own block is 1278.
pub(crate) const WIDENING_COVERAGE_FRAME_1277: (i64, i64) = (1272, 1528);

/// run679 walked from run651's start with the recorder on. It holds the
/// word 1277 (item 1552), block 1278, and the word 1408 (item 1558), block
/// 1409; it was the `AI_WORDS` window until item 1563 moved the word past
/// it, to run710's.
pub(crate) fn coverage_frame_1277_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run679",
        &[(RUN679, 1272)],
        WIDENING_COVERAGE_FRAME_1277,
        1,
        &[1278, 1409],
        true,
    )
}

#[test]
fn run679_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_frame_1277_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    // **144** on the tree item 1588 landed (146 on item 1586's, 203 on
    // item 1563's, 457 on item 1558's, 608 on item 1552's); 127 stand from
    // the window's first block. Item 1588's two: the Oil Platform site
    // `1/2037`, held by ours alone from 1420 — the original disbands it on
    // frame 1419, its 128-frame phase (1419 + 2037 = 27 × 128), with no
    // unit of leader 1's holding it as its action (`docs/AI.md` §162) —
    // and leader 1's `gather_stamp`, 1391 in ours against 1423 on 1424.
    // Item 1578: 144 → 102, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    pin_eq!(w.firsts.len(), 102, "initial run679 baseline");
    // **The word 1277's value diff, item 1558** (`docs/AI.md` §153): the
    // scout `1/0` (`TypeIndex` 77) stood apart from the window's first
    // block — (29065, 27134) in ours against (29060, 27038), its move bound
    // for (24312, 26616) in ours and the goody cell's centre (29592, 26520)
    // in the original — and on 1277 the original's stepped into cell
    // (38, 34) and spent five `Unit::explore_goody+0x27c` draws. Ours
    // re-aims on frame 1200 now, and `1/0` agrees in every compared field
    // through the window since item 1563: on item 1558's tree its one row
    // was the barge it boards on 1424, `1/47` in ours against `1/48`, a
    // number the word's own barge `1/46` shifted.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), _)| (*who, *o) == (1, 0))
            .map(|((_, _, k), (f, _))| (k.clone(), *f))
            .collect::<Vec<_>>(),
        Vec::<(String, i64)>::new(),
        "the word's scout 1/0 agrees in every compared field"
    );
    // **The word 1408's value diff, item 1563** (`docs/AI.md` §155): the
    // citizen `1/8` (`TypeIndex` 50, `PEASANTS`) parted first on block
    // 1340, its `ExploreTo` (36168, 35976) and `off` (72, 648) in ours —
    // the Bunker `1/2030`'s ring — against the Mine `1/2028`'s (37416,
    // 33864) and (552, 72) in the original. Its `find_build_spot` on 1339
    // counted two builders on each site; the original's circle walk drops
    // `1/24`, on the water of a coastal cell, from 2028's. Ours sends it to
    // 2028 now, it casts its transport on 1408 and is inside the barge
    // `1/46` on 1409 in both, and no key of `1/8` parts past its standing
    // `form` on 1272.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| (*who, *o) == (1, 8) && *f > 1272)
            .count(),
        0,
        "the word's citizen 1/8 agrees in every compared field"
    );
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1272)
        .min();
    // **The word 1532's value diff, item 1586** (`docs/AI.md` §161): army
    // `1/0`'s `1/38`, `1/43` and `1/45` parted first on block 1435 — ours
    // (42511, 37009), (42587, 38956), (42572, 38466) against (42517,
    // 37030), (42588, 38964), (42574, 38482) — stepping at the group's cap
    // of 25 (`1/45` 31, the Modern Infantry's `× 5 / 4`) where the original
    // steps at the leader `1/28`'s 47: frame 1434 is the army's normalize
    // (`1434 − 30 + 4 ≡ 0 mod 128`), and `Group::normalize`'s tail lifts
    // the cap. No key of army 1's members parts in the window past `1/28`'s
    // `order:group.id` stamp on 1277 (1276002 against 1282502) now.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| {
                *who == 1 && [28, 38, 43, 44, 45].contains(o) && *f > 1277
            })
            .count(),
        0,
        "the word's army 1/0 agrees in every compared field"
    );
    // The Oil Platform site `1/2037` (`orig_type` 422) is gone on block
    // 1420 in both (item 1588).
    pin_eq!(
        w.firsts
            .keys()
            .filter(|(who, o, _)| (*who, *o) == (1, 2037))
            .count(),
        0,
        "the Oil Platform site 1/2037 is disbanded on 1419 in both"
    );
    pin_eq!(
        first,
        Some(1277),
        "the first parting past the standing block"
    );
}

/// run710, item 1563: the lobby's blocks 1527..1783 at the long's detail —
/// the word 1532's block 1533 with six before it and 250 after.
pub(crate) const RUN710: &str = "gamelog-run710-eastindies-persian-alltech-window-1527-1783.txt";

/// The window: block 1527 through 1783; the word 1532's own block is 1533.
pub(crate) const WIDENING_COVERAGE_FRAME_1532: (i64, i64) = (1527, 1783);

/// **The newest pair's word's window** (`AI_WORDS`' `Coverage pair` row):
/// run710 walked from run651's start with the recorder on. It holds the
/// word 1532 (item 1563), block 1533, the word 1610 (item 1586), block
/// 1611, the word 1696 (item 1588), block 1697, and the word 1719 (item
/// 1589), block 1720.
pub(crate) fn coverage_pair_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[COVERAGE.start],
        true,
        COVERAGE.long,
        "run710",
        &[(RUN710, 1527)],
        WIDENING_COVERAGE_FRAME_1532,
        1,
        &[1533, 1611, 1697, 1720],
        true,
    )
}

#[test]
fn run710_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = coverage_pair_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    // **360** on the tree item 1589 landed (478 on item 1588's, 764 on
    // item 1586's, 1034 on item 1563's); 151 stand from the window's first block (167 on
    // 1563's), and army 1's
    // `1/28`, `1/38`, `1/43`, `1/44` and `1/45` are no longer among them:
    // the army's normalize on 1434 puts its group's cap back to the
    // leader's 47 (`docs/AI.md` §161).
    // Item 1578: 360 → 315 on the tree item 1589 landed, a coverage-lobby military unit's cached level is 0, so its price takes the floored 1% (`docs/AI.md` §159).
    // Item 1579: 315 → 213, the missile `1/42` ordered on 1570 by its silo's arm and its strike on `0/2000` on 1719 agree (`docs/AI.md` §160).
    pin_eq!(w.firsts.len(), 213, "initial run710 baseline");
    // **The word 1696's value diff, item 1589** (`docs/AI.md` §163): the
    // Freighter `1/52` (`TypeIndex` 322, `TRANSPORTFREIGHTER`, carrying
    // `1/40`) parted first on block 1686 — `path:length` 9 against 17,
    // `path_recursion` 10 against 1, `order:move.last_x/y` (42816, 37233)
    // against −1 — where on 1685 the original's tile search ran 63
    // `calc_cost`s from tile (221, 191) to its waypoint (42168, 37704)
    // and ours, walking the goal back from region 12 toward its own tile's
    // 0, refused the plan, popped the waypoint and spent a second grid draw
    // on 1696. No key of `1/52` parts past its standing `form` on its
    // birth block, 1529.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| *who == 1 && *o == 52 && *f > 1529)
            .count(),
        0,
        "the word's Freighter 1/52 agrees in every compared field"
    );
    // **The word 1610's value diff, item 1588** (`docs/AI.md` §162): the
    // citizen `1/10` (`TypeIndex` 50, `PEASANTS`) parted first on block
    // 1584 — ours' stack 4 with an `ExploreTo` (28488, 31272) for the
    // Oil Platform site ours alone held at (28032, 31104), against the
    // original's 2, `[Build 2038, ExploreTo (34632, 33528)]` — and on 1610
    // its caravan path (586 `calc_road_cost` draws against 578). The
    // original disbanded that site on frame 1419 (`Wall::process`'s
    // oil-platform arm), so the next site placed took its number: on 1583
    // `1/2037` is at (32640, 34176), city 2, in both, where ours' was
    // (28032, 31104), city −1, and `1/2043` the Wonder at (34560, 37632).
    // `1/10` holds two orders on 1584 in both and no key of it parts in
    // the window past its standing `form` on 1527; nor does a key of the
    // four sites.
    pin_eq!(
        w.firsts
            .iter()
            .filter(|((who, o, _), (f, _))| {
                *who == 1 && [10, 2037, 2043, 2044].contains(o) && *f > 1527
            })
            .count(),
        0,
        "the word's citizen 1/10 and the sites agree in every compared field"
    );
    let first = w
        .firsts
        .values()
        .map(|(f, _)| *f)
        .filter(|f| *f > 1527)
        .min();
    // `1/52`'s `form`, −1 in ours against 0, on the block it is born
    // (`1/28`'s `half_step` and move on 1528 until item 1586).
    pin_eq!(
        first,
        Some(1529),
        "the first parting past the standing block"
    );
}
