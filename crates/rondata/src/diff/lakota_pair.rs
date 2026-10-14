//! **The peace pair** (DECISIONS 53 §2, 61 §6, item 2320): the successor
//! pair closed at its game's end (item 2293), and its successor is the
//! lobby that reaches the most census rows no trace has entered. East
//! Indies (`MAP_STYLE 18`), human Nubians (4) against **the Lakota (19)**
//! at Difficulty 3, seed 12345, with the successor pair's
//! `STARTING_TECHNOLOGY 8` and `STARTING_RESOURCES 7`, and **`RUSH_RULES
//! 10`**, "Peace for first 5 minutes": the nation and the rush rules are
//! what it varies (`docs/AI.md` §218). Captures: run846 (a `DUMP_ALL`
//! start) and run847 (the draw stream to the game's end, `cover=0`).

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;

pub(crate) const LAKOTA: Pair = Pair {
    name: "EastIndiesLakotaPeace",
    map_style: 18,
    ai_tribe: 19,
    difficulty: 3,
    starting_technology: 8,
    starting_resources: 7,
    start: "gamelog-run846-eastindies-lakota-peace-start.txt",
    long: (
        "gamelog-run847-eastindies-lakota-peace-trace.txt",
        "rontrace-run847.log",
    ),
    length: 6822,
    // Item 2320: the first try, count and sequence both at 1384 (ours 596
    // draws, theirs 564, index 28: ours `Leader::create_buildings+0xffb`,
    // theirs `Animal::think_bird+0x82`), inside the five minutes' peace.
    count: 1384,
    sequence: 1384,
};

/// The pair's word as the handoff's `Peace pair:` line and `AI_WORDS` carry it.
pub(crate) const LAKOTA_PAIR_WORD: i64 = if LAKOTA.count < LAKOTA.sequence {
    LAKOTA.count
} else {
    LAKOTA.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn lakota_pair_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(LAKOTA.start, LAKOTA.long) else {
        return;
    };
    eprintln!(
        "LAKOTA {}: count {} sequence {} of {}; {}",
        LAKOTA.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, LAKOTA.difficulty, "the lobby read back");
    pin_eq!(w.last, LAKOTA.length, "the game's last frame");
    pin_eq!(w.count, LAKOTA.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        LAKOTA.sequence,
        "the draw sequence's first parting"
    );
}

/// run848: the word 1384's window, blocks 1378..1634 at the long's detail (item 2320).
pub(crate) const RUN848: &str = "gamelog-run848-eastindies-lakota-peace-window-1378-1635.txt";

/// The window: block 1378 through 1634; the word 1384's own block is 1385.
pub(crate) const WIDENING_LAKOTA_FRAME_1384: (i64, i64) = (1378, 1634);

pub(crate) fn lakota_frame_1384_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[LAKOTA.start],
        true,
        LAKOTA.long,
        "run848",
        &[(RUN848, WIDENING_LAKOTA_FRAME_1384.0)],
        WIDENING_LAKOTA_FRAME_1384,
        1,
        &[1385],
        true,
    )
}

#[test]
fn run848_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = lakota_frame_1384_window() else {
        return;
    };
    pin_eq!(w.blocks, 257, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.firsts.len(), 870, "initial run848 baseline");
    // Item 2320: 870 keys part on blocks 1378..1634, the word's own block
    // being 1385 (frame 1384: ours 596 draws, theirs 564, index 28; ours
    // `Leader::create_buildings+0xffb`, theirs `Animal::think_bird+0x82`).
    // 74 stand on 1378, among them who=1's food economy — `income[food]`
    // ours 320 theirs 1728, `rate` 20/108, `econ` 3/8 — which parts on the
    // start's block 1 already (run846: 160 against 864). On the word's
    // frame, `leader:MAKE[5].val` 28124/29387 and the unit `1/13`'s `pos`
    // (38963,38203)/(39009,38201); on 1383 the building `1/2026`
    // stands at (35904,39744) with `constr_time` 70000 in ours and at
    // (40800,41568) with 560000 in theirs (`docs/AI.md` §218).
}
