//! **The Egyptians at a held Industrial age, East Indies, at Hard**
//! (DECISIONS 90, 91, 93, 94, 97, 98, item 3700): East Indies, `MAP_STYLE
//! 18`, human Nubians (4) against the **Egyptians (7)** at **Hard (4)**,
//! seed 12345, `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple, `(18, 7,
//! 4)`, is new: the Egyptian lobbies are `egyptian_word`'s `(7, 7, 4)`,
//! `egyptian_gunpowder_pair`'s `(7, 7, 5)`, `egyptian_classical_pair`'s
//! `(18, 7, 5)` and the held-out `EastIndiesEgyptianTier3`'s `(18, 7, 3)`
//! — another map or another difficulty. Captures: run1373 (a `DUMP_ALL`
//! start) and run1374 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §360.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const EGYPTIAN_INDUSTRIAL: Pair = Pair {
    name: "EastIndiesEgyptianIndustrial",
    map_style: 18,
    ai_tribe: 7,
    difficulty: 4,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1373-eastindies-egyptian-industrial-start.txt",
    long: (
        "gamelog-run1374-eastindies-egyptian-industrial-trace.txt",
        "rontrace-run1374.log",
    ),
    // The game ended itself on 5689 (closing block 5690).
    length: 5689,
    // Item 3700, first try: frame 4959 of 5689 by the count and the
    // sequence. On 4959 ours spends 14 game draws against theirs' 16,
    // parting at index 3 — ours `Guy::set_anim+0x97a < Guy::move+0x19f`,
    // theirs `Guy::set_anim+0x97a < Unit::move_step+0x823`; theirs also
    // spends a fourth gaia `Guy::set_anim+0x104b` (`docs/AI.md` §360).
    // Item 3707: **5689 of 5689, the game's last frame** — piece 193 reads
    // piece 189's turret rows (`sim::pivot::SAME_AS`), and who=1's Light
    // Tank `1/54`'s round of 4940 leaves (6098, 10851, 200), theirs, and
    // wounds the human's Citizen `0/10` on 4943 on both sides (§360.2).
    count: 5689,
    sequence: 5689,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const EGYPTIAN_INDUSTRIAL_PAIR_WORD: i64 =
    if EGYPTIAN_INDUSTRIAL.count < EGYPTIAN_INDUSTRIAL.sequence {
        EGYPTIAN_INDUSTRIAL.count
    } else {
        EGYPTIAN_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn egyptian_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn egyptian_industrial_pair_captures_say_their_lobby() {
    let p = &EGYPTIAN_INDUSTRIAL;
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
        "the Egyptian Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3700; the battery's identity
/// test refuses it), once, and no other Egyptian lobby's: the battery's
/// held-out `EastIndiesEgyptianTier3` `(18, 7, 3)`, another difficulty;
/// `egyptian_word`'s `(7, 7, 4)` and `egyptian_gunpowder_pair`'s `(7, 7,
/// 5)`, another map; `egyptian_classical_pair`'s `(18, 7, 5)`, another
/// difficulty (item 3700).
#[test]
fn egyptian_industrial_pair_is_no_other_lobby() {
    let p = &EGYPTIAN_INDUSTRIAL;
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
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Egyptian lobby on this map at this difficulty"
        );
    }
    let w = &super::egyptian_word::EGYPTIAN;
    let g = &super::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER;
    let c = &super::egyptian_classical_pair::EGYPTIAN_CLASSICAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Egyptians");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 4959** (item 3700):
/// only run1374's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_EGYPTIAN_INDUSTRIAL_STRIDE: (i64, i64) = (101, 4901);

pub(crate) fn egyptian_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_INDUSTRIAL.start],
        true,
        EGYPTIAN_INDUSTRIAL.long,
        "run1374 stride",
        &[(
            EGYPTIAN_INDUSTRIAL.long.0,
            WIDENING_EGYPTIAN_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_EGYPTIAN_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn egyptian_industrial_pair_the_long_s_stride_walks_back_from_4959() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_industrial_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 49, "every stride block in the window");
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
    // Item 3700: 155 keys, every one a standing family — `form` on 35
    // units (the last, `1/112`, on 4901), the buildings' `build:extra`
    // (which the stride does not print) and who=1's groups 64..84's `held`
    // and pools as each is minted. No unit row parts on the stride.
    pin_eq!(w.firsts.len(), 155, "the stride's baseline");
}

/// run1374b: the word 4959's window, blocks 4895..5210 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 4901 and 250 past the word's own block 4960 (item 3700).
pub(crate) const RUN1374B: &str =
    "gamelog-run1374b-eastindies-egyptian-industrial-window-4895-5210.txt";

/// The window: block 4895 through 5210; the word 4959's own block is 4960.
pub(crate) const WIDENING_EGYPTIAN_INDUSTRIAL_FRAME_4959: (i64, i64) = (4895, 5210);

/// **The word 4959's window**: run1374b walked from run1373's start with
/// the recorder on and every round compared (item 3700).
pub(crate) fn egyptian_industrial_frame_4959_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_INDUSTRIAL.start],
        true,
        EGYPTIAN_INDUSTRIAL.long,
        "run1374b",
        &[(RUN1374B, WIDENING_EGYPTIAN_INDUSTRIAL_FRAME_4959.0)],
        WIDENING_EGYPTIAN_INDUSTRIAL_FRAME_4959,
        1,
        &[4960],
        true,
    )
}

#[test]
fn egyptian_industrial_pair_run1374b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_industrial_frame_4959_window() else {
        return;
    };
    pin_eq!(w.blocks, 315, "every captured block");
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
    // Item 3700, the first walk: **1132** keys. Block 4895 carries the
    // standing families (both leaders' `SITE[i].reg`, who=0's
    // `aggression[1]`) and who=1's `bucket[0:food]` 11965 against 11966,
    // `escrow[0:food]` 82 against 83 and `leftover[2:wealth]` 4576 against
    // 5536 — standing from before the window, where no stride block prints
    // a leader (parked 3700a). **The first unit row is the human's Citizen `0/10` (TI 50)
    // on 4944**: wounded on 4943 in theirs (`damage_frame` 4943,
    // `hits_left` 33 against ours' 65) and ordered to move (5544, 7320),
    // both leaders' `treaties` 3 against ours' 1 with it. The round is the
    // Light Tank `1/54`'s (TI 243, TY 193 `A7Vd`, piece 193): on 4940 its
    // `ammo[0]` leaves (6120, 10968, 22) in ours — the tank's own square —
    // against theirs' (6098, 10851, 200), and flies a frame longer. On the
    // word 4959 theirs' `0/10` is still moving (`Unit::move_step+0x823`,
    // index 3) and ours' stands.
    // Item 3707, piece 193 on piece 189's turret rows: 1132 → **119**. No
    // unit row but group 71's `order:target` (`0/10` ours, `0/2003` theirs
    // from 4949 — the buried `ATTACKORDER` at the bottom of theirs' ring;
    // the current order is `0/10` on both, parked 3707a); the rest is
    // block 4895's standing families, `form` on `1/112` and `1/122`,
    // who=1's `bucket[2:wealth]` one low on 4943 (3700a's family), and
    // groups 65, 83 and 84's `held` and pools as each is minted.
    pin_eq!(w.firsts.len(), 119, "run1374b baseline");
    pin_eq!(w.rounds_read, (46, 46), "the rounds read");
    // Every round, both directions: 46 round-blocks ours, 54 theirs; 13
    // rows part, the first `1/54`'s launch on 4940 (`total_time` 6 against
    // 5, `v1z` 65629173 against 31618750): piece 193 has no
    // `launch::BAYS` row, the word's residue (`docs/AI.md` §360). After
    // it, rounds one side holds alone: the Light Tank `1/66`'s and group
    // 71's Riflemen `1/68`, `1/76` and `1/77`'s, downstream of the wound.
    // (The pair is `(theirs, ours)`: 46 theirs, 54 ours.) Item 3707: 46
    // against 46, **none parting** — `1/54`'s round of 4940 leaves the
    // turret's muzzle, as do `1/54`'s and `1/66`'s six after it.
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1384: the pair's last blocks, 5433..5690 at the long's detail with
/// `LEADERS=9` and `AMMO=5`, through the closing block (item 3707): no dump
/// held them once the word reached the game's last frame — run1374's
/// stride prints units every hundred frames and no `AMMO`, leader,
/// building or city record past the start, and run1374b ends on 5210.
pub(crate) const RUN1384: &str =
    "gamelog-run1384-eastindies-egyptian-industrial-window-5433-5690.txt";

/// The closing window: block 5433 through the closing block 5690.
pub(crate) const WIDENING_EGYPTIAN_INDUSTRIAL_CLOSING: (i64, i64) = (5433, 5690);

/// **The pair's last blocks**: run1384 walked from run1373's start with
/// the recorder on and every round compared (item 3707).
pub(crate) fn egyptian_industrial_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_INDUSTRIAL.start],
        true,
        EGYPTIAN_INDUSTRIAL.long,
        "run1384",
        &[(RUN1384, WIDENING_EGYPTIAN_INDUSTRIAL_CLOSING.0)],
        WIDENING_EGYPTIAN_INDUSTRIAL_CLOSING,
        1,
        &[5690],
        true,
    )
}

#[test]
fn egyptian_industrial_pair_run1384_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_industrial_closing_window() else {
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
    // Item 3707: **128 keys**, all standing — on 5433 `form` on 45 units,
    // both leaders' `SITE[i].reg`, who=0's `aggression[1]`, who=1's
    // `bucket`/`escrow`/`leftover` (3700a's family) and groups and pools;
    // after it, who=1's `bucket[2:wealth]` one low on 5436, `form` on
    // eighteen units as each is built, and groups 69 and 83 minted (5488,
    // 5516). No unit position, order, building or city row parts.
    pin_eq!(w.firsts.len(), 128, "run1384 baseline");
    // Every round, both directions: 122 a side, 19 parting — the
    // Artillery's (TY 269) seven launches from `1/81`, `1/89` and `1/91`
    // at their figures' squares, 136 low (5441..5621; 3683b's unfitted
    // bays, parked 3707b). None moves a draw.
    pin_eq!(w.rounds_read, (122, 122), "the rounds read");
    pin_eq!(w.rounds.len(), 19, "the rounds that part");
}

/// **The pair's closing state** (item 3700): run1374's closing block 5690,
/// the whole map's units, walked from run1373's start.
#[test]
fn egyptian_industrial_pair_run1374_s_closing_state() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_INDUSTRIAL;
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
        "Egyptian Industrial pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5690, "the closing block");
    // Item 3700, the first try: the pair parts at 4959, and the closing
    // block 5690 compares 189 units — who=1's army off, `1/112` and
    // `1/182` unlinked, the human's `0/4` and `0/11` extra in ours; who=1's
    // buildings and the cities `1/2018` and `1/2047` part in the closing
    // fields, the slot `1/2047` holding a different city each side
    // (parked 3700b).
    // Item 3707, the pair whole: **191 units, none off, torn, unlinked or
    // extra**, and no closing field parts.
    pin_eq!(e.compared, 191, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
