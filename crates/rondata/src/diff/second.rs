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
    /// block 1) and the two after it kept standing.
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
        // **The parting's own rows on the word's block** — the seed is one
        // draw short from frame 0's index 24 on, so every later roll of the
        // frame reads another value: the AI scout `1/0`'s ring walk picks
        // another cell, two AI citizens' idle clocks and 46 gaia animals'
        // `cur_anim` roll other values. Beside them stand the families the
        // first pair carries on every block (player 0's census, `SITE.reg`,
        // `form`, the city record; `docs/AI.md` §33.4).
        assert_eq!(
            row(1, 0, "orders_x").as_deref(),
            Some("1: ours 41976 theirs 35832"),
            "the scout's walk target, x"
        );
        assert_eq!(
            row(1, 0, "orders_y").as_deref(),
            Some("1: ours 36600 theirs 42744"),
            "the scout's walk target, y"
        );
        assert_eq!(
            row(1, 1, "g.end_time[0]").as_deref(),
            Some("1: ours 232 theirs 33"),
            "an AI citizen's idle clock"
        );
        let gaia_anims = w
            .firsts
            .iter()
            .filter(|((who, _, what), (f, _))| *who == 8 && what == "gaia:cur_anim" && *f == 1)
            .count();
        assert_eq!(gaia_anims, 46, "gaia's animals roll other idles on block 1");
        assert_eq!(
            by.iter().take(3).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 130), (2, 44), (3, 2)],
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
