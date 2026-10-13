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
    // Item 2049: 3834 -> 3835 (ours 46 draws, theirs 40, index 0). The
    // extra idle roll on 3834 was the raiding `1/113`'s: `think`'s
    // off-phase `near_o` arm hands it the ceiling city `0/2000` on 3647
    // (`docs/AI.md` section 205).
    count: 3835,
    sequence: 3835,
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

pub(crate) fn mongol_frame_1354_window() -> Option<harness::tests::Widened> {
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
    let Some(w) = mongol_frame_1354_window() else {
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
    pin_eq!(w.firsts.len(), 106, "initial run779 baseline");
    // Item 1744: 1406 keys part on blocks 1349..1605 before the build, the
    // word's own block being 1355 (frame 1354: ours 5 draws, theirs 7, index
    // 1; building `1/2024`'s `queue[0].job_counter` 17200 against 17150 and
    // on 1355 `num_units[206]` 0 against 1, a unit `1/42` born in theirs).
    // The Mongol stable arm of `train_time` (`MONGOL_STABLE_SPEED`, 20 percent
    // off an attacker trained in the Stable line) closes it: 1406 -> 109, and
    // the word moves 1354 -> 2207, past this window (`docs/AI.md` §193).
}

/// run780: the word 2207's window, blocks 2201..2458 at the long's detail (item 1768).
pub(crate) const RUN780: &str = "gamelog-run780-eastindies-mongol-alltech-window-2201-2459.txt";

/// The window: block 2201 through 2458; the word 2207's own block is 2208.
pub(crate) const WIDENING_MONGOL_FRAME_2207: (i64, i64) = (2201, 2458);

/// run780's blocks around the word 2207 (item 1768), the window the word left.
pub(crate) fn mongol_frame_2207_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run780",
        &[(RUN780, WIDENING_MONGOL_FRAME_2207.0)],
        WIDENING_MONGOL_FRAME_2207,
        1,
        &[2208],
        true,
    )
}

#[test]
fn run780_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_frame_2207_window() else {
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
    pin_eq!(w.firsts.len(), 135, "initial run780 baseline");
    // Item 1768: 897 keys part on blocks 2201..2457, 129 of them on the
    // window's first block (nothing dumped parts on the word's own block
    // 2208). The word 2207 (ours 9 draws, theirs 8, index 8) is the idle
    // anim draw of the citizen `1/72`, which ours stands idle from 2178 and
    // theirs walks to (35736, 24984) (`docs/AI.md` §194). Item 1845 built
    // `Unit::think_carry`'s worker arm (`docs/AI.md` §195): the Freighter
    // now walks there too, the word moved 2207 -> 2838, and the window's
    // keys fell 897 -> 136 (the word's delta; the 897 keys stand in §194).
}

/// run788: the word 2838's window, blocks 2832..3089 at the long's detail (item 1868).
pub(crate) const RUN788: &str = "gamelog-run788-eastindies-mongol-alltech-window-2832-3090.txt";

/// The window: block 2832 through 3089; the word 2838's own block is 2839.
pub(crate) const WIDENING_MONGOL_FRAME_2838: (i64, i64) = (2832, 3090);

/// run788's blocks around the word 2838 (item 1868), the window the word left.
pub(crate) fn mongol_frame_2838_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run788",
        &[(RUN788, WIDENING_MONGOL_FRAME_2838.0)],
        WIDENING_MONGOL_FRAME_2838,
        1,
        &[2839],
        true,
    )
}

#[test]
fn run788_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_frame_2838_window() else {
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
        w.firsts.len(),
        137,
        "run788 after item 1868's sea break test (163 -> 151, item 1937: the patrol look and the strike release)"
    );
    // Item 1868: 833 keys part on blocks 2832..3088 before the build, 128 of them
    // on the window's first block; the word 2838 (ours 101 draws, theirs 16, index 1)
    // is the Fishermen `1/122`'s arrival, whose final path point theirs planned
    // (32277, 44997) against ours (32276, 44952) (`docs/AI.md` section 196).
    // Item 1948: 151 -> 149 keys, `1/27`'s `path[63].flags`/`.to` (the BMP's
    // first leg, flags 4 against 0 on 3067) gone with the transport bit's
    // hand-down (`docs/AI.md` section 200).
}

/// run799: the word 3131's window, blocks 3125..3382 at the long's detail (item 1948).
pub(crate) const RUN799: &str = "gamelog-run799-eastindies-mongol-alltech-window-3125-3383.txt";

/// The window: block 3125 through 3382; the word 3131's own block is 3132.
pub(crate) const WIDENING_MONGOL_FRAME_3131: (i64, i64) = (3125, 3383);

/// run799's window, the successor pair's word's until item 2001.
pub(crate) fn mongol_frame_3131_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run799",
        &[(RUN799, WIDENING_MONGOL_FRAME_3131.0)],
        WIDENING_MONGOL_FRAME_3131,
        1,
        &[3132],
        true,
    )
}

#[test]
fn run799_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_frame_3131_window() else {
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
    pin_eq!(w.firsts.len(), 165, "run799 baseline");
    // Item 1948: 5,308 keys part on blocks 3125..3382 before the build, 182 of
    // them on the window's first block; the word 3131 (ours 62 draws, theirs
    // 65, index 0) is the BMP `1/27`'s cast of the transport (kind 14, spell
    // 0x28a) and the Freighter `1/165` born inside it on 3132, which ours had
    // not pushed because `do_move` refused the first leg's transport waypoint
    // on 3067 and dropped its `flags & 4` instead of handing it to the next
    // leg (`docs/AI.md` section 200). With it the word moved 3131 -> 3188
    // and the window's keys fell 5,308 -> 2,210 (the word's delta; the
    // 5,308 stand in section 200).
    // Item 1982: 2,198 -> 165 keys on the merged tree (2,210 -> 177 before 1937's
    // re-pin), and the word 3188 -> 3632 (ours 193 draws,
    // theirs 192, index 35). The AA missile `1/107`'s resumed 48-grid search
    // gave up on 3180 and ours left its stack at 61 where the original
    // pops the non-final top (60): `find_upath`'s failure teardown is guarded
    // by the suspended flag, which every failure tail of `astar_path` clears,
    // not by "resumed" (`docs/AI.md` section 201). 147 of the 165 stand on
    // the window's first block (the leaders' counts), the rest are the
    // freighters' `form` and the births (3126, 3132, 3134, 3180, 3183).    // Item 2001: the word 3632 -> 3834 (ours 169 draws, theirs 167, index
    // 9). The Aegis Cruiser `1/113`, a computer's ship on the raid stance,
    // took the finished `0/2000` at score 1 and fired on 3632; the
    // original's `compare_target` answers 0 for it (`0064e7a5`..`0064e81f`,
    // run810's packet) and `find_nearby_target` never takes a zero
    // (`docs/AI.md` section 203).
}

/// run809: the word 3632's window, blocks 3626..3883 at the long's detail (item 2001).
pub(crate) const RUN809: &str = "gamelog-run809-eastindies-mongol-alltech-window-3626-3884.txt";

/// The window: block 3626 through 3883; the word 3632's own block is 3633.
pub(crate) const WIDENING_MONGOL_FRAME_3632: (i64, i64) = (3626, 3884);

/// **The successor pair's word's window** (`AI_WORDS`' `Successor pair` row).
pub(crate) fn mongol_word_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL.start],
        true,
        MONGOL.long,
        "run809",
        &[(RUN809, WIDENING_MONGOL_FRAME_3632.0)],
        WIDENING_MONGOL_FRAME_3632,
        1,
        &[3633],
        true,
    )
}

#[test]
fn run809_s_word_frame_is_widened_whole() {
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
    // The word 3632's block is 3633. Item 2001: 357 keys after the build
    // (957 before it), 160 on the first block (the leaders' counts); the
    // first of `1/113` is now `orders.len` on 3648, theirs 1 (`docs/AI.md`
    // section 203.5).
    // Item 2049: 357 -> 353 — `1/113`'s thirteen rows (`orders.len` on
    // 3648, `pos` on 3649, `heading` on 3833, `angle` on 3834) leave the
    // window whole; nine of `1/167` open on 3851, past the new word 3835
    // (section 205).
    pin_eq!(w.firsts.len(), 353, "run809 baseline");
}
