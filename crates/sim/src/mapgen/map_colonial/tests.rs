use super::*;

/// The stock files' values on a Standard map: `default.xml`'s
/// `AVOID_CENTER` and `AVOID_CONTINENT`, `COLONIALPOWERS.xml`'s four
/// zero edges.
/// [`a_short_growth_restarts_the_generation`]'s pin, the port's own.
const LAND_AND_DRAWS_28: (usize, usize) = (545, 8428);

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
/// 24×24 the ring is `3·12/4 = 9` about 12, so the ring's own points
/// fall as near as 3 to an edge, and every start lands in `5..=19`.
#[test]
fn a_start_is_held_five_in_from_the_edge() {
    let mut held = 0;
    for (players, outsiders) in [(2, 0), (4, 1), (6, 3), (8, 4)] {
        let (grid, _, _) = run(24, players, outsiders, 99);
        for (&x, &y) in grid.starts[0].list.iter().zip(&grid.starts[1].list) {
            assert!((5..=19).contains(&x) && (5..=19).contains(&y), "({x}, {y})");
            held += usize::from(x == 5 || y == 5 || x == 19 || y == 19);
        }
    }
    assert_eq!(held, 20, "every start of these lobbies reaches the clamp");
}

/// A growth that ends short starts the generation again, the angle
/// drawn anew and `avoid_center` restored for the old world's growth:
/// 28×28 at seed 99 stalls twice before a generation holds. The land
/// and the stream's length are the port's own (no take reaches a
/// restart, parked 3951a) — a pin that the restart's two writes move.
#[test]
fn a_short_growth_restarts_the_generation() {
    let (grid, draws, m) = run(28, 2, 0, 99);
    assert_eq!(m.restarts, 2);
    let opens = draws.log.iter().filter(|d| d.0 == site::ANGLE_HI).count();
    assert_eq!(opens, 3);
    let land = grid.cells.iter().filter(|c| c.land == 0).count();
    assert_eq!((land, draws.log.len()), LAND_AND_DRAWS_28);
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
