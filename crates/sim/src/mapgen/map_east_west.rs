//! **East Meets West's land/sea mask** (`docs/MAPGEN.md` §24, §5, §15 step
//! 1): `MapEastMeetsWest::make_continents@00696640`, on the region growers
//! Great Lakes' port carries (`crate::mapgen::map_lakes`), and the three arms no
//! other ported style runs — `Map::fill_cont@0068a960`'s continents,
//! `Map::find_region_centroid@0068ae50` and
//! `Map::eliminate_edge_canals@0068b360` — with the start placement's
//! `Map::place_start_in_region@0068ac00`, `Map::is_near_ocean@0068b0a0` and
//! `Map::check_player_land@0068ef00`'s guarded arm.
//!
//! **The held-out style** (item 3941, DECISIONS 54, 61): written from the
//! decompile and the listing before its take was launched, and scored on
//! that take's first comparison; nothing here is changed by what the take
//! shows (§24).
//!
//! One continent a side, opened on a ring about the map's centre and grown
//! in two `grow_region` passes; a pass that stalls wipes the world and
//! starts again, loosening `avoid_continent` every fourth stall. Then the
//! pools are merged into land, the canals a cell from an edge are widened,
//! the regions are relabelled, and each seat's start is sought on its
//! continent: a random coordinate of the region first, then two walks out
//! along the continent's angle, then the region's list. No float: the
//! distances are the engine's integer `vector_dist`, the angles its integer
//! `find_angle` and `project`.

use crate::mapgen::map_lakes::{
    Grower, LakeMap, LakeStyle, Lakes, eliminate_pools, scale_land_area,
};
use crate::mapgen::terrain_groups::Draws;
use crate::mapgen::{MapGrid, Regions, project};
use crate::movement::{Angle, find_angle};
use crate::units::think::scout::circle;
use crate::world::vector_dist;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle, its high half…
    pub const ANGLE_HI: u32 = 0x0069_68a4;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_68bc;
    /// A lone player's offset from its continent's angle.
    pub const LONE: u32 = 0x0069_6f87;
    /// `Map::fill_cont`: one per ordered pair of continents of equal count.
    pub const FILL_SWAP: u32 = 0x0068_ab64;
    /// `Map::place_start_in_region`: the list index each pass starts at.
    pub const REGION_START: u32 = 0x0068_ac90;
}

/// `LeaderData::get_team`'s "no team".
pub const NO_TEAM: i32 = 8;

/// What the mask leaves besides the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EastWestMap {
    /// `avoid_continent` as the last generation left it, and `avoid_modify`.
    pub map: LakeMap,
    /// The generations a stalled `grow_region` pass threw away.
    pub stalls: u32,
    /// The generations a seat with no start threw away.
    pub unplaced: u32,
    /// Each continent's centroid, as `find_region_centroid` gave it.
    pub centroids: Vec<(i32, i32)>,
    /// The seats' starts, in seat order.
    pub starts: Vec<(i32, i32)>,
}

/// `move_x`/`move_y` (0xadcaf0, 0xadc400): the engine's hand-typed square
/// spiral, ring `r` ending at `MOVE_RADIUS[r]` = (2r+1)². Rings 1 and 2 are
/// not in scan order and rings 8–10 carry the source's typos (ring 8 ends on
/// (−8, −16), ring 9 on (−9, −10) and repeats (9, 9), ring 10 ends on
/// (−10, 0)); they are copied as the image holds them, since
/// `check_player_land` walks them.
pub(crate) const SPIRAL_X: [i8; 441] = [
    0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2, 2, -2, -3, -2,
    -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, -1, -2, -3, -3, -3, -3, -3, -3, -4, -3, -2, -1, 0,
    1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0, -1, -2, -3, -4, -4, -4, -4, -4, -4, -4, -4, -5,
    -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4,
    -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -6, -6, -6, -6, -6, -6,
    -6, -6, -6, -6, -6, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    7, 7, 7, 7, 7, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -7, -7, -7, -7, -7, -7, -7,
    -7, -7, -7, -7, -7, -7, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -8,
    -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0,
    1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4,
    3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9,
    -9, -9, -9, -9, -9, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 8, 7, 6, 5,
    4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -10, -10, -10, -10, -10, -10, -10, -10,
    -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10,
];
pub(crate) const SPIRAL_Y: [i8; 441] = [
    0, -1, -1, -1, 0, 1, 1, 1, 0, -2, -2, -2, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, 2, 2, -3, -3,
    -3, -3, -3, -3, -3, -2, -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, -1, -2, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -3, -2, -1, 0, 1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0, -1, -2, -3, -5,
    -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -6, -6, -6, -6, -6, -6, -6, -6, -6, -6, -6, -6, -6, -5,
    -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 5, 4, 3, 2, 1, 0, -1,
    -2, -3, -4, -5, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -7, -6, -5, -4, -3, -2,
    -1, 0, 1, 2, 3, 4, 5, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 5, 4, 3, 2, 1, 0, -1,
    -2, -3, -4, -5, -6, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -8, -7, -6,
    -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -16, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9,
    -9, -9, -9, -9, -9, -9, -9, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3,
    -4, -5, -6, -7, -8, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10, -10,
    -10, -10, -10, -10, -10, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 8, 7, 6,
    5, 4, 3, 2, 1, 0, -1, -2, -3, -4, -5, -6, -7, -8, -9, 0,
];

/// `radius` (0xadd1e0): where each ring of the spiral ends.
pub(crate) const MOVE_RADIUS: [i32; 11] = [1, 9, 25, 49, 81, 121, 169, 225, 289, 361, 441];

/// **`Map::fill_cont@0068a960`**: which continent each team takes, and how
/// many players each continent holds. `teams[seat]` is `get_team` for a live
/// seat. The teams no live seat names are listed in order; each live seat
/// with no team takes the next continent at `cont[free[k]]` (`k` the count
/// of such seats so far) and that continent counts 1; each seat with a team
/// opens `cont[team]` on its first member and counts every member. Then, for
/// every ordered pair of continents `(i, j)`, `i ≠ j`, with the same non-zero
/// count, one draw: an odd one swaps the two in `cont`, and an empty `cont`
/// slot is written by both arms, so it ends as `i`.
pub fn fill_cont(teams: &[Option<i32>; 8], draws: &mut Draws) -> ([i32; 8], [i32; 8]) {
    let free: Vec<i32> = (0..8).filter(|&t| !teams.contains(&Some(t))).collect();
    let (mut count, mut cont) = ([0i32; 8], [-1i32; 8]);
    let (mut next, mut lone) = (0, 0usize);
    for &team in teams.iter().flatten() {
        if team == NO_TEAM {
            let slot = free[lone] as usize;
            cont[slot] = next;
            next += 1;
            lone += 1;
            count[cont[slot] as usize] = 1;
        } else {
            let t = team as usize;
            if cont[t] == -1 {
                cont[t] = next;
                next += 1;
            }
            count[cont[t] as usize] += 1;
        }
    }
    for i in 0..8 {
        if count[i] == 0 {
            continue;
        }
        for j in 0..8 {
            if j == i || count[j] != count[i] || draws.get(site::FILL_SWAP) & 1 == 0 {
                continue;
            }
            for c in &mut cont {
                let v = *c;
                if v == i as i32 || v == -1 {
                    *c = j as i32;
                }
                if v == j as i32 || v == -1 {
                    *c = i as i32;
                }
            }
        }
    }
    (count, cont)
}

/// **`Map::find_region_centroid@0068ae50`**: the mean of a region's
/// coordinates, each axis summed in 32 bits and divided toward zero by the
/// count. The original unrolls the sums by eight and then by two; the order
/// of a wrapping sum does not change it.
pub fn find_region_centroid(points: &[(i32, i32)]) -> (i32, i32) {
    let n = points.len() as i32;
    let (sx, sy) = points.iter().fold((0i32, 0i32), |(sx, sy), &(x, y)| {
        (sx.wrapping_add(x), sy.wrapping_add(y))
    });
    (sx / n, sy / n)
}

/// **`Map::eliminate_edge_canals@0068b360`**: along each edge, wherever
/// three edge cells in a row are `land` 2 and the middle one's inward
/// neighbour is `land` 0, that neighbour becomes `land` 2 (the 16-bit store,
/// so `land_sub` 0); and wherever three first-row-in cells in a row are then
/// water (the row as read, with that write) and the middle one's next
/// inward neighbour is `land` 0, it becomes `land` 2 too. West, east,
/// north, south. Then `Regions::clear_all` and `find_all`, returned.
///
/// The original reads the raw `land` byte, not `is_ocean`, and walks every
/// edge `xs` long with `xs` as the row count too: the south edge is row
/// `xs − 1`. On a square map that is the edge; on any other the walk is the
/// original's, as written.
pub fn eliminate_edge_canals(grid: &mut MapGrid) -> Regions {
    let xs = grid.xs;
    let land = |g: &MapGrid, i: i32| g.cells[i as usize].land;
    let flood = |g: &mut MapGrid, i: i32| {
        let c = &mut g.cells[i as usize];
        c.land = 2;
        c.land_sub = 0;
    };
    // Each edge: the edge cell, its first and second neighbours inward, as
    // flat indices of the walk's step `i`.
    let edges: [&dyn Fn(i32) -> [i32; 3]; 4] = [
        &|i| [xs * i, xs * i + 1, xs * i + 2],
        &|i| [xs * i + xs - 1, xs * i + xs - 2, xs * i + xs - 3],
        &|i| [i, xs + i, 2 * xs + i],
        &|i| [(xs - 1) * xs + i, (xs - 2) * xs + i, (xs - 3) * xs + i],
    ];
    for at in edges {
        let mut edge = vec![false; xs.max(0) as usize];
        let mut first = vec![false; xs.max(0) as usize];
        for i in 0..xs {
            let [e, f, _] = at(i);
            edge[i as usize] = land(grid, e) == 2;
            first[i as usize] = land(grid, f) == 2;
        }
        let mut i = 0;
        while i + 2 < xs {
            let u = i as usize;
            if edge[u] && edge[u + 1] && edge[u + 2] {
                let [_, f, s] = at(i + 1);
                if land(grid, f) == 0 {
                    first[u + 1] = true;
                    flood(grid, f);
                }
                if first[u] && first[u + 1] && first[u + 2] && land(grid, s) == 0 {
                    flood(grid, s);
                }
            }
            i += 1;
        }
    }
    let mut regions = Regions::default();
    regions.clear_all(grid);
    regions.find_all(grid);
    regions
}

/// Region `id`'s coordinate list as `Regions::rebuild_coords@0067f800`
/// leaves it: every cell of the region, y outer and x inner.
pub(crate) fn coords(grid: &MapGrid, id: i16) -> Vec<(i32, i32)> {
    (0..grid.ys)
        .flat_map(|y| (0..grid.xs).map(move |x| (x, y)))
        .filter(|&(x, y)| grid.cell(x, y).region == id)
        .collect()
}

fn in_bounds(grid: &MapGrid, x: i32, y: i32) -> bool {
    (0..grid.xs).contains(&x) && (0..grid.ys).contains(&y)
}

/// **`Map::is_near_ocean@0068b0a0`**: `(x, y)` on the grid; when `far` is
/// not 0, a cell of `land` other than 0 in the circle's rings `far − 1`
/// through `far` (both under 0x41 with `far + 1`); and when `near` is not 0,
/// no such cell within ring `near` (at most 0x40), the centre included.
pub fn is_near_ocean(grid: &MapGrid, x: i32, y: i32, near: i32, far: i32) -> bool {
    if !in_bounds(grid, x, y) {
        return false;
    }
    let t = circle::table();
    let wet = |i: usize| {
        let (px, py) = (x + i32::from(t.x[i]), y + i32::from(t.y[i]));
        in_bounds(grid, px, py) && grid.cell(px, py).land != 0
    };
    if far != 0 {
        // `far − 1 < 0x41 && far + 1 < 0x41`: the second is the bound.
        if far + 1 >= 0x41 {
            return false;
        }
        let (lo, hi) = (t.radius[(far - 1) as usize], t.radius[(far + 1) as usize]);
        if !(lo..hi).any(|i| wet(i as usize)) {
            return false;
        }
    }
    if near != 0 {
        if near > 0x40 {
            return false;
        }
        if (t.radius[0]..t.radius[near as usize]).any(|i| wet(i as usize)) {
            return false;
        }
    }
    true
}

/// The least `vector_dist` from `(x, y)` to a start already placed, 9999
/// with none.
fn least_dist(grid: &MapGrid, x: i32, y: i32) -> i32 {
    grid.starts[0]
        .list
        .iter()
        .zip(&grid.starts[1].list)
        .map(|(&sx, &sy)| vector_dist(x - sx, y - sy))
        .fold(9999, i32::min)
}

/// **`Map::place_start_in_region@0068ac00`** with the world's start arrays:
/// two passes over region `id`'s list, each from an index one draw picks
/// (none for a one-cell list) and on round to it. A cell takes the start
/// when it is inside the edge band — 5 to `size − 6` on the first pass, 3 to
/// `size − 4` on the second — at least `min_dist` from every start (on the
/// second pass at least `min(6, min_dist)`), and `is_near_ocean(…, 3, 9)`
/// (the second pass `(…, 2, 0)`: only the dry ring).
pub fn place_start_in_region(
    grid: &MapGrid,
    id: i16,
    min_dist: i32,
    draws: &mut Draws,
) -> Option<(i32, i32)> {
    let list = coords(grid, id);
    let n = list.len() as i32;
    for pass in 1..=2 {
        let (edge, high) = if pass == 1 { (5, 6) } else { (3, 4) };
        let start = if n == 1 || n - 1 < 0 {
            0
        } else {
            draws.get(site::REGION_START) % n
        };
        if n == 0 {
            continue;
        }
        let mut i = start;
        loop {
            i = if i + 1 < n { i + 1 } else { 0 };
            let (x, y) = list[i as usize];
            if edge <= x && edge <= y && x <= grid.xs - high && y <= grid.ys - high {
                let lim = if pass == 2 { min_dist.min(6) } else { min_dist };
                let near = if pass == 1 { (3, 9) } else { (2, 0) };
                if least_dist(grid, x, y) >= lim && is_near_ocean(grid, x, y, near.0, near.1) {
                    return Some((x, y));
                }
            }
            if i == start {
                break;
            }
        }
    }
    None
}

/// The region a cell holds, read where the original reads it: unchecked.
fn region_at(grid: &MapGrid, (x, y): (i32, i32)) -> i16 {
    grid.cells[(y * grid.xs + x) as usize].region
}

/// One seat's start when `place_start_in_region` finds none: the listing's
/// three walks (0x697003..0x69744d). `None` sends the original back to a
/// fresh generation.
///
/// First, from the continent's centroid along `angle` at `3·half/2`,
/// stepping in by 2: a cell of the region, at least 8 from every start and
/// `is_near_ocean(…, 3, 9)` takes it while under 30 tries; a radius that
/// falls under 3 turns the angle by 0x71c71c6 and starts again at `half`,
/// three times at most. Then along the first angle from `3·half/2`, in by
/// 2, at least 6 and `is_near_ocean(…, 2, 7)`, under 30 tries. Then the
/// region's list: each cell is taken while the point last held — the last
/// projected, or the last taken — is at least 8 from every start, so the
/// last cell taken is the start.
fn walk_out(
    grid: &MapGrid,
    id: i16,
    centroid: (i32, i32),
    angle: i32,
    half: i32,
) -> Option<(i32, i32)> {
    let (cx, cy) = centroid;
    let far = half * 3 / 2;
    let fits = |x: i32, y: i32, least: i32, near: i32, ring: i32| {
        in_bounds(grid, x, y)
            && grid.cell(x, y).region == id
            && least_dist(grid, x, y) >= least
            && is_near_ocean(grid, x, y, near, ring)
    };
    let (mut tries, mut turns, mut a, mut radius) = (0, 0, angle, far);
    loop {
        tries += 1;
        let (x, y) = project(cx, cy, Angle(a), radius);
        if fits(x, y, 8, 3, 9) {
            if tries < 30 {
                return Some((x, y));
            }
            break;
        }
        radius -= 2;
        if radius < 3 && turns < 3 {
            a = a.wrapping_add(0x071c_71c6);
            tries = 0;
            turns += 1;
            radius = half;
            continue;
        }
        if tries >= 30 {
            break;
        }
    }
    let (mut tries, mut radius) = (0, far);
    let mut last;
    loop {
        tries += 1;
        let (x, y) = project(cx, cy, Angle(angle), radius);
        last = (x, y);
        if fits(x, y, 6, 2, 7) {
            if tries < 30 {
                return Some((x, y));
            }
            break;
        }
        radius -= 2;
        if tries >= 30 {
            break;
        }
    }
    let list = coords(grid, id);
    let mut took = false;
    for cell in list {
        if grid.starts[0].list.is_empty() || least_dist(grid, last.0, last.1) >= 8 {
            last = cell;
            took = true;
        }
    }
    took.then_some(last)
}

/// **`Map::check_player_land@0068ef00`** as East Meets West calls it: ring
/// 5 (the `-1` argument), the guard on (`ecx` 1) at `avoid_continent`
/// (`edx`). `lists` is each region's coordinate list (its `num` at +0x70 is
/// what the guard reads).
///
/// For each start-city cell: a cell of region 0 first looks for the first
/// spiral neighbour (rings 1–3) of region 1..0x3f, and, finding one, makes
/// its own 3×3 land and moves each of those cells' list entries to that
/// region — never writing their `region`. Then every circle cell to ring 5
/// round the city cell becomes land of the city cell's region (`region`
/// written, appended to its list) unless a spiral neighbour within ring
/// `min(avoid_continent, 10)` that is not water belongs to another region
/// whose list holds more than four cells.
pub(crate) fn check_player_land(grid: &mut MapGrid, lists: &mut [Vec<(i32, i32)>], avoid: i32) {
    let t = circle::table();
    let ring = 5usize;
    let guard = MOVE_RADIUS[avoid.min(10) as usize];
    let cities: Vec<(i32, i32)> = grid.starts[2]
        .list
        .iter()
        .zip(&grid.starts[3].list)
        .map(|(&x, &y)| (x, y))
        .collect();
    for (cx, cy) in cities {
        let mut own = i32::from(region_at(grid, (cx, cy)));
        if own == 0 {
            let found = (1..=48).find_map(|d| {
                let (x, y) = (cx + i32::from(SPIRAL_X[d]), cy + i32::from(SPIRAL_Y[d]));
                let r = in_bounds(grid, x, y).then(|| i32::from(grid.cell(x, y).region))?;
                (1..=0x3f).contains(&r).then_some(r)
            });
            if let Some(to) = found {
                for d in 0..9 {
                    let (x, y) = (cx + i32::from(SPIRAL_X[d]), cy + i32::from(SPIRAL_Y[d]));
                    if !in_bounds(grid, x, y) {
                        continue;
                    }
                    let c = grid.cell_mut(x, y);
                    c.land = 0;
                    c.land_sub = 0;
                    let from = &mut lists[own as usize];
                    if let Some(k) = from.iter().position(|&p| p == (x, y)) {
                        from.remove(k);
                    }
                    lists[to as usize].push((x, y));
                    own = to;
                }
            }
        }
        for i in t.radius[0] as usize..t.radius[ring] as usize {
            let (x, y) = (cx + i32::from(t.x[i]), cy + i32::from(t.y[i]));
            if !in_bounds(grid, x, y) {
                continue;
            }
            let guarded = (1..guard as usize).any(|j| {
                let (nx, ny) = (x + i32::from(SPIRAL_X[j]), y + i32::from(SPIRAL_Y[j]));
                if !in_bounds(grid, nx, ny) || grid.cell(nx, ny).is_ocean() {
                    return false;
                }
                let r = i32::from(grid.cell(nx, ny).region);
                r != own && lists[r as usize].len() > 4
            });
            if guarded {
                continue;
            }
            let c = grid.cell_mut(x, y);
            c.land = 0;
            c.land_sub = 0;
            c.region = own as i16;
            lists[own as usize].push((x, y));
        }
    }
}

/// **`MapEastMeetsWest::make_continents@00696640`** (`docs/MAPGEN.md` §24).
///
/// `avoid_continent` becomes `scale_land_area(avoid_continent)` held to
/// 4..16 once; a stall counter starts at 4, or at 0 when that value is 12 or
/// more, and on the stalls that bring it to 4, 8 and 12 `avoid_continent`
/// becomes 12, 8 and 4. Each generation wipes the world and the regions,
/// runs [`fill_cont`], and draws the ring's angle, `(r % 0xffff) << 16 + r %
/// 0xffff`. The ring's radius is `3·(m/2)/4`, `m` the shorter side; each
/// continent `k` opens region `k + 1` on the ring about `(xs/2, ys/2)`, and
/// the angle steps by half of `count[k]` and half of `count[k + 1]` shares
/// of `0xffffffff / players`. Then `grow_region` takes every continent to
/// `count·(A/2)` and then to `count·A` within `3·radius/4` of its first
/// cell, `A = scale_land_area(5·(cells / players) / 9)`; a pass that ends
/// short is a stall.
///
/// Then each continent's [`find_region_centroid`], `eliminate_pools` over
/// the world, and [`eliminate_edge_canals`]. Each live seat in seat order
/// takes its continent — a seat with no team `cont[seat]`, one with a team
/// the first continent whose centroid's region is that of `cont[team]`'s —
/// reads that region `R`, and its angle: `find_angle(xs/2 − cx, xs/2 − cy)`
/// from the centroid, less `(n/2)·spacing` for `n` players on it (spacing 0,
/// 0x2aaaaaaa under four, else 0x6aaaaaaa / n), or for a lone player plus
/// `0xaaaaaaa − r % 0x15555555`. The start is [`place_start_in_region`] at
/// `scale_land_area(scale_by_area(12))`, else the three walks of
/// [`walk_out`] with `half` the ring's radius over 2; a seat neither places
/// starts a fresh generation. Last, [`check_player_land`].
///
/// `teams[seat]` is `get_team` for each live seat; `sides` is the game's
/// `num_sides`. `None` when the lobby's nations pass the map size's
/// allowance, where `scale_land_area` takes its float path.
pub fn east_west_make_continents(
    grid: &mut MapGrid,
    players: i32,
    sides: i32,
    teams: &[Option<i32>; 8],
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<EastWestMap> {
    let allow = style.overage_allowance;
    let scaled = scale_land_area(style.avoid_continent, players, allow)?.max(4);
    let mut tier = if scaled >= 12 { 0 } else { 4 };
    let step = (u32::MAX / players.max(1) as u32) as i32;
    let sides = sides.max(1);
    let mut g = Grower {
        grid,
        lakes: Lakes::cleared(),
        draws,
        style,
        map: LakeMap {
            avoid_continent: scaled.min(16),
            avoid_modify: 0,
        },
    };
    let (xs, ys) = (g.grid.xs, g.grid.ys);
    let (mut stalls, mut unplaced) = (0, 0);
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let (count, cont) = fill_cont(teams, g.draws);
        let ring = xs.min(ys) / 2 * 3 / 4;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = hi.wrapping_mul(0x10000).wrapping_add(lo);
        let size = scale_land_area(g.size() / players * 5 / 9, players, allow)?;
        for k in 0..sides {
            let (x, y) = project(xs / 2, ys / 2, Angle(angle), ring);
            g.make_region(k + 1, x, y);
            let share = |c: i32| ((c.wrapping_mul(step)) as u32 >> 1) as i32;
            let next = ((k + 1) % sides) as usize;
            angle = angle
                .wrapping_add(share(count[k as usize]))
                .wrapping_add(share(count[next]));
        }
        let reach = ring * 3 / 4;
        for pass in [size / 2, size] {
            for k in 0..sides {
                if g.grow_region(k + 1, pass * count[k as usize], reach) {
                    stalls += 1;
                    tier += 1;
                    match tier {
                        12 => g.map.avoid_continent = 4,
                        8 => g.map.avoid_continent = 8,
                        4 => g.map.avoid_continent = 12,
                        _ => {}
                    }
                    continue 'generation;
                }
            }
        }
        let centroids: Vec<(i32, i32)> = (1..=sides)
            .map(|k| find_region_centroid(&g.lakes.points[k as usize]))
            .collect();
        eliminate_pools(g.grid);
        let regions = eliminate_edge_canals(g.grid);
        let half = ring / 2;
        let min_dist = scale_land_area(g.scale_by_area(12), players, allow)?;
        let mut starts = Vec::new();
        for (seat, team) in teams.iter().enumerate() {
            let Some(team) = *team else { continue };
            let k = if team == NO_TEAM {
                match cont[seat] {
                    -1 => seat as i32,
                    c => c,
                }
            } else {
                let p = match cont[team as usize] {
                    -1 => seat as i32,
                    c => c,
                };
                let want = region_at(g.grid, centroids[p as usize]);
                (0..=sides)
                    .find(|&j| {
                        centroids
                            .get(j as usize)
                            .is_some_and(|&c| region_at(g.grid, c) == want)
                    })
                    .unwrap_or(sides)
            };
            let (cx, cy) = centroids[k as usize];
            let id = region_at(g.grid, (cx, cy));
            let mut a = find_angle(xs / 2 - cx, xs / 2 - cy).0;
            let n = count[k as usize];
            let spacing = match n {
                1 => 0,
                n if n < 4 => 0x2aaa_aaaa,
                n => 0x6aaa_aaaa / n,
            };
            if n > 1 {
                a = a.wrapping_sub((n >> 1).wrapping_mul(spacing));
            } else {
                let r = g.draws.get(site::LONE) % 0x1555_5555;
                a = a.wrapping_add(0x0aaa_aaaa - r);
            }
            let placed = place_start_in_region(g.grid, id, min_dist, g.draws)
                .or_else(|| walk_out(g.grid, id, (cx, cy), a, half));
            let Some((x, y)) = placed else {
                unplaced += 1;
                continue 'generation;
            };
            g.grid.add_starting_location(x, y);
            starts.push((x, y));
        }
        let mut lists: Vec<Vec<(i32, i32)>> = (0..128).map(|r| coords(g.grid, r as i16)).collect();
        debug_assert!(
            (0..128).all(|r| lists[r].len() as i32 == regions.count[r]),
            "rebuild_coords' lists are find_all's counts"
        );
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        return Some(EastWestMap {
            map: g.map,
            stalls,
            unplaced,
            centroids,
            starts,
        });
    }
}

#[cfg(test)]
mod tests;
