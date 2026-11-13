use super::*;
use crate::world::{Cell, Terrain, World};

fn sim() -> Sim {
    let mut w = World::new(40, 40);
    let land = w.add_region(Terrain::Land);
    for y in 0..40 {
        for x in 0..40 {
            w.set_region(Cell::new(x, y), land);
        }
    }
    Sim::new(crate::Tuning::RON, w, 2)
}

/// `Leader::init`'s image is every cell −1, and both leaves are answered
/// at their widths (§19.2).
#[test]
fn a_seated_leader_starts_at_minus_one() {
    let s = sim();
    let b = s.leader_finished_leaf(0, "last_building_finished").unwrap();
    let u = s.leader_finished_leaf(1, "last_unit_finished").unwrap();
    assert_eq!((b.len(), u.len()), (FINISHED_BUILDINGS, FINISHED_UNITS));
    assert!(b.iter().chain(&u).all(|&x| x == -1));
    assert_eq!(s.leader_finished_leaf(0, "score"), None);
}

/// **An overwrite, never an increment** (§19.2): a building writes its
/// own `o` into its owner's cell of its type, a unit the captain's `o`
/// into the trained type's cell; a second of the type takes the cell.
#[test]
fn each_cell_takes_the_last_object_number() {
    let mut s = sim();
    let b = s.add_building(1, crate::Pos::default(), 0);
    s.buildings[b].ty = Some(3);
    s.buildings[b].index = 2007;
    s.note_last_building(b);
    s.buildings[b].index = 2015;
    s.note_last_building(b);
    let v = s.leader_finished_leaf(1, "last_building_finished").unwrap();
    assert_eq!((v[3], v[2]), (2015, -1));
    assert!(s.leader_finished_leaf(0, "last_building_finished").unwrap()[3] == -1);
}
