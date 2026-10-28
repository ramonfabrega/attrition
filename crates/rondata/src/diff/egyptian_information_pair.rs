//! **The Egyptians at a held Information age, Great Lakes** (DECISIONS
//! 90, 91, 93, 94, 97, 98, 99, item 3720): Great Lakes, `MAP_STYLE 14`,
//! human Nubians (4) against the **Egyptians (7)** at difficulty 5, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7` — the first pair to hold
//! age 7. Its triple, `(14, 7, 5)`, is new: the Egyptians hold East Indies
//! at 3 (the held-out `EastIndiesEgyptianTier3`), 4
//! (`egyptian_industrial_pair`) and 5 (`egyptian_classical_pair`), and
//! Great Sahara at 4 (`egyptian_word`, `GreatSaharaEgyptianTier4`) and 5
//! (`egyptian_gunpowder_pair`); none holds Great Lakes. Captures: run1398
//! (a `DUMP_ALL` start) and run1399 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §367.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const EGYPTIAN_INFORMATION: Pair = Pair {
    name: "GreatLakesEgyptianInformation",
    map_style: 14,
    ai_tribe: 7,
    difficulty: 5,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1398-greatlakes-egyptian-information-start.txt",
    long: (
        "gamelog-run1399-greatlakes-egyptian-information-trace.txt",
        "rontrace-run1399.log",
    ),
    // The game ended itself on 3797 (closing block 3798).
    length: 3797,
    // Item 3720, first try: count 3244, sequence 3244 of 3797 — on 3244
    // ours 11 draws against theirs' 10, index 1 ours' `Unit::fight+0x824`
    // (the T80DES `1/42`, TI 257) against theirs' `Unit::do_guard+0x8fb`
    // (`docs/AI.md` §367). Item 3740: 3244 → 3519, piece 54's bays
    // (`sim::launch::BAYS`): `1/34`'s round of 3204 leaves the muzzle and
    // lands on 3205 on both sides; on 3519 ours 13 draws against theirs'
    // 14, index 13 theirs' `Unit::close+0xcb6` (`docs/AI.md` §367.1).
    // Item 3743: 3519 → **3797, whole**: piece 203's bays
    // (`sim::launch::BAYS`) put the Main Battle Tank `1/55`'s round of 3514
    // at (5166, 30496, 553), so the Citizen `0/10` dies on 3519 on both
    // sides, and piece 207's pivot rows (`sim::pivot::RELEASES`) put the
    // T80DES rounds at the muzzle (`docs/AI.md` §367.2).
    count: 3797,
    sequence: 3797,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const EGYPTIAN_INFORMATION_PAIR_WORD: i64 =
    if EGYPTIAN_INFORMATION.count < EGYPTIAN_INFORMATION.sequence {
        EGYPTIAN_INFORMATION.count
    } else {
        EGYPTIAN_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn egyptian_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "EGYPTIAN_INFORMATION {}: count {} sequence {} of {}; {}",
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
fn egyptian_information_pair_captures_say_their_lobby() {
    let p = &EGYPTIAN_INFORMATION;
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
        "the Egyptian Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3720; the battery's identity
/// test refuses it), once, and none of the battery's — whose one Egyptian
/// game is the held-out `EastIndiesEgyptianTier3`, `(18, 7, 3)`, another
/// map — and no other Egyptian lobby's: `egyptian_classical_pair`'s
/// `(18, 7, 5)`, `egyptian_industrial_pair`'s `(18, 7, 4)`,
/// `egyptian_gunpowder_pair`'s `(7, 7, 5)` and `egyptian_word`'s
/// `(7, 7, 4)`, all other maps: no Egyptian lobby holds Great Lakes (item
/// 3720).
#[test]
fn egyptian_information_pair_is_no_other_lobby() {
    let p = &EGYPTIAN_INFORMATION;
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
            "{} fields the Egyptians here",
            l.name
        );
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style,
            "{l:?} is another Egyptian lobby on this map"
        );
    }
    let c = &super::egyptian_classical_pair::EGYPTIAN_CLASSICAL;
    let i = &super::egyptian_industrial_pair::EGYPTIAN_INDUSTRIAL;
    let g = &super::egyptian_gunpowder_pair::EGYPTIAN_GUNPOWDER;
    let w = &super::egyptian_word::EGYPTIAN;
    for (name, l) in [
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (i.name, (i.map_style, i.ai_tribe, i.difficulty)),
        (g.name, (g.map_style, g.ai_tribe, g.difficulty)),
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Egyptians");
        assert_ne!(l, triple, "{name} is this lobby");
        assert_ne!(l.0, p.map_style, "{name} holds Great Lakes");
    }
}

/// **The long's stride, walked back from the word** (item 3720, from
/// 3244): only run1399's stride blocks are on disk before the word, 101
/// through 3201.
pub(crate) const WIDENING_EGYPTIAN_INFORMATION_STRIDE: (i64, i64) = (101, 3201);

pub(crate) fn egyptian_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[EGYPTIAN_INFORMATION.start],
        true,
        EGYPTIAN_INFORMATION.long,
        "run1399 stride",
        &[(
            EGYPTIAN_INFORMATION.long.0,
            WIDENING_EGYPTIAN_INFORMATION_STRIDE.0,
        )],
        WIDENING_EGYPTIAN_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn egyptian_information_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 32, "every stride block in the window");
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
    // Item 3720: 92 keys over 32 blocks, the standing families alone —
    // `build:extra` on 53 buildings (which the stride does not print),
    // `form` on 27 units, and who=1's six groups' `held` with their pools.
    // No position, order or hit parts before the word.
    pin_eq!(w.firsts.len(), 92, "the stride's baseline");
}

/// run1399b: the word 3244's window, blocks 3195..3494 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3201 and 250 past the word's own block 3245 (item 3720).
/// run1399 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1399B: &str =
    "gamelog-run1399b-greatlakes-egyptian-information-window-3195-3494.txt";

/// The window: block 3195 through 3494.
pub(crate) const WIDENING_EGYPTIAN_INFORMATION_FRAME_3244: (i64, i64) = (3195, 3494);

/// **The word 3244's window**: run1399b walked from run1398's start with
/// the recorder on and every round compared (item 3720).
pub(crate) fn egyptian_information_frame_3244_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_INFORMATION.start],
        true,
        EGYPTIAN_INFORMATION.long,
        "run1399b",
        &[(RUN1399B, WIDENING_EGYPTIAN_INFORMATION_FRAME_3244.0)],
        WIDENING_EGYPTIAN_INFORMATION_FRAME_3244,
        1,
        &[3245],
        true,
    )
}

#[test]
fn egyptian_information_pair_run1399b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_information_frame_3244_window() else {
        return;
    };
    pin_eq!(w.blocks, 299, "every captured block");
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
    // Item 3720, the first walk: **2625** keys. Block 3195 carries the
    // standing families (who=0's `SITE[i].reg` 1 against 0 and
    // `aggression[1]` 0 against 1, `form`, who=1's groups and pools). **The
    // state parts on 3206, before the word**: on 3204 the Mechanized
    // Infantry `1/34` (TI 104, `gpiece 54`, which neither `launch` nor
    // `pivot` holds) looses at the Citizen `0/13` (TI 50); theirs' round
    // leaves from (5530, 31784, z 533), about 111 out toward the target,
    // flies 2 frames and wounds `0/13` on 3205 (hits 85 → 67, damage 18),
    // which flees to (2808, 31416); ours' leaves from the guy's own square,
    // (5640, 31800, z 364), and flies 3. The draws hold to 3244, where ours'
    // T80DES `1/42` (TI 257) spends a `Unit::fight+0x824` draw theirs does
    // not. Item 3740 (piece 54's bays): 107. Item 3743 (piece 203's bays,
    // piece 207's pivot rows): **65** — the T80DES `1/42`'s round of 3214
    // leaves its muzzle on both sides; `1/29`'s of 3216 still parts, a
    // node-1 round ours fires from `WALK` (parked 3743a).
    pin_eq!(w.firsts.len(), 65, "run1399b baseline");
    // Every round, both directions: 648 in ours against 684, the first
    // parting `1/34`'s `ammo[0]` on 3205 (`sx` 5640 against 5530, `sz` 364
    // against 533, `total_time` 3 against 2).
    pin_eq!(w.rounds_read, (648, 654), "the rounds read");
    pin_eq!(w.rounds.len(), 95, "the rounds that part");
}

/// run1421: the word 3519's window, blocks 3495..3770 at run1399b's
/// detail — from run1399b's last block 3494 (six blocks before the
/// stride's last block 3501) to 250 past the word's own block 3520 (item
/// 3743). run1399 prints units only every hundred frames.
pub(crate) const RUN1421: &str =
    "gamelog-run1421-greatlakes-egyptian-information-window-3495-3770.txt";

/// The window: block 3495 through 3770.
pub(crate) const WIDENING_EGYPTIAN_INFORMATION_FRAME_3519: (i64, i64) = (3495, 3770);

/// **The word 3519's window**: run1421 walked from run1398's start with
/// the recorder on and every round compared (item 3743).
pub(crate) fn egyptian_information_frame_3519_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_INFORMATION.start],
        true,
        EGYPTIAN_INFORMATION.long,
        "run1421",
        &[(RUN1421, WIDENING_EGYPTIAN_INFORMATION_FRAME_3519.0)],
        WIDENING_EGYPTIAN_INFORMATION_FRAME_3519,
        1,
        &[3520],
        true,
    )
}

#[test]
fn egyptian_information_pair_run1421_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_information_frame_3519_window() else {
        return;
    };
    pin_eq!(w.blocks, 275, "every captured block");
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
    // Item 3743, the first walk: **436** keys; the word's block 3520 held
    // the Citizen `0/10` (TI 50) in ours alone, who=0's `active`, `control`
    // (9 against 8), `num_units[0]` and `peasants` (8 against 7) parting
    // with it: the Main Battle Tank `1/55` (TI 253, piece 203)'s `ammo[1]`
    // of 3514 left (5352, 30792, z 515), its own square, in ours against
    // (5166, 30496, z 553) and flew 7 frames against 6, so theirs killed on
    // 3519 and ours on 3520. With piece 203's bays and piece 207's pivot
    // rows: **62** keys — block 3495's 53 standing rows (who=0's
    // `aggression[1]`, `form`, who=1's groups and pools, `mirror` on the
    // craft), group 67's `held` on 3507, `1/124`'s `mirror` on 3511, and
    // the same families later; `0/10` dies on 3519 on both sides and the
    // leader rows agree (`docs/AI.md` §367.2).
    pin_eq!(w.firsts.len(), 62, "run1421 baseline");
    // Every round, both directions: 655 against 697 and 367 parting before;
    // now 655 against 662 and 74, none of them `1/55`'s — the T80DES
    // node-1 rounds (parked 3743a).
    pin_eq!(w.rounds_read, (655, 662), "the rounds read");
    pin_eq!(w.rounds.len(), 74, "the rounds that part");
}

/// run1421b: the pair's closing window, blocks 3765..3798 at run1421's
/// detail (item 3743). The word reached the game's last frame; run1421
/// ends on 3770 and run1399's stride prints units only.
pub(crate) const RUN1421B: &str =
    "gamelog-run1421b-greatlakes-egyptian-information-closing-window-3765-3798.txt";

/// The window: block 3765 through the closing block 3798.
pub(crate) const WIDENING_EGYPTIAN_INFORMATION_CLOSING: (i64, i64) = (3765, 3798);

/// **The pair's last blocks**: run1421b walked from run1398's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3743).
pub(crate) fn egyptian_information_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[EGYPTIAN_INFORMATION.start],
        true,
        EGYPTIAN_INFORMATION.long,
        "run1421b",
        &[(RUN1421B, WIDENING_EGYPTIAN_INFORMATION_CLOSING.0)],
        WIDENING_EGYPTIAN_INFORMATION_CLOSING,
        1,
        &[3798],
        true,
    )
}

#[test]
fn egyptian_information_pair_run1421b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = egyptian_information_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every captured block");
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
    // Item 3743, the pair whole: **54** keys. Block 3765 carries 52
    // standing rows — `form` on 20 units, who=0's `SITE[i].reg` (12) and
    // `aggression[1]`, who=1's groups 64, 65, 66 and 70 with their pools,
    // and `mirror` on eleven of who=1's craft (one family) — and the two
    // craft minted later add their `mirror` (`1/128` on 3767, `1/129` on
    // 3789). No other unit, building, city or leader field parts, no draw
    // parts to 3797, and the closing block 3798 agrees.
    pin_eq!(w.firsts.len(), 54, "run1421b baseline");
    // Every round, both directions: 109 in ours against 110, 26 parting,
    // the first the T80DES `1/42`'s `ammo[1]` on 3772 (`sx, sy, sz` 4392,
    // 32088, 603 against 4245, 32066, 733: piece 207's node 1, which ours
    // fires from `WALK`; parked 3743a).
    pin_eq!(w.rounds_read, (109, 110), "the rounds read");
    pin_eq!(w.rounds.len(), 26, "the rounds that part");
}

/// **The pair's closing state** (item 3720): run1399's closing block 3798,
/// the whole map's units, walked from run1398's start.
#[test]
fn egyptian_information_pair_run1399_s_closing_state() {
    let _pins = Pins::hold();
    let p = &EGYPTIAN_INFORMATION;
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
        "Egyptian Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3798, "the closing block");
    // Item 3720, first try: 553 frames past the word 3244 — 137 units
    // compared, counts [70, 0, 0, 7, 3, 1, 0], `1/1` and `1/28` torn, none
    // extra or unlinked. A floor, not a score.
    pin_eq!(e.compared, 137, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
