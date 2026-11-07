//! **The British at a held Industrial age, Great Sahara, at difficulty 4**
//! (DECISIONS 90, 91, 93, 94, 97 §3, item 3668): Great Sahara, `MAP_STYLE
//! 7`, human Nubians (4) against the **British (11)** at **Hard (4)**, seed
//! 12345, `STARTING_RESOURCES 7`, the age held at Industrial:
//! `STARTING_TECHNOLOGY 5`, `ENDING_TECHNOLOGY 5`. Its triple, `(7, 11, 4)`,
//! is new: every other British lobby is at 0 or 5 but the Classical pair's
//! `(18, 11, 4)`, another map, and the battery fields no British. Captures:
//! run1335 (a `DUMP_ALL` start) and run1336 (the draw stream to the game's
//! end, `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §347.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const BRITISH_INDUSTRIAL: Pair = Pair {
    name: "GreatSaharaBritishIndustrial",
    map_style: 7,
    ai_tribe: 11,
    difficulty: 4,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1335-greatsahara-british-industrial-start.txt",
    long: (
        "gamelog-run1336-greatsahara-british-industrial-trace.txt",
        "rontrace-run1336.log",
    ),
    // The game ended itself on 4767 (closing block 4768).
    length: 4767,
    // Item 3668, first try: frame 2382, by the count and the sequence —
    // ours 71 draws against 15, parting at index 5, ours
    // `Leader::produce_building+0xc99`, theirs `Leader::make_stuff+0x63d`
    // (`docs/AI.md` §347).
    //
    // Item 3678: **2382 → 2939**, by both. The AA Gun takes the British
    // `BRITISH_AA_COST` (25 %) off its price (`get_cost:283`), so on 2184
    // who=1's `create_units` values it 1028320 as theirs does (941484
    // without it: food and metal 79 against 59, the income factor 206
    // against 225), and its offer overwrites the Flamethrower's 1014609
    // at the make list's head as theirs' does. On 2939 ours spends 22
    // draws against 21, parting at index 4: ours `Unit::fight+0x9b0`,
    // theirs `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
    // Unit::move_step+0x3b6`.
    //
    // Item 3683: **2939 → 2918 → 4767, the game's end**, by both. Piece
    // 190's turret and muzzle (`pivot::RELEASES`, the British Light Tank
    // `MARK4`) put who=1's `1/28` rounds where theirs' leave — its 2868
    // round strikes the Citizen `0/8` on 2871 as theirs' does — and the
    // word fell to 2918 on the Black Watch's rounds, thrown from the
    // figures' squares. Piece 65's three `launch::BAYS` rows close it:
    // `1/15`'s round of 2897 misses `0/8` as theirs' does, and the pair is
    // whole (`docs/AI.md` §347).
    count: 4767,
    sequence: 4767,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const BRITISH_INDUSTRIAL_PAIR_WORD: i64 =
    if BRITISH_INDUSTRIAL.count < BRITISH_INDUSTRIAL.sequence {
        BRITISH_INDUSTRIAL.count
    } else {
        BRITISH_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn british_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &BRITISH_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "BRITISH_INDUSTRIAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 5, and the long is scoreable.
#[test]
fn british_industrial_pair_captures_say_their_lobby() {
    let p = &BRITISH_INDUSTRIAL;
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
        "the British Industrial pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (minted by item 3668; the battery's identity
/// test refuses it), once, and none of the battery's: the battery fields no
/// British; and no other British lobby's: every other British triple is at
/// difficulty 0 or 5 but the Classical pair's `(18, 11, 4)`, another map
/// (item 3668).
#[test]
fn british_industrial_pair_is_no_other_lobby() {
    let p = &BRITISH_INDUSTRIAL;
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
        "the battery fields the British"
    );
    for l in battery::SCORED_LOBBIES.iter().filter(|l| l.1 == p.ai_tribe) {
        assert!(
            *l == triple || l.2 == 0 || l.2 == 5 || l.0 != p.map_style,
            "{l:?} is another British lobby on this map at difficulty {}",
            l.2
        );
    }
    for other in [
        &super::british_classical_pair::BRITISH_CLASSICAL,
        &super::british_enlightenment_pair::BRITISH_ENLIGHTENMENT,
    ] {
        assert_eq!(other.ai_tribe, p.ai_tribe);
        assert_ne!(
            (other.map_style, other.ai_tribe, other.difficulty),
            triple,
            "{} is this lobby",
            other.name
        );
    }
}

/// **The long's stride, walked back from the word 2382** (item 3668):
/// only run1336's stride blocks are on disk before the word's window.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_STRIDE: (i64, i64) = (101, 2301);

pub(crate) fn british_industrial_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336 stride",
        &[(
            BRITISH_INDUSTRIAL.long.0,
            WIDENING_BRITISH_INDUSTRIAL_STRIDE.0,
        )],
        WIDENING_BRITISH_INDUSTRIAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_the_long_s_stride_walks_back_from_2382() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every stride block in the window");
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
    // Item 3668: 80 keys, all of them the standing families — 41
    // buildings' `build:extra` (which the stride does not print), `form`
    // on 27 units, and who=1's groups' `held` and pools 64..69 as each is
    // minted. **No unit row parts on any stride block to 2301**: the
    // parting is in 2302..2382, which is run1336b's.
    pin_eq!(w.firsts.len(), 80, "the stride's baseline");
}

/// run1336b: the word 2382's window, blocks 2295..2633 at the long's
/// detail with `LEADERS=9` and `AMMO=5` — six blocks before the stride's
/// last block 2301 and 250 after the word's block 2383 (item 3668).
pub(crate) const RUN1336B: &str =
    "gamelog-run1336b-greatsahara-british-industrial-window-2295-2633.txt";

/// The window: block 2295 through 2633; the word 2382's own block is 2383.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_FRAME_2382: (i64, i64) = (2295, 2633);

/// **The word 2382's window**: run1336b walked from run1335's start with
/// the recorder on and every round compared (item 3668).
pub(crate) fn british_industrial_frame_2382_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336b",
        &[(RUN1336B, WIDENING_BRITISH_INDUSTRIAL_FRAME_2382.0)],
        WIDENING_BRITISH_INDUSTRIAL_FRAME_2382,
        1,
        &[2383],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_frame_2382_window() else {
        return;
    };
    pin_eq!(w.blocks, 338, "every captured block");
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
    // Item 3668, the first walk: 1189 keys. **Block 2295 already carries
    // who=1's leader and two of its buildings' queues parted**, beside the
    // standing families: `1/2013` and `1/2024` queue the Flamethrower
    // (131) in ours where theirs queues the Machine Gun (123)
    // (`queue[*].type`, `num_queued[81]` 5 against `num_queued[73]` 5),
    // and the `MAKE` list, three buckets and the human leader's
    // `SITE[*].reg` and `aggression[1]` stand parted too. On the word
    // 2382 who=1's `MAKE[3]` is ours `cat` 4 at city 2 against theirs'
    // 6 at city 0: ours places a building (`1/2034`, 15000
    // `constr_time`) theirs does not (`docs/AI.md` §347).
    //
    // Item 3678: **1189 → 53**, the standing families alone (`form`,
    // `build:extra`, who=1's groups' `held` and pools, the human leader's
    // `SITE[*].reg` and `aggression[1]`): the AA Gun's British discount
    // keeps the Flamethrower off who=1's make list on 2184, and the
    // queues, the list and the buckets agree to 2633. The value diff on
    // the old word's block 2383 is empty.
    pin_eq!(w.firsts.len(), 53, "run1336b baseline");
    // No round is in flight on either side over the window.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}

/// **A British AA Gun costs a quarter less** (item 3678): `get_cost:283`–
/// `289`, a British leader (`has_tribe_bonus(0xb)`) and a unit that
/// `is(0x119, 0)` take `BRITISH_AA_COST` (25) off. On run1335's start the
/// British leader's AA Gun costs 60 food and 60 metal, and the Nubian
/// human's 80 and 80. On 2183 the walk prices it 59 and 59 for who=1 —
/// the income factor 225 and the offer 1028320 theirs' make list holds
/// on 2184 — where without the arm it was 79 and 79, the factor 206 and
/// the offer 941484, which let the Flamethrower keep the head.
///
/// Made to fail first: with `BRITISH_AA_COST` at 0, who=1's price reads
/// the Nubian's 80 and 80.
#[test]
fn british_industrial_pair_a_british_aa_gun_costs_a_quarter_less() {
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let Some(path) = dump(BRITISH_INDUSTRIAL.start) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = replay_initial(&log).unwrap();
    let mut built = build_sim(&loaded, &init, sim::Tuning::RON);
    let sim = &mut built.sim;
    let aa = loaded
        .unit_named("Anti-Aircraft Gun")
        .expect("an AA Gun type");
    let tree = sim.unit_types[aa].tree.expect("in the tree");
    assert_eq!(
        loaded.type_index(tree),
        0x119,
        "ANTIAIRCRAFTGUN in the original"
    );
    assert!(sim.nation[1].british && !sim.nation[0].british);
    let price = sim.price_of(1, aa);
    assert_eq!((price[0], price[2]), (60, 60), "the British price");
    let nubian = sim.price_of(0, aa);
    assert_eq!((nubian[0], nubian[2]), (80, 80), "the Nubian pays it whole");
    sim.tuning.british_aa_cost = 0;
    let whole = sim.price_of(1, aa);
    assert_eq!((whole[0], whole[2]), (80, 80), "without BRITISH_AA_COST");
}

/// run1352: the word 2939's window, blocks 2933..3190 at run1336b's
/// detail — six blocks before the word's block 2940 and 250 after it
/// (item 3678).
pub(crate) const RUN1352: &str =
    "gamelog-run1352-greatsahara-british-industrial-window-2933-3190.txt";

/// The window: block 2933 through 3190; the word 2939's own block is 2940.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_FRAME_2939: (i64, i64) = (2933, 3190);

/// **The word 2939's window**: run1352 walked from run1335's start with
/// the recorder on and every round compared (item 3678).
pub(crate) fn british_industrial_frame_2939_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1352",
        &[(RUN1352, WIDENING_BRITISH_INDUSTRIAL_FRAME_2939.0)],
        WIDENING_BRITISH_INDUSTRIAL_FRAME_2939,
        1,
        &[2940],
        true,
    )
}

#[test]
fn british_industrial_pair_run1352_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_frame_2939_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
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
    // Item 3678: 1533 keys. Block 2933 carries the standing families
    // alone (`form`, who=1's groups' `held` and pools 64..69, the human
    // leader's `SITE[*].reg` and `aggression[1]`). **The first record
    // to part is a round, on 2934**: who=1's Light Tank `1/28` (TY 240,
    // `MARK4`, attacking the human's Citizen `0/3`) launches `ammo[1]`
    // from (9432, 31128) at `sz` 619 in ours — the figure's square — and
    // from (9218, 31078) at 757 in theirs, `total_time` 6 against 5. So
    // on 2937 theirs' round lands on `0/3` (32 damage, `hits_left` 33)
    // and ours' is still flying on 2938; on the word 2939 `0/3` walks a
    // different path and ours' `Unit::fight` draws where theirs does not
    // (`docs/AI.md` §347). Item 3683: **53**, the standing families
    // alone; `1/28`'s round leaves the `MARK4`'s muzzle (piece 190) at
    // (9218, 31078) and 757 as theirs' does, and the rounds part (306, 278)
    // → (306, 306), 110 → 6, all dead targets' `whom`/`ox`.
    // Item 3708: 53 → 52, a fort placed but not built no longer
    // counts in the site sweep (`fort_at` asks `active`, `docs/AI.md` §358.3).
    pin_eq!(w.firsts.len(), 52, "run1352 baseline");
    // Rounds are in flight on both sides over the window.
    pin_eq!(w.rounds_read, (306, 306), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// run1355b: the word 2918's window, blocks 2903..3168 at run1352's
/// detail — the Armored Car `1/30`'s rounds of 2911 and the human's
/// Citizen `0/8`'s death on 2912 in it, and 250 blocks past the word's
/// block 2919 (item 3683).
pub(crate) const RUN1355B: &str =
    "gamelog-run1355b-greatsahara-british-industrial-window-2903-3168.txt";

/// The window: block 2903 through 3168; the word 2918's own block is 2919.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_FRAME_2918: (i64, i64) = (2903, 3168);

/// **The word 2918's window**: run1355b walked from run1335's start with
/// the recorder on and every round compared (item 3683).
pub(crate) fn british_industrial_frame_2918_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1355b",
        &[(RUN1355B, WIDENING_BRITISH_INDUSTRIAL_FRAME_2918.0)],
        WIDENING_BRITISH_INDUSTRIAL_FRAME_2918,
        1,
        &[2919],
        true,
    )
}

#[test]
fn british_industrial_pair_run1355b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_frame_2918_window() else {
        return;
    };
    pin_eq!(w.blocks, 265, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
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
    // Item 3683: **52**, the standing families alone. Before piece 65's
    // bays: 1788, the first non-standing row on 2903 the Citizen `0/8`'s
    // `hits:damage` (54 against 41), `damage_o` (15, the Black Watch
    // `1/15`, against 28, the `MARK4` `1/28`) and `damage_frame` (2901
    // against 2871).
    pin_eq!(w.firsts.len(), 52, "run1355b baseline");
    pin_eq!(w.rounds_read, (363, 363), "the rounds read");
    // 363 a side, 10 parting: dead targets' `whom`/`ox` alone. Before
    // piece 65's bays: 363 against 402, 122 parting, the Black Watch's
    // throws from the figures' squares (`1/16`'s of 2911 from (8568,
    // 31080, 641) against (8435, 31093, 763)).
    pin_eq!(w.rounds.len(), 10, "the rounds that part");
}

/// run1355c: the closing window, blocks 4512..4768 at run1352's detail,
/// `cover=0` (item 3683) — 257 blocks through the closing block, so the
/// word 4767, the game's last frame, sits inside a window.
pub(crate) const RUN1355C: &str =
    "gamelog-run1355c-greatsahara-british-industrial-window-4512-4768.txt";

/// The window: block 4512 through the closing block 4768.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_FRAME_4767: (i64, i64) = (4512, 4768);

/// **The closing window**: run1355c walked from run1335's start with the
/// recorder on, the rounds compared (item 3683).
pub(crate) fn british_industrial_frame_4767_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1355c",
        &[(RUN1355C, WIDENING_BRITISH_INDUSTRIAL_FRAME_4767.0)],
        WIDENING_BRITISH_INDUSTRIAL_FRAME_4767,
        1,
        &[4768],
        true,
    )
}

#[test]
fn british_industrial_pair_run1355c_s_closing_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_frame_4767_window() else {
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
    // Item 3683: **58** — the standing families (`form`, who=1's groups'
    // `held` and pools, the human leader's `SITE[*].reg` and
    // `aggression[1]`) and two idle Black Watch's `dest_angle` (`1/83` on
    // 4624, `1/61` on 4626; parked 3683c).
    pin_eq!(w.firsts.len(), 58, "run1355c baseline");
    pin_eq!(w.rounds_read, (295, 295), "the rounds read");
    // 295 a side, 37 parting: the Artillery's (`1/68`, `1/94`, `1/106`,
    // TY 269) eight launches, 136 below theirs and some 40 behind (parked
    // 3683b), and dead targets' `whom`/`ox`.
    pin_eq!(w.rounds.len(), 37, "the rounds that part");
}

/// **The closing state** (item 3683): run1336's closing block 4768, the
/// whole map's units, walked from run1335's start.
#[test]
fn british_industrial_pair_run1336_s_closing_state() {
    let _pins = Pins::hold();
    let p = &BRITISH_INDUSTRIAL;
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
        "British Industrial pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    for &(who, o) in &e.extra {
        if let Some(u) = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
        {
            let name = u.ty.and_then(|ty| loaded.unit_names.get(ty).cloned());
            eprintln!("    extra {who}/{o} {name:?} at {:?}", u.pos);
        }
    }
    for b in &fin.builds {
        if (0..2).contains(&b.who) && harness::link_building(&built.sim, b.who, b.o).is_none() {
            eprintln!(
                "    unlinked building {}/{} at {:?} type {:?}",
                b.who, b.o, b.pos, b.orig_type
            );
            for x in built
                .sim
                .buildings
                .iter()
                .filter(|x| i64::from(x.pos.x) == b.pos.x && i64::from(x.pos.y) == b.pos.y)
            {
                eprintln!(
                    "      ours {}/{} alive {} hits {} health {} ty {:?}",
                    x.owner, x.index, x.alive, x.hits, x.health, x.ty
                );
            }
        }
    }
    pin_eq!(e.frame, 4768, "the closing block");
    // Item 3683, the pair whole: 179 units, none off, torn, unlinked or
    // extra, `Closing fields: none` — the game ends itself on 4767.
    pin_eq!(e.compared, 179, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}

/// run1336c: the long's stride to 2301 again, with `BUILDS=7`, `CITIES=5`
/// and `LEADERS=9` on every stride block — run1336 prints neither the
/// leaders nor the buildings' queues, which run1336b's first block 2295
/// carries parted (item 3668).
pub(crate) const RUN1336C: &str =
    "gamelog-run1336c-greatsahara-british-industrial-leaders-stride.txt";

/// The leaders' stride: block 101 through 2301.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_LEADERS: (i64, i64) = (101, 2301);

pub(crate) fn british_industrial_leaders_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336c",
        &[(RUN1336C, WIDENING_BRITISH_INDUSTRIAL_LEADERS.0)],
        WIDENING_BRITISH_INDUSTRIAL_LEADERS,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336c_s_leaders_stride_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_leaders_window() else {
        return;
    };
    pin_eq!(w.blocks, 23, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
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
    // Item 3668: 97 keys. From 101 the standing families — `form` on 27
    // units, who=1's groups and pools 64..69, and both leaders'
    // `SITE[*].reg` and `aggression` (§2.7's seam) — and nothing else to
    // 2101. **On 2201 who=1's make list and two buildings' queues part**,
    // 38 keys: `1/2013`'s `queue[3]` and `1/2024`'s `queue[0..3]` hold the
    // Flamethrower (131) in ours and the Machine Gun (123) in theirs,
    // `num_queued[81]` 5 against `num_queued[73]` 5, the `MAKE[0..3]`
    // rows and the food, wealth and metal buckets. Both sides agree on
    // 2101; run1336d holds the gap.
    //
    // Item 3678: **97 → 59**, the standing families alone to 2301: the
    // make list, the queues and the buckets agree on 2201 with the
    // British AA Gun's discount (`docs/AI.md` §347).
    pin_eq!(w.firsts.len(), 59, "run1336c baseline");
}

/// run1336d: the gap 2095..2201 between run1336c's agreeing 2101 and its
/// parted 2201, at the window's detail (item 3668).
pub(crate) const RUN1336D: &str =
    "gamelog-run1336d-greatsahara-british-industrial-gap-2095-2201.txt";

/// The gap: block 2095 through 2201.
pub(crate) const WIDENING_BRITISH_INDUSTRIAL_GAP: (i64, i64) = (2095, 2201);

pub(crate) fn british_industrial_gap_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings_with_rounds(
        &[BRITISH_INDUSTRIAL.start],
        true,
        BRITISH_INDUSTRIAL.long,
        "run1336d",
        &[(RUN1336D, WIDENING_BRITISH_INDUSTRIAL_GAP.0)],
        WIDENING_BRITISH_INDUSTRIAL_GAP,
        1,
        &[],
        true,
    )
}

#[test]
fn british_industrial_pair_run1336d_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = british_industrial_gap_window() else {
        return;
    };
    pin_eq!(w.blocks, 106, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
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
    // Item 3668: 93 keys. Block 2095 carries the standing families
    // alone; **the first record to part is who=1's make list on 2184**,
    // rebuilt that frame: theirs heads it with the Anti-Aircraft Gun (281,
    // `val` 1028320) and holds the Machine Gun (123, `val` 78152, `num` 2)
    // at `MAKE[3]`; ours heads it with the Flamethrower (131, `val`
    // 1014609, `num` 2), which theirs' list never holds, and the AA Gun
    // second at 941484. The queues take the pick on 2186 (`1/2013`'s
    // `queue[3]`, `1/2024`'s `queue[0..3]`: 131 against 123), and the
    // buckets follow (`docs/AI.md` §347).
    //
    // Item 3678: **93 → 54**, the standing families alone. The two
    // offers are the same on both sides: `create_units` offers the
    // Flamethrower first (1014609, the count cap's tail) and the AA Gun
    // after it, and `MakeList::make_me` overwrites a beaten head without
    // shifting it down — so the head is whichever is dearer. The AA Gun
    // was 941484 here without the British `BRITISH_AA_COST` (its price 79
    // food and 79 metal, the income factor 206) and is 1028320 with it
    // (59 and 59, 225), as theirs: the value diff on 2184 is empty.
    pin_eq!(w.firsts.len(), 54, "run1336d baseline");
    // No round is in flight on either side over the gap.
    pin_eq!(w.rounds_read, (0, 0), "the rounds read");
    pin_eq!(w.rounds.len(), 0, "the rounds that part");
}
