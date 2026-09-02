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
//! The arithmetic is the original's in **millionths**, not its `f32`. A
//! mean divided in `f32` and one divided exactly part on a truncation
//! about three tiles in fifty-eight thousand (`docs/QUEUE.md` item 58);
//! the sim carries no floats, and the scale the dump prints is the scale
//! this keeps.

use crate::Sim;
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
        // close it. A corner at or below zero abandons the terraform.
        let mut sum: i64 = 0;
        let mut n: i64 = 0;
        for y in y0..y1 {
            match self.world.corner_z(x1, y) {
                Some(z) if z > 0 => {
                    sum += z;
                    n += 1;
                }
                Some(_) => return,
                None => {}
            }
        }
        for x in x0..x1 {
            match self.world.corner_z(x, y1) {
                Some(z) if z > 0 => {
                    sum += z;
                    n += 1;
                }
                Some(_) => return,
                None => {}
            }
        }
        for y in y0..y1 {
            for x in x0..x1 {
                match self.world.corner_z(x, y) {
                    Some(z) if z > 0 => {
                        sum += z;
                        n += 1;
                    }
                    Some(_) => return,
                    None => {}
                }
            }
        }
        if n == 0 {
            return;
        }
        let mean = sum / n;

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
                let Some(was) = self.world.corner_z(x, y) else {
                    continue;
                };
                self.world
                    .set_corner_z(x, y, if border { (was + mean) / 2 } else { mean });
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
