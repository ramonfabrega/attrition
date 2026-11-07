//! **The sixth melee: the third's lobby with two massed armies, past 100
//! live corpses** (item 3971, `docs/GOLDEN.md` §177; the shape is §155).
//! run1467's lobby held whole — Great Sahara, `MAP_STYLE 7`, seed 12345,
//! `--profile PLAYERS=2`, the idle human Nubians (4) in seat 0, the French
//! (10) as who 1 and the Russians (13) as who 2, `STARTING_TECHNOLOGY 7`,
//! `ENDING_TECHNOLOGY 7`, `STARTING_RESOURCES 7` — with
//! `tools/gamelog/golden/melee-corpses.cmd`: each computer's buckets
//! filled on 600 and 602, then three massed lines a side from 610. The
//! subject is `Objects::add_death@00653b60`'s pool of 100 `DeathObj`
//! slots, which no capture on disk had filled.

use super::testkit::*;
use super::*;
use crate::testenv::{dump, install};

pub(crate) use super::information_melee::MELEE_START;

/// The melee's script, read from the tree.
pub(crate) fn melee_script() -> crate::golden::Script {
    let path = format!(
        "{}/../../tools/gamelog/golden/melee-corpses.cmd",
        env!("CARGO_MANIFEST_DIR")
    );
    crate::golden::Script::read(std::path::Path::new(&path))
        .unwrap_or_else(|e| panic!("tools/gamelog/golden/melee-corpses.cmd: {e}"))
}

/// The script's own line count: two `resource` lines and three `add`
/// twins.
pub(crate) const MELEE_STAGED: usize = 8;

/// **The staging, walked before any capture holds it** (`docs/GOLDEN.md`
/// §3, §177): run1467's start stood up with the script staged by the
/// harness's own tick, to frame `RON_MELEE_STAGE` (default 2600), the
/// death list and the pool printed every 50 frames.
#[test]
fn corpse_melee_staging_walk() {
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
        .unwrap_or(2600);
    let loaded = crate::load::load(&inst).unwrap();
    let text = crate::capture::read(&path);
    let log = Log::parse(&text);
    let mut init = log.initial().expect("a start dump");
    let own = log.initial().expect("a start dump");
    borrow_from_siblings(&mut init, &[&own]);
    let mut built =
        crate::diff::setup::with_staged_script(script, || build_sim(&loaded, &init, Tuning::RON));
    let mut most = (0usize, 0i64);
    while built.sim.frame <= to {
        built.tick();
        let s = &built.sim;
        let live: usize = s
            .deaths
            .iter()
            .map(|d| d.figures.live(s.frame - d.first_frame) as usize)
            .sum();
        if live > most.0 {
            most = (live, s.frame);
        }
        if s.frame % 50 == 0 {
            let alive = |w| s.units.iter().filter(|u| u.alive() && u.owner == w).count();
            eprintln!(
                "  {}: deaths {} records {} alive who1 {} who2 {}",
                s.frame,
                s.deaths.len(),
                live,
                alive(1),
                alive(2)
            );
        }
    }
    let st = built.staged.as_ref().expect("the script is staged");
    eprintln!(
        "melee staged to {to}: {} line(s) ran, {} unit(s); {} carried and not acted on; most records {} on {}",
        st.applied.ran,
        st.applied.units,
        st.applied.skipped_total(),
        most.0,
        most.1
    );
    for ((word, why), k) in &st.applied.skipped {
        eprintln!("  {k:5} {word}: {why}");
    }
    assert_eq!(st.applied.skipped_total(), 0, "add refuses nothing");
    assert_eq!(st.applied.ran, MELEE_STAGED, "every line ran");
}
