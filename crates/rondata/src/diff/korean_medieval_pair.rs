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
use crate::testenv::{dump, install};

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
    // Item 3679: **4559, the game's end**, by both. run1353 widened 4379
    // and walked back to run1353b's 4308: who=1's decoy Royal Hwarang
    // `1/139` closed on 4267 with a spent arrow in the pool, which holds
    // its number in theirs until the arrow goes on 4338; ours had no spent
    // arrows, freed 139 by 4297 and numbered the 4308 squad's third Royal
    // Hwarang 139 where theirs takes 183 (`docs/COMBAT.md` §46.7, `docs/AI.md`
    // §345).
    count: 4559,
    sequence: 4559,
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
    // Item 3679: **(3443, 3443), 46 parting** — the spent arrows compared
    // too (`flags 1`, COMBAT §46.7), and every slot agrees. The 46 rows
    // are 23 live rounds' `whom`/`ox` from 3398: the round flies on at a
    // target that died in flight (the Citizen `0/13` on 3397, `1/52`'s
    // 3395 arrow), theirs keeps the pair to the landing and ours'
    // `Sim::forget` clears it at the death (parked 3679).
    pin_eq!(w.rounds_read, (3443, 3443), "the rounds read");
    pin_eq!(w.rounds.len(), 46, "the rounds that part");
}

/// run1353: the word 4379's window, blocks 4373..4559 at run1331b's detail
/// (`LEADERS=9`, `AMMO=5`) — six blocks before the word and on to the
/// game's own end, so the closing window since the word closed (item 3679).
pub(crate) const RUN1353: &str = "gamelog-run1353-greatlakes-korean-medieval-window-4373-4559.txt";

/// The window: block 4373 through the closing block 4560, so the word
/// 4559, the game's last frame, sits inside it; the old word 4379's own
/// block is 4380.
pub(crate) const WIDENING_KOREAN_MEDIEVAL_FRAME_4559: (i64, i64) = (4373, 4560);

/// **The closing window**: run1353 walked from run1330's start with the
/// recorder on and every round compared (item 3679).
pub(crate) fn korean_medieval_frame_4559_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[KOREAN_MEDIEVAL.start],
        true,
        KOREAN_MEDIEVAL.long,
        "run1353",
        &[(RUN1353, WIDENING_KOREAN_MEDIEVAL_FRAME_4559.0)],
        WIDENING_KOREAN_MEDIEVAL_FRAME_4559,
        1,
        &[4380],
        true,
    )
}

#[test]
fn korean_medieval_pair_run1353_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_medieval_frame_4559_window() else {
        return;
    };
    pin_eq!(w.blocks, 186, "every captured block");
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
    // Item 3679, the first walk, before the spent arrow: 2007 keys, 223 of
    // them on the window's first block 4373 — who=1's Royal Hwarang squad
    // `1/181`..`1/185` a formation slot apart, `1/186` a Royal Hwarang
    // (uid 247, hits 97, group 74) in theirs and a Korean Citizen in ours,
    // and `1/139` held by ours alone: the numbering parted in the gap
    // (run1353b, 4308). With the spent arrow: **50**, the standing
    // families alone (`form`, both leaders, who=1's groups and pools,
    // who=0's `aggression[1]`).
    pin_eq!(w.firsts.len(), 50, "run1353 baseline");
    // Every round, live and spent, slot for slot: 3730 a side, none
    // parting.
    pin_eq!(w.rounds_read, (3730, 3730), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **The closing state** (item 3679): the long's own closing block,
/// walked to its close and compared unit for unit — a draw stream that
/// agrees to a game's end says nothing of the end itself (item 1099).
#[test]
fn korean_medieval_pair_run1331_s_closing_state() {
    let _pins = Pins::hold();
    let p = &KOREAN_MEDIEVAL;
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
        "Korean Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 4560, "the closing block");
    // Item 3679, the pair whole: 198 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 4559.
    pin_eq!(e.compared, 198, "units compared");
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

/// run1353b: the gap before the word's window, blocks 4195..4373 at the
/// same detail — six blocks before the long's stride block 4201, to the
/// word's window's first block (item 3679).
pub(crate) const RUN1353B: &str =
    "gamelog-run1353b-greatlakes-korean-medieval-window-4195-4373.txt";

/// The gap's window: block 4195 through 4373.
pub(crate) const WIDENING_KOREAN_MEDIEVAL_GAP_4195: (i64, i64) = (4195, 4373);

/// **The gap before the word 4379**: run1353b walked from run1330's start
/// with every round compared (item 3679).
pub(crate) fn korean_medieval_gap_4195_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[KOREAN_MEDIEVAL.start],
        true,
        KOREAN_MEDIEVAL.long,
        "run1353b",
        &[(RUN1353B, WIDENING_KOREAN_MEDIEVAL_GAP_4195.0)],
        WIDENING_KOREAN_MEDIEVAL_GAP_4195,
        1,
        &[],
        true,
    )
}

#[test]
fn korean_medieval_pair_run1353b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = korean_medieval_gap_4195_window() else {
        return;
    };
    pin_eq!(w.blocks, 178, "every captured block");
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
    // Item 3679, the first walk, before the spent arrow: 281 keys, the
    // records first parting on **4308** — theirs numbers a squad of three
    // Royal Hwarang `1/181`..`1/183` (uids 242..244, group 71) where ours
    // numbers the third `1/139`, the slot of who=1's decoy Royal Hwarang
    // that closed on 4267. Theirs' `1/139` arrow is spent (`flags 1`,
    // `cur_time` 57 on 4195, 199 on 4337) and holds its dead shooter's
    // number until it goes on 4338 (`Ammo::inc_time`'s bump). With the
    // spent arrow: **49**, the standing families alone.
    pin_eq!(w.firsts.len(), 49, "run1353b baseline");
    // Every round, live and spent, slot for slot (before: 2374 a side with
    // theirs' spent records set aside, 7 slot rows parting).
    pin_eq!(w.rounds_read, (3230, 3230), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}
