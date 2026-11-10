use crate::world::World;
use crate::{Cell, Pos, Sim, Tuning};

/// `place_unit`'s cell test refuses each predicate alone, and admits a
/// plain cell of the anchor's region.
#[test]
fn place_unit_s_cell_test_refuses_each_predicate() {
    let mut s = Sim::new(Tuning::RON, World::new(8, 8), 2);
    let anchor = Cell::new(3, 3);
    let c = Cell::new(4, 4);
    // `World::new`'s cells default `down` to 0; a map's empty cell is
    // `-1`, as the map maker and a dump leave it.
    let mut plain = s.world.cell_data(c);
    plain.down = -1;
    s.world.set_cell_data(c, plain);
    assert!(s.place_unit_admits(anchor, c), "a plain cell");
    assert!(!s.place_unit_admits(anchor, Cell::new(8, 4)), "off the map");
    for (flags, land, down) in [
        (0x10, 0, -1),
        (0x20, 0, -1),
        (0x100, 0, -1),
        (0x8000, 0, -1),
        (0, 1, -1),
        (0, 2, -1),
        (0, 0, 0),
    ] {
        let mut d = plain;
        d.flags = flags;
        d.land = land;
        d.down = down;
        s.world.set_cell_data(c, d);
        assert!(
            !s.place_unit_admits(anchor, c),
            "flags {flags:#x} land {land} down {down}"
        );
    }
    // A good's `-2` and a goody's `-3` are no object.
    for down in [-2, -3] {
        let mut d = plain;
        d.down = down;
        s.world.set_cell_data(c, d);
        assert!(s.place_unit_admits(anchor, c), "down {down}");
    }
    s.world.set_cell_data(c, plain);
    for mask in [0x4000u16, 3] {
        s.world.set_tile_mask(Pos::new(18, 18), mask);
        assert!(!s.place_unit_admits(anchor, c), "mask {mask:#x}");
    }
    s.world.set_tile_mask(Pos::new(18, 18), 1);
    assert!(s.place_unit_admits(anchor, c), "mask 1");
    s.add_building(0, Pos::new(4 * 0x300 + 0x180, 4 * 0x300 + 0x180), 8);
    assert!(!s.place_unit_admits(anchor, c), "a building's own cell");
}

/// A Nomad start is refused before a draw is spent.
#[test]
fn a_nomad_start_is_refused_without_a_draw() {
    let mut s = Sim::new(Tuning::RON, World::new(8, 8), 2);
    s.ai.lobby.starting_town = 0;
    let seed = s.rng.seed;
    assert_eq!(
        s.place_unit(0, None, 0, Pos::new(0x180, 0x180)),
        Err(super::PlaceRefused::Nomad)
    );
    assert_eq!(s.rng.seed, seed);
}
