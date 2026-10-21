//! **The battery's Spanish word at Great Lakes** (DECISIONS 72, 83, 84;
//! item 3390): the held-out battery's lowest open game on the measure of
//! `02005991` (the Inca at Great Sahara being item 3385's), promoted by item
//! 3335's recipe. Great Lakes (`MAP_STYLE 14`), human Nubians (4) against
//! **the Spanish (9)** at Difficulty 4, seed 12345 — minted by item 3384
//! under entry 72's rule (2), scored by its first try and promoted out of
//! the battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. The names
//! `spanish_word` and `spanish_east_word` are the closed Great Sahara and
//! East Indies games' (items 3318, 3367). Captures: run1008 (a `DUMP_ALL`
//! start) and run1008b (the draw stream to the game's end, `cover=0`, its
//! setup and closing blocks only); the game ends itself on 14068
//! (`docs/AI.md` §294).

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const SPANISH_LAKES: Lobby = Lobby {
    name: "GreatLakesSpanishTier4",
    map_style: 14,
    ai_tribe: 9,
    difficulty: 4,
    start: "gamelog-run1008-greatlakes-spanish-diff4-start.txt",
    long: (
        "gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt",
        "rontrace-run1008b.log",
    ),
    length: 14068,
    // Item 3384's lobby, its first try: count 8384, sequence 8384 (ours
    // 3266 draws, theirs 3271, at 3262: ours `Herd::process+0x17`, theirs
    // `PathFinder::calc_road_cost+0x46`). Item 3390 promoted it to the AI
    // word (`docs/AI.md` §294) and moved it 8384 -> 13496 with the
    // terraform in the original's singles: one road search on 7984 priced
    // the step onto (45600, 23520) 339 against 342, a climb one short off a
    // corner the millionths truncated differently.
    count: 13496,
    sequence: 13496,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const SPANISH_LAKES_WORD: i64 = if SPANISH_LAKES.count < SPANISH_LAKES.sequence {
    SPANISH_LAKES.count
} else {
    SPANISH_LAKES.sequence
};
/// **The lobby's first parting**, walked from its own start.
#[test]
fn spanish_lakes_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(SPANISH_LAKES.start, SPANISH_LAKES.long) else {
        return;
    };
    eprintln!(
        "SPANISH_LAKES {}: count {} sequence {} of {}; {}",
        SPANISH_LAKES.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(
        w.difficulty,
        SPANISH_LAKES.difficulty,
        "the lobby read back"
    );
    pin_eq!(w.last, SPANISH_LAKES.length, "the game's last frame");
    pin_eq!(
        w.count,
        SPANISH_LAKES.count,
        "the draw count's first parting"
    );
    pin_eq!(
        w.sequence,
        SPANISH_LAKES.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn spanish_lakes_word_is_no_other_lobby() {
    let p = &SPANISH_LAKES;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    assert!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        let info = |key: &str| {
            init.game_info
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, v)| v.parse::<i32>().ok())
        };
        assert_eq!(info("MAP_STYLE"), Some(p.map_style), "{name}");
        assert_eq!(info("DIFFICULTY"), Some(p.difficulty), "{name}");
    }
}

/// run1013: the word 13496's window, blocks 13490..13746 at `GUYS=4`,
/// `LEADERS=9` and `GOODS=3` (item 3390). run1008 dumps blocks 0..36 and
/// run1008b its setup and closing blocks only, so no dump on disk held the
/// word's block.
pub(crate) const RUN1013: &str = "gamelog-run1013-greatlakes-spanish-diff4-window-13490-13747.txt";

/// The window: block 13490 through 13746; the word 13496's own block is
/// 13497.
pub(crate) const WIDENING_SPANISH_LAKES_FRAME_13496: (i64, i64) = (13490, 13746);

pub(crate) fn spanish_lakes_frame_13496_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_LAKES.start],
        true,
        SPANISH_LAKES.long,
        "run1013",
        &[(RUN1013, WIDENING_SPANISH_LAKES_FRAME_13496.0)],
        WIDENING_SPANISH_LAKES_FRAME_13496,
        1,
        &[13497],
        true,
    )
}

#[test]
fn run1013_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_lakes_frame_13496_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 0, "keys that part in the window");
}

/// run1013b: the gap 8380..13489 at a stride of 16 (item 3390): blocks
/// 8381, 8397, …, 13485, at run1013's detail; run1008b dumps its setup
/// and closing blocks only, and run1013 opens on 13490 already parted.
pub(crate) const RUN1013B: &str =
    "gamelog-run1013b-greatlakes-spanish-diff4-stride16-8380-13489.txt";

/// The window: block 8381 through 13485, one in sixteen.
pub(crate) const WIDENING_SPANISH_LAKES_GAP_1013B: (i64, i64) = (8381, 13485);

pub(crate) fn spanish_lakes_gap_1013b_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_LAKES.start],
        true,
        SPANISH_LAKES.long,
        "run1013b",
        &[(RUN1013B, WIDENING_SPANISH_LAKES_GAP_1013B.0)],
        WIDENING_SPANISH_LAKES_GAP_1013B,
        1,
        &[13485],
        true,
    )
}

#[test]
fn run1013b_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_lakes_gap_1013b_window() else {
        return;
    };
    pin_eq!(w.blocks, 320, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        Vec::<String>::new(),
        "the keys the capture prints and nothing reads"
    );
    // Item 3390: 129 keys. The standing family on 8381 (31); a newborn's
    // `g.angle[0]` (8557), the groups 67 and 68's pools (8877, 9805),
    // `SITE[2].reg` (9181), a one-unit `des_x` (9773) — none parts a
    // draw. Every unit agrees on 13469; on **13485** three Spanish units
    // part (90 keys): the Bombards `1/113` and `1/114` move half again
    // ours' speed (`avg_speed[0]` 33 against 22, `x[0]` 8991 against 9025),
    // and `1/56`'s guard order points 48 apart (theirs (8952, 31032),
    // ours (9000, 30984)). run1013c owes 13470..13484.
    pin_eq!(w.firsts.len(), 129, "keys that part in the gap");
}

/// **The word's road search, node for node** (item 3390): every road
/// search run1008b's call window prices, from the start to the word 8384,
/// against ours.
#[test]
fn run1008b_s_road_search_on_8384_holds_node_for_node() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) = (
        dump(SPANISH_LAKES.long.0),
        dump(SPANISH_LAKES.start),
        trace(SPANISH_LAKES.long.1),
    ) else {
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
    built.sim.trace_costs = true;
    let mut first: Option<String> = None;
    let mut searched = 0;
    let mut on_word = (0, 0);
    while built.sim.frame <= 8384 {
        let f = built.sim.frame;
        built.sim.road_marks.clear();
        built.tick();
        let ours = std::mem::take(&mut built.sim.road_marks);
        let theirs = tr.road_nodes(f);
        if f == 8384 {
            on_word = (ours.len(), theirs.len());
        }
        if ours.is_empty() && theirs.is_empty() {
            continue;
        }
        searched += 1;
        if first.is_none()
            && let Some(i) =
                (0..ours.len().max(theirs.len())).find(|&i| ours.get(i) != theirs.get(i))
        {
            first = Some(format!(
                "{f} at {i}: ours {:?} theirs {:?} ({} against {})",
                ours.get(i),
                theirs.get(i),
                ours.len(),
                theirs.len()
            ));
            if std::env::var("RON_ROAD_NODES").is_ok() {
                for j in i.saturating_sub(8)..(i + 12).min(ours.len().max(theirs.len())) {
                    eprintln!(
                        "  node {j}: ours {:?} theirs {:?}",
                        ours.get(j),
                        theirs.get(j)
                    );
                }
            }
        }
    }
    eprintln!("ROAD searched {searched}; first parting {first:?}; on 8384 {on_word:?}");
    // Item 3390: (3202, 3207) and the first node parting on 7984 (node 855,
    // the step onto (45600, 23520), ours 339 against theirs 342) until the
    // terraform carried the original's singles.
    pin_eq!(
        on_word,
        (3207, 3207),
        "the word's road nodes, ours and theirs"
    );
    pin_eq!(first, None, "the first node that parts");
}
