//! **The Egyptians at a held Classical age, East Indies** (DECISIONS 90,
//! 91, 93, item 3585): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Egyptians (7)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical: `STARTING_TECHNOLOGY
//! 1`, `ENDING_TECHNOLOGY 1` — the Greek Classical pair's lobby with the
//! nation moved. The Egyptians' other lobbies are the held-out
//! `EastIndiesEgyptianTier3`, `(18, 7, 3)`, and `egyptian_word`'s
//! `GreatSaharaEgyptianTier4`, `(7, 7, 4)` — another difficulty and
//! another map. Captures: run1229 (a `DUMP_ALL` start) and run1230 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try is
//! in `docs/AI.md` §318.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const EGYPTIAN_CLASSICAL: Pair = Pair {
    name: "EastIndiesEgyptianClassical",
    map_style: 18,
    ai_tribe: 7,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1229-eastindies-egyptian-classical-start.txt",
    long: (
        "gamelog-run1230-eastindies-egyptian-classical-trace.txt",
        "rontrace-run1230.log",
    ),
    // The game ended itself on 6748 (closing block 6749).
    length: 6748,
    // Item 3585, first try: frame 5961 of 6748 by the count and the
    // sequence. On 5961 ours spent 92 draws against theirs' 89, parting at
    // index 77 — ours `Guy::init_real+0x52`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` section 318).
    count: 5961,
    sequence: 5961,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const EGYPTIAN_CLASSICAL_PAIR_WORD: i64 =
    if EGYPTIAN_CLASSICAL.count < EGYPTIAN_CLASSICAL.sequence {
        EGYPTIAN_CLASSICAL.count
    } else {
        EGYPTIAN_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn egyptian_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN_CLASSICAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 1, and the long is scoreable.
#[test]
fn egyptian_classical_pair_captures_say_their_lobby() {
    let p = &EGYPTIAN_CLASSICAL;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Egyptian Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery's Egyptians are the held-out
/// `EastIndiesEgyptianTier3`, `(18, 7, 3)` — another difficulty — and the
/// scored `GreatSaharaEgyptianTier4`, `(7, 7, 4)` — another map (item
/// 3585).
#[test]
fn egyptian_classical_pair_is_no_other_lobby() {
    let p = &EGYPTIAN_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let e = &super::egyptian_word::EGYPTIAN;
    assert_ne!((e.map_style, e.ai_tribe, e.difficulty), triple);
}

/// run1230b: the word 5961's window, blocks 5956..6212 at the long's
/// detail — six blocks before the word's block 5962 and 250 after (item
/// 3585); run1229 dumps blocks 0..2 alone and the long's stride prints 5901
/// and 6001.
pub(crate) const RUN1230B: &str =
    "gamelog-run1230b-eastindies-egyptian-classical-window-5956-6212.txt";

/// The window: block 5956 through 6212; the word 5961's own block is 5962.
pub(crate) const WIDENING_EGYPTIAN_CLASSICAL_FRAME_5961: (i64, i64) = (5956, 6212);

/// **The word 5961's window**: run1230b walked from run1229's start with
/// the recorder on (item 3585).
pub(crate) fn egyptian_classical_frame_5961_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_CLASSICAL.start],
        true,
        EGYPTIAN_CLASSICAL.long,
        "run1230b",
        &[(RUN1230B, WIDENING_EGYPTIAN_CLASSICAL_FRAME_5961.0)],
        WIDENING_EGYPTIAN_CLASSICAL_FRAME_5961,
        1,
        &[5962],
        true,
    )
}

#[test]
fn egyptian_classical_pair_run1230b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_classical_frame_5961_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3585, the first walk: **4874** keys. Block 5956, the window's
    // first, is apart on who=1's economy beside the standing families
    // (`form`, `SITE[i].reg`, `aggression`, groups' `held`/`pool`): the
    // buckets (food ours 13346 against theirs 13286, timber 11166/11161,
    // wealth 17822/17852, wealth escrow 102/103), `num_queued[11]` (the
    // Merchant) ours 1 against 0 and `num_queued[9]` (the Caravan) 0
    // against 1, and the queue `1/2020`'s head, `type` ours 61 (Merchant)
    // against theirs 59 (Caravan), `cost` (50, 30) against (60, 55). Its
    // `job_counter` parts on 5961 (18198 against 18200); on the word's
    // block 5962 ours births the Merchant `1/167` (TypeIndex 61, land) the
    // queue holds and theirs does not — `queued` 0 against 1,
    // `num_units[11]` 3 against 2, `active` 125/124, `control` 136/135.
    // Theirs' `1/167` is a Javelineers (TypeIndex 83, uid 328), born on
    // 5977.
    pin_eq!(w.firsts.len(), 4874, "run1230b baseline");
}

/// The long's own stride blocks, 1 through 5901 — the last before run1230b's
/// first block 5956 (item 3585).
pub(crate) const WINDOW_EGYPTIAN_CLASSICAL_STRIDE: (i64, i64) = (1, 5901);

/// **The walk back on the stride**: run1230's every hundredth block from
/// run1229's start with the recorder on (item 3585).
pub(crate) fn egyptian_classical_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_CLASSICAL.start],
        true,
        EGYPTIAN_CLASSICAL.long,
        "run1230 stride",
        &[(
            EGYPTIAN_CLASSICAL.long.0,
            WINDOW_EGYPTIAN_CLASSICAL_STRIDE.0,
        )],
        WINDOW_EGYPTIAN_CLASSICAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn egyptian_classical_pair_the_long_s_stride_walks_back_from_5956() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_classical_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 60, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3585: 187 keys, every one a standing family (`build:extra`,
    // which this crate holds alone, `form`, groups' `held`/`pool`): no
    // unit parts on a stride block through 5901. The stride prints no
    // `LEADERDATA` or `BUILDDATA`, so run1230b's economy rows on 5956 are
    // not walked back here.
    pin_eq!(w.firsts.len(), 187, "run1230 stride baseline");
}

/// run1241: the walk back from run1230b's first block 5956, blocks
/// 5576..5786 at run1230b's detail (item 3594) — six blocks before the
/// Market `1/2020`'s first Caravan (ours queued on 5582) through the head
/// ours turns to the Merchant on 5778. The long's stride prints no
/// `LEADERDATA` or `BUILDDATA`, and run1230b starts on 5956 with the queue
/// already apart.
pub(crate) const RUN1241: &str = "gamelog-run1241-eastindies-egyptian-classical-window-5576-5786.txt";

/// The window: block 5576 through 5786.
pub(crate) const WINDOW_EGYPTIAN_CLASSICAL_MARKET: (i64, i64) = (5576, 5786);

/// **The Market's queue walked back**: run1241 walked from run1229's start
/// with the recorder on (item 3594).
pub(crate) fn egyptian_classical_market_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_CLASSICAL.start],
        true,
        EGYPTIAN_CLASSICAL.long,
        "run1241",
        &[(RUN1241, WINDOW_EGYPTIAN_CLASSICAL_MARKET.0)],
        WINDOW_EGYPTIAN_CLASSICAL_MARKET,
        1,
        &[],
        true,
    )
}

#[test]
fn egyptian_classical_pair_run1241_walks_the_market_s_queue_back() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_classical_market_window() else {
        return;
    };
    pin_eq!(w.blocks, 210, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(w.firsts.len(), 128, "run1241 baseline");
}
