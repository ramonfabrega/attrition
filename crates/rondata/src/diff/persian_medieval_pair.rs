//! **The Persians at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! item 3535): the coverage pair's own lobby (run652's: East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Persians (23)** at
//! Difficulty 5, seed 12345, `STARTING_RESOURCES 7`) with the age held at
//! Medieval instead of All Technologies: `STARTING_TECHNOLOGY 2`,
//! `ENDING_TECHNOLOGY 2`. Captures: run1180 (a `DUMP_ALL` start) and
//! run1181 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §308.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const PERSIAN_MEDIEVAL: Pair = Pair {
    name: "EastIndiesPersianMedieval",
    map_style: 18,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1180-eastindies-persian-medieval-start.txt",
    long: (
        "gamelog-run1181-eastindies-persian-medieval-trace.txt",
        "rontrace-run1181.log",
    ),
    // The game ended itself on 4759 (closing block 4760).
    length: 4759,
    // Item 3535, first try: frame 2226 by the count and the sequence, ours
    // 10 draws against 8, index 0 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`
    // (`docs/AI.md` section 308).
    count: 2226,
    sequence: 2226,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_MEDIEVAL_PAIR_WORD: i64 =
    if PERSIAN_MEDIEVAL.count < PERSIAN_MEDIEVAL.sequence {
        PERSIAN_MEDIEVAL.count
    } else {
        PERSIAN_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_MEDIEVAL {}: count {} sequence {} of {}; {}",
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
fn persian_medieval_pair_captures_say_their_lobby() {
    let p = &PERSIAN_MEDIEVAL;
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
        "the Persian Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: its triple is the coverage pair's (item 1466),
/// which the held age alone tells apart.
#[test]
fn persian_medieval_pair_is_no_other_lobby() {
    let p = &PERSIAN_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1181b: the word 2226's window, blocks 2220..2476 at the long's
/// detail — six blocks before the word's block 2227 and 250 after (item
/// 3535).
pub(crate) const RUN1181B: &str =
    "gamelog-run1181b-eastindies-persian-medieval-window-2220-2476.txt";

/// The window: block 2220 through 2476; the word 2226's own block is 2227.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_FRAME_2226: (i64, i64) = (2220, 2476);

/// **The word 2226's window**: run1181b walked from run1180's start with
/// the recorder on (item 3535).
pub(crate) fn persian_medieval_frame_2226_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1181b",
        &[(RUN1181B, WIDENING_PERSIAN_MEDIEVAL_FRAME_2226.0)],
        WIDENING_PERSIAN_MEDIEVAL_FRAME_2226,
        1,
        &[2227],
        true,
    )
}

#[test]
fn persian_medieval_pair_run1181b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_frame_2226_window() else {
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
    // Item 3535, the first walk: 1130 keys. Block 2220 carries 79 rows:
    // the standing families (`form`, `SITE[i].reg`, who=1's groups'
    // `held`) and **the Explorer `1/0` (TypeIndex 71, land)**, whose
    // orders are already parted: theirs holds two — the
    // `EXPLORETOORDER` to (34296, 33528) under a current
    // `ATTACKTOORDER` to (42072, 43368), its path 16 nodes — and ours the
    // explore order alone (`order:kind` ours 3 theirs 2, a path of 1 to
    // (34272, 33504)). On 2226 both reach (34272, 33504); ours' order is
    // done and the unit idles (`Unit::do_idle`), theirs walks on
    // (`heading` ours -1073741824 theirs 1962213376, `orders_x` 34272
    // against 34296), and block 2227 reads `idle` ours 1 theirs 0. The
    // long's stride blocks place theirs' two orders between 2101 (none)
    // and 2201 (both); no dump on disk covers the span at detail
    // (`docs/AI.md` section 308).
    pin_eq!(w.firsts.len(), 1130, "run1181b baseline");
}
