//! `WorldData::is_river@0046d390` under the emulator against the port (item 1630).

use crate::Pos;
use crate::world::{TILES_PER_CELL, World, tile};

/// `WorldData::is_river@0046d390`, run by `tools/emu/sweep_world_is_river.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `TCoord`
/// words, over a `tdata` array of `u16` masks in mapped memory, answering
/// `mask & 0x800`. The port has no function of its own for it (callers test
/// the bit inline), so this drives the nearest real thing: `World::tile_mask`
/// masked with `tile::RIVER`, on a `World::new` of the row's extent, the
/// queried tile carrying the row's mask and every other tile the row's decoy
/// (0, 0xFFFF, 0xF7FF or 0x0800) so a read of the wrong tile shows.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_river.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| l.contains(" -> ")) {
        let (a, want) = super::row(line);
        let [cw, ch, x, y, mask, decoy] = a[..] else {
            panic!("six arguments in `{line}`");
        };
        let (cw, ch) = (cw as i32, ch as i32);
        let fill = [0u16, 0xFFFF, 0xF7FF, 0x0800][decoy as usize];
        let mut w = World::new(cw, ch);
        for ty in 0..ch * TILES_PER_CELL {
            for tx in 0..cw * TILES_PER_CELL {
                w.set_tile_mask(Pos::new(tx, ty), fill);
            }
        }
        let t = Pos::new(x as i32, y as i32);
        w.set_tile_mask(t, u16::try_from(mask).expect("a 16-bit mask"));
        let ours = i64::from(w.tile_mask(t) & tile::RIVER);
        assert_eq!(ours, want, "is_river — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_river@0046d390: {n} rows agree");
}
