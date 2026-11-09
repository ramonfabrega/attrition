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

fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, NileDelta) {
    let mut grid = MapGrid::new(xs, xs);
    let mut draws = Draws::new(seed);
    let m = nile_delta_make_continents(&mut grid, players, style(), &mut draws)
        .expect("inside the allowance");
    (grid, draws, m)
}

/// Two draws a generation open it, the angle's halves.
#[test]
fn the_angle_is_two_draws() {
    let (_, d, _) = run(80, 2, 12345);
    let opening: Vec<u32> = d.log.iter().map(|x| x.0).take(3).collect();
    assert_eq!(&opening[..2], [site::ANGLE_HI, site::ANGLE_LO]);
    assert_ne!(opening[2], site::ANGLE_LO);
}

/// Two players on 80×80: the radius `((80/2)·3)/4` = 30, four passes,
/// a continent `(6400/2)·6/8` = 2,400 cells when no growth restarts.
#[test]
fn the_size_is_three_quarters_of_a_share_and_the_radius_three_eighths() {
    let (_, _, m) = run(80, 2, 12345);
    assert_eq!((m.radius, m.passes), (30, 4));
    if m.restarts == 0 {
        assert_eq!(m.size, 2400);
    }
}

/// The floor: a share whose three quarters are under 100 grows 100.
#[test]
fn a_small_share_grows_a_hundred() {
    let (_, _, m) = run(20, 4, 7);
    assert!(m.size <= 100);
    assert_eq!(m.restarts == 0, m.size == 100);
}

/// A restart shrinks the size by a fifth, floored at a quarter share.
#[test]
fn a_restart_shrinks_the_size_by_a_fifth_to_a_quarter_share() {
    for seed in 0..40 {
        let (_, _, m) = run(40, 4, seed);
        let per = 1600 / 4;
        let mut s = (per * 6 / 8).max(100);
        for _ in 0..m.restarts {
            s = (s * 4 / 5).max(per / 4);
        }
        assert_eq!(m.size, s, "seed {seed}");
    }
}

/// The starts sit on the ring, half a turn apart for two; each is land.
#[test]
fn the_starts_are_on_the_ring_and_land() {
    let (grid, _, _) = run(80, 2, 12345);
    let (x0, y0) = (grid.starts[0].list[0], grid.starts[1].list[0]);
    let (x1, y1) = (grid.starts[0].list[1], grid.starts[1].list[1]);
    assert!((39..=41).contains(&((x0 + x1) / 2)) && (39..=41).contains(&((y0 + y1) / 2)));
    let d2 = (x0 - x1) * (x0 - x1) + (y0 - y1) * (y0 - y1);
    assert!((58 * 58..=62 * 62).contains(&d2), "{d2}");
    assert_eq!(grid.cell(x0, y0).land, 0);
    assert_eq!(grid.cell(x1, y1).land, 0);
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
    assert!(nile_delta_make_continents(&mut grid, 3, s, &mut draws).is_none());
    assert!(draws.log.is_empty());
}
