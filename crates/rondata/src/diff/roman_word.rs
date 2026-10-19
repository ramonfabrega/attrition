//! **The battery's third word** (DECISIONS 72, 83; item 3314): the
//! held-out battery's lowest game, promoted by item 2904's and 3107's
//! recipe. East Indies (`MAP_STYLE 18`), human Nubians (4) against **the
//! Romans (6)** at Difficulty 2, seed 12345 — `docs/AI.md` §243's fifth
//! lobby, scored by its first try at 576 of 18,620 and promoted out of the
//! battery here, so it is a scored lobby from this item
//! (`battery::SCORED_LOBBIES`) and no longer held out. Captures: run899 (a
//! `DUMP_ALL` start, blocks 0..36) and run899b (the draw stream to the
//! game's end, `cover=0`, its setup and closing blocks only); the word's
//! windows are run956's and run956b's (`docs/AI.md` §263). `GreatSaharaSpanishTier3` parts
//! on 576 too and stays held out: it is never debugged against.

use super::battery::{self, Lobby};
use super::testkit::*;
use super::*;
use crate::testenv::dump;

pub(crate) const ROMAN: Lobby = Lobby {
    name: "EastIndiesRomanTier2",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 2,
    start: "gamelog-run899-eastindies-roman-diff2-start.txt",
    long: (
        "gamelog-run899b-eastindies-roman-diff2-24k-trace.txt",
        "rontrace-run899b.log",
    ),
    length: 18620,
    // Item 2975: the battery's first try, count and sequence both at 576
    // (ours 28 draws, theirs 89, index 0: ours `Animal::think_bird+0x82`,
    // theirs `Leader::produce_building+0xc99`). Item 3314 promoted it to
    // the AI word (`docs/AI.md` §263). **Item 3314 moved it 576 →
    // 18026** with `get_cost`'s fort arm (`ROMAN_FORT_COST`, 25 % off a
    // fort or a Tower-line building for a Roman leader, `00665e55`..), which
    // this crate never carried: who=1's Tower, bought on frame 1, cost ours
    // a quarter more, so run956's block 570 had its bucket timber 46 and
    // wealth 31 against theirs' 62 and 39, and frame 576's fourth Farm (52
    // timber) was refused in ours and bought in theirs (`1/2007`, its
    // `Farms::add` draws, then four `place_city_with_cost` rounds). Both
    // read 62 and 39 now, and the Farm is on both sides on block 577. At
    // 18026 ours 199 draws, theirs 200, index 192: ours `Guy::set_anim+0x97a
    // < Guy::inc_time+0x271`, theirs `PathFinder::calc_road_cost+0x46`,
    // past run956's last block (826). **And 18026 → 18620, the game's
    // last frame**, with `place_roads`' teardown (`docs/ROADS.md` §13):
    // the Library `0/2005`, dead on 18011, took its ring road with it in
    // theirs, and on 18026 who=1's caravan search priced tile (28, 40) on
    // road here and off it there (run956b, run899b's call window).
    count: 18620,
    sequence: 18620,
};

/// The lobby's word as the handoff's `Battery word:` line and `AI_WORDS`
/// carry it.
pub(crate) const ROMAN_WORD: i64 = if ROMAN.count < ROMAN.sequence {
    ROMAN.count
} else {
    ROMAN.sequence
};

/// **The lobby's first parting**, walked from its own start.
#[test]
fn roman_word_first_parting() {
    let _pins = Pins::hold();
    let Some(w) = third::walk_from(ROMAN.start, ROMAN.long) else {
        return;
    };
    eprintln!(
        "ROMAN {}: count {} sequence {} of {}; {}",
        ROMAN.name, w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.difficulty, ROMAN.difficulty, "the lobby read back");
    pin_eq!(w.last, ROMAN.length, "the game's last frame");
    pin_eq!(w.count, ROMAN.count, "the draw count's first parting");
    pin_eq!(
        w.sequence,
        ROMAN.sequence,
        "the draw sequence's first parting"
    );
}

/// The lobby is a scored one and none of the battery's; its long and its
/// start say their lobby, as the battery's identity test read them.
#[test]
fn roman_word_is_no_other_lobby() {
    let p = &ROMAN;
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

/// run956: the word 576's window, blocks 570..826 at the long's detail
/// (item 3314, run886's recipe); run899 dumps blocks 0..36 and run899b its
/// setup and closing blocks only.
pub(crate) const RUN956: &str = "gamelog-run956-eastindies-roman-diff2-window-570-827.txt";

/// The window: block 570 through 826; the word 576's own block is 577.
pub(crate) const WIDENING_ROMAN_FRAME_576: (i64, i64) = (570, 826);

pub(crate) fn roman_frame_576_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN.start],
        true,
        ROMAN.long,
        "run956",
        &[(RUN956, WIDENING_ROMAN_FRAME_576.0)],
        WIDENING_ROMAN_FRAME_576,
        1,
        &[577],
        true,
    )
}

#[test]
fn run956_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_frame_576_window() else {
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
    pin_eq!(w.firsts.len(), 29, "initial run956 baseline");
    // Item 3314: 239 keys part on blocks 570..826, the word's own block
    // being 577 (frame 576: ours 28 draws, theirs 89, index 0 — theirs one
    // `place_building_with_cost` of a Farm, `produce_building+0xc99` ×39 and
    // `+0x1805` ×4, its `Farms::add` pair, then four `place_city_with_cost`
    // rounds; ours none of them). Standing from 570: who=1's
    // `bucket[timber]` 46/62 and `bucket[wealth]` 31/39 (the Tower of frame
    // 1 priced without the Romans' quarter off); on 577 theirs holds the
    // Farm `1/2007` and ours none (`build:unlinked`), `1/2000`'s
    // `city_down` 2007, the Citizen `1/2`'s order on it, the city's
    // `filled`/`gatherers`/`space`, who=1's `script_step` 9/10; from 584
    // the timber `income`, `resources`, `leftover` and `gather_stamp`, then
    // the walkers both sides route around the Farm.
    //
    // **Item 3314: 239 → 29**, with `get_cost`'s fort arm
    // (`cost::Modifiers::fort`). The word's value diff, block 570: who=1's
    // bucket timber 62 and wealth 39 on both (ours 46 and 31 before); block
    // 577: the Farm `1/2007` on both. What stands is the control's 29 from
    // 570 — `SITE[n].reg`, `form`, the groups' `held`, the pool and both
    // leaders' `aggression` — and who=1's `SITE[2].reg` on 576 as its
    // sites fill. The word moved to 18026, past the window's last block.
}

/// run956b: the old word 18026's window, blocks 18020..18276 at run956's
/// detail (item 3314); run956 ends on block 826 and run899b dumps its setup
/// and closing blocks only.
pub(crate) const RUN956B: &str = "gamelog-run956b-eastindies-roman-diff2-window-18020-18277.txt";

/// The window: block 18020 through 18276; the word 18026's own block was
/// 18027.
pub(crate) const WIDENING_ROMAN_FRAME_18026: (i64, i64) = (18020, 18276);

pub(crate) fn roman_frame_18026_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN.start],
        true,
        ROMAN.long,
        "run956b",
        &[(RUN956B, WIDENING_ROMAN_FRAME_18026.0)],
        WIDENING_ROMAN_FRAME_18026,
        1,
        &[18027],
        true,
    )
}

#[test]
fn run956b_s_word_frame_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_frame_18026_window() else {
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
    pin_eq!(w.firsts.len(), 51, "keys that part in the window");
    // Item 3314: 51 keys, every one standing from 18020 — the control's set
    // (`SITE[n].reg`, `form`, groups' `held`, the pool, `aggression`) and
    // who=1's `1/22`..`1/37` `g.angle[0]`, ours `0x55555555` against 0.
    // Before the Library's ring went with it (`docs/ROADS.md` §13), `1/64`'s
    // move offsets parted from 18028 too.
}

/// **The word 18026's caravan road search, node for node** (item 3314,
/// `docs/ROADS.md` §13): theirs' `PathFinder::astar_caravan_road` spent
/// 194 `calc_road_cost` draws on 18026 against ours' 192, and the first
/// node to part was 32, the step onto tile (28, 40): ours priced it on
/// road, theirs off. The road was the Library `0/2005`'s ring, which its
/// death on 18011 takes away in theirs (`mask_me`'s unmask ends in
/// `place_roads(…, 0, REGEN_SIMPLE)`) and nothing took away here.
/// run899b's call window (`--callwin 0 24000`) holds every
/// `valid_roadcoord`/`calc_road_cost` pair.
#[test]
fn run899b_s_road_search_on_18026_holds_node_for_node() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(sib), Some(tr)) =
        (dump(ROMAN.long.0), dump(ROMAN.start), trace(ROMAN.long.1))
    else {
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
    // Every road search to the word and past it, node for node.
    let mut first: Option<String> = None;
    let mut searched = 0;
    let mut on_word = (0, 0);
    while built.sim.frame <= 18030 {
        let f = built.sim.frame;
        built.sim.road_marks.clear();
        built.tick();
        let ours = std::mem::take(&mut built.sim.road_marks);
        let theirs = tr.road_nodes(f);
        if f == 18026 {
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
        }
    }
    eprintln!("ROAD searched {searched}; first parting {first:?}; on 18026 {on_word:?}");
    pin_eq!(
        on_word,
        (194, 194),
        "the word's road nodes, ours and theirs"
    );
    pin_eq!(searched, 107, "frames with a road search to 18030");
    pin_eq!(first, None, "the first node that parts");
}

/// run956c: the gap from run956b's last block to the game's end, blocks
/// 18277..18621 at run956's detail (item 3314), to find where who=0's
/// Village `0/2000` takes its last point in theirs and not in ours.
pub(crate) const RUN956C: &str = "gamelog-run956c-eastindies-roman-diff2-window-18277-18622.txt";

/// The window: block 18277 through the closing block 18621.
pub(crate) const WIDENING_ROMAN_GAP_956C: (i64, i64) = (18277, 18621);

pub(crate) fn roman_gap_956c_window() -> Option<harness::tests::Widened> {
    harness::tests::widen_on_siblings(
        &[ROMAN.start],
        true,
        ROMAN.long,
        "run956c",
        &[(RUN956C, WIDENING_ROMAN_GAP_956C.0)],
        WIDENING_ROMAN_GAP_956C,
        1,
        &[18621],
        true,
    )
}

#[test]
fn run956c_s_gap_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = roman_gap_956c_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
    }
    pin_eq!(w.blocks, 344, "every captured block");
    pin_eq!(w.firsts.len(), 51, "keys that part in the window");
    // Item 3314: run956b's 51 standing keys, and one more — `0/2000`'s
    // `damage` on block 18369, ours 868 against theirs 869, from tick
    // 18368. **Item 3322: the second point is a decoy's round.** Two rounds
    // land on the Village on 18368 (run959's packet: `1/53`'s at t 8 of 9,
    // `1/110`'s at 7 of 8), and `1/110` is a Horse Archers **decoy**
    // (`unit_masks` `0x851007`, `mana_burn` 2499 on 18366) that closed on
    // 18367. `Unit::close@0060ee50:603` clears the decoy bit, so its round
    // strikes for real, 1.0 beside `1/53`'s 1.0; ours kept `decoy` on the
    // closed slot and `do_damage`'s decoy return swallowed it. With
    // `Sim::clear_decoy_at_close`, the window holds the 51 standing keys.
    // (`1/53`'s `up` 86 on 18367 is the collision chain's `1/86`, a
    // Legion, not the Senator — the previous item's hypothesis.)
}

/// The closing state's window: run899b's closing block 18621 alone, the
/// game's last frame 18620 writing it (item 3314).
pub(crate) const WIDENING_ROMAN_CLOSING: (i64, i64) = (18619, 18622);

/// **The lobby's closing state** (item 3314, DECISIONS 55): the draws agree
/// to the game's last frame, 18620, once the Library's ring road goes with
/// it (`docs/ROADS.md` §13), and run899b — the draw stream's own capture,
/// `cover=0` — prints the closing block 18621 whole. Walked to it and
/// compared whole, as the Germans' `run643_german_closing_state` is.
#[test]
fn run899b_roman_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = crate::testenv::install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(ROMAN.long.0), dump(ROMAN.start), trace(ROMAN.long.1))
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
        "Roman closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.frame, 18621, "the closing block");
    // Item 3314, the first walk: 110 units compared on block 18621, none
    // apart, no closing field parted — and six buildings and a city
    // unlinked (`[0, 0, 0, 6, 0, 1, 0]`): theirs' `1/2040`..`1/2045` and
    // city `1/2040` are who=0's Village `0/2000` and its five buildings,
    // captured on the game's last frame, where ours' Village stood at
    // damage 1199 of 1200 (level 5; a city centre is taken at 6,
    // `check_capture_eligible@0062d1d0`). Item 3322: the missing point was
    // the decoy `1/110`'s last round on 18368 (`clear_decoy_at_close`,
    // `run956c_s_gap_is_widened_whole`); with it ours captures the Village
    // on 18620 too, and the closing block is whole: 110 units, nothing
    // apart, nothing unlinked, no closing field parted.
    pin_eq!(e.compared, 110, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
}
