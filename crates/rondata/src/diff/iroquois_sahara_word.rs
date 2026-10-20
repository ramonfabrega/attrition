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
    // the AI word (`docs/AI.md` §288), and moved it 13757 -> 13810 (+53):
    // `HEAVYARCHERS-NA`'s `CHAR_ATTACK2` release node (run1001b's block
    // 13750: `1/35`'s round leaves from (8976, 31176), `total_time` 8,
    // where ours left the guy's (9048, 31128), 9; run1001's block 13757:
    // the Scout `0/0`'s `hits:damage` 16 ours, 21 theirs, and a second
    // `FLEE_TO`; agreeing after). 13810: ours 14 draws, theirs 15, at 10:
    // ours `Farms::inc_time+0x1ae`, theirs `Unit::close+0xcb6`.
    count: 13810,
    sequence: 13810,
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

/// run1001: the word 13757's window, blocks 13751..14007 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3375). run994 dumps blocks 0..36 and
/// run994b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN1001: &str =
    "gamelog-run1001-greatsahara-iroquois-diff3-window-13751-14008.txt";

/// The window: block 13751 through 14007; the word 13757's own block is
/// 13758.
pub(crate) const WIDENING_IROQUOIS_SAHARA_FRAME_13757: (i64, i64) = (13751, 14007);

pub(crate) fn iroquois_sahara_frame_13757_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[IROQUOIS_SAHARA.start],
        true,
        IROQUOIS_SAHARA.long,
        "run1001",
        &[(RUN1001, WIDENING_IROQUOIS_SAHARA_FRAME_13757.0)],
        WIDENING_IROQUOIS_SAHARA_FRAME_13757,
        1,
        &[13758],
        true,
    )
}

#[test]
fn run1001_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = iroquois_sahara_frame_13757_window() else {
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
    // Item 3375: 1404 keys on the first try. The window opens apart on the
    // standing family on 13751 (`form`, the leaders' `SITE[·].reg`,
    // `aggression[1]`, the army pools and `group:N.held` with no
    // `GROUPDATA` printed); then, on the word's block 13757, the human Scout
    // `0/0` (TI 69) — `hits:damage` 16 ours, 21 theirs, `damage_frame`
    // 13711/13756, `damage_o` 30/35, `orders.len` 0/1 (a second `FLEE_TO`
    // to (7944, 33384)): `1/35`'s round (`HEAVYARCHERS-NA`, piece 1531)
    // struck it on its arrival frame in theirs and the frame after in ours.
    // Built (piece 1531's `CHAR_ATTACK2` release node): 1404 -> 1109.
    pin_eq!(w.firsts.len(), 1109, "keys that part in the window");
}
