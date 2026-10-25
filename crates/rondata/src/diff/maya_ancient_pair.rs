//! **The Maya from an Ancient start, East Indies** (DECISIONS 90, 91, 93,
//! item 3587): East Indies, `MAP_STYLE 18`, human Nubians (4) against the
//! **Maya (1)** at Difficulty 5, seed 12345, `STARTING_RESOURCES 7`,
//! `STARTING_TECHNOLOGY 0` and `ENDING_TECHNOLOGY 1` (read back 1; age 0
//! cannot be held, 3559a) — the Bantu's and the Koreans' Ancient-start
//! lobby with the nation moved. The Maya's other scored lobby is
//! `maya_word`'s `GreatSaharaMayaTier4`, `(7, 1, 4)`: another map.
//! Captures: run1233 (a `DUMP_ALL` start) and run1234 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §320.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const MAYA_ANCIENT: Pair = Pair {
    name: "EastIndiesMayaAncient",
    map_style: 18,
    ai_tribe: 1,
    difficulty: 5,
    starting_technology: 0,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1233-eastindies-maya-ancient-start.txt",
    long: (
        "gamelog-run1234-eastindies-maya-ancient-trace.txt",
        "rontrace-run1234.log",
    ),
    // The game ended itself on 5935 (closing block 5936).
    length: 5935,
    // Item 3587, first try: frame 4182 by the count and the sequence, ours
    // 31 draws against 29, index 16 — ours `Leader::produce_building+0x1805`
    // (who=1 places a second Senate, `1/2042` at (38016, 41376)), theirs
    // `Leader::make_stuff+0x63d` (`docs/AI.md` section 320).
    //
    // Item 3597: 4182 → 4324. The Senate arm of `LeaderData::type_avail`
    // refuses the second Senate while the first's site is unfinished
    // (`num_queued[SENATE]`); the word is now frame 4324, ours 11 draws
    // against 12, index 1 — ours `Unit::fight+0x9b0`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` (section 320.1).
    //
    // Item 3599: 4324 → 4429. The Balamob Slingers' stones leave from
    // piece 38's bays (`sim::launch`); the word is now frame 4429, ours 10
    // draws against 12, index 1 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (section 320.2).
    //
    // Item 3600: 4429 → whole, 5935 of 5935. The Bowmen's arrows leave
    // from piece 824's bays (`sim::launch`): `1/46`'s at the Citizen
    // `0/10`, loosed on 4318, from (6481, 9189) in 20 frames where ours
    // loosed from the unit's square (6552, 9240) in 21, so `0/10`'s third
    // point lands on 4337 as theirs' does (section 320.3).
    count: 5935,
    sequence: 5935,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MAYA_ANCIENT_PAIR_WORD: i64 = if MAYA_ANCIENT.count < MAYA_ANCIENT.sequence {
    MAYA_ANCIENT.count
} else {
    MAYA_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn maya_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MAYA_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MAYA_ANCIENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age capped at 1, and the long is scoreable.
#[test]
fn maya_ancient_pair_captures_say_their_lobby() {
    let p = &MAYA_ANCIENT;
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
        "the Maya Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery holds no Maya game, and the Maya's
/// scored word is `(7, 1, 4)` — another map (item 3587).
#[test]
fn maya_ancient_pair_is_no_other_lobby() {
    let p = &MAYA_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(!battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe));
    let other = &super::maya_word::MAYA;
    assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
}

/// run1234b: the word 4182's window, blocks 4177..4433 at the long's
/// detail — six blocks before the word's block 4183 and 250 after (item
/// 3587); the start dumps blocks 0..2 alone and the long's stride prints
/// units every hundredth block and no `LEADERS` past the start.
pub(crate) const RUN1234B: &str = "gamelog-run1234b-eastindies-maya-ancient-window-4177-4433.txt";

/// The window: block 4177 through 4433; the word 4182's own block is 4183,
/// and item 3597's 4324's is 4325.
pub(crate) const WIDENING_MAYA_ANCIENT_FRAME_4182: (i64, i64) = (4177, 4433);

/// **The word 4182's window**: run1234b walked from run1233's start with
/// the recorder on (item 3587).
pub(crate) fn maya_ancient_frame_4182_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MAYA_ANCIENT.start],
        true,
        MAYA_ANCIENT.long,
        "run1234b",
        &[(RUN1234B, WIDENING_MAYA_ANCIENT_FRAME_4182.0)],
        WIDENING_MAYA_ANCIENT_FRAME_4182,
        1,
        &[4183],
        true,
    )
}

#[test]
fn maya_ancient_pair_run1234b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = maya_ancient_frame_4182_window() else {
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3587, the first walk: 2407 keys. Block 4177, the window's
    // first, carries 75 rows: the standing families (every unit's `form`,
    // who=0's `SITE[i].reg` ours 65 against theirs 0, both leaders'
    // `aggression`, who=1's groups' `held` and pools); group 73's `held`
    // and pool on 4179. On the word's block 4183 ours holds a second
    // Senate, `1/2042` at (38016, 41376), `constr_time` 41666, that theirs
    // does not (both hold `1/2041`, the Senate at (35904, 41376)); the
    // Citizen `1/94` builds in ours (`order:kind` 3, to (37560, 41256))
    // and gathers in theirs (7); who=1's `MAKE[8].t` is ours -1 against
    // theirs 438 (the Senate, still on theirs' list), `bucket` timber 15102 against 15139 and wealth 19210
    // against 19260, `gatherers` 27 against 28. Theirs' `1/2042`, on 4186,
    // is a wonder (`constr_time` 400000) at (37632, 41664).
    //
    // Item 3597, the Senate arm of `type_avail`: 2407 → 1233 keys. Block
    // 4183 agrees whole (no second Senate, `MAKE[8].t` 438 on both sides,
    // `1/94` gathers); 4177's standing families stand, and groups 77 and
    // 78 hold a pool member in ours alone from 4202 and 4238. The state
    // parts on block 4311: who=1's army (`1/26`..`1/51`) holds attack
    // orders on the human's Citizen `0/10` (TypeIndex 50, land, idle at
    // (5016, 8088)) on both sides, and theirs lands a blow on frame 4310
    // (`hits_left` 37 against ours 40, `damage_frame` 4310 against 0) —
    // so both leaders' `treaties` read 3 in theirs (the hit's bit 1)
    // against ours 1, and `0/10` flees to (4536, 6648) in theirs alone.
    // The word is 4324 (block 4325).
    //
    // Item 3599, piece 38's bays: 1233 → 167 keys. `1/32`'s and `1/33`'s
    // stones at `0/10`, thrown on 4305, leave from (5950, 8535) and
    // (5963, 8757) in 6 and 7 frames, as run1247's `AMMO` records print,
    // where ours threw from the units' squares in 7 and 8; block 4311
    // agrees whole. The word is 4429 (block 4430).
    //
    // Item 3600, piece 824's bays: 167 → 82 keys. `1/46`'s arrow at
    // `0/10`, loosed on 4318, leaves from (6481, 9189) in 20 frames where
    // ours loosed from (6552, 9240) in 21, so `0/10`'s `damage` is 9 on
    // block 4338 on both sides (ours 8 before); the Citizen `0/11`'s
    // wound on 4386 and the Bowmen `1/44`..`1/46`'s angles from 4396
    // agree with it. What stands is the standing families (4177, 4179,
    // groups 77 and 78 from 4202 and 4238, `1/79`'s `form` on 4302).
    pin_eq!(w.firsts.len(), 82, "run1234b baseline");
}

/// **The closing state** (item 3600): run1234's closing block 5936, the
/// whole map's units, walked from run1233's start.
#[test]
fn maya_ancient_pair_run1234_s_closing_state() {
    let _pins = Pins::hold();
    let p = &MAYA_ANCIENT;
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
        "Maya Ancient pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5936, "the closing block");
    pin_eq!(e.compared, 180, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1248: the closing window, blocks 5680..5936 at the long's detail
/// with `LEADERS=9` (item 3600) — run1234 prints units every hundredth
/// block and no leader, building or city record past the start, so no
/// dump held the last blocks.
pub(crate) const RUN1248: &str = "gamelog-run1248-eastindies-maya-ancient-window-5680-5936.txt";

/// The closing window: block 5680 through the closing block 5936.
pub(crate) const WINDOW_MAYA_ANCIENT_CLOSING: (i64, i64) = (5680, 5936);

/// **The closing window**: run1248 walked from run1233's start with the
/// recorder on (item 3600).
pub(crate) fn maya_ancient_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MAYA_ANCIENT.start],
        true,
        MAYA_ANCIENT.long,
        "run1248",
        &[(RUN1248, WINDOW_MAYA_ANCIENT_CLOSING.0)],
        WINDOW_MAYA_ANCIENT_CLOSING,
        1,
        &[5936],
        true,
    )
}

#[test]
fn maya_ancient_pair_run1248_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = maya_ancient_closing_window() else {
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
    pin_eq!(w.firsts.len(), 86, "run1248 baseline");
}
