//! **Sea Power's land/sea mask** (`docs/MAPGEN.md` §33, §5, §15 step 1):
//! `MapSeaPower::make_continents@00696080`, on the region growers Great
//! Lakes' port carries (`crate::map_lakes`), with East Meets West's
//! guarded `check_player_land` and `eliminate_edge_canals`
//! (`crate::map_east_west`) and Mediterranean's `EDGES_ONLY`.
//!
//! One island a player, opened on a ring about the map's centre on a wiped
//! (all-water) world and grown in two `grow_region` passes — a quarter of
//! the island, then the whole. A first-pass stall draws the ring again
//! unchanged; a second-pass stall shrinks the island by a tenth first.
//! Then a disc of land round each start city, the pools that touch the
//! map's edge merged, and the canals a cell from an edge widened. No
//! float: the growers' distances are the engine's integer `vector_dist`,
//! the ring its integer `project`.

use crate::map_east_west::{check_player_land, eliminate_edge_canals};
use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, eliminate_pools_in, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle, its high half…
    pub const ANGLE_HI: u32 = 0x0069_61d0;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_61e8;
}

/// What the mask leaves besides the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeaPowerMap {
    /// `avoid_continent` after `scale_land_area`, and `avoid_modify`.
    pub map: LakeMap,
    /// Each island's size as the last generation grew it.
    pub land: i32,
    /// The generations a first-pass stall threw away (the island kept).
    pub quarter_stalls: u32,
    /// The generations a second-pass stall threw away (the island shrunk).
    pub whole_stalls: u32,
    /// The seats' starts, in seat order.
    pub starts: Vec<(i32, i32)>,
}

/// The island's first size (0x696099..0x6960c9): `5·(cells / players) / 8`
/// through `scale_land_area`, at least 80.
fn island(cells: i32, players: i32, allow: i32) -> Option<i32> {
    Some(scale_land_area(cells / players * 5 / 8, players, allow)?.max(0x50))
}

/// A second-pass stall's island (0x696596..0x6965d1): nine tenths of it,
/// no less than a quarter of each player's share of the cells.
fn shrunk(land: i32, cells: i32, players: i32) -> i32 {
    (land * 9 / 10).max(cells / players / 4)
}

/// **`MapSeaPower::make_continents@00696080`** (`docs/MAPGEN.md` §33.1).
///
/// The island is `max(80, scale_land_area(5·(cells / players) / 8))`
/// cells; `avoid_continent` becomes `scale_land_area(avoid_continent)`, at
/// least 4. Each generation wipes the world and the regions; the ring's
/// radius is `7·(m/2)/8`, `m` the shorter side; two draws make the angle
/// `(r % 0xffff) << 16 + r % 0xffff`. For each player `i` from 1 the angle
/// steps by `0xffffffff / players` *before* the projection about `(xs/2,
/// ys/2)`, the point is a starting location, and record `i` opens there.
/// Then every record grows to `island/4`, and then every record to
/// `island`, within `3·radius/4` of its first cell. A first-pass stall
/// starts a fresh generation with the island as it was; a second-pass
/// stall makes it `max(9·island/10, (cells / players)/4)` first.
///
/// Grown, [`check_player_land`] with ring 5 (the call's −1) and
/// `avoid_continent`'s guard on the growers' own lists — the regions are
/// still the islands, unlike Mediterranean's —, `eliminate_pools` over the
/// map's outer ring (`EDGES_ONLY`), and [`eliminate_edge_canals`].
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn sea_power_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<SeaPowerMap> {
    let allow = style.overage_allowance;
    let cells = grid.xs * grid.ys;
    let mut land = island(cells, players, allow)?;
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
    let (xs, ys) = (g.grid.xs, g.grid.ys);
    let (mut quarter_stalls, mut whole_stalls) = (0, 0);
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let ring = xs.min(ys) / 2 * 7 / 8;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = hi.wrapping_mul(0x10000).wrapping_add(lo);
        let mut starts = Vec::new();
        for id in 1..=players {
            angle = angle.wrapping_add(step);
            let (x, y) = project(xs / 2, ys / 2, Angle(angle), ring);
            g.grid.add_starting_location(x, y);
            starts.push((x, y));
            g.make_region(id, x, y);
        }
        let reach = ring * 3 / 4;
        for id in 1..=players {
            if g.grow_region(id, land / 4, reach) {
                quarter_stalls += 1;
                continue 'generation;
            }
        }
        for id in 1..=players {
            if g.grow_region(id, land, reach) {
                whole_stalls += 1;
                land = shrunk(land, cells, players);
                continue 'generation;
            }
        }
        let mut lists = std::mem::replace(&mut g.lakes, Lakes::cleared()).points;
        lists.resize(128, Vec::new());
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        eliminate_pools_in(g.grid, true);
        eliminate_edge_canals(g.grid);
        return Some(SeaPowerMap {
            map: g.map,
            land,
            quarter_stalls,
            whole_stalls,
            starts,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sea Power's stock values on a two-nation 60×60 map, as run1535's
    /// lobby reads them.
    fn style() -> LakeStyle {
        LakeStyle {
            avoid_center: 0,
            avoid_continent: 8,
            avoid_edge: [0; 4],
            base_edge: 0,
            standard: 60,
            start_land: 6,
            overage_allowance: 2,
        }
    }

    /// The island is five eighths of a player's share, floored at 80; a
    /// lobby past its allowance takes the float path and is refused.
    #[test]
    fn the_island_is_five_eighths_of_a_share_at_least_80() {
        assert_eq!(island(3600, 2, 2), Some(1125));
        assert_eq!(island(3600, 3, 8), Some(750));
        assert_eq!(island(256, 2, 2), Some(80));
        assert_eq!(island(259, 2, 2), Some(80));
        assert_eq!(island(260, 2, 2), Some(81));
        assert_eq!(island(3600, 3, 2), None);
    }

    /// A second-pass stall keeps nine tenths, truncated, and never less
    /// than a quarter of a share.
    #[test]
    fn a_whole_stall_keeps_nine_tenths_down_to_a_quarter_share() {
        assert_eq!(shrunk(1125, 3600, 2), 1012);
        assert_eq!(shrunk(1019, 3600, 2), 917);
        assert_eq!(shrunk(500, 3600, 2), 450);
        assert_eq!(shrunk(460, 3600, 2), 450);
        assert_eq!(shrunk(449, 3600, 3), 404);
        assert_eq!(shrunk(100, 3600, 2), 450);
    }

    /// The angle steps *before* each projection, on the ring `7·(m/2)/8`
    /// about the map's middle: the first start is a step past the drawn
    /// angle, and the second a step past that.
    #[test]
    fn the_ring_steps_before_it_projects() {
        for seed in [1u32, 12345, 0xdead_beef, 77] {
            let mut grid = MapGrid::new(60, 60);
            let mut d = Draws::new(seed);
            let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
            let mut twin = Draws::new(seed);
            let hi = twin.get(site::ANGLE_HI) % 0xffff;
            let lo = twin.get(site::ANGLE_LO) % 0xffff;
            let a = hi.wrapping_mul(0x10000).wrapping_add(lo);
            let step = (u32::MAX / 2) as i32;
            if m.quarter_stalls + m.whole_stalls != 0 {
                continue;
            }
            let ring = 30 * 7 / 8;
            let want: Vec<(i32, i32)> = [
                a.wrapping_add(step),
                a.wrapping_add(step).wrapping_add(step),
            ]
            .iter()
            .map(|&a| project(30, 30, Angle(a), ring))
            .collect();
            assert_eq!(m.starts, want, "seed {seed}");
            assert_eq!(&d.log[..2], &twin.log[..2]);
        }
    }

    /// Each seat's start is a starting location, and each island is land
    /// that holds its start city's disc (`check_player_land`'s ring 5).
    #[test]
    fn every_start_stands_on_land_with_its_disc() {
        let mut grid = MapGrid::new(60, 60);
        let mut d = Draws::new(12345);
        let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
        assert_eq!(m.starts.len(), 2);
        assert_eq!(grid.starts[0].list.len(), 2);
        for &(x, y) in &m.starts {
            assert!(!grid.cell(x, y).is_ocean(), "({x}, {y}) is water");
        }
        assert_eq!(m.map.avoid_continent, 8);
    }

    /// `check_player_land` makes land of the circle to ring 5 round every
    /// start-city cell. On run1535 each 1,125-cell island already covers
    /// it; a lone player's 250-cell island on a 20×20 map does not, so the
    /// call is what fills it — one island, so no guard (§33.3).
    #[test]
    fn every_start_city_has_its_ring_5_disc_of_land() {
        let t = crate::scout::circle::table();
        let mut filled = 0;
        for seed in 1..=24u32 {
            let mut grid = MapGrid::new(20, 20);
            let mut d = Draws::new(seed);
            let m = sea_power_make_continents(&mut grid, 1, style(), &mut d).expect("allowed");
            assert_eq!(m.whole_stalls + m.quarter_stalls, 0, "seed {seed}");
            let cities: Vec<(i32, i32)> = grid.starts[2]
                .list
                .iter()
                .zip(&grid.starts[3].list)
                .map(|(&x, &y)| (x, y))
                .collect();
            for (cx, cy) in cities {
                for i in t.radius[0] as usize..t.radius[5] as usize {
                    let (x, y) = (cx + i32::from(t.x[i]), cy + i32::from(t.y[i]));
                    if grid.get(x, y).is_some() {
                        assert!(!grid.cell(x, y).is_ocean(), "seed {seed}: ({x}, {y})");
                        filled += 1;
                    }
                }
            }
        }
        assert!(filled > 0);
    }

    /// Two 125-cell islands on a 20×20 map stall the whole pass, the arm
    /// run1535 never takes: each stall shrinks the island once by
    /// [`shrunk`] and draws the ring again; a first-pass stall would
    /// keep it.
    #[test]
    fn a_whole_stall_shrinks_the_island_and_draws_again() {
        let mut whole = 0;
        for seed in 1..=24u32 {
            let mut grid = MapGrid::new(20, 20);
            let mut d = Draws::new(seed);
            let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
            let mut land = 125;
            for _ in 0..m.whole_stalls {
                land = shrunk(land, 400, 2);
            }
            assert_eq!(m.land, land, "seed {seed}");
            // The angle's two draws open every generation.
            let opened = d.log.iter().filter(|e| e.0 == site::ANGLE_HI).count() as u32;
            assert_eq!(opened, 1 + m.whole_stalls + m.quarter_stalls, "seed {seed}");
            whole += m.whole_stalls;
        }
        assert!(whole > 0);
    }

    /// `avoid_continent` is held to at least 4 after `scale_land_area`.
    #[test]
    fn avoid_continent_is_at_least_4() {
        let mut grid = MapGrid::new(60, 60);
        let mut d = Draws::new(12345);
        let low = LakeStyle {
            avoid_continent: 2,
            ..style()
        };
        let m = sea_power_make_continents(&mut grid, 2, low, &mut d).expect("allowed");
        assert_eq!(m.map.avoid_continent, 4);
    }
}
