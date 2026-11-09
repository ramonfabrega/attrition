use crate::num::single::Single;
use crate::world::{Cell, Pos, TILES_PER_CELL, Terrain, World};
use crate::{Sim, Tuning};

/// **The terraform is the original's SSE, bit for bit** (item 3390):
/// a grid of singles, twenty building boxes flattened one after
/// another, against the host's own `f32` doing the listing's
/// operations in the listing's order — an oracle in the arithmetic
/// the sim refuses, which is the only way to know it agrees.
#[test]
fn a_terraform_lands_on_the_original_s_singles() {
    let cells = 8;
    let mut world = World::new(cells, cells);
    world.fill_region(
        Terrain::Land,
        Cell::new(0, 0),
        Cell::new(cells - 1, cells - 1),
    );
    let mut sim = Sim::new(Tuning::RON, world, 2);
    let side = (cells * TILES_PER_CELL + 1) as usize;
    let mut seed: u32 = 0x1234_5678;
    let mut next = || {
        seed = seed.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
        seed >> 8
    };
    // Heights from 16 up, where a six-decimal print names one single.
    let mut host: Vec<f32> = (0..side * side)
        .map(|_| 16.0 + (next() % 200_000) as f32 / 997.0)
        .collect();
    let grid = host
        .iter()
        .map(|&v| (f64::from(v) * 1e6).round() as i64)
        .collect();
    assert!(sim.world.set_corner_grid(grid));
    for _ in 0..20 {
        let (cx, cy) = (2 + (next() % 22) as i32, 2 + (next() % 22) as i32);
        let (w, h) = (1 + (next() % 4) as i32, 1 + (next() % 4) as i32);
        sim.terraform_for_building(Pos::new(cx, cy), w, h);
        // The host's own singles, in `00875210`'s order.
        let s = side as i32;
        let (x0, y0) = (cx - 1, cy - 1);
        let (x1, y1) = ((cx + w + 2).min(s - 2), (cy + h + 2).min(s - 2));
        let at = |x: i32, y: i32| (y * s + x) as usize;
        let mut sum = 0.0f32;
        let mut n = 0;
        for y in y0..y1 {
            sum += host[at(x1, y)];
            n += 1;
        }
        for x in x0..x1 {
            sum += host[at(x, y1)];
            n += 1;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                sum += host[at(x, y)];
                n += 1;
            }
        }
        let mean = sum / n as f32;
        for y in y0..y1 {
            for x in x0..x1 {
                let border = y == y0 || y == y1 - 1 || x == x0 || x == x1 - 1;
                host[at(x, y)] = if border {
                    (host[at(x, y)] + mean) * 0.5
                } else {
                    mean
                };
            }
        }
    }
    for y in 0..side as i32 {
        for x in 0..side as i32 {
            let (z, exact) = sim.world.corner_z(x, y).expect("on the grid");
            let want = host[y as usize * side + x as usize];
            assert_eq!(z, Single::from_bits(want.to_bits()), "corner ({x}, {y})");
            assert!(exact, "corner ({x}, {y}) is known");
        }
    }
    for ty in 0..side as i32 - 1 {
        for tx in 0..side as i32 - 1 {
            let a = host[(ty as usize + 1) * side + tx as usize];
            let b = host[ty as usize * side + tx as usize + 1];
            assert_eq!(
                sim.world.tile_z(Pos::new(tx, ty)),
                ((a + b) * 0.5) as i32,
                "tile ({tx}, {ty})"
            );
        }
    }
}
