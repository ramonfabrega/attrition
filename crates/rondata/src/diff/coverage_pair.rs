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
    /// the frame the draw sequence parts on.
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
    count: 0,
    sequence: 0,
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

/// The lobby is none of the scored words' and none of the battery's: the
/// pair exists for what no other lobby enters.
#[test]
fn coverage_pair_is_no_other_lobby() {
    let triple = (COVERAGE.map_style, COVERAGE.ai_tribe, COVERAGE.difficulty);
    assert!(!battery::SCORED_LOBBIES.contains(&triple));
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
