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
mod tests;
