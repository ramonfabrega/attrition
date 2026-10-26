//! **The Germans at a held Industrial age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, item 3667): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Germans (12)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple,
//! `(14, 12, 5)`, is new: the German lobbies are `german_alltech_pair`'s
//! `(18, 12, 5)`, `german_ancient_pair`'s `(7, 12, 5)`, the held-out
//! `german_word`'s `(18, 12, 3)` and `german_sahara_word`'s `(7, 12, 3)`,
//! and the battery fields no Germans. Captures: run1333 (a `DUMP_ALL`
//! start) and run1334 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §346.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const GERMAN_INDUSTRIAL: Pair = Pair {
    name: "GreatLakesGermanIndustrial",
    map_style: 14,
    ai_tribe: 12,
    difficulty: 5,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1333-greatlakes-german-industrial-start.txt",
    long: (
        "gamelog-run1334-greatlakes-german-industrial-trace.txt",
        "rontrace-run1334.log",
    ),
    // The game ended itself on 4772 (closing block 4773).
    length: 4772,
    // Item 3667, first try: frame 4323, by the count and the sequence —
    // ours 17 draws against 18, parting at index 11, ours
    // `Farms::inc_time+0x1ae`, theirs `Unit::close+0xcb6` (`docs/AI.md`
    // §346).
    count: 4323,
    sequence: 4323,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const GERMAN_INDUSTRIAL_PAIR_WORD: i64 =
    if GERMAN_INDUSTRIAL.count < GERMAN_INDUSTRIAL.sequence {
        GERMAN_INDUSTRIAL.count
    } else {
        GERMAN_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn german_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &GERMAN_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "GERMAN_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5, and the long is scoreable.
#[test]
fn german_industrial_pair_captures_say_their_lobby() {
    let p = &GERMAN_INDUSTRIAL;
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
        "the German Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3667; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Germans; and no other German lobby's: the All Technologies pair's
/// `(18, 12, 5)` and the Ancient-start pair's `(7, 12, 5)` are another map,
/// and the held-out words' `(18, 12, 3)` and `(7, 12, 3)` another map and
/// difficulty (item 3667).
#[test]
fn german_industrial_pair_is_no_other_lobby() {
    let p = &GERMAN_INDUSTRIAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Germans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another German lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::german_alltech_pair::GERMAN_ALLTECH,
        &super::german_ancient_pair::GERMAN_ANCIENT,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    for k in [
        &super::german_word::GERMAN,
        &super::german_sahara_word::GERMAN_SAHARA,
    ] {
        assert_eq!(k.ai_tribe, p.ai_tribe);
        assert_ne!((k.map_style, k.ai_tribe, k.difficulty), triple);
    }
}
/// **The long's stride, walked back from the word 4323** (item 3667):
/// only run1334's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_GERMAN_INDUSTRIAL_STRIDE: (i64, i64) = (101, 4301);

pub(crate) fn german_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[GERMAN_INDUSTRIAL.start],
        true,
        GERMAN_INDUSTRIAL.long,
        "run1334 stride",
        &[(
            GERMAN_INDUSTRIAL.long.0,
            WIDENING_GERMAN_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_GERMAN_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn german_industrial_pair_the_long_s_stride_walks_back_from_4323() {
    let _pins = Pins::hold();
    let Some(w) = german_industrial_stride_window() else {
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
    // Item 3667: 135 keys. The standing families — 63 buildings'
    // `build:extra` (which the stride does not print), `form` on 27 units,
    // and who=1's groups' `held` and pools 64..71 as each is minted — and, **the
    // first unit rows, from 3001: who=1's Albatrosses (TY 288) at
    // `myspeed` 55 in ours against 68 in theirs**, each as it is built,
    // 1/41's on 3901 with `g.z[0]` 0 against 426. The human's Citizens'
    // damage parts from 3601 (`0/13` `hits_left` ours 34 theirs 31) and
    // on 4301 (`0/11` 23 against 19, `0/12` 53 against 52).
    pin_eq!(w.firsts.len(), 135, "the stride's baseline");
}

/// run1334b: the word 4323's window, blocks 4295..4574 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4301 and 250 after the word's block 4324 (item 3667).
pub(crate) const RUN1334B: &str =
    "gamelog-run1334b-greatlakes-german-industrial-window-4295-4574.txt";

/// The window: block 4295 through 4574; the word 4323's own block is 4324.
pub(crate) const WIDENING_GERMAN_INDUSTRIAL_FRAME_4323: (i64, i64) = (4295, 4574);

/// **The word 4323's window**: run1334b walked from run1333's start with
/// the recorder on and every round compared (item 3667).
pub(crate) fn german_industrial_frame_4323_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[GERMAN_INDUSTRIAL.start],
        true,
        GERMAN_INDUSTRIAL.long,
        "run1334b",
        &[(RUN1334B, WIDENING_GERMAN_INDUSTRIAL_FRAME_4323.0)],
        WIDENING_GERMAN_INDUSTRIAL_FRAME_4323,
        1,
        &[4324],
        true,
    )
}

#[test]
fn german_industrial_pair_run1334b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = german_industrial_frame_4323_window() else {
        return;
    };
    pin_eq!(w.blocks, 279, "every captured block");
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
    // Item 3667, the first walk: 2841 keys. Block 4295 carries the
    // standing families (`form`, both leaders, who=1's groups and pools),
    // the Albatrosses' `myspeed` (55 against 68) and the Citizens' damage
    // the stride already parts (`0/11` `hits_left` 23 against 19, `0/12`
    // 53 against 52, `0/13` 49 against 46). **On the word's block 4324
    // the human's leader holds a unit more in ours** (`num_units[0]` 13
    // against 12, `peasants` likewise) and `0/11` is ours alone: theirs'
    // draw at index 11 is `Unit::close+0xcb6`, the Citizen's death.
    pin_eq!(w.firsts.len(), 2841, "run1334b baseline");
    // Every round, both directions: 1018 round-blocks theirs, 246 ours,
    // 168 parting. **The first launch that parts is on 4304, who=1's A7V
    // `1/48`** (TY 242, the Light Tank): theirs fires from (5427, 31534)
    // at `sz` 587, ours from the figure's own square (5544, 31512) at
    // 409; the German Anti-tank Rifles `1/50`, `1/51` (TY 138) part the
    // same way on 4305, `1/51`'s in 2 frames against 3. The hypothesis:
    // neither piece has a `launch::BAYS` row (`docs/AI.md` §346).
    pin_eq!(w.rounds_read, (1018, 246), "the rounds read");
    pin_eq!(w.rounds.len(), 168, "the rounds that part");
}
