//! **The Greeks at a held Classical age, East Indies** (DECISIONS 90, 91,
//! 92, item 3557): the first pair below age 2 — East Indies, `MAP_STYLE
//! 18`, human Nubians (4) against the **Greeks (5)** at Difficulty 5, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Classical:
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`. Captures: run1195 (a
//! `DUMP_ALL` start) and run1196 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §310.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GREEK_CLASSICAL: Pair = Pair {
    name: "EastIndiesGreekClassical",
    map_style: 18,
    ai_tribe: 5,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1195-eastindies-greek-classical-start.txt",
    long: (
        "gamelog-run1196-eastindies-greek-classical-trace.txt",
        "rontrace-run1196.log",
    ),
    // The game ended itself on 8126 (closing block 8127).
    length: 8126,
    // Item 3557, first try: frame 8 by the sequence, 11 by the count. On 8
    // both sides spend 61 draws and part at index 0 — ours
    // `Leader::make_stuff+0x221`, theirs `Leader::produce_building+0xc99`
    // (`docs/AI.md` section 310). The state parts first on block 2: the
    // Greeks' `bucket[3:knowledge]` ours 0 theirs 10000.
    count: 11,
    sequence: 8,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GREEK_CLASSICAL_PAIR_WORD: i64 =
    if GREEK_CLASSICAL.count < GREEK_CLASSICAL.sequence {
        GREEK_CLASSICAL.count
    } else {
        GREEK_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn greek_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GREEK_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GREEK_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn greek_classical_pair_captures_say_their_lobby() {
    let p = &GREEK_CLASSICAL;
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
        "the Greek Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: no held-out game is the Greeks on East Indies at
/// Difficulty 5 (`GreatSaharaGreekTier1` is `(7, 5, 1)`), and no pinned
/// lobby holds the triple before this pair (item 3557).
#[test]
fn greek_classical_pair_is_no_other_lobby() {
    let p = &GREEK_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1196b: the word 8's window, blocks 2..258 at the long's detail — the
/// word's block 9 is the seventh, and 249 after it (item 3557); the start
/// dumps blocks 0..2 alone and the long's stride first prints 101.
pub(crate) const RUN1196B: &str = "gamelog-run1196b-eastindies-greek-classical-window-2-258.txt";

/// The window: block 2 through 258; the word 8's own block is 9.
pub(crate) const WIDENING_GREEK_CLASSICAL_FRAME_8: (i64, i64) = (2, 258);

/// **The word 8's window**: run1196b walked from run1195's start with the
/// recorder on (item 3557).
pub(crate) fn greek_classical_frame_8_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GREEK_CLASSICAL.start],
        true,
        GREEK_CLASSICAL.long,
        "run1196b",
        &[(RUN1196B, WIDENING_GREEK_CLASSICAL_FRAME_8.0)],
        WIDENING_GREEK_CLASSICAL_FRAME_8,
        1,
        &[9],
        true,
    )
}

#[test]
fn greek_classical_pair_run1196b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = greek_classical_frame_8_window() else {
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
    // Item 3557, the first walk: 515 keys. Block 2 carries 45 rows: the
    // standing families (`form`, `SITE[i].reg`, `aggression`, group 65's
    // `held` and pool) and one more — **who=1's `bucket[3:knowledge]`
    // ours 0 theirs 10000**: theirs' Greeks, started at the Classical age,
    // hold the opening knowledge, and ours' `lay_starting_goods` takes it
    // off them (`GREEK_DELAY_KNOWLEDGE`) for a Classical re-grant a game
    // started there never pays. The AI's make list parts on block 7
    // (`MAKE[0]` ours cat 4, t 52, val 9999999 against cat 8, t 551, val
    // 3) and on 8; on the word's block 9 the knowledge rows are joined by
    // the other buckets (`food` ours 19784 theirs 19730, `wealth` 19780
    // against 19950), `num_queued[2]` 5 against 0, `tech_cat_frame[2]` 0
    // against 8, and the Citizen `1/10`'s move (`dest_x` 37524 against
    // 37500, `off_x` 72 against 24).
    pin_eq!(w.firsts.len(), 515, "run1196b baseline");
}
