//! **Mediterranean's land/sea mask** (`docs/MAPGEN.md` §29, §5, §15 step
//! 1): `MapMediterranean::make_continents@0069add0`, on the region growers
//! Great Lakes' port carries (`crate::map_lakes`).
//!
//! One region grown as land at the map's centre on a wiped world, then
//! inverted, so the sea is the grown region and the wiped background the
//! land round it; the starts walked in towards the sea's opening cell; a
//! disc of land round each start city; and an island, region 0x20, grown
//! as land in the middle of the sea. A sea that stalls draws again; an
//! island that stalls throws the whole generation away. No float: the
//! growers' distances are the engine's integer `vector_dist`, the walk's
//! angle and radius its integer `project`.

use crate::map_east_west::is_near_ocean;
use crate::map_lakes::{
    Grower, LakeMap, LakeStyle, Lakes, check_player_land, eliminate_pools_in, scale_land_area,
};
use crate::mapgen::{MapGrid, Regions, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

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
mod tests {
    use super::*;
    use crate::map_east_west::is_near_ocean;

    /// Mediterranean's stock values on a Standard map: `AVOID_CENTER` 12,
    /// `AVOID_CONTINENT` 8, `AVOID_EDGE_0..3` 12.
    fn style() -> LakeStyle {
        LakeStyle {
            avoid_center: 12,
            avoid_continent: 8,
            avoid_edge: [12; 4],
            base_edge: 0,
            standard: 60,
            start_land: 6,
            overage_allowance: 8,
        }
    }

    /// A land map with a water disc of radius 4 about `(15, 15)`.
    fn pond() -> MapGrid {
        pond_of(4)
    }

    /// A land map with a water disc of radius `r` about `(15, 15)`.
    fn pond_of(r: i32) -> MapGrid {
        let mut grid = MapGrid::new(30, 30);
        grid.wipe();
        for c in &mut grid.cells {
            c.land = 0;
        }
        for y in 0..30 {
            for x in 0..30 {
                if (x - 15) * (x - 15) + (y - 15) * (y - 15) <= r * r {
                    grid.cell_mut(x, y).land = 2;
                }
            }
        }
        grid
    }

    /// The walk's candidates, read off the listing rather than the port:
    /// radius `3·xs/2` falling by 3 while the next is at least `reach`, the
    /// angle turned by 0x71c71c6 each time the radius runs out; a point
    /// whose five orthogonal cells sit inside the outer ring, that is land
    /// and `is_near_ocean(…, 3, 9)`, is the start.
    fn listed_start(grid: &MapGrid, sea: (i32, i32), angle: i32, reach: i32) -> ((i32, i32), i32) {
        let (xs, ys) = (grid.xs, grid.ys);
        let inside = |x: i32, y: i32| (1..xs - 1).contains(&x) && (1..ys - 1).contains(&y);
        for turn in 0..1024i32 {
            let a = angle.wrapping_add(turn.wrapping_mul(0x071c_71c6));
            let mut r = xs * 3 / 2;
            while r - 3 >= reach {
                let (x, y) = project(sea.0, sea.1, Angle(a), r);
                if ORTHOG.iter().all(|&(dx, dy)| inside(x + dx, y + dy))
                    && !grid.cell(x, y).is_ocean()
                    && is_near_ocean(grid, x, y, 3, 9)
                {
                    return ((x, y), turn);
                }
                r -= 3;
            }
        }
        panic!("no start in 1024 turns at {angle:#x}, reach {reach}");
    }

    /// The walk takes the listing's start on every angle, on a pond where
    /// most angles turn and many starts stand on the outer ring's band.
    /// (On a pond of 4 with `reach` 10, angle 0 turns some 8.45 million
    /// times: 36 turns of 0x71c71c6 come 8 short of a circle, and the walk
    /// creeps until a projection rounds onto a cell near the water.)
    #[test]
    fn the_walk_in_is_the_listing_s_candidates() {
        {
            let (radius, reach) = (6, 10);
            let grid = pond_of(radius);
            let mut turned = 0;
            let mut band = 0;
            for k in 0..64 {
                let angle = (k as u32).wrapping_mul(0x0400_0000) as i32;
                let (x, y) = walk_in(&grid, (15, 15), angle, reach);
                let (want, turns) = listed_start(&grid, (15, 15), angle, reach);
                assert_eq!((x, y), want, "pond {radius}, angle {k}");
                turned += usize::from(turns > 0);
                band += usize::from(x == 2 || y == 2 || x == 27 || y == 27);
            }
            eprintln!("pond {radius}: {turned} of 64 turned, {band} on the band");
            assert!(
                turned > 0 && band > 0,
                "{turned} turned, {band} on the band"
            );
        }
    }

    /// A HALFLAND cell is land to the walk: `is_ocean` refuses it, so the
    /// near test runs on it and it can take the start.
    #[test]
    fn a_halfland_cell_is_land_to_the_walk() {
        let mut grid = pond();
        let angle = 0x4000_0000; // due east: x grows, y holds.
        let (x, y) = walk_in(&grid, (15, 15), angle, 4);
        grid.cell_mut(x, y).flags |= 0x100;
        grid.cell_mut(x, y).land = 2;
        assert_eq!(walk_in(&grid, (15, 15), angle, 4), (x, y));
        grid.cell_mut(x, y).flags &= !0x100;
        assert_ne!(walk_in(&grid, (15, 15), angle, 4), (x, y));
    }

    /// The whole mask on a Standard two-nation map: the sea is region 1
    /// inverted, so the opening cell's ring is water and the corners land;
    /// two starts on land; the island's cells land in the sea; the draws
    /// start with the angle's halves and the sea's opening.
    #[test]
    fn the_sea_is_inverted_and_the_island_grows_in_it() {
        for seed in (1u32..=24).chain([12345, 0xdead_beef]) {
            let mut grid = MapGrid::new(60, 60);
            let mut d = Draws::new(seed);
            let m = mediterranean_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
            let sites: Vec<u32> = d.log.iter().take(4).map(|x| x.0).collect();
            assert_eq!(
                sites,
                [site::ANGLE_HI, site::ANGLE_LO, site::SEA_X, site::SEA_Y]
            );
            assert!(d.log.iter().any(|x| x.0 == site::ISLAND_Y));
            assert!((30..=31).contains(&m.sea.0) && (30..=31).contains(&m.sea.1));
            assert_eq!(m.starts.len(), 2);
            for &(x, y) in &m.starts {
                assert!(!grid.cell(x, y).is_ocean(), "{seed}: start ({x}, {y})");
            }
            for (x, y) in [(0, 0), (59, 0), (0, 59), (59, 59)] {
                assert!(!grid.cell(x, y).is_ocean(), "{seed}: corner ({x}, {y})");
            }
            let wet = (0..60)
                .flat_map(|y| (0..60).map(move |x| (x, y)))
                .filter(|&(x, y)| grid.cell(x, y).is_ocean())
                .count();
            assert!(wet > 600 && wet < 1300, "{seed}: {wet} water cells");
            assert_eq!(m.map.avoid_continent, 8);
            // `check_player_land`'s disc to `start_land` round each
            // start-city cell is land; the island may have taken some of it.
            let t = crate::scout::circle::table();
            let cities: Vec<(i32, i32)> = grid.starts[2]
                .list
                .iter()
                .zip(&grid.starts[3].list)
                .map(|(&x, &y)| (x, y))
                .collect();
            for (cx, cy) in cities {
                for i in 0..t.radius[style().start_land as usize] as usize {
                    let (x, y) = (cx + i32::from(t.x[i]), cy + i32::from(t.y[i]));
                    if let Some(c) = grid.get(x, y) {
                        assert!(!c.is_ocean(), "{seed}: ({x}, {y}) by ({cx}, {cy})");
                    }
                }
            }
        }
    }

    /// A lobby past its allowance takes `scale_land_area`'s float path:
    /// refused before a draw.
    #[test]
    fn a_lobby_past_its_allowance_draws_nothing() {
        let mut grid = MapGrid::new(60, 60);
        let mut d = Draws::new(1);
        let s = LakeStyle {
            overage_allowance: 2,
            ..style()
        };
        assert_eq!(mediterranean_make_continents(&mut grid, 3, s, &mut d), None);
        assert!(d.log.is_empty());
    }
}
