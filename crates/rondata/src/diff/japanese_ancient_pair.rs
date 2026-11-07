//! **The Japanese from an Ancient start, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3602): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Japanese (15)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, `STARTING_TECHNOLOGY 0` and `ENDING_TECHNOLOGY
//! 1` (read back 1; age 0 cannot be held, 3559a) — the Maya's
//! Ancient-start lobby with the nation moved. Its triple, `(18, 15, 5)`, is
//! the Japanese Industrial pair's too (`japanese_industrial_pair`, a held
//! Industrial age); the two are told apart by their `GAME INFO` ages. The
//! Japanese's other lobbies are `japanese_gunpowder_pair`'s `(7, 15, 5)`
//! and `japanese_word`'s `GreatSaharaJapaneseTier1`, `(7, 15, 1)`.
//! Captures: run1251 (a `DUMP_ALL` start) and run1252 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §323.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_ANCIENT: Pair = Pair {
    name: "EastIndiesJapaneseAncient",
    map_style: 18,
    ai_tribe: 15,
    difficulty: 5,
    starting_technology: 0,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1251-eastindies-japanese-ancient-start.txt",
    long: (
        "gamelog-run1252-eastindies-japanese-ancient-trace.txt",
        "rontrace-run1252.log",
    ),
    // The game ended itself on 6126 (closing block 6127).
    length: 6126,
    // Item 3602, first try: frame 4934 by the count and the sequence, ours
    // 180 draws against theirs' 179, parting at index 0 — ours
    // `Guy::set_anim+0x97a < Guy::move+0x19f`, theirs
    // `Unit::think_fish+0x27a` (`docs/AI.md` section 323). Item 3610: 4934
    // → 6126, the game's last frame — whole. The Bowmen loose from piece
    // 1176's bays (`sim::launch`), so `1/50`'s arrow at the Citizen `0/6`
    // leaves (7529, 5749) and flies 16 frames, where it left the unit's
    // own square (7608, 5784) and flew 17.
    count: 6126,
    sequence: 6126,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_ANCIENT_PAIR_WORD: i64 =
    if JAPANESE_ANCIENT.count < JAPANESE_ANCIENT.sequence {
        JAPANESE_ANCIENT.count
    } else {
        JAPANESE_ANCIENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_ANCIENT {}: count {} sequence {} of {}; {}",
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
fn japanese_ancient_pair_captures_say_their_lobby() {
    let p = &JAPANESE_ANCIENT;
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
        "the Japanese Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery fields no Japanese game. Its triple
/// `(18, 15, 5)` was scored by the Japanese Industrial pair (item 3510) and
/// is not minted again; the two lobbies part on the age, so both `GAME
/// INFO`s are read (3513's shape, Loop 3544). The Gunpowder pair's `(7, 15,
/// 5)` and the AI word's `(7, 15, 1)` are another map (item 3602).
#[test]
fn japanese_ancient_pair_is_no_other_lobby() {
    let p = &JAPANESE_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(!battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe));
    for (name, other) in [
        (
            super::japanese_word::JAPANESE.name,
            (
                super::japanese_word::JAPANESE.map_style,
                super::japanese_word::JAPANESE.ai_tribe,
                super::japanese_word::JAPANESE.difficulty,
            ),
        ),
        (
            super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER.name,
            (
                super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER.map_style,
                super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER.ai_tribe,
                super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
    let industrial = &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL;
    assert_eq!(
        (
            industrial.map_style,
            industrial.ai_tribe,
            industrial.difficulty
        ),
        triple,
        "the Industrial pair shares the triple"
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
    let (Some(theirs), Some(ours)) = (info(industrial.long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(
        theirs,
        (Some(18), Some(5), Some(5), Some(5)),
        "the Japanese Industrial pair's lobby"
    );
    assert_eq!(
        ours,
        (Some(18), Some(5), Some(0), Some(1)),
        "this pair's lobby"
    );
}

/// run1252b: the word 4934's window, blocks 4928..5184 at the long's
/// detail with `AMMO=5` — six blocks before the word's block 4935 and 250
/// after (item 3602); the long's stride prints units every hundredth block
/// (4901 and 5001) and no `LEADERS` past the start.
pub(crate) const RUN1252B: &str =
    "gamelog-run1252b-eastindies-japanese-ancient-window-4928-5184.txt";

/// The window: block 4928 through 5184; the word 4934's own block is 4935.
pub(crate) const WIDENING_JAPANESE_ANCIENT_FRAME_4934: (i64, i64) = (4928, 5184);

/// **The word 4934's window**: run1252b walked from run1251's start with
/// the recorder on (item 3602).
pub(crate) fn japanese_ancient_frame_4934_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_ANCIENT.start],
        true,
        JAPANESE_ANCIENT.long,
        "run1252b",
        &[(RUN1252B, WIDENING_JAPANESE_ANCIENT_FRAME_4934.0)],
        WIDENING_JAPANESE_ANCIENT_FRAME_4934,
        1,
        &[4935],
        true,
    )
}

#[test]
fn japanese_ancient_pair_run1252b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_ancient_frame_4934_window() else {
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
    // Item 3602, the first walk: 1962 keys. The window opens apart on
    // 4928, past the stride's agreeing 4901: beside the standing families
    // (every unit's `form`, who=0's `SITE[i].reg`, `aggression[1]`, who=1's
    // groups' `held` and pools), the Bowmen `1/51` (TypeIndex 170, land,
    // who=1's group 64) moves to (7416, 6024) in ours against (6984, 5832)
    // in theirs (`order:move.off_x/off_y` 504/648 against 72/456), and the
    // Citizen `0/6` (TypeIndex 50) took its wound on 4905 in ours against
    // 4904 in theirs. On 4933 `1/51`'s current order is ours 10 against
    // theirs 1 (`orders.len` 2 against 3) and the Bowmen `1/29`'s move
    // goes to (4488, 4968) against (4248, 5112); on the word's block 4935
    // `1/51` takes the extra `Guy::set_anim < Guy::move` draw in ours.
    // Item 3610: 1962 → 82, the standing families alone, with piece
    // 1176's bays: `1/50`'s arrow lands on `0/6` on 4904 on both sides.
    pin_eq!(w.firsts.len(), 82, "run1252b baseline");
}

/// **The long's stride, walked back from the word 4934** (item 3602):
/// run1252b's window opens apart on 4928, and only run1252's stride blocks
/// are on disk before it.
pub(crate) const WIDENING_JAPANESE_ANCIENT_STRIDE: (i64, i64) = (101, 4901);

pub(crate) fn japanese_ancient_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_ANCIENT.start],
        true,
        JAPANESE_ANCIENT.long,
        "run1252 stride",
        &[(JAPANESE_ANCIENT.long.0, WIDENING_JAPANESE_ANCIENT_STRIDE.0)],
        WIDENING_JAPANESE_ANCIENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_ancient_pair_the_long_s_stride_walks_back_from_4934() {
    let _pins = Pins::hold();
    let Some(w) = japanese_ancient_stride_window() else {
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
    // Item 3602: 134 keys, the standing families alone — every unit's
    // `form`, and who=1's groups' `held` and pools, which the stride does
    // not print — from 101 through 4901: no unit field parts on any stride
    // block, so the state run1252b opens apart on 4928 parts after 4901.
    pin_eq!(w.firsts.len(), 134, "the stride's baseline");
}

/// run1264: the gap's window, blocks 4895..4940 at run1252b's detail with
/// `AMMO=5` (item 3610) — the stride agrees through 4901 and run1252b
/// opens apart on 4928, so no dump held 4902..4927.
pub(crate) const RUN1264: &str = "gamelog-run1264-eastindies-japanese-ancient-window-4895-4940.txt";

/// The window: block 4895 through 4940; the word 4934's own block is 4935.
pub(crate) const WIDENING_JAPANESE_ANCIENT_GAP: (i64, i64) = (4895, 4940);

/// **The gap's window**: run1264 walked from run1251's start with the
/// recorder on (item 3610).
pub(crate) fn japanese_ancient_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_ANCIENT.start],
        true,
        JAPANESE_ANCIENT.long,
        "run1264",
        &[(RUN1264, WIDENING_JAPANESE_ANCIENT_GAP.0)],
        WIDENING_JAPANESE_ANCIENT_GAP,
        1,
        &[4935],
        true,
    )
}

#[test]
fn japanese_ancient_pair_run1264_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_ancient_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 45, "every captured block");
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
    // Item 3610, before piece 1176's bays: 153 keys, the first unit field
    // on block 4905 — the Citizen `0/6`'s wound (`hits_left` 40 against
    // 37, `damage_frame` 0 against 4904), struck by the Bowmen `1/50`'s
    // arrow, which left (7608, 5784) in 17 frames in ours and (7529, 5749)
    // in 16 in theirs; then `0/6`'s flight (order 7 against 4) and, on
    // 4926, `1/51`'s re-aim. With the bays: 79, the standing families
    // alone (every unit's `form`, who=0's `SITE[i].reg`, `aggression[1]`,
    // who=1's groups' `held`).
    pin_eq!(w.firsts.len(), 79, "run1264 baseline");
}

/// **The closing state** (item 3610): run1252's closing block 6127, the
/// whole map's units, walked from run1251's start.
#[test]
fn japanese_ancient_pair_run1252_s_closing_state() {
    let _pins = Pins::hold();
    let p = &JAPANESE_ANCIENT;
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
        "Japanese Ancient pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 6127, "the closing block");
    pin_eq!(e.compared, 169, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1264c: the closing window, blocks 5871..6127 at the long's detail
/// with `LEADERS=9` (item 3610) — run1252 prints units every hundredth
/// block and no leader, building or city record past the start, so no
/// dump held the last blocks.
pub(crate) const RUN1264C: &str =
    "gamelog-run1264c-eastindies-japanese-ancient-window-5871-6127.txt";

/// The closing window: block 5871 through the closing block 6127.
pub(crate) const WINDOW_JAPANESE_ANCIENT_CLOSING: (i64, i64) = (5871, 6127);

/// **The closing window**: run1264c walked from run1251's start with the
/// recorder on (item 3610).
pub(crate) fn japanese_ancient_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_ANCIENT.start],
        true,
        JAPANESE_ANCIENT.long,
        "run1264c",
        &[(RUN1264C, WINDOW_JAPANESE_ANCIENT_CLOSING.0)],
        WINDOW_JAPANESE_ANCIENT_CLOSING,
        1,
        &[6127],
        true,
    )
}

#[test]
fn japanese_ancient_pair_run1264c_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_ancient_closing_window() else {
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
    // Item 3610: 59 keys, the standing families alone, through the
    // closing block 6127.
    pin_eq!(w.firsts.len(), 59, "run1264c baseline");
}
