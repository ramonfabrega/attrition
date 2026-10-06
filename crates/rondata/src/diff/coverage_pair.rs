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
    /// make list, which this crate never places (`docs/AI.md` §129).
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
    count: 8,
    sequence: 8,
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
    pin_eq!(w.firsts.len(), 87, "initial run651 baseline");
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
    pin_eq!(w.firsts.len(), 364, "initial run656 baseline");
}
