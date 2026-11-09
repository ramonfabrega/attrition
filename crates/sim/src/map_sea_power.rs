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
mod tests;
