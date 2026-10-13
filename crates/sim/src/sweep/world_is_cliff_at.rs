//! `WorldData::is_cliff_at@0046f8c0` under the emulator against the port (item 1630).

use crate::world::{Pos, World, tile};

/// `WorldData::is_cliff_at@0046f8c0`, run by `tools/emu/sweep_world_is_cliff_at.py`
/// under unicorn (tools/emu): it reads the singleton `GameAccessConst::worldc`
/// (a pointer cell at `0x00c061d0`, mapped and pointed at a fake world), indexes
/// `tdata` by `y * tile_xs + x` at a stride of two bytes, and answers whether
/// the mask byte's low two bits are 1. The port has no standalone function for
/// it: the predicate is the inline `tile_mask(t) & tile::OBJECT ==
/// tile::OBJECT_CLIFF` that `world.rs` documents, so that is what is driven
/// here, over a `World` whose queried tile carries the row's mask and every
/// other tile the row's decoy. Left out: tiles off the map (the original
/// reads past its array; the port's `tile_mask` answers 0).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_cliff_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (txs, tys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let mask = u16::try_from(a[4]).expect("a 16-bit mask");
        let decoy = u16::try_from(a[5]).expect("a decoy mask");
        // The port's world is whole cells of `TILES_PER_CELL` tiles a side;
        // the rows' tile extent is covered by a world at least that big.
        let per = crate::world::TILES_PER_CELL;
        let mut w = World::new((txs + per - 1) / per, (tys + per - 1) / per);
        for ty in 0..tys {
            for tx in 0..txs {
                w.set_tile_mask(Pos::new(tx, ty), decoy);
            }
        }
        w.set_tile_mask(Pos::new(x, y), mask);
        let ours = i64::from(w.tile_mask(Pos::new(x, y)) & tile::OBJECT == tile::OBJECT_CLIFF);
        assert_eq!(ours, want, "is_cliff_at — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_cliff_at@0046f8c0: {n} rows agree");
}
