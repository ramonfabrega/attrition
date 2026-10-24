//! **The Bantu from an Ancient start, East Indies** (DECISIONS 90, 91,
//! 93, item 3562): East Indies, `MAP_STYLE 18`, human Nubians (4) against
//! the **Bantu (3)** at Difficulty 5, seed 12345, `STARTING_RESOURCES 7`,
//! `STARTING_TECHNOLOGY 0` — the Koreans' Ancient-start lobby with the
//! nation moved. The profile's `ENDING_TECHNOLOGY=0` reads back as
//! `ENDING_TECHNOLOGY 1` (3559a: `SetupWin::setup_game` stores the combo's
//! index plus one), so this is an Ancient start capped at Classical, not a
//! held age. The Bantu's other scored lobby is `bantu_word`'s
//! `GreatSaharaBantuTier4`, another map. Captures: run1205 (a `DUMP_ALL`
//! start) and run1206 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §315.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const BANTU_ANCIENT: Pair = Pair {
    name: "EastIndiesBantuAncient",
    map_style: 18,
    ai_tribe: 3,
    difficulty: 5,
    starting_technology: 0,
    // Asked 0; the lobby's floor is 1 (`SetupWin::setup_game`, 3559a).
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1205-eastindies-bantu-ancient-start.txt",
    long: (
        "gamelog-run1206-eastindies-bantu-ancient-trace.txt",
        "rontrace-run1206.log",
    ),
    // The game ended itself on 6210 (closing block 6211).
    length: 6210,
    // Item 3562, first try: frame 4185 by the count and the sequence, ours
    // 350 draws against 342, index 242 — ours `Leader::make_stuff+0x63d`,
    // theirs `Leader::produce_building+0x1805` (`docs/AI.md` section 315).
    // Item 3589: **5072** (+887) — the lobby's type ages
    // (`Types::finalize_grafting`, `TechTree::age_in`): the Trireme is an
    // age-3 type in this lobby, so `upgrade_units` values the Galley at 0
    // where ours offered it at 21504000 on 4179. Ours 39 draws against 40,
    // index 31, ours `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823`.
    // Item 3591: **5322** (+250) — the Umpakati's and the Yangombi
    // Umpakati's bays (`launch::BAYS`, pieces 43 and 44): `1/86`'s javelin
    // at the Citizen `0/11` leaves from (6799, 8469) and flies 15 frames,
    // where it left the figure's own square and flew 16. Ours 26 draws
    // against 27, index 22, ours `Farms::inc_time+0x1ae`, theirs
    // `Object::take_damage+0xe1`.
    count: 5322,
    sequence: 5322,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BANTU_ANCIENT_PAIR_WORD: i64 = if BANTU_ANCIENT.count < BANTU_ANCIENT.sequence {
    BANTU_ANCIENT.count
} else {
    BANTU_ANCIENT.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn bantu_ancient_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BANTU_ANCIENT;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BANTU_ANCIENT {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, `ENDING_TECHNOLOGY` 1 beside `STARTING_TECHNOLOGY` 0, and the
/// long is scoreable.
#[test]
fn bantu_ancient_pair_captures_say_their_lobby() {
    let p = &BANTU_ANCIENT;
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
        "the Bantu Ancient pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery fields no Bantu, and the Bantu's one
/// other lobby, `bantu_word`'s `GreatSaharaBantuTier4`, is `(7, 3, 4)` —
/// another map and difficulty (item 3562).
#[test]
fn bantu_ancient_pair_is_no_other_lobby() {
    let p = &BANTU_ANCIENT;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(!battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe));
    let other = &super::bantu_word::BANTU;
    assert_ne!((other.map_style, other.ai_tribe, other.difficulty), triple);
}

/// run1206b: the word 4185's window, blocks 4180..4436 at the long's
/// detail — six blocks before the word's block 4186 and 250 after (item
/// 3562); the start dumps blocks 0..2 alone and the long's stride prints
/// 4101 and 4201.
pub(crate) const RUN1206B: &str = "gamelog-run1206b-eastindies-bantu-ancient-window-4180-4436.txt";

/// The window: block 4180 through 4436; the word 4185's own block is 4186.
pub(crate) const WIDENING_BANTU_ANCIENT_FRAME_4185: (i64, i64) = (4180, 4436);

/// **The word 4185's window**: run1206b walked from run1205's start with
/// the recorder on (item 3562).
pub(crate) fn bantu_ancient_frame_4185_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1206b",
        &[(RUN1206B, WIDENING_BANTU_ANCIENT_FRAME_4185.0)],
        WIDENING_BANTU_ANCIENT_FRAME_4185,
        1,
        &[4186],
        true,
    )
}

#[test]
fn bantu_ancient_pair_run1206b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_frame_4185_window() else {
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
    // Item 3562, the first walk: 1368 keys. Block 4180, the window's
    // first, carries 115 rows: the standing families (`form`,
    // `SITE[i].reg`, both leaders' `aggression`, who=1's groups' `held`
    // and pools) and who=1's make list, already apart — `MAKE[0]` ours
    // cat 7 t 341 (the Galley) val 21504000 against theirs cat 6 t 596 val
    // 9999999, and `MAKE[7]` ours t 341 val 21504000 against theirs t 94
    // (the Yangombi Umpakati) val 2016000. On 4183 ours queues at `1/2014`
    // where theirs does not (`queue:queued` 1 against 0, `bucket` timber
    // and metal 80 lower). On the word's block 4186 theirs holds two new
    // buildings and ours one: theirs' `1/2054` is a Stable (428, city 3,
    // `constr_time` 42000, `myhits` 1200) at (35136, 38976) and its
    // `1/2055` the Hanging Gardens (528, city 1, `constr_time` 400000) at
    // (34560, 37632); ours' `1/2054` is the Hanging Gardens, at (34752,
    // 37824), and ours places no Stable.
    //
    // Item 3589: **113**, from the lobby's type ages. Who=1's make list agrees
    // through the window (ours no longer offers the Galley on 4179), the
    // Stable and the Hanging Gardens are both sides', and what is left is
    // the standing families: 111 rows on 4180 (every unit's `form`,
    // `SITE[i].reg`, both leaders' `aggression`, who=1's groups' `held`
    // and pools), group `75`'s `held` and pool on 4186, and on 4360
    // `1/175`'s `form`, ours -1 against theirs 0.
    pin_eq!(w.firsts.len(), 113, "run1206b baseline");
}

/// run1236: who=1's make list walked back, blocks 3970..4186 at the long's
/// detail with `LEADERS=9` (item 3589) — two make passes before the
/// word, the Classical age on 3991 between them, and six blocks over
/// run1206b's first. Its packet, at logger frame 4179, is the original's
/// `upgrade_units` on the state before tick 4179 (`docs/AI.md` §315).
pub(crate) const RUN1236: &str = "gamelog-run1236-eastindies-bantu-ancient-window-3970-4186.txt";

/// The make list's window: block 3970 through 4186.
pub(crate) const WIDENING_BANTU_ANCIENT_MAKE_4179: (i64, i64) = (3970, 4186);

/// **The make list walked back**: run1236 walked from run1205's start
/// with the recorder on (item 3589).
pub(crate) fn bantu_ancient_make_4179_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1236",
        &[(RUN1236, WIDENING_BANTU_ANCIENT_MAKE_4179.0)],
        WIDENING_BANTU_ANCIENT_MAKE_4179,
        1,
        &[4180],
        true,
    )
}

#[test]
fn bantu_ancient_pair_run1236_s_make_list_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_make_4179_window() else {
        return;
    };
    // Blocks 3970..4185: frame `f` writes block `f + 1`, and the walk
    // stops at the window's tail.
    pin_eq!(w.blocks, 216, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    // No `leader:MAKE` row on any block: who=1's make list agrees from
    // 3970 to 4186, through both passes. Before item 3589 it parted on
    // 4180 (`MAKE[0]` and `MAKE[7]` ours the Galley 341 at 21504000,
    // theirs Allegiance 596 at 9999999 and the Yangombi Umpakati 94 at
    // 2016000).
    let make: Vec<_> = w
        .firsts
        .keys()
        .filter(|(_, _, what)| what.starts_with("leader:MAKE"))
        .collect();
    assert_eq!(make, Vec::<&(i64, i64, String)>::new(), "who=1's make list");
    // 115 keys, the standing families alone: 111 on 3970 (every unit's
    // `form`, `SITE[i].reg`, both leaders' `aggression`, who=1's groups'
    // `held` and pools) and two groups' `held` and pool on 4034 and 4106
    // (`80`: `[47]`, `86`: `[30]`), which theirs never prints.
    pin_eq!(w.firsts.len(), 115, "run1236 baseline");
}

/// run1236b: the word 5072's window, blocks 5066..5322 at the long's
/// detail — six blocks before the word's block 5073 and 250 after (item
/// 3589).
pub(crate) const RUN1236B: &str = "gamelog-run1236b-eastindies-bantu-ancient-window-5066-5322.txt";

/// The window: block 5066 through 5322; the word 5072's own block is 5073.
pub(crate) const WIDENING_BANTU_ANCIENT_FRAME_5072: (i64, i64) = (5066, 5322);

/// **The word 5072's window**: run1236b walked from run1205's start with
/// the recorder on (item 3589).
pub(crate) fn bantu_ancient_frame_5072_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1236b",
        &[(RUN1236B, WIDENING_BANTU_ANCIENT_FRAME_5072.0)],
        WIDENING_BANTU_ANCIENT_FRAME_5072,
        1,
        &[5073],
        true,
    )
}

#[test]
fn bantu_ancient_pair_run1236b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_frame_5072_window() else {
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
    // Item 3589, the first walk: 4465 keys. The word's block 5073 parts
    // `1/86`'s `collide_o` and `collide_who`, ours -1 and -1 against theirs
    // 87 and 1. The state is already apart on 5066, the window's first
    // block: beside the standing families, the human's `0/10` and `0/11`
    // (`damage_frame` ours 5056 and 5036 against theirs 5055 and 5035, and
    // their positions) and who=1's `1/86`..`1/88`, whose move orders' `y`
    // are 48 apart (`1/86` ours 7896 against theirs 7848).
    //
    // Item 3591: **97**, from the Umpakati's bays. `1/86`'s javelin strikes
    // `0/11` on 5035 on both sides, and on the old word's block 5073
    // `1/86`'s `collide_o` and `collide_who` agree. The standing families
    // (84 rows on 5066, and two groups' `held` and pool on 5084 and 5086)
    // and nine rows after them: the first is `1/86`'s `order:target` on
    // 5107, ours `0/13` against theirs `0/11`; then `1/41`'s target on
    // 5253 (ours `0/3`, theirs `0/2000`), the `dest_angle`s of `1/110`,
    // `1/140` (5279) and `1/107` (5303), and `0/2000`'s `damage_frac`,
    // `raid_stamp` (5299) and `damage` (5315, ours 30 against 31).
    pin_eq!(w.firsts.len(), 97, "run1236b baseline");
}

/// **The long's stride, walked back from the word 5072** (item 3591):
/// run1236b's window opens apart on 5066, and only run1206's stride blocks
/// are on disk between run1236's 4186 and run1236b's 5066.
pub(crate) const WIDENING_BANTU_ANCIENT_STRIDE: (i64, i64) = (101, 5001);

pub(crate) fn bantu_ancient_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1206 stride",
        &[(BANTU_ANCIENT.long.0, WIDENING_BANTU_ANCIENT_STRIDE.0)],
        WIDENING_BANTU_ANCIENT_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn bantu_ancient_pair_the_long_s_stride_walks_back_from_5072() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 50, "every stride block in the window");
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
    // Item 3591: 182 keys, the standing families alone — every unit's
    // `form` and `build:extra` — from 101 through 5001: no unit field
    // parts on any stride block, so the state run1236b opens apart on
    // 5066 parts after 5001.
    pin_eq!(w.firsts.len(), 182, "the stride's baseline");
}

/// run1238: the stride's gap walked, blocks 4995..5073 at the long's
/// detail with `LEADERS=9` (item 3591) — six blocks over the stride's
/// 5001, and through run1236b's first block 5066 to the word's 5073.
pub(crate) const RUN1238: &str = "gamelog-run1238-eastindies-bantu-ancient-window-4995-5073.txt";

/// The window: block 4995 through 5073.
pub(crate) const WIDENING_BANTU_ANCIENT_GAP_5066: (i64, i64) = (4995, 5073);

/// **The state's parting walked back from 5066**: run1238 walked from
/// run1205's start with the recorder on (item 3591).
pub(crate) fn bantu_ancient_gap_5066_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BANTU_ANCIENT.start],
        true,
        BANTU_ANCIENT.long,
        "run1238",
        &[(RUN1238, WIDENING_BANTU_ANCIENT_GAP_5066.0)],
        WIDENING_BANTU_ANCIENT_GAP_5066,
        1,
        &[5066, 5073],
        true,
    )
}

#[test]
fn bantu_ancient_pair_run1238_walks_the_parting_back_from_5066() {
    let _pins = Pins::hold();
    let Some(w) = bantu_ancient_gap_5066_window() else {
        return;
    };
    // Blocks 4995..5072: frame `f` writes block `f + 1`, and the walk
    // stops at the window's tail.
    pin_eq!(w.blocks, 78, "every captured block");
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
    // Item 3591, the first walk: 181 keys. Beside the standing families,
    // the first parted field is on block 5036: both leaders'
    // `treaties[·]` ours 1 against theirs 3 (bit 2, the struck mark) and
    // the Citizen `0/11`'s blow — `damage_frame` ours 0 against theirs
    // 5035, `hits_left` 40 against 37 — and its flight (`orders_x`,
    // `orders_y` ours 7320, 7320 against 7944, 5928). The round is
    // `1/86`'s (TI 93, land), thrown on 5021 on both sides: run1238b's
    // `AMMO=5` prints it leaving (6799, 8469) with `total_time` 15, and
    // ours threw from the figure's own square (6792, 8520) for 16.
    //
    // With the Umpakati's bays: **92**, the standing families alone.
    pin_eq!(w.firsts.len(), 92, "run1238 baseline");
}
