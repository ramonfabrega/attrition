//! **The battery's Dutch word at Great Sahara** (DECISIONS 72, 83, 84;
//! item 3369): the held-out battery's lowest open game on the measure of
//! `efc5d4c1`, promoted by item 3335's recipe. Great Sahara (`MAP_STYLE 7`),
//! human Nubians (4) against **the Dutch (22)** at Difficulty 3, seed 12345
//! — minted by item 3365 (entry 72's rule (2): a nation held only by a
//! closed game on another map), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Not
//! `diff::dutch_word`, which is the closed East Indies game at Difficulty
//! 4. Captures: run990 (a `DUMP_ALL` start, blocks 0..36) and run990b (the
//! draw stream to the game's end, `cover=0`, its setup and closing blocks
//! only); the game ends itself on 16109 (`docs/AI.md` §285).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const DUTCH_SAHARA: Lobby = Lobby {
    name: "GreatSaharaDutchTier3",
    map_style: 7,
    ai_tribe: 22,
    difficulty: 3,
    start: "gamelog-run990-greatsahara-dutch-diff3-start.txt",
    long: (
        "gamelog-run990b-greatsahara-dutch-diff3-24k-trace.txt",
        "rontrace-run990b.log",
    ),
    length: 16109,
    // Item 3365's first try, held by the battery's measure to `efc5d4c1`:
    // count 311, sequence 311 (ours 7 draws, theirs 6, at 1: ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Farms::inc_time+0x1ae`). Item 3369 promoted it to the AI word
    // (`docs/AI.md` §285).
    count: 311,
    sequence: 311,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const DUTCH_SAHARA_WORD: i64 = if DUTCH_SAHARA.count < DUTCH_SAHARA.sequence {
    DUTCH_SAHARA.count
} else {
    DUTCH_SAHARA.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn dutch_sahara_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(DUTCH_SAHARA.start, DUTCH_SAHARA.long) else {
        return;
    };
    eprintln!(
        "DUTCH_SAHARA {}: count {} sequence {} of {}; {}",
        DUTCH_SAHARA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, DUTCH_SAHARA.difficulty, "the lobby read back");
    pin_eq!(w.last, DUTCH_SAHARA.length, "the game's last frame");
    pin_eq!(
        w.count,
        DUTCH_SAHARA.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        DUTCH_SAHARA.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn dutch_sahara_word_is_no_other_lobby() {
    let p = &DUTCH_SAHARA;
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

/// run995: the word 311's window, blocks 305..561 at `GUYS=4`, `LEADERS=9`
/// and `GOODS=3` (item 3369). run990 dumps blocks 0..36 and run990b its
/// setup and closing blocks only, so no dump on disk held the word's block.
pub(crate) const RUN995: &str = "gamelog-run995-greatsahara-dutch-diff3-window-305-562.txt";

/// The window: block 305 through 561; the word 311's own block is 312.
pub(crate) const WIDENING_DUTCH_SAHARA_FRAME_311: (i64, i64) = (305, 561);

pub(crate) fn dutch_sahara_frame_311_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH_SAHARA.start],
        true,
        DUTCH_SAHARA.long,
        "run995",
        &[(RUN995, WIDENING_DUTCH_SAHARA_FRAME_311.0)],
        WIDENING_DUTCH_SAHARA_FRAME_311,
        1,
        &[312],
        true,
    )
}

#[test]
fn run995_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_sahara_frame_311_window() else {
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
    // Item 3369: 433 keys on the first try, the standing family on 305
    // (`form` −1 against 9, the leaders' `SITE[·].reg` 1 against 0,
    // `aggression`, `group:64`/`65.held` and `pool:65` with no `GROUPDATA`
    // printed) and then, on block 310, the Armed Merchant `1/1`
    // (`MERCHANTDUTCH`, type 62) drafted into who=1's army in theirs —
    // `group` −1 against 64, `orders.len` 0 against 1 (`ATTACK_TO`
    // (41304, 17304)), `form_mod` −1 against 50 — the frame after its
    // unpack seated it.
    pin_eq!(w.firsts.len(), 433, "keys that part in the window");
}
