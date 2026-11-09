use super::*;

#[test]
fn the_spacing_root_is_the_integer_newton_loop() {
    for n in 0..5000 {
        let r = isqrt(n);
        assert!(r * r <= n && (r + 1) * (r + 1) > n, "isqrt({n}) = {r}");
    }
}

/// A pool the lakes enclose is merged into land; the open water
/// round them, the larger region, is left. 7807 cannot say this on the
/// corpus's one Great Lakes run, whose background is one region.
#[test]
fn eliminate_pools_keeps_the_largest_water() {
    let mut grid = MapGrid::new(10, 10);
    grid.wipe();
    grid.clear_regions();
    for y in 3..=7 {
        for x in 3..=7 {
            if x == 3 || x == 7 || y == 3 || y == 7 {
                grid.cell_mut(x, y).land = 0;
            }
        }
    }
    eliminate_pools(&mut grid);
    for y in 0..10 {
        for x in 0..10 {
            let inside = (4..=6).contains(&x) && (4..=6).contains(&y);
            let ring = (3..=7).contains(&x) && (3..=7).contains(&y);
            let want = if ring || inside { 0 } else { 2 };
            assert_eq!(grid.cell(x, y).land, want, "({x}, {y})");
        }
    }
}

/// `EDGES_ONLY` lists only the water regions met on the outer ring, so
/// an enclosed pool is left water where `ENTIRE_WORLD` merges it
/// (Mediterranean's last call, `docs/MAPGEN.md` §29).
#[test]
fn edges_only_spares_a_pool_off_the_outer_ring() {
    let ring = |edges_only: bool| {
        let mut grid = MapGrid::new(10, 10);
        grid.wipe();
        grid.clear_regions();
        for y in 3..=7 {
            for x in 3..=7 {
                if x == 3 || x == 7 || y == 3 || y == 7 {
                    grid.cell_mut(x, y).land = 0;
                }
            }
        }
        eliminate_pools_in(&mut grid, edges_only);
        (grid.cell(5, 5).land, grid.cell(0, 0).land)
    };
    assert_eq!(ring(false), (0, 2), "the whole world merges the pool");
    assert_eq!(
        ring(true),
        (2, 2),
        "the outer ring lists the open sea alone"
    );
}

/// A pool that touches the outer ring on one side only is listed under
/// `EDGES_ONLY`, on each of the four: two such seas merge the smaller.
#[test]
fn edges_only_merges_a_bay_on_each_side() {
    // A U of land walls a 2×2 bay off the middle of the top side.
    let wall = [
        (3, 0),
        (3, 1),
        (3, 2),
        (4, 2),
        (5, 2),
        (6, 2),
        (6, 1),
        (6, 0),
    ];
    let bay = [(4, 0), (5, 0), (4, 1), (5, 1)];
    // The four sides: top, left, bottom, right.
    let turn = |side: i32, (x, y): (i32, i32)| match side {
        0 => (x, y),
        1 => (y, x),
        2 => (x, 9 - y),
        _ => (9 - y, x),
    };
    for side in 0..4 {
        let mut grid = MapGrid::new(10, 10);
        grid.wipe();
        grid.clear_regions();
        for &c in &wall {
            let (x, y) = turn(side, c);
            grid.cell_mut(x, y).land = 0;
        }
        eliminate_pools_in(&mut grid, true);
        for &c in &bay {
            let (x, y) = turn(side, c);
            assert_eq!(grid.cell(x, y).land, 0, "side {side}: ({x}, {y}) merged");
        }
        assert_eq!(grid.cell(8, 8).land, 2, "side {side}: the open sea left");
    }
}

/// `point`'s masks for a record with flag bit 1 (Australian Outback,
/// MAPGEN §30.1): records 20 and 21 meet first, then 21 meets 9, and
/// 20 — which never touched 9 — learns 9's bit through the pass over
/// the first 32 records. A record without the flag keeps no mask.
#[test]
fn a_join_reaches_every_record_whose_mask_meets_it() {
    let mut grid = MapGrid::new(20, 20);
    grid.wipe();
    grid.clear_regions();
    let mut draws = Draws::new(1);
    let mut g = Grower {
        grid: &mut grid,
        lakes: Lakes::cleared(),
        draws: &mut draws,
        style: LakeStyle {
            avoid_center: 0,
            avoid_continent: 0,
            avoid_edge: [0; 4],
            base_edge: 0,
            standard: 60,
            start_land: 6,
            overage_allowance: 8,
        },
        map: LakeMap {
            avoid_continent: 0,
            avoid_modify: 0,
        },
    };
    g.make_region(20, 2, 2);
    g.make_region(21, 10, 10);
    g.make_region(9, 16, 16);
    g.make_region(5, 2, 16);
    for id in [20, 21, 9] {
        g.lakes.flags[id] |= 1;
    }
    assert!(g.point(21, 3, 2), "21 beside 20");
    let pair = (1 << 20) | (1 << 21);
    assert_eq!(g.lakes.adjacent[20], pair);
    assert_eq!(g.lakes.adjacent[21], pair);
    assert!(g.point(21, 15, 15), "21 beside 9");
    let all = pair | (1 << 9);
    assert_eq!(g.lakes.adjacent[9], all);
    assert_eq!(g.lakes.adjacent[21], all);
    assert_eq!(g.lakes.adjacent[20], all, "20 learns 9 through 21");
    assert!(g.point(5, 15, 17), "the unflagged 5 beside 9");
    assert_eq!(g.lakes.adjacent[5], 0, "5 keeps no mask");
    assert_eq!(g.lakes.adjacent[9], all, "nor writes 9's");
}

#[test]
fn a_lobby_past_its_allowance_is_refused() {
    assert_eq!(scale_land_area(8, 4, 4), Some(8));
    assert_eq!(scale_land_area(8, 5, 4), None);
}
