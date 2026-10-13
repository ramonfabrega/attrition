//! `WorldData::is_oil_at@00472af0` under the emulator against the port (item 1630).

use crate::world::{Cell, CellData, World, cell};

/// `WorldData::is_oil_at@00472af0`, run by `tools/emu/sweep_world_is_oil_at.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words (x, then y), over a `WData` array laid out in mapped memory,
/// answering `flags & 0x800` of the queried cell. The port has no method of
/// its own: the predicate is the inline `cell_data(c).flags & cell::OIL` that
/// `World::land_class` tests, so that expression on a `World` built with the
/// same extent (queried cell carrying the row's `flags`, every other cell the
/// row's decoy flags) is what is driven; the original's result is `0x800` or
/// `0`, and the port's masked bit must equal it.
/// Left out: cells off the map, where the original reads past its array.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_oil_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let flags = u16::try_from(a[4]).expect("a 16-bit flags word");
        let decoy = [0u16, 0x800, 0xF7FF][a[5] as usize];
        let mut w = World::new(xs, ys);
        for cy in 0..ys {
            for cx in 0..xs {
                w.set_cell_data(
                    Cell { x: cx, y: cy },
                    CellData {
                        flags: decoy,
                        ..CellData::default()
                    },
                );
            }
        }
        w.set_cell_data(
            Cell { x, y },
            CellData {
                flags,
                ..CellData::default()
            },
        );
        let ours = i64::from(w.cell_data(Cell { x, y }).flags & cell::OIL);
        assert_eq!(ours, want, "is_oil_at — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_oil_at@00472af0: {n} rows agree");
}
