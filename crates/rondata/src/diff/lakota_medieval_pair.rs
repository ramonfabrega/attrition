//! **The Lakota at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, item 3561): East Indies, `MAP_STYLE 18`, human Nubians (4) against
//! the **Lakota (19)** at Difficulty 5, seed 12345, `STARTING_RESOURCES
//! 7`, the age held at Medieval: `STARTING_TECHNOLOGY 2`,
//! `ENDING_TECHNOLOGY 2` — the Persians' held Medieval lobby with the
//! nation moved. The Lakota's other pair on this map is `lakota_pair`, the
//! peace pair, at Difficulty 3. Captures: run1203 (a `DUMP_ALL` start) and
//! run1204 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §314.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const LAKOTA_MEDIEVAL: Pair = Pair {
    name: "EastIndiesLakotaMedieval",
    map_style: 18,
    ai_tribe: 19,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1203-eastindies-lakota-medieval-start.txt",
    long: (
        "gamelog-run1204-eastindies-lakota-medieval-trace.txt",
        "rontrace-run1204.log",
    ),
    // The game ended itself on 4973 (closing block 4974).
    length: 4973,
    // Item 3561, first try: frame 4168 by the count and the sequence, ours
    // 90 draws against 89, index 87 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x1ed`, theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` section 314). Item 3588: 4168 →
    // **4973, the game's last frame** — the Eagle Feather Horse `1/32`
    // (TypeIndex 206, piece 156) looses through its pivot
    // (`sim::pivot::RELEASES`), its round at the Citizen `0/5` flies 17
    // frames from (5484, 10096) and strikes on 4143 beside `1/12`'s, and
    // the Heavy Archers `1/19` re-aims at the wounded `0/5` on 4168.
    count: 4973,
    sequence: 4973,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const LAKOTA_MEDIEVAL_PAIR_WORD: i64 =
    if LAKOTA_MEDIEVAL.count < LAKOTA_MEDIEVAL.sequence {
        LAKOTA_MEDIEVAL.count
    } else {
        LAKOTA_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &LAKOTA_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "LAKOTA_MEDIEVAL {}: count {} sequence {} of {}; {}",
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
fn lakota_medieval_pair_captures_say_their_lobby() {
    let p = &LAKOTA_MEDIEVAL;
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
        "the Lakota Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(18, 19, 5)` is no held-out game's — the
/// battery's one Lakota game is `GreatSaharaLakotaTier2`, `(7, 19, 2)`, and
/// `GreatLakesLakotaTier2`'s `(14, 19, 2)` was promoted to a scored word
/// (item 3345) — and no other pinned pair's: the Lakota's other pair is
/// `lakota_pair`'s `(18, 19, 3)`, the peace pair on this map at
/// Difficulty 3 and no held age. Both `GAME INFO`s are read.
#[test]
fn lakota_medieval_pair_is_no_other_lobby() {
    let p = &LAKOTA_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let lakota: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        lakota,
        [("GreatSaharaLakotaTier2", 7, 2)],
        "the battery's Lakota games"
    );
    let other = &super::lakota_pair::LAKOTA;
    assert_eq!(other.ai_tribe, p.ai_tribe);
    assert_ne!(
        (other.map_style, other.ai_tribe, other.difficulty),
        triple,
        "{} is this lobby",
        other.name
    );
    let info = |name: &str| {
        let path = dump(name)?;
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial()?;
        let get = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        Some((
            get("MAP_STYLE"),
            get("DIFFICULTY"),
            get("STARTING_TECHNOLOGY"),
            get("ENDING_TECHNOLOGY"),
        ))
    };
    let (Some(peace), Some(ours)) = (info(other.long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(peace.0, Some(18), "the peace pair's map");
    assert_eq!(peace.1, Some(3), "the peace pair's difficulty");
    assert_eq!(
        ours,
        (Some(18), Some(5), Some(2), Some(2)),
        "this pair's lobby"
    );
}

/// run1204b: the word 4168's window, blocks 4163..4419 at the long's
/// detail — six blocks before the word's block 4169 and 250 after (item
/// 3561); the stride's blocks are 4101 and 4201.
pub(crate) const RUN1204B: &str =
    "gamelog-run1204b-eastindies-lakota-medieval-window-4163-4419.txt";

/// The window: block 4163 through 4419; the word 4168's own block is 4169.
pub(crate) const WIDENING_LAKOTA_MEDIEVAL_FRAME_4168: (i64, i64) = (4163, 4419);

/// **The word 4168's window**: run1204b walked from run1203's start with
/// the recorder on (item 3561).
pub(crate) fn lakota_medieval_frame_4168_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_MEDIEVAL.start],
        true,
        LAKOTA_MEDIEVAL.long,
        "run1204b",
        &[(RUN1204B, WIDENING_LAKOTA_MEDIEVAL_FRAME_4168.0)],
        WIDENING_LAKOTA_MEDIEVAL_FRAME_4168,
        1,
        &[4169],
        true,
    )
}

#[test]
fn lakota_medieval_pair_run1204b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_medieval_frame_4168_window() else {
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
    // Item 3561, the first walk: 2163 keys. Block 4163 carries the
    // standing families (`form`, `SITE[i].reg`). **The state first parts
    // on block 4169**, at the Lakota Heavy Archers `1/19` (TypeIndex 173,
    // land, uid 39), standing at (6264, 10008) in an `ATTACKORDER` on the
    // fleeing Nubian Citizen `0/10` (uid 17): theirs re-aims the attack at
    // the Citizen `0/5` (uid 12; `new_ord` 1, `in_range` 0, `orders.len`
    // 2, `near` 0/2000) and pushes its move to (5400, 9000) on 4170; ours
    // keeps `0/10` and pushes a chase to (6696, 9528) on 4169 (`order:kind`
    // ours 1 theirs 10, `orders.len` 3 against 2). **The word 4168** (block
    // 4169), ours 90 draws against 89, index 87 ours `Guy::set_anim+0x97a
    // < Guy::inc_time+0x1ed` (`1/19`), theirs `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` (`docs/AI.md` section 314).
    //
    // Item 3588: **2163 → 115**, every one a standing family — block 4163's
    // (`form`, `SITE[i].reg`, who=0's `aggression[1]`, who=1's groups'
    // `held`/`pool`; 96 rows) and later blocks' groups 77 and 86 (4164,
    // 4189), `SITE[3..6].reg` (4176, 4376) and the newborns' `form`. With
    // the Eagle Feather Horse's round through its pivot, `0/5` stands
    // struck for 30 + 10/16 on 4163, `1/19` re-aims at it on 4168 as
    // theirs does, and block 4169 agrees.
    pin_eq!(w.firsts.len(), 115, "run1204b baseline");
}

/// **The pair's closing state**: run1204's closing dump, block 4974,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3588).
#[test]
fn run1204_lakota_medieval_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &LAKOTA_MEDIEVAL;
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
        "Lakota Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4974, "the closing block");
    // Item 3588, the pair whole: 214 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 214, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1235: the Citizen `0/5`'s rounds, blocks 4086..4170 at `AMMO=5` over
/// the long's detail (item 3588) — the stride's block 4101 prints it
/// unstruck and run1204b's 4163 struck for 30, so no dump held the blows.
pub(crate) const RUN1235: &str = "gamelog-run1235-eastindies-lakota-medieval-window-4086-4170.txt";

/// The window: block 4086 through 4170; the Eagle Feather Horse's round
/// lands on 4143 (block 4144).
pub(crate) const WIDENING_LAKOTA_MEDIEVAL_ROUNDS: (i64, i64) = (4086, 4170);

/// **The rounds at `0/5`**: run1235 walked from run1203's start with the
/// recorder on (item 3588).
pub(crate) fn lakota_medieval_rounds_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_MEDIEVAL.start],
        true,
        LAKOTA_MEDIEVAL.long,
        "run1235",
        &[(RUN1235, WIDENING_LAKOTA_MEDIEVAL_ROUNDS.0)],
        WIDENING_LAKOTA_MEDIEVAL_ROUNDS,
        1,
        &[4144],
        true,
    )
}

#[test]
fn lakota_medieval_pair_run1235_s_rounds_are_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_medieval_rounds_window() else {
        return;
    };
    pin_eq!(w.blocks, 84, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3588, the first walk: **101** keys, every one a standing
    // family — block 4086's (`form`, `SITE[i].reg`, `aggression[1]`, the
    // groups' `held`/`pool`; 97 rows), group 85 on 4087 and two newborns'
    // `form` (4098, 4120). The rounds at `0/5` agree: `1/11`'s on 4092,
    // `1/17`'s on 4116, and `1/32`'s (5484, 10096, 263) on 4127, 17 frames,
    // with `1/12`'s on 4130 — the two strike together on 4143 (block 4144:
    // `damage` 30, `damage_frac` 10, `damage_o` 32 on both sides). Without
    // piece 156's rows ours looses from the figure's (5544, 10152), flies
    // 18, and `0/5` stands at 21 + 5/16 from 1/11 on 4163.
    pin_eq!(w.firsts.len(), 101, "run1235 baseline");
}

/// run1235c: the pair's last blocks, 4718..4974 at the long's detail —
/// the word 4973 is the game's last frame and 4974 its closing block (item
/// 3588). run1204 prints units only every hundredth block, and run1204b
/// ends on 4419, so no dump held the last blocks.
pub(crate) const RUN1235C: &str =
    "gamelog-run1235c-eastindies-lakota-medieval-window-4718-4974.txt";

/// The window: block 4718 through the closing block 4974.
pub(crate) const WIDENING_LAKOTA_MEDIEVAL_CLOSING: (i64, i64) = (4718, 4974);

/// **The pair's last blocks**: run1235c walked from run1203's start with
/// the recorder on. The word is the game's last frame (item 3588).
pub(crate) fn lakota_medieval_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_MEDIEVAL.start],
        true,
        LAKOTA_MEDIEVAL.long,
        "run1235c",
        &[(RUN1235C, WIDENING_LAKOTA_MEDIEVAL_CLOSING.0)],
        WIDENING_LAKOTA_MEDIEVAL_CLOSING,
        1,
        &[4974],
        true,
    )
}

#[test]
fn lakota_medieval_pair_run1235c_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_medieval_closing_window() else {
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
    // Item 3588, the first walk: **115** keys. Block 4718 carries the
    // standing families (`form`, `SITE[i].reg`, `aggression[1]`, the
    // groups' `held`/`pool`), and later blocks groups 80 and 86's (4719,
    // 4828), `SITE[7].reg` (4776) and the `1/132`'s `form` (4775). **Two
    // rows are not standing families**: `dest_angle` on the `1/103` (4782)
    // and `1/104` (4790), 3567a's shape (parked 3580b). The closing block
    // 4974 agrees.
    pin_eq!(w.firsts.len(), 115, "run1235c baseline");
}
