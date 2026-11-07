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
mod tests {
    use super::*;

    /// The stock files' values on a Standard map: `default.xml`'s
    /// `AVOID_*`, edges 0.
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

    fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, NewWorld) {
        let mut grid = MapGrid::new(xs, xs);
        let mut draws = Draws::new(seed);
        let m = new_world_make_continents(&mut grid, players, style(), &mut draws)
            .expect("inside the allowance");
        (grid, draws, m)
    }

    /// Under four players one draw a generation, the quadrant; from four,
    /// two, the angle's halves.
    #[test]
    fn the_angle_is_one_draw_under_four_players_and_two_from_four() {
        let (_, d, m) = run(80, 2, 12345);
        let opening: Vec<u32> = d.log.iter().map(|x| x.0).take(2).collect();
        assert_eq!(opening[0], site::ANGLE_HI);
        assert_ne!(opening[1], site::ANGLE_LO);
        assert_eq!(m.restarts, 0);
        let (_, d, _) = run(80, 4, 12345);
        let opening: Vec<u32> = d.log.iter().map(|x| x.0).take(2).collect();
        assert_eq!(opening, [site::ANGLE_HI, site::ANGLE_LO]);
    }

    /// Two players on 80×80: the ring's radius `((80/2)·7)/8` = 35, the
    /// centre `(6400 − 500)·2/3` cells; three players take `·4/10`.
    #[test]
    fn the_centre_s_size_is_two_thirds_under_three_players_and_four_tenths_from_three() {
        let (_, _, m) = run(80, 2, 12345);
        assert_eq!((m.radius, m.centre), (35, (6400 - 500) * 2 / 3));
        let (_, _, m) = run(80, 3, 12345);
        assert_eq!(m.centre, (6400 - 750) * 4 / 10);
    }

    /// The starts sit on the ring at the quadrant's diagonal, stepped half
    /// a turn for two; the centre is land, a corner water.
    #[test]
    fn the_starts_are_on_the_ring_and_the_centre_is_land() {
        let (grid, _, _) = run(80, 2, 12345);
        let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
        let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
        assert!((39..=41).contains(&((x0 + x1) / 2)) && (39..=41).contains(&((y0 + y1) / 2)));
        let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
        assert!((68 * 68..=72 * 72).contains(&d2), "{d2}");
        assert!((x0 - 40).abs() > 10 && (y0 - 40).abs() > 10, "a diagonal");
        assert_eq!(grid.cell(40, 40).land, 0, "the centre continent");
        assert_eq!(grid.cell(0, 0).land, 2, "the corner is sea");
    }

    /// No inversion: the islands and the centre are land, and every pool
    /// left is merged into it, so one water body remains.
    #[test]
    fn the_regions_stay_land() {
        let (grid, _, m) = run(80, 2, 12345);
        let land = grid.cells.iter().filter(|c| c.land == 0).count() as i32;
        assert!(land >= m.centre + 250, "{land} land cells");
        assert!(land < 6400 - 400, "{land} land cells");
    }

    /// A lobby past the map size's allowance refuses before any draw.
    #[test]
    fn a_lobby_past_its_allowance_refuses_before_a_draw() {
        let mut grid = MapGrid::new(60, 60);
        let mut draws = Draws::new(1);
        let s = LakeStyle {
            overage_allowance: 2,
            ..style()
        };
        assert!(new_world_make_continents(&mut grid, 3, s, &mut draws).is_none());
        assert!(draws.log.is_empty());
    }
}
