use super::*;

/// The stock files' values on a Standard map: `default.xml`'s
/// `AVOID_*`, edges 0.
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

fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, NewWorld) {
    let mut grid = MapGrid::new(xs, xs);
    let mut draws = Draws::new(seed);
    let m = new_world_make_continents(&mut grid, players, style(), &mut draws)
        .expect("inside the allowance");
    (grid, draws, m)
}

/// Under four players one draw a generation, the quadrant; from four,
/// two, the angle's halves.
#[test]
fn the_angle_is_one_draw_under_four_players_and_two_from_four() {
    let (_, d, m) = run(80, 2, 12345);
    let opening: Vec<u32> = d.log.iter().map(|x| x.0).take(2).collect();
    assert_eq!(opening[0], site::ANGLE_HI);
    assert_ne!(opening[1], site::ANGLE_LO);
    assert_eq!(m.restarts, 0);
    let (_, d, _) = run(80, 4, 12345);
    let opening: Vec<u32> = d.log.iter().map(|x| x.0).take(2).collect();
    assert_eq!(opening, [site::ANGLE_HI, site::ANGLE_LO]);
}

/// Two players on 80×80: the ring's radius `((80/2)·7)/8` = 35, the
/// centre `(6400 − 500)·2/3` cells; three players take `·4/10`.
#[test]
fn the_centre_s_size_is_two_thirds_under_three_players_and_four_tenths_from_three() {
    let (_, _, m) = run(80, 2, 12345);
    assert_eq!((m.radius, m.centre), (35, (6400 - 500) * 2 / 3));
    let (_, _, m) = run(80, 3, 12345);
    assert_eq!(m.centre, (6400 - 750) * 4 / 10);
}

/// The starts sit on the ring at the quadrant's diagonal, stepped half
/// a turn for two; the centre is land, a corner water.
#[test]
fn the_starts_are_on_the_ring_and_the_centre_is_land() {
    let (grid, _, _) = run(80, 2, 12345);
    let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
    let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
    assert!((39..=41).contains(&((x0 + x1) / 2)) && (39..=41).contains(&((y0 + y1) / 2)));
    let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
    assert!((68 * 68..=72 * 72).contains(&d2), "{d2}");
    assert!((x0 - 40).abs() > 10 && (y0 - 40).abs() > 10, "a diagonal");
    assert_eq!(grid.cell(40, 40).land, 0, "the centre continent");
    assert_eq!(grid.cell(0, 0).land, 2, "the corner is sea");
}

/// No inversion: the islands and the centre are land, and every pool
/// left is merged into it, so one water body remains.
#[test]
fn the_regions_stay_land() {
    let (grid, _, m) = run(80, 2, 12345);
    let land = grid.cells.iter().filter(|c| c.land == 0).count() as i32;
    assert!(land >= m.centre + 250, "{land} land cells");
    assert!(land < 6400 - 400, "{land} land cells");
}

/// A lobby past the map size's allowance refuses before any draw.
#[test]
fn a_lobby_past_its_allowance_refuses_before_a_draw() {
    let mut grid = MapGrid::new(60, 60);
    let mut draws = Draws::new(1);
    let s = LakeStyle {
        overage_allowance: 2,
        ..style()
    };
    assert!(new_world_make_continents(&mut grid, 3, s, &mut draws).is_none());
    assert!(draws.log.is_empty());
}
