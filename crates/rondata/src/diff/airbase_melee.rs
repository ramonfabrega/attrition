//! **The fifth melee: the fourth's lobby with an Airbase a computer**
//! (item 3868, `docs/GOLDEN.md` §156; the shape is §155 as the sixty-sixth
//! pass amended it — a chapter that moves no lever stages the trainer its
//! lines need — and the staging is §154.3's Proposal A without the third
//! seat). run1482's lobby — East Indies, `MAP_STYLE 18`, seed 12345,
//! `--profile PLAYERS=2`, the idle human Nubians (4) in seat 0, the French
//! (10) as who 1 and the Russians (13) as who 2, `STARTING_TECHNOLOGY 7`,
//! `ENDING_TECHNOLOGY 7`, `STARTING_RESOURCES 7` — with
//! `tools/gamelog/golden/melee-airbase.cmd`: `melee-sea.cmd`'s lines and,
//! before its first unit `add` at 610, `603 add airbase who=1 95,193` and
//! `605 add airbase who=2 220,107`, each inside its own capital's city, so
//! `Wall::activate`'s call takes `Wall::increment_stats`' active arm and
//! files the building in `reg_buildings` and `mil_trainers` (`docs/AI.md`
//! §29). What should stand: the Jet Fighter (`MIRAGE`, 296) on who 1 by
//! block 1101 and the Attack Helicopter (`HIND`, 312) on who 2 by 2601.
//!
//! Captures: run1482, the fourth melee's `DUMP_ALL` start, which this one
//! shares (the two `add` lines are after it); run1492, the staged game to
//! its end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride
//! of 100; run1492b, the same script at `cover=1`.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) use super::sea_melee::MELEE_START;

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-airbase.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-airbase.cmd: {e}"))
}

/// The script's own line count: the fourth melee's twenty-two and the two
/// Airbases.
pub(crate) const MELEE_STAGED: usize = 24;

/// The two Airbases' centre tiles, as the script writes them.
const AIRBASES: [(sim::Player, i32, i32); 2] = [(1, 95, 193), (2, 220, 107)];

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3, §156): run1482's start stood up with the script staged by the
/// harness's own tick, to frame `RON_MELEE_STAGE` (default 700). Before
/// 603 each Airbase's site is checked clear and inside its owner's
/// capital city; after, each Airbase stands active, in that city, counted
/// by `num_buildings_of` and on its owner's `mil_trainers`. Every
/// aircraft standing at the end is printed by `TypeIndex`.
#[test]
fn airbase_melee_staging_walk() {
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
    let ab = built
        .sim
        .build_types
        .iter()
        .position(|t| t.ident == sim::build::Ident::Airbase)
        .expect("an Airbase type");
    while built.sim.frame < 603 {
        built.tick();
    }
    // The sites, on the frame the first line runs: clear, and the capital's.
    for &(who, tx, ty) in &AIRBASES {
        let s = &built.sim;
        let at = s.snap_center_placed(
            ab,
            sim::Pos::new(tx * 0xc0 + 0x60, ty * 0xc0 + 0x60),
            Some(who),
        );
        let capital = s
            .cities
            .iter()
            .position(|c| c.alive && c.owner == who && c.capital);
        assert_eq!(
            s.blocked_site(Some(who), ab, at, None),
            sim::place::Blocked::Clear,
            "who {who}'s Airbase site ({tx}, {ty}) is clear on {}",
            s.frame
        );
        assert_eq!(
            s.find_city_at(who, at, Some(ab)),
            capital,
            "who {who}'s Airbase site is in its capital's city"
        );
    }
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
    let s = &built.sim;
    for (b, bd) in s.buildings.iter().enumerate() {
        if !bd.alive || bd.ty != Some(ab) {
            continue;
        }
        eprintln!(
            "  Airbase {}/{} at ({}, {}) active {} city {:?} mil_trainer {} queue {:?}",
            bd.owner,
            bd.index,
            bd.pos.x,
            bd.pos.y,
            bd.active,
            bd.city,
            s.ai[bd.owner as usize].mil_trainers.contains(&b),
            bd.queue.items.iter().map(|i| i.ty).collect::<Vec<_>>()
        );
    }
    for u in s.units.iter().filter(|u| u.alive()) {
        let Some(ty) = u.ty else { continue };
        if !(290..=315).contains(&(ty + 50)) {
            continue;
        }
        eprintln!(
            "  aircraft {}/{} TI {} at ({}, {})",
            u.owner,
            u.index,
            ty + 50,
            u.pos.x,
            u.pos.y
        );
    }
    assert_eq!(st.applied.skipped_total(), 0, "add refuses nothing");
    assert_eq!(st.applied.ran, MELEE_STAGED, "every line ran");
    assert_eq!(st.applied.buildings, 2, "both Airbases placed");
    for &(who, _, _) in &AIRBASES {
        let filed = s.buildings.iter().enumerate().filter(|(b, bd)| {
            bd.alive
                && bd.owner == who
                && bd.ty == Some(ab)
                && bd.active
                && bd.city.is_some()
                && s.ai[who as usize].mil_trainers.contains(b)
        });
        assert_eq!(
            filed.count(),
            1,
            "who {who}'s Airbase is active, in a city, a trainer"
        );
        assert_eq!(
            s.num_buildings_of(who, ab),
            1,
            "who {who}'s Airbase counted"
        );
    }
}

/// run1492: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1492-eastindies-airbase-melee-trace.txt",
    "rontrace-run1492.log",
);

use super::gunpowder_melee::Melee;

pub(crate) const AIRBASE_MELEE: Melee = Melee {
    length: 2600,
    // Item 3868, first try: **1817 of 2600**, the sequence; the count
    // parts at 2184. On 1817 both spend 60: at index 2 ours spends
    // `Guy::set_anim+0x97a < Guy::do_turn+0x4a < Unit::move_step+0x389`
    // where theirs spends `Unit::do_move+0xe84` (`docs/GOLDEN.md` §156).
    count: 2184,
    sequence: 1817,
};

/// The melee's word: the lower of the two partings.
pub(crate) const AIRBASE_MELEE_WORD: i64 = if AIRBASE_MELEE.count < AIRBASE_MELEE.sequence {
    AIRBASE_MELEE.count
} else {
    AIRBASE_MELEE.sequence
};

/// **Where the fifth melee's draw stream first parts** (item 3868): the
/// harness walks run1482's start with the script staged and compares every
/// frame's draws against run1492's trace, the count and the sequence.
#[test]
fn airbase_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &AIRBASE_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "AIRBASE_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}

/// **The melee's closing state, as the `Melee:` line says it** (Loop
/// 3842): the sum of [`endpoint::EndpointResult::counts`]' seven at
/// run1492's closing block, the `<n> off` suffix of this row's part of the
/// handoff's `Melee:` line.
///
/// Item 3868, first try: 39 off at a close 784 frames past the word — 18
/// units off, 1 unlinked, 11 buildings diverged, 9 cities diverged.
pub(crate) const AIRBASE_MELEE_CLOSING_OFF: usize = 39;

/// The word's window on run1492's stride: the block before the word to
/// 250 frames past it (item 3868: the word 1817, so 1801..2067, blocks
/// 1801, 1901 and 2001).
pub(crate) const WIDENING_AIRBASE_MELEE: (i64, i64) = (1801, 2067);

/// The word's window, widened whole on run1492's stride blocks, the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn airbase_melee_word_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1492",
            &[(MELEE_LONG.0, WIDENING_AIRBASE_MELEE.0)],
            WIDENING_AIRBASE_MELEE,
            1,
            &[],
            true,
        )
    })
}

/// **The fifth melee's word window, widened whole** (item 3868): every
/// dumped record on run1492's stride blocks over
/// [`WIDENING_AIRBASE_MELEE`], both directions, the rounds too.
/// `RON_FIRSTS=1` prints every row.
#[test]
fn airbase_melee_run1492_s_word_window_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = airbase_melee_word_window() else {
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
    pin_eq!(w.blocks, 3, "every stride block in the window");
    pin_eq!(
        w.missing.iter().cloned().collect::<Vec<_>>(),
        ["gaia:cur_anim", "gaia:cur_time"],
        "the keys the capture prints and nothing reads"
    );
    pin_eq!(w.firsts.len(), 178, "run1492's word window");
    pin_eq!(w.rounds_read, (9, 9), "the rounds read");
    pin_eq!(w.rounds.len(), 6, "the rounds that part");
}

/// **Every seat's leader** (the shape item 3791 gave the first two,
/// `docs/GOLDEN.md` §151): run1482's frames 1 and 37 print all three
/// seats' `LEADERDATA` whole, and run1492's closing dump at 2601.
#[test]
fn airbase_melee_s_leaders_are_compared_on_every_seat() {
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
    pin_eq!(w.standing(), vec![12, 8, 7], "standing keys, by seat");
    pin_eq!(w.rest().len(), 43, "the keys that part, by seat");
}

/// **The melee's closing state** (item 3868): run1492's closing block, the
/// whole map's units, walked from run1482's start with the script staged.
#[test]
fn airbase_melee_run1492_s_closing_state() {
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
            "Airbase melee closing: frame {} units {} counts {:?} torn {:?} off {:?} unlinked {:?} extra {:?}",
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
    pin_eq!(e.compared, 137, "units compared");
    pin_eq!(e.counts(), [18, 1, 0, 0, 11, 0, 9], "the closing counts");
    pin_eq!(
        e.counts().iter().sum::<usize>(),
        AIRBASE_MELEE_CLOSING_OFF,
        "the closing records off, the `Melee:` line's suffix"
    );
    pin_eq!(e.extra.clone(), vec![], "the closing extras");
}

/// The stride before the word, walked back: run1492's blocks 1401..1701,
/// where the dead Aircraft Carrier `1/18`'s number is freed and the two
/// births of 1502..1601 take 18 and 41.
pub(crate) const WIDENING_AIRBASE_MELEE_BACK: (i64, i64) = (1401, 1701);

/// run1492's stride over [`WIDENING_AIRBASE_MELEE_BACK`], the rounds
/// compared — the coverage driver's slice too.
pub(crate) fn airbase_melee_back_window() -> Option<harness::tests::Widened> {
    crate::diff::setup::with_staged_script(melee_script(), || {
        harness::tests::widen_on_siblings_with_rounds(
            &[MELEE_START],
            true,
            MELEE_LONG,
            "run1492",
            &[(MELEE_LONG.0, WIDENING_AIRBASE_MELEE_BACK.0)],
            WIDENING_AIRBASE_MELEE_BACK,
            1,
            &[],
            true,
        )
    })
}

/// **The stride before the word, widened whole** (item 3868): the frame
/// the state first parted, walked back from the word's block 1801.
/// `RON_FIRSTS=1` prints every row.
#[test]
fn airbase_melee_run1492_s_stride_before_the_word_is_widened_whole() {
    let _pins = Pins::hold();
    let Some(w) = airbase_melee_back_window() else {
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
    }
    pin_eq!(w.blocks, 4, "every stride block in the window");
    pin_eq!(w.firsts.len(), 149, "run1492's stride before the word");
    pin_eq!(w.rounds_read, (12, 12), "the rounds read");
    pin_eq!(w.rounds.len(), 10, "the rounds that part");
}
