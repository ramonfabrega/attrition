use super::*;

/// The stock East Meets West values on a Standard map (`AVOID_*` as
/// Great Lakes' and East Indies' fixtures carry them).
fn style() -> LakeStyle {
    LakeStyle {
        avoid_center: 3,
        avoid_continent: 8,
        avoid_edge: [2; 4],
        base_edge: 0,
        standard: 60,
        start_land: 6,
        overage_allowance: 8,
    }
}

fn lone(n: usize) -> [Option<i32>; 8] {
    let mut t = [None; 8];
    for s in t.iter_mut().take(n) {
        *s = Some(NO_TEAM);
    }
    t
}

/// Seats with no team take continents in order and count 1; a team
/// opens one continent and counts its members. Every ordered pair of
/// equal counts draws once, and an odd draw swaps the pair in `cont`.
#[test]
fn fill_cont_counts_teams_and_swaps_equal_continents_on_an_odd_draw() {
    let mut d = Draws::new(1);
    let (count, cont) = fill_cont(&lone(2), &mut d);
    assert_eq!(&count[..3], &[1, 1, 0]);
    assert_eq!(d.log.len(), 2, "(0, 1) and (1, 0)");
    let mut r = Draws::new(1);
    let (a, b) = (r.get(0) & 1, r.get(0) & 1);
    let want = if a ^ b == 1 { [1, 0] } else { [0, 1] };
    assert_eq!(&cont[..2], &want);

    let mut d = Draws::new(1);
    let teams = [Some(3), Some(5), Some(3), None, None, None, None, None];
    let (count, cont) = fill_cont(&teams, &mut d);
    assert_eq!(&count[..3], &[2, 1, 0]);
    assert!(d.log.is_empty(), "no two continents share a count");
    assert_eq!((cont[3], cont[5], cont[0]), (0, 1, -1));
}

/// An empty `cont` slot is written by both arms of a swap, so it ends
/// as the first of the pair.
#[test]
fn fill_cont_s_swap_writes_an_empty_slot_twice() {
    let teams = [Some(0), Some(1), None, None, None, None, None, None];
    let seed = (1..)
        .find(|&s| {
            let mut r = Draws::new(s);
            r.get(0) & 1 == 1
        })
        .expect("an odd draw");
    let mut d = Draws::new(seed);
    let (_, cont) = fill_cont(&teams, &mut d);
    assert_eq!(cont[2], 0, "the (0, 1) swap's second arm leaves i = 0");
}

/// The mean of a region's cells, divided toward zero, over a list longer
/// than the original's unroll of eight with a tail of two and one.
#[test]
fn the_centroid_is_the_truncated_mean() {
    let pts: Vec<(i32, i32)> = (0..11).map(|i| (i * 3, 20 - i)).collect();
    assert_eq!(find_region_centroid(&pts), (165 / 11, 165 / 11));
    assert_eq!(find_region_centroid(&[(1, 1), (2, 2)]), (1, 1));
    assert_eq!(find_region_centroid(&[(-3, 0), (0, 0)]), (-1, 0));
}

/// A west edge of water three high widens into a land cell beside it;
/// where the first column is then water three high, the next one in
/// widens too. The other three edges mirror the first rule.
#[test]
fn an_edge_canal_is_widened_twice() {
    let mut g = MapGrid::new(10, 10);
    for y in 2..=6 {
        g.cell_mut(0, y).land = 2;
        g.cell_mut(1, y).land = 2;
    }
    g.cell_mut(1, 4).land = 0;
    eliminate_edge_canals(&mut g);
    for y in 0..10 {
        let row: Vec<i8> = (0..4).map(|x| g.cell(x, y).land).collect();
        let want = match y {
            4 | 5 => vec![2, 2, 2, 0],
            2 | 3 | 6 => vec![2, 2, 0, 0],
            _ => vec![0, 0, 0, 0],
        };
        assert_eq!(row, want, "row {y}");
    }
    let mut g = MapGrid::new(10, 10);
    for i in 2..=6 {
        g.cell_mut(9, i).land = 2;
        g.cell_mut(i, 0).land = 2;
        g.cell_mut(i, 9).land = 2;
    }
    eliminate_edge_canals(&mut g);
    assert_eq!((g.cell(8, 4).land, g.cell(7, 4).land), (2, 0));
    assert_eq!((g.cell(4, 1).land, g.cell(4, 2).land), (2, 0));
    assert_eq!((g.cell(4, 8).land, g.cell(4, 7).land), (2, 0));
    assert_eq!((g.cell(8, 2).land, g.cell(8, 6).land), (0, 0), "the ends");
}

/// Near water: some wet cell in the rings either side of `far`, none
/// within `near`.
#[test]
fn is_near_ocean_wants_a_far_ring_wet_and_the_near_disc_dry() {
    let mut g = MapGrid::new(30, 30);
    for c in &mut g.cells {
        c.land = 0;
    }
    assert!(!is_near_ocean(&g, 15, 15, 3, 9), "no water at all");
    assert!(is_near_ocean(&g, 15, 15, 3, 0), "only the dry disc asked");
    g.cell_mut(24, 15).land = 2;
    assert!(is_near_ocean(&g, 15, 15, 3, 9));
    g.cell_mut(17, 15).land = 2;
    assert!(!is_near_ocean(&g, 15, 15, 3, 9), "wet within three");
    assert!(is_near_ocean(&g, 15, 15, 1, 9));
    assert!(!is_near_ocean(&g, -1, 15, 0, 0), "off the grid");
}

/// The guard keeps another continent's coast: a circle cell with a
/// cell of another region of more than four cells in its spiral to ring
/// `avoid_continent` stays water; the rest of the disc becomes land of
/// the city's region.
#[test]
fn check_player_land_spares_a_cell_beside_another_continent() {
    let build = |other: bool| {
        let mut g = MapGrid::new(30, 30);
        g.wipe();
        g.add_starting_location(10, 10);
        for (x, y) in [(10, 10), (9, 10), (10, 9), (9, 9)] {
            let c = g.cell_mut(x, y);
            c.land = 0;
            c.region = 1;
        }
        if other {
            for y in 6..=14 {
                let c = g.cell_mut(15, y);
                c.land = 0;
                c.region = 2;
            }
        }
        let mut lists: Vec<Vec<(i32, i32)>> = (0..128).map(|r| coords(&g, r as i16)).collect();
        check_player_land(&mut g, &mut lists, 4);
        g
    };
    let (g, open) = (build(true), build(false));
    assert_eq!((g.cell(8, 10).land, g.cell(8, 10).region), (0, 1));
    assert_eq!(
        (g.cell(12, 10).land, g.cell(12, 10).region),
        (2, 0x40),
        "guarded"
    );
    assert_eq!((open.cell(12, 10).land, open.cell(12, 10).region), (0, 1));
}

/// The second pass widens the band to 3 from the edge, drops the far
/// ring and relaxes the spacing to `min(6, min_dist)`; a one-cell list
/// draws nothing.
#[test]
fn place_start_in_region_s_second_pass_is_looser() {
    let mut g = MapGrid::new(30, 30);
    g.cell_mut(3, 3).region = 5;
    let mut d = Draws::new(1);
    assert_eq!(place_start_in_region(&g, 5, 9, &mut d), Some((3, 3)));
    assert!(d.log.is_empty(), "one cell, no draw");
    g.add_starting_location(3, 10);
    let got = place_start_in_region(&g, 5, 9, &mut d);
    assert_eq!(got, Some((3, 3)), "7 is at least min(6, 9)");
    g.add_starting_location(8, 3);
    let got = place_start_in_region(&g, 5, 9, &mut d);
    assert_eq!(got, None, "5 from a start");
    g.cell_mut(3, 3).region = 0;
    g.cell_mut(20, 20).region = 5;
    g.cell_mut(20, 24).region = 5;
    g.cell_mut(20, 21).land = 2;
    g.cell_mut(20, 25).land = 2;
    let got = place_start_in_region(&g, 5, 6, &mut d);
    assert_eq!(got, None, "water within two of both");
    assert_eq!(d.log.len(), 2, "a draw a pass");
}

/// A whole mask on a 60×60, two lone players: two continents, each start
/// on its own continent's region, the starts apart, and the run
/// repeatable.
#[test]
fn two_lone_players_start_on_their_own_continents() {
    for seed in [7u32, 12345, 99] {
        let run = || {
            let mut g = MapGrid::new(60, 60);
            let mut d = Draws::new(seed.wrapping_mul(2_654_435_761));
            let m = east_west_make_continents(&mut g, 2, 2, &lone(2), style(), &mut d)
                .expect("inside the allowance");
            (g, d, m)
        };
        let (g, d, m) = run();
        assert_eq!(m.starts.len(), 2);
        let (a, b) = (m.starts[0], m.starts[1]);
        assert!(vector_dist(a.0 - b.0, a.1 - b.1) >= 8, "seed {seed}");
        for &(x, y) in &m.starts {
            assert_eq!(g.cell(x, y).land, 0, "a start on land, seed {seed}");
        }
        assert_eq!(d.log[0].0, site::FILL_SWAP);
        let (g2, d2, m2) = run();
        assert_eq!((m, d.log), (m2, d2.log));
        assert!(g.cells == g2.cells);
    }
}
