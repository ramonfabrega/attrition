//! **The battery's Great Sahara word** (DECISIONS 72, 83; item 3318): the
//! held-out battery's lowest game once item 3306 closed the Iroquois,
//! promoted by item 2904's, 3107's and 3314's recipe. Great Sahara
//! (`MAP_STYLE 7`), human Nubians (4) against **the Spanish (9)** at
//! Difficulty 3, seed 12345 — `docs/AI.md` §243's lobby, scored by its first
//! try at 576 of 22,700 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run896 (a `DUMP_ALL` start, blocks 0..36) and run896b
//! (the draw stream to the game's end, `cover=0`, its setup and closing
//! blocks only); the word's window is run958's (`docs/AI.md` §264).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH: Lobby = Lobby {
    name: "GreatSaharaSpanishTier3",
    map_style: 7,
    ai_tribe: 9,
    difficulty: 3,
    start: "gamelog-run896-greatsahara-spanish-diff3-start.txt",
    long: (
        "gamelog-run896b-greatsahara-spanish-diff3-24k-trace.txt",
        "rontrace-run896b.log",
    ),
    length: 22700,
    // Item 2975: the battery's first try, count and sequence both at 576.
    // Item 3318 promoted it to the AI word (`docs/AI.md` §264).
    count: 576,
    sequence: 576,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_WORD: i64 = if SPANISH.count < SPANISH.sequence {
    SPANISH.count
} else {
    SPANISH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH.start, SPANISH.long) else {
        return;
    };
    eprintln!(
        "SPANISH {}: count {} sequence {} of {}; {}",
        SPANISH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SPANISH.difficulty, "the lobby read back");
    pin_eq!(w.last, SPANISH.length, "the game's last frame");
    pin_eq!(w.count, SPANISH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        SPANISH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_word_is_no_other_lobby() {
    let p = &SPANISH;
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
    }
}

/// run958: the word 576's window, blocks 570..826 at run956's detail
/// (`LEADERS=9`, `GUYS=2`; item 3318); run896 dumps blocks 0..36 and
/// run896b its setup and closing blocks only.
pub(crate) const RUN958: &str = "gamelog-run958-greatsahara-spanish-diff3-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_SPANISH_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn spanish_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH.start],
        true,
        SPANISH.long,
        "run958",
        &[(RUN958, WIDENING_SPANISH_FRAME_576.0)],
        WIDENING_SPANISH_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run958_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_frame_576_window() else {
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
    pin_eq!(w.firsts.len(), 0, "run958 on its first build");
}
