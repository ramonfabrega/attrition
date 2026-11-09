use crate::ai::transport::ty::TRANSPORTBARGE;
use crate::orders::{MoveKind, QueuePos};
use crate::territory::attrition::Domain;
use crate::world::{Cell, Terrain, World, tile};
use crate::{Pos, Sim, Tuning, UnitType};

/// A 16×8 world: land on the left eight cells, sea on the right eight,
/// the sea cells `OCEAN` (`land` 2) with their tiles surfaced as ocean.
/// Types: a land foot unit, a ship, and a barge of `TRANSPORTBARGE`.
struct Fix {
    sim: Sim,
    foot: usize,
    ship: usize,
    barge: usize,
}

fn fix() -> Fix {
    let mut w = World::new(16, 8);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(15, 7));
    for y in 0..8 {
        for x in 8..16 {
            let c = Cell::new(x, y);
            let mut d = w.cell_data(c);
            d.land = 2;
            w.set_cell_data(c, d);
            for tx in x * 4..x * 4 + 4 {
                for ty in y * 4..y * 4 + 4 {
                    w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
                }
            }
        }
    }
    let mut sim = Sim::new(Tuning::RON, w, 2);
    let foot = sim.add_unit_type(UnitType {
        hits: 40,
        ..UnitType::default()
    });
    let mut sea = UnitType {
        hits: 100,
        ..UnitType::default()
    };
    sea.combat.domain = Domain::Sea;
    let ship = sim.add_unit_type(sea.clone());
    let barge = sim.add_unit_type(sea);
    sim.tables.unit_types[barge].tree = Some(TRANSPORTBARGE);
    Fix {
        sim,
        foot,
        ship,
        barge,
    }
}

/// Quarter-cell (tile) coordinates of land, of land within `0xc00` of
/// the sea, and of the open sea.
const LAND: (i32, i32) = (12, 16);
const SHORE: (i32, i32) = (28, 16);
const SEA: (i32, i32) = (52, 16);

/// **The refusals** (`add_unit`'s `-1`s): a player out of range, `num`
/// over 2,000, the barge itself as the type, and a point off the map.
/// Made to fail with each guard dropped in turn.
#[test]
fn add_unit_refuses_the_barge_a_stranger_and_a_point_off_the_map() {
    let mut f = fix();
    assert_eq!(f.sim.scenario_add_unit(5, f.foot, LAND, 1), None);
    assert_eq!(f.sim.scenario_add_unit(1, f.foot, LAND, 2001), None);
    assert_eq!(f.sim.scenario_add_unit(1, f.barge, SEA, 1), None);
    assert_eq!(f.sim.scenario_add_unit(1, f.foot, (64, 16), 1), None);
    assert!(f.sim.units.is_empty(), "a refusal makes nothing");
}

/// **The wrong element places nothing but is no refusal**: a ship on
/// land, and a land unit on the ocean while its leader cannot
/// transport, return an empty list — the original's `return` short of
/// `-1`. The ship's point is within `0xc00` of the water, so a
/// placement search would find it a spot. Made to fail with the
/// `(Sea, false)` arm folded into the placement arm.
#[test]
fn a_ship_on_land_and_a_soldier_at_sea_without_docks_place_nothing() {
    let mut f = fix();
    assert_eq!(f.sim.scenario_add_unit(1, f.ship, SHORE, 1), Some(vec![]));
    assert_eq!(f.sim.scenario_add_unit(1, f.foot, SEA, 1), Some(vec![]));
    assert!(f.sim.units.is_empty());
}

/// **A land unit on land** is seated round the point and left there,
/// on its own tile, with no order standing: `action_form` gave each a
/// move to its slot and `dbg_jump_to_action` took it. Made to fail
/// with the jump dropped (the `CHANGE_FORM` stays current).
#[test]
fn a_land_squad_on_land_is_formed_and_snapped_to_its_slots() {
    let mut f = fix();
    let made = f.sim.scenario_add_unit(1, f.foot, LAND, 3).unwrap();
    assert_eq!(made.len(), 3);
    for &u in &made {
        let unit = &f.sim.units[u];
        assert!(unit.alive() && unit.on_map && unit.owner == 1);
        assert!(!f.sim.world.is_ocean(unit.pos.cell()), "seated on land");
        assert!(
            f.sim.current_order(u).is_none(),
            "the jump popped every move: {:?}",
            f.sim.current_order(u)
        );
    }
}

/// **A land unit at sea, with a dock's level, goes aboard**
/// (`action_transport(1)`): one barge per squad, of the leader's
/// `TRANSPORTBARGE`, the passenger inside it and off the map, and the
/// barge on the ocean. The host swap makes the barge what is formed: it
/// takes the formation's width, where a boat is born with −1 (run1511's
/// barge `0/9`, `form_mod 50` on 621). Made to fail with
/// `action_transport` dropped and with the host swap dropped.
#[test]
fn a_soldier_at_sea_with_a_dock_goes_aboard_a_barge() {
    let mut f = fix();
    f.sim.ai.transport[1].military = true;
    let made = f.sim.scenario_add_unit(1, f.foot, SEA, 2).unwrap();
    assert_eq!(made.len(), 2);
    for &u in &made {
        let host = f.sim.units[u].inside_unit.expect("aboard");
        assert_eq!(f.sim.units[host].ty, Some(f.barge));
        assert!(!f.sim.units[u].on_map, "a passenger is off the map");
        assert!(f.sim.units[host].on_map);
        assert!(f.sim.world.is_ocean(f.sim.units[host].pos.cell()));
        assert_ne!(f.sim.units[host].form_width, -1, "the barge is formed");
    }
    let hosts: Vec<_> = made.iter().map(|&u| f.sim.units[u].inside_unit).collect();
    assert_ne!(hosts[0], hosts[1], "a barge for each squad");
}

/// **`Unit::dbg_jump_to_action` pops the move family to its last
/// point** and stops at the first order outside it; a unit whose
/// current order is not a move is left where it stands. Made to fail
/// with the pop loop run once.
#[test]
fn the_jump_takes_every_leading_move_and_lands_on_the_last() {
    let mut f = fix();
    let u = f.sim.init_unit(1, f.foot, Pos::new(1000, 1000));
    let first = Pos::new(1200, 1000);
    let last = Pos::new(1200, 1400);
    f.sim
        .add_move_order(u, first, MoveKind::MoveTo, QueuePos::New, true);
    f.sim
        .add_move_order(u, last, MoveKind::MoveTo, QueuePos::Last, true);
    let snapped = match f.sim.units[u].orders.back().map(|o| &o.body) {
        Some(crate::orders::Body::Move(m)) => m.dest,
        o => panic!("the last order is the second move: {o:?}"),
    };
    f.sim.unit_dbg_jump_to_action(u);
    assert_eq!(f.sim.units[u].pos, snapped, "on the last move's point");
    assert!(f.sim.current_order(u).is_none(), "both moves popped");

    let v = f.sim.init_unit(1, f.foot, Pos::new(600, 600));
    let at = f.sim.units[v].pos;
    f.sim.unit_dbg_jump_to_action(v);
    assert_eq!(f.sim.units[v].pos, at, "no move, no jump");
}

/// **An invalid last point is replaced by `find_nearby_spot` round
/// it** (`0xc0`..`0x600`): a foot unit sent into the sea lands on the
/// shore, not in the water. Made to fail with the `invalid_loc` test
/// dropped.
#[test]
fn a_jump_into_the_sea_lands_on_the_nearest_shore() {
    let mut f = fix();
    let u = f.sim.init_unit(1, f.foot, Pos::new(5000, 1500));
    let wet = Pos::new(8 * 768 + 200, 1500);
    f.sim
        .add_move_order(u, wet, MoveKind::MoveTo, QueuePos::New, true);
    f.sim.unit_dbg_jump_to_action(u);
    let pos = f.sim.units[u].pos;
    assert!(!f.sim.world.is_ocean(pos.cell()), "on land: {pos:?}");
    assert!((pos.x - wet.x).abs() <= 0x600 + 0x60, "round the point");
}
