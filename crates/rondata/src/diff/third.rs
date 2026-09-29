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
}

/// A Great Sahara capture walked from its own start dump with run381's head
/// borrowed, until the draw stream parts (or the trace ends). `None` when
/// the captures are not on this machine.
pub(crate) fn walk_sahara((gamelog, tracelog): (&str, &str)) -> Option<Word> {
    let inst = install()?;
    let (Some(path), Some(sib), Some(tr)) = (dump(gamelog), dump(SAHARA_START), trace(tracelog))
    else {
        eprintln!("skipping: no {gamelog}/{SAHARA_START} (set RON_GAMELOG_DIR)");
        return None;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let sib_text = crate::capture::read(&sib);
    let log = Log::parse(&text);
    let sib_log = Log::parse(&sib_text);
    let sib_init = sib_log.initial().expect("run381 is a start dump");
    let mut init = log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sib_init]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    built.sim.trace_phases = true;
    let last = tr.frames.last().map_or(0, |(n, _)| *n);
    let (mut count, mut sequence) = (None, None);
    for _ in 0..last {
        built.tick();
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
pub(crate) fn sahara_word_window() -> Option<crate::diff::harness::tests::Widened> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// **The third map's word, widened whole** (item 1066, CLAUDE.md "A
    /// word is pinned with its widening"): [`sahara_word_window`] over
    /// run382's blocks 1..259, both directions, every record run10's detail
    /// prints. The word's frame 8 writes block 9. No mechanism is named
    /// (DECISIONS 54 §3); the rows below are the value diff.
    #[test]
    fn run382_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = sahara_word_window() else {
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
        pin_eq!(w.missing.len(), 1_060, "the keys run382 does not print");
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
        // **The first parting past block 1 is now block 202, one key**: the
        // AI scout `1/0`'s `mylos`, ours 6 against 4. It moves no draw and
        // no order over run382's 1,850 frames.
        pin_eq!(
            first(1, 0, "mylos"),
            Some((202, "ours 6 theirs 4".to_string())),
            "the scout's line of sight, the first parting past block 1"
        );
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 22), (202, 1)],
            "the blocks keys first part on"
        );
        // Gaia's herds hold over the whole window.
        pin!(
            w.firsts.iter().all(|((who, _, _), _)| *who < 8),
            "gaia's animals agree to block 259"
        );
        pin_eq!(w.firsts.len(), 23, "keys parted over the window");
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
