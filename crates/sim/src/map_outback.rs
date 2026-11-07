//! **Australian Outback's land/sea mask** (`docs/MAPGEN.md` §5, §30):
//! `MapAustralianOutback::make_continents@0069a650`, on the region growers
//! Great Lakes' port already carries (`crate::map_lakes`).
//!
//! One island in a sea: a region opened on each player's ring spot and
//! one on the map's middle (record 9), all grown as land on the wiped
//! (all-water) world, the players' toward the middle, until every player's
//! region is joined to the middle's through the adjacency masks `point`
//! keeps for a flagged record. Then the players' cells are folded into the
//! middle's, which grows to `size·5/9` cells, and the pools are filled. A
//! growth that stalls wipes the world and draws again. No float:
//! `grow_region`'s distances are the engine's integer `vector_dist`.

use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, eliminate_pools};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle, high half then low, drawn each generation.
    pub const ANGLE_HI: u32 = 0x0069_a762;
    pub const ANGLE_LO: u32 = 0x0069_a77a;
    /// The middle's x and y parity, each generation.
    pub const MIDDLE_X: u32 = 0x0069_a833;
    pub const MIDDLE_Y: u32 = 0x0069_a867;
}

/// The middle's record: its bit (`0x200`) is what every player's region
/// must reach.
const MIDDLE: i32 = 9;

/// What the mask leaves besides the grid: the `Map` fields, how many
/// times a stalled growth restarted it, and `Map::start_dist` (+0x28).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outback {
    pub map: LakeMap,
    /// The generations a stalled `grow_region` threw away.
    pub restarts: u32,
    /// The passes over records 1..8 the last generation took before
    /// record 9 held every player's bit.
    pub passes: u32,
    pub start_dist: i32,
}

/// **`MapAustralianOutback::make_continents@0069a650`**
/// (`docs/MAPGEN.md` §30.1).
///
/// Each generation wipes the world and the region table; `half` is half
/// the shorter side, the growers' reach `half·55/100`, the island
/// `total = size·5/9` cells. Two draws make the angle
/// `(r % 0xffff) << 16 + r % 0xffff`, two more the middle
/// `(xs/2 + (r & 1), ys/2 + (r & 1))`;
/// `start_dist = max(2, xs/16)`. Player `i` (from 1) starts on the ring
/// `half·66/100` about the middle at the angle, stepped by `0xffffffff /
/// players` after each, and opens record `i` there with flag bit 1; the
/// middle opens record 9 without it. Each player's record grows to
/// `total/players/4` and then `total/players/2` cells within the reach of
/// the middle; record 9 takes flag bit 1 and grows to `total/3` within a
/// third of the reach. Then, until record 9's mask holds every player's
/// bit and its own: for each of records 1..8 (whatever the player count)
/// not yet joined to 9, the record grows by `total/64` toward the middle —
/// or, when that would pass `total`, record 9 grows by `total/64` within a
/// quarter of the reach — and a player growth that stalls has record 9
/// grow instead; a stall of record 9 there starts the generation again.
/// Joined, every player's cells become record 9's, record 9 grows to
/// `total` within the reach, and `eliminate_pools` keeps the largest
/// water.
pub fn australian_outback_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Outback {
    let step = (u32::MAX / players.max(1) as u32) as i32;
    let mut g = Grower {
        grid,
        lakes: Lakes::cleared(),
        draws,
        style,
        map: LakeMap {
            avoid_continent: style.avoid_continent,
            avoid_modify: 0,
        },
    };
    let (xs, ys) = (g.grid.xs, g.grid.ys);
    let mut restarts = 0;
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let half = xs.min(ys) / 2;
        let reach = half * 55 / 100;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = (hi << 16).wrapping_add(lo);
        let total = g.size() * 5 / 9;
        let cx = xs / 2 + (g.draws.get(site::MIDDLE_X) & 1);
        let cy = ys / 2 + (g.draws.get(site::MIDDLE_Y) & 1);
        let start_dist = (xs / 16).max(2);
        let mut joined = 1u32 << MIDDLE;
        let ring = half * 66 / 100;
        for id in 1..=players {
            let (x, y) = project(cx, cy, Angle(angle), ring);
            g.grid.add_starting_location(x, y);
            g.make_region(id, x, y);
            g.lakes.flags[id as usize] |= 1;
            joined |= 1u32.wrapping_shl(id as u32);
            angle = angle.wrapping_add(step);
        }
        g.make_region(MIDDLE, cx, cy);
        for share in [4, 2] {
            for j in 0..players {
                let _ = g.grow_region_about(
                    (j & 7) + 1,
                    total / players / share,
                    reach,
                    Some((cx, cy)),
                );
            }
        }
        g.lakes.flags[MIDDLE as usize] |= 1;
        let _ = g.grow_region(MIDDLE, total / 3, reach / 3);
        let more = total / 64;
        let mut passes = 0;
        while g.lakes.adjacent[MIDDLE as usize] & joined != joined {
            passes += 1;
            for id in 1..=8 {
                if g.lakes.adjacent[id as usize] & (1 << MIDDLE) != 0 {
                    continue;
                }
                let count = g.lakes.points[id as usize].len() as i32;
                let middle = g.lakes.points[MIDDLE as usize].len() as i32;
                let restart = if total < count + more {
                    g.grow_region(MIDDLE, middle + more, reach / 4)
                } else {
                    g.grow_region_about(id, count + more, reach, Some((cx, cy))) && {
                        let middle = g.lakes.points[MIDDLE as usize].len() as i32;
                        g.grow_region(MIDDLE, middle + more, reach / 4)
                    }
                };
                if restart {
                    restarts += 1;
                    continue 'generation;
                }
            }
        }
        for id in 1..=players {
            let cells = std::mem::take(&mut g.lakes.points[id as usize]);
            for &(x, y) in &cells {
                g.grid.cell_mut(x, y).region = MIDDLE as i16;
            }
            g.lakes.points[MIDDLE as usize].extend(cells);
        }
        let _ = g.grow_region(MIDDLE, total, reach);
        eliminate_pools(g.grid);
        g.grid.clear_regions();
        return Outback {
            map: g.map,
            restarts,
            passes,
            start_dist,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stock files' values on a Standard map: `default.xml`'s
    /// `AVOID_CENTER` and `AVOID_CONTINENT`, `OUTBACK.xml`'s four edges.
    fn style() -> LakeStyle {
        LakeStyle {
            avoid_center: 4,
            avoid_continent: 4,
            avoid_edge: [4; 4],
            base_edge: 0,
            standard: 60,
            start_land: 6,
            overage_allowance: 8,
        }
    }

    fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, Outback) {
        let mut grid = MapGrid::new(xs, xs);
        let mut draws = Draws::new(seed);
        let m = australian_outback_make_continents(&mut grid, players, style(), &mut draws);
        (grid, draws, m)
    }

    /// One island: the middle is land, the corners sea, and the land at
    /// least the `size·5/9` cells record 9 grows to (2,000 on 60×60).
    #[test]
    fn the_island_is_five_ninths_of_the_map() {
        let (grid, _, m) = run(60, 2, 12345);
        let land = grid.cells.iter().filter(|c| c.land == 0).count();
        assert!((2000..2200).contains(&land), "{land} land cells");
        assert_eq!(grid.cell(30, 30).land, 0, "the middle is land");
        assert_eq!(grid.cell(0, 0).land, 2, "the corner is sea");
        assert_eq!(m.start_dist, 3, "max(2, 60/16)");
        assert_eq!(
            (m.restarts, m.passes),
            (0, 0),
            "run1532's shape: joined at once"
        );
        assert!(grid.cells.iter().all(|c| c.region == 0), "cleared");
    }

    /// Four draws open each generation: the angle's halves, then the
    /// middle's parities; everything after is the growers'.
    #[test]
    fn the_angle_and_the_middle_are_drawn_first() {
        let (_, draws, m) = run(60, 2, 12345);
        let sites: Vec<u32> = draws.log.iter().map(|d| d.0).collect();
        assert_eq!(
            sites[..4],
            [
                site::ANGLE_HI,
                site::ANGLE_LO,
                site::MIDDLE_X,
                site::MIDDLE_Y
            ]
        );
        let opens = sites.iter().filter(|&&s| s == site::ANGLE_HI).count();
        assert_eq!(opens as u32, m.restarts + 1);
    }

    /// Each player stands on the ring `half·66/100` (19 on 60×60) about
    /// the middle, and two stand half a turn apart.
    #[test]
    fn the_starts_are_on_the_ring() {
        let (grid, _, _) = run(60, 2, 12345);
        let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
        let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
        for (x, y) in [(x0, y0), (x1, y1)] {
            let d2 = (2 * x - 61) * (2 * x - 61) + (2 * y - 61) * (2 * y - 61);
            assert!((36 * 36..=40 * 40).contains(&d2), "({x}, {y}): {d2}");
        }
        let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
        assert!((36 * 36..=40 * 40).contains(&d2), "{d2}");
    }
}
