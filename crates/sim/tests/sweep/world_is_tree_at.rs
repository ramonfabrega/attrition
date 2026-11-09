//! `WorldData::is_tree_at@0046f930` under the emulator against the port (item 1630).

use crate::Pos;
use crate::world::{TILES_PER_CELL, World, tile};

/// `WorldData::is_tree_at@0046f930`, run by `tools/emu/sweep_world_is_tree_at.py`
/// under unicorn (tools/emu): `ecx` the world, two pointers to the `TCoord`
/// ints, over a `TData` array (one `u16` mask a tile) laid out in mapped
/// memory, answering 1 where `mask & 0x30 == 0x30`. Each row's port `World`
/// is built on the enclosing whole cells of the row's tile extent, the
/// queried tile carrying the row's mask and every other in-extent tile the
/// row's decoy mask; `World::tile_mask` is asserted to read the row's mask
/// back, and the port's predicate (`gather.rs`'s `gather_tile_kind`, timber
/// arm, private to its module and so not driven directly: the same
/// `tile::SURFACE == tile::SURFACE_FOREST` test is applied here to the real
/// `World::tile_mask`) is asserted against the original. Left out: tiles off
/// the map, where the original reads past its array and the port answers 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_tree_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (txs, tys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let mask = u16::try_from(a[4]).expect("a 16-bit mask");
        let decoy = if a[5] == 0 { 0u16 } else { 0xFFFF };
        let cw = (txs + TILES_PER_CELL - 1) / TILES_PER_CELL;
        let ch = (tys + TILES_PER_CELL - 1) / TILES_PER_CELL;
        let mut w = World::new(cw, ch);
        for ty in 0..tys {
            for tx in 0..txs {
                w.set_tile_mask(Pos::new(tx, ty), decoy);
            }
        }
        let t = Pos::new(x, y);
        w.set_tile_mask(t, mask);
        assert_eq!(w.tile_mask(t), mask, "tile_mask — row `{line}`");
        let ours = i64::from(w.tile_mask(t) & tile::SURFACE == tile::SURFACE_FOREST);
        assert_eq!(ours, want, "is_tree_at — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_tree_at@0046f930: {n} rows agree");
}
