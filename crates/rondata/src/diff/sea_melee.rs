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
