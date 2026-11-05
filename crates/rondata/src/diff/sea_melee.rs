//! **The fourth melee: a sea fight on East Indies at the held Information
//! age, every seat a computer** (item 3847, `docs/GOLDEN.md` §153; the
//! shape is §155, the lever moved the map, and the lobby is §152's).
//! East Indies, `MAP_STYLE 18`, seed 12345, `--profile PLAYERS=2`: the idle
//! human Nubians (4) in seat 0, the French (10) as who 1 and the Russians
//! (13) as who 2 (`--seat-tribe 1=10 --seat-tribe 2=13`), all three at war
//! from frame 0 with no `war` line; `STARTING_TECHNOLOGY 7`,
//! `ENDING_TECHNOLOGY 7`, `STARTING_RESOURCES 7`.
//! `tools/gamelog/golden/melee-sea.cmd` grants each computer 100,000 of
//! every resource on 600 and 602 and spawns seven ships a side from 610,
//! two frames apart, at water cells of the strait between the two
//! computers' islands — `Fluyt` 332 and the `Aircraft Carrier` 351 among
//! them — a Minuteman a side on its shore, and the Nubian lines no lobby
//! fields to who 0 on the unowned island north of the strait, with a probe
//! of `parse_type`'s name column for the German Machine Gun (124).
//!
//! Captures: run1482, the `DUMP_ALL` start; run1483, the staged game to its
//! end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride of
//! 100, walked from run1482's initial state with the script staged by
//! [`Built::tick`] ([`crate::diff::setup::with_staged_script`]); run1483b,
//! the same script at `cover=1`, pinned in `blind::TRACES`.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// run1482: the lobby's `DUMP_ALL` start, 37 frames.
pub(crate) const MELEE_START: &str = "gamelog-run1482-eastindies-information-melee-start.txt";

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-sea.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-sea.cmd: {e}"))
}

/// The script's own line count: two `resource` lines, seven ship twins,
/// the Minuteman twin and the four lines on who 0.
pub(crate) const MELEE_STAGED: usize = 22;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3): run1482's start stood up with the script staged by the harness's
/// own tick, to frame `RON_MELEE_STAGE` (default 700), printing where
/// every `add` landed, whether its cell is water, and every line the
/// interpreter carried and did not act on.
#[test]
fn sea_melee_staging_walk() {
    let Some(inst) = install() else {
        return;
    };
    let Some(path) = dump(MELEE_START) else {
        eprintln!("skipping: no {MELEE_START} (set RON_GAMELOG_DIR)");
        return;
    };
    let script = melee_script();
    assert_eq!(
        script.lines().len(),
        MELEE_STAGED,
        "the melee's staged lines"
    );
    let to: i64 = std::env::var("RON_MELEE_STAGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(700);
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().expect("a start dump");
    let own = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    while built.sim.frame <= to {
        built.tick();
    }
    let st = built.staged.as_ref().expect("the script is staged");
    eprintln!(
        "melee staged to {to}: {} line(s) ran, {} unit(s), {} building(s); {} carried and not acted on",
        st.applied.ran,
        st.applied.units,
        st.applied.buildings,
        st.applied.skipped_total()
    );
    for ((word, why), k) in &st.applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    for line in &st.applied.landed {
        eprintln!("  {line}");
    }
    for u in built.sim.units.iter().filter(|u| u.alive()) {
        let Some(ty) = u.ty else { continue };
        if ty + 50 < 300 && ![67, 123, 124, 175, 192, 194].contains(&(ty + 50)) {
            continue;
        }
        let cell = u.pos.cell();
        eprintln!(
            "  {}/{} TI {} at ({}, {}) cell {:?} ocean {}",
            u.owner,
            u.index,
            ty + 50,
            u.pos.x,
            u.pos.y,
            cell,
            built.sim.world.is_ocean(cell)
        );
    }
    // Every line runs: the twenty-two name a type each (`machine_gun_german`
    // the generic 123, on its eleven characters — 124's `NAME` is 123's).
    assert_eq!(st.applied.skipped_total(), 0, "add refuses nothing");
    assert_eq!(st.applied.ran, MELEE_STAGED, "every line ran");
}

/// run1483: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1483-eastindies-information-melee-trace.txt",
    "rontrace-run1483.log",
);

pub(crate) use super::gunpowder_melee::Melee;

pub(crate) const SEA_MELEE: Melee = Melee {
    length: 2600,
    // Item 3847, first try: **677 of 2600**, the count and the sequence
    // together, 67 frames past the first `add` (610). Ours 13 draws
    // against 12: at index 4 ours spends `Unit::close+0xcb6` — a unit
    // dies in ours — where theirs spends `Farms::inc_time+0x1ae`. Ours
    // acted (`docs/GOLDEN.md` §153). Item 3849: **677 → 820** — the
    // leaders visited in the start permutation left it at 677; the sea
    // pieces' release rows (280, 289, 300) and the cruisers' and the
    // battleship's turret seats moved it. On 820 theirs spends 16 against
    // ours' 14: at index 4 theirs spends `Ammo::do_damage+0xc59` where
    // ours spends `Farms::inc_time+0x1ae` — theirs acted (§153.1): the
    // Royal Kushite Archer `0/17`'s arrow of 801 punctures the ground a
    // frame sooner (run1485). **820 → 1233** with piece 125's bays: ours
    // 15 against 14, at index 5 ours spends `Unit::close+0xcb6` where
    // theirs spends `Farms::inc_time+0x1ae` — ours acted. Item 3854:
    // **1233 → 1613** with the ship heal (`sim::heal`): run1485's carriers
    // heal a point on `(o + frame) % 20`, ours never did, and on 1201
    // `1/18` stood at 545 damage against 522 and `2/18` at 341 against
    // 318; now both agree on every block to 1201. On 1613 ours 12 against
    // 11: at index 6 ours spends `Unit::close+0xcb6` where theirs spends
    // `Farms::inc_time+0x1ae` — ours acted again (§153.2). Item 3855:
    // **1613 → 1980** (the count 1986) with piece 144 reading piece 143's
    // release rows (`pivot::SAME_AS`): the Camel Raider `0/14`'s rounds
    // leave from its turret, and run1486's block 1435, where theirs' `2/18`
    // took 6 from its round of 1430 in theirs alone, agrees. On 1980 both
    // spend 121: at index 53 ours spends `PathFinder::calc_road_cost+0x46`
    // where theirs spends `Unit::think_scout+0xaba` (§153.3). Item 3857:
    // **1980 → 2090** with the scout's sibling test on the lineage
    // (`FILTER_BASE_TYPE`, `sim::scout`'s `scout_unit_near`): on 1957 the
    // Fluyt `2/20` refuses cell (55,36), where the Aegis Cruiser `2/48`,
    // its lineage-mate under the Bark, is bound, and takes (57,50) as
    // theirs does; the fog the two Fluyts uncover then agrees, and on 1980
    // the cruiser's region scan accepts 51 cells as theirs does (run1487's
    // packet). On 2090 ours 13 against 14: at index 0 ours spends
    // `Unit::do_move+0xe84` where theirs spends `Guy::set_anim+0x97a <
    // Unit::move_step+0x823` (§153.4). Item 3860: **2090 → 2598** with
    // the spiral's centre test on a *started* building (`sim::ai_place`'s
    // `cell_has_centre`, the cell's `0x4000` written by `Wall::start`): on
    // 1985 the French dock of city 1 takes cell (19, 48), beside the
    // unstarted Shipyard `1/2033`, at 4255, as theirs does (run1487's
    // packet), and `1/2036` goes to `1/6` at (15360, 37248). On 2598 ours
    // 10 against 12: at index 4 ours spends `Guy::set_anim+0x97a <
    // Guy::inc_time+0x271` where theirs spends `Guy::init_real+0x52` — a
    // birth in theirs (§153.5). Item 3862: **2598 → 2600, the game
    // whole**, with `Object::close`'s contents arm on a unit
    // ([`sim::Sim::kill_contents`]): the Freighter `1/54` sinks with the
    // Elite Special Forces `1/0` aboard, and theirs closes `1/0` with it
    // (no draw, `dtype` 0) on 2584; ours kept it alive in a dead boat,
    // and `1/56` (uid 102, an Elite Special Forces in `1/0`'s group 68,
    // born on 2598 at `Guy::init_real`) was born in theirs alone. Now both
    // (§153.6); which of `1/0`'s holdings the birth waited on is not read.
    count: 2600,
    sequence: 2600,
};

/// **The melee's closing state, as the `Melee:` line says it** (Loop
/// 3842): the records its closing-state test reads off at the close — the
/// sum of [`endpoint::EndpointResult::counts`]' seven — and the `<n> off`
/// suffix of this row's part of the handoff's `Melee:` line
/// (`floors::ai_line_closes`). A draw stream whole to the game's last frame
/// with a close that is not reads `w2600 of 2,600 1 off`, not the closed
/// melees' line; a close that moves re-pins this and the line together.
///
/// Item 3849: 448 → 474, with the personalities on their own seats
/// ([`crate::diff::setup::leader_visit_order`]) — a game parted at 677
/// plays on differently once the computers' personalities are right; then
/// 474 → 466 with the word at 820, back to 474 with piece 282's bays, and
/// 352 with piece 125's (the word at 1233). Item 3854: 352 → 247 with the
/// ship heal (the word at 1613). Item 3855: 247 → 187 with piece 144's
/// release (the word at 1980). Item 3857: 187 → 157 with the scout's
/// lineage test (the word at 2090). Item 3860: 157 → 2 with the spiral's
/// started-centre test (the word at 2598). Item 3862: 2 → 0 with the
/// sunk transport's rider closed with it (the game whole): `1/56` linked,
/// `1/0` no longer extra, 137 units compared.
pub(crate) const SEA_MELEE_CLOSING_OFF: usize = 0;

/// The melee's word: the lower of the two partings.
pub(crate) const SEA_MELEE_WORD: i64 = if SEA_MELEE.count < SEA_MELEE.sequence {
    SEA_MELEE.count
} else {
    SEA_MELEE.sequence
};

/// **Where the fourth melee's draw stream first parts** (item 3847): the
/// harness walks run1482's start with the script staged and compares every
/// frame's draws against run1483's trace, the count and the sequence.
#[test]
fn sea_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &SEA_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "SEA_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// The word's window on run1483's stride: the block before the word to
/// 250 frames past it (item 3847: the word 677, so 601..927, blocks 601,
/// 701, 801 and 901). Item 3849 moved the word to 1233: 1201..1483,
/// blocks 1201, 1301 and 1401. Item 3854 moved it to 1613: 1601..1863,
/// blocks 1601, 1701 and 1801. Item 3855 moved it to 1980: 1901..2230,
/// blocks 1901, 2001, 2101 and 2201. Item 3857 moved it to 2090:
/// 2001..2340, blocks 2001, 2101, 2201 and 2301. Item 3860 moved it to
/// 2598: 2501..2600, block 2501 alone — 250 past the word is beyond the
/// game's last frame, and the closing block 2601 is
/// [`sea_melee_run1483_s_closing_state`]'s. Item 3862 closes the game: the
/// window is the block before the word, 2501, through the closing block
/// 2601, as the third melee's.
pub(crate) const WIDENING_SEA_MELEE: (i64, i64) = (2501, 2601);

/// The word's window, widened whole on run1483's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn sea_melee_closing_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1483",
            &[(MELEE_LONG.0, WIDENING_SEA_MELEE.0)],
            WIDENING_SEA_MELEE,
            1,
            &[2601],
            true,
        )
    })
}

/// **The fourth melee's word window, widened whole** (item 3847): every
/// dumped record on run1483's stride blocks over
/// [`WIDENING_SEA_MELEE`], both directions, the rounds too.
/// `RON_FIRSTS=1` prints every row.
#[test]
fn sea_melee_run1483_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sea_melee_closing_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        for ((who, o, what), (f, r)) in &w.rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    // WIDEN_NOTE
    // Item 3855, the word at 1980: block 1901 parts on the standing
    // families alone; on 2001 the Russian Fluyt `2/20` (`TI` 332, sea) is
    // first, its `EXPLORETOORDER` bound for (44280, 38904) in theirs and
    // (42744, 28152) in ours, beside the Aegis Cruiser `2/48`'s and the
    // French Citizens `1/6`' and `1/11`'s orders. Item 3857, the word at
    // 2090 (2001..2340): the Fluyt and the cruiser agree on 2001, and what
    // parts there is the standing families and `1/6`' and `1/11`'s order
    // lists (1 against 3, 4 against 2), first parted on run1487's 1986.
    // Item 3860, the word at 2598 (2501..2600): block 2501 parts on the
    // standing families and the Aegis Cruiser `2/48`'s rounds alone, their
    // `ez` 47 against 41 (parked 3855b's family); the Citizens agree.
    // Item 3862, the game whole (2501..2601, the closing block in): on
    // 2601 `1/56` (uid 102, TI 77, Elite Special Forces, land) stands at
    // (22145, 41970) in both, and `1/0` is gone from both.
    pin_eq!(w.blocks, 2, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 192, "run1483's word window");
    pin_eq!(w.rounds_read, (8, 8), "the rounds read");
    pin_eq!(w.rounds.len(), 4, "the rounds that part");
}

/// run1484: run1483's game with a window at every frame of 660..680 and
/// a packet at 676 (item 3849, `docs/GOLDEN.md` §153.1): the rounds of
/// the word's frame, both submarines and both cruisers frame by frame.
pub(crate) const MELEE_WINDOW: (&str, &str) = (
    "gamelog-run1484-eastindies-information-melee-window-660-680.txt",
    "rontrace-run1484.log",
);

/// run1484's window, every block.
pub(crate) const WIDENING_SEA_MELEE_WINDOW: (i64, i64) = (660, 680);

/// run1484's window widened whole, the rounds compared — the coverage
/// driver's slice too.
pub(crate) fn sea_melee_run1484_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1484",
            &[(MELEE_WINDOW.0, WIDENING_SEA_MELEE_WINDOW.0)],
            WIDENING_SEA_MELEE_WINDOW,
            1,
            &[],
            true,
        )
    })
}

/// **run1484's window, widened whole** (item 3849): every dumped record
/// on every block of [`WIDENING_SEA_MELEE_WINDOW`], both directions, the
/// rounds too — the word 677's value diff. `RON_FIRSTS=1` prints every
/// row.
#[test]
fn sea_melee_run1484_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sea_melee_run1484_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        eprintln!("  missing {:?}", w.missing);
        eprintln!("  rounds read {:?}", w.rounds_read);
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
        for (b, rows) in &w.standing {
            for ((who, o, what), r) in rows {
                eprintln!("  standing {b} {who}/{o} {what}: {r}");
            }
        }
    }
    pin_eq!(w.blocks, 20, "every block in the window");
    pin_eq!(w.firsts.len(), 90, "run1484's window");
    pin_eq!(w.rounds_read, (157, 157), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// run1486: run1483's game with a window at every frame of 1400..1502
/// (item 3854, `docs/GOLDEN.md` §153.2): the frames where the Aircraft
/// Carrier `2/18`, which dies in ours on the word's frame 1613, first
/// parts.
pub(crate) const MELEE_WINDOW_1400: (&str, &str) = (
    "gamelog-run1486-eastindies-information-melee-window-1400-1502.txt",
    "rontrace-run1486.log",
);

/// run1486's window, every block.
pub(crate) const WIDENING_SEA_MELEE_WINDOW_1400: (i64, i64) = (1400, 1502);

/// run1486's window widened whole, the rounds compared — the coverage
/// driver's slice too.
pub(crate) fn sea_melee_run1486_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1486",
            &[(MELEE_WINDOW_1400.0, WIDENING_SEA_MELEE_WINDOW_1400.0)],
            WIDENING_SEA_MELEE_WINDOW_1400,
            1,
            &[],
            true,
        )
    })
}

/// **run1486's window, widened whole** (item 3854): every dumped record
/// on every block of [`WIDENING_SEA_MELEE_WINDOW_1400`], both directions,
/// the rounds too — the value diff the word 1613 walks back to. The first
/// unit row is block 1435: theirs' `2/18` takes 6 damage from the Camel
/// Corps `0/14` (500 against ours' 494, `damage_frame` 1434), whose round
/// of block 1430 leaves (30461, 27533, 298) and flies 6 frames in theirs
/// against the figure's square (30456, 27384, 17) and 7 in ours — parked
/// 3849b's release. `RON_FIRSTS=1` prints every row.
#[test]
fn sea_melee_run1486_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sea_melee_run1486_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
    }
    let carrier: Vec<_> = w
        .firsts
        .iter()
        .filter(|((who, o, what), _)| *who == 2 && *o == 18 && what == "hits:damage")
        .map(|(_, (f, r))| (*f, r.clone()))
        .collect();
    // Item 3855: the carrier's damage agrees on every block, and what parts
    // is the standing families, the `ez` 25 against 24 of `2/14`'s and
    // `2/16`'s rounds, and 3849a's dead targets (118 → 111, 46 → 24).
    pin_eq!(carrier, vec![], "the carrier's first parted damage");
    pin_eq!(w.blocks, 102, "every block in the window");
    pin_eq!(w.firsts.len(), 111, "run1486's window");
    pin_eq!(w.rounds_read, (381, 381), "the rounds read");
    pin_eq!(w.rounds.len(), 24, "the rounds that part");
}

/// run1487: run1483's game with a window at every frame of 1975..2001 and
/// a packet at logger frame 1980 (item 3857, `docs/GOLDEN.md` §153.4): the
/// frames round 1980, where the Aegis Cruiser `2/48`'s region scan drew 49
/// cells in ours against 51, and where nothing of run1483 dumps.
pub(crate) const MELEE_WINDOW_1975: (&str, &str) = (
    "gamelog-run1487-eastindies-information-melee-window-1975-2001.txt",
    "rontrace-run1487.log",
);

/// run1487's window, every block.
pub(crate) const WIDENING_SEA_MELEE_WINDOW_1975: (i64, i64) = (1975, 2001);

/// run1487's window widened whole, the rounds compared — the coverage
/// driver's slice too.
pub(crate) fn sea_melee_run1487_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1487",
            &[(MELEE_WINDOW_1975.0, WIDENING_SEA_MELEE_WINDOW_1975.0)],
            WIDENING_SEA_MELEE_WINDOW_1975,
            1,
            &[],
            true,
        )
    })
}

/// **run1487's window, widened whole** (item 3857): every dumped record on
/// every block of [`WIDENING_SEA_MELEE_WINDOW_1975`], both directions, the
/// rounds too — the value diff the word 1980 walks back to. Before the
/// lineage test the Fluyt `2/20` stood at (42941, 31979) bound for
/// (42720, 28128) on 1975 in ours, and at (43377, 33404) bound for
/// (44280, 38904) in theirs: its explore point of 1957. Before item
/// 3860's started-centre test the French Citizens' order lists parted on
/// 1986 — `1/6` 1 against 3, `1/11` 4 against 2 — city 1's dock `1/2036`
/// sited at (14400, 41664) for `1/11` in ours and at (15360, 37248) for
/// `1/6` in theirs. `RON_FIRSTS=1` prints every row.
#[test]
fn sea_melee_run1487_s_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = sea_melee_run1487_window() else {
        return;
    };
    if std::env::var("RON_FIRSTS").is_ok() {
        let mut rows: Vec<_> = w.firsts.iter().collect();
        rows.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rows {
            eprintln!("  first {f} {who}/{o} {what}: {r}");
        }
        let mut rounds: Vec<_> = w.rounds.iter().collect();
        rounds.sort_by_key(|(k, (f, _))| (*f, (*k).clone()));
        for ((who, o, what), (f, r)) in rounds {
            eprintln!("  round {f} {who}/{o} {what}: {r}");
        }
    }
    let scouts: Vec<_> = w
        .firsts
        .iter()
        .filter(|((who, o, _), _)| *who == 2 && (*o == 20 || *o == 48))
        .map(|((_, o, what), (f, _))| (*f, *o, what.clone()))
        .collect();
    pin_eq!(scouts, vec![], "the two scouts' first parted rows");
    pin_eq!(w.blocks, 26, "every block in the window");
    pin_eq!(w.firsts.len(), 144, "run1487's window");
    pin_eq!(w.rounds_read, (121, 121), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// **Every seat's leader on the start dump's frames** (item 3847, the
/// shape item 3791 gave the first two, `docs/GOLDEN.md` §151): run1482's frames 1 and 37 print all three
/// seats' `LEADERDATA` whole. run1483's only other leader records are its
/// closing dump at 2601, compared too.
#[test]
fn sea_melee_s_leaders_are_compared_on_every_seat() {
    let _pins = Pins::hold();
    let Some(w) = crate::diff::gunpowder_melee::seat_walk(
        melee_script,
        MELEE_START,
        MELEE_LONG,
        &[MELEE_START],
        true,
    ) else {
        return;
    };
    w.report("run1482+closing");
    pin_eq!(w.players, 3, "the seats the lobby seats");
    pin_eq!(
        w.records.clone(),
        vec![3, 3, 3],
        "leader records compared, by seat"
    );
    // Item 3847, first try: the standing family on both start frames and
    // every seat (12, 9 and 9 keys), and 210 keys that part. 26 are the two
    // computers' `PERSONALITY` blocks from frame 1, all thirteen fields a
    // seat, **swapped between who 1 and who 2** (`air` ours −1 and 1
    // against theirs' 1 and −1, `naval` −1 and 0 against 0 and −1, …):
    // `setup::build_sim` handed the checksum trace's `random_personality`
    // brackets out in seat order, and each lands on its far end, so the
    // rolls were the original's on the other seat (parked 3847a). The
    // other 184 are the closing dump's at 2601, a game parted since 677.
    //
    // Item 3849: 210 → 203 and the 26 → 0. `Setup::build_game` visits the
    // leaders in its start permutation, here 3, 2, 5, 4, 7, 6, 1, 0 —
    // who 2 first, the human last — and the brackets are now handed out
    // in that order ([`crate::diff::setup::leader_visit_order`]). The
    // standing family is 12, 10 and 7; the 203 are the closing dump's.
    pin_eq!(w.standing(), vec![12, 9, 6], "standing keys, by seat");
    pin_eq!(w.rest().len(), 3, "the keys that part, by seat");
    pin_eq!(
        w.rest()
            .iter()
            .filter(|k| k.contains("PERSONALITY") && k.contains(" 1: "))
            .count(),
        0,
        "the personality fields parted from frame 1"
    );
}

/// **The melee's closing state** (item 3847): run1483's closing block, the
/// whole map's units, walked from run1482's start with the script staged.
#[test]
fn sea_melee_run1483_s_closing_state() {
    let _pins = Pins::hold();
    let Some(inst) = install() else {
        return;
    };
    let (Some(path), Some(start), Some(tr)) =
        (dump(MELEE_LONG.0), dump(MELEE_START), trace(MELEE_LONG.1))
    else {
        eprintln!("skipping: no {} (set RON_GAMELOG_DIR)", MELEE_LONG.0);
        return;
    };
    let Some(e) = crate::diff::setup::with_staged_script(melee_script(), || {
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
            "Sea melee closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
            e.frame,
            e.compared,
            e.counts(),
            e.torn,
            e.off,
            e.unlinked,
            e.extra
        );
        Some(e)
    }) else {
        return;
    };
    pin_eq!(e.frame, 2601, "the closing block");
    // Item 3847, first try: 129 units compared at a close 1,924 frames
    // past the word, 448 records off — 115 units off, 8 unlinked, 4 extra
    // (who 1's `0`, `54`, `57` and `58` in ours alone), 7 buildings
    // unlinked and 289 diverged, 25 cities diverged. Item 3849, the
    // personalities on their own seats: 130 compared, 474 off — 115 units
    // off, 7 unlinked, 2 extra (who 1's `0` and `54`), 6 buildings
    // unlinked and 310 diverged, 34 cities diverged.
    pin_eq!(e.compared, 137, "units compared");
    pin_eq!(e.counts(), [0, 0, 0, 0, 0, 0, 0], "the closing counts");
    pin_eq!(
        e.counts().iter().sum::<usize>(),
        SEA_MELEE_CLOSING_OFF,
        "the closing records off, the `Melee:` line's suffix"
    );
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}
