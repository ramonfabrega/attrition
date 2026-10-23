//! **The Japanese at a held Industrial age, East Indies** (DECISIONS 90,
//! item 3510): the coverage pair's lobby (run652's: East Indies,
//! `MAP_STYLE 18`, human Nubians (4), Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`) with the nation varied to the **Japanese (15)**
//! and the age held at Industrial: `STARTING_TECHNOLOGY 5`,
//! `ENDING_TECHNOLOGY 5`. Captures: run1152 (a `DUMP_ALL` start) and
//! run1153 (the draw stream to the game's end, `cover=0`,
//! `end:UNITS=3,GUYS=2` on a stride of 100, walked from the start's
//! initial state). The first try is in `docs/AI.md` §303.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const JAPANESE_INDUSTRIAL: Pair = Pair {
    name: "EastIndiesJapaneseIndustrial",
    map_style: 18,
    ai_tribe: 15,
    difficulty: 5,
    starting_technology: 5,
    ending_technology: 5,
    starting_resources: 7,
    start: "gamelog-run1152-eastindies-japanese-industrial-start.txt",
    long: (
        "gamelog-run1153-eastindies-japanese-industrial-trace.txt",
        "rontrace-run1153.log",
    ),
    // The game ended itself on 4547 (closing block 4548).
    length: 4547,
    // Item 3510, first try: frame 1351 by the count and the sequence, ours
    // 5 draws against 12, index 0 — ours `Guy::set_anim+0x104b`, theirs
    // `Guy::init_real+0x52`: theirs makes a birth's draws ours does not
    // (`docs/AI.md` section 303), three Riflemen out of Barracks `1/2013`.
    // Item 3510: 1351 -> 3674, the Japanese Barracks arms of `get_cost`
    // and `train_time` built (`JAPANESE_BARRACKS_COST` -7 and
    // `JAPANESE_BARRACKS_SPEED` -10 a tier): ours 73 draws against 94,
    // index 0 — ours `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `Unit::think_scout+0x941`.
    // Item 3521: 3674 -> 3769, `think_carry_civilian`'s shore eject
    // (`Group::action_eject_all` on a one-boat group) built: who=1's
    // Freighter `1/121` puts Citizen `1/10` ashore at (37416, 26280) on 3673
    // and dies, as theirs does. Ours 14 draws against 15, index 14 — theirs
    // `Object::take_damage+0xe1`, the human's Citizen `0/5` struck.
    // Item 3523: 3769 -> 3816, the Dreadnought's (piece 298) two turrets
    // built (`sim::pivot`): `1/74`'s round of 3758 leaves from (2885, 11948,
    // 121), flies 12 and strikes `0/5` on 3769, as theirs does. Ours 159
    // draws against 160, index 90 — ours `Guy::set_anim+0x97a <
    // Unit::do_idle+0x7d`, theirs `Unit::fight+0x9b0`.
    // Item 3525: 3816 -> 4343, the Destroyer's (piece 278) three turrets
    // (`sim::pivot`) and `do_move`'s captain retarget taking its own target
    // (no `c != t`): `1/66`'s stern round of 3800 flies 9 and strikes `0/5`
    // on 3808, and on 3815 `1/66` drops its chase at (2764, 8930) with
    // `near` `0/5` in plain reach and fires on 3816, as theirs does. Ours
    // 17 draws against 15, index 1 — ours `Unit::fight+0x9b0`, theirs
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`.
    count: 4343,
    sequence: 4343,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const JAPANESE_INDUSTRIAL_PAIR_WORD: i64 =
    if JAPANESE_INDUSTRIAL.count < JAPANESE_INDUSTRIAL.sequence {
        JAPANESE_INDUSTRIAL.count
    } else {
        JAPANESE_INDUSTRIAL.sequence
    };

/// **The pair's first parting**, walked from its own start.
#[test]
fn japanese_industrial_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &JAPANESE_INDUSTRIAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "JAPANESE_INDUSTRIAL {}: count {} sequence {} of {}; {}",
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
fn japanese_industrial_pair_captures_say_their_lobby() {
    let p = &JAPANESE_INDUSTRIAL;
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
        "the Japanese pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's.
#[test]
fn japanese_industrial_pair_is_no_other_lobby() {
    let p = &JAPANESE_INDUSTRIAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
}

/// **The Japanese pay less for a Barracks unit, by the tier** (item 3510,
/// `get_cost:236`–`253`): on run1152's lobby who=1's Rifleman is priced
/// `(100 − 7 × tier) × cost / 100` of the Nubian human's, the tier
/// `min(ages, epoch[0])`; and a Dock's unit 10% less.
#[test]
fn japanese_industrial_pair_a_japanese_barracks_unit_costs_less_by_the_tier() {
    let Some(inst) = install() else {
        return;
    };
    let Some(path) = dump(JAPANESE_INDUSTRIAL.start) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = replay_initial(&log).unwrap();
    let mut built = build_sim(&loaded, &init, sim::Tuning::RON);
    let sim = &mut built.sim;
    let rifle = loaded.unit_named("Riflemen").expect("a Riflemen type");
    // The start holds Military level 0: the tier is 0, and the arm pays
    // nothing until the first Military library.
    assert_eq!(sim.tech[1].epoch[0], 0);
    assert_eq!(sim.nation_unit_discount(1, rifle), 0);
    // Level 2, the tier block 1345's 68/60 implies: 14% off.
    sim.tech[1].epoch[0] = 2;
    assert!(sim.tech[1].ages >= 2);
    assert_eq!(sim.nation_unit_discount(1, rifle), 14);
    assert_eq!(sim.nation_unit_discount(0, rifle), 0, "the human is Nubian");
    let ship = loaded.unit_named("Frigate").expect("a Frigate type");
    assert_eq!(sim.nation_unit_discount(1, ship), 10);
    assert_eq!(sim.nation_unit_discount(0, ship), 0);
}

/// run1153b: the word 1351's window, blocks 1345..1601 at the long's
/// detail — six blocks before the word's block 1352 and 250 after (item
/// 3510).
pub(crate) const RUN1153B: &str =
    "gamelog-run1153b-eastindies-japanese-industrial-window-1345-1601.txt";

/// The window: block 1345 through 1601; the word 1351's own block is 1352.
pub(crate) const WIDENING_JAPANESE_INDUSTRIAL_FRAME_1351: (i64, i64) = (1345, 1601);

/// **The word 1351's window**: run1153b walked from run1152's start with
/// the recorder on (item 3510).
pub(crate) fn japanese_industrial_frame_1351_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_INDUSTRIAL.start],
        true,
        JAPANESE_INDUSTRIAL.long,
        "run1153b",
        &[(RUN1153B, WIDENING_JAPANESE_INDUSTRIAL_FRAME_1351.0)],
        WIDENING_JAPANESE_INDUSTRIAL_FRAME_1351,
        1,
        &[1352],
        true,
    )
}

#[test]
fn japanese_industrial_pair_run1153b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_industrial_frame_1351_window() else {
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
    // Item 3510, the first walk: 708 keys. **The state parts before the
    // window**: on block 1345 who=1's two Barracks price their Riflemen
    // queues ours 80/70 against theirs 68/60 (`1/2013 queue[0].cost`), the
    // Japanese Barracks arm of `get_cost` at tier 2 (`(100 − 14) × 80 /
    // 100`), and the three buckets with them; on the word's block 1352
    // theirs' `1/19`..`1/21` are out of `1/2013` (`num_units[50]` ours 0,
    // theirs 1). After the arms: 61, every one a standing family the other
    // late-age pairs carry — `SITE[i].reg`, `aggression`, `form`, and the
    // groups' `held` and pool (`docs/AI.md` section 303).
    pin_eq!(w.firsts.len(), 61, "run1153b baseline");
}

/// run1153c: the word 3674's window, blocks 3668..3924 at the long's
/// detail — six blocks before the word's block 3675 and 250 after (item
/// 3510).
pub(crate) const RUN1153C: &str =
    "gamelog-run1153c-eastindies-japanese-industrial-window-3668-3924.txt";

/// The window: block 3668 through 3924; the word 3674's own block is 3675,
/// the word 3769's (item 3521) is 3770, and the word 3816's (item 3523)
/// is 3817.
pub(crate) const WIDENING_JAPANESE_INDUSTRIAL_FRAME_3674: (i64, i64) = (3668, 3924);

/// **The word 3674's window**: run1153c walked from run1152's start with
/// the recorder on (item 3510).
pub(crate) fn japanese_industrial_frame_3674_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[JAPANESE_INDUSTRIAL.start],
        true,
        JAPANESE_INDUSTRIAL.long,
        "run1153c",
        &[(RUN1153C, WIDENING_JAPANESE_INDUSTRIAL_FRAME_3674.0)],
        WIDENING_JAPANESE_INDUSTRIAL_FRAME_3674,
        1,
        &[3675],
        true,
    )
}

#[test]
fn japanese_industrial_pair_run1153c_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = japanese_industrial_frame_3674_window() else {
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
    // Item 3510, the first walk after the Barracks arms: 2956 keys. **The
    // state parts before the window**: on block 3668 who=1's income is
    // food ours 1440 against theirs 1520 and wealth 1752 against 1832 (80
    // each), with `rate`, `leftover`, `escrow[2]` and the make list's
    // values beside them. On the word's block 3675 ours holds `1/121` alone
    // and who=1's Citizen `1/10` is inside it (`inside` 121 against -1),
    // where theirs' stands out at (37416, 26280) and re-scouts
    // (`Unit::think_scout+0x941 < Unit::think_peasant`); `docs/AI.md`
    // section 303.
    //
    // Item 3521: 2946 with the Japanese boats' quarter on a fish
    // (`calc_rare`'s water arm): on 3668 who=1's food and wealth income
    // agree, 1520 and 1832 — two Fishermen on fish, 40 each; the state had
    // first parted on frame 2959, the first boat's first paid gather (a
    // probe; no leader record is dumped there). Then **955**
    // with the shore eject: on 3674 `1/10` stands at (37416, 26280) on both
    // sides and `1/121` is gone on both. The next word's block 3770 is in
    // the window: theirs' human Citizen `0/5` is struck for 8 on 3769
    // (`hits_left` ours 65, theirs 57; `damage_frame` 3769), Barracks
    // `0/2004` for 11, city `0/2000`'s `raid_stamp` is 3769, and both
    // leaders' `treaties` read ours 1 against theirs 3. **950** on the tree
    // merged with item 3520's landing (`attack_pos.rs`), which closes five.
    //
    // Item 3523: **692** with the Dreadnought's turrets. Block 3770 agrees
    // whole: `0/5`'s `hits_left` 57 and `damage_frame` 3769 on both sides,
    // `0/2004` struck for 11 on both, and the leaders' `treaties` 3 on both.
    // The state next parts on 3808: `0/5` is struck again, theirs on 3804
    // (`raid_stamp` 3804, `hits_left` 35) and ours on 3807 (3807, 32).
    //
    // Item 3525: 692 with the Destroyer's turrets — block 3808 agrees whole
    // (`1/66`'s stern round of 3800 flies 9, not 8, and lands on 3808 as
    // theirs does; the next blow parts on 3837) — and **151** with the
    // captain retarget on its own target: on the word's block 3816 `1/66`
    // stands at (2764, 8930) on its `ATTACKORDER` (`in_range` 1) on both
    // sides, where ours had sailed on to (2832, 8916) under its
    // `MOVEORDER`. What is left from 3790 is the standing `form` family and
    // `1/82`'s path on 3813 (parked 3525a).
    pin_eq!(w.firsts.len(), 151, "run1153c baseline");
}
