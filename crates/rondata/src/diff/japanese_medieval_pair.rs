//! **The Japanese at a held Medieval age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, item 3664): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Japanese (15)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 15, 5)`, is new: the Japanese lobbies are the Industrial and
//! Ancient pairs' `(18, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and the
//! AI word `GreatSaharaJapaneseTier1`'s `(7, 15, 1)`, and the battery
//! fields no Japanese. Captures: run1328 (a `DUMP_ALL` start) and run1329
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on
//! a stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §344.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_MEDIEVAL: Pair = Pair {
    name: "GreatLakesJapaneseMedieval",
    map_style: 14,
    ai_tribe: 15,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1328-greatlakes-japanese-medieval-start.txt",
    long: (
        "gamelog-run1329-greatlakes-japanese-medieval-trace.txt",
        "rontrace-run1329.log",
    ),
    // The game ended itself on 5140 (closing block 5141).
    length: 5140,
    // Item 3664, first try: whole — frame 5140 of 5140 by the count and
    // the sequence (`docs/AI.md` §344).
    count: 5140,
    sequence: 5140,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_MEDIEVAL_PAIR_WORD: i64 =
    if JAPANESE_MEDIEVAL.count < JAPANESE_MEDIEVAL.sequence {
        JAPANESE_MEDIEVAL.count
    } else {
        JAPANESE_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 1, and the long is scoreable.
#[test]
fn japanese_medieval_pair_captures_say_their_lobby() {
    let p = &JAPANESE_MEDIEVAL;
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
        "the Japanese Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3664; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Japanese; and no other Japanese lobby's: the Industrial and Ancient
/// pairs' `(18, 15, 5)`, the Gunpowder pair's `(7, 15, 5)` and the AI word
/// `GreatSaharaJapaneseTier1`'s `(7, 15, 1)` are another map (item 3664).
#[test]
fn japanese_medieval_pair_is_no_other_lobby() {
    let p = &JAPANESE_MEDIEVAL;
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
        "the battery fields the Japanese"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Japanese lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::japanese_industrial_pair::JAPANESE_INDUSTRIAL,
        &super::japanese_ancient_pair::JAPANESE_ANCIENT,
        &super::japanese_gunpowder_pair::JAPANESE_GUNPOWDER,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    let w = &super::japanese_word::JAPANESE;
    assert_eq!(w.ai_tribe, p.ai_tribe);
    assert_ne!((w.map_style, w.ai_tribe, w.difficulty), triple);
}

/// **The long's stride, walked to the game's end** (item 3664): run1329's
/// stride blocks, 101 through 5101, the last before the closing window.
pub(crate) const WIDENING_JAPANESE_MEDIEVAL_STRIDE: (i64, i64) = (101, 5101);

pub(crate) fn japanese_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_MEDIEVAL.start],
        true,
        JAPANESE_MEDIEVAL.long,
        "run1329 stride",
        &[(
            JAPANESE_MEDIEVAL.long.0,
            WIDENING_JAPANESE_MEDIEVAL_STRIDE.0,
        )],
        WIDENING_JAPANESE_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn japanese_medieval_pair_the_long_s_stride_walks_to_5101() {
    let _pins = Pins::hold();
    let Some(w) = japanese_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 51, "every stride block in the window");
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
    // Item 3664: 108 keys, all of them the standing families — 65
    // buildings' `build:extra` (which the stride does not print), `form`
    // on 27 units, and who=1's groups' `held` and pools 64..71 as each is
    // minted. **No unit row parts on any stride block to 5101.**
    pin_eq!(w.firsts.len(), 108, "the stride's baseline");
}

/// run1329b: the closing window, blocks 4884..5141 at the word window's
/// detail with `LEADERS=9` and `AMMO=5` (item 3664): the pair is whole on
/// its first try, and run1329 prints units only every hundredth block and
/// no `AMMO`, `LEADERS`, `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1329B: &str =
    "gamelog-run1329b-greatlakes-japanese-medieval-window-4884-5141.txt";

/// The window: block 4884 through the closing block 5141.
pub(crate) const WIDENING_JAPANESE_MEDIEVAL_FRAME_5140: (i64, i64) = (4884, 5141);

/// **The closing window**: run1329b walked from run1328's start with the
/// recorder on and every round compared (item 3664).
pub(crate) fn japanese_medieval_frame_5140_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[JAPANESE_MEDIEVAL.start],
        true,
        JAPANESE_MEDIEVAL.long,
        "run1329b",
        &[(RUN1329B, WIDENING_JAPANESE_MEDIEVAL_FRAME_5140.0)],
        WIDENING_JAPANESE_MEDIEVAL_FRAME_5140,
        1,
        &[5141],
        true,
    )
}

#[test]
fn japanese_medieval_pair_run1329b_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_medieval_frame_5140_window() else {
        return;
    };
    // 4884..5140; the closing block 5141 is the endpoint's.
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
    // Item 3664, the first walk: 52 keys, every one on the window's first
    // block 4884 and a standing family — `form` on 27 units, who=0's
    // `SITE[i].reg` (12), `aggression`, and who=1's groups' `held` and
    // pools. No unit, building or city field parts.
    pin_eq!(w.firsts.len(), 52, "run1329b baseline");
    // Every round, both directions: 143 a side, every one the Trebuchet
    // `1/59`'s (TI 266), four throws at the human's `0/2000`. The four rows
    // that part are its launch, first on 4884 and on every block after:
    // ours throws from the figure's square (6936, 32184) at `sz` 92, theirs
    // from (7219, 32287) at 785, and `v1z` 210946716 against 192709869;
    // the landing (3187, 30722), the flight and every hit agree to the
    // closing block (parked 3658b, 3664a).
    pin_eq!(w.rounds_read, (143, 143), "the rounds read");
    pin_eq!(w.rounds.len(), 4, "the rounds that part");
}

/// **The closing state** (item 3664): the long's own closing block,
/// walked to its close and compared unit for unit — a draw stream that
/// agrees to a game's end says nothing of the end itself (item 1099).
#[test]
fn japanese_medieval_pair_run1329_s_closing_state() {
    let _pins = Pins::hold();
    let p = &JAPANESE_MEDIEVAL;
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
        "Japanese Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 5141, "the closing block");
    // Item 3664, the pair whole on its first try: 187 units, none off,
    // torn, unlinked or extra, `Closing fields: none` — the game ends
    // itself on 5140.
    pin_eq!(e.compared, 187, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
    pin_eq!(
        e.torn.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing tears"
    );
}
