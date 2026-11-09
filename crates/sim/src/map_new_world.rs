//! **New World's land/sea mask** (`docs/MAPGEN.md` §31):
//! `MapNewWorld::make_continents@006984d0`, on the region growers Great
//! Lakes' port carries (`crate::map_lakes`) and the guarded
//! `Map::check_player_land` East Meets West's port carries
//! (`crate::map_east_west`).
//!
//! An island a player on a ring about the map's centre, and one continent
//! in the middle. Each player's region opens on the ring and is grown in
//! two passes, to half its size and then to all of it, held off the
//! centre by `avoid_center`; a growth that ends under half its pass wipes
//! the world and draws again. Then the centre's region is grown with
//! `avoid_center` cleared, the start cities' land is checked, and the
//! pools are merged into land. No `invert_land`: the regions are grown as
//! land on the wiped (all-water) world and stay land. No float: the
//! distances are the engine's integer `vector_dist`, the ring its integer
//! `project`.

use crate::map_east_west::check_player_land;
use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, eliminate_pools, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle: under four players its quadrant `r & 3`, else its
    /// high half…
    pub const ANGLE_HI: u32 = 0x0069_8655;
    /// …and, from four players, its low half.
    pub const ANGLE_LO: u32 = 0x0069_86d8;
}

/// The centre continent's region record.
pub const CENTRE: i32 = 9;

/// A player's island: `max(0x3c, scale_land_area(0xfa))` cells, which is
/// 250 inside the allowance.
const ISLAND: i32 = 0xfa;

/// What the mask leaves besides the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewWorld {
    /// `avoid_continent` and `avoid_modify` as the last generation left
    /// them.
    pub map: LakeMap,
    /// The generations a short island growth threw away.
    pub restarts: u32,
    /// The centre continent's size and the ring's radius.
    pub centre: i32,
    pub radius: i32,
}

/// **`MapNewWorld::make_continents@006984d0`** (`docs/MAPGEN.md` §31).
///
/// `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`
/// once. Each generation wipes the world and the region table; the island
/// size `I` is `max(0x3c, scale_land_area(0xfa))`, the centre's size
/// `scale_land_area((cells − I·players)·4/10)`, and the ring's radius
/// `((m/2)·7)/8`, `m` the shorter side. One draw `r`: under four players
/// the angle is `(r & 3)·0x3fffffff + 0x1fffffff`, and under three the
/// centre's size `(cells − I·players)·2/3`; from four, a second draw and
/// the angle `(r % 0xffff) << 16 + r2 % 0xffff`. Each player steps the
/// angle by `0xffffffff / players` (the first a step past it), takes its
/// start on the ring about `(xs/2, ys/2)` and opens region `k + 1` there.
/// Region 9 opens on the centre. `avoid_center` becomes
/// `scale_land_area((xs/2)·70/100)`; then two passes, the divisor 2 and
/// then 1, grow every island to `I / div` cells within `(radius / 4) /
/// div`, and an island that ends under half of that starts the generation
/// again. Last, `avoid_center` 0, region 9 grown to the centre's size
/// within the radius, [`check_player_land`] at ring 5 guarded by
/// `avoid_continent`, and `eliminate_pools` over the world.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn new_world_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<NewWorld> {
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
    let (xs, ys) = (g.grid.xs, g.grid.ys);
    let island = scale_land_area(ISLAND, players, allow)?.max(0x3c);
    let mut restarts = 0;
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let m = xs.min(ys);
        let rest = g.size() - island * players;
        let mut centre = scale_land_area(rest * 4 / 10, players, allow)?;
        let r = g.draws.get(site::ANGLE_HI);
        let radius = m / 2 * 7 / 8;
        let mut angle = if players < 4 {
            if players != 3 {
                centre = rest * 2 / 3;
            }
            (r & 3).wrapping_mul(0x3fff_ffff).wrapping_add(0x1fff_ffff)
        } else {
            let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
            ((r % 0xffff) << 16).wrapping_add(lo)
        };
        for k in 0..players {
            angle = angle.wrapping_add(step);
            let (x, y) = project(xs / 2, ys / 2, Angle(angle), radius);
            g.grid.add_starting_location(x, y);
            g.make_region(k + 1, x, y);
            // Bit 2, not the bit 1 `grow_valid` and `point` read: inert
            // in the growers (§31.1).
            g.lakes.flags[(k + 1) as usize] |= 2;
        }
        g.make_region(CENTRE, xs / 2, ys / 2);
        g.style.avoid_center = scale_land_area(xs / 2 * 70 / 100, players, allow)?;
        let quarter = scale_land_area(4, players, allow)?.max(4);
        for div in [2, 1] {
            let reach = radius / quarter / div;
            let size = island / div;
            for k in 0..players {
                let _ = g.grow_region(k + 1, size, reach);
                if (g.lakes.points[(k + 1) as usize].len() as i32) < size / 2 {
                    restarts += 1;
                    continue 'generation;
                }
            }
        }
        g.style.avoid_center = 0;
        let _ = g.grow_region(CENTRE, centre, radius);
        let mut lists = std::mem::replace(&mut g.lakes, Lakes::cleared()).points;
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        eliminate_pools(g.grid);
        return Some(NewWorld {
            map: g.map,
            restarts,
            centre,
            radius,
        });
    }
}

#[cfg(test)]
mod tests;
