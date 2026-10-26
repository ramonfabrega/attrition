//! **The British at a held Classical age, East Indies, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, item 3651): East Indies, `MAP_STYLE 18`,
//! human Nubians (4) against the **British (11)** at **Difficulty 4**,
//! seed 12345, `STARTING_RESOURCES 7`, the age held at Classical:
//! `STARTING_TECHNOLOGY 1`, `ENDING_TECHNOLOGY 1`. Its triple,
//! `(18, 11, 4)`, is new: the British lobbies are the headline pairs'
//! `(14|18|7, 11, 0|5)`, the old held-out map's `(9, 11, 0)` and
//! `british_enlightenment_pair`'s `(18, 11, 5)`, and the battery fields no
//! British. Captures: run1315 (a `DUMP_ALL` start) and run1316 (the draw
//! stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of
//! 100, walked from the start's initial state). The first try is in
//! `docs/AI.md` §342.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BRITISH_CLASSICAL: Pair = Pair {
    name: "EastIndiesBritishClassical",
    map_style: 18,
    ai_tribe: 11,
    difficulty: 4,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1315-eastindies-british-classical-start.txt",
    long: (
        "gamelog-run1316-eastindies-british-classical-trace.txt",
        "rontrace-run1316.log",
    ),
    // The game ended itself on 8980 (closing block 8981).
    length: 8980,
    // Item 3651, first try: frame 4567, by the count and the sequence —
    // ours 13 draws against 16, parting at index 1, ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x1ed` (`docs/AI.md` §342).
    count: 4567,
    sequence: 4567,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_CLASSICAL_PAIR_WORD: i64 =
    if BRITISH_CLASSICAL.count < BRITISH_CLASSICAL.sequence {
        BRITISH_CLASSICAL.count
    } else {
        BRITISH_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn british_classical_pair_captures_say_their_lobby() {
    let p = &BRITISH_CLASSICAL;
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
        "the British Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3651; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// British; and no other British lobby's: every other tribe-11 triple the
/// scored words hold is at difficulty 0 or 5 — the headline pairs, the old
/// held-out map and `british_enlightenment_pair`'s `(18, 11, 5)` (item
/// 3651).
#[test]
fn british_classical_pair_is_no_other_lobby() {
    let p = &BRITISH_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields the British"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.2 == 0 || l.2 == 5,
            "{l:?} is another British lobby at difficulty {}",
            l.2
        );
    }
    let other = &super::british_enlightenment_pair::BRITISH_ENLIGHTENMENT;
    assert_eq!(other.ai_tribe, p.ai_tribe);
    assert_ne!(
        (other.map_style, other.ai_tribe, other.difficulty),
        triple,
        "{} is this lobby",
        other.name
    );
}

/// **The long's stride, walked back from the word 4567** (item 3651):
/// only run1316's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_BRITISH_CLASSICAL_STRIDE: (i64, i64) = (101, 4501);

pub(crate) fn british_classical_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_CLASSICAL.start],
        true,
        BRITISH_CLASSICAL.long,
        "run1316 stride",
        &[(
            BRITISH_CLASSICAL.long.0,
            WIDENING_BRITISH_CLASSICAL_STRIDE.0,
        )],
        WIDENING_BRITISH_CLASSICAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn british_classical_pair_the_long_s_stride_walks_back_from_4567() {
    let _pins = Pins::hold();
    let Some(w) = british_classical_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 45, "every stride block in the window");
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
    // Item 3651: 186 keys. The standing families: 56 buildings'
    // `build:extra` (which the stride does not print), `form` on 71
    // units, who=1's groups' `held` and pools as each is minted. **The
    // first unit rows are on block 4401**: the human's Citizen `0/5` (TY
    // 0, TI 50) wounded in ours on 4398 alone (`damage_frame` 4398 against
    // 4399, `pos` (4370, 5540) against (4357, 5518)), and who=1's Galleys
    // `1/21` and `1/27` (TY 291, TI 341, sea) — `1/21`'s move order
    // `off_y` ours 24 theirs 744, `dest_y` 6936 against 6888. The gap
    // 4302..4400 is run1316c's (`docs/AI.md` §342).
    pin_eq!(w.firsts.len(), 186, "the stride's baseline");
}

/// run1316c: the stride's gap before its first unit row, blocks
/// 4295..4401 at the window's detail (item 3651): the stride agrees on
/// 4301 and parts on 4401, and nothing else dumps between.
pub(crate) const RUN1316C: &str =
    "gamelog-run1316c-eastindies-british-classical-window-4295-4401.txt";

pub(crate) const WIDENING_BRITISH_CLASSICAL_GAP: (i64, i64) = (4295, 4400);

pub(crate) fn british_classical_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_CLASSICAL.start],
        true,
        BRITISH_CLASSICAL.long,
        "run1316c",
        &[(RUN1316C, WIDENING_BRITISH_CLASSICAL_GAP.0)],
        WIDENING_BRITISH_CLASSICAL_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn british_classical_pair_run1316c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_classical_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 106, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3651, the first walk: 140 keys. Block 4295 carries the
    // standing families (both leaders' `SITE[·].reg` 65 against 0 and
    // `aggression[1]`, `form`, `build:extra`, who=1's pools). **The
    // records first part on 4328**, after the rounds below (4298): the
    // human's building `0/2004` takes a blow of its own in each,
    // `build:damage` ours 90 theirs 87,
    // `damage_frac` 3 against 14, and its city `0/2000`'s `raid_stamp`
    // 4327 against 4321; on 4399 a round wounds the Citizen `0/5` (TI
    // 50) in ours alone (`hits_left` 48 against 50), and it flees.
    pin_eq!(w.firsts.len(), 140, "run1316c baseline");
    // Every round, both directions: 167 round-blocks theirs, 164 ours,
    // 26 parting. **The first is on 4298, who=1's Galley `1/29`** (TY 291,
    // TI 341) at `0/2004`: theirs launches from (2690, 3917) at `sz` 52,
    // ours from the hull's square (2472, 4200) at 0, so the end point
    // parts, (4241, 5678) against (4258, 5659) — the Galley has no
    // `launch::BAYS` row (`docs/AI.md` §342).
    pin_eq!(w.rounds_read, (167, 164), "the rounds read");
    pin_eq!(w.rounds.len(), 26, "the rounds that part");
}

/// run1316b: the word 4567's window, blocks 4561..4817 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 4568 and 250 after (item 3651); the stride's blocks are 4501 and
/// 4601.
pub(crate) const RUN1316B: &str =
    "gamelog-run1316b-eastindies-british-classical-window-4561-4817.txt";

/// The window: block 4561 through 4817; the word 4567's own block is 4568.
pub(crate) const WIDENING_BRITISH_CLASSICAL_FRAME_4567: (i64, i64) = (4561, 4817);

/// **The word 4567's window**: run1316b walked from run1315's start with
/// the recorder on and every round compared (item 3651).
pub(crate) fn british_classical_frame_4567_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_CLASSICAL.start],
        true,
        BRITISH_CLASSICAL.long,
        "run1316b",
        &[(RUN1316B, WIDENING_BRITISH_CLASSICAL_FRAME_4567.0)],
        WIDENING_BRITISH_CLASSICAL_FRAME_4567,
        1,
        &[4568],
        true,
    )
}

#[test]
fn british_classical_pair_run1316b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_classical_frame_4567_window() else {
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
    // Item 3651, the first walk: 959 keys. Block 4561 carries the
    // standing families and the Galleys' parted state since 4401: `1/21`
    // (TI 341) at (3384, 7512) against (3384, 7464), and its target, the
    // human's `0/10`, at (5200, 8160) against (5200, 8168). On the word's
    // block 4568 `1/21` is in range of `0/10` and recharging 40 in
    // theirs, while ours has lost it and turned to `0/3` (`in_range` 0
    // against 1, `new_ord` 1 against 0): theirs' 4567 draws a shot ours
    // does not — ours 13 draws against 16, index 1, ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Guy::inc_time+0x1ed` (`docs/AI.md` §342).
    pin_eq!(w.firsts.len(), 959, "run1316b baseline");
    // Every round, both directions: 809 round-blocks theirs, 899 ours,
    // 145 parting — the Galleys' rounds from the hull's square in ours.
    pin_eq!(w.rounds_read, (809, 899), "the rounds read");
    pin_eq!(w.rounds.len(), 145, "the rounds that part");
}
