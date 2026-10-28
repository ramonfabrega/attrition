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
    // (`docs/AI.md` §368). Item 3744: **whole, 6706 of 6706** — on 5887
    // the MLRS `1/99`'s attack-move look ranks the city `0/2000` and
    // `find_nearby_target`'s siege-city arm makes the attack mandatory and
    // points army 1/0's muster at it, where ours added it with mandatory 0
    // (run1423b: `attack[0].mandatory` 0 against 1); 6153's idle draw goes
    // (`docs/AI.md` §368.1).
    count: 6706,
    sequence: 6706,
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
    // Item 3744, the siege-city arm: 293 — the MLRS `1/99`'s
    // `attack[1].mandatory` on 5901 goes (`docs/AI.md` §368.1).
    pin_eq!(w.firsts.len(), 293, "the stride's baseline");
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
    //
    // Item 3744, the siege-city arm landed (`docs/AI.md` §368.1): **169**.
    // Group 72's rows go — `1/99`'s move, `1/114`'s guard, the idle on
    // 6154 — and what is left is the standing families (`form`, group
    // 95's pool from 6202) and `1/198`'s `airorder.mandatory` on 6203.
    pin_eq!(w.firsts.len(), 169, "run1401b baseline");
    // Every round, both directions: 408 in ours against 227. Item 3744:
    // 408 against 408, 98 parting — the MLRS `1/99`'s launch point (40,
    // `sx`/`sy`/`sz`, 3744a) and `1/198`'s rounds (58, 3744b); none spends
    // a draw. Item 3760, the Strategic Bomber's bays (piece 255) and the
    // Jet Fighter's guns (245): 98 → **46** (`docs/AI.md` §372.4).
    pin_eq!(w.rounds_read, (408, 408), "the rounds read");
    pin_eq!(w.rounds.len(), 46, "the rounds that part");
}

/// run1423: the stride's last gap, blocks 5895..6101 at run1401b's detail
/// (item 3744). The stride parts group 72's MLRS on 5901 and nothing on
/// disk prints a block between 5901 and 6095, so the walk back from 6141
/// had no frame to stand on.
pub(crate) const RUN1423: &str =
    "gamelog-run1423-eastindies-russian-information-window-5895-6101.txt";

/// The window: block 5895 through 6101.
pub(crate) const WIDENING_RUSSIAN_INFORMATION_GAP_5895: (i64, i64) = (5895, 6101);

/// **The gap's window**: run1423 walked from run1400's start with every
/// round compared (item 3744).
pub(crate) fn russian_information_gap_5895_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_INFORMATION.start],
        true,
        RUSSIAN_INFORMATION.long,
        "run1423",
        &[(RUN1423, WIDENING_RUSSIAN_INFORMATION_GAP_5895.0)],
        WIDENING_RUSSIAN_INFORMATION_GAP_5895,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_information_pair_run1423_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_information_gap_5895_window() else {
        return;
    };
    // Item 3744: 5895..6101 is 207 frames; the walk reads 206 blocks.
    pin_eq!(w.blocks, 206, "every captured block");
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
    // Item 3744, with the siege-city arm: **168** — the stride's standing
    // families and rows (the leader's `leftover`, `1/74`, `mirror`,
    // `g.angle[0]`, the Strategic Bombers' air orders) and nothing of
    // group 72. Before the arm, 170: `1/99`'s `attack[1].mandatory` (0
    // against 1) standing from the window's first block and its
    // `attack[0].mandatory` on 5908.
    pin_eq!(w.firsts.len(), 168, "run1423 baseline");
    pin_eq!(w.rounds_read, (408, 408), "the rounds read");
    // 94 parting: the MLRS `1/99`'s launch point from 5913 (3744a). Item
    // 3760, the Strategic Bomber's bays and the Jet Fighter's guns: 94 →
    // **40** (`docs/AI.md` §372.4).
    pin_eq!(w.rounds.len(), 40, "the rounds that part");
}

/// run1423b: the stride's gap before run1423, blocks 5795..5900 (item
/// 3744): the MLRS `1/99` leaves its carrier and takes group 72's orders
/// between the stride's 5801 and run1423's first block.
pub(crate) const RUN1423B: &str =
    "gamelog-run1423b-eastindies-russian-information-window-5795-5900.txt";

/// The window: block 5795 through 5900.
pub(crate) const WIDENING_RUSSIAN_INFORMATION_GAP_5795: (i64, i64) = (5795, 5900);

/// **The earlier gap's window**: run1423b walked from run1400's start with
/// every round compared (item 3744).
pub(crate) fn russian_information_gap_5795_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_INFORMATION.start],
        true,
        RUSSIAN_INFORMATION.long,
        "run1423b",
        &[(RUN1423B, WIDENING_RUSSIAN_INFORMATION_GAP_5795.0)],
        WIDENING_RUSSIAN_INFORMATION_GAP_5795,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_information_pair_run1423b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_information_gap_5795_window() else {
        return;
    };
    pin_eq!(w.blocks, 105, "every captured block");
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
    // Item 3744, with the siege-city arm: **175**, the standing families
    // and rows from 5795 and the Strategic Bombers' air orders — `1/196`'s
    // order kind on 5805 (16 against 17) the first. Before the arm the
    // MLRS `1/99` parted on 5887: its attack-move look ranks the city
    // `0/2000` at `attack_dist` 4684, past the packed shortcut's 4608, and
    // adds the attack `mandatory` 0 in ours against 1.
    pin_eq!(w.firsts.len(), 175, "run1423b baseline");
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1423c: the closing window, blocks 6450..6707 at run1401b's detail
/// (item 3744). The word reached the game's last frame; run1401b ends on
/// 6403 and run1401's stride prints units only.
pub(crate) const RUN1423C: &str =
    "gamelog-run1423c-eastindies-russian-information-closing-window-6450-6707.txt";

/// The window: block 6450 through the closing block 6707.
pub(crate) const WIDENING_RUSSIAN_INFORMATION_CLOSING: (i64, i64) = (6450, 6707);

/// **The closing window**: run1423c walked from run1400's start with every
/// round compared (item 3744).
pub(crate) fn russian_information_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_INFORMATION.start],
        true,
        RUSSIAN_INFORMATION.long,
        "run1423c",
        &[(RUN1423C, WIDENING_RUSSIAN_INFORMATION_CLOSING.0)],
        WIDENING_RUSSIAN_INFORMATION_CLOSING,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_information_pair_run1423c_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_information_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
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
    // Item 3744: **157** keys over the 257 blocks. The standing families
    // (`form`, who=1's pools and `held`, the leader's rows, `mirror`,
    // `g.angle[0]`, `1/74` at rest) and two late rows no draw reads: the
    // city `0/2000` on 6636 (`damage` 1191 against 1200, `city_flags`
    // `0x2`/`0x4`, `reduce_stamp` 6310 against 6635; 3744d) and `1/49`'s
    // `dest_angle` on 6703 (3744e).
    pin_eq!(w.firsts.len(), 157, "run1423c baseline");
    pin_eq!(w.rounds_read, (29, 30), "the rounds read");
    // 29 against 30, 12 parting, every one the T80 `1/59`'s (TI 260)
    // launch point from 6605 (3744d).
    pin_eq!(w.rounds.len(), 12, "the rounds that part");
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
    // Item 3744, the game whole: 195 units, none torn, unlinked or extra;
    // one off — the Attack Submarine `1/74` at rest (−96, −720), the
    // stride's standing row since 2801 (3721a).
    pin_eq!(e.compared, 195, "units compared");
    pin_eq!(e.counts(), [1, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
