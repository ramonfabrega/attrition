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
    // ours spends `Farms::inc_time+0x1ae` — theirs acted (§153.1).
    count: 820,
    sequence: 820,
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
/// 474 → 466 with the word at 820, and back to 474 with piece 282's bays.
pub(crate) const SEA_MELEE_CLOSING_OFF: usize = 474;

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
/// 701, 801 and 901).
pub(crate) const WIDENING_SEA_MELEE: (i64, i64) = (601, 927);

/// The word's window, widened whole on run1483's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn sea_melee_word_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1483",
            &[(MELEE_LONG.0, WIDENING_SEA_MELEE.0)],
            WIDENING_SEA_MELEE,
            1,
            &[],
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
    let Some(w) = sea_melee_word_window() else {
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
    pin_eq!(w.blocks, 4, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 131, "run1483's word window");
    pin_eq!(w.rounds_read, (15, 17), "the rounds read");
    pin_eq!(w.rounds.len(), 14, "the rounds that part");
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
    pin_eq!(w.standing(), vec![12, 10, 6], "standing keys, by seat");
    pin_eq!(w.rest().len(), 207, "the keys that part, by seat");
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
    pin_eq!(e.compared, 135, "units compared");
    pin_eq!(e.counts(), [120, 2, 5, 4, 308, 0, 35], "the closing counts");
    pin_eq!(
        e.counts().iter().sum::<usize>(),
        SEA_MELEE_CLOSING_OFF,
        "the closing records off, the `Melee:` line's suffix"
    );
    pin_eq!(
        e.extra.clone(),
        vec![(1, 0), (1, 54), (1, 58), (1, 57), (1, 59)],
        "the closing extras"
    );
}
