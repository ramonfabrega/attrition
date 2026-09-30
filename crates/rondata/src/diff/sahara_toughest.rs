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

/// run483: run470's game at run476's detail over blocks 7065..7321, the
/// word 7070's widening (item 1251) — six blocks before the word's block
/// 7071 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_7070: &str = "gamelog-run483-greatsahara-toughest-7070.txt";

/// **run483's window** (item 1251): blocks 7065..7321.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_7070: (i64, i64) = (7_065, 7_321);

/// **The word's block, 7071**: frame 7070 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_7071: i64 = 7_071;

/// run488: run470's game at run483's detail over blocks 7780..8036, the
/// word 7785's widening (item 1260) — six blocks before the word's block
/// 7786 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_7785: &str = "gamelog-run488-greatsahara-toughest-7785.txt";

/// **run488's window** (item 1260): blocks 7780..8036.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_7785: (i64, i64) = (7_780, 8_036);

/// **The word's block, 7786**: frame 7785 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_7786: i64 = 7_786;

/// run491: run470's game at run488's detail over blocks 8177..8433, the
/// word 8182's widening (item 1264) — six blocks before the word's block
/// 8183 and 250 after, on the click-free lane.
pub(crate) const TOUGHEST_WORD_8182: &str = "gamelog-run491-greatsahara-toughest-8182.txt";

/// **run491's window** (item 1264): blocks 8177..8433.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_8182: (i64, i64) = (8_177, 8_433);

/// **The word's block, 8183**: frame 8182 writes it.
pub(crate) const TOUGHEST_WORD_BLOCK_8183: i64 = 8_183;

/// **The road search 7070 was** (item 1260): caravan `1/52`'s replan from
/// `1/2007` to `1/2022`, which run483's trace carries node for node.
pub(crate) const TOUGHEST_ROAD_FRAME_7070: i64 = 7_070;

/// run482: run470's game at `end:MISC,LEADERS=2` over blocks 1..5378 —
/// every leader's goods record, `LeaderDataEncrypt::log_data` and the
/// gather-slot arrays beside it, on every frame the dark gap between run469
/// and run471 left unprinted (item 1251).
pub(crate) const TOUGHEST_GOODS: &str = "gamelog-run482-greatsahara-toughest-goods.txt";

/// **run482's window** (item 1251): blocks 1..5378.
pub(crate) const WIDENING_GREAT_SAHARA_TOUGHEST_GOODS: (i64, i64) = (1, 5_378);

/// **The block the goods first parted on, 4577**: frame 4576, the script
/// step that researches who=1's Empire (item 1251).
pub(crate) const TOUGHEST_GOODS_BLOCK_4577: i64 = 4_577;

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

/// run483's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_7070`], walked as
/// run476's are (item 1251).
pub(crate) fn great_sahara_toughest_7070_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run483",
        (TOUGHEST_WORD_7070, WIDENING_GREAT_SAHARA_TOUGHEST_7070.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_7070,
        TOUGHEST_WORD_BLOCK_7071,
    )
}

/// run488's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_7785`], walked as
/// run483's are (item 1260).
pub(crate) fn great_sahara_toughest_7785_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run488",
        (TOUGHEST_WORD_7785, WIDENING_GREAT_SAHARA_TOUGHEST_7785.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_7785,
        TOUGHEST_WORD_BLOCK_7786,
    )
}

/// run482's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_GOODS`], walked as
/// run471's are (item 1251): the goods of every leader from block 1.
pub(crate) fn great_sahara_toughest_goods_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run482",
        (TOUGHEST_GOODS, WIDENING_GREAT_SAHARA_TOUGHEST_GOODS.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_GOODS,
        TOUGHEST_GOODS_BLOCK_4577,
    )
}

/// run491's blocks over [`WIDENING_GREAT_SAHARA_TOUGHEST_8182`], walked as
/// run488's are (item 1264).
pub(crate) fn great_sahara_toughest_8182_window() -> Option<crate::diff::harness::tests::Widened> {
    toughest_window_over(
        "run491",
        (TOUGHEST_WORD_8182, WIDENING_GREAT_SAHARA_TOUGHEST_8182.0),
        WIDENING_GREAT_SAHARA_TOUGHEST_8182,
        TOUGHEST_WORD_BLOCK_8183,
    )
}

/// **The compared pin's window on the third map at Toughest** (item 1221):
/// the open word [`THIRD_WORD_GREAT_SAHARA_TOUGHEST`]'s block and two on
/// either side, as `second::east_indies_word_window` takes East Indies',
/// on run491 since item 1264 (run488 from item 1260, run483 from item
/// 1251, run476 from item 1241, run471 before it). `coverage`'s compared pin walks these blocks with the
/// recorder on.
pub(crate) fn great_sahara_toughest_word_window() -> Option<crate::diff::harness::tests::Widened> {
    // Frame `f` writes block `f + 1`, and the walk reads `first..=tail`.
    let word = THIRD_WORD_GREAT_SAHARA_TOUGHEST;
    toughest_window_over(
        "run491",
        (TOUGHEST_WORD_8182, WIDENING_GREAT_SAHARA_TOUGHEST_8182.0),
        (word - 1, word + 2),
        TOUGHEST_WORD_BLOCK_8183,
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
        // **Block 5371 stands on 85 keys**, the families the first pair's
        // widenings carry: `form` on every citizen and soldier, the human's
        // census and city rows (this crate fills neither for player 0),
        // both leaders' `SITE[i].reg` (ours 1, theirs 0), who=1's army
        // groups' `role` and pool lists, and two rows of who=1's own that
        // this crate reads: `scholars` 0 against 1 and the
        // `tech_cat_frame`s. Who=1's `bucket[0:food]` stood too, 381 against
        // 417, until item 1251 priced frame 4576's Empire as the original
        // does (run482; `docs/AI.md` §99.8).
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food bucket agrees"
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
            [(5371, 85), (5376, 1)],
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
        // **The make list's values agree** since item 1251: on 5380 who=1's
        // `MAKE[2]` and `MAKE[3]` swapped and `MAKE[9].val` read 4800
        // against 48000 — Feudalism, a civic epoch, unaffordable at the
        // price without Dye's quarter. What stands of the list is its
        // `city`, ours one over the original's from 5382.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val"),
            None,
            "who=1's tenth make row agrees"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[1].city").as_deref(),
            Some("5382: ours 1 theirs 0"),
            "the make list's city stands"
        );
        pin_eq!(w.firsts.len(), 202, "every key parted on run471");
    }

    /// **The third map's word at Toughest, 5782, widened whole** (item
    /// 1241): [`great_sahara_toughest_5782_window`] over run476's blocks
    /// 5777..6033, both directions, every record run471's detail prints.
    /// The word's frame writes block 5783. Item 1251 named its cause, 36
    /// food short from frame 4576, and moved the word to 7070.
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
        // **Block 5777 stands on 102 keys**: run471's families — `form`,
        // the human's census rows, both leaders' `SITE[i].reg` (ours 1,
        // theirs 0). Who=1's food bucket stood too, 90 against 126, until
        // item 1251 priced frame 4576's Empire with Dye's quarter off
        // (`docs/AI.md` §99.8); it agrees.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food bucket agrees"
        );
        // **The make list's values agree**: `MAKE[2].val` and `MAKE[9].val`
        // read 1200 against 4800 on 5779 before item 1251 — the Feudalism
        // offer, a civic epoch the food could not reach at our price. On
        // 5781 six rows' `city` part, ours one over the original's.
        pin_eq!(
            row(1, -1, "leader:MAKE[9].val"),
            None,
            "who=1's tenth make row agrees"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_5783)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(5777, 102), (5781, 6)],
            "the blocks keys first part on, to the old word's"
        );
        // **The old word's value diff, block 5783, closed by item 1251**:
        // ours spent a `Leader::make_stuff+0x63d` the original did not,
        // queuing one more Hoplite at `1/2017` — `queued` 2 against 1,
        // `num_queued[82]` 1 against 0 — and paying for it: metal 7
        // against 43. The original's market had sold the food that would
        // have paid; with the 36 back all three agree.
        for (o, what) in [
            (2017, "queue:queued"),
            (-1, "leader:num_queued[82]"),
            (-1, "leader:bucket[4:metal]"),
        ] {
            pin_eq!(row(1, o, what), None, "1/{o}'s {what} agrees on 5783");
        }
        pin_eq!(w.firsts.len(), 147, "every key parted on run476");
    }

    /// **The third map's word at Toughest, 7070, widened whole** (item
    /// 1251): [`great_sahara_toughest_7070_window`] over run483's blocks
    /// 7065..7321, both directions, every record run476's detail prints.
    /// The word's frame writes block 7071. No mechanism is named.
    #[test]
    fn run483_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_7070_window() else {
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
        pin_eq!(w.blocks, 257, "run483 whole: blocks 7065..7321");
        pin!(
            w.missing.is_empty(),
            "run483 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 7065 stands on 129 keys**: run476's families — `form`,
        // the human's census and city rows, `SITE[i].reg`, the make list's
        // `city` one over — and three of who=1's that part in the gap past
        // run476: wealth 161 against 162, its `leftover` 1584 against 48,
        // and `MAKE[4].val` 1431372 against 1228956.
        pin_eq!(
            row(1, -1, "leader:bucket[2:wealth]").as_deref(),
            Some("7065: ours 161 theirs 162"),
            "who=1's wealth stands"
        );
        pin_eq!(
            row(1, -1, "leader:leftover[2:wealth]").as_deref(),
            Some("7065: ours 1584 theirs 48"),
            "who=1's wealth leftover stands"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[4].val").as_deref(),
            Some("7065: ours 1431372 theirs 1228956"),
            "who=1's fifth make row stands"
        );
        // **The word's block 7071 is quiet**: nothing parts between the
        // standing block and 7075. Frame 7070's extra draws are one road
        // search, `1/52`'s — ours 3204 `PathFinder::calc_road_cost+0x46`
        // against the original's 2543 — and no record prints a search.
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_7071 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(7065, 121)],
            "the blocks keys first part on, to three past the word's"
        );
        // **The word moved to 7785 on item 1260**, past this window: with
        // `leech_codes` the road beside the Farm stands and 7070's search
        // arrives as the original's does. What parts past the standing
        // block is five keys — who=1's `peasants` 30 against 31 and `1/56`'s
        // `form` on 7144 first — where it was 616.
        pin_eq!(
            by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
            [(7065, 121), (7144, 2), (7152, 1), (7184, 1), (7201, 1)],
            "the blocks keys first part on, the window whole"
        );
        pin_eq!(
            w.firsts.len(),
            126,
            "every key parted on run483 (745 before item 1260)"
        );
    }

    /// **The third map's word at Toughest, 7785, widened whole** (item
    /// 1260): [`great_sahara_toughest_7785_window`] over run488's blocks
    /// 7780..8036, both directions, every record run483's detail prints.
    /// The word's frame writes block 7786. No mechanism is named.
    #[test]
    fn run488_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_7785_window() else {
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
        pin_eq!(w.blocks, 257, "run488 whole: blocks 7780..8036");
        pin!(
            w.missing.is_empty(),
            "run488 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **Block 7780 stands on 132 keys**: run483's families, and new past
        // it who=1's `known_rares`, ours one short.
        pin_eq!(
            row(1, -1, "leader:known_rares").as_deref(),
            Some("7780: ours 3 theirs 4"),
            "who=1's known rares stand"
        );
        // **The make list parted before the word** (item 1251's 7785): on
        // 7785 its third row was a Senate (438) in category 8 here and a
        // Mine (419) in 4 there, and on 7786 both laid the Senate `1/2030`
        // apart. **Item 1264 took both** (`docs/TECH.md` step 8): who=1's
        // Tower is a Keep in the original's `num_buildings` from before
        // run483's 7065 (Tower 1, Keep 0 through run476's 6033), and this
        // crate kept a Tower. **The move's value diff:** on 7785 `MAKE[2].t`
        // ours 438 against 419 → agreeing; on 7786 `1/2030`'s
        // `x_internal` 38976 against 38784 and `y_internal` 19584 against
        // 17760 → agreeing. The word went 7785 → 8182, past this window.
        pin_eq!(
            row(1, -1, "leader:MAKE[2].t"),
            None,
            "who=1's third make row"
        );
        pin_eq!(row(1, 2030, "build:x_internal"), None, "the Senate's x");
        pin_eq!(row(1, 2030, "build:y_internal"), None, "the Senate's y");
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_7786 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [(7780, 124), (7781, 1), (7782, 4), (7784, 3), (7785, 2)],
            "the blocks keys first part on, to three past the word's"
        );
        // Item 1264: 876 → 150.
        pin_eq!(w.firsts.len(), 142, "every key parted on run488");
    }

    /// **The third map's word at Toughest, 8182, widened whole** (item
    /// 1264): [`great_sahara_toughest_8182_window`] over run491's blocks
    /// 8177..8433, both directions, every record run488's detail prints.
    /// The word's frame writes block 8183. No mechanism is named.
    #[test]
    fn run491_s_word_frame_is_widened_whole() {
        let _pins = Pins::hold();
        use std::collections::BTreeMap;
        let Some(w) = great_sahara_toughest_8182_window() else {
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
        pin_eq!(w.blocks, 257, "run491 whole: blocks 8177..8433");
        pin!(
            w.missing.is_empty(),
            "run491 carries every key: {:?}",
            w.missing
        );
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The make list parts before the word**: on 8181 who=1's fifth
        // row is a Merchant (`TypeIndex` 61, `unitrules.xml`) at 936,170
        // here and Trade (`TypeIndex` 560, `techrules.xml`) at 307,560
        // there. On the word's block 8183 who=1's food, timber and metal
        // part (94/164, 76/21, 5/57) and `1/2017` queues `TypeIndex` 132
        // where the original queues 178.
        pin_eq!(
            row(1, -1, "leader:MAKE[4].t").as_deref(),
            Some("8181: ours 61 theirs 560"),
            "who=1's fifth make row"
        );
        pin_eq!(
            row(1, -1, "leader:MAKE[4].val").as_deref(),
            Some("8181: ours 936170 theirs 307560"),
            "its offer"
        );
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]").as_deref(),
            Some("8183: ours 94 theirs 164"),
            "the word's block: who=1's food"
        );
        pin_eq!(
            by.iter()
                .filter(|(b, _)| **b <= TOUGHEST_WORD_BLOCK_8183 + 3)
                .map(|(b, n)| (*b, *n))
                .collect::<Vec<_>>(),
            [
                (8177, 128),
                (8178, 1),
                (8181, 8),
                (8183, 14),
                (8184, 14),
                (8186, 5)
            ],
            "the blocks keys first part on, to three past the word's"
        );
        pin_eq!(w.firsts.len(), 959, "every key parted on run491");
    }

    /// **The goods, every frame to the first word's block** (item 1251):
    /// [`great_sahara_toughest_goods_window`] over run482's blocks 1..5378,
    /// both directions, every field `LEADERS=2` prints for every leader.
    #[test]
    fn run482_s_goods_are_widened_whole() {
        let _pins = Pins::hold();
        let Some(w) = great_sahara_toughest_goods_window() else {
            return;
        };
        if std::env::var("RON_FIRSTS").is_ok() {
            let mut rows: Vec<_> = w.firsts.iter().collect();
            rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
            for ((who, o, what), (f, r)) in rows {
                eprintln!("  first {f} {who}/{o} {what}: {r}");
            }
            eprintln!("  missing {:?}", w.missing);
        }
        pin_eq!(w.blocks, 5_377, "run482 whole: blocks 1..5378");
        let row = |who: i64, o: i64, what: &str| {
            w.firsts
                .get(&(who, o, what.to_string()))
                .map(|(f, r)| format!("{f}: {r}"))
        };
        // **The value diff, block 4577** (frame 4576, the script step that
        // researches Empire): who=1's food stood ours 161 against 197 from
        // here to the word 5782 — Empire at 144 food here, 108 there, the
        // quarter Dye takes off a civic epoch (`docs/AI.md` §99.8). With
        // the arm the whole goods record agrees on every block.
        pin_eq!(
            row(1, -1, "leader:bucket[0:food]"),
            None,
            "who=1's food agrees through 4577"
        );
        // **What parts is not the goods**: the human's `filled_gather_slots`
        // from block 1 (a standing row every widening of this game carries),
        // and the building and group records `LEADERS=2` does not print,
        // which this crate holds alone.
        let leader: Vec<_> = w
            .firsts
            .keys()
            .filter(|(_, o, what)| *o == -1 && what.starts_with("leader:"))
            .map(|(who, _, what)| format!("{who} {what}"))
            .collect();
        pin_eq!(
            leader,
            [
                "0 leader:filled_gather_slots[0:food]",
                "0 leader:filled_gather_slots[1:timber]"
            ],
            "the leader rows that part on run482"
        );
        pin_eq!(w.firsts.len(), 40, "every key parted on run482");
    }

    /// **Every road search to 7070, node for node** (item 1260). run483's
    /// trace proxies `valid_roadcoord` and `calc_road_cost` over the whole
    /// game, so each search's priced nodes are comparable with ours from
    /// frame 0: 49 frames of them to the old word, the last caravan `1/52`'s
    /// replan of its road from `1/2007` to `1/2022`. It parted at node 1129
    /// on 7070 — tile (150, 123) 114 against 37, a road the original still
    /// had and ours had swept away on 6949 — until `leech_codes` joined the
    /// tile beside the Farm's footprint back to its run (`docs/ROADS.md`
    /// §11, `docs/AI.md` §99.9).
    #[test]
    fn run483_s_road_searches_hold_node_for_node_to_7070() {
        let _pins = Pins::hold();
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(t483)) = (
            dump(TOUGHEST_LONG.0),
            dump(TOUGHEST_START),
            trace(TOUGHEST_LONG.1),
            trace("rontrace-run483.log"),
        ) else {
            eprintln!("skipping: no run470/run468/run483 (set RON_GAMELOG_DIR)");
            return;
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
        built.sim.trace_costs = true;
        let mut searched = Vec::new();
        let mut parted = Vec::new();
        while built.sim.frame <= TOUGHEST_ROAD_FRAME_7070 {
            let f = built.sim.frame;
            built.sim.road_marks.clear();
            built.tick();
            let ours = std::mem::take(&mut built.sim.road_marks);
            let theirs = t483.road_nodes(f);
            if ours.is_empty() && theirs.is_empty() {
                continue;
            }
            searched.push((f, theirs.len()));
            if let Some(i) =
                (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i))
            {
                parted.push(format!(
                    "{f} at {i}: ours {:?} theirs {:?} ({} against {})",
                    ours.get(i),
                    theirs.get(i),
                    ours.len(),
                    theirs.len()
                ));
            }
        }
        pin!(parted.is_empty(), "a road search parted: {parted:?}");
        pin_eq!(searched.len(), 49, "the road-search frames to 7070");
        pin_eq!(
            searched.last().copied(),
            Some((TOUGHEST_ROAD_FRAME_7070, 2_543)),
            "7070's search arrives on its frame"
        );
        // The endpoints, from `astar_caravan_road`'s own bracket: caravan
        // slot 2's route, the second city to the third.
        let bracket = t483.calls_in(TOUGHEST_ROAD_FRAME_7070, 5);
        pin_eq!(
            bracket
                .iter()
                .map(|c| (c.args[1], c.args[2], c.args[3], c.args[5]))
                .collect::<Vec<_>>(),
            [(2007, 1, 2022, 2)],
            "one road plan on 7070, 1/2007 to 1/2022 for caravan slot 2"
        );
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
