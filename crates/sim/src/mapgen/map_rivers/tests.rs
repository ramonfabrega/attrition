use super::*;

fn forest(xs: i32, ys: i32) -> MapGrid {
    let mut g = MapGrid::new(xs, ys);
    for c in &mut g.cells {
        c.flags = 0x20;
        c.region = 1;
    }
    g
}

fn flags(g: &mut MapGrid, x: i32, y: i32) -> &mut u16 {
    let i = at(g, x, y);
    &mut g.cells[i].flags
}

/// A 7×7 forest has one cell whose square is on the map, its centre.
#[test]
fn a_source_s_square_must_be_on_the_map() {
    let mut g = forest(7, 7);
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
    assert_eq!(
        g.cells
            .iter()
            .filter(|c| c.flags & bit::SOURCE != 0)
            .count(),
        1
    );
}

/// A source blocks every later cell whose 7×7 holds it, so sources
/// stand four apart along a row and none share a square.
#[test]
fn a_source_blocks_its_square() {
    let mut g = forest(14, 7);
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3), (7, 3)]);
}

/// The threshold counts the eight neighbours without a mountain tile
/// or rock: one rock neighbour takes 8 to 7.
#[test]
fn the_threshold_counts_the_eight() {
    let mut g = forest(7, 7);
    *flags(&mut g, 4, 4) |= 0x8;
    assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![]);
    assert_eq!(make_riverbeds(&mut g, 7, 20), vec![(3, 3)]);
    let mut g = forest(7, 7);
    // A tile of cell (2, 2), a neighbour: row 8, column 9.
    g.tiles[8 * 28 + 9] = 2;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
}

/// The ring past the eight only refuses: a rock there is no matter,
/// an ocean cell there is.
#[test]
fn the_outer_rings_only_refuse() {
    let mut g = forest(7, 7);
    *flags(&mut g, 0, 0) = 0x8;
    assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![(3, 3)]);
    *flags(&mut g, 0, 0) = 0;
    g.cells[0].land = 2;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
}

/// Land class 2 is plain water: an oil-flagged sea cell is 7, a
/// coastal one 3, a `SANDY` cell 1, so none of them refuse.
#[test]
fn only_class_2_refuses() {
    for (land, f, refuses) in [
        (2, 0, true),
        (2, 0x800, false),
        (2, 0x4, false),
        (2, 0x20, false),
        (2, 0x900, true),
        (1, 0, false),
    ] {
        let mut g = forest(7, 7);
        g.cells[0].land = land;
        g.cells[0].flags = f;
        assert_eq!(
            make_riverbeds(&mut g, 8, 20).is_empty(),
            refuses,
            "{land} {f:#x}"
        );
    }
}

/// Bit 0x1 refuses the cell itself, and the cell needs the forest bit.
#[test]
fn a_source_is_forest_without_0x1() {
    let mut g = forest(7, 7);
    *flags(&mut g, 3, 3) |= 0x1;
    assert_eq!(make_riverbeds(&mut g.clone(), 8, 20), vec![]);
    *flags(&mut g, 3, 3) = 0;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
}

/// A refusal in the square zeroes the count rather than skipping the
/// cell, so a threshold of 0 makes every eligible forest cell a source,
/// edge or not.
#[test]
fn a_zero_threshold_takes_every_forest_cell() {
    let mut g = forest(3, 3);
    assert_eq!(make_riverbeds(&mut g, 0, 20).len(), 9);
}

/// A mountain tile is `mask & 3 == 2`: a mask of 3 is not one.
#[test]
fn a_mask_of_3_is_no_mountain() {
    let mut g = forest(7, 7);
    g.tiles[8 * 28 + 9] = 3;
    let t = (4 * 3 * 4 * 7 + 4 * 3) as usize;
    g.tiles[t] = 0x7;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
}

/// `find_all` runs after 5116: an unlabelled cell is labelled by 5118.
#[test]
fn find_all_runs_at_5118() {
    let mut g = forest(7, 7);
    g.cells[0].region = 0;
    let mut regions = Regions::default();
    regions.count[1] = 48;
    let mut at_5116 = 0;
    make_rivers(
        &mut g,
        &mut regions,
        &phase(&[]),
        &mut Draws::new(1),
        |l, g, _| {
            if l == 5116 {
                at_5116 = g.cells[0].region;
            }
        },
    );
    assert_eq!((at_5116, g.cells[0].region), (0, 1));
}

/// A cell with a mountain tile of its own is never a source.
#[test]
fn a_mountain_tile_refuses_the_cell() {
    let mut g = forest(7, 7);
    let t = (4 * 3 * 4 * 7 + 4 * 3 + 3) as usize;
    g.tiles[t] = 0x12;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![]);
}

/// A start city at distance d refuses the cell when 4·d < radius − 1:
/// at radius 20, distance 4 refuses and 5 does not.
#[test]
fn a_start_city_s_radius_refuses() {
    for (cx, refused) in [(7, true), (8, false)] {
        let mut g = forest(15, 7);
        g.starts[2].list.push(cx);
        g.starts[3].list.push(3);
        let beds = make_riverbeds(&mut g, 8, 20);
        assert_eq!(beds.first() == Some(&(3, 3)), !refused, "city at {cx}");
    }
    // `− 1`: at radius 21, distance 5 (4·5 = 20) is outside.
    let mut g = forest(15, 7);
    g.starts[2].list.push(8);
    g.starts[3].list.push(3);
    assert_eq!(make_riverbeds(&mut g, 8, 21).first(), Some(&(3, 3)));
    assert_eq!(octagonal(4, 0), 4);
    assert_eq!(octagonal(3, 4), 3 * 3 / 8 + 4);
    assert_eq!(octagonal(4, 3), 3 * 3 / 8 + 4);
}

/// `make_riverlist`'s nearest start is a nation's `start_x`/`start_y`
/// entry, not a footprint cell (item 3944): two cities twenty cells
/// apart, and a river end beside the second is the second's.
#[test]
fn the_nearest_start_is_a_nation_s_start_not_a_footprint_cell() {
    let mut g = forest(30, 10);
    g.add_starting_location(5, 5);
    g.add_starting_location(25, 5);
    assert_eq!(nearest_start(&g, 24, 5, 2), Some(1));
    assert_eq!(nearest_start(&g, 6, 5, 2), Some(0));
    assert_eq!(
        nearest_start(&g, 24, 5, 1),
        Some(0),
        "one nation, one start"
    );
}

/// The first clear takes 0x2000 and 0x2 and nothing else, before the
/// scan, so an old source blocks nothing.
#[test]
fn the_first_clear_runs_before_the_scan() {
    let mut g = forest(7, 7);
    *flags(&mut g, 0, 0) |= bit::SOURCE | bit::BANK | bit::PATH;
    assert_eq!(make_riverbeds(&mut g, 8, 20), vec![(3, 3)]);
    assert_eq!(g.cells[0].flags, 0x20 | bit::PATH);
}

fn phase(rivers: &[River]) -> Phase<'_> {
    Phase {
        thresh: 8,
        city_radius: 20,
        rivers,
        standard: 70,
        seed: 0,
        nations: 0,
    }
}

/// The whole phase on a forest with sources and no `RIVER`: the five
/// checkpoints, the paths cleared at 5014's tail, and the sources gone
/// again by 5116.
#[test]
fn the_phase_takes_its_sources_back() {
    let mut g = forest(7, 7);
    *flags(&mut g, 0, 0) |= bit::PATH;
    let before = g.cell_part();
    let mut seen = Vec::new();
    let mut d = Draws::new(1);
    let beds = make_rivers(
        &mut g,
        &mut Regions::default(),
        &phase(&[]),
        &mut d,
        |l, g, _| seen.push((l, g.cell_part())),
    );
    assert_eq!(beds, vec![(3, 3)]);
    let lines: Vec<i64> = seen.iter().map(|s| s.0).collect();
    assert_eq!(lines, [5012, 5014, 5076, 5116, 5118]);
    assert_eq!(seen[0].1, before);
    assert_ne!(seen[1].1, before);
    assert!(g.cells.iter().all(|c| c.flags == 0x20));
    assert_eq!(seen[3].1, seen[4].1);
    assert!(d.log.is_empty());
}

/// No source: the paths are still cleared, and nothing past 5014 is
/// printed or drawn.
#[test]
fn no_source_ends_at_5014() {
    let mut g = forest(5, 5);
    *flags(&mut g, 2, 2) |= bit::PATH;
    let mut lines = Vec::new();
    let mut d = Draws::new(1);
    let beds = make_rivers(
        &mut g,
        &mut Regions::default(),
        &phase(&[GREAT_LAKES]),
        &mut d,
        |l, _, _| lines.push(l),
    );
    assert_eq!(beds, vec![]);
    assert_eq!(lines, [5012, 5014]);
    assert_eq!(*flags(&mut g, 2, 2), 0x20);
    assert!(d.log.is_empty());
}

/// `GREATLAKES.xml`'s one `RIVER`.
const GREAT_LAKES: River = River {
    start_range: 1,
    end_range: 2,
    river_length: 12,
    river_min_length: 6,
    river_valley: 8,
    river_wide: 0,
    river_med: 100,
    river_twist: 60,
    river_flags: 11,
};

/// A land strip with water along its south edge: `(x, y)` is land for
/// `y < water`, ocean from there.
fn shore(xs: i32, ys: i32, water: i32) -> MapGrid {
    let mut g = MapGrid::new(xs, ys);
    for y in 0..ys {
        for x in 0..xs {
            let i = at(&g, x, y);
            let c = &mut g.cells[i];
            c.flags = 0;
            c.land = if y < water { 0 } else { 2 };
            c.region = if y < water { 1 } else { 0x40 };
        }
    }
    g
}

/// The search walks south to the nearer water, one draw a dry cell it prices,
/// and returns the path source first.
#[test]
fn a_river_runs_to_the_water() {
    let g = shore(21, 20, 12);
    let mut d = Draws::new(7);
    let path = astar_river(&g, (10, 8), 60, 0, 0, &mut d);
    assert_eq!(path.first(), Some(&(10, 8)));
    let last = *path.last().expect("a path");
    assert!(wet(&g.cells[at(&g, last.0, last.1)]), "{path:?}");
    for w in path.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert_eq!((a.0 - b.0).abs() + (a.1 - b.1).abs(), 1, "{path:?}");
    }
    assert!(!d.log.is_empty());
    assert!(d.log.iter().all(|&(s, _)| s == site::PRICE));
}

/// A source next to the water is refused before the search, with no
/// draw.
#[test]
fn a_source_on_the_shore_is_refused() {
    let g = shore(9, 12, 8);
    let mut d = Draws::new(7);
    assert!(astar_river(&g, (4, 7), 60, 0, 0, &mut d).is_empty());
    assert!(d.log.is_empty());
}

/// Under flag 8 a path no longer than `max_len` is not returned.
#[test]
fn a_short_river_is_not_returned() {
    let g = shore(9, 12, 8);
    let mut d = Draws::new(7);
    assert!(astar_river(&g, (4, 4), 60, 10, 0, &mut d).is_empty());
}

/// A kept river marks its path 0x1000, and its valley's disc 0x2 only in
/// the region of the path cell it is drawn round.
#[test]
fn a_river_marks_its_path_and_banks() {
    let mut g = shore(21, 20, 12);
    let beds = [(10, 8)];
    let r = River {
        river_flags: 1,
        ..GREAT_LAKES
    };
    let mut d = Draws::new(3);
    make_riverlist(&mut g, &beds, &r, (0, 0, 2), &phase(&[]), &mut d);
    assert_eq!(d.log[0].0, site::SPREAD);
    let path: Vec<usize> = (0..g.cells.len())
        .filter(|&i| g.cells[i].flags & bit::PATH != 0)
        .collect();
    assert!(!path.is_empty());
    assert!(g.cells[at(&g, 10, 8)].flags & bit::PATH != 0);
    let regions: BTreeSet<i16> = path.iter().map(|&i| g.cells[i].region).collect();
    assert!(
        g.cells
            .iter()
            .all(|c| c.flags & bit::BANK == 0 || regions.contains(&c.region))
    );
    assert!(g.cells.iter().any(|c| c.flags & bit::BANK != 0));
}
