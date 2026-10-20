//! **The battery's Iroquois word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3375): the held-out battery's lowest open game on the measure of
//! `ac79accd`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Iroquois (18)** at Difficulty 3, seed
//! 12345 — minted by item 3369 (entry 72's rule (2): a nation held only by a
//! closed game on another map), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::iroquois_word`, which is the closed Great Lakes game at the same
//! difficulty. Captures: run994 (a `DUMP_ALL` start) and run994b (the draw
//! stream to the game's end, `cover=0`, its setup and closing blocks only);
//! the game ends itself on 16371 (`docs/AI.md` §288).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const IROQUOIS_SAHARA: Lobby = Lobby {
    name: "GreatSaharaIroquoisTier3",
    map_style: 7,
    ai_tribe: 18,
    difficulty: 3,
    start: "gamelog-run994-greatsahara-iroquois-diff3-start.txt",
    long: (
        "gamelog-run994b-greatsahara-iroquois-diff3-24k-trace.txt",
        "rontrace-run994b.log",
    ),
    length: 16371,
    // Item 3369's first try, held by the battery's measure to `ac79accd`:
    // count 13757, sequence 13757 (ours 13 draws, theirs 11, at 1: ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `GameAccess::rnd+0x20 < Unit::do_job+0x67`). Item 3375 promoted it to
    // the AI word (`docs/AI.md` §288).
    count: 13757,
    sequence: 13757,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const IROQUOIS_SAHARA_WORD: i64 = if IROQUOIS_SAHARA.count < IROQUOIS_SAHARA.sequence {
    IROQUOIS_SAHARA.count
} else {
    IROQUOIS_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn iroquois_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(IROQUOIS_SAHARA.start, IROQUOIS_SAHARA.long) else {
        return;
    };
    eprintln!(
        "IROQUOIS_SAHARA {}: count {} sequence {} of {}; {}",
        IROQUOIS_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        IROQUOIS_SAHARA.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, IROQUOIS_SAHARA.length, "the game's last frame");
    pin_eq!(
        w.count,
        IROQUOIS_SAHARA.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        IROQUOIS_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn iroquois_sahara_word_is_no_other_lobby() {
    let p = &IROQUOIS_SAHARA;
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
