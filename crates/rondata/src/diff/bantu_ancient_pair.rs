//! **The Bantu from an Ancient start, East Indies** (DECISIONS 90, 91,
//! 93, item 3562): East Indies, `MAP_STYLE 18`, human Nubians (4) against
//! the **Bantu (3)** at Difficulty 5, seed 12345, `STARTING_RESOURCES 7`,
//! `STARTING_TECHNOLOGY 0` — the Koreans' Ancient-start lobby with the
//! nation moved. The profile's `ENDING_TECHNOLOGY=0` reads back as
//! `ENDING_TECHNOLOGY 1` (3559a: `SetupWin::setup_game` stores the combo's
//! index plus one), so this is an Ancient start capped at Classical, not a
//! held age. The Bantu's other scored lobby is `bantu_word`'s
//! `GreatSaharaBantuTier4`, another map. Captures: run1205 (a `DUMP_ALL`
//! start) and run1206 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §315.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BANTU_ANCIENT: Pair = Pair {
    name: "EastIndiesBantuAncient",
    map_style: 18,
    ai_tribe: 3,
    difficulty: 5,
    starting_technology: 0,
    // Asked 0; the lobby's floor is 1 (`SetupWin::setup_game`, 3559a).
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1205-eastindies-bantu-ancient-start.txt",
    long: (
        "gamelog-run1206-eastindies-bantu-ancient-trace.txt",
        "rontrace-run1206.log",
    ),
    // The game ended itself on 6210 (closing block 6211).
    length: 6210,
    // Item 3562, first try: frame 4185 by the count and the sequence, ours
    // 350 draws against 342, index 242 — ours `Leader::make_stuff+0x63d`,
    // theirs `Leader::produce_building+0x1805` (`docs/AI.md` section 315).
    count: 4185,
    sequence: 4185,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BANTU_ANCIENT_PAIR_WORD: i64 = if BANTU_ANCIENT.count < BANTU_ANCIENT.sequence {
    BANTU_ANCIENT.count
} else {
    BANTU_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn bantu_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BANTU_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BANTU_ANCIENT {}: count {} sequence {} of {}; {}",
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
fn bantu_ancient_pair_captures_say_their_lobby() {
    let p = &BANTU_ANCIENT;
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
        "the Bantu Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery fields no Bantu, and the Bantu's one
/// other lobby, `bantu_word`'s `GreatSaharaBantuTier4`, is `(7, 3, 4)` —
/// another map and difficulty (item 3562).
#[test]
fn bantu_ancient_pair_is_no_other_lobby() {
    let p = &BANTU_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(!battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe));
    let other = &super::bantu_word::BANTU;
    assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
}

/// run1206b: the word 4185's window, blocks 4180..4436 at the long's
/// detail — six blocks before the word's block 4186 and 250 after (item
/// 3562); the start dumps blocks 0..2 alone and the long's stride prints
/// 4101 and 4201.
pub(crate) const RUN1206B: &str = "gamelog-run1206b-eastindies-bantu-ancient-window-4180-4436.txt";

/// The window: block 4180 through 4436; the word 4185's own block is 4186.
pub(crate) const WIDENING_BANTU_ANCIENT_FRAME_4185: (i64, i64) = (4180, 4436);

/// **The word 4185's window**: run1206b walked from run1205's start with
/// the recorder on (item 3562).
pub(crate) fn bantu_ancient_frame_4185_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1206b",
        &[(RUN1206B, WIDENING_BANTU_ANCIENT_FRAME_4185.0)],
        WIDENING_BANTU_ANCIENT_FRAME_4185,
        1,
        &[4186],
        true,
    )
}

#[test]
fn bantu_ancient_pair_run1206b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_frame_4185_window() else {
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
    // Item 3562, the first walk: 1368 keys. Block 4180, the window's
    // first, carries 115 rows: the standing families (`form`,
    // `SITE[i].reg`, both leaders' `aggression`, who=1's groups' `held`
    // and pools) and who=1's make list, already apart — `MAKE[0]` ours
    // cat 7 t 341 (the Galley) val 21504000 against theirs cat 6 t 596 val
    // 9999999, and `MAKE[7]` ours t 341 val 21504000 against theirs t 94
    // (the Yangombi Umpakati) val 2016000. On 4183 ours queues at `1/2014`
    // where theirs does not (`queue:queued` 1 against 0, `bucket` timber
    // and metal 80 lower). On the word's block 4186 theirs holds two new
    // buildings and ours one: theirs' `1/2054` is a Stable (428, city 3,
    // `constr_time` 42000, `myhits` 1200) at (35136, 38976) and its
    // `1/2055` the Hanging Gardens (528, city 1, `constr_time` 400000) at
    // (34560, 37632); ours' `1/2054` is the Hanging Gardens, at (34752,
    // 37824), and ours places no Stable.
    pin_eq!(w.firsts.len(), 1368, "run1206b baseline");
}
