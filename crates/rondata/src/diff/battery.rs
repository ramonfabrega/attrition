//! **The held-out battery** (DECISIONS 61 §6, item 1465): lobbies no scored
//! word shares a nation, a map and a difficulty with, each walked once from
//! the first try and scored by the frame its draw stream first parts on.
//!
//! **A measure, never a floor and never debugged against.** The numbers in
//! [`BATTERY`] are not in `FLOORS`, `AI_WORDS` or the handoff's scoreboard;
//! no item names a battery word, and a parting here that tempts a fix is
//! parked, not built. The pass reads them with
//! `cargo test --release -p rondata battery_measure -- --ignored --nocapture`
//! and re-pins what moved. The identity test below runs in every gate, so a
//! lobby that stopped being held out (a scored word moved onto its nation,
//! map and difficulty) fails it.
//!
//! The start sibling is the map's existing `DUMP_ALL` start — heights,
//! herds and goods do not depend on the AI's nation or difficulty — and the
//! long trace is the lobby's own, in run600's shape (`cover=0`, the setup
//! and closing records explicit). `docs/AI.md` §125 has the lobbies, the
//! census rows each should enter, and the first numbers.

use super::testkit::*;
use super::*;
use crate::testenv::dump;

/// One held-out lobby: human Nubians (tribe 4) against `ai_tribe` at
/// `difficulty`, seed 12345.
pub(crate) struct Lobby {
    pub name: &'static str,
    pub map_style: i32,
    pub ai_tribe: i32,
    pub difficulty: i32,
    /// The sibling that supplies heights, herds and goods: a scored lobby's
    /// start dump on the same map.
    pub start: &'static str,
    pub long: (&'static str, &'static str),
    /// The trace's last frame: the frame the game ended on.
    pub length: i64,
    /// The measure, first try: the frame the draw count first parts on,
    /// and the frame the draw sequence first parts on.
    pub count: i64,
    pub sequence: i64,
}

/// The lobbies the scored words use, `(map_style, ai_tribe, difficulty)`,
/// read off every gamelog on the disk's `GAME INFO` (item 1465's survey):
/// the first pair's British at Easiest, the second's at Toughest, the third's
/// French at Toughest, and the old held-out map's.
pub(crate) const SCORED_LOBBIES: &[(i32, i32, i32)] = &[
    (14, 11, 0),
    (18, 11, 0),
    (7, 11, 0),
    (14, 11, 5),
    (18, 11, 5),
    (7, 11, 5),
    (14, 10, 5),
    (18, 10, 5),
    (9, 11, 0),
];

pub(crate) const BATTERY: &[Lobby] = &[
    Lobby {
        name: "EastIndiesGermanTier3",
        map_style: 18,
        ai_tribe: 12,
        difficulty: 3,
        start: "gamelog-run646-eastindies-german-diff3-start.txt",
        long: (
            "gamelog-run643-eastindies-german-diff3-24k-trace.txt",
            "rontrace-run643.log",
        ),
        length: 13519,
        count: 576,
        sequence: 576,
    },
    Lobby {
        name: "GreatLakesRussianTier2",
        map_style: 14,
        ai_tribe: 13,
        difficulty: 2,
        start: "gamelog-run647-greatlakes-russian-diff2-start.txt",
        long: (
            "gamelog-run644-greatlakes-russian-diff2-24k-trace.txt",
            "rontrace-run644.log",
        ),
        length: 7303,
        count: 6566,
        sequence: 6563,
    },
    Lobby {
        name: "GreatSaharaEgyptianTier4",
        map_style: 7,
        ai_tribe: 7,
        difficulty: 4,
        start: "gamelog-run648-greatsahara-egyptian-diff4-start.txt",
        long: (
            "gamelog-run645-greatsahara-egyptian-diff4-24k-trace.txt",
            "rontrace-run645.log",
        ),
        length: 20380,
        count: 2974,
        sequence: 2974,
    },
];

/// **The measure, read at every pass** (`--ignored`): each lobby walked
/// from its own start sibling, the frames its draw stream first parts on,
/// and the lobby read back from the dump. A moved number is a landing that
/// reached the battery: the pass reads it as generalisation (up) or
/// regression (down), re-pins it with the move's delta in the comment, and
/// builds nothing against it.
#[test]
#[ignore = "a measure, read at every pass (DECISIONS 61 §6)"]
fn battery_measure() {
    let _pins = Pins::hold();
    for lobby in BATTERY {
        let Some(w) = third::walk_from(lobby.start, lobby.long) else {
            continue;
        };
        eprintln!(
            "BATTERY {}: count {} sequence {} of {}; {}",
            lobby.name, w.count, w.sequence, w.last, w.row
        );
        pin_eq!(
            w.difficulty,
            lobby.difficulty,
            "{}: the lobby read back",
            lobby.name
        );
        pin_eq!(
            w.last,
            lobby.length,
            "{}: the game's last frame",
            lobby.name
        );
        pin_eq!(
            w.count,
            lobby.count,
            "{}: the draw count's first parting",
            lobby.name
        );
        pin_eq!(
            w.sequence,
            lobby.sequence,
            "{}: the draw sequence's first parting",
            lobby.name
        );
    }
}

/// Every battery capture says its lobby — map, difficulty, nations, seed —
/// from its own `GAME INFO` and `PLAYER` blocks, the long trace and its start
/// sibling alike, is complete, and is none of the scored lobbies. The
/// control behind the second clause is in `docs/AI.md` §125: a scored trace
/// walked with another lobby's sibling parts at frame 0, so a sibling that
/// is not the lobby's own measures the instrument.
#[test]
fn battery_captures_are_complete_and_say_their_lobby() {
    for lobby in BATTERY {
        assert!(
            !SCORED_LOBBIES.contains(&(lobby.map_style, lobby.ai_tribe, lobby.difficulty)),
            "{} is a scored lobby",
            lobby.name
        );
        for name in [lobby.long.0, lobby.start] {
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
            assert_eq!(info("MAP_STYLE"), Some(lobby.map_style), "{name}");
            assert_eq!(info("DIFFICULTY"), Some(lobby.difficulty), "{name}");
            let tribes: Vec<_> = init
                .leaders
                .iter()
                .filter(|l| l.who < 2)
                .map(|l| (l.who, l.tribe))
                .collect();
            assert_eq!(tribes, [(0, 4), (1, i64::from(lobby.ai_tribe))], "{name}");
        }
        let Some(path) = dump(lobby.long.0) else {
            continue;
        };
        let text = crate::capture::read(&path);
        assert_eq!(
            third_pair::incomplete_long(&Log::parse(&text)),
            Vec::<&str>::new(),
            "{} is not scoreable",
            lobby.name
        );
    }
}
