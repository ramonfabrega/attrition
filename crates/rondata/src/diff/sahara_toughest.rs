//! **The closed map in the newest pair's lobby** (DECISIONS 56 §1, item
//! 1221): Great Sahara, `MAP_STYLE 7`, seed 12345, at the second pair's
//! difficulty, Toughest — run382's and run383's game with the one setting
//! the first pair's lobby held fixed moved. Three captures: run468, a
//! `DUMP_ALL` start in run381's shape at `DIFFICULTY 5`, which is the
//! sibling everything here stands up from; run469, 1,850 blocks in run33's
//! shape; and run470, the draw stream to 24,000 or the game's end at
//! `cover=0`. The word is [`THIRD_WORD_GREAT_SAHARA_TOUGHEST`], the
//! `AI_WORDS` row `GreatSaharaToughest` on the `Third map` line. No
//! mechanism is named here.

use super::*;

use crate::diff::testkit::*;
use crate::diff::third::walk_sahara_from;
use crate::testenv::{dump, install};

/// run468: the `DUMP_ALL` start at Toughest — the height table, the cells,
/// the checksum trace, the herds and the frame seeds.
pub(crate) const TOUGHEST_START: &str = "gamelog-run468-greatsahara-toughest-start.txt";
/// run469: 1,850 blocks at run10's detail, run382's shape at Toughest.
pub(crate) const TOUGHEST_SCORE: (&str, &str) = (
    "gamelog-run469-greatsahara-toughest-longtrace.txt",
    "rontrace-run469.log",
);
/// run470: the draw stream at `cover=0`, run383's shape at Toughest.
pub(crate) const TOUGHEST_LONG: (&str, &str) = (
    "gamelog-run470-greatsahara-toughest-24k-trace.txt",
    "rontrace-run470.log",
);

/// run471: run470's game at run449's detail over blocks 5371..5627, the
/// first word 5376's widening (item 1221) — six blocks before the word's
/// block 5377 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_5376: &str = "gamelog-run471-greatsahara-toughest-5376.txt";

/// **run471's window** (item 1221): blocks 5371..5627.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST: (i64, i64) = (5_371, 5_627);

/// **The first word's block, 5377**: frame 5376 writes it. The word has
/// moved on (item 1241); run471 keeps its value diff, the Granary's place.
pub(crate) const TOUGHEST_WORD_BLOCK: i64 = 5_377;

/// run476: run470's game at run471's detail over blocks 5777..6033, the
/// word 5782's widening (item 1241) — six blocks before the word's block
/// 5783 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_5782: &str = "gamelog-run476-greatsahara-toughest-5782.txt";

/// **run476's window** (item 1241): blocks 5777..6033.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_5782: (i64, i64) = (5_777, 6_033);

/// **The word's block, 5783**: frame 5782 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_5783: i64 = 5_783;

/// run471's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST`], walked from
/// run470's start with run468's head, as `third::sahara_17623_window` walks
/// run449: every record run449's detail prints, the group record and the
/// attack row's pass included. `None` when the captures are not on this
/// machine.
pub(crate) fn great_sahara_toughest_5376_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run471",
        (TOUGHEST_WORD_5376, WIDENING_GREAT_SAHARA_TOUGHEST.0),
        WIDENING_GREAT_SAHARA_TOUGHEST,
        TOUGHEST_WORD_BLOCK,
    )
}

/// run476's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_5782`], walked as
/// run471's are (item 1241).
pub(crate) fn great_sahara_toughest_5782_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run476",
        (TOUGHEST_WORD_5782, WIDENING_GREAT_SAHARA_TOUGHEST_5782.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_5782,
        TOUGHEST_WORD_BLOCK_5783,
    )
}

/// **The compared pin's window on the third map at Toughest** (item 1221):
/// the open word [`THIRD_WORD_GREAT_SAHARA_TOUGHEST`]'s block and two on
/// either side, as `second::east_indies_word_window` takes East Indies',
/// on run476 since item 1241 (run471 before it). `coverage`'s compared pin
/// walks these blocks with the recorder on.
pub(crate) fn great_sahara_toughest_word_window() -> Option<crate::diff::harness::tests::Widened> {
    // Frame `f` writes block `f + 1`, and the walk reads `first..=tail`.
    let word = THIRD_WORD_GREAT_SAHARA_TOUGHEST;
    toughest_window_over(
        "run476",
        (TOUGHEST_WORD_5782, WIDENING_GREAT_SAHARA_TOUGHEST_5782.0),
        (word - 1, word + 2),
        TOUGHEST_WORD_BLOCK_5783,
    )
}

fn toughest_window_over(
    run: &str,
    capture: (&str, i64),
    window: (i64, i64),
    word_block: i64,
) -> Option<crate::diff::harness::tests::Widened> {
    crate::diff::harness::tests::widen_on_siblings(
        &[TOUGHEST_START],
        true,
        TOUGHEST_LONG,
        run,
        &[capture],
        window,
        1,
        &[word_block],
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **run468 and run469 are Great Sahara at Toughest**: one `GAME INFO`
    /// between them, and it is run381's but for `DIFFICULTY 5` against 0 —
    /// the one setting moved (DECISIONS 53 §2, 56 §1).
    #[test]
    fn run469_is_great_sahara_at_toughest_and_nothing_else_moved() {
        let (Some(start), Some(score), Some(first)) = (
            dump(TOUGHEST_START),
            dump(TOUGHEST_SCORE.0),
            dump(crate::diff::third::SAHARA_START),
        ) else {
            eprintln!("skipping: no run468/run469/run381 (set RON_GAMELOG_DIR)");
            return;
        };
        let (start_text, score_text, first_text) = (
            crate::capture::read(&start),
            crate::capture::read(&score),
            crate::capture::read(&first),
        );
        let (start_log, score_log, first_log) = (
            Log::parse(&start_text),
            Log::parse(&score_text),
            Log::parse(&first_text),
        );
        let start = start_log.initial().expect("run468 is a start dump");
        let score = score_log.initial().expect("run469's head");
        let first = first_log.initial().expect("run381 is a start dump");
        assert!(
            !start.heights.is_empty() && !start.herds.is_empty(),
            "run468 carries the height table and the herds"
        );
        assert!(
            !start.game_info.is_empty() && start.game_info == score.game_info,
            "run468 and run469 print one GAME INFO"
        );
        assert_eq!(start.game_info.len(), first.game_info.len());
        let apart: Vec<_> = start
            .game_info
            .iter()
            .zip(first.game_info.iter())
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(
            apart,
            [(&("DIFFICULTY", "5"), &("DIFFICULTY", "0"))],
            "run468's GAME INFO against run381's"
        );
    }

    /// **The closed map's score at Toughest** (item 1221): run469 walked
    /// from its own head with run468's borrowed. Measured, and held where
    /// it stands: 1850 and 1850, run382's to the frame — the one unit that
    /// parts is the human citizen `0/5` on the shutdown block 1851.
    #[test]
    fn run469_s_score_holds() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump(TOUGHEST_SCORE.0),
            dump(TOUGHEST_START),
            trace(TOUGHEST_SCORE.1),
        ) else {
            eprintln!("skipping: no run469/run468 (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run468 is a start dump");
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
        assert_eq!(report.frames.len(), 1851, "run469's length");
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: frame 0: ours 100 draws, the original's 100")),
            "frame 0 at Toughest: {:?}",
            report.notes
        );
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        assert!(
            ticks >= 1850 && orders >= 1850,
            "run469's score fell: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
    }

    /// **The word on the 1,850-frame capture**: run469's trace, frame for
    /// frame, against this crate's draws, walked from run468's start.
    #[test]
    fn run469_s_trace_holds_to_its_end() {
        let Some(w) = walk_sahara_from(TOUGHEST_START, TOUGHEST_SCORE) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run469's GAME INFO reads DIFFICULTY 5");
        assert!(
            w.count >= w.last && w.sequence >= w.last,
            "run469's draws part: count {}, sequence {} of {}; {}",
            w.count,
            w.sequence,
            w.last,
            w.row
        );
    }

    /// **The third map's first word at Toughest, 5376, widened whole** (item
    /// 1221): [`great_sahara_toughest_5376_window`] over run471's blocks
    /// 5371..5627, both directions, every record run449's detail prints.
    /// The word's frame writes block 5377. Item 1241 moved the word past
    /// it: `find_friends`' enhancer arm puts the Granary where the
    /// original does (`docs/AI.md` §99.7).
    #[test]
    fn run471_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_5376_window() else {
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
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run471 whole: blocks 5371..5627");
        pin!(
            w.missing.is_empty(),
            "run471 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 5371 stands on 86 keys**, the families the first pair's
        // widenings carry: `form` on every citizen and soldier, the human's
        // census and city rows (this crate fills neither for player 0),
        // both leaders' `SITE[i].reg` (ours 1, theirs 0), who=1's army
        // groups' `role` and pool lists, and three rows of who=1's own that
        // this crate reads: `bucket[0:food]` 381 against 417, `scholars` 0
        // against 1 and the `tech_cat_frame`s.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]").as_deref(),
            Some("5371: ours 381 theirs 417"),
            "who=1's food bucket stands"
        );
        // **Frame 5375's block adds who=1's `SITE[2].reg`**, and the word's
        // block 5377 its 42: the value diff below.
        pin_eq!(
            row(1, -1, "leader:SITE[2].reg").as_deref(),
            Some("5376: ours 1 theirs 0"),
            "who=1's third site's region"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(5371, 86), (5376, 1)],
            "the blocks keys first part on, to the first word's"
        );
        // **The first word's value diff, block 5377, closed by item 1241**:
        // who=1's Granary `1/2023` stood at ours (40608, 19680) against the
        // original's (41184, 15072), and the two citizens sent to it parted
        // with it (`1/12`'s order kind 3 against 7, `1/8`'s 1 against 3).
        // With the enhancer arm the two farms north of the city count as
        // its friends at cell (53, 19), and all four rows agree.
        for (o, what) in [
            (2023, "build:x_internal"),
            (2023, "build:y_internal"),
            (12, "order:kind"),
            (8, "order:kind"),
        ] {
            pin_eq!(row(1, o, what), None, "1/{o}'s {what} agrees on 5377");
        }
        // **The make list parts on 5380**, three blocks past the Granary:
        // who=1's `MAKE[2]` and `MAKE[3]` swap (ours `t` 553 and 567,
        // theirs 567 and 553), and `MAKE[9].val` reads 4800 against 48000.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val").as_deref(),
            Some("5380: ours 4800 theirs 48000"),
            "who=1's tenth make row"
        );
        pin_eq!(w.firsts.len(), 212, "every key parted on run471");
    }

    /// **The third map's word at Toughest, 5782, widened whole** (item
    /// 1241): [`great_sahara_toughest_5782_window`] over run476's blocks
    /// 5777..6033, both directions, every record run471's detail prints.
    /// The word's frame writes block 5783. No mechanism is named.
    #[test]
    fn run476_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_5782_window() else {
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
            eprintln!("  by {by:?}");
        }
        pin_eq!(w.blocks, 257, "run476 whole: blocks 5777..6033");
        pin!(
            w.missing.is_empty(),
            "run476 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 5777 stands on 103 keys**: run471's families — `form`,
        // the human's census rows, both leaders' `SITE[i].reg` (ours 1,
        // theirs 0) — and who=1's food bucket, 90 against 126.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]").as_deref(),
            Some("5777: ours 90 theirs 126"),
            "who=1's food bucket stands"
        );
        // **The make list parts on 5779**: `MAKE[2].val` and `MAKE[9].val`
        // read 1200 against 4800, and on 5781 six rows' `city` part.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val").as_deref(),
            Some("5779: ours 1200 theirs 4800"),
            "who=1's tenth make row"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_5783)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(5777, 103), (5779, 2), (5781, 7), (5783, 5)],
            "the blocks keys first part on, to the word's"
        );
        // **The word's value diff, block 5783**: on the frame ours spends a
        // `Leader::make_stuff+0x63d` the original does not, who=1 queues one
        // more at `1/2017` (its Barracks) — `queued` 2 against 1,
        // `num_queued[82]` 1 against 0 — and pays for it: wealth 105
        // against 158, metal 7 against 43.
        pin_eq!(
            row(1, 2017, "queue:queued").as_deref(),
            Some("5783: ours 2 theirs 1"),
            "1/2017's queue"
        );
        pin_eq!(
            row(1, -1, "leader:num_queued[82]").as_deref(),
            Some("5783: ours 1 theirs 0"),
            "who=1's queued count of type 82"
        );
        pin_eq!(
            row(1, -1, "leader:bucket[4:metal]").as_deref(),
            Some("5783: ours 7 theirs 43"),
            "who=1's metal"
        );
        pin_eq!(w.firsts.len(), 1_296, "every key parted on run476");
    }

    /// **The third map's word at Toughest** (item 1221): run470, the draw
    /// stream at `cover=0` to the game's end, whose first 1,851 frames are
    /// run469's word for word (its stanza's `rngcmp.py` check), walked from
    /// run468's start.
    #[test]
    fn run470_is_great_sahara_at_toughest_and_its_word_holds() {
        let Some(w) = walk_sahara_from(TOUGHEST_START, TOUGHEST_LONG) else {
            return;
        };
        assert_eq!(w.difficulty, 5, "run470's GAME INFO reads DIFFICULTY 5");
        assert_eq!(
            w.last,
            ai_word_length("GreatSaharaToughest"),
            "run470's trace runs to the length `AI_WORDS` gives the game"
        );
        assert!(
            w.count >= THIRD_WORD_GREAT_SAHARA_TOUGHEST
                && w.sequence >= THIRD_WORD_GREAT_SAHARA_TOUGHEST,
            "the third map's word at Toughest fell: count {}, sequence {} of {} — the \
             floor is {THIRD_WORD_GREAT_SAHARA_TOUGHEST}; {}",
            w.count,
            w.sequence,
            w.last,
            w.row
        );
    }
}
