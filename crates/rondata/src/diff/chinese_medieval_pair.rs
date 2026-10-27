//! **The Chinese at a held Medieval age, Great Sahara** (DECISIONS 90, 91,
//! 92, item 3560): Great Sahara, `MAP_STYLE 7`, human Nubians (4) against
//! the **Chinese (14)** at Difficulty 5, seed 12345, `STARTING_RESOURCES
//! 7`, the age held at Medieval: `STARTING_TECHNOLOGY 2`,
//! `ENDING_TECHNOLOGY 2` — the Mongols' held Classical lobby with the
//! nation and the age moved. The Chinese's other pair on this map is
//! `chinese_industrial_pair`, at Difficulty 3 and a held Industrial age.
//! Captures: run1201 (a `DUMP_ALL` start) and run1202 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §313.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const CHINESE_MEDIEVAL: Pair = Pair {
    name: "GreatSaharaChineseMedieval",
    map_style: 7,
    ai_tribe: 14,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1201-greatsahara-chinese-medieval-start.txt",
    long: (
        "gamelog-run1202-greatsahara-chinese-medieval-trace.txt",
        "rontrace-run1202.log",
    ),
    // The game ended itself on 4520 (closing block 4521).
    length: 4520,
    // Item 3560, first try: frame 2981 by the count and the sequence, ours
    // 12 draws against 13, index 0 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (`docs/AI.md` section 313).
    // Item 3577: 2981 -> 3030 — the Fire Lances loose from piece 46's bays
    // (`sim::launch`), so `1/22`'s round flies 10 frames, not 11, and strikes
    // the Citizen `0/10` on 2977 as theirs does. Ours 14 draws against 16,
    // index 9, ours `Farms::inc_time+0x1ae`, theirs `Unit::close+0xce6 <
    // Object::die` (a death after a round's `AmmoOut::dirt_splash`).
    // Item 3580: 3030 -> 4520 of 4520, the pair whole — the Fire Lances
    // `1/31`'s round (`FirelanceAmmo`, `flame="1"`, `ammo_flags & 0x10`)
    // kills the Citizen `0/10` with `dtype` 4, so `Unit::close` spends its
    // three draws, `+0xcb6`, `+0xce6` and `+0xd0a`, where ours spent one.
    count: 4520,
    sequence: 4520,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const CHINESE_MEDIEVAL_PAIR_WORD: i64 =
    if CHINESE_MEDIEVAL.count < CHINESE_MEDIEVAL.sequence {
        CHINESE_MEDIEVAL.count
    } else {
        CHINESE_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn chinese_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &CHINESE_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "CHINESE_MEDIEVAL {}: count {} sequence {} of {}; {}",
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
fn chinese_medieval_pair_captures_say_their_lobby() {
    let p = &CHINESE_MEDIEVAL;
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
        "the Chinese Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(7, 14, 5)` is no held-out game's — the
/// battery's one Chinese game is `GreatLakesChineseTier1`, `(14, 14, 1)` —
/// and no other pinned pair's: the Chinese's other pair is
/// `chinese_industrial_pair`'s `(7, 14, 3)`, this map at Difficulty 3 and a
/// held Industrial age. Both `GAME INFO`s are read: the same map's Chinese
/// pair against this pair's held age (3513's shape, Loop 3544).
#[test]
fn chinese_medieval_pair_is_no_other_lobby() {
    let p = &CHINESE_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let chinese: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        chinese,
        [("GreatLakesChineseTier1", 14, 1)],
        "the battery's Chinese games"
    );
    let other = &super::chinese_industrial_pair::CHINESE_INDUSTRIAL;
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
    let (Some(industrial), Some(ours)) = (info(other.long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(
        industrial,
        (Some(7), Some(3), Some(5), Some(5)),
        "the Chinese Industrial pair's lobby"
    );
    assert_eq!(
        ours,
        (Some(7), Some(5), Some(2), Some(2)),
        "this pair's lobby"
    );
}

/// **The pair's closing state**: run1202's closing dump, block 4521,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3580).
#[test]
fn run1202_chinese_medieval_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &CHINESE_MEDIEVAL;
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
        "Chinese Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4521, "the closing block");
    // Item 3580, the pair whole: 206 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 206, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1202b: the word 2981's window, blocks 2975..3231 at the long's
/// detail — six blocks before the word's block 2982 and 250 after (item
/// 3560); the stride's blocks are 2901 and 3001.
pub(crate) const RUN1202B: &str =
    "gamelog-run1202b-greatsahara-chinese-medieval-window-2975-3231.txt";

/// The window: block 2975 through 3231; the word 2981's own block is 2982.
pub(crate) const WIDENING_CHINESE_MEDIEVAL_FRAME_2981: (i64, i64) = (2975, 3231);

/// **The word 2981's window**: run1202b walked from run1201's start with
/// the recorder on (item 3560).
pub(crate) fn chinese_medieval_frame_2981_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_MEDIEVAL.start],
        true,
        CHINESE_MEDIEVAL.long,
        "run1202b",
        &[(RUN1202B, WIDENING_CHINESE_MEDIEVAL_FRAME_2981.0)],
        WIDENING_CHINESE_MEDIEVAL_FRAME_2981,
        1,
        &[2982],
        true,
    )
}

#[test]
fn chinese_medieval_pair_run1202b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_medieval_frame_2981_window() else {
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
    // Item 3560, the first walk: 2437 keys. Block 2975 carries the
    // standing families (`form`, `SITE[i].reg`, who=0's `aggression[1]`,
    // who=1's groups' `held` and `pool`). **The state first parts on block
    // 2978**: the Nubian Citizen `0/10` (TypeIndex 50, land, uid 17), under
    // a volley of who=1's Fire Lances (`1/21`, `1/22`, `1/23`, TypeIndex
    // 96), reads `hits:damage` ours 0 theirs 7, `damage_frame` 0 against
    // 2977 and `orders.len` 0 against 1 (theirs' `FLEEORDER` to (6552,
    // 30984)); `leader:treaties[0]`/`[1]` ours 1 theirs 3 on the same
    // block. **The word 2981** (block 2982), ours 12 draws against 13,
    // index 0 ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` (`docs/AI.md` section
    // 313).
    //
    // Item 3577, 2437 -> 2738 keys: piece 46's bays put `1/22`'s blow on
    // 2977, and block 2978's Citizen rows agree. After 2976 the first rows
    // are who=1's groups 70 and 68 (`held`, `pool`, 3019/3020, the
    // standing family), then the Crossbowmen `1/127` (TypeIndex 172, land)
    // on 3037: `order:move.dest` ours (36264, 17304) against theirs (35496,
    // 17112), `path:length` 22 against 12. **The word 3030** (block 3031,
    // no record parted on it): ours 14 draws against 16, index 9, ours
    // `Farms::inc_time+0x1ae`, theirs `Unit::close+0xce6 < Object::die`.
    //
    // Item 3580, 2738 -> 53 keys: the Fire Lances `1/31`'s round kills the
    // Citizen `0/10` on 3030 with `dtype` 4 (`flame="1"`), `Unit::close`'s
    // three draws in both, and the stream agrees to the game's end. What is
    // left is the standing families: block 2975's, who=1's `SITE[0].reg`
    // on 2976, groups 70 and 68 on 3019/3020; the Crossbowmen `1/127`'s
    // 3037 rows agree.
    pin_eq!(w.firsts.len(), 53, "run1202b baseline");
}

/// run1220: the Fire Lances' volley at `AMMO=5`, blocks 2955..2982 (item
/// 3577) — the three rounds `1/21`, `1/22` and `1/23` loosed at the Citizen
/// `0/10` over 2963..2973, each record's `sx, sy, sz` and `total_time`.
pub(crate) const RUN1220: &str =
    "gamelog-run1220-greatsahara-chinese-medieval-window-2955-2982.txt";

/// The window: block 2955 through 2982, the old word 2981's block the last.
pub(crate) const WINDOW_CHINESE_MEDIEVAL_VOLLEY_2955: (i64, i64) = (2955, 2982);

/// **The volley's window**: run1220 walked from run1201's start with the
/// recorder on (item 3577).
pub(crate) fn chinese_medieval_volley_2955_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_MEDIEVAL.start],
        true,
        CHINESE_MEDIEVAL.long,
        "run1220",
        &[(RUN1220, WINDOW_CHINESE_MEDIEVAL_VOLLEY_2955.0)],
        WINDOW_CHINESE_MEDIEVAL_VOLLEY_2955,
        1,
        &[2978],
        true,
    )
}

/// **run1220's volley window, walked whole** (item 3577): with piece 46's
/// bays every round leaves from theirs' point and flies theirs' frames, so
/// the Citizen `0/10`'s blow on 2977 and its flight agree to the window's end.
#[test]
fn chinese_medieval_pair_run1220_s_volley_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_medieval_volley_2955_window() else {
        return;
    };
    pin_eq!(w.blocks, 27, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // 84 keys, every one on the window's first block 2955 and every one a
    // standing family: each unit's `form`, `build:extra` on who=0's seven
    // buildings (the capture prints no `BUILDS`) and who=1's groups 64..69
    // `held`/`pool` (nor `GROUPS`). Nothing parts after 2955: the three
    // rounds, the Citizen's blow on 2977 and its flight agree. Without piece
    // 46's bays the Citizen `0/10` parts on 2978.
    pin_eq!(w.firsts.len(), 84, "run1220 baseline");
}

/// run1223: the pair's last blocks, 4265..4521 at the window's detail —
/// the word 4520 is the game's last frame and 4521 its closing block (item
/// 3580). run1202 prints units only every hundredth block, and run1202b
/// ends on 3231, so no dump held the last blocks.
pub(crate) const RUN1223: &str =
    "gamelog-run1223-greatsahara-chinese-medieval-window-4265-4521.txt";

/// The window: block 4265 through the closing block 4521.
pub(crate) const WIDENING_CHINESE_MEDIEVAL_CLOSING: (i64, i64) = (4265, 4521);

/// **The pair's last blocks**: run1223 walked from run1201's start with
/// the recorder on. The word is the game's last frame (item 3580).
pub(crate) fn chinese_medieval_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_MEDIEVAL.start],
        true,
        CHINESE_MEDIEVAL.long,
        "run1223",
        &[(RUN1223, WIDENING_CHINESE_MEDIEVAL_CLOSING.0)],
        WIDENING_CHINESE_MEDIEVAL_CLOSING,
        1,
        &[4521],
        true,
    )
}

#[test]
fn chinese_medieval_pair_run1223_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_medieval_closing_window() else {
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
    // Item 3580, the first walk: **52** keys. Block 4265 carries the
    // standing families (`SITE[i].reg`, who=0's `aggression[1]`, `form`,
    // who=1's groups' `held`/`pool`), and later blocks groups 69, 70 and
    // 72's (4293, 4371, 4393) and who=1's `SITE[0].reg` (4376). **Four
    // rows are not standing families**: `dest_angle` on the `1/86` (4424),
    // `1/96` (4427) and `1/97` (4428), 3567a's shape (parked 3580b); and
    // on block 4452 the Nubian capital `0/2000`'s `damage_frac`, ours 2
    // against theirs 12 — the Crossbowmen `1/69` (TypeIndex 172, piece
    // 1178, no `sim::launch` bay) lands its round a frame late, and the
    // two agree again on 4453 (parked 3580a). The closing block 4521
    // agrees.
    // Item 3712: 52 → **51**. Piece 1178's `launch::BAYS` rows (the
    // Mongols' Crossbowmen, the same Asian style) launch `1/69`'s round
    // from its bay, and 4452's `0/2000` `damage_frac` agrees (parked 3580a,
    // struck).
    pin_eq!(w.firsts.len(), 51, "run1223 baseline");
}
