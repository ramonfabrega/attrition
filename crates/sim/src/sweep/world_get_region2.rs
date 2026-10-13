//! `WorldData::get_region2@006b3960` under the emulator against the port (item 1721).

use crate::world::{Cell, CellData, World};

/// `WorldData::get_region2@006b3960`, run by `tools/emu/sweep_world_get_region2.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words, over a `WData` array laid out in mapped memory, answering the
/// cell's `region2` (`WData++0x6`) sign-extended. The port reads it as `CellData::region2`, `None` for -1.
/// Each row's port `World` has the same extent, the queried cell carrying the
/// row's value and every other cell the row's decoy, so a read of the wrong
/// cell shows. Cells off the map are left out.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_region2.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let (v, decoy) = (a[4], a[5]);
        let mut w = World::new(xs, ys);
        let dv: Option<u16> = if decoy == 0 { Some(3) } else { None };
        for cy in 0..ys {
            for cx in 0..xs {
                w.set_cell_data(
                    Cell { x: cx, y: cy },
                    CellData {
                        region2: dv,
                        ..CellData::default()
                    },
                );
            }
        }
        w.set_cell_data(
            Cell { x, y },
            CellData {
                region2: u16::try_from(v).ok(),
                ..CellData::default()
            },
        );
        let ours: i64 = w.cell_data(Cell { x, y }).region2.map_or(-1, i64::from);
        assert_eq!(ours, want, "get_region2 — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("get_region2@006b3960: {n} rows agree");
}
