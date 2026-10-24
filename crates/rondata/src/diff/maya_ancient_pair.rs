//! **The Maya from an Ancient start, East Indies** (DECISIONS 90, 91, 93,
//! item 3587): East Indies, `MAP_STYLE 18`, human Nubians (4) against the
//! **Maya (1)** at Difficulty 5, seed 12345, `STARTING_RESOURCES 7`,
//! `STARTING_TECHNOLOGY 0` and `ENDING_TECHNOLOGY 1` (read back 1; age 0
//! cannot be held, 3559a) — the Bantu's and the Koreans' Ancient-start
//! lobby with the nation moved. The Maya's other scored lobby is
//! `maya_word`'s `GreatSaharaMayaTier4`, `(7, 1, 4)`: another map.
//! Captures: run1233 (a `DUMP_ALL` start) and run1234 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §320.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const MAYA_ANCIENT: Pair = Pair {
    name: "EastIndiesMayaAncient",
    map_style: 18,
    ai_tribe: 1,
    difficulty: 5,
    starting_technology: 0,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1233-eastindies-maya-ancient-start.txt",
    long: (
        "gamelog-run1234-eastindies-maya-ancient-trace.txt",
        "rontrace-run1234.log",
    ),
    // The game ended itself on 5935 (closing block 5936).
    length: 5935,
    // Item 3587, first try: frame 4182 by the count and the sequence, ours
    // 31 draws against 29, index 16 — ours `Leader::produce_building+0x1805`
    // (who=1 places a second Senate, `1/2042` at (38016, 41376)), theirs
    // `Leader::make_stuff+0x63d` (`docs/AI.md` section 320).
    count: 4182,
    sequence: 4182,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MAYA_ANCIENT_PAIR_WORD: i64 = if MAYA_ANCIENT.count < MAYA_ANCIENT.sequence {
    MAYA_ANCIENT.count
} else {
    MAYA_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn maya_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MAYA_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MAYA_ANCIENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age capped at 1, and the long is scoreable.
#[test]
fn maya_ancient_pair_captures_say_their_lobby() {
    let p = &MAYA_ANCIENT;
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
        "the Maya Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery holds no Maya game, and the Maya's
/// scored word is `(7, 1, 4)` — another map (item 3587).
#[test]
fn maya_ancient_pair_is_no_other_lobby() {
    let p = &MAYA_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(!battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe));
    let other = &super::maya_word::MAYA;
    assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
}

/// run1234b: the word 4182's window, blocks 4177..4433 at the long's
/// detail — six blocks before the word's block 4183 and 250 after (item
/// 3587); the start dumps blocks 0..2 alone and the long's stride prints
/// units every hundredth block and no `LEADERS` past the start.
pub(crate) const RUN1234B: &str = "gamelog-run1234b-eastindies-maya-ancient-window-4177-4433.txt";

/// The window: block 4177 through 4433; the word 4182's own block is 4183.
pub(crate) const WIDENING_MAYA_ANCIENT_FRAME_4182: (i64, i64) = (4177, 4433);

/// **The word 4182's window**: run1234b walked from run1233's start with
/// the recorder on (item 3587).
pub(crate) fn maya_ancient_frame_4182_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MAYA_ANCIENT.start],
        true,
        MAYA_ANCIENT.long,
        "run1234b",
        &[(RUN1234B, WIDENING_MAYA_ANCIENT_FRAME_4182.0)],
        WIDENING_MAYA_ANCIENT_FRAME_4182,
        1,
        &[4183],
        true,
    )
}

#[test]
fn maya_ancient_pair_run1234b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = maya_ancient_frame_4182_window() else {
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3587, the first walk: 2407 keys. Block 4177, the window's
    // first, carries 75 rows: the standing families (every unit's `form`,
    // who=0's `SITE[i].reg` ours 65 against theirs 0, both leaders'
    // `aggression`, who=1's groups' `held` and pools); group 73's `held`
    // and pool on 4179. On the word's block 4183 ours holds a second
    // Senate, `1/2042` at (38016, 41376), `constr_time` 41666, that theirs
    // does not (both hold `1/2041`, the Senate at (35904, 41376)); the
    // Citizen `1/94` builds in ours (`order:kind` 3, to (37560, 41256))
    // and gathers in theirs (7); who=1's `MAKE[8].t` is ours -1 against
    // theirs 438 (the Senate, still on theirs' list), `bucket` timber 15102 against 15139 and wealth 19210
    // against 19260, `gatherers` 27 against 28. Theirs' `1/2042`, on 4186,
    // is a wonder (`constr_time` 400000) at (37632, 41664).
    pin_eq!(w.firsts.len(), 2407, "run1234b baseline");
}
