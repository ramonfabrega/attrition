//! **`World::analyze_map@006b58b0`**: the setup's census of the map
//! (`docs/MAPGEN.md` §36, item 4147).
//!
//! `Setup::build_game@005ac190` calls it once, after every empire is built
//! (the `setup.cpp` 1148 word) and just before `compute_all_territory`; the
//! export and the listing name no other caller. It writes ten tallies of
//! `WorldData` (`+0x50..+0x74`), the region bits `4` and `8`, and finishes
//! `sea_map`. Its other writers are `World::init` (the two region counts),
//! `World::wipe` and `World::close` (all ten to 0); no store in a running
//! game reaches them, so the values are the setup's to the end of a game.
//!
//! **The regions**, numbers 1 to 63 (`Region` records from `+0x88`, one
//! stride in; the loop's bound is `0xe4255a`, 63 `reg_cities` shorts on).
//! A region in which an active leader holds a city takes `REGION_PLAYER`
//! and counts in `player_reg`; one that holds none and is at least
//! `scale_by_area(60)` cells (`max(1, (size·60 + std²/2) / std²)`, `std`
//! `rules.xml`'s Standard map size, `map_sizes` record 3's `+0x144` at
//! `0xe7fca8`) takes [`REGION_RESOURCE`] and counts in `resource_reg`.
//!
//! **The cells**, every one. A cell that is not `HALFLAND` and whose
//! `land` is `SANDY` or `OCEAN` (1 or 2) is the sea's: its `0x1` counts in
//! `sea_resources` and nothing else of it is read. Every other cell counts
//! its `0x1` in `land_resources`, `GOODY` in `goodies`, `FOREST` in
//! `forest_size`, `OIL` in `total_oil`, `ROCK` in `rock_size`, and its
//! centre tile `(4x + 2, 4y + 2)` in `mountain_size` when that tile's
//! `mask & 3` is 2.
//!
//! **`total_metal`** is `MountainsData::total_gather_size@0089c110`: every
//! placed mountain's `MountainRangeData::gather_size@0089d170` asked with
//! no leader (−1) and no list, which makes every counted cell usable and so
//! returns the rung its template's tile count picks
//! ([`crate::economy::gather::MOUNTAIN_GATHER`]) unscaled.

use crate::economy::gather::MOUNTAIN_GATHER;
use crate::world::{Cell, Pos, REGION_PLAYER, cell};
use crate::{Player, Sim};

/// `Region.flags`' `0x8`: a land region no leader holds a city in at setup
/// and at least `scale_by_area(60)` cells, which `World::analyze_map` marks
/// and counts in `resource_reg`. Nothing in this crate reads it.
pub const REGION_RESOURCE: i32 = 8;

/// `WorldData` `+0x50..+0x74`, the ten tallies `World::analyze_map` writes
/// at setup (`docs/MAPGEN.md` §36), in the record's order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapTallies {
    pub player_reg: i32,
    pub resource_reg: i32,
    pub forest_size: i32,
    pub mountain_size: i32,
    pub rock_size: i32,
    pub total_metal: i32,
    pub total_oil: i32,
    pub goodies: i32,
    pub land_resources: i32,
    pub sea_resources: i32,
}

impl MapTallies {
    /// The ten, in `WorldData`'s order (`HEADER_FIELDS[18..28]`).
    pub const fn words(&self) -> [i32; 10] {
        [
            self.player_reg,
            self.resource_reg,
            self.forest_size,
            self.mountain_size,
            self.rock_size,
            self.total_metal,
            self.total_oil,
            self.goodies,
            self.land_resources,
            self.sea_resources,
        ]
    }
}

/// What one `World::analyze_map` finds: the tallies, the regions it marks,
/// and the sea class it leaves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapAnalysis {
    pub tallies: MapTallies,
    /// The regions (this crate's index) given [`REGION_PLAYER`].
    pub player_regions: Vec<u16>,
    /// The regions given [`REGION_RESOURCE`].
    pub resource_regions: Vec<u16>,
    /// `sea_map` as the function leaves it.
    pub sea_map: i32,
}

impl Sim {
    /// `World::analyze_map`'s counts over the state as it stands, written
    /// nowhere ([`Sim::analyze_map`] writes them). `standard` is `rules.xml`'s
    /// Standard map size.
    pub fn map_analysis(&self, standard: i32) -> MapAnalysis {
        let w = &self.world;
        let size = w.width() * w.height();
        let std2 = standard * standard;
        let big = if std2 > 0 {
            ((std2 / 2 + size * 60) / std2).max(1)
        } else {
            1
        };
        let players = self.territory.players.len();
        let mut t = MapTallies::default();
        let (mut player_regions, mut resource_regions) = (Vec::new(), Vec::new());
        let (mut player_land, mut resource_land, mut land_size) = (0, 0, 0);
        for (r, _) in w.regions() {
            if !(1..0x40).contains(&w.region_number(r)) {
                continue;
            }
            let n = w.region_size(r);
            land_size += n;
            if (0..players).any(|p| self.leader_reg_cities(p as Player, r) != 0) {
                t.player_reg += 1;
                player_land += n;
                player_regions.push(r);
            } else if big <= n {
                t.resource_reg += 1;
                resource_land += n;
                resource_regions.push(r);
            }
        }
        for y in 0..w.height() {
            for x in 0..w.width() {
                let d = w.cell_data(Cell::new(x, y));
                let f = d.flags;
                if f & cell::HALFLAND == 0 && (d.land == 1 || d.land == 2) {
                    t.sea_resources += i32::from(f & 1 != 0);
                    continue;
                }
                t.land_resources += i32::from(f & 1 != 0);
                t.goodies += i32::from(f & cell::GOODY != 0);
                t.forest_size += i32::from(f & cell::FOREST != 0);
                t.mountain_size += i32::from(w.tile_mask(Pos::new(4 * x + 2, 4 * y + 2)) & 3 == 2);
                t.total_oil += i32::from(f & cell::OIL != 0);
                t.rock_size += i32::from(f & cell::ROCK != 0);
            }
        }
        t.total_metal = self
            .terrain
            .mountains
            .iter()
            .map(|m| {
                let tiles = i32::try_from(m.tiles.len()).unwrap_or(i32::MAX);
                MOUNTAIN_GATHER
                    .iter()
                    .find(|&&(s, _)| tiles < s)
                    .map_or(10, |&(_, g)| g)
            })
            .sum();
        let s = w.sea_map();
        let sea_map = if !(0..=4).contains(&s) {
            if t.player_reg > 1 {
                3 + i32::from(player_land * 2 <= resource_land)
            } else if t.resource_reg != 0 {
                2
            } else {
                i32::from(land_size < size * 4 / 5)
            }
        } else if s == 3 && t.player_reg > 1 && player_land * 2 <= resource_land {
            4
        } else {
            s
        };
        MapAnalysis {
            tallies: t,
            player_regions,
            resource_regions,
            sea_map,
        }
    }

    /// **`World::analyze_map`, run where `Setup::build_game` runs it**: the
    /// tallies kept on the world's record, the two region bits set and the
    /// sea class finished.
    pub fn analyze_map(&mut self, standard: i32) -> MapTallies {
        let a = self.map_analysis(standard);
        for &r in &a.player_regions {
            self.world.or_region_flags(r, REGION_PLAYER);
        }
        for &r in &a.resource_regions {
            self.world.or_region_flags(r, REGION_RESOURCE);
        }
        self.world.set_sea_map(a.sea_map);
        self.world.set_tallies(a.tallies);
        a.tallies
    }
}
