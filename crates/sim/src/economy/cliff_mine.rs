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
//! one per queued cliff (`sim::cliffs`, [`crate::mapgen::cliffs::CliffsInit::mining`]):
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

use crate::economy::gather::{GATHERED_FROM, MOUNTAIN_GATHER};
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
        self.terrain.cliffs = mining.iter().map(|t| CliffMining::from_tiles(t)).collect();
    }

    /// **`CliffsData::find_nearest@008a8da0`**: `(distance, cliff index)` of
    /// the cliff cell nearest `centre`, in world units, or `None` when no
    /// cliff qualifies. A cliff is skipped when it has no cell, or when its
    /// first cell is off the map or, with a region at the site, in another
    /// region.
    pub fn find_nearest_cliff(&self, centre: Pos) -> Option<(i32, usize)> {
        let site_region = self.world.region_of(centre.cell());
        let mut best: Option<(i32, usize)> = None;
        for (k, c) in self.terrain.cliffs.iter().enumerate() {
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
        self.terrain.cliffs[k].tiles.iter().any(|&t| {
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
        let tiles = &self.terrain.cliffs[k].tiles;
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
        self.terrain.cliffs[k]
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
mod tests;
