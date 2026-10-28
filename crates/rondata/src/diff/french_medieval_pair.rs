//! **The French at a held Medieval age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, item 3717): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **French (10)** at **Hard (4)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 10, 4)`, is new: the French hold this map at 5 (the third pair),
//! East Indies at 4 (`french_enlightenment_pair`) and 5 (the third pair),
//! and Great Sahara at 4 (`french_word`) and 5 (`french_classical_pair`);
//! the battery fields no French. Captures: run1392 (a `DUMP_ALL` start)
//! and run1393 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §364.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const FRENCH_MEDIEVAL: Pair = Pair {
    name: "GreatLakesFrenchMedieval",
    map_style: 14,
    ai_tribe: 10,
    difficulty: 4,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1392-greatlakes-french-medieval-start.txt",
    long: (
        "gamelog-run1393-greatlakes-french-medieval-trace.txt",
        "rontrace-run1393.log",
    ),
    // The game ended itself on 4664 (closing block 4665).
    length: 4664,
    // Item 3717, first try: 3108 of 4664 by the count and the sequence —
    // ours 5 draws against 4, index 4 ours' `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` against none (`docs/AI.md` §364). Item 3735
    // closes it, 3108 → 4664, the game's last frame (§364.1): a turn
    // animation loops — every shipped turn slot names a `<LOOPING>` file,
    // the packed Trebuchet `1/98`'s its wagon's walk — so its
    // `CHAR_TURN_LEFT` restarts on 3108 with no idle roll.
    count: 4664,
    sequence: 4664,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const FRENCH_MEDIEVAL_PAIR_WORD: i64 =
    if FRENCH_MEDIEVAL.count < FRENCH_MEDIEVAL.sequence {
        FRENCH_MEDIEVAL.count
    } else {
        FRENCH_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn french_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &FRENCH_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "FRENCH_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2 and the difficulty at 4, and the long is
/// scoreable.
#[test]
fn french_medieval_pair_captures_say_their_lobby() {
    let p = &FRENCH_MEDIEVAL;
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
        "the French Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3717; the battery's identity
/// test refuses it), once, and none of the battery's — the battery fields
/// no French — and no other French lobby's: the third pair's
/// `(14, 10, 5)`, this map at another difficulty, and `(18, 10, 5)`, the
/// Enlightenment pair's `(18, 10, 4)`, the Classical pair's `(7, 10, 5)`
/// and the AI word `GreatSaharaFrenchTier4`'s `(7, 10, 4)`, other maps
/// (item 3717).
#[test]
fn french_medieval_pair_is_no_other_lobby() {
    let p = &FRENCH_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        battery::BATTERY.iter().all(|l| l.ai_tribe != p.ai_tribe),
        "the battery fields the French"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another French lobby on this map at this difficulty"
        );
    }
    let w = &super::french_word::FRENCH;
    let c = &super::french_classical_pair::FRENCH_CLASSICAL;
    let e = &super::french_enlightenment_pair::FRENCH_ENLIGHTENMENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
        (e.name, (e.map_style, e.ai_tribe, e.difficulty)),
        ("the third pair, East Indies", (18, 10, 5)),
        ("the third pair, Great Lakes", (14, 10, 5)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the French");
        assert_ne!(l, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word** (item 3717, from
/// 3108): only run1393's stride blocks are on disk before the word's
/// window, 101 through 3101.
pub(crate) const WIDENING_FRENCH_MEDIEVAL_STRIDE: (i64, i64) = (101, 3101);

pub(crate) fn french_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[FRENCH_MEDIEVAL.start],
        true,
        FRENCH_MEDIEVAL.long,
        "run1393 stride",
        &[(FRENCH_MEDIEVAL.long.0, WIDENING_FRENCH_MEDIEVAL_STRIDE.0)],
        WIDENING_FRENCH_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn french_medieval_pair_the_long_s_stride_walks_back_from_the_word() {
    let _pins = Pins::hold();
    let Some(w) = french_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 31, "every stride block in the window");
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
    // Item 3717: 84 keys over 31 blocks, every one a standing family —
    // `form` on 27 units and the buildings' `build:extra` on 45 (which the
    // stride does not print), and who=1's groups 64..69 with their pools
    // as each is minted. No unit row parts before the word's window.
    pin_eq!(w.firsts.len(), 84, "the stride's baseline");
}

/// run1393b: the word 3108's window, blocks 3077..3359 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — 24 blocks before the stride's
/// last block 3101 and 250 past the word's own block 3109 (item 3717).
/// run1393 prints units only every hundred frames, no `AMMO` and no
/// `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1393B: &str =
    "gamelog-run1393b-greatlakes-french-medieval-window-3077-3359.txt";

/// The window: block 3077 through 3359.
pub(crate) const WIDENING_FRENCH_MEDIEVAL_FRAME_3108: (i64, i64) = (3077, 3359);

/// **The word 3108's window**: run1393b walked from run1392's start with
/// the recorder on and every round compared (item 3717).
pub(crate) fn french_medieval_frame_3108_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[FRENCH_MEDIEVAL.start],
        true,
        FRENCH_MEDIEVAL.long,
        "run1393b",
        &[(RUN1393B, WIDENING_FRENCH_MEDIEVAL_FRAME_3108.0)],
        WIDENING_FRENCH_MEDIEVAL_FRAME_3108,
        1,
        &[3109],
        true,
    )
}

#[test]
fn french_medieval_pair_run1393b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_medieval_frame_3108_window() else {
        return;
    };
    pin_eq!(w.blocks, 282, "every captured block");
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
    // Item 3717, the first walk: **424** keys. Block 3077 carries the
    // standing families (49: `form` on 27 units, both leaders' rows, who=1's
    // groups and pools). **The word**: the Trebuchet `1/98` (TI 266, TY
    // 216, land, packed), on an `ATTACKTOORDER` to (36072, 26328) and held
    // in place by a re-path that alternates its move's `dest` between
    // (39144, 14040) and (40680, 12504) every frame on both sides, enters
    // `CHAR_TURN_LEFT` (slot 21) in ours on 3080; ours ends it on 3108
    // (`t29/30`) and resets its four figures to the idle with one
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271` draw that theirs does not
    // spend. The dump prints no `cur_anim` at `GUYS=2`, so the draw stream
    // is the record. The first unit row is its `path:length` on 3110, 14
    // against 18 (`path_recursion` 0 against 10); then the light cavalry
    // `1/90`'s and `1/79`'s (TI 210) guards (3114, 3116) and orders (3121,
    // 3124).
    // Item 3735 closes the word (§364.1): the turn loops, so `1/98`
    // restarts its `CHAR_TURN_LEFT` on 3108 with no draw, and **56** keys
    // remain. Block 3077's 49 standing rows; who=1's groups 66 (3155) and
    // 65 (3203) with their pools as each is minted, and who=1's
    // `SITE[4].reg` on 3176 (1 against 0), standing families too; and
    // **two rows that are not**: `dest_angle` on group 67's guards of
    // `1/83` — the General `1/78` (TI 54) on 3160, 392757248 against
    // 882769920, and the Peltasts `1/82` (TI 84) on 3164, 600702976 against
    // 693043200 — 3567a's shape (parked 3735a). No other unit, building or
    // city field parts, and no draw parts to the game's end.
    pin_eq!(w.firsts.len(), 56, "run1393b baseline");
    // Every round, both directions: no round flies in the window.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1414: the pair's closing window, blocks 4408..4665 at the long's
/// detail with `LEADERS=9` and `AMMO=5` (item 3735). The word reached the
/// game's last frame; run1393's stride prints units every hundred frames
/// and run1393b ends on 3359, so no dump held the last blocks.
pub(crate) const RUN1414: &str =
    "gamelog-run1414-greatlakes-french-medieval-closing-window-4408-4665.txt";

/// The window: block 4408 through the closing block 4665.
pub(crate) const WIDENING_FRENCH_MEDIEVAL_CLOSING: (i64, i64) = (4408, 4665);

/// **The pair's last blocks**: run1414 walked from run1392's start with
/// the recorder on and every round compared. The word is the game's last
/// frame (item 3735).
pub(crate) fn french_medieval_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[FRENCH_MEDIEVAL.start],
        true,
        FRENCH_MEDIEVAL.long,
        "run1414",
        &[(RUN1414, WIDENING_FRENCH_MEDIEVAL_CLOSING.0)],
        WIDENING_FRENCH_MEDIEVAL_CLOSING,
        1,
        &[4665],
        true,
    )
}

#[test]
fn french_medieval_pair_run1414_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = french_medieval_closing_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
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
    // Item 3735, the pair whole: **55** keys. Block 4408 carries the
    // standing families (`form` on 27 units, both leaders' `SITE[i].reg`,
    // who=0's `aggression[1]`, who=1's groups 64, 67, 68 and 70 with their
    // pools), and later blocks groups 66, 65 and 69's as each is minted
    // (4457, 4465, 4473). **Two rows are not standing families**:
    // `dest_angle` on group 64's guards of the Trebuchet `1/71` — the
    // Peltasts `1/35` (TI 84) on 4443, -2147483648 against -1571028992,
    // and the Crossbowmen `1/149` (TI 172) on 4453, -1716781056 against
    // -1696333824 — 3567a's shape (parked 3735a). No other unit, building
    // or city field parts, no draw parts to 4664, and the closing block
    // 4665 agrees.
    pin_eq!(w.firsts.len(), 55, "run1414 baseline");
    // Every round, both directions: 287 each side, 10 rows parting — the
    // Trebuchets `1/70`'s `ammo[1]` and `1/71`'s `ammo[0]` (TI 266) on
    // 4408, each round's `sx`, `sy`, `sz`, `angle` and `v1z`: they leave
    // from the figure's own point, `sz` 127 against 820 — 3711b's shape,
    // a piece with no `launch::BAYS` row (parked 3735c). The draws never
    // feel it.
    pin_eq!(w.rounds_read, (287, 287), "the rounds read");
    pin_eq!(w.rounds.len(), 10, "the rounds that part");
}

/// **The pair's closing state** (item 3717): run1393's closing block 4665,
/// the whole map's units, walked from run1392's start past the word 3108.
#[test]
fn french_medieval_pair_run1393_s_closing_state() {
    let _pins = Pins::hold();
    let p = &FRENCH_MEDIEVAL;
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
        "French Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4665, "the closing block");
    // Item 3717, first try: 1556 frames past the word 3108 the game had
    // parted everywhere — counts [186, 0, 5, 2, 164, 0, 34], who=1's
    // `1/188`..`1/192` extra in ours, the cities' `busy`, `gatherers` and
    // `space[i]` apart. **Item 3735 closes the pair** (§364.1): the
    // closing block 4665 compares 202 units with every count 0, nothing
    // torn, off, unlinked or extra, and no closing field apart — the
    // closing state is scored.
    pin_eq!(e.compared, 202, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
    pin_eq!(
        harness::compare(&built, &fin, 8).closing_fields(),
        "none".to_string(),
        "the closing fields"
    );
}
