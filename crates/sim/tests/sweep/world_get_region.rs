//! `WorldData::get_region@006b5680` under the emulator against the port (item 1721).

use crate::world::{Cell, Terrain, World};

/// `WorldData::get_region@006b5680`, run by `tools/emu/sweep_world_get_region.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words, over a `WData` array laid out in mapped memory, answering the
/// cell's `region` (`WData++0x4`) sign-extended. The port reads it as `region_of(c)`, `None` for -1.
/// Each row's port `World` has the same extent, the queried cell carrying the
/// row's value and every other cell the row's decoy, so a read of the wrong
/// cell shows. Cells off the map are left out.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_region.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let (v, decoy) = (a[4], a[5]);
        let mut w = World::new(xs, ys);
        let r = w.add_region(Terrain::Land);
        for _ in 0..200 {
            w.add_region(Terrain::Land);
        }
        let _ = r;
        let dv: i64 = if decoy == 0 { 3 } else { -1 };
        for cy in 0..ys {
            for cx in 0..xs {
                if dv >= 0 {
                    w.set_region(Cell { x: cx, y: cy }, dv as u16);
                }
            }
        }
        if v >= 0 {
            w.set_region(Cell { x, y }, v as u16);
        } else {
            w = World::new(xs, ys);
            for _ in 0..201 {
                w.add_region(Terrain::Land);
            }
            for cy in 0..ys {
                for cx in 0..xs {
                    if (cx, cy) != (x, y) && dv >= 0 {
                        w.set_region(Cell { x: cx, y: cy }, dv as u16);
                    }
                }
            }
        }
        let ours: i64 = w.region_of(Cell { x, y }).map_or(-1, i64::from);
        assert_eq!(ours, want, "get_region — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("get_region@006b5680: {n} rows agree");
}
