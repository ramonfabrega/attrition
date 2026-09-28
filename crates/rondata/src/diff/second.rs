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
    /// 4619 (69 blocks after its first and 187 before its last). **Since
    /// item 1014 the word is 4673**, block 4674 (124 blocks after its first
    /// and 132 before its last). **Since item 1023 the word is 4688**, block
    /// 4689 (139 blocks after its first and 117 before its last). **Since
    /// item 1034 the word is 4781**, block 4782 (232 blocks after its first
    /// and 24 before its last). **Since item 1040 the word is 4846**, block
    /// 4847, past this window: run373 is its widening.
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
        // order agrees through that too; it first parted again on 4761
        // until item 1028, and since then not in the window.
        assert_eq!(
            row(1, 21, "order:kind"),
            None,
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
        // first parted on 4771 until item 1014, and since then its order
        // agrees through the window.
        assert_eq!(
            row(1, 9, "order:kind"),
            None,
            "an army member's order, which parted on 4606 until item 1012"
        );
        // **The old word, 4618, agrees** (item 1014, `docs/SCOUT.md` §8.3):
        // the AI scout `1/0` chooses its explore target on tick 4506 with
        // the human's cells undoubled — `treaties[0] & 3` is the met bit,
        // set on 4456 — and takes cell (3, 42), (2808, 32760), on both
        // sides, where ours took (2, 40), (2040, 31224), arrived on 4618
        // and idled. Its order agrees from block 4550 to its next explore
        // target on 4737; `1/0`'s group and `1/7`'s gather `wait`, which
        // parted on 4619, no longer part in the window. That target read
        // 1272 here from item 1023 and 2808 again from item 1028, against
        // the original's 504; **since item 1034 it agrees**, and the
        // scout's order agrees through the window. Which read of the city
        // moved it is not established: the fix changed only the city's
        // three flags and its heal (`docs/COMBAT.md` §69).
        assert_eq!(
            row(1, 0, "order:move.x"),
            None,
            "the scout's target, which parted on 4550 until item 1014 and on \
             4737 until item 1034"
        );
        assert_eq!(
            row(1, 0, "group"),
            None,
            "the scout's group, 4619 until 1014"
        );
        assert_eq!(
            row(1, 7, "order:gather.wait"),
            None,
            "the gatherer's wait, 4619 until item 1014"
        );
        // **The old word, 4673, agrees** (item 1023, `docs/COMBAT.md` §67):
        // on 4616, the review's phase for `1/24` (`(4616 + 24) % 16 == 0`),
        // its target, the citizen `0/3`, is walking away and out of reach
        // of the chase spot, so `check_target_path` re-aims: block 4617
        // prints the move at (4104, 31512) on both sides, where ours kept
        // (4200, 31608). `1/24`'s order (kind 10 against 1 on 4673) and
        // `stopped` (4674) no longer part; its rows agree from 4605 to 4688.
        // Its order parted again on 4722 until item 1028, and since then
        // not in the window.
        assert_eq!(
            row(1, 24, "order:kind"),
            None,
            "the chaser's order, which parted on 4673 until item 1023"
        );
        // **The old word, 4688, agrees** (item 1028, `docs/COMBAT.md` §68):
        // `1/24` is a computer's raider (stance 3), and `compare_target`'s
        // RAID arm scores the citizen `0/3` at 9800 and the human's scout
        // `0/0` at 15, as run368's packet does on logger frame 4688. So its
        // one-in-five re-search keeps `0/3`, and it strikes: `recharging`
        // (0 against 33 until 1028) no longer parts on block 4689. Its walk
        // spot does, (3763, 31600) against the cell centre (3768, 31608).
        assert_eq!(
            row(1, 24, "recharging"),
            None,
            "the chaser's strike, on 4689 until item 1028"
        );
        assert_eq!(
            row(1, 24, "orders_x").as_deref(),
            Some("4689: ours 3763 theirs 3768"),
            "and its spot, which parted on 4617 until item 1023"
        );
        // **The old word, 4690, agrees** (item 1034, `docs/COMBAT.md` §69):
        // `1/26`'s first strike on the human's city on 4657 sets its
        // `city_flags` `0x2 | 0x4 | 0x8` on both sides, and the city no
        // longer heals the wound off (ours 1/0 against 2/5 on 4661 until
        // 1034), so 4690's strike is not a first wound and throws no
        // `take_damage+0xe1`. The three bits now agree through the window,
        // `0x4`'s clear on 4801 (`(4800 + 2000) % 200 == 0`) included.
        for bit in ["0x2", "0x4", "0x8"] {
            assert_eq!(
                row(0, 2000, &format!("city:city_flags[{bit}]")),
                None,
                "the human city's {bit}, which 1034 carries"
            );
        }
        // **A Hoplite on the city's north face strikes due south**
        // (item 1040, `docs/COMBAT.md` §70.6): `1/22`'s facing stood parted
        // from 4758 (ours the centre's bearing, 0x959a0000, against
        // 0x80000000) until the side arm.
        assert_eq!(
            row(1, 22, "g.angle[0]"),
            None,
            "the Hoplite's facing, which parted on 4758 until item 1040"
        );
        // What the heal had hidden since 4661: `1/26`'s strikes from the
        // second on landed here a frame after the original's (4689, 4716,
        // 4722 there; 4690, 4717, 4723 here) until item 1040: its stones
        // left the Slinger's own square, not its bay (`launch::BAYS`, piece
        // 32, `docs/COMBAT.md` §70), and flew a frame long across a
        // boundary. Now every strike lands on the original's frame.
        assert_eq!(
            row(0, 2000, "build:damage"),
            None,
            "the city's damage, one strike behind on 4690 until item 1040"
        );
        assert_eq!(
            row(0, 3, "hits:damage").as_deref(),
            Some("4723: ours 3 theirs 2"),
            "the citizen's damage, which first parts on 4723"
        );
        // **The old word, 4781, agrees** (item 1040, `docs/COMBAT.md` §70):
        // `1/24`'s stone at the citizen `0/4`, launched on 4775, leaves the
        // Slinger's bay and flies 5 frames on both sides, so `0/4` is struck
        // on 4779 on both; and a citizen walking to its drop site under its
        // `GATHER` is not on duty (`Unit::on_duty@005fff70`), so it does not
        // turn on the Slinger (`600863`): no `ATTACK` on 4780, no
        // `fight+0x9b0` on 4781. None of `0/4`'s rows parts in the window.
        for what in ["damage_frame", "order:kind", "pos"] {
            assert_eq!(
                row(0, 4, what),
                None,
                "the citizen's {what}, which parted on 4780 until item 1040"
            );
        }
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4550, 76), (4585, 1), (4598, 1)],
            "the blocks keys first part on, the first three (86, 18 and 1 on \
             4550, 4575 and 4585 until item 1014, the scout's ten and eighteen)"
        );
    }

    /// **The second pair's Great Lakes word, 4846, widened whole** (item
    /// 1040): run373 is run347's game at run356's detail over blocks
    /// 4841..5097, walked from run347's start. The word's frame writes
    /// block 4847, six blocks after the window's first. **Within the same
    /// item the word moved to 4852 and then 4877**, block 4878, inside the
    /// window (37 blocks after its first and 219 before its last).
    #[test]
    fn run373_s_word_frame_is_widened_whole() {
        use std::collections::BTreeMap;
        let Some(w) = crate::diff::harness::tests::widen_great_lakes_on(
            (
                "gamelog-run347-greatlakes-toughest-24k-trace.txt",
                "rontrace-run347.log",
            ),
            "run373",
            &[(
                "gamelog-run373-greatlakes-toughest-4846.txt",
                WIDENING_SECOND_GREAT_LAKES_4846.0,
            )],
            WIDENING_SECOND_GREAT_LAKES_4846,
            1,
            &[SECOND_WORD_GREAT_LAKES + 1],
        ) else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        assert_eq!(w.blocks, 257, "run373 whole: blocks 4841..5097");
        assert!(
            w.missing.is_empty(),
            "run373 carries every key: {:?}",
            w.missing
        );
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        // **The old word, 4846, agrees** (item 1040, `docs/COMBAT.md`
        // §70.6): the Hoplite `1/19` stands on the row above the human's
        // city and strikes it due south on both sides — a building is
        // struck square to its side — where ours turned 21° to the centre,
        // deferred the swing and struck a frame late. Its figure first
        // parts on the army's tick, 4861.
        assert_eq!(
            first(1, 19, "g.angle[0]").map(|(f, _)| f),
            Some(4_861),
            "the Hoplite's facing, which parted on 4847 until item 1040"
        );
        // **The old word, 4852, agrees** (§70.7): `1/24`'s one-in-five
        // re-search on 4852 names `0/3` on both sides and freezes the
        // frame, so its attack slot holds at 32 of 33 on block 4853 and
        // wraps on 4854; its clock no longer parts, and its figure first
        // parts on the army's tick too.
        for what in ["g.cur_time[0]", "g.last_time[0]", "g.end_time[0]"] {
            assert!(
                first(1, 24, what).is_none_or(|(f, _)| f > 4_853),
                "the Slinger's {what}, which parted on 4853 until item 1040: {:?}",
                first(1, 24, what)
            );
        }
        // **The new word, 4877, writes block 4878** (no mechanism is named,
        // DECISIONS 42): ours 7 draws and the original 8, parting at index
        // 1, where the original spends `Guy::set_anim+0x97a <
        // Unit::move_step+0x823`. The earliest block past the window's
        // standing rows is 4861, frame 4860, the army's tick (`4860 ≡ 252
        // mod 256`): 121 keys, who=1's army members walking off there and
        // holding the city here.
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(4_841, 111), (4_861, 121), (4_862, 13)],
            "the blocks keys first part on, the first three"
        );
        // A standing row the window opens on: the citizen `0/4` has taken
        // more here since run356's window closed (block 4841: `damage` 6
        // 10/16 here, 5 0/16 there).
        assert_eq!(
            first(0, 4, "hits:damage"),
            Some((4_841, "ours 6 theirs 5".to_string())),
            "the citizen's damage, standing from the window's first block"
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
