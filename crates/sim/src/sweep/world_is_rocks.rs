//! `WorldData::is_rocks@006b4380` under the emulator against the port (item 1630).

use crate::world::{Cell, CellData, World, cell};

/// `WorldData::is_rocks@006b4380`, run by `tools/emu/sweep_world_is_rocks.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words, over a `WData` array laid out in mapped memory, answering the
/// cell's `flags & 8`. The port has no function of that name: the test is
/// inline where it is used (`land_class`), so what is driven is
/// `cell_data(c).flags & cell::ROCK`. Each row's port `World` is built with
/// the same extent, the queried cell carrying the row's `flags` and every
/// other cell the row's decoy (flags 0 or 0xFFFF), so a read of the wrong
/// cell shows. Left out: cells off the map, where the original reads past
/// its array and the port answers a zero record.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_rocks.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let flags = u16::try_from(a[4]).expect("a 16-bit flags word");
        let decoy = CellData {
            flags: if a[5] == 0 { 0 } else { 0xFFFF },
            ..CellData::default()
        };
        let mut w = World::new(xs, ys);
        for cy in 0..ys {
            for cx in 0..xs {
                w.set_cell_data(Cell { x: cx, y: cy }, decoy);
            }
        }
        w.set_cell_data(
            Cell { x, y },
            CellData {
                flags,
                ..CellData::default()
            },
        );
        let ours = i64::from(w.cell_data(Cell { x, y }).flags & cell::ROCK);
        assert_eq!(ours, want, "is_rocks — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_rocks@006b4380: {n} rows agree");
}
