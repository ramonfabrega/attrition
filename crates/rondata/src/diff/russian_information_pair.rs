//! **The Russians at a held Information age, East Indies** (DECISIONS
//! 90, 91, 93, 94, 97, 98, 99, item 3721): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **Russians (13)** at difficulty 4, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple, `(18, 13,
//! 4)`, is new: the Russians hold Great Lakes at 2 (`russian_word`,
//! `GreatLakesRussianTier2`) and 5 (`russian_gunpowder_pair`), East Indies
//! at 5 (`russian_modern_pair`), and Great Sahara at 4 (the held-out
//! `GreatSaharaRussianTier4`) and 5 (`russian_medieval_pair`). Captures:
//! run1400 (a `DUMP_ALL` start) and run1401 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from
//! the start's initial state). The first try is in `docs/AI.md` §368.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const RUSSIAN_INFORMATION: Pair = Pair {
    name: "EastIndiesRussianInformation",
    map_style: 18,
    ai_tribe: 13,
    difficulty: 4,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1400-eastindies-russian-information-start.txt",
    long: (
        "gamelog-run1401-eastindies-russian-information-trace.txt",
        "rontrace-run1401.log",
    ),
    // The game ended itself on 6706 (closing block 6707).
    length: 6706,
    // Item 3721, first try: count 6153, sequence 6153 of 6706 — on 6153
    // ours 562 draws against theirs' 554, index 3 ours' `Guy::set_anim+0x97a
    // < Unit::do_idle+0x7d` against theirs' `PathFinder::calc_road_cost+0x46`
    // (`docs/AI.md` §368).
    count: 6153,
    sequence: 6153,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const RUSSIAN_INFORMATION_PAIR_WORD: i64 =
    if RUSSIAN_INFORMATION.count < RUSSIAN_INFORMATION.sequence {
        RUSSIAN_INFORMATION.count
    } else {
        RUSSIAN_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn russian_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN_INFORMATION {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 7 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn russian_information_pair_captures_say_their_lobby() {
    let p = &RUSSIAN_INFORMATION;
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
        "the Russian Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3721; the battery's identity
/// test refuses it), once, and none of the battery's — whose two Russian
/// games are `GreatLakesRussianTier2`, `(14, 13, 2)`, and the held-out
/// `GreatSaharaRussianTier4`, `(7, 13, 4)`, other maps — and no other
/// Russian lobby's: `russian_modern_pair`'s `(18, 13, 5)`, this map at
/// another difficulty, `russian_gunpowder_pair`'s `(14, 13, 5)`,
/// `russian_medieval_pair`'s `(7, 13, 5)` and `russian_word`'s `(14, 13,
/// 2)` (item 3721).
#[test]
fn russian_information_pair_is_no_other_lobby() {
    let p = &RUSSIAN_INFORMATION;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    for l in battery::BATTERY.iter().filter(|l| l.ai_tribe == p.ai_tribe) {
        assert_ne!(
            (l.map_style, l.ai_tribe, l.difficulty),
            triple,
            "{} is this lobby",
            l.name
        );
        assert_ne!(
            l.map_style, p.map_style,
            "{} fields the Russians here",
            l.name
        );
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is this lobby"
        );
    }
    let m = &super::russian_modern_pair::RUSSIAN_MODERN;
    let g = &super::russian_gunpowder_pair::RUSSIAN_GUNPOWDER;
    let d = &super::russian_medieval_pair::RUSSIAN_MEDIEVAL;
    let w = &super::russian_word::RUSSIAN;
    for (name, l) in [
        (m.name, (m.map_style, m.ai_tribe, m.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (d.name, (d.map_style, d.ai_tribe, d.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Russians");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3721, from
/// 6153): only run1401's stride blocks are on disk before the word, 101
/// through 6101.
pub(crate) const WIDENING_RUSSIAN_INFORMATION_STRIDE: (i64, i64) = (101, 6101);

pub(crate) fn russian_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_INFORMATION.start],
        true,
        RUSSIAN_INFORMATION.long,
        "run1401 stride",
        &[(
            RUSSIAN_INFORMATION.long.0,
            WIDENING_RUSSIAN_INFORMATION_STRIDE.0,
        )],
        WIDENING_RUSSIAN_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = russian_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 61, "every stride block in the window");
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
    // Item 3721: 294 keys over 61 blocks. The standing families — `form`
    // 110, `build:extra` 88, who=1's groups' `held` 37 and pools 37 — and
    // 22 rows besides: the Attack Submarine `1/74` (TI 339, sea) at rest
    // 96 and 720 off from 2801 (`pos`, `orders`); from 5101 `mirror` (9)
    // and `g.angle[0]` (4) on who=1's fresh units; on 5901 the Strategic
    // Bombers' (TI 305) `airorder.returning`/`mandatory` (3) and the MLRS
    // `1/99`'s (TI 273) `attack[1].mandatory`, 0 against 1.
    pin_eq!(w.firsts.len(), 294, "the stride's baseline");
}

/// run1401b: the word 6153's window, blocks 6095..6403 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 6101 and 250 past the word's own block 6154 (item 3721).
/// run1401 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1401B: &str =
    "gamelog-run1401b-eastindies-russian-information-window-6095-6403.txt";

/// The window: block 6095 through 6403.
pub(crate) const WIDENING_RUSSIAN_INFORMATION_FRAME_6153: (i64, i64) = (6095, 6403);

/// **The word 6153's window**: run1401b walked from run1400's start with
/// the recorder on and every round compared (item 3721).
pub(crate) fn russian_information_frame_6153_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_INFORMATION.start],
        true,
        RUSSIAN_INFORMATION.long,
        "run1401b",
        &[(RUN1401B, WIDENING_RUSSIAN_INFORMATION_FRAME_6153.0)],
        WIDENING_RUSSIAN_INFORMATION_FRAME_6153,
        1,
        &[6154],
        true,
    )
}

#[test]
fn russian_information_pair_run1401b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_information_frame_6153_window() else {
        return;
    };
    pin_eq!(w.blocks, 308, "every captured block");
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
    // Item 3721, the first walk: **1900** keys. Block 6095 carries the
    // stride's standing families and rows (157). **The word's units part
    // on 6141, both sides acting**: who=1's group 72 takes new orders and
    // the points part — the MLRS `1/99` (TI 273) moves to (6734, 7090) in
    // ours against (6491, 9412), the Flamethrower `1/114` (TI 131) guards
    // (7128, 6888) against (7272, 7848). On 6153 ours' `1/99` ends its
    // move (`orders.len` 0 against 1) and on 6154 idles (`idle` 1 against
    // 0): the word's `do_idle` draw. The first round to part is the
    // `1/198`'s on 6295 (`sx` 6601 against 6565).
    pin_eq!(w.firsts.len(), 1900, "run1401b baseline");
    // Every round, both directions: 408 in ours against 227.
    pin_eq!(w.rounds_read, (408, 227), "the rounds read");
    pin_eq!(w.rounds.len(), 66, "the rounds that part");
}

/// **The pair's closing state** (item 3721): run1401's closing block 6707,
/// the whole map's units, walked from run1400's start.
#[test]
fn russian_information_pair_run1401_s_closing_state() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_INFORMATION;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Russian Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 6707, "the closing block");
    // Item 3721, first try: 553 frames past the word 6153 — 194 units
    // compared, counts [112, 1, 2, 2, 50, 1, 11], none torn, `1/97`
    // unlinked, `1/177` and `1/126` extra. A floor, not a score.
    pin_eq!(e.compared, 194, "units compared");
    pin_eq!(e.counts(), [112, 1, 2, 2, 50, 1, 11], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        vec![(1, 177), (1, 126)],
        "the closing extras"
    );
}
