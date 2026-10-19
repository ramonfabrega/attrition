//! **The battery's fifth word** (DECISIONS 72, 83; item 3329): the
//! held-out battery's lowest game after the Greeks (item 3326's), promoted
//! by item 3314's and 3318's recipe. East Indies (`MAP_STYLE 18`), human
//! Nubians (4) against **the Dutch (22)** at Difficulty 4, seed 12345 —
//! the lobby item 3107 minted (`docs/AI.md` §246), scored by its first try
//! at 1567 of 24,000 and promoted out of the battery here, so it is a
//! scored lobby from this item (`battery::SCORED_LOBBIES`) and no longer
//! held out. Captures: run915 (a `DUMP_ALL` start, blocks 0..36) and run917
//! (the draw stream to 24000, `cover=0`, its setup and closing blocks
//! only); the game does not end itself. The word's windows are run963's
//! (`docs/AI.md` §267).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const DUTCH: Lobby = Lobby {
    name: "EastIndiesDutchTier4",
    map_style: 18,
    ai_tribe: 22,
    difficulty: 4,
    start: "gamelog-run915-eastindies-dutch-diff4-start.txt",
    long: (
        "gamelog-run917-eastindies-dutch-diff4-24k-trace.txt",
        "rontrace-run917.log",
    ),
    length: 24000,
    // Item 3107, the battery's first try: count and sequence both at 1567
    // (ours 6 draws, theirs 5, at index 2: ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Farms::inc_time+0x1ae`). Item 3329
    // promoted it to the AI word (`docs/AI.md` §267). **Item 3329 moved it
    // 1567 → 1769** with the Dutch merchant's leg of `Unit::think`'s
    // auto-attack arm (`think@005f6e40:145`, `docs/MERCHANT.md` §4): the
    // packed `1/2` (`MERCHANTDUTCH`) asks `think_merchant` one frame in
    // thirty-two, so on 1566 it took its `MOVE_TO` (31800, 37176) in theirs
    // and stood idle in ours, whose only call was the tail's one in 128
    // (run963's block 1567: `orders.len` 0 against 1). **And 1769 → 5583**
    // with the unit piece table keyed without case
    // (`rondata::artdata::graph_types`): the Dutch caravan `1/14`
    // (`CARADUTCH`, `GRAPH` `ArmedCaravan`), born on 1764, played no piece
    // in ours — `gpiece` 0 against theirs' 10 and 12682 on block 1765, its
    // clock 33 against 31 — and on 1769 its idle draw fell a frame apart.
    // At 5583 ours 10 draws, theirs 9, at 4: ours `Leader::make_stuff+0x63d`,
    // theirs `GameDaemon::calc_market+0x54`, past run963's last block.
    count: 5583,
    sequence: 5583,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const DUTCH_WORD: i64 = if DUTCH.count < DUTCH.sequence {
    DUTCH.count
} else {
    DUTCH.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn dutch_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(DUTCH.start, DUTCH.long) else {
        return;
    };
    eprintln!(
        "DUTCH {}: count {} sequence {} of {}; {}",
        DUTCH.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, DUTCH.difficulty, "the lobby read back");
    pin_eq!(w.last, DUTCH.length, "the game's last frame");
    pin_eq!(w.count, DUTCH.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        DUTCH.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn dutch_word_is_no_other_lobby() {
    let p = &DUTCH;
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

/// run963: the word 1567's window, blocks 1561..1817 at `GUYS=4` (item
/// 3329, run958e's recipe); run915 dumps blocks 0..36 and run917 its setup
/// and closing blocks only.
pub(crate) const RUN963: &str = "gamelog-run963-eastindies-dutch-diff4-window-1561-1818-guys4.txt";

/// The window: block 1561 through 1817; the word 1567's own block is 1568.
pub(crate) const WIDENING_DUTCH_FRAME_1567: (i64, i64) = (1561, 1817);

pub(crate) fn dutch_frame_1567_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963",
        &[(RUN963, WIDENING_DUTCH_FRAME_1567.0)],
        WIDENING_DUTCH_FRAME_1567,
        1,
        &[1568],
        true,
    )
}

#[test]
fn run963_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_1567_window() else {
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
    pin_eq!(w.firsts.len(), 38, "keys that part in the window");
    // Item 3329: 531 keys part on blocks 1561..1817, the word's own block
    // being 1568. Standing from 1561 (36): the control's set —
    // `SITE[n].reg`, `form`, the groups' `held`, the pool and both
    // leaders' `aggression`. The first that is not: block 1567, who=1's
    // Dutch merchant `1/2` (`TypeIndex` 62, land) holds a `MOVEORDER` to
    // (31800, 37176) in theirs and nothing in ours (`orders.len` 0/1,
    // `orders_x` 40344/31800), then its path, idle and figures.
    //
    // **531 → 239** with the Dutch merchant's leg of the auto-attack arm:
    // the order is on both sides on 1567. The first that parted next was
    // block 1765, the newborn caravan `1/14`'s `g.gpiece[0]` 0/10 and
    // `g.gpiece[1]` 0/12682, `g.end_time[0]` 33/31. **239 → 38** with the
    // piece table keyed without case: both read 10 and 12682. What stands
    // is the control's set and its two late rows, who=1's `SITE[6].reg` on
    // 1576 and `group:64.held` on 1577.
}

/// run963b: the word 5583's window, blocks 5577..5833 at run963's detail
/// (item 3329); run963 ends on block 1817 and run917 dumps its setup and
/// closing blocks only.
pub(crate) const RUN963B: &str =
    "gamelog-run963b-eastindies-dutch-diff4-window-5577-5834-guys4.txt";

/// The window: block 5577 through 5833; the word 5583's own block is 5584.
pub(crate) const WIDENING_DUTCH_FRAME_5583: (i64, i64) = (5577, 5833);

pub(crate) fn dutch_frame_5583_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[DUTCH.start],
        true,
        DUTCH.long,
        "run963b",
        &[(RUN963B, WIDENING_DUTCH_FRAME_5583.0)],
        WIDENING_DUTCH_FRAME_5583,
        1,
        &[5584],
        true,
    )
}

#[test]
fn run963b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = dutch_frame_5583_window() else {
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
    pin_eq!(w.firsts.len(), 0, "keys that part in the window");
}
