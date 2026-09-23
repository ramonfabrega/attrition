//! `BuildData::gather_max` — how many citizens a gather building has room
//! for, surveyed from the map.
//!
//! This is `docs/ECONOMY.md`'s open item, the one number the income pipeline
//! took as an input. A farm is one citizen and always was; a woodcutter's
//! camp, a mine and a university are not, and the count comes out of
//! `BuildTypeData::calc_gather@00639e40` — a thousand lines whose *rate*
//! half `economy.rs` already models and whose *count* half is here.
//!
//! # The spine
//!
//! `BuildTypeData::max_gatherers@0063c430` is the only caller that wants the
//! count, and it wants nothing else: it calls `calc_gather` with
//! `(…, 0, city, who, 0, corner_tx, corner_ty, 1, 1, 0, mining_list)`, takes
//! the count out-parameter and clamps it at zero. Its three callers are
//! `Build::init@00629740` (line 275), `Build::find_gather_tiles@00623350`
//! (line 88) and `Build::update_max_gatherers@00623310`. `find_gather_tiles`
//! matters: it fills `BuildData::gather_from` first and *then* recomputes the
//! count, so an existing building's survey reads its own tile list rather
//! than the map.
//!
//! `max_gatherers` answers `1` outright for a flat type (`is_flat`, vtable
//! `+0x94`, `build_flags & 0x10000000`), and `BuildType::init_final_flags`
//! makes exactly the FARM, OILWELL and OILPLATFORM lineages flat — **so an
//! oil well is one citizen, like a farm**, and the only non-flat gather
//! types the shipped data has are the woodcutter's camp, the mine and the
//! university.
//!
//! Inside `calc_gather` the count is decided in one of four places:
//!
//! | type | line | count |
//! | --- | --- | --- |
//! | flat (farm, oil well, oil platform) | 904 | `max_flat_gatherers` = **1** |
//! | knowledge (university) | 868 | `max_knowledge_gatherers` = **7** |
//! | metal (mine) | 683 / 800 | `CliffsData::gather_size` / `MountainRangeData::gather_size` |
//! | anything else (the camp) | 472 | the circle walk below |
//!
//! # The circle walk (`calc_gather` lines 214–556)
//!
//! `WCoord` is a **cell** coordinate — `WCoord::operator_TCoord@004613b0` is
//! `cell * 4 + 2`, the cell's centre tile — and that is what makes the walk
//! legible. In order:
//!
//! 1. `radius = gather_radius@0063bc60`: `WOODCUTTER_RADIUS` (8 tiles) for a
//!    timber type, `MINE_RADIUS` (6) otherwise, `0` for a knowledge type.
//! 2. `corner_tile@006364c0` turns the footprint's corner tile into the
//!    footprint's **centre** in world units, `(size + 2·corner) · 96`. The
//!    walk anchors on that point's cell; the distance test uses its tile.
//! 3. For every entry of `circle_x`/`circle_y` inside ring
//!    `(radius + 3) / 4` — the same octagonal spiral `ai_place` already
//!    rebuilt, in **cells** — take `cell = anchor_cell + offset` and its
//!    centre tile `sample`.
//! 4. Skip the cell unless `WorldData::valid`, unless
//!    `vector_dist(anchor_tile, sample) <= radius`, and unless the cell's
//!    owner is unclaimed, ours or an ally's.
//! 5. Qualify the cell. With a `MiningList` in hand (an existing building)
//!    that is `find(list, sample)` — the sample tile must be one of the
//!    building's own tiles. Without one it is the survey: skip the cell if
//!    `sample` is already `is_gathered_from` (`TData.mask & 0x1000`) — and
//!    remember that it was, line 409 — then walk the cell's sixteen tiles
//!    adding the trees (or, for a non-timber type, the mountains) to a
//!    scratch list, and qualify the cell on `is_tree_at(sample)` /
//!    `is_mountain_at(sample)`.
//! 6. A qualifying cell adds `16 × LandData::num_make[good]` to the
//!    accumulator — **sixteen times, unconditionally** (the inner loop at
//!    line 386 has no test in it; the tile coordinates it computes are dead).
//!    That is why a cell with nine forest tiles counts exactly as much as a
//!    cell with sixteen.
//! 7. `accum += (accum · rivers) >> 4`, where `rivers` is the number of river
//!    tiles under the footprint (lines 428–450).
//! 8. `slots = (accum + 8) >> 4` — round to nearest sixteenth. With
//!    `num_make = 1` that is simply **the number of qualifying cells**.
//! 9. If the count came out zero *and* something was skipped as already
//!    gathered from, the answer is `-1` (line 474), which `max_gatherers`
//!    clamps to zero.
//! 10. `slots = min(slots, 2 × total_gather_access@00636500)` — twice the
//!     number of list tiles that have gather access and stand more than three
//!     tiles from the footprint centre.
//!
//! # The oracle
//!
//! `gamelog-run9-world6.txt` frame 1 carries the map and both players' camps.
//! Player 0's camp stands at tile `(22, 149)` with 82 tiles in its
//! `gather_from`; those 82 tiles fall in **seven** cells, and the leader
//! record says `gather_slots[1] = 7`. Player 1's camp at `(211, 93)` has 61
//! tiles in **five** cells and `gather_slots[1] = 5`. Sixteen-per-cell
//! reproduces both; one-per-tile (`(82 + 8) / 16 = 5`) reproduces neither.
//! Every cell of both sets is inside ring 2 and inside
//! `vector_dist ≤ 8` of the anchor tile — the furthest, `(30, 146)` against
//! `(22, 149)`, is `8 + 3·3 / 16 = 8` exactly, which is also the check that
//! settles `vector_dist` as the octagonal metric rather than `hi + lo/2`.
//! The dump's own tile masks confirm the `0x1000` bit: it is set on exactly
//! 143 tiles of the map, and 143 is 82 + 61.
//!
//! # What is not established
//!
//! - **`LandData::num_make`.** The per-cell amount is a table the sim does
//!   not carry ([`Sim::land_amount`], a seam answering 1). Run9's cells all
//!   answer 1; nothing here proves another land does.
//! - **The mountain ranges, without the generator's placements.**
//!   `MountainRangeData::gather_size` walks a range the map generator built.
//!   With the placements a `DUMP_ALL` head prints ([`Sim::mountains`],
//!   `docs/AI.md` §60) it is that range. Without them
//!   [`Sim::mountain_range`] rebuilds one as the connected component of
//!   mountain cells. The *arithmetic* on top of it is the original's.
//! - **The nation and wonder layer.** `french_woodies`, `taj_farms`,
//!   `kremlin_farms`, `german_miners` and the Iroquois food bonus all add to
//!   the count and are not modelled.
//! - **`TData.mask & 0x8000`**, which `has_gather_access` requires, is
//!   unnamed here and in `world::tile`. It is set on 1,711 tiles of run9's
//!   map — enough that the access cap never binds there.

use std::collections::BTreeSet;

use crate::ai_place::{circle, gather_good};
use crate::build::flags;
use crate::economy::Resource;
use crate::world::{
    Cell, TILES_PER_CELL, UNITS_PER_CELL, UNITS_PER_TILE, World, tile, vector_dist,
};
use crate::{Player, Pos, Sim};

/// `WOODCUTTER_RADIUS`, in tiles (`rules.xml`, `woodcutter_radius 8` in the
/// dump's `CONSTANTS`). Not in [`crate::tuning::Tuning`] yet.
pub const WOODCUTTER_RADIUS: i32 = 8;
/// `MINE_RADIUS`, in tiles.
pub const MINE_RADIUS: i32 = 6;

/// `BuildTypeData::max_flat_gatherers@006365f0` — a constant.
pub const MAX_FLAT_GATHERERS: i32 = 1;
/// `BuildTypeData::max_knowledge_gatherers@006365e0` — a constant.
pub const MAX_KNOWLEDGE_GATHERERS: i32 = 7;

/// `MTN_*_SIZE` against `MTN_*_GATHER`, in the order `gather_size` tests
/// them: a range smaller than the size gets that many citizens.
pub const MOUNTAIN_GATHER: [(i32, i32); 5] =
    [(100, 3), (210, 5), (275, 6), (400, 8), (i32::MAX, 10)];

/// `TData.mask & 0x1000` — some building already gathers from this tile
/// (`WorldData::is_gathered_from@00472ac0`). Named with the rest of the
/// tile bits; the site pass below is its only writer, and
/// [`crate::world::World::gather_at`] the other reader.
pub use crate::world::tile::GATHERED_FROM;
/// `TData.mask & 0x8000` — the bit `WorldData::has_gather_access@006b4e50`
/// requires of a tile before it counts towards the access cap.
pub const GATHERABLE: u16 = 0x8000;

/// The mining list's shuffle — `Build::find_gather_tiles@00623350+0x10a`,
/// one `Random::get` an iteration over `4 × length` of them. It is the one
/// draw site of a gather building's creation, and on East Indies it is 192
/// draws on the frame the AI's second camp goes up.
pub const SITE_SHUFFLE: &str = "Build::find_gather_tiles+0x10a";

/// `orthog_x`/`orthog_y` entries 1–4, the four orthogonal neighbours.
const ORTHOG: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Half the width of the ring `total_gather_access` ignores: `0x240` world
/// units, three tiles.
const ACCESS_NEAR: i32 = 0x240;

/// What a gather building's slot count is being computed for: an existing
/// building, which reads its own `MiningList`, or a prospective site, which
/// surveys the map (`max_gatherers`' `o < 0 || who < 0` branch).
enum Source<'a> {
    /// `BuildData::gather_from`.
    List(&'a [Pos]),
    /// `param_13 == nullptr` — survey the tiles.
    Survey,
}

/// The running state of one circle walk.
#[derive(Default)]
struct Walk {
    /// The accumulator, in sixteenths of a citizen.
    accum: i32,
    /// `local_e0` — a cell was passed over because it was already gathered
    /// from, so a zero count means "taken", not "empty".
    taken: bool,
    /// The scratch `MiningList` the survey fills, for the access cap.
    found: Vec<Pos>,
}

impl Sim {
    /// `BuildTypeData::max_gatherers` for a placed building: the number of
    /// citizens that can work it, at its current position and from its own
    /// [`crate::Building::gather_from`].
    ///
    /// Zero for a building that is not a gather type at all.
    pub fn max_gatherers(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        let Some(ty) = bd.ty else { return 0 };
        let corner = self.tile_corner(ty, bd.pos);
        self.gather_slots(ty, bd.owner, corner, &Source::List(&bd.gather_from))
    }

    /// The same for a prospective site — `max_gatherers` with `o < 0`, which
    /// surveys the map instead of reading a list. `corner` is the footprint's
    /// corner tile, as [`Sim::tile_corner`] gives it.
    pub fn max_gatherers_at(&self, ty: usize, who: Player, corner: Pos) -> i32 {
        self.gather_slots(ty, who, corner, &Source::Survey)
    }

    /// Test support: the trees a woodcutter's camp needs before
    /// `blocked_location`'s gather tail will let one stand at `at`
    /// (`docs/CITIES.md` §2.6.7). The bare test worlds have none, and a
    /// camp with nothing to gather is `Blocked::NoForest`.
    ///
    /// The survey qualifies a **cell** on its centre tile, so one gatherable
    /// forest tile per neighbouring cell is the whole of what it wants: the
    /// eight cells around the camp's own, each centre tile that is neither
    /// under the footprint nor off the map. That leaves the footprint clear,
    /// which matters because a `BLOCKED` forest tile under a building is a
    /// different refusal.
    #[cfg(test)]
    pub(crate) fn plant_camp_forest(&mut self, at: Pos) {
        let c = at.cell();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                let t = Cell::new(c.x + dx, c.y + dy).centre_tile();
                if !self.world.tile_in_bounds(t) {
                    continue;
                }
                self.world
                    .set_tile_field(t, tile::SURFACE, tile::SURFACE_FOREST);
                self.world.set_tile_bits(t, GATHERABLE);
            }
        }
    }

    /// `LandData::num_make[good]` for the land under a tile — how much of a
    /// good one cell of that land is worth.
    ///
    /// **A seam.** The sim carries no `Lands` table; every land run9 puts a
    /// camp on answers 1, which is what makes its seven cells seven slots.
    #[allow(clippy::unused_self)]
    pub fn land_amount(&self, _t: Pos, _good: usize) -> i32 {
        1
    }

    /// `calc_gather`'s count out-parameter as
    /// [`Sim::blocked_location`](crate::Sim::blocked_location) reads it —
    /// **unclamped**, so the `−1` of step 9 (nothing found, and something
    /// was skipped as already gathered from) is still distinguishable from
    /// a plain zero. `blocked_location` reads the sign and
    /// [`Sim::max_gatherers`] does not, which is the whole reason the two
    /// forms are separate.
    ///
    /// `exclude` is `blocked_site`'s `exclude_o`: with one, the walk reads
    /// that building's own `MiningList` instead of surveying the map
    /// (`blocked_location@006375b0:693`, which nulls the list pointer when
    /// either the object or the player is negative).
    pub(crate) fn site_gather_count(
        &self,
        ty: usize,
        who: Player,
        corner: Pos,
        exclude: Option<usize>,
    ) -> i32 {
        match exclude.and_then(|b| self.buildings.get(b)) {
            Some(bd) => self.gather_slots_raw(ty, who, corner, &Source::List(&bd.gather_from)),
            None => self.gather_slots_raw(ty, who, corner, &Source::Survey),
        }
    }

    /// The count out-parameter of `calc_gather`, clamped at zero the way
    /// `max_gatherers` clamps it.
    fn gather_slots(&self, ty: usize, who: Player, corner: Pos, from: &Source) -> i32 {
        self.gather_slots_raw(ty, who, corner, from).max(0)
    }

    /// The same count before the clamp: [`Sim::site_gather_count`] and
    /// [`Sim::max_gatherers`] are its two readers.
    fn gather_slots_raw(&self, ty: usize, who: Player, corner: Pos, from: &Source) -> i32 {
        let t = &self.build_types[ty];
        if t.has(flags::FLAT) {
            return MAX_FLAT_GATHERERS;
        }
        let Some(good) = gather_good(t.ident) else {
            return 0;
        };
        if good == Resource::Knowledge.index() {
            return MAX_KNOWLEDGE_GATHERERS;
        }
        let (anchor_tile, anchor_cell) = self.gather_anchor(ty, corner);
        if good == Resource::Metal.index() {
            return self.mine_slots(self.footprint_centre(ty, corner), who, from);
        }
        let radius = if good == Resource::Timber.index() {
            WOODCUTTER_RADIUS
        } else {
            MINE_RADIUS
        };
        let timber = good == Resource::Timber.index();

        let mut walk = Walk::default();
        let circle = circle();
        let ring = ((radius + 3) / 4).clamp(0, 0x40) as usize;
        for i in 0..circle.radius[ring] {
            let cell = Cell::new(anchor_cell.x + circle.x[i], anchor_cell.y + circle.y[i]);
            if !self.world.contains(cell) {
                continue;
            }
            let sample = cell.centre_tile();
            if vector_dist(sample.x - anchor_tile.x, sample.y - anchor_tile.y) > radius {
                continue;
            }
            if let Some(o) = self.world.owner(cell).player()
                && o != who
                && !self.is_ally(who, o)
            {
                continue;
            }
            if !self.cell_qualifies(cell, sample, timber, from, &mut walk) {
                continue;
            }
            // The land's four `(make, num_make)` slots, of which only the one
            // this type gathers pays. Sixteen times, once per tile of the
            // cell, with no test on the tile.
            let amount = self.land_amount(sample, good);
            if amount != 0 {
                walk.accum += amount * (TILES_PER_CELL * TILES_PER_CELL);
            }
        }

        let rivers = self.footprint_rivers(ty, corner);
        walk.accum += (walk.accum * rivers) >> 4;
        let mut slots = (walk.accum + 8) >> 4;
        if slots == 0 && walk.taken {
            return -1;
        }
        // The nation and wonder additions (`french_woodies`, `taj_farms`,
        // `kremlin_farms`) belong here and are not modelled.
        let list: &[Pos] = match from {
            Source::List(l) => l,
            Source::Survey => &walk.found,
        };
        let cap = 2 * self.total_gather_access(list, ty, corner, who);
        slots = slots.min(cap);
        slots.max(0)
    }

    /// `BuildTypeData::find_gather_tcoords@0063bdc0`, the timber branch —
    /// the tiles a new camp's `MiningList` is filled with, in the circle
    /// walk's own order and before the shuffle.
    ///
    /// It is `calc_gather`'s survey walk again — the same ring, the same
    /// `vector_dist`, the same owner test and the same `0x1000` skip — and
    /// what it does inside a qualifying cell is add the cell's **tree**
    /// tiles. The decompiler prints the inner loop with only the `0x1000`
    /// test in it, which would make a cell worth all sixteen of its tiles;
    /// the record says otherwise, and says it twice. run39's two camps each
    /// list **73** tiles across **six** cells — 16, 12, 12, 12, 12, 9 — and
    /// the same six cells hold exactly 16, 12, 12, 12, 12 and 9 forest
    /// tiles, every listed tile among them. Sixteen-a-cell would be 96.
    /// (`docs/ECONOMY.md`, "The gather list".)
    ///
    /// The **metal** branch is not this walk at all: `0x1a3` takes the tiles
    /// of the nearest mountain range whole ([`Sim::mountain_range`]),
    /// keeping every one that is not `SURFACE_FOREST`, stands on nobody
    /// else's territory and is not already gathered from — the three tests
    /// the decompiled loop makes, in that order. run97's frame 8383 is the
    /// value diff: Great Lakes' first mine lists **207** tiles, the range
    /// here holds 244, and the 37 it drops are exactly the forest-surfaced
    /// ones (`docs/ECONOMY.md`, "The mine's range").
    ///
    /// **The cliff arm is not modelled.** `CliffsData::find_nearest` runs
    /// beside the mountain one and wins when a scary cliff is nearer; no
    /// capture on either map has a cliff tile at all, so a mine sited on one
    /// still gets nothing here. The falsifier is a map with `OBJECT_CLIFF`
    /// tiles and a mine beside them.
    pub fn gather_tcoords(&self, ty: usize, who: Player, corner: Pos) -> Vec<Pos> {
        let t = &self.build_types[ty];
        let Some(good) = gather_good(t.ident) else {
            return Vec::new();
        };
        if good == Resource::Metal.index() {
            let Some(range) = self.mountain_range(self.footprint_centre(ty, corner)) else {
                return Vec::new();
            };
            return range
                .tiles
                .into_iter()
                .filter(|&p| {
                    let m = self.world.tile_mask(p);
                    m & tile::SURFACE != tile::SURFACE_FOREST
                        && m & GATHERED_FROM == 0
                        && self
                            .world
                            .owner(World::cell_of_tile(p))
                            .player()
                            .is_none_or(|o| o == who || self.is_ally(who, o))
                })
                .collect();
        }
        if good != Resource::Timber.index() {
            return Vec::new();
        }
        let radius = WOODCUTTER_RADIUS;
        let (anchor_tile, anchor_cell) = self.gather_anchor(ty, corner);
        let circle = circle();
        let ring = ((radius + 3) / 4).clamp(0, 0x40) as usize;
        let mut out = Vec::new();
        for i in 0..circle.radius[ring] {
            let cell = Cell::new(anchor_cell.x + circle.x[i], anchor_cell.y + circle.y[i]);
            if !self.world.contains(cell) {
                continue;
            }
            let sample = cell.centre_tile();
            if vector_dist(sample.x - anchor_tile.x, sample.y - anchor_tile.y) > radius {
                continue;
            }
            if let Some(o) = self.world.owner(cell).player()
                && o != who
                && !self.is_ally(who, o)
            {
                continue;
            }
            // The cell's own centre tile carrying the bit skips the whole
            // cell; a tile inside one that does not is skipped on its own.
            if self.world.tile_mask(sample) & GATHERED_FROM != 0 {
                continue;
            }
            let base = Pos::new(cell.x * TILES_PER_CELL, cell.y * TILES_PER_CELL);
            for k in 0..TILES_PER_CELL * TILES_PER_CELL {
                let p = Pos::new(base.x + (k % TILES_PER_CELL), base.y + (k / TILES_PER_CELL));
                if self.world.tile_mask(p) & GATHERED_FROM == 0 && self.gather_tile_kind(p, true) {
                    out.push(p);
                }
            }
        }
        out
    }

    /// `Build::find_gather_tiles@00623350` — a non-flat, non-university
    /// gather building's list, taken at **placement**.
    ///
    /// Four steps, in this order, and the third is the one that costs the
    /// stream: fill the list from [`Sim::gather_tcoords`]; mark every tile
    /// of it `0x1000`, so the next camp cannot take the same ground;
    /// **shuffle**, if the list grew, by `4 × length` rounds of "draw an
    /// index, move that entry to the back"; then recompute `gather_max`
    /// from the list that is now there.
    ///
    /// The shuffle is why this is a sim-visible event rather than
    /// bookkeeping: each round is `Random::get(game_random, 0, 0xffff) %
    /// length` off the sync stream, and a 48-tile list therefore spends
    /// **192** draws on one frame. `Unit::do_non_flat_gather` ranks tiles by
    /// `i >> 2`, so the order the shuffle leaves is the order citizens work
    /// the ground in (`docs/ORDERS.md` §6.1).
    ///
    /// The original removes the picked entry **by value**; the list holds
    /// each tile once — a cell is walked once and cells do not overlap — so
    /// removing by index is the same operation.
    pub(crate) fn find_gather_tiles(&mut self, b: usize) {
        let (Some(ty), who, pos) = ({
            let bd = &self.buildings[b];
            (bd.ty, bd.owner, bd.pos)
        }) else {
            return;
        };
        let corner = self.tile_corner(ty, pos);
        let before = self.buildings[b].gather_from.len();
        let found = self.gather_tcoords(ty, who, corner);
        self.buildings[b].gather_from.extend(found);
        for i in 0..self.buildings[b].gather_from.len() {
            let t = self.buildings[b].gather_from[i];
            self.world.set_tile_bits(t, GATHERED_FROM);
        }
        let n = self.buildings[b].gather_from.len();
        if before < n {
            self.mark(SITE_SHUFFLE);
            for _ in 0..4 * n {
                let k = self.rnd(n as i32) as usize;
                let e = self.buildings[b].gather_from.remove(k);
                self.buildings[b].gather_from.push(e);
            }
        }
        self.buildings[b].gather_max = Some(self.max_gatherers(b));
    }

    /// `corner_tile@006364c0`'s own answer: the footprint's centre in world
    /// units, `(size + 2·corner) · 96`.
    pub fn footprint_centre(&self, ty: usize, corner: Pos) -> Pos {
        let t = &self.build_types[ty];
        let half = UNITS_PER_TILE / 2;
        Pos::new(
            corner.x * UNITS_PER_TILE + t.x_size * half,
            corner.y * UNITS_PER_TILE + t.y_size * half,
        )
    }

    /// `corner_tile@006364c0`: the footprint's centre in world units, as the
    /// tile the distance test measures from and the cell the walk anchors on.
    fn gather_anchor(&self, ty: usize, corner: Pos) -> (Pos, Cell) {
        let t = &self.build_types[ty];
        let half = UNITS_PER_TILE / 2;
        let wx = corner.x * UNITS_PER_TILE + t.x_size * half;
        let wy = corner.y * UNITS_PER_TILE + t.y_size * half;
        let tile = Pos::new(wx.div_euclid(UNITS_PER_TILE), wy.div_euclid(UNITS_PER_TILE));
        (tile, World::cell_of_tile(tile))
    }

    /// Step 5: does this cell pay, and — on the survey path — what does its
    /// sixteen-tile walk put in the scratch list.
    fn cell_qualifies(
        &self,
        cell: Cell,
        sample: Pos,
        timber: bool,
        from: &Source,
        walk: &mut Walk,
    ) -> bool {
        match from {
            Source::List(list) => list.contains(&sample),
            Source::Survey => {
                if self.world.tile_mask(sample) & GATHERED_FROM != 0 {
                    walk.taken = true;
                    return false;
                }
                if !timber && self.world.cell_data(cell).flags & 0x20 != 0 {
                    // `WorldData::is_forest` — a non-timber type skips a
                    // forest cell outright.
                    return false;
                }
                let base = Pos::new(cell.x * TILES_PER_CELL, cell.y * TILES_PER_CELL);
                for i in 0..TILES_PER_CELL * TILES_PER_CELL {
                    let t = Pos::new(base.x + (i % TILES_PER_CELL), base.y + (i / TILES_PER_CELL));
                    if self.gather_tile_kind(t, timber) {
                        walk.found.push(t);
                    }
                }
                self.gather_tile_kind(sample, timber)
            }
        }
    }

    /// `WorldData::is_tree_at@0046f930` / `is_mountain_at@0046f900`.
    fn gather_tile_kind(&self, t: Pos, timber: bool) -> bool {
        let m = self.world.tile_mask(t);
        if timber {
            m & tile::SURFACE == tile::SURFACE_FOREST
        } else {
            m & tile::OBJECT == tile::OBJECT_MOUNTAIN
        }
    }

    /// Step 7: river tiles under the footprint (`calc_gather` lines 428–450).
    fn footprint_rivers(&self, ty: usize, corner: Pos) -> i32 {
        self.footprint(ty, corner)
            .into_iter()
            .filter(|&t| self.world.tile_in_bounds(t) && self.world.tile_mask(t) & tile::RIVER != 0)
            .count() as i32
    }

    /// `BuildTypeData::total_gather_access@00636500`: list tiles that have
    /// gather access and stand more than three tiles from the footprint's
    /// centre. Twice this is the cap on the count.
    fn total_gather_access(&self, list: &[Pos], ty: usize, corner: Pos, who: Player) -> i32 {
        let t = &self.build_types[ty];
        let half = UNITS_PER_TILE / 2;
        let cx = corner.x * UNITS_PER_TILE + t.x_size * half;
        let cy = corner.y * UNITS_PER_TILE + t.y_size * half;
        let mut n = 0;
        for &p in list {
            if !self.has_gather_access(p, who) {
                continue;
            }
            let dx = (cx - p.x * UNITS_PER_TILE - half).abs();
            let dy = (cy - p.y * UNITS_PER_TILE - half).abs();
            if dx + dy > ACCESS_NEAR || dx >= ACCESS_NEAR || dy >= ACCESS_NEAR {
                n += 1;
            }
        }
        n
    }

    /// `WorldData::has_gather_access@006b4e50` with `param_4 = param_5 = 0`:
    /// the tile carries [`GATHERABLE`], its cell is not an enemy's, and at
    /// least one orthogonal neighbour is on the map, not ocean and not
    /// blocked.
    pub fn has_gather_access(&self, t: Pos, who: Player) -> bool {
        if !self.world.tile_in_bounds(t) || self.world.tile_mask(t) & GATHERABLE == 0 {
            return false;
        }
        if let Some(o) = self.world.owner(World::cell_of_tile(t)).player()
            && o != who
            && !self.is_ally(who, o)
        {
            return false;
        }
        ORTHOG.iter().any(|&(dx, dy)| {
            let n = Pos::new(t.x + dx, t.y + dy);
            if !self.world.tile_in_bounds(n) {
                return false;
            }
            let m = self.world.tile_mask(n);
            m & tile::SURFACE != tile::SURFACE_OCEAN && m & tile::BLOCKED == 0
        })
    }

    /// The mine's branch: the size of the mountain range it stands on, scaled
    /// by how much of that range is usable (`MountainRangeData::gather_size`
    /// `@0089d170`, `CliffsData::gather_size@008a8f80` — the same shape).
    ///
    /// The range itself is [`Sim::mountain_range`]'s reconstruction.
    ///
    /// **A range another building already mines is taken** (`calc_gather
    /// @00639e40`'s mountain arm, `docs/AI.md` §62). On the survey — a
    /// site, not a standing building's own list — the arm walks the range's
    /// tiles in the template's order, skipping a tree, a tile that is not a
    /// mountain and one in an enemy's cell, and answers **−1** at the first
    /// that is `is_gathered_from`. `blocked_location` reads that as
    /// `MountainTaken`, so a second Mine on the range is refused.
    fn mine_slots(&self, centre: Pos, who: Player, from: &Source) -> i32 {
        let Some(range) = self.mountain_range(centre) else {
            return 0;
        };
        if matches!(from, Source::Survey) {
            for &t in &range.tiles {
                if self.gather_tile_kind(t, true) || !self.gather_tile_kind(t, false) {
                    continue;
                }
                if let Some(o) = self.world.owner(World::cell_of_tile(t)).player()
                    && o != who
                    && !self.is_ally(who, o)
                {
                    continue;
                }
                if self.world.tile_mask(t) & GATHERED_FROM != 0 {
                    return -1;
                }
            }
        }
        let tiles = i32::try_from(range.tiles.len()).unwrap_or(i32::MAX);
        let mut total = 0;
        let mut usable = 0;
        for c in &range.cells {
            let sample = c.centre_tile();
            total += 1;
            let mine = match self.world.owner(*c).player() {
                Some(o) => o == who || self.is_ally(who, o),
                None => true,
            };
            let listed = match from {
                Source::List(l) => l.contains(&sample),
                Source::Survey => true,
            };
            if mine && listed {
                usable += 1;
            }
        }
        let mut base = MOUNTAIN_GATHER
            .iter()
            .find(|&&(size, _)| tiles < size)
            .map_or(10, |&(_, gather)| gather);
        if total != 0 {
            base = base * usable / total;
        }
        if usable != 0 && base < 1 {
            base = 1;
        }
        base
    }

    /// `MountainsData::find_nearest@0089cd30`, for the one question every
    /// caller here asks of it: the **solid mountain cell** nearest a site,
    /// and its distance, or `None` when the nearest is further than a mine
    /// may reach.
    ///
    /// The original walks every placed mountain and, for each, its range's
    /// `solid_mount_wx`/`_wy` — **cells**, not the `mount_tx` tile list —
    /// keeping the smallest `vector_dist` in world units from the
    /// footprint's centre to the cell's centre, `(loc + off) · 0x300 +
    /// 0x180`. `calc_gather` then drops the answer outright when it exceeds
    /// `gather_radius · 0xc0`. The listing settles the arrays (the offsets
    /// are in `docs/AI.md` §59.2) and the run144 packet settles the
    /// arithmetic: five of East Indies' 10582
    /// spiral sites stand **1536** from the nearest solid cell, two whole
    /// cells, and are refused, where the nearest mountain **tile** is inside
    /// 1152 (`docs/AI.md` §59).
    ///
    /// With the generator's placements in hand ([`Sim::mountains`]) this is
    /// [`Sim::nearest_placed`], the original's walk. Without them, a cell is
    /// solid when its centre tile is a mountain — a superset, 139 cells
    /// against the templates' 107 on East Indies (`docs/AI.md` §59.3) —
    /// and the region test is made per cell.
    pub fn nearest_mountain_cell(&self, centre: Pos) -> Option<(i32, Cell)> {
        if !self.mountains.is_empty() {
            return self
                .nearest_placed(centre)
                .map(|(d, i, k)| (d, self.mountains[i].solid[k]));
        }
        let reach = MINE_RADIUS * UNITS_PER_TILE;
        let at = centre.cell();
        let span = reach / UNITS_PER_CELL + 2;
        let site_region = self.world.region_of(at);
        let mut best: Option<(i32, Cell)> = None;
        for cy in (at.y - span)..=(at.y + span) {
            for cx in (at.x - span)..=(at.x + span) {
                let c = Cell::new(cx, cy);
                if !self.world.contains(c) || !self.is_solid_mountain(c) {
                    continue;
                }
                if let (Some(a), Some(b)) = (site_region, self.world.region_of(c))
                    && a != b
                {
                    continue;
                }
                let d = cell_centre_dist(centre, c);
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, c));
                }
            }
        }
        best.filter(|&(d, _)| d <= reach)
    }

    /// [`Sim::find_nearest_mountain`] inside a mine's reach,
    /// `gather_radius · 0xc0` — what `calc_gather` keeps of it.
    pub fn nearest_placed(&self, centre: Pos) -> Option<(i32, usize, usize)> {
        let reach = MINE_RADIUS * UNITS_PER_TILE;
        self.find_nearest_mountain(centre)
            .filter(|&(d, _, _)| d <= reach)
    }

    /// `MountainsData::find_nearest@0089cd30` over the generator's own
    /// placements: `(distance, placed index, solid-cell index)` — the
    /// distance is the function's out-parameter — or `None` when no range
    /// qualifies.
    ///
    /// The walk is the original's order — placed mountains as the
    /// generator laid them, each range's solid cells in the template's
    /// order — and a cell replaces the best only when it is **strictly**
    /// nearer (`(int)d < best || best < 0`), so a tie goes to the first
    /// placed range and its first cell. A range is skipped when its
    /// **first** solid cell is in another region than the site's; with no
    /// region at the site there is no test (`param_3 < 0`). The placed
    /// index is `MiningList::mtn`, the number a dump prints.
    pub fn find_nearest_mountain(&self, centre: Pos) -> Option<(i32, usize, usize)> {
        let site_region = self.world.region_of(centre.cell());
        let mut best: Option<(i32, usize, usize)> = None;
        for (i, m) in self.mountains.iter().enumerate() {
            let Some(&first) = m.solid.first() else {
                continue;
            };
            if let Some(a) = site_region
                && self.world.region_of(first) != Some(a)
            {
                continue;
            }
            for (k, &c) in m.solid.iter().enumerate() {
                let d = cell_centre_dist(centre, c);
                if best.is_none_or(|(b, _, _)| d < b) {
                    best = Some((d, i, k));
                }
            }
        }
        best
    }

    /// Lays the generator's placements down (`Mountains::add_mountain
    /// @0089c2e0`): each `(location, template)` in the generator's order,
    /// its template's solid cells at `loc + off` and its tiles at
    /// `4 · loc + off`. There is no rotation and no mirror: `add_mountain`
    /// adds the offsets as they stand. A template index the list does not
    /// have places an empty range, which `find_nearest` skips as the
    /// original skips a range with no solid cells, and keeps the indices
    /// aligned with the dump's `mtn`.
    pub fn place_mountains(&mut self, placed: &[(Cell, usize)], templates: &[MountainTemplate]) {
        self.mountains = placed
            .iter()
            .map(|&(loc, template)| {
                let t = templates.get(template);
                PlacedMountain {
                    loc,
                    tiles: t.map_or_else(Vec::new, |t| {
                        t.tiles
                            .iter()
                            .map(|&(dx, dy)| {
                                Pos::new(loc.x * TILES_PER_CELL + dx, loc.y * TILES_PER_CELL + dy)
                            })
                            .collect()
                    }),
                    solid: t.map_or_else(Vec::new, |t| {
                        t.solid
                            .iter()
                            .map(|&(dx, dy)| Cell::new(loc.x + dx, loc.y + dy))
                            .collect()
                    }),
                }
            })
            .collect();
    }

    /// Stand-in `solid_mount_wx`/`_wy` membership: the cell's centre tile
    /// is a mountain.
    fn is_solid_mountain(&self, c: Cell) -> bool {
        let t = c.centre_tile();
        self.world.tile_in_bounds(t)
            && self.world.tile_mask(t) & tile::OBJECT == tile::OBJECT_MOUNTAIN
    }

    /// The mountain range a mine draws on.
    ///
    /// With the generator's placements ([`Sim::mountains`]) it is the
    /// placed range [`Sim::nearest_placed`] names: its template's tiles in
    /// the template's order (`mount_tx`, row by row, which is the order
    /// the gather list is shuffled from), and its solid cells.
    ///
    /// Without them it is **a reconstruction**: the connected component of
    /// mountain **tiles** reachable from [`Sim::nearest_mountain_cell`]'s
    /// centre tile, eight-connected. It was pinned by value rather than by
    /// argument: run97's frame 8383 dumps Great Lakes' first mine with
    /// `mtn 6` and a `length` of **207**, and the component holds **244**
    /// tiles of which exactly 37 carry `SURFACE_FOREST` (`docs/ECONOMY.md`,
    /// "The mine's range"). Template 9, which the generator placed as
    /// range 6, has 244 tiles too.
    ///
    /// [`MountainRange::cells`] is what `MountainRangeData::gather_size
    /// @0089d170` counts: a solid cell whose **centre tile** is a mountain
    /// and whose cell is not a forest one.
    pub fn mountain_range(&self, centre: Pos) -> Option<MountainRange> {
        let is_mtn = |t: Pos| {
            self.world.tile_in_bounds(t)
                && self.world.tile_mask(t) & tile::OBJECT == tile::OBJECT_MOUNTAIN
        };
        if !self.mountains.is_empty() {
            let (_, i, _) = self.nearest_placed(centre)?;
            let m = &self.mountains[i];
            let cells = m
                .solid
                .iter()
                .copied()
                .filter(|&c| {
                    self.world.contains(c)
                        && is_mtn(c.centre_tile())
                        && self.world.cell_data(c).flags & 0x20 == 0
                })
                .collect();
            let tiles = m
                .tiles
                .iter()
                .copied()
                .filter(|&t| self.world.tile_in_bounds(t))
                .collect();
            return Some(MountainRange { tiles, cells });
        }
        let (_, cell) = self.nearest_mountain_cell(centre)?;
        let seed = cell.centre_tile();
        let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
        seen.insert((seed.y, seed.x));
        let mut queue = vec![seed];
        while let Some(t) = queue.pop() {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if (dx, dy) == (0, 0) {
                        continue;
                    }
                    let n = Pos::new(t.x + dx, t.y + dy);
                    if is_mtn(n) && seen.insert((n.y, n.x)) {
                        queue.push(n);
                    }
                }
            }
        }
        // Row-major, which is the template's own order too: a range whose
        // component is its template's tiles lists them in the same sequence.
        let tiles: Vec<Pos> = seen.iter().map(|&(y, x)| Pos::new(x, y)).collect();
        let mut cells: Vec<Cell> = Vec::new();
        for &t in &tiles {
            let c = World::cell_of_tile(t);
            if cells.contains(&c) || !is_mtn(c.centre_tile()) {
                continue;
            }
            if self.world.cell_data(c).flags & 0x20 != 0 {
                continue;
            }
            cells.push(c);
        }
        Some(MountainRange { tiles, cells })
    }
}

/// `vector_dist` in world units from a site to a cell's centre,
/// `c · 0x300 + 0x180` — `find_nearest`'s own measure.
fn cell_centre_dist(centre: Pos, c: Cell) -> i32 {
    vector_dist(
        centre.x - (c.x * UNITS_PER_CELL + UNITS_PER_CELL / 2),
        centre.y - (c.y * UNITS_PER_CELL + UNITS_PER_CELL / 2),
    )
}

/// One `<MOUNTAIN>` template, as `MountainRange::init@008998b0` derives it
/// from the alpha of its `TEMPLATE_TEX` (`docs/FORMATS.md`, "The mountain
/// templates"): offsets from the placed location, in the template's order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MountainTemplate {
    /// `mount_tx`/`_ty`: tile offsets from `4 · loc`.
    pub tiles: Vec<(i32, i32)>,
    /// `solid_mount_wx`/`_wy`: cell offsets from `loc`.
    pub solid: Vec<(i32, i32)>,
}

/// One mountain the generator placed: `MountainsData::mountain_locs[i]` and
/// `mountain_types[i]`, with its template laid at the location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedMountain {
    pub loc: Cell,
    /// The template's tiles, in world tiles.
    pub tiles: Vec<Pos>,
    /// The template's solid cells, in world cells.
    pub solid: Vec<Cell>,
}

/// One mountain range, as [`Sim::mountain_range`] finds or rebuilds it.
pub struct MountainRange {
    /// Every mountain tile of the range — `MountainRangeData::mount_tx`,
    /// which is both the mine's own tile list and the count the gather
    /// rungs are chosen by.
    pub tiles: Vec<Pos>,
    /// `solid_mount_wx`/`_wy`: the range's cells whose centre tile is a
    /// mountain and whose cell is not a forest one — what
    /// `MountainRangeData::gather_size` counts.
    pub cells: Vec<Cell>,
}

#[cfg(test)]
mod tests {
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
                .map(|(_, i, _)| s.mountains[i].loc),
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
}
