//! **African Watering Hole's land/sea mask** (`docs/MAPGEN.md` §5, §28):
//! `MapAfricanWateringHole::make_continents@0069b650`, on the region
//! growers Great Lakes' port already carries (`crate::map_lakes`).
//!
//! One lake in the middle of a whole land, and the players on Great
//! Sahara's ring about it. The lake is grown as a *land* region on the
//! wiped (all-water) world, as Great Lakes grows its lakes, and the world
//! is inverted after it, so the region's cells end as water and the
//! background as land. A growth that stalls wipes the world and draws
//! again, angle and all. No float: `grow_region`'s distances are the
//! engine's integer `vector_dist`.

use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle, high half then low, drawn each generation.
    pub const ANGLE_HI: u32 = 0x0069_b78c;
    pub const ANGLE_LO: u32 = 0x0069_b7a3;
    /// The lake's centre, x and y parity, each generation.
    pub const LAKE_X: u32 = 0x0069_b858;
    pub const LAKE_Y: u32 = 0x0069_b889;
    /// The ring's centre, x and y parity, once the lake has grown.
    pub const RING_X: u32 = 0x0069_ba30;
    pub const RING_Y: u32 = 0x0069_ba61;
}

/// What the mask leaves besides the grid: the `Map` fields, how many
/// times a stalled growth restarted it, and `Map::start_dist` (+0x28).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WateringHole {
    pub map: LakeMap,
    /// The generations a stalled `grow_region` threw away.
    pub restarts: u32,
    pub start_dist: i32,
}

/// **`MapAfricanWateringHole::make_continents@0069b650`**
/// (`docs/MAPGEN.md` §28).
///
/// `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`
/// once. Each generation wipes the world and the region table, takes the
/// radius `min(m/2 − 4, (m/2)·80/100)` (`m` the shorter side), draws the
/// angle's two halves `(r % 0xffff) << 16 + r % 0xffff`, then opens region
/// 1 on `(xs/2 + (r & 1), ys/2 + (r & 1))` and grows it to `size·5/100`
/// cells within the radius; a growth that ends short starts the
/// generation again. Then `invert_land`, two fresh parity draws for the
/// ring's centre, `start_dist = max(4, xs/16)`, and each player at the
/// last generation's angle stepped by `0xffffffff / players` (the first a
/// step past it), on the ring.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn african_watering_hole_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<WateringHole> {
    let avoid_continent =
        scale_land_area(style.avoid_continent, players, style.overage_allowance)?.max(4);
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
    let half = xs.min(ys) / 2;
    let radius = (half - 4).min(half * 80 / 100);
    let size = g.size() * 5 / 100;
    let mut restarts = 0;
    let mut angle = loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let angle = (hi << 16).wrapping_add(lo);
        let x = xs / 2 + (g.draws.get(site::LAKE_X) & 1);
        let y = ys / 2 + (g.draws.get(site::LAKE_Y) & 1);
        g.make_region(1, x, y);
        if !g.grow_region(1, size, radius) {
            break angle;
        }
        restarts += 1;
    };
    g.grid.invert_land();
    let cx = xs / 2 + (g.draws.get(site::RING_X) & 1);
    let cy = ys / 2 + (g.draws.get(site::RING_Y) & 1);
    let start_dist = (xs / 16).max(4);
    for _ in 0..players {
        angle = angle.wrapping_add(step);
        let (x, y) = project(cx, cy, Angle(angle), radius);
        g.grid.add_starting_location(x, y);
    }
    Some(WateringHole {
        map: g.map,
        restarts,
        start_dist,
    })
}

#[cfg(test)]
mod tests;
