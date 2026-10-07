use crate::build::BuildType;
use crate::{Pos, Sim, Tuning, World};

/// `BuildTypeData::corner_tile@006364c0` — the original's function run under
/// unicorn by `tools/emu/sweep_build_type_corner_tile.py` — against
/// [`Sim::footprint_centre`], this crate's `(size + 2·corner) · 0x60` per
/// axis. A row is `<x_size> <y_size> <corner_x> <corner_y> <axis> -> <value>`.
#[test]
fn the_emulated_corner_tile_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_corner_tile.py") else {
        return;
    };
    let mut sim = Sim::new(Tuning::RON, World::new(2, 2), 2);
    let ty = sim.add_build_type(BuildType::default());
    let mut n = 0;
    for line in rows.lines() {
        let (a, want) = super::row(line);
        let [xs, ys, cx, cy, axis] = a[..] else {
            panic!("row `{line}`: five arguments expected");
        };
        sim.build_types[ty].x_size = xs as i32;
        sim.build_types[ty].y_size = ys as i32;
        let got = sim.footprint_centre(ty, Pos::new(cx as i32, cy as i32));
        let got = if axis == 0 { got.x } else { got.y } as i64;
        assert_eq!(got, want, "row `{line}`: ours {got}, the original's {want}");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
}
