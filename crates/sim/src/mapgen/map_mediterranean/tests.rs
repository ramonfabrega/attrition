use super::*;
use crate::mapgen::map_east_west::is_near_ocean;

/// Mediterranean's stock values on a Standard map: `AVOID_CENTER` 12,
/// `AVOID_CONTINENT` 8, `AVOID_EDGE_0..3` 12.
fn style() -> LakeStyle {
    LakeStyle {
        avoid_center: 12,
        avoid_continent: 8,
        avoid_edge: [12; 4],
        base_edge: 0,
        standard: 60,
        start_land: 6,
        overage_allowance: 8,
    }
}

/// A land map with a water disc of radius 4 about `(15, 15)`.
fn pond() -> MapGrid {
    pond_of(4)
}

/// A land map with a water disc of radius `r` about `(15, 15)`.
fn pond_of(r: i32) -> MapGrid {
    let mut grid = MapGrid::new(30, 30);
    grid.wipe();
    for c in &mut grid.cells {
        c.land = 0;
    }
    for y in 0..30 {
        for x in 0..30 {
            if (x - 15) * (x - 15) + (y - 15) * (y - 15) <= r * r {
                grid.cell_mut(x, y).land = 2;
            }
        }
    }
    grid
}

/// The walk's candidates, read off the listing rather than the port:
/// radius `3·xs/2` falling by 3 while the next is at least `reach`, the
/// angle turned by 0x71c71c6 each time the radius runs out; a point
/// whose five orthogonal cells sit inside the outer ring, that is land
/// and `is_near_ocean(…, 3, 9)`, is the start.
fn listed_start(grid: &MapGrid, sea: (i32, i32), angle: i32, reach: i32) -> ((i32, i32), i32) {
    let (xs, ys) = (grid.xs, grid.ys);
    let inside = |x: i32, y: i32| (1..xs - 1).contains(&x) && (1..ys - 1).contains(&y);
    for turn in 0..1024i32 {
        let a = angle.wrapping_add(turn.wrapping_mul(0x071c_71c6));
        let mut r = xs * 3 / 2;
        while r - 3 >= reach {
            let (x, y) = project(sea.0, sea.1, Angle(a), r);
            if ORTHOG.iter().all(|&(dx, dy)| inside(x + dx, y + dy))
                && !grid.cell(x, y).is_ocean()
                && is_near_ocean(grid, x, y, 3, 9)
            {
                return ((x, y), turn);
            }
            r -= 3;
        }
    }
    panic!("no start in 1024 turns at {angle:#x}, reach {reach}");
}

/// The walk takes the listing's start on every angle, on a pond where
/// most angles turn and many starts stand on the outer ring's band;
/// `reach` 12 is a radius the walk's steps of 3 from 45 land on, so a
/// step that stops at `reach` rather than under it is caught.
/// (On a pond of 4 with `reach` 10, angle 0 turns some 8.45 million
/// times: 36 turns of 0x71c71c6 come 8 short of a circle, and the walk
/// creeps until a projection rounds onto a cell near the water.)
#[test]
fn the_walk_in_is_the_listing_s_candidates() {
    {
        let (radius, reach) = (7, 12);
        let grid = pond_of(radius);
        let mut turned = 0;
        let mut band = 0;
        for k in 0..64 {
            let angle = (k as u32).wrapping_mul(0x0400_0000) as i32;
            let (x, y) = walk_in(&grid, (15, 15), angle, reach);
            let (want, turns) = listed_start(&grid, (15, 15), angle, reach);
            assert_eq!((x, y), want, "pond {radius}, angle {k}");
            turned += usize::from(turns > 0);
            band += usize::from(x == 2 || y == 2 || x == 27 || y == 27);
        }
        eprintln!("pond {radius}: {turned} of 64 turned, {band} on the band");
        assert!(
            turned > 0 && band > 0,
            "{turned} turned, {band} on the band"
        );
    }
}

/// A HALFLAND cell is land to the walk: `is_ocean` refuses it, so the
/// near test runs on it and it can take the start.
#[test]
fn a_halfland_cell_is_land_to_the_walk() {
    let mut grid = pond();
    let angle = 0x4000_0000; // due east: x grows, y holds.
    let (x, y) = walk_in(&grid, (15, 15), angle, 4);
    grid.cell_mut(x, y).flags |= 0x100;
    grid.cell_mut(x, y).land = 2;
    assert_eq!(walk_in(&grid, (15, 15), angle, 4), (x, y));
    grid.cell_mut(x, y).flags &= !0x100;
    assert_ne!(walk_in(&grid, (15, 15), angle, 4), (x, y));
}

/// The whole mask on a Standard two-nation map: the sea is region 1
/// inverted, so the opening cell's ring is water and the corners land;
/// two starts on land; the island's cells land in the sea; the draws
/// start with the angle's halves and the sea's opening.
#[test]
fn the_sea_is_inverted_and_the_island_grows_in_it() {
    for seed in (1u32..=24).chain([12345, 0xdead_beef]) {
        let mut grid = MapGrid::new(60, 60);
        let mut d = Draws::new(seed);
        let m = mediterranean_make_continents(&mut grid, 2, style(), &mut d).expect("allowed");
        let sites: Vec<u32> = d.log.iter().take(4).map(|x| x.0).collect();
        assert_eq!(
            sites,
            [site::ANGLE_HI, site::ANGLE_LO, site::SEA_X, site::SEA_Y]
        );
        assert!(d.log.iter().any(|x| x.0 == site::ISLAND_Y));
        assert!((30..=31).contains(&m.sea.0) && (30..=31).contains(&m.sea.1));
        assert_eq!(m.starts.len(), 2);
        for &(x, y) in &m.starts {
            assert!(!grid.cell(x, y).is_ocean(), "{seed}: start ({x}, {y})");
        }
        for (x, y) in [(0, 0), (59, 0), (0, 59), (59, 59)] {
            assert!(!grid.cell(x, y).is_ocean(), "{seed}: corner ({x}, {y})");
        }
        let wet = (0..60)
            .flat_map(|y| (0..60).map(move |x| (x, y)))
            .filter(|&(x, y)| grid.cell(x, y).is_ocean())
            .count();
        assert!(wet > 600 && wet < 1300, "{seed}: {wet} water cells");
        assert_eq!(m.map.avoid_continent, 8);
        // `check_player_land`'s disc to `start_land` round each
        // start-city cell is land; the island may have taken some of it.
        let t = crate::units::think::scout::circle::table();
        let cities: Vec<(i32, i32)> = grid.starts[2]
            .list
            .iter()
            .zip(&grid.starts[3].list)
            .map(|(&x, &y)| (x, y))
            .collect();
        for (cx, cy) in cities {
            for i in 0..t.radius[style().start_land as usize] as usize {
                let (x, y) = (cx + i32::from(t.x[i]), cy + i32::from(t.y[i]));
                if let Some(c) = grid.get(x, y) {
                    assert!(!c.is_ocean(), "{seed}: ({x}, {y}) by ({cx}, {cy})");
                }
            }
        }
    }
}

/// A lobby past its allowance takes `scale_land_area`'s float path:
/// refused before a draw.
#[test]
fn a_lobby_past_its_allowance_draws_nothing() {
    let mut grid = MapGrid::new(60, 60);
    let mut d = Draws::new(1);
    let s = LakeStyle {
        overage_allowance: 2,
        ..style()
    };
    assert_eq!(mediterranean_make_continents(&mut grid, 3, s, &mut d), None);
    assert!(d.log.is_empty());
}
