use crate::world::{Cell, Pos};
use crate::{Sim, Tuning, Unit, World};

const CELL: i32 = 768;

/// A land of one region (a second, empty one beside it), a builder of
/// player 0 at the middle of cell (8, 8), and the sites `(owner, x, y)`.
fn land(sites: &[(u8, i32, i32)]) -> (Sim, usize, Vec<usize>) {
    let mut world = World::new(16, 16);
    world.fill_region(
        crate::world::Terrain::Land,
        Cell::new(0, 0),
        Cell::new(15, 15),
    );
    world.add_region(crate::world::Terrain::Land);
    let mut s = Sim::new(Tuning::RON, world, 4);
    let u = s.add_unit(Unit::new(
        0,
        1,
        Pos::new(8 * CELL + CELL / 2, 8 * CELL + CELL / 2),
        10,
    ));
    let b = sites
        .iter()
        .map(|&(owner, x, y)| {
            let b = s.add_building(owner, Pos::new(x, y), 1);
            s.buildings[b].active = false;
            b
        })
        .collect();
    (s, u, b)
}

fn at(cx: i32, cy: i32) -> (i32, i32) {
    (cx * CELL + 10, cy * CELL + 10)
}

#[test]
fn a_cell_lists_its_newest_site_first_and_keeps_only_mine_unfinished_and_unattacked() {
    let (x, y) = at(8, 8);
    let (mut s, u, b) = land(&[
        (0, x, y),
        (0, x + 5, y),
        (1, x, y + 5),
        (0, x + 9, y),
        (0, x + 20, y),
    ]);
    s.buildings[b[3]].active = true;
    s.buildings[b[4]].under_attack = 2;
    // Range 0x300 is ring 1: the circle walk.
    assert_eq!(s.find_construct_sites(u, 0x300), vec![b[1], b[0]]);
}

#[test]
fn a_site_in_another_region_is_not_found() {
    let (x, y) = at(9, 8);
    let (mut s, u, b) = land(&[(0, x, y)]);
    s.world.set_region(Cell::new(9, 8), 1);
    assert!(s.find_construct_sites(u, 0x300).is_empty());
    s.world.set_region(Cell::new(9, 8), 0);
    assert_eq!(s.find_construct_sites(u, 0x300), vec![b[0]]);
}

#[test]
fn the_circle_walks_nearer_cells_first_and_the_list_walk_slots_in_order() {
    // Slot order is east four, east one, south four. The ring puts the near
    // one first, then the column x = 0 before x = 4.
    let sites = [at(12, 8), at(9, 8), at(8, 12)].map(|(x, y)| (0, x, y));
    let (s, u, b) = land(&sites);
    // Range 0xc00 is ring 4 (69 points): the circle walk.
    assert_eq!(s.find_construct_sites(u, 0xc00), vec![b[1], b[2], b[0]]);
    // Range 0x1000 is ring 6 (145 points): still the circle, and it reaches
    // the east-four one the list's x-twice distance (4608 against 4096) would not.
    assert_eq!(s.find_construct_sites(u, 0x1000), vec![b[1], b[2], b[0]]);
    // Range 0x1600 is ring 8 (237 points, past `num_def_builds`): the list, by slot.
    assert_eq!(s.find_construct_sites(u, 0x1600), vec![b[0], b[1], b[2]]);
}

#[test]
fn a_site_in_a_cell_without_a_region_is_not_found() {
    let mut world = World::new(16, 16);
    world.fill_region(
        crate::world::Terrain::Land,
        Cell::new(0, 0),
        Cell::new(8, 15),
    );
    world.fill_region(
        crate::world::Terrain::Land,
        Cell::new(10, 0),
        Cell::new(15, 15),
    );
    let mut s = Sim::new(Tuning::RON, world, 4);
    let u = s.add_unit(Unit::new(0, 1, Pos::new(8 * CELL + 10, 8 * CELL + 10), 10));
    let (x, y) = at(9, 8);
    let b = s.add_building(0, Pos::new(x, y), 1);
    s.buildings[b].active = false;
    assert!(s.find_construct_sites(u, 0x300).is_empty());
}

#[test]
fn the_list_walk_measures_the_x_offset_twice() {
    // Range 0x2400 is ring 12: the list walk. 3000 units straight north is
    // distance 0 to the original (it never reads `+0x14`); 7000 east is 10500.
    let here = 8 * CELL + CELL / 2;
    let (s, u, b) = land(&[(0, here, here - 3 * CELL), (0, here + 7000, here)]);
    assert_eq!(s.find_construct_sites(u, 0x2400), vec![b[0]]);
    let (s, u, b) = land(&[
        (0, here + 6000, here),
        (0, here + 2 * CELL, here - 4 * CELL),
    ]);
    assert_eq!(s.find_construct_sites(u, 0x2400), vec![b[0], b[1]]);
}

#[test]
fn the_list_walk_takes_the_footprint_off_both_operands() {
    let here = 8 * CELL + CELL / 2;
    let (mut s, u, b) = land(&[(0, here + 6700, here)]);
    assert!(s.find_construct_sites(u, 0x2400).is_empty());
    // x_size 4 takes 768 off each operand: 5932, distance 8898 against 9216 (10050 without).
    let t = s.add_build_type(crate::cities::build::BuildType {
        x_size: 4,
        y_size: 4,
        ..crate::cities::build::BuildType::default()
    });
    s.buildings[b[0]].ty = Some(t);
    assert_eq!(s.find_construct_sites(u, 0x2400), vec![b[0]]);
    // One operand alone does not do it: (5932, 6700) is 9326.
    s.tables.build_types[t].y_size = 0;
    assert!(s.find_construct_sites(u, 0x2400).is_empty());
    s.tables.build_types[t].y_size = 4;
    s.tables.build_types[t].x_size = 0;
    assert!(s.find_construct_sites(u, 0x2400).is_empty());
}

#[test]
fn the_list_walk_reaches_the_owners_last_site() {
    let here = 8 * CELL + CELL / 2;
    let (s, u, b) = land(&[(0, here, here), (0, here + 100, here), (1, here, here)]);
    assert_eq!(s.find_construct_sites(u, 0x2400), vec![b[0], b[1]]);
}
