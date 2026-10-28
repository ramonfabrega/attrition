//! **The Spanish at a held Modern age, East Indies** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3718): East Indies, `MAP_STYLE 18`, human
//! Nubians (4) against the **Spanish (9)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Modern:
//! `STARTING_TECHNOLOGY 6`, `ENDING_TECHNOLOGY 6`. Its triple,
//! `(18, 9, 4)`, is new: the Spanish hold this map at 3
//! (`spanish_east_word`) and 5 (`spanish_gunpowder_pair`), Great Lakes at 4
//! (`spanish_lakes_word`) and 5 (`spanish_industrial_pair`), and Great
//! Sahara at 3 (`spanish_word`) and 5 (`spanish_classical_pair`); the
//! battery fields no Spanish. Captures: run1394 (a `DUMP_ALL` start) and
//! run1395 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §365.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const SPANISH_MODERN: Pair = Pair {
    name: "EastIndiesSpanishModern",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 4,
    starting_technology: 6,
    ending_technology: 6,
    starting_resources: 7,
    start: "gamelog-run1394-eastindies-spanish-modern-start.txt",
    long: (
        "gamelog-run1395-eastindies-spanish-modern-trace.txt",
        "rontrace-run1395.log",
    ),
    // The game ended itself on 4081 (closing block 4082).
    length: 4081,
    // Item 3718, first try: 3770 of 4081 by the count and the sequence —
    // ours 3212 draws against 3216, index 0 ours' `Guy::set_anim+0x97a <
    // Guy::do_turn+0x4a < Unit::move_step+0x389` against theirs'
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d` (`docs/AI.md` §365).
    // Item 3736: **3770 → 4021**, by the count and the sequence — piece
    // 299's pivot and release rows (`sim::pivot`, off run1165's packet) put
    // the Battleships' rounds at their turrets, so the Citizen `0/13` is
    // struck on 3704 in ours too. On 4021 ours spends 17 draws against 18,
    // index 12: theirs' `Unit::close+0xcb6 < Object::die+0x29 <
    // Unit::die+0x19`, a death ours does not deal, against ours'
    // `Farms::inc_time+0x1ae` (`docs/AI.md` §365.1).
    count: 4021,
    sequence: 4021,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_MODERN_PAIR_WORD: i64 = if SPANISH_MODERN.count < SPANISH_MODERN.sequence {
    SPANISH_MODERN.count
} else {
    SPANISH_MODERN.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_modern_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_MODERN;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_MODERN {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 6 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn spanish_modern_pair_captures_say_their_lobby() {
    let p = &SPANISH_MODERN;
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
        "the Spanish Modern pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3718; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no Spanish — and no other Spanish lobby's: `spanish_east_word`'s
/// `(18, 9, 3)` and `spanish_gunpowder_pair`'s `(18, 9, 5)`, this map at
/// other difficulties, `spanish_lakes_word`'s `(14, 9, 4)`,
/// `spanish_industrial_pair`'s `(14, 9, 5)`, `spanish_word`'s `(7, 9, 3)`
/// and `spanish_classical_pair`'s `(7, 9, 5)`, other maps (item 3718).
#[test]
fn spanish_modern_pair_is_no_other_lobby() {
    let p = &SPANISH_MODERN;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the Spanish"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Spanish lobby on this map at this difficulty"
        );
    }
    let w = &super::spanish_word::SPANISH;
    let k = &super::spanish_lakes_word::SPANISH_LAKES;
    let e = &super::spanish_east_word::SPANISH_EAST;
    let c = &super::spanish_classical_pair::SPANISH_CLASSICAL;
    let g = &super::spanish_gunpowder_pair::SPANISH_GUNPOWDER;
    let i = &super::spanish_industrial_pair::SPANISH_INDUSTRIAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (k.name, (k.map_style, k.ai_tribe, k.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Spanish");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3718, from
/// 3770): only run1395's stride blocks are on disk before the word's
/// window, 101 through 3701.
pub(crate) const WIDENING_SPANISH_MODERN_STRIDE: (i64, i64) = (101, 3701);

pub(crate) fn spanish_modern_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_MODERN.start],
        true,
        SPANISH_MODERN.long,
        "run1395 stride",
        &[(SPANISH_MODERN.long.0, WIDENING_SPANISH_MODERN_STRIDE.0)],
        WIDENING_SPANISH_MODERN_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_modern_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = spanish_modern_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 37, "every stride block in the window");
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
    // Item 3718: 163 keys over 37 blocks. The standing families — `form`
    // on every unit, the buildings' `build:extra` (which the stride does
    // not print) and who=1's groups with their pools — and, from block
    // 3201, the Submarine `1/109`'s (TI 338) `pos`, heading and move: its
    // birth on 3119 lands 480 north of theirs' (run1395c).
    pin_eq!(w.firsts.len(), 163, "the stride's baseline");
}

/// run1395b: the word 3770's window, blocks 3677..4021 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — 24 blocks before the stride's
/// last block 3701 and 250 past the word's own block 3771 (item 3718).
/// run1395 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1395B: &str = "gamelog-run1395b-eastindies-spanish-modern-window-3677-4021.txt";

/// The window: block 3677 through 4021.
pub(crate) const WIDENING_SPANISH_MODERN_FRAME_3770: (i64, i64) = (3677, 4021);

/// **The word 3770's window**: run1395b walked from run1394's start with
/// the recorder on and every round compared (item 3718).
pub(crate) fn spanish_modern_frame_3770_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[SPANISH_MODERN.start],
        true,
        SPANISH_MODERN.long,
        "run1395b",
        &[(RUN1395B, WIDENING_SPANISH_MODERN_FRAME_3770.0)],
        WIDENING_SPANISH_MODERN_FRAME_3770,
        1,
        &[3771],
        true,
    )
}

#[test]
fn spanish_modern_pair_run1395b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_modern_frame_3770_window() else {
        return;
    };
    pin_eq!(w.blocks, 344, "every captured block");
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
    // Item 3718, the first walk: **1983** keys. Block 3677 carries the
    // standing families (both leaders' `SITE[i].reg`, `aggression[1]`,
    // `form`, groups and pools) and the Submarine `1/109`'s position, parted
    // since its birth on 3119 (run1395c). **The word**: on 3704 theirs'
    // round wounds the Citizen `0/13` (TI 50, hits 85 → 79) and ours' does
    // not; theirs' citizen flees on a `FLEETOORDER` to (6888, 7752), arrives
    // on 3770, drops the order and goes idle with one `Guy::set_anim+0x97a
    // < Unit::set_anim+0x56 < Unit::do_idle+0x7d` draw ours never spends.
    // The rounds part first on 3693: the Battleships `1/45` and `1/46` (TI
    // 349, piece 299, which neither `launch` nor `pivot` holds) loose from
    // the guy's own square in ours, `sz` 0 against theirs' 105–106, and fly
    // 14 frames against 13.
    //
    // Item 3736: 1983 → **145** with piece 299's rows. Block 3677's
    // standing families stand (101); after them the Submarine `1/109`'s
    // order and facing, who=1's new units' `form`, group 76, and the
    // Citizen `0/13`: `hits_left` 62 against 51 on 3933, the block the Tank
    // `1/25`'s round of 3928 lands on (it parts: `sz` 66 against 210, 7
    // frames against 6), then on 4018 theirs' Bazooka
    // `1/31` strikes it (`damage_frame` 4017) and it flees, which ours
    // does not (`orders.len` 0 against 1).
    pin_eq!(w.firsts.len(), 145, "run1395b baseline");
    // Every round, both directions: 654 in ours against 680, the first
    // parting the Battleship `1/45`'s on 3693. Item 3736: (654, 656), 28
    // parting — the Tanks `1/25` and `1/35` (TI 250, `PANZER`, a pivot
    // piece no row holds), the Bazookas `1/32` and `1/33` (TI 140), and
    // the Battleship `1/71`'s round of 4013, which leaves the hull's pose
    // after its move step in ours and before it in theirs (parked 3736a).
    pin_eq!(w.rounds_read, (654, 656), "the rounds read");
    pin_eq!(w.rounds.len(), 28, "the rounds that part");
}

/// **run1415, the word 4021's window**: blocks 4015 through the closing
/// block 4082 at run1395b's detail (`LEADERS=9`, `AMMO=5`), six blocks
/// before the word and through the game's own end (item 3736). run1395b
/// ends on block 4021, so the word's own block 4022 was on no disk.
pub(crate) const RUN1415: &str = "gamelog-run1415-eastindies-spanish-modern-window-4015-4082.txt";

/// The window: block 4015 through 4082.
pub(crate) const WIDENING_SPANISH_MODERN_FRAME_4021: (i64, i64) = (4015, 4082);

/// **The word 4021's window**: run1415 walked from run1394's start with
/// the recorder on and every round compared (item 3736).
pub(crate) fn spanish_modern_frame_4021_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[SPANISH_MODERN.start],
        true,
        SPANISH_MODERN.long,
        "run1415",
        &[(RUN1415, WIDENING_SPANISH_MODERN_FRAME_4021.0)],
        WIDENING_SPANISH_MODERN_FRAME_4021,
        1,
        &[4022],
        true,
    )
}

#[test]
fn spanish_modern_pair_run1415_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_modern_frame_4021_window() else {
        return;
    };
    pin_eq!(w.blocks, 67, "every captured block");
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
    // Item 3736: **389** keys. Block 4015 carries 108 standing rows. **The
    // word**: theirs' Citizen `0/13` reaches 4018 with 10 hits against
    // ours' 23 (`damage_frame` 4017, `damage_o` 31 against 3979, 97) and
    // flees; on 4021 a round explodes and it dies (`Unit::close+0xcb6 <
    // Object::die+0x29`), so block 4022 holds it in ours alone and who=0's
    // `peasants` read 8 against 7. Past the word the game parts widely.
    pin_eq!(w.firsts.len(), 389, "run1415 baseline");
    pin_eq!(w.rounds_read, (394, 400), "the rounds read");
    // Every round, both directions: 394 in ours against 400, 99 parting.
    // The Tanks `1/25` and `1/35` (TI 250, `PANZER`, `<RESTRICTION
    // name="PANZER" node="4">`, no `pivot` row) loose from the hull, `sz`
    // 168 against 312 for `1/35`'s round of 4017, and fly 7 frames against
    // 6: theirs' has landed by 4022 and ours' is still in the air.
    pin_eq!(w.rounds.len(), 99, "the rounds that part");
}

/// run1395c: the walk back from the stride's first unit row, blocks
/// 3095..3210 at the word window's detail — six blocks before the stride's
/// block 3101, which agrees, through its block 3201, where `1/109`'s `pos`
/// first parts (item 3718).
pub(crate) const RUN1395C: &str = "gamelog-run1395c-eastindies-spanish-modern-window-3095-3210.txt";

/// The window: block 3095 through 3210.
pub(crate) const WIDENING_SPANISH_MODERN_WALK_BACK: (i64, i64) = (3095, 3210);

/// **The state's first parting, walked back** from the stride: run1395c
/// walked from run1394's start with every round compared (item 3718).
pub(crate) fn spanish_modern_walk_back_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[SPANISH_MODERN.start],
        true,
        SPANISH_MODERN.long,
        "run1395c",
        &[(RUN1395C, WIDENING_SPANISH_MODERN_WALK_BACK.0)],
        WIDENING_SPANISH_MODERN_WALK_BACK,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_modern_pair_run1395c_walks_the_state_back() {
    let _pins = Pins::hold();
    let Some(w) = spanish_modern_walk_back_window() else {
        return;
    };
    pin_eq!(w.blocks, 115, "every captured block");
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
    // Item 3718: 108 keys. The state's first parting is a birth: the
    // Submarine `1/109` (TI 338, TY 288, sea), trained at who=1's Dock
    // `1/2016` (35904, 43776), is placed on 3119 at (36696, 44760) in ours
    // against (36696, 45240) in theirs; its move and path follow (3120,
    // 3130). It spends no draw, so the stream holds to 3770.
    pin_eq!(w.firsts.len(), 108, "run1395c baseline");
    // Every round, both directions: 21 against 21, none parting.
    pin_eq!(w.rounds_read, (21, 21), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The pair's closing state** (item 3718): run1395's closing block 4082,
/// the whole map's units, walked from run1394's start past the word 3770.
#[test]
fn spanish_modern_pair_run1395_s_closing_state() {
    let _pins = Pins::hold();
    let p = &SPANISH_MODERN;
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
        "Spanish Modern pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4082, "the closing block");
    // Item 3718, first try: 311 frames past the word 3770 the game has
    // parted widely — 151 units compared, counts [56, 2, 2, 2, 24, 1, 10],
    // who=1's `1/142` and `1/147` extra in ours, `1/127` and `1/141`
    // unlinked, and the cities' and buildings' records apart. A floor, not
    // a score.
    //
    // Item 3736, the word at 4021: 153 units, 13 off and nothing else —
    // no torn, unlinked or extra unit; `Closing fields: none`.
    pin_eq!(e.compared, 153, "units compared");
    pin_eq!(e.counts(), [13, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
