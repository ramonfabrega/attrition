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
