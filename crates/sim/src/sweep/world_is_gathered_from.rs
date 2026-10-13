//! `WorldData::is_gathered_from@00472ac0` under the emulator against the port (item 1630).

use crate::Pos;
use crate::world::{TILES_PER_CELL, World, tile};

/// `WorldData::is_gathered_from@00472ac0`, run by
/// `tools/emu/sweep_world_is_gathered_from.py` under unicorn (tools/emu):
/// `ecx` the world, two pointers to the `TCoord` words, over a `TData`
/// array laid out in mapped memory, answering `mask & 0x1000`. The port has
/// no function of its own for it (readers test the bit inline), so the
/// nearest real port is `World::tile_mask` masked with
/// `tile::GATHERED_FROM`, which is what is asserted against the original's
/// value. Each row builds a world whose tile grid is the row's `tile_xs` x
/// `tile_ys` (a whole number of 4-tile cells; the script's extents are
/// multiples of 4), the queried tile carrying the row's mask and every other
/// tile the row's decoy. Left out: tiles off the map, where the original
/// reads past its array.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_gathered_from.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| l.contains(" -> ")) {
        let (a, want) = super::row(line);
        let [txs, tys, x, y, mask, decoy] = a[..] else {
            panic!("six arguments in `{line}`");
        };
        let (txs, tys, x, y) = (txs as i32, tys as i32, x as i32, y as i32);
        let mask = u16::try_from(mask).expect("a 16-bit mask");
        let decoy = [0u16, 0x1000, 0xEFFF][decoy as usize];
        // The port's grid is cells x 4; round up and fill the slack with the
        // decoy as well, so only in-grid tiles are ever queried.
        let (cw, ch) = ((txs + 3) / 4, (tys + 3) / 4);
        let mut w = World::new(cw, ch);
        for ty in 0..ch * TILES_PER_CELL {
            for tx in 0..cw * TILES_PER_CELL {
                w.set_tile_mask(Pos::new(tx, ty), decoy);
            }
        }
        w.set_tile_mask(Pos::new(x, y), mask);
        let ours = i64::from(w.tile_mask(Pos::new(x, y)) & tile::GATHERED_FROM);
        assert_eq!(ours, want, "is_gathered_from: row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_gathered_from@00472ac0: {n} rows agree");
}
