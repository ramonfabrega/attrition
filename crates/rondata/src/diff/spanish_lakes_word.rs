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
    // corner the millionths truncated differently. **13496 -> 13639** with
    // the Monarch's siege doubling (`UnitData::get_speed`'s
    // `has_general(0, 0x162)` arm): run1013c's Bombards `1/113` and `1/114`
    // stepped 49 on 13484 against ours' 24, `x[0]` 9025 against 9042; both
    // read 9025 on the built tree. **13639 -> 14068**, the game's end, with
    // the same arm in `Unit::do_cast`'s pack clock: run1013's Bombard
    // `1/114` unpacks (spell 652) from 13600, theirs' `spell_time` climbing
    // by two (78 on 13639, done on 80) against ours' one, so ours still
    // played `CHAR_UNPACK` on 13639 (`Guy::set_anim+0x104b`) where theirs'
    // `Guy::inc_time` ended it. Both read the cast popped on 13640.
    count: 14068,
    sequence: 14068,
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
    // Item 3390: 3488 keys on the march's parting (13490, the Spanish army
    // 1/26..1/114); **3488 -> 1151** with the Monarch's siege doubling: the
    // standing family on 13490 (34), then 13576 and 13601, and the word's
    // own parting from 13640. **1151 -> 38** with the Monarch's arm in the
    // pack clock (`1/114`'s unpack, `spell_time` 1 against 2 on 13601):
    // the standing family, 13576's `leader:attack` (269 against 275), two
    // `dest_angle` rows (`1/74` 13710, `1/78` 13732) and a newborn's
    // `g.angle[0]` (`1/89`, 13735), none parting a draw.
    pin_eq!(w.firsts.len(), 38, "keys that part in the window");
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
    // ours (9000, 30984)). run1013c owes 13470..13484. **129 -> 39** with
    // the Monarch's siege doubling (run1013c's 13484): the 90 close.
    pin_eq!(w.firsts.len(), 39, "keys that part in the gap");
}

/// run1013c: the army's parting, blocks 13463..13489 at run1013's detail
/// (item 3390); run1013b reads every unit agreeing on 13469 and three
/// Spanish units parted on 13485.
pub(crate) const RUN1013C: &str =
    "gamelog-run1013c-greatlakes-spanish-diff4-window-13463-13489.txt";

/// The window: block 13463 through 13489.
pub(crate) const WIDENING_SPANISH_LAKES_MARCH_1013C: (i64, i64) = (13463, 13489);

pub(crate) fn spanish_lakes_march_1013c_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[SPANISH_LAKES.start],
        true,
        SPANISH_LAKES.long,
        "run1013c",
        &[(RUN1013C, WIDENING_SPANISH_LAKES_MARCH_1013C.0)],
        WIDENING_SPANISH_LAKES_MARCH_1013C,
        1,
        &[13485],
        true,
    )
}

#[test]
fn run1013c_s_march_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = spanish_lakes_march_1013c_window() else {
        return;
    };
    pin_eq!(w.blocks, 26, "every captured block");
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
    // Item 3390: 260 keys — the standing family on 13463 (34), and from
    // **13484** the Bombards `1/113` and `1/114` (TI 267, group 65,
    // `GROUPATTACKTO`) stepping 49 against ours' 24 (`g.last_speed[0]`),
    // `x[0]` 9025 against 9042, `myspeed` 23 both sides: the figures'
    // uncapped `get_speed` doubled, the unit's capped one not. The Spanish
    // Monarch (TI 354, `0x162`) is in range: `UnitData::get_speed`'s
    // `has_general(0, 0x162)` arm, which this crate left a SEAM. **260 ->
    // 34** with it built (`Sim::near_monarch`).
    pin_eq!(w.firsts.len(), 34, "keys that part in the window");
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

/// The closing state's window: run1008b's closing block alone, the long's
/// last frame 14068 writing it (item 3390).
pub(crate) const WIDENING_SPANISH_LAKES_CLOSING: (i64, i64) = (14067, 14070);

/// **The lobby's closing state** (item 3390, DECISIONS 55): run1008b — the
/// draw stream's own capture, `cover=0` — prints its closing block whole.
/// The game ends itself on 14068.
#[test]
fn run1008b_spanish_lakes_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (
        dump(SPANISH_LAKES.long.0),
        dump(SPANISH_LAKES.start),
        trace(SPANISH_LAKES.long.1),
    ) else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = start_log.initial().unwrap();
    let sibling = start_log.initial().unwrap();
    borrow_from_siblings(&mut init, &[&sibling]);
    borrow_pasture(&mut init, &tr);
    let mut built = build_sim(&loaded, &init, Tuning::RON);
    let fin = log
        .final_state()
        .or_else(|| log.frame_states().pop())
        .expect("closing whole-map state");
    let e = endpoint::walk_to_close(&mut built, &fin, 8);
    eprintln!(
        "Spanish (Great Lakes) closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
        e.frame,
        e.compared,
        e.counts(),
        e.torn,
        e.off,
        e.unlinked,
        e.extra
    );
    eprintln!(
        "Closing fields: {}",
        harness::compare(&built, &fin, 8).closing_fields()
    );
    pin_eq!(e.frame, 14069, "the closing block");
    pin_eq!(e.compared, 110, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    // Item 3390, the first walk (the Monarch's siege doubling and cast
    // clock, the word at 14068): 110 units compared on block 14069, none
    // apart, no building or city field-row, no closing field.
}
