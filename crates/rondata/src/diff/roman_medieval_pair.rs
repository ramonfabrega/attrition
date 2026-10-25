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
use crate::testenv::dump;

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
    count: 3607,
    sequence: 3607,
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
    pin!(
        w.firsts.values().all(|(f, _)| !(3604..=3614).contains(f)),
        "nothing parts from block 3604 to 3614"
    );
    pin_eq!(w.firsts.len(), 720, "run1277 baseline");
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
        if std::env::var("RON_PROBE_3631").is_ok() {
            use sim::world::Cell;
            let w = &built.sim.world;
            let now = (
                w.seen2(110, 85),
                w.owner(Cell::new(55, 42)),
                w.region_of(Cell::new(55, 42)),
            );
            thread_local!(static LAST: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) });
            let txt = format!("{now:?}");
            LAST.with(|l| {
                if *l.borrow() != txt {
                    eprintln!("PROBE {f}: seen2/owner/region {txt}");
                    *l.borrow_mut() = txt;
                }
            });
            thread_local!(static LIT: std::cell::RefCell<std::collections::BTreeSet<(i32, i32)>> = const { std::cell::RefCell::new(std::collections::BTreeSet::new()) });
            for fy in 83..93 {
                for fx in 106..118 {
                    let lit = w.seen2(fx, fy).is_some_and(|b| b & 2 != 0);
                    if lit && LIT.with(|l| l.borrow_mut().insert((fx, fy))) {
                        let near: Vec<String> = built
                            .sim
                            .units
                            .iter()
                            .filter(|u| u.owner == 1 && u.on_map)
                            .filter(|u| (u.pos.x - fx * 384).abs() < 4000 && (u.pos.y - fy * 384).abs() < 4000)
                            .map(|u| format!("1/{} ty{:?} ({}, {})", u.index, u.ty, u.pos.x, u.pos.y))
                            .collect();
                        let bnear: Vec<String> = built
                            .sim
                            .buildings
                            .iter()
                            .filter(|b| b.owner == 1)
                            .filter(|b| (b.pos.x - fx * 384).abs() < 6000 && (b.pos.y - fy * 384).abs() < 6000)
                            .map(|b| format!("1/{} ty{:?} ({}, {})", b.index, b.ty, b.pos.x, b.pos.y))
                            .collect();
                        eprintln!("PROBE lit ({fx}, {fy}) on {f}: {near:?} buildings {bnear:?}");
                    }
                }
            }
            for (j, m) in theirs.iter().enumerate() {
                if (m.to.0 / 384, m.to.1 / 384) == (110, 85) {
                    eprintln!("PROBE cellnode {f} {j} theirs {} ours {:?}", m.cost, ours.get(j).map(|o| (o.to, o.cost)));
                }
            }
            thread_local!(static PREV: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) });
            let cur: Vec<u8> = (0..w.height() * 2)
                .flat_map(|fy| (0..w.width() * 2).map(move |fx| (fx, fy)))
                .map(|(fx, fy)| w.seen2(fx, fy).unwrap_or(0) & 2)
                .collect();
            if f == 1170 {
                PREV.with(|p| {
                    let p = p.borrow();
                    let fw = w.width() * 2;
                    let new: Vec<(i32, i32)> = (0..cur.len())
                        .filter(|&i| cur[i] != 0 && p.get(i).copied().unwrap_or(0) == 0)
                        .map(|i| (i as i32 % fw, i as i32 / fw))
                        .collect();
                    eprintln!("PROBE new on 1170: {new:?}");
                });
                for b in built.sim.buildings.iter().filter(|b| b.owner == 1) {
                    eprintln!("PROBE b 1/{} ty{:?} ({}, {}) name {:?}", b.index, b.ty, b.pos.x, b.pos.y, b.ty.map(|t| { let bt = &built.sim.build_types[t]; (bt.ident, bt.x_size, bt.los) }));
                }
            }
            PREV.with(|p| *p.borrow_mut() = cur);
            if f == 3605 {
                let mut out = String::new();
                for fy in 0..w.height() * 2 {
                    for fx in 0..w.width() * 2 {
                        out += &format!("{fx} {fy} {}\n", w.seen2(fx, fy).unwrap_or(0));
                    }
                }
                std::fs::write(std::env::var("RON_PROBE_3631").unwrap(), out).unwrap();
                for cy in 40..45 {
                    let row: Vec<String> = (52..59)
                        .map(|cx| format!("{:?}/{:?}", w.owner(Cell::new(cx, cy)), w.region_of(Cell::new(cx, cy))))
                        .collect();
                    eprintln!("PROBE owner cy {cy}: {}", row.join(" "));
                }
            }
            if [0, 1, 2, 50, 200, 500, 1000, 2000, 3000, 3606].contains(&f) {
                eprintln!("PROBE grid at {f}");
                for fy in 80..91 {
                    let row: Vec<String> = (104..117)
                        .map(|fx| format!("{:>3}", w.seen2(fx, fy).map_or(-1, i32::from)))
                        .collect();
                    eprintln!("PROBE seen2 fy {fy}: {}", row.join(" "));
                }
                for (j, m) in theirs.iter().enumerate() {
                    if (m.to.0 / 384, m.to.1 / 384) == (110, 85) {
                        eprintln!("PROBE theirs node {j} {m:?} ours {:?}", ours.get(j));
                    }
                }
            }
        }
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
    // (110, 85)).
    pin_eq!(
        on_word,
        (3201, 3204),
        "the word's road nodes, ours and theirs"
    );
    pin_eq!(
        first.as_deref().map(|s| s.split(':').next().unwrap_or("")),
        Some("3606 at 126"),
        "the first node that parts"
    );
}
