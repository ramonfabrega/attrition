//! **The Romans at a held Medieval age, East Indies** (DECISIONS 90, 91,
//! 93, 94, item 3620): East Indies, `MAP_STYLE 18`, human Nubians (4)
//! against the **Romans (6)** at Difficulty 5, seed 12345,
//! `STARTING_RESOURCES 7`, the age held at Medieval: `STARTING_TECHNOLOGY
//! 2`, `ENDING_TECHNOLOGY 2` — the Lakota Medieval pair's lobby with the
//! nation moved. The Romans' other lobbies are `roman_word`'s
//! `EastIndiesRomanTier2`, `(18, 6, 2)`, the held-out
//! `GreatLakesRomanTier2`, `(14, 6, 2)`, and the Classical pair's
//! `(7, 6, 5)` — another difficulty, or another map. Captures: run1274 (a
//! `DUMP_ALL` start) and run1275 (the draw stream to the game's end,
//! `cover=0`, `end:UNITS=3,GUYS=2` on a stride of 100, walked from the
//! start's initial state). The first try is in `docs/AI.md` §330.

use super::coverage_pair::Pair;
use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) const ROMAN_MEDIEVAL: Pair = Pair {
    name: "EastIndiesRomanMedieval",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 5,
    starting_technology: 2,
    ending_technology: 2,
    starting_resources: 7,
    start: "gamelog-run1274-eastindies-roman-medieval-start.txt",
    long: (
        "gamelog-run1275-eastindies-roman-medieval-trace.txt",
        "rontrace-run1275.log",
    ),
    // The game ended itself on 5979 (closing block 5980).
    length: 5979,
    // Item 3620, first try: frame 1875 by the count and the sequence, ours
    // 9 draws against 8, index 1 — ours `Guy::set_anim+0x97a <
    // Guy::move+0x19f`, theirs `Guy::set_anim+0x97a <
    // Unit::do_guard+0x7f4` (`docs/AI.md` section 330). Item 3622: 1875 ->
    // **3607** by the count and the sequence — `swarm_around` queues
    // nothing when the ring finds no spot, so `do_build`'s re-swarm of the
    // Dock site `1/2023` drops `1/17`'s build as theirs does. The new
    // word: ours 3205 draws against 3208, index 3201 — ours
    // `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, theirs
    // `PathFinder::calc_road_cost+0x46`.
    // Item 3631: 3607 -> **5582** by the count and the sequence — a
    // finished fort sees `FORT_UPGRADE_LOS[get_fort_los - 1]` further, so
    // the Roman Fort `1/2013`'s disc lights fog cell (110, 85) and the
    // Caravan `1/49`'s road search on 3606 prices node 126 at 89, not 179.
    // The new word: ours 31 draws against 35, index 7 — ours
    // `Unit::fight+0x824`, theirs `Guy::set_anim+0x97a <
    // Unit::move_step+0x823`.
    // Item 3633: 5582 -> **5971** by the count and the sequence — the
    // Cavalry Archer (piece 138) and the Caravel (piece 275) loose from
    // the original's own release points: `1/79`'s arrow at the Village
    // `0/2000` flies 2 frames, not 3, and `1/130`'s broadside at the
    // Citizen `0/10` 7, not 8. The new word: ours 148 draws against 149,
    // index 0 — ours `Unit::think_fish+0x27a`, theirs
    // `Unit::fight+0x9b0`.
    // Item 3635: 5971 -> **5979 of 5979**, the pair whole —
    // `Guy::set_all_pivots`' gone test is the target slot's `flags & 1`,
    // never its hit points, so the Cavalry Archer `1/124`'s turret keeps
    // bearing on the Village `0/2000` at its ceiling (5963) and
    // `move_step` keeps `cavarch_o` 2000 to 5969, as theirs does.
    count: 5979,
    sequence: 5979,
};

/// The pair's word as the handoff's line carries it: the lower of the two
/// partings.
pub(crate) const ROMAN_MEDIEVAL_PAIR_WORD: i64 = if ROMAN_MEDIEVAL.count < ROMAN_MEDIEVAL.sequence {
    ROMAN_MEDIEVAL.count
} else {
    ROMAN_MEDIEVAL.sequence
};

/// **The pair's first parting**, walked from its own start.
#[test]
fn roman_medieval_pair_first_parting() {
    let _pins = Pins::hold();
    let p = &ROMAN_MEDIEVAL;
    let Some(w) = third::walk_from(p.start, p.long) else {
        return;
    };
    eprintln!(
        "ROMAN_MEDIEVAL {}: count {} sequence {} of {}; {}",
        p.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, p.difficulty, "the lobby read back");
    pin_eq!(w.last, p.length, "the game's last frame");
    pin_eq!(w.count, p.count, "the draw count's first parting");
    pin_eq!(w.sequence, p.sequence, "the draw sequence's first parting");
}

/// Both captures say their lobby from their own `GAME INFO` and `PLAYER`
/// blocks, the age held at 2, and the long is scoreable.
#[test]
fn roman_medieval_pair_captures_say_their_lobby() {
    let p = &ROMAN_MEDIEVAL;
    for name in [p.long.0, p.start] {
        let Some(path) = dump(name) else {
            continue;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().expect("an initial state");
        p.assert_lobby(&init, name);
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    assert_eq!(
        third_pair::incomplete_long(&Log::parse(&text)),
        Vec::<&str>::new(),
        "the Roman Medieval pair's long is not scoreable"
    );
}

/// The pair is a scored lobby (the battery's identity test refuses it) and
/// none of the battery's: the battery's one Roman game is the held-out
/// `GreatLakesRomanTier2`, `(14, 6, 2)`; the AI word `EastIndiesRomanTier2`
/// is `(18, 6, 2)` — this map at another difficulty — and the Classical
/// pair's `(7, 6, 5)` another map (item 3620). Both `GAME INFO`s are read.
#[test]
fn roman_medieval_pair_is_no_other_lobby() {
    let p = &ROMAN_MEDIEVAL;
    let triple = (p.map_style, p.ai_tribe, p.difficulty);
    assert!(battery::SCORED_LOBBIES.contains(&triple));
    pin!(
        !battery::BATTERY
            .iter()
            .any(|l| (l.map_style, l.ai_tribe, l.difficulty) == triple)
    );
    let romans: Vec<_> = battery::BATTERY
        .iter()
        .filter(|l| l.ai_tribe == p.ai_tribe)
        .map(|l| (l.name, l.map_style, l.difficulty))
        .collect();
    assert_eq!(
        romans,
        [("GreatLakesRomanTier2", 14, 2)],
        "the battery's Roman games"
    );
    let w = &super::roman_word::ROMAN;
    let c = &super::roman_classical_pair::ROMAN_CLASSICAL;
    for (name, l) in [
        (w.name, (w.map_style, w.ai_tribe, w.difficulty)),
        (c.name, (c.map_style, c.ai_tribe, c.difficulty)),
    ] {
        assert_eq!(l.1, p.ai_tribe, "{name} fields the Romans");
        assert_ne!(l, triple, "{name} is this lobby");
    }
    let Some(path) = dump(p.long.0) else {
        return;
    };
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let init = log.initial().expect("an initial state");
    let get = |key: &str| {
        init.game_info
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.parse::<i32>().ok())
    };
    assert_eq!(
        (
            get("MAP_STYLE"),
            get("DIFFICULTY"),
            get("STARTING_TECHNOLOGY"),
            get("ENDING_TECHNOLOGY"),
        ),
        (Some(18), Some(5), Some(2), Some(2)),
        "this pair's lobby"
    );
}

/// run1275b: the word 1875's window, blocks 1870..2126 at the long's
/// detail — six blocks before the word's block 1876 and 250 after (item
/// 3620); the stride's blocks are 1801 and 1901.
pub(crate) const RUN1275B: &str = "gamelog-run1275b-eastindies-roman-medieval-window-1870-2126.txt";

/// The window: block 1870 through 2126; the word 1875's own block is 1876.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_FRAME_1875: (i64, i64) = (1870, 2126);

/// **The word 1875's window**: run1275b walked from run1274's start with
/// the recorder on (item 3620).
pub(crate) fn roman_medieval_frame_1875_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1275b",
        &[(RUN1275B, WIDENING_ROMAN_MEDIEVAL_FRAME_1875.0)],
        WIDENING_ROMAN_MEDIEVAL_FRAME_1875,
        1,
        &[1876],
        true,
    )
}

#[test]
fn roman_medieval_pair_run1275b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_frame_1875_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3620, the first walk: **1211** keys. Block 1870 carries the
    // standing families (`form`, `SITE[i].reg`, the pools; 57 rows).
    // **The state first parts on block 1875**, at the Roman Citizen `1/17`
    // (TypeIndex 50, land, uid 37) at (35340, 42696), its current order a
    // `BUILDORDER` on the building `1/2023` (uid 46) over an
    // `EXPLORETOORDER` to (36456, 43320) and a `BUILDORDER` on `1/2030`:
    // theirs pops the build and makes the explore, its destination moved
    // to (36600, 43320), current (`orders.len` 2, `order:kind` 3); ours
    // moves the explore's destination alike and keeps the build current
    // (`orders.len` 3, `order:kind` 6, `orders_x`/`_y` (35340, 42696)
    // against (36600, 43320), `dest_angle` -1073741824 against
    // 2110717952). On 1876 theirs walks (`path:length` 4, `pos` (35349,
    // 42720)) and ours stands. **The word 1875**, ours 9 draws against 8,
    // index 1: ours spends `1/17`'s `Guy::set_anim+0x97a <
    // Guy::move+0x19f`, and the other eight agree in order — ours acts
    // (`docs/AI.md` section 330). The rest follows from `1/17`.
    //
    // Item 3622: **1211 -> 83**. `do_build` finds `1/17` out of reach of
    // the Dock site `1/2023` and re-swarms it through the insert dance;
    // the ring finds no spot, so the original queues neither the approach
    // nor the build, and `finish_insert` re-swarms the build on `1/2030`
    // under it (the new explore to (36600, 43320)). Ours queued the build
    // with no approach and stood on it. On 1875 both sides now hold
    // `orders.len` 2, `order:kind` 3 at (36600, 43320), and on 1876 both
    // stand at (35349, 42720) with a path of 4. What is left is the
    // standing families on 1870 and value rows past 1918 (the pools and
    // groups from 1919, `SITE[i]` on 1976, `1/64`'s `form` on 2030), none
    // of them the Citizen's.
    pin!(
        w.firsts.keys().all(|(who, o, _)| (*who, *o) != (1, 17)),
        "the Citizen 1/17 parts nowhere in the window"
    );
    pin_eq!(w.firsts.len(), 83, "run1275b baseline");
}

/// run1277: the word 3607's window, blocks 3602..3858 at the long's detail
/// — six blocks before the word's block 3608 and 250 after (item 3622);
/// the stride's blocks are 3601 and 3701.
pub(crate) const RUN1277: &str = "gamelog-run1277-eastindies-roman-medieval-window-3602-3858.txt";

/// The window: block 3602 through 3858; the word 3607's own block is 3608.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_FRAME_3607: (i64, i64) = (3602, 3858);

/// **The word 3607's window**: run1277 walked from run1274's start with
/// the recorder on (item 3622).
pub(crate) fn roman_medieval_frame_3607_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1277",
        &[(RUN1277, WIDENING_ROMAN_MEDIEVAL_FRAME_3607.0)],
        WIDENING_ROMAN_MEDIEVAL_FRAME_3607,
        1,
        &[3608],
        true,
    )
}

#[test]
fn roman_medieval_pair_run1277_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_frame_3607_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3622, the first walk: **720** keys. Block 3602 carries the
    // standing families (`form` 63, `leader` 19, the pools and groups 22;
    // 104 rows), block 3603 a pool and its group. **Nothing parts on the
    // word's block 3608**, nor on any block to 3614: the word 3607 is the
    // Caravan `1/49` (TypeIndex 59, uid 81) on a `TRADEORDER` from
    // `1/2032` to `1/2000`, whose `PathFinder::astar_caravan_road` spends
    // 3201 `calc_road_cost` draws in ours against 3204 in theirs; the four
    // draws after agree in order (`1/141`'s `do_idle`, `0/12`'s and
    // `9/13`'s `Guy::inc_time`, `Farms::inc_time`). The state first parts
    // on block 3615, at `1/49`'s move: `order:move.x`/`.y` (37944, 24504)
    // against (38328, 23928), `off_x`/`off_y` (312, 696) against (696,
    // 120), `path:length` 46 against 47.
    //
    // Item 3631: 720 -> **112**. The road search parted on 3606, node 126
    // (ours 179 against 89: `was_seen`'s doubling), because the Roman Fort
    // `1/2013` saw 14 here and 16 there (`FORT_UPGRADE_LOS` at fort level
    // 2) and its disc left nine of who=1's fog cells dark (run1293's
    // packet). With the fort term, `1/49`'s rows are gone; what stays is
    // the standing families (106 on 3602..3603) and, later, the pools and
    // groups 73 (3635) and 77 (3782) and `form` on `1/186` (3683) and
    // `1/187` (3736) — no position, no order.
    pin!(
        w.firsts.values().all(|(f, _)| !(3604..=3614).contains(f)),
        "nothing parts from block 3604 to 3614"
    );
    pin!(
        w.firsts.iter().all(|((_, _, what), (f, _))| *f <= 3603
            || what == "form"
            || what.starts_with("pool:")
            || what.starts_with("group:")),
        "past the standing blocks only pools, groups and form part"
    );
    pin_eq!(w.firsts.len(), 112, "run1277 baseline");
}

/// **The word's road search, node for node** (item 3631): every road search
/// run1275's call window prices, from the start to the word 3607, against
/// ours.
#[test]
fn roman_medieval_pair_s_road_search_on_3607_holds_node_for_node() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let p = &ROMAN_MEDIEVAL;
    let (Some(path), Some(sib), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1)) else {
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
    while built.sim.frame <= 3607 {
        let f = built.sim.frame;
        built.sim.road_marks.clear();
        built.tick();
        let ours = std::mem::take(&mut built.sim.road_marks);
        let theirs = tr.road_nodes(f);
        if f == 3607 {
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
    eprintln!("ROAD searched {searched}; first parting {first:?}; on 3607 {on_word:?}");
    // Item 3631: (3201, 3204) on the word, and the first node parting on
    // 3606 (node 126, the diagonal onto (42336, 32928), tile (220, 171):
    // ours 179 against theirs 89, `was_seen`'s doubling of fog cell
    // (110, 85)) until a finished fort carried `FORT_UPGRADE_LOS`: the
    // Fort `1/2013`'s disc reaches it at radius 8.
    pin_eq!(
        on_word,
        (3204, 3204),
        "the word's road nodes, ours and theirs"
    );
    pin_eq!(
        first.as_deref().map(|s| s.split(':').next().unwrap_or("")),
        None,
        "the first node that parts"
    );
}

/// run1293b: the word 5582's window, blocks 5577..5833 at the long's detail
/// — six blocks before the word's block 5583 and 250 after (item 3631);
/// the stride's blocks are 5501 and 5601.
pub(crate) const RUN1293B: &str = "gamelog-run1293b-eastindies-roman-medieval-window-5577-5833.txt";

/// The window: block 5577 through 5833; the word 5582's own block is 5583.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_FRAME_5582: (i64, i64) = (5577, 5833);

/// **The word 5582's window**: run1293b walked from run1274's start with
/// the recorder on (item 3631).
pub(crate) fn roman_medieval_frame_5582_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1293b",
        &[(RUN1293B, WIDENING_ROMAN_MEDIEVAL_FRAME_5582.0)],
        WIDENING_ROMAN_MEDIEVAL_FRAME_5582,
        1,
        &[5583],
        true,
    )
}

#[test]
fn roman_medieval_pair_run1293b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_frame_5582_window() else {
        return;
    };
    pin_eq!(w.blocks, 256, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    // Item 3631: 2550 keys. Block 5577 carried the standing families and,
    // from run1293c's gap, the human's `0/10` already apart (`pos` y 4201
    // against 4226). Item 3633: **102** — the Caravel `1/130`'s broadside
    // lands on 5550 as theirs does, so `0/10` flees in step. What is left
    // is the standing families on 5577 (89 rows), pools and groups 75, 81
    // and 83, `form` on five units born in the window, `SITE[7].reg` and
    // two `dest_angle` rows; no position and no order.
    pin!(
        w.firsts
            .iter()
            .all(|((_, _, k), (f, _))| *f == 5577 || !(k == "pos" || k.starts_with("order"))),
        "no position or order parts past the standing block"
    );
    pin_eq!(w.firsts.len(), 102, "run1293b baseline");
}

/// **The long's stride, walked back from the word 5582** (item 3631):
/// run1275's stride blocks 3901..5501, between run1277's window and
/// run1293b's.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_STRIDE: (i64, i64) = (3901, 5501);

pub(crate) fn roman_medieval_stride_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1275 stride",
        &[(ROMAN_MEDIEVAL.long.0, WIDENING_ROMAN_MEDIEVAL_STRIDE.0)],
        WIDENING_ROMAN_MEDIEVAL_STRIDE,
        1,
        &[],
        true,
    )
}

#[test]
fn roman_medieval_pair_the_long_s_stride_walks_back_from_5582() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_stride_window() else {
        return;
    };
    pin_eq!(w.blocks, 17, "every stride block in the window");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    pin_eq!(w.firsts.len(), 180, "run1275 stride baseline");
}

/// run1293c: the gap before the word 5582's window, blocks 5501..5577 at
/// the long's detail (item 3631): run1275's stride prints 5501 and run1293b
/// opens with the human's `0/10` already apart.
pub(crate) const RUN1293C: &str = "gamelog-run1293c-eastindies-roman-medieval-gap-5501-5577.txt";

/// The window: block 5501 through 5577.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_GAP_5582: (i64, i64) = (5501, 5577);

/// **The gap before the word 5582's window**: run1293c walked from
/// run1274's start with the recorder on (item 3631).
pub(crate) fn roman_medieval_gap_5582_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1293c",
        &[(RUN1293C, WIDENING_ROMAN_MEDIEVAL_GAP_5582.0)],
        WIDENING_ROMAN_MEDIEVAL_GAP_5582,
        1,
        &[],
        true,
    )
}

#[test]
fn roman_medieval_pair_run1293c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_gap_5582_window() else {
        return;
    };
    pin_eq!(w.blocks, 76, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3631: 113 keys. Block 5501 carries the standing families (91
    // rows) and agrees on every unit. The state first parted on block
    // 5519, at the human's Village `0/2000`: `damage` ours 552 against 554
    // (548 on both on 5518; theirs' blows on tick 5518 were six, ours
    // four). Then the human's `0/10` on 5551: theirs took a hit from
    // `1/130` on 5550, ours none.
    //
    // Item 3633: **93**. The six were the Knight `1/14`'s 2, `1/88`'s
    // arrow's 2 and the Cavalry Archer `1/79`'s arrow's 2, which ours
    // landed a tick late: it left from piece 138's facing-only node, 305
    // units out at a speed of 100, and flew 3. Through the pivot it leaves
    // from (6086, 7611), 293 out, and flies 2. The Caravel `1/130`'s
    // first round at `0/10` left the hull's own square, 1653 out at 200,
    // and flew 8; from its broadside at (3476, 2151) it flies 7 and lands
    // on 5550. What is left past the standing block is `form` on `1/174`
    // (5544) and `1/186` (5564), units born in the window.
    pin!(
        w.firsts
            .iter()
            .filter(|(_, (f, _))| *f > 5501)
            .map(|(k, (f, _))| (*f, k.clone()))
            .min()
            == Some((5544, (1, 174, "form".to_string()))),
        "the first parting past the standing block is a newborn's form on 5544"
    );
    pin_eq!(w.firsts.len(), 93, "run1293c baseline");
}

/// run1295: the word 5971's window, blocks 5833..5979 at the long's detail
/// and `AMMO=5` (item 3633): from run1293b's last block to the game's last
/// frame, since the long prints only 5901 and the closing block 5980.
pub(crate) const RUN1295: &str = "gamelog-run1295-eastindies-roman-medieval-window-5833-5979.txt";

/// The window: block 5833 through 5979; the word 5971's own block is 5972.
/// Item 3635: through the closing block 5980, the pair whole at 5979.
pub(crate) const WIDENING_ROMAN_MEDIEVAL_FRAME_5971: (i64, i64) = (5833, 5980);

/// **The word 5971's window**: run1295 walked from run1274's start with
/// the recorder on (item 3633).
pub(crate) fn roman_medieval_frame_5971_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN_MEDIEVAL.start],
        true,
        ROMAN_MEDIEVAL.long,
        "run1295",
        &[(RUN1295, WIDENING_ROMAN_MEDIEVAL_FRAME_5971.0)],
        WIDENING_ROMAN_MEDIEVAL_FRAME_5971,
        1,
        &[],
        true,
    )
}

#[test]
fn roman_medieval_pair_run1295_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_medieval_frame_5971_window() else {
        return;
    };
    pin_eq!(w.blocks, 146, "every captured block");
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
    }
    // Item 3633: **104** keys. Block 5833 carries the standing families
    // (89 rows); then `form` on a newborn (5847) and pools and groups 72,
    // 84 and 85. **The state first parts on block 5964, at the Cavalry
    // Archer `1/124`** (TI 188, on an `ATTACKTOORDER` walking at
    // (4488, 8328)): `cavarch_o` ours -1 against theirs' 2000, the human's
    // Village — theirs' fire on the move takes a target ours does not. Then
    // its `recharging` 5 against 0 on 5969 and its guy's target on 5971;
    // `1/145`'s move pause on 5972, `1/17`'s gather wait on 5975 and the
    // leader's peasant counts on 5978 follow.
    // Item 3635: **98** keys, and no unit parts past the standing block —
    // `1/124` keeps bearing on the Village at its ceiling (`set_all_pivots`'
    // gone test is the slot's `flags & 1`), and its four rows go with
    // `1/145`'s and `1/17`'s. Left: the standing families, the newborn's
    // `form`, the pools and groups, and the leader's `free_peasants` and
    // `gatherers` on 5978 (ours 3 and 33 against 2 and 34).
    pin!(
        w.firsts
            .iter()
            .filter(|(_, (f, _))| *f > 5833)
            .filter(|((_, o, _), _)| *o >= 0)
            .filter(|((_, _, k), _)| k != "form")
            .map(|(k, (f, _))| (*f, k.clone()))
            .min()
            .is_none(),
        "no unit parts past the standing block"
    );
    pin_eq!(w.firsts.len(), 98, "run1295 baseline");
}

/// **The closing state** (item 3635): run1275's closing block 5980, the
/// whole map's units, walked from run1274's start.
#[test]
fn roman_medieval_pair_run1275_s_closing_state() {
    let _pins = Pins::hold();
    let p = &ROMAN_MEDIEVAL;
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) = (dump(p.long.0), dump(p.start), trace(p.long.1))
    else {
        return;
    };
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(path);
    let start_text = crate::capture::read(start);
    let log = Log::parse(&text);
    let start_log = Log::parse(&start_text);
    let mut init = log.initial().unwrap();
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
        "Roman Medieval pair closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    for &(who, o) in &e.extra {
        if let Some(u) = built
            .sim
            .units
            .iter()
            .find(|u| u.alive() && i64::from(u.owner) == who && i64::from(u.index) == o)
        {
            let name = u.ty.and_then(|ty| loaded.unit_names.get(ty).cloned());
            eprintln!("    extra {who}/{o} {name:?} at {:?}", u.pos);
        }
    }
    pin_eq!(e.frame, 5980, "the closing block");
    // Item 3635, the pair whole: 247 units, none off, torn or unlinked,
    // no extra, `Closing fields: none`.
    pin_eq!(e.compared, 247, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.extra.clone(),
        Vec::<(i64, i64)>::new(),
        "the closing extras"
    );
}
