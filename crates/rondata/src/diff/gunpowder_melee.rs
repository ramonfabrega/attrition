//! **The first melee: a held Gunpowder age on Great Sahara, every seat a
//! computer** (item 3733, `docs/GOLDEN.md` §149; the shape is §155).
//! Great Sahara, `MAP_STYLE 7`, seed 12345, `--profile PLAYERS=2`: the idle
//! human Nubians (4) in seat 0, the British (11, London) as who 1 and the
//! Inca (2, Cuzco) as who 2, all three at war from frame 0 with no `war`
//! line; `STARTING_TECHNOLOGY 3`, `ENDING_TECHNOLOGY 3`,
//! `STARTING_RESOURCES 7`. `tools/gamelog/golden/melee-gunpowder.cmd`
//! grants each computer 100,000 of every resource on 600 and 602 and spawns
//! twelve of the age's types a side from 610, two frames apart, between the
//! two territories, with the age's two never-seen lines (Nubian) on who 0.
//!
//! Captures: run1412, the `DUMP_ALL` start; run1413, the staged game to its
//! end at `cover=0`, `end:UNITS=3,GUYS=2,DEATHS=1,AMMO=5` on a stride of
//! 100, walked from run1412's initial state with the script staged by
//! [`Built::tick`] ([`crate::diff::setup::with_staged_script`]).

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

/// run1412: the lobby's `DUMP_ALL` start, 37 frames.
pub(crate) const MELEE_START: &str = "gamelog-run1412-greatsahara-gunpowder-melee-start.txt";

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-gunpowder.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-gunpowder.cmd: {e}"))
}

/// The script's own line count: two `resource` lines, twelve `add` twins
/// and the two Nubian lines on who 0.
pub(crate) const MELEE_STAGED: usize = 28;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3: a chapter's staging is walked first): run1412's start stood up with
/// the script staged by the harness's own tick, to frame
/// `RON_MELEE_STAGE` (default 700), printing where every `add` landed and
/// every line the interpreter carried and did not act on.
#[test]
fn gunpowder_melee_staging_walk() {
    let inst = install();
    let Some(inst) = inst else {
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
    for (w, n) in built.sim.nation.iter().enumerate() {
        eprintln!(
            "  who={w} human={} age {}",
            n.human, built.sim.players[w].age
        );
    }
    assert_eq!(
        st.applied.skipped_total(),
        0,
        "every staged line reached the simulation"
    );
}

/// run1413: the staged game, the draw stream to its end at `cover=0`, the
/// melee's records on a stride of 100.
pub(crate) const MELEE_LONG: (&str, &str) = (
    "gamelog-run1413-greatsahara-gunpowder-melee-trace.txt",
    "rontrace-run1413.log",
);

/// Where the melee's draw stream first parts (item 3733, first try).
pub(crate) struct Melee {
    pub length: i64,
    pub count: i64,
    pub sequence: i64,
}

pub(crate) const GUNPOWDER_MELEE: Melee = Melee {
    length: 2600,
    count: 0,
    sequence: 0,
};

/// The melee's word: the lower of the two partings.
pub(crate) const GUNPOWDER_MELEE_WORD: i64 = if GUNPOWDER_MELEE.count < GUNPOWDER_MELEE.sequence {
    GUNPOWDER_MELEE.count
} else {
    GUNPOWDER_MELEE.sequence
};

/// **The melee's first parting**, walked from run1412's start with the
/// script staged by the harness's own tick.
#[test]
fn gunpowder_melee_first_parting() {
    let _pins = Pins::hold();
    let m = &GUNPOWDER_MELEE;
    let Some(w) = crate::diff::setup::with_staged_script(melee_script(), || {
        third::walk_from(MELEE_START, MELEE_LONG)
    }) else {
        return;
    };
    eprintln!(
        "GUNPOWDER_MELEE: count {} sequence {} of {}; {}",
        w.count, w.sequence, w.last, w.row
    );
    pin_eq!(w.last, m.length, "the game's last frame");
    pin_eq!(w.count, m.count, "the draw count's first parting");
    pin_eq!(w.sequence, m.sequence, "the draw sequence's first parting");
}
