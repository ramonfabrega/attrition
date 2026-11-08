//! `GatherPoint::is_inside@00730400` against this crate's port.

use crate::Sim;
use crate::rally::GatherPoint;
use crate::tuning::Tuning;
use crate::world::{Cell, Pos, Terrain, World};

/// `GatherPoint::is_inside@00730400` run under the emulator by
/// `tools/emu/sweep_gather_point_is_inside.py` on `<x> <y> <action>` rows,
/// against [`Sim::gather_inside`] — the port keeps the predicate inline on a
/// building's head point (`BuildData::gather_inside@0046f180`), so each row
/// is a one-point list on a building.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_gather_point_is_inside.py") else {
        return;
    };
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut sim = Sim::new(Tuning::RON, w, 2);
    let b = sim.add_building(0, Pos::new(1000, 1000), 4);
    let mut n = 0;
    for line in rows.lines() {
        let (args, want) = super::row(line);
        let [x, y, action] = args[..] else {
            panic!("row `{line}`: three arguments")
        };
        sim.buildings[b].gather = vec![GatherPoint {
            pos: Pos::new(i32::try_from(x).unwrap(), i32::try_from(y).unwrap()),
            action: u8::try_from(action).unwrap(),
        }];
        assert_eq!(
            i64::from(sim.gather_inside(b)),
            want,
            "row `{line}`: ours vs the original's"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("agreed {n} rows");
}
