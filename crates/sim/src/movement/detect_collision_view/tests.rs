use super::*;

/// `move_x`/`move_y`'s first nine entries are the crate's ring-1 spiral, which
/// `docs/COLLISION.md` §2.1 checked against the PE.
#[test]
fn the_ring_is_the_spiral_s_first_nine() {
    assert_eq!(RING1.to_vec(), crate::movement::collide::spiral(1));
}

/// The two table lookups are the crate's floor divisions.
#[test]
fn the_cell_tables_are_floor_divisions() {
    for v in [0, 1, 0x2f, 0x30, 0x2ff, 0x300, 0x1000, 0x2f00] {
        assert_eq!(
            unit_cell_of(v),
            v.div_euclid(crate::movement::collide::UNITS_PER_UCELL)
        );
        assert_eq!(world_cell_of(v), v.div_euclid(crate::world::UNITS_PER_CELL));
    }
}
