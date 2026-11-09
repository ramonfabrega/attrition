//! `WorldData::get_tregion@006b52e0` under the emulator against the port
//! (item 1630).

use crate::world::{Cell, CellData, Pos, Terrain, World};

/// `WorldData::get_tregion@006b52e0`, run by
/// `tools/emu/sweep_world_get_tregion.py` under unicorn (tools/emu): `ecx`
/// the world, two pointers to the `TCoord` words, over `WData` and `TData`
/// arrays laid out in mapped memory; it answers the covering cell's
/// `region2` where its `flags & 0x100` is set and the tile's
/// `mask & 0x30 == 0x20`, else its `region`, as a sign-extended short.
/// The port is `World::tregion_alt` (the coastal read); `World::tregion` is
/// the plain one, asserted against the rows where the refinement does not
/// fire. Each row's port `World` has the row's extent, the queried cell and
/// tile carrying the row's values and every other cell and tile a decoy.
/// Region ids are driven from -1 (none) to 32767; other negative shorts are
/// not representable in the port's `Option<u16>`.
///
/// **A parting, asserted still parting:** where the refinement fires and
/// `region2` is -1, the original answers -1; the port's `tregion_alt` falls
/// back to the cell's `region`. The rows of that shape are kept apart and
/// asserted to differ (except where the cell's own region is -1 too).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_tregion.py") else {
        return;
    };
    let mut n = 0;
    let mut parting = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, tx, ty) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let flags = u16::try_from(a[4]).expect("a 16-bit flags word");
        let reg = |v: i64| (v >= 0).then(|| u16::try_from(v).expect("a region id"));
        let (r1, r2) = (reg(a[5]), reg(a[6]));
        let mask = u16::try_from(a[7]).expect("a 16-bit mask");
        let (df, dr1, dr2, dm) = if a[8] != 0 {
            (0, 92, 93, 0x00)
        } else {
            (0x100, 90, 91, 0x20)
        };
        let cell = Cell {
            x: tx >> 2,
            y: ty >> 2,
        };
        let top = [r1, r2, Some(93)].into_iter().flatten().max().unwrap_or(0);
        let mut w = World::new(xs, ys);
        for _ in 0..=top {
            w.add_region(Terrain::Land);
        }
        for cy in 0..ys {
            for cx in 0..xs {
                let c = Cell { x: cx, y: cy };
                let own = c == cell;
                w.set_cell_data(
                    c,
                    CellData {
                        flags: if own { flags } else { df },
                        region2: if own { r2 } else { Some(dr2) },
                        ..CellData::default()
                    },
                );
                if let Some(r) = if own { r1 } else { Some(dr1) } {
                    w.set_region(c, r);
                }
                for dy in 0..4 {
                    for dx in 0..4 {
                        w.set_tile_mask(Pos::new(cx * 4 + dx, cy * 4 + dy), dm);
                    }
                }
            }
        }
        let t = Pos::new(tx, ty);
        w.set_tile_mask(t, mask);
        let ours = w.tregion_alt(t).map_or(-1, i64::from);
        let fires = flags & 0x100 != 0 && mask & 0x30 == 0x20;
        if fires && r2.is_none() && r1.is_some() {
            parting += 1;
            assert_eq!(want, -1, "original answers region2 = -1 — row `{line}`");
            assert_ne!(ours, want, "still parting — row `{line}`");
        } else {
            assert_eq!(ours, want, "tregion_alt — row `{line}`");
        }
        if !fires {
            let plain = w.tregion(t).map_or(-1, i64::from);
            assert_eq!(plain, want, "tregion — row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    assert!(parting > 0, "the parting shape is not in the rows");
    eprintln!("get_tregion@006b52e0: {n} rows, {parting} parting (coastal read, region2 = -1)");
}
