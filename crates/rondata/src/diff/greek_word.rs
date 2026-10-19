//! **The battery's Greek word** (DECISIONS 72, 83; item 3326): the held-out
//! battery's lowest game once item 3318 closed the Spanish, promoted by item
//! 2904's, 3107's, 3314's and 3318's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Greeks (5)** at Difficulty 1, seed 12345
//! — the lobby item 3318 minted (`docs/AI.md` §264), scored by its first
//! try at 176 of 24,000 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run957 (a `DUMP_ALL` start, blocks 0..36) and run957b
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks only);
//! the word's window is run961's (`docs/AI.md` §266).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GREEK: Lobby = Lobby {
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
    // Item 3318: the battery's first try, count and sequence both at 176;
    // ours 15 draws, theirs 36, at 0 ours `Animal::think_bird+0x82`, theirs
    // `Leader::compute_sites+0x4ac`.
    count: 176,
    sequence: 176,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const GREEK_WORD: i64 = if GREEK.count < GREEK.sequence {
    GREEK.count
} else {
    GREEK.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn greek_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(GREEK.start, GREEK.long) else {
        return;
    };
    eprintln!(
        "GREEK {}: count {} sequence {} of {}; {}",
        GREEK.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, GREEK.difficulty, "the lobby read back");
    pin_eq!(w.last, GREEK.length, "the game's last frame");
    pin_eq!(w.count, GREEK.count, "the draw count's first parting");
    pin_eq!(w.sequence, GREEK.sequence, "the draw sequence's first parting");
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn greek_word_is_no_other_lobby() {
    let p = &GREEK;
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

/// run961: the word 176's window, blocks 170..426 at run958d's detail
/// (`LEADERS=9`, `GUYS=4`; item 3326); run957 dumps blocks 0..36 and run957b
/// its setup and closing blocks only.
pub(crate) const RUN961: &str = "gamelog-run961-greatsahara-greek-diff1-window-170-427-guys4.txt";

/// The window: block 170 through 426; the word 176's own block is 177.
pub(crate) const WIDENING_GREEK_FRAME_176: (i64, i64) = (170, 426);

pub(crate) fn greek_frame_176_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK.start],
        true,
        GREEK.long,
        "run961",
        &[(RUN961, WIDENING_GREEK_FRAME_176.0)],
        WIDENING_GREEK_FRAME_176,
        1,
        &[177],
        true,
    )
}

#[test]
fn run961_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_frame_176_window() else {
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
    pin_eq!(w.firsts.len(), 0, "run961 on the first build");
}
