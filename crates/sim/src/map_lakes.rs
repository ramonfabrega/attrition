//! **Great Lakes' land/sea mask** (`docs/MAPGEN.md` §5, §15 step 1):
//! `MapGreatLakes::make_continents@00699e40` and the region growers it
//! drives — `Map::land_dist`, `Map::grow_valid` with its persistent
//! `avoid_modify`, `Map::make_region`, `Map::grow_region`, `Map::point`,
//! `Map::stamp` and `Map::large_stamp` — then the drawless tail:
//! `Map::eliminate_pools`, `Map::invert_land`, the starts walked in from
//! outside the map, and `Map::check_player_land`.
//!
//! The lakes are grown as *land* regions on a wiped (all-water) world and
//! the world is inverted at the end, so a lake's cells end as water and the
//! wiped background as land. No float: `grow_valid`'s and `grow_region`'s
//! distances are the engine's integer `vector_dist`, and the lake spacing's
//! square root is the function's own integer Newton loop.

use crate::ai_place::{MOVE_X, MOVE_Y};
use crate::mapgen::{MapGrid, Regions, project};
use crate::movement::Angle;
use crate::scout::circle;
use crate::terrain_groups::Draws;
use crate::world::vector_dist;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// `MapGreatLakes::make_continents`: the angle's two halves, the lake
    /// count's parity, each lake centre's x and y, and each later size.
    pub const ANGLE_HI: u32 = 0x0069_9f8d;
    pub const ANGLE_LO: u32 = 0x0069_9fa5;
    pub const COUNT: u32 = 0x0069_a007;
    pub const CENTRE_X: u32 = 0x0069_a17a;
    pub const CENTRE_Y: u32 = 0x0069_a1a6;
    pub const SIZE: u32 = 0x0069_a44f;
    /// `Map::grow_region`: the `count / 8` gate, the outer pick's jitter,
    /// the ring pick, and the whole-region pick.
    pub const GATE: u32 = 0x0069_c860;
    pub const OUTER: u32 = 0x0069_c8ae;
    pub const RING: u32 = 0x0069_c9d0;
    pub const ANY: u32 = 0x0069_ca04;
    /// `Map::large_stamp`: the case, and case 4's two side draws.
    pub const STAMP_CASE: u32 = 0x0069_cb53;
    pub const STAMP_UP: u32 = 0x0069_ce2b;
    pub const STAMP_DOWN: u32 = 0x0069_ce6a;
    /// `Map::grow_valid`: one per non-zero `avoid_edge` reached.
    pub const EDGE: u32 = 0x0069_d2b0;
}

/// The `Map` fields the mask reads, as `Map::init_map_data@0069f250` leaves
/// them for the style, and two lobby facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LakeStyle {
    /// `avoid_center` (+0x2c): `AVOID_CENTER`, scaled.
    pub avoid_center: i32,
    /// `avoid_continent` (+0x30): `AVOID_CONTINENT`, scaled, at least
    /// `rules.xml`'s `MIN_CONTINENT_SPACING`.
    pub avoid_continent: i32,
    /// `avoid_edge[4]` (+0x34): `AVOID_EDGE_0..3`, scaled.
    pub avoid_edge: [i32; 4],
    /// `base_edge` (+0x44): `Map::make`'s `r % 4` when the style leaves −1.
    pub base_edge: i32,
    /// `rules.xml`'s Standard map size, which `Map::scale_by_area` measures
    /// against.
    pub standard: i32,
    /// `check_player_land`'s ring: unit type 349's `max_range` (+0x1fc),
    /// a quarter of it rounded up (24 in the shipped rules, so 6).
    pub start_land: i32,
    /// `Map::get_player_overage@0068b280`'s allowance for the lobby's map
    /// size (2, 3, 4, 6 or 8 nations).
    pub overage_allowance: i32,
}

/// The `Map` fields the mask writes: `avoid_continent` after
/// `scale_land_area`, and `avoid_modify` (+0x64) as `grow_valid` leaves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LakeMap {
    pub avoid_continent: i32,
    pub avoid_modify: i32,
}

/// `Regions`' 64 land records as the growers use them: each record's
/// coordinate list (+0x7c), whose length is the record's count (+0x14).
/// `region_flags` is 0 on every corpus style, so the adjacency masks
/// (+0x24) `point` would keep are never written, and every record shares
/// flag bit 1 with none.
pub(crate) struct Lakes {
    pub(crate) points: Vec<Vec<(i32, i32)>>,
}

impl Lakes {
    /// `Regions::clear_all@00680060`'s reset of the 64 records: every
    /// coordinate list empty.
    pub(crate) fn cleared() -> Self {
        Lakes {
            points: vec![Vec::new(); 64],
        }
    }
}

/// One run of the growers: the grid, the regions, the stream and the map.
pub(crate) struct Grower<'a> {
    pub(crate) grid: &'a mut MapGrid,
    pub(crate) lakes: Lakes,
    pub(crate) draws: &'a mut Draws,
    pub(crate) style: LakeStyle,
    pub(crate) map: LakeMap,
}

impl Grower<'_> {
    pub(crate) fn size(&self) -> i32 {
        self.grid.xs * self.grid.ys
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        (0..self.grid.xs).contains(&x) && (0..self.grid.ys).contains(&y)
    }

    /// A region number on the grid that is neither empty nor `id`.
    fn other(region: i16, id: i32) -> bool {
        region != 0 && i32::from(region) != id
    }

    /// **`Map::grow_valid@0069d000`**: whether region `id` may take `(x,
    /// y)`. No draw unless the region tests pass: another live region and
    /// the point within `avoid_center` of the map's centre refuses; so does
    /// another region inside the disc of `avoid_continent` round the point.
    /// Then each non-zero `avoid_edge`, in slot order, takes one draw that
    /// nudges `avoid_modify` by `r % 3 − 1` (clamped to 0..4) and tests the
    /// edge `(base_edge + slot) & 3` at `avoid_edge + avoid_modify`; the
    /// first edge that fails ends the walk.
    pub(crate) fn grow_valid(&mut self, id: i32, x: i32, y: i32) -> bool {
        let ac = self.map.avoid_continent;
        if ac > 0x40 {
            return false;
        }
        let (xs, ys) = (self.grid.xs, self.grid.ys);
        let live = (1..64).any(|k| k != id && !self.lakes.points[k as usize].is_empty());
        if live && vector_dist(x - xs / 2, y - ys / 2) < self.style.avoid_center {
            return false;
        }
        let t = circle::table();
        for i in 0..t.radius[ac as usize] as usize {
            let (px, py) = (x + i32::from(t.x[i]), y + i32::from(t.y[i]));
            if self.in_bounds(px, py) && Self::other(self.grid.cell(px, py).region, id) {
                return false;
            }
        }
        for slot in 0..4 {
            let edge = self.style.avoid_edge[slot];
            if edge == 0 {
                continue;
            }
            let r = self.draws.get(site::EDGE);
            self.map.avoid_modify = (self.map.avoid_modify + r % 3 - 1).clamp(0, 4);
            let v = self.map.avoid_modify + edge;
            let refused = match (self.style.base_edge + slot as i32) & 3 {
                0 => x < v,
                1 => y < v,
                2 => xs - x <= v,
                _ => ys - y <= v,
            };
            if refused {
                return false;
            }
        }
        true
    }

    /// **`Map::point@0069d5a0`**: an empty cell on the grid that
    /// `grow_valid` passes joins region `id` — `land` and `land_sub` 0 (the
    /// 16-bit store at +2), its `region`, and the record's next coordinate.
    /// Only a `grow_valid` refusal returns false: a cell off the grid or
    /// already taken is a success that changes nothing.
    fn point(&mut self, id: i32, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) || self.grid.cell(x, y).region != 0 {
            return true;
        }
        if !self.grow_valid(id, x, y) {
            return false;
        }
        let c = self.grid.cell_mut(x, y);
        c.land = 0;
        c.land_sub = 0;
        c.region = id as i16;
        self.lakes.points[id as usize].push((x, y));
        true
    }

    /// **`Map::stamp@0069cf80`**: the point and its four orthogonal
    /// neighbours, east, west, north, south, stopping at the first refusal.
    fn stamp(&mut self, id: i32, x: i32, y: i32) -> bool {
        self.point(id, x, y)
            && self.point(id, x + 1, y)
            && self.point(id, x - 1, y)
            && self.point(id, x, y - 1)
            && self.point(id, x, y + 1)
    }

    /// A chain of stamps at offsets from `(x, y)`, stopping at the first
    /// refusal.
    fn stamps(&mut self, id: i32, x: i32, y: i32, at: &[(i32, i32)]) -> bool {
        at.iter().all(|&(dx, dy)| self.stamp(id, x + dx, y + dy))
    }

    /// **`Map::large_stamp@0069cb20`**: a stamp, then on success one draw
    /// `r & 15` picks one of nine shapes of further stamps (seven to
    /// fifteen choose none); shape 4 draws twice more for the side its last
    /// two stamps lean.
    fn large_stamp(&mut self, id: i32, x: i32, y: i32) -> bool {
        if !self.stamp(id, x, y) {
            return false;
        }
        match self.draws.get(site::STAMP_CASE) & 15 {
            0 => self.stamps(
                id,
                x,
                y,
                &[(-1, -1), (-2, -2), (-4, -1), (-4, 0), (-5, -1), (-5, -2)],
            ),
            1 => self.stamps(id, x, y, &[(1, 1), (2, 2), (4, 1), (4, 0), (5, 1), (5, 2)]),
            2 => self.stamps(
                id,
                x,
                y,
                &[(1, 0), (1, -2), (1, 2), (2, -4), (3, -4), (4, 1)],
            ),
            3 => self.stamps(
                id,
                x,
                y,
                &[(-1, 0), (-1, -2), (-1, 2), (-2, 4), (-3, 4), (-4, -1)],
            ),
            4 => {
                if !self.stamps(
                    id,
                    x,
                    y,
                    &[(0, -1), (0, 1), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0)],
                ) {
                    return false;
                }
                let side = |r: i32| if r & 1 != 0 { 1 } else { -1 };
                let up = side(self.draws.get(site::STAMP_UP));
                if !self.stamp(id, x + up, y - 1) {
                    return false;
                }
                let down = side(self.draws.get(site::STAMP_DOWN));
                self.stamp(id, x + down, y + 1)
            }
            5 => self.stamps(id, x, y, &[(1, -2), (2, -4)]),
            6 => self.stamps(id, x, y, &[(-1, -2), (-2, -4)]),
            7 => self.stamps(id, x, y, &[(-1, 2), (-2, 4)]),
            8 => self.stamps(id, x, y, &[(1, 2), (2, 4)]),
            _ => true,
        }
    }

    /// **`Map::make_region@0069d3f0`**: record `id` opens on `(x, y)` —
    /// the cell's `land` 0 and `region` `id`, the record's first coordinate,
    /// count 1.
    pub(crate) fn make_region(&mut self, id: i32, x: i32, y: i32) {
        let c = self.grid.cell_mut(x, y);
        c.land = 0;
        c.land_sub = 0;
        c.region = id as i16;
        self.lakes.points[id as usize] = vec![(x, y)];
    }

    /// **`Map::scale_by_area@006a0910`** of `n`: `n` cells of the standard
    /// map, on this one, rounded, at least 1.
    fn scale_by_area(&self, n: i32) -> i32 {
        if n < 1 {
            return 0;
        }
        let s2 = self.style.standard * self.style.standard;
        ((s2 / 2 + self.size() * n) / s2).max(1)
    }

    /// **`Map::grow_region@0069c600`** with the centre left to the region's
    /// first cell and no return value, as `make_continents` calls it: grow
    /// region `id` to `size` cells no further than `max_dist` from it.
    ///
    /// A one-cell region first takes the disc to ring `k` (the first `k ≥
    /// 3` with `3k² ≥ size / 4`, else 2) where `grow_valid` passes, each
    /// cell tested twice since `point` asks again. Then each step picks one
    /// of the region's cells: past `scale_by_area(512)` cells, one region
    /// in `count / 8` (one draw) picks in its newest eighth (a second draw)
    /// within `max_dist`; otherwise, while ring `k` has not been filled,
    /// any cell (one draw), else one of ring `k`'s cells onward (one draw).
    /// The pick takes a cross of points while the region is within 64 of
    /// `size`, a `large_stamp` before that. A step that grows nothing
    /// counts: `8·size` of them in all, or `size` in a row, ends it.
    ///
    /// True when it ends short of `size` — an empty region, or either
    /// stall — which is the original's non-zero return with `param_6` 0:
    /// the record's count, or 1 for an empty one. Great Lakes drops it;
    /// East Indies restarts on it.
    pub(crate) fn grow_region(&mut self, id: i32, size: i32, max_dist: i32) -> bool {
        let t = circle::table();
        let mut k: i32 = 2;
        if size / 4 > 12 {
            loop {
                if k > 0x3f {
                    break;
                }
                k += 1;
                if k * k * 3 >= size / 4 {
                    break;
                }
            }
        }
        let ku = k as usize;
        let count = |g: &Self| g.lakes.points[id as usize].len() as i32;
        if count(self) == 0 {
            return true;
        }
        let (ox, oy) = self.lakes.points[id as usize][0];
        let (cx, cy) = (ox, oy);
        let ring_base = if count(self) < 2 {
            for i in 1..t.radius[ku].max(1) as usize {
                let (px, py) = (ox + i32::from(t.x[i]), oy + i32::from(t.y[i]));
                if self.in_bounds(px, py) && self.grow_valid(id, px, py) {
                    self.point(id, px, py);
                }
            }
            t.radius[ku - 1]
        } else {
            0
        };
        let ring_len = t.radius[ku] - t.radius[ku - 1];
        let (mut idle, mut stalled) = (0, 0);
        while count(self) < size {
            let c0 = count(self);
            let mut outer = None;
            if c0 >= self.scale_by_area(0x200) && c0 / 8 > 1 {
                let gate = c0 / 8;
                if self.draws.get(site::GATE) % gate != 0 {
                    let eighth = (c0 + 7) / 8;
                    let jitter = if eighth > 1 {
                        self.draws.get(site::OUTER) % eighth
                    } else {
                        0
                    };
                    let p = (c0 * 7 / 8 + jitter).min(c0 - 1).max(0);
                    let (px, py) = self.lakes.points[id as usize][p as usize];
                    if vector_dist(px - cx, py - cy) <= max_dist {
                        outer = Some(p);
                    }
                }
            }
            let pick = match outer {
                Some(p) => p,
                None if ring_base == 0 || c0 - ring_base < ring_len => {
                    if c0 > 1 {
                        self.draws.get(site::ANY) % c0
                    } else {
                        0
                    }
                }
                None => {
                    let n = c0 - ring_base;
                    ring_base
                        + if n > 1 {
                            self.draws.get(site::RING) % n
                        } else {
                            0
                        }
                }
            };
            let (x, y) = self.lakes.points[id as usize][pick as usize];
            if size - c0 < 0x40 {
                let _ = self.point(id, x, y)
                    && self.point(id, x + 1, y)
                    && self.point(id, x - 1, y)
                    && self.point(id, x, y - 1)
                    && self.point(id, x, y + 1);
            } else {
                self.large_stamp(id, x, y);
            }
            if count(self) == c0 {
                idle += 1;
                if idle >= size * 8 {
                    return true;
                }
                stalled += 1;
                if stalled >= size {
                    return true;
                }
            } else {
                stalled = 0;
            }
        }
        false
    }

    /// **`Map::land_dist@0069d970`** with its edge flag set: 0 on a cell
    /// that is not water, else the first ring round it that holds a cell off
    /// the grid, a HALFLAND cell or a cell that is not water (0x41 past
    /// ring 0x40).
    pub(crate) fn land_dist(&self, x: i32, y: i32) -> i32 {
        if !self.grid.cell(x, y).is_ocean() {
            return 0;
        }
        let t = circle::table();
        for r in 1..=0x40 {
            let (lo, hi) = t.ring(r);
            for i in lo as usize..hi as usize {
                let (px, py) = (x + i32::from(t.x[i]), y + i32::from(t.y[i]));
                if !self.in_bounds(px, py) || !self.grid.cell(px, py).is_ocean() {
                    return r;
                }
            }
        }
        0x41
    }
}

/// `Map::get_player_overage@0068b280`'s gate on `Map::scale_land_area`:
/// at or under the map size's allowance the value is returned as given;
/// past it the original divides in floats, which no corpus lobby reaches,
/// so this refuses.
pub(crate) fn scale_land_area(v: i32, players: i32, allowance: i32) -> Option<i32> {
    (players == 0 || players <= allowance).then_some(v)
}

/// The integer square root `make_continents` takes of a lake's spacing
/// term: Newton's iteration from `n / 2`, stopping when it no longer falls;
/// `n` itself for `n ≤ 1`.
pub(crate) fn isqrt(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    let mut x = n >> 1;
    loop {
        let r = x;
        x = (r + n / r) >> 1;
        if x >= r {
            return r;
        }
    }
}

/// **`Map::eliminate_pools@0068b890`** over the whole world, its cell
/// writes: the regions relabelled, then every water region but one merged
/// into land (`land` 0) — the smallest first (the first of equals, in
/// first-seen scan order), until one is left. Each merged region's cells
/// take the first land region a cell of it touches; `invert_land` zeroes
/// every `region` straight after, so only the `land` writes reach 7807.
fn eliminate_pools(grid: &mut MapGrid) {
    let mut regions = Regions::default();
    regions.clear_all(grid);
    regions.find_all(grid);
    let mut seas: Vec<i16> = Vec::new();
    for c in &grid.cells {
        if c.region > 0x40 && !seas.contains(&c.region) {
            seas.push(c.region);
        }
    }
    while seas.len() > 1 {
        let mut least = grid.xs * grid.xs;
        let mut pick = None;
        for (i, &s) in seas.iter().enumerate() {
            if regions.count[s as usize] < least {
                least = regions.count[s as usize];
                pick = Some(i);
            }
        }
        let Some(i) = pick else { break };
        let s = seas.remove(i);
        let mut into = s;
        'cells: for (x, y) in (0..grid.ys).flat_map(|y| (0..grid.xs).map(move |x| (x, y))) {
            if grid.cell(x, y).region != s {
                continue;
            }
            for d in 1..=8 {
                if let Some(n) = grid.get(x + MOVE_X[d], y + MOVE_Y[d])
                    && n.region != s
                    && n.region < 0x41
                {
                    into = n.region;
                    break 'cells;
                }
            }
        }
        for c in grid.cells.iter_mut().filter(|c| c.region == s) {
            c.land = 0;
            c.land_sub = 0;
            c.region = into;
        }
        regions.count[into as usize] += regions.count[s as usize];
        regions.count[s as usize] = 0;
    }
}

/// **`MapGreatLakes::make_continents@00699e40`** (`docs/MAPGEN.md` §5):
/// the lakes, grown as land regions on a wiped world and inverted.
///
/// Three opening draws: the angle's halves and the lake count, `xs / 12 +
/// (r & 1)`. Each lake's centre is drawn `(r % xs, r % ys)` until
/// `land_dist` reaches `max(3, (avoid_continent + 1) / 2 + isqrt(7·size /
/// 22))`; one that `grow_valid` passes opens a region grown to the lake's
/// size. The first lake is `max(90, cells / 20)` cells; after every
/// attempt the next is `small + r % (first − small)` with `small = max(24,
/// cells / 40)`. A thousand misses shrink the lake to `small`, then
/// `small` by 13/16 while it stays over 19; a hundred attempts in a row
/// that open nothing end the lakes. Then `eliminate_pools`,
/// `invert_land`, each start walked in from `3·xs/2` along its angle in
/// steps of 3 to the first land cell two in from every edge, and
/// `check_player_land`'s disc of land round each start-city cell.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn great_lakes_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<LakeMap> {
    let avoid_continent =
        scale_land_area(style.avoid_continent, players, style.overage_allowance)?.max(6);
    grid.wipe();
    grid.clear_regions();
    let mut g = Grower {
        grid,
        lakes: Lakes::cleared(),
        draws,
        style,
        map: LakeMap {
            avoid_continent,
            avoid_modify: 0,
        },
    };
    let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
    let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
    let mut angle = hi.wrapping_mul(0x10000).wrapping_add(lo);
    let (xs, ys) = (g.grid.xs, g.grid.ys);
    let first = (g.size() / 20).max(90);
    let mut small = (g.size() / 40).max(24);
    let (mut lake, mut big) = (first, first);
    let mut left = xs / 12 + (g.draws.get(site::COUNT) & 1);
    let (mut last_id, mut fails) = (0, 0);
    let draw_mod = |g: &mut Grower<'_>, at: u32, n: i32| {
        if n == 1 || n - 1 < 0 {
            0
        } else {
            g.draws.get(at) % n
        }
    };
    'lakes: loop {
        loop {
            if left == 0 {
                break 'lakes;
            }
            fails += 1;
            if fails > 99 {
                break 'lakes;
            }
            let spacing = ((g.map.avoid_continent + 1) / 2 + isqrt(lake * 7 / 22)).max(3);
            let mut misses = 0;
            let (x, y) = loop {
                let x = draw_mod(&mut g, site::CENTRE_X, xs);
                let y = draw_mod(&mut g, site::CENTRE_Y, ys);
                if g.land_dist(x, y) >= spacing {
                    break (x, y);
                }
                misses += 1;
                if misses >= 1000 {
                    break (x, y);
                }
            };
            if misses > 999 {
                break;
            }
            let id = last_id + 1;
            if g.grow_valid(id, x, y) {
                g.make_region(id, x, y);
                let _ = g.grow_region(id, lake, xs);
                last_id = id;
                left -= 1;
                fails = 0;
            }
            lake = small;
            if big - small > 1 {
                lake += g.draws.get(site::SIZE) % (big - small);
            }
        }
        big = small;
        if small < lake {
            lake = small;
            continue;
        }
        small = small * 13 / 16;
        (lake, big) = (small, small);
        if small <= 19 {
            break;
        }
    }
    let map = g.map;
    eliminate_pools(g.grid);
    g.grid.invert_land();
    let grid = g.grid;
    let step = (u32::MAX / players.max(1) as u32) as i32;
    let centre = xs >> 1;
    const ORTHOG: [(i32, i32); 5] = [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)];
    for _ in 0..players {
        angle = angle.wrapping_add(step);
        let mut radius = xs * 3 / 2;
        let (x, y) = loop {
            let (x, y) = project(centre, centre, Angle(angle), radius);
            radius -= 3;
            let edge = ORTHOG.iter().any(|&(dx, dy)| {
                let (px, py) = (x + dx, y + dy);
                px <= 0 || py <= 0 || px >= xs - 1 || py >= ys - 1
            });
            if !(0..xs).contains(&x) || !(0..ys).contains(&y) || grid.cell(x, y).is_ocean() || edge
            {
                continue;
            }
            break (x, y);
        };
        grid.add_starting_location(x, y);
    }
    check_player_land(grid, style.start_land);
    Some(map)
}

/// **`Map::check_player_land@0068ef00`** as Great Lakes calls it, on a grid
/// `invert_land` has just cleared of regions: every cell of the disc to
/// ring `radius` round each start-city cell, bar the centre, becomes land
/// (`land` and `land_sub` 0, `region` 0). With every `region` 0 the
/// neighbour searches that would adopt or protect a region find none.
fn check_player_land(grid: &mut MapGrid, radius: i32) {
    let t = circle::table();
    let ring = if radius < 0 { 5 } else { radius } as usize;
    let cities: Vec<(i32, i32)> = grid.starts[2]
        .list
        .iter()
        .zip(&grid.starts[3].list)
        .map(|(&x, &y)| (x, y))
        .collect();
    for (sx, sy) in cities {
        for i in t.radius[0] as usize..t.radius[ring] as usize {
            let (x, y) = (sx + i32::from(t.x[i]), sy + i32::from(t.y[i]));
            if grid.get(x, y).is_some() {
                let c = grid.cell_mut(x, y);
                c.land = 0;
                c.land_sub = 0;
                c.region = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spacing_root_is_the_integer_newton_loop() {
        for n in 0..5000 {
            let r = isqrt(n);
            assert!(r * r <= n && (r + 1) * (r + 1) > n, "isqrt({n}) = {r}");
        }
    }

    /// A pool the lakes enclose is merged into land; the open water
    /// round them, the larger region, is left. 7807 cannot say this on the
    /// corpus's one Great Lakes run, whose background is one region.
    #[test]
    fn eliminate_pools_keeps_the_largest_water() {
        let mut grid = MapGrid::new(10, 10);
        grid.wipe();
        grid.clear_regions();
        for y in 3..=7 {
            for x in 3..=7 {
                if x == 3 || x == 7 || y == 3 || y == 7 {
                    grid.cell_mut(x, y).land = 0;
                }
            }
        }
        eliminate_pools(&mut grid);
        for y in 0..10 {
            for x in 0..10 {
                let inside = (4..=6).contains(&x) && (4..=6).contains(&y);
                let ring = (3..=7).contains(&x) && (3..=7).contains(&y);
                let want = if ring || inside { 0 } else { 2 };
                assert_eq!(grid.cell(x, y).land, want, "({x}, {y})");
            }
        }
    }

    #[test]
    fn a_lobby_past_its_allowance_is_refused() {
        assert_eq!(scale_land_area(8, 4, 4), Some(8));
        assert_eq!(scale_land_area(8, 5, 4), None);
    }
}
