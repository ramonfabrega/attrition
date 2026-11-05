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
        let at = s.snap_center_placed(ab, sim::Pos::new(tx * 0xc0 + 0x60, ty * 0xc0 + 0x60), Some(who));
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
        assert_eq!(filed.count(), 1, "who {who}'s Airbase is active, in a city, a trainer");
        assert_eq!(s.num_buildings_of(who, ab), 1, "who {who}'s Airbase counted");
    }
}
