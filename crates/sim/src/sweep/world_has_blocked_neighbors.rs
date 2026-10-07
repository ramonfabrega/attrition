//! `WorldData::has_blocked_neighbors@006b2990` against the port (item 1619).

use crate::Pos;
use crate::world::{TILES_PER_CELL, World, tile};

/// `WorldData::has_blocked_neighbors@006b2990`, run under unicorn by
/// `tools/emu/sweep_world_has_blocked_neighbors.py` (the real `move_x` /
/// `move_y` tables read from the image, a `tdata` grid laid out in mapped
/// memory) against `World::has_blocked_neighbors`: each row's
/// `<cw> <ch> <tx> <ty> <ring> <mode>` builds a `cw` x `ch`-cell world (the
/// original's `tile_xs`/`tile_ys` are 4 x that), sets `tile::BLOCKED`
/// (0x4000) on the neighbours the `ring` bits name, fills the rest by
/// `mode` as the script does, and the answers are compared. Positions run
/// off the map on every side and to +-2^29 (beyond that the port's `i32`
/// adds would overflow where the original wraps; no game coordinate gets
/// there).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_has_blocked_neighbors.py") else {
        return;
    };
    // The script's ring, bit i = the neighbour at the i-th compass offset.
    const RING: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
    ];
    let mut n = 0;
    for line in rows.lines().filter(|l| l.contains(" -> ")) {
        let (a, original) = super::row(line);
        let [cw, ch, tx, ty, ring, mode] = a[..] else {
            panic!("six arguments in `{line}`");
        };
        let (cw, ch) = (i32::try_from(cw).unwrap(), i32::try_from(ch).unwrap());
        let (tx, ty) = (i32::try_from(tx).unwrap(), i32::try_from(ty).unwrap());
        let mut w = World::new(cw, ch);
        let (xs, ys) = (cw * TILES_PER_CELL, ch * TILES_PER_CELL);
        let (base, far): (u16, u16) = match mode {
            0 => (0, 0),
            1 => (0xBFFF, 0xBFFF),
            _ => (0xBFFF, 0xFFFF),
        };
        for y in 0..ys {
            for x in 0..xs {
                w.set_tile_mask(Pos::new(x, y), far);
            }
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                w.set_tile_mask(Pos::new(tx + dx, ty + dy), base);
            }
        }
        if mode == 2 {
            w.set_tile_mask(Pos::new(tx, ty), 0xFFFF);
        }
        for (i, (dx, dy)) in RING.iter().enumerate() {
            if ring >> i & 1 != 0 {
                w.set_tile_bits(Pos::new(tx + dx, ty + dy), tile::BLOCKED);
            }
        }
        assert_eq!(
            i64::from(w.has_blocked_neighbors(Pos::new(tx, ty))),
            original,
            "has_blocked_neighbors on row `{line}`"
        );
        n += 1;
    }
    eprintln!("has_blocked_neighbors: {n} rows agree");
    assert!(n >= 200, "only {n} rows");
}
