//! **The Koreans at a held Classical age, Great Sahara** (DECISIONS 90,
//! 91, 93, 94, item 3601): Great Sahara, `MAP_STYLE 7`, human Nubians (4)
//! against the **Koreans (16)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Classical: `STARTING_TECHNOLOGY
//! 1`, `ENDING_TECHNOLOGY 1` — the Korean Classical pair's lobby with the
//! nation moved. The Koreans' other lobbies are `korean_word`'s
//! `GreatSaharaKoreanTier3`, `(7, 16, 3)`, and `korean_ancient_pair`'s
//! `EastIndiesKoreanAncient`, `(18, 16, 5)` — another difficulty, or
//! another map. Captures: run1249 (a `DUMP_ALL` start) and run1250 (the
//! draw stream to the game's end, `cover=0`, `end:UNITS=3,GUYS=2` on a
//! stride of 100, walked from the start's initial state). The first try is
//! in `docs/AI.md` §322.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const KOREAN_CLASSICAL: Pair = Pair {
    name: "GreatSaharaKoreanClassical",
    map_style: 7,
    ai_tribe: 16,
    difficulty: 5,
    starting_technology: 1,
    ending_technology: 1,
    starting_resources: 7,
    start: "gamelog-run1249-greatsahara-korean-classical-start.txt",
    long: (
        "gamelog-run1250-greatsahara-korean-classical-trace.txt",
        "rontrace-run1250.log",
    ),
    // The game ended itself on 8777 (closing block 8778).
    length: 8777,
    // Item 3601, first try: frame 8379 by the count and the sequence, ours
    // 29 draws against theirs' 27, parting at index 10 — ours
    // `Guy::set_anim+0x97a < Unit::move_step+0x823` (the Javelineers
    // `1/267`), theirs `Unit::fight+0x824` (`docs/AI.md` section 322). The
    // state parts first on block 8375: the human Citizen `0/4` takes an
    // Elite Hwarang's round on 8374 in theirs and on 8375 in ours.
    count: 8379,
    sequence: 8379,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const KOREAN_CLASSICAL_PAIR_WORD: i64 =
    if KOREAN_CLASSICAL.count < KOREAN_CLASSICAL.sequence {
        KOREAN_CLASSICAL.count
    } else {
        KOREAN_CLASSICAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn korean_classical_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &KOREAN_CLASSICAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "KOREAN_CLASSICAL {}: count {} sequence {} of {}; {}",
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
fn korean_classical_pair_captures_say_their_lobby() {
    let p = &KOREAN_CLASSICAL;
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
        "the Korean Classical pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the held-out `GreatSaharaKoreanTier3` is
/// `(7, 16, 3)` — this map at another difficulty — and the Ancient-start
/// pair `(18, 16, 5)` another map (item 3601).
#[test]
fn korean_classical_pair_is_no_other_lobby() {
    let p = &KOREAN_CLASSICAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for l in [
        (
            super::korean_word::KOREAN.map_style,
            super::korean_word::KOREAN.ai_tribe,
            super::korean_word::KOREAN.difficulty,
        ),
        (
            super::korean_ancient_pair::KOREAN_ANCIENT.map_style,
            super::korean_ancient_pair::KOREAN_ANCIENT.ai_tribe,
            super::korean_ancient_pair::KOREAN_ANCIENT.difficulty,
        ),
    ] {
        assert_ne!(l, triple);
    }
}

/// run1250b: the word 8379's window, blocks 8374..8630 at the long's
/// detail — five blocks before the word's block 8380 and 250 after (item
/// 3601); the start dumps blocks 0..2 alone and the long's stride prints
/// units every hundredth block and no `LEADERS` past the start.
pub(crate) const RUN1250B: &str =
    "gamelog-run1250b-greatsahara-korean-classical-window-8374-8630.txt";

/// The window: block 8374 through 8630; the word 8379's own block is 8380.
pub(crate) const WIDENING_KOREAN_CLASSICAL_FRAME_8379: (i64, i64) = (8374, 8630);

/// **The word 8379's window**: run1250b walked from run1249's start with
/// the recorder on (item 3601).
pub(crate) fn korean_classical_frame_8379_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[KOREAN_CLASSICAL.start],
        true,
        KOREAN_CLASSICAL.long,
        "run1250b",
        &[(RUN1250B, WIDENING_KOREAN_CLASSICAL_FRAME_8379.0)],
        WIDENING_KOREAN_CLASSICAL_FRAME_8379,
        1,
        &[8380],
        true,
    )
}

#[test]
fn korean_classical_pair_run1250b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_classical_frame_8379_window() else {
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
    // Item 3601, the first walk: 9513 keys. Block 8374, the window's
    // first, carries the standing families (every unit's `form`, who=0's
    // `SITE[i].reg`, both leaders' `aggression`, who=1's groups' `held`
    // and pools) and five garrisoned units' `g.angle[0]` (ours 1431655765
    // against theirs 0, 3586's shape). The state parts on block 8375: the
    // human Citizen `0/4` (TypeIndex 50, land) holds `hits_left` 48
    // against ours 50, `damage` 2 and `damage_frac` 5 against 0,
    // `damage_frame` 8374 against 0 — theirs' only extra draw on 8374 is
    // the sound generator's `SoundGlobal::play < Object::do_damage <
    // Ammo::do_damage`, a round's landing — with both leaders' `treaties`
    // 3 against 1 and who=0's city `raid_stamp` 8374. The round is the
    // Elite Hwarang `1/135`'s (TypeIndex 181), launched on 8361 on both
    // sides (the same `Ammo::init` draws): ours lands it on 8375
    // (`damage_frame` 8375 standing on 8378, `hits` agreeing). `0/4`
    // takes its flee order on 8376 in theirs, a frame ahead, and the draws
    // part on 8379 (block 8380).
    pin_eq!(w.firsts.len(), 9513, "run1250b baseline");
}
