//! **East Indies' land/sea mask** (`docs/MAPGEN.md` §5, §15 step 1):
//! `MapEastIndies::make_continents@00697540`, on the region growers Great
//! Lakes' port already carries (`crate::map_lakes`).
//!
//! One island a player, opened on a ring about the map's centre that
//! shrinks a cell at a time until its point clears every edge, and grown in
//! two `Map::grow_region` passes, to a third of `A` and then to `A`; a pass
//! that stalls on any island wipes the world and starts the whole mask
//! again, angle draws and all. Then the further islands, placed as Great
//! Lakes places its lakes, and — unlike Great Lakes — never inverted: the
//! islands are land regions on the wiped water, and their region numbers
//! reach 7807's part 5. No float: the distances are the engine's integer
//! `vector_dist`, the spacing's root the integer Newton loop.

use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, isqrt, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The angle: a quadrant under four players, else its high half…
    pub const ANGLE: u32 = 0x0069_76e2;
    /// …and its low half, with four or more.
    pub const ANGLE_LO: u32 = 0x0069_7728;
    /// The further islands' count parity.
    pub const COUNT: u32 = 0x0069_7b77;
    /// Each further island's centre, x and y.
    pub const CENTRE_X: u32 = 0x0069_7c44;
    pub const CENTRE_Y: u32 = 0x0069_7c6e;
    /// Each later island's size.
    pub const SIZE: u32 = 0x0069_7d6f;
}

/// What the mask leaves besides the grid: the `Map` fields, and how many
/// times a stalled pass restarted it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandMap {
    pub map: LakeMap,
    /// `A`, `Map::scale_land_area`(250): each player island's size.
    pub player_land: i32,
    /// The generations a stalled `grow_region` pass threw away.
    pub restarts: u32,
    /// The further islands opened.
    pub islands: i32,
}

/// **`MapEastIndies::make_continents@00697540`** (`docs/MAPGEN.md` §5).
///
/// `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`
/// once, and `A = scale_land_area(250)`. Each generation wipes the world
/// and the region table, then draws the angle: under four players one draw,
/// a quadrant's centre `(r % 4)·0x3fffffff + 0x1fffffff`; with four or more
/// two, `(r % 0xffff) << 16 + r % 0xffff`. The ring's radius is `m/2`, or
/// `(m/2)·7/8` with three or more players, `m` the shorter side. Each
/// player, at the angle stepped by `0xffffffff / players` (the first a step
/// past it), takes the ring's point about `(xs/2, ys/2)`; while that point
/// is within `max(avoid_edge) + 4` of an edge the radius falls by one, and
/// it stays fallen for the players after. The point is the player's start
/// and opens its region. Then `grow_region` takes every island to `A/3`,
/// then every island to `A`, both within half the first radius of its
/// first cell; a pass that ends short restarts the generation.
///
/// Then the further islands: `xs/12 − players + 8 + (r & 1)` of them, each
/// as Great Lakes' lakes are placed (`crate::map_lakes`), numbered on from
/// the players' and grown to `xs`.
///
/// `None` when the lobby's nations pass the map size's allowance, where
/// `scale_land_area` takes its float path.
pub fn east_indies_make_continents(
    grid: &mut MapGrid,
    players: i32,
    style: LakeStyle,
    draws: &mut Draws,
) -> Option<IslandMap> {
    let allow = style.overage_allowance;
    let avoid_continent = scale_land_area(style.avoid_continent, players, allow)?.max(4);
    let player_land = scale_land_area(250, players, allow)?;
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
    let edge = style.avoid_edge.iter().copied().max().unwrap_or(0) + 4;
    let mut restarts = 0;
    'generation: loop {
        g.grid.wipe();
        g.grid.clear_regions();
        g.lakes = Lakes::cleared();
        let m = xs.min(ys);
        let r = g.draws.get(site::ANGLE);
        let mut angle;
        let mut radius;
        if players < 4 {
            angle = (r % 4).wrapping_mul(0x3fff_ffff).wrapping_add(0x1fff_ffff);
            radius = if players == 3 { m / 2 * 7 / 8 } else { m / 2 };
        } else {
            let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
            angle = (r % 0xffff).wrapping_mul(0x10000).wrapping_add(lo);
            radius = m / 2 * 7 / 8;
        }
        let first_radius = radius;
        for id in 1..=players {
            angle = angle.wrapping_add(step);
            let (x, y) = loop {
                let (x, y) = project(xs / 2, ys / 2, Angle(angle), radius);
                if x > edge && xs - x > edge && y > edge && ys - y > edge {
                    break (x, y);
                }
                radius -= 1;
            };
            g.grid.add_starting_location(x, y);
            g.make_region(id, x, y);
        }
        for size in [player_land / 3, player_land] {
            for id in 1..=players {
                if g.grow_region(id, size, first_radius / 2) {
                    restarts += 1;
                    continue 'generation;
                }
            }
        }
        break;
    }
    let first = (g.size() / 20).max(90);
    let small_at = (g.size() / 40).max(24);
    let (mut lake, mut big, mut small) = (first, first, small_at);
    let mut left = xs / 12 - players + 8 + (g.draws.get(site::COUNT) & 1);
    let (mut last_id, mut fails, mut islands) = (players, 0, 0);
    let draw_mod = |g: &mut Grower<'_>, at: u32, n: i32| {
        if n == 1 || n - 1 < 0 {
            0
        } else {
            g.draws.get(at) % n
        }
    };
    'islands: while left != 0 {
        fails += 1;
        if fails > 99 {
            break;
        }
        let spacing = ((g.map.avoid_continent + 1) / 2 + isqrt(lake * 7 / 22)).max(3);
        let mut misses = 0;
        let found = loop {
            let x = draw_mod(&mut g, site::CENTRE_X, xs);
            let y = draw_mod(&mut g, site::CENTRE_Y, ys);
            if g.land_dist(x, y) >= spacing {
                break Some((x, y));
            }
            misses += 1;
            if misses >= 1000 {
                break None;
            }
        };
        let Some((x, y)) = found else {
            let shrink = lake <= small;
            (lake, big) = (small, small);
            if shrink {
                small = small * 13 / 16;
                (lake, big) = (small, small);
                if small < 20 {
                    break 'islands;
                }
            }
            continue;
        };
        let id = last_id + 1;
        if g.grow_valid(id, x, y) {
            g.make_region(id, x, y);
            let _ = g.grow_region(id, lake, xs);
            last_id = id;
            left -= 1;
            fails = 0;
            islands += 1;
        }
        lake = small;
        if big - small > 1 {
            lake += g.draws.get(site::SIZE) % (big - small);
        }
    }
    Some(IslandMap {
        map: g.map,
        player_land,
        restarts,
        islands,
    })
}
