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
        &[LONG_WORD_GREAT_SAHARA + 1],
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
        use std::collections::BTreeMap;
        let Some(w) = sahara_word_window() else {
            return;
        };
        assert_eq!(w.blocks, 259, "run382 over blocks 1..259");
        let first = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| (*f, r.clone()))
        };
        let mut by: BTreeMap<i64, usize> = BTreeMap::new();
        for (f, _) in w.firsts.values() {
            *by.entry(*f).or_default() += 1;
        }
        // **What run10's detail does not print** is a key unprinted, never
        // a parting: the leader's long record (`LEADERS=1` prints the short
        // one) and gaia's clocks (`GUYS=2` prints no `cur_anim`).
        assert_eq!(w.missing.len(), 1_060, "the keys run382 does not print");
        assert!(
            w.missing.contains("gaia:cur_anim") && w.missing.contains("resources[0:food]"),
            "gaia's clocks and the long leader record are unprinted"
        );
        // **Block 1 stands** with frame 0's draws agreeing (99 against 99):
        // 22 keys, all the first pair's standing families (`docs/AI.md`
        // §33.4) — `form` on the ten citizens and the scout, and the human
        // capital's city record, which this crate does not fill for player
        // 0. None is who=1's economy.
        assert!(
            w.firsts.iter().all(|((who, o, what), (f, _))| *f > 1
                || what == "form"
                || (*who, *o) == (0, 2000) && what.starts_with("city:")),
            "block 1's rows are the standing families"
        );
        // **The first parting past block 1 is block 6, one unit**: the AI
        // citizen `1/2` on its way from the capital. Ours marks a collision
        // (`collide` 1 against 0) and walks a one-leg move to (38232,
        // 16056) where the original holds a two-leg move to (38328, 15768).
        for (what, want) in [
            ("collide", "ours 1 theirs 0"),
            (
                "order:move.dest_x",
                "Move { field: \"dest_x\", ours: 38232, theirs: 38328 }",
            ),
            (
                "order:move.dest_y",
                "Move { field: \"dest_y\", ours: 16056, theirs: 15768 }",
            ),
            ("path:length", "PathLength { ours: 1, theirs: 2 }"),
        ] {
            assert_eq!(
                first(1, 2, what),
                Some((6, want.to_string())),
                "1/2's {what}, the first parting"
            );
        }
        // Its position parts on block 7, (38352, 15814) against (38328,
        // 15768), and on block 8 the two destinations have swapped sides:
        // the original takes the side-step two frames after ours.
        assert_eq!(
            first(1, 2, "pos"),
            Some((7, "ours (38352,15814) theirs (38328,15768)".to_string())),
            "1/2's position"
        );
        // **The word's block, 9**: `1/2` stands at (38328, 15768) on both
        // sides again, and what parts is its collision's age and its facing
        // — `collide_frame` 6 against 8, `collide` 2 against 1, facing
        // −328728576 against −1925840896. Its partner is `1/1` on both
        // sides (ours from block 8, the original's from block 10). The
        // original spends `Guy::set_anim+0x97a < Unit::move_step+0x823`
        // first on frame 8 and ours `Farms::inc_time+0x1ae`.
        assert_eq!(
            first(1, 2, "collide_frame"),
            Some((9, "ours 6 theirs 8".to_string())),
            "the word's block: the collision's frame"
        );
        assert_eq!(
            first(1, 2, "angle:Facing"),
            Some((9, "ours -328728576 theirs -1925840896".to_string())),
            "the word's block: the facing"
        );
        assert_eq!(
            by.iter().take(5).map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(1, 22), (6, 4), (7, 8), (8, 2), (9, 3)],
            "the blocks keys first part on, the first five"
        );
        // Gaia's herds hold until block 31 (`8/2`'s position).
        assert!(
            w.firsts
                .iter()
                .filter(|((who, _, _), _)| *who >= 8)
                .all(|(_, (f, _))| *f >= 31),
            "gaia's animals agree to block 30"
        );
        assert_eq!(w.firsts.len(), 244, "keys parted over the window");
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
        assert_eq!(w.last, 24_000, "run383's trace runs to 24,000");
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
