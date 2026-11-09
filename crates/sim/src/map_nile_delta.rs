//! **Nile Delta's land/sea mask** (`docs/MAPGEN.md` §34):
//! `MapNileDelta::make_continents@00695bf0`, on the region growers Great
//! Lakes' port carries (`crate::map_lakes`) and the edge-canal pass and
//! guarded `Map::check_player_land` East Meets West's port carries
//! (`crate::map_east_west`).
//!
//! One continent a player, opened on a ring about the map's centre and
//! grown in `scale_land_area(4)` passes, each a further share of its size;
//! a growth that ends short wipes the world, shrinks the size by a fifth
//! and draws again. Then the pools are merged into land, the edge canals
//! widened, and the start cities' land checked. No `invert_land`: the
//! regions are grown as land on the wiped (all-water) world and stay land.
//! No float: the distances are the engine's integer `vector_dist`, the ring
//! its integer `project`.

use crate::map_east_west::{check_player_land, coords, eliminate_edge_canals};
use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, eliminate_pools, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle: its high half…
    pub const ANGLE_HI: u32 = 0x0069_5d3b;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_5d53;
}

/// What the mask leaves besides the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NileDelta {
    /// `avoid_continent` and `avoid_modify` as the last generation left
    /// them.
    pub map: LakeMap,
    /// The generations a short growth threw away.
    pub restarts: u32,
    /// A continent's size as the last generation grew it, the ring's
    /// radius, and the passes.
    pub size: i32,
    pub radius: i32,
    pub passes: i32,
}

/// **`MapNileDelta::make_continents@00695bf0`** (`docs/MAPGEN.md` §34).
///
/// A continent's size `S` is `max(100, scale_land_area((cells /
/// players)·6/8))`, and `avoid_continent` becomes `max(4,
/// scale_land_area(avoid_continent))`, both once. Each generation wipes
/// the world and the region table; the ring's radius is `((m/2)·3)/4`, `m`
/// the shorter side, and two draws make the angle `(r % 0xffff) << 16 +
/// r2 % 0xffff`. Each player steps the angle by `0xffffffff / players`
/// (the first a step past it), takes its start on the ring about `(xs/2,
/// ys/2)` and opens region `k + 1` there. Then `n = scale_land_area(4)`
/// passes: pass `p` grows every continent to `S·(p + 1) / n` cells within
/// `radius / 2` of its start, and one that ends short makes `S` `max(S·4/5,
/// (cells / players)/4)` and starts the generation again. Last,
/// `eliminate_pools` over the world, [`eliminate_edge_canals`], and
/// [`check_player_land`] at ring 5 guarded by `avoid_continent`, on the
/// region lists the canals' `find_all` rebuilt.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn nile_delta_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<NileDelta> {
    let allow = style.overage_allowance;
    let per = grid.xs * grid.ys / players.max(1);
    let mut size = scale_land_area(per * 6 / 8, players, allow)?.max(100);
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
    let mut restarts = 0;
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let radius = xs.min(ys) / 2 * 3 / 4;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = (hi << 16).wrapping_add(lo);
        for k in 0..players {
            angle = angle.wrapping_add(step);
            let (x, y) = project(xs / 2, ys / 2, Angle(angle), radius);
            g.grid.add_starting_location(x, y);
            g.make_region(k + 1, x, y);
        }
        let passes = scale_land_area(4, players, allow)?;
        let reach = radius / 2;
        let mut sum = size;
        for _ in 0..passes {
            let want = sum / passes;
            for k in 0..players {
                if g.grow_region(k + 1, want, reach) {
                    restarts += 1;
                    size = (size * 4 / 5).max(per / 4);
                    continue 'generation;
                }
            }
            sum += size;
        }
        eliminate_pools(g.grid);
        let regions = eliminate_edge_canals(g.grid);
        let mut lists: Vec<Vec<(i32, i32)>> = (0..128).map(|r| coords(g.grid, r as i16)).collect();
        debug_assert!(
            (0..128).all(|r| lists[r].len() as i32 == regions.count[r]),
            "rebuild_coords' lists are find_all's counts"
        );
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        return Some(NileDelta {
            map: g.map,
            restarts,
            size,
            radius,
            passes,
        });
    }
}

#[cfg(test)]
mod tests;
