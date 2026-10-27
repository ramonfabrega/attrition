//! **The Russians at a held Gunpowder age, Great Lakes** (DECISIONS 90,
//! 91, 93, 94, 97, item 3670): Great Lakes, `MAP_STYLE 14`, human Nubians
//! (4) against the **Russians (13)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder:
//! `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`. Its triple,
//! `(14, 13, 5)`, is new: the Russian lobbies are `russian_modern_pair`'s
//! `(18, 13, 5)`, `russian_medieval_pair`'s `(7, 13, 5)`, the AI word
//! `russian_word`'s `GreatLakesRussianTier2`, `(14, 13, 2)`, another
//! difficulty, and the held-out battery's `GreatSaharaRussianTier4`,
//! `(7, 13, 4)`. Captures: run1339 (a `DUMP_ALL` start) and run1340 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try
//! is in `docs/AI.md` §349; item 3688 closed it whole (§349.1): the
//! Monarch's cavalry arm of `UnitData::attack`, its closing window
//! run1357b.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const RUSSIAN_GUNPOWDER: Pair = Pair {
    name: "GreatLakesRussianGunpowder",
    map_style: 14,
    ai_tribe: 13,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1339-greatlakes-russian-gunpowder-start.txt",
    long: (
        "gamelog-run1340-greatlakes-russian-gunpowder-trace.txt",
        "rontrace-run1340.log",
    ),
    // The game ended itself on 5341 (closing block 5342).
    length: 5341,
    // Item 3670, first try: frame 3711 by the sequence (3712 by the
    // count), 19 draws a side, parting at index 2 — ours
    // `Unit::fight+0x824`, theirs `Unit::close+0xcb6` (`docs/AI.md`
    // §349). Item 3688, **whole**: 5341 of 5341 by both — the Monarch's
    // cavalry arm (`UnitData::attack@006103c0`, `THEMONARCH_CAV_ATTACK ×
    // 10`) on who=1's Heavy Knight `1/32`'s blow of 3440, 26 → 29, the
    // value run1357's packet returns (`docs/AI.md` §349.1).
    count: 5341,
    sequence: 5341,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const RUSSIAN_GUNPOWDER_PAIR_WORD: i64 =
    if RUSSIAN_GUNPOWDER.count < RUSSIAN_GUNPOWDER.sequence {
        RUSSIAN_GUNPOWDER.count
    } else {
        RUSSIAN_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn russian_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "RUSSIAN_GUNPOWDER {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 3, and the long is scoreable.
#[test]
fn russian_gunpowder_pair_captures_say_their_lobby() {
    let p = &RUSSIAN_GUNPOWDER;
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
        "the Russian Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3670; the battery's identity
/// test refuses it), once, and none of the battery's: the held-out
/// `GreatSaharaRussianTier4` is `(7, 13, 4)`, another map and difficulty;
/// and no other Russian lobby's: `russian_word`'s `(14, 13, 2)` is this
/// map at another difficulty, and the Modern and Medieval pairs' `(18, 13,
/// 5)` and `(7, 13, 5)` other maps (item 3670).
#[test]
fn russian_gunpowder_pair_is_no_other_lobby() {
    let p = &RUSSIAN_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert_eq!(
        battery::SCORED_LOBBIES
            .iter()
            .filter(|l| **l == triple)
            .count(),
        1
    );
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let russians: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        russians,
        [("GreatSaharaRussianTier4", 7, 4)],
        "the battery's Russian games"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Russian lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::russian_modern_pair::RUSSIAN_MODERN,
        &super::russian_medieval_pair::RUSSIAN_MEDIEVAL,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    let w = &super::russian_word::RUSSIAN;
    assert_eq!(w.ai_tribe, p.ai_tribe);
    assert_ne!((w.map_style, w.ai_tribe, w.difficulty), triple);
}

/// **The long's stride, walked back from the word 3711** (item 3670):
/// only run1340's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_RUSSIAN_GUNPOWDER_STRIDE: (i64, i64) = (101, 3701);

pub(crate) fn russian_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[RUSSIAN_GUNPOWDER.start],
        true,
        RUSSIAN_GUNPOWDER.long,
        "run1340 stride",
        &[(
            RUSSIAN_GUNPOWDER.long.0,
            WIDENING_RUSSIAN_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_RUSSIAN_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_gunpowder_pair_the_long_s_stride_walks_back_from_3711() {
    let _pins = Pins::hold();
    let Some(w) = russian_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 37, "every stride block in the window");
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
    // Item 3670: 102 keys. The standing families — 53 buildings'
    // `build:extra` (which the stride does not print), `form` on 27 units,
    // and who=1's groups' `held` and pools 64..71 as each is minted — and
    // **one unit row, on 3501: the human's Citizen `0/5`'s damage**
    // (`hits_left` ours 25 against theirs 22, `hits:damage` 25 against 28).
    // Item 3688: **100** — the Citizen's two rows are gone with the
    // Monarch's cavalry arm; the standing families stay.
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1340b: the word 3711's window, blocks 3695..3961 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3701 and 250 after the word's block 3712 (item 3670).
pub(crate) const RUN1340B: &str =
    "gamelog-run1340b-greatlakes-russian-gunpowder-window-3695-3961.txt";

/// The window: block 3695 through 3961; the word 3711's own block is 3712.
pub(crate) const WIDENING_RUSSIAN_GUNPOWDER_FRAME_3711: (i64, i64) = (3695, 3961);

/// **The word 3711's window**: run1340b walked from run1339's start with
/// the recorder on and every round compared (item 3670).
pub(crate) fn russian_gunpowder_frame_3711_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_GUNPOWDER.start],
        true,
        RUSSIAN_GUNPOWDER.long,
        "run1340b",
        &[(RUN1340B, WIDENING_RUSSIAN_GUNPOWDER_FRAME_3711.0)],
        WIDENING_RUSSIAN_GUNPOWDER_FRAME_3711,
        1,
        &[3712],
        true,
    )
}

#[test]
fn russian_gunpowder_pair_run1340b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_gunpowder_frame_3711_window() else {
        return;
    };
    pin_eq!(w.blocks, 266, "every captured block");
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
    // Item 3670, the first walk: 3037 keys. Block 3695 carries the
    // standing families (`form`, who=0's `SITE[i].reg` and `aggression`,
    // who=1's groups and pools), who=1's leader's `attack` (938 against
    // 946) and the Citizen `0/5`'s damage the stride already parts
    // (`hits_left` 30 against 27). **On the word's block 3712 `0/5` is
    // ours alone** — theirs' draw at index 2 is `Unit::close+0xcb6`, the
    // Citizen's death under who=1's Heavy Knight `1/32` (TY 229) — and the
    // human's leader holds a unit more (`num_units[0]` 13 against 12);
    // who=1's Heavy Knight `1/42`, whose guard took `0/4` on 3711, pushes
    // its move on 3712 in ours and on 3714 in theirs, 48 units apart.
    // Item 3688: **58** — the standing families alone (`form`, who=0's
    // `SITE[i].reg` and `aggression`, who=1's groups' `held` and pools
    // 64..73). who=1's `leader:attack` is gone with the Citizen's rows:
    // the leader's attack tally parted on the same Monarch's arm.
    pin_eq!(w.firsts.len(), 58, "run1340b baseline");
    // Every round, both directions: none — the window's fighting is
    // melee.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1340c: the window over the Citizen `0/5`'s damage parting, blocks
/// 3395..3751 at run1340b's detail — six blocks before the stride's 3401,
/// where the damage still agrees, and 250 after its 3501 (item 3670).
pub(crate) const RUN1340C: &str =
    "gamelog-run1340c-greatlakes-russian-gunpowder-window-3395-3751.txt";

/// The window: block 3395 through 3751.
pub(crate) const WIDENING_RUSSIAN_GUNPOWDER_DAMAGE: (i64, i64) = (3395, 3751);

/// **The damage's window**: run1340c walked from run1339's start with the
/// recorder on and every round compared (item 3670).
pub(crate) fn russian_gunpowder_damage_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_GUNPOWDER.start],
        true,
        RUSSIAN_GUNPOWDER.long,
        "run1340c",
        &[(RUN1340C, WIDENING_RUSSIAN_GUNPOWDER_DAMAGE.0)],
        WIDENING_RUSSIAN_GUNPOWDER_DAMAGE,
        1,
        &[],
        true,
    )
}

#[test]
fn russian_gunpowder_pair_run1340c_s_damage_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_gunpowder_damage_window() else {
        return;
    };
    pin_eq!(w.blocks, 356, "every captured block");
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3670, the first walk: 227 keys. Block 3395 carries the
    // standing families and who=1's leader's `attack` (888 against 894,
    // `mongol_gunpowder_pair`'s standing row). **The first unit row is
    // the blow of 3440: who=1's Heavy Knight `1/32` (TY 229) strikes the
    // human's Citizen `0/5` — `hits:damage` 26 in ours against 29 in
    // theirs on block 3441** (`hits_left` 24 against 21); `0/5` flees
    // on both sides. The rest is the word's block 3712 and after, as
    // run1340b's.
    // Item 3688: **58**, the standing families alone. The blow of 3440 is
    // 29 on both sides (`hits_left` 21 on block 3441): the Monarch's
    // cavalry arm adds `THEMONARCH_CAV_ATTACK × 10` = 20 to the Heavy
    // Knight's 170, the attack run1357's packet returns (190); and
    // `leader:attack` (888 against 894 on 3395) agrees.
    pin_eq!(w.firsts.len(), 58, "run1340c baseline");
    // No round: the blow is melee.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// run1357b: the closing window, blocks 5086..5342 at run1340b's detail,
/// `cover=0` (item 3688) — 257 blocks through the closing block, so the
/// word 5341, the game's last frame, sits inside a window.
pub(crate) const RUN1357B: &str =
    "gamelog-run1357b-greatlakes-russian-gunpowder-window-5086-5342.txt";

/// The window: block 5086 through the closing block 5342.
pub(crate) const WIDENING_RUSSIAN_GUNPOWDER_FRAME_5341: (i64, i64) = (5086, 5342);

/// **The closing window**: run1357b walked from run1339's start with the
/// recorder on, the rounds compared (item 3688).
pub(crate) fn russian_gunpowder_frame_5341_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[RUSSIAN_GUNPOWDER.start],
        true,
        RUSSIAN_GUNPOWDER.long,
        "run1357b",
        &[(RUN1357B, WIDENING_RUSSIAN_GUNPOWDER_FRAME_5341.0)],
        WIDENING_RUSSIAN_GUNPOWDER_FRAME_5341,
        1,
        &[5342],
        true,
    )
}

#[test]
fn russian_gunpowder_pair_run1357b_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = russian_gunpowder_frame_5341_window() else {
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
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3688: **60** — the standing families (`form`, who=0's
    // `SITE[i].reg` and `aggression`, who=1's groups' `held` and pools),
    // two Crossbowmen's walk ends a few units apart (`1/60`'s `orders_x`
    // 6312 against 6308 on 5237, `1/59`'s on 5275) and two idle units'
    // `dest_angle` (`1/65` on 5239, `1/75` on 5271): draw-silent to the
    // close (parked 3688b).
    pin_eq!(w.firsts.len(), 60, "run1357b baseline");
    pin_eq!(w.rounds_read, (504, 504), "the rounds read");
    // 504 a side, 153 parting: who=1's Crossbowmen (TY 172) launch a unit
    // or two off theirs (`1/36`'s of 5146 from (3777, 31725) against
    // (3779, 31724)), and its Bombards' (TY 267) height 269 against 373
    // (parked 3688a). No draw parts.
    pin_eq!(w.rounds.len(), 153, "the rounds that part");
}

/// **The closing state** (item 3688): run1340's closing block 5342, the
/// whole map's units, walked from run1339's start.
#[test]
fn russian_gunpowder_pair_run1340_s_closing_state() {
    let _pins = Pins::hold();
    let p = &RUSSIAN_GUNPOWDER;
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
        "Russian Gunpowder pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    // Item 3688, the pair whole: 248 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 5341.
    pin_eq!(e.frame, 5342, "the closing block");
    pin_eq!(e.compared, 248, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
