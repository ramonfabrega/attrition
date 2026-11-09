//! City sites and the city AI — `Leader::compute_sites@006cc950`,
//! `Leader::compute_site_stats@006cd040`, `Leader::found_cities@006c7a60`
//! and `Leader::produce_city@006cb120`; `docs/AI.md` §2.7, §2.13, §2.12,
//! §2.17.
//!
//! **Coordinates.** The original's "tile" — its `WData` record, `world+0x134`
//! indexed by `world.xs` — is this simulation's **cell**, and its `0x300`
//! coordinate step is [`UNITS_PER_CELL`]. A site's `wx, wy` are therefore
//! cell coordinates, a site position is `Pos::new(wx * UNITS_PER_CELL, …)`
//! (the cell's *corner*, which is what `blocked_site` and `init_build` are
//! passed), and every `vector_dist` on this path is measured in cells. The
//! original's finer `TData` grid (`world+0x138`, four by four per `WData`)
//! is this simulation's tile grid, and the two masks these functions read
//! are indexed at the cell's **centre tile** (`(4x + 2, 4y + 2)`, the `+ 4`
//! byte bias in the decompile's address arithmetic).
//!
//! **What the flat harness world cannot supply is a named seam**, each
//! answering as an empty, fully-explored map would; the list is repeated in
//! the module's report:
//!
//! * `WorldData::was_seen` — always true (the lobby is `REVEAL_MAP`).
//! * `GameAccessConst::find_tcoord_z` — [`World::tile_z`], a table the map
//!   loader pins from a `DUMP_ALL` dump's `master_land_heights`; 0 on a
//!   flat world, where the `z / 25` term of a candidate tile's score
//!   vanishes. (Seam retired 2026-08-24: on run9 it is the whole difference
//!   between the sampler's `(51,16)/291` and the original's `(52,14)/370`.)
//! * `WorldData::danger[who]` (`world+0x13c`, a half-cell-resolution `int`
//!   grid — **not** territory; `docs/AI.md` §2.13 step 8 and §2.12 name it
//!   "my territory at the site's cell", which the PDB contradicts) —
//!   answered 0 by [`Sim::site_danger`].
//! * `LeaderData::get_target` (team style 2) — no target.
//! * `leader_flags2 & 0x10` (the `found_cities` early-out) and
//!   `leader_flags & 8` (the human-cap bypass) — never set.
//! * `LeaderData::village_num` — 0; run8's leader block shows it 0 with a
//!   Small City standing, so it is not the settlement count.
//! * The map type `world+0x30` is taken as [`crate::ai::Lobby::map_style`],
//!   which is also `GameInfo.map_style`; the original reads two fields.
//! * `Region.coords` order — the original walks the region's own coordinate
//!   list; [`crate::world::World::cells_in`] is row-major. On run1385p's
//!   2775 sweep the two visit the same cells in the same order, all 77
//!   calls in two regions (item 3708, `docs/AI.md` §358.3); no other map is
//!   read.
//! * `ObjectsData::find_city`/`find_unit` search *friendly* objects (mine
//!   and my allies'); here they search mine. Both are only used as presence
//!   flags on this path.
//! * The per-player object arrays are iterated as this simulation's global
//!   `units` / `buildings` / `cities` lists filtered by owner, in index
//!   order; the original iterates object numbers.
//! * `TypeData.type == PEASANTS || == PEASANTSKOREAN` (an exact pair) is
//!   taken as the citizen *lineage*, the same predicate `ai_place.rs` uses
//!   for `FILTER_BASE_TYPE 0x32`.

use crate::ai::{Census, SITE_SLOTS, Site};
use crate::build::{self, Ident};
use crate::economy::RESOURCES;
use crate::orders::{Body, index};
use crate::place::Blocked;
use crate::tech::{Kind, TypeId};
#[cfg(test)]
use crate::world::Terrain;
use crate::world::{CORNER_X, CORNER_Y, Cell, Owner, Pos, UNITS_PER_CELL, tile, vector_dist};
use crate::{Player, Sim, cost};

/// The sweep's two draw sites, under the original's own offsets from
/// `Leader::compute_sites@006cc950`. [`Sim::mark`] writes them into
/// [`Sim::phase_marks`], so the frame's first two draws are compared
/// against `rondata::trace`'s by name (`docs/SYNC.md` §5).
pub const SITE_STRIDE: &str = "Leader::compute_sites+0x4ac";
pub const SITE_MARK: &str = "Leader::compute_sites+0x50a";

/// `FOUCHE`, `TypeIndex` 371: the unit type whose count `WorldData::was_seen`
/// reads as `num_units[0x141]`. `num_units` starts at `BASE_UNITTYPES` (50):
/// `Leader::unit_prod_value@006cc580` walks it beside the unit-type list from
/// offset `200 = 0x32 × 4`, so slot `0x141` is `TypeIndex` 50 + 321 — not
/// `TRANSPORTGALLEON` (321), the reading the floors killed (`docs/AI.md` §141).
const FOUCHE: i32 = 0x32 + 0x141;

/// `move_x[0..25]` / `move_y[0..25]` — read from `.rdata` at `0x00adcaf0`
/// and `0x00adc400` (`?move_x@@3QBHB` / `?move_y@@3QBHB` in `rise_z.map`,
/// `compass.obj`). Entry 0 is the centre, 1..8 the compass ring, 9..24 the
/// sixteen cells of the next ring: the 5×5 square, each cell once.
const MOVE_X: [i32; 25] = [
    0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2, 2, -2,
];
const MOVE_Y: [i32; 25] = [
    0, -1, -1, -1, 0, 1, 1, 1, 0, -2, -2, -2, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, 2, 2,
];

/// `move_x[81..121]` / `move_y[81..121]` — the same tables' radius-5 ring,
/// the forty cells of the 11×11 border, walked clockwise from `(−5, −5)`.
/// (81 = 1 + 8 + 16 + 24 + 32, the cells inside radius 4.)
const RING5_X: [i32; 40] = [
    -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3,
    -4, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5,
];
const RING5_Y: [i32; 40] = [
    -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 4, 3, 2, 1, 0, -1, -2, -3, -4,
];

// `corner_x[0..5]` / `corner_y[0..5]` at `0x00adc3e0` / `0x00adc3c0` — the
// centre and the four corners, the nomad's wood test. The table itself is
// `crate::world::CORNER_X`; the pasture indexes the same one
// (`docs/SYNC.md` §3.11).

/// What `compute_site_stats` writes back: the score, the distance figure,
/// and the site's coordinates, which a `keep` call may have moved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SiteStats {
    val: i32,
    dist: i32,
    wx: i32,
    wy: i32,
}

/// `a * b`, wrapping — the original's arithmetic is 32-bit `imul` and the
/// score genuinely overflows on a rich tile (`found_cities` clamps the
/// product to 999,999 afterwards, which only makes sense if it wraps).
const fn mul(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// `a * n / d`, the product wrapping and the division truncating toward
/// zero, as `imul` then `idiv` do.
const fn mul_div(a: i32, n: i32, d: i32) -> i32 {
    a.wrapping_mul(n) / d
}

impl Sim {
    // ------------------------------------------------------------------
    // Seams and small helpers
    // ------------------------------------------------------------------

    /// `WorldData::danger[who]` at a site's cell — `world+0x13c`, an `int`
    /// grid at half the cell resolution (`(wy >> 1) * reg_xs + (wx >> 1)`),
    /// which is the divisor of §2.13 step 8 and the `> 0` test of §2.12.
    /// [`crate::danger`] writes it.
    fn site_danger(&self, who: Player, wx: i32, wy: i32) -> i32 {
        self.world.danger_half(who, crate::world::Cell::new(wx, wy))
    }

    /// `WorldData::was_seen@006b53f0(2wx + 1, 2wy + 1, who)`, read whole:
    /// true outright when `reveal_map > 1` or `who > 7`; then, unless the
    /// leader has flag `0x1000` or `0x800` or owns a `types[0x141]`, **a
    /// cell owned by an ally (oneself included — `is_ally` is reflexive)
    /// is seen whenever that owner's `reg_cities` or `reg_forts` for the
    /// cell's region is non-zero** (`LeaderData +0x125e`/`+0x12de`); else
    /// `seen2[fy × fog_xs + fx] & ally_mask` on the fog grid. Run20 is
    /// where the territory arm shows: the AI's ten sites sit on its own
    /// land with `seen2 == 0` at every one of them, and the original
    /// scores them because `reg_cities[home] == 1`. The fog grid is the
    /// dump's start-of-game snapshot (`World::seen2`); with none loaded
    /// the last arm answers true, which is the reading the flat world
    /// always had. The three leader exits are [`Sim::sees_every_cell`]'s
    /// (`docs/AI.md` §141). Not modelled: `reg_forts` (the census does not
    /// keep it; no fort stands in any capture at the sweep).
    /// `LeaderData +0x125e[region]` — a **leader's** count of cities in a
    /// region, which is what `was_seen`'s territory shortcut reads.
    ///
    /// The census recounts it from the four city types at step 8
    /// (`census_zero_regions`) and the original runs the census for a human
    /// leader too (`docs/AI.md` §23.1); this crate does not yet, so a
    /// leader whose census has never run is answered by the same recount
    /// rather than by a zero that is only an artefact of the gate. run62 is
    /// where that showed: six nodes of the human's road search stand on the
    /// human's own land with `seen2` clear, and the original prices them
    /// **undoubled** (`docs/ROADS.md` §7.3).
    pub(crate) fn leader_reg_cities(&self, who: Player, r: u16) -> i32 {
        match self.ai.get(who as usize) {
            Some(a) if !a.census.reg_cities.is_empty() => Census::reg(&a.census.reg_cities, r),
            _ => self.reg_city_buildings_pub(who, r),
        }
    }

    fn site_was_seen(&self, wx: i32, wy: i32, who: Player) -> bool {
        self.was_seen_fog(2 * wx + 1, 2 * wy + 1, who)
    }

    /// The same read at the fog grid's **own** coordinates, which is the
    /// pair `WorldData::was_seen` actually takes — `site_was_seen` is the
    /// whole-cell sampler on top of it, and `Sim::find_goody_box` is the
    /// caller that wants all four half-cells of a cell rather than one.
    ///
    /// The territory shortcut reads the **cell** the half-cell falls in
    /// (`(fy >> 1) * xs + (fx >> 1)` at `006b5460`), not the half-cell, so
    /// it is the same answer for all four samples of a cell.
    pub(crate) fn was_seen_fog(&self, fx: i32, fy: i32, who: Player) -> bool {
        // Every remaining arm is also an early true return. A present seen
        // bit already proves the final OR, without scanning live wonders and
        // units. Nothing is retained across calls or mutations.
        if who < 8
            && let Some(ai) = self.ai.get(who as usize)
            && let Some(bits) = self.world.seen2(fx, fy)
            && u32::from(bits) & (ai.census.ally_mask | (1 << who)) != 0
        {
            return true;
        }
        self.was_seen_fog_reference(fx, fy, who)
    }

    fn was_seen_fog_reference(&self, fx: i32, fy: i32, who: Player) -> bool {
        if self.lobby.reveal_map > 1 || who > 7 || self.sees_every_cell(who) {
            return true;
        }
        let cell = Cell::new(fx >> 1, fy >> 1);
        if let Owner::Player(o) = self.world.owner(cell)
            && self.is_ally(who, o)
            && let Some(r) = self.world.region_of(cell)
            && (self.leader_reg_cities(o, r) != 0 || self.reg_forts(o, r) != 0)
        {
            return true;
        }
        let Some(bits) = self.world.seen2(fx, fy) else {
            return true;
        };
        let mask = self.ai[who as usize].census.ally_mask | (1 << who);
        u32::from(bits) & mask != 0
    }

    /// `WorldData::was_seen@006b53f0`'s leader arm, ahead of the territory
    /// shortcut and the fog: `leader_flags & 0x1000`, `& 0x800`, or
    /// `num_units[0x141]` — any of them, and every cell is seen.
    ///
    /// - **`0x1000`** is `EXPLORE_MAP_BONUS` held: `Leader::gain_tech@006dcb60`
    ///   raises it when the tech gained is the bonus's `preq0`
    ///   ([`crate::tech::Roles::explore_map_preq`], Electronics), and
    ///   `fix_tech_flags@006d2480` sets or clears it from `has_tech` of the
    ///   same tech. A tech is never lost here, so holding it is the bit.
    ///   **And the Spanish**: `Setup::build_empire@005abb80` raises it for a
    ///   leader with `has_tribe_bonus(9)` once its cities and units stand,
    ///   and `Build::activate@00623e20` again on every city it founds;
    ///   only `fix_tech_flags` (from `lose_tech`, never reached here)
    ///   clears it. `has_tribe_bonus` is false until the leader has a city
    ///   or a starting town, which is the order both writers keep. run958's
    ///   who=1 dumps `leader_flags` 4099 (`0x1003`) on every frame (item
    ///   3318, `docs/AI.md` §264).
    /// - **`0x800`** is `REVEAL_ENEMY_BONUS`'s `preq0` (`disable` in the
    ///   shipped file) or the **Space Program** (`has_wonder(0x21e)`, which
    ///   `gain_tech` tests on every gain and `fix_tech_flags` on a loss).
    ///   Read live: a Space Program lost before the next tech gain keeps the
    ///   original's bit and not this one — no capture holds one.
    /// - **`num_units[0x141]`**, a **Fouché** standing ([`FOUCHE`]; the
    ///   count is read live, as the original keeps it). No capture holds one.
    ///
    /// The coverage pair's All Technologies start owns Electronics on frame
    /// 0, so its AI scores every cell of a site's 5×5 as seen — the water
    /// it counts and the cells it slides to (run651's block 1, `docs/AI.md`
    /// §141).
    pub(crate) fn sees_every_cell(&self, who: Player) -> bool {
        self.holds_bonus(who, self.tables.tech_tree.roles.explore_map_preq)
            || self.tech.get(who as usize).is_some_and(|p| {
                self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    p,
                    crate::nations::power::SPANISH,
                )
            })
            || self.sees_every_unit(who)
    }

    /// The two always-true leader arms `WorldData::was_seen@006b53f0` shares
    /// with `WorldData::is_seen@006b55c0`: `leader_flags & 0x800`
    /// (`REVEAL_ENEMY_BONUS` held, or the Space Program) and
    /// `num_units[0x141]`, a Fouché standing. `is_seen` lacks the `0x1000`
    /// arm [`Sim::sees_every_cell`] adds — an explored map is not a lit one.
    pub(crate) fn sees_every_unit(&self, who: Player) -> bool {
        if self.holds_bonus(who, self.tables.tech_tree.roles.reveal_enemy_preq) {
            return true;
        }
        if self.wonders_held(who) & (1 << crate::tech::wonder::SPACE_PROGRAM) != 0 {
            return true;
        }
        self.units
            .iter()
            .any(|u| u.owner == who && u.alive() && u.type_index == FOUCHE)
    }

    /// Whether `who` holds a bonus's one prerequisite — `has_tech` of it,
    /// read live, which is what `fix_tech_flags@006d2480` turns each
    /// `leader_flags` bonus bit into. A role the tree does not know is not
    /// held.
    pub(crate) fn holds_bonus(&self, who: Player, preq: Option<TypeId>) -> bool {
        let Some(tech) = self.tech.get(who as usize) else {
            return false;
        };
        preq.is_some_and(|t| self.tables.tech_tree.has_tech(&self.tables.setup, tech, t))
    }

    /// `WorldData::is_ocean@006b4830`: **the cell's own kind**, not its
    /// region — `land == 1` (SANDY, the shallows) or `2` (OCEAN), and never
    /// when the coastal flag `0x100` is set. Read by region terrain until
    /// run20 (2026-08-25), where the beaches of an islands map — SANDY
    /// cells inside a land region — made three of the ten sites' `water`
    /// counts differ from the original's.
    fn site_is_ocean(&self, c: Cell) -> bool {
        let d = self.world.cell_data(c);
        d.flags & 0x100 == 0 && (d.land == 1 || d.land == 2)
    }

    /// `Forts::find_fort(wx, wy, who)`: a live fort of `who` standing on
    /// exactly this cell.
    ///
    /// **A fort counts once it is built** (item 3708, `docs/AI.md` §358.3).
    /// `find_fort@0073ebe0` walks `forts.lists[who]`, the entries with
    /// `+6 & 1`; `Fort::init@0073eac0` sets that byte and is reached only
    /// through `Forts::init_fort@0073ed70`, whose one caller is
    /// `Build::activate@00623e20:566` (the type's vslot `+0xfc` arm, beside
    /// `UnbuiltForts::remove_unbuilt_fort`). A placed fort is an
    /// `UnbuiltFort` and is not on the list — the census's
    /// [`Sim::reg_forts`] reads the same list and already asks `active`.
    fn fort_at(&self, who: Player, c: Cell) -> bool {
        self.buildings.iter().any(|b| {
            b.alive
                && b.active
                && b.owner == who
                && b.ty
                    .is_some_and(|t| build::is_fort(&self.tables.build_types, t))
                && b.pos.cell() == c
        })
    }

    /// `types.list[BASE_UNITTYPES]` — the tree's first unit, the Citizen.
    fn citizen_root(&self) -> Option<TypeId> {
        self.tables
            .tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_unit())
    }

    /// A unit in the Citizen line (the seam for `type == PEASANTS ||
    /// type == PEASANTSKOREAN`).
    fn unit_is_citizen(&self, u: usize) -> bool {
        let Some(root) = self.citizen_root() else {
            return false;
        };
        self.units[u]
            .ty
            .and_then(|r| self.tables.unit_types[r].tree)
            .is_some_and(|t| self.tables.tech_tree.is(t, root, false))
    }

    /// The build record of a city level, by ident.
    fn city_record(&self, ident: Ident) -> Option<usize> {
        self.tables
            .build_types
            .iter()
            .position(|b| b.ident == ident)
    }

    /// The six goods' tree ids, in `econ`/`Resource` order.
    fn good_types(&self) -> Vec<TypeId> {
        self.tables
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == Kind::Good)
            .map(|(i, _)| i)
            .take(RESOURCES)
            .collect()
    }

    /// `blocked_site(t, wx * 0x300, wy * 0x300, who, −1, 0) == 0` — the
    /// cell-corner form every call on this path takes.
    fn site_clear(&self, who: Player, rec: usize, wx: i32, wy: i32) -> bool {
        self.blocked_site(
            Some(who),
            rec,
            Pos::new(wx * UNITS_PER_CELL, wy * UNITS_PER_CELL),
            None,
        ) == Blocked::Clear
    }

    /// The reference objects `compute_sites` hands `compute_site_stats`:
    /// with cities of mine in the region, the nearest friendly city (and
    /// its centre object); with none, the nearest friendly citizen. Only
    /// their *presence* is read downstream, so this answers two flags.
    fn site_reference(&self, who: Player, reg: u16, at: Pos) -> (bool, bool) {
        if Census::reg(&self.ai[who as usize].census.reg_cities, reg) == 0 {
            let unit = self.units.iter().enumerate().any(|(u, x)| {
                x.owner == who
                    && x.alive()
                    && x.on_map
                    && self.world.region_of(x.pos.cell()) == Some(reg)
                    && self.unit_is_citizen(u)
            });
            (false, unit)
        } else {
            let found = self
                .cities
                .iter()
                .any(|c| c.alive && c.owner == who && c.reg == Some(reg));
            let _ = at;
            (found, found)
        }
    }

    // ------------------------------------------------------------------
    // compute_site_stats — §2.13
    // ------------------------------------------------------------------

    /// `Leader::compute_site_stats@006cd040`, read whole (506 lines).
    ///
    /// `city` and `unit` are the two object arguments as presence flags
    /// (the original passes indices and only ever tests their sign).
    /// `keep` is the original's eighth argument: with it set, the site may
    /// **slide** to the best-valued open cell of its own 5×5, and the moved
    /// coordinates come back in the result — the sampler's calls set it,
    /// the re-score of a kept site does not. Every early return leaves
    /// `val = dist = 0`.
    #[allow(clippy::too_many_arguments)]
    fn compute_site_stats(
        &self,
        who: Player,
        wx: i32,
        wy: i32,
        city: bool,
        unit: bool,
        reg: u16,
        keep: bool,
    ) -> SiteStats {
        let w = who as usize;
        // An early return leaves `val = dist = 0` — but **not** the site's
        // coordinates: the original's out-parameters are written by the
        // slide in step 3 and are not rolled back, so a return after the
        // slide reports the moved cell with a score of zero.
        let zero = |x: i32, y: i32| SiteStats {
            val: 0,
            dist: 0,
            wx: x,
            wy: y,
        };
        let (world_w, world_h) = (self.world.width(), self.world.height());
        let here = Cell::new(wx, wy);

        // 1. The centre tile inside a city radius, unseen, another
        //    leader's, or water.
        if self.world.tile_mask(here.centre_tile()) & tile::CITY_RADIUS != 0 {
            return zero(wx, wy);
        }
        // `was_seen(2wx + 1, 2wy + 1, who)` — from the fog grid when the
        // world has one (run20 is the first capture where it matters).
        if !self.site_was_seen(wx, wy, who) {
            return zero(wx, wy);
        }
        let mut ally_land = false;
        if let Owner::Player(o) = self.world.owner(here)
            && o != who
        {
            // A Conquer-the-World game, before my first city, may site on
            // an ally's land at a tenth of the score.
            if !self.lobby.conquest || self.city_num(who) != 0 || !self.is_ally(o, who) {
                return zero(wx, wy);
            }
            ally_land = true;
        }
        if self.site_is_ocean(here) {
            return zero(wx, wy);
        }

        // 2. The base is the cell's `val` byte, zeroed when the **Town's**
        //    footprint does not fit (not the Village's).
        let town = self.city_record(Ident::Town);
        let mut base = i32::from(self.world.cell_data(here).val);
        if town.is_none_or(|t| !self.site_clear(who, t, wx, wy)) {
            base = 0;
        }

        // 3. The 5×5. `danger`, `water`, `forts` and `target_adj` are read
        //    around the *original* centre even after the site slides.
        let mut danger = 0i32;
        let mut water = 0i32;
        let mut forts = 0i32;
        let mut target_adj = 1i32;
        let (mut sx, mut sy) = (wx, wy);
        for k in 0..25 {
            let (x, y) = (wx + MOVE_X[k], wy + MOVE_Y[k]);
            if x < 0 || y < 0 || x >= world_w || y >= world_h {
                continue;
            }
            // `was_seen` again: an unseen cell is neither land nor water
            // — it adds nothing and cannot be slid to.
            if !self.site_was_seen(x, y, who) {
                continue;
            }
            let cc = Cell::new(x, y);
            let owner = self.world.owner(cc);
            let coastal = self.world.cell_data(cc).flags & 0x100 != 0;
            if !self.site_is_ocean(cc) && !coastal {
                match owner {
                    Owner::Player(o) if o == who => {}
                    Owner::Player(o) => {
                        if self.lobby.team_style == 2 {
                            // `get_target` — the seam; no target, so this
                            // arm never fires.
                            if self.leader_target(who) == Some(o) {
                                target_adj += 1;
                                danger += 2;
                            }
                        } else if !self.is_ally(who, o) {
                            danger += 2;
                        }
                    }
                    _ => danger += if self.city_num(who) > 2 { 2 } else { 1 },
                }
            } else {
                water += 1;
                danger = (danger - 1).max(0);
            }

            // The slide, on a cell that is mine (or my ally's, on ally
            // land) and outside every city radius.
            let mine = match owner.player() {
                Some(o) => o == who || (ally_land && self.is_ally(who, o)),
                None => false,
            };
            if !(mine && keep && self.world.tile_mask(cc.centre_tile()) & tile::CITY_RADIUS == 0) {
                continue;
            }
            if self.fort_at(who, cc) {
                forts += 1;
            }
            let mut q = i32::from(self.world.cell_data(cc).val);
            if self.world.contains(Cell::new(x + 1, y + 1)) {
                q += i32::from(self.world.cell_data(Cell::new(x + 1, y)).val)
                    + i32::from(self.world.cell_data(Cell::new(x + 1, y + 1)).val)
                    + i32::from(self.world.cell_data(Cell::new(x, y + 1)).val);
            }
            // `find_tcoord_z(wx × 0x300 + 0x180, …) / 25` — the height of
            // the cell's centre tile, from the pinned table (`World::tile_z`;
            // 0 on a flat world).
            let q = (q >> 2) + self.world.tile_z(cc.centre_tile()) / 25;
            if base < q && town.is_some_and(|t| self.site_clear(who, t, x, y)) {
                sx = x;
                sy = y;
                base = q;
            }
        }

        // 4.
        if base < 1 {
            return zero(sx, sy);
        }
        let parity = base & 3;
        let mut v = base;

        // 5. A sea map (`world+0x34 > 2` — the style's sea class, not a
        //    landmass count; `World::sea_map`) and no dock: an ocean cell
        //    on the radius-5 ring is worth thirty times as much. Live for
        //    the first time in run20 (East Indies, `sea_map 4`); dead in
        //    every capture before it. The ring is centred on the
        //    **original** cell, not the slid one — the original reloads the
        //    saved `local_14`/`local_18` immediately before this loop
        //    (`compute_site_stats` 233–234) and only refreshes its working
        //    pair from the moved `param_1`/`param_2` afterwards (253–254).
        //    Steps 7, 8 and 10–13 below do use the moved cell.
        //    (`docs/audit/2026-08-25-ai.md`, B4-k.)
        if self.world.sea_map() > 2
            && self
                .city_record(Ident::Dock)
                .is_none_or(|d| self.buildings_of_line(who, d) == 0)
        {
            for k in 0..40 {
                let (x, y) = (wx + RING5_X[k], wy + RING5_Y[k]);
                if x < 0 || y < 0 || x >= world_w || y >= world_h {
                    continue;
                }
                if self.site_is_ocean(Cell::new(x, y)) {
                    v = mul(base, 30);
                    break;
                }
            }
        }

        // 6. Water, danger and forts.
        let cities = self.city_num(who);
        v = mul(v, 250) / (water + 1);
        if cities == 1 {
            if danger != 0 {
                v = mul_div(danger.min(9) + 7, v, 8);
            }
        } else if cities == 2 {
            if danger != 0 {
                v = mul_div(danger.min(4) + 7, v, 8);
            }
        } else if forts != 0 {
            v = mul_div(v, 3, 2);
        }
        v = mul_div(target_adj + 1, v, 2);

        // 7. The map edge, on the moved coordinates.
        let map = self.lobby.map_style;
        if map != 0xc && map != 0x11 {
            let (lo_x, hi_x, lo_y, hi_y) = if cities < 3 {
                (world_w / 5, world_w * 4 / 5, world_h / 5, world_h * 4 / 5)
            } else {
                (
                    world_w / 10,
                    world_w * 9 / 10,
                    world_h / 10,
                    world_h * 9 / 10,
                )
            };
            if sx < lo_x || hi_x < sx {
                v /= 4;
            }
            if !(lo_y <= sy && sy <= hi_y) {
                v /= 4;
            }
        }

        // 8. Danger at the site, then the cell's *second* claimant.
        let site = Cell::new(sx, sy);
        v /= self.site_danger(who, sx, sy).max(1);
        match self.world.second(site).player() {
            Some(o) if o == who => {}
            Some(o) => {
                if self.is_ally(o, who) {
                    v /= 2;
                } else {
                    v = mul(v, 4);
                }
            }
            // `Ambiguous` is the original's −2 and reads as unowned.
            None => v = mul(v, 2),
        }

        // 9. The region's crowding.
        if self.lobby.map_style != 0x14 {
            let c = &self.ai[w].census;
            let rc = Census::reg(&c.reg_cities, reg);
            let rl = Census::reg(&c.reg_land, reg);
            if rc == 0 {
                v = mul(v, 2);
            }
            if rc == 1 {
                v = mul(v, 2);
            }
            if rl == 0 {
                v = mul(v, 4);
            }
            if rl < 3 {
                v = mul(v, 4);
            }
            if rl < rc {
                v = mul(v, 2);
            }
            if rl < rc * 2 {
                v = mul(v, 2);
            }
        }

        let goods = self.world.cell_data(site).goods;

        // 11 (the decompile's order: the nomad block precedes the goods
        //    loop). A nomad scores by the distance to its nearest citizen,
        //    and needs wood in one of the five corner cells.
        if cities == 0 {
            let centre = Pos::new(
                sx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                sy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            );
            let mut best: Option<i32> = None;
            for u in 0..self.units.len() {
                let x = &self.units[u];
                if x.owner != who || !x.alive() || !self.unit_is_citizen(u) {
                    continue;
                }
                let d = vector_dist(x.pos.x - centre.x, x.pos.y - centre.y);
                if best.is_none_or(|b| d <= b) {
                    best = Some(d);
                }
            }
            if let Some(d) = best {
                v /= d / 0xc00 + 1;
            }
            if self.lobby.starting_resources != 8 {
                let mut wood = false;
                for k in 0..5 {
                    let (x, y) = (sx + CORNER_X[k], sy + CORNER_Y[k]);
                    if x < 0 || y < 0 || x >= world_w || y >= world_h {
                        continue;
                    }
                    if self.world.cell_data(Cell::new(x, y)).goods & 2 != 0 {
                        wood = true;
                        break;
                    }
                }
                if !wood {
                    return zero(sx, sy);
                }
            }
        }

        // 10. The goods near the cell against the economy's wants.
        let mut have = 0i32;
        let mut lack = 0i32;
        for (g, t) in self.good_types().into_iter().enumerate() {
            if self.type_avail(who, t) == 0 {
                continue;
            }
            let econ = self.ai[w].econ[g];
            if goods & (1u8 << g) == 0 {
                lack |= econ & 6;
            } else {
                have |= econ & 6;
                if econ & 2 != 0 {
                    v = mul_div(v, 3, 2);
                }
                if econ & 4 != 0 {
                    v = mul_div(v, 3, 2);
                }
            }
        }
        if have == 0 {
            v = mul_div(v, 2, 3);
        }
        if lack & !have != 0 {
            if parity == 0 {
                v /= 2;
            } else {
                v = mul_div(v, 3, 4);
            }
        }

        // 12. The distance term. **Settled** (`docs/AI.md` §13's first open
        //     item): the two tests inside the per-city loop are `return`s,
        //     not `break`s — at `006cd9e7` and `006cd9f4` both branches
        //     jump to `006cdc68`, the epilogue, which is *past* the stores
        //     of `val` and `dist` at `006cdc60`. A `break` would have to
        //     land on `006cda91`, and could not: the accumulator and the
        //     city pointer share `%ecx` inside the loop. So a dead city
        //     slot, or one city of mine outside the site's region, scores
        //     the site **0**.
        if !unit {
            // No reference object at all: the constants, and the loop is
            // never entered.
            let stats = SiteStats {
                val: mul(3, v),
                dist: 9,
                wx: sx,
                wy: sy,
            };
            return self.site_stats_tail(who, stats, sx, sy, cities, ally_land);
        }
        let k = {
            let mut k = 1i32;
            if city {
                for c in self.cities.iter().filter(|c| c.owner == who) {
                    if !c.alive || c.reg != Some(reg) {
                        return zero(sx, sy);
                    }
                    let cp = c.pos.cell();
                    let d = vector_dist(sx - cp.x, sy - cp.y);
                    if d < 5 {
                        v /= 2;
                    }
                    if d < 8 {
                        v /= 2;
                    }
                    k += if cities > 2 { d.min(10) } else { d };
                }
            }
            k
        };
        let stats = SiteStats {
            val: mul(k, v),
            dist: mul(mul(k, k), 2),
            wx: sx,
            wy: sy,
        };
        self.site_stats_tail(who, stats, sx, sy, cities, ally_land)
    }

    /// `LeaderData::get_target` — the seam; team style 2 is not modelled.
    fn leader_target(&self, _who: Player) -> Option<Player> {
        None
    }

    /// Steps 13 and 14: enemy proximity, then the ally-land tenth.
    fn site_stats_tail(
        &self,
        who: Player,
        mut stats: SiteStats,
        sx: i32,
        sy: i32,
        cities: i32,
        ally_land: bool,
    ) -> SiteStats {
        let map = self.lobby.map_style;
        if map != 0xc && map != 0x11 && (cities == 1 || self.lobby.team_style == 2) {
            let mut sum = 0i32;
            let nations = self.players.len() as i32;
            for i in 0..self.players.len() {
                let l = i as Player;
                if self.defeated[i] || l == who || self.is_ally(who, l) {
                    continue;
                }
                if self.lobby.team_style == 2 && cities >= 2 && self.leader_target(who) != Some(l) {
                    continue;
                }
                for c in self.cities.iter().filter(|c| c.owner == l) {
                    // `city_flags & 0x11 == 0x11`: alive **and capital**,
                    // not "every active city" as `docs/AI.md` §2.13 has it.
                    if !(c.alive && c.capital) {
                        continue;
                    }
                    let cp = c.pos.cell();
                    let per = vector_dist(sx - cp.x, sy - cp.y) / nations.max(1);
                    // The original computes this quotient into `local_34`,
                    // the **same stack slot** that holds the out-distance
                    // (set to 9 or `2k²` at step 12, and stored to
                    // `*out_dist` at the epilogue, line 502). So every
                    // capital examined overwrites the distance figure, and
                    // `Site::dist` ends up as the *last* enemy capital's
                    // `distance / num_nations` whenever this loop runs and
                    // finds one. Confirmed against run9's own record: slot
                    // 9's `dist` is 27 there and would be `2k² = 128`
                    // without the clobber.
                    // (`docs/audit/2026-08-25-ai.md`, B4-f.)
                    stats.dist = per;
                    sum = sum.wrapping_add(per);
                }
            }
            if self.lobby.team_style == 2 {
                sum = mul(sum, 100);
            }
            if sum != 0 {
                stats.val /= sum;
            }
        }
        if ally_land {
            stats.val /= 10;
        }
        stats
    }

    // ------------------------------------------------------------------
    // compute_sites — §2.7
    // ------------------------------------------------------------------

    /// `compute_sites(force)`: re-score the kept sites, sample every region
    /// where the leader has peasants, keep the best ten, rank them. Draws
    /// from the sync stream per large region (§2.7). `force` is the
    /// argument `place_city_with_cost` passes (1).
    pub fn compute_sites(&mut self, who: Player, force: bool) {
        let w = who as usize;

        if self.frame == 0 {
            self.ai[w].sites = [Site::default(); SITE_SLOTS];
        } else {
            for i in 0..SITE_SLOTS {
                let s = self.ai[w].sites[i];
                if s.rank < 6 || s.val < 1 {
                    self.ai[w].sites[i] = Site::default();
                    continue;
                }
                let reg = s.reg.clamp(0, i32::from(u16::MAX)) as u16;
                let centre = Pos::new(
                    s.wx * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                    s.wy * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                );
                let (city, unit) = self.site_reference(who, reg, centre);
                let st = self.compute_site_stats(who, s.wx, s.wy, city, unit, reg, false);
                let e = &mut self.ai[w].sites[i];
                e.val = st.val;
                e.dist = st.dist;
                e.wx = st.wx;
                e.wy = st.wy;
            }
        }

        // `param_1 != 0 || (leader_flags & 0xc) != 4` — forced, or not a
        // human leader.
        if !(force || !self.nation[w].human) {
            return;
        }

        for r in 0..self.world.region_count() {
            let reg = r as u16;
            if Census::reg(&self.ai[w].census.reg_peasants, reg) == 0 {
                continue;
            }
            let cells: Vec<Cell> = self.world.cells_in(reg).collect();
            let Some(first) = cells.first().copied() else {
                continue;
            };
            let at = Pos::new(
                first.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
                first.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            );
            let (city, unit) = self.site_reference(who, reg, at);

            let size = cells.len() as i32;
            let mut stride = 2i32;
            let mut start = 0i32;
            if size >= 200 {
                // Two draws, each skipped when its modulus is degenerate.
                let world_w = self.world.width();
                let r0 = if world_w <= 1 {
                    0
                } else {
                    self.mark(SITE_STRIDE);
                    self.rng.roll() % world_w
                };
                let den = world_w - r0 / 2;
                stride = (size / den).max(10);
                start = (self.ai[w].site_mark % stride as u32) as i32;
                let q = stride / 4;
                let r1 = if q <= 1 {
                    0
                } else {
                    self.mark(SITE_MARK);
                    self.rng.roll() % q
                };
                self.ai[w].site_mark = self.ai[w].site_mark.wrapping_add((r1 + 3) as u32);
            }

            let mut i = start;
            while i < size {
                let c = cells[i as usize];
                let st = self.compute_site_stats(who, c.x, c.y, city, unit, reg, true);
                i += stride;

                // The insertion, transliterated: the running minimum starts
                // at `(0, −1)` and is *replaced* unless the old minimum is
                // strictly smaller **and** (it is positive, or the new
                // score does not beat this slot). An empty slot's 0 is
                // always taken; a positive slot the new site beats can
                // displace an empty one already chosen.
                let mut min = 0i32;
                let mut idx: i32 = -1;
                let mut duplicate = false;
                for j in 0..SITE_SLOTS {
                    let s = self.ai[w].sites[j];
                    if s.wx == st.wx && s.wy == st.wy && s.val > 0 {
                        duplicate = true;
                        break;
                    }
                    if !(min < s.val && (min > 0 || st.val <= s.val)) {
                        min = s.val;
                        idx = j as i32;
                    }
                }
                if duplicate || idx < 0 {
                    continue;
                }
                let e = &mut self.ai[w].sites[idx as usize];
                e.wx = st.wx;
                e.wy = st.wy;
                e.val = st.val;
                e.reg = r as i32;
                e.dist = st.dist;
            }
        }

        // `rank[i]` = how many of the ten score no more than site `i` —
        // itself included, so a set of one 370 and nine zeroes ranks 10 and
        // 9. (`docs/AI.md` §2.7's "1 + count" counts the other nine.)
        for i in 0..SITE_SLOTS {
            let v = self.ai[w].sites[i].val;
            let rank = (0..SITE_SLOTS)
                .filter(|j| self.ai[w].sites[*j].val <= v)
                .count();
            self.ai[w].sites[i].rank = rank as i32;
        }
    }

    // ------------------------------------------------------------------
    // found_cities — §2.12
    // ------------------------------------------------------------------

    /// `found_cities`: the city AI — offers the best site to the make list
    /// and, under the city limit, buys it on the spot (§2.12).
    ///
    /// The type loop runs `BASE_BUILDTYPES .. 0x19f`, i.e. exactly once, on
    /// the Small City.
    pub fn found_cities(&mut self, who: Player) {
        let w = who as usize;
        // `leader_flags2 & 0x10` — the seam, never set.
        let Some(village) = self.city_record(Ident::Village) else {
            return;
        };
        let Some(village_t) = self.tables.build_types[village].tree else {
            return;
        };

        let mut want = false;
        if self.total_cities(who) < self.city_limit(who)
            && self.type_available(who, village_t)
            && (self.ai[w].census.free_peasants != 0 || self.ai[w].census.gatherers != 0)
            && !self.human_capped(who)
        {
            for i in 0..SITE_SLOTS {
                want |= self.offer_site(who, village, village_t, i);
            }
        }

        if want && self.ai[w].make_list.list[0].t != -1 && self.make_stuff(who) {
            self.ai[w].make_list.clear();
        }
    }

    /// The human cap: unless `leader_flags & 8` (the seam), an AI on
    /// difficulty 0 or 1 stops at the most cities any human holds (at least
    /// two), on 2 at one more than that; 3 and up are uncapped.
    fn human_capped(&self, who: Player) -> bool {
        let m = self.max_human_cities();
        if m <= 0 {
            return false;
        }
        let total = self.total_cities(who);
        match self.ai_difficulty() {
            0 | 1 => m.max(2) <= total,
            2 => (m + 1).max(2) <= total,
            _ => false,
        }
    }

    /// `Leaders::max_human_cities`: the most cities-plus-queued any alive
    /// human holds, less one for the Forbidden City. `−1` when there is no
    /// human.
    fn max_human_cities(&self) -> i32 {
        let mut best = -1;
        for i in 0..self.nation.len() {
            if self.defeated[i] || !self.nation[i].human {
                continue;
            }
            best = best.max(self.total_cities(i as Player));
        }
        best
    }

    /// One of the ten sites, scored into the make list. Returns whether it
    /// set `want` — i.e. whether the leader is under its own city limit.
    fn offer_site(&mut self, who: Player, village: usize, village_t: TypeId, i: usize) -> bool {
        let w = who as usize;
        let s = self.ai[w].sites[i];
        if s.val <= 0 {
            return false;
        }
        let reg = s.reg.clamp(0, i32::from(u16::MAX)) as u16;
        let c = &self.ai[w].census;
        if Census::reg(&c.reg_free_peasants, reg) == 0 && Census::reg(&c.reg_gatherers, reg) == 0 {
            return false;
        }
        // With two or three cities on a many-landmass map, expand only off
        // the home region.
        let cities = self.city_num(who);
        if !(self.world.sea_map() < 4
            || self.lobby.map_style == 0x14
            || !(2..=3).contains(&cities)
            || s.reg != self.ai[w].census.home_reg)
        {
            return false;
        }
        if !self.site_clear(who, village, s.wx, s.wy) {
            return false;
        }

        let c = &self.ai[w].census;
        let rc = Census::reg(&c.reg_cities, reg);
        let mut v = s.val;
        let cmp = if rc == 0 {
            v = mul(v, 4);
            0
        } else {
            v = mul_div(Census::reg(&c.reg_pop, reg) + 1, v, rc * 2);
            if Census::reg(&c.strategy, reg) & 1 == 0 {
                v /= 2;
            }
            rc
        };
        let queued = self.num_sites_of(who, village);
        if queued == 0 {
            let c = &self.ai[w].census;
            if Census::reg(&c.reg_pop, reg) <= Census::reg(&c.reg_peasants, reg) {
                v = mul(v, 2);
            }
            if cmp <= Census::reg_u16(&c.reg_free_peasants, reg) {
                v = mul(v, 2);
            }
        }

        // Any live city — mine, or any leader's who holds one in this
        // region — within five cells drops the site.
        for l in 0..self.players.len() {
            let other = l as Player;
            if self.defeated[l] {
                continue;
            }
            if !(other == who || self.reg_cities(other, reg) != 0) {
                continue;
            }
            for b in &self.buildings {
                if b.owner != other
                    || !b.alive
                    || !b
                        .ty
                        .is_some_and(|t| build::is_city(&self.tables.build_types, t))
                {
                    continue;
                }
                let bp = b.pos.cell();
                if vector_dist(s.wx - bp.x, s.wy - bp.y) < 5 {
                    return false;
                }
            }
        }
        if v == 0 {
            return false;
        }

        let c = &self.ai[w].census;
        let rl = Census::reg(&c.reg_land, reg);
        if rl < rc {
            v = mul(v, 2);
        }
        if rl < rc * 2 {
            v = mul_div(v, 3, 2);
        }

        let mut want = false;
        if self.total_cities(who) < self.city_limit(who) {
            want = true;
            v = mul(v, 30);
        }

        let pop_cap = self.muster[w].cap;
        if pop_cap < 200 {
            let eff = self.ai[w].effective_pop;
            if eff > pop_cap * 3 / 4 {
                // `(city_num + village_num) / 5`; `village_num` is the
                // seam (0).
                let n = cities / 5;
                v = mul_div(n + 2, v, n + 1);
            }
            if eff > pop_cap - 4 {
                v = mul_div(v, 3, 2);
            }
        }

        // A thin region I am not the weaker side in.
        if Census::reg(&self.ai[w].census.strategy, reg) & 5 == 1 {
            let n = cities;
            let p = self.ai[w].census.peasants;
            if p < n * 5 {
                if n * 3 <= p {
                    v = mul_div(v, 3, 2);
                }
            } else {
                v = mul(v, 2);
            }
            if n * 2 <= p {
                if n < 4 {
                    v = mul(v, 2);
                } else if n < 8 {
                    v = mul_div(v, 3, 2);
                }
            }
        }

        let mut vv = mul(v, 10);
        if vv < 0 {
            vv = 999_999;
        }
        if self.site_danger(who, s.wx, s.wy) > 0 {
            vv /= 3;
        }
        if self.world.owner(Cell::new(s.wx, s.wy)) != Owner::Player(who) {
            vv /= 3;
        }
        // An unaffordable site is quartered.
        let f = if self.type_affordable(who, village_t, true) > 0 {
            0x100
        } else {
            0x40
        };
        let scaled = mul(vv / 256, f) / 256;
        let val = scaled / (queued + 1);
        self.ai[w]
            .make_list
            .make_me(village_t as i32, val, 1, 9, -1, 0, 1, s.wx, s.wy);
        want
    }

    // ------------------------------------------------------------------
    // produce_city — §2.17
    // ------------------------------------------------------------------

    /// `produce_city(t, wx, wy, escrow)`: `true` when the site was placed
    /// and a citizen sent (the original's 0).
    pub fn produce_city(&mut self, who: Player, t: TypeId, wx: i32, wy: i32, escrow: i32) -> bool {
        let w = who as usize;
        let Some(rec) = self.build_record(t) else {
            return false;
        };
        let site = Cell::new(wx, wy);
        let site_reg = self.world.region_of(site);

        // The best citizen: on the map, idle, gathering or exploring, in
        // the site's region, nearest by cells — a gatherer handicapped by
        // twenty-four.
        let mut best = 99_999_999i32;
        let mut best_u: Option<usize> = None;
        // `local_10` in the original is *not* reset per candidate and is
        // read again after the loop: the census decrement below is keyed by
        // the **last examined** citizen's action, not the chosen one's.
        // Confirmed in the listing — one stack slot, `-0xc(%ebp)`, written
        // at `006cb1fb`/`006cb49b` and read at `006cb44f`.
        //
        // **And the loop walks `units[who]` by object number**
        // (`006cb172`..`006cb224`: `iVar8` from 0 to `objects +0x15c[who]`,
        // the list indexed by `o`), not this crate's slots, which a reused
        // number leaves out of order — so the last examined citizen is the
        // highest-numbered one. run1387's 4777: ours ended on `1/59` (a
        // gatherer at a reused number) and took the builder from the
        // gatherers; theirs ended on `1/179` (a builder) and took it from
        // the free (`docs/AI.md` §358.4, item 3710).
        let mut last_kind = index::NONE;
        for u in self.units_by_o(who) {
            let unit = &self.units[u];
            if !unit.alive() || !unit.on_map {
                continue;
            }
            if !self.unit_is_citizen(u) {
                continue;
            }
            last_kind = self
                .action_of(u)
                .map_or(index::NONE, |i| self.units[u].orders[i].index());
            if !matches!(last_kind, index::NONE | index::GATHER | index::EXPLORE_TO) {
                continue;
            }
            let up = self.units[u].pos.cell();
            if self.world.region_of(up) != site_reg {
                continue;
            }
            let d = vector_dist(wx - up.x, wy - up.y);
            let score = if last_kind == index::GATHER {
                d + 24
            } else {
                d
            };
            if score >= best {
                continue;
            }
            best = score;
            best_u = Some(u);
        }
        let Some(chosen) = best_u else {
            return false;
        };

        // The charge lands before the site is checked — the original's
        // order, and reachable through `place_city_with_cost`.
        let charges = self.building_price(who, rec);
        let available = self.holdings[w].available;
        cost::pay(&charges, &mut self.ledgers[w], &available, escrow != 0);
        self.economy_changed(who);

        if !self.site_clear(who, rec, wx, wy) {
            return false;
        }
        let o = self.init_build(
            who,
            rec,
            Pos::new(wx * UNITS_PER_CELL, wy * UNITS_PER_CELL),
            false,
        );

        // A nomad sends every citizen it owns; anyone else sends the one.
        // The nomad's group is the same list walked the same way
        // (`006cb3a7`..`006cb3f7`, `Group::add` in `o` order).
        let group: Vec<usize> = if self.city_num(who) == 0 {
            self.units_by_o(who)
                .into_iter()
                .filter(|u| self.units[*u].alive() && self.unit_is_citizen(*u))
                .collect()
        } else {
            vec![chosen]
        };
        for u in group {
            self.clear_orders(u);
            self.swarm_around(u, o, Body::Build(o), true);
        }

        // The census, adjusted in place. The nearest city of mine to the
        // *chosen citizen* loses the peasant.
        let cp = self.units[chosen].pos.cell();
        let centre = Pos::new(
            cp.x * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            cp.y * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        );
        let ureg = self.world.region_of(cp);
        let mut near: Option<(i32, usize)> = None;
        for (ci, c) in self.cities.iter().enumerate() {
            if !c.alive || c.owner != who || c.reg != ureg {
                continue;
            }
            let d = vector_dist(c.pos.x - centre.x, c.pos.y - centre.y);
            if near.is_none_or(|(bd, _)| d <= bd) {
                near = Some((d, ci));
            }
        }
        let reg = site_reg.unwrap_or(0);
        let gatherer = last_kind == index::GATHER;
        if self.ai[w].city_ai.len() < self.cities.len() {
            self.ai[w]
                .city_ai
                .resize(self.cities.len(), crate::ai::CityAi::default());
        }
        if let Some((_, ci)) = near {
            if gatherer {
                let g = &mut self.ai[w].city_ai[ci].gatherers;
                *g = g.wrapping_sub(1);
            } else {
                let f = &mut self.ai[w].city_ai[ci].free;
                *f = f.wrapping_sub(1);
            }
        }
        let census = &mut self.ai[w].census;
        let slot = reg as usize;
        if gatherer {
            census.gatherers -= 1;
            if let Some(x) = census.reg_gatherers.get_mut(slot) {
                *x -= 1;
            }
        } else {
            census.free_peasants -= 1;
            if let Some(x) = census.reg_free_peasants.get_mut(slot) {
                *x -= 1;
            }
        }
        true
    }

    /// `ScenarioFuncSet::place_city_with_cost`'s body: a sweep, then the city
    /// AI, with the **make list cleared on both sides of it**.
    /// `ai_host.rs`'s `Sim::place_city_ai` is the seam that should call this.
    ///
    /// The listing is five calls between the city-limit guard and the return
    /// (`009f58bd`..`009f58e1`): `push $0` / `compute_sites`,
    /// `MakeList::clear`, `found_cities`, `MakeList::clear`. Two things it
    /// settles that the prose had wrong:
    ///
    /// - **`compute_sites` is called with `force = 0`**, not 1 — the `push
    ///   $0x0` at `009f58bd`. For an AI leader the two are the same (§2.7's
    ///   gate is `force || !human`), so nothing in a traced game moves; for a
    ///   human-driven leader they are not.
    /// - **The list is cleared before the offers and again after them.** The
    ///   pre-clear is what makes `found_cities`' make list *only* the city
    ///   offers, so `make_stuff`'s head is a city and its expiry walk
    ///   (`make_stuff+0x221`) counts city slots and nothing else; the
    ///   post-clear is unconditional, where `found_cities`' own tail clears
    ///   only when it bought.
    pub fn place_city_ai_impl(&mut self, who: Player) {
        self.compute_sites(who, false);
        self.ai[who as usize].make_list.clear();
        self.found_cities(who);
        self.ai[who as usize].make_list.clear();
    }
}

#[cfg(test)]
mod tests;
