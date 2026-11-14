//! **The Indians at a held Enlightenment age, East Indies** (DECISIONS 90,
//! 91, 93, 94, item 3649): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Indians (21)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Enlightenment:
//! `STARTING_TECHNOLOGY 4`, `ENDING_TECHNOLOGY 4` —
//! `iroquois_enlightenment_pair`'s lobby with the nation moved. Its triple,
//! `(18, 21, 5)`, is new: the battery fields no Indians (both its Indian
//! games left it for AI words: `indian_word`'s `(18, 21, 2)` and
//! `indian_sahara_word`'s `(7, 21, 3)`). Captures: run1311 (a `DUMP_ALL`
//! start) and run1312 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §340.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const INDIAN_ENLIGHTENMENT: Pair = Pair {
    name: "EastIndiesIndianEnlightenment",
    map_style: 18,
    ai_tribe: 21,
    difficulty: 5,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1311-eastindies-indian-enlightenment-start.txt",
    long: (
        "gamelog-run1312-eastindies-indian-enlightenment-trace.txt",
        "rontrace-run1312.log",
    ),
    // The game ended itself on 5000 (closing block 5001).
    length: 5000,
    // Item 3649, first try: frame 4157, by the count and the sequence —
    // ours 7 draws against 11, parting at index 0, ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` (`docs/AI.md` §340).
    // Item 3660, piece 294's bays (the Man o' War's broadsides): 4157 →
    // 4911 — ours 8 draws against 9, parting at index 6, ours
    // `Farms::inc_time+0x1ae`, theirs `Unit::close+0xcb6` (§340.1).
    // Piece 1809's bays (the Indians' Musketeers): 4911 → 5000, the
    // game's last frame — the pair whole.
    count: 5000,
    sequence: 5000,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const INDIAN_ENLIGHTENMENT_PAIR_WORD: i64 =
    if INDIAN_ENLIGHTENMENT.count < INDIAN_ENLIGHTENMENT.sequence {
        INDIAN_ENLIGHTENMENT.count
    } else {
        INDIAN_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn indian_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &INDIAN_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "INDIAN_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 4, and the long is scoreable.
#[test]
fn indian_enlightenment_pair_captures_say_their_lobby() {
    let p = &INDIAN_ENLIGHTENMENT;
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
        "the Indian Enlightenment pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3649; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no
/// Indians; and no other pinned Indian lobby's: `indian_word`'s
/// `(18, 21, 2)`, the same map at another difficulty, and
/// `indian_sahara_word`'s `(7, 21, 3)` (item 3649).
#[test]
fn indian_enlightenment_pair_is_no_other_lobby() {
    let p = &INDIAN_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the Indians"
    );
    let word = &super::indian_word::INDIAN;
    let sahara = &super::indian_sahara_word::INDIAN_SAHARA;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            sahara.name,
            (sahara.map_style, sahara.ai_tribe, sahara.difficulty),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Indians");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 4157** (item 3649):
/// only run1312's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_INDIAN_ENLIGHTENMENT_STRIDE: (i64, i64) = (101, 4101);

pub(crate) fn indian_enlightenment_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[INDIAN_ENLIGHTENMENT.start],
        true,
        INDIAN_ENLIGHTENMENT.long,
        "run1312 stride",
        &[(
            INDIAN_ENLIGHTENMENT.long.0,
            WIDENING_INDIAN_ENLIGHTENMENT_STRIDE.0,
        )],
        WIDENING_INDIAN_ENLIGHTENMENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn indian_enlightenment_pair_the_long_s_stride_walks_back_from_4157() {
    let _pins = Pins::hold();
    let Some(w) = indian_enlightenment_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 41, "every stride block in the window");
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
    // Item 3649: 139 keys. The standing families: 49 buildings'
    // `build:extra` (which the stride does not print), `form` on 46 units,
    // and who=1's 19 groups' `held` and pools (64..82) as each is minted.
    // **The first unit row is on block 4101, the human's `0/13`**:
    // `damage_frame` ours 4091 theirs 4092, `pos` (5214, 7428) against
    // (5192, 7416), `heading` and the guy's `x`, `y` and `angle` — the
    // state parts in the undumped gap 4002..4100 (`docs/AI.md` §340).
    // Item 3660, piece 294's bays: 133, the standing families alone —
    // `0/13`'s rows leave with the Man o' War's round (§340.1).
    pin_eq!(w.firsts.len(), 133, "the stride's baseline");
}

/// run1312b: the word 4157's window, blocks 4151..4407 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 4158 and 250 after (item 3649); the stride's blocks are 4101 and
/// 4201.
pub(crate) const RUN1312B: &str =
    "gamelog-run1312b-eastindies-indian-enlightenment-window-4151-4407.txt";

/// The window: block 4151 through 4407; the word 4157's own block is 4158.
pub(crate) const WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4157: (i64, i64) = (4151, 4407);

/// **The word 4157's window**: run1312b walked from run1311's start with
/// the recorder on and every round compared (item 3649).
pub(crate) fn indian_enlightenment_frame_4157_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INDIAN_ENLIGHTENMENT.start],
        true,
        INDIAN_ENLIGHTENMENT.long,
        "run1312b",
        &[(RUN1312B, WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4157.0)],
        WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4157,
        1,
        &[4158],
        true,
    )
}

#[test]
fn indian_enlightenment_pair_run1312b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_enlightenment_frame_4157_window() else {
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
    // Item 3649, the first walk: 1549 keys. Block 4151 carries the
    // standing families (`SITE[i].reg`, `form`, who=0's `aggression[1]`,
    // who=1's groups' `held` and pools) and **the human's Citizen `0/13`**
    // (TypeIndex 50), struck in the undumped gap 4002..4100 a frame apart
    // — `damage_frame` ours 4091 theirs 4092 — and fleeing to (6504,
    // 7752) a frame ahead: `pos` (6372, 7730) against (6348, 7726). On
    // the word 4157 ours' `0/13` has arrived — `orders.len` and the path
    // ours 0 theirs 1 — and on 4158 it is idle (`idle` 1 against 0): the
    // draw on 4157, ours 7 against 11, index 0, ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` (`docs/AI.md` §340).
    // Item 3660, piece 294's bays: **109**, the standing families alone;
    // `0/13` agrees on every block (§340.1).
    pin_eq!(w.firsts.len(), 109, "run1312b baseline");
    // Every round, both directions: 711 round-blocks theirs, 631 ours,
    // 72 parting. The first to part is the Bomb Ketch `1/53`'s (TypeIndex
    // 347, sea, stance Raid) on 4165, its hull agreeing to 4312: it leaves
    // (3410, 10633, 133) in ours against (3241, 10766, 130) in theirs, for
    // (6105, 7255) against (5992, 7345). Every parting round is a Ketch's
    // (`1/53`, `1/54`, `1/65`, `1/66`, `1/83`) but the Man o' War `1/97`'s
    // (TypeIndex 344) from 4383. Item 3660: 711 a side, none parting —
    // the Ketches' anim rolls were downstream of the word's draw, and the
    // Man o' War's rounds leave piece 294's bays.
    pin_eq!(w.rounds_read, (711, 711), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1324: the gap window over the undumped 4002..4100, blocks 4001..4150
/// at run1312b's detail — from the stride's last dumped block 4001 to the
/// block before run1312b's first (item 3660).
pub(crate) const RUN1324: &str =
    "gamelog-run1324-eastindies-indian-enlightenment-window-4001-4150.txt";

/// The gap window: block 4001 through 4150.
pub(crate) const WIDENING_INDIAN_ENLIGHTENMENT_GAP: (i64, i64) = (4001, 4150);

/// **The gap 4002..4100**: run1324 walked from run1311's start with every
/// round compared (item 3660).
pub(crate) fn indian_enlightenment_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INDIAN_ENLIGHTENMENT.start],
        true,
        INDIAN_ENLIGHTENMENT.long,
        "run1324",
        &[(RUN1324, WIDENING_INDIAN_ENLIGHTENMENT_GAP.0)],
        WIDENING_INDIAN_ENLIGHTENMENT_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn indian_enlightenment_pair_run1324_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_enlightenment_gap_window() else {
        return;
    };
    // Blocks 4001..4149: the capture's last block, 4150, is its closing
    // one and is not walked.
    pin_eq!(w.blocks, 149, "every captured block");
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3660, before piece 294's bays: 119 keys — the standing
    // families and the Citizen `0/13` (TypeIndex 50), struck on 4091 in
    // ours alone by block 4092 (`damage_frame` 4091 against 0,
    // `hits_left` 50 against 65) and on a `FLEEORDER` to (6504, 7752) a
    // frame ahead; rounds 197 theirs against 196 ours, 7 parting, all the
    // Man o' War `1/97`'s (TypeIndex 344, piece 294): its round on 4082
    // left (3000, 8472, 0) in ours, flying 11 frames, against (2866,
    // 8510, −33) in theirs, flying 12. After: **99**, all standing.
    pin_eq!(w.firsts.len(), 99, "run1324 baseline");
    pin_eq!(w.rounds_read, (197, 197), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1324b: the word 4911's window, blocks 4905..5001 at run1312b's
/// detail — six blocks before the word's block 4912, to the game's
/// closing block (item 3660); since piece 1809's bays the pair is whole
/// and this is its closing window.
pub(crate) const RUN1324B: &str =
    "gamelog-run1324b-eastindies-indian-enlightenment-window-4905-5001.txt";

/// The window: block 4905 through 5001; the word 4911's own block is 4912.
pub(crate) const WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4911: (i64, i64) = (4905, 5001);

/// **The word 4911's window**: run1324b walked from run1311's start with
/// the recorder on and every round compared (item 3660).
pub(crate) fn indian_enlightenment_frame_4911_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[INDIAN_ENLIGHTENMENT.start],
        true,
        INDIAN_ENLIGHTENMENT.long,
        "run1324b",
        &[(RUN1324B, WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4911.0)],
        WIDENING_INDIAN_ENLIGHTENMENT_FRAME_4911,
        1,
        &[4912],
        true,
    )
}

#[test]
fn indian_enlightenment_pair_run1324b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = indian_enlightenment_frame_4911_window() else {
        return;
    };
    pin_eq!(w.blocks, 96, "every captured block");
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
    // Item 3660, before piece 1809's bays: 786 keys — the Citizen `0/10`
    // struck a frame apart in the gap before the window (`damage_frame`
    // ours 4900 theirs 4899 on 4905, `hits_left` 24 against 10 on 4908),
    // closed on 4911 in theirs alone (who=0's `num_units[0]` 12 against
    // 11 on 4912: the word's `Unit::close` draw); rounds 321 theirs
    // against 320 ours with 87 parting, first the Musketeers `1/80` and
    // `1/81`'s (TypeIndex 99, piece 1809) — `1/80`'s on 4905 from (11256,
    // 7608, 315) in 5 frames against (11095, 7683, 489) in 4. After:
    // **133**, the standing families and three rows: the Musketeer
    // `1/56`'s target on 4955 (`0/5` against `0/12`), and who=1's timber
    // and the building `1/2005`'s `queue[2].cost[0]` on 4983 (250 against
    // 125) — parked 3660a and 3660b; rounds 320 a side, none parting.
    pin_eq!(w.firsts.len(), 133, "run1324b baseline");
    pin_eq!(w.rounds_read, (320, 320), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The closing state** (item 3660): run1312's closing block 5001, the
/// whole map's units, walked from run1311's start.
#[test]
fn indian_enlightenment_pair_run1312_s_closing_state() {
    let _pins = Pins::hold();
    let p = &INDIAN_ENLIGHTENMENT;
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
        "Indian Enlightenment pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    for &(who, o) in &e.extra {
        if let Some(u) = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
        {
            let name = u.ty.and_then(|ty| loaded.unit_names.get(ty).cloned());
            eprintln!("    extra {who}/{o} {name:?} at {:?}", u.pos);
        }
    }
    for b in &fin.builds {
        if (0..2).contains(&b.who) && harness::link_building(&built.sim, b.who, b.o).is_none() {
            eprintln!(
                "    unlinked building {}/{} at {:?} type {:?}",
                b.who, b.o, b.pos, b.orig_type
            );
            for x in built
                .sim
                .buildings
                .iter()
                .filter(|x| i64::from(x.pos.x) == b.pos.x && i64::from(x.pos.y) == b.pos.y)
            {
                eprintln!(
                    "      ours {}/{} alive {} hits {} health {} ty {:?}",
                    x.owner, x.index, x.alive, x.hits, x.health, x.ty
                );
            }
        }
    }
    pin_eq!(e.frame, 5001, "the closing block");
    // Item 3660, the pair whole: 176 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 5000.
    pin_eq!(e.compared, 176, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
