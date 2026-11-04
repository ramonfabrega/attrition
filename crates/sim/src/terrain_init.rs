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
const ORTHOG: [(i32, i32); 5] = [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)];

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

impl HalfLand {
    /// Parts 10–13. An empty list walks as its 4-byte count alone
    /// (`SimpleArray<int>::walk_data@00473120` returns after a zero count),
    /// which is every Great Sahara file's 262,145 at every record. `None`
    /// for a full list: its entries' layout is not read.
    pub fn parts(&self) -> Option<[u32; 4]> {
        let empty = self.locs.is_empty()
            && self.types.is_empty()
            && self.subtypes.is_empty()
            && self.nuke_hits.is_empty();
        empty.then(|| [adler32(&0u32.to_le_bytes()); 4])
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
mod tests {
    use super::*;
    use crate::mapgen::adler_rewrite;

    fn grid(xs: i32, ys: i32) -> MapGrid {
        MapGrid::new(xs, ys)
    }

    fn cell(g: &mut MapGrid, x: i32, y: i32) -> &mut crate::mapgen::WCell {
        let i = at(g, x, y);
        &mut g.cells[i]
    }

    #[test]
    fn a_style_without_sea_map_leaves_the_world_s() {
        assert_eq!(sea_map_after_make(-1, -1), -1);
        assert_eq!(sea_map_after_make(-1, 0), 0);
        assert_eq!(sea_map_after_make(-1, 3), 3);
    }

    /// The rewrite is the sum of the patched run, at every offset.
    #[test]
    fn the_rewrite_is_the_patched_sum() {
        let mut run: Vec<u8> = (0..PART4_LEN as u32)
            .map(|i| (i * 37 % 251) as u8)
            .collect();
        run[PART4_SEA_MAP_AT..PART4_SEA_MAP_AT + 4].copy_from_slice(&(-1i32).to_le_bytes());
        let before = adler32(&run);
        run[PART4_SEA_MAP_AT..PART4_SEA_MAP_AT + 4].copy_from_slice(&7i32.to_le_bytes());
        assert_eq!(
            adler_rewrite(
                before,
                PART4_LEN,
                PART4_SEA_MAP_AT,
                &(-1i32).to_le_bytes(),
                &7i32.to_le_bytes()
            ),
            adler32(&run)
        );
    }

    /// An `OCEAN` cell beside a 0x4 cell is shallow; one two cells off is
    /// still shallow (5×5) and three is not; a `BASELAND` cell is coastal
    /// only within the ring of eight.
    #[test]
    fn the_flags_read_their_rings() {
        let mut g = grid(8, 8);
        cell(&mut g, 4, 4).flags = 0x4;
        for &(x, y) in &[(5, 4), (6, 4), (7, 4)] {
            cell(&mut g, x, y).land = 2;
        }
        let f = coord_flags(&g);
        assert_eq!(f[at(&g, 4, 4)], coord::NONE | coord::HALF);
        assert_eq!(f[at(&g, 5, 4)], coord::OCEAN | coord::SHALLOW);
        assert_eq!(f[at(&g, 6, 4)], coord::OCEAN | coord::SHALLOW);
        assert_eq!(f[at(&g, 7, 4)], coord::OCEAN);
        assert_eq!(f[at(&g, 3, 3)], coord::COASTAL);
        assert_eq!(f[at(&g, 2, 4)], 0);
        assert_eq!(halfland_entered(&g, &f).len(), 3);
    }

    /// A HALFLAND or 0x400 cell is gated; a plain one is not.
    #[test]
    fn halfland_and_0x400_open_the_gate() {
        let mut g = grid(4, 4);
        cell(&mut g, 1, 1).flags = 0x100;
        cell(&mut g, 2, 2).flags = 0x400;
        let f = coord_flags(&g);
        assert_eq!(halfland_entered(&g, &f), vec![(1, 1), (2, 2)]);
        assert!(matches!(
            check_and_set_halfland(&mut g, &mut f.clone()),
            Err(Unported::HalfLand(_))
        ));
    }

    /// An all-`BASELAND` grid passes the phase untouched.
    #[test]
    fn a_dry_grid_is_untouched() {
        let mut g = grid(6, 6);
        g.tiles[3] = 0x38;
        let before = (g.cell_part(), g.tile_part());
        let (sea, f, half) = terrain_init_without_heights(&mut g, 0).expect("nothing unported");
        assert_eq!(sea, 0);
        assert!(f.iter().all(|&x| x == 0));
        assert_eq!((g.cell_part(), g.tile_part()), before);
        assert_eq!(half.parts(), Some([262_145; 4]));
    }

    /// A `SANDY` cell's tiles turn wet, and the wet tiles within the 7×7
    /// of the first dry neighbour gain 0x400.
    #[test]
    fn a_sandy_cell_marks_its_tiles_and_its_coast() {
        let mut g = grid(4, 4);
        cell(&mut g, 1, 1).land = 1;
        let t = tile_at(&g, 4, 4);
        g.tiles[t] = 0x10;
        let mut f = coord_flags(&g);
        check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
        for ty in 4..8 {
            for tx in 4..8 {
                let m = g.tiles[tile_at(&g, tx, ty)];
                assert_eq!(m & 0x30, 0x20, "tile ({tx}, {ty})");
                // Every tile of the cell is within 3 of the dry (3, 3)
                // or of a dry tile its own scan found first.
                assert_ne!(m & 0x400, 0, "tile ({tx}, {ty})");
            }
        }
        assert_eq!(g.tiles[tile_at(&g, 3, 3)], 0);
    }

    /// The square is 7×7: a wet tile three columns from the dry one is
    /// marked, four is not.
    #[test]
    fn the_coast_reaches_three_tiles() {
        let mut g = grid(4, 1);
        for x in 1..4 {
            cell(&mut g, x, 0).land = 1;
        }
        let mut f = coord_flags(&g);
        check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
        assert_ne!(g.tiles[tile_at(&g, 6, 0)] & 0x400, 0);
        assert_eq!(g.tiles[tile_at(&g, 7, 0)] & 0x400, 0);
    }

    /// A tile with no dry tile among its neighbours marks nothing.
    #[test]
    fn a_wet_tile_without_a_dry_neighbour_marks_nothing() {
        let mut g = grid(3, 3);
        for c in g.cells.iter_mut() {
            c.land = 1;
        }
        let mut f = coord_flags(&g);
        check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
        assert!(g.tiles.iter().all(|&m| m == 0x20));
    }

    /// The tail: an `OCEAN` cell with HALFLAND west of it turns shallow.
    /// [`coord_flags`] would gate the HALFLAND cell, so the flags are
    /// handed in clear to reach the tail alone.
    #[test]
    fn an_ocean_cell_beside_halfland_turns_shallow() {
        let mut g = grid(4, 3);
        cell(&mut g, 1, 1).flags = 0x100;
        cell(&mut g, 2, 1).land = 2;
        cell(&mut g, 3, 1).land = 2;
        let mut f = vec![0u16; g.cells.len()];
        check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
        assert_eq!(f[at(&g, 2, 1)], coord::SHALLOW);
        assert_eq!(f[at(&g, 3, 1)], 0);
    }

    /// `orthog[0]` is the cell itself: a HALFLAND `OCEAN` cell with no
    /// HALFLAND neighbour is shallow.
    #[test]
    fn a_halfland_ocean_cell_reads_itself() {
        let mut g = grid(3, 3);
        cell(&mut g, 1, 1).land = 2;
        cell(&mut g, 1, 1).flags = 0x100;
        let mut f = vec![0u16; g.cells.len()];
        check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
        assert_eq!(f[at(&g, 1, 1)], coord::SHALLOW);
        assert_eq!(f.iter().filter(|&&x| x != 0).count(), 1);
    }

    /// Every `BASELAND` cell touching the `OCEAN` one, row by row.
    #[test]
    fn a_beach_cell_is_refused_not_guessed() {
        let mut g = grid(3, 3);
        cell(&mut g, 2, 1).land = 2;
        assert_eq!(
            convert_beach_to_sandy(&g),
            Err(Unported::Beach(vec![
                (1, 0),
                (2, 0),
                (1, 1),
                (1, 2),
                (2, 2)
            ]))
        );
    }
}
