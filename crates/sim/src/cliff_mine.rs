//! **The cliff arm of a mine** (`docs/ECONOMY.md`, "The cliff arm";
//! `docs/AI.md` §400): `BuildTypeData::calc_gather@00639e40`'s mine arm and
//! `BuildTypeData::find_gather_tcoords@0063bdc0`'s metal branch, where a
//! scary cliff nearer than any mountain range wins.
//!
//! Three pieces, each the original's:
//!
//! 1. **`CliffsData::find_nearest@008a8da0`** ([`Sim::find_nearest_cliff`]):
//!    every cliff of `cliff_mining_data` with a cell, skipped when its
//!    **first** cell is off the map or in another region than the site's,
//!    then each of its cells (`wcoords`) measured by `vector_dist` from the
//!    footprint's centre to the cell's centre, `c · 0x300 + 0x180`; a cell
//!    replaces the best only when strictly nearer, so a tie goes to the
//!    first cliff. The answer is the cliff's index, `MiningList::cliff`.
//! 2. **The `is_gathered_from` walk** ([`Sim::cliff_taken`]): on a survey
//!    (no building's own list), `calc_gather` walks the cliff's tiles
//!    (`tcoords`) in order, passing a forest-surfaced tile and one in an
//!    enemy's cell, and answers **−1** at the first carrying `0x1000`
//!    (`cliff = -2`, `count = -1`) — the refusal `blocked_location` reads as
//!    `MountainTaken`. Unlike the range's walk it has no mountain test.
//! 3. **`CliffsData::gather_size@008a8f80`** ([`Sim::cliff_gather_size`]):
//!    the cliff's tiles whose cell is unclaimed, ours or an ally's (and, with
//!    a building's list, in that list) are counted, and the count scales the
//!    band the cliff's **whole** tile count falls in — `MTN_*_SIZE` against
//!    `MTN_*_GATHER`, the range's own constants ([`MOUNTAIN_GATHER`]):
//!    `band · counted / total`, at least 1 when anything counted.
//!
//! The cliffs themselves are `Cliffs::init@008a8020`'s `cliff_mining_data`,
//! one per queued cliff (`sim::cliffs`, [`crate::cliffs::CliffsInit::mining`]):
//! its tiles are the marked tiles of each cell under its index that counts
//! any, cells y outer, and its cells are those cells in the same order
//! ([`CliffMining::from_tiles`]). Nothing a dump prints carries them, so a
//! [`Sim`] has them only when a caller lays them ([`Sim::place_cliffs`]).
//!
//! # What is not established
//!
//! - **The count.** No dump prints `gather_max` on the frame a mine is
//!   placed; run1516's frame 100 settles the cliff, the list and its length,
//!   not `gather_size`'s answer, which is a reading (`docs/AI.md` §400).
//! - **The nation and wonder additions** (`german_miners`, `taj_farms`,
//!   `kremlin_farms`) are not modelled here, as on the range's arm.

use crate::gather::{GATHERED_FROM, MOUNTAIN_GATHER};
use crate::world::{Cell, UNITS_PER_CELL, World, tile, vector_dist};
use crate::{Player, Pos, Sim};

/// One cliff's `CliffMiningData`: its cells (`wcoords`, +0x0) and its tiles
/// (`tcoords`, +0x1c), in `Cliffs::init`'s order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliffMining {
    /// `wcoords`: what `find_nearest` measures to.
    pub cells: Vec<Cell>,
    /// `tcoords`: what the walk, `gather_size` and the gather list read.
    pub tiles: Vec<Pos>,
}

impl CliffMining {
    /// A cliff from its mining tiles as `Cliffs::init` appends them: each
    /// cell's tiles together, so its cells are the tiles' cells in first-seen
    /// order.
    pub fn from_tiles(tiles: &[(i32, i32)]) -> Self {
        let tiles: Vec<Pos> = tiles.iter().map(|&(x, y)| Pos::new(x, y)).collect();
        let mut cells: Vec<Cell> = Vec::new();
        for &t in &tiles {
            let c = World::cell_of_tile(t);
            if cells.last() != Some(&c) {
                cells.push(c);
            }
        }
        Self { cells, tiles }
    }
}

impl Sim {
    /// Lays `Cliffs::init`'s mining data down, by queue index — the index a
    /// dump's `MiningList::cliff` prints.
    pub fn place_cliffs(&mut self, mining: &[Vec<(i32, i32)>]) {
        self.cliffs = mining.iter().map(|t| CliffMining::from_tiles(t)).collect();
    }

    /// **`CliffsData::find_nearest@008a8da0`**: `(distance, cliff index)` of
    /// the cliff cell nearest `centre`, in world units, or `None` when no
    /// cliff qualifies. A cliff is skipped when it has no cell, or when its
    /// first cell is off the map or, with a region at the site, in another
    /// region.
    pub fn find_nearest_cliff(&self, centre: Pos) -> Option<(i32, usize)> {
        let site_region = self.world.region_of(centre.cell());
        let mut best: Option<(i32, usize)> = None;
        for (k, c) in self.cliffs.iter().enumerate() {
            let Some(&first) = c.cells.first() else {
                continue;
            };
            if !self.world.contains(first) {
                continue;
            }
            if let Some(a) = site_region
                && self.world.region_of(first) != Some(a)
            {
                continue;
            }
            for &cell in &c.cells {
                let d = vector_dist(
                    centre.x - (cell.x * UNITS_PER_CELL + UNITS_PER_CELL / 2),
                    centre.y - (cell.y * UNITS_PER_CELL + UNITS_PER_CELL / 2),
                );
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, k));
                }
            }
        }
        best
    }

    /// A tile's cell is unclaimed, `who`'s, or an ally's — the owner test
    /// the cliff arm makes at every step (`LeaderData::is_ally` inlined in
    /// `gather_size`).
    fn cliff_tile_ours(&self, t: Pos, who: Player) -> bool {
        self.world
            .owner(World::cell_of_tile(t))
            .player()
            .is_none_or(|o| o == who || self.is_ally(who, o))
    }

    /// **`calc_gather`'s walk over a cliff's tiles**, the survey only: true
    /// at the first tile that is not forest-surfaced, not in an enemy's cell,
    /// and already `is_gathered_from` — the arm's −1.
    pub fn cliff_taken(&self, k: usize, who: Player) -> bool {
        self.cliffs[k].tiles.iter().any(|&t| {
            let m = self.world.tile_mask(t);
            m & tile::SURFACE != tile::SURFACE_FOREST
                && self.cliff_tile_ours(t, who)
                && m & GATHERED_FROM != 0
        })
    }

    /// **`CliffsData::gather_size@008a8f80`**: the cliff's band, by its whole
    /// tile count, scaled by the share of its tiles that are ours (and in
    /// `list`, when one is handed), at least 1 when any is.
    pub fn cliff_gather_size(&self, k: usize, who: Player, list: Option<&[Pos]>) -> i32 {
        let tiles = &self.cliffs[k].tiles;
        let total = i32::try_from(tiles.len()).unwrap_or(i32::MAX);
        let counted = tiles
            .iter()
            .filter(|&&t| self.cliff_tile_ours(t, who) && list.is_none_or(|l| l.contains(&t)))
            .count() as i32;
        if total == 0 {
            return 0;
        }
        let band = MOUNTAIN_GATHER
            .iter()
            .find(|&&(size, _)| total < size)
            .map_or(10, |&(_, gather)| gather);
        let n = band * counted / total;
        if counted != 0 && n < 1 { 1 } else { n }
    }

    /// **`find_gather_tcoords`' cliff branch**: the cliff's tiles in order,
    /// each kept when it is not forest-surfaced (`mask & 0x30 != 0x30`), its
    /// cell is unclaimed, ours or an ally's, and it is not already
    /// `0x1000`.
    pub fn cliff_gather_tcoords(&self, k: usize, who: Player) -> Vec<Pos> {
        self.cliffs[k]
            .tiles
            .iter()
            .copied()
            .filter(|&t| {
                let m = self.world.tile_mask(t);
                m & tile::SURFACE != tile::SURFACE_FOREST
                    && self.cliff_tile_ours(t, who)
                    && m & GATHERED_FROM == 0
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BuildType, Ident, flags};
    use crate::gather::{MineArm, MountainTemplate};
    use crate::tuning::Tuning;
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
                (0..TILES_PER_CELL)
                    .map(move |u| (c.x * TILES_PER_CELL + u, c.y * TILES_PER_CELL + v))
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
        s.mountains.clear();
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
}
