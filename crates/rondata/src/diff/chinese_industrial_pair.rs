//! **The Chinese at a held Industrial age, Great Sahara** (DECISIONS 90,
//! item 3511): the land map's held Industrial pair. The American pair's
//! lobby (Great Sahara, `MAP_STYLE 7`, human Nubians (4), Difficulty 3,
//! seed 12345, `STARTING_RESOURCES 7`) with the nation varied to the
//! **Chinese (14)** and the age held at Industrial: `STARTING_TECHNOLOGY
//! 5`, `ENDING_TECHNOLOGY 5`. Captures: run1154 (a `DUMP_ALL` start) and
//! run1155 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §305.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const CHINESE_INDUSTRIAL: Pair = Pair {
    name: "GreatSaharaChineseIndustrial",
    map_style: 7,
    ai_tribe: 14,
    difficulty: 3,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1154-greatsahara-chinese-industrial-start.txt",
    long: (
        "gamelog-run1155-greatsahara-chinese-industrial-trace.txt",
        "rontrace-run1155.log",
    ),
    // The game ended itself on 4467 (closing block 4468).
    length: 4467,
    // Item 3511, first try: frame 3007 by the count and the sequence, ours
    // 27 draws against 30, index 12 — ours `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271`, theirs `Ammo::init+0xcd9`: theirs fires a
    // round ours does not (`docs/AI.md` section 305).
    // Item 3511: 3007 -> 3017, the Armored Car's (piece 166) turret rows
    // built (`sim::pivot`): `1/23`'s round of 3001 leaves from (9569,
    // 31152, 750) and flies 6, as theirs' does (run1155c's packet). Ours
    // 19 draws against 17, index 0 — ours `Unit::fight+0x9b0`, theirs
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x3b6`.
    count: 3017,
    sequence: 3017,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const CHINESE_INDUSTRIAL_PAIR_WORD: i64 =
    if CHINESE_INDUSTRIAL.count < CHINESE_INDUSTRIAL.sequence {
        CHINESE_INDUSTRIAL.count
    } else {
        CHINESE_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn chinese_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &CHINESE_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "CHINESE_INDUSTRIAL {}: count {} sequence {} of {}; {}",
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
fn chinese_industrial_pair_captures_say_their_lobby() {
    let p = &CHINESE_INDUSTRIAL;
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
        "the Chinese pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn chinese_industrial_pair_is_no_other_lobby() {
    let p = &CHINESE_INDUSTRIAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1155b: the word 3007's window, blocks 3001..3257 at the long's
/// detail — six blocks before the word's block 3008 and 250 after (item
/// 3511).
pub(crate) const RUN1155B: &str =
    "gamelog-run1155b-greatsahara-chinese-industrial-window-3001-3257.txt";

/// The window: block 3001 through 3257; the word 3007's own block is 3008,
/// and the word 3017's (item 3511) is 3018.
pub(crate) const WIDENING_CHINESE_INDUSTRIAL_FRAME_3007: (i64, i64) = (3001, 3257);

/// **The word 3007's window**: run1155b walked from run1154's start with
/// the recorder on (item 3511).
pub(crate) fn chinese_industrial_frame_3007_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[CHINESE_INDUSTRIAL.start],
        true,
        CHINESE_INDUSTRIAL.long,
        "run1155b",
        &[(RUN1155B, WIDENING_CHINESE_INDUSTRIAL_FRAME_3007.0)],
        WIDENING_CHINESE_INDUSTRIAL_FRAME_3007,
        1,
        &[3008],
        true,
    )
}

#[test]
fn chinese_industrial_pair_run1155b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = chinese_industrial_frame_3007_window() else {
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
    // Item 3511, the first walk: 1121 keys. Block 3001 carries 50
    // standing rows — `SITE[i].reg`, `aggression[1]`, `form`, who=1's
    // groups' `held` and pools — and city `0/2000`'s `raid_stamp`, ours
    // 2996 against theirs 2995; on 3002 the human's Citizen `0/3`
    // (TypeIndex 50, land) reads `hits_left` ours 52 against theirs 50,
    // and agrees again on 3003: **ours' blow lands a frame late**. The
    // shooters are who=1's Armored Cars (TypeIndex 216, land, piece 166);
    // ours' rounds left from the car's square, theirs' from its turret
    // (run1155c's packet: `1/23`'s round of 3001 from (9569, 31152) against
    // ours' (9617, 31210), 6 frames against 7). On the word's block 3008
    // nothing parts first: theirs launches two rounds on 3007 and ours one.
    //
    // Then **1026** with piece 166's rows: block 3001 holds only the 49
    // standing rows, and 3002 agrees. The word's block 3018 is in the
    // window: on 3017 `1/23`'s `cavarch_o` and `g.ox[0]` read ours 11
    // against theirs 3 — ours' new attack-walk cycle of 3016 is on the
    // human's Citizen `0/11`, theirs' on `0/3` — and `recharging` ours 0
    // against theirs 30 (`docs/AI.md` section 305).
    pin_eq!(w.firsts.len(), 1026, "run1155b baseline");
}
