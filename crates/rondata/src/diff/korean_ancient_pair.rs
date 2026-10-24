//! **The Koreans from an Ancient start, East Indies** (DECISIONS 90, 91,
//! 92, item 3559): the first pair to start at age 0 — East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Koreans (16)** at
//! Difficulty 5, seed 12345, `STARTING_RESOURCES 7`, `STARTING_TECHNOLOGY
//! 0`. The booking asked for the age held at Ancient. The lobby cannot do
//! that: `SetupWin::setup_game` (`0x005b3860`) stores `ending_technology`
//! as the combo's index plus one, so its lowest value is 1. The profile's
//! `ENDING_TECHNOLOGY=0` reads back as `ENDING_TECHNOLOGY 1`, and the
//! computer stands at Classical on the long's closing leader record.
//! Captures: run1199 (a `DUMP_ALL` start) and run1200 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §312.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const KOREAN_ANCIENT: Pair = Pair {
    name: "EastIndiesKoreanAncient",
    map_style: 18,
    ai_tribe: 16,
    difficulty: 5,
    starting_technology: 0,
    // Asked 0; the lobby's floor is 1 (`SetupWin::setup_game`).
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1199-eastindies-korean-ancient-start.txt",
    long: (
        "gamelog-run1200-eastindies-korean-ancient-trace.txt",
        "rontrace-run1200.log",
    ),
    // The game ended itself on 5639 (closing block 5640).
    length: 5639,
    // Item 3559, first try: frame 4350 by the count and by the sequence.
    // On 4350 ours spent 21 draws against theirs' 19, parting at index 0 —
    // ours `Unit::fight+0x9b0`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (`docs/AI.md` section 312).
    //
    // **The delta: 4350 → 5639, the game's last frame — whole** (item
    // 3578): the Korean stones leave from piece 1088's bays and the
    // Hwarang's arrows from piece 130's (`sim::launch`). The value diff on
    // the frame the state first parted, 4294: `1/29`'s stone at `0/13`
    // left from (6600, 9768) in 10 frames where the original's left from
    // (6541, 9663) in 9, and `1/44`'s arrow (4279) from (6072, 10728) in
    // 23 where it left from (6085, 10656) in 22 (run1221's `AMMO`); both
    // now agree, and `0/13`'s `damage_frame` 4294 and `damage_frac` 10
    // with them.
    count: 5639,
    sequence: 5639,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const KOREAN_ANCIENT_PAIR_WORD: i64 = if KOREAN_ANCIENT.count < KOREAN_ANCIENT.sequence {
    KOREAN_ANCIENT.count
} else {
    KOREAN_ANCIENT.sequence
};

/// **The pair's closing state**: run1200's closing dump, block 5640,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3578).
#[test]
fn run1200_korean_ancient_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &KOREAN_ANCIENT;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
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
        "Korean pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5640, "the closing block");
    // Item 3578, the pair whole: 156 units compared, none torn, off or
    // unlinked, no building or city field-row.
    pin_eq!(e.compared, 156, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// **The pair's first parting**, walked from its own start.
#[test]
fn korean_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &KOREAN_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "KOREAN_ANCIENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, `ENDING_TECHNOLOGY` 1 beside `STARTING_TECHNOLOGY` 0, and the
/// long is scoreable.
#[test]
fn korean_ancient_pair_captures_say_their_lobby() {
    let p = &KOREAN_ANCIENT;
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
        "the Korean Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the held-out `GreatSaharaKoreanTier3` is `(7, 16,
/// 3)`, another map and difficulty, and no pinned lobby holds `(18, 16, 5)`
/// before this pair (item 3559).
#[test]
fn korean_ancient_pair_is_no_other_lobby() {
    let p = &KOREAN_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1200b: the word 4350's window, blocks 4345..4601 at the long's
/// detail — six blocks before the word's block 4351 and 250 after (item
/// 3559); the start dumps blocks 0..2 alone and the long's stride prints
/// 4301 and 4401.
pub(crate) const RUN1200B: &str = "gamelog-run1200b-eastindies-korean-ancient-window-4345-4601.txt";

/// The window: block 4345 through 4601; the word 4350's own block is 4351.
pub(crate) const WIDENING_KOREAN_ANCIENT_FRAME_4350: (i64, i64) = (4345, 4601);

/// **The word 4350's window**: run1200b walked from run1199's start with
/// the recorder on (item 3559).
pub(crate) fn korean_ancient_frame_4350_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[KOREAN_ANCIENT.start],
        true,
        KOREAN_ANCIENT.long,
        "run1200b",
        &[(RUN1200B, WIDENING_KOREAN_ANCIENT_FRAME_4350.0)],
        WIDENING_KOREAN_ANCIENT_FRAME_4350,
        1,
        &[4351],
        true,
    )
}

#[test]
fn korean_ancient_pair_run1200b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_ancient_frame_4350_window() else {
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
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3559, the first walk: 2330 keys. Block 4345, the window's
    // first, carries 84 rows: the standing families (`form`,
    // `SITE[i].reg`, the groups' `held` and pools) and the Nubian Citizen
    // `0/10` (TI 50, land), already apart — `damage_frame` ours 4334
    // theirs 4333, `pos` (4796,7968) against (4774,7955) — beside `0/13`'s
    // `damage_frame` 4295 against 4294. On the word's frame 4350 the
    // Korean Slingers `1/17` (TI 82, land) re-aim: `order:kind` ours 10
    // theirs 1, `orders.len` 2 against 3, `heading` 415236096 against
    // -414449664; on its block 4351 `order:target` is ours `0/13` against
    // theirs `0/10`, and `move.dest` (5544,8088) against (5208,8616).
    //
    // Item 3578, with piece 1088's and piece 130's bays: 2330 → 83, the
    // standing families alone (every unit's `form`, `SITE[i].reg`, both
    // leaders' `aggression`, who=1's groups' `held` and pools); no unit
    // field parts on any block.
    pin_eq!(w.firsts.len(), 83, "run1200b baseline");
}

/// **The long's stride, walked back from the word 4350** (item 3578):
/// run1200b's window opens apart on 4345 (`0/10`'s and `0/13`'s
/// `damage_frame`), and only run1200's stride blocks are on disk between
/// run1199's 2 and run1200b's 4345.
pub(crate) const WIDENING_KOREAN_ANCIENT_STRIDE: (i64, i64) = (101, 4301);

pub(crate) fn korean_ancient_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[KOREAN_ANCIENT.start],
        true,
        KOREAN_ANCIENT.long,
        "run1200 stride",
        &[(KOREAN_ANCIENT.long.0, WIDENING_KOREAN_ANCIENT_STRIDE.0)],
        WIDENING_KOREAN_ANCIENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn korean_ancient_pair_the_long_s_stride_walks_back_from_4350() {
    let _pins = Pins::hold();
    let Some(w) = korean_ancient_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 43, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3578: 128 keys before the bays — every row a standing family
    // (`build:extra`, every unit's `form`, who=1's groups' `held` and
    // pool) up to 4201, and on 4301 the Citizen `0/13` (TI 50, land):
    // `damage_frame` ours 4295 against theirs 4294 (`1/29`'s stone, a
    // frame long from the figure's own square) and `damage_frac` 2 against
    // 10 (`1/44`'s arrow, the same). With the bays, 125: the standing
    // families alone.
    pin_eq!(w.firsts.len(), 125, "the stride's baseline");
}
