//! **The battery's Indian word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3383): the held-out battery's lowest open game on the measure of
//! `bb82188d`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Indians (21)** at Difficulty 3, seed
//! 12345 — minted by item 3378 (entry 72's rule (2): a nation held only by a
//! closed game on another map), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::indian_word`, which is the closed East Indies game at Difficulty
//! 2. Captures: run1002 (a `DUMP_ALL` start) and run1002b (the draw stream
//! to the game's end, `cover=0`, its setup and closing blocks only); the
//! game ends itself on 16564 (`docs/AI.md` §291).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INDIAN_SAHARA: Lobby = Lobby {
    name: "GreatSaharaIndianTier3",
    map_style: 7,
    ai_tribe: 21,
    difficulty: 3,
    start: "gamelog-run1002-greatsahara-indian-diff3-start.txt",
    long: (
        "gamelog-run1002b-greatsahara-indian-diff3-24k-trace.txt",
        "rontrace-run1002b.log",
    ),
    length: 16564,
    // Item 3378's first try, held by the battery's measure to `bb82188d`:
    // count and sequence 15922 (ours 9 draws, theirs 10, at 7: ours
    // `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`). Item
    // 3383 promoted it to the AI word (`docs/AI.md` §291), and moved it
    // 15922 -> 16548 (+626): piece 1898's pivot releases (run1007b's
    // `1/64` round at `0/2005`: `sx, sy` (8760, 30264) ours, (8675, 30245)
    // theirs, `total_time` 9 against 8; agreeing after).
    count: 16548,
    sequence: 16548,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INDIAN_SAHARA_WORD: i64 = if INDIAN_SAHARA.count < INDIAN_SAHARA.sequence {
    INDIAN_SAHARA.count
} else {
    INDIAN_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn indian_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INDIAN_SAHARA.start, INDIAN_SAHARA.long) else {
        return;
    };
    eprintln!(
        "INDIAN_SAHARA {}: count {} sequence {} of {}; {}",
        INDIAN_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        INDIAN_SAHARA.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, INDIAN_SAHARA.length, "the game's last frame");
    pin_eq!(
        w.count,
        INDIAN_SAHARA.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        INDIAN_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn indian_sahara_word_is_no_other_lobby() {
    let p = &INDIAN_SAHARA;
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

/// run1007c: the word 16548's window, blocks 16542 to the game's end at
/// `GUYS=4`, `LEADERS=9` and `AMMO=5` (item 3383). run1002 dumps blocks
/// 0..36, run1002b its setup and closing blocks only, and run1007 stops at
/// 16172, so no dump on disk held the word's block.
pub(crate) const RUN1007C: &str = "gamelog-run1007c-greatsahara-indian-diff3-window-16542-16565.txt";

/// The window: block 16542 through the game's last frame 16564; the word
/// 16548's own block is 16549.
pub(crate) const WIDENING_INDIAN_SAHARA_FRAME_16548: (i64, i64) = (16542, 16564);

pub(crate) fn indian_sahara_frame_16548_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN_SAHARA.start],
        true,
        INDIAN_SAHARA.long,
        "run1007c",
        &[(RUN1007C, WIDENING_INDIAN_SAHARA_FRAME_16548.0)],
        WIDENING_INDIAN_SAHARA_FRAME_16548,
        1,
        &[16549],
        true,
    )
}

#[test]
fn run1007c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_sahara_frame_16548_window() else {
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
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 0, "keys that part in the window");
}
