//! **The Japanese at a held Gunpowder age, Great Sahara** (DECISIONS 90,
//! 91, 93, item 3563): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Japanese (15)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — the Chinese held Medieval lobby with the
//! nation and the age moved. The Japanese's other pair is
//! `japanese_industrial_pair`, East Indies at a held Industrial age, and
//! the Japanese on this map at Difficulty 1 is the AI word
//! `japanese_word`'s `GreatSaharaJapaneseTier1`. Captures: run1207 (a
//! `DUMP_ALL` start) and run1208 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §316.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_GUNPOWDER: Pair = Pair {
    name: "GreatSaharaJapaneseGunpowder",
    map_style: 7,
    ai_tribe: 15,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1207-greatsahara-japanese-gunpowder-start.txt",
    long: (
        "gamelog-run1208-greatsahara-japanese-gunpowder-trace.txt",
        "rontrace-run1208.log",
    ),
    // The game ended itself on 3673 (closing block 3674).
    length: 3673,
    // Item 3563, first try: frame 2839 by the count and the sequence, ours
    // 8 draws against 9, index 1 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Unit::fight+0x824` (`docs/AI.md`
    // section 316). Item 3590: 2839 → **3673, the game's last frame** —
    // the Japanese barracks term (`get_damage`'s step 13, `n = min(ages,
    // epoch[0])` = 3, +15 %) puts the Arquebusiers `1/20`'s blow on the
    // Citizen `0/12` at 13 + 10/16 on 2675, and piece 1104's bays
    // (`sim::launch::BAYS`, run1237's packet) put `1/19`'s round of 2693 on
    // (9940, 30945), 6 frames, as theirs.
    count: 3673,
    sequence: 3673,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_GUNPOWDER_PAIR_WORD: i64 =
    if JAPANESE_GUNPOWDER.count < JAPANESE_GUNPOWDER.sequence {
        JAPANESE_GUNPOWDER.count
    } else {
        JAPANESE_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn japanese_gunpowder_pair_captures_say_their_lobby() {
    let p = &JAPANESE_GUNPOWDER;
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
        "the Japanese Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(7, 15, 5)` is no held-out game's — the battery
/// fields no Japanese since `GreatSaharaJapaneseTier1`, `(7, 15, 1)`, left
/// it for the AI word (`japanese_word`, item 3331) — and no other pinned
/// pair's: the Japanese Industrial pair is `(18, 15, 5)`, East Indies at a
/// held Industrial age. Both `GAME INFO`s are read: the Industrial pair's
/// against this pair's held age (3513's shape, Loop 3544).
#[test]
fn japanese_gunpowder_pair_is_no_other_lobby() {
    let p = &JAPANESE_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let japanese: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        japanese,
        Vec::<(&str, i32, i32)>::new(),
        "the battery's Japanese games"
    );
    let word = &super::japanese_word::JAPANESE;
    assert_eq!(word.ai_tribe, p.ai_tribe);
    assert_ne!(
        (word.map_style, word.ai_tribe, word.difficulty),
        triple,
        "{} is this lobby",
        word.name
    );
    let other = &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL;
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
        (Some(18), Some(5), Some(5), Some(5)),
        "the Japanese Industrial pair's lobby"
    );
    assert_eq!(
        ours,
        (Some(7), Some(5), Some(3), Some(3)),
        "this pair's lobby"
    );
}

/// run1208b: the word 2839's window, blocks 2834..3090 at the long's
/// detail — six blocks before the word's block 2840 and 250 after (item
/// 3563); the stride's blocks are 2801 and 2901.
pub(crate) const RUN1208B: &str =
    "gamelog-run1208b-greatsahara-japanese-gunpowder-window-2834-3090.txt";

/// The window: block 2834 through 3090; the word 2839's own block is 2840.
pub(crate) const WIDENING_JAPANESE_GUNPOWDER_FRAME_2839: (i64, i64) = (2834, 3090);

/// **The word 2839's window**: run1208b walked from run1207's start with
/// the recorder on (item 3563).
pub(crate) fn japanese_gunpowder_frame_2839_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_GUNPOWDER.start],
        true,
        JAPANESE_GUNPOWDER.long,
        "run1208b",
        &[(RUN1208B, WIDENING_JAPANESE_GUNPOWDER_FRAME_2839.0)],
        WIDENING_JAPANESE_GUNPOWDER_FRAME_2839,
        1,
        &[2840],
        true,
    )
}

#[test]
fn japanese_gunpowder_pair_run1208b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_gunpowder_frame_2839_window() else {
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
    // Item 3563, the first walk: 1333 keys. Block 2834 carries the
    // standing families (`form`, `SITE[i].reg`, who=0's `aggression[1]`,
    // who=1's groups 64..66 `held`/`pool`) and the wounds run1208c and
    // run1208d walk back: the Citizens `0/10`, `0/11` and `0/12`'s
    // `hits_left` ours 8/27/18 against theirs 5/25/14, `0/11`'s flight
    // (ours (5952, 31920), theirs (5927, 31921)), and the Elite Knight
    // `1/23` (TypeIndex 229, land, uid 42)'s chase `orders_y` ours 31944
    // against theirs 31896. **On block 2839 `1/23` parts by its orders**:
    // ours pushes a move (`order:kind` 1, `orders.len` 3), theirs keeps
    // its `ATTACKORDER` (10, 2); on 2840 theirs re-aims it off the Citizen
    // `0/11` (uid 18) onto `0/13` (uid 20, `new_ord` 1) and strikes `0/13`
    // by block 2842. **The word 2839** (block 2840), ours 8 draws against
    // 9, index 1, ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Unit::fight+0x824` (`docs/AI.md` section 316).
    // Item 3590: 1333 → **51**. Block 2834's 45 are the standing families
    // alone; groups 67 and 68's `held`/`pool` (2916, 2962) and who=1's
    // `SITE[1].reg` (2976) are standing families too. **One row is not**:
    // the `1/77`'s `dest_angle` on 3054, ours 1498415104 against theirs
    // 943259648, 3567a's shape (parked 3580b). Block 2840 agrees: `1/23`
    // re-aims onto `0/13` as theirs does.
    pin_eq!(w.firsts.len(), 51, "run1208b baseline");
}

/// run1208c: the blocks before the word's window, 2771..2834 (item 3563) —
/// run1208b's first block already holds `1/23`'s move and the Citizens'
/// wounds parted, and the stride's last block before it is 2801.
pub(crate) const RUN1208C: &str =
    "gamelog-run1208c-greatsahara-japanese-gunpowder-window-2771-2834.txt";

/// The window: block 2771 through 2834, run1208b's first block the last.
pub(crate) const WINDOW_JAPANESE_GUNPOWDER_BACK_2771: (i64, i64) = (2771, 2834);

/// **The walk back**: run1208c walked from run1207's start with the
/// recorder on (item 3563).
pub(crate) fn japanese_gunpowder_back_2771_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_GUNPOWDER.start],
        true,
        JAPANESE_GUNPOWDER.long,
        "run1208c",
        &[(RUN1208C, WINDOW_JAPANESE_GUNPOWDER_BACK_2771.0)],
        WINDOW_JAPANESE_GUNPOWDER_BACK_2771,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_gunpowder_pair_run1208c_walks_the_wound_back() {
    let _pins = Pins::hold();
    let Some(w) = japanese_gunpowder_back_2771_window() else {
        return;
    };
    pin_eq!(w.blocks, 63, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3563: 93 keys. Block 2771 carries the standing families and the
    // Citizens `0/10`'s and `0/12`'s wounds (run1208d's); who=1's
    // `SITE[0].reg` on 2776 and group 67's `held`/`pool` on 2787 are
    // standing families too. **On block 2809 the Citizen `0/11` (TypeIndex
    // 50, land, uid 18) parts**: theirs is struck on 2808 (`hits:damage`
    // 13 + 10/16, `hits_left` 37) and flees to (5016, 31944), ours is
    // unstruck at 50 and idles; on 2831 `1/23`'s chase of it follows
    // (`orders_y` 31944 against 31896).
    // Item 3590: 93 → **49**, every one a standing family (46 on 2771,
    // `SITE[0].reg` on 2776, group 67 on 2787); `0/11`'s blow on 2808
    // agrees.
    pin_eq!(w.firsts.len(), 49, "run1208c baseline");
}

/// The long's own stride blocks, 1 through 2801 — the Citizens `0/10` and
/// `0/12` are wounded apart on run1208c's first block 2771 (item 3563).
pub(crate) const WINDOW_JAPANESE_GUNPOWDER_STRIDE: (i64, i64) = (1, 2801);

/// **The walk back on the stride**: run1208's every hundredth block from
/// run1207's start with the recorder on (item 3563).
pub(crate) fn japanese_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_GUNPOWDER.start],
        true,
        JAPANESE_GUNPOWDER.long,
        "run1208 stride",
        &[(
            JAPANESE_GUNPOWDER.long.0,
            WINDOW_JAPANESE_GUNPOWDER_STRIDE.0,
        )],
        WINDOW_JAPANESE_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_gunpowder_pair_the_long_s_stride_walks_back_from_2771() {
    let _pins = Pins::hold();
    let Some(w) = japanese_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 29, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3563: 87 keys, all but six standing families (`build:extra`,
    // which this crate holds alone, `form`, `SITE[i].reg`, groups'
    // `held`/`pool`). The Citizen `0/12` agrees on 2601 and parts on 2701
    // (`hits:damage` ours 35 against theirs 39 + 10/16), `0/10` on 2801.
    // Item 3590: 87 → **81**, every one a standing family; `0/12` on 2701
    // and `0/10` on 2801 agree.
    pin_eq!(w.firsts.len(), 81, "run1208 stride baseline");
}

/// run1208d: blocks 2601..2701 at the window's detail and `AMMO=5` (item
/// 3563) — the stride's `0/12` agrees on 2601 and is wounded apart on 2701.
pub(crate) const RUN1208D: &str =
    "gamelog-run1208d-greatsahara-japanese-gunpowder-window-2601-2701.txt";

/// The window: block 2601 through 2701, the stride's two blocks either end.
pub(crate) const WINDOW_JAPANESE_GUNPOWDER_WOUND_2601: (i64, i64) = (2601, 2701);

/// **The first wound's window**: run1208d walked from run1207's start with
/// the recorder on (item 3563).
pub(crate) fn japanese_gunpowder_wound_2601_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_GUNPOWDER.start],
        true,
        JAPANESE_GUNPOWDER.long,
        "run1208d",
        &[(RUN1208D, WINDOW_JAPANESE_GUNPOWDER_WOUND_2601.0)],
        WINDOW_JAPANESE_GUNPOWDER_WOUND_2601,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_gunpowder_pair_run1208d_s_first_wound_is_widened() {
    let _pins = Pins::hold();
    let Some(w) = japanese_gunpowder_wound_2601_window() else {
        return;
    };
    pin_eq!(w.blocks, 100, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3563: 52 keys. 46 on block 2601 and two on 2643 are the
    // standing families. **The state first parts on block 2676**, four
    // rows on the Citizen `0/12` (TypeIndex 50, land, uid 19): the
    // Arquebusiers `1/20` (TypeIndex 98, land, uid 38)'s round, loosed on
    // 2673 with `total_time` 4, lands on 2675 in both, and `hits:damage`
    // reads ours 12 against theirs 13 + 10/16 (`hits_left` 38 against 37).
    // Theirs' `AMMO` leaves from (9443, 30828, 784), ours from the
    // figure's square (9576, 30888).
    // Item 3590: 52 → **48**, the standing families on 2601 and 2643
    // alone. Block 2676 agrees (`hits:damage` 13, `damage_frac` 10 on both
    // sides) with the Japanese term in, and the rounds of `1/20` (2673),
    // `1/18` (2679) and `1/19` (2693) leave from theirs' (9443, 30828),
    // (9838, 30835) and (9940, 30945) with piece 1104's bays.
    pin_eq!(w.firsts.len(), 48, "run1208d baseline");
}

/// **The pair's closing state**: run1208's closing dump, block 3674,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3590).
#[test]
fn japanese_gunpowder_pair_run1208_s_closing_state() {
    let _pins = Pins::hold();
    let p = &JAPANESE_GUNPOWDER;
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
        "Japanese Gunpowder pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3674, "the closing block");
    // Item 3590, the pair whole: 170 units compared, none off, torn or
    // unlinked, and no building or city field-row.
    pin_eq!(e.compared, 170, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1237b: the pair's last blocks, 3418..3674 at the long's detail —
/// the word 3673 is the game's last frame and 3674 its closing block (item
/// 3590). run1208 prints units only every hundredth block, and run1208b
/// ends on 3090, so no dump held the last blocks.
pub(crate) const RUN1237B: &str =
    "gamelog-run1237b-greatsahara-japanese-gunpowder-window-3418-3674.txt";

/// The window: block 3418 through the closing block 3674.
pub(crate) const WINDOW_JAPANESE_GUNPOWDER_CLOSING: (i64, i64) = (3418, 3674);

/// **The pair's last blocks**: run1237b walked from run1207's start with
/// the recorder on. The word is the game's last frame (item 3590).
pub(crate) fn japanese_gunpowder_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_GUNPOWDER.start],
        true,
        JAPANESE_GUNPOWDER.long,
        "run1237b",
        &[(RUN1237B, WINDOW_JAPANESE_GUNPOWDER_CLOSING.0)],
        WINDOW_JAPANESE_GUNPOWDER_CLOSING,
        1,
        &[3674],
        true,
    )
}

#[test]
fn japanese_gunpowder_pair_run1237b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_gunpowder_closing_window() else {
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
    // Item 3590, the first walk: **47** keys. Block 3418 carries the
    // standing families (40 rows: `form`, `SITE[i].reg`, `aggression[1]`,
    // the groups' `held`/`pool`), and later blocks groups 67 and 70's (3429,
    // 3478) and who=1's `SITE[2].reg` (3576). **Two rows are not standing
    // families**: `dest_angle` on the Arquebusiers `1/132` (TypeIndex 98,
    // uid 170; 3487, ours 897908736 against theirs 969932800) and `1/131`
    // (uid 169; 3492), 3567a's shape (parked 3580b). The closing block
    // 3674 agrees.
    pin_eq!(w.firsts.len(), 47, "run1237b baseline");
}
