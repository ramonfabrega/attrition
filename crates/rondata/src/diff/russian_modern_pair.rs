//! **The Russians at a held Modern age, East Indies** (DECISIONS 90, item
//! 3508): the first late-age pair to hold one age. The coverage pair's
//! lobby (run652's: East Indies, `MAP_STYLE 18`, human Nubians (4),
//! Difficulty 5, seed 12345, `STARTING_RESOURCES 7`) with the nation varied
//! to the **Russians (13)** and the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Captures: run1148 (a
//! `DUMP_ALL` start) and run1149 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §304.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const RUSSIAN_MODERN: Pair = Pair {
    name: "EastIndiesRussianModern",
    map_style: 18,
    ai_tribe: 13,
    difficulty: 5,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1148-eastindies-russian-modern-start.txt",
    long: (
        "gamelog-run1149-eastindies-russian-modern-trace.txt",
        "rontrace-run1149.log",
    ),
    // The game ended itself on 5841 (closing block 5842).
    length: 5841,
    // Item 3508, first try: the sequence parted on frame 8, ours 18 draws
    // against 18, index 2 — ours `Leader::produce_building+0x1805`, theirs
    // `Leader::make_stuff+0x63d` — and the count on 11. Item 3515: 8 → 3682
    // for both. `TRANSPORT_BONUS`'s prerequisite, Written Word, takes
    // `research_techs`' ×30 on a sea map, so who=1's block-5 `MAKE[8].val`
    // is 9999999 on both sides (ours 1620003 before). Frame 3682: ours 15
    // draws against 14, index 0 — ours `Unit::fight+0x9b0`, theirs
    // `Unit::do_move+0xe84` (`docs/AI.md` section 304).
    count: 3682,
    sequence: 3682,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const RUSSIAN_MODERN_PAIR_WORD: i64 = if RUSSIAN_MODERN.count < RUSSIAN_MODERN.sequence {
    RUSSIAN_MODERN.count
} else {
    RUSSIAN_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn russian_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks — `ENDING_TECHNOLOGY` beside `STARTING_TECHNOLOGY` — and the long
/// is scoreable.
#[test]
fn russian_modern_pair_captures_say_their_lobby() {
    let p = &RUSSIAN_MODERN;
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
        "the Russian pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn russian_modern_pair_is_no_other_lobby() {
    let p = &RUSSIAN_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1149b: the word 8's window, blocks 3..259 at the long's detail — six
/// blocks before the word's block 9 and 250 after (item 3508).
pub(crate) const RUN1149B: &str = "gamelog-run1149b-eastindies-russian-modern-window-3-259.txt";

/// The window: block 3 through 259; the word 8's own block is 9.
pub(crate) const WIDENING_RUSSIAN_MODERN_FRAME_8: (i64, i64) = (3, 259);

/// **The word 8's window**: run1149b walked from run1148's start with the
/// recorder on (item 3508).
pub(crate) fn russian_modern_frame_8_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_MODERN.start],
        true,
        RUSSIAN_MODERN.long,
        "run1149b",
        &[(RUN1149B, WIDENING_RUSSIAN_MODERN_FRAME_8.0)],
        WIDENING_RUSSIAN_MODERN_FRAME_8,
        1,
        &[9],
        true,
    )
}

#[test]
fn russian_modern_pair_run1149b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_modern_frame_8_window() else {
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
    // Item 3508, the first walk: 342 keys. On block 3 the standing rows the
    // other late-age pairs carry — both leaders' `SITE[i].reg` (ours 65
    // against 0) and `aggression`, every unit's `form`, group 65's `held`
    // and pool. **The state first parts on block 5** (frame 4), who=1's
    // make list: the research slot `MAKE[8]` holds Written Word (551) at
    // `val` ours 1620003 against theirs 9999999, so theirs sorts it to the
    // head (`MAKE[0]` t 551 cat 8) and every later entry sits one slot on.
    // The word 8's block 9 parts building `1/2005`'s research queue in the same
    // order (ours 558, 572, 551, 566 against 551, 558, 566, 572) and
    // `MAKE[10].val` (`docs/AI.md` section 304). Item 3515: 342 → 52. The
    // block-5 make list agrees (`MAKE[8]` Written Word at 9999999 both
    // sides, the ×30 of `TRANSPORT_BONUS`'s prerequisite); what is left is
    // the standing families — block 3's 43, groups 64, 66 and 67's `held`
    // and pool on 108, 126 and 138, and `SITE[2..4].reg` on 176.
    pin_eq!(w.firsts.len(), 52, "run1149b baseline");
}

/// run1161b: the word 3682's window, blocks 3677..3933 at the long's detail —
/// six blocks before the word's block 3683 and 250 after (item 3515).
pub(crate) const RUN1161B: &str = "gamelog-run1161b-eastindies-russian-modern-window-3677-3933.txt";

/// The window: block 3677 through 3933; the word 3682's own block is 3683.
pub(crate) const WIDENING_RUSSIAN_MODERN_FRAME_3682: (i64, i64) = (3677, 3933);

/// **The word 3682's window**: run1161b walked from run1148's start with
/// the recorder on (item 3515).
pub(crate) fn russian_modern_frame_3682_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_MODERN.start],
        true,
        RUSSIAN_MODERN.long,
        "run1161b",
        &[(RUN1161B, WIDENING_RUSSIAN_MODERN_FRAME_3682.0)],
        WIDENING_RUSSIAN_MODERN_FRAME_3682,
        1,
        &[3683],
        true,
    )
}

/// **The word 3682's frame, widened whole** (item 3515): every dumped
/// record on run1161b's blocks, both directions.
#[test]
fn russian_modern_pair_run1161b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_modern_frame_3682_window() else {
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
    // Item 3515, the first walk: 929 keys. The standing families of the
    // frame-8 window, and on the window's head (block 3677) who=1's wealth
    // already parted — `income[2]` ours 3816 against theirs 4000, `rate[2]`
    // 238 against 250, `over_cap[2]` 0 against 1 — and timber and metal
    // buckets 40 over: the state parted in 3601..3676, which no dump
    // prints. On block 3682, the word's eve, Cruiser `1/25` (TypeIndex 329,
    // `EXPLORETOORDER`): `recharging` ours 0 against theirs 25,
    // `cavarch_o` 11 against 5 (`docs/AI.md` section 304).
    pin_eq!(w.firsts.len(), 929, "run1161b baseline");
}
