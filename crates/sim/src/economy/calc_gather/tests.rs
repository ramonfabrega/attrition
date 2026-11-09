use super::*;
use crate::world::{CellData, Good, Terrain, UNITS_PER_TILE, World, tile};
use crate::{Tuning, tech};

/// `TypeIndex::FISH`.
const A_GOOD: TypeId = 6;

/// A 40 × 40 all-ocean world with one AI player and a Fisherman at the
/// centre of cell `(20, 20)` — `fish.rs`'s fixture with the **tile**
/// grid filled in, which is the half this walk reads and that one does
/// not.
fn sea_sim() -> (Sim, usize) {
    let mut world = World::new(40, 40);
    world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
    for y in 0..40 {
        for x in 0..40 {
            world.set_cell_data(
                Cell::new(x, y),
                CellData {
                    land: 2,
                    down: -1,
                    ..CellData::default()
                },
            );
            for ty in 0..TILES_PER_CELL {
                for tx in 0..TILES_PER_CELL {
                    world.set_tile_mask(
                        Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty),
                        tile::SURFACE_OCEAN,
                    );
                }
            }
        }
    }
    let mut s = Sim::new(Tuning::RON, world, 2);
    s.cities.nation[0].human = true;
    s.cities.nation[1].human = false;
    {
        let t = &mut s.tables.tech_tree;
        for n in [
            "Food",
            "Timber",
            "Wealth",
            "Knowledge",
            "Metal",
            "Oil",
            "Fish",
        ] {
            t.add(tech::TypeDef::good(n));
        }
        while t.types.len() < crate::fish::FISHERMEN {
            t.add(tech::TypeDef::plain("filler", -1));
        }
        let boat = t.add(tech::TypeDef::unit(
            "Fishing Boat",
            tech::UnitTraits::default(),
        ));
        assert_eq!(
            boat,
            crate::fish::FISHERMEN,
            "the fixture's ids are TypeIndex's"
        );
    }
    s.tech = (0..2)
        .map(|_| tech::PlayerTech::new(&s.tables.tech_tree))
        .collect();
    let t = s.add_unit_type(crate::UnitType {
        hits: 20,
        moves: 40,
        tree: Some(crate::fish::FISHERMEN),
        ..crate::UnitType::default()
    });
    s.tables.unit_types[t].combat.packs = true;
    s.tables.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
    let at = Pos::new(20 * 768 + 384, 20 * 768 + 384);
    let mut u = crate::Unit::new(1, 14, at, 20);
    u.ty = Some(t);
    let i = s.add_unit(u);
    (s, i)
}

/// Puts a good in a cell the way the map loader does, and marks the
/// tile it sits on — `Objects::init_good` writes the cell's chain
/// terminator and `Object::add_to_world` the tile's own bit, and the
/// **two are not the same tile as the boat's**.
fn put_good(s: &mut Sim, at: Pos, ty: TypeId) {
    s.world.add_good(Good {
        pos: at,
        ty,
        alive: true,
    });
    let t = at.tile();
    let m = s.world.tile_mask(t);
    s.world.set_tile_mask(t, m | tile::AS_BUILDING);
}

/// `<FROM>none</FROM>` on the shipped Fishermen, so the radius is the
/// bottom of the ladder: four tiles, one cell.
#[test]
fn a_type_with_no_from_searches_four_tiles() {
    let (s, u) = sea_sim();
    assert_eq!(s.upgrade_level(s.unit_tree(u).unwrap()), 0);
    assert_eq!(s.gather_radius(u), 4);
}

/// Nothing anywhere: `rare` goes to `-1`, `good_obj` stands, and the
/// answer is "no, move".
#[test]
fn an_empty_sea_answers_no_and_leaves_rare_at_minus_one() {
    let (mut s, u) = sea_sim();
    assert!(!s.calc_gather(u));
    assert_eq!(s.units[u].rare, -1);
    assert_eq!(s.units[u].good_obj, -1);
    assert!(!s.units[u].gather_here);
}

/// **The 5106 shape.** A fish in the boat's own cell but not under its
/// own tile: ring 0 fails the `AS_BUILDING` test and ring **1** — the
/// spiral's first neighbour — carries it, which is the `good_obj 1`
/// run58's dump prints from frame 4992 on.
#[test]
fn a_fish_beside_the_boat_s_tile_answers_yes_at_ring_one() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        A_GOOD,
    );
    let seed = s.rng.seed;
    assert!(s.calc_gather(u), "alone on it");
    assert_eq!(s.units[u].rare, i32::try_from(A_GOOD).unwrap());
    assert_eq!(s.units[u].good_obj, 1);
    assert_eq!(s.rng.seed, seed, "and the whole walk spends no draw");
}

/// The remembered index is retried on its own and keeps its value —
/// block A, which is what makes the field state rather than an output.
#[test]
fn the_remembered_index_is_retried_first_and_kept() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        A_GOOD,
    );
    assert!(s.calc_gather(u));
    assert_eq!(s.units[u].good_obj, 1);
    assert!(s.calc_gather(u), "the second call takes block A");
    assert_eq!(s.units[u].good_obj, 1);
}

/// A remembered index whose tile is still water but has lost its good
/// is cleared to `-1` before the spiral runs.
#[test]
fn a_remembered_index_that_has_lost_its_good_is_cleared() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    // The tile keeps its `AS_BUILDING` bit and the cell loses its good.
    let m = s.world.tile_mask(one);
    s.world.set_tile_mask(one, m | tile::AS_BUILDING);
    s.units[u].good_obj = 1;
    assert!(!s.calc_gather(u));
    assert_eq!(s.units[u].good_obj, -1);
}

/// **The crowd test is the return value.** A second boat of the same
/// lineage, unpacked, inside `(4 + 4) × 192`, turns the same fish from
/// "stay" into "go" — and it is `rare` that says the good was found
/// either way.
#[test]
fn a_second_boat_on_the_same_fish_turns_the_answer_round() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        A_GOOD,
    );
    let ty = s.units[u].ty;
    let mut other = crate::Unit::new(1, 15, s.units[u].pos, 20);
    other.ty = ty;
    // `add_unit` is `Object::add_to_world`: it puts the boat in the
    // cell's chain itself.
    let o = s.add_unit(other);
    s.units[o].combat.packed = false;
    assert!(!s.calc_gather(u), "somebody else is sharing it");
    assert_eq!(
        s.units[u].rare,
        i32::try_from(A_GOOD).unwrap(),
        "the good was found; the count is what refuses it"
    );
    // A **packed** neighbour is not competition at all.
    s.units[o].combat.packed = true;
    assert!(s.calc_gather(u));
}

/// Another player's boat on the same fish is nobody's competition:
/// the chain walk is fenced on `who` before anything else.
#[test]
fn another_player_s_boat_is_not_counted() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        A_GOOD,
    );
    let ty = s.units[u].ty;
    let mut other = crate::Unit::new(0, 15, s.units[u].pos, 20);
    other.ty = ty;
    // `add_unit` is `Object::add_to_world`: it puts the boat in the
    // cell's chain itself.
    let o = s.add_unit(other);
    s.units[o].combat.packed = false;
    assert!(s.calc_gather(u));
}

/// Oil is in no cell's chain and is refused by type besides, so a boat
/// sitting on an oil patch is sitting on nothing.
#[test]
fn oil_is_not_a_good_calc_gather_can_find() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        crate::world::OIL_GOOD,
    );
    assert!(!s.calc_gather(u));
    assert_eq!(s.units[u].rare, -1);
}

/// A **land** tile is refused for anything that is not a merchant, and
/// the fish under it with it.
#[test]
fn a_fisherman_refuses_a_land_tile() {
    let (mut s, u) = sea_sim();
    let t0 = s.units[u].pos.tile();
    let circle = crate::ai_place::circle();
    let one = Pos::new(t0.x + circle.x[1], t0.y + circle.y[1]);
    put_good(
        &mut s,
        Pos::new(one.x * UNITS_PER_TILE + 96, one.y * UNITS_PER_TILE + 96),
        A_GOOD,
    );
    let m = s.world.tile_mask(one);
    s.world
        .set_tile_mask(one, (m & !tile::SURFACE) | tile::AS_BUILDING);
    assert!(!s.calc_gather(u), "the surface field is the first test");
}
