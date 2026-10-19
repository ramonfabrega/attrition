//! **The battery's Indian word** (DECISIONS 72, 83; item 3335): the
//! held-out battery's lowest game once item 3333 closed the Dutch, promoted
//! by item 3314's, 3318's, 3326's, 3329's and 3331's recipe. East Indies
//! (`MAP_STYLE 18`), human Nubians (4) against **the Indians (21)** at
//! Difficulty 2, seed 12345 — the lobby item 3329 minted (`docs/AI.md`
//! §267), scored by its first try at 221 of 24,000 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run962 (a
//! `DUMP_ALL` start, blocks 0..36) and run962b (the draw stream to 24000,
//! `cover=0`, its setup and closing blocks only); the game does not end
//! itself. The word's window is run970's (`docs/AI.md` §271).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INDIAN: Lobby = Lobby {
    name: "EastIndiesIndianTier2",
    map_style: 18,
    ai_tribe: 21,
    difficulty: 2,
    start: "gamelog-run962-eastindies-indian-diff2-start.txt",
    long: (
        "gamelog-run962b-eastindies-indian-diff2-24k-trace.txt",
        "rontrace-run962b.log",
    ),
    length: 24000,
    // Item 3329, the battery's first try: count and sequence both at 221
    // (ours 5 draws, theirs 17, at 0: ours `Farms::inc_time+0x1ae`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`). Item 3335 promoted it to
    // the AI word (`docs/AI.md` §271) and moved it 221 → 1176 (a goody
    // box's own `ever_seen`, `reveal_fog`'s goody arm) → 1177 (`get_cost`'s
    // Indian ramp arm): ours 10 draws, theirs 8, at 1: ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `Guy::set_anim+0x104b`.
    count: 1177,
    sequence: 1177,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const INDIAN_WORD: i64 = if INDIAN.count < INDIAN.sequence {
    INDIAN.count
} else {
    INDIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn indian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(INDIAN.start, INDIAN.long) else {
        return;
    };
    eprintln!(
        "INDIAN {}: count {} sequence {} of {}; {}",
        INDIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, INDIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, INDIAN.length, "the game's last frame");
    pin_eq!(w.count, INDIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        INDIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn indian_word_is_no_other_lobby() {
    let p = &INDIAN;
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

/// run970: the word 221's window, blocks 37..472 at `GUYS=4` (item 3335,
/// run965's recipe from the start dump's last block, so the gap and the
/// word are one capture); run962 dumps blocks 0..36 and run962b its setup
/// and closing blocks only.
pub(crate) const RUN970: &str = "gamelog-run970-eastindies-indian-diff2-window-37-473-guys4.txt";

/// The window: block 37 through 472; the word 221's own block is 222.
pub(crate) const WIDENING_INDIAN_FRAME_221: (i64, i64) = (37, 472);

pub(crate) fn indian_frame_221_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN.start],
        true,
        INDIAN.long,
        "run970",
        &[(RUN970, WIDENING_INDIAN_FRAME_221.0)],
        WIDENING_INDIAN_FRAME_221,
        1,
        &[222],
        true,
    )
}

#[test]
fn run970_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_frame_221_window() else {
        return;
    };
    pin_eq!(w.blocks, 435, "every captured block");
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
    // Item 3335: 496 keys part on blocks 37..472, the word's own block
    // being 222. The first that is not the standing set: block 180,
    // ours' scout `1/0` (TI 69) sweeping the goody box `o 1` at (34944,
    // 38016) and re-targeting where theirs walks on — the item's
    // `ever_seen` is its own byte, 0 in the dump while the cells' `seen2`
    // read 2. **496 → 33** with the item's byte (the word 221 → 1176),
    // **33 → 32** with `get_cost`'s Indian ramp arm (who=1's timber 29
    // against 41 from block 37; the word 1176 → 1177, past the window).
    // What stands is the control's set — `SITE[n].reg`, `aggression`,
    // `form`, the groups' `held` and the pool — 30 on block 37 and two
    // late rows, who=1's `group:66.held` on 177 and `pool:64` on 222.
    pin_eq!(w.firsts.len(), 32, "keys that part in the window");
}
