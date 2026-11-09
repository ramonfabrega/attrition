use super::*;

/// **`largest_gather` is 1 for every good on the shipped table, never
/// 0** (`GoodType::compute_largest_gather@0066e920`, item 1451): plain
/// land makes a knowledge and a food at 1, forest a timber, mountains a
/// metal, oil an oil, and wealth — made by no land — is clamped up to
/// one. Read as 0 until item 1451, which put `max(3, ter)` into every
/// gather building's value where the original has `max(2, ter)`.
/// Made to fail by dropping the clamp's floor (wealth reads 0).
#[test]
fn largest_gather_is_one_for_every_good() {
    assert_eq!((0..6).map(largest_gather).collect::<Vec<_>>(), [1; 6]);
}

#[test]
fn cached_region_sizes_match_membership_after_reassignment() {
    let mut world = World::new(5, 2);
    let land = world.add_region(Terrain::Land);
    let sea = world.add_region(Terrain::Sea);
    let other = world.add_region(Terrain::Land);
    for (cell, region) in [
        (Cell::new(0, 0), land),
        (Cell::new(1, 0), land),
        (Cell::new(2, 0), sea),
        (Cell::new(3, 0), other),
    ] {
        world.set_region(cell, region);
    }
    world.set_region(Cell::new(0, 0), sea);
    world.set_region(Cell::new(0, 0), sea);
    world.set_region(Cell::new(4, 1), other);

    for region in [land, sea, other, u16::MAX] {
        let scanned = world
            .region
            .iter()
            .filter(|member| **member == Some(region))
            .count() as i32;
        assert_eq!(world.region_size(region), scanned, "region {region}");
    }
}

#[test]
fn cell_conversion_matches_the_originals_two_step_form() {
    // The original does `div_3_table[pos >> 8]` for a cell and
    // `div_3_table[pos >> 6]` for a tile, where the table is floor(i / 3).
    // Nesting two floor divisions is one floor division by the product,
    // which is what Pos::cell does directly. Check it over a range that
    // straddles zero, since that is the only place the two could differ.
    for p in -4000..4000i32 {
        let cell_two_step = floor_div(p >> 8, 3);
        let tile_two_step = floor_div(p >> 6, 3);
        let pos = Pos::new(p, p);
        assert_eq!(pos.cell().x, cell_two_step, "cell of {p}");
        assert_eq!(pos.tile().x, tile_two_step, "tile of {p}");
    }
}

#[test]
fn the_three_units_relate_as_claimed() {
    assert_eq!(UNITS_PER_CELL, 768);
    assert_eq!(Pos::new(768, 1535).cell(), Cell::new(1, 1));
    assert_eq!(Pos::new(767, 0).cell(), Cell::new(0, 0));
    assert_eq!(Pos::new(-1, -768).cell(), Cell::new(-1, -1));
    assert_eq!(Pos::new(192 * 5, 0).tile().x, 5);
}

/// **`coast_here`'s neighbour read is `get_tregion`, not the cell's
/// plain region** (`0068106a`, 2026-09-01, `docs/TRANSPORT.md` §9.3).
/// A coastal cell — `flags & 0x100`, `region2` the sea — is a *land*
/// cell in `WData.region`, and it is exactly the cell a boat's half of
/// the shore lies in. Asked with the plain region the function cannot
/// see it, and `think_civilian_transport` then never picks the shore
/// cell the original picks (`docs/SYNC.md` §3.25).
#[test]
fn coast_here_sees_a_shore_whose_water_half_is_a_land_cell() {
    let mut w = World::new(8, 8);
    let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(6, 7));
    let sea = w.fill_region(Terrain::Sea, Cell::new(7, 0), Cell::new(7, 7));
    w.rebuild_coasts();
    assert!(w.is_coast(land, sea), "the two touch at x = 6/7");

    // (3, 3) is inland: every neighbour is a land cell by `WData.region`.
    assert_eq!(w.coast_here(land, sea, Cell::new(3, 3)), 0);

    // Make its eastern neighbour the half-water shore — still a land
    // cell in `region`, but `HALFLAND` with `region2` the sea and an
    // ocean tile at its centre, which is the tile `get_tregion` reads.
    let shore = Cell::new(4, 3);
    let mut d = w.cell_data(shore);
    d.flags |= 0x100;
    d.region2 = Some(sea);
    w.set_cell_data(shore, d);
    w.set_tile_field(shore.centre_tile(), tile::SURFACE, tile::SURFACE_OCEAN);
    assert_eq!(w.region_of(shore), Some(land), "still a land cell");
    assert_eq!(w.tregion(shore.centre_tile()), Some(land), "the plain read");
    assert_eq!(w.tregion_alt(shore.centre_tile()), Some(sea), "get_tregion");

    // MOVE_8[3] is (1, 0), so the answer is that neighbour's 1-based
    // index — and with the plain read it would still be 0.
    assert_eq!(w.coast_here(land, sea, Cell::new(3, 3)), 4);
}

#[test]
fn cell_centres_land_on_the_half_tile() {
    assert_eq!(Cell::new(0, 0).centre_tile(), Pos::new(2, 2));
    assert_eq!(Cell::new(3, 7).centre_tile(), Pos::new(14, 30));
}

#[test]
fn the_hypotenuse_is_exact_on_the_axes() {
    for n in 0..200 {
        assert_eq!(vector_dist(n, 0), n);
        assert_eq!(vector_dist(0, -n), n);
    }
}

#[test]
fn the_hypotenuse_overshoots_on_the_diagonal() {
    // 3-4-5 comes out exact by luck: 4 + 9/8 = 5.
    assert_eq!(vector_dist(3, 4), 5);
    // The pure diagonal is where the approximation is worst: it returns
    // 1.5x the leg where the true answer is 1.414x, so borders bulge
    // *inward* at the corners and the outline reads as an octagon.
    assert_eq!(vector_dist(40, 40), 60);
    assert_eq!(vector_dist(100, 100), 150);
}

#[test]
fn the_hypotenuse_is_symmetric_and_off_by_at_most_a_truncation() {
    for dx in -60..60 {
        for dy in -60..60 {
            let d = vector_dist(dx, dy);
            assert_eq!(d, vector_dist(dy, dx), "{dx},{dy}");
            assert_eq!(d, vector_dist(-dx, dy), "{dx},{dy}");
            // Never shorter than the longer leg.
            assert!(d >= dx.abs().max(dy.abs()), "{dx},{dy} gave {d}");
            // The exact first-order form is an upper bound on the true
            // distance, but the division truncates, so the result can land
            // one below — 60,30 is the first case, giving 67 where the
            // true distance is 67.08. It is never worse than that.
            let (d, true_sq) = (
                i64::from(d),
                i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy),
            );
            assert!((d + 1) * (d + 1) >= true_sq, "{dx},{dy} gave {d}");
        }
    }
}

#[test]
fn ambiguous_is_claimed_but_owned_by_nobody() {
    assert!(Owner::Ambiguous.is_claimed());
    assert_eq!(Owner::Ambiguous.player(), None);
    assert!(!Owner::None.is_claimed());
    assert_eq!(Owner::Player(3).player(), Some(3));
}

#[test]
fn regions_partition_the_grid() {
    let mut w = World::new(4, 4);
    let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(1, 3));
    let sea = w.fill_region(Terrain::Sea, Cell::new(2, 0), Cell::new(3, 3));
    assert_eq!(w.cells_in(land).count(), 8);
    assert_eq!(w.cells_in(sea).count(), 8);
    assert_eq!(w.terrain(land), Terrain::Land);
    assert_eq!(w.region_of(Cell::new(3, 3)), Some(sea));
}

/// `Region.coords` is `Regions::rebuild_coords@0067f800`'s order and
/// that is the grid's own: y outer, x inner. A column-major list would
/// hold the same eight cells and hand a strided walk a different four,
/// which is what `docs/SCOUT.md` §11 rides on.
#[test]
fn a_region_s_coords_are_row_major() {
    let mut w = World::new(4, 4);
    let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(1, 3));
    w.fill_region(Terrain::Sea, Cell::new(2, 0), Cell::new(3, 3));
    assert_eq!(
        w.region_coords_strided(land, 0, 1),
        vec![
            Cell::new(0, 0),
            Cell::new(1, 0),
            Cell::new(0, 1),
            Cell::new(1, 1),
            Cell::new(0, 2),
            Cell::new(1, 2),
            Cell::new(0, 3),
            Cell::new(1, 3),
        ]
    );
    // And the stride is over that list, not over the grid.
    assert_eq!(
        w.region_coords_strided(land, 1, 3),
        vec![Cell::new(1, 0), Cell::new(0, 2), Cell::new(1, 3)]
    );
}

/// A world of plain land with one forest cell, one mountain and one
/// offshore oil patch — enough to exercise every arm of
/// [`World::land_class`].
fn lands_world() -> World {
    let mut w = World::new(6, 6);
    let land = w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(5, 5));
    assert_eq!(w.terrain(land), Terrain::Land);
    let set = |w: &mut World, c: Cell, flags: u16, l: i8| {
        let mut d = w.cell_data(c);
        d.flags = flags;
        d.land = l;
        w.set_cell_data(c, d);
    };
    // The whole grid is BASELAND, land class 0.
    for y in 0..6 {
        for x in 0..6 {
            set(&mut w, Cell::new(x, y), 0, 0);
        }
    }
    set(&mut w, Cell::new(2, 2), cell::FOREST, 0);
    set(&mut w, Cell::new(3, 2), cell::MOUNTAIN, 0);
    set(&mut w, Cell::new(4, 2), cell::ROCK | cell::OIL, 0);
    // An ocean cell carrying oil: not rock, so the `is_ocean` arm.
    set(&mut w, Cell::new(0, 5), cell::OIL, 2);
    w
}

/// `WorldData::get_land@006b4730`'s five flag tests, in order.
#[test]
fn a_cell_s_land_class_is_its_flags_before_its_stored_kind() {
    let w = lands_world();
    assert_eq!(w.land_class(Cell::new(1, 1)), 0, "BASELAND");
    assert_eq!(w.land_class(Cell::new(2, 2)), 4, "Forest");
    assert_eq!(w.land_class(Cell::new(3, 2)), 5, "Mountains");
    assert_eq!(w.land_class(Cell::new(4, 2)), 7, "Rocks with oil");
    assert_eq!(w.land_class(Cell::new(0, 5)), 7, "ocean with oil");
}

/// Step 13's caller — `centre_only`. A flat good comes off the cell at
/// its face amount; a non-flat one is doubled, and only if nothing
/// already gathers the cell's centre tile.
#[test]
fn the_centre_only_gather_doubles_the_goods_the_ground_holds() {
    use crate::economy::Resource::{Food, Knowledge, Metal, Oil, Timber};
    let mut w = lands_world();
    let mut want = [0; crate::economy::RESOURCES];
    want[Knowledge as usize] = 1;
    want[Food as usize] = 1;
    assert_eq!(w.gather_at(Cell::new(1, 1), true), want, "plain land");

    let mut want = [0; crate::economy::RESOURCES];
    want[Timber as usize] = 2;
    assert_eq!(w.gather_at(Cell::new(2, 2), true), want, "forest");
    let mut want = [0; crate::economy::RESOURCES];
    want[Metal as usize] = 2;
    assert_eq!(w.gather_at(Cell::new(3, 2), true), want, "mountains");
    let mut want = [0; crate::economy::RESOURCES];
    want[Oil as usize] = 2;
    assert_eq!(w.gather_at(Cell::new(4, 2), true), want, "an oil cell");

    // A tile some building already gathers is worth nothing — and only
    // to the non-flat half, which is the whole of a forest cell's
    // value and none of plain land's.
    w.set_tile_bits(Cell::new(2, 2).centre_tile(), tile::GATHERED_FROM);
    w.set_tile_bits(Cell::new(1, 1).centre_tile(), tile::GATHERED_FROM);
    assert_eq!(w.gather_at(Cell::new(2, 2), true), [0; 6]);
    let mut want = [0; crate::economy::RESOURCES];
    want[Knowledge as usize] = 1;
    want[Food as usize] = 1;
    assert_eq!(w.gather_at(Cell::new(1, 1), true), want);
}

/// `produce_building`'s caller — the neighbourhood sum. The flat goods
/// still come from the centre alone; a non-flat good is summed over
/// the nine cells and doubled once at the end.
#[test]
fn the_full_gather_sums_the_nine_cells_and_doubles_once() {
    use crate::economy::Resource::{Food, Knowledge, Metal, Timber};
    let mut w = lands_world();
    // (3, 2) is mountains; its ring holds the forest at (2, 2) and the
    // oiled rock at (4, 2), neither of which makes metal.
    let got = w.gather_at(Cell::new(3, 2), false);
    assert_eq!(got[Timber as usize], 2, "one forest neighbour, doubled");
    assert_eq!(got[Metal as usize], 2, "the centre's own mountain");
    assert_eq!(got[Knowledge as usize], 0, "mountains make no knowledge");

    // Standing on plain land beside the forest: the flat pair comes
    // from the centre, the timber from the neighbour.
    let got = w.gather_at(Cell::new(1, 1), false);
    assert_eq!(got[Knowledge as usize], 1);
    assert_eq!(got[Food as usize], 1);
    assert_eq!(got[Timber as usize], 2);

    // And a gathered-from neighbour drops out of the sum.
    w.set_tile_bits(Cell::new(2, 2).centre_tile(), tile::GATHERED_FROM);
    assert_eq!(w.gather_at(Cell::new(1, 1), false)[Timber as usize], 0);
}

/// The nine are the centre and [`MOVE_8`], so a cell two away never
/// counts and the map's edge is not walked off.
#[test]
fn the_full_gather_reaches_one_cell_and_stays_on_the_map() {
    use crate::economy::Resource::Timber;
    let w = lands_world();
    assert_eq!(w.gather_at(Cell::new(0, 0), false)[Timber as usize], 0);
    assert_eq!(w.gather_at(Cell::new(1, 1), false)[Timber as usize], 2);
    // (5, 5) is the far corner: three of its nine are off the map.
    assert_eq!(w.gather_at(Cell::new(5, 5), false)[Timber as usize], 0);
}
