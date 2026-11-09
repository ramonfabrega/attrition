use super::*;
use crate::world::{Cell, Terrain, World, tile};

/// Player 0's armed walker whose path top is a flagged waypoint over
/// player 1's building, one blocked footprint tile at `(10, 10)`.
fn blocked_top() -> (Sim, usize, usize) {
    let mut world = World::new(10, 10);
    world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(9, 9));
    let mut sim = Sim::new(crate::Tuning::RON, world, 2);
    let mut ty = crate::UnitType::default();
    ty.combat.domain = crate::attrition::Domain::Land;
    ty.combat.attack = 160;
    let t = sim.add_unit_type(ty);
    let mut unit = crate::Unit::new(0, 1, Pos::new(0x180, 0x180), 10);
    unit.ty = Some(t);
    let u = sim.units.len();
    sim.units.push(unit);
    let at = Pos::new(10, 10);
    sim.world
        .set_tile_mask(at, tile::OBJECT_BUILDING | tile::BLOCKED);
    let b = sim.add_building(1, Pos::new(10 * 0xc0, 10 * 0xc0), 0);
    sim.units[u].path.push(PathData {
        to: Pos::new(10 * 0xc0 + 0x60, 10 * 0xc0 + 0x60),
        tolerance: 0x60,
        flags: path_flag::BLOCK,
    });
    (sim, u, b)
}

/// **`Unit::resolve_block@005fccc0`, both live arms** (item 1209,
/// `5fcdc2`..`5fce11`): at war the walker takes the building as an
/// attack in front of what it holds and `do_move` returns; at peace
/// the walker's leader ors `2` into `agendas[owner]`, one direction,
/// and the walk goes on. A top on open ground asks nothing. Made to
/// fail first with each arm answering the other's.
#[test]
fn a_building_across_the_walk_is_attacked_at_war_and_noted_at_peace() {
    let (mut sim, u, b) = blocked_top();
    sim.at_war[0][1] = true;
    assert!(sim.resolve_block(u), "at war the walk stops");
    assert!(matches!(
        sim.units[u].orders.front().map(|o| o.body),
        Some(Body::Attack(_))
    ));
    assert_eq!(sim.units[u].combat.target, Some(Obj::Building(b)));
    assert_eq!(sim.agendas[0][1], 0);

    let (mut sim, u, _) = blocked_top();
    assert!(!sim.resolve_block(u), "at peace the walk goes on");
    assert!(sim.units[u].orders.is_empty(), "no order at peace");
    assert_eq!(sim.agendas[0][1], 2, "the walker's leader's slot");
    assert_eq!(sim.agendas[1][0], 0, "one direction only");

    let (mut sim, u, _) = blocked_top();
    sim.at_war[0][1] = true;
    sim.units[u].path.last_mut().expect("a top").to = Pos::new(0x480, 0x480);
    assert!(!sim.resolve_block(u), "open ground asks nothing");
    assert!(sim.units[u].orders.is_empty());
}
