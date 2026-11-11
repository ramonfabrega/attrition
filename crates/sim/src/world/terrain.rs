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
use crate::mapgen::halfland::{Combo, Lands, is_combo_acceptable, spot_update_land};
use crate::num::single::Single;
use crate::world::{Cell, MOVE_8, OIL_GOOD, Pos, TILES_PER_CELL, World, cell, tile};

/// **The current tileset's numbers a placed building's ground reads**
/// (`tilesets.cur_tileset`, `TileSet::init@0087b370`): the `TILESET`
/// element's `weatherbase` attribute (`TileSet+0x228`, a `uchar` stored
/// from `get_attrib_num`'s answer, so an absent attribute's −1 is 255),
/// and the numbers `Terrain::verify_land`'s `is_combo_acceptable` reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileSetLand {
    pub weatherbase: u8,
    pub combo: Combo,
}

/// The running world's cells as `verify_land` reads and writes them
/// ([`Lands`]), with its static depth.
struct WorldLands<'a> {
    w: &'a mut World,
    combo: Combo,
    depth: i32,
}

impl Lands for WorldLands<'_> {
    fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.w.contains(Cell::new(x, y))
    }

    fn land_at(&self, x: i32, y: i32) -> (i32, i32) {
        let d = self.w.cell_data(Cell::new(x, y));
        (i32::from(d.land), i32::from(d.land_sub))
    }

    fn put_land(&mut self, x: i32, y: i32, land: i32, sub: i32) {
        let c = Cell::new(x, y);
        let mut d = self.w.cell_data(c);
        if sub >= 0 {
            d.land_sub = sub as u8;
        }
        if land >= 0 {
            d.land = land as i8;
        }
        self.w.set_cell_data(c, d);
    }

    fn acceptable(&mut self, l1: i32, s1: i32, l2: i32, s2: i32) -> i32 {
        // A running world holds lands 0 to 2, whose `num_tiles` the
        // tileset loads; an unloaded entry answers "not acceptable".
        is_combo_acceptable(&self.combo, l1, s1, l2, s2).unwrap_or(0)
    }

    fn coast_base(&self) -> i32 {
        self.combo.coast_base
    }

    fn depth(&mut self) -> &mut i32 {
        &mut self.depth
    }
}

/// `refresh_nearby_tiles`' two `int[5]` offset tables (`.rdata`
/// `0x00b698b0` for x, `0x00b69810` for y, each completed by a stack
/// word): `(0, 0)`, `(1, 0)`, `(0, 1)`, `(2, 0)`, `(0, 2)`, walked along
/// the diagonal `d = −3..=3`.
const NEARBY: [(i32, i32); 5] = [(0, 0), (1, 0), (0, 1), (2, 0), (0, 2)];
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
            .is_some_and(|(_, g)| g.alive && g.ty != OIL_GOOD)
    }

    /// **`Terrain::object_placed@00850c40`'s world half** (`docs/MAPGEN.md`
    /// §35, item 4136): `Wall::start@0063e810`'s call, after the
    /// competing buildings are killed and before `mask_me`, with the
    /// footprint's tile corner, `x_size`, `y_size` and 1. Every cell from
    /// `corner >> 2` to `(corner + size) >> 2` **inclusive** on each axis —
    /// one past the footprint's last tile, so a footprint ending on a cell's
    /// edge reaches the next cell — y outer, that is on the map and
    /// `BASELAND` (`land == 0`) goes through `spot_update_land(x, y, 0,
    /// weatherbase)`, whose `verify_land` then repairs the neighbours (a
    /// cell beside a coast goes back to `COAST_BASE`). The rest of the
    /// function, `invalidate_wcoord`, is the renderer's. Without a tileset
    /// ([`World::land_set`]) it writes nothing.
    ///
    /// `verify_land`'s static depth is taken at 0: each top-level call
    /// returns it there unless a flood passed 500, and the deepest
    /// `weatherbase` flood measured is 7 (`docs/MAPGEN.md` §19.6).
    pub(crate) fn object_placed(&mut self, corner: Pos, w: i32, h: i32) {
        let Some(set) = self.world.land_set() else {
            return;
        };
        let mut g = WorldLands {
            w: &mut self.world,
            combo: set.combo,
            depth: 0,
        };
        for cy in (corner.y >> 2)..=((corner.y + h) >> 2) {
            for cx in (corner.x >> 2)..=((corner.x + w) >> 2) {
                if g.in_bounds(cx, cy) && g.land_at(cx, cy).0 == 0 {
                    spot_update_land(&mut g, cx, cy, 0, i32::from(set.weatherbase));
                }
            }
        }
    }

    /// **`Wall::mark_behind_tiles@0063d230`** (item 4136): the tiles a
    /// building hides, its [`tile::BEHIND`] bit set (`on`) or cleared.
    /// For each `k` from 1 to the type's `BEHIND_HEIGHT` (`BuildType
    /// +0x2dc`, `-1` when absent), the footprint moved `k` tiles up and
    /// `k` left lends its top row — `x_size` tiles at `y = corner.y − k`
    /// from `x = corner.x − k` — and its left column below that —
    /// `y_size − 1` tiles at `x = corner.x − k`: the building's own
    /// rectangle slid back along the diagonal, never its own tiles. A tile
    /// off the tile grid is skipped. `Wall::start@0063e810` sets it as its
    /// last statement; `Wall::close@0063f6e0` clears it for a started
    /// building, and [`Sim::refresh_nearby_tiles`] sets it again for the
    /// active neighbours.
    pub(crate) fn mark_behind_tiles(&mut self, b: usize, on: bool) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let t = &self.tables.build_types[ty];
        let (n, xs, ys) = (t.behind_height, t.x_size, t.y_size);
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let put = |w: &mut World, x: i32, y: i32| {
            let p = Pos::new(x, y);
            if on {
                w.set_tile_bits(p, tile::BEHIND);
            } else {
                w.clear_tile_bits(p, tile::BEHIND);
            }
        };
        for k in 1..=n {
            let y = corner.y - k;
            for x in corner.x - k..corner.x - k + xs {
                put(&mut self.world, x, y);
            }
            let x = corner.x - k;
            for y in corner.y - k + 1..corner.y - k + ys {
                put(&mut self.world, x, y);
            }
        }
    }

    /// **`Wall::refresh_nearby_tiles@0063d0a0`** (item 4136): after a
    /// building's close cleared its [`tile::BEHIND`] tiles, every building
    /// standing in a cell of the diagonal band beside its own — `(cx + d +
    /// ox, cy + d + oy)` for `d` in −3..=3 and the five [`NEARBY`] offsets,
    /// in the cell's object chain — that is **active** (`flags & 4`) marks
    /// its own again. `Wall::close@0063f6e0` runs it last, after
    /// `Object::close`, for every reason but 0 and 5.
    pub(crate) fn refresh_nearby_tiles(&mut self, b: usize) {
        let c = self.buildings[b].pos.cell();
        for d in -3..=3 {
            for (ox, oy) in NEARBY {
                let n = Cell::new(c.x + d + ox, c.y + d + oy);
                if !self.world.contains(n) {
                    continue;
                }
                for o in self.object_chain(n) {
                    if let crate::combat::Obj::Building(o) = o
                        && self.buildings[o].active
                    {
                        self.mark_behind_tiles(o, true);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_arithmetic)] // the oracle: the host's own `f32`
mod tests;
