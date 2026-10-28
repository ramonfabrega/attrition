//! **The Maya at a held Information age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, 97, 98, 99, 101, item 3726): Great Lakes, `MAP_STYLE 14`, human
//! Nubians (4) against the **Maya (1)** at difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Information:
//! `STARTING_TECHNOLOGY 7`, `ENDING_TECHNOLOGY 7`. Its triple, `(14, 1, 5)`,
//! is new: the Maya hold East Indies at 5 (`maya_ancient_pair`) and Great
//! Sahara at 4 (`maya_word`, once the battery's `GreatSaharaMayaTier4`);
//! none holds Great Lakes. Captures: run1410 (a `DUMP_ALL` start) and
//! run1411 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §373.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const MAYA_INFORMATION: Pair = Pair {
    name: "GreatLakesMayaInformation",
    map_style: 14,
    ai_tribe: 1,
    difficulty: 5,
    starting_technology: 7,
    ending_technology: 7,
    starting_resources: 7,
    start: "gamelog-run1410-greatlakes-maya-information-start.txt",
    long: (
        "gamelog-run1411-greatlakes-maya-information-trace.txt",
        "rontrace-run1411.log",
    ),
    // The game ended itself on 3330 (closing block 3331).
    length: 3330,
    // Item 3726, first try: **whole**, count 3330 and sequence 3330 of
    // 3330 — no draw parts to the game's last frame, and the closing block
    // 3331 agrees on all 123 units (`docs/AI.md` §373).
    count: 3330,
    sequence: 3330,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MAYA_INFORMATION_PAIR_WORD: i64 =
    if MAYA_INFORMATION.count < MAYA_INFORMATION.sequence {
        MAYA_INFORMATION.count
    } else {
        MAYA_INFORMATION.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn maya_information_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MAYA_INFORMATION;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MAYA_INFORMATION {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 7 and the difficulty at 5, and the long is
/// scoreable.
#[test]
fn maya_information_pair_captures_say_their_lobby() {
    let p = &MAYA_INFORMATION;
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
        "the Maya Information pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3726; the battery's identity
/// test refuses it), once, and none of the battery's — which fields no
/// Maya since `GreatSaharaMayaTier4`, `(7, 1, 4)`, was promoted to the AI
/// word — and no other Maya lobby's: `maya_word`'s `(7, 1, 4)` and
/// `maya_ancient_pair`'s `(18, 1, 5)`, other maps: no Maya lobby holds
/// Great Lakes (item 3726).
#[test]
fn maya_information_pair_is_no_other_lobby() {
    let p = &MAYA_INFORMATION;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    for l in battery::BATTERY.iter().filter(|l| l.ai_tribe == p.ai_tribe) {
        assert_ne!(
            (l.map_style, l.ai_tribe, l.difficulty),
            triple,
            "{} is this lobby",
            l.name
        );
        assert_ne!(l.map_style, p.map_style, "{} fields the Maya here", l.name);
    }
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style,
            "{l:?} is another Maya lobby on this map"
        );
    }
    let w = &super::maya_word::MAYA;
    let a = &super::maya_ancient_pair::MAYA_ANCIENT;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (a.name, (a.map_style, a.ai_tribe, a.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Maya");
        assert_ne!(l, triple, "{name} is this lobby");
        assert_ne!(l.0, p.map_style, "{name} holds Great Lakes");
    }
}

/// **The long's stride** (item 3726): run1411's stride blocks, 101
/// through the last, 3301; the pair is whole, so there is no word to walk
/// back from.
pub(crate) const WIDENING_MAYA_INFORMATION_STRIDE: (i64, i64) = (101, 3301);

pub(crate) fn maya_information_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MAYA_INFORMATION.start],
        true,
        MAYA_INFORMATION.long,
        "run1411 stride",
        &[(MAYA_INFORMATION.long.0, WIDENING_MAYA_INFORMATION_STRIDE.0)],
        WIDENING_MAYA_INFORMATION_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn maya_information_pair_the_long_s_stride_is_widened() {
    let _pins = Pins::hold();
    let Some(w) = maya_information_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 33, "every stride block in the window");
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
    // Item 3726: 109 keys over 33 blocks, the standing families alone —
    // `build:extra` on 56 buildings (which the stride does not print),
    // `form` on 27 units, who=1's nine groups' `held` with their pools, and
    // `mirror` on eight of who=1's craft. No position, order or hit parts
    // to the last stride block 3301.
    pin_eq!(w.firsts.len(), 109, "the stride's baseline");
}

/// run1411b: the pair's closing window, blocks 3075..3331 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — 256 blocks to the closing block
/// 3331 (item 3726). The pair is whole on its first try; run1411's stride
/// prints units only every hundred frames, no `AMMO` and no `LEADERS`,
/// `BUILDS` or `CITIES` record past the start.
pub(crate) const RUN1411B: &str =
    "gamelog-run1411b-greatlakes-maya-information-closing-window-3075-3331.txt";

/// The window: block 3075 through the closing block 3331.
pub(crate) const WIDENING_MAYA_INFORMATION_CLOSING: (i64, i64) = (3075, 3331);

/// **The pair's last blocks**: run1411b walked from run1410's start with
/// the recorder on and every round compared (item 3726).
pub(crate) fn maya_information_closing_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[MAYA_INFORMATION.start],
        true,
        MAYA_INFORMATION.long,
        "run1411b",
        &[(RUN1411B, WIDENING_MAYA_INFORMATION_CLOSING.0)],
        WIDENING_MAYA_INFORMATION_CLOSING,
        1,
        &[3331],
        true,
    )
}

#[test]
fn maya_information_pair_run1411b_s_closing_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = maya_information_closing_window() else {
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
    // Item 3726, the pair whole on its first try: **57** keys. Block 3075
    // carries the standing families — who=0's `SITE[0..9].reg` (1 against
    // 0) and `aggression[1]` (0 against 1), who=1's `SITE[0..2].reg`,
    // `form` on 13 units, who=1's nine groups' `held` with their pools and
    // `mirror` on eight craft — and three `dest_angle` rows on who=1's
    // units (`1/43` on 3130, `1/19` on 3151, `1/106` on 3283). One building
    // field parts: `0/2000`'s `damage`, 1156 against 1157 on 3265, the
    // block a round of the Armored Cavalry `1/31` is in ours alone. No
    // position, order or hit parts, no draw parts to 3330, and the closing
    // block 3331 agrees.
    pin_eq!(w.firsts.len(), 57, "run1411b baseline");
    pin_eq!(w.rounds_read, (240, 243), "the rounds read");
    // Every round, both directions: 240 in ours against 243, 121 parting.
    // The first is the Armored Cavalry `1/31`'s (TI 220, piece 874 at the
    // Maya's art style 2, which neither `launch::BAYS` nor `pivot` holds)
    // `ammo[0]` on 3205: `sx, sy, sz` 6223, 32469, 219 — its own square —
    // against 6099, 32440, 359; `1/48` and `1/58` part the same way. The
    // Strategic Bombers (`1/83`, `1/99`, TI 305) and Jet Fighters (`1/91`,
    // `1/100`, TI 295) part on angle and `accuracy` (parked 3726a).
    pin_eq!(w.rounds.len(), 121, "the rounds that part");
}

/// **The pair's closing state** (item 3726): run1411's closing block 3331,
/// the whole map's units, walked from run1410's start.
#[test]
fn maya_information_pair_run1411_s_closing_state() {
    let _pins = Pins::hold();
    let p = &MAYA_INFORMATION;
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
        "Maya Information pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 3331, "the closing block");
    // Item 3726, first try: the pair whole — 123 units compared, the
    // counts all zero, none torn, extra or unlinked.
    pin_eq!(e.compared, 123, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
