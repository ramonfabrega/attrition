use super::*;

/// Sea Power's stock values on a two-nation 60×60 map, as run1535's
/// lobby reads them.
fn style() -> LakeStyle {
    LakeStyle {
        avoid_center: 0,
        avoid_continent: 8,
        avoid_edge: [0; 4],
        base_edge: 0,
        standard: 60,
        start_land: 6,
        overage_allowance: 2,
    }
}

/// The island is five eighths of a player's share, floored at 80; a
/// lobby past its allowance takes the float path and is refused.
#[test]
fn the_island_is_five_eighths_of_a_share_at_least_80() {
    assert_eq!(island(3600, 2, 2), Some(1125));
    assert_eq!(island(3600, 3, 8), Some(750));
    assert_eq!(island(256, 2, 2), Some(80));
    assert_eq!(island(259, 2, 2), Some(80));
    assert_eq!(island(260, 2, 2), Some(81));
    assert_eq!(island(3600, 3, 2), None);
}

/// A second-pass stall keeps nine tenths, truncated, and never less
/// than a quarter of a share.
#[test]
fn a_whole_stall_keeps_nine_tenths_down_to_a_quarter_share() {
    assert_eq!(shrunk(1125, 3600, 2), 1012);
    assert_eq!(shrunk(1019, 3600, 2), 917);
    assert_eq!(shrunk(500, 3600, 2), 450);
    assert_eq!(shrunk(460, 3600, 2), 450);
    assert_eq!(shrunk(449, 3600, 3), 404);
    assert_eq!(shrunk(100, 3600, 2), 450);
}

/// The angle steps *before* each projection, on the ring `7·(m/2)/8`
/// about the map's middle: the first start is a step past the drawn
/// angle, and the second a step past that.
#[test]
fn the_ring_steps_before_it_projects() {
    for seed in [1u32, 12345, 0xdead_beef, 77] {
        let mut grid = MapGrid::new(60, 60);
        let mut d = Draws::new(seed);
        let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
        let mut twin = Draws::new(seed);
        let hi = twin.get(site::ANGLE_HI) % 0xffff;
        let lo = twin.get(site::ANGLE_LO) % 0xffff;
        let a = hi.wrapping_mul(0x10000).wrapping_add(lo);
        let step = (u32::MAX / 2) as i32;
        if m.quarter_stalls + m.whole_stalls != 0 {
            continue;
        }
        let ring = 30 * 7 / 8;
        let want: Vec<(i32, i32)> = [
            a.wrapping_add(step),
            a.wrapping_add(step).wrapping_add(step),
        ]
        .iter()
        .map(|&a| project(30, 30, Angle(a), ring))
        .collect();
        assert_eq!(m.starts, want, "seed {seed}");
        assert_eq!(&d.log[..2], &twin.log[..2]);
    }
}

/// Each seat's start is a starting location, and each island is land
/// that holds its start city's disc (`check_player_land`'s ring 5).
#[test]
fn every_start_stands_on_land_with_its_disc() {
    let mut grid = MapGrid::new(60, 60);
    let mut d = Draws::new(12345);
    let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
    assert_eq!(m.starts.len(), 2);
    assert_eq!(grid.starts[0].list.len(), 2);
    for &(x, y) in &m.starts {
        assert!(!grid.cell(x, y).is_ocean(), "({x}, {y}) is water");
    }
    assert_eq!(m.map.avoid_continent, 8);
}

/// `check_player_land` makes land of the circle to ring 5 round every
/// start-city cell. On run1535 each 1,125-cell island already covers
/// it; a lone player's 105- and 122-cell islands on 13×13 and 14×14 maps
/// do not — without the call, eight of these twelve lobbies leave
/// water in a disc — and one island means no guard (§33.4).
#[test]
fn every_start_city_has_its_ring_5_disc_of_land() {
    let t = crate::units::think::scout::circle::table();
    let mut filled = 0;
    for (n, seed) in [13, 14]
        .into_iter()
        .flat_map(|n| (1..=6u32).map(move |s| (n, s)))
    {
        let mut grid = MapGrid::new(n, n);
        let mut d = Draws::new(seed);
        let m = sea_power_make_continents(&mut grid, 1, style(), &mut d).expect("allowed");
        assert_eq!(m.whole_stalls + m.quarter_stalls, 0, "seed {seed}");
        let cities: Vec<(i32, i32)> = grid.starts[2]
            .list
            .iter()
            .zip(&grid.starts[3].list)
            .map(|(&x, &y)| (x, y))
            .collect();
        for (cx, cy) in cities {
            for i in t.radius[0] as usize..t.radius[5] as usize {
                let (x, y) = (cx + i32::from(t.x[i]), cy + i32::from(t.y[i]));
                if grid.get(x, y).is_some() {
                    assert!(!grid.cell(x, y).is_ocean(), "seed {seed}: ({x}, {y})");
                    filled += 1;
                }
            }
        }
    }
    assert!(filled > 0);
}

/// Two 125-cell islands on a 20×20 map stall the whole pass, the arm
/// run1535 never takes: each stall shrinks the island once by
/// [`shrunk`] and draws the ring again; a first-pass stall would
/// keep it.
#[test]
fn a_whole_stall_shrinks_the_island_and_draws_again() {
    let mut whole = 0;
    for seed in 1..=24u32 {
        let mut grid = MapGrid::new(20, 20);
        let mut d = Draws::new(seed);
        let m = sea_power_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
        let mut land = 125;
        for _ in 0..m.whole_stalls {
            land = shrunk(land, 400, 2);
        }
        assert_eq!(m.land, land, "seed {seed}");
        // The angle's two draws open every generation.
        let opened = d.log.iter().filter(|e| e.0 == site::ANGLE_HI).count() as u32;
        assert_eq!(opened, 1 + m.whole_stalls + m.quarter_stalls, "seed {seed}");
        whole += m.whole_stalls;
    }
    assert!(whole > 0);
}

/// `avoid_continent` is held to at least 4 after `scale_land_area`.
#[test]
fn avoid_continent_is_at_least_4() {
    let mut grid = MapGrid::new(60, 60);
    let mut d = Draws::new(12345);
    let low = LakeStyle {
        avoid_continent: 2,
        ..style()
    };
    let m = sea_power_make_continents(&mut grid, 2, low, &mut d).expect("allowed");
    assert_eq!(m.map.avoid_continent, 4);
}
