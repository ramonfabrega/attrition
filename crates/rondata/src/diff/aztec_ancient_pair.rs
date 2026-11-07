//! **The Aztecs from an Ancient start, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3604): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Aztecs (0)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, `STARTING_TECHNOLOGY 0` and `ENDING_TECHNOLOGY
//! 1` (read back 1; age 0 cannot be held, 3559a) — the Japanese's
//! Ancient-start lobby with the nation moved. Its triple, `(18, 0, 5)`, is
//! new: the held-out `EastIndiesAztecTier1` is `(18, 0, 1)`, another
//! difficulty, and the Aztecs' other lobbies are Great Sahara's —
//! `aztec_alltech_pair`'s `(7, 0, 3)` and `aztec_word`'s
//! `GreatSaharaAztecTier2`, `(7, 0, 2)`. Captures: run1255 (a `DUMP_ALL`
//! start) and run1256 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §325.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const AZTEC_ANCIENT: Pair = Pair {
    name: "EastIndiesAztecAncient",
    map_style: 18,
    ai_tribe: 0,
    difficulty: 5,
    starting_technology: 0,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1255-eastindies-aztec-ancient-start.txt",
    long: (
        "gamelog-run1256-eastindies-aztec-ancient-trace.txt",
        "rontrace-run1256.log",
    ),
    // The game ended itself on 9258 (closing block 9259).
    length: 9258,
    // Item 3604, first try: frame 8716 by the count and the sequence, ours
    // 16 draws against theirs' 17, parting at index 2 — ours
    // `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs `Guy::set_anim+0x97a
    // < Unit::move_step+0x823` (`docs/AI.md` section 325). Item 3613: the
    // Royal Atl-Atls throw from piece 36's bays (`sim::launch`), and the
    // pair is whole — 8716 → 9258, the game's last frame (section 325.1).
    count: 9258,
    sequence: 9258,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const AZTEC_ANCIENT_PAIR_WORD: i64 = if AZTEC_ANCIENT.count < AZTEC_ANCIENT.sequence {
    AZTEC_ANCIENT.count
} else {
    AZTEC_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn aztec_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &AZTEC_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "AZTEC_ANCIENT {}: count {} sequence {} of {}; {}",
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
fn aztec_ancient_pair_captures_say_their_lobby() {
    let p = &AZTEC_ANCIENT;
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
        "the Aztec Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3604; the battery's identity
/// test refuses it) and none of the battery's: the held-out
/// `EastIndiesAztecTier1` is `(18, 0, 1)`, another difficulty, and the
/// Aztecs' other pinned lobbies are Great Sahara's (item 3604).
#[test]
fn aztec_ancient_pair_is_no_other_lobby() {
    let p = &AZTEC_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let held_out = battery::BATTERY
        .iter()
        .find(|l| l.name == "EastIndiesAztecTier1")
        .expect("the held-out Aztec game");
    assert_eq!(
        (held_out.map_style, held_out.ai_tribe, held_out.difficulty),
        (18, 0, 1),
        "the held-out Aztec game is another difficulty"
    );
    for (name, other) in [
        (
            super::aztec_word::AZTEC.name,
            (
                super::aztec_word::AZTEC.map_style,
                super::aztec_word::AZTEC.ai_tribe,
                super::aztec_word::AZTEC.difficulty,
            ),
        ),
        (
            super::aztec_alltech_pair::AZTEC_ALLTECH.name,
            (
                super::aztec_alltech_pair::AZTEC_ALLTECH.map_style,
                super::aztec_alltech_pair::AZTEC_ALLTECH.ai_tribe,
                super::aztec_alltech_pair::AZTEC_ALLTECH.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// run1256b: the word 8716's window, blocks 8710..8966 at the long's
/// detail with `AMMO=5` — six blocks before the word's block 8717 and 250
/// after (item 3604); the long's stride prints units every hundredth block
/// (8701 and 8801) and no `LEADERS` past the start.
pub(crate) const RUN1256B: &str = "gamelog-run1256b-eastindies-aztec-ancient-window-8710-8966.txt";

/// The window: block 8710 through 8966; the word 8716's own block is 8717.
pub(crate) const WIDENING_AZTEC_ANCIENT_FRAME_8716: (i64, i64) = (8710, 8966);

/// **The word 8716's window**: run1256b walked from run1255's start with
/// the recorder on (item 3604).
pub(crate) fn aztec_ancient_frame_8716_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_ANCIENT.start],
        true,
        AZTEC_ANCIENT.long,
        "run1256b",
        &[(RUN1256B, WIDENING_AZTEC_ANCIENT_FRAME_8716.0)],
        WIDENING_AZTEC_ANCIENT_FRAME_8716,
        1,
        &[8717],
        true,
    )
}

#[test]
fn aztec_ancient_pair_run1256b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_ancient_frame_8716_window() else {
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
    // Item 3604, the first walk: 1684 keys. The window opens apart on
    // 8710, past the stride's 8701, which already parts: beside the
    // standing families (every unit's `form`, who=1's `g.angle[0]` 1431655765
    // against 0 on the units the stride names), three Citizens (TypeIndex
    // 50) took their wounds a frame earlier in theirs — `0/4` on 8693
    // against 8694, `0/13` on 8699 against 8700, and `0/11` on 8709 in
    // theirs alone (`hits_left` 36 against 40, fleeing to (7080, 7320)).
    // The shooters are who=1's Royal Atl-Atls (TypeIndex 86, group 92):
    // ours' `1/195` round at `0/11` leaves its own square (7464, 9768) and
    // flies 11, theirs' leaves (7476, 9641, 348) and flies 10. On 8717 `0/4`
    // collides in theirs alone (`collide_o` 3, path 4 against 1); the word's
    // extra draw is theirs' `Guy::set_anim < Unit::move_step`, at index 2.
    // Item 3613, with piece 36's bays: 136, the standing families alone.
    pin_eq!(w.firsts.len(), 136, "run1256b baseline");
}

/// **The long's stride, walked back from the word 8716** (item 3604):
/// only run1256's stride blocks are on disk before run1256b's window.
pub(crate) const WIDENING_AZTEC_ANCIENT_STRIDE: (i64, i64) = (101, 8701);

pub(crate) fn aztec_ancient_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_ANCIENT.start],
        true,
        AZTEC_ANCIENT.long,
        "run1256 stride",
        &[(AZTEC_ANCIENT.long.0, WIDENING_AZTEC_ANCIENT_STRIDE.0)],
        WIDENING_AZTEC_ANCIENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn aztec_ancient_pair_the_long_s_stride_walks_back_from_8716() {
    let _pins = Pins::hold();
    let Some(w) = aztec_ancient_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 87, "every stride block in the window");
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
    // Item 3604: 324 keys — every unit's `form` and `build:extra`, which
    // the stride does not print, the pools, `1/107`'s `mylos` 4 against 10
    // on 5701 (the Catapult, TypeIndex 265), and who=1's `g.angle[0]`
    // 1431655765 against 0 from 8101. The first unit rows past those are on
    // 8701: the Citizens `0/4` and `0/13` wounded a frame earlier in theirs
    // (`damage_frame` 8693 against 8694, 8699 against 8700). So the state
    // parts in the gap 8602..8700. Item 3613, with piece 36's bays: 306 —
    // the wounds agree, and `1/107`'s `mylos` and the standing families
    // stay.
    pin_eq!(w.firsts.len(), 306, "the stride's baseline");
}

/// run1266: the gap's window, blocks 8596..8710 at run1256b's detail with
/// `AMMO=5` (item 3613) — the stride agrees through 8601 and run1256b
/// opens apart on 8710, so no dump held 8602..8709.
pub(crate) const RUN1266: &str = "gamelog-run1266-eastindies-aztec-ancient-window-8596-8710.txt";

/// The window: block 8596 through 8710.
pub(crate) const WIDENING_AZTEC_ANCIENT_GAP: (i64, i64) = (8596, 8710);

/// **The gap's window**: run1266 walked from run1255's start with the
/// recorder on (item 3613).
pub(crate) fn aztec_ancient_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_ANCIENT.start],
        true,
        AZTEC_ANCIENT.long,
        "run1266",
        &[(RUN1266, WIDENING_AZTEC_ANCIENT_GAP.0)],
        WIDENING_AZTEC_ANCIENT_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn aztec_ancient_pair_run1266_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_ancient_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 114, "every captured block");
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
    // Item 3613, before piece 36's bays: 189 keys. Past the standing
    // families (every unit's `form`, who=0's `SITE[i].reg`,
    // `aggression[1]`, who=1's `g.angle[0]` and groups' `held`), the
    // Royal Atl-Atls `1/175`'s `dest_angle` on 8692 (1073741824 against
    // 852099072, parked), then the Citizen `0/4`'s wound on block 8694 —
    // `damage_frame` 0 against 8693, `hits_left` 40 against 36, its flight
    // (order 7 against 4) — struck by `1/163`'s javelin, which left the
    // figure's own (6312, 10008) in 15 frames in ours and (6327, 9920) in
    // 14 in theirs; then `0/13`'s on 8700, `1/155`'s from (6540, 9742).
    // With the bays: 141, the standing families and `1/175`'s
    // `dest_angle` alone.
    pin_eq!(w.firsts.len(), 141, "run1266 baseline");
}

/// **The closing state** (item 3613): run1256's closing block 9259, the
/// whole map's units, walked from run1255's start.
#[test]
fn aztec_ancient_pair_run1256_s_closing_state() {
    let _pins = Pins::hold();
    let p = &AZTEC_ANCIENT;
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
        "Aztec Ancient pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 9259, "the closing block");
    pin_eq!(e.compared, 371, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1266d: the closing window, blocks 9003..9259 at the long's detail
/// with `LEADERS=9` (item 3613) — run1256 prints units every hundredth
/// block and no leader, building or city record past the start, so no
/// dump held the last blocks.
pub(crate) const RUN1266D: &str = "gamelog-run1266d-eastindies-aztec-ancient-window-9003-9259.txt";

/// The closing window: block 9003 through the closing block 9259.
pub(crate) const WINDOW_AZTEC_ANCIENT_CLOSING: (i64, i64) = (9003, 9259);

/// **The closing window**: run1266d walked from run1255's start with the
/// recorder on (item 3613).
pub(crate) fn aztec_ancient_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[AZTEC_ANCIENT.start],
        true,
        AZTEC_ANCIENT.long,
        "run1266d",
        &[(RUN1266D, WINDOW_AZTEC_ANCIENT_CLOSING.0)],
        WINDOW_AZTEC_ANCIENT_CLOSING,
        1,
        &[9259],
        true,
    )
}

#[test]
fn aztec_ancient_pair_run1266d_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = aztec_ancient_closing_window() else {
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
    // Item 3613: 163 keys, the standing families alone, through the
    // closing block 9259.
    pin_eq!(w.firsts.len(), 163, "run1266d baseline");
}
