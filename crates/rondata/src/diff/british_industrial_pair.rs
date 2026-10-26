//! **The British at a held Industrial age, Great Sahara, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97 §3, item 3668): Great Sahara, `MAP_STYLE
//! 7`, human Nubians (4) against the **British (11)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple, `(7, 11, 4)`,
//! is new: every other British lobby is at 0 or 5 but the Classical pair's
//! `(18, 11, 4)`, another map, and the battery fields no British. Captures:
//! run1335 (a `DUMP_ALL` start) and run1336 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §347.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BRITISH_INDUSTRIAL: Pair = Pair {
    name: "GreatSaharaBritishIndustrial",
    map_style: 7,
    ai_tribe: 11,
    difficulty: 4,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1335-greatsahara-british-industrial-start.txt",
    long: (
        "gamelog-run1336-greatsahara-british-industrial-trace.txt",
        "rontrace-run1336.log",
    ),
    // The game ended itself on 4767 (closing block 4768).
    length: 4767,
    // Item 3668, first try: frame 2382, by the count and the sequence —
    // ours 71 draws against 15, parting at index 5, ours
    // `Leader::produce_building+0xc99`, theirs `Leader::make_stuff+0x63d`
    // (`docs/AI.md` §347).
    count: 2382,
    sequence: 2382,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_INDUSTRIAL_PAIR_WORD: i64 =
    if BRITISH_INDUSTRIAL.count < BRITISH_INDUSTRIAL.sequence {
        BRITISH_INDUSTRIAL.count
    } else {
        BRITISH_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_INDUSTRIAL {}: count {} sequence {} of {}; {}",
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
fn british_industrial_pair_captures_say_their_lobby() {
    let p = &BRITISH_INDUSTRIAL;
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
        "the British Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3668; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// British; and no other British lobby's: every other British triple is at
/// difficulty 0 or 5 but the Classical pair's `(18, 11, 4)`, another map
/// (item 3668).
#[test]
fn british_industrial_pair_is_no_other_lobby() {
    let p = &BRITISH_INDUSTRIAL;
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
        "the battery fields the British"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.2 == 0 || l.2 == 5 || l.0 != p.map_style,
            "{l:?} is another British lobby on this map at difficulty {}",
            l.2
        );
    }
    for other in [
        &super::british_classical_pair::BRITISH_CLASSICAL,
        &super::british_enlightenment_pair::BRITISH_ENLIGHTENMENT,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
}

/// **The long's stride, walked back from the word 2382** (item 3668):
/// only run1336's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_STRIDE: (i64, i64) = (101, 2301);

pub(crate) fn british_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336 stride",
        &[(
            BRITISH_INDUSTRIAL.long.0,
            WIDENING_BRITISH_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_BRITISH_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_the_long_s_stride_walks_back_from_2382() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every stride block in the window");
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
    // Item 3668: 80 keys, all of them the standing families — 41
    // buildings' `build:extra` (which the stride does not print), `form`
    // on 27 units, and who=1's groups' `held` and pools 64..69 as each is
    // minted. **No unit row parts on any stride block to 2301**: the
    // parting is in 2302..2382, which is run1336b's.
    pin_eq!(w.firsts.len(), 80, "the stride's baseline");
}

/// run1336b: the word 2382's window, blocks 2295..2633 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2301 and 250 after the word's block 2383 (item 3668).
pub(crate) const RUN1336B: &str =
    "gamelog-run1336b-greatsahara-british-industrial-window-2295-2633.txt";

/// The window: block 2295 through 2633; the word 2382's own block is 2383.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_FRAME_2382: (i64, i64) = (2295, 2633);

/// **The word 2382's window**: run1336b walked from run1335's start with
/// the recorder on and every round compared (item 3668).
pub(crate) fn british_industrial_frame_2382_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336b",
        &[(RUN1336B, WIDENING_BRITISH_INDUSTRIAL_FRAME_2382.0)],
        WIDENING_BRITISH_INDUSTRIAL_FRAME_2382,
        1,
        &[2383],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_frame_2382_window() else {
        return;
    };
    pin_eq!(w.blocks, 338, "every captured block");
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
    // Item 3668, the first walk: 1189 keys. **Block 2295 already carries
    // who=1's leader and two of its buildings' queues parted**, beside the
    // standing families: `1/2013` and `1/2024` queue the Flamethrower
    // (131) in ours where theirs queues the Machine Gun (123)
    // (`queue[*].type`, `num_queued[81]` 5 against `num_queued[73]` 5),
    // and the `MAKE` list, three buckets and the human leader's
    // `SITE[*].reg` and `aggression[1]` stand parted too. On the word
    // 2382 who=1's `MAKE[3]` is ours `cat` 4 at city 2 against theirs'
    // 6 at city 0: ours places a building (`1/2034`, 15000
    // `constr_time`) theirs does not (`docs/AI.md` §347).
    pin_eq!(w.firsts.len(), 1189, "run1336b baseline");
    // No round is in flight on either side over the window.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1336c: the long's stride to 2301 again, with `BUILDS=7`, `CITIES=5`
/// and `LEADERS=9` on every stride block — run1336 prints neither the
/// leaders nor the buildings' queues, which run1336b's first block 2295
/// carries parted (item 3668).
pub(crate) const RUN1336C: &str =
    "gamelog-run1336c-greatsahara-british-industrial-leaders-stride.txt";

/// The leaders' stride: block 101 through 2301.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_LEADERS: (i64, i64) = (101, 2301);

pub(crate) fn british_industrial_leaders_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336c",
        &[(RUN1336C, WIDENING_BRITISH_INDUSTRIAL_LEADERS.0)],
        WIDENING_BRITISH_INDUSTRIAL_LEADERS,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336c_s_leaders_stride_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_leaders_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
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
    // Item 3668: 97 keys. From 101 the standing families — `form` on 27
    // units, who=1's groups and pools 64..69, and both leaders'
    // `SITE[*].reg` and `aggression` (§2.7's seam) — and nothing else to
    // 2101. **On 2201 who=1's make list and two buildings' queues part**,
    // 38 keys: `1/2013`'s `queue[3]` and `1/2024`'s `queue[0..3]` hold the
    // Flamethrower (131) in ours and the Machine Gun (123) in theirs,
    // `num_queued[81]` 5 against `num_queued[73]` 5, the `MAKE[0..3]`
    // rows and the food, wealth and metal buckets. Both sides agree on
    // 2101; run1336d holds the gap.
    pin_eq!(w.firsts.len(), 97, "run1336c baseline");
}

/// run1336d: the gap 2095..2201 between run1336c's agreeing 2101 and its
/// parted 2201, at the window's detail (item 3668).
pub(crate) const RUN1336D: &str =
    "gamelog-run1336d-greatsahara-british-industrial-gap-2095-2201.txt";

/// The gap: block 2095 through 2201.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_GAP: (i64, i64) = (2095, 2201);

pub(crate) fn british_industrial_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336d",
        &[(RUN1336D, WIDENING_BRITISH_INDUSTRIAL_GAP.0)],
        WIDENING_BRITISH_INDUSTRIAL_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336d_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 106, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3668: 93 keys. Block 2095 carries the standing families
    // alone; **the first record to part is who=1's make list on 2184**,
    // rebuilt that frame: theirs heads it with the Anti-Aircraft Gun (281,
    // `val` 1028320) and holds the Machine Gun (123, `val` 78152, `num` 2)
    // at `MAKE[3]`; ours heads it with the Flamethrower (131, `val`
    // 1014609, `num` 2), which theirs' list never holds, and the AA Gun
    // second at 941484. The queues take the pick on 2186 (`1/2013`'s
    // `queue[3]`, `1/2024`'s `queue[0..3]`: 131 against 123), and the
    // buckets follow (`docs/AI.md` §347).
    pin_eq!(w.firsts.len(), 93, "run1336d baseline");
    // No round is in flight on either side over the gap.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}
