//! **The Egyptians at a held Gunpowder age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, item 3603): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Egyptians (7)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — the Japanese Gunpowder pair's lobby with the
//! nation moved. The Egyptians' other lobbies are `egyptian_word`'s
//! `GreatSaharaEgyptianTier4`, `(7, 7, 4)`, the held-out
//! `EastIndiesEgyptianTier3`, `(18, 7, 3)`, and `egyptian_classical_pair`'s
//! `EastIndiesEgyptianClassical`, `(18, 7, 5)` — another difficulty, or
//! another map. Captures: run1253 (a `DUMP_ALL` start) and run1254 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try is
//! in `docs/AI.md` §324.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const EGYPTIAN_GUNPOWDER: Pair = Pair {
    name: "GreatSaharaEgyptianGunpowder",
    map_style: 7,
    ai_tribe: 7,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1253-greatsahara-egyptian-gunpowder-start.txt",
    long: (
        "gamelog-run1254-greatsahara-egyptian-gunpowder-trace.txt",
        "rontrace-run1254.log",
    ),
    // The game ended itself on 4454 (closing block 4455).
    length: 4454,
    // Item 3603, first try: frame 3118 by the count and the sequence, ours
    // 13 draws against theirs' 14, parting at index 11 — ours
    // `Farms::inc_time+0x1ae`, theirs `Unit::close+0xcb6` (`docs/AI.md`
    // section 324): the state parts on block 3119, the Citizen `0/9`
    // (TypeIndex 50) dying in theirs to the Arquebusiers `1/32`'s round a
    // frame before ours. Item 3603: 3118 → **3238** (+120) — piece 48's
    // bays (`sim::launch::BAYS`, run1254c's packet) put `1/32`'s round of
    // 3114 on (7459, 30915), 5 frames, as theirs. The new word is ours 8
    // draws against 7, index 5 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x1ed`, theirs `Farms::inc_time+0x1ae`; the state
    // parts on block 3225, the standing Arquebusiers `1/47`'s
    // `damage_frac` ours 10 against theirs 12, no round at it.
    count: 3238,
    sequence: 3238,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const EGYPTIAN_GUNPOWDER_PAIR_WORD: i64 =
    if EGYPTIAN_GUNPOWDER.count < EGYPTIAN_GUNPOWDER.sequence {
        EGYPTIAN_GUNPOWDER.count
    } else {
        EGYPTIAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn egyptian_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3, and the long is scoreable.
#[test]
fn egyptian_gunpowder_pair_captures_say_their_lobby() {
    let p = &EGYPTIAN_GUNPOWDER;
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
        "the Egyptian Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the scored `GreatSaharaEgyptianTier4` is
/// `(7, 7, 4)` — this map at another difficulty — the held-out
/// `EastIndiesEgyptianTier3` `(18, 7, 3)` and the Classical pair
/// `(18, 7, 5)` another map (item 3603).
#[test]
fn egyptian_gunpowder_pair_is_no_other_lobby() {
    let p = &EGYPTIAN_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let egyptian: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        egyptian,
        [("EastIndiesEgyptianTier3", 18, 3)],
        "the battery's Egyptian games"
    );
    for l in [
        (
            super::egyptian_word::EGYPTIAN.map_style,
            super::egyptian_word::EGYPTIAN.ai_tribe,
            super::egyptian_word::EGYPTIAN.difficulty,
        ),
        (
            super::egyptian_classical_pair::EGYPTIAN_CLASSICAL.map_style,
            super::egyptian_classical_pair::EGYPTIAN_CLASSICAL.ai_tribe,
            super::egyptian_classical_pair::EGYPTIAN_CLASSICAL.difficulty,
        ),
    ] {
        assert_eq!(l.1, p.ai_tribe);
        assert_ne!(l, triple);
    }
}

/// run1254b: the word 3118's window, blocks 3113..3369 at the long's
/// detail — six blocks before the word's block 3119 and 250 after (item
/// 3603); the start dumps blocks 0..2 alone and the long's stride prints
/// units every hundredth block and no `LEADERS` past the start.
pub(crate) const RUN1254B: &str =
    "gamelog-run1254b-greatsahara-egyptian-gunpowder-window-3113-3369.txt";

/// The window: block 3113 through 3369; the word 3118's own block is 3119.
pub(crate) const WIDENING_EGYPTIAN_GUNPOWDER_FRAME_3118: (i64, i64) = (3113, 3369);

/// **The word 3118's window**: run1254b walked from run1253's start with
/// the recorder on (item 3603).
pub(crate) fn egyptian_gunpowder_frame_3118_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_GUNPOWDER.start],
        true,
        EGYPTIAN_GUNPOWDER.long,
        "run1254b",
        &[(RUN1254B, WIDENING_EGYPTIAN_GUNPOWDER_FRAME_3118.0)],
        WIDENING_EGYPTIAN_GUNPOWDER_FRAME_3118,
        1,
        &[3119],
        true,
    )
}

#[test]
fn egyptian_gunpowder_pair_run1254b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_gunpowder_frame_3118_window() else {
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
    // Item 3603, the first walk: 1685 keys. Block 3113, the window's
    // first, carries the standing families alone (who=0's and who=1's
    // `SITE[i].reg`, who=0's `aggression[1]`, who=1's groups 64..70
    // `held` and pools, every unit's `form`). The state parts on block
    // 3119: the human Citizen `0/9` (TypeIndex 50, land, fleeing at full
    // 50 hits) dies in theirs — `Unit::close < Object::die` after one
    // round's landing on 3118 — and stands in ours (who=0's `active`,
    // `control`, `num_units[0]`, `peasants` ours 14/14/13/13 against
    // 13/13/12/12). The round is the Arquebusiers `1/32`'s (TypeIndex 98,
    // piece 48), fired on 3114: ours left from the figure's own (7560,
    // 30840) and flew 6 frames (`docs/AI.md` section 324).
    // Item 3603, after piece 48's bays: 1685 → **880**. Block 3119 agrees;
    // past block 3113's standing families and groups 68/69 the state
    // parts on block 3225, the Arquebusiers `1/47`'s `damage_frac` ours 10
    // against theirs 12 (`1/45`'s on 3226, 12 against 14), with no round
    // at either: the word 3238 (block 3239) follows.
    pin_eq!(w.firsts.len(), 880, "run1254b baseline");
}
