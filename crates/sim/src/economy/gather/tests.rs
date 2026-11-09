use super::*;
use crate::build::{BuildType, Ident};
use crate::tuning::Tuning;
use crate::world::{Owner, Terrain, UNITS_PER_TILE};

fn tile_pos(tx: i32, ty: i32) -> Pos {
    Pos::new(
        tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

fn bt(ident: Ident, xs: i32, ys: i32) -> BuildType {
    BuildType {
        ident,
        x_size: xs,
        y_size: ys,
        flags: flags::parse("gda"),
        job_time: 150,
        hits: 400,
        ..BuildType::default()
    }
}

/// A 16×16-cell land world with one player.
fn sim() -> Sim {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    Sim::new(Tuning::RON, w, 2)
}

/// Forest over a whole cell, and the access bit the cap wants.
fn forest_cell(sim: &mut Sim, c: Cell) {
    for v in 0..TILES_PER_CELL {
        for u in 0..TILES_PER_CELL {
            let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
            sim.world
                .set_tile_bits(t, tile::SURFACE_FOREST | GATHERABLE);
        }
    }
}

/// Only `n` tiles of the cell are forest — the cell still pays in full.
fn part_forest_cell(sim: &mut Sim, c: Cell, n: i32) {
    // The centre tile must be forest: it is the one the survey tests.
    let centre = c.centre_tile();
    sim.world
        .set_tile_bits(centre, tile::SURFACE_FOREST | GATHERABLE);
    let mut left = n - 1;
    for v in 0..TILES_PER_CELL {
        for u in 0..TILES_PER_CELL {
            if left <= 0 {
                return;
            }
            let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
            if t == centre {
                continue;
            }
            sim.world
                .set_tile_bits(t, tile::SURFACE_FOREST | GATHERABLE);
            left -= 1;
        }
    }
}

fn camp(sim: &mut Sim) -> usize {
    sim.add_build_type(bt(Ident::Woodcutter, 2, 2))
}

#[test]
fn a_flat_type_is_one_citizen_and_never_looks_at_the_map() {
    let mut s = sim();
    let mut farm = bt(Ident::Farm, 4, 4);
    farm.flags |= flags::FLAT;
    let ty = s.add_build_type(farm);
    let b = s.init_build(0, ty, tile_pos(20, 20), false);
    assert_eq!(s.max_gatherers(b), MAX_FLAT_GATHERERS);
}

#[test]
fn a_university_is_seven_citizens() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::University, 4, 4));
    let b = s.init_build(0, ty, tile_pos(20, 20), false);
    assert_eq!(s.max_gatherers(b), MAX_KNOWLEDGE_GATHERERS);
}

#[test]
fn a_building_that_gathers_nothing_has_no_slots() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::Barracks, 4, 4));
    let b = s.init_build(0, ty, tile_pos(20, 20), false);
    assert_eq!(s.max_gatherers(b), 0);
}

/// The finding the run9 camps forced: a cell pays sixteen sixteenths
/// whether nine of its tiles are forest or all sixteen are, so the count
/// is the number of forest **cells**.
#[test]
fn a_partly_forested_cell_pays_as_much_as_a_full_one() {
    let mut full = sim();
    let ty = camp(&mut full);
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        forest_cell(&mut full, c);
    }
    assert_eq!(full.max_gatherers_at(ty, 0, Pos::new(21, 21)), 3);

    let mut part = sim();
    let ty = camp(&mut part);
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        part_forest_cell(&mut part, c, 9);
    }
    assert_eq!(
        part.max_gatherers_at(ty, 0, Pos::new(21, 21)),
        3,
        "nine tiles is still one cell"
    );
}

/// A camp with nothing around it has nothing to give.
#[test]
fn a_camp_on_bare_ground_has_no_slots() {
    let mut s = sim();
    let ty = camp(&mut s);
    let b = s.init_build(0, ty, tile_pos(21, 21), false);
    assert_eq!(s.max_gatherers(b), 0);
}

#[test]
fn french_timber_capacity_adds_a_worker_to_survey_and_existing_list() {
    let mut s = sim();
    let ty = camp(&mut s);
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        forest_cell(&mut s, c);
    }
    let corner = Pos::new(21, 21);
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 3);
    s.nation[0].french = true;
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 4);
    s.tables.tuning.french_woodies = 3;
    assert_eq!(
        s.max_gatherers_at(ty, 0, corner),
        6,
        "the tuning value is used"
    );
    s.tables.tuning.french_woodies = 1;
    let b = s.init_build(0, ty, tile_pos(21, 21), false);
    assert_eq!(s.max_gatherers(b), 4);
    s.nation[0].french = false;
    assert_eq!(s.max_gatherers(b), 3);
}

#[test]
fn french_timber_capacity_needs_trees_and_respects_access() {
    let mut s = sim();
    let ty = camp(&mut s);
    s.nation[0].french = true;
    let corner = Pos::new(21, 21);
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 0, "bare ground");
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        for v in 0..TILES_PER_CELL {
            for u in 0..TILES_PER_CELL {
                s.world.set_tile_bits(
                    Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v),
                    tile::SURFACE_FOREST,
                );
            }
        }
    }
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 0, "no access");
    // One accessible tile caps both ordinary and French capacity at two.
    s.world.set_tile_bits(Pos::new(27, 23), GATHERABLE);
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 2, "bonus precedes cap");
    let mut farm = bt(Ident::Farm, 4, 4);
    farm.flags |= flags::FLAT;
    let farm = s.add_build_type(farm);
    assert_eq!(s.max_gatherers_at(farm, 0, corner), 1, "no flat-farm bonus");
}

#[test]
fn french_timber_capacity_keeps_the_taken_site_sentinel() {
    let mut s = sim();
    let ty = camp(&mut s);
    s.nation[0].french = true;
    for c in [Cell::new(3, 5), Cell::new(7, 5)] {
        forest_cell(&mut s, c);
        s.world.set_tile_bits(c.centre_tile(), GATHERED_FROM);
    }
    let corner = Pos::new(21, 21);
    assert_eq!(s.site_gather_count(ty, 0, corner, None), -1);
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 0);
}

/// Ring `(8 + 3) / 4 = 2` and `vector_dist <= 8` between the anchor tile
/// and the cell's centre tile: a forest cell three cells away is out of
/// reach, one two cells away is not.
#[test]
fn the_reach_is_two_cells_and_eight_tiles() {
    let mut s = sim();
    let ty = camp(&mut s);
    // Anchor: corner (5, 5), 2×2, so centre world 5·192 + 192 = 1152 →
    // tile (6, 6), cell (1, 1).
    forest_cell(&mut s, Cell::new(3, 1));
    forest_cell(&mut s, Cell::new(4, 1));
    // Cell (3, 1) is two cells out: sample tile (14, 6), distance 8.
    // Cell (4, 1) is three: outside ring 2 altogether.
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(5, 5)), 1);
}

/// With a `MiningList` the cell qualifies on the list holding its centre
/// tile, not on the map — the path `max_gatherers` takes for a building
/// that has already surveyed.
#[test]
fn an_existing_building_counts_the_cells_its_own_list_names() {
    let mut s = sim();
    let ty = camp(&mut s);
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        forest_cell(&mut s, c);
    }
    let b = s.init_build(0, ty, tile_pos(21, 21), false);
    // Two of the three cells' centre tiles, plus a tile that is not any
    // cell's centre and so pays nothing.
    s.buildings[b].gather_from = vec![
        Cell::new(5, 5).centre_tile(),
        Cell::new(6, 5).centre_tile(),
        Pos::new(21, 21),
    ];
    assert_eq!(s.max_gatherers(b), 2);
}

/// `is_gathered_from`: a cell another building already took is passed
/// over, and a survey that finds nothing *because* of that answers zero
/// rather than a bare zero.
#[test]
fn a_cell_already_gathered_from_is_passed_over() {
    let mut s = sim();
    let ty = camp(&mut s);
    // Both cells stand more than three tiles out, so the access cap
    // stays clear of the count either way.
    for c in [Cell::new(3, 5), Cell::new(7, 5)] {
        forest_cell(&mut s, c);
    }
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 2);
    s.world
        .set_tile_bits(Cell::new(7, 5).centre_tile(), GATHERED_FROM);
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 1);
    s.world
        .set_tile_bits(Cell::new(3, 5).centre_tile(), GATHERED_FROM);
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 0);
}

/// An enemy's cell is skipped; an ally's is not.
#[test]
fn an_enemy_s_cell_does_not_pay() {
    let mut s = sim();
    let ty = camp(&mut s);
    for c in [Cell::new(3, 5), Cell::new(7, 5)] {
        forest_cell(&mut s, c);
    }
    s.world
        .set_owner(Cell::new(7, 5), crate::Owner::Player(1), crate::Owner::None);
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 1);
    s.allied[0][1] = true;
    s.allied[1][0] = true;
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 2);
}

/// The access cap: twice the number of list tiles that carry
/// [`GATHERABLE`] and stand more than three tiles out.
#[test]
fn the_access_cap_bites_when_no_tile_has_access() {
    let mut s = sim();
    let ty = camp(&mut s);
    for c in [Cell::new(5, 5), Cell::new(6, 5), Cell::new(5, 6)] {
        for v in 0..TILES_PER_CELL {
            for u in 0..TILES_PER_CELL {
                let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
                // Forest, but no `GATHERABLE` bit anywhere.
                s.world.set_tile_bits(t, tile::SURFACE_FOREST);
            }
        }
    }
    assert_eq!(
        s.max_gatherers_at(ty, 0, Pos::new(21, 21)),
        0,
        "three cells, but 2 × 0 access"
    );
}

/// The river term: `accum += (accum · rivers) >> 4`, so sixteen river
/// tiles under the footprint would double the count. A 4×4 camp on four
/// river tiles turns eight cells into ten.
#[test]
fn rivers_under_the_footprint_add_sixteenths() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::Woodcutter, 4, 4));
    for dy in 0..3 {
        for dx in 0..3 {
            forest_cell(&mut s, Cell::new(4 + dx, 4 + dy));
        }
    }
    let dry = s.max_gatherers_at(ty, 0, Pos::new(20, 20));
    assert_eq!(dry, 9, "nine forest cells, all in reach");
    for v in 0..2 {
        for u in 0..2 {
            s.world.set_tile_bits(Pos::new(20 + u, 20 + v), tile::RIVER);
        }
    }
    // 9·16 = 144, plus (144·4) >> 4 = 36, is 180; (180 + 8) >> 4 = 11.
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(20, 20)), 11);
}

/// The mine: the range's tile count picks a rung of `MTN_*_GATHER`, and
/// the rung is scaled by the share of the range that is usable.
#[test]
fn a_mine_takes_its_count_from_the_mountain_s_size() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::Mine, 2, 2));
    // Six cells of solid mountain: 96 tiles, under `MTN_TINY_SIZE`.
    for dy in 0..2 {
        for dx in 0..3 {
            let c = Cell::new(5 + dx, 5 + dy);
            for v in 0..TILES_PER_CELL {
                for u in 0..TILES_PER_CELL {
                    let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
                    s.world.set_tile_bits(t, tile::OBJECT_MOUNTAIN | GATHERABLE);
                }
            }
        }
    }
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 3);
    // Two more cells take it over 100 tiles, to the next rung.
    for dx in 0..2 {
        let c = Cell::new(5 + dx, 7);
        for v in 0..TILES_PER_CELL {
            for u in 0..TILES_PER_CELL {
                let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
                s.world.set_tile_bits(t, tile::OBJECT_MOUNTAIN | GATHERABLE);
            }
        }
    }
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 5);
}

/// Marks every tile of `c` a mountain.
fn mountain_cell(s: &mut Sim, c: Cell) {
    for v in 0..TILES_PER_CELL {
        for u in 0..TILES_PER_CELL {
            let t = Pos::new(c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v);
            s.world.set_tile_bits(t, tile::OBJECT_MOUNTAIN | GATHERABLE);
        }
    }
}

/// A cell's centre in world units.
fn centre(c: Cell) -> Pos {
    Pos::new(
        c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    )
}

/// **With the generator's placements, only a template's solid cells
/// count** — a mountain cell no template lists is not reached,
/// however near (`docs/AI.md` §60).
#[test]
fn a_placed_range_measures_to_its_solid_cells_only() {
    let mut s = sim();
    mountain_cell(&mut s, Cell::new(5, 5));
    mountain_cell(&mut s, Cell::new(6, 5));
    let site = centre(Cell::new(4, 5));
    // The stand-in reaches (5, 5), one cell away.
    assert_eq!(s.nearest_mountain_cell(site), Some((768, Cell::new(5, 5))));
    // The template lists (6, 5) alone: two cells, 1536, past the reach.
    let t = MountainTemplate {
        tiles: vec![(0, 0)],
        solid: vec![(1, 0)],
    };
    s.place_mountains(&[(Cell::new(5, 5), 0)], &[t]);
    assert_eq!(s.find_nearest_mountain(site), Some((1536, 0, 0)));
    assert_eq!(s.nearest_mountain_cell(site), None);
    assert_eq!(s.nearest_placed(centre(Cell::new(5, 5))), Some((768, 0, 0)));
    // Its tiles are the template's, at `4 · loc`.
    let r = s.mountain_range(centre(Cell::new(6, 6))).unwrap();
    assert_eq!(r.tiles, vec![Pos::new(20, 20)]);
    assert_eq!(r.cells, vec![Cell::new(6, 5)]);
}

/// **A range another building already mines is taken** (`docs/AI.md`
/// §62). The survey walks the range's tiles and answers −1 at the
/// first that is gathered from, which `blocked_location` reads as
/// `MountainTaken`; a tile in an enemy's cell does not count, and a
/// standing building's own list is never asked.
#[test]
fn a_range_already_mined_is_taken_on_the_survey() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::Mine, 2, 2));
    mountain_cell(&mut s, Cell::new(5, 5));
    mountain_cell(&mut s, Cell::new(6, 5));
    let t = MountainTemplate {
        tiles: (0..4).flat_map(|v| (0..8).map(move |u| (u, v))).collect(),
        solid: vec![(0, 0)],
    };
    s.place_mountains(&[(Cell::new(5, 5), 0)], &[t]);
    // A 2×2 footprint at tile (17, 21) is centred in cell (4, 5), one
    // cell from the solid cell.
    let corner = Pos::new(17, 21);
    let free = s.site_gather_count(ty, 0, corner, None);
    assert!(free > 0, "an unmined range pays: {free}");
    s.world.set_tile_bits(Pos::new(25, 22), GATHERED_FROM);
    assert_eq!(s.site_gather_count(ty, 0, corner, None), -1);
    assert_eq!(s.max_gatherers_at(ty, 0, corner), 0, "the clamp");
    // The gathered tile's cell belongs to an enemy: it is skipped, and
    // no other tile is taken.
    s.world
        .set_owner(Cell::new(6, 5), Owner::Player(1), Owner::None);
    assert_eq!(s.site_gather_count(ty, 0, corner, None), free);
}

/// A tie goes to the **first placed** range: `find_nearest` replaces
/// its best only on a strictly smaller distance.
#[test]
fn a_tie_goes_to_the_first_placed_range() {
    let mut s = sim();
    mountain_cell(&mut s, Cell::new(3, 5));
    mountain_cell(&mut s, Cell::new(7, 5));
    let t = MountainTemplate {
        tiles: vec![],
        solid: vec![(0, 0)],
    };
    let placed = [(Cell::new(7, 5), 0), (Cell::new(3, 5), 0)];
    s.place_mountains(&placed, &[t]);
    let site = centre(Cell::new(5, 5));
    assert_eq!(s.find_nearest_mountain(site), Some((1536, 0, 0)));
    s.place_mountains(
        &[placed[1], placed[0]],
        &[MountainTemplate {
            tiles: vec![],
            solid: vec![(0, 0)],
        }],
    );
    assert_eq!(s.nearest_mountain_cell(site).map(|(_, c)| c), None);
    assert_eq!(
        s.find_nearest_mountain(site)
            .map(|(_, i, _)| s.terrain.mountains[i].loc),
        Some(Cell::new(3, 5))
    );
}

/// The region test is made on a range's **first** solid cell: a range
/// that starts in another region is skipped whole, even where a later
/// cell of it stands in the site's.
#[test]
fn a_range_is_regioned_by_its_first_solid_cell() {
    let mut s = sim();
    let home = s.world.region_of(Cell::new(5, 5)).unwrap();
    let other = s.world.add_region(Terrain::Land);
    s.world.set_region(Cell::new(9, 5), other);
    let t = MountainTemplate {
        tiles: vec![],
        solid: vec![(3, 0), (0, 0)],
    };
    // First solid cell (9, 5) is another region; (6, 5) is home.
    s.place_mountains(&[(Cell::new(6, 5), 0)], &[t]);
    assert_eq!(s.find_nearest_mountain(centre(Cell::new(5, 5))), None);
    s.world.set_region(Cell::new(9, 5), home);
    assert_eq!(
        s.find_nearest_mountain(centre(Cell::new(5, 5))),
        Some((768, 0, 1))
    );
}

/// A mine with no mountain near it has no slots at all.
#[test]
fn a_mine_with_no_mountain_has_no_slots() {
    let mut s = sim();
    let ty = s.add_build_type(bt(Ident::Mine, 2, 2));
    assert_eq!(s.max_gatherers_at(ty, 0, Pos::new(21, 21)), 0);
}

/// Run9's two camps, as arithmetic: seven cells is seven slots and five
/// is five, which one-per-tile (82 and 61 tiles) cannot produce.
#[test]
fn run9_s_camps_are_cell_counts_not_tile_counts() {
    for (cells, tiles) in [(7, 82), (5, 61)] {
        assert_eq!((cells * 16 + 8) >> 4, cells, "sixteen a cell");
        assert_ne!((tiles + 8) >> 4, cells, "one a tile does not fit");
    }
}

/// Two woodland cells beside a camp's corner (5, 5), and the region
/// they stand in.
fn woods(s: &mut Sim) -> u16 {
    forest_cell(s, Cell::new(2, 1));
    forest_cell(s, Cell::new(2, 2));
    s.world.region_of(Cell::new(1, 1)).unwrap()
}

/// **`Build::close`'s tail** (`docs/ECONOMY.md` §17.1): a camp the
/// script places and destroys on one frame gives its ground back, so
/// the next camp on the same corner lists the same tiles. Without the
/// tail the second camp found every cell marked and listed nothing —
/// East Indies at Toughest's frame 1576 (`docs/AI.md` §82).
#[test]
fn a_closed_camp_gives_its_ground_back_to_the_next_one() {
    let mut s = sim();
    let ty = camp(&mut s);
    let region = woods(&mut s);
    let first = s.init_build(1, ty, tile_pos(5, 5), false);
    let tiles = s.buildings[first].gather_from.clone();
    assert_eq!(tiles.len(), 32, "two whole cells");
    assert!(
        tiles
            .iter()
            .all(|&t| s.world.tile_mask(t) & GATHERED_FROM != 0)
    );
    s.close_building(first, false);
    assert!(
        s.buildings[first].gather_from.is_empty(),
        "the list is emptied"
    );
    assert!(
        tiles
            .iter()
            .all(|&t| s.world.tile_mask(t) & GATHERED_FROM == 0),
        "every tile is unmarked"
    );
    assert_eq!(
        s.world.region_flags(region),
        REGION_VERIFY_GATHER,
        "the camp's region is flagged"
    );
    let second = s.init_build(1, ty, tile_pos(5, 5), false);
    let mut again = s.buildings[second].gather_from.clone();
    let mut was = tiles;
    again.sort_by_key(|p| (p.y, p.x));
    was.sort_by_key(|p| (p.y, p.x));
    assert_eq!(again, was, "the same ground, listed again");
}

/// A building that walks no ground — a farm is flat — flags nothing.
#[test]
fn a_flat_building_s_close_flags_no_region() {
    let mut s = sim();
    let region = woods(&mut s);
    let mut farm = bt(Ident::Farm, 4, 4);
    farm.flags |= flags::FLAT;
    let ty = s.add_build_type(farm);
    let b = s.init_build(1, ty, tile_pos(5, 5), false);
    s.close_building(b, false);
    assert_eq!(s.world.region_flags(region), 0);
}

/// **`GameDaemon::process_all`'s region pass and `Build::process`'s
/// re-walk** (`docs/ECONOMY.md` §17.2): `0x20` becomes `0x10` on the
/// next pass and `0x10` is cleared on the one after, and on the frame
/// it stands an active camp in the region takes the freed ground and
/// shuffles it — `4 × length` draws.
#[test]
fn a_freed_region_is_re_walked_by_the_camp_beside_it() {
    let mut s = sim();
    let ty = camp(&mut s);
    let region = woods(&mut s);
    forest_cell(&mut s, Cell::new(4, 1));
    // The neighbour, corner (13, 5): it reaches cell (4, 1) and cell
    // (2, 1)'s ground is the first camp's.
    let first = s.init_build(1, ty, tile_pos(5, 5), false);
    let near = s.init_build(1, ty, tile_pos(13, 5), false);
    s.buildings[near].active = true;
    let held = s.buildings[near].gather_from.len();
    s.close_building(first, false);
    s.world.cycle_gather_flags();
    assert_eq!(s.world.region_flags(region), REGION_REFIND_GATHER);
    let seed = s.rng.seed;
    for b in 0..s.buildings.len() {
        s.gather_region_building(b);
    }
    let grown = s.buildings[near].gather_from.len();
    assert!(
        grown > held,
        "the neighbour took freed ground: {held} → {grown}"
    );
    let mut r = crate::combat::Rng::new(seed);
    for _ in 0..4 * grown {
        r.get(0, 0xffff);
    }
    assert_eq!(s.rng.seed, r.seed, "4 × {grown} draws");
    s.world.cycle_gather_flags();
    assert_eq!(s.world.region_flags(region), 0, "cleared on the next pass");
    let seed = s.rng.seed;
    for b in 0..s.buildings.len() {
        s.gather_region_building(b);
    }
    assert_eq!(s.rng.seed, seed, "no flag, no walk");
}

/// **`Build::verify_gather_tiles`** (`docs/ECONOMY.md` §17.2): under
/// `0x20`, a listed tile on an unallied player's ground is unmarked and
/// dropped, the rest keep their order, and nothing is drawn.
#[test]
fn a_verified_camp_drops_the_ground_another_player_holds() {
    let mut s = sim();
    let ty = camp(&mut s);
    let region = woods(&mut s);
    let b = s.init_build(1, ty, tile_pos(5, 5), false);
    s.buildings[b].active = true;
    let before = s.buildings[b].gather_from.clone();
    s.world
        .set_owner(Cell::new(2, 2), Owner::Player(0), Owner::None);
    s.world.or_region_flags(region, REGION_VERIFY_GATHER);
    let seed = s.rng.seed;
    for b in 0..s.buildings.len() {
        s.gather_region_building(b);
    }
    assert_eq!(s.rng.seed, seed, "no draw");
    let kept: Vec<Pos> = before
        .iter()
        .copied()
        .filter(|&t| World::cell_of_tile(t) != Cell::new(2, 2))
        .collect();
    assert_eq!(
        s.buildings[b].gather_from, kept,
        "in order, less cell (2, 2)"
    );
    assert!(
        before
            .iter()
            .filter(|&&t| World::cell_of_tile(t) == Cell::new(2, 2))
            .all(|&t| s.world.tile_mask(t) & GATHERED_FROM == 0),
        "the dropped tiles are unmarked"
    );
}
