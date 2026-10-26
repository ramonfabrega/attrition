//! **The Lakota at a held Gunpowder age, Great Sahara** (DECISIONS 90, 91,
//! 93, 94, item 3628): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Lakota (19)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Gunpowder: `STARTING_TECHNOLOGY
//! 3`, `ENDING_TECHNOLOGY 3` — `greek_gunpowder_pair`'s lobby with the
//! nation moved. Its triple, `(7, 19, 5)`, is new: the battery's one Lakota
//! game is `GreatSaharaLakotaTier2`, `(7, 19, 2)`, another difficulty;
//! `lakota_word`'s `GreatLakesLakotaTier2` is `(14, 19, 2)`, and the Peace
//! and Medieval pairs are East Indies' `(18, 19, 3)` and `(18, 19, 5)`.
//! Captures: run1288 (a `DUMP_ALL` start) and run1289 (the draw stream to
//! the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100,
//! walked from the start's initial state). The first try is in
//! `docs/AI.md` §336.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const LAKOTA_GUNPOWDER: Pair = Pair {
    name: "GreatSaharaLakotaGunpowder",
    map_style: 7,
    ai_tribe: 19,
    difficulty: 5,
    starting_technology: 3,
    ending_technology: 3,
    starting_resources: 7,
    start: "gamelog-run1288-greatsahara-lakota-gunpowder-start.txt",
    long: (
        "gamelog-run1289-greatsahara-lakota-gunpowder-trace.txt",
        "rontrace-run1289.log",
    ),
    // The game ended itself on 4750 (closing block 4751).
    length: 4750,
    // Item 3628, first try: frame 4187, by the count and the sequence —
    // ours 14 draws against 12, parting at index 3 (`docs/AI.md` §336).
    count: 4187,
    sequence: 4187,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const LAKOTA_GUNPOWDER_PAIR_WORD: i64 =
    if LAKOTA_GUNPOWDER.count < LAKOTA_GUNPOWDER.sequence {
        LAKOTA_GUNPOWDER.count
    } else {
        LAKOTA_GUNPOWDER.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_gunpowder_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &LAKOTA_GUNPOWDER;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "LAKOTA_GUNPOWDER {}: count {} sequence {} of {}; {}",
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
fn lakota_gunpowder_pair_captures_say_their_lobby() {
    let p = &LAKOTA_GUNPOWDER;
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
        "the Lakota Gunpowder pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3628; the battery's identity
/// test refuses it) and none of the battery's: the battery's one Lakota
/// game is `GreatSaharaLakotaTier2`, `(7, 19, 2)`, this map at another
/// difficulty; and no other pinned Lakota lobby's: `lakota_word`'s
/// `(14, 19, 2)`, the Peace pair's `(18, 19, 3)` and the Medieval pair's
/// `(18, 19, 5)` (item 3628).
#[test]
fn lakota_gunpowder_pair_is_no_other_lobby() {
    let p = &LAKOTA_GUNPOWDER;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let lakota: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        lakota,
        [("GreatSaharaLakotaTier2", 7, 2)],
        "the battery's Lakota games"
    );
    let word = &super::lakota_word::LAKOTA;
    let peace = &super::lakota_pair::LAKOTA;
    let medieval = &super::lakota_medieval_pair::LAKOTA_MEDIEVAL;
    for (name, other) in [
        (word.name, (word.map_style, word.ai_tribe, word.difficulty)),
        (
            peace.name,
            (peace.map_style, peace.ai_tribe, peace.difficulty),
        ),
        (
            medieval.name,
            (medieval.map_style, medieval.ai_tribe, medieval.difficulty),
        ),
    ] {
        assert_eq!(other.1, p.ai_tribe, "{name} fields the Lakota");
        assert_ne!(other, triple, "{name} is this lobby");
    }
}

/// **The long's stride, walked back from the word 4187** (item 3628):
/// only run1289's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_LAKOTA_GUNPOWDER_STRIDE: (i64, i64) = (101, 4101);

pub(crate) fn lakota_gunpowder_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_GUNPOWDER.start],
        true,
        LAKOTA_GUNPOWDER.long,
        "run1289 stride",
        &[(LAKOTA_GUNPOWDER.long.0, WIDENING_LAKOTA_GUNPOWDER_STRIDE.0)],
        WIDENING_LAKOTA_GUNPOWDER_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn lakota_gunpowder_pair_the_long_s_stride_walks_back_from_4187() {
    let _pins = Pins::hold();
    let Some(w) = lakota_gunpowder_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 41, "every stride block in the window");
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
    // Item 3628: 83 keys, every one standing: the start's buildings'
    // `build:extra` (which the stride does not print), who=1's groups'
    // `held` and pools as each group is minted (65 on 101 through 70 on
    // 4101), and `form` on 24 units. No unit's position, orders or hits
    // part through block 4101: the Musket Horse `1/54` (TypeIndex 207)
    // and the Citizen `0/5` (TypeIndex 50) agree there, `1/54` on an
    // `ATTACKTOORDER` to (10440, 28248).
    pin_eq!(w.firsts.len(), 83, "the stride's baseline");
}

/// run1289b: the word 4187's window, blocks 4182..4438 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the word's
/// block 4188 and 250 after (item 3628); the stride's blocks are 4101 and
/// 4201.
pub(crate) const RUN1289B: &str =
    "gamelog-run1289b-greatsahara-lakota-gunpowder-window-4182-4438.txt";

/// The window: block 4182 through 4438; the word 4187's own block is 4188.
pub(crate) const WIDENING_LAKOTA_GUNPOWDER_FRAME_4187: (i64, i64) = (4182, 4438);

/// **The word 4187's window**: run1289b walked from run1288's start with
/// the recorder on (item 3628).
pub(crate) fn lakota_gunpowder_frame_4187_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA_GUNPOWDER.start],
        true,
        LAKOTA_GUNPOWDER.long,
        "run1289b",
        &[(RUN1289B, WIDENING_LAKOTA_GUNPOWDER_FRAME_4187.0)],
        WIDENING_LAKOTA_GUNPOWDER_FRAME_4187,
        1,
        &[4188],
        true,
    )
}

#[test]
fn lakota_gunpowder_pair_run1289b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_gunpowder_frame_4187_window() else {
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
    // Item 3628, the first walk: 2311 keys. Block 4182 carries the standing
    // families (`form`, `SITE[i].reg`, who=0's `aggression[1]`, who=1's
    // groups 64, 65, 66, 68 and 70's `held` and pools). **The state first
    // parts on block 4184, frame 4183, and theirs acts**: theirs' Musket
    // Horse `1/54` (TypeIndex 207, land, uid 80), a guard of group 64 with
    // `near` 0/5, has a round in flight on the window's first block (cur_time
    // 1 of 3, fired in the undumped gap 4102..4181) at the human's Citizen
    // `0/5` (TypeIndex 50, uid 12); it lands on 4183 — `0/5`'s `hits_left`
    // ours 50 theirs 24, `damage_frame` ours 0 theirs 4183, the city
    // `0/2000`'s `raid_stamp` 0 against 4183 — and the hit ors treaty bit 1
    // into both leaders (`treaties[1]` on who=0 and `treaties[0]` on who=1,
    // ours 1 theirs 3). Ours' `1/54` stands where theirs' does and fires
    // nothing. On 4185 theirs' `0/5` flees (`FLEEORDER`) where ours' gathers.
    // **The word 4187**: ours 14 draws against 12, index 3 ours
    // `Guy::set_anim < Unit::move_step`, theirs `< Unit::do_idle`; on 4188
    // ours' Elite Pikeman `1/75` (TypeIndex 135) collides with `1/74`
    // (`collide_o` 74, `retry` 6) where theirs' steps clear, and `1/74`'s
    // order is ours 10 against theirs 12 (`docs/AI.md` §336).
    pin_eq!(w.firsts.len(), 2311, "run1289b baseline");
}
