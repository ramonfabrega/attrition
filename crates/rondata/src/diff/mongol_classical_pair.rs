//! **The Mongols at a held Classical age, Great Sahara** (DECISIONS 90, 91,
//! 92, item 3558): Great Sahara, `MAP_STYLE 7`, human Nubians (4) against
//! the **Mongols (17)** at Difficulty 5, seed 12345, `STARTING_RESOURCES
//! 7`, the age held at Classical: `STARTING_TECHNOLOGY 1`,
//! `ENDING_TECHNOLOGY 1` — the Romans' held Classical lobby with the nation
//! moved. The Mongols' other pairs are `mongol_pair` and `sahara_mongol`, at
//! Difficulty 3 and All Technologies. Captures: run1197 (a `DUMP_ALL`
//! start) and run1198 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §311.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const MONGOL_CLASSICAL: Pair = Pair {
    name: "GreatSaharaMongolClassical",
    map_style: 7,
    ai_tribe: 17,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1197-greatsahara-mongol-classical-start.txt",
    long: (
        "gamelog-run1198-greatsahara-mongol-classical-trace.txt",
        "rontrace-run1198.log",
    ),
    // The game ended itself on 9884 (closing block 9885).
    length: 9884,
    // Item 3558, first try: frame 8444 by the count and the sequence, ours
    // 21 draws against 22, index 8 — ours `Unit::do_guard+0x8fb`, theirs
    // `Guy::set_anim+0x97a < Unit::do_guard+0x7f4` (`docs/AI.md` section
    // 311). Item 3575: 8444 -> 9744, `Sim::land_size` ranged over the
    // original's region numbers, so who=1's territory timer starts on
    // 8195 as theirs does and its army's `about_to_win` x100 sends group
    // 65 at who=0 on 8440. At 9744 ours 117 draws against 118, index 90 —
    // ours `Guy::set_anim+0xf2f < Guy::move+0x166`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`. Item 3579: 9744 ->
    // 9772, the Mongol Archers (piece 1177) and Javelineers (piece 1089)
    // loose from their bays, so the rounds at the fleeing Citizen `0/8`
    // fly 15 and 17 frames, not 16 and 18, and strike on 9708, 9714 and
    // 9720 as theirs do. At 9772 ours 31 draws against 32, index 29 —
    // ours `Farms::inc_time+0x1ae`, theirs `Object::take_damage+0xe1`.
    count: 9772,
    sequence: 9772,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const MONGOL_CLASSICAL_PAIR_WORD: i64 =
    if MONGOL_CLASSICAL.count < MONGOL_CLASSICAL.sequence {
        MONGOL_CLASSICAL.count
    } else {
        MONGOL_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn mongol_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &MONGOL_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "MONGOL_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn mongol_classical_pair_captures_say_their_lobby() {
    let p = &MONGOL_CLASSICAL;
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
        "the Mongol Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: `(7, 17, 5)` is no held-out game's — the battery
/// fields no Mongols — and no other pinned pair's: the Mongols' two pairs
/// are `mongol_pair`'s `(18, 17, 3)` and `sahara_mongol`'s `(7, 17, 3)`,
/// both at Difficulty 3 and All Technologies. Both `GAME INFO`s are read:
/// the same map's Mongol pair against this pair's held age (3513's shape,
/// Loop 3544).
#[test]
fn mongol_classical_pair_is_no_other_lobby() {
    let p = &MONGOL_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Mongols"
    );
    let others = [
        &super::mongol_pair::MONGOL,
        &super::sahara_mongol::SAHARA_MONGOL,
    ];
    for o in others {
        assert_eq!(o.ai_tribe, p.ai_tribe);
        assert_ne!(
            (o.map_style, o.ai_tribe, o.difficulty),
            triple,
            "{} is this lobby",
            o.name
        );
    }
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
    let (Some(sahara), Some(ours)) = (info(others[1].long.0), info(p.long.0)) else {
        return;
    };
    assert_eq!(sahara.0, Some(7), "the Sahara Mongol pair's map");
    assert_eq!(sahara.1, Some(3), "the Sahara Mongol pair's difficulty");
    assert_eq!(sahara.2, Some(8), "the Sahara Mongol pair's starting age");
    assert_eq!(
        ours,
        (Some(7), Some(5), Some(1), Some(1)),
        "this pair's lobby"
    );
}

/// run1198b: the word 8444's window, blocks 8438..8694 at the long's
/// detail — six blocks before the word's block 8445 and 250 after (item
/// 3558); the stride's blocks are 8401 and 8501.
pub(crate) const RUN1198B: &str =
    "gamelog-run1198b-greatsahara-mongol-classical-window-8438-8694.txt";

/// The window: block 8438 through 8694; the word 8444's own block is 8445.
pub(crate) const WIDENING_MONGOL_CLASSICAL_FRAME_8444: (i64, i64) = (8438, 8694);

/// **The word 8444's window**: run1198b walked from run1197's start with
/// the recorder on (item 3558).
pub(crate) fn mongol_classical_frame_8444_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_CLASSICAL.start],
        true,
        MONGOL_CLASSICAL.long,
        "run1198b",
        &[(RUN1198B, WIDENING_MONGOL_CLASSICAL_FRAME_8444.0)],
        WIDENING_MONGOL_CLASSICAL_FRAME_8444,
        1,
        &[8445],
        true,
    )
}

#[test]
fn mongol_classical_pair_run1198b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_classical_frame_8444_window() else {
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
    // Item 3558, the first walk: 6457 keys. Block 8438 carries the
    // standing families (`form`, `SITE[i].reg`, who=0's `aggression[1]`,
    // who=1's groups' `held` and `pool`), who=1's `bucket` and
    // `popwin_*`, and the `g.angle[0]` of `1/307` and `1/308`. **The state
    // first parts on block 8441**: who=0's `attacked_by` ours -1 theirs 1
    // and `frame_attacked` ours 0 theirs 8440, and the march of who=1's
    // group 65 — the Catapult `1/99` (TypeIndex 265, land) `order:move.x`
    // ours 12024 theirs 1320, `path:length` 23 against 38, and `1/101`,
    // `1/117` the same. **The word 8444** (block 8445), ours 21 draws
    // against 22, index 8 ours `Unit::do_guard+0x8fb`, theirs
    // `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`: theirs' group 64 —
    // the General `1/69` (TypeIndex 54), the Phalanx `1/209` (133) and 22
    // more — keeps its `GUARDORDER` on the Catapult `1/67`, where ours'
    // takes a fresh order: `order:kind` ours 2 theirs 12, `orders.len` 2
    // against 1, on 24 units; the Catapult `1/66`'s `orders.len` 1
    // against 0 (`docs/AI.md` section 311).
    // Item 3575: 6457 -> 77. With the land counted by the original's
    // region number, who=1's `popwin_timer`/`popwin_stamp` (1, 8195) agree
    // on block 8438, block 8441 is whole (who=0's `attacked_by` 1 and
    // `frame_attacked` 8440 both sides; group 65's `1/99` marches to
    // (1320, 28872) with `path:length` 38) and so is the word's block 8445
    // (group 64 keeps its `GUARD`). What stands: block 8438's families,
    // who=1's `bucket`/`leftover` (3558a), the `g.angle[0]` of newborn
    // `1/307`.. on their first block, and on 8586 the `cost[0]`/`cost[1]`
    // of `1/2017`'s and `1/2022`'s queues, ours 116/86 against 110/82.
    pin_eq!(w.firsts.len(), 77, "run1198b baseline");
}

/// run1218: the word 9744's window, blocks 9738..9885 at the long's detail —
/// six blocks before the word's block 9745 and on to the game's closing
/// block 9885 (item 3575); the stride's blocks are 9701 and 9801.
pub(crate) const RUN1218: &str =
    "gamelog-run1218-greatsahara-mongol-classical-window-9738-9885.txt";

/// The window: block 9738 through the closing block 9885; the word 9744's
/// own block is 9745.
pub(crate) const WIDENING_MONGOL_CLASSICAL_FRAME_9744: (i64, i64) = (9738, 9885);

/// **The word 9744's window**: run1218 walked from run1197's start with the
/// recorder on, to the closing block (item 3575).
pub(crate) fn mongol_classical_frame_9744_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_CLASSICAL.start],
        true,
        MONGOL_CLASSICAL.long,
        "run1218",
        &[(RUN1218, WIDENING_MONGOL_CLASSICAL_FRAME_9744.0)],
        WIDENING_MONGOL_CLASSICAL_FRAME_9744,
        1,
        &[9745],
        true,
    )
}

#[test]
fn mongol_classical_pair_run1218_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_classical_frame_9744_window() else {
        return;
    };
    pin_eq!(w.blocks, 147, "every captured block");
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
    // Item 3575, the first walk: 2493 keys, and the window opens apart.
    // Block 9738 carries the standing families and who=1's `bucket`/
    // `leftover` (3558a), and **the Nubian Citizen `0/8` (TypeIndex 50,
    // land, uid 15), fleeing on a `FLEETOORDER`**: `damage_frame` ours 9721
    // against theirs 9720, `pos` (5272, 28856) against (5261, 28879). The
    // who=1 units chasing it — the Javelineers `1/49` (83) of group 64, the
    // Phalanx `1/133` (133) of group 65 and more, each with an `ATTACKORDER`
    // on uid 15 — hold moves 48 apart in `y` (`1/49`'s `order:move.y` 28248
    // against 28296). **The word 9744** (block 9745): ours 117 draws
    // against 118, index 90, ours `Guy::set_anim+0xf2f < Guy::move+0x166`,
    // theirs `Guy::set_anim+0x97a < Unit::move_step+0x823`; `0/8`'s
    // `collide` 0 against 1 and `path:length` 1 against 4. The stride
    // (`mongol_classical_pair_the_long_s_stride_walks_back_from_9744`)
    // agrees through 9701, so the state parts in 9702..9737.
    // Item 3579: 2493 -> 6619. run1222 found the gap's parting (the Mongol
    // Archers' and Javelineers' bays); with it `0/8` agrees on 9738 and
    // after, and the walk runs on together to 9761, where the Citizen
    // `0/11` (TypeIndex 50) is struck for 15 in theirs by the Nomad `1/31`
    // (TypeIndex 199, `damage_o` 31) and not in ours (`hits:damage` 35
    // against 20, `damage_frame` 9761 against 9724, block 9762). The word
    // is 9772 (block 9773), and what follows parts wider than the old
    // walk's tail did: the count is the later parting's, not a regression
    // before it.
    pin_eq!(w.firsts.len(), 6619, "run1218 baseline");
}

/// **The long's stride, walked back from the word 9744** (item 3575):
/// run1218's window opens apart on 9738, and only the stride's blocks are on
/// disk between run1198b's 8694 and run1218's 9738.
pub(crate) const WIDENING_MONGOL_CLASSICAL_STRIDE_9744: (i64, i64) = (8701, 9701);

pub(crate) fn mongol_classical_stride_9744_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_CLASSICAL.start],
        true,
        MONGOL_CLASSICAL.long,
        "run1198 stride 9744",
        &[(
            MONGOL_CLASSICAL.long.0,
            WIDENING_MONGOL_CLASSICAL_STRIDE_9744.0,
        )],
        WIDENING_MONGOL_CLASSICAL_STRIDE_9744,
        1,
        &[],
        true,
    )
}

#[test]
fn mongol_classical_pair_the_long_s_stride_walks_back_from_9744() {
    let _pins = Pins::hold();
    let Some(w) = mongol_classical_stride_9744_window() else {
        return;
    };
    pin_eq!(w.blocks, 11, "every stride block in the window");
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
    // Item 3575: 148 keys, every one a standing family — `build:extra`,
    // every unit's `form`, who=1's groups' `held` and pool, the `g.angle[0]`
    // of who=1's newborns on their first block. No unit row parts on a
    // stride block through 9701.
    pin_eq!(w.firsts.len(), 148, "the stride's baseline");
}

/// run1222: the gap 9702..9737 the stride and run1218 leave undumped,
/// blocks 9696..9745 at run1218's detail with `AMMO=5` (item 3579).
pub(crate) const RUN1222: &str =
    "gamelog-run1222-greatsahara-mongol-classical-window-9696-9745.txt";

/// The window: block 9696 through the word 9744's own block 9745.
pub(crate) const WIDENING_MONGOL_CLASSICAL_GAP_9744: (i64, i64) = (9696, 9745);

/// **The gap before the word 9744**: run1222 walked from run1197's start
/// with the recorder on (item 3579).
pub(crate) fn mongol_classical_gap_9744_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_CLASSICAL.start],
        true,
        MONGOL_CLASSICAL.long,
        "run1222",
        &[(RUN1222, WIDENING_MONGOL_CLASSICAL_GAP_9744.0)],
        WIDENING_MONGOL_CLASSICAL_GAP_9744,
        1,
        &[9745],
        true,
    )
}

#[test]
fn mongol_classical_pair_run1222_walks_the_gap_before_9744() {
    let _pins = Pins::hold();
    let Some(w) = mongol_classical_gap_9744_window() else {
        return;
    };
    pin_eq!(w.blocks, 49, "every captured block");
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
    // Item 3579, the first walk: 171 keys. The first unit field apart is
    // on block 9709: **the Nubian Citizen `0/8` (TypeIndex 50, land, uid
    // 15)**, `hits:damage` ours 31 against theirs 33, `damage_frac` 15
    // against 4, `hits_left` 19 against 17 — the Mongol Archers `1/74`'s
    // round (TypeIndex 171, piece 1177) struck on 9708 in theirs and 9709
    // in ours. The `AMMO` records say why: theirs' round leaves from (7087,
    // 27292) at `total_time` 15, ours' from the figure's own (7128, 27240)
    // at 16, and `1/49`'s (TypeIndex 83, piece 1089) javelin 17 against
    // 18. Built (`sim::launch`, the two pieces' bays): 171 -> 82, every
    // `0/8` row agreeing. What stands: the standing families, 1/187's
    // `orders_x`/`orders_y` on 9714 (9627, 30596 against 9634, 30612), and
    // the `dest_angle` family (3574b) from 9733.
    pin_eq!(w.firsts.len(), 82, "run1222 baseline");
}

/// run1224: the word 9772's window, blocks 9740..9885 at run1218's detail
/// with `AMMO=5` (item 3581) — the Nomad `1/31`'s round at the Citizen
/// `0/11`, loosed on 9746, and on to the closing block.
pub(crate) const RUN1224: &str =
    "gamelog-run1224-greatsahara-mongol-classical-window-9740-9885.txt";

/// The window: block 9740 through the closing block 9885; the word 9772's
/// own block is 9773.
pub(crate) const WIDENING_MONGOL_CLASSICAL_FRAME_9772: (i64, i64) = (9740, 9885);

/// **The word 9772's window**: run1224 walked from run1197's start with the
/// recorder on, to the closing block (item 3581).
pub(crate) fn mongol_classical_frame_9772_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[MONGOL_CLASSICAL.start],
        true,
        MONGOL_CLASSICAL.long,
        "run1224",
        &[(RUN1224, WIDENING_MONGOL_CLASSICAL_FRAME_9772.0)],
        WIDENING_MONGOL_CLASSICAL_FRAME_9772,
        1,
        &[9773],
        true,
    )
}

#[test]
fn mongol_classical_pair_run1224_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = mongol_classical_frame_9772_window() else {
        return;
    };
    pin_eq!(w.blocks, 146, "every captured block");
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
    pin_eq!(w.firsts.len(), 0, "run1224 baseline");
}
