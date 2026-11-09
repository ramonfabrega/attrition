use super::*;

/// A hand-built East Indies style: the stock file's values on a
/// Standard map, with `base_edge` given.
fn style(base_edge: i32) -> LakeStyle {
    LakeStyle {
        avoid_center: 3,
        avoid_continent: 4,
        avoid_edge: [2; 4],
        base_edge,
        standard: 60,
        start_land: 6,
        overage_allowance: 8,
    }
}

fn run(xs: i32, players: i32, seed: u32) -> (MapGrid, Draws, IslandMap) {
    let mut grid = MapGrid::new(xs, xs);
    let mut draws = Draws::new(seed.wrapping_mul(2_654_435_761));
    let m = east_indies_make_continents(&mut grid, players, style((seed % 4) as i32), &mut draws)
        .expect("inside the allowance");
    (grid, draws, m)
}

/// **A stalled pass restarts the generation** (3820a): no corpus lobby
/// stalls, so a six-player 60×60 that does holds the arm. The restart
/// draws the angle again and `wipe` empties the start arrays, so the
/// grid ends with one start a player.
#[test]
fn a_stalled_pass_restarts_the_generation() {
    let (grid, draws, m) = run(60, 6, 2);
    assert_eq!(m.restarts, 1);
    let angles = draws.log.iter().filter(|d| d.0 == site::ANGLE).count();
    assert_eq!(angles, 2, "one angle a generation");
    assert_eq!(grid.starts[0].list.len(), 6);
    assert_eq!(grid.starts[2].list.len(), 24);
}

/// Under four players the angle is one draw, a quadrant's middle; with
/// four or more it is two, high half then low.
#[test]
fn the_angle_is_two_draws_from_four_players() {
    for players in [2, 3, 4, 6] {
        let (_, draws, _) = run(60, players, 7);
        assert_eq!(draws.log[0].0, site::ANGLE, "{players} players");
        assert_eq!(
            draws.log[1].0 == site::ANGLE_LO,
            players >= 4,
            "{players} players"
        );
    }
}

/// With three players the ring starts at `(m/2)·7/8`, two at `m/2`:
/// the first start is the first radius down from there whose point
/// clears every edge by `max(avoid_edge) + 4`. No corpus lobby shrinks
/// the ring (3820a), so edges of 10 here make it fall; on 60×60 the
/// edges bind before the starting radius, so a 120×120 holds the ring
/// where it starts.
#[test]
fn the_ring_s_radius_reads_the_player_count() {
    let cases = [
        (60, 2, 30, 2),
        (60, 3, 26, 2),
        (60, 2, 30, 10),
        (60, 3, 26, 10),
        (120, 3, 52, 2),
    ];
    for (xs, players, from, edges) in cases {
        let mut grid = MapGrid::new(xs, xs);
        let mut draws = Draws::new(11u32.wrapping_mul(2_654_435_761));
        let st = LakeStyle {
            avoid_edge: [edges; 4],
            ..style(3)
        };
        east_indies_make_continents(&mut grid, players, st, &mut draws).expect("allowed");
        let word = Draws::new(draws.log[0].1).get(site::ANGLE);
        let angle = (word % 4)
            .wrapping_mul(0x3fff_ffff)
            .wrapping_add(0x1fff_ffff)
            .wrapping_add((u32::MAX / players as u32) as i32);
        let e = edges + 4;
        let mut radius = from;
        let at = loop {
            let (x, y) = project(xs / 2, xs / 2, Angle(angle), radius);
            if x > e && xs - x > e && y > e && xs - y > e {
                break (x, y);
            }
            radius -= 1;
        };
        match (xs, edges) {
            (60, 10) => assert!(radius < from, "the ring fell"),
            (120, _) => assert_eq!(radius, from, "the ring held"),
            _ => {}
        }
        assert_eq!(
            (grid.starts[0].list[0], grid.starts[1].list[0]),
            at,
            "{xs}×{xs}, {players} players, edges {edges}"
        );
    }
}
