//! **The Romans at a held Gunpowder age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, item 3671): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Romans (6)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder:
//! `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`. Its triple,
//! `(14, 6, 5)`, is new: the Roman lobbies are `roman_word`'s
//! `(18, 6, 2)`, `roman_classical_pair`'s `(7, 6, 5)`,
//! `roman_medieval_pair`'s `(18, 6, 5)` and the held-out battery's
//! `GreatLakesRomanTier2`, `(14, 6, 2)` — this map at another difficulty.
//! Captures: run1341 (a `DUMP_ALL` start) and run1342 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §350.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const ROMAN_GUNPOWDER: Pair = Pair {
    name: "GreatLakesRomanGunpowder",
    map_style: 14,
    ai_tribe: 6,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1341-greatlakes-roman-gunpowder-start.txt",
    long: (
        "gamelog-run1342-greatlakes-roman-gunpowder-trace.txt",
        "rontrace-run1342.log",
    ),
    // The game ended itself on 4062 (closing block 4063).
    length: 4062,
    // Item 3671, first try: frame 2926, by the count and the sequence —
    // ours 14 draws against 15, parting at index 2, ours
    // `Guy::set_anim+0x97a` under `Unit::move_step+0x823`, theirs
    // `Unit::close+0xcb6` (`docs/AI.md` §350).
    count: 2926,
    sequence: 2926,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const ROMAN_GUNPOWDER_PAIR_WORD: i64 =
    if ROMAN_GUNPOWDER.count < ROMAN_GUNPOWDER.sequence {
        ROMAN_GUNPOWDER.count
    } else {
        ROMAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn roman_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &ROMAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "ROMAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn roman_gunpowder_pair_captures_say_their_lobby() {
    let p = &ROMAN_GUNPOWDER;
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
        "the Roman Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3671; the battery's identity
/// test refuses it), once, and none of the battery's: the battery's one
/// Roman game is the held-out `GreatLakesRomanTier2`, `(14, 6, 2)` — this
/// map at another difficulty; and no other Roman lobby's: the AI word
/// `roman_word`'s `(18, 6, 2)`, the Classical pair's `(7, 6, 5)` and the
/// Medieval pair's `(18, 6, 5)` are another map (item 3671).
#[test]
fn roman_gunpowder_pair_is_no_other_lobby() {
    let p = &ROMAN_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple),
        "the battery holds this lobby"
    );
    let romans: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        romans,
        [("GreatLakesRomanTier2", 14, 2)],
        "the battery's Roman games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Roman lobby on this map at this difficulty"
        );
    }
    let w = &super::roman_word::ROMAN;
    let c = &super::roman_classical_pair::ROMAN_CLASSICAL;
    let m = &super::roman_medieval_pair::ROMAN_MEDIEVAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Romans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 2926** (item 3671):
/// only run1342's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_ROMAN_GUNPOWDER_STRIDE: (i64, i64) = (101, 2901);

pub(crate) fn roman_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_GUNPOWDER.start],
        true,
        ROMAN_GUNPOWDER.long,
        "run1342 stride",
        &[(ROMAN_GUNPOWDER.long.0, WIDENING_ROMAN_GUNPOWDER_STRIDE.0)],
        WIDENING_ROMAN_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn roman_gunpowder_pair_the_long_s_stride_walks_back_from_2926() {
    let _pins = Pins::hold();
    let Some(w) = roman_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 29, "every stride block in the window");
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
    // Item 3671: 94 keys. The standing families — 49 buildings'
    // `build:extra` (which the stride does not print), `form` on 27 units,
    // who=1's groups' `held` and pools 64..70 as each is minted — and **one
    // unit family: the human's Citizens' damage, from 2701** (`0/13`
    // `hits_left` ours 26 theirs 23; on 2801 `0/4` 24 against 21): theirs
    // take more.
    pin_eq!(w.firsts.len(), 94, "the stride's baseline");
}

/// run1342b: the word 2926's window, blocks 2895..3177 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2901 and 250 after the word's block 2927 (item 3671).
pub(crate) const RUN1342B: &str =
    "gamelog-run1342b-greatlakes-roman-gunpowder-window-2895-3177.txt";

/// The window: block 2895 through 3177; the word 2926's own block is 2927.
pub(crate) const WIDENING_ROMAN_GUNPOWDER_FRAME_2926: (i64, i64) = (2895, 3177);

/// **The word 2926's window**: run1342b walked from run1341's start with
/// the recorder on and every round compared (item 3671).
pub(crate) fn roman_gunpowder_frame_2926_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[ROMAN_GUNPOWDER.start],
        true,
        ROMAN_GUNPOWDER.long,
        "run1342b",
        &[(RUN1342B, WIDENING_ROMAN_GUNPOWDER_FRAME_2926.0)],
        WIDENING_ROMAN_GUNPOWDER_FRAME_2926,
        1,
        &[2927],
        true,
    )
}

#[test]
fn roman_gunpowder_pair_run1342b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_gunpowder_frame_2926_window() else {
        return;
    };
    pin_eq!(w.blocks, 282, "every captured block");
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
    // Item 3671, the first walk: 2059 keys. Block 2895 carries the
    // standing families (`form`, both leaders' `SITE[i].reg`, who=1's
    // groups and pools, who=1's `attack` 659 against 669) and the
    // Citizens' damage the stride already parts (`0/4` `hits_left` 26
    // against 23, `0/13` 30 against 27). **On the word's block 2927 the
    // human's leader holds a unit more in ours** (`num_units[0]` 13
    // against 12, `peasants`, `active` and `control` likewise) and `0/4`
    // is ours alone: theirs' draw at index 2 is `Unit::close+0xcb6`, the
    // Citizen's death under who=1's Heavy Knight `1/24` (TY 229), whose
    // blow ours' `0/4` survives with 3 less damage and flees; from 2927
    // who=1's Heavy Knight `1/25` re-targets `0/5` in theirs alone.
    pin_eq!(w.firsts.len(), 2059, "run1342b baseline");
    // Every round, both directions: 17 round-blocks theirs, 11 ours, 7
    // parting, the first on 2960, after the word.
    pin_eq!(w.rounds_read, (17, 11), "the rounds read");
    pin_eq!(w.rounds.len(), 7, "the rounds that part");
}
