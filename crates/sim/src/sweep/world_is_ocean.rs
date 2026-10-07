//! `WorldData::is_ocean@006b4830` under the emulator against
//! `world::World::is_ocean` (item 1619).

use crate::world::{Cell, CellData, World};

/// `WorldData::is_ocean@006b4830`, run by `tools/emu/sweep_world_is_ocean.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `WCoord`
/// words, over a `WData` array laid out in mapped memory, answering 1 where
/// the cell's `flags & 0x100` is clear and its `land` is 1 or 2. Each row's
/// port `World` is built with the same extent, the queried cell carrying the
/// row's `flags` and `land` (`land` is the port's `i8`, so every signed-char
/// value is representable, and `flags` its `u16`) and every other cell the
/// row's decoy record; `World::is_ocean` is asserted against the original.
/// Left out: cells off the map, where the original reads past its array and
/// the port answers false by its bounds test. `ai_sites.rs`'s private
/// `site_is_ocean` is the same predicate inline and is not driven here.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_ocean.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let flags = u16::try_from(a[4]).expect("a 16-bit flags word");
        let land = i8::try_from(a[5]).expect("a signed-char land");
        let decoy = if a[6] == 0 {
            CellData {
                flags: 0,
                land: 1,
                ..CellData::default()
            }
        } else {
            CellData {
                flags: 0x100,
                land: 3,
                ..CellData::default()
            }
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
                land,
                ..CellData::default()
            },
        );
        let ours = i64::from(w.is_ocean(Cell { x, y }));
        assert_eq!(ours, want, "is_ocean — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_ocean@006b4830: {n} rows agree");
}
