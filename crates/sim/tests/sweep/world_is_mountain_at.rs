//! `WorldData::is_mountain_at@0046f900` under the emulator against the port (item 1630).

use crate::world::{Pos, World, tile};

/// `WorldData::is_mountain_at@0046f900`, run by
/// `tools/emu/sweep_world_is_mountain_at.py` under unicorn (tools/emu):
/// `ecx` the world, two pointers to the `TCoord` words, over a `TData`
/// array (stride 2) laid out in mapped memory, answering 1 where
/// `mask & 3 == 2`. The port has no function of its own: the predicate is
/// inline at every reader (`gather.rs` `gather_tile_kind`, `path.rs`,
/// `place.rs`, `transport.rs`), so this drives the nearest real thing —
/// `World::set_tile_mask` / `World::tile_mask` with the `tile::OBJECT` and
/// `tile::OBJECT_MOUNTAIN` constants those readers test. Each row's `World`
/// has the row's extent, the queried tile carrying the row's mask and every
/// other tile the row's decoy (0, or a mountain), so a read of the wrong
/// tile shows. Left out: tiles off the map, where the original reads past
/// its array and the port's `tile_mask` answers 0.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_is_mountain_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (xs, ys, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32);
        let mask = u16::try_from(a[4]).expect("a 16-bit mask");
        let decoy = if a[5] == 0 { 0 } else { tile::OBJECT_MOUNTAIN };
        let mut w = World::new(xs, ys);
        for ty in 0..ys * 4 {
            for tx in 0..xs * 4 {
                w.set_tile_mask(Pos::new(tx, ty), decoy);
            }
        }
        w.set_tile_mask(Pos::new(x, y), mask);
        let m = w.tile_mask(Pos::new(x, y));
        let ours = i64::from(m & tile::OBJECT == tile::OBJECT_MOUNTAIN);
        assert_eq!(ours, want, "is_mountain_at — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("is_mountain_at@0046f900: {n} rows agree");
}
