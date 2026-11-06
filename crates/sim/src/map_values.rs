//! **Map generation's fifth phase** (`docs/MAPGEN.md` §9, §15 step 5):
//! `World::adjust_for_blocking@006b3230` between `map.cpp` 7927 and 7937,
//! which draws nothing, and `World::compute_all_vals@006b3790` between 7937
//! and 7945, one draw a non-water cell. Both are checked at their own
//! checkpoint by parts 5 and 6 (§3), and the values' draws site for site
//! against the setup trace.
//!
//! Every operation is an integer one: the value's divides truncate toward
//! zero (`cltd`/`idiv`, the `0x66666667` and `0x55555556` reciprocals, and
//! the `and 3`/`sar 2` quarters), which Rust's `/` is.

use crate::mapgen::{MapGrid, Regions, WCell};
use crate::scout::circle;
use crate::terrain_groups::{Draws, Template, flag, owner, set_blocked_at, tile_at, tile_inb};

/// The draw site, as the trace's return address: `Random::get`'s return
/// in `World::compute_val@006b0350` (listing 0x6b068e → 0x6b0693).
pub const VAL_SITE: u32 = 0x006b_0350 + 0x343;

/// `move_x/move_y[0..9]` (0xadcaf0, 0xadc400): the cell and its eight
/// neighbours, read off the PE.
const RING: [(i32, i32); 9] = [
    (0, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

/// `orthog_x/orthog_y[1..=4]` (0xadd254, 0xadd214): north, east, south,
/// west.
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// The region flag `adjust_for_blocking` releases on (`Region.flags`,
/// +0x8). [`Regions`] holds no flags, so no region is released here: on
/// Great Sahara 7937 agrees with none released, and the flag's writer is
/// unread (`docs/MAPGEN.md` §9).
pub const RELEASED_REGION: i32 = 0x40;

fn inb(g: &MapGrid, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < g.xs && y < g.ys
}

fn at(g: &MapGrid, x: i32, y: i32) -> usize {
    (y * g.xs + x) as usize
}

/// The 16 tiles of cell `(x, y)` in the original's order: `k & 3` across,
/// `k >> 2` down.
fn tiles_of(x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..16).map(move |k| (4 * x + (k & 3), 4 * y + (k >> 2)))
}

/// **`World::adjust_for_blocking@006b3230`** (`docs/MAPGEN.md` §9), with no
/// draw:
/// 1. y outer, x inner: a cell with forest, mountain or 0x40 marks itself
///    and its eight neighbours 0x200; a forest cell then blocks its 16
///    tiles, takes back the `solid` a tile not yet 0x30 was given, and
///    marks each tile 0x38.
/// 2. The regions flagged [`RELEASED_REGION`] are released (none, here) and
///    `Regions::find_all` re-runs.
/// 3. `Terrain::eliminate_dead_mount_spots@0084fd90`.
/// 4. y outer, x inner: a border cell of open water marks its tiles within
///    two of the tile border 0x2000, counting `bad`; a forest cell whose
///    east or south neighbour carries neither forest nor mountain clears
///    the 0x30 and 0x08 of its tiles on that side, restores their `solid`
///    and unblocks every one that is not mountain art.
/// 5. `World::set_edge_flags@006b0240`: the gather bit 0x8000.
pub fn adjust_for_blocking(g: &mut MapGrid, regions: &mut Regions, templates: &[Template]) {
    for y in 0..g.ys {
        for x in 0..g.xs {
            if g.cells[at(g, x, y)].flags & 0x70 == 0 {
                continue;
            }
            for (dx, dy) in RING {
                let (u, v) = (x + dx, y + dy);
                if inb(g, u, v) {
                    let i = at(g, u, v);
                    g.cells[i].flags |= 0x200;
                }
            }
            if g.cells[at(g, x, y)].flags & flag::FOREST == 0 {
                continue;
            }
            for (tx, ty) in tiles_of(x, y) {
                set_blocked_at(g, tx, ty, true);
                let t = tile_at(g, tx, ty);
                if g.tiles[t] & 0x30 != 0x30 {
                    let c = owner(g, tx, ty);
                    c.solid = c.solid.wrapping_sub(1);
                }
                g.tiles[t] |= 0x38;
            }
        }
    }
    regions.find_all(g);
    eliminate_dead_mount_spots(g);
    let (txs, tys) = g.tile_dims();
    for y in 0..g.ys {
        for x in 0..g.xs {
            let here = g.cells[at(g, x, y)];
            let border = x == 0 || y == 0 || x == g.xs - 1 || y == g.ys - 1;
            if border && here.is_ocean() {
                for (tx, ty) in tiles_of(x, y) {
                    if tx < 2 || ty < 2 || tx > txs - 3 || ty > tys - 3 {
                        let t = tile_at(g, tx, ty);
                        if g.tiles[t] & 0x2000 == 0 {
                            let c = owner(g, tx, ty);
                            c.bad = c.bad.wrapping_add(1);
                        }
                        g.tiles[t] |= 0x2000;
                    }
                }
            }
            if here.flags & flag::FOREST == 0 {
                continue;
            }
            let open = |g: &MapGrid, u: i32, v: i32| {
                inb(g, u, v) && g.cells[at(g, u, v)].flags & 0x30 == 0
            };
            if open(g, x + 1, y) {
                for ty in 4 * y..4 * y + 4 {
                    unforest_tile(g, 4 * x + 3, ty);
                }
            }
            if open(g, x, y + 1) {
                for tx in 4 * x..4 * x + 4 {
                    unforest_tile(g, tx, 4 * y + 3);
                }
            }
        }
    }
    set_edge_flags(g, templates);
}

/// One border tile of a forest cell, in the original's order of writes:
/// a 0x30 tile loses it and gives back its `solid`, the 0x08 goes, and a
/// tile that is not mountain art (`mask & 3 != 2`) is unblocked.
fn unforest_tile(g: &mut MapGrid, tx: i32, ty: i32) {
    let t = tile_at(g, tx, ty);
    if g.tiles[t] & 0x30 == 0x30 {
        g.tiles[t] &= !0x30;
        let c = owner(g, tx, ty);
        c.solid = c.solid.wrapping_add(1);
    }
    g.tiles[t] &= !0x08;
    if g.tiles[t] & 3 != 2 {
        set_blocked_at(g, tx, ty, false);
    }
}

/// **`Terrain::eliminate_dead_mount_spots@0084fd90`**: two passes, y outer
/// and x inner (the pass counter is set to 2 at 0x84fdd5 and decremented at
/// 0x850009, so the loop the decompile prints without an exit ends there),
/// over one visited set that the second pass does not clear. Each mountain
/// cell visits itself and its eight neighbours once a game: a cell none of
/// whose in-bound orthogonal neighbours is open (`flags & 0x30 == 0`) has
/// every unblocked tile blocked, and each block rewrites the cell's flags
/// to mountain (`& 0xffc3 | 0x10`). The `land` byte is read and stored back
/// unchanged (0x84ff6a..0x84ff89).
fn eliminate_dead_mount_spots(g: &mut MapGrid) {
    let mut visited = vec![false; (g.xs * g.xs) as usize];
    for _ in 0..2 {
        for y in 0..g.ys {
            for x in 0..g.xs {
                if g.cells[at(g, x, y)].flags & flag::MOUNTAIN == 0 {
                    continue;
                }
                for (dx, dy) in RING {
                    let (u, v) = (x + dx, y + dy);
                    if !inb(g, u, v) {
                        continue;
                    }
                    let seen = (v * g.xs + u) as usize;
                    if visited[seen] {
                        continue;
                    }
                    let open = ORTHOG.iter().any(|&(ox, oy)| {
                        let (p, q) = (u + ox, v + oy);
                        inb(g, p, q) && g.cells[at(g, p, q)].flags & 0x30 == 0
                    });
                    if !open {
                        for (tx, ty) in tiles_of(u, v) {
                            if g.tiles[tile_at(g, tx, ty)] & 0x4000 != 0 {
                                continue;
                            }
                            set_blocked_at(g, tx, ty, true);
                            let i = at(g, u, v);
                            g.cells[i].flags = (g.cells[i].flags & 0xffc3) | flag::MOUNTAIN;
                        }
                    }
                    visited[seen] = true;
                }
            }
        }
    }
}

/// **`WorldData::has_gather_access@006b4e50`** as `set_edge_flags` calls it
/// (`-1, 0, 1`): a forest (0x30), mountain-art (2) or 1 tile with an
/// in-bound orthogonal neighbour that is neither plain forest
/// (`& 0x30 == 0x20`) nor blocked.
pub(crate) fn has_gather_access(g: &MapGrid, tx: i32, ty: i32) -> bool {
    let m = g.tiles[tile_at(g, tx, ty)];
    if !(m & 0x30 == 0x30 || m & 3 == 2 || m & 3 == 1) {
        return false;
    }
    ORTHOG.iter().any(|&(ox, oy)| {
        let (u, v) = (tx + ox, ty + oy);
        tile_inb(g, u, v) && {
            let n = g.tiles[tile_at(g, u, v)];
            n & 0x30 != 0x20 && n & 0x4000 == 0
        }
    })
}

/// **`World::set_edge_flags@006b0240`**: the gather bit 0x8000 on every
/// tile with gather access, first over each placed range's tiles
/// (`Mountains::set_edge_flags@0089c1a0`, in placement and template
/// order), then the cliffs' (`Cliffs::set_edge_flags@008a85e0`, over
/// `cliff_mining_data`, which only `Cliffs::init` fills, inside
/// `Terrain::init` after 7945: nothing to walk here, §9), then every
/// forest cell's 16 tiles, y outer.
fn set_edge_flags(g: &mut MapGrid, templates: &[Template]) {
    let mark = |g: &mut MapGrid, tx: i32, ty: i32| {
        if has_gather_access(g, tx, ty) {
            let t = tile_at(g, tx, ty);
            g.tiles[t] |= 0x8000;
        }
    };
    for (mx, my, range) in g.mountains.clone() {
        for &(tx, ty) in &templates[range].tiles {
            mark(g, tx + 4 * mx, ty + 4 * my);
        }
    }
    for y in 0..g.ys {
        for x in 0..g.xs {
            if g.cells[at(g, x, y)].flags & flag::FOREST != 0 {
                for (tx, ty) in tiles_of(x, y) {
                    mark(g, tx, ty);
                }
            }
        }
    }
}

/// `LandData::make[4]` (+0x4) of each `Land` record, `rules.xml`'s `LANDS`
/// in file order: the `TypeIndex` of each `MAKE` slot, `TYPE_NONE` (−1)
/// for `none`.
pub type LandMakes = Vec<[i32; 4]>;

/// **`WorldData::get_land@006b4730`** with its third argument 1: the land
/// record a cell is counted as — coast 3, forest 4, mountain or 0x40 5,
/// rock 6 (7 with oil), ocean with oil 7, else its `land`.
fn land_record(c: &WCell) -> i32 {
    let f = c.flags;
    if f & flag::COAST != 0 {
        3
    } else if f & flag::FOREST != 0 {
        4
    } else if f & 0x50 != 0 {
        5
    } else if f & flag::ROCK != 0 {
        6 + i32::from(f & flag::OIL != 0)
    } else if c.is_ocean() && f & flag::OIL != 0 {
        7
    } else {
        i32::from(c.land)
    }
}

/// `land` `SANDY` or `OCEAN`, or HALFLAND: what both of `compute_val`'s
/// loops count as water.
fn wet(c: &WCell) -> bool {
    c.flags & 0x100 != 0 || c.land == 1 || c.land == 2
}

/// **`World::compute_val@006b0350`** (`docs/MAPGEN.md` §9): the cell's
/// `(val, goods)`. Open water draws nothing and is `(0, 0)`. Otherwise:
/// - rings 3 and 4 of the circle tables: every in-bound cell counts, and
///   unless its centre tile carries 0x1000 adds its land record's `make`
///   slots by type and, if wet, moves from the count to the water count;
/// - the 5×5 square (`move_x/y[0..25]`, its own centre included): −4 off
///   the map, −2 a mountain, forest or 0x40 cell (coast exempt) or a dry
///   cell whose `land` is 4 or 5, −1 a wet one;
/// - one draw, `count / 4 + r % 10`, then the water, timber, metal and
///   region-size factors, clamped to a byte.
///
/// `goods` is 0x20 with bit 0, 1, 2 and 4 set where more than four food,
/// timber, wealth and metal slots were counted.
pub fn compute_val(
    g: &MapGrid,
    d: &mut Draws,
    makes: &LandMakes,
    region_size: &[i32],
    x: i32,
    y: i32,
) -> (u8, u8) {
    let here = g.cells[at(g, x, y)];
    if here.is_ocean() {
        return (0, 0);
    }
    let circles = circle::table();
    let mut slots = [0i32; 6];
    let (mut count, mut water) = (0i32, 0i32);
    for i in circles.radius[2]..circles.radius[4] {
        let (u, v) = (
            x + i32::from(circles.x[i as usize]),
            y + i32::from(circles.y[i as usize]),
        );
        if !inb(g, u, v) {
            continue;
        }
        count += 1;
        if g.tiles[tile_at(g, 4 * u + 2, 4 * v + 2)] & 0x1000 != 0 {
            continue;
        }
        let c = g.cells[at(g, u, v)];
        let make = makes[land_record(&c) as usize];
        for (k, n) in slots.iter_mut().enumerate() {
            if k != 3 {
                *n += make.iter().filter(|&&m| m == k as i32).count() as i32;
            }
        }
        if wet(&c) {
            water += 1;
            count -= 1;
        }
    }
    let goods = u8::from(slots[0] > 4)
        | u8::from(slots[1] > 4) << 1
        | u8::from(slots[2] > 4) << 2
        | u8::from(slots[4] > 4) << 4
        | 0x20;
    for dy in -2..=2 {
        for dx in -2..=2 {
            let (u, v) = (x + dx, y + dy);
            if !inb(g, u, v) {
                count -= 4;
                continue;
            }
            let c = g.cells[at(g, u, v)];
            let f = c.flags;
            let rough = f & 0x70 != 0 || (f & flag::ROCK == 0 && (c.land == 4 || c.land == 5));
            if f & flag::COAST == 0 && rough {
                count -= 2;
            }
            if wet(&c) {
                count -= 1;
            }
        }
    }
    let mut v = count / 4 + d.get(VAL_SITE) % 10;
    if water > count / 5 && water < count / 3 {
        v = v * 3 / 2;
    }
    if water >= count / 3 {
        v /= 10;
    }
    if slots[1] > 4 {
        v *= (slots[1] / 4).min(5) + 1;
    }
    if slots[4] != 0 {
        v = v * 3 / 2;
    }
    let size = g.xs * g.ys;
    let s = region_size[here.region as usize];
    if s < size / 128 {
        v /= 2;
    }
    if s < size / 64 {
        v = v * 3 / 4;
    }
    if s < size / 16 {
        v = v * 3 / 4;
    }
    (v.clamp(0, 255) as u8, goods)
}

/// **`World::compute_all_vals@006b3790`**: x outer, y inner — column-major
/// — each cell's `val` and `goods` from [`compute_val`]. A region's size is
/// its record's (+0x14), which the labelling keeps equal to its cells.
pub fn compute_all_vals(g: &mut MapGrid, d: &mut Draws, makes: &LandMakes) {
    let mut region_size = vec![0i32; 0x80];
    for c in &g.cells {
        region_size[c.region as usize] += 1;
    }
    for x in 0..g.xs {
        for y in 0..g.ys {
            let (val, goods) = compute_val(g, d, makes, &region_size, x, y);
            let i = at(g, x, y);
            g.cells[i].val = val;
            g.cells[i].goods = goods;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `rules.xml`'s nine lands as the type indices read them.
    fn makes() -> LandMakes {
        vec![
            [3, 0, -1, -1],
            [-1; 4],
            [-1; 4],
            [-1; 4],
            [1, -1, -1, -1],
            [4, -1, -1, -1],
            [-1; 4],
            [5, -1, -1, -1],
            [4, -1, -1, -1],
        ]
    }

    #[test]
    fn open_water_draws_nothing_and_is_zero() {
        let mut g = MapGrid::new(8, 8);
        g.cells[0].land = 2;
        let mut d = Draws::new(1);
        assert_eq!(compute_val(&g, &mut d, &makes(), &[64; 128], 0, 0), (0, 0));
        assert!(d.log.is_empty());
        g.cells[0].flags |= 0x100;
        compute_val(&g, &mut d, &makes(), &[64; 128], 0, 0);
        assert_eq!(d.log.len(), 1, "a HALFLAND cell draws");
    }

    #[test]
    fn the_walk_is_column_major() {
        let mut g = MapGrid::new(12, 10);
        let mut d = Draws::new(7);
        compute_all_vals(&mut g, &mut d, &makes());
        assert_eq!(d.log.len(), 120);
        assert!(d.log.iter().all(|&(s, _)| s == VAL_SITE));
        let vals = |order: &[(i32, i32)]| {
            let mut d = Draws::new(7);
            let mut out = vec![0u8; 120];
            for &(x, y) in order {
                out[at(&g, x, y)] = compute_val(&g, &mut d, &makes(), &[120; 128], x, y).0;
            }
            out
        };
        let columns: Vec<_> = (0..12).flat_map(|x| (0..10).map(move |y| (x, y))).collect();
        let rows: Vec<_> = (0..10).flat_map(|y| (0..12).map(move |x| (x, y))).collect();
        let ours: Vec<u8> = g.cells.iter().map(|c| c.val).collect();
        assert_eq!(ours, vals(&columns));
        assert_ne!(ours, vals(&rows), "the order reaches the values");
    }

    #[test]
    fn a_dead_mountain_spot_becomes_mountain() {
        // A plain cell walled by mountain and forest on all four sides.
        let mut g = MapGrid::new(5, 5);
        for (x, y) in [(2, 1), (1, 2), (3, 2), (2, 3)] {
            let i = at(&g, x, y);
            g.cells[i].flags = if x == 2 && y == 1 {
                flag::MOUNTAIN
            } else {
                flag::FOREST
            };
        }
        eliminate_dead_mount_spots(&mut g);
        assert_eq!(g.cells[at(&g, 2, 2)].flags & 0x3c, flag::MOUNTAIN);
        assert!(
            tiles_of(2, 2).all(|(tx, ty)| g.tiles[tile_at(&g, tx, ty)] & 0x4000 != 0),
            "every tile blocked"
        );
        assert_eq!(g.cells[at(&g, 0, 0)].flags, 0, "an open cell is left");
    }
}
