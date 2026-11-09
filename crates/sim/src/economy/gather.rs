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
//! - **The cliffs, without a caller to lay them.** A mine's cliff arm
//!   (`cliff_mine.rs`) reads [`Sim::cliffs`], which no dump prints; with none
//!   laid a mine beside a cliff gets the empty list it always had.
//! - **The remaining nation and wonder layer.** `taj_farms`,
//!   `kremlin_farms`, `german_miners` and the Iroquois food bonus all add to
//!   the count and are not modelled. French timber capacity is implemented
//!   in item 1443 (`docs/ECONOMY.md`, "French timber capacity").
//! - **`TData.mask & 0x8000`**, which `has_gather_access` requires, is
//!   unnamed here and in `world::tile`. It is set on 1,711 tiles of run9's
//!   map — enough that the access cap never binds there.

use std::collections::BTreeSet;

use crate::ai_place::{circle, gather_good};
use crate::build::flags;
use crate::economy::Resource;
use crate::world::{
    Cell, REGION_REFIND_GATHER, REGION_VERIFY_GATHER, TILES_PER_CELL, UNITS_PER_CELL,
    UNITS_PER_TILE, World, tile, vector_dist,
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

/// `total_gather_access`'s test on one list tile, in world units from the
/// footprint's centre `(cx, cy)` to the tile's: past the ring when the
/// Manhattan sum is over [`ACCESS_NEAR`] or either axis reaches it. The
/// arithmetic wraps as the listing's 32-bit `sub`/`cdq`/`add` do.
pub(crate) fn beyond_access_ring(cx: i32, cy: i32, p: Pos) -> bool {
    let half = UNITS_PER_TILE / 2;
    let d = |c: i32, t: i32| {
        c.wrapping_sub(t.wrapping_mul(UNITS_PER_TILE))
            .wrapping_sub(half)
            .wrapping_abs()
    };
    let (dx, dy) = (d(cx, p.x), d(cy, p.y));
    dx.wrapping_add(dy) > ACCESS_NEAR || dx >= ACCESS_NEAR || dy >= ACCESS_NEAR
}

/// `BuildTypeData::max_gatherers@0063c430`'s shape, its callees closures
/// (swept in `sweep::build_type_data_max_gatherers`).
///
/// A flat type (`is_flat`, vtable `+0x94`) answers [`MAX_FLAT_GATHERERS`]
/// before anything is asked. Unplaced (`o < 0 || who < 0`), `calc` is handed
/// city `-1` and no list — the survey. Placed, the city is the building's
/// (`BuildData::city`, `+0x72`) when the object is `active` — its flag bit 0,
/// vtable `+0x20` and `+0x4c` (`WallData::is_active`), asked in that order and
/// short-circuited — and `-1` otherwise; then `calc` gets that city and the
/// building's own `MiningList` (`+0x98`). The count `calc` answers is clamped
/// at zero (`cmovs`, `0063c527` and its twin after `0063c559`).
pub(crate) fn max_gatherers_steps<L>(
    flat: bool,
    placed: bool,
    active: impl FnOnce() -> bool,
    city: impl FnOnce() -> i32,
    list: impl FnOnce() -> L,
    calc: impl FnOnce(i32, Option<L>) -> i32,
) -> i32 {
    if flat {
        return MAX_FLAT_GATHERERS;
    }
    if !placed {
        return calc(-1, None).max(0);
    }
    let city = if active() { city() } else { -1 };
    calc(city, Some(list())).max(0)
}

/// `Build::find_gather_tiles@00623350`'s list step: the found tiles appended
/// to the list, then — if the list grew — `4 × length` rounds of "draw an
/// index, move that entry to the back" (`rnd(n)` is `Random::get(0, 0xffff) %
/// n`, and `0` without a draw for `n <= 1`). The original removes the picked
/// entry by value and the list holds each tile once, so removing by index is
/// the same operation. The tile marking sits between append and shuffle in
/// the original and changes nothing the draws read, so it follows here.
pub fn extend_and_shuffle_gather_list(
    list: &mut Vec<Pos>,
    found: Vec<Pos>,
    mut rnd: impl FnMut(i32) -> i32,
) {
    let before = list.len();
    list.extend(found);
    let n = list.len();
    if before < n {
        for _ in 0..4 * n {
            let k = rnd(n as i32) as usize;
            let e = list.remove(k);
            list.push(e);
        }
    }
}

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
        // The city `calc_gather` is handed is a seam here: the count it
        // decides reads none of what the sim models of it.
        max_gatherers_steps(
            self.build_types[ty].has(flags::FLAT),
            true,
            || bd.alive && bd.active,
            || bd.city.map_or(-1, |c| i32::try_from(c).unwrap_or(-1)),
            || &bd.gather_from[..],
            |_city, list| match list {
                Some(l) => self.gather_slots_raw(ty, bd.owner, corner, &Source::List(l)),
                None => self.gather_slots_raw(ty, bd.owner, corner, &Source::Survey),
            },
        )
    }

    /// The same for a prospective site — `max_gatherers` with `o < 0`, which
    /// surveys the map instead of reading a list. `corner` is the footprint's
    /// corner tile, as [`Sim::tile_corner`] gives it.
    pub fn max_gatherers_at(&self, ty: usize, who: Player, corner: Pos) -> i32 {
        max_gatherers_steps(
            self.build_types[ty].has(flags::FLAT),
            false,
            || false,
            || -1,
            || &[][..],
            |_city, list| match list {
                Some(l) => self.gather_slots_raw(ty, who, corner, &Source::List(l)),
                None => self.gather_slots_raw(ty, who, corner, &Source::Survey),
            },
        )
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

    /// [`Sim::gather_slots_raw`] on a standing building's own list.
    #[cfg(test)]
    pub(crate) fn gather_slots_for_list(
        &self,
        ty: usize,
        who: Player,
        corner: Pos,
        list: &[Pos],
    ) -> i32 {
        self.gather_slots_raw(ty, who, corner, &Source::List(list))
    }

    /// The count out-parameter of `calc_gather`, before the clamp
    /// [`max_gatherers_steps`] applies: [`Sim::site_gather_count`] and
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
        // `calc_gather@00639e40`: positive timber capacity takes the
        // French term before the access cap. Empty/taken ground stays so.
        if slots > 0 && timber && self.nation[who as usize].french {
            slots += self.tuning.french_woodies;
        }
        // The wonder additions (`taj_farms`, `kremlin_farms`) remain unmodelled.
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
    /// **A cliff nearer than any range** takes the cliff branch instead
    /// ([`Sim::mine_arm`], [`Sim::cliff_gather_tcoords`]): run1516's mine
    /// beside a cliff on Southwest Mesa lists cliff 7's 189 tiles
    /// (`docs/AI.md` §395, §400).
    pub fn gather_tcoords(&self, ty: usize, who: Player, corner: Pos) -> Vec<Pos> {
        let t = &self.build_types[ty];
        let Some(good) = gather_good(t.ident) else {
            return Vec::new();
        };
        if good == Resource::Metal.index() {
            let centre = self.footprint_centre(ty, corner);
            match self.mine_arm(centre) {
                MineArm::Cliff(k) => return self.cliff_gather_tcoords(k, who),
                MineArm::None => return Vec::new(),
                MineArm::Range => {}
            }
            let Some(range) = self.mountain_range(centre) else {
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
        let found = self.gather_tcoords(ty, who, corner);
        // The shuffle's site marks the stream only when the list grew.
        if !found.is_empty() {
            self.mark(SITE_SHUFFLE);
        }
        // The list is taken out for the draws, which borrow the whole sim.
        let mut list = std::mem::take(&mut self.buildings[b].gather_from);
        extend_and_shuffle_gather_list(&mut list, found, |n| self.rnd(n));
        for &t in &list {
            self.world.set_tile_bits(t, GATHERED_FROM);
        }
        self.buildings[b].gather_from = list;
        self.buildings[b].gather_max = Some(self.max_gatherers(b));
    }

    /// The gate `Build::close`, `Build::process` and `Build::init` share
    /// (`docs/ECONOMY.md` §17.1): the type's `is_gather_type` (vtable
    /// `+0x90`) and not its `is_flat` (`+0x94`). `Build::init` adds a third
    /// test, not the university, that the other two do not; the
    /// university's list is always empty, so nothing it walks differs.
    fn walks_gather_tiles(&self, b: usize) -> bool {
        self.buildings[b].ty.is_some_and(|ty| {
            let t = &self.build_types[ty];
            t.has(flags::GATHERER) && !t.has(flags::FLAT)
        })
    }

    /// `Build::close@00628980`'s tail, `006292a1`–`00629335` in the listing
    /// (`docs/ECONOMY.md` §17.1). On **every** close — a site the script
    /// destroys, a camp killed, the old half of a transfer — a gather
    /// building that walks the ground gives its tiles back: each listed
    /// tile loses `0x1000` (`and $0xefff`), and the building's own region
    /// takes [`REGION_VERIFY_GATHER`] (`or $0x20`), so the camps around it
    /// re-walk the freed ground. Then, whatever the type, the list is
    /// emptied (`+0x9c = 0`) and `MiningList::mtn`/`cliff` go back to −1,
    /// which this crate does not carry.
    ///
    /// Without it a camp the script placed and destroyed on one frame —
    /// `aibestbuildlibrary.bhs`'s `place_woodcutter`, whose `destroy_building`
    /// is `Build::close` by vtable `+0x150` — left its ground marked for
    /// the rest of the game, and the next woodcutter went elsewhere
    /// (East Indies at Toughest, frame 1576; `docs/AI.md` §82).
    pub(crate) fn give_back_gather_tiles(&mut self, b: usize) {
        if self.walks_gather_tiles(b) {
            for i in 0..self.buildings[b].gather_from.len() {
                let t = self.buildings[b].gather_from[i];
                self.world.clear_tile_bits(t, GATHERED_FROM);
            }
            if let Some(r) = self.world.region_of(self.buildings[b].pos.cell()) {
                self.world.or_region_flags(r, REGION_VERIFY_GATHER);
            }
        }
        self.buildings[b].gather_from.clear();
    }

    /// `Build::verify_gather_tiles@00623570` (`docs/ECONOMY.md` §17.2): a
    /// listed tile whose cell a player owns who is not this building's
    /// owner nor allied both ways with it is unmarked and removed — by
    /// value, so the entries behind it close up in order, and the walk
    /// stays on the index it removed. It draws nothing.
    pub(crate) fn verify_gather_tiles(&mut self, b: usize) {
        let who = self.buildings[b].owner;
        let mut i = 0;
        while i < self.buildings[b].gather_from.len() {
            let t = self.buildings[b].gather_from[i];
            let foreign = self
                .world
                .owner(World::cell_of_tile(t))
                .player()
                .is_some_and(|o| !self.is_ally(who, o));
            if foreign {
                self.world.clear_tile_bits(t, GATHERED_FROM);
                self.buildings[b].gather_from.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// `Build::process@0061edf0`'s two re-entries, `0061f3c1`–`0061f41f`
    /// (`docs/ECONOMY.md` §17.2), for one building: an
    /// **active** gather building that walks the ground, standing in a
    /// region carrying [`REGION_VERIFY_GATHER`], runs
    /// [`Sim::verify_gather_tiles`]; one in a region carrying
    /// [`REGION_REFIND_GATHER`] runs [`Sim::find_gather_tiles`] again — which
    /// appends whatever unmarked ground it now reaches and, if the list
    /// grew, shuffles it off the sync stream.
    ///
    /// The original takes both inside `Build::process`, right after the
    /// building's own `do_queue`, and so does [`Sim::tick`] now (twenty-
    /// fourth pass, group 18; A8 row 21): this is one building's.
    pub(crate) fn gather_region_building(&mut self, b: usize) {
        if !self.buildings[b].alive || !self.buildings[b].active || !self.walks_gather_tiles(b) {
            return;
        }
        let Some(r) = self.world.region_of(self.buildings[b].pos.cell()) else {
            return;
        };
        if self.world.region_flags(r) & REGION_VERIFY_GATHER != 0 {
            self.verify_gather_tiles(b);
        }
        if self.world.region_flags(r) & REGION_REFIND_GATHER != 0 {
            self.find_gather_tiles(b);
        }
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
        list.iter()
            .filter(|&&p| self.has_gather_access(p, who) && beyond_access_ring(cx, cy, p))
            .count() as i32
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
    ///
    /// **A cliff nearer than any range** takes the cliff arm
    /// ([`Sim::mine_arm`]): the same −1 walk over the cliff's tiles
    /// ([`Sim::cliff_taken`]), then [`Sim::cliff_gather_size`].
    fn mine_slots(&self, centre: Pos, who: Player, from: &Source) -> i32 {
        match self.mine_arm(centre) {
            MineArm::Cliff(k) => {
                let list = match from {
                    Source::List(l) => Some(*l),
                    Source::Survey => None,
                };
                if list.is_none() && self.cliff_taken(k, who) {
                    return -1;
                }
                return self.cliff_gather_size(k, who, list);
            }
            MineArm::None => return 0,
            MineArm::Range => {}
        }
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

    /// **Which of `calc_gather`'s two mine arms a site takes**
    /// (`63a555…`; `find_gather_tcoords`' metal branch the same): the
    /// nearest range ([`Sim::find_nearest_mountain`], or the stand-in's
    /// [`Sim::nearest_mountain_cell`]) unless a cliff
    /// ([`Sim::find_nearest_cliff`]) is **strictly** nearer, and then
    /// whichever won only inside `gather_radius · 0xc0`. A range that wins
    /// and is out of reach leaves the site nothing; the cliff is not asked
    /// again.
    pub fn mine_arm(&self, centre: Pos) -> MineArm {
        let reach = MINE_RADIUS * UNITS_PER_TILE;
        let mtn = if self.mountains.is_empty() {
            self.nearest_mountain_cell(centre).map(|(d, _)| d)
        } else {
            self.find_nearest_mountain(centre).map(|(d, _, _)| d)
        };
        let cliff = self.find_nearest_cliff(centre);
        if let Some(dm) = mtn
            && cliff.is_none_or(|(dc, _)| dm <= dc)
        {
            return if dm <= reach {
                MineArm::Range
            } else {
                MineArm::None
            };
        }
        match cliff {
            Some((dc, k)) if dc <= reach => MineArm::Cliff(k),
            _ => MineArm::None,
        }
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

/// The arm [`Sim::mine_arm`] names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MineArm {
    /// The nearest mountain range, inside the reach.
    Range,
    /// This cliff of [`Sim::cliffs`], strictly nearer than any range and
    /// inside the reach.
    Cliff(usize),
    /// Neither: whichever was nearest is out of reach, or there is none.
    None,
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
mod tests;
