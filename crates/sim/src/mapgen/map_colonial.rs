//! **Colonial Powers' land/sea mask** (`docs/MAPGEN.md` §5, §32):
//! `MapColonialPowers::make_continents@00697dd0`, on the region growers
//! Great Lakes' port already carries (`crate::mapgen::map_lakes`).
//!
//! Two continents in a sea, half a turn apart on a ring about the map's
//! centre: the old world (region 1), where the lobby's unteamed computers
//! stand, and the new (region 2), where everyone else does. Both are grown
//! as land on the wiped (all-water) world, the second with `avoid_center`
//! raised so that it keeps off the middle; a growth that stalls wipes the
//! world and draws again. Then `check_player_land` and `eliminate_pools`
//! over the border only. No float: `grow_region`'s distances are the
//! engine's integer `vector_dist`.

use crate::mapgen::map_east_west::check_player_land;
use crate::mapgen::map_lakes::{
    Grower, LakeMap, LakeStyle, Lakes, eliminate_pools_in, scale_land_area,
};
use crate::mapgen::terrain_groups::Draws;
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The ring's angle, high half then low, drawn each generation.
    pub const ANGLE_HI: u32 = 0x0069_805a;
    pub const ANGLE_LO: u32 = 0x0069_8072;
}

/// Each continent's arc of starts, as a turn of `2³²`: a sixth for the
/// old world's, a third for the new's (`0x2aaaaaaa`, `0x55555555`).
const ARC: [u32; 2] = [0x2aaa_aaaa, 0x5555_5555];

/// What the mask leaves besides the grid: the `Map` fields and how many
/// times a stalled growth restarted it. The function writes no
/// `start_dist` (+0x28).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colonial {
    pub map: LakeMap,
    /// The generations a stalled `grow_region` threw away.
    pub restarts: u32,
}

/// **`MapColonialPowers::make_continents@00697dd0`** (`docs/MAPGEN.md`
/// §32.1).
///
/// `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`
/// once. `outsiders` is the old world's count: with `team_style` 7 and the
/// game's semaphore bit 8 clear, the live non-human seats (`flags & 5 ==
/// 1`) whose team no human seat (`flags & 5 == 5`) is on; 0 otherwise. The
/// new world takes `players − outsiders`.
///
/// Each generation wipes the world and the region table; `r = 3·(m/2)/4`
/// (`m` the shorter side) and `area = 2·size/3`. Two draws make the angle
/// `(h % 0xffff) << 16 + l % 0xffff`. Continent `k` (0, then 1) sits on
/// the ring `r` about `(xs/2, ys/2)` at the angle; its `n` starts, each
/// clamped to `5..=side − 5`, stand on the same ring from `angle − ARC[k]/2`
/// stepped by `ARC[k] / (n − 1)` (0 for one); then region `k + 1` opens on
/// the continent's spot. The angle then steps by `0x7fffffff`. Each region
/// grows to `area/2` cells within `3r/4` of its first cell, the second
/// with `avoid_center` at `7r/9`; a growth that ends short starts the
/// generation again with `avoid_center` restored. Last, with
/// `avoid_center` restored, `check_player_land` at ring 5 with the guard
/// on `avoid_continent`, and `eliminate_pools(EDGES_ONLY)`.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn colonial_powers_make_continents(
    grid: &mut MapGrid,
    players: i32,
    outsiders: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<Colonial> {
    let avoid_continent =
        scale_land_area(style.avoid_continent, players, style.overage_allowance)?.max(4);
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
    let counts = [outsiders, players - outsiders];
    let steps = [0, 1].map(|k| {
        if counts[k] < 2 {
            0
        } else {
            ARC[k] / (counts[k] - 1) as u32
        }
    });
    let mut restarts = 0;
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let r = xs.min(ys) / 2 * 3 / 4;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = (hi << 16).wrapping_add(lo) as u32;
        let area = g.size() * 2 / 3;
        let mut at = angle.wrapping_sub(ARC[0] / 2);
        for k in 0..2 {
            let (cx, cy) = project(xs / 2, ys / 2, Angle(angle as i32), r);
            for _ in 0..counts[k] {
                let (x, y) = project(xs / 2, ys / 2, Angle(at as i32), r);
                let x = x.max(5).min(xs - 5);
                let y = y.max(5).min(ys - 5);
                g.grid.add_starting_location(x, y);
                at = at.wrapping_add(steps[k]);
            }
            g.make_region(k as i32 + 1, cx, cy);
            angle = angle.wrapping_add(0x7fff_ffff);
            if k < 1 {
                at = angle.wrapping_sub(ARC[1] / 2);
            }
        }
        for id in 1..=2 {
            if id > 1 {
                g.style.avoid_center = r * 7 / 9;
            }
            let short = g.grow_region(id, area / 2, r * 3 / 4);
            if short {
                g.style.avoid_center = style.avoid_center;
                restarts += 1;
                continue 'generation;
            }
        }
        g.style.avoid_center = style.avoid_center;
        let mut lists: Vec<Vec<(i32, i32)>> = g.lakes.points.clone();
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        eliminate_pools_in(g.grid, true);
        return Some(Colonial {
            map: g.map,
            restarts,
        });
    }
}

#[cfg(test)]
mod tests;
