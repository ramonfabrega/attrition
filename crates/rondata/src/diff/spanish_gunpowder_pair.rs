//! **The Spanish at a held Gunpowder age, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3627): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Spanish (9)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `mongol_gunpowder_pair`'s lobby with the
//! nation moved. Its triple, `(18, 9, 5)`, is new: the battery fields no
//! Spanish, `spanish_east_word`'s `EastIndiesSpanishTier3` is `(18, 9, 3)`,
//! another difficulty, `spanish_classical_pair`'s `(7, 9, 5)` is another
//! map, and `spanish_word`'s `(7, 9, 3)` and `spanish_lakes_word`'s
//! `(14, 9, 4)` are both. Captures: run1286 (a `DUMP_ALL` start) and
//! run1287 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §335.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_GUNPOWDER: Pair = Pair {
    name: "EastIndiesSpanishGunpowder",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1286-eastindies-spanish-gunpowder-start.txt",
    long: (
        "gamelog-run1287-eastindies-spanish-gunpowder-trace.txt",
        "rontrace-run1287.log",
    ),
    // The game ended itself on 6128 (closing block 6129).
    length: 6128,
    // Item 3627, first try: frame 2168, by the count and the sequence —
    // ours 35 draws against 34, parting at index 30 (`docs/AI.md` §335).
    count: 2168,
    sequence: 2168,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_GUNPOWDER_PAIR_WORD: i64 =
    if SPANISH_GUNPOWDER.count < SPANISH_GUNPOWDER.sequence {
        SPANISH_GUNPOWDER.count
    } else {
        SPANISH_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn spanish_gunpowder_pair_captures_say_their_lobby() {
    let p = &SPANISH_GUNPOWDER;
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
        "the Spanish Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3627; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no
/// Spanish since `EastIndiesSpanishTier3`, `(18, 9, 3)`, left it for the AI
/// word (another difficulty); the Classical pair's `(7, 9, 5)` is another
/// map, and `GreatSaharaSpanishTier3`'s `(7, 9, 3)` and
/// `GreatLakesSpanishTier4`'s `(14, 9, 4)` are both (item 3627).
#[test]
fn spanish_gunpowder_pair_is_no_other_lobby() {
    let p = &SPANISH_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Spanish"
    );
    for (name, other) in [
        (
            super::spanish_word::SPANISH.name,
            (
                super::spanish_word::SPANISH.map_style,
                super::spanish_word::SPANISH.ai_tribe,
                super::spanish_word::SPANISH.difficulty,
            ),
        ),
        (
            super::spanish_lakes_word::SPANISH_LAKES.name,
            (
                super::spanish_lakes_word::SPANISH_LAKES.map_style,
                super::spanish_lakes_word::SPANISH_LAKES.ai_tribe,
                super::spanish_lakes_word::SPANISH_LAKES.difficulty,
            ),
        ),
        (
            super::spanish_east_word::SPANISH_EAST.name,
            (
                super::spanish_east_word::SPANISH_EAST.map_style,
                super::spanish_east_word::SPANISH_EAST.ai_tribe,
                super::spanish_east_word::SPANISH_EAST.difficulty,
            ),
        ),
        (
            super::spanish_classical_pair::SPANISH_CLASSICAL.name,
            (
                super::spanish_classical_pair::SPANISH_CLASSICAL.map_style,
                super::spanish_classical_pair::SPANISH_CLASSICAL.ai_tribe,
                super::spanish_classical_pair::SPANISH_CLASSICAL.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 2168** (item 3627):
/// only run1287's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_STRIDE: (i64, i64) = (101, 2101);

pub(crate) fn spanish_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1287 stride",
        &[(
            SPANISH_GUNPOWDER.long.0,
            WIDENING_SPANISH_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_SPANISH_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_the_long_s_stride_walks_back_from_2168() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 21, "every stride block in the window");
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
    // Item 3627: 100 keys, every one standing — the start's buildings'
    // `build:extra` (which the stride does not print), who=1's groups'
    // `held` and pools as each group is minted (64 on 101 through 78 on
    // 2101), and `form` on 30 units. No unit's position, orders or
    // containment parts through block 2101: the Arquebusiers `1/48`, `1/49`
    // and `1/50` (TypeIndex 98) agree there, `1/48` and `1/49` on an
    // `ATTACKTOORDER` to (40776, 37128).
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1287b: the word 2168's window, blocks 2163..2419 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 2169 and 250 after (item 3627); the stride's blocks are 2101 and
/// 2201.
pub(crate) const RUN1287B: &str =
    "gamelog-run1287b-eastindies-spanish-gunpowder-window-2163-2419.txt";

/// The window: block 2163 through 2419; the word 2168's own block is 2169.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_FRAME_2168: (i64, i64) = (2163, 2419);

/// **The word 2168's window**: run1287b walked from run1286's start with
/// the recorder on (item 3627).
pub(crate) fn spanish_gunpowder_frame_2168_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1287b",
        &[(RUN1287B, WIDENING_SPANISH_GUNPOWDER_FRAME_2168.0)],
        WIDENING_SPANISH_GUNPOWDER_FRAME_2168,
        1,
        &[2169],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1287b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_frame_2168_window() else {
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
    // Item 3627, the first walk: 1343 keys. Block 2163 carries the standing
    // families (`form` on 28 units, `SITE[i].reg`, who=0's
    // `aggression[1]`, who=1's groups' `held` and pools) and **the gap's
    // parting**: ours holds a Transport Galleon `1/68` (TypeIndex 321)
    // theirs never prints, with the Arquebusiers `1/48`, `1/49` and `1/50`
    // (TypeIndex 98) `inside` 68, where theirs' three stand on the shore,
    // `1/48` at (41688, 36360) against ours' (41928, 36300). Ours casts it:
    // `RON_DEBUG_UNIT` puts the Galleon's birth on 2153 at (42097, 36253),
    // through `do_cast_effect`'s `cast_transport`, carrying `1/48`'s
    // `ATTACKTOORDER`, in the gap 2102..2162 that no dump prints — ours
    // acts (`docs/AI.md` section 335). The first rows past it are `1/61`'s
    // `half_step` (2165) and `pos` (2166), and on **the word 2168**, ours
    // 35 draws against 34 at index 30, ours spends `1/63`'s `Guy::set_anim
    // < Unit::move_step`; on 2169 `1/63` collides with `1/68` (`collide_o`
    // 68, `path:length` 1 against 15). 1343 → **1347** on the tree that
    // merged item 3637, whose move row compares `retry` and `attempts`: four
    // rows past the word, `1/63`'s `retry` 0 against 8 (2194) and
    // `attempts` 0 against 3 (2202), and `1/21`'s `retry` 8 against 6
    // (2283) and `attempts` 0 against 3 (2289).
    pin_eq!(w.firsts.len(), 1347, "run1287b baseline");
}
