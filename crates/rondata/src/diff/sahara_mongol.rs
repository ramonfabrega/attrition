//! **Great Sahara in the successor pair's lobby** (DECISIONS 56 §1, item
//! 2107): the closed map's successor once the coverage pair's Great Sahara
//! closed at its end (item 2106). `MAP_STYLE 7`, human Nubians (4) against
//! **Mongols (17)** at **Difficulty 3**, `STARTING_TECHNOLOGY 8` and
//! `STARTING_RESOURCES 7`, seed 12345 — run774's and run775's recipes with
//! the map moved and nothing else. The captures are run832 (the `DUMP_ALL`
//! start) and run831 (the draw stream to the game's end, `cover=0`); run830
//! is the same game with a unit dump on every frame and no closing records.
//! The first parting is the third map's word in the newest pair's lobby;
//! `docs/AI.md` §211 has it.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SAHARA_MONGOL: Pair = Pair {
    name: "GreatSaharaMongolAllTech",
    map_style: 7,
    ai_tribe: 17,
    difficulty: 3,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run832-greatsahara-mongol-alltech-start.txt",
    long: (
        "gamelog-run831-greatsahara-mongol-alltech-trace.txt",
        "rontrace-run831.log",
    ),
    length: 3268,
    // Item 2107, first try: frame 2995 by the count and the sequence, ours
    // 30 draws against 26, index 3: ours `Guy::init_real+0x52`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` (`docs/AI.md` §211).
    // Item 2107's build moved it 2995 -> 3268, the game's last frame:
    // `Build::finished` holds a done Caravan at `get_caravan_limit(0)` and
    // `do_queue` runs `get_next_non_caravan`'s Merchant behind it (the
    // Market `1/2019`, 2995 to the end; `docs/AI.md` §211).
    count: 3268,
    sequence: 3268,
};

/// The map's word as the handoff's `Third map:` line and `AI_WORDS` carry it.
pub(crate) const SAHARA_MONGOL_WORD: i64 = if SAHARA_MONGOL.count < SAHARA_MONGOL.sequence {
    SAHARA_MONGOL.count
} else {
    SAHARA_MONGOL.sequence
};

/// **The map's first parting**, walked from its own start.
#[test]
fn sahara_mongol_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SAHARA_MONGOL.start, SAHARA_MONGOL.long) else {
        return;
    };
    eprintln!(
        "SAHARA_MONGOL {}: count {} sequence {} of {}; {}",
        SAHARA_MONGOL.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SAHARA_MONGOL.difficulty, "the lobby read back");
    pin_eq!(w.last, SAHARA_MONGOL.length, "the game's last frame");
    pin_eq!(w.count, SAHARA_MONGOL.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        SAHARA_MONGOL.sequence,
        "the draw sequence's first parting"
    );
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — map, difficulty, the two settings, nations and seed — and the
/// long is scoreable. A setting that did not take reads back as its default.
#[test]
fn sahara_mongol_captures_say_their_lobby() {
    let p = &SAHARA_MONGOL;
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
        "the Sahara Mongol long is not scoreable"
    );
}

/// The lobby is a scored one and none of the battery's.
#[test]
fn sahara_mongol_is_no_other_lobby() {
    let p = &SAHARA_MONGOL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run834: the word 2995's window, blocks 2989..3269 at the long's detail —
/// six blocks before the word's block 2996, and on to the game's closing
/// block 3269 (item 2107).
pub(crate) const RUN834: &str = "gamelog-run834-greatsahara-mongol-alltech-window-2989-3269.txt";

/// The window: block 2989 through the closing block 3269; the word 2995's
/// own block is 2996.
pub(crate) const WIDENING_SAHARA_MONGOL_FRAME_2995: (i64, i64) = (2989, 3269);

/// **The map's word's window** (`AI_WORDS`' `GreatSaharaMongolAllTech` row).
pub(crate) fn sahara_mongol_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SAHARA_MONGOL.start],
        true,
        SAHARA_MONGOL.long,
        "run834",
        &[(RUN834, WIDENING_SAHARA_MONGOL_FRAME_2995.0)],
        WIDENING_SAHARA_MONGOL_FRAME_2995,
        1,
        &[2996],
        true,
    )
}

#[test]
fn run834_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sahara_mongol_word_window() else {
        return;
    };
    pin_eq!(w.blocks, 0, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 0, "run834 baseline");
}
