//! **The successor pair** (DECISIONS 53 §2, 61 §6, item 1728): the coverage
//! pair closed at its trace's end, and its successor is the lobby that
//! reaches census rows no trace has entered. East Indies (`MAP_STYLE 18`),
//! human Nubians (4) against **Mongols (17)** at **Difficulty 3**, seed
//! 12345, with the coverage pair's `STARTING_TECHNOLOGY 8` and
//! `STARTING_RESOURCES 7`: the nation and the difficulty are what it
//! varies (`docs/AI.md` §190). Captures: run774 (a `DUMP_ALL` start) and
//! run775 (the draw stream to the game's end, `cover=0`).

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;

pub(crate) const MONGOL: Pair = Pair {
    name: "EastIndiesMongolAllTech",
    map_style: 18,
    ai_tribe: 17,
    difficulty: 3,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run774-eastindies-mongol-alltech-start.txt",
    long: (
        "gamelog-run775-eastindies-mongol-alltech-trace.txt",
        "rontrace-run775.log",
    ),
    length: 5056,
    count: 2207,
    sequence: 2207,
};

/// The pair's word as the handoff's `Successor pair:` line and `AI_WORDS` carry it.
pub(crate) const MONGOL_PAIR_WORD: i64 = if MONGOL.count < MONGOL.sequence {
    MONGOL.count
} else {
    MONGOL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn mongol_pair_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(MONGOL.start, MONGOL.long) else {
        return;
    };
    eprintln!(
        "MONGOL {}: count {} sequence {} of {}; {}",
        MONGOL.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, MONGOL.difficulty, "the lobby read back");
    pin_eq!(w.last, MONGOL.length, "the game's last frame");
    pin_eq!(w.count, MONGOL.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        MONGOL.sequence,
        "the draw sequence's first parting"
    );
}

/// run776: the lobby's blocks 1..34 at the long's detail, the word 11's
/// widening (item 1728).
pub(crate) const RUN776: &str = "gamelog-run776-eastindies-mongol-alltech-window-1-34.txt";

/// The window: block 1 through 33; the word's own block is 12.
pub(crate) const WIDENING_MONGOL_FRAME_11: (i64, i64) = (1, 33);

pub(crate) fn mongol_frame_11_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run776",
        &[(RUN776, 1)],
        WIDENING_MONGOL_FRAME_11,
        1,
        &[12],
        true,
    )
}

#[test]
fn run776_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_frame_11_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every captured block");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 1728: 128 keys part on blocks 1..33 of the Mongol start's
    // window, the word's own block being 12 (frame 11, before 1737: ours 556 draws,
    // theirs 629, index 3; the leader's MAKE list holds the Oil Well's
    // category first in theirs, `docs/AI.md` §190).
    pin_eq!(w.firsts.len(), 46, "initial run776 baseline");
    // Item 1737: the Mongol food term closes the leader economy (frame 3's
    // `econ`/`rate`, food income 435 against 160 from frame 1) and with it
    // the make list's order on 11; 128 -> 46, the control's start set. The
    // word moves 11 -> 1354, outside this window (`docs/AI.md` §191).
}

/// run779: the word 1354's window, blocks 1349..1605 at the long's detail (item 1744).
pub(crate) const RUN779: &str = "gamelog-run779-eastindies-mongol-alltech-window-1349-1606.txt";

/// The window: block 1349 through 1605; the word 1354's own block is 1355.
pub(crate) const WIDENING_MONGOL_FRAME_1354: (i64, i64) = (1349, 1605);

/// **The successor pair's word's window** (`AI_WORDS`' `Successor pair` row).
pub(crate) fn mongol_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run779",
        &[(RUN779, WIDENING_MONGOL_FRAME_1354.0)],
        WIDENING_MONGOL_FRAME_1354,
        1,
        &[1355],
        true,
    )
}

#[test]
fn run779_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_word_window() else {
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
    pin_eq!(w.firsts.len(), 109, "initial run779 baseline");
    // Item 1744: 1406 keys part on blocks 1349..1605 before the build, the
    // word's own block being 1355 (frame 1354: ours 5 draws, theirs 7, index
    // 1; building `1/2024`'s `queue[0].job_counter` 17200 against 17150 and
    // on 1355 `num_units[206]` 0 against 1, a unit `1/42` born in theirs).
    // The Mongol stable arm of `train_time` (`MONGOL_STABLE_SPEED`, 20 percent
    // off an attacker trained in the Stable line) closes it: 1406 -> 109, and
    // the word moves 1354 -> 2207, past this window (`docs/AI.md` §193).
}
