use super::*;
use crate::mapgen::adler_rewrite;

fn grid(xs: i32, ys: i32) -> MapGrid {
    MapGrid::new(xs, ys)
}

fn cell(g: &mut MapGrid, x: i32, y: i32) -> &mut crate::mapgen::WCell {
    let i = at(g, x, y);
    &mut g.cells[i]
}

#[test]
fn a_style_without_sea_map_leaves_the_world_s() {
    assert_eq!(sea_map_after_make(-1, -1), -1);
    assert_eq!(sea_map_after_make(-1, 0), 0);
    assert_eq!(sea_map_after_make(-1, 3), 3);
}

/// The rewrite is the sum of the patched run, at every offset.
#[test]
fn the_rewrite_is_the_patched_sum() {
    let mut run: Vec<u8> = (0..PART4_LEN as u32)
        .map(|i| (i * 37 % 251) as u8)
        .collect();
    run[PART4_SEA_MAP_AT..PART4_SEA_MAP_AT + 4].copy_from_slice(&(-1i32).to_le_bytes());
    let before = adler32(&run);
    run[PART4_SEA_MAP_AT..PART4_SEA_MAP_AT + 4].copy_from_slice(&7i32.to_le_bytes());
    assert_eq!(
        adler_rewrite(
            before,
            PART4_LEN,
            PART4_SEA_MAP_AT,
            &(-1i32).to_le_bytes(),
            &7i32.to_le_bytes()
        ),
        adler32(&run)
    );
}

/// An `OCEAN` cell beside a 0x4 cell is shallow; one two cells off is
/// still shallow (5×5) and three is not; a `BASELAND` cell is coastal
/// only within the ring of eight.
#[test]
fn the_flags_read_their_rings() {
    let mut g = grid(8, 8);
    cell(&mut g, 4, 4).flags = 0x4;
    for &(x, y) in &[(5, 4), (6, 4), (7, 4)] {
        cell(&mut g, x, y).land = 2;
    }
    let f = coord_flags(&g);
    assert_eq!(f[at(&g, 4, 4)], coord::NONE_CELL | coord::GATE_HALF);
    assert_eq!(f[at(&g, 5, 4)], coord::OCEAN | coord::SHALLOW);
    assert_eq!(f[at(&g, 6, 4)], coord::OCEAN | coord::SHALLOW);
    assert_eq!(f[at(&g, 7, 4)], coord::OCEAN);
    assert_eq!(f[at(&g, 3, 3)], coord::COASTAL);
    assert_eq!(f[at(&g, 2, 4)], 0);
    assert_eq!(halfland_entered(&g, &f).len(), 3);
}

/// A HALFLAND or 0x400 cell is gated; a plain one is not.
#[test]
fn halfland_and_0x400_open_the_gate() {
    let mut g = grid(4, 4);
    cell(&mut g, 1, 1).flags = 0x100;
    cell(&mut g, 2, 2).flags = 0x400;
    let f = coord_flags(&g);
    assert_eq!(halfland_entered(&g, &f), vec![(1, 1), (2, 2)]);
    assert!(matches!(
        check_and_set_halfland(&mut g, &mut f.clone()),
        Err(Unported::HalfLand(_))
    ));
}

/// An all-`BASELAND` grid passes the phase untouched.
#[test]
fn a_dry_grid_is_untouched() {
    let mut g = grid(6, 6);
    g.tiles[3] = 0x38;
    let before = (g.cell_part(), g.tile_part());
    let (sea, f, half) = terrain_init_without_heights(&mut g, 0).expect("nothing unported");
    assert_eq!(sea, 0);
    assert!(f.iter().all(|&x| x == 0));
    assert_eq!((g.cell_part(), g.tile_part()), before);
    assert_eq!(half.parts(), Some([262_145; 4]));
}

/// A `SANDY` cell's tiles turn wet, and the wet tiles within the 7×7
/// of the first dry neighbour gain 0x400.
#[test]
fn a_sandy_cell_marks_its_tiles_and_its_coast() {
    let mut g = grid(4, 4);
    cell(&mut g, 1, 1).land = 1;
    let t = tile_at(&g, 4, 4);
    g.tiles[t] = 0x10;
    let mut f = coord_flags(&g);
    check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
    for ty in 4..8 {
        for tx in 4..8 {
            let m = g.tiles[tile_at(&g, tx, ty)];
            assert_eq!(m & 0x30, 0x20, "tile ({tx}, {ty})");
            // Every tile of the cell is within 3 of the dry (3, 3)
            // or of a dry tile its own scan found first.
            assert_ne!(m & 0x400, 0, "tile ({tx}, {ty})");
        }
    }
    assert_eq!(g.tiles[tile_at(&g, 3, 3)], 0);
}

/// The square is 7×7: a wet tile three columns from the dry one is
/// marked, four is not.
#[test]
fn the_coast_reaches_three_tiles() {
    let mut g = grid(4, 1);
    for x in 1..4 {
        cell(&mut g, x, 0).land = 1;
    }
    let mut f = coord_flags(&g);
    check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
    assert_ne!(g.tiles[tile_at(&g, 6, 0)] & 0x400, 0);
    assert_eq!(g.tiles[tile_at(&g, 7, 0)] & 0x400, 0);
}

/// A tile with no dry tile among its neighbours marks nothing.
#[test]
fn a_wet_tile_without_a_dry_neighbour_marks_nothing() {
    let mut g = grid(3, 3);
    for c in g.cells.iter_mut() {
        c.land = 1;
    }
    let mut f = coord_flags(&g);
    check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
    assert!(g.tiles.iter().all(|&m| m == 0x20));
}

/// The tail: an `OCEAN` cell with HALFLAND west of it turns shallow.
/// [`coord_flags`] would gate the HALFLAND cell, so the flags are
/// handed in clear to reach the tail alone.
#[test]
fn an_ocean_cell_beside_halfland_turns_shallow() {
    let mut g = grid(4, 3);
    cell(&mut g, 1, 1).flags = 0x100;
    cell(&mut g, 2, 1).land = 2;
    cell(&mut g, 3, 1).land = 2;
    let mut f = vec![0u16; g.cells.len()];
    check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
    assert_eq!(f[at(&g, 2, 1)], coord::SHALLOW);
    assert_eq!(f[at(&g, 3, 1)], 0);
}

/// `orthog[0]` is the cell itself: a HALFLAND `OCEAN` cell with no
/// HALFLAND neighbour is shallow.
#[test]
fn a_halfland_ocean_cell_reads_itself() {
    let mut g = grid(3, 3);
    cell(&mut g, 1, 1).land = 2;
    cell(&mut g, 1, 1).flags = 0x100;
    let mut f = vec![0u16; g.cells.len()];
    check_and_set_halfland(&mut g, &mut f).expect("nothing unported");
    assert_eq!(f[at(&g, 1, 1)], coord::SHALLOW);
    assert_eq!(f.iter().filter(|&&x| x != 0).count(), 1);
}

/// The relabel: a `BASELAND` cell in a water region with a `region2`
/// swaps the two.
#[test]
fn the_relabel_swaps_region_and_region2() {
    let mut g = grid(1, 1);
    cell(&mut g, 0, 0).region = 0x41;
    cell(&mut g, 0, 0).region2 = 3;
    relabel(&mut g);
    let c = *cell(&mut g, 0, 0);
    assert_eq!((c.region, c.region2), (3, 0x41));
}

/// Every `BASELAND` cell touching the `OCEAN` one, row by row.
#[test]
fn a_beach_cell_is_refused_not_guessed() {
    let mut g = grid(3, 3);
    cell(&mut g, 2, 1).land = 2;
    assert_eq!(
        convert_beach_to_sandy(&g),
        Err(Unported::Beach(vec![
            (1, 0),
            (2, 0),
            (1, 1),
            (1, 2),
            (2, 2)
        ]))
    );
}
