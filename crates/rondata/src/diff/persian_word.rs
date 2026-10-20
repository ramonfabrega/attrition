//! **The battery's Persian word** (DECISIONS 72, 83, 84; item 3355): the
//! held-out battery's lowest open game once item 3345 closed the Lakota,
//! promoted by item 3335's recipe. Great Lakes (`MAP_STYLE 14`), human
//! Nubians (4) against **the Persians (23)** at Difficulty 2, seed 12345 —
//! minted by item 3345 under entry 72's rule (2) as amended by 84, scored
//! by its first try and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Not the coverage pair's lobbies (`diff::coverage_pair`, East
//! Indies, and `diff::sahara_coverage`, Great Sahara, both at Toughest with
//! every tech): the nation is the same, the game is not. Captures: run979
//! (a `DUMP_ALL` start, blocks 0..36) and run979b (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the game
//! ends itself on 7105 (`docs/AI.md` §279).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const PERSIAN: Lobby = Lobby {
    name: "GreatLakesPersianTier2",
    map_style: 14,
    ai_tribe: 23,
    difficulty: 2,
    start: "gamelog-run979-greatlakes-persian-diff2-start.txt",
    long: (
        "gamelog-run979b-greatlakes-persian-diff2-24k-trace.txt",
        "rontrace-run979b.log",
    ),
    length: 7105,
    // Item 3345's first try, held by the battery's measure to `a98d672b`:
    // count 6046, sequence 6046 (ours 5 draws, theirs 4, at 1: ours
    // `Unit::fight+0x9b0`, theirs `Farms::inc_time+0x1ae`). Item 3355
    // promoted it to the AI word (`docs/AI.md` §279). Item 3355, the
    // Immortals' release bays (piece 112, `sim::launch`): 6046 → 6252
    // (run984's `1/16` arrow thrown on 6012 at the Scout `0/0` from (3766,
    // 32893) in 10 frames); and `do_move`'s captain retarget searching on
    // every sixteenth `o + frame` (`005f803f`): 6252 → 6323 (run984c's
    // `targeted` 3 and 1 on `0/3` and `0/4`, so `1/10` takes `0/4`);
    // the Slingers' bays in the Persians' art style (piece 1792, piece
    // 1440's own): 6323 → 6387 (run984d's `1/22` stone at `0/3` from
    // (2504, 33053) in 5 frames); and the Immortals' special strike inside
    // `0xc0` (`Unit::fight@005fd4d0:784`): 6387 → 7105, the game's last
    // frame, count and sequence (run984d's block 6387, `0/4`'s `damage`
    // 17 after `1/10`'s blow at 144).
    count: 7105,
    sequence: 7105,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const PERSIAN_WORD: i64 = if PERSIAN.count < PERSIAN.sequence {
    PERSIAN.count
} else {
    PERSIAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn persian_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(PERSIAN.start, PERSIAN.long) else {
        return;
    };
    eprintln!(
        "PERSIAN {}: count {} sequence {} of {}; {}",
        PERSIAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, PERSIAN.difficulty, "the lobby read back");
    pin_eq!(w.last, PERSIAN.length, "the game's last frame");
    pin_eq!(w.count, PERSIAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        PERSIAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn persian_word_is_no_other_lobby() {
    let p = &PERSIAN;
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
/// run984: the word 6046's window, blocks 6020..6276 at `GUYS=4` and
/// `AMMO=5` (item 3355). run979 dumps blocks 0..36 and run979b its setup
/// and closing blocks only, so no dump on disk held the word's block.
pub(crate) const RUN984: &str =
    "gamelog-run984-greatlakes-persian-diff2-window-6020-6277-guys4-ammo.txt";

/// The window: block 6020 through 6276; the word 6046's own block is 6047.
pub(crate) const WIDENING_PERSIAN_FRAME_6046: (i64, i64) = (6020, 6276);

pub(crate) fn persian_frame_6046_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN.start],
        true,
        PERSIAN.long,
        "run984",
        &[(RUN984, WIDENING_PERSIAN_FRAME_6046.0)],
        WIDENING_PERSIAN_FRAME_6046,
        1,
        &[6047],
        true,
    )
}

#[test]
fn run984_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_frame_6046_window() else {
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
    // Item 3355: 1282 keys parted on the first try, the window opening
    // apart: the Immortal `1/16`'s arrow at the Scout `0/0`, thrown on
    // 6012, flew 11 frames against 10 from the unit's own square, where
    // run984's `AMMO` prints (3766, 32893) — piece 112 had no bays. Built:
    // 1282 → 271; then `do_move`'s captain search on every sixteenth `o +
    // frame`, which `1/10`'s 6251 re-search read through `targeted`: 271 →
    // 32. What stands: 6020's standing family alone (`form` −1 against 9,
    // the leaders' `SITE[·].reg` 1 against 0, `aggression[1]`, the army
    // pools `pool:65`/`67` and `group:65..67.held` with no `GROUPDATA`
    // printed).
    pin_eq!(w.firsts.len(), 32, "keys that part in the window");
}

/// run984d: the word 6323's window, blocks 6317..6573 at `GUYS=4` and
/// `AMMO=5` (item 3355). run984 closes on 6276, before the word.
pub(crate) const RUN984D: &str =
    "gamelog-run984d-greatlakes-persian-diff2-window-6317-6574-guys4-ammo.txt";

/// The window: block 6317 through 6573; the word 6323's own block is 6324.
pub(crate) const WIDENING_PERSIAN_FRAME_6323: (i64, i64) = (6317, 6573);

pub(crate) fn persian_frame_6323_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN.start],
        true,
        PERSIAN.long,
        "run984d",
        &[(RUN984D, WIDENING_PERSIAN_FRAME_6323.0)],
        WIDENING_PERSIAN_FRAME_6323,
        1,
        &[6324],
        true,
    )
}

#[test]
fn run984d_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_frame_6323_window() else {
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
    // Item 3355: 1136 keys parted on the first try, from 6319: the
    // Slingers `1/22` and `1/24`, who play piece 1792 (the Persians' art
    // style), threw at the citizen `0/3` from their own squares in 6
    // frames where run984d prints (2504, 33053) and (2398, 32523) in 5.
    // Built: 1136 → 594, the next parting 6387, where `1/10` at 144 from
    // `0/4` struck it inline under `CHAR_ATTACKSPECIAL` (`0/4`'s `damage`
    // 17 against 13). Built: 594 → 31, 6317's standing family alone.
    pin_eq!(w.firsts.len(), 31, "keys that part in the window");
}

/// The closing state's window: run979b's closing block alone, the long's
/// last frame 7105 writing it (item 3355).
pub(crate) const WIDENING_PERSIAN_CLOSING: (i64, i64) = (7104, 7107);

/// **The lobby's closing state** (item 3355, DECISIONS 55): run979b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 7105.
#[test]
fn run979b_persian_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(PERSIAN.long.0),
        dump(PERSIAN.start),
        trace(PERSIAN.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Persian closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 7106, "the closing block");
    pin_eq!(e.compared, 38, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}
