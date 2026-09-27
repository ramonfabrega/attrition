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
            &[SECOND_WORD_EAST_INDIES + 1],
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
        // **The move's value diff** (item 971, `docs/AI.md` §80.5). Before
        // `think_spellcaster`'s coin, frame 0 was a draw short from index
        // 24 on and the AI scout `1/0`'s ring walk read another cell: on
        // block 1 its `orders_x`/`orders_y` were 41976/36600 here against
        // 35832/42744, its path 9 slots against 3, and 28 of its rows
        // parted on blocks 1..3. With the coin every one of them agrees
        // until block 97, where the walk after next parts.
        assert_eq!(
            row(1, 0, "orders_x").as_deref(),
            Some("97: ours 38136 theirs 41976"),
            "the scout's walk target agrees until block 97"
        );
        assert!(
            !w.firsts
                .iter()
                .any(|((who, o, _), (f, _))| (*who, *o) == (1, 0) && *f < 97),
            "none of the scout's rows parts before block 97"
        );
        // **What stands on the word's block with the stream agreeing** —
        // 119 keys, none of them moved by the coin: player 0's census,
        // `SITE.reg`, `form` and the city record (the first pair's families,
        // `docs/AI.md` §33.4), two AI citizens' idle clocks (`1/1`'s
        // `g.end_time[0]` 232 here, 33 there) and 46 gaia animals'
        // `cur_anim`. The draws agree through frame 9, so none of these
        // rolls a different value; they are what block 1 reads on this
        // game with nothing parted.
        assert_eq!(
            row(1, 1, "g.end_time[0]").as_deref(),
            Some("1: ours 232 theirs 33"),
            "an AI citizen's idle clock, standing"
        );
        let gaia_anims = w
            .firsts
            .iter()
            .filter(|((who, _, what), (f, _))| *who == 8 && what == "gaia:cur_anim" && *f == 1)
            .count();
        assert_eq!(gaia_anims, 46, "gaia's animals, standing on block 1");
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 119), (2, 27), (3, 2)],
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
            &[SECOND_WORD_GREAT_LAKES + 1],
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
        // **Block 1 stands** with frame 0's draws agreeing: 58 keys, the
        // first pair's families (player 0's census, `SITE.reg`, `form`, the
        // city record; `docs/AI.md` §33.4) and nothing of who=1's economy.
        // **The word's block, 2, is who=1's opening** (`docs/AI.md` §80.5):
        // the original's script stands at step 6 where this crate's is at
        // 11, it holds 92 timber and 100 wealth against 28 and 50, it has
        // stamped a tech on frame 1, and two buildings are in its pools;
        // 90 of the 130 keys are `1/2001`'s gather slots, in another
        // order. No mechanism is named here (DECISIONS 42).
        for (what, want) in [
            ("leader:script_step", "2: ours 11 theirs 6"),
            ("leader:bucket[1:timber]", "2: ours 28 theirs 92"),
            ("leader:bucket[2:wealth]", "2: ours 50 theirs 100"),
            ("leader:tech_frame", "2: ours 0 theirs 1"),
            ("leader:gatherers", "2: ours 4 theirs 3"),
        ] {
            assert_eq!(row(1, -1, what).as_deref(), Some(want), "who=1's {what}");
        }
        assert_eq!(
            row(1, -2, "pool:64").as_deref(),
            Some("2: ours [] theirs [2005]"),
            "the original's pool 64"
        );
        let slots = w
            .firsts
            .iter()
            .filter(|((who, o, _), (f, _))| (*who, *o) == (1, 2001) && *f == 2)
            .count();
        assert_eq!(slots, 90, "1/2001's gather slots on the word's block");
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 58), (2, 130), (4, 3)],
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
