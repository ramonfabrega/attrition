//! **Map generation's sixth phase** (`docs/MAPGEN.md` §10, §15 step 6):
//! the cell writes between `map.cpp` 7945 and 5012 that no height decides.
//!
//! - `Map::make@0068bc90`'s tail, after 7945: `world.sea_map` takes the
//!   style's `SEA_MAP` when it is not negative — part 4 at `setup.cpp` 734.
//! - `TerrainOut::generate_land_lists@0085f6a0` (666 → 699): each cell's
//!   `CoordInfo.flags` (+0x1c), through `TerrainList::add_new_coord_info@0084ba10`.
//!   No checksum part walks them; they gate the half-land pass and feed the
//!   heights (step 7).
//! - `Terrain::check_and_set_halfland@0084e610` (706 → 720) with its
//!   argument 0, as `Terrain::init` calls it: its height-free passes.
//!   `Terrain::check_and_set_halfland_wcoord@0084dec0` — the only reader
//!   of a corner height, and so of `mark_halfwater` — enters a cell only
//!   when its `CoordInfo.flags & 0x84` is set: [`halfland_entered`].
//! - `Terrain::convert_beach_to_sandy@0084b8e0` (815 → 5012): its
//!   predicate.
//!
//! On a map no cell of which is `SANDY`, `OCEAN`, `NONE`, HALFLAND (0x100),
//! 0x400 or 0x4, every arm here but the first is empty, and no float is
//! read: on Great Sahara the flags gate nothing, so the 105 files' parts
//! 2, 5, 6 and 10–13 stand still from 734 to 5012, and the port agrees
//! with all of them.

use crate::mapgen::{MapGrid, adler32};
use crate::terrain_groups::{tile_at, tile_inb};

/// `World.sea_map` (`WorldData +0x34`) before `Map::make`'s tail writes it:
/// −1. Read off the checksums, not a constructor: the part-4 step from 7945
/// to 734 on Great Sahara is exactly these four bytes going to `0`.
pub const WORLD_SEA_MAP_BEFORE: i32 = -1;

/// Part 4's run, `WorldData +0x8 .. +0x80` (`size` through `seed`), and
/// `sea_map`'s place in it.
pub const PART4_LEN: usize = 0x78;
pub const PART4_SEA_MAP_AT: usize = 0x34 - 0x8;

/// `Map::make`'s last write: `if (-1 < map->sea_map) world->sea_map =
/// map->sea_map`. `map->sea_map` is `Map::init_map_data@0069f250`'s
/// `SEA_MAP` attribute, −1 when the style has none (`Map::Map@006a06e0`).
pub fn sea_map_after_make(world: i32, map: i32) -> i32 {
    if map >= 0 { map } else { world }
}

/// `CoordInfo.flags` bits [`coord_flags`] sets.
pub mod coord {
    /// A `NONE` cell, or one whose world flags carry 0x4.
    pub const NONE: u16 = 0x8000;
    /// `NONE`, HALFLAND (0x100) or 0x400: the half-land gate's first bit.
    pub const HALF: u16 = 0x4;
    /// `BASELAND` with a `NONE` or 0x4 cell among its eight neighbours.
    pub const COASTAL: u16 = 0x20;
    /// `OCEAN`.
    pub const OCEAN: u16 = 0x1000;
    /// `OCEAN` with a `NONE` or 0x4 cell within the 5×5 square: the gate's
    /// second bit. [`check_and_set_halfland`]'s tail sets it too.
    pub const SHALLOW: u16 = 0x80;
    /// What `check_and_set_halfland_wcoord` tests: `flags & 0x84`.
    pub const GATE: u16 = HALF | SHALLOW;
    /// Cleared by `convert_beach_to_sandy` on a cell it converts.
    pub const BEACH: u16 = 0x2000;
}

/// `move_x/move_y[0..49]` (0xadcaf0, 0xadc400, read off the PE): the
/// centre, its eight neighbours in `ai_place`'s order, then the 5×5 and
/// 7×7 rings. A reader of `[1..9]` stops at the first match, so that
/// order is the original's; the wider rings are only ever OR-ed over.
pub(crate) const MOVE: [(i32, i32); 49] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -2),
    (0, -2),
    (1, -2),
    (2, -1),
    (2, 0),
    (2, 1),
    (1, 2),
    (0, 2),
    (-1, 2),
    (-2, 1),
    (-2, 0),
    (-2, -1),
    (-2, -2),
    (2, -2),
    (2, 2),
    (-2, 2),
    (-3, -3),
    (-2, -3),
    (-1, -3),
    (0, -3),
    (1, -3),
    (2, -3),
    (3, -3),
    (3, -2),
    (3, -1),
    (3, 0),
    (3, 1),
    (3, 2),
    (3, 3),
    (2, 3),
    (1, 3),
    (0, 3),
    (-1, 3),
    (-2, 3),
    (-3, 3),
    (-3, 2),
    (-3, 1),
    (-3, 0),
    (-3, -1),
    (-3, -2),
];

/// `orthog_x/orthog_y[0..5]` (0xadd250, 0xadd210): the cell itself, then
/// north, east, south, west. `check_and_set_halfland`'s tail reads from 0.
pub(crate) const ORTHOG: [(i32, i32); 5] = [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)];

fn inb(g: &MapGrid, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < g.xs && y < g.ys
}

fn at(g: &MapGrid, x: i32, y: i32) -> usize {
    (y * g.xs + x) as usize
}

/// `add_new_coord_info`'s land: `NONE` (3) when the world flags carry 0x4.
fn coord_land(g: &MapGrid, i: usize) -> i8 {
    let c = &g.cells[i];
    if c.flags & 0x4 != 0 { 3 } else { c.land }
}

/// `NONE` or flagged 0x4: what both of `add_new_coord_info`'s scans look
/// for around a cell.
fn shore(g: &MapGrid, x: i32, y: i32) -> bool {
    let c = &g.cells[at(g, x, y)];
    c.flags & 0x4 != 0 || c.land == 3
}

/// **`CoordInfo.flags` per cell**, as `generate_land_lists` leaves them
/// (`add_new_coord_info@0084ba10`, once per cell; the diagonal walk orders
/// the render lists, never the flags). Indexed `y·xs + x`.
pub fn coord_flags(g: &MapGrid) -> Vec<u16> {
    let mut out = vec![0u16; g.cells.len()];
    for y in 0..g.ys {
        for x in 0..g.xs {
            let i = at(g, x, y);
            let land = coord_land(g, i);
            let flags = g.cells[i].flags;
            let mut f = 0u16;
            if land == 3 {
                f |= coord::NONE | coord::HALF;
            } else if flags & 0x100 != 0 || flags & 0x400 != 0 {
                f |= coord::HALF;
            }
            let near = |ring: &[(i32, i32)]| {
                ring.iter()
                    .any(|&(dx, dy)| inb(g, x + dx, y + dy) && shore(g, x + dx, y + dy))
            };
            match land {
                2 => {
                    f |= coord::OCEAN;
                    if near(&MOVE[1..25]) {
                        f |= coord::SHALLOW;
                    }
                }
                0 if near(&MOVE[1..9]) => f |= coord::COASTAL,
                _ => {}
            }
            out[i] = f;
        }
    }
    out
}

/// The cells `check_and_set_halfland_wcoord` enters: `CoordInfo.flags &
/// 0x84`. Each reads its sixteen sub-tiles' corner heights against −30.0f
/// and may call `mark_halfwater` — step 7's, since a height decides it.
pub fn halfland_entered(g: &MapGrid, coord: &[u16]) -> Vec<(i32, i32)> {
    (0..g.ys)
        .flat_map(|y| (0..g.xs).map(move |x| (x, y)))
        .filter(|&(x, y)| coord[at(g, x, y)] & coord::GATE != 0)
        .collect()
}

/// An arm this port does not build, with the cells that would enter it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unported {
    /// `check_and_set_halfland_wcoord`'s body: the heights (step 7).
    HalfLand(Vec<(i32, i32)>),
    /// `check_and_set_halfland`'s relabel of a `BASELAND` cell standing in
    /// a region above 0x40, which moves it between `Region` lists.
    Relabel(Vec<(i32, i32)>),
    /// `convert_beach_to_sandy`'s write, `spot_update_land`, whose
    /// `verify_land@0084d8a0` is not read.
    Beach(Vec<(i32, i32)>),
}

/// The half-land lists parts 10–13 walk (`docs/MAPGEN.md` §3): one
/// `WCoordList` and three `SimpleArray<int>`. `Terrain::init` empties
/// them; only `mark_halfwater` (step 7) appends.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HalfLand {
    pub locs: Vec<(i32, i32)>,
    pub types: Vec<i32>,
    pub subtypes: Vec<i32>,
    pub nuke_hits: Vec<i32>,
}

/// One list's walk (`Array<WCoordData>::walk_data@00478990`,
/// `SimpleArray<int>::walk_data@00473120`): an empty list is its 4-byte
/// zero count alone; a full one is `num` (+0x4), `size` (+0x8), the 2-byte
/// grow step (+0xc), the flag byte (+0x14) `& 0xbf`, then the entries.
/// `TerrainData::TerrainData@0049bdd0` builds all four with grow −1 and
/// flags 0, and `increase_size` at grow −1 takes 4, then the current size
/// again, so `size` is the first `4·2^k ≥ num`.
fn list_part(entries: &[u8], num: usize) -> u32 {
    let mut b = (num as u32).to_le_bytes().to_vec();
    if num > 0 {
        let mut size = 4u32;
        while (size as usize) < num {
            size *= 2;
        }
        b.extend_from_slice(&size.to_le_bytes());
        b.extend_from_slice(&0xffffu16.to_le_bytes());
        b.push(0);
        b.extend_from_slice(entries);
    }
    adler32(&b)
}

impl HalfLand {
    /// Parts 10–13 ([`list_part`]): `halfland_locs` as `(x, y)` pairs of
    /// `i32`, the other three as `i32`s. Every Great Sahara file's 262,145
    /// at every record is four empty lists.
    pub fn parts(&self) -> Option<[u32; 4]> {
        let ints = |v: &[i32]| v.iter().flat_map(|n| n.to_le_bytes()).collect::<Vec<u8>>();
        let locs: Vec<u8> = self
            .locs
            .iter()
            .flat_map(|&(x, y)| x.to_le_bytes().into_iter().chain(y.to_le_bytes()))
            .collect();
        Some([
            list_part(&locs, self.locs.len()),
            list_part(&ints(&self.types), self.types.len()),
            list_part(&ints(&self.subtypes), self.subtypes.len()),
            list_part(&ints(&self.nuke_hits), self.nuke_hits.len()),
        ])
    }
}

/// **`Terrain::check_and_set_halfland(0)`'s height-free passes**, in
/// order, on the grid and `coord` ([`coord_flags`]):
/// 1. (y outer, x inner) each cell not HALFLAND whose `land` is `SANDY` or
///    `OCEAN` marks its sixteen tiles `& 0xffef | 0x20`. The original runs
///    `check_and_set_halfland_wcoord` on the cell first, and then a second
///    sweep over the cells it queued; with no cell entered (refused
///    otherwise, [`Unported::HalfLand`]) both are this pass alone.
/// 2. `Terrain::check_and_set_coastal@0084dd70` on every cell.
/// 3. A `BASELAND` cell in a region above 0x40 is relabelled
///    ([`Unported::Relabel`]).
/// 4. A cell flagged 0x4 or `OCEAN` with a HALFLAND cell at itself or an
///    orthogonal neighbour sets `CoordInfo` 0x80.
///
/// `TerrainOut::sort_waterline_wcoords` follows; it is the renderer's.
pub fn check_and_set_halfland(g: &mut MapGrid, coord: &mut [u16]) -> Result<(), Unported> {
    let entered = halfland_entered(g, coord);
    if !entered.is_empty() {
        return Err(Unported::HalfLand(entered));
    }
    for y in 0..g.ys {
        for x in 0..g.xs {
            let c = g.cells[at(g, x, y)];
            if c.flags & 0x100 == 0 && (c.land == 1 || c.land == 2) {
                for k in 0..16 {
                    let t = tile_at(g, 4 * x + (k & 3), 4 * y + (k >> 2));
                    g.tiles[t] = g.tiles[t] & 0xffef | 0x20;
                }
            }
        }
    }
    for y in 0..g.ys {
        for x in 0..g.xs {
            check_and_set_coastal(g, x, y);
        }
    }
    let relabel: Vec<(i32, i32)> = (0..g.ys)
        .flat_map(|y| (0..g.xs).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let c = &g.cells[at(g, x, y)];
            c.land == 0 && c.region > 0x40
        })
        .collect();
    if !relabel.is_empty() {
        return Err(Unported::Relabel(relabel));
    }
    for y in 0..g.ys {
        for x in 0..g.xs {
            let c = &g.cells[at(g, x, y)];
            if c.flags & 0x4 == 0 && c.land != 2 {
                continue;
            }
            let half = ORTHOG.iter().any(|&(dx, dy)| {
                inb(g, x + dx, y + dy) && g.cells[at(g, x + dx, y + dy)].flags & 0x100 != 0
            });
            if half {
                coord[at(g, x, y)] |= coord::SHALLOW;
            }
        }
    }
    Ok(())
}

/// **`Terrain::check_and_set_halfland(0)` on a map whose gate opens**:
/// the two sweeps of `_wcoord` on the 706 heights `z`
/// ([`crate::halfland::Pass::sweeps`]), then passes 2–4 of
/// [`check_and_set_halfland`] — the coast, the relabel and the 0x80 tail.
/// Returns the half-land lists parts 10–12 walk. `depth` is
/// `verify_land`'s static, carried in and out.
pub fn check_and_set_halfland_heights(
    g: &mut MapGrid,
    coord: &mut [u16],
    z: &[crate::single::Single],
    combo: crate::halfland::Combo,
    depth: &mut i32,
) -> Result<HalfLand, crate::halfland::Unported> {
    let mut pass = crate::halfland::Pass::new(g, coord, z, combo);
    pass.depth = *depth;
    pass.sweeps()?;
    *depth = pass.depth;
    let lists = std::mem::take(&mut pass.lists);
    for y in 0..g.ys {
        for x in 0..g.xs {
            check_and_set_coastal(g, x, y);
        }
    }
    relabel(g);
    shallow_tail(g, coord);
    Ok(lists)
}

/// `check_and_set_halfland`'s third pass: a `BASELAND` cell in a water
/// region (above 0x40) takes its `region2` and keeps the water region as
/// `region2` when it has one; with none it takes the region of the first
/// `move[1..25]` neighbour that is `BASELAND` in a land region not its own.
/// The `Regions` lists' bookkeeping is not held; no cell write depends on
/// it (the search stops at the first such neighbour either way).
fn relabel(g: &mut MapGrid) {
    for y in 0..g.ys {
        for x in 0..g.xs {
            let i = at(g, x, y);
            let c = g.cells[i];
            if c.land != 0 || c.region <= 0x40 {
                continue;
            }
            if c.region2 != 0 {
                g.cells[i].region = c.region2;
                g.cells[i].region2 = c.region;
                continue;
            }
            for &(dx, dy) in &MOVE[1..25] {
                let (u, v) = (x + dx, y + dy);
                if !inb(g, u, v) {
                    continue;
                }
                let n = g.cells[at(g, u, v)];
                if n.land == 0 && n.region < 0x41 && n.region != c.region {
                    g.cells[i].region = n.region;
                    break;
                }
            }
        }
    }
}

/// `check_and_set_halfland`'s fourth pass: a cell flagged 0x4 or `OCEAN`
/// with a HALFLAND cell at `orthog[0..5]` sets `CoordInfo` 0x80.
fn shallow_tail(g: &MapGrid, coord: &mut [u16]) {
    for y in 0..g.ys {
        for x in 0..g.xs {
            let c = &g.cells[at(g, x, y)];
            if c.flags & 0x4 == 0 && c.land != 2 {
                continue;
            }
            let half = ORTHOG.iter().any(|&(dx, dy)| {
                inb(g, x + dx, y + dy) && g.cells[at(g, x + dx, y + dy)].flags & 0x100 != 0
            });
            if half {
                coord[at(g, x, y)] |= coord::SHALLOW;
            }
        }
    }
}

/// A tile `check_and_set_halfland`'s first pass marked wet: `mask & 0x30
/// == 0x20`.
fn wet_tile(g: &MapGrid, tx: i32, ty: i32) -> bool {
    g.tiles[tile_at(g, tx, ty)] & 0x30 == 0x20
}

/// **`Terrain::check_and_set_coastal@0084dd70`**: each of the cell's
/// sixteen tiles (row outer) that is wet takes its first in-bound dry
/// neighbour (`move[1..9]`, off-map neighbours skipped), and every wet
/// tile in that neighbour's 7×7 square gains 0x400.
fn check_and_set_coastal(g: &mut MapGrid, x: i32, y: i32) {
    for sy in 0..4 {
        for sx in 0..4 {
            let (tx, ty) = (4 * x + sx, 4 * y + sy);
            if !wet_tile(g, tx, ty) {
                continue;
            }
            let dry = MOVE[1..9]
                .iter()
                .map(|&(dx, dy)| (tx + dx, ty + dy))
                .find(|&(u, v)| tile_inb(g, u, v) && !wet_tile(g, u, v));
            let Some((u, v)) = dry else {
                continue;
            };
            for &(dx, dy) in &MOVE[1..49] {
                let (p, q) = (u + dx, v + dy);
                if tile_inb(g, p, q) && wet_tile(g, p, q) {
                    let t = tile_at(g, p, q);
                    g.tiles[t] |= 0x400;
                }
            }
        }
    }
}

/// **`Terrain::convert_beach_to_sandy@0084b8e0`'s predicate** (y outer, x
/// inner): a `BASELAND` cell with an `OCEAN` cell among its eight
/// neighbours becomes `SANDY` through `spot_update_land(x, y, 1, 0)` and
/// clears `CoordInfo` 0x2000. The write is refused, not guessed:
/// `spot_update_land` ends in `verify_land`, which is not read
/// ([`Unported::Beach`]). No cell enters on a map without `OCEAN`.
pub fn convert_beach_to_sandy(g: &MapGrid) -> Result<(), Unported> {
    let beach: Vec<(i32, i32)> = (0..g.ys)
        .flat_map(|y| (0..g.xs).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            g.cells[at(g, x, y)].land == 0
                && MOVE[1..9].iter().any(|&(dx, dy)| {
                    inb(g, x + dx, y + dy) && g.cells[at(g, x + dx, y + dy)].land == 2
                })
        })
        .collect();
    if beach.is_empty() {
        Ok(())
    } else {
        Err(Unported::Beach(beach))
    }
}

/// **The phase, 7945 → 5012**, on a grid that has stood through
/// `compute_all_vals`: returns `world.sea_map` after `Map::make`, the
/// `CoordInfo` flags and the half-land lists, or the first arm the grid
/// enters that is not built.
pub fn terrain_init_without_heights(
    g: &mut MapGrid,
    map_sea_map: i32,
) -> Result<(i32, Vec<u16>, HalfLand), Unported> {
    let sea_map = sea_map_after_make(WORLD_SEA_MAP_BEFORE, map_sea_map);
    let mut coord = coord_flags(g);
    check_and_set_halfland(g, &mut coord)?;
    convert_beach_to_sandy(g)?;
    Ok((sea_map, coord, HalfLand::default()))
}

#[cfg(test)]
mod tests;
