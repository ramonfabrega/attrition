//! **The Persians at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! item 3535): the coverage pair's own lobby (run652's: East Indies,
//! `MAP_STYLE 18`, human Nubians (4) against the **Persians (23)** at
//! Difficulty 5, seed 12345, `STARTING_RESOURCES 7`) with the age held at
//! Medieval instead of All Technologies: `STARTING_TECHNOLOGY 2`,
//! `ENDING_TECHNOLOGY 2`. Captures: run1180 (a `DUMP_ALL` start) and
//! run1181 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §308.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const PERSIAN_MEDIEVAL: Pair = Pair {
    name: "EastIndiesPersianMedieval",
    map_style: 18,
    ai_tribe: 23,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1180-eastindies-persian-medieval-start.txt",
    long: (
        "gamelog-run1181-eastindies-persian-medieval-trace.txt",
        "rontrace-run1181.log",
    ),
    // The game ended itself on 4759 (closing block 4760).
    length: 4759,
    // Item 3535, first try: frame 2226 by the count and the sequence, ours
    // 10 draws against 8, index 0 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Guy::set_anim+0x97a < Unit::do_guard+0x7f4`
    // (`docs/AI.md` section 308). Item 3550, 2226 -> 4038 (+1812): the
    // ferried Explorer `1/0` keeps its army's `ATTACKTOORDER`, the boat's
    // orders queued behind it (`eject_contents`' step 4). Frame 4038: ours 8
    // draws against 9, index 0 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Unit::fight+0x9b0`. Item 3554, 4038 ->
    // 4083 (+45): a `k` type's fire on the move survives an `ATTACK` action
    // under its move (`do_move`'s exemption), so the Mahouts `1/27` and
    // `1/46` keep `cavarch_o` -2 through 4001 and `1/27` fires on the
    // Citizen `0/10` on 4038 as theirs does. Frame 4083: ours 7 draws
    // against 8, index 0 — ours `Guy::set_anim+0x97a < Unit::do_idle+0x7d`,
    // theirs `Unit::fight+0x9b0`. Item 3555, 4083 -> 4170 (+87): the Persian
    // Peltasts' piece 1794 and the Mahout's piece 212 leave from their bays
    // (`launch::BAYS`), so `1/22`'s javelin strikes `0/2000` on 4047 (13
    // frames, not 14) and `1/27`'s round the Citizen `0/10` on 4059 (8, not
    // 9). Frame 4170: ours 8 draws against 6, index 0 — ours
    // `Guy::set_anim+0x97a < Unit::fight+0x169`, theirs `Unit::fight+0x824`.
    // Item 3568, 4170 -> 4523 (+353): a melee-and-ranged type walks past
    // `fight`'s reload gate (`unit_flags & 0x400`), so the Mahout `1/46`,
    // recharging 35, rolls the guard's `fight+0x824` on 4170 and stands on
    // (6648, 7800) on 4171 as theirs does. Frame 4523: ours 49 draws
    // against 45, index 3 — ours `Unit::fight+0x824`, theirs
    // `Guy::set_anim+0xf2f < Guy::move+0x166`. Item 3570, 4523 -> 4759
    // (+236), **the pair closes**: a type whose graphic releases a round
    // lands no melee blow (`fight@005fd4d0:853`, `UnitType +0x2cc` zeroed by
    // `init_unit_events`), and `do_attack`'s tail runs a `k` type's
    // `cavarch_fight` after a strike, so on run1189b's block 4241 `0/2000`
    // reads `damage` 108 and `1/46` `cavarch_o` 2000 on both sides, where
    // ours read 112 and -2.
    count: 4759,
    sequence: 4759,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const PERSIAN_MEDIEVAL_PAIR_WORD: i64 =
    if PERSIAN_MEDIEVAL.count < PERSIAN_MEDIEVAL.sequence {
        PERSIAN_MEDIEVAL.count
    } else {
        PERSIAN_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn persian_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &PERSIAN_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "PERSIAN_MEDIEVAL {}: count {} sequence {} of {}; {}",
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
fn persian_medieval_pair_captures_say_their_lobby() {
    let p = &PERSIAN_MEDIEVAL;
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
        "the Persian Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: its triple is the coverage pair's (item 1466),
/// which the held age alone tells apart.
#[test]
fn persian_medieval_pair_is_no_other_lobby() {
    let p = &PERSIAN_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// run1181b: the word 2226's window, blocks 2220..2476 at the long's
/// detail — six blocks before the word's block 2227 and 250 after (item
/// 3535).
pub(crate) const RUN1181B: &str =
    "gamelog-run1181b-eastindies-persian-medieval-window-2220-2476.txt";

/// The window: block 2220 through 2476; the word 2226's own block is 2227.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_FRAME_2226: (i64, i64) = (2220, 2476);

/// **The word 2226's window**: run1181b walked from run1180's start with
/// the recorder on (item 3535).
pub(crate) fn persian_medieval_frame_2226_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1181b",
        &[(RUN1181B, WIDENING_PERSIAN_MEDIEVAL_FRAME_2226.0)],
        WIDENING_PERSIAN_MEDIEVAL_FRAME_2226,
        1,
        &[2227],
        true,
    )
}

#[test]
fn persian_medieval_pair_run1181b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_frame_2226_window() else {
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
    // Item 3535, the first walk: 1130 keys. Block 2220 carries 79 rows:
    // the standing families (`form`, `SITE[i].reg`, who=1's groups'
    // `held`) and **the Explorer `1/0` (TypeIndex 71, land)**, whose
    // orders are already parted: theirs holds two — the
    // `EXPLORETOORDER` to (34296, 33528) under a current
    // `ATTACKTOORDER` to (42072, 43368), its path 16 nodes — and ours the
    // explore order alone (`order:kind` ours 3 theirs 2, a path of 1 to
    // (34272, 33504)). On 2226 both reach (34272, 33504); ours' order is
    // done and the unit idles (`Unit::do_idle`), theirs walks on
    // (`heading` ours -1073741824 theirs 1962213376, `orders_x` 34272
    // against 34296), and block 2227 reads `idle` ours 1 theirs 0. The
    // long's stride blocks place theirs' two orders between 2101 (none)
    // and 2201 (both); no dump on disk covers the span at detail
    // (`docs/AI.md` section 308). Item 3550, 1130 -> 86: the Explorer's
    // orders agree (run1189 puts theirs' push on 2172, the landing), and
    // what is left is the standing families alone — 74 rows on 2220
    // (`form`, `leader`, who=1's groups' `held` and `pool`), and the
    // births' `form` and groups' `held` as they come.
    pin_eq!(w.firsts.len(), 86, "run1181b baseline");
}

/// run1189: the landing's window, blocks 2160..2220 at the long's detail
/// (item 3550) — the span the stride (2101, 2201) left undumped, in which
/// theirs' Explorer `1/0` came out of the Transport Barge `1/45` holding an
/// `ATTACKTOORDER` ours never pushed.
pub(crate) const RUN1189: &str = "gamelog-run1189-eastindies-persian-medieval-window-2160-2220.txt";

/// The window: block 2160 through 2220; the landing's block is 2172.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_LANDING_2172: (i64, i64) = (2160, 2220);

/// **The landing's window**: run1189 walked from run1180's start with the
/// recorder on (item 3550).
pub(crate) fn persian_medieval_landing_2172_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1189",
        &[(RUN1189, WIDENING_PERSIAN_MEDIEVAL_LANDING_2172.0)],
        WIDENING_PERSIAN_MEDIEVAL_LANDING_2172,
        1,
        &[2172],
        true,
    )
}

/// **The ferried Explorer lands on its army's attack** (item 3550,
/// `docs/AI.md` section 308): on run1189's block 2172 theirs' `1/0`
/// (TypeIndex 71) is ashore at (34728, 32376) in the Mahout `1/72`'s group,
/// with `ATTACKTOORDER` (42072, 43368), angle 1744502784, current over the
/// barge's `EXPLORETOORDER` (34296, 33528), and a 17-node path whose top is
/// the barge's (34296, 32760). `come_out`'s tail joins the army and
/// `go_to_unit` pushes the attack; `eject_contents`' step 4 then queues the
/// boat's orders and path **behind** it. Before item 3550 ours replaced the
/// list with the boat's, and the Explorer's `order:kind` and `orders.len`
/// parted on this block. No row on the Explorer parts in the window.
#[test]
fn persian_medieval_pair_run1189_lands_the_explorer_on_its_attack() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_landing_2172_window() else {
        return;
    };
    pin_eq!(w.blocks, 60, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    let explorer: Vec<_> = w
        .firsts
        .iter()
        .filter(|((who, o, _), _)| (*who, *o) == (1, 0))
        .map(|((_, _, what), (f, r))| format!("{f} {what}: {r}"))
        .collect();
    assert_eq!(explorer, Vec::<String>::new(), "the Explorer 1/0 parts");
    // Item 3550: 77 keys, the standing families alone — 72 on 2160
    // (`form`, `leader`, who=1's groups' `held` and `pool`), ours' groups 77
    // and 78 on 2164 and 2172 (78 is the Explorer's `go_to` group, which
    // theirs' pool prints as no list) and `SITE[8].val` on 2176. The
    // Explorer's orders agree on 2172.
    pin_eq!(w.firsts.len(), 77, "run1189 baseline");
}

/// run1189b: the word 4038's window, blocks 4033..4289 at the long's
/// detail — six blocks before the word's block 4039 and 250 after (item
/// 3550); the stride's blocks are 4001 and 4101.
pub(crate) const RUN1189B: &str =
    "gamelog-run1189b-eastindies-persian-medieval-window-4033-4289.txt";

/// The window: block 4033 through 4289; the word 4038's own block is 4039.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_FRAME_4038: (i64, i64) = (4033, 4289);

/// **The word 4038's window**: run1189b walked from run1180's start with
/// the recorder on (item 3550).
pub(crate) fn persian_medieval_frame_4038_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1189b",
        &[(RUN1189B, WIDENING_PERSIAN_MEDIEVAL_FRAME_4038.0)],
        WIDENING_PERSIAN_MEDIEVAL_FRAME_4038,
        1,
        &[4039],
        true,
    )
}

#[test]
fn persian_medieval_pair_run1189b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_frame_4038_window() else {
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
    // Item 3550, the first walk: 1184 keys. Block 4033 carries 101 rows,
    // the standing families and two already parted: the Mahouts `1/27` and
    // `1/46` (TypeIndex 262, land) read `cavarch_o` ours -1 theirs -2. On
    // the word 4038 theirs' `1/27`, at (5241, 9419) on its
    // `ATTACKTOORDER`, takes the Citizen `0/10` (TypeIndex 50, land) at
    // (5016, 8088): `cavarch_who` ours -1 theirs 0, `g.ox[0]` -1 against
    // 10, `g.whom[0]` -1 against 0 — `Unit::fight+0x9b0`, index 0 of
    // theirs' 9 draws against ours' 8 — and block 4039 reads `fought`,
    // `blown` ours 0 theirs 1 and `recharging` 0 against 40 (`docs/AI.md`
    // section 308). Item 3554, 1184 -> 1312: the Mahouts' `cavarch_o` rows
    // leave 4033 (99 rows there, the standing families), and so do `1/27`'s
    // shot on 4038 and its reload on 4039. The window now parts on 4048:
    // the Nubian building `0/2000`'s `build:damage` ours 3 theirs 6
    // (`damage_frac` 5 against 10) and `city:raid_stamp` ours 4044 theirs
    // 4047; then the Citizen `0/10` on 4060 (`hits_left` ours 50 theirs
    // 25, `damage_frame` 0 against 4059) and its flight on 4061; `1/46`
    // on 4067 (`pos` ours (5469, 9811) theirs (5468, 9811)); and the word
    // 4083's own block 4084, `1/46` taking `cavarch_o` 2005 here against
    // 10 there (4083) and firing, `fought`/`blown` ours 1 theirs 0. Item
    // 3555, 1312 -> 745: the Persian Peltasts' javelins (piece 1794) and the
    // Mahouts' rounds (piece 212) leave from their bays, and `0/2000`'s
    // damage reads 6 (`damage_frac` 10) on 4048 and the Citizen `0/10`'s
    // `hits_left` 25 on 4060 on both sides. Through 4170 only the standing
    // families stand (`form`, the ten `SITE[i].reg`, who=1's groups' `held`
    // and `pool`); the first state parting is `1/46`'s `pos` on 4171, ours
    // (6657, 7777) against theirs (6648, 7800), and on 4173 theirs' `1/46`
    // attacks `0/2000` (`g.ox[0]` 2000, `in_range` 1) where ours' takes
    // `0/11` (`new_ord` 1). Item 3568, 745 -> 119: the Mahout walks past
    // `fight`'s reload gate (`unit_flags & 0x400`), and `1/46`'s rows on
    // 4171 and 4173 leave — (6648, 7800) on 4171 and `in_range` 1 on 4173
    // on both sides. The window now parts on 4241: `0/2000`'s
    // `build:damage` ours 112 theirs 108 (`city:raid_stamp` ours 4240
    // theirs 4238), and `1/46`'s `cavarch_o` ours -2 theirs 2000 — theirs'
    // Mahout fires there through `cavarch_fight` (`recharging` 1 -> 40).
    // Item 3570, 119 -> 116: both 4241 rows leave. The Mahout's melee swing
    // lands nothing (its graphic releases the round: `UnitType +0x2cc` is
    // 0), so `0/2000` reads 108 there on both sides, and `do_attack`'s tail
    // runs its `cavarch_fight`, so `1/46` reads `cavarch_o` 2000 (and `1/27`
    // on 4242). The window now parts on 4283, the sixteenths of one frame:
    // `0/2000`'s `build:damage` ours 145 theirs 146 (`damage_frac` 14
    // against 8), both 149 13/16 on 4284.
    pin_eq!(w.firsts.len(), 116, "run1189b baseline");
    // The count cannot see the Peltasts' bays: `0/2000`'s `build:damage`
    // parts later on either tree, so the key set is the same. Its frame is
    // the pin — 4048 when the javelins leave the figure, 4193 from the bay,
    // 4241 with the Mahout past the reload gate (item 3568), 4283 with its
    // melee swing landing nothing (item 3570).
    pin_eq!(
        w.firsts
            .get(&(0, 2000, "build:damage".to_string()))
            .map(|(f, _)| *f),
        Some(4283),
        "`0/2000`'s damage parts past the word"
    );
}

/// run1211: the word 4523's window, blocks 4518..4760 at the long's
/// detail — six blocks before the word's block 4524, to the game's own
/// closing block (item 3568); the stride's blocks are 4501 and 4601.
pub(crate) const RUN1211: &str = "gamelog-run1211-eastindies-persian-medieval-window-4518-4760.txt";

/// The window: block 4518 through the closing block 4760; the word 4523's
/// own block is 4524.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_FRAME_4523: (i64, i64) = (4518, 4760);

/// **The word 4523's window**: run1211 walked from run1180's start with
/// the recorder on (item 3568).
pub(crate) fn persian_medieval_frame_4523_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1211",
        &[(RUN1211, WIDENING_PERSIAN_MEDIEVAL_FRAME_4523.0)],
        WIDENING_PERSIAN_MEDIEVAL_FRAME_4523,
        1,
        &[4524],
        true,
    )
}

#[test]
fn persian_medieval_pair_run1211_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_frame_4523_window() else {
        return;
    };
    pin_eq!(w.blocks, 242, "every captured block");
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
    // Item 3568, the first walk: 1729 keys, 104 of them on block 4518.
    // Past the standing families (`form`, the `SITE[i].reg`, who=1's
    // groups' `held` and `pool`), the window opens parted: the Nubian
    // capital `0/2000`'s `build:damage` ours 629 theirs 573, and the
    // Mahouts `1/27` and `1/46` (TypeIndex 262) reading `cavarch_o` ours
    // 11 and -2 against theirs 2000, as run1189b's 4241 left them. On the
    // word's own blocks the Knight `1/62` (TypeIndex 228, land, uid 98),
    // under its `GUARDORDER`, holds its `ATTACKORDER` (kind 10, two
    // orders) on 4523 where theirs has pushed a `MOVEORDER` to (5352,
    // 6024) over the attack on the Citizen `0/4` (three orders, `pos`
    // (4803, 6436) against ours (4776, 6456)); on 4524 ours names `0/3`.
    // Item 3570, 1729 -> 112, the pair closed: the Mahouts' `cavarch_o` and
    // the Knight's rows leave. Past the standing families, `0/2000`'s
    // sixteenths alone: `damage_frac` ours 5 theirs 15 on 4518, and
    // `damage` 584 against 585 on 4530.
    pin_eq!(w.firsts.len(), 112, "run1211 baseline");
}

/// **The long's stride, walked back from the word 4038** (item 3554):
/// run1181 dumps `UNITS=3,GUYS=2` every hundredth block, and run1189b's
/// window opens with the Mahouts' `cavarch_o` already apart, so the state
/// parted before 4033 and only the stride's blocks are on disk there.
pub(crate) const WIDENING_PERSIAN_MEDIEVAL_STRIDE: (i64, i64) = (101, 4001);

pub(crate) fn persian_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[PERSIAN_MEDIEVAL.start],
        true,
        PERSIAN_MEDIEVAL.long,
        "run1181 stride",
        &[(PERSIAN_MEDIEVAL.long.0, WIDENING_PERSIAN_MEDIEVAL_STRIDE.0)],
        WIDENING_PERSIAN_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn persian_medieval_pair_the_long_s_stride_walks_back_from_4038() {
    let _pins = Pins::hold();
    let Some(w) = persian_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 40, "every stride block in the window");
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
    // Item 3554: 155 keys before the fix, the standing families (every
    // unit's `form`, the `build:extra` rows a `UNITS=3` stride prints no
    // building for, who=1's groups' `held` and `pool`) and the Mahouts
    // `1/27` and `1/46` (TypeIndex 262, `mjvyk`) on 4001: `cavarch_o` ours
    // -1 theirs -2, under `[AttackTo, Attack(0/10), Move]`. Both read -2
    // on every stride block 1501..3901 (`1/46` from 1801). The fix is
    // `do_move`'s `k` exemption; 153 keys, the standing families alone.
    pin_eq!(w.firsts.len(), 153, "the stride's baseline");
}
