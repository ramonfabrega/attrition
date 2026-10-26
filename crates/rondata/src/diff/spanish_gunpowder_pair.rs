//! **The Spanish at a held Gunpowder age, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3627): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Spanish (9)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `mongol_gunpowder_pair`'s lobby with the
//! nation moved. Its triple, `(18, 9, 5)`, is new: the battery fields no
//! Spanish, `spanish_east_word`'s `EastIndiesSpanishTier3` is `(18, 9, 3)`,
//! another difficulty, `spanish_classical_pair`'s `(7, 9, 5)` is another
//! map, and `spanish_word`'s `(7, 9, 3)` and `spanish_lakes_word`'s
//! `(14, 9, 4)` are both. Captures: run1286 (a `DUMP_ALL` start) and
//! run1287 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's initial
//! state). The first try is in `docs/AI.md` §335.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_GUNPOWDER: Pair = Pair {
    name: "EastIndiesSpanishGunpowder",
    map_style: 18,
    ai_tribe: 9,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1286-eastindies-spanish-gunpowder-start.txt",
    long: (
        "gamelog-run1287-eastindies-spanish-gunpowder-trace.txt",
        "rontrace-run1287.log",
    ),
    // The game ended itself on 6128 (closing block 6129).
    length: 6128,
    // Item 3627, first try: frame 2168, by the count and the sequence —
    // ours 35 draws against 34, parting at index 30 (`docs/AI.md` §335).
    // Item 3640: 2168 → 3877 by the count and **3866** by the sequence —
    // `think_carry`'s army arm puts the Transport Galleon `1/68`'s
    // Arquebusiers ashore on 2163 and the emptied boat dies (§335.1). On
    // 3866 both sides spend 11 draws and part at index 0: ours `Guy::
    // set_anim+0x97a < Guy::inc_time+0x271`, theirs `Ammo::init+0xcd9`.
    // Item 3642: 3866 → **3876** by both — a computer's raiding ship
    // zeroes a building only when it is a dock (`compare_target`'s
    // vslot `+0x108` is `is_dock`), so the Frigate `1/29` takes the
    // human's Farm `0/2004` on 3859 and fires with theirs (§335.2). On
    // 3876 ours spends 8 draws against 7, parting at index 4: ours
    // `Object::take_damage+0xe1` (its round lands on the Farm), theirs
    // `Farms::inc_time+0x1ae` (its round lands on 3877).
    // Item 3643: 3876 → **4859** by both — the Frigate's piece 293 fires
    // its broadside from `launch::BAYS`' rows (run1304's packet), so the
    // round leaves (1809, 5233) for 12 frames with theirs and lands on
    // 3877 (§335.3). On 4859 ours spends 16 draws against 18, parting at
    // index 7: ours `Unit::fight+0x9b0`, theirs `Unit::come_out+0x25ca`.
    // Item 3646: 4859 → **6128**, the game's end, by both — a sea-only
    // attacker (`unit_flags & 0x102000`) takes only a sea unit
    // (`valid_target_const`'s `6473c3` arm), so the Fire Ship `1/86` is
    // not handed its group's Citizen `0/12` on 4859 and spends no
    // `fight+0x9b0` (§335.4). The pair closes.
    count: 6128,
    sequence: 6128,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const SPANISH_GUNPOWDER_PAIR_WORD: i64 =
    if SPANISH_GUNPOWDER.count < SPANISH_GUNPOWDER.sequence {
        SPANISH_GUNPOWDER.count
    } else {
        SPANISH_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn spanish_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &SPANISH_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "SPANISH_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn spanish_gunpowder_pair_captures_say_their_lobby() {
    let p = &SPANISH_GUNPOWDER;
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
        "the Spanish Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3627; the battery's identity
/// test refuses it) and none of the battery's: the battery fields no
/// Spanish since `EastIndiesSpanishTier3`, `(18, 9, 3)`, left it for the AI
/// word (another difficulty); the Classical pair's `(7, 9, 5)` is another
/// map, and `GreatSaharaSpanishTier3`'s `(7, 9, 3)` and
/// `GreatLakesSpanishTier4`'s `(14, 9, 4)` are both (item 3627).
#[test]
fn spanish_gunpowder_pair_is_no_other_lobby() {
    let p = &SPANISH_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    assert!(
        !battery::BATTERY.iter().any(|l| l.ai_tribe == p.ai_tribe),
        "the battery fields no Spanish"
    );
    for (name, other) in [
        (
            super::spanish_word::SPANISH.name,
            (
                super::spanish_word::SPANISH.map_style,
                super::spanish_word::SPANISH.ai_tribe,
                super::spanish_word::SPANISH.difficulty,
            ),
        ),
        (
            super::spanish_lakes_word::SPANISH_LAKES.name,
            (
                super::spanish_lakes_word::SPANISH_LAKES.map_style,
                super::spanish_lakes_word::SPANISH_LAKES.ai_tribe,
                super::spanish_lakes_word::SPANISH_LAKES.difficulty,
            ),
        ),
        (
            super::spanish_east_word::SPANISH_EAST.name,
            (
                super::spanish_east_word::SPANISH_EAST.map_style,
                super::spanish_east_word::SPANISH_EAST.ai_tribe,
                super::spanish_east_word::SPANISH_EAST.difficulty,
            ),
        ),
        (
            super::spanish_classical_pair::SPANISH_CLASSICAL.name,
            (
                super::spanish_classical_pair::SPANISH_CLASSICAL.map_style,
                super::spanish_classical_pair::SPANISH_CLASSICAL.ai_tribe,
                super::spanish_classical_pair::SPANISH_CLASSICAL.difficulty,
            ),
        ),
    ] {
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 2168** (item 3627):
/// only run1287's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_STRIDE: (i64, i64) = (101, 2101);

pub(crate) fn spanish_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1287 stride",
        &[(
            SPANISH_GUNPOWDER.long.0,
            WIDENING_SPANISH_GUNPOWDER_STRIDE.0,
        )],
        WIDENING_SPANISH_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_the_long_s_stride_walks_back_from_2168() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 21, "every stride block in the window");
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
    // Item 3627: 100 keys, every one standing — the start's buildings'
    // `build:extra` (which the stride does not print), who=1's groups'
    // `held` and pools as each group is minted (64 on 101 through 78 on
    // 2101), and `form` on 30 units. No unit's position, orders or
    // containment parts through block 2101: the Arquebusiers `1/48`, `1/49`
    // and `1/50` (TypeIndex 98) agree there, `1/48` and `1/49` on an
    // `ATTACKTOORDER` to (40776, 37128).
    pin_eq!(w.firsts.len(), 100, "the stride's baseline");
}

/// run1287b: the word 2168's window, blocks 2163..2419 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 2169 and 250 after (item 3627); the stride's blocks are 2101 and
/// 2201.
pub(crate) const RUN1287B: &str =
    "gamelog-run1287b-eastindies-spanish-gunpowder-window-2163-2419.txt";

/// The window: block 2163 through 2419; the word 2168's own block is 2169.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_FRAME_2168: (i64, i64) = (2163, 2419);

/// **The word 2168's window**: run1287b walked from run1286's start with
/// the recorder on (item 3627).
pub(crate) fn spanish_gunpowder_frame_2168_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1287b",
        &[(RUN1287B, WIDENING_SPANISH_GUNPOWDER_FRAME_2168.0)],
        WIDENING_SPANISH_GUNPOWDER_FRAME_2168,
        1,
        &[2169],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1287b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_frame_2168_window() else {
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
    // Item 3627, the first walk: 1343 keys. Block 2163 carries the standing
    // families (`form` on 28 units, `SITE[i].reg`, who=0's
    // `aggression[1]`, who=1's groups' `held` and pools) and **the gap's
    // parting**: ours holds a Transport Galleon `1/68` (TypeIndex 321)
    // theirs never prints, with the Arquebusiers `1/48`, `1/49` and `1/50`
    // (TypeIndex 98) `inside` 68, where theirs' three stand on the shore,
    // `1/48` at (41688, 36360) against ours' (41928, 36300). Ours casts it:
    // `RON_DEBUG_UNIT` puts the Galleon's birth on 2153 at (42097, 36253),
    // through `do_cast_effect`'s `cast_transport`, carrying `1/48`'s
    // `ATTACKTOORDER`, in the gap 2102..2162 that no dump prints — ours
    // acts (`docs/AI.md` section 335). The first rows past it are `1/61`'s
    // `half_step` (2165) and `pos` (2166), and on **the word 2168**, ours
    // 35 draws against 34 at index 30, ours spends `1/63`'s `Guy::set_anim
    // < Unit::move_step`; on 2169 `1/63` collides with `1/68` (`collide_o`
    // 68, `path:length` 1 against 15). 1343 → **1347** on the tree that
    // merged item 3637, whose move row compares `retry` and `attempts`: four
    // rows past the word, `1/63`'s `retry` 0 against 8 (2194) and
    // `attempts` 0 against 3 (2202), and `1/21`'s `retry` 8 against 6
    // (2283) and `attempts` 0 against 3 (2289).
    //
    // Item 3640: 1347 → **85**, every one standing from 2163 — `form` on
    // 28 units, who=1's groups' `held` and pools, `SITE[i].reg` and who=0's
    // `aggression[1]`, then the same families as groups are minted (77 on
    // 2224 to 81 on 2311) and `form` on three more. `think_carry`'s army
    // arm now ejects at the shore: on 2163 ours' `1/48` stands at (41688,
    // 36360), `1/49` at (41688, 36168) and `1/50` at (41832, 36504), as
    // theirs, `inside` −1, and `1/68` is gone on both sides.
    pin_eq!(w.firsts.len(), 85, "run1287b baseline");
}

/// run1301: the gap 2102..2162 that the long's stride does not print, at
/// run1287b's detail (item 3640) — six blocks before the stride's block
/// 2101 and through 2170, past run1287b's first block 2163.
pub(crate) const RUN1301: &str =
    "gamelog-run1301-eastindies-spanish-gunpowder-window-2095-2170.txt";

/// The window: block 2095 through 2170.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_GAP: (i64, i64) = (2095, 2170);

/// **The gap's window**: run1301 walked from run1286's start with the
/// recorder on (item 3640).
pub(crate) fn spanish_gunpowder_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1301",
        &[(RUN1301, WIDENING_SPANISH_GUNPOWDER_GAP.0)],
        WIDENING_SPANISH_GUNPOWDER_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1301_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 75, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3640, on the tree that ejects: 76 keys, every one standing —
    // block 2095's `form` on 27 units, who=1's groups' `held` and pools,
    // `SITE[i].reg` and who=0's `aggression[1]`; group 70's on 2102, and
    // `form` on the births `1/67` (2108) and `1/68` (2153). **Theirs casts
    // the same Transport**: on 2152 both sides spend `1/48`'s `Guy::
    // set_anim < Unit::do_cast+0xc89` and the boat's `Guy::init_real`;
    // `1/48`, `1/49` and `1/50` are `inside` the Galleon `1/68` from 2153,
    // which sails to (41928, 36168) and stands on 2162 with no path and no
    // order, both sides. **The state parted on 2163** before this item:
    // theirs puts the three ashore and `1/68` dies; ours kept them aboard
    // (`think_carry_army`'s eject was a seam, `docs/AI.md` §335.1).
    pin_eq!(w.firsts.len(), 76, "run1301 baseline");
}

/// run1303: the gap 3802..3860 that the long's stride does not print, at
/// run1287b's detail (item 3642) — six blocks before the stride's block
/// 3801 and through the word 3866.
pub(crate) const RUN1303: &str =
    "gamelog-run1303-eastindies-spanish-gunpowder-window-3795-3866.txt";

/// The window: block 3795 through 3866.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_GAP_3866: (i64, i64) = (3795, 3866);

/// **The second gap's window**: run1303 walked from run1286's start with
/// the recorder on (item 3642).
pub(crate) fn spanish_gunpowder_gap_3866_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1303",
        &[(RUN1303, WIDENING_SPANISH_GUNPOWDER_GAP_3866.0)],
        WIDENING_SPANISH_GUNPOWDER_GAP_3866,
        1,
        &[],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1303_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_gap_3866_window() else {
        return;
    };
    pin_eq!(w.blocks, 71, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3642, on the tree that weighs the Farm: 89 keys, every one
    // standing — block 3795's `form` on 30 units, who=1's groups' `held`
    // and pools, `SITE[i].reg` and who=0's `aggression[1]`, and `form` on
    // the birth `1/100` (3814). Before the build, 110: **the state parted
    // on 3859**, `1/29`'s `orders.len` ours 0 against 1 — on its first
    // idle frame after its `ATTACKTOORDER` ends on 3858 (both sides),
    // theirs' search takes the Farm `0/2004` and ours' weighed it at 0
    // (`docs/AI.md` §335.2).
    pin_eq!(w.firsts.len(), 89, "run1303 baseline");
}

/// run1301b: the word 3866's window, blocks 3861..4117 at run1287b's
/// detail — six blocks before the word's block 3867 and 250 after (item
/// 3640); the stride's blocks are 3801 and 3901.
pub(crate) const RUN1301B: &str =
    "gamelog-run1301b-eastindies-spanish-gunpowder-window-3861-4117.txt";

/// The window: block 3861 through 4117; the word 3866's own block is 3867.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_FRAME_3866: (i64, i64) = (3861, 4117);

/// **The word 3866's window**: run1301b walked from run1286's start with
/// the recorder on (item 3640).
pub(crate) fn spanish_gunpowder_frame_3866_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1301b",
        &[(RUN1301B, WIDENING_SPANISH_GUNPOWDER_FRAME_3866.0)],
        WIDENING_SPANISH_GUNPOWDER_FRAME_3866,
        1,
        &[3867],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1301b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_frame_3866_window() else {
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
    // Item 3640, the first walk: 738 keys. Block 3861 carries the standing
    // families (`form` on 33 units, 16 groups' `held` and pools,
    // `SITE[i].reg`, who=0's `aggression[1]`) and **one unit already
    // parted**: who=1's Frigate `1/29` (TypeIndex 343, sea, uid 57) stands
    // at (1896, 5352) on both sides, where theirs holds an `ATTACKORDER`
    // on the human's `0/2004` (uid 4, `in_range` 1) with `fought` 1,
    // `blown` 1 and `recharging` 39, and ours none (`orders.len` 0 against
    // 1, `idle` 1 against 0 on 3862). On the stride's block 3801 both
    // sides walk it on its `ATTACKTOORDER` to (1896, 5352); 3802..3860 is
    // undumped. The word 3866: 11 draws each, parting at index 0, ours
    // `Guy::set_anim+0x97a < Guy::inc_time+0x271`, theirs
    // `Ammo::init+0xcd9` — a round theirs fires (`docs/AI.md` §335.1).
    // Item 3642: 738 → 102. The Frigate attacks the Farm on both sides
    // from 3859 (run1303) and the first row past the standing families
    // is block 3877, the word 3876's: the Farm `0/2004`'s `damage` ours
    // 9 against 0, who=0's city `0/2000`'s raid flags and `raid_stamp`
    // 3876, and both leaders' `treaties`. Ours' round lands a frame
    // early: launched from the ship's centre (1896, 5352) for 11 frames,
    // where theirs' broadside leaves (1809, 5233) for 12 (`docs/AI.md`
    // §335.2; this walk compares no `AMMO` record).
    // Item 3643: 102 → 95. The Frigate's broadside leaves its piece 293's
    // bay (1809, 5233) for 12 frames with theirs, and the Farm is struck
    // on 3877 both sides; what is left past block 3861 is who=1's groups'
    // `held` and pools on 3882, 3966, 4018, 4028 and 4106 (§335.3).
    pin_eq!(w.firsts.len(), 95, "run1301b baseline");
}

/// run1304b: the word 4859's window, blocks 4853..5109 at run1287b's
/// detail — six blocks before the word's block 4860 and 250 after (item
/// 3643); the stride's blocks are 4801 and 4901.
pub(crate) const RUN1304B: &str =
    "gamelog-run1304b-eastindies-spanish-gunpowder-window-4853-5109.txt";

/// The window: block 4853 through 5109; the word 4859's own block is 4860.
pub(crate) const WIDENING_SPANISH_GUNPOWDER_FRAME_4859: (i64, i64) = (4853, 5109);

/// **The word 4859's window**: run1304b walked from run1286's start with
/// the recorder on (item 3643).
pub(crate) fn spanish_gunpowder_frame_4859_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_GUNPOWDER.start],
        true,
        SPANISH_GUNPOWDER.long,
        "run1304b",
        &[(RUN1304B, WIDENING_SPANISH_GUNPOWDER_FRAME_4859.0)],
        WIDENING_SPANISH_GUNPOWDER_FRAME_4859,
        1,
        &[4860],
        true,
    )
}

#[test]
fn spanish_gunpowder_pair_run1304b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_gunpowder_frame_4859_window() else {
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
    // Item 3643, the first walk: 857 keys. Block 4853 carries only the
    // standing families (`form`, who=1's groups' `held` and pools,
    // `SITE[i].reg`, who=0's `aggression[1]`). The first row past them is
    // the word's block 4860: who=1's Fire Ship `1/86` (TypeIndex 336, uid
    // 132, group 71, on an `ATTACKTOORDER`) at (31656, 30984) against
    // theirs' (31643, 31002); on 4861 the Frigate `1/58` (TypeIndex 343,
    // uid 93, group 71) holds 3 orders in theirs against ours' 1, the
    // current a `MOVEORDER` to (3240, 6168) over its `ATTACKTOORDER` to
    // (600, 7944) (`docs/AI.md` §335.3). The word 4859: ours 16 draws
    // against 18, parting at index 7, ours `Unit::fight+0x9b0`, theirs
    // `Unit::come_out+0x25ca`.
    pin_eq!(w.firsts.len(), 857, "run1304b baseline");
}
