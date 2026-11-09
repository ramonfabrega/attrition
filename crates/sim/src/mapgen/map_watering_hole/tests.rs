use super::*;

/// The stock files' values on a Standard map: `default.xml`'s
/// `AVOID_*` (Africa's file carries none), edges 0.
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

fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, WateringHole) {
    let mut grid = MapGrid::new(xs, xs);
    let mut draws = Draws::new(seed);
    let m = african_watering_hole_make_continents(&mut grid, players, style(), &mut draws)
        .expect("inside the allowance");
    (grid, draws, m)
}

/// The lake is the region grown, inverted: `size·5/100` cells of water
/// (180 on 60×60) about the middle, the rest land — the growth's
/// cross ends it at `size` or a few past.
#[test]
fn the_lake_is_five_percent_of_the_map_and_inverted() {
    let (grid, _, m) = run(60, 2, 12345);
    assert_eq!(m.restarts, 0);
    let water = grid.cells.iter().filter(|c| c.land == 2).count();
    assert!((180..185).contains(&water), "{water} water cells");
    assert_eq!(grid.cell(30, 30).land, 2, "the middle is the lake");
    assert_eq!(grid.cell(0, 0).land, 0, "the corner is land");
    assert_eq!(m.start_dist, 4);
}

/// Four draws a generation before the growth — the angle's halves,
/// then the lake's parities — and two after it, the ring's parities.
#[test]
fn the_ring_s_centre_is_drawn_after_the_lake() {
    let (_, draws, _) = run(60, 2, 12345);
    let sites: Vec<u32> = draws.log.iter().map(|d| d.0).collect();
    assert_eq!(
        sites[..4],
        [site::ANGLE_HI, site::ANGLE_LO, site::LAKE_X, site::LAKE_Y]
    );
    assert_eq!(sites[sites.len() - 2..], [site::RING_X, site::RING_Y]);
}

/// The players sit on the ring at radius `min(m/2 − 4, (m/2)·80/100)`
/// (24 on 60×60) about the ring's centre, half a turn apart for two.
#[test]
fn the_starts_are_on_the_ring() {
    let (grid, _, _) = run(60, 2, 12345);
    let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
    let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
    let mid = ((x0 + x1) / 2, (y0 + y1) / 2);
    assert!((29..=31).contains(&mid.0) && (29..=31).contains(&mid.1));
    let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
    assert!((46 * 46..=50 * 50).contains(&d2), "{d2}");
}

/// A lobby past the map size's allowance refuses before any draw,
/// where `scale_land_area` would take its float path.
#[test]
fn a_lobby_past_its_allowance_refuses_before_a_draw() {
    let mut grid = MapGrid::new(60, 60);
    let mut draws = Draws::new(1);
    let s = LakeStyle {
        overage_allowance: 2,
        ..style()
    };
    assert!(african_watering_hole_make_continents(&mut grid, 3, s, &mut draws).is_none());
    assert!(draws.log.is_empty());
}
