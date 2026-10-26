//! **The Inca at a held Enlightenment age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, item 3648): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Inca (2)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4` —
//! `turkish_enlightenment_pair`'s map and age with the nation moved and
//! the difficulty at 5. Its triple, `(7, 2, 5)`, is new: the battery
//! fields no Inca; the AI words `inca_word` and `inca_sahara_word` are
//! `(18, 2, 3)` and `(7, 2, 3)`, and the Classical pair East Indies'
//! `(18, 2, 5)`. Captures: run1309 (a `DUMP_ALL` start) and run1310 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §339.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const INCA_ENLIGHTENMENT: Pair = Pair {
    name: "GreatSaharaIncaEnlightenment",
    map_style: 7,
    ai_tribe: 2,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1309-greatsahara-inca-enlightenment-start.txt",
    long: (
        "gamelog-run1310-greatsahara-inca-enlightenment-trace.txt",
        "rontrace-run1310.log",
    ),
    // The game ended itself on 4045 (closing block 4046).
    length: 4045,
    // Item 3648, first try: frame 3095, by the count and the sequence —
    // ours 13 draws against 12, parting at index 9, ours
    // `Unit::close+0xcb6`, theirs `Farms::inc_time+0x1ae` (`docs/AI.md`
    // §339).
    count: 3095,
    sequence: 3095,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INCA_ENLIGHTENMENT_PAIR_WORD: i64 =
    if INCA_ENLIGHTENMENT.count < INCA_ENLIGHTENMENT.sequence {
        INCA_ENLIGHTENMENT.count
    } else {
        INCA_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn inca_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &INCA_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "INCA_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4, and the long is scoreable.
#[test]
fn inca_enlightenment_pair_captures_say_their_lobby() {
    let p = &INCA_ENLIGHTENMENT;
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
        "the Inca Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3648; the battery's identity
/// test refuses it) and none of the battery's, which fields no Inca; and
/// no other pinned Inca lobby's: `inca_word`'s `(18, 2, 3)`,
/// `inca_sahara_word`'s `(7, 2, 3)` and the Classical pair's `(18, 2, 5)`
/// (item 3648).
#[test]
fn inca_enlightenment_pair_is_no_other_lobby() {
    let p = &INCA_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Inca"
    );
    let word = &super::inca_word::INCA;
    let sahara = &super::inca_sahara_word::INCA_SAHARA;
    let classical = &super::inca_classical_pair::INCA_CLASSICAL;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
        (
            classical.name,
            (
                classical.map_style,
                classical.ai_tribe,
                classical.difficulty,
            ),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Inca");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 3095** (item 3648):
/// only run1310's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 3001);

pub(crate) fn inca_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1310 stride",
        &[(
            INCA_ENLIGHTENMENT.long.0,
            WIDENING_INCA_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_INCA_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_the_long_s_stride_walks_back_from_3095() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 30, "every stride block in the window");
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
    // Item 3648: 84 keys, every one standing: 47 buildings' `build:extra`
    // (which the stride does not print), `form` on 27 units, and who=1's
    // five groups' `held` and pools as each is minted (65 on 101 through
    // 68 on 2101). No unit's position, orders or hits part through block
    // 3001.
    pin_eq!(w.firsts.len(), 84, "the stride's baseline");
}

/// run1310b: the word 3095's window, blocks 3089..3345 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 3096 and 250 after (item 3648); the stride's blocks are 3001 and
/// 3101.
pub(crate) const RUN1310B: &str =
    "gamelog-run1310b-greatsahara-inca-enlightenment-window-3089-3345.txt";

/// The window: block 3089 through 3345; the word 3095's own block is 3096.
pub(crate) const WIDENING_INCA_ENLIGHTENMENT_FRAME_3095: (i64, i64) = (3089, 3345);

/// **The word 3095's window**: run1310b walked from run1309's start with
/// the recorder on and every round compared (item 3648).
pub(crate) fn inca_enlightenment_frame_3095_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INCA_ENLIGHTENMENT.start],
        true,
        INCA_ENLIGHTENMENT.long,
        "run1310b",
        &[(RUN1310B, WIDENING_INCA_ENLIGHTENMENT_FRAME_3095.0)],
        WIDENING_INCA_ENLIGHTENMENT_FRAME_3095,
        1,
        &[3096],
        true,
    )
}

#[test]
fn inca_enlightenment_pair_run1310b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = inca_enlightenment_frame_3095_window() else {
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
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3648, the first walk: 1205 keys. **The state parts in the
    // undumped gap 3002..3088**, and block 3089 carries it beside the
    // standing families (`form`, `SITE[i].reg`, who=0's `aggression[1]`,
    // who=1's groups' `held` and pools): who=1's Musketeers `1/36` (TY 49,
    // TypeIndex 99, stance Raid) holds an attack on the human's Citizen
    // `0/5` (TypeIndex 50) in ours and on the Citizen `0/12` in theirs —
    // `RON_DEBUG_UNIT` has ours take `0/5` on 3085 — and `0/5`'s
    // `damage_frame` is 3054 against 3053, its position (7536, 29352)
    // against (7525, 29330). On block 3095 theirs' `0/12` is struck
    // (`hits_left` 12 ours, 8 theirs); on the word 3095 ours kills `0/5`
    // (ours `Unit::close+0xcb6`: its damage 72 against theirs' 58, of 65)
    // where theirs' `0/5` lives, and theirs draws `Farms::inc_time+0x1ae`
    // first: ours 13 draws against 12, index 9 (`docs/AI.md` §339). On
    // 3096 who=0's `num_units[0]` is 11 against 12.
    pin_eq!(w.firsts.len(), 1205, "run1310b baseline");
    // Every round, both directions: 104 round-blocks theirs, 138 ours. The
    // first to part is `1/36`'s, on 3092: ours at `0/5`, theirs at `0/12`.
    pin_eq!(w.rounds_read, (104, 138), "the rounds read");
    pin_eq!(w.rounds.len(), 109, "the rounds that part");
}
