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
    // The battery's lowest lobby after item 3322's booking beside the
    // Greeks, promoted to the AI word (item 3329, DECISIONS 72, 83): the
    // Dutch at Difficulty 4, East Indies.
    (18, 22, 4),
    // The battery's lowest lobby after item 3318 closed the Spanish,
    // promoted to the AI word (item 3326, DECISIONS 72, 83): the Greeks at
    // Difficulty 1, Great Sahara.
    (7, 5, 1),
    // The battery's lowest lobby after item 3326 closed the Greeks,
    // promoted to the AI word (item 3331, DECISIONS 72, 83): the Japanese
    // at Difficulty 1, Great Sahara.
    (7, 15, 1),
    // The battery's lowest open lobby on the measure of `987c4c33`,
    // promoted to the AI word (item 3332, DECISIONS 72, 83): the Egyptians
    // at Difficulty 4, Great Sahara.
    (7, 7, 4),
    // The battery's lowest open lobby once item 3336 closed the Egyptians,
    // promoted to the AI word (item 3338, DECISIONS 72, 83): the Maya at
    // Difficulty 4, Great Sahara.
    (7, 1, 4),
    // The battery's lowest lobby after item 3333 closed the Dutch, promoted
    // to the AI word (item 3335, DECISIONS 72, 83): the Indians at
    // Difficulty 2, East Indies.
    (18, 21, 2),
    // The battery's lowest open lobby once item 3338 closed the Maya,
    // promoted to the AI word (item 3343, DECISIONS 72, 83): the Bantu at
    // Difficulty 4, Great Sahara.
    (7, 3, 4),
    // The battery's lowest open lobby after item 3340 closed the Indians,
    // the Maya being item 3338's, promoted to the AI word (item 3342,
    // DECISIONS 72, 83): the Russians at Difficulty 2, Great Lakes.
    (14, 13, 2),
    // The battery's second open lobby once item 3343 closed the Bantu,
    // promoted to the AI word (item 3347, DECISIONS 72, 83, 84): the French
    // at Difficulty 4, Great Sahara.
    (7, 10, 4),
    // The battery's lowest open lobby once item 3343 closed the Bantu,
    // promoted to the AI word (item 3345, DECISIONS 72, 83, 84): the Lakota
    // at Difficulty 2, Great Lakes.
    (14, 19, 2),
    // The battery's lowest open lobby once item 3345 closed the Lakota,
    // promoted to the AI word (item 3355, DECISIONS 72, 83, 84): the
    // Persians at Difficulty 2, Great Lakes.
    (14, 23, 2),
    // The battery's lowest open lobby on the measure of `3eb2c76c`,
    // promoted to the AI word (item 3360, DECISIONS 72, 83, 84): the Inca
    // at Difficulty 3, East Indies.
    (18, 2, 3),
    // The battery's lowest open lobby once item 3359 closed the French, the
    // Inca being item 3360's, promoted to the AI word (item 3361, DECISIONS
    // 72, 83, 84): the Aztecs at Difficulty 2, Great Sahara.
    (7, 0, 2),
    // The battery's lowest open lobby on the measure of `9e63ec11`,
    // promoted to the AI word (item 3367, DECISIONS 72, 83, 84): the
    // Spanish at Difficulty 3, East Indies.
    (18, 9, 3),
    // The battery's next open lobby once item 3361 closed the Aztecs, the
    // Inca being item 3360's, promoted to the AI word (item 3365, DECISIONS
    // 72, 83, 84): the Koreans at Difficulty 3, Great Sahara.
    (7, 16, 3),
    // The battery's lowest open lobby on the measure of `efc5d4c1`, the
    // Spanish being item 3367's, promoted to the AI word (item 3369,
    // DECISIONS 72, 83, 84): the Dutch at Difficulty 3, Great Sahara.
    (7, 22, 3),
    // The battery's lowest open lobby on the measure of `ac79accd`, the
    // Americans being item 3374's, promoted to the AI word (item 3375,
    // DECISIONS 72, 83, 84): the Iroquois at Difficulty 3, Great Sahara.
    (7, 18, 3),
];

pub(crate) const BATTERY: &[Lobby] = &[
    // `EastIndiesGermanTier3` (Germans 12, Difficulty 3, map 18; run646 and
    // run643, 576 of 13519) left the battery on item 2904 for the AI word
    // (`diff::german_word`, DECISIONS 72).
    // `GreatLakesRussianTier2` (Russians 13, Difficulty 2, map 14; run647
    // and run644, 6563 of 7303), one of item 1465's three, left the battery
    // on item 3342 for the AI word (`diff::russian_word`, DECISIONS 72, 83).
    // `GreatSaharaEgyptianTier4` (Egyptians 7, Difficulty 4, map 7; run648
    // and run645, 2974 of 20380), one of item 1465's three, left the battery
    // on item 3332 for the AI word (`diff::egyptian_word`, DECISIONS 72, 83).
    // `GreatSaharaMayaTier4` (Maya 1, Difficulty 4, map 7; run966 and
    // run966b, 776 of 13086), minted by item 3332, left the battery on item
    // 3338 for the AI word (`diff::maya_word`, DECISIONS 72, 83).
    // `GreatSaharaBantuTier4` (Bantu 3, Difficulty 4, map 7; run972 and
    // run972b, 4 of 23627), minted by item 3338, left the battery on item
    // 3343 for the AI word (`diff::bantu_word`, DECISIONS 72, 83).
    // `GreatSaharaFrenchTier4` (French 10, Difficulty 4, map 7; run977 and
    // run977b, 13422 of 16764), minted by item 3343, left the battery on
    // item 3347 for the AI word (`diff::french_word`, DECISIONS 72, 83, 84).
    // The lobby minted by item 3347 (DECISIONS 72, 83, 84, `docs/AI.md`
    // §125, §278; runs 981 and 981b) when the French left for the AI word,
    // so the held-out set stays eight: Great Sahara, the Russians at
    // Difficulty 4, by entry 72's rule (2) — a nation held only by closed
    // games on another map, never on Great Sahara. The Romans and the
    // Russians tie on reach (10 census functions each, `docs/NATIONS.md`);
    // the Russians' only game (Great Lakes, closed) ended at 7303 against
    // the Romans' 18620, so theirs is the nation whose later arms no trace
    // has run. Tier 4 is the tier the French leave; the Persians are item
    // 3345's. The game ends itself on 18883.
    Lobby {
        name: "GreatSaharaRussianTier4",
        map_style: 7,
        ai_tribe: 13,
        difficulty: 4,
        start: "gamelog-run981-greatsahara-russian-diff4-start.txt",
        long: (
            "gamelog-run981b-greatsahara-russian-diff4-24k-trace.txt",
            "rontrace-run981b.log",
        ),
        length: 18883,
        // Item 3347, the first try: the draw stream agrees to the game's
        // end, count and sequence both at 18883.
        count: 18883,
        sequence: 18883,
    },
    // `GreatLakesLakotaTier2` (Lakota 19, Difficulty 2, map 14; run975
    // and run975b, 6071 of 7902), minted by item 3342, left the battery on
    // item 3345 for the AI word (`diff::lakota_word`, DECISIONS 72, 83, 84).
    // `GreatLakesPersianTier2` (Persians 23, Difficulty 2, map 14; run979
    // and run979b, 6046 of 7105), minted by item 3345, left the battery on
    // item 3355 for the AI word (`diff::persian_word`, DECISIONS 72, 83, 84).
    // The lobby minted by item 3355 (DECISIONS 72 as amended by 84,
    // `docs/AI.md` §125, §279; runs 983 and 983b) when the Persians left
    // for the AI word, so the held-out set stays eight: Great Lakes, the
    // Romans at Difficulty 2 — **rule (2)**: rule (1) has no answer (a
    // header survey of every gamelog on disk holds every nation), and of
    // the nations no Great Lakes dump holds and only closed games hold (the
    // open battery's Aztecs, Koreans and Americans out), the Romans reach
    // the most census functions (10, `docs/NATIONS.md`; the Spanish and the
    // Dutch 9), held only by `EastIndiesRomanTier2`; tier 2 is the tier the
    // Persians left (Chinese 1 and Turks 4 remain). The game ends itself on
    // 7799.
    Lobby {
        name: "GreatLakesRomanTier2",
        map_style: 14,
        ai_tribe: 6,
        difficulty: 2,
        start: "gamelog-run983-greatlakes-roman-diff2-start.txt",
        long: (
            "gamelog-run983b-greatlakes-roman-diff2-24k-trace.txt",
            "rontrace-run983b.log",
        ),
        length: 7799,
        // Item 3355, the first try: count 7799, sequence 7799 — the draw
        // stream agrees to the game's end.
        count: 7799,
        sequence: 7799,
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
    // `GreatSaharaAztecTier2` (Aztecs 0, Difficulty 2, map 7; run895 and
    // run895b, 9130 of 17554) left the battery on item 3361 for the AI
    // word (`diff::aztec_word`, DECISIONS 72, 83, 84).
    // The lobby minted by item 3361 (DECISIONS 72 as amended by 84,
    // `docs/AI.md` §125, §282; runs 988 and 988b) when the Aztecs left for
    // the AI word, so the held-out set stays eight: Great Sahara, the
    // Lakota at Difficulty 2 — **rule (2)**: rule (1) has no answer (a
    // header survey of the 529 gamelogs on disk holds every nation); of
    // the nations no Great Sahara dump holds and only closed games hold
    // (`GreatLakesLakotaTier2` and the peace pair; the open battery's
    // Inca, Turks, Chinese, Americans and Romans out), the Lakota reach the
    // most census functions (17, `docs/NATIONS.md`; the Dutch 9); tier 2
    // is the tier the Aztecs left (the Russians hold 4 and the Koreans 3).
    // The game does not end itself: the long runs to 24000.
    Lobby {
        name: "GreatSaharaLakotaTier2",
        map_style: 7,
        ai_tribe: 19,
        difficulty: 2,
        start: "gamelog-run988-greatsahara-lakota-diff2-start.txt",
        long: (
            "gamelog-run988b-greatsahara-lakota-diff2-24k-trace.txt",
            "rontrace-run988b.log",
        ),
        length: 24000,
        count: 24000,
        sequence: 24000,
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
    // `EastIndiesDutchTier4` (Dutch 22, Difficulty 4, map 18; run915 and
    // run917, 1567 of 24000) left the battery on item 3329 for the AI word
    // (`diff::dutch_word`, DECISIONS 72, 83).
    // `GreatSaharaGreekTier1` (Greeks 5, Difficulty 1, map 7; run957 and
    // run957b, 176 of 24000), minted by item 3318, left the battery on item
    // 3326 for the AI word (`diff::greek_word`, DECISIONS 72, 83).
    // `GreatSaharaJapaneseTier1` (Japanese 15, Difficulty 1, map 7; run960
    // and run960b, 576 of 24000), minted by item 3326, left the battery on
    // item 3331 for the AI word (`diff::japanese_word`, DECISIONS 72, 83).
    // `GreatSaharaKoreanTier3` (Koreans 16, Difficulty 3, map 7; run964
    // and run964b, 9185 of 15944), minted by item 3331, left the battery on
    // item 3365 for the AI word (`diff::korean_word`, DECISIONS 72, 83, 84).
    // `GreatSaharaDutchTier3` (Dutch 22, Difficulty 3, map 7; run990 and
    // run990b, 311 of 16109), minted by item 3365 under rule (2), left the
    // battery on item 3369 for the AI word (`diff::dutch_sahara_word`,
    // DECISIONS 72, 83, 84).
    // `GreatSaharaIroquoisTier3` (Iroquois 18, Difficulty 3, map 7; run994
    // and run994b, 13757 of 16371), minted by item 3369 under rule (2), left
    // the battery on item 3375 for the AI word
    // (`diff::iroquois_sahara_word`, DECISIONS 72, 83, 84).
    // The lobby minted by item 3375 (DECISIONS 72 as amended by 84, rule
    // (2); `docs/AI.md` §288; runs 1000 and 1000b) when the Iroquois left
    // for the AI word, so the held-out set stays eight: Great Sahara, the
    // Germans at Difficulty 3 — no map-7 dump held the Germans, who are
    // held only by a closed game on another map (`EastIndiesGermanTier3`)
    // and reach the most census functions of the nations so held (8; the
    // Indians 4, the Inca 3), at the tier the Iroquois leave. The game ends
    // itself on 19483.
    Lobby {
        name: "GreatSaharaGermanTier3",
        map_style: 7,
        ai_tribe: 12,
        difficulty: 3,
        start: "gamelog-run1000-greatsahara-german-diff3-start.txt",
        long: (
            "gamelog-run1000b-greatsahara-german-diff3-24k-trace.txt",
            "rontrace-run1000b.log",
        ),
        length: 19483,
        // Item 3375, the first try: the count parts at 13585; the
        // sequence at 13582, ours 64 draws, theirs 64, at 0: ours
        // `Leader::use_market+0x1ed`, theirs
        // `Leader::produce_building+0xc99`.
        count: 13585,
        sequence: 13582,
    },
    // `EastIndiesIndianTier2` (Indians 21, Difficulty 2, map 18; run962
    // and run962b, 221 of 24000), minted by item 3329, left the battery on
    // item 3335 for the AI word (`diff::indian_word`, DECISIONS 72, 83).
    // `EastIndiesIncaTier3` (Inca 2, Difficulty 3, map 18; run969 and
    // run969b, 7782 of 24000), minted by item 3335, left the battery on
    // item 3360 for the AI word (`diff::inca_word`, DECISIONS 72, 83, 84).
    // `EastIndiesSpanishTier3` (Spanish 9, Difficulty 3, map 18; run986
    // and run986b, 9028 of 24000), minted by item 3360 under rule (2), left
    // the battery on item 3367 for the AI word (`diff::spanish_east_word`,
    // DECISIONS 72, 83, 84).
    // The lobby minted by item 3367 (DECISIONS 72 as amended by 84,
    // `docs/AI.md` §125, §284; runs 992 and 992b) when the Spanish left for
    // the AI word, so the held-out set stays eight: East Indies, the
    // Egyptians at Difficulty 3 — **rule (2)**: rule (1) has no answer
    // (every nation is in some dump); a header survey of every dump puts
    // the Egyptians only at Great Sahara 4 and the Iroquois only at Great
    // Lakes 3, both closed games, both 8 census functions (`docs/NATIONS.md`)
    // and the widest reach left among nations no East Indies dump holds;
    // the tie goes to the Egyptians, whose lobby moves both map and tier
    // off their only game. Tier 3 is the tier the Spanish left; East
    // Indies' battery then holds tier 1 alone. The game ends itself on
    // 14540 (closing block 14541).
    Lobby {
        name: "EastIndiesEgyptianTier3",
        map_style: 18,
        ai_tribe: 7,
        difficulty: 3,
        start: "gamelog-run992-eastindies-egyptian-diff3-start.txt",
        long: (
            "gamelog-run992b-eastindies-egyptian-diff3-24k-trace.txt",
            "rontrace-run992b.log",
        ),
        length: 14540,
        // Item 3367, the first try: the draw stream runs whole to the
        // game's own end, 14540 of 14540.
        count: 14540,
        sequence: 14540,
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
