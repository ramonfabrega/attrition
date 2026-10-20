//! **The battery's East Indies Spanish word** (DECISIONS 72, 83, 84; item
//! 3367): the held-out battery's lowest open game on the measure of
//! `9e63ec11`, promoted by item 3335's recipe. East Indies (`MAP_STYLE
//! 18`), human Nubians (4) against **the Spanish (9)** at Difficulty 3,
//! seed 12345 — minted by item 3360 under entry 72's rule (2), scored by
//! its first try and promoted out of the battery here, so it is a scored
//! lobby from this item (`battery::SCORED_LOBBIES`) and no longer held
//! out. Captures: run986 (a `DUMP_ALL` start) and run986b (the draw
//! stream to 24000, `cover=0`, its setup and closing blocks only); the
//! game does not end itself (`docs/AI.md` §284). `diff::spanish_word` is
//! the nation's other game, the closed Great Sahara one; the two are kept
//! apart.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_EAST: Lobby = Lobby {
    name: "EastIndiesSpanishTier3",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 3,
    start: "gamelog-run986-eastindies-spanish-diff3-start.txt",
    long: (
        "gamelog-run986b-eastindies-spanish-diff3-24k-trace.txt",
        "rontrace-run986b.log",
    ),
    length: 24000,
    // Item 3360's first try, held by the battery's measure to `9e63ec11`:
    // count 9028, sequence 9028 (ours 2 draws, theirs 12, at 0: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`). Item 3367 promoted it to
    // the AI word (`docs/AI.md` §284). **9028 → 12756** (item 3367): the
    // danger pass asks a building's own `ever_seen` (`BuildData::is_seen`),
    // not the fog a Spanish leader sees whole — run993b's block 8959,
    // `danger[1]` at half-cell (4, 4) ours 125, theirs 60, and the scout
    // `1/0` chose cell (7, 11) over theirs' (9, 8).
    count: 12756,
    sequence: 12756,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_EAST_WORD: i64 = if SPANISH_EAST.count < SPANISH_EAST.sequence {
    SPANISH_EAST.count
} else {
    SPANISH_EAST.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_east_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH_EAST.start, SPANISH_EAST.long) else {
        return;
    };
    eprintln!(
        "SPANISH_EAST {}: count {} sequence {} of {}; {}",
        SPANISH_EAST.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, SPANISH_EAST.difficulty, "the lobby read back");
    pin_eq!(w.last, SPANISH_EAST.length, "the game's last frame");
    pin_eq!(
        w.count,
        SPANISH_EAST.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SPANISH_EAST.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_east_word_is_no_other_lobby() {
    let p = &SPANISH_EAST;
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

/// run993: the word 9028's window, blocks 9022..9278 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3367). run986 dumps blocks 0..36 and
/// run986b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN993: &str =
    "gamelog-run993-eastindies-spanish-diff3-window-9022-9279-leaders.txt";

/// The window: block 9022 through 9278; the word 9028's own block is 9029.
pub(crate) const WIDENING_SPANISH_EAST_FRAME_9028: (i64, i64) = (9022, 9278);

pub(crate) fn spanish_east_frame_9028_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_EAST.start],
        true,
        SPANISH_EAST.long,
        "run993",
        &[(RUN993, WIDENING_SPANISH_EAST_FRAME_9028.0)],
        WIDENING_SPANISH_EAST_FRAME_9028,
        1,
        &[9029],
        true,
    )
}

#[test]
fn run993_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_east_frame_9028_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3367: 519 keys on the first try, the window opening with the
    // Scout `1/0` (TI 69, land) already apart — ours walking an
    // `ExploreTo` to (5880, 8952), theirs at (7588, 6631) on its way to
    // (7416, 6648). Walked back: both chose on 8959 with the same draws
    // over the same three cells around the human's city, and the score's
    // danger term parted — `danger[1]` at half-cell (4, 4) ours 125,
    // theirs 60 (run993b), the human's buildings unhalved for a viewer
    // whose fog `was_seen` lights whole (`leader_flags & 0x1000`, the
    // Spanish bonus) and whose `ever_seen` bit they do not carry. Built:
    // `danger_build_is_seen` is `build_is_seen`. 519 → 49, the standing
    // family alone (`SITE[·].reg`, `form`, `aggression`, the army `pool:n`
    // and `group:n.held`, the newborn `g.angle[0]`).
    pin_eq!(w.firsts.len(), 49, "keys that part in the window");
}

/// run993c: the word 12756's window, blocks 12750..13006 at `GUYS=4`,
/// `LEADERS=9` and `AMMO=5` (item 3367). run993 closes on 9278, before the
/// word.
pub(crate) const RUN993C: &str =
    "gamelog-run993c-eastindies-spanish-diff3-window-12750-13007-ammo.txt";

/// The window: block 12750 through 13006; the word 12756's own block is
/// 12757.
pub(crate) const WIDENING_SPANISH_EAST_FRAME_12756: (i64, i64) = (12750, 13006);

pub(crate) fn spanish_east_frame_12756_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_EAST.start],
        true,
        SPANISH_EAST.long,
        "run993c",
        &[(RUN993C, WIDENING_SPANISH_EAST_FRAME_12756.0)],
        WIDENING_SPANISH_EAST_FRAME_12756,
        1,
        &[12757],
        true,
    )
}

#[test]
fn run993c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_east_frame_12756_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
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
    // Item 3367, the first try on 12756: 765 keys. The Caravel `1/49` (TI
    // 325, sea, born a Bark on 10369 out of a Dock) parts first on 12752,
    // `cavarch_o` ours −2, theirs −1: theirs' 12751 search took `0/2004`.
    pin_eq!(w.firsts.len(), 765, "keys that part in the window");
}
