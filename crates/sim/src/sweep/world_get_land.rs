//! `WorldData::get_land@006b4c70` against the port.

use crate::world::{Cell, CellData, Pos, World};

/// `WorldData::get_land@006b4c70`, run under the emulator (unicorn) by
/// `tools/emu/sweep_world_get_land.py` on a laid-out `WorldData` (`xs`,
/// `tile_xs`, `wdata`, `tdata`; every cell and tile but the one asked about
/// a bit-inverted decoy), against `World::land_class_tile`.
///
/// The port implements only the **positive-`mode` arm** — the tile form the
/// callers pass `1` to; the original's `mode > 0` arm is one arm for every
/// positive value, so every positive-mode row (1, 2, 3, 100, `i32::MAX`) is
/// asserted against it. The two other arms — `mode == 0` (a `COAST` cell
/// answers 2, else the stored `land`) and `mode < 0` (a `COAST` cell answers
/// 3, else the stored `land`) — are **unported**: the port has no
/// counterpart, so those rows are counted and not compared. Tiles are
/// always on the map (the original does no bounds test; the port answers a
/// zero record off the map).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_land.py") else {
        return;
    };
    let (mut n, mut compared, mut unported) = (0, 0, 0);
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let [width, tx, ty, mode, flags, land, mask] = a[..] else {
            panic!("a row is seven arguments: `{line}`");
        };
        n += 1;
        if mode <= 0 {
            unported += 1;
            continue;
        }
        let width = i32::try_from(width).unwrap();
        let mut w = World::new(width, 3);
        let flags = u16::try_from(flags).unwrap();
        let land = i8::try_from(land).unwrap();
        let mask = u16::try_from(mask).unwrap();
        let (tx, ty) = (i32::try_from(tx).unwrap(), i32::try_from(ty).unwrap());
        // The decoys, as the script lays them: bit-inverted everywhere else.
        let decoy_land = if (-128..=127).contains(&!i32::from(land)) {
            !land
        } else {
            0
        };
        for y in 0..3 {
            for x in 0..width {
                w.set_cell_data(
                    Cell::new(x, y),
                    CellData {
                        flags: !flags,
                        land: decoy_land,
                        ..CellData::default()
                    },
                );
            }
        }
        for y in 0..12 {
            for x in 0..4 * width {
                w.set_tile_mask(Pos { x, y }, !mask);
            }
        }
        w.set_cell_data(
            World::cell_of_tile(Pos { x: tx, y: ty }),
            CellData {
                flags,
                land,
                ..CellData::default()
            },
        );
        w.set_tile_mask(Pos { x: tx, y: ty }, mask);
        let ours = i64::from(w.land_class_tile(Pos { x: tx, y: ty }));
        assert_eq!(ours, want, "land_class_tile — row `{line}`");
        compared += 1;
    }
    assert!(n >= 200, "only {n} rows");
    assert!(compared >= 200, "only {compared} compared rows");
    eprintln!(
        "WorldData::get_land@006b4c70: {n} rows, {compared} agree (positive mode), \
         {unported} unported (mode <= 0)"
    );
}
