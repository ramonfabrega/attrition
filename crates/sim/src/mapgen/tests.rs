use super::*;

/// **`get_height` is the host's double, bit for bit** (item 3812):
/// every vertex of fractals of several sizes, smoothings and flags,
/// against the listing's operations in the host's own `f64` — the
/// arithmetic the sim refuses, as the oracle.
#[test]
#[allow(clippy::float_arithmetic)]
fn get_height_is_the_listing_s_doubles() {
    for (n, smooth, flags, seed) in [
        (241, 5, 2, 7236u32),
        (241, 3, 2, 14472),
        (61, 0, 2, 7236),
        (97, 4, 0, 99),
        (130, 2, 0, 5),
        (33, 5, 2, 0xdead_beef),
    ] {
        let f = Fractal::init(n, n, smooth, flags, seed);
        let open = flags & 1;
        let (xi, yi) = (
            f64::from(n) / f64::from(n - open + 1),
            f64::from(n) / f64::from(n),
        );
        for y in 0..n {
            for x in 0..n {
                let dx = xi * (f64::from(x) + 0.5);
                let dy = yi * (f64::from(y) + 0.5);
                let (i, j) = (dx as i32, dy as i32);
                let (fx, fy) = (dx - f64::from(i), dy - f64::from(j));
                let (ox, oy) = (1.0 - fx, 1.0 - fy);
                let b = |u: i32, v: i32| f64::from(f.at(u, v));
                let mut t = ox * oy * b(i, j);
                t += 0.0;
                t += oy * fx * b(i + 1, j);
                t += ox * fy * b(i, j + 1);
                t += b(i + 1, j + 1) * (fy * fx);
                let h = (t as i32).clamp(0, 255);
                let want = if flags & 2 != 0 { (h * 100) >> 8 } else { h };
                assert_eq!(
                    i32::from(f.get_height(x, y)),
                    want,
                    "({x}, {y}) of a {n}-wide fractal, smooth {smooth}, flags {flags}"
                );
            }
        }
    }
}

/// The tell §3 names: an empty walk prints 1, and N zero bytes print
/// `N·65536 + 1`.
#[test]
fn the_sum_is_adler_32() {
    assert_eq!(adler32(&[]), 1);
    assert_eq!(adler32(&[0; 16]), 16 * 65536 + 1);
    assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
}

/// Four empty start arrays are part 2 at `map.cpp` 7805 on every
/// trace: 1,048,577, sixteen zero bytes.
#[test]
fn four_empty_start_arrays_are_sixteen_zero_bytes() {
    assert_eq!(MapGrid::new(60, 60).start_part(), 1_048_577);
}

/// Great Sahara's grid at 7807 is uniform whatever the draws: every
/// cell land 0, region 0, `down`/`who`/`who2` −1. run1000's part 5
/// there is 1,267,075,849.
#[test]
fn great_sahara_s_mask_is_one_cell_repeated() {
    let mut g = MapGrid::new(60, 60);
    let mut rng = Rng::new(0x1234);
    great_sahara_make_continents(&mut g, 2, &mut rng);
    assert!(g.cells.iter().all(|c| *c
        == WCell {
            land: 0,
            region: 0,
            ..WCell::WIPED
        }));
    assert_eq!(g.cell_part(), 1_267_075_849);
}

/// A grid from rows of `#` (`BASELAND`) and `.` (`OCEAN`).
fn sketch(rows: &[&str]) -> MapGrid {
    let mut g = MapGrid::new(rows[0].len() as i32, rows.len() as i32);
    for (y, r) in rows.iter().enumerate() {
        for (x, ch) in r.bytes().enumerate() {
            g.cell_mut(x as i32, y as i32).land = if ch == b'#' { 0 } else { 2 };
        }
    }
    g
}

fn regions(g: &MapGrid) -> Vec<Vec<i16>> {
    (0..g.ys)
        .map(|y| (0..g.xs).map(|x| g.cell(x, y).region).collect())
        .collect()
}

/// The partitions' single-precision product is the integer quotient for
/// every frequency of magnitude under 6,420, where the first parts
/// (§11: SSE, correctly rounded; the files write percentages).
#[test]
fn the_partition_float_is_the_integer_quotient() {
    let float = |f: i32| ((f as f32 / 100.0f32) * 255.0f32) as i32;
    for f in -6_419i32..=6_419 {
        assert_eq!(float(f), f * 255 / 100, "frequency {f}");
    }
    assert_ne!(float(6_420), 6_420 * 255 / 100);
    assert_eq!(fertile_partitions(&[80, 5, 5, 10]), [204, 216, 228]);
    assert_eq!(fertile_partitions(&[100]), [] as [u8; 0]);
}

/// Labels: land from 1 by the first free record, sea counting up from
/// 0x41, in y-outer scan order; eight-connected, so a diagonal joins.
#[test]
fn find_all_labels_eight_connected_land_and_sea() {
    let mut g = sketch(&["##..", "..#.", "....", ".#.#"]);
    let mut r = Regions::default();
    r.clear_all(&mut g);
    r.find_all(&mut g);
    assert_eq!(
        regions(&g),
        [
            vec![1, 1, 0x41, 0x41],
            vec![0x41, 0x41, 1, 0x41],
            vec![0x41, 0x41, 0x41, 0x41],
            vec![0x41, 2, 0x41, 3],
        ]
    );
    assert_eq!(
        (r.count[1], r.count[2], r.count[3], r.count[0x41]),
        (3, 1, 1, 11)
    );
    assert_eq!(r.sea, 0x41);
}

/// The sea clamp (`docs/MAPGEN.md` §6), from the reading alone — no map
/// reaches it (parked 3825a): 64 one-cell seas in land, numbered in scan
/// order. The 61st lands on 0x7d and takes record 0x40, which never
/// counted; from the 62nd no record is smaller, so each goes to 0x7e.
#[test]
fn the_sea_clamp_evicts_into_0x40_then_bins() {
    let rows: Vec<String> = (0..8)
        .map(|y| {
            if y % 2 == 0 {
                "#.".repeat(16)
            } else {
                "##".repeat(16)
            }
        })
        .collect();
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut g = sketch(&rows);
    let mut r = Regions::default();
    r.clear_all(&mut g);
    r.find_all(&mut g);
    let sea = |k: i32| g.cell(2 * (k % 16) + 1, 2 * (k / 16)).region;
    assert_eq!((sea(0), sea(59)), (0x41, 0x7c));
    assert_eq!(sea(60), 0x40);
    assert_eq!((sea(61), sea(62), sea(63)), (0x7e, 0x7e, 0x7e));
    assert_eq!((r.count[0x40], r.count[0x7d], r.count[0x7e]), (1, 0, 3));
    assert_eq!(g.cell(0, 0).region, 1);
}

/// `region2` is the second pass's, on HALFLAND cells only, and a value
/// already there survives.
#[test]
fn region2_takes_the_first_ocean_neighbour_and_keeps_a_stale_one() {
    let mut g = sketch(&["#.", "##"]);
    g.cell_mut(0, 0).flags |= HALFLAND;
    g.cell_mut(1, 1).flags |= HALFLAND;
    g.cell_mut(1, 1).region2 = 9;
    let mut r = Regions::default();
    r.clear_all(&mut g);
    r.find_all(&mut g);
    assert_eq!(g.cell(0, 0).region2, 0x41);
    assert_eq!(g.cell(1, 1).region2, 9);
    assert_eq!(g.cell(0, 1).region2, 0);
}

/// Two land cells meeting only across a corner: the first in x-outer
/// order whose flanks are both water is broken.
#[test]
fn fix_diag_land_breaks_a_corner_join() {
    let mut g = sketch(&["#.", ".#"]);
    g.fix_diag_land();
    assert_eq!((g.cell(0, 0).land, g.cell(1, 1).land), (2, 0));
    // The other diagonal: x outer, (0, 1) meets (1, 0) across its
    // (+1, −1) corner first. The (−1, ±1) corners can never fire — a
    // partner there is earlier in the scan with the same two flanks.
    let mut anti = sketch(&[".#", "#."]);
    anti.fix_diag_land();
    assert_eq!((anti.cell(0, 1).land, anti.cell(1, 0).land), (2, 0));
    let mut joined = sketch(&["##", ".#"]);
    joined.fix_diag_land();
    assert!(joined.cells.iter().filter(|c| c.land == 0).count() == 3);
}

/// A one-cell lake is filled; then the coast ring is marked, and a COAST
/// cell with no inland neighbour is thinned to water.
#[test]
fn make_coastlines_fills_lakes_marks_and_thins() {
    let mut g = sketch(&["#####", "#####", "##.##", "#####", "#####"]);
    g.make_coastlines();
    assert!(g.cells.iter().all(|c| c.land == 0 && c.flags == 0));
    // Exactly six dry neighbours fills; five does not.
    let mut six = sketch(&["######", "###.##", "###...", "......", "......"]);
    six.fix_lakes();
    assert_eq!(six.cell(3, 1).land, 0);
    let mut five = sketch(&["######", "###.##", "##....", "......", "......"]);
    five.fix_lakes();
    assert_eq!(five.cell(3, 1).land, 2);
    let mut spit = sketch(&[".....", ".###.", ".....", "....."]);
    spit.make_coastlines();
    // Each of the three cells is COAST; the ends have the middle as their
    // land neighbour but the middle is COAST too, so all three go.
    assert!(spit.cells.iter().all(|c| c.land == 2 && c.flags == 0));
    let mut block = sketch(&["......", ".####.", ".####.", ".####.", "......"]);
    block.make_coastlines();
    assert_eq!(block.cell(1, 1).flags, COAST | 0x400);
    assert_eq!(block.cell(2, 2).flags, 0);
    assert_eq!(block.cell(1, 1).land, 0);
}

/// Four draws, whatever the player count.
#[test]
fn great_sahara_draws_four() {
    for players in [2, 3, 8] {
        let mut g = MapGrid::new(60, 60);
        let mut ours = Rng::new(0xa289_127d);
        great_sahara_make_continents(&mut g, players, &mut ours);
        let mut four = Rng::new(0xa289_127d);
        for _ in 0..4 {
            four.roll();
        }
        assert_eq!(ours.seed, four.seed);
        assert_eq!(g.starts[0].list.len(), players as usize);
        assert_eq!(g.starts[2].list.len(), 4 * players as usize);
    }
}
