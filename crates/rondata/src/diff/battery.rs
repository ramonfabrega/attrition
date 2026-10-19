//! **The held-out battery** (DECISIONS 61 §6, item 1465): lobbies no scored
//! word shares a nation, a map and a difficulty with, each walked once from
//! the first try and scored by the frame its draw stream first parts on.
//!
//! **A measure, a floor since the thirty-ninth steer, and never debugged
//! against.** The numbers in [`BATTERY`] are not in `FLOORS`, `AI_WORDS` or
//! the handoff's scoreboard; no item names a battery word, and a parting
//! here that tempts a fix is parked, not built. The pass reads them with
//! `cargo test --release -p rondata battery_measure -- --ignored --nocapture`
//! and re-pins what moved. Two tests run in every gate: the identity test,
//! so a lobby that stopped being held out (a scored word moved onto its
//! nation, map and difficulty) fails it; and [`battery_floors_hold`]
//! (DECISIONS 77, Ramon's steer of 2026-10-17), so a landing that lowers an
//! unseen lobby's first-try frame — a regression on a game nobody tuned
//! for — is red in its own gate instead of found at the next pass. Nobody
//! reads the lobby's dump for it: the lane finds a fix that does not lower
//! it, or re-pins the lobby down in the same landing with the item in the
//! comment and a `FABLE:` row, and the pass rules.
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
    // The coverage pair's (item 1466): Persians at Toughest, East Indies.
    (18, 23, 5),
    // The same lobby on Great Sahara (item 1538).
    (7, 23, 5),
    // Great Sahara in the successor pair's lobby (item 2107).
    (7, 17, 3),
    // The peace pair's (item 2320): the Lakota at Difficulty 3, East Indies.
    (18, 19, 3),
    // The battery's first lobby, promoted to the AI word (item 2904,
    // DECISIONS 72): the Germans at Difficulty 3, East Indies.
    (18, 12, 3),
    // The battery's lowest lobby after the Germans closed, promoted to the
    // AI word (item 3107, DECISIONS 72): the Iroquois at Difficulty 3,
    // Great Lakes.
    (14, 18, 3),
    // The battery's lowest lobby after item 3306 closed the Iroquois,
    // promoted to the AI word (item 3318, DECISIONS 72, 83): the Spanish at
    // Difficulty 3, Great Sahara.
    (7, 9, 3),
    // The battery's lowest lobby after item 3306's booking, promoted to the
    // AI word (item 3314, DECISIONS 72, 83): the Romans at Difficulty 2,
    // East Indies.
    (18, 6, 2),
];

pub(crate) const BATTERY: &[Lobby] = &[
    // `EastIndiesGermanTier3` (Germans 12, Difficulty 3, map 18; run646 and
    // run643, 576 of 13519) left the battery on item 2904 for the AI word
    // (`diff::german_word`, DECISIONS 72).
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
    // The fourth lobby (item 2904, DECISIONS 72), minted when the first
    // left for the AI word so three stay held out: Great Lakes, the Chinese
    // at Difficulty 1 — a nation and a difficulty no scored or battery
    // lobby has. The game does not end itself: the long runs to 24000.
    Lobby {
        name: "GreatLakesChineseTier1",
        map_style: 14,
        ai_tribe: 14,
        difficulty: 1,
        start: "gamelog-run887-greatlakes-chinese-diff1-start.txt",
        long: (
            "gamelog-run888-greatlakes-chinese-diff1-24k-trace.txt",
            "rontrace-run888.log",
        ),
        length: 24000,
        // Item 2904, the first try: ours 12 draws, theirs 14, at 6 ours
        // `Farms::inc_time+0x1ae`, theirs `Guy::init_real+0x52`. Item 3092,
        // the Chinese citizen arm of `train_time` (NATIONS §7, first candidate): 176 →
        // 5663, ours 6 draws, theirs 8, at 0 ours `Guy::set_anim+0x97a <
        // Guy::inc_time+0x271`, theirs `Guy::init_real+0x52`. Item 3113, the
        // Chinese herbal-lore free-tech block (`gain_tech@006dcb60`, NATIONS
        // §7.4): 5663 → 12712, ours 34 draws, theirs 40, at 30 ours
        // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
        // `Guy::init_real+0x52`.
        count: 12712,
        sequence: 12712,
    },
    // The five lobbies minted by item 2975 (DECISIONS 77 (ii), `docs/AI.md`
    // §125, §243; runs 895–899): a nation no dump holds, a tier of 2, 3 or 4,
    // no two sharing a nation or a tier on a map. Each game ended itself.
    Lobby {
        name: "GreatSaharaAztecTier2",
        map_style: 7,
        ai_tribe: 0,
        difficulty: 2,
        start: "gamelog-run895-greatsahara-aztec-diff2-start.txt",
        long: (
            "gamelog-run895b-greatsahara-aztec-diff2-24k-trace.txt",
            "rontrace-run895b.log",
        ),
        length: 17554,
        count: 9130,
        sequence: 9130,
    },
    // `GreatSaharaSpanishTier3` (Spanish 9, Difficulty 3, map 7; run896
    // and run896b, 576 of 22700) left the battery on item 3318 for the AI
    // word (`diff::spanish_word`, DECISIONS 72, 83).
    // `GreatLakesIroquoisTier3` (Iroquois 18, Difficulty 3, map 14; run897
    // and run897b, 0 of 12658) left the battery on item 3107 for the AI
    // word (`diff::iroquois_word`, DECISIONS 72).
    Lobby {
        name: "GreatLakesTurkishTier4",
        map_style: 14,
        ai_tribe: 8,
        difficulty: 4,
        start: "gamelog-run898-greatlakes-turkish-diff4-start.txt",
        long: (
            "gamelog-run898b-greatlakes-turkish-diff4-24k-trace.txt",
            "rontrace-run898b.log",
        ),
        length: 15503,
        // 1440 → 6982 on item 3160 (`get_cost`'s `TURK_CITIZEN` and
        // `WOOL_CITIZENS_COST` arms, `cost::Modifiers::citizens`).
        count: 6982,
        sequence: 6982,
    },
    // `EastIndiesRomanTier2` (Romans 6, Difficulty 2, map 18; run899 and
    // run899b, 576 of 18620) left the battery on item 3314 for the AI word
    // (`diff::roman_word`, DECISIONS 72, 83).
    // The lobby minted by item 3107 (DECISIONS 72, `docs/AI.md` §125,
    // §246; runs 915 and 917) when the Iroquois left for the AI word, so
    // the held-out set stays replenished: East Indies, the Dutch at
    // Difficulty 4 — a nation no dump held and a tier no lobby on the map
    // has. The game does not end itself: the long runs to 24000.
    Lobby {
        name: "EastIndiesDutchTier4",
        map_style: 18,
        ai_tribe: 22,
        difficulty: 4,
        start: "gamelog-run915-eastindies-dutch-diff4-start.txt",
        long: (
            "gamelog-run917-eastindies-dutch-diff4-24k-trace.txt",
            "rontrace-run917.log",
        ),
        length: 24000,
        // Item 3107, the first try: ours 6 draws, theirs 5, at 2 ours
        // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
        // `Farms::inc_time+0x1ae`.
        count: 1567,
        sequence: 1567,
    },
    // The lobby minted by item 3318 (DECISIONS 72, `docs/AI.md` §125,
    // §264; runs 957 and 957b) when the Spanish left for the AI word, so
    // the held-out set stays eight: Great Sahara, the Greeks at Difficulty
    // 1 — a nation no dump held and a tier no lobby on the map has. The
    // game does not end itself: the long runs to 24000.
    Lobby {
        name: "GreatSaharaGreekTier1",
        map_style: 7,
        ai_tribe: 5,
        difficulty: 1,
        start: "gamelog-run957-greatsahara-greek-diff1-start.txt",
        long: (
            "gamelog-run957b-greatsahara-greek-diff1-24k-trace.txt",
            "rontrace-run957b.log",
        ),
        length: 24000,
        // Item 3318, the first try: ours 15 draws, theirs 36, at 0 ours
        // `Animal::think_bird+0x82`, theirs `Leader::compute_sites+0x4ac`.
        count: 176,
        sequence: 176,
    },
    // The lobby minted by item 3314 (DECISIONS 72, 83, `docs/AI.md` §125,
    // §263; runs 955 and 955b) when the Romans left for the AI word, so
    // the held-out set stays eight: East Indies, the Americans at
    // Difficulty 1 — a nation no dump held and a tier no lobby on the map
    // has. The game does not end itself: the long runs to 24000.
    Lobby {
        name: "EastIndiesAmericanTier1",
        map_style: 18,
        ai_tribe: 20,
        difficulty: 1,
        start: "gamelog-run955-eastindies-american-diff1-start.txt",
        long: (
            "gamelog-run955b-eastindies-american-diff1-24k-trace.txt",
            "rontrace-run955b.log",
        ),
        length: 24000,
        // Item 3314, the first try: ours 14 draws, theirs 12, at 1 ours
        // `Leader::make_stuff+0x221`, theirs `Leader::use_market+0x1ed`.
        count: 9382,
        sequence: 9382,
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

/// **The floors** (DECISIONS 77): each held-out lobby's first-try frames,
/// count and sequence, never lower than pinned on this tree. Higher passes —
/// a landing that generalised — and the pass re-pins it upward with
/// `battery_measure`. Lower is red: an unseen game regressed under this
/// landing. About two seconds for three lobbies in release.
#[test]
fn battery_floors_hold() {
    let mut fell = Vec::new();
    for lobby in BATTERY {
        let Some(w) = third::walk_from(lobby.start, lobby.long) else {
            continue;
        };
        if w.count < lobby.count || w.sequence < lobby.sequence {
            fell.push(format!(
                "{}: count {} (pinned {}), sequence {} (pinned {}); {}",
                lobby.name, w.count, lobby.count, w.sequence, lobby.sequence, w.row
            ));
        }
    }
    assert!(
        fell.is_empty(),
        "a held-out lobby fell under this tree — an unseen game regressed (DECISIONS 61 §6, 77). \
         Nobody reads its dump: find a fix that does not lower it, or re-pin `count`/`sequence` \
         down in `diff::battery::BATTERY` in this landing with the item in the comment and a \
         `FABLE:` row in docs/PARKED.md's Loop section, and the pass rules:\n  {}",
        fell.join("\n  ")
    );
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
