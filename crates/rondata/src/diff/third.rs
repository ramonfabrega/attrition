//! **The third scored map** (DECISIONS 54 §3, item 1066): Great Sahara,
//! `MAP_STYLE 7`, in the first pair's lobby — Easiest, seed 12345, the
//! profile's lobby — so the one setting moved against the first pair is the
//! map. Three captures: run381, a `DUMP_ALL` start in run38's shape, which is
//! the sibling everything here stands up from; run382, 1,850 blocks in
//! run33's shape, the score; and run383, the draw stream to 24,000 at
//! `cover=0`. The floors are `FLOORS[2]` and [`LONG_WORD_GREAT_SAHARA`];
//! `docs/AI.md` §83 has the map and the first parting. No mechanism is named
//! here (DECISIONS 54 §3): a later pass may choose the third map again
//! before an item is opened against its word.

use super::*;

use crate::diff::testkit::*;
use crate::testenv::{dump, install};

/// run381: the `DUMP_ALL` start — the height table, the cells, the checksum
/// trace, the herds and the frame seeds of Great Sahara at seed 12345.
pub(crate) const SAHARA_START: &str = "gamelog-run381-greatsahara-start.txt";
/// run382: the score capture, run33's shape on map 7.
pub(crate) const SAHARA_SCORE: (&str, &str) = (
    "gamelog-run382-greatsahara-longtrace.txt",
    "rontrace-run382.log",
);
/// run383: the draw stream to 24,000 frames, `cover=0`.
pub(crate) const SAHARA_LONG: (&str, &str) = (
    "gamelog-run383-greatsahara-24k-trace.txt",
    "rontrace-run383.log",
);

/// The row in [`FLOORS`] that is Great Sahara's.
pub(crate) fn sahara_floors() -> &'static MapFloors {
    FLOORS
        .iter()
        .find(|f| f.map == "GreatSahara")
        .expect("FLOORS has a GreatSahara row")
}

/// Where a Great Sahara trace's draw stream first parts from this crate's:
/// the frame the count parts, the frame the sequence parts, the trace's
/// last frame, and the parting frame's row.
pub(crate) struct Word {
    pub count: i64,
    pub sequence: i64,
    pub last: i64,
    pub row: String,
    /// The lobby's difficulty as the simulation stood up at it, read from
    /// the dump's own `GAME INFO` (item 1221).
    pub difficulty: i32,
}

/// A Great Sahara capture walked from its own start dump with run381's head
/// borrowed, until the draw stream parts (or the trace ends). `None` when
/// the captures are not on this machine.
pub(crate) fn walk_sahara(capture: (&str, &str)) -> Option<Word> {
    walk_sahara_from(SAHARA_START, capture)
}

/// [`walk_sahara`] from a named start dump: run381 for the first pair's
/// lobby, run468 for the second's (item 1221, `diff::sahara_toughest`).
pub(crate) fn walk_sahara_from(start: &str, (gamelog, tracelog): (&str, &str)) -> Option<Word> {
    let inst = install()?;
    let (Some(path), Some(sib), Some(tr)) = (dump(gamelog), dump(start), trace(tracelog)) else {
        eprintln!("skipping: no {gamelog}/{start} (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    let difficulty = built.sim.lobby.difficulty;
    let last = tr.frames.last().map_or(0, |(n, _)| *n);
    let (mut count, mut sequence) = (None, None);
    for _ in 0..last {
        let at = built.sim.frame;
        built.tick();
        crate::diff::harness::debug_leader(&built, at);
        crate::diff::harness::debug_ammo(&built, at);
        crate::diff::harness::debug_builds(&built, at);
        crate::diff::harness::debug_tech(&built, at);
        // `RON_DEBUG_UNIT` and `RON_DEBUG_SITES=<lo>-<hi>`'s attributed
        // sites, as the second pair's long walk prints them (item 1206).
        crate::diff::harness::debug_watch(&built, at + 1);
        if crate::diff::harness::site_window().is_some_and(|(lo, hi)| (lo..=hi).contains(&at)) {
            for (label, who) in crate::diff::harness::attributed_sites(&built) {
                eprintln!("  f{at} {who}: {label}");
            }
        }
        let Some((f, ours)) = built.frame_sites.last() else {
            continue;
        };
        let theirs = tr.labels(*f);
        if count.is_none() && ours.len() != theirs.len() {
            count = Some(*f);
        }
        if sequence.is_none() && *ours != theirs {
            sequence = Some(*f);
        }
        if count.is_some() {
            break;
        }
    }
    let (count, sequence) = (count.unwrap_or(last), sequence.unwrap_or(last));
    let mut row = String::new();
    if let Some((f, ours)) = built.frame_sites.iter().find(|(f, _)| *f == sequence) {
        let theirs = tr.labels(*f);
        let at = (0..ours.len().max(theirs.len()))
            .find(|&i| ours.get(i) != theirs.get(i))
            .unwrap_or(0);
        row = format!(
            "frame {f}: ours {} theirs {} — at {at}, ours {:?} theirs {:?}",
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
    eprintln!("{gamelog}: word parts at {count}, sequence at {sequence} of {last}; {row}");
    Some(Word {
        count,
        sequence,
        last,
        row,
        difficulty,
    })
}

/// **The first word's block, 9** (item 1066): frame 8 writes it. Item 1133
/// moved the word to 12783, past every block run382 holds, and this block
/// stays the move's value diff and the coverage driver's run382 window.
pub(crate) const SAHARA_FIRST_WORD_BLOCK: i64 = 9;

/// **The widening's window** (item 1066): run382's blocks 1..259. The
/// word's frame 8 writes block 9; every block before it is on the capture,
/// and 250 follow it (DECISIONS 50 §7, stated in blocks).
pub(crate) const WIDENING_GREAT_SAHARA: (i64, i64) = (1, 259);

/// run382's blocks over [`WIDENING_GREAT_SAHARA`], walked from run382's
/// start with run381's head: every record `widen_block` reads on every unit
/// and building of both players, both leaders' rows and gaia's animals —
/// every record run10's detail prints. It prints no `GROUPDATA`, so the
/// group record, the attack row's `second::widen_records` pass and the pool
/// lists are not walked. `None` when the captures are not on this machine.
pub(crate) fn sahara_first_word_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_SCORE,
        "run382",
        &[(SAHARA_SCORE.0, WIDENING_GREAT_SAHARA.0)],
        WIDENING_GREAT_SAHARA,
        // run10's detail prints no `GROUPDATA`: a pool or group row here
        // would compare against a record the capture never wrote.
        i64::MAX,
        &[SAHARA_FIRST_WORD_BLOCK],
        false,
    )
}

/// run416: run383's game at run414's detail over blocks 12778..13034, the
/// long word 12783's widening (item 1133) — six blocks before the word's
/// block 12784 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_12783: &str = "gamelog-run416-greatsahara-12783.txt";

/// **run416's window** (item 1133): blocks 12778..13034.
pub(crate) const WIDENING_GREAT_SAHARA_12783: (i64, i64) = (12_778, 13_034);

/// run416's blocks over [`WIDENING_GREAT_SAHARA_12783`], walked from
/// run383's start with run381's head: every record run414's detail prints,
/// both directions, the pool and group rows and the attack order's row
/// included. `None` when the captures are not on this machine.
pub(crate) fn sahara_long_word_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run416",
        &[(SAHARA_WORD_12783, WIDENING_GREAT_SAHARA_12783.0)],
        WIDENING_GREAT_SAHARA_12783,
        1,
        &[SAHARA_12783_BLOCK],
        true,
    )
}

/// **The old word's block, 12784** (item 1133): frame 12783 writes it.
/// Item 1147 moved the word to 13182, and run416 stays the move's value
/// diff.
pub(crate) const SAHARA_12783_BLOCK: i64 = 12_784;

/// run417: run383's game at run414's detail over blocks 13177..13433, the
/// long word 13182's widening (item 1147) — six blocks before the word's
/// block 13183 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_13182: &str = "gamelog-run417-greatsahara-13182.txt";

/// **run417's window** (item 1147): blocks 13177..13433.
pub(crate) const WIDENING_GREAT_SAHARA_13182: (i64, i64) = (13_177, 13_433);

/// run417's blocks over [`WIDENING_GREAT_SAHARA_13182`], walked from
/// run383's start with run381's head, as [`sahara_long_word_window`] walks
/// run416. `None` when the captures are not on this machine.
pub(crate) fn sahara_13182_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run417",
        &[(SAHARA_WORD_13182, WIDENING_GREAT_SAHARA_13182.0)],
        WIDENING_GREAT_SAHARA_13182,
        1,
        &[SAHARA_13182_BLOCK],
        true,
    )
}

/// **The old word's block, 13183** (item 1147): frame 13182 writes it.
/// Item 1163 moved the word to 14587, and run417 stays the move's widening.
pub(crate) const SAHARA_13182_BLOCK: i64 = 13_183;

/// run418: run383's game at run414's detail over blocks 14582..14838, the
/// long word 14587's widening (item 1163) — six blocks before the word's
/// block 14588 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_14587: &str = "gamelog-run418-greatsahara-14587.txt";

/// **run418's window** (item 1163): blocks 14582..14838.
pub(crate) const WIDENING_GREAT_SAHARA_14587: (i64, i64) = (14_582, 14_838);

/// run418's blocks over [`WIDENING_GREAT_SAHARA_14587`], walked from
/// run383's start with run381's head, as [`sahara_long_word_window`] walks
/// run416. `None` when the captures are not on this machine.
pub(crate) fn sahara_14587_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run418",
        &[(SAHARA_WORD_14587, WIDENING_GREAT_SAHARA_14587.0)],
        WIDENING_GREAT_SAHARA_14587,
        1,
        &[SAHARA_14587_BLOCK],
        true,
    )
}

/// **The old word's block, 14588** (item 1163): frame 14587 writes it.
/// Item 1171 moved the word to 15586, and run418 stays the move's value
/// diff.
pub(crate) const SAHARA_14587_BLOCK: i64 = 14_588;

/// run426: run383's game at run414's detail over blocks 15581..15837, the
/// long word 15586's widening (item 1171) — six blocks before the word's
/// block 15587 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_15586: &str = "gamelog-run426-greatsahara-15586.txt";

/// **run426's window** (item 1171): blocks 15581..15837.
pub(crate) const WIDENING_GREAT_SAHARA_15586: (i64, i64) = (15_581, 15_837);

/// **The old word 15586's block, 15587** (item 1177): the word moved to
/// 15982, past run426's last block, and this block stays the move's value
/// diff and the coverage driver's run426 window.
pub(crate) const SAHARA_15586_BLOCK: i64 = 15_587;

/// run426's blocks over [`WIDENING_GREAT_SAHARA_15586`], walked from
/// run383's start with run381's head, as [`sahara_long_word_window`] walks
/// run416. `None` when the captures are not on this machine.
pub(crate) fn sahara_15586_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run426",
        &[(SAHARA_WORD_15586, WIDENING_GREAT_SAHARA_15586.0)],
        WIDENING_GREAT_SAHARA_15586,
        1,
        &[SAHARA_15586_BLOCK],
        true,
    )
}

/// run428: run383's game at run414's detail over blocks 15977..16233, the
/// long word 15982's widening (item 1177) — six blocks before the word's
/// block 15983 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_15982: &str = "gamelog-run428-greatsahara-15982.txt";

/// **run428's window** (item 1177): blocks 15977..16233.
pub(crate) const WIDENING_GREAT_SAHARA_15982: (i64, i64) = (15_977, 16_233);

/// run442: run383's game at run414's detail over blocks 16676..16932, the
/// long word 16681's widening (item 1189) — six blocks before the word's
/// block 16682 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_16681: &str = "gamelog-run442-greatsahara-16681.txt";

/// **run442's window** (item 1189): blocks 16676..16932.
pub(crate) const WIDENING_GREAT_SAHARA_16681: (i64, i64) = (16_676, 16_932);

/// run442's blocks over [`WIDENING_GREAT_SAHARA_16681`], walked from
/// run383's start with run381's head, as [`sahara_15982_window`] walks
/// run428. `None` when the captures are not on this machine.
pub(crate) fn sahara_16681_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run442",
        &[(SAHARA_WORD_16681, WIDENING_GREAT_SAHARA_16681.0)],
        WIDENING_GREAT_SAHARA_16681,
        1,
        &[SAHARA_16681_BLOCK],
        true,
    )
}

/// run449: run383's game at run442's detail over blocks 17618..17874, the
/// long word 17623's widening (item 1194) — six blocks before the word's
/// block 17624 and 250 after, on the click-free lane.
pub(crate) const SAHARA_WORD_17623: &str = "gamelog-run449-greatsahara-17623.txt";

/// **run449's window** (item 1194): blocks 17618..17874.
pub(crate) const WIDENING_GREAT_SAHARA_17623: (i64, i64) = (17_618, 17_874);

/// run449's blocks over [`WIDENING_GREAT_SAHARA_17623`], walked from
/// run383's start with run381's head, as [`sahara_16681_window`] walks
/// run442. `None` when the captures are not on this machine.
pub(crate) fn sahara_17623_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run449",
        &[(SAHARA_WORD_17623, WIDENING_GREAT_SAHARA_17623.0)],
        WIDENING_GREAT_SAHARA_17623,
        1,
        &[SAHARA_17623_BLOCK],
        true,
    )
}

/// run457: run383's game at run449's detail over blocks 17140..17618, the
/// gap between run442's last block and run449's first (item 1206), where
/// who=1's four soldiers' positions parted with no draw spent.
pub(crate) const SAHARA_GAP_17140: &str = "gamelog-run457-greatsahara-gap-17140.txt";

/// **run457's window** (item 1206): blocks 17140..17618.
pub(crate) const WIDENING_GREAT_SAHARA_GAP: (i64, i64) = (17_140, 17_618);

/// run457's blocks over [`WIDENING_GREAT_SAHARA_GAP`], walked from run383's
/// start with run381's head, as [`sahara_17623_window`] walks run449.
/// `None` when the captures are not on this machine.
pub(crate) fn sahara_gap_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run457",
        &[(SAHARA_GAP_17140, WIDENING_GREAT_SAHARA_GAP.0)],
        WIDENING_GREAT_SAHARA_GAP,
        1,
        &[SAHARA_GAP_17493_BLOCK],
        true,
    )
}

/// **The move's value-diff block on run457, 17493** (item 1206): `1/51`
/// leaves army group 64 and `1/62` takes the formation over, and the
/// original's `curr` is turned by `1/62`'s waypoint.
pub(crate) const SAHARA_GAP_17493_BLOCK: i64 = 17_493;

/// **The old word 17623's block, 17624** (item 1206): the word moved to
/// 24000, the trace's end, and run449 stays the move's value diff.
pub(crate) const SAHARA_17623_BLOCK: i64 = 17_624;

/// run458: run383's game at run449's detail over its last blocks,
/// 23744..24000 (item 1206). The word 24000 is the trace's last frame and
/// writes block 24001, which run458 does not print: run383's closing dump
/// holds it, and `endpoint::great_sahara_endpoint_is_pinned` scores it.
pub(crate) const SAHARA_END: &str = "gamelog-run458-greatsahara-end.txt";

/// **run458's window** (item 1206): blocks 23744..24000, and the word's
/// block 24001 closes it.
pub(crate) const WIDENING_GREAT_SAHARA_END: (i64, i64) = (23_744, 24_001);

/// run458's blocks over [`WIDENING_GREAT_SAHARA_END`], walked from
/// run383's start with run381's head, as [`sahara_17623_window`] walks
/// run449. `None` when the captures are not on this machine.
pub(crate) fn sahara_end_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run458",
        &[(SAHARA_END, WIDENING_GREAT_SAHARA_END.0)],
        WIDENING_GREAT_SAHARA_END,
        1,
        &[LONG_WORD_GREAT_SAHARA + 1],
        true,
    )
}

/// **The old word 16681's block, 16682** (item 1194): the word moved to
/// 17623, past run442's last block, and this block stays the move's value
/// diff.
pub(crate) const SAHARA_16681_BLOCK: i64 = 16_682;

/// **The old word 15982's block, 15983** (item 1189): the word moved to
/// 16681, past run428's last block, and this block stays the move's value
/// diff.
pub(crate) const SAHARA_15982_BLOCK: i64 = 15_983;

/// run428's blocks over [`WIDENING_GREAT_SAHARA_15982`], walked from
/// run383's start with run381's head, as [`sahara_15586_window`] walks
/// run426. `None` when the captures are not on this machine.
pub(crate) fn sahara_15982_window() -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[SAHARA_START],
        true,
        SAHARA_LONG,
        "run428",
        &[(SAHARA_WORD_15982, WIDENING_GREAT_SAHARA_15982.0)],
        WIDENING_GREAT_SAHARA_15982,
        1,
        &[SAHARA_15982_BLOCK],
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The third map's word, widened whole** (item 1066, CLAUDE.md "A
    /// word is pinned with its widening"): [`sahara_first_word_window`] over
    /// run382's blocks 1..259, both directions, every record run10's detail
    /// prints. The word's frame 8 writes block 9. No mechanism is named
    /// (DECISIONS 54 §3); the rows below are the value diff.
    #[test]
    fn run382_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_first_word_window() else {
            return;
        };
        pin_eq!(w.blocks, 259, "run382 over blocks 1..259");
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // `RON_FIRSTS=1` prints every key's first parting past block 1.
        if std::env::var("RON_FIRSTS").is_ok() {
            for ((who, o, what), (f, row)) in w.firsts.iter().filter(|(_, (f, _))| *f > 1) {
                eprintln!("  first {f} {who}/{o} {what}: {row}");
            }
        }
        // **What run10's detail does not print** is a key unprinted, never
        // a parting: the leader's long record (`LEADERS=1` prints the short
        // one) and gaia's clocks (`GUYS=2` prints no `cur_anim`).
        pin_eq!(w.missing.len(), 1_064, "the keys run382 does not print");
        pin!(
            w.missing.contains("gaia:cur_anim") && w.missing.contains("resources[0:food]"),
            "gaia's clocks and the long leader record are unprinted"
        );
        // **Block 1 stands** with frame 0's draws agreeing (99 against 99):
        // 22 keys, all the first pair's standing families (`docs/AI.md`
        // §33.4) — `form` on the ten citizens and the scout, and the human
        // capital's city record, which this crate does not fill for player
        // 0. None is who=1's economy.
        pin!(
            w.firsts.iter().all(|((who, o, what), (f, _))| *f > 1
                || what == "form"
                || (*who, *o) == (0, 2000) && what.starts_with("city:")),
            "block 1's rows are the standing families"
        );
        // **The move's value diff (item 1133): block 6 and the old word's
        // block 9 agree.** Until the setup's birth was modelled
        // (`docs/COLLISION.md` §20), `1/2` parted here first: blocked by
        // `1/1` on frame 5, ours refused both sidestep cells on `1/1`'s
        // corner (797, 329) and marked `collide` 1 with a one-leg move to
        // (38232, 16056), where the original, whose `1/2` had cleared that
        // corner coming out of the camp `1/2001`, pushed the sidestep to
        // (38328, 15768) with `collide` 0 and a two-leg path. On block 9 the
        // collision's frame was 6 against 8 and the facing −328728576
        // against −1925840896. Every one of those rows now agrees.
        for what in [
            "collide",
            "order:move.dest_x",
            "order:move.dest_y",
            "path:length",
            "pos",
            "collide_frame",
            "angle:Facing",
        ] {
            pin_eq!(first(1, 2, what), None, "1/2's {what} agrees");
        }
        // **Block 202's one key agrees since item 1354**: the AI scout
        // `1/0`'s `mylos` was ours 6 against 4 — the Science epoch reaching
        // the derivation a frame before the original's `calc_unit_stats`
        // refreshed its cache. With the cache (`docs/VISION.md` §2) nothing
        // parts past block 1.
        pin_eq!(
            first(1, 0, "mylos"),
            None,
            "the scout's line of sight agrees"
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 22)],
            "the blocks keys first part on"
        );
        // Gaia's herds hold over the whole window.
        pin!(
            w.firsts.iter().all(|((who, _, _), _)| *who < 8),
            "gaia's animals agree to block 259"
        );
        // Item 1354, the `mylos` cache: 23 → 22.
        pin_eq!(w.firsts.len(), 22, "keys parted over the window");
    }

    /// **The third map's long word, 12783, widened whole** (item 1133):
    /// [`sahara_long_word_window`] over run416's blocks 12778..13034. The
    /// word's frame writes block 12784.
    #[test]
    fn run416_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_long_word_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run416 whole: blocks 12778..13034");
        pin!(
            w.missing.is_empty(),
            "run416 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 12778 stands on 133 keys**, the standing families: who=0's
        // leader counters this crate does not fill for the human, who=1's
        // `tech_cat_frame`, the citizens' `form`, `orders_x/y` and figure
        // angle, and the capital's city record.
        pin_eq!(
            row(1, -1, "leader:tech_frame").as_deref(),
            Some("12778: ours 0 theirs 8376"),
            "a standing row"
        );
        // **The move's value diff (item 1147): block 12780 agrees.** Until
        // the Peacocks term was wired (`docs/AI.md` §91), the AI's make list
        // parted here first: slots 1 and 10, the Mercenaries offer, priced
        // 9999999 here against 1632000 there, because who=1's `pop_cap` was
        // 50 against 55 and `cap × 5 / 6 < effective_pop` read 41 < 44 and
        // took `×20`. `pop_cap` is compared since item 1147 and agrees on
        // every block of the window.
        pin_eq!(
            row(1, -1, "leader:MAKE[10].val"),
            None,
            "the offer's other slot agrees over the window"
        );
        pin_eq!(row(1, -1, "leader:pop_cap"), None, "the cap agrees");
        pin_eq!(
            row(1, 39, "g.cur_anim[0]"),
            None,
            "the old word's block: 1/39's figure agrees"
        );
        // **The move's value diff (item 1163): the old word's block 12784
        // agrees.** Until Wine's research discount was carried
        // (`docs/AI.md` §94), `1/2014` queued the Militia research (66) at
        // 80 food and 80 metal here against 64 there, and the food and
        // metal buckets parted here, 70 and 199 against 86 and 215. The AI
        // holds Wine, and `get_cost`'s research arm takes 20% off.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "the old word's block: the food bucket agrees"
        );
        pin_eq!(
            row(1, 2014, "queue:queue[0].cost[0]"),
            None,
            "the old word's block: the Militia research's price agrees"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(12778, 133), (12782, 5), (12783, 1)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_12783_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(12778, 97)],
            "the blocks keys first part on, to the old word's"
        );
        // The make list's value parted on 12979 until item 1251, the
        // Feudalism offer at 10000 here against 40002 there: a civic epoch
        // who=1 could not afford until Dye took its quarter off the price
        // (`get_cost`'s library-line tail, `docs/AI.md` §99.8). It agrees.
        pin_eq!(
            row(1, -1, "leader:MAKE[1].val"),
            None,
            "the Feudalism offer's value agrees"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 248.
        // Item 1354, the `mylos` cache (`docs/VISION.md` §2): 198 → 175, the
        // 23 Citizens' `mylos` of block 12944, ours 4 against 2 a frame
        // ahead of the original's refresh.
        pin_eq!(w.firsts.len(), 175, "every key parted on run416");
    }

    /// **The third map's long word, 13182, widened whole** (item 1147):
    /// [`sahara_13182_window`] over run417's blocks 13177..13433. The
    /// word's frame writes block 13183.
    #[test]
    fn run417_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_13182_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run417 whole: blocks 13177..13433");
        pin!(
            w.missing.is_empty(),
            "run417 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 13177 stands on 136 keys**: run416's standing families.
        // ~~And 25 AI citizens' `myhits` 40 against 50 and `mylos` 2 against
        // 4, which run416 first shows on 12945~~: item 1111 built a
        // Citizen's hits and LOS off the Militia line (`docs/GOLDEN.md` §48).
        // **The move's value diff (item 1163)**: who=1's food and metal
        // buckets, 16 short here until Wine's research discount was carried
        // (run416's block 12784, `docs/AI.md` §94), agree.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "the food bucket agrees over the window"
        );
        pin_eq!(
            row(1, 1, "myhits").as_deref(),
            None,
            "a citizen's hits, agreeing since item 1111"
        );
        // **The make list agrees on 13181**, where it parted: slot 0 is the
        // Slingers (82) and slot 1 the Longbowmen (177), both at the
        // clamp. With the food 16 short, this crate could not afford the
        // Slingers (53 food against 55), took `check_income`'s factor 64
        // for 256, and the offer wrapped positive to 3462368 and fell out
        // of the list. What parts on 13181 is the city shift alone
        // (`docs/AI.md` §53.2): six slots' `city`, ours one above theirs.
        pin_eq!(
            row(1, -1, "leader:MAKE[1].val"),
            None,
            "the make list's values agree"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[0].t"),
            None,
            "the make list's head is the Slingers on both sides"
        );
        // **The old word's block, 13183** (frame 13182): the barracks
        // `1/2017` queues the Slingers on both sides.
        pin_eq!(
            row(1, 2017, "queue:queue[3].type"),
            None,
            "the old word's block: the barracks' queue agrees"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(13177, 136), (13181, 6)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= 13_183)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(13177, 97)],
            "the blocks keys first part on, to the old word's"
        );
        pin!(
            w.firsts
                .iter()
                .filter(|(_, (f, _))| *f == 13_181)
                .all(
                    |((_, _, what), _)| what.starts_with("leader:MAKE[") && what.ends_with(".city")
                ),
            "13181 is the city shift alone"
        );
        // **What parts next**: the rares the AI knows on 13184, 7 here
        // against 6, then an AI group of three on 13222 that the original
        // stands up and this crate does not.
        pin_eq!(
            row(1, -1, "leader:known_rares").as_deref(),
            Some("13184: ours 7 theirs 6"),
            "the known rares"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 173.
        pin_eq!(w.firsts.len(), 121, "every key parted on run417");
    }

    /// **The third map's long word, 14587, widened whole** (item 1163):
    /// [`sahara_14587_window`] over run418's blocks 14582..14838. The
    /// word's frame writes block 14588.
    #[test]
    fn run418_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_14587_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run418 whole: blocks 14582..14838");
        pin!(
            w.missing.is_empty(),
            "run418 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 14582 stands on 155 keys**: run417's standing families
        // (the city shift among them), and the AI's
        // army group, which the original stamps six frames later than this
        // crate — every member's `order:group.id` is 14330130 here against
        // 14336430 there — with `role` 0 here against 1379331 there.
        pin_eq!(
            row(1, 54, "myhits"),
            None,
            "a citizen's hits agree, since item 1111"
        );
        pin_eq!(
            row(1, 52, "order:group.id").as_deref(),
            Some(r#"14582: Group { field: "id", ours: 14330130, theirs: 14336430 }"#),
            "the army group's id"
        );
        // **The block before the old word's, 14587** (item 1171, the
        // move's value diff): on frame 14586 the original's army group 64
        // loses `1/28`–`1/30` and `1/65`–`1/67` to group 68, and its
        // arrays shift with the list — 15 slots, `1/52` fifth at `off`
        // (−3, 0), `curr` (68, 126). Until item 1171 this crate shifted
        // the list alone: 21 slots, and `1/52`, fifth, read `1/47`'s old
        // `curr` (−173, −16), walked for (28373, 20104) and stood at
        // (28637, 20233) against the original's (28623, 20241). Every
        // slot and `1/52`'s position now agree.
        for what in ["group:64.off[4]", "group:64.curr[4]", "group:64.curr[15]"] {
            pin_eq!(row(1, -3, what), None, "group 64's {what} agrees");
        }
        pin_eq!(row(1, 52, "pos"), None, "1/52's position agrees");
        // **The old word's block, 14588** (frame 14587, ours 9 draws
        // against 10 until item 1171): the original's `1/52` collided with
        // `1/30`, and so does this crate's now.
        pin_eq!(row(1, 52, "collide_o"), None, "the old word's collision");
        // What 14587 still parts on: the new group 68's speed pair, ours
        // 26 against 0 — standing since before item 1171, and no draw
        // reads it over the window.
        pin_eq!(
            row(1, -3, "group:68.speed").as_deref(),
            Some("14587: ours 26 theirs 0"),
            "group 68's speed"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_14587_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(14582, 107), (14584, 1), (14587, 2)],
            "the blocks keys first part on, to the old word's"
        );
        // **14661: the Spice route** (item 1189, the value diff of the
        // long word's move 15982 → 16681). A caravan's return home on
        // frame 14660 recomputes who=1's two Villages' route, and the
        // original's holds Spice: `trade_val` 176 here against 208 there
        // on `1/2000` and `1/2007`, then wealth's `income` and `resources`
        // 992 against 1056 on 14664, `bucket` on 14678 and `rate` 62
        // against 66 on 14777. `Caravan::trade_value`'s Spice arm now
        // reads the rare, and all seven agree (`docs/CARAVAN.md` §4).
        for (o, what) in [
            (2000, "city:trade_val"),
            (2007, "city:trade_val"),
            (-1, "leader:income[2:wealth]"),
            (-1, "leader:leftover[2:wealth]"),
            (-1, "leader:resources[2:wealth]"),
            (-1, "leader:bucket[2:wealth]"),
            (-1, "leader:rate[2:wealth]"),
        ] {
            pin_eq!(row(1, o, what), None, "1/{o}'s {what} agrees");
        }
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 161.
        pin_eq!(w.firsts.len(), 111, "every key parted on run418");
    }

    /// **The third map's long word, 15586, widened whole** (item 1171):
    /// [`sahara_15586_window`] over run426's blocks 15581..15837. The
    /// word's frame writes block 15587.
    #[test]
    fn run426_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_15586_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run426 whole: blocks 15581..15837");
        pin!(
            w.missing.is_empty(),
            "run426 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 15581 stands on 152 keys**: run418's standing families,
        // the army group among them — its members' `order:group.id` is
        // 15354136 here against 15360436 there, stamped six frames apart
        // again. Item 1189 took seven: who=1's two `trade_val`s, 176
        // against 208, and its five wealth rows (run418's block 14661).
        pin_eq!(
            row(1, 52, "order:group.id").as_deref(),
            Some(r#"15581: Group { field: "id", ours: 15354136, theirs: 15360436 }"#),
            "the army group's id"
        );
        // **15582: who=1's make list**, the values of two offers.
        pin_eq!(
            row(1, -1, "leader:MAKE[1].val").as_deref(),
            Some("15582: ours 59500 theirs 51000"),
            "a make offer's value"
        );
        // **15585: the raider `1/29`, and the move's value diff** (item
        // 1177): group 68's Longbowman reached the human's city at
        // `attack_dist` 2284 on 15584 with its squad-mate `1/30` two unit
        // cells off, so the building arm's `find_collision(my spot, o,
        // who, 1)` held the chase for one more step (`docs/COMBAT.md`
        // §65.8). Before 1177 ours ended it a frame early: `order:kind`
        // 10 against 1, `pos` (7292,27822) against (7269,27840), and the
        // word's block 15587 parted on its figure clock, 13 against 9.
        // Now every `1/29` row agrees across the window.
        for what in ["order:kind", "pos", "g.cur_anim[0]", "attack[0].in_range"] {
            pin_eq!(row(1, 29, what), None, "1/29's {what} agrees");
        }
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_15586_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(15581, 107), (15582, 2), (15584, 1), (15585, 1)],
            "the blocks keys first part on, to the old word's"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 176.
        pin_eq!(w.firsts.len(), 124, "every key parted on run426");
    }

    /// **The third map's long word, 15982, widened whole** (item 1177):
    /// [`sahara_15982_window`] over run428's blocks 15977..16233. The
    /// word's frame writes block 15983.
    #[test]
    fn run428_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_15982_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run428 whole: blocks 15977..16233");
        pin!(
            w.missing.is_empty(),
            "run428 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 15977 stands on 144 keys**, the human leader's census
        // rows and the army group's ids among them. Item 1189 took seven:
        // who=1's wealth — `bucket` 96 against 103, `income` and
        // `resources` 992 against 1056, `rate` 62 against 66, `leftover`
        // — and both Villages' `trade_val`, 176 against 208, the Spice
        // route run418's block 14661 parts on first.
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]"),
            None,
            "who=1's wealth agrees"
        );
        // **15978: the human's Farm `0/2004`**, under who=1's archers,
        // took a javelin's damage a block early in the original, 130
        // against 134: every Javelineers round left the unit's own square
        // here until item 1194 gave piece 33 its bays, and flew one frame
        // long. It agrees now, on every block of the window.
        pin_eq!(
            row(0, 2004, "build:damage"),
            None,
            "the Farm's damage, since item 1194"
        );
        // **15982: who=1's make list**, an offer's value and its city.
        pin_eq!(
            row(1, -1, "leader:MAKE[2].val").as_deref(),
            Some("15982: ours 59500 theirs 51000"),
            "a make offer's value"
        );
        // **The old word's block, 15983** (item 1189, the move's value
        // diff): frame 15982 spent ours 17 draws against 16, ours
        // `Leader::use_market+0x1ed` first. `use_market`'s need is the
        // first `epoch[Commerce]` = 2 slots, two Senates at 50 wealth
        // each, and ours held 96 against the original's 104, so ours sold
        // and drew. With the Spice route the wealth agrees, the market
        // draws on neither side, and who=1's new site `1/2024` (`x` 41280
        // against 41088 until 1189), its Senate purchase (`MAKE[0].t` −1
        // against 438) and `1/61`'s group move all agree.
        pin_eq!(
            row(1, 2024, "build:x_internal"),
            None,
            "the old word's block: 1/2024's site"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[0].t"),
            None,
            "the old word's block: the Senate's slot"
        );
        pin_eq!(row(1, 61, "pos"), None, "the old word's block: 1/61");
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(15977, 144), (15982, 9)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_15982_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(15977, 99), (15982, 2)],
            "the blocks keys first part on, to the old word's"
        );
        // What the window still parts on past it: the human's
        // `production_step` on 16001, the army group's id on 16123 (stamped
        // six frames apart again) and a make offer's value on 16182, 56700
        // against 48600 — the 15982 pair's ratio.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 164.
        pin_eq!(w.firsts.len(), 112, "every key parted on run428");
    }

    /// **The third map's long word, 16681, widened whole** (item 1189):
    /// [`sahara_16681_window`] over run442's blocks 16676..16932. The
    /// word's frame writes block 16682.
    #[test]
    fn run442_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_16681_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run442 whole: blocks 16676..16932");
        pin!(
            w.missing.is_empty(),
            "run442 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 16676 stands on 163 keys**: run428's standing families —
        // the human leader's census rows, the army group's ids (16634146
        // here against 16640446 there, stamped six frames apart again) —
        // and who=1's make list, whose `city` is this crate's global index
        // against the original's per-leader one and whose Mine offer
        // stands 56700 against 48600.
        pin_eq!(
            row(1, 52, "order:group.id").as_deref(),
            Some(r#"16676: Group { field: "id", ours: 16634146, theirs: 16640446 }"#),
            "the army group's id"
        );
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]"),
            None,
            "who=1's wealth agrees, since item 1189"
        );
        // **16681: the human's Farm `0/2004`**, at `damage` 397 of 400 on
        // both sides through block 16680, was gone in the original and
        // held here until item 1194 (its chain's `city_down`, the human's
        // roads and who=1's food, 146 against 166, parted with it). The
        // Javelineers `1/67`'s round launched on tick 16664 left the
        // unit's own square here, (6120, 28248), and the release node
        // there, (6028, 28300) (run448's `AMMO`): `total_time` 18 against
        // 17, and the Farm fell a tick late. Piece 33's bays close it.
        pin_eq!(
            row(0, 2004, "build:extra"),
            None,
            "the Farm falls on the original's tick, since item 1194"
        );
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]").as_deref(),
            Some("16783: ours 118 theirs 125"),
            "who=1's food, beside `1/2024`'s queue price 70 against 63"
        );
        // **The old word's block, 16682** (item 1194, the move's value
        // diff): frame 16681 spent ours 11 draws against 8, `1/30`'s
        // release and its round's scatter pair on the Farm the original
        // had lost. The archer `1/28` held `order:kind` 10 against 19;
        // it agrees now.
        pin_eq!(
            row(1, 28, "order:kind"),
            None,
            "the old word's block: 1/28's order"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(16676, 163)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_16681_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(16676, 111)],
            "the blocks keys first part on, to the old word's"
        );
        // What the window still parts on past it: a make offer's value on
        // 16782 (the 15982 pair's 7/6), who=1's food and `1/2024`'s queue
        // price on 16783, the human's `free_peasants` on 16801, and the
        // army group's id on 16891, stamped six frames apart again.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 177.
        pin_eq!(w.firsts.len(), 125, "every key parted on run442");
    }

    /// **The third map's long word, 17623, widened whole** (item 1194):
    /// [`sahara_17623_window`] over run449's blocks 17618..17874. The
    /// word's frame writes block 17624.
    #[test]
    fn run449_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_17623_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run449 whole: blocks 17618..17874");
        pin!(
            w.missing.is_empty(),
            "run449 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 17618 stands on 151 keys**: run442's families (the
        // human's census rows, who=1's make list and food, the army groups'
        // ids and roles). Until item 1206 it stood on 189, with four of
        // who=1's soldiers' positions among them, `1/46`, `1/48`, `1/51` and
        // `1/52`, parted in the gap after run442's last block — run457's
        // 17493, `refresh_group_order`'s tail. **The move's value diff**:
        // `1/52`'s `pos` (28322,20471) against (28344,20484) → agreeing.
        pin_eq!(row(1, 52, "pos"), None, "1/52 stands in its place");
        // **The old word's block, 17624** (frame 17623, ours 7 draws
        // against 8, at index 2 ours `Guy::set_anim+0x97a <
        // Guy::inc_time+0x271` and theirs `Guy::set_anim+0x97a <
        // Unit::do_idle+0x7d`): `1/52` went idle there and walked on here →
        // it idles on both sides, 8 draws against 8.
        pin_eq!(row(1, 52, "idle"), None, "the old word's block: 1/52 idles");
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was [(17618, 151)].
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= SAHARA_17623_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(17618, 101)],
            "the blocks keys first part on, to the old word's"
        );
        // **Past it, the standing families and one row they reach**:
        // who=1's `bucket[2:wealth]` 119 against 194 on 17683. `1/0` takes
        // a goody box on 17682 on both sides, four lottery draws agreeing
        // (`Unit::explore_goody+0x27c`), and the lottery scores `draw % 25
        // + bucket` per good: ours' food is the standing 109 against 136,
        // so it pays food where the original pays wealth. Before item 1206
        // the draws had parted at 17623 and the pick fell to wealth by a
        // desynced stream. The SITE rows on 17776 and the human's
        // `production_step` on 17801 are run442's families.
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]").as_deref(),
            Some("17683: ours 119 theirs 194"),
            "the goody pays food where the original pays wealth"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 157.
        pin_eq!(w.firsts.len(), 107, "every key parted on run449");
    }

    /// **The gap before the third map's word, widened** (item 1206):
    /// [`sahara_gap_window`] over run457's blocks 17140..17618.
    #[test]
    fn run457_s_gap_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_gap_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 479, "run457 whole: blocks 17140..17618");
        pin!(
            w.missing.is_empty(),
            "run457 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 17140 stands on 154 keys**: run442's families — the
        // census, who=1's make list and food (`1/2024`'s
        // `queue[0].cost[0]` 70 against 63), the army groups' ids, the
        // `form` rows. What parts after it is the same families' stamps:
        // the group-move ids on 17147 and 17403, six frames apart as on
        // every window since run442, a SITE row and `production_step`.
        //
        // **The move's value diff, 17493** (item 1206): `1/51` leaves army
        // group 64 and `1/62` takes the formation over standing.
        // `refresh_group_order`'s tail is `update_positions` whole, so the
        // original turns the table by `find_angle` to `1/62`'s waypoint,
        // (−7, −6) off it, and ours turned it by `1/62`'s heading, 2.5°
        // away: every one of the fifteen `curr` slots parted there, and the
        // window's 283 keys fall to 172 when none does.
        pin_eq!(
            row(1, -3, "group:64.curr[8]"),
            None,
            "group 64's slot 8 on 17493"
        );
        pin_eq!(
            row(1, 52, "pos"),
            None,
            "1/52 walks where the original's does"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            [(17140, 103), (17147, 9), (17176, 1), (17201, 1), (17403, 6)],
            "the blocks keys first part on"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 172.
        pin_eq!(w.firsts.len(), 120, "every key parted on run457");
    }

    /// **The third map's word at its end, widened whole** (item 1206):
    /// [`sahara_end_window`] over run458's blocks 23744..24000. The word
    /// 24000 is run383's trace's last frame; its block, 24001, is run383's
    /// closing dump, which the endpoint scores (0 off).
    #[test]
    fn run458_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_end_window() else {
            return;
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 257, "run458 whole: blocks 23744..24000");
        pin!(
            w.missing.is_empty(),
            "run458 carries every key: {:?}",
            w.missing
        );
        // **Block 23744 stands on 154 keys**, run442's families: the
        // census, who=1's make list and food, the army groups' ids and
        // roles, the SITE and `form` rows. Past it, the same families
        // only — the human's `production_step` on 23801, who=1's
        // `MAKE[1].city` on 23981 and its `gather_stamp` (23983 against
        // 23879) on 23984 — and **no unit's position or order parts** over
        // the game's last 257 blocks.
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index.
        pin_eq!(
            by.into_iter().collect::<Vec<_>>(),
            [(23744, 104), (23801, 1), (23984, 1)],
            "the blocks keys first part on"
        );
        // Item 1326 re-pinned, the make list's `city` compared as the leader's own index: was 157.
        pin_eq!(w.firsts.len(), 106, "every key parted on run458");
    }

    /// **The third map's score** (item 1066): run382 walked from run381's
    /// head. Every figure is compared against `FLOORS`' Great Sahara row.
    #[test]
    fn run382_s_desert_game_is_the_third_map_s_score() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump(SAHARA_SCORE.0),
            dump(SAHARA_START),
            trace(SAHARA_SCORE.1),
        ) else {
            eprintln!("skipping: no Great Sahara capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run381 is a start dump");
        // **The map stands up on run381 alone**: its own heights, cells,
        // regions, herds and goods, and the lobby both dumps print is one.
        assert!(
            !sib_init.heights.is_empty() && !sib_init.herds.is_empty(),
            "run381 carries the height table and the herds"
        );
        assert!(
            same_lobby(&log.initial().unwrap(), &sib_init),
            "run381 and run382 print one GAMEINFO"
        );
        let report = run_traced(
            &loaded,
            &log,
            Tuning::RON,
            None,
            None,
            &[&sib_init],
            Some(&tr),
        )
        .unwrap();
        assert_eq!(report.frames.len(), 1851, "run382's length");
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: frame 0: ours 99 draws, the original's 99")),
            "frame 0 on the third map: {:?}",
            report.notes
        );
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        eprintln!(
            "run382: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
        let f = sahara_floors();
        assert!(
            ticks >= f.ticks && orders >= f.orders,
            "the third map's score fell: ticks {ticks}, orders {orders} — the floor is \
             ticks {}, orders {}",
            f.ticks,
            f.orders
        );
    }

    /// **The third map's word on the score capture** (item 1066): run382's
    /// trace, frame for frame, against this crate's draws.
    #[test]
    fn run382_s_trace_says_where_the_third_map_s_word_parts() {
        let Some(w) = walk_sahara(SAHARA_SCORE) else {
            return;
        };
        let f = sahara_floors();
        assert!(
            w.count >= f.word && w.sequence >= f.word,
            "the third map's word fell: count {}, sequence {} — the floor is {}; {}",
            w.count,
            w.sequence,
            f.word,
            w.row
        );
    }

    /// **The third map's long word** (item 1066): run383, the draw stream
    /// to 24,000 at `cover=0`, whose first 1,851 frames are run382's word
    /// for word (its stanza's `rngcmp.py` check).
    #[test]
    fn run383_s_long_trace_says_where_the_third_map_s_word_parts() {
        let Some(w) = walk_sahara(SAHARA_LONG) else {
            return;
        };
        assert_eq!(
            w.last,
            ai_word_length("GreatSahara"),
            "run383's trace runs to the length `AI_WORDS` gives the game"
        );
        assert!(
            w.count >= LONG_WORD_GREAT_SAHARA && w.sequence >= LONG_WORD_GREAT_SAHARA,
            "the third map's long word fell: count {}, sequence {} — the floor is {}; {}",
            w.count,
            w.sequence,
            LONG_WORD_GREAT_SAHARA,
            w.row
        );
    }
}
