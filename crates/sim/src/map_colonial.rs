//! **Colonial Powers' land/sea mask** (`docs/MAPGEN.md` §5, §32):
//! `MapColonialPowers::make_continents@00697dd0`, on the region growers
//! Great Lakes' port already carries (`crate::map_lakes`).
//!
//! Two continents in a sea, half a turn apart on a ring about the map's
//! centre: the old world (region 1), where the lobby's unteamed computers
//! stand, and the new (region 2), where everyone else does. Both are grown
//! as land on the wiped (all-water) world, the second with `avoid_center`
//! raised so that it keeps off the middle; a growth that stalls wipes the
//! world and draws again. Then `check_player_land` and `eliminate_pools`
//! over the border only. No float: `grow_region`'s distances are the
//! engine's integer `vector_dist`.

use crate::map_east_west::check_player_land;
use crate::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, eliminate_pools_in, scale_land_area};
use crate::mapgen::{MapGrid, project};
use crate::movement::Angle;
use crate::terrain_groups::Draws;

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
mod tests {
    use super::*;

    /// The stock files' values on a Standard map: `default.xml`'s
    /// `AVOID_CENTER` and `AVOID_CONTINENT`, `COLONIALPOWERS.xml`'s four
    /// zero edges.
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

    fn run(xs: i32, players: i32, outsiders: i32, seed: u32) -> (MapGrid, Draws, Colonial) {
        let mut grid = MapGrid::new(xs, xs);
        let mut draws = Draws::new(seed);
        let m = colonial_powers_make_continents(&mut grid, players, outsiders, style(), &mut draws)
            .expect("inside the allowance");
        (grid, draws, m)
    }

    /// Two continents of `size/3` cells each (1,200 on 60×60): the land is
    /// about two thirds of the map, the middle is sea.
    #[test]
    fn two_continents_of_a_third_each() {
        let (grid, _, m) = run(60, 2, 0, 12345);
        let land = grid.cells.iter().filter(|c| c.land == 0).count();
        assert!((2350..2600).contains(&land), "{land} land cells");
        assert_eq!(grid.cell(30, 30).land, 2, "the middle is sea");
        assert_eq!(m.restarts, 0);
    }

    /// The angle's two halves are the mask's only draws of its own.
    #[test]
    fn the_angle_is_drawn_first() {
        let (_, draws, m) = run(60, 2, 0, 12345);
        let sites: Vec<u32> = draws.log.iter().map(|d| d.0).collect();
        assert_eq!(sites[..2], [site::ANGLE_HI, site::ANGLE_LO]);
        let opens = sites.iter().filter(|&&s| s == site::ANGLE_HI).count();
        assert_eq!(opens as u32, m.restarts + 1);
    }

    /// With no outsiders every start is on the new world's arc: on the
    /// ring `3·30/4 = 22` about the centre, the two a third of a turn
    /// apart — 22·√3 ≈ 38 cells — and far from the old world's spot.
    #[test]
    fn the_starts_are_on_the_new_world_s_arc() {
        let (grid, _, _) = run(60, 2, 0, 12345);
        let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
        let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
        for (x, y) in [(x0, y0), (x1, y1)] {
            let d2 = (x - 30) * (x - 30) + (y - 30) * (y - 30);
            assert!((20 * 20..=23 * 23).contains(&d2), "({x}, {y}): {d2}");
        }
        let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
        assert!((36 * 36..=40 * 40).contains(&d2), "{d2}");
    }

    /// An outsider stands on the old world's arc, half a turn from the
    /// new world's: a three-seat lobby with one outsider puts it 44 cells
    /// (the ring's diameter, clamped) from the new world's middle seat.
    #[test]
    fn an_outsider_stands_on_the_old_world() {
        let (grid, _, _) = run(60, 3, 1, 12345);
        let xs: Vec<i32> = grid.starts[0].list.clone();
        let ys: Vec<i32> = grid.starts[1].list.clone();
        assert_eq!(xs.len(), 3);
        // The old world's single start is its arc's first (angle − a
        // twelfth turn); the new world's two straddle the opposite spot.
        let mid = ((xs[1] + xs[2]) / 2, (ys[1] + ys[2]) / 2);
        let d2 = (xs[0] - mid.0).pow(2) + (ys[0] - mid.1).pow(2);
        assert!(d2 >= 30 * 30, "{d2}");
    }

    /// Every start is on its continent's arc, exactly: the old world's
    /// from `angle − 0x15555555` by `0x2aaaaaaa / (n − 1)`, the new
    /// world's from `angle + 0x7fffffff − 0x2aaaaaaa` by `0x55555555 / (n −
    /// 1)`, the angle the mask's first two draws.
    #[test]
    fn the_starts_are_the_arcs_own() {
        for (players, outsiders) in [(3, 1), (4, 2), (5, 2), (2, 0)] {
            let (grid, _, m) = run(60, players, outsiders, 777);
            assert_eq!(m.restarts, 0);
            let mut d = Draws::new(777);
            let hi = d.get(site::ANGLE_HI) % 0xffff;
            let lo = d.get(site::ANGLE_LO) % 0xffff;
            let angle = (hi << 16).wrapping_add(lo) as u32;
            let mut want = Vec::new();
            for (k, n, from) in [
                (0, outsiders, angle.wrapping_sub(0x1555_5555)),
                (
                    1,
                    players - outsiders,
                    angle.wrapping_add(0x7fff_ffff).wrapping_sub(0x2aaa_aaaa),
                ),
            ] {
                let step = if n < 2 { 0 } else { ARC[k] / (n - 1) as u32 };
                for j in 0..n as u32 {
                    let a = from.wrapping_add(step.wrapping_mul(j));
                    let (x, y) = project(30, 30, Angle(a as i32), 22);
                    want.push((x.clamp(5, 55), y.clamp(5, 55)));
                }
            }
            let got: Vec<(i32, i32)> = grid.starts[0]
                .list
                .iter()
                .zip(&grid.starts[1].list)
                .map(|(&x, &y)| (x, y))
                .collect();
            assert_eq!(got, want, "{players} players, {outsiders} outsiders");
        }
    }

    /// A start off the ring's reach is held five in from each edge: on
    /// 40×40 the ring is 15 about 20, so a start may fall at 5 or 35, and
    /// none falls outside.
    #[test]
    fn a_start_is_held_five_in_from_the_edge() {
        let mut held = 0;
        for seed in 1..40 {
            let (grid, _, _) = run(36, 6, 3, seed);
            for (&x, &y) in grid.starts[0].list.iter().zip(&grid.starts[1].list) {
                assert!((5..=31).contains(&x) && (5..=31).contains(&y), "({x}, {y})");
                held += usize::from(x == 5 || y == 5 || x == 31 || y == 31);
            }
        }
        assert!(held > 0, "no start reached the clamp");
    }

    /// `EDGES_ONLY`: a pool no border cell holds stays water, and each of
    /// the four borders lists the pools its cells hold — one pool on the
    /// middle of each (none on a corner), so a border left out of the
    /// predicate leaves its pool water too.
    #[test]
    fn an_inland_pool_survives_the_edge_only_pass() {
        let mut grid = MapGrid::new(20, 20);
        grid.wipe();
        grid.clear_regions();
        for c in grid.cells.iter_mut() {
            c.land = 0;
        }
        let pools = [(10, 0), (0, 10), (10, 19), (19, 10)];
        for (x, y) in pools.into_iter().chain([(10, 10)]) {
            grid.cell_mut(x, y).land = 2;
        }
        let mut both = grid.clone();
        eliminate_pools_in(&mut grid, true);
        assert_eq!(grid.cell(10, 10).land, 2, "inland, never listed");
        let wet = pools.map(|(x, y)| grid.cell(x, y).land == 2);
        assert_eq!(
            wet,
            [false, false, true, false],
            "the last listed, row-major, is kept"
        );
        eliminate_pools_in(&mut both, false);
        let wet = pools.map(|(x, y)| both.cell(x, y).land == 2);
        assert_eq!(both.cell(10, 10).land, 0, "the whole world lists it");
        assert_eq!(
            wet.iter().filter(|&&w| w).count(),
            1,
            "the whole world: one"
        );
    }
}
