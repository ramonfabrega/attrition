//! **British Isles' land/sea mask** (`docs/MAPGEN.md` §37):
//! `MapBritishIsles::make_continents@00695130`, on the region growers Great
//! Lakes' port carries (`crate::mapgen::map_lakes`), with East Meets West's
//! `Map::place_start_in_region` and guarded `Map::check_player_land`
//! (`crate::mapgen::map_east_west`).
//!
//! Islands on a ring about the map's centre, each opened at a quarter of
//! its share and then grown to the whole share; an island that grows short
//! gives its players to the island before it when it is the last, and
//! otherwise starts the generation again. Each player's start is sought on
//! its island against the starts already found, and the seats take the
//! starts in a drawn order. No `invert_land`: the islands are grown as land
//! on the wiped (all-water) world and stay land. No float: the distances
//! are the engine's integer `vector_dist`, the ring its integer `project`.

use crate::mapgen::map_east_west::{check_player_land, place_start_in_list};
use crate::mapgen::map_lakes::{
    Grower, LakeMap, LakeStyle, Lakes, eliminate_pools, scale_land_area,
};
use crate::mapgen::terrain_groups::Draws;
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The island count past two players: `2 + r % players`.
    pub const ISLAND_COUNT: u32 = 0x0069_5425;
    /// The ring's angle: its high half…
    pub const ANGLE_HI: u32 = 0x0069_54f5;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_550d;
    /// The seats' order: one draw a pick, re-drawn while the pick is taken.
    pub const ORDER: u32 = 0x0069_5a3f;
}

/// What the mask leaves besides the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BritishIsles {
    /// `avoid_continent` and `avoid_modify` as the last generation left
    /// them.
    pub map: LakeMap,
    /// The generations an island grown short threw away.
    pub restarts: u32,
    /// The islands, and each island's players, as the last generation
    /// placed them.
    pub islands: i32,
    pub counts: Vec<i32>,
    /// A player's share `S`, and the ring's radius.
    pub share: i32,
    pub radius: i32,
    /// The starts in the order the islands found them, and the order the
    /// seats took them in.
    pub found: Vec<(i32, i32)>,
    pub order: Vec<i32>,
}

/// Why the port stops where the original would go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unported {
    /// The nations pass the map size's allowance, where `scale_land_area`
    /// takes its float path.
    Overage,
    /// The only island grows short: the original then credits a player to
    /// the word before its count array (the listing's `incl (%eax)` at
    /// 0x695950 with the pointer one slot below `counts[0]`).
    LoneIslandShort,
    /// `place_start_in_region` finds no cell: the original appends the
    /// out-parameters' previous values, which the first call has not set.
    NoStart,
}

/// **`MapBritishIsles::make_continents@00695130`** (`docs/MAPGEN.md` §37).
///
/// Once: `avoid_continent` (+0x30) becomes `max(4,
/// scale_land_area(avoid_continent))`; the step is `0xffffffff / players`,
/// unsigned. Each generation:
///
/// 1. `World::wipe`, `Regions::clear_all`. The islands are the players
///    under three, else `min(players, 2 + r % players)` (one draw). Island
///    `i` holds `(players + i % islands) / islands` players, at least 1
///    while `i < players`.
/// 2. The ring's radius `R = ((m / 2)·6) / 8`, `m` the shorter side; two
///    draws make the angle `(r % 0xffff) << 16 + r2 % 0xffff`. A player's
///    share is `S = max(0x78, scale_land_area((cells / players)·5 / 9))`.
/// 3. Each island `i` takes the point `R` along the angle from `(xs/2,
///    ys/2)`, opens region `i + 1` there and grows it to a quarter of
///    `counts[i]·S` within `R / 6` (`param_6` 0, its return dropped); the
///    angle then steps by half of `counts[i]` and half of `counts[(i + 1) %
///    islands]` steps.
/// 4. Each island in turn grows on to `counts[i]·S` within `R / 3`
///    (`param_6` 1: the record's count). Short of half of that, or of 0x78,
///    the islands drop by one: when the short island is the new last, the
///    island before it takes its player count and the loop runs on (and so
///    ends); otherwise the generation starts again.
/// 5. Each island's players, in island order, take
///    `place_start_in_region(i + 1, min 10)` against the starts found so
///    far; the seats then take them in a drawn order — `r % players`,
///    re-drawn while that start is taken — by `World::add_starting_location`.
/// 6. `Map::check_player_land` at ring 5 guarded by `avoid_continent` (the
///    listing's `ecx` 1, `edx` +0x30, ring −1) on the records' lists as
///    grown, then `Map::eliminate_pools(ENTIRE_WORLD)`.
pub fn british_isles_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Result<BritishIsles, Unported> {
    let allow = style.overage_allowance;
    let avoid_continent = scale_land_area(style.avoid_continent, players, allow)
        .ok_or(Unported::Overage)?
        .max(4);
    let step = u32::MAX / players.max(1) as u32;
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
        let mut islands = if players < 3 {
            players
        } else {
            let r = if players - 1 > 0 {
                g.draws.get(site::ISLAND_COUNT) % players
            } else {
                0
            };
            (r + 2).min(players)
        };
        let mut counts: Vec<i32> = (0..islands)
            .map(|i| {
                let c = (players + i % islands) / islands;
                if c == 0 && i < players { 1 } else { c }
            })
            .collect();
        let radius = xs.min(ys) / 2 * 6 / 8;
        let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
        let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
        let mut angle = ((hi << 16) as u32).wrapping_add(lo as u32);
        let per = xs * ys / players.max(1);
        let share = scale_land_area(per * 5 / 9, islands, allow)
            .ok_or(Unported::Overage)?
            .max(0x78);
        for i in 0..islands {
            let (x, y) = project(xs / 2, ys / 2, Angle(angle as i32), radius);
            let size = counts[i as usize] * share;
            g.make_region(i + 1, x, y);
            let _ = g.grow_region(i + 1, size / 4, radius / 6);
            let next = counts[((i + 1) % islands) as usize] as u32;
            angle = angle
                .wrapping_add(next.wrapping_mul(step) >> 1)
                .wrapping_add((counts[i as usize] as u32).wrapping_mul(step) >> 1);
        }
        // `get_player_overage` is 0 inside the allowance, so every island
        // grows to its own players' shares (the `S` alone arm is past it).
        let mut i = 0;
        while i < islands {
            let size = counts[i as usize] * share;
            let _ = g.grow_region(i + 1, size, radius / 3);
            let grown = g.lakes.points[(i + 1) as usize].len() as i32;
            if grown < size / 2 || grown < 0x78 {
                islands -= 1;
                if i != islands {
                    restarts += 1;
                    continue 'generation;
                }
                if i == 0 {
                    return Err(Unported::LoneIslandShort);
                }
                counts[(i - 1) as usize] += 1;
            }
            i += 1;
        }
        let (mut fx, mut fy) = (Vec::new(), Vec::new());
        for i in 0..islands {
            for _ in 0..counts[i as usize] {
                let list = g.lakes.points[(i + 1) as usize].clone();
                let (x, y) = place_start_in_list(g.grid, &list, (&fx, &fy), 10, g.draws)
                    .ok_or(Unported::NoStart)?;
                fx.push(x);
                fy.push(y);
            }
        }
        let mut taken = vec![false; fx.len().max(players as usize)];
        let mut order = Vec::new();
        for _ in 0..players {
            let k = loop {
                let k = if players - 1 > 0 {
                    g.draws.get(site::ORDER) % players
                } else {
                    0
                };
                if !taken[k as usize] {
                    break k;
                }
            };
            taken[k as usize] = true;
            order.push(k);
            g.grid.add_starting_location(fx[k as usize], fy[k as usize]);
        }
        let mut lists = g.lakes.points.clone();
        check_player_land(g.grid, &mut lists, g.map.avoid_continent);
        eliminate_pools(g.grid);
        return Ok(BritishIsles {
            map: g.map,
            restarts,
            islands,
            counts,
            share,
            radius,
            found: fx.into_iter().zip(fy).collect(),
            order,
        });
    }
}
