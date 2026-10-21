//! The terrain's height grid, and what a building does to it.
//!
//! `docs/ROADS.md` §7.4. The map's heights live on a **corner** grid of
//! `4·xs + 1` columns — `TerrainData::master_land_heights` — and a tile's
//! own height is the truncated mean of two of them
//! (`TerrainOut::find_tcoord_z@008544a0`, [`crate::world::World::retile_z`]).
//!
//! Until run62 that grid was a table the loader pinned once and nothing
//! ever wrote. It is not: **a building flattens the ground it stands on the
//! moment it is placed**, before it lays its ring and before it plans its
//! road, and the road search prices the climb off the flattened grid. That
//! is what made two of run32's three placement searches cost the wrong
//! number of nodes, and the run62 proxies are what showed it — every node's
//! price, and every difference a multiple of the climb's `× 3`.
//!
//! The arithmetic is the original's own SSE single precision, done in
//! integers by [`Single`] (item 3390, `docs/DECISIONS.md` entry 31's owed
//! upgrade): `addss` in the box's own order, `cvtdq2ps` and `divss` for
//! the mean, `(h + mean) × 0.5` on the border (`00875210`..`008755c4`).
//! Until then it was exact millionths, and the two part on a truncation
//! about three tiles in fifty-eight thousand (`docs/QUEUE.md` item 58);
//! one of them, on the Spanish at Great Lakes, priced a road one climb
//! short on 7984 (`docs/ROADS.md` §7.4).

use crate::Sim;
use crate::single::Single;
use crate::world::{Cell, MOVE_8, OIL, Pos, TILES_PER_CELL, cell, tile};

impl Sim {
    /// `TerrainOut::terraform_for_building@00875210(corner.x, corner.y,
    /// x_size, y_size, …)` — reached from `Terrain::object_placed`, which
    /// is `Wall::start@0063e810`'s first statement after the competing
    /// buildings are killed and **before** `mask_me`, whose own tail is
    /// `place_roads`.
    ///
    /// The box is the footprint grown by one corner on the near side and
    /// two on the far — `[cx − 1, cx + w + 2) × [cy − 1, cy + h + 2)`,
    /// clamped to the grid — and it is walked twice. The first pass takes
    /// the **mean** of every corner in it plus the column and row that
    /// close it; a corner at or below zero anywhere in that sweep abandons
    /// the whole terraform, which is how a building on the shore leaves the
    /// water alone. The second pass writes: the interior takes the mean
    /// outright, the box's own border takes `(h + mean) / 2`, and three
    /// predicates hold a corner back — the cell is water, a mountain or a
    /// cliff tile stands in the corner's own 3×3, or the cell carries a
    /// good.
    pub(crate) fn terraform_for_building(&mut self, corner: Pos, w: i32, h: i32) {
        let (cx, cy) = (corner.x, corner.y);
        let (gw, gh) = (
            self.world.width() * TILES_PER_CELL,
            self.world.height() * TILES_PER_CELL,
        );
        // `param_1 == 0 || param_2 == 0 || param_1 == 4·xs − 1 || param_2
        // == 4·xs − 1` — the original measures both against the *x* limit.
        if cx == 0 || cy == 0 || cx == gw - 1 || cy == gw - 1 {
            return;
        }
        let x0 = (cx - 1).max(0);
        let y0 = (cy - 1).max(0);
        let x1 = (cx + w + 2).min(gw - 1);
        let y1 = (cy + h + 2).min(gh - 1);

        // Pass one: the mean, over the box **and** the column and row that
        // close it, summed in the listing's order. A corner at or below zero
        // abandons the terraform (`comiss` against 0, `jae`).
        let mut sum = Single::ZERO;
        let mut n: i32 = 0;
        let mut exact = true;
        let column = (y0..y1).map(|y| (x1, y));
        let row = (x0..x1).map(|x| (x, y1));
        let inside = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y)));
        for (x, y) in column.chain(row).chain(inside) {
            if let Some((z, known)) = self.world.corner_z(x, y) {
                if !z.gt(Single::ZERO) {
                    return;
                }
                sum = sum.addss(z);
                n += 1;
                exact &= known;
            }
        }
        if n == 0 {
            return;
        }
        let mean = sum.divss(Single::from_i32(n));

        // Pass two: the write, and its three refusals.
        for y in y0..y1 {
            for x in x0..x1 {
                let c = Cell::new(x >> 2, y >> 2);
                let d = self.world.cell_data(c);
                // `WorldData::is_ocean`'s own predicate: a cell of the
                // shallows or the ocean that is not marked coastal is left
                // alone outright.
                if d.flags & cell::HALFLAND == 0 && (d.land == 1 || d.land == 2) {
                    continue;
                }
                // A mountain or a cliff anywhere in the corner's own 3×3
                // holds the corner back.
                if MOVE_8
                    .iter()
                    .copied()
                    .chain(std::iter::once((0, 0)))
                    .any(|(dx, dy)| {
                        let t = Pos::new(x + dx, y + dy);
                        self.world.tile_in_bounds(t)
                            && matches!(
                                self.world.tile_mask(t) & tile::OBJECT,
                                tile::OBJECT_CLIFF | tile::OBJECT_MOUNTAIN
                            )
                    })
                {
                    continue;
                }
                if d.land != 0 {
                    continue;
                }
                if self.world.tile_mask(Pos::new(x, y)) & tile::AS_BUILDING != 0
                    && self.good_here(c)
                {
                    continue;
                }
                let border = y == y0 || y == y1 - 1 || x == x0 || x == x1 - 1;
                let Some((was, known)) = self.world.corner_z(x, y) else {
                    continue;
                };
                let z = if border {
                    was.addss(mean).mulss(Single::HALF)
                } else {
                    mean
                };
                self.world.set_corner_z(x, y, z, exact && known);
            }
        }
        // The tiles the moved corners feed — one further out each way,
        // because a tile reads the corner above it and the one to its right.
        for y in (y0 - 1).max(0)..(y1 + 1).min(gh) {
            for x in (x0 - 1).max(0)..(x1 + 1).min(gw) {
                self.world.retile_z(Pos::new(x, y));
            }
        }
    }

    /// `ObjectsData::find_good_at(x, y, −1, 0, 0)` — the terraform's own
    /// call, whose `who` is `−1` and so asks nothing about availability.
    fn good_here(&self, c: Cell) -> bool {
        self.world
            .good_at(c)
            .is_some_and(|(_, g)| g.alive && g.ty != OIL)
    }
}

#[cfg(test)]
#[allow(clippy::float_arithmetic)] // the oracle: the host's own `f32`
mod tests {
    use crate::single::Single;
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
}
