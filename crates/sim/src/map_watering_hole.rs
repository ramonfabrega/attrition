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
mod tests {
    use super::*;

    /// The stock files' values on a Standard map: `default.xml`'s
    /// `AVOID_*` (Africa's file carries none), edges 0.
    fn style() -> LakeStyle {
        LakeStyle {
            avoid_center: 4,
            avoid_continent: 4,
            avoid_edge: [0; 4],
            base_edge: 0,
            standard: 60,
            start_land: 6,
            overage_allowance: 8,
        }
    }

    fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, WateringHole) {
        let mut grid = MapGrid::new(xs, xs);
        let mut draws = Draws::new(seed);
        let m = african_watering_hole_make_continents(&mut grid, players, style(), &mut draws)
            .expect("inside the allowance");
        (grid, draws, m)
    }

    /// The lake is the region grown, inverted: `size·5/100` cells of water
    /// (180 on 60×60) about the middle, the rest land — the growth's
    /// cross ends it at `size` or a few past.
    #[test]
    fn the_lake_is_five_percent_of_the_map_and_inverted() {
        let (grid, _, m) = run(60, 2, 12345);
        assert_eq!(m.restarts, 0);
        let water = grid.cells.iter().filter(|c| c.land == 2).count();
        assert!((180..185).contains(&water), "{water} water cells");
        assert_eq!(grid.cell(30, 30).land, 2, "the middle is the lake");
        assert_eq!(grid.cell(0, 0).land, 0, "the corner is land");
        assert_eq!(m.start_dist, 4);
    }

    /// Four draws a generation before the growth — the angle's halves,
    /// then the lake's parities — and two after it, the ring's parities.
    #[test]
    fn the_ring_s_centre_is_drawn_after_the_lake() {
        let (_, draws, _) = run(60, 2, 12345);
        let sites: Vec<u32> = draws.log.iter().map(|d| d.0).collect();
        assert_eq!(
            sites[..4],
            [site::ANGLE_HI, site::ANGLE_LO, site::LAKE_X, site::LAKE_Y]
        );
        assert_eq!(sites[sites.len() - 2..], [site::RING_X, site::RING_Y]);
    }

    /// The players sit on the ring at radius `min(m/2 − 4, (m/2)·80/100)`
    /// (24 on 60×60) about the ring's centre, half a turn apart for two.
    #[test]
    fn the_starts_are_on_the_ring() {
        let (grid, _, _) = run(60, 2, 12345);
        let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
        let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
        let mid = ((x0 + x1) / 2, (y0 + y1) / 2);
        assert!((29..=31).contains(&mid.0) && (29..=31).contains(&mid.1));
        let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
        assert!((46 * 46..=50 * 50).contains(&d2), "{d2}");
    }

    /// A lobby past the map size's allowance refuses before any draw,
    /// where `scale_land_area` would take its float path.
    #[test]
    fn a_lobby_past_its_allowance_refuses_before_a_draw() {
        let mut grid = MapGrid::new(60, 60);
        let mut draws = Draws::new(1);
        let s = LakeStyle {
            overage_allowance: 2,
            ..style()
        };
        assert!(african_watering_hole_make_continents(&mut grid, 3, s, &mut draws).is_none());
        assert!(draws.log.is_empty());
    }
}
