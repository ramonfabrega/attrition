//! **The Romans at a held Classical age, Great Sahara** (DECISIONS 90, 91,
//! 92, item 3556): the land map's first pair below age 2 — Great Sahara, `MAP_STYLE
//! 7`, human Nubians (4) against the **Romans (6)** at Difficulty 5, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Classical:
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`. Captures: run1193 (a
//! `DUMP_ALL` start) and run1194 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §309.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const ROMAN_CLASSICAL: Pair = Pair {
    name: "GreatSaharaRomanClassical",
    map_style: 7,
    ai_tribe: 6,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1193-greatsahara-roman-classical-start.txt",
    long: (
        "gamelog-run1194-greatsahara-roman-classical-trace.txt",
        "rontrace-run1194.log",
    ),
    // The game ended itself on 7130 (closing block 7131).
    length: 7130,
    // Item 3556, first try: frame 4069 by the count and the sequence, ours
    // 16 draws against 21, index 1 — ours `Guy::set_anim+0xf2f <
    // Guy::move+0x166`, theirs `Unit::close+0xcb6` (`docs/AI.md` section
    // 309). **Item 3567: whole at 7130**, the game's last frame, 4069 →
    // 7130: a unit struck while moving in contact takes the blow doubled
    // (`unit_masks & 0x10`, `sim::Unit::contact`). The Citizen `0/13`
    // walking to its gather took the Light Horse `1/42`'s blow on 4068 at
    // 14 in theirs (`damage` 32 → 46, `unit_masks` 12 → 28) and 7 in ours
    // (32 → 39), so the javelin on 4069 killed it in theirs only.
    count: 7130,
    sequence: 7130,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const ROMAN_CLASSICAL_PAIR_WORD: i64 =
    if ROMAN_CLASSICAL.count < ROMAN_CLASSICAL.sequence {
        ROMAN_CLASSICAL.count
    } else {
        ROMAN_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn roman_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &ROMAN_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "ROMAN_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn roman_classical_pair_captures_say_their_lobby() {
    let p = &ROMAN_CLASSICAL;
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
        "the Roman Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(7, 6, 5)` is no held-out game's — the battery's
/// Romans are `GreatLakesRomanTier2`, `(14, 6, 2)` — and no other pinned
/// pair's. Both `GAME INFO`s are read: the battery's Roman game against
/// this pair's held age (3513's shape, Loop 3544).
#[test]
fn roman_classical_pair_is_no_other_lobby() {
    let p = &ROMAN_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let romans: Vec<&battery::Lobby> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .collect();
    assert_eq!(
        romans.iter().map(|l| l.name).collect::<Vec<_>>(),
        ["GreatLakesRomanTier2"]
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
    let (Some(held_out), Some(ours)) = (info(romans[0].long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(held_out.0, Some(14), "the held-out Roman lobby's map");
    assert_eq!(held_out.1, Some(2), "the held-out Roman lobby's difficulty");
    assert_eq!(
        ours,
        (Some(7), Some(5), Some(1), Some(1)),
        "this pair's lobby"
    );
}

/// **The pair's closing state**: run1194's closing dump, block 7131,
/// walked to and compared whole — a draw stream that agrees to a game's end
/// says nothing of the end itself (item 1099; item 3567).
#[test]
fn run1194_roman_classical_pair_closing_state() {
    let _pins = Pins::hold();
    let p = &ROMAN_CLASSICAL;
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
        "Roman pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 7131, "the closing block");
    // Item 3567, the pair whole: 268 units compared, none off or
    // unlinked, no building or city field-row; one torn, `1/40`
    // (printed, not pinned: `endpoint`'s module note).
    pin_eq!(e.compared, 268, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}

/// run1194b: the word 4069's window, blocks 4063..4319 at the long's
/// detail — six blocks before the word's block 4070 and 250 after (item
/// 3556); the stride's blocks are 4001 and 4101.
pub(crate) const RUN1194B: &str =
    "gamelog-run1194b-greatsahara-roman-classical-window-4063-4319.txt";

/// The window: block 4063 through 4319; the word 4069's own block is 4070.
pub(crate) const WIDENING_ROMAN_CLASSICAL_FRAME_4069: (i64, i64) = (4063, 4319);

/// **The word 4069's window**: run1194b walked from run1193's start with
/// the recorder on (item 3556).
pub(crate) fn roman_classical_frame_4069_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_CLASSICAL.start],
        true,
        ROMAN_CLASSICAL.long,
        "run1194b",
        &[(RUN1194B, WIDENING_ROMAN_CLASSICAL_FRAME_4069.0)],
        WIDENING_ROMAN_CLASSICAL_FRAME_4069,
        1,
        &[4070],
        true,
    )
}

#[test]
fn roman_classical_pair_run1194b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_classical_frame_4069_window() else {
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
    // Item 3556, the first walk: 2260 keys. Block 4063 carries 43 rows:
    // the standing families (`form`, `SITE[i].reg`, who=1's groups' `held`
    // and `pool`) and who=0's `aggression[1]` (ours 0, theirs 1). On 4068
    // the Light Horse `1/28` and `1/42` (TypeIndex 209, land) both take
    // the Nubian Citizen `0/13` (TypeIndex 50, land) and each draws
    // `Unit::fight+0x824` and `+0x9b0` on both sides; theirs' Citizen goes
    // `damage` 32 -> 46 (two blows of 7), ours' 32 -> 39, so block 4069
    // reads `hits_left` ours 11 theirs 4 and `hits:damage` 39 against 46.
    // On the word 4069 theirs' Citizen dies (`Unit::close+0xcb6 <
    // Object::die`, index 1 of theirs' 21 draws against ours' 16), and
    // block 4070 reads who=0's `num_units[0]` and `peasants` ours 5 theirs
    // 4 and the three Light Horse `1/122`, `1/123`, `1/127` on a fresh
    // order in theirs (`order:kind` ours 1 theirs 10) (`docs/AI.md`
    // section 309).
    //
    // Item 3567: 2260 → 48 keys, the `contact` bit read (`unit_masks &
    // 0x10`). Block 4069 now reads the Citizen at `damage` 46 and
    // `contact` 1 on both sides, and the word's death is both sides'.
    // What is left: block 4063's 43 standing rows, the standing `pool`
    // and `held` of who=1's groups 68 (4110) and 71 (4163), and the Light
    // Horse `1/28`'s `dest_angle` on 4079 — ours 137756672, theirs
    // -1351155712, ours a frame late at its guard post, agreeing from
    // 4080 (parked 3567a).
    pin_eq!(w.firsts.len(), 48, "run1194b baseline");
}

/// run1210: the pair's last blocks, 6875..7131 at the window's detail —
/// the word 7130 is the game's last frame and 7131 its closing block (item
/// 3567). run1194 prints units only every hundredth block, and run1194b
/// ends on 4319, so no dump held the last blocks.
pub(crate) const RUN1210: &str = "gamelog-run1210-greatsahara-roman-classical-window-6875-7131.txt";

/// The window: block 6875 through the closing block 7131.
pub(crate) const WIDENING_ROMAN_CLASSICAL_CLOSING: (i64, i64) = (6875, 7131);

/// **The pair's last blocks**: run1210 walked from run1193's start with
/// the recorder on. The word is the game's last frame (item 3567).
pub(crate) fn roman_classical_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_CLASSICAL.start],
        true,
        ROMAN_CLASSICAL.long,
        "run1210",
        &[(RUN1210, WIDENING_ROMAN_CLASSICAL_CLOSING.0)],
        WIDENING_ROMAN_CLASSICAL_CLOSING,
        1,
        &[7131],
        true,
    )
}

#[test]
fn roman_classical_pair_run1210_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_classical_closing_window() else {
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
    // Item 3567, the first walk: **46** keys. Block 6875 carries the
    // standing families alone (`SITE[i].reg`, who=0's `aggression[1]`,
    // `form`, who=1's groups' `held`/pool), and later blocks groups 71
    // (6943) and 66 (7065)'s `held`/pool. **One row is not a standing
    // family**: on block 6931 the `1/195`'s `dest_angle` reads ours
    // -610729984 against theirs -620363776, 3567a's row again (parked
    // 3567a). The closing block 7131 agrees.
    pin_eq!(w.firsts.len(), 46, "run1210 baseline");
}
