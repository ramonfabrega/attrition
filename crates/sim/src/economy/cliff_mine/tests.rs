use super::*;
use crate::cities::build::{BuildType, Ident, flags};
use crate::core::tuning::Tuning;
use crate::economy::gather::{MineArm, MountainTemplate};
use crate::world::{Owner, TILES_PER_CELL, Terrain};

/// A 16×16-cell land world, two players, and a 2×2 Mine type.
fn sim() -> (Sim, usize) {
    let mut w = World::new(16, 16);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
    let mut s = Sim::new(Tuning::RON, w, 2);
    let ty = s.add_build_type(BuildType {
        ident: Ident::Mine,
        x_size: 2,
        y_size: 2,
        flags: flags::parse("gda"),
        job_time: 150,
        hits: 400,
        ..BuildType::default()
    });
    (s, ty)
}

/// The sixteen tiles of a cell, row by row — `Cliffs::init`'s order.
fn cell_tiles(c: Cell) -> Vec<(i32, i32)> {
    (0..TILES_PER_CELL)
        .flat_map(|v| {
            (0..TILES_PER_CELL).map(move |u| (c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v))
        })
        .collect()
}

/// A 2×2 footprint at tile (17, 21): centred at (3456, 4224), 768 from
/// cell (5, 5)'s centre and 1536 from (6, 5)'s.
const CORNER: Pos = Pos::new(17, 21);

/// A cliff's cells are its tiles' cells in first-seen order.
#[test]
fn a_cliff_s_cells_are_its_tiles_cells_in_order() {
    let mut tiles = cell_tiles(Cell::new(5, 5));
    tiles.extend(cell_tiles(Cell::new(3, 6)));
    let c = CliffMining::from_tiles(&tiles);
    assert_eq!(c.cells, vec![Cell::new(5, 5), Cell::new(3, 6)]);
    assert_eq!(c.tiles.len(), 32);
}

/// **The nearest cliff, strictly**: of two cliffs the same distance
/// away the first keeps the site, and a cliff is measured to its
/// nearest cell.
#[test]
fn find_nearest_measures_every_cell_and_a_tie_keeps_the_first() {
    let (mut s, ty) = sim();
    let site = s.footprint_centre(ty, CORNER);
    let mut far = cell_tiles(Cell::new(9, 9));
    far.extend(cell_tiles(Cell::new(5, 5)));
    s.place_cliffs(&[
        cell_tiles(Cell::new(3, 3)),
        far,
        cell_tiles(Cell::new(4, 4)),
    ]);
    // Cliff 1's second cell, (5, 5), is 768 away; cliff 2's (4, 4) is
    // 768 + 384·384/1536 = 864.
    assert_eq!(s.find_nearest_cliff(site), Some((768, 1)));
    s.place_cliffs(&[cell_tiles(Cell::new(5, 5)), cell_tiles(Cell::new(5, 5))]);
    assert_eq!(s.find_nearest_cliff(site), Some((768, 0)));
    s.place_cliffs(&[Vec::new()]);
    assert_eq!(s.find_nearest_cliff(site), None, "a cliff with no cell");
}

/// **Only the first cell is tested**: a cliff whose first cell is off
/// the map or in another region than the site's is skipped whole,
/// however near its other cells; one whose first cell passes is
/// measured on every cell, another region's included.
#[test]
fn find_nearest_gates_a_cliff_on_its_first_cell() {
    let (mut s, ty) = sim();
    let site = s.footprint_centre(ty, CORNER);
    let mut off = vec![(-1, 0)];
    off.extend(cell_tiles(Cell::new(5, 5)));
    s.place_cliffs(&[off.clone(), cell_tiles(Cell::new(9, 9))]);
    assert_eq!(
        s.find_nearest_cliff(site).map(|(_, k)| k),
        Some(1),
        "off the map"
    );
    // With no region at the site the region gate is off, and the map's
    // edge is what refuses the cliff.
    let mut bare = Sim::new(Tuning::RON, World::new(16, 16), 2);
    bare.place_cliffs(&[off, cell_tiles(Cell::new(9, 9))]);
    assert_eq!(bare.world.region_of(site.cell()), None);
    assert_eq!(
        bare.find_nearest_cliff(site).map(|(_, k)| k),
        Some(1),
        "off, no region"
    );
    s.world
        .fill_region(Terrain::Land, Cell::new(5, 5), Cell::new(5, 5));
    s.place_cliffs(&[cell_tiles(Cell::new(5, 5)), cell_tiles(Cell::new(9, 9))]);
    assert_eq!(
        s.find_nearest_cliff(site).map(|(_, k)| k),
        Some(1),
        "another region"
    );
    let mut tail = cell_tiles(Cell::new(9, 9));
    tail.extend(cell_tiles(Cell::new(5, 5)));
    s.place_cliffs(&[tail]);
    assert_eq!(
        s.find_nearest_cliff(site),
        Some((768, 0)),
        "a later cell's region"
    );
}

/// An ally's cell counts as ours, in the count and the list alike.
#[test]
fn an_ally_s_cells_count() {
    let (mut s, ty) = sim();
    let mut tiles = cell_tiles(Cell::new(5, 5));
    tiles.extend(cell_tiles(Cell::new(6, 5)));
    s.place_cliffs(&[tiles]);
    s.world
        .set_owner(Cell::new(6, 5), Owner::Player(1), Owner::None);
    assert_eq!(s.cliff_gather_size(0, 0, None), 1);
    assert_eq!(s.gather_tcoords(ty, 0, CORNER).len(), 16);
    s.set_diplo(0, 1, 2);
    assert_eq!(s.cliff_gather_size(0, 0, None), 3);
    assert_eq!(s.gather_tcoords(ty, 0, CORNER).len(), 32);
}

/// **A cliff strictly nearer than the range takes the site**; a range
/// as near keeps it, and whichever wins answers only inside the reach.
#[test]
fn a_strictly_nearer_cliff_takes_the_mine() {
    let (mut s, ty) = sim();
    let site = s.footprint_centre(ty, CORNER);
    let range = |s: &mut Sim, c: Cell| {
        let t = MountainTemplate {
            tiles: vec![(0, 0)],
            solid: vec![(0, 0)],
        };
        s.place_mountains(&[(c, 0)], &[t]);
    };
    s.place_cliffs(&[cell_tiles(Cell::new(5, 5))]);
    assert_eq!(s.mine_arm(site), MineArm::Cliff(0), "no range");
    range(&mut s, Cell::new(6, 5));
    assert_eq!(s.mine_arm(site), MineArm::Cliff(0), "the range at 1536");
    range(&mut s, Cell::new(3, 5));
    assert_eq!(s.mine_arm(site), MineArm::Range, "a range as near");
    s.place_cliffs(&[cell_tiles(Cell::new(6, 5))]);
    assert_eq!(s.mine_arm(site), MineArm::Range, "a range nearer");
    range(&mut s, Cell::new(8, 5));
    assert_eq!(s.mine_arm(site), MineArm::None, "the cliff past the reach");
    s.place_cliffs(&[cell_tiles(Cell::new(5, 5))]);
    range(&mut s, Cell::new(10, 10));
    assert_eq!(s.mine_arm(site), MineArm::Cliff(0), "the range past it");
    s.terrain.mountains.clear();
    s.place_cliffs(&[cell_tiles(Cell::new(9, 5))]);
    assert_eq!(s.mine_arm(site), MineArm::None, "nothing in reach");
}

/// **`gather_size`**: the band by the cliff's whole tile count, scaled
/// by the share of its tiles in our cells, at least 1 — and with a
/// building's list, the share in that list too.
#[test]
fn gather_size_scales_the_band_by_the_share_that_is_ours() {
    let (mut s, ty) = sim();
    // 112 tiles over seven cells: the 100..210 band, 5.
    let cells: Vec<Cell> = (0..7).map(|k| Cell::new(5, 2 + k)).collect();
    let tiles: Vec<(i32, i32)> = cells.iter().flat_map(|&c| cell_tiles(c)).collect();
    s.place_cliffs(std::slice::from_ref(&tiles));
    assert_eq!(s.cliff_gather_size(0, 0, None), 5);
    assert_eq!(s.site_gather_count(ty, 0, CORNER, None), 5);
    // Four of the seven cells an enemy's: 5 · 48 / 112 = 2.
    for &c in &cells[..4] {
        s.world.set_owner(c, Owner::Player(1), Owner::None);
    }
    assert_eq!(s.cliff_gather_size(0, 0, None), 2);
    assert_eq!(
        s.cliff_gather_size(0, 1, None),
        5,
        "theirs and the unclaimed"
    );
    // Six: 5 · 16 / 112 = 0, raised to 1.
    for &c in &cells[4..6] {
        s.world.set_owner(c, Owner::Player(1), Owner::None);
    }
    assert_eq!(s.cliff_gather_size(0, 0, None), 1);
    // All seven: nothing counted, nothing raised.
    s.world.set_owner(cells[6], Owner::Player(1), Owner::None);
    assert_eq!(s.cliff_gather_size(0, 0, None), 0);
    assert_eq!(s.site_gather_count(ty, 0, CORNER, None), 0);
    // A building's list: only its tiles count.
    let list: Vec<Pos> = tiles[..56].iter().map(|&(x, y)| Pos::new(x, y)).collect();
    assert_eq!(s.cliff_gather_size(0, 1, Some(&list)), 2, "5 · 56 / 112");
    // 210 tiles is the next band: 6 at 14 cells less one tile.
    let big: Vec<(i32, i32)> = (0..14)
        .flat_map(|k| cell_tiles(Cell::new(10 + k % 2, 1 + k / 2)))
        .take(210)
        .collect();
    s.place_cliffs(&[big]);
    assert_eq!(s.cliff_gather_size(0, 0, None), 6);
}

/// **The walk over a cliff's tiles**: on the survey a tile already
/// gathered from refuses the site (−1) unless it is forest-surfaced or
/// in an enemy's cell; a standing building's own list is never walked.
#[test]
fn a_cliff_already_mined_is_taken_on_the_survey() {
    let (mut s, ty) = sim();
    let mut tiles = cell_tiles(Cell::new(5, 5));
    tiles.extend(cell_tiles(Cell::new(6, 5)));
    s.place_cliffs(&[tiles]);
    let free = s.site_gather_count(ty, 0, CORNER, None);
    assert_eq!(free, 3, "32 tiles: the tiny band");
    let taken = Pos::new(25, 22);
    s.world.set_tile_bits(taken, GATHERED_FROM);
    assert_eq!(s.site_gather_count(ty, 0, CORNER, None), -1);
    // Its cell an enemy's: passed, and the half that is ours pays.
    s.world
        .set_owner(Cell::new(6, 5), Owner::Player(1), Owner::None);
    assert_eq!(s.site_gather_count(ty, 0, CORNER, None), 1);
    s.world.set_owner(Cell::new(6, 5), Owner::None, Owner::None);
    // A forest surface: passed.
    s.world.set_tile_bits(taken, tile::SURFACE_FOREST);
    assert_eq!(s.site_gather_count(ty, 0, CORNER, None), free);
    s.world.clear_tile_bits(taken, tile::SURFACE_FOREST);
    // A building's own list is not walked.
    let list = s.cliff_gather_tcoords(0, 0);
    assert_eq!(
        s.gather_slots_for_list(ty, 0, CORNER, &list),
        3 * 31 / 32,
        "the list's 31 of 32"
    );
}

/// **The gather list**: the cliff's tiles in order, less the
/// forest-surfaced, the enemy's and the already gathered.
#[test]
fn the_list_is_the_cliff_s_tiles_less_three() {
    let (mut s, ty) = sim();
    let mut tiles = cell_tiles(Cell::new(5, 5));
    tiles.extend(cell_tiles(Cell::new(6, 5)));
    s.place_cliffs(std::slice::from_ref(&tiles));
    let all: Vec<Pos> = tiles.iter().map(|&(x, y)| Pos::new(x, y)).collect();
    assert_eq!(s.gather_tcoords(ty, 0, CORNER), all);
    s.world.set_tile_bits(all[0], tile::SURFACE_FOREST);
    s.world.set_tile_bits(all[1], GATHERED_FROM);
    s.world
        .set_owner(Cell::new(6, 5), Owner::Player(1), Owner::None);
    assert_eq!(s.gather_tcoords(ty, 0, CORNER), all[2..16].to_vec());
    assert_eq!(
        s.gather_tcoords(ty, 1, CORNER),
        all[2..].to_vec(),
        "theirs and the unclaimed"
    );
}
