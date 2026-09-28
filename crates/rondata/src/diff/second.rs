//! **The second pair** (DECISIONS 53 §2, item 971): run54's and run53's games
//! with the lobby's difficulty at its top setting, Toughest, and nothing else
//! moved — run346 on East Indies and run347 on Great Lakes. The words are
//! [`SECOND_WORD_EAST_INDIES`] and [`SECOND_WORD_GREAT_LAKES`]; the first
//! pair's `LONG_WORD_*` stay as its closed floor. `docs/AI.md` §80 has the
//! lever, the arms as hypotheses, and what each word turned out to be.

use super::*;

use crate::diff::harness::{attributed_sites, debug_leader, site_window};
use crate::diff::testkit::*;
use crate::testenv::{dump, install};

/// A long capture of the second pair walked from its own start dump: the
/// frame the draw count first parts, the frame the label sequence first
/// parts, and the trace's last frame. `None` when the capture is not on
/// this machine.
pub(crate) struct Walk {
    pub count: i64,
    pub sequence: i64,
    pub last: i64,
    pub difficulty: i32,
}

/// The two maps' setup differs in one borrow each, as run54's and run53's
/// own tests do: East Indies takes run38's start dump and the pasture from
/// its own trace, Great Lakes the sibling dumps.
pub(crate) fn walk_second(gamelog: &str, tracelog: &str, east_indies: bool) -> Option<Walk> {
    let inst = install()?;
    let (Some(path), Some(trace)) = (dump(gamelog), trace(tracelog)) else {
        eprintln!("skipping: no {gamelog} (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().unwrap();
    let sib_texts: Vec<String> = if east_indies {
        dump("gamelog-run38-islands-start.txt")
            .map(crate::capture::read)
            .into_iter()
            .collect()
    } else {
        sibling_texts()
    };
    let sib_logs: Vec<Log> = sib_texts.iter().map(|t| Log::parse(t)).collect();
    let sib_inits: Vec<Initial> = sib_logs.iter().filter_map(|l| l.initial()).collect();
    let refs: Vec<&Initial> = sib_inits.iter().collect();
    borrow_from_siblings(&mut init, &refs);
    if east_indies {
        borrow_pasture(&mut init, &trace);
    }
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    let difficulty = built.sim.lobby.difficulty;
    let last = trace.frames.last().map_or(0, |(n, _)| *n);
    let window = site_window();
    for f in 0..last {
        built.tick();
        debug_leader(&built, f);
        if window.is_some_and(|(lo, hi)| (lo..=hi).contains(&f)) {
            for (label, who) in attributed_sites(&built) {
                eprintln!("  f{f} {who}: {label}");
            }
            let theirs = trace.labels(f);
            if theirs != built.frame_sites.last().map_or(&[][..], |(_, v)| v) {
                eprintln!("  f{f} PARTS: theirs {theirs:?}");
            }
        }
    }
    let sequence = built
        .frame_sites
        .iter()
        .find(|(f, ours)| *ours != trace.labels(*f))
        .map_or(last, |(f, _)| *f);
    let count = built
        .frame_sites
        .iter()
        .find(|(f, ours)| ours.len() != trace.labels(*f).len())
        .map_or(last, |(f, _)| *f);
    eprintln!("{gamelog}: word parts at {count}, sequence at {sequence} of {last}");
    for (f, ours) in built.frame_sites.iter() {
        if *f != count && *f != sequence {
            continue;
        }
        let theirs = trace.labels(*f);
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i) != theirs.get(i))
            .unwrap_or(0);
        eprintln!(
            "  frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
            ours.len(),
            theirs.len(),
            ours.get(at),
            theirs.get(at)
        );
        if std::env::var("RON_DEBUG_SITES").is_ok() {
            for (i, l) in ours.iter().enumerate() {
                eprintln!("    ours  {i}: {l}");
            }
            for (i, l) in theirs.iter().enumerate() {
                eprintln!("    thrs  {i}: {l}");
            }
        }
    }
    Some(Walk {
        count,
        sequence,
        last,
        difficulty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::harness::tests::widen_east_indies_on;

    /// **The second pair's East Indies word, widened whole** (item 971):
    /// run349 is run346's game at run299's per-frame detail over blocks
    /// 1..250 (`rngcmp.py`: 263 frames, 0 differing), walked from run346's
    /// start by [`widen_east_indies_on`] — every record, every unit, both
    /// leaders, both directions — with the word's block (frame 0 writes
    /// block 1) and the two after it kept standing. It keeps the move's
    /// value diff; East Indies' next word, 10, is inside its window.
    #[test]
    fn run349_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run349",
            "gamelog-run349-islands-toughest-open.txt",
            WIDENING_SECOND_EAST_INDIES,
            &[2],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 250, "run349 whole: blocks 1..250");
        assert!(
            w.missing.is_empty(),
            "run349 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **item 971's value diff**, `docs/AI.md` §80.5: before
        // `think_spellcaster`'s coin the AI scout `1/0`'s walk target on
        // block 1 was 41976/36600 here against 35832/42744, and 28 of its
        // rows parted on blocks 1..3. **Item 979's** (§81): with run38's
        // frame words — an Easiest game's — no longer installed over this
        // one, the scout's walk parting (`orders_x` 38136 against 41976 on
        // block 97) closed too; its first row now is its sight count on
        // block 202.
        assert_eq!(row(1, 0, "orders_x"), None, "the scout's walk agrees");
        assert_eq!(
            row(1, 0, "mylos").as_deref(),
            Some("202: ours 6 theirs 4"),
            "the scout's first parting"
        );
        assert!(
            !w.firsts
                .iter()
                .any(|((who, o, _), (f, _))| (*who, *o) == (1, 0) && *f < 202),
            "none of the scout's rows parts before block 202"
        );
        // **And run38's clocks**: block 1's 46 gaia `cur_anim` rows and
        // `1/1`'s `g.end_time[0]` 232 against 33 were the sibling's
        // figures, installed at frame 0's end; both closed with the words.
        assert_eq!(row(1, 1, "g.end_time[0]"), None, "an AI citizen's clock");
        assert!(
            !w.firsts
                .iter()
                .any(|((who, _, what), _)| *who == 8 && what == "gaia:cur_anim"),
            "no gaia animal's clock parts"
        );
        // **What stands**: block 1's 67 keys (player 0's census,
        // `SITE.reg`, `form`, the city record — `docs/AI.md` §33.4's
        // families) and block 2's tech stamps, `tech_frame` and two
        // `tech_cat_frame`s, which the original's `Build::queue_up` writes
        // and this crate does not (§33.4's named unmodelled state).
        assert_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the original's tech stamp, standing"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 67), (2, 3), (184, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, widened whole** (item 971):
    /// run350 is run347's game at run349's detail over blocks 1..250, walked
    /// from run347's start by [`widen_great_lakes_on`]. The word is frame 1,
    /// which writes block 2.
    #[test]
    fn run350_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run350",
            &[("gamelog-run350-greatlakes-toughest-open.txt", 1)],
            WIDENING_SECOND_GREAT_LAKES,
            1,
            &[2],
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 250, "run350 whole: blocks 1..250");
        assert!(
            w.missing.is_empty(),
            "run350 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 1 stands** with frame 0's draws agreeing: 55 keys, the
        // first pair's families (player 0's census, `SITE.reg`, `form`, the
        // city record; `docs/AI.md` §33.4) and nothing of who=1's economy.
        // **Item 979's value diff** (§81): before the lobby gate, run12's
        // frame-0 word — the Easiest game's, one draw short — was installed
        // over this game's, frame 1's rush roll read 9 where the trace's
        // seeds give 3, and who=1's block 2 parted on 130 keys: `script_step`
        // 11 here against 6, timber 28 against 92, wealth 50 against 100,
        // `gatherers` 4 against 3, `1/2001`'s 90 gather slots, citizen
        // `1/2`'s walk and the rush's second farm `2007`. Every one of them
        // agrees now.
        for what in [
            "leader:script_step",
            "leader:bucket[1:timber]",
            "leader:bucket[2:wealth]",
            "leader:gatherers",
        ] {
            assert_eq!(row(1, -1, what), None, "who=1's {what} agrees");
        }
        assert!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && *f >= 2
                && [2, 2001, 2005, 2006, 2007].contains(o)),
            "the citizen, the camp's slots and the rush's sites agree past block 1"
        );
        // **What stands on the old word's block**: the original's tech
        // stamps (`Build::queue_up` writes `tech_frame` and
        // `tech_cat_frame[cat]`; §33.4's named unmodelled state) and two
        // pools of the script's buildings, the library `2005` in 64 and the
        // city `2000` in 66, which this crate never seats. No draw reads
        // either before the word.
        assert_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("2: ours 0 theirs 1"),
            "the Art of War's stamp"
        );
        assert_eq!(
            row(1, -2, "pool:64").as_deref(),
            Some("2: ours [] theirs [2005]"),
            "the original's pool 64"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 55), (2, 4), (8, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 1576, widened whole** (item
    /// 979): run352 is run346's game at run349's detail over blocks
    /// 1571..1827 (`rngcmp.py`: 1841 frames, 0 differing), walked from
    /// run346's own start. The word's frame writes block 1577.
    #[test]
    fn run352_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run352",
            "gamelog-run352-islands-toughest-1576.txt",
            WIDENING_SECOND_EAST_INDIES_1576,
            &[1_577],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run352 whole: blocks 1571..1827");
        assert!(
            w.missing.is_empty(),
            "run352 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 1577, as item 979 found it** (`docs/AI.md`
        // §81.4): the building `1/2009` parted on 91 of its 93 keys — its
        // gather tiles in another order, damage 1 here against 0, city −1
        // against 1 — and the citizen `1/2` walked to another cell
        // (`orders_x` 31944 against 37752). **Item 989's value diff**
        // (§82): the script's `place_woodcutter` places a camp at (38016,
        // 36480) on frame 976 and destroys it the same frame, and
        // `Build::close` gives its 48 tiles back; this crate kept them
        // marked, so the camp re-placed on 1576 went to (31680, 34752) with
        // 62 tiles (248 draws against 192). And the harness linked the
        // original's live `2009` to this crate's dead frame-976 site, which
        // is where damage 1 and city −1 came from. Every row of the word's
        // block now agrees, and so does capital `2000`'s `ter[1]` on 1571,
        // 0 against 2 → 2 on both: the census's `gather_at` had read the
        // dead camp's marks.
        assert!(
            !w.firsts.iter().any(|((who, o, _), (f, _))| *who == 1
                && [2, 2009].contains(o)
                && (1_572..=1_577).contains(f)),
            "the camp and the citizen agree from 1572 through the word's block"
        );
        assert_eq!(row(1, 2000, "city:ter[1]"), None, "the capital's timber");
        // **What stands, drawing nothing before the next word** (5606):
        // city `2007`'s site picture, one cell apart from its sweep of 1575
        // — `filled` one high here and the three `space` counts one low, so
        // `reg_land[11]` 102 against 103 — and, from the sweep before
        // (block 1571), the same city counted by the original while still a
        // site (`land` 9, `reg_cities[11]` 2) and not here. No mechanism is
        // named (DECISIONS 42).
        assert_eq!(
            row(1, -1, "leader:reg_land[11]").as_deref(),
            Some("1576: ours 102 theirs 103"),
            "the block before the word"
        );
        assert_eq!(
            row(1, 2007, "city:space[0]").as_deref(),
            Some("1576: ours 64 theirs 65"),
            "the city's site picture"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1571, 82), (1576, 5), (1601, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, 3776, widened whole** (item
    /// 979): run355 is run347's game at run350's detail over blocks
    /// 3771..4027 (`rngcmp.py`: 4041 frames, 0 differing), walked from
    /// run347's start. The word's frame writes block 3777.
    #[test]
    fn run355_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run355",
            &[(
                "gamelog-run355-greatlakes-toughest-3776.txt",
                WIDENING_SECOND_GREAT_LAKES_3776.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_3776,
            1,
            &[3_777],
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run355 whole: blocks 3771..4027");
        assert!(
            w.missing.is_empty(),
            "run355 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The word's block, 3777, as item 979 found it** (`docs/AI.md`
        // §81.4), East Indies' shape: a building who=1 places on the word's
        // frame, `1/2010` — 79 of its 80 keys its gather slots' tiles, and
        // damage 1 here against 0 — with `2007`'s `city_down` pointing at
        // it here and not there, the city `2000` one gatherer short (7
        // against 8), and citizens `1/6` and `1/28` on other orders.
        // **Item 989's value diff** (§82): the same close tail — a camp the
        // script destroyed kept its ground marked here — and the same dead
        // site linked in place of the live `2010`. All 127 keys of the
        // word's block agree now, and nothing parts on 3772..3804.
        assert!(
            !w.firsts
                .iter()
                .any(|((who, _, _), (f, _))| *who == 1 && (3772..3805).contains(f)),
            "who=1 agrees from the window's second block through 3804"
        );
        assert_eq!(row(1, 2000, "city:gatherers"), None, "the city's census");
        assert_eq!(row(1, 6, "orders_x"), None, "a citizen's order");
        // **What parts next, drawing nothing before the next word**
        // (4555): a city site `1/2009` the original counts in its sweep of
        // 3804 (`land` 9, `filled` 1) and this crate does not, and who=1's
        // `territory` 488 here against 290 — East Indies' `2007` on its
        // block 1571 has the same shape. No mechanism is named (DECISIONS
        // 42).
        assert_eq!(
            row(1, 2009, "city:land").as_deref(),
            Some("3805: ours 0 theirs 9"),
            "the city site's sweep"
        );
        assert_eq!(
            row(1, -1, "leader:territory").as_deref(),
            Some("3805: ours 488 theirs 290"),
            "and who=1's territory"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(3771, 78), (3801, 1), (3805, 4)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's East Indies word, 5606, widened whole** (item
    /// 989): run357 is run346's game at run352's detail over blocks
    /// 5601..5857, walked from run346's own start. The word's frame writes
    /// block 5607.
    #[test]
    fn run357_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = widen_east_indies_on(
            (
                "gamelog-run346-islands-toughest-24k-trace.txt",
                "rontrace-run346.log",
            ),
            "run357",
            "gamelog-run357-islands-toughest-5606.txt",
            WIDENING_SECOND_EAST_INDIES_5606,
            &[SECOND_WORD_EAST_INDIES + 1],
            true,
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run357 whole: blocks 5601..5857");
        assert!(
            w.missing.is_empty(),
            "run357 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The block before the word, 5605, parts on one key**
        // (`docs/AI.md` §82.4): who=1's unit `1/14` has a move order whose
        // `dest` reads 0 here against 1 there, and its path's third slot
        // parts on the word's block. The original's extra draw on 5606 is
        // `Unit::do_move+0xe84`. Block 5601 stands on 159 keys, the
        // families the window opened on. No mechanism is named (DECISIONS
        // 42).
        assert_eq!(
            row(1, 14, "order:move.dest").as_deref(),
            Some(r#"5605: Move { field: "dest", ours: 0, theirs: 1 }"#),
            "the unit's move, the block before the word"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(5601, 159), (5605, 1), (5607, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **The second pair's Great Lakes word, 4555, widened whole** (item
    /// 989): run356 is run347's game at run355's detail over blocks
    /// 4550..4806, walked from run347's start. The word's frame writes
    /// block 4556. **Since item 997 the word is 4593**, block 4594, inside
    /// the same window (44 blocks after its first and 212 before its last),
    /// so this capture is its widening too. **Since item 1002 the word is
    /// 4605**, block 4606, inside it again (56 blocks after its first and
    /// 200 before its last). **Since item 1012 the word is 4618**, block
    /// 4619 (69 blocks after its first and 187 before its last).
    #[test]
    fn run356_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run356",
            &[(
                "gamelog-run356-greatlakes-toughest-4555.txt",
                WIDENING_SECOND_GREAT_LAKES_4555.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4555,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run356 whole: blocks 4550..4806");
        assert!(
            w.missing.is_empty(),
            "run356 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The old word's block, 4555, agrees** (item 997, `docs/COMBAT.md`
        // §64): `1/21`'s attack-move no longer takes the human's unarmed
        // Woodcutter's Camp on 4554, so its `ATTACK_TO` stands on both
        // sides until the army's attack on 4605, and since item 1012 its
        // order agrees through that too, first parting on 4761.
        assert_eq!(
            row(1, 21, "order:kind").as_deref(),
            Some("4761: Kind { ours: 10, theirs: 1 }"),
            "the unit's order, which parted on 4555 until item 997"
        );
        // **The old word, 4593, agrees** (item 1002, `docs/COMBAT.md`
        // §65): `1/26`'s chase on the human's city `0/2000` stops at
        // (5079, 31628) on block 4592 on both sides — a building target is
        // asked without the `0x90` margin — and holds its `ATTACK` there
        // until block 4605, where both sides drop it; since item 1012 the
        // army's attack puts it back on the city on 4605 on both sides, and
        // neither its order nor its position parts again in the window.
        assert_eq!(
            row(1, 26, "order:kind"),
            None,
            "the unit's order, which parted on 4592 until item 1002"
        );
        assert_eq!(row(1, 26, "pos"), None, "and where it stands");
        // **The old word, 4605, agrees** (item 1012, `docs/COMBAT.md` §66):
        // the army group's attack-move looks on 4605 (`1/15`, `(4605 + 15)
        // % 15 == 0`), finds the human's city, and hands it to the whole
        // group — `Group::action_attack(…, QUEUE_FIRST, 4)` — whose retarget
        // takes a building with the word 2, and `1/17`'s ring starts
        // mid-face. Every member holds the city's `ATTACK` over the
        // re-issued `GROUP_ATTACK_TO` on block 4606 on both sides; `1/9`
        // first parts on 4771.
        assert_eq!(
            row(1, 9, "order:kind").as_deref(),
            Some("4771: Kind { ours: 10, theirs: 1 }"),
            "an army member's order, which parted on 4606 until item 1012"
        );
        // **The new word, 4618, writes block 4619** (no mechanism is
        // named, DECISIONS 42): ours spends 94 `Guy::set_anim` draws under
        // `Unit::do_idle` where the original spends 4 under
        // `Unit::do_non_flat_gather`. Block 4619 parts on `1/0`'s group,
        // 66 here against 67 there, its idle and its move, and on `1/7`'s
        // gather `wait`; block 4617 on `1/24`'s chase spot.
        assert_eq!(
            row(1, 0, "group").as_deref(),
            Some("4619: ours 66 theirs 67"),
            "the unit's group on the new word's block"
        );
        assert_eq!(
            row(1, 7, "order:gather.wait").as_deref(),
            Some(r#"4619: Gather { field: "wait", ours: 361, theirs: 335 }"#),
            "the gatherer's wait on the new word's block"
        );
        assert_eq!(
            row(1, 24, "orders_x").as_deref(),
            Some("4617: ours 4200 theirs 4104"),
            "the chase spot two blocks before it"
        );
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4550, 86), (4575, 18), (4585, 1)],
            "the blocks keys first part on, the first three"
        );
    }

    /// **run346 — East Indies at Toughest.** The lobby read back from the
    /// dump's own `GAME INFO` is 5, and the harness stands the simulation up
    /// at it; the word is pinned as a floor.
    #[test]
    fn run346_is_east_indies_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run346-islands-toughest-24k-trace.txt",
            "rontrace-run346.log",
            true,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run346's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= SECOND_WORD_EAST_INDIES && w.sequence >= SECOND_WORD_EAST_INDIES,
            "run346's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_EAST_INDIES}",
            w.count,
            w.sequence,
            w.last
        );
    }

    /// **run347 — Great Lakes at Toughest**, on the same terms.
    #[test]
    fn run347_is_great_lakes_at_toughest_and_its_word_holds() {
        let Some(w) = walk_second(
            "gamelog-run347-greatlakes-toughest-24k-trace.txt",
            "rontrace-run347.log",
            false,
        ) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run347's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= SECOND_WORD_GREAT_LAKES && w.sequence >= SECOND_WORD_GREAT_LAKES,
            "run347's word fell: count {}, sequence {} of {}; the floor is \
             {SECOND_WORD_GREAT_LAKES}",
            w.count,
            w.sequence,
            w.last
        );
    }
}
