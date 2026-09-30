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
}
