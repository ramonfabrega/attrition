//! **The Turks at a held Enlightenment age, Great Sahara** (DECISIONS 90,
//! item 3513): the land map's held Enlightenment pair. The Chinese pair's
//! lobby (Great Sahara, `MAP_STYLE 7`, human Nubians (4), Difficulty 3,
//! seed 12345, `STARTING_RESOURCES 7`) with the nation varied to the
//! **Turks (8)** and the age held at Enlightenment: `STARTING_TECHNOLOGY
//! 4`, `ENDING_TECHNOLOGY 4`. Captures: run1158 (a `DUMP_ALL` start) and
//! run1159 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §307.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const TURKISH_ENLIGHTENMENT: Pair = Pair {
    name: "GreatSaharaTurkishEnlightenment",
    map_style: 7,
    ai_tribe: 8,
    difficulty: 3,
    starting_technology: 4,
    ending_technology: 4,
    starting_resources: 7,
    start: "gamelog-run1158-greatsahara-turkish-enlightenment-start.txt",
    long: (
        "gamelog-run1159-greatsahara-turkish-enlightenment-trace.txt",
        "rontrace-run1159.log",
    ),
    // The game ended itself on 3812 (closing block 3813).
    length: 3812,
    // Item 3513, first try: frame 2501 by the count and the sequence, ours
    // 12 draws against 13, index 0 — ours `Guy::set_anim+0x97a <
    // Guy::do_turn+0x4a < Unit::move_step+0x3b6`, theirs `Guy::set_anim+0x97a
    // < Unit::do_idle+0x7d`: ours turns a unit on a move where theirs idles
    // it (`docs/AI.md` section 307).
    count: 2501,
    sequence: 2501,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const TURKISH_ENLIGHTENMENT_PAIR_WORD: i64 =
    if TURKISH_ENLIGHTENMENT.count < TURKISH_ENLIGHTENMENT.sequence {
        TURKISH_ENLIGHTENMENT.count
    } else {
        TURKISH_ENLIGHTENMENT.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn turkish_enlightenment_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &TURKISH_ENLIGHTENMENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "TURKISH_ENLIGHTENMENT {}: count {} sequence {} of {}; {}",
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
fn turkish_enlightenment_pair_captures_say_their_lobby() {
    let p = &TURKISH_ENLIGHTENMENT;
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
        "the Turkish pair's long is not scoreable"
    );
}

/// The pair shares its `(map_style, ai_tribe, difficulty)` with the
/// held-out `GreatSaharaTurkishTier3` (item 3385), so it stays out of
/// `battery::SCORED_LOBBIES`, whose triples the battery's own identity test
/// refuses. The two are different games: that lobby plays every age from
/// the first (`STARTING_TECHNOLOGY 0`, `ENDING_TECHNOLOGY 7`) on
/// `STARTING_RESOURCES 1`, and this one holds Enlightenment on 7 — read
/// here off both captures' own `GAME INFO` (item 3513).
#[test]
fn turkish_enlightenment_pair_is_no_other_lobby() {
    let p = &TURKISH_ENLIGHTENMENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(!battery::SCORED_LOBBIES.contains(&triple));
    let same: Vec<&battery::Lobby> = battery::BATTERY
        .iter()
        .filter(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
        .collect();
    assert_eq!(
        same.iter().map(|l| l.name).collect::<Vec<_>>(),
        ["GreatSaharaTurkishTier3"]
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
            get("STARTING_TECHNOLOGY"),
            get("ENDING_TECHNOLOGY"),
            get("STARTING_RESOURCES"),
        ))
    };
    let (Some(held_out), Some(ours)) = (info(same[0].long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(held_out, (Some(0), Some(7), Some(1)), "the held-out lobby");
    assert_eq!(ours, (Some(4), Some(4), Some(7)), "this pair's lobby");
}

/// run1159b: the word 2501's window, blocks 2495..2751 at the long's
/// detail — six blocks before the word's block 2502 and 250 after (item
/// 3513).
pub(crate) const RUN1159B: &str =
    "gamelog-run1159b-greatsahara-turkish-enlightenment-window-2495-2751.txt";

/// The window: block 2495 through 2751; the word 2501's own block is 2502.
pub(crate) const WIDENING_TURKISH_ENLIGHTENMENT_FRAME_2501: (i64, i64) = (2495, 2751);

/// **The word 2501's window**: run1159b walked from run1158's start with
/// the recorder on (item 3513).
pub(crate) fn turkish_enlightenment_frame_2501_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[TURKISH_ENLIGHTENMENT.start],
        true,
        TURKISH_ENLIGHTENMENT.long,
        "run1159b",
        &[(RUN1159B, WIDENING_TURKISH_ENLIGHTENMENT_FRAME_2501.0)],
        WIDENING_TURKISH_ENLIGHTENMENT_FRAME_2501,
        1,
        &[2502],
        true,
    )
}

#[test]
fn turkish_enlightenment_pair_run1159b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = turkish_enlightenment_frame_2501_window() else {
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
    // Item 3513, the first walk: **2651** keys. Block 2495 carries 47
    // standing rows (the ten `SITE[i].reg`, `aggression[1]`, every unit's
    // `form`, who=1's groups 64–66 ours alone) and the Village `1/2020`
    // (TypeIndex 414) three frames behind: `construct_hits` ours 1181
    // against theirs 1187, `job_counter` ours 59100 against 59400 of
    // 60000. **The word's own rows**: on block 2501 theirs completes it —
    // `build:city` ours -1 against theirs 2, who=1's `city_num` 2 against
    // 3 — and its builder, the Citizen `1/20` (TypeIndex 50, land), drops
    // its `BUILDORDER` (`orders.len` ours 1, theirs 0); on 2502 it is
    // `idle` in theirs (0 against 1), which is theirs' extra
    // `Unit::do_idle+0x7d` draw. Walked back: theirs' `frame_started` is
    // 1901 and its clock 100 a frame from it; ours' first work is on 1904
    // (`RON_DEBUG_BUILDS`), when ours' `1/20` reaches the site at (37224,
    // 3912), three frames after theirs (block 1901). The Citizen agrees on
    // the long's block 1401 at (40885, 14504) and has parted by 1501, ours
    // (41596, 12371) against theirs (41524, 12335) (`docs/AI.md` section
    // 307). The 440 keys of block 2555 are past the word.
    pin_eq!(w.firsts.len(), 2651, "run1159b baseline");
}

/// run1186: the walk back from the word 2501, blocks 1395..1502 at
/// run1159b's detail (item 3543) — the long's stride has the Citizen
/// `1/20` agreeing on 1401 and parted by 1501.
pub(crate) const RUN1186: &str =
    "gamelog-run1186-greatsahara-turkish-enlightenment-window-1395-1502.txt";

/// The window: block 1395 through 1502.
pub(crate) const WIDENING_TURKISH_ENLIGHTENMENT_GAP_1186: (i64, i64) = (1395, 1502);

pub(crate) fn turkish_enlightenment_gap_1186_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[TURKISH_ENLIGHTENMENT.start],
        true,
        TURKISH_ENLIGHTENMENT.long,
        "run1186",
        &[(RUN1186, WIDENING_TURKISH_ENLIGHTENMENT_GAP_1186.0)],
        WIDENING_TURKISH_ENLIGHTENMENT_GAP_1186,
        1,
        &[],
        true,
    )
}

#[test]
fn turkish_enlightenment_pair_run1186_s_walk_back_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = turkish_enlightenment_gap_1186_window() else {
        return;
    };
    pin_eq!(w.blocks, 107, "every captured block");
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
    pin_eq!(w.firsts.len(), 61, "run1186 baseline");
}
