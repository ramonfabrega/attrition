//! **Mediterranean's land/sea mask** (`docs/MAPGEN.md` §29, §5, §15 step
//! 1): `MapMediterranean::make_continents@0069add0`, on the region growers
//! Great Lakes' port carries (`crate::mapgen::map_lakes`).
//!
//! One region grown as land at the map's centre on a wiped world, then
//! inverted, so the sea is the grown region and the wiped background the
//! land round it; the starts walked in towards the sea's opening cell; a
//! disc of land round each start city; and an island, region 0x20, grown
//! as land in the middle of the sea. A sea that stalls draws again; an
//! island that stalls throws the whole generation away. No float: the
//! growers' distances are the engine's integer `vector_dist`, the walk's
//! angle and radius its integer `project`.

use crate::mapgen::map_east_west::is_near_ocean;
use crate::mapgen::map_lakes::{
    Grower, LakeMap, LakeStyle, Lakes, check_player_land, eliminate_pools_in, scale_land_area,
};
use crate::mapgen::terrain_groups::Draws;
use crate::mapgen::{MapGrid, Regions, project};
use crate::movement::Angle;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The start angle, its high half…
    pub const ANGLE_HI: u32 = 0x0069_af28;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_af3f;
    /// The sea's opening cell: its x jitter, then its y jitter.
    pub const SEA_X: u32 = 0x0069_afde;
    pub const SEA_Y: u32 = 0x0069_b00f;
    /// The island's opening cell: its x jitter, then its y jitter.
    pub const ISLAND_X: u32 = 0x0069_b47b;
    pub const ISLAND_Y: u32 = 0x0069_b4ac;
}

/// The island's region number, which `make_region` opens it under.
pub const ISLAND: i32 = 0x20;

/// What the mask leaves besides the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediterraneanMap {
    /// `avoid_continent` after `scale_land_area`, and `avoid_modify`.
    pub map: LakeMap,
    /// The sea's opening cell, which the starts are walked towards.
    pub sea: (i32, i32),
    /// The sea draws a stalled `grow_region` threw away.
    pub sea_stalls: u32,
    /// The generations a stalled island threw away.
    pub island_stalls: u32,
    /// The seats' starts, in seat order.
    pub starts: Vec<(i32, i32)>,
}

/// `orthog_x`/`orthog_y` (0xadd250, 0xadd210): the cell and its four
/// orthogonal neighbours.
const ORTHOG: [(i32, i32); 5] = [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)];

fn in_bounds(grid: &MapGrid, x: i32, y: i32) -> bool {
    (0..grid.xs).contains(&x) && (0..grid.ys).contains(&y)
}

/// One opening cell: `size / 2 + (r & 1)` on each axis, x drawn first.
fn opening(g: &mut Grower<'_>, (sx, sy): (u32, u32)) -> (i32, i32) {
    let x = g.grid.xs / 2 + (g.draws.get(sx) & 1);
    let y = g.grid.ys / 2 + (g.draws.get(sy) & 1);
    (x, y)
}

/// One seat's start (0x69b1f0..0x69b431): the angle advanced by `step`,
/// then a walk in from `3·xs/2` towards the sea's opening cell in steps of
/// 3.
///
/// Each step projects at the radius, then takes 3 off it. A radius that
/// falls under `reach` turns the angle by 0x71c71c6 and starts again at
/// `3·xs/2`, and tests nothing. Otherwise the point is `edge` when any of
/// its five orthogonal cells is off the grid or on its outer ring; a point
/// that is not, and not water, sets `near` when `is_near_ocean(…, 3, 9)`.
/// A point off the grid projects again. One on it that is water, or
/// `edge`, or met while `near` is still clear, walks on; any other is the
/// start.
///
/// The original clears `near` once a seat, not once a step, and the turning
/// step tests the point with `edge` as the step before left it (one stack
/// slot for every seat). Neither is ever read: a step that sets `near`
/// passes every later test and is the start, so `near` is clear on every
/// step that reaches the refusal, the turning step's among them, and the
/// turning step is refused whatever `edge` holds.
fn walk_in(grid: &MapGrid, sea: (i32, i32), angle: i32, reach: i32) -> (i32, i32) {
    let (xs, ys) = (grid.xs, grid.ys);
    let far = xs * 3 / 2;
    let (mut a, mut radius, mut near, mut edge) = (angle, far, false, false);
    loop {
        let (x, y) = project(sea.0, sea.1, Angle(a), radius);
        radius -= 3;
        if radius < reach {
            a = a.wrapping_add(0x071c_71c6);
            radius = far;
        } else {
            edge = ORTHOG.iter().any(|&(dx, dy)| {
                let (px, py) = (x + dx, y + dy);
                !in_bounds(grid, px, py) || px == 0 || py == 0 || px == xs - 1 || py == ys - 1
            });
            if !edge && !grid.cell(x, y).is_ocean() && is_near_ocean(grid, x, y, 3, 9) {
                near = true;
            }
        }
        if !in_bounds(grid, x, y) || grid.cell(x, y).is_ocean() || edge || !near {
            continue;
        }
        return (x, y);
    }
}

/// **`MapMediterranean::make_continents@0069add0`** (`docs/MAPGEN.md` §29).
///
/// `avoid_continent` becomes `scale_land_area(avoid_continent)`, at least
/// 4. `reach` is a third of the shorter side rounded down to even, and the
/// sea `3·reach²` cells. Each sea draw wipes the world and the regions,
/// draws the start angle `(r % 0xffff) << 16 + r % 0xffff` and the sea's
/// opening cell, and grows region 1 to the sea's size within `reach` of
/// it; one that ends short draws again.
///
/// Then `eliminate_pools` over the world, `invert_land`, and again; each
/// seat's start by [`walk_in`], the angle stepping by `0xffffffff /
/// players`; `check_player_land`'s disc of land to `start_land` round each
/// start-city cell; and the island: `max(75, scale_land_area(150))` cells,
/// region 0x20 opened like the sea and grown within `reach`. An island that
/// ends short starts a fresh generation from the sea's wipe; one that grows
/// ends with `eliminate_pools` over the map's outer ring.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn mediterranean_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<MediterraneanMap> {
    let allow = style.overage_allowance;
    let avoid_continent = scale_land_area(style.avoid_continent, players, allow)?.max(4);
    let step = (u32::MAX / players.max(1) as u32) as i32;
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
    let (mut sea_stalls, mut island_stalls) = (0, 0);
    loop {
        let reach = (g.grid.xs.min(g.grid.ys) & !1) / 3;
        let size = reach * reach * 3;
        let (mut angle, sea) = loop {
            g.grid.wipe();
            g.grid.clear_regions();
            g.lakes = Lakes::cleared();
            let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
            let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
            let angle = hi.wrapping_mul(0x10000).wrapping_add(lo);
            let sea = opening(&mut g, (site::SEA_X, site::SEA_Y));
            g.make_region(1, sea.0, sea.1);
            if !g.grow_region(1, size, reach) {
                break (angle, sea);
            }
            sea_stalls += 1;
        };
        let mut regions = Regions::default();
        regions.clear_all(g.grid);
        regions.find_all(g.grid);
        eliminate_pools_in(g.grid, false);
        regions.clear_all(g.grid);
        g.grid.invert_land();
        regions.find_all(g.grid);
        eliminate_pools_in(g.grid, false);
        regions.clear_all(g.grid);
        g.lakes = Lakes::cleared();
        let mut starts = Vec::new();
        for _ in 0..players {
            angle = angle.wrapping_add(step);
            let (x, y) = walk_in(g.grid, sea, angle, reach);
            g.grid.add_starting_location(x, y);
            starts.push((x, y));
        }
        check_player_land(g.grid, style.start_land);
        let island = scale_land_area(150, players, allow)?.max(75);
        let at = opening(&mut g, (site::ISLAND_X, site::ISLAND_Y));
        g.make_region(ISLAND, at.0, at.1);
        if g.grow_region(ISLAND, island, reach) {
            island_stalls += 1;
            continue;
        }
        eliminate_pools_in(g.grid, true);
        return Some(MediterraneanMap {
            map: g.map,
            sea,
            sea_stalls,
            island_stalls,
            starts,
        });
    }
}

#[cfg(test)]
mod tests;
