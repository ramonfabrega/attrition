//! **Warring States' land/sea mask** (`docs/MAPGEN.md` §37):
//! `MapWarringStates::make_continents@00698b60`, on the region growers Great
//! Lakes' port carries (`crate::mapgen::map_lakes`), with East Meets West's
//! `Map::fill_cont` and `Map::check_player_land` (its unguarded arm), and
//! `MapFairness::calc_distances@0068a1c0` (§18.2).
//!
//! One continent, grown from a walk of regions across the wiped (all-water)
//! world; the players are seated at the walk's ends and the regions merged.
//! Then islands of 0xdc cells are placed in the sea, each where the start
//! nearest it is the one the islands have so far served most — the
//! original's fairness. No float: the fairness scores are `vector_dist`s
//! times the image's 1.0f, so every sum is an integer under 2²⁴ (§18.2),
//! and its square root is the function's own integer Newton loop.

use crate::mapgen::map_east_west::{NO_TEAM, check_player_land_guarded, coords, fill_cont};
use crate::mapgen::map_lakes::{Grower, LakeMap, LakeStyle, Lakes, isqrt, scale_land_area};
use crate::mapgen::terrain_groups::Draws;
use crate::mapgen::{MapGrid, Regions, project};
use crate::movement::Angle;
use crate::world::vector_dist;

/// The draw sites, as return addresses (the trace's `Function+0xNN`).
pub mod site {
    /// The walk's first angle: its high half…
    pub const ANGLE_HI: u32 = 0x0069_8c9d;
    /// …and its low half.
    pub const ANGLE_LO: u32 = 0x0069_8cb5;
    /// Each walk step's turn, `r % 0x4000`.
    pub const STEP: u32 = 0x0069_8ed4;
    /// The islands' count's odd one, `r % 2`.
    pub const ODD_ISLAND: u32 = 0x0069_967c;
    /// Each island's spacing, `r % 55 + 193`.
    pub const SPACING: u32 = 0x0069_99d2;
    /// Each island's first candidate, `r % count`.
    pub const CANDIDATE: u32 = 0x0069_9a2c;
}

/// `MapFairness::lowest_dist`'s float start, `[0x00b69768]`: larger than any
/// distance sum the map can make.
const FAR: i64 = 1 << 40;

/// `highest_player` (+0x48) before its first write: the constructor and
/// both `MapFairness::clear` calls leave it, so the first island's test
/// reads the frame's stale word. Measured, not read: run1122's trace takes
/// the first island on its first attempt, which only a team index here
/// allows (item 4211).
pub const STALE_HIGHEST: i32 = 0;

/// What the mask leaves besides the grid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WarringStates {
    /// `avoid_continent` and `avoid_modify` as the mask left them.
    pub map: LakeMap,
    /// `start_dist` (+0x28): a quarter of the walk's step.
    pub start_dist: i32,
    /// The walk's regions, and the list index each seat took.
    pub walk: Vec<(i32, i32)>,
    pub order: Vec<i32>,
    /// The growth passes the continent took.
    pub passes: u32,
    /// The islands asked for, those grown whole, and the attempts.
    pub islands: i32,
    pub grown: i32,
    pub attempts: u32,
}

/// Why the port stops where the original would go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unported {
    /// The nations pass the map size's allowance, where `scale_land_area`
    /// takes its float path (and the teams of `MapFairness` group seats).
    Overage,
    /// A walk step off the grid, which the original's `make_region` writes
    /// through unchecked.
    WalkOffGrid,
    /// A live seat no walk end was given: the original reads its start from
    /// a `malloc`'d array it never wrote.
    Unseated,
    /// An island search with its first candidate at 1 that accepts no cell:
    /// the original's loop never ends there.
    EndlessSearch,
}

/// The fairness tally (`MapFairness`, its integer-exact reading, §18.2).
struct Fairness {
    teams: Vec<i32>,
    dists: [i64; 8],
    fairness: [i64; 8],
    lowest_dist: i32,
    highest_player: i32,
}

impl Fairness {
    /// **`MapFairness::calc_distances@0068a1c0`**: each team's `vector_dist`
    /// from its start (`world.start_x/start_y[team]`) to `(x, y)`, and the
    /// first team (in `teams` order) with the least.
    fn calc_distances(&mut self, grid: &MapGrid, x: i32, y: i32) {
        let mut low = FAR;
        for &t in &self.teams {
            let (sx, sy) = (
                grid.starts[0].list[t as usize],
                grid.starts[1].list[t as usize],
            );
            let d = i64::from(vector_dist(sx - x, sy - y));
            self.dists[t as usize] = d;
            if d < low {
                self.lowest_dist = t;
                low = d;
            }
        }
    }

    /// `fairness[team] ± dists[team]` for each listed team, then the argmin
    /// and argmax of the tallies, strict, from `FAR` and from 0 (a tally at
    /// or under 0 names no highest, which keeps the last).
    fn add(&mut self, sign: i64) {
        for &t in &self.teams {
            self.fairness[t as usize] += sign * self.dists[t as usize];
        }
        let (mut low, mut high) = (FAR, 0);
        for &t in &self.teams {
            let f = self.fairness[t as usize];
            if f < low {
                low = f;
            }
            if f > high {
                self.highest_player = t;
                high = f;
            }
        }
    }
}

/// **`MapWarringStates::make_continents@00698b60`** (`docs/MAPGEN.md` §37).
///
/// 1. `avoid_continent` becomes `max(4, scale_land_area(avoid_continent))`
///    and `randomize_starts` (+0x2ec) 0. `World::wipe`, `Regions::clear_all`.
/// 2. **The walk.** `D = m/2 − avoid_edge[0] − avoid_continent`, `m` the
///    shorter side; two draws make the angle `a = (r % 0xffff) << 16 + r2 %
///    0xffff`, and the walk opens `D` along it from `(xs/2, ys/2)`. The
///    step is `2D / e`, `e` the players rounded up to even and doubled
///    until at least 6, and `start_dist` a quarter of it. Each step opens
///    region `i` there (flag bit 1, so the walk's regions never refuse each
///    other), then moves a step at `a + ((r % 0x4000 + 0x6000) << 16)`
///    (one draw); it stops once the point is further from the centre than
///    `D + avoid_continent` or `m − avoid_continent`.
/// 3. **The seats.** The walk's indices, by seat number `j`: an odd `j`
///    takes `n − (j − 1 >> s) − 1` in order, an even one `j >> s` from the
///    far end back (`s` 1 past the allowance or with fewer than two regions
///    a player, else 0). [`fill_cont`] gives each team its continent; the
///    k-th index goes to the first live, unseated seat whose continent is
///    the k-th's, and the continent moves on unless a later teamed seat
///    shares it. Each live seat in seat order takes `add_starting_location`.
/// 4. **The continent.** Every walk region grows by 0x30 cells at a pass
///    (within the map's width), until their sum is `cells / 7` and region 1
///    touches all of them, or a pass ends short on every one. The walk's
///    regions merge into region 1; `Regions::clear_all`, `find_all`.
/// 5. **The islands.** `xs / 15 + r % 2` of them, rounded up to even, from
///    the first sea region's
///    cells, whose labels are cleared — at least the players when the
///    allowance is not above them. Each draws its spacing `d = max(5,
///    (avoid_continent + isqrt(r % 55 + 193)) / 2)` and a first candidate;
///    a candidate whose `land_dist` is at least `d` is scored, and taken
///    when an attempt has failed or the start nearest it is the team the
///    tally favours most. It opens region `2, 3, …` and grows to 0xdc cells
///    within `d + 1`; whole, it adds to the tally; short, or with no
///    candidate, the attempt fails and its distances come off the tally,
///    four times, after which each failure gives the island up.
/// 6. `Map::check_player_land` at ring 5, unguarded (`ecx` 0).
pub fn warring_states_make_continents(
    grid: &mut MapGrid,
    players: i32,
    teams: &[Option<i32>; 8],
    style: LakeStyle,
    draws: &mut Draws,
) -> Result<WarringStates, Unported> {
    let allow = style.overage_allowance;
    if players > allow {
        return Err(Unported::Overage);
    }
    let avoid_continent = scale_land_area(style.avoid_continent, players, allow)
        .ok_or(Unported::Overage)?
        .max(4);
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
    g.grid.wipe();
    g.grid.clear_regions();
    // 2. The walk.
    let m = xs.min(ys);
    let reach = m / 2 - style.avoid_edge[0] - avoid_continent;
    let hi = g.draws.get(site::ANGLE_HI) % 0xffff;
    let lo = g.draws.get(site::ANGLE_LO) % 0xffff;
    let angle = (hi << 16).wrapping_add(lo);
    let size = xs * ys / 7;
    let (mut x, mut y) = project(xs / 2, ys / 2, Angle(angle), reach);
    let mut e = (players + (players & 1)).max(1);
    while e < 6 {
        e *= 2;
    }
    let step = reach * 2 / e;
    let start_dist = step / 4;
    let mut walk = Vec::new();
    let mut mask = 0u32;
    loop {
        let id = walk.len() as i32 + 1;
        if g.grid.get(x, y).is_none() || id >= 0x40 {
            return Err(Unported::WalkOffGrid);
        }
        g.make_region(id, x, y);
        g.lakes.flags[id as usize] |= 1;
        mask |= 1u32.wrapping_shl(id as u32);
        walk.push((x, y));
        let turn = (g.draws.get(site::STEP) & 0x3fff) + 0x6000;
        (x, y) = project(x, y, Angle((turn << 16).wrapping_add(angle)), step);
        let d = vector_dist(x - xs / 2, y - ys / 2);
        if d > reach + avoid_continent || d > m - avoid_continent {
            break;
        }
    }
    let n = walk.len() as i32;
    // 3. The seats.
    let shift = i32::from(n < players * 2);
    let (mut ends, mut evens) = (Vec::new(), Vec::new());
    for j in 0..players {
        if j & 1 != 0 {
            ends.push(n - ((j - 1) >> shift) - 1);
        } else {
            evens.push(j >> shift);
        }
    }
    ends.extend(evens.into_iter().rev());
    let (_, cont) = fill_cont(teams, g.draws);
    let free: Vec<i32> = (0..8).filter(|&t| !teams.contains(&Some(t))).collect();
    let team = |s: usize| teams[s].unwrap_or(NO_TEAM);
    let (mut used, mut taken) = ([false; 8], [false; 8]);
    let mut seat_of: [Option<(i32, i32)>; 8] = [None; 8];
    let mut order = vec![-1; 8];
    let mut c = 0;
    for &idx in &ends {
        let mut cursor = 0usize;
        let mut stays = false;
        for s in 0..8 {
            if teams[s].is_none() || used[s] {
                continue;
            }
            let t = team(s);
            let slot_team = if t == NO_TEAM {
                while taken.get(cursor).copied().unwrap_or(false) {
                    cursor += 1;
                }
                cursor += 1;
                free.get(cursor - 1).copied().unwrap_or(-1)
            } else {
                t
            };
            if (0..8).contains(&slot_team) && cont[slot_team as usize] == c {
                used[s] = true;
                seat_of[s] = Some(walk[idx as usize]);
                order[s] = idx;
                if t == NO_TEAM {
                    taken[cursor - 1] = true;
                }
                stays = (s + 1..8).any(|s2| {
                    teams[s2].is_some() && team(s2) != NO_TEAM && cont[team(s2) as usize] == c
                });
                break;
            }
        }
        if !stays {
            c += 1;
        }
    }
    for s in 0..8 {
        if teams[s].is_some() {
            let (sx, sy) = seat_of[s].ok_or(Unported::Unseated)?;
            g.grid.add_starting_location(sx, sy);
        }
    }
    order.retain(|&i| i >= 0);
    // 4. The continent.
    let mut passes = 0;
    loop {
        passes += 1;
        let (mut total, mut short) = (0, 0);
        for id in 1..=n {
            let want = g.lakes.points[id as usize].len() as i32 + 0x30;
            if g.grow_region(id, want, xs) {
                short += 1;
            }
            total += g.lakes.points[id as usize].len() as i32;
        }
        if total >= size && g.lakes.adjacent[1] & mask == mask {
            break;
        }
        if short >= n {
            break;
        }
    }
    for id in 2..=n as usize {
        let moved = std::mem::take(&mut g.lakes.points[id]);
        for (px, py) in moved {
            g.lakes.points[1].push((px, py));
            g.grid.cell_mut(px, py).region = 1;
        }
    }
    let mut regions = Regions::default();
    regions.clear_all(g.grid);
    regions.find_all(g.grid);
    g.lakes = Lakes::cleared();
    for id in 1..0x40 {
        g.lakes.points[id] = coords(g.grid, id as i16);
    }
    // 5. The islands.
    let odd = g.draws.get(site::ODD_ISLAND) & 1;
    // The listing's `edx` is `xs / 15 + r % 2` throughout (both
    // `MapFairness::clear` calls leave it): past the allowance or at it the
    // count is at least the players (0x6997f3), under it `edx` alone.
    let mut islands = xs / 15 + odd;
    if allow <= players {
        islands = islands.max(players);
    }
    islands += islands & 1;
    let sea = coords(g.grid, 0x41);
    for &(px, py) in &sea {
        g.grid.cell_mut(px, py).region = 0;
    }
    let mut tally = Fairness {
        teams: (0..players).collect(),
        dists: [0; 8],
        fairness: [0; 8],
        lowest_dist: -1,
        highest_player: STALE_HIGHEST,
    };
    let (mut left, mut forced, mut fails) = (islands, false, 0);
    let (mut id, mut grown, mut attempts) = (1, 0, 0u32);
    let count = sea.len() as i32;
    while left != 0 {
        attempts += 1;
        let r = g.draws.get(site::SPACING) % 0x37 + 0xc1;
        let spacing = ((avoid_continent + isqrt(r)) >> 1).max(5);
        let first = if count - 1 > 0 {
            match g.draws.get(site::CANDIDATE) % count {
                0 => 1,
                k => k,
            }
        } else {
            1
        };
        let mut i = first;
        let mut tested = 0;
        let found = loop {
            if i == count {
                i = 0;
            }
            let (cx, cy) = sea[i as usize];
            if g.land_dist(cx, cy) >= spacing {
                tally.calc_distances(g.grid, cx, cy);
                if forced || tally.lowest_dist == tally.highest_player {
                    break Some((cx, cy));
                }
            }
            i += 1;
            if i == first - 1 {
                break None;
            }
            tested += 1;
            if tested > 2 * count + 2 {
                return Err(Unported::EndlessSearch);
            }
        };
        let whole = match found {
            Some((cx, cy)) => {
                tally.add(1);
                id += 1;
                g.make_region(id, cx, cy);
                !g.grow_region(id, 0xdc, spacing + 1)
            }
            None => false,
        };
        if whole {
            grown += 1;
            left -= 1;
            forced = false;
            fails = 0;
        } else {
            fails += 1;
            forced = true;
            if fails <= 4 {
                tally.add(-1);
            } else {
                left -= 1;
            }
        }
    }
    // 6.
    let mut lists: Vec<Vec<(i32, i32)>> = g.lakes.points.clone();
    lists.resize(0x80, Vec::new());
    check_player_land_guarded(g.grid, &mut lists, None);
    Ok(WarringStates {
        map: g.map,
        start_dist,
        walk,
        order,
        passes,
        islands,
        grown,
        attempts,
    })
}
