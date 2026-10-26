//! **The Koreans at a held Medieval age, Great Lakes** (DECISIONS 90, 91,
//! 93, 94, item 3665): Great Lakes, `MAP_STYLE 14`, human Nubians (4)
//! against the **Koreans (16)** at **Toughest (5)**, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval:
//! `STARTING_TECHNOLOGY 2`, `ENDING_TECHNOLOGY 2`. Its triple,
//! `(14, 16, 5)`, is new: the Korean lobbies are `korean_ancient_pair`'s
//! `(18, 16, 5)`, `korean_classical_pair`'s `(7, 16, 5)` and the held-out
//! `GreatSaharaKoreanTier3`'s `(7, 16, 3)` (`korean_word`), and the battery
//! fields no Koreans. Captures: run1330 (a `DUMP_ALL` start) and run1331
//! (the draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try is
//! in `docs/AI.md` §345.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const KOREAN_MEDIEVAL: Pair = Pair {
    name: "GreatLakesKoreanMedieval",
    map_style: 14,
    ai_tribe: 16,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1330-greatlakes-korean-medieval-start.txt",
    long: (
        "gamelog-run1331-greatlakes-korean-medieval-trace.txt",
        "rontrace-run1331.log",
    ),
    // The game ended itself on 4559 (closing block 4560).
    length: 4559,
    // Item 3665, first try: frame 3401, by the count and the sequence —
    // ours 18 draws against 16, parting at index 0, ours `Guy::set_anim+0x97a
    // < Guy::do_turn+0x4a < Unit::move_step+0x3b6`, theirs
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` (`docs/AI.md` §345).
    // Item 3666: **4379**, by both. Piece 1090's bays took it to 3627 (ours
    // 24 draws against 25, index 8: theirs' blocked stand of `1/126`, a
    // Heavy Flaming Arrow, rolls a crew figure's idle), and `Guy::do_turn`'s
    // own crew loop to 4379 — ours 25 draws against 28, parting at index 3,
    // ours `Guy::set_anim+0x97a < Unit::move_step+0x823`, theirs
    // `Unit::resolve_unit_collision+0xb52`. Past run1331b's last block, so
    // its widening is owed (parked 3666).
    count: 4379,
    sequence: 4379,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const KOREAN_MEDIEVAL_PAIR_WORD: i64 =
    if KOREAN_MEDIEVAL.count < KOREAN_MEDIEVAL.sequence {
        KOREAN_MEDIEVAL.count
    } else {
        KOREAN_MEDIEVAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn korean_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &KOREAN_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "KOREAN_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2, and the long is scoreable.
#[test]
fn korean_medieval_pair_captures_say_their_lobby() {
    let p = &KOREAN_MEDIEVAL;
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
        "the Korean Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3665; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// Koreans; and no other Korean lobby's: the Ancient-start pair's
/// `(18, 16, 5)`, the Classical pair's `(7, 16, 5)` and the held-out
/// `GreatSaharaKoreanTier3`'s `(7, 16, 3)` are another map (item 3665).
#[test]
fn korean_medieval_pair_is_no_other_lobby() {
    let p = &KOREAN_MEDIEVAL;
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
        "the battery fields the Koreans"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.0 != p.map_style || l.2 != p.difficulty,
            "{l:?} is another Korean lobby on this map at this difficulty"
        );
    }
    for other in [
        &super::korean_ancient_pair::KOREAN_ANCIENT,
        &super::korean_classical_pair::KOREAN_CLASSICAL,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
    let k = &super::korean_word::KOREAN;
    assert_eq!(k.ai_tribe, p.ai_tribe);
    assert_ne!((k.map_style, k.ai_tribe, k.difficulty), triple);
}

/// **The long's stride, walked back from the word 3401** (item 3665):
/// only run1331's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_KOREAN_MEDIEVAL_STRIDE: (i64, i64) = (101, 3301);

pub(crate) fn korean_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[KOREAN_MEDIEVAL.start],
        true,
        KOREAN_MEDIEVAL.long,
        "run1331 stride",
        &[(KOREAN_MEDIEVAL.long.0, WIDENING_KOREAN_MEDIEVAL_STRIDE.0)],
        WIDENING_KOREAN_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn korean_medieval_pair_the_long_s_stride_walks_back_from_3401() {
    let _pins = Pins::hold();
    let Some(w) = korean_medieval_stride_window() else {
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
    // Item 3665: 108 keys, all of them the standing families — 56
    // buildings' `build:extra` (which the stride does not print), `form`
    // on 28 units, and who=1's groups' `held` and pools 64..75 as each is
    // minted. **No unit row parts on any stride block to 3301**: the
    // parting is in 3302..3401, which is run1331b's.
    pin_eq!(w.firsts.len(), 108, "the stride's baseline");
}

/// run1331b: the word 3401's window, blocks 3295..3652 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 3301 and 250 after the word's block 3402 (item 3665).
pub(crate) const RUN1331B: &str =
    "gamelog-run1331b-greatlakes-korean-medieval-window-3295-3652.txt";

/// The window: block 3295 through 3652; the word 3401's own block is 3402.
pub(crate) const WIDENING_KOREAN_MEDIEVAL_FRAME_3401: (i64, i64) = (3295, 3652);

/// **The word 3401's window**: run1331b walked from run1330's start with
/// the recorder on and every round compared (item 3665).
pub(crate) fn korean_medieval_frame_3401_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[KOREAN_MEDIEVAL.start],
        true,
        KOREAN_MEDIEVAL.long,
        "run1331b",
        &[(RUN1331B, WIDENING_KOREAN_MEDIEVAL_FRAME_3401.0)],
        WIDENING_KOREAN_MEDIEVAL_FRAME_3401,
        1,
        &[3402],
        true,
    )
}

#[test]
fn korean_medieval_pair_run1331b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_medieval_frame_3401_window() else {
        return;
    };
    pin_eq!(w.blocks, 357, "every captured block");
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
    // Item 3665, the first walk: 4369 keys. Block 3295 carries the
    // standing families (`form`, both leaders, who=1's groups and pools);
    // **the records first part on 3367**, after the round below: the
    // human's Citizen `0/13` (TY 50) is struck a frame earlier in theirs
    // (`hits:damage` ours 33 theirs 35, `hits_left` 17 against 15), and
    // `0/2000`'s `city:raid_stamp` 3365 against 3366.
    // Item 3666: **64**, the standing families alone (blocks 3295 and
    // 3297, and who=1's `SITE[·].reg` from 3376). Without piece 1090's
    // bays the rounds parted first; without `Guy::do_turn`'s crew loop
    // the records part first on 3630, `1/95`'s `move.pause` ours 7
    // theirs 5, then `1/130` on 3632 (facing −945422336 against
    // −707788800, `recharging` 0 against 30).
    pin_eq!(w.firsts.len(), 64, "run1331b baseline");
    // Every round, both directions: 2152 round-blocks theirs, 1975 ours,
    // 298 parting. The slot rows from 3351 are theirs' spent arrows
    // (`flags 1`, held in their slot, COMBAT §46.7: the Royal Hwarang
    // `1/52`'s 3314 arrow re-armed on 3333). **The first launch that
    // parts is on 3354, who=1's Elite Javelineers `1/47`** (TY 84,
    // `PELTASTS`) at `0/13`: theirs throws from (6934, 30317) at `sz` 436
    // in 14 frames, ours from the figure's own square (6984, 30312) at
    // 266 in 15 — 170 under theirs, the javelin bay's height. The
    // hypothesis: the Korean Elite Javelineers' piece has no
    // `launch::BAYS` row (`docs/AI.md` §345).
    // Item 3666: (2152, 2152), 244 parting by slot — and **none as a set**:
    // every block's launches, landings and flight times agree over the
    // whole window once theirs' spent `flags 1` records are set aside (0
    // of 288 blocks), so what parts is the slot a round sits in.
    pin_eq!(w.rounds_read, (2152, 2152), "the rounds read");
    pin_eq!(w.rounds.len(), 244, "the rounds that part");
}
