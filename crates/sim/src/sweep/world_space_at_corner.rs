//! `WorldData::space_at_corner@006b27f0` against the port (item 1619).

use crate::world::{Cell, Owner, World};
use crate::{Pos, Sim, Tuning};

const GRIDS: usize = 12;
const DENS: [u32; GRIDS] = [0, 0, 1, 2, 3, 5, 8, 12, 20, 40, 80, 160];

/// The script's integer hash, repeated exactly.
fn mix(a: u32, b: i32, c: i32) -> u32 {
    let mut h = a
        .wrapping_mul(0x9E37_79B1)
        .wrapping_add((b as u32).wrapping_mul(0x85EB_CA6B))
        .wrapping_add((c as u32).wrapping_mul(0xC2B2_AE35))
        .wrapping_add(0x27D4_EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

fn dims(g: usize) -> (i32, i32) {
    (4 + (g % 3) as i32, 3 + (g % 2) as i32)
}

fn tile_mask(g: usize, x: i32, y: i32) -> u16 {
    let d = DENS[g];
    let city_off = (g as u32 * 37) % 96;
    let (h1, h2) = (mix(g as u32, x, y), mix(g as u32 + 100, x, y));
    let mut m = if ((h1 >> 2) & 255) < d {
        3
    } else {
        (h1 >> 10) % 3
    };
    if ((h1 >> 18) & 255) < d {
        m |= 0x80;
    }
    if (h2 & 255) < d {
        m |= 0x4000;
    }
    if ((h2 >> 8) & 255) >= city_off {
        m |= 0x100;
    }
    (m | ((h2 >> 16) & 0xBE7C)) as u16
}

fn cell_who(g: usize, cx: i32, cy: i32) -> Owner {
    let d = DENS[g];
    let h3 = mix(g as u32 + 200, cx, cy);
    let r = h3 & 255;
    if r < d * 2 {
        Owner::Player(((h3 >> 8) % 3) as u8)
    } else if r < d * 2 + 8 {
        Owner::Ambiguous
    } else {
        Owner::None
    }
}

/// `WorldData::space_at_corner@006b27f0`, run under unicorn by
/// `tools/emu/sweep_world_space_at_corner.py` on a laid-out `WorldData`
/// (its `tdata` masks, `wdata` `who` bytes and the original's own `.rdata`
/// walk tables), against `Sim::space_at_corner`: each row's `<grid> <tx> <ty>
/// <who> <junk> <need_city>` is rebuilt here from the script's integer hash
/// (twelve grids of 4..6 by 3..4 cells, rising density of the bits the
/// function reads) and the answers compared. The original takes its corner
/// by two `TCoord` pointers and a fifth argument tested `!= 0`; the port's
/// `junk` (`param_4`) is not an argument, so agreement over varying junk is
/// the proof it is unread. Corners beyond +/-2^30 are the largest swept (the
/// port's `i32` addition would overflow near `i32::MAX` where the original
/// wraps).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_space_at_corner.py") else {
        return;
    };
    let sims: Vec<Sim> = (0..GRIDS)
        .map(|g| {
            let (xs, ys) = dims(g);
            let mut w = World::new(xs, ys);
            for y in 0..ys * 4 {
                for x in 0..xs * 4 {
                    w.set_tile_mask(Pos::new(x, y), tile_mask(g, x, y));
                }
            }
            for cy in 0..ys {
                for cx in 0..xs {
                    w.set_owner(Cell::new(cx, cy), cell_who(g, cx, cy), Owner::None);
                }
            }
            Sim::new(Tuning::RON, w, 4)
        })
        .collect();
    let mut n = 0;
    for line in rows.lines() {
        let (a, original) = super::row(line);
        let [g, tx, ty, who, _junk, need] = a[..] else {
            panic!("six arguments in `{line}`");
        };
        let got = sims[g as usize].space_at_corner(who as u8, tx as i32, ty as i32, need != 0);
        assert_eq!(i64::from(got), original, "space_at_corner on row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("space_at_corner: {n} rows agree");
}
