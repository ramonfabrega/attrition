//! `Leader::calc_gather`'s **inputs**: filling [`economy::Holdings`] from the
//! live simulation.
//!
//! `crates/sim/src/economy.rs` is the arithmetic — the rate in sixteenths, the
//! caps, the accrual, the 512-frame cadence. It reads a [`economy::Holdings`], and
//! until this module existed nothing in the simulation ever wrote one: every
//! test hand-built it and the replay harness ran whole games in which both
//! players earned `[0, 0, 0, 0, 0, 0]` while their citizens stood on farms.
//! This is the missing half — the walk over cities, buildings and units that
//! `Leader::calc_gather@006ceee0` does at the top of its own recompute.
//!
//! # The original's walk, and what each `Holdings` field comes from
//!
//! `Leader::calc_gather` runs under a cadence ([`economy::should_recompute`])
//! and, when it runs, rebuilds all six rates from scratch. Its sources, in
//! its own order:
//!
//! | Step | Original | Here |
//! | --- | --- | --- |
//! | 3 | every **city** — `City::calc_gather@00737c60` → `LeaderData::calc_city_resources@006d5530` | [`Sim::city_holdings`] |
//! | 4 | every **oil well**, the `oil_wells` list | [`Sim::gather_sites_outside_cities`] |
//! | 5 | every **camp/mine outside a city** (`0x1a2`, `0x1a3`, `city < 0`) | [`Sim::gather_sites_outside_cities`] |
//! | 6 | every idle fisherman and merchant, `Unit::do_gather` | [`Sim::gather_rares`] ([`crate::rares`]) |
//! | 7 | refineries — `get_buildings(REFINERY)` | [`economy::Holdings::refineries`] |
//! | 11 | the territory tax | `Sim::update_territory_holdings` (untouched) |
//!
//! A **site** is a building. `calc_city_resources` walks the city's member
//! chain and calls `BuildData::calc_gather` on every member that is active,
//! whose type `is_gather_type` (`build_flags & 0x40`, [`build::flags::GATHER`]),
//! that is not an oil well or platform (those are counted once at the player
//! level, from the `oil_wells` list, whether or not they sit in a city), and
//! whose `city` link is this city. `BuildData::calc_gather@0062d360` then
//! passes `num_gatherers(1, 1)` down — **arrived and skipping decoys** — which
//! is exactly [`Sim::num_gatherers`]`(b, true, true)`. So a citizen that has
//! been ordered onto a farm and is still walking earns nothing; the rate
//! begins on the frame `been_there` flips (`docs/ORDERS.md` §6.3).
//!
//! # When the original reassembles
//!
//! `calc_gather` returns immediately unless the cadence says so, and the
//! cadence has a fast path: `LeaderData`'s flag `0x2000000`, which drops the
//! period from 512 frames to 8. Its writers, from
//! `grep -rn '| 0x2000000' decomp/funcs` — every one of them a place the
//! *inputs* above changed:
//!
//! - `Unit::do_gather@005ef2a0:279` and `Unit::do_non_flat_gather@005f0170:100,392,508`
//!   — **a gatherer arriving**, on the same statement that sets `been_there`.
//!   This is the one that answers "does a citizen arriving at a farm set it":
//!   it does, and the income appears within eight frames of the arrival.
//! - `Unit::work@0060d180:268` — a unit that has an order again dropping the
//!   idle latch (`ObjectData + 0x8 & 8`), for a **fisherman** (`is(0x13d)`) or
//!   a merchant (`type ∈ {0x3d, 0x3e, 0x190}`).
//! - `Unit::check_idle@006032c0:74` — the same kinds *taking* that latch, on
//!   their first idle frame. The pair is what step 6's membership turns on, so
//!   both are modelled ([`crate::rares`]); an earlier draft of this list had
//!   them as "a peasant, a scholar or a merchant" and missed the fisherman,
//!   which is the one that matters — it is the whole of the fishing economy.
//! - `Unit::kill_current_order@005e2cb0:82` — a `GATHER` order (`type == 7`)
//!   being killed.
//! - `Unit::init@00612100:626,639`, `Unit::close@0060ee50:154,168,174`,
//!   `Unit::come_out@00617c10:312`, `Unit::action_come_out@005e20b0:42`,
//!   `Object::insert_inside@00647e90:94`, `Object::remove_from_inside@006480f0:117`
//!   — a scholar entering or leaving a university, a gatherer born or dying.
//! - `Build::activate@00623e20:398,402,443` and `Build::close@00628980:95`,
//!   `Wall::init@0063e9b0:36`, `Wall::activate@0063e4b0:123` — **a building
//!   finished or lost**.
//! - `Leader::gain_tech@006dcb60:319` — any tech.
//! - `Cities::capture_city@00733380:174,175` (both leaders),
//!   `City::assimilate@00738e90:63`, `City::compute_trade@00739640:44`.
//! - `Leader::init@006e3930:145` — so frame 0 is always a recompute.
//! - `Group::action_alarm@0070ec30:103`, `action_alarm_peasant@006fd980:83`,
//!   the `SpellType::cast_*` conversions, and the scenario scripts'
//!   `buildings_gather_{enable,disable}`.
//!
//! So it is **both**: a building finishing sets it and a citizen arriving sets
//! it. [`Sim::economy_changed`] is this crate's name for the flag, and
//! `crates/sim/src/orders.rs` raises it on the arrival and on both halves of
//! the idle latch; the building half is listed as an open edit in this
//! module's notes.
//!
//! # Arithmetic
//!
//! Counting only. Every number this module produces is a count, a percentage
//! taken straight from `Tuning`, or a level; the sixteenths and the
//! truncations all live in `economy.rs`.

use crate::Sim;
use crate::build::{self, Ident};
use crate::economy::{self, Resource, Site};
use crate::tech::Line;
use crate::world::Player;

/// The `BONUS` types the enhancer and taxation levels are read from.
///
/// `CityData::granary_level@00736890` is `get_granary` gated on the city's
/// flag; `LeaderData::get_granary@006db340` is a ladder of
/// `has_preq(GRANARY5 … GRANARY2)` returning 5…2 and **1** when none is held.
/// `lumber_level@00736820` (LUMBERMILL4…2) and `get_smelter` are the same
/// shape, `get_university@006db1f0` runs to 6, and
/// `get_taxation@006d6e20` is the odd one out: `TAX_4…TAX_1` returning 4…1 and
/// **0** when none is held.
///
/// **These five were a seam until item 232.** `GRANARY2` is `TypeIndex`
/// `0x2bb` and `BASE_BONUSTYPES` is `0x2ac`, so all of them — `GRANARY2..5`,
/// `LUMBERMILL2..4`, `SMELTER2..4`, `UNIVERSITY2..6`, `TAX_1..4` — are
/// *bonus* types, and the tree read them off nowhere: [`Levels::for_player`]
/// answered [`Levels::BASE`] whatever the player held, so `taxation` was 0
/// and the territory tax identically zero for the life of every game
/// (`docs/audit/2026-09-05-economy-vs-code.md` R8). They are now loaded like
/// the two ladders beside them — [`crate::tech::Roles::granary_preq`] and its
/// four siblings, from the same `TECHBONUSES` list `FISHERMEN` and
/// `MERCHANTS` come from — and read here through
/// [`Sim::bonus_level`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Levels {
    /// `LeaderData::get_granary`, 1..=5.
    pub granary: i32,
    /// `CityData::lumber_level`, 1..=4.
    pub lumber_mill: i32,
    /// `LeaderData::get_smelter`, 1..=4.
    pub smelter: i32,
    /// `LeaderData::get_university`, 1..=6 — the level `SCHOLAR_RATE` is
    /// indexed by. A **leader** figure, not a per-university one:
    /// `BuildTypeData::calc_gather@00639e40:878` reads it off the leader.
    pub university: i32,
    /// `LeaderData::get_taxation`, 0..=4.
    pub taxation: usize,
}

impl Levels {
    /// What every ladder answers with no bonus type held: one everywhere,
    /// except taxation, which is zero.
    pub const BASE: Levels = Levels {
        granary: 1,
        lumber_mill: 1,
        smelter: 1,
        university: 1,
        taxation: 0,
    };

    /// The levels for one player: each ladder's highest row whose
    /// prerequisite the player holds.
    ///
    /// The four enhancers floor at **1** and taxation at **0**, which is the
    /// only difference between `LeaderData::get_granary@006db340`,
    /// `CityData::lumber_level@00736820`, `get_smelter@006db3f0` and
    /// `get_university@006db1f0` on one side and
    /// `get_taxation@006d6e20` on the other. A tree that names none of the
    /// rows — every fixture that builds its own — therefore still answers
    /// [`Levels::BASE`], so this is a widening and not a change of default.
    ///
    /// `lumber_level` and `granary_level` are `CityData` methods gated on the
    /// city's own `city_flags & 0x400` / `& 0x200`; that gate is
    /// [`Sim::city_holdings`]' `any(Ident::Lumbermill)` and
    /// `any(Ident::Granary)`, and only the leader half is here.
    pub fn for_player(sim: &Sim, who: Player) -> Levels {
        debug_assert!((who as usize) < sim.holdings.len(), "no such player");
        let roles = &sim.tech_tree.roles;
        Levels {
            granary: 1 + sim.bonus_level(who, &roles.granary_preq) as i32,
            lumber_mill: 1 + sim.bonus_level(who, &roles.lumbermill_preq) as i32,
            smelter: 1 + sim.bonus_level(who, &roles.smelter_preq) as i32,
            university: 1 + sim.bonus_level(who, &roles.university_preq) as i32,
            taxation: sim.bonus_level(who, &roles.taxation_preq),
        }
    }
}

/// `CityData::lumber_level@00736820`: 0 unless the city's flags carry `0x400` (a Lumber Mill), else one plus
/// the [`crate::rares::top_held`] of `LUMBERMILL2..4` — `has_preq(i)` asked for row `i`, the top first. The
/// leader half is [`Levels::for_player`]'s `lumber_mill`; item 2090's sweep holds the gate and the asks.
pub fn lumber_level_by(city_flags: i32, has_preq: impl FnMut(usize) -> bool) -> i32 {
    if city_flags & 0x400 == 0 {
        return 0;
    }
    1 + crate::rares::top_held(3, has_preq) as i32
}

/// `CityData::smelter_level@007367a0`: the enhancer of the fifth good type's kind (`goodtypes.list[4] + 4`, the
/// metal good, kind 4) — 0 `GRANARY`, 1 `LUMBERMILL`, 4 `SMELTER`, 5 `REFINERY` (`0x1a7..=0x1aa`), any other kind 0
/// — asked of the city's `count_buildings(t, 0, 1)`; none answers 0, one or more `LeaderData::get_smelter`. Item
/// 2112's sweep holds the table, the gate and the order; [`Sim::city_holdings`] asks it for the metal kind.
pub fn smelter_level_by(
    kind: i32,
    count_buildings: impl FnOnce(i32) -> bool,
    get_smelter: impl FnOnce() -> i32,
) -> i32 {
    let t = match kind {
        0 => 0x1a7,
        1 => 0x1a8,
        4 => 0x1a9,
        5 => 0x1aa,
        _ => return 0,
    };
    if !count_buildings(t) {
        return 0;
    }
    get_smelter()
}

/// `GoodTypeData::get_enhancer@0066fcf0`: the building type that enhances a
/// good of `kind` — 0 `GRANARY`, 1 `LUMBERMILL`, 4 `SMELTER`, 5 `REFINERY`
/// (`0x1a7..=0x1aa`), any other kind -1. The shipped goods' kinds are their
/// [`Resource`] index.
pub fn good_enhancer(kind: i32) -> i32 {
    match kind {
        0 => 0x1a7,
        1 => 0x1a8,
        4 => 0x1a9,
        5 => 0x1aa,
        _ => -1,
    }
}

/// The metal good's kind, which `smelter_level` reads off the good-type list (`GoodTypeData`'s `+0x4`).
const METAL_KIND: i32 = 4;

impl Default for Levels {
    fn default() -> Levels {
        Levels::BASE
    }
}

/// What `BuildData::calc_gather@0062d360` hands
/// `BuildTypeData::calc_gather@00639e40` beside its own arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TypeGatherArgs {
    /// `WallData::tile_corner`'s two tile coordinates.
    pub corner: (i32, i32),
    /// `1` when the caller forces it (`param_5`), else `is_active`'s answer
    /// (the base `WallData::is_active@00472350` inlined as `flags & 4`).
    pub active: i32,
    /// `num_gatherers(1, 1)` — arrived, skipping decoys.
    pub gatherers: i32,
    /// `count_inside(COUNT_NON_DECOY_TYPE, 0x34)` for a university
    /// (`is(0x1a4)`), else 0.
    pub scholars: i32,
}

/// `BuildData::calc_gather@0062d360`, its steps in the listing's order
/// (`62d373` the corner, `62d378..62d399` the active flag, `62d3a5..62d3c4`
/// the university's scholars, `62d3da` the gatherers, `62d403` the type's
/// own `calc_gather`), the callees as closures. Answers the count
/// out-parameter clamped at zero (`62d408..62d40d`).
pub(crate) fn build_calc_gather_steps(
    forced_active: bool,
    tile_corner: impl FnOnce() -> (i32, i32),
    is_active: impl FnOnce() -> i32,
    is_university: impl FnOnce() -> bool,
    count_scholars: impl FnOnce() -> i32,
    num_gatherers: impl FnOnce() -> i32,
    type_calc_gather: impl FnOnce(TypeGatherArgs) -> i32,
) -> i32 {
    let corner = tile_corner();
    let active = if forced_active { 1 } else { is_active() };
    let scholars = if is_university() { count_scholars() } else { 0 };
    let gatherers = num_gatherers();
    let count = type_calc_gather(TypeGatherArgs {
        corner,
        active,
        gatherers,
        scholars,
    });
    count.max(0)
}

impl Sim {
    /// The resource a building gathers — `BuildTypeData::get_good`, as far as
    /// the shipped rows use it.
    ///
    /// The gate is `is_gather_type` (`build_flags & 0x40`); the answer is the
    /// type's `GOOD` column, which this crate's [`Ident`] lineage stands in
    /// for. Those are the only six gather lineages the shipped data has —
    /// `crates/sim/src/gather.rs` reaches the same list from the other end,
    /// where farm, oil well and oil platform are the flat ones and the camp,
    /// the mine and the university are not.
    pub fn gather_good(&self, b: usize) -> Option<Resource> {
        let t = self.buildings[b].ty?;
        if !self.build_types[t].has(build::flags::GATHER) {
            return None;
        }
        let is = |i| build::is(&self.build_types, t, i);
        if is(Ident::OilWell) || is(Ident::OilPlatform) {
            Some(Resource::Oil)
        } else if is(Ident::Farm) {
            Some(Resource::Food)
        } else if is(Ident::Woodcutter) {
            Some(Resource::Timber)
        } else if is(Ident::Mine) {
            Some(Resource::Metal)
        } else if is(Ident::University) {
            Some(Resource::Knowledge)
        } else {
            None
        }
    }

    /// Whether a building is standing and finished — the pair
    /// `count_buildings` and `num_buildings` both insist on (`flags & 1`
    /// alive, `WallData::is_active`).
    fn counts_for_economy(&self, b: usize) -> bool {
        self.buildings[b].alive && self.buildings[b].active
    }

    fn is_kind(&self, b: usize, ident: Ident) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| build::is(&self.build_types, t, ident))
    }

    /// One city, as income sees it — `City::calc_gather` and the part of
    /// `LeaderData::calc_city_resources` that reads the city rather than the
    /// leader.
    ///
    /// The three enhancer bytes are filled here because that is where the
    /// original fills them: `City::calc_gather` writes all four every time it
    /// runs, before anything reads them, and the fourth — oil — it writes as
    /// an unconditional zero. Their gates differ and all three are read at
    /// their source: the granary on the city's flag `0x200`, the lumber mill
    /// on `0x400`, the smelter on `count_buildings(SMELTER)`. The two flags
    /// are not independent state — `Build::add_to_city@00622380:83,103` sets
    /// them when an **active** granary or lumber mill joins and
    /// `Build::remove_from_city@00622030` clears them when the last one
    /// leaves — so all three gates are the same predicate and are written as
    /// one here.
    pub fn city_holdings(&self, c: usize, levels: Levels) -> economy::City {
        let mut out = economy::City::default();
        let chain: Vec<usize> = self
            .city_chain(c)
            .into_iter()
            .filter(|&b| self.counts_for_economy(b))
            .collect();
        let any = |ident: Ident| chain.iter().any(|&b| self.is_kind(b, ident));

        // `CityData::num_buildings`, for `BUILDING_TAXES`. The city centre is
        // the head of the chain, so it counts.
        out.buildings = i32::try_from(chain.len()).unwrap_or(i32::MAX);

        // The caravan income the city already carries — a field, not a
        // survey: `City::compute_trade` writes it when a route changes.
        out.trade_val = self.cities[c].trade_val;

        // `CityData::get_taxes` and `get_literacy`, which are
        // `count_buildings(MARKET | TEMPLE)` and
        // `count_buildings(UNIVERSITY | LIBRARY)`.
        out.market = any(Ident::Market);
        // `get_taxes@00737b50`: the Porcelain Tower raises a market's taxes
        // (`docs/GOLDEN.md` §96, item 2598).
        out.porcelain = out.market
            && self.wonders_held(self.cities[c].owner)
                & (1 << crate::tech::wonder::PORCELAIN_TOWER)
                != 0;
        out.temple = any(Ident::Temple);
        out.university = any(Ident::University);
        out.library = any(Ident::Library);

        // `City::calc_gather@00737c60`'s four bytes ([`economy::calc_gather_steps`],
        // item 2368's sweep): the flags `0x200` / `0x400` are the granary and
        // lumber mill standing, the metal and oil goods' enhancers are
        // `get_enhancer` of kinds 4 and 5, and `calc_city_resources` is the
        // rest of this function.
        let enh = |t: i32| match t {
            0x1a7 => any(Ident::Granary),
            0x1a8 => any(Ident::Lumbermill),
            0x1a9 => any(Ident::Smelter),
            0x1aa => any(Ident::Refinery),
            _ => false,
        };
        let flags = if any(Ident::Granary) { 0x200 } else { 0 }
            | if any(Ident::Lumbermill) { 0x400 } else { 0 };
        let tuning = &self.tuning;
        let bytes = economy::calc_gather_steps(
            &mut (),
            flags,
            |_| levels.granary.max(1),
            |_| levels.lumber_mill.max(1),
            |_, r| good_enhancer(r.index() as i32),
            |_, t| i32::from(enh(t)),
            |_| smelter_level_by(METAL_KIND, enh, || levels.smelter.max(1)),
            |_, r, lvl| economy::enhancer_bonus(tuning, r, lvl),
            |_, _| {},
        );
        out.granary = i32::from(bytes[0]);
        out.lumber_mill = i32::from(bytes[1]);
        out.smelter = i32::from(bytes[2]);

        for &b in &chain {
            // The oil exclusion is the original's: an oil well or platform in
            // a city is skipped here and picked up once, at the player level,
            // from the `oil_wells` list.
            if self.is_kind(b, Ident::OilWell) || self.is_kind(b, Ident::OilPlatform) {
                continue;
            }
            let Some(resource) = self.gather_good(b) else {
                continue;
            };
            out.sites.push(self.site_of(b, resource, levels));
        }
        out
    }

    /// One gather building as a [`Site`] — the `num_gatherers(1, 1)` that
    /// `BuildData::calc_gather` passes down, and the university level that
    /// selects `SCHOLAR_RATE`.
    fn site_of(&self, b: usize, resource: Resource, levels: Levels) -> Site {
        let bd = &self.buildings[b];
        let mut gatherers = 0;
        // `calc_city_resources` passes `param_5 = 0`, so the building's own
        // `is_active` is asked. The count out-parameter is `gather.rs`'s
        // survey ([`Sim::max_gatherers`]); income takes the gatherers.
        build_calc_gather_steps(
            false,
            || {
                bd.ty.map_or((0, 0), |t| {
                    let c = self.tile_corner(t, bd.pos);
                    (c.x, c.y)
                })
            },
            || i32::from(bd.active),
            || self.is_kind(b, Ident::University),
            || {
                bd.garrison
                    .iter()
                    .filter(|&&u| self.worker_of(u) == crate::orders::Worker::Scholar)
                    .filter(|&&u| !self.units[u].decoy)
                    .count() as i32
            },
            || self.num_gatherers(b, true, true),
            |h| {
                gatherers = h.gatherers;
                0
            },
        );
        // `calc_gather:978`–`985`: the flat branch's food, for a Japanese
        // leader (item 3331).
        let nation_pct = if resource == Resource::Food
            && self.is_kind(b, Ident::Farm)
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[bd.owner as usize],
                crate::nations::power::JAPANESE,
            ) {
            self.tuning.japanese_fishing_boats
        } else {
            0
        };
        // `calc_gather:997`–`1005`: a Farm's wealth, for an Egyptian leader
        // (item 3332).
        let farm_wealth = if self.is_kind(b, Ident::Farm)
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[bd.owner as usize],
                crate::nations::power::EGYPTIANS,
            ) {
            self.tuning.egyptian_farm_wealth
        } else {
            0
        };
        // `calc_gather:850`–`856`: a mine's wealth per gatherer, for an Inca
        // leader (item 3360). The branch is the mine's — metal, beside
        // `german_miners` — and the constant must be positive.
        let miner_wealth = if resource == Resource::Metal
            && self.tuning.inca_wealth_per_miner > 0
            && self.tech_tree.has_tribe_bonus(
                &self.setup,
                &self.tech[bd.owner as usize],
                crate::nations::power::INCA,
            ) {
            self.tuning.inca_wealth_per_miner
        } else {
            0
        };
        Site {
            resource,
            gatherers,
            level: if resource == Resource::Knowledge {
                levels.university.max(1)
            } else {
                1
            },
            nation_pct,
            farm_wealth,
            miner_wealth,
        }
    }

    /// Steps 4 and 5: the gather buildings that pay outside any city.
    ///
    /// Every active oil well and oil platform, wherever it stands — the
    /// `oil_wells` list is per player and the city loop skips them, so they
    /// are counted here exactly once — plus every woodcutter's camp and mine
    /// whose `city` link is negative.
    ///
    /// **These are returned rather than stored**, because [`economy::Holdings`] has no
    /// slot for them: it is a `Vec<economy::City>`, and folding a stray camp
    /// into a city would hand it that city's `CITY_GATHER` and its lumber
    /// mill's percentage, neither of which the original gives it
    /// (`BuildTypeData::calc_gather` leaves the enhancer at 100 when the city
    /// index is negative). Adding `Holdings::outside: Vec<Site>` and one loop
    /// in `economy::assemble` closes it; see this module's notes.
    pub fn gather_sites_outside_cities(&self, who: Player) -> Vec<Site> {
        let levels = Levels::for_player(self, who);
        let mut out = Vec::new();
        for b in 0..self.buildings.len() {
            if self.buildings[b].owner != who || !self.counts_for_economy(b) {
                continue;
            }
            let Some(resource) = self.gather_good(b) else {
                continue;
            };
            let oil = self.is_kind(b, Ident::OilWell) || self.is_kind(b, Ident::OilPlatform);
            if !oil && self.buildings[b].city.is_some() {
                continue;
            }
            out.push(self.site_of(b, resource, levels));
        }
        out
    }

    /// `get_buildings(REFINERY)`: refineries anywhere, which scale oil at the
    /// player level rather than per city.
    ///
    /// `LeaderData::get_buildings@006e0680` sums a per-type census over the
    /// type's upgrade chain; the refinery has no upgrade, so the lineage test
    /// is the same answer.
    /// `LeaderData::has_wonder@006ebc10` for every wonder at once, a bit
    /// per [`crate::tech::wonder`] offset.
    ///
    /// The original walks the player's wonder list: an entry is written by
    /// `Wonders::init_wonder` from `Build::activate`'s wonder arm and
    /// cleared by `close_wonder` from `Build::close`, and it holds when its
    /// object is of the asked type and stands in a city (`city >= 0`,
    /// `+0x72`) — except the Red Fort, which holds without one. So it is
    /// an activated, standing wonder of the type, which is what this reads
    /// off the buildings. The Conquer-the-World arm (`conquest_wonders`,
    /// `local_8 = 2`) is cut from v1. `docs/ECONOMY.md` §15.
    pub fn wonders_held(&self, who: Player) -> u32 {
        let line = &self.tech_tree.roles.wonder_line;
        let mut held = 0u32;
        for b in &self.buildings {
            if b.owner != who || !b.alive || !b.active {
                continue;
            }
            let Some(tree) = b.ty.and_then(|t| self.build_types[t].tree) else {
                continue;
            };
            let Some(k) = line.iter().position(|&w| w == tree) else {
                continue;
            };
            if k == crate::tech::wonder::RED_FORT || b.city.is_some() {
                held |= 1 << k;
            }
        }
        held
    }

    pub fn refinery_count(&self, who: Player) -> i32 {
        let n = (0..self.buildings.len())
            .filter(|&b| self.buildings[b].owner == who)
            .filter(|&b| self.counts_for_economy(b))
            .filter(|&b| self.is_kind(b, Ident::Refinery))
            .count();
        i32::try_from(n).unwrap_or(i32::MAX)
    }

    /// Rebuilds the derivable half of one player's [`economy::Holdings`] from the live
    /// state — the walk `Leader::calc_gather` does before it starts summing.
    ///
    /// Writes `cities`, `refineries`, `commerce`, `handicap` and — since
    /// item 232 — `taxation`, from the `levels` it is handed. **Leaves
    /// alone** the fields the simulation maintains elsewhere or has no
    /// source for: `territory` and `land_size`
    /// (`Sim::update_territory_holdings`), `available` and `discovered` (the
    /// tech layer) and `bonus_cap` (scenario scripts only —
    /// `ScenarioFuncSet::set_bonus_cap` is its one writer in the executable).
    pub fn assemble_holdings(&mut self, who: Player) {
        let levels = Levels::for_player(self, who);
        self.assemble_holdings_with(who, levels);
    }

    /// [`Sim::assemble_holdings`] with the enhancer levels supplied.
    pub fn assemble_holdings_with(&mut self, who: Player, levels: Levels) {
        let w = who as usize;

        // `known_rares = Σ reg_known_rares`, the first thing
        // `calc_gather@006ceee0:66–75` does past its gate — the census
        // writes the array and this is the sum's only writer
        // (`docs/AI.md` §55).
        if let Some(a) = self.ai.get_mut(w) {
            a.known_rares = a.census.reg_known_rares.iter().sum();
        }

        let cities: Vec<economy::City> = self
            .cities_of(who)
            .into_iter()
            .map(|c| self.city_holdings(c, levels))
            .collect();
        let refineries = self.refinery_count(who);

        // `COMMERCE_CAP[commerce_level]`, and the level is
        // `LeaderDataEncrypt::epoch[2]` — `calc_resource_caps@006ce900` reads
        // it at `+0xf0`, which is `epoch + 2 * 4` off `+0xe8`, and the
        // engine's own category order puts Commerce at 2.
        let commerce = self.tech[w].epoch[Line::Commerce.index()].max(0) as usize;
        // The republic term's tier: `has_preq(REPUBLIC_3)`, then 2, then 1
        // (`calc_resource_caps@006ce900`, `docs/AI.md` §72).
        let republic = self.government_bonus_level(who, &self.tech_tree.roles.republic_preq);

        // `LeaderData::get_gather_handicap@006d66a0`: a human takes zero
        // unless the multiplayer handicap option is on, and an AI takes the
        // difficulty's percentage. `semaphore[0] & 4` — the MP handicap — is
        // not modelled, so a human is flatly zero here.
        let handicap = if self.nation[w].human {
            0
        } else {
            let d = self.lobby.difficulty.clamp(0, i32::from(u8::MAX));
            economy::gather_handicap(u8::try_from(d).unwrap_or(0))
        };

        let outside = self.gather_sites_outside_cities(who);
        // Step 6, and the `rare_owned` it rebuilds ([`crate::rares`]).
        let (rares, rare_owned) = self.gather_rares(who);
        let lakota = self.nation[w].lakota && self.tuning.lakota_food != 0;
        let iroquois = self.tech.get(w).is_some_and(|p| {
            self.tech_tree
                .has_tribe_bonus(&self.setup, p, crate::ai::tribe::IROQUOIS)
        });
        let dutch = self.tech.get(w).is_some_and(|p| {
            self.tech_tree
                .has_tribe_bonus(&self.setup, p, crate::ai::tribe::DUTCH)
        });
        let starting = std::array::from_fn(|g| self.starting_good(who, g));
        let capitalism = self.holds_capitalism_bonus(who);
        let lakota_units = if lakota {
            self.lakota_food_units(who)
        } else {
            0
        };
        let h = &mut self.holdings[w];
        h.cities = cities;
        h.outside = outside;
        h.rares = rares;
        h.rare_owned = rare_owned;
        h.refineries = refineries;
        h.commerce = commerce;
        h.republic = republic;
        h.handicap = handicap;
        h.taxation = levels.taxation;
        h.british = self.nation[w].british;
        h.mongols = self.nation[w].mongols;
        h.num_nations = self.players.len() as i32;
        h.lakota = lakota;
        h.lakota_units = lakota_units;
        h.capitalism = capitalism;
        h.romans = self.nation[w].romans;
        h.germans = self.nation[w].germans;
        h.egyptians = self.nation[w].egyptians;
        h.russians = self.nation[w].russians;
        h.french = self.nation[w].french;
        h.inca = self.nation[w].inca;
        h.iroquois = iroquois;
        h.dutch = dutch;
        h.starting = starting;
    }

    /// `has_preq(CAPITALISM_1)` (`0x325`, `calc_gather`'s `006cf454`): the
    /// first Capitalism bonus, whose prerequisites are
    /// `gov_bonuses`' second row — `get_gov`'s own order, Socialism,
    /// Capitalism, Monarchy, Democracy (`crates/rondata/src/load.rs`). A
    /// first-tier bonus asks nothing past its row.
    pub(crate) fn holds_capitalism_bonus(&self, who: Player) -> bool {
        let p = &self.tech[who as usize];
        self.tech_tree
            .roles
            .gov_bonuses
            .get(1)
            .is_some_and(|(preqs, _)| {
                preqs
                    .iter()
                    .all(|&q| self.tech_tree.has_tech_p(&self.setup, p, q))
            })
    }

    /// The count `Leader::calc_gather@006ceee0:116`'s Lakota arm multiplies:
    /// `stable_units − scout_garr − stable_garr − peasants_garr + scouts +
    /// peasants` (`docs/AI.md` §219).
    ///
    /// The original keeps all six live: `Leader::track_unit_type@006e0dd0`
    /// moves `peasants` (`is_peasant`), `scouts` (`role & 0x10`; both
    /// the census's) and `stable_units` (a type with `+0x1e8` set whose `+0x40` is the
    /// Stable, `0x1ac`), and `Object::insert_inside@00647e90` and
    /// `remove_from_inside@006480f0` move the three garrison counts for a
    /// captain in a building — a peasant, else a Stable type (no `+0x1e8`
    /// test there), else `UnitData::is_special` (`+0x2b8 & 0x10`, the
    /// scout bit). The sweep also zeroes and recounts `peasants`; the
    /// Stable and garrison counts are read off the units here, which is the
    /// same number at the moment the rate is assembled.
    pub(crate) fn lakota_food_units(&self, who: Player) -> i32 {
        let w = who as usize;
        let c = &self.ai[w].census;
        let mut n = c.peasants + c.scouts - c.peasants_garr - c.stable_garr - c.scout_garr;
        for t in 0..self.unit_types.len() {
            let k = self.muster[w].by_type[t];
            if k != 0
                && self.unit_types[t].combat.attack != 0
                && self.trainer_ident(t) == Some(crate::build::Ident::Stable)
            {
                n += k;
            }
        }
        n
    }

    /// Whether this player's rate would be reassembled on this frame, so the
    /// holdings need rebuilding first.
    ///
    /// The same predicate `economy::process` uses, which is the point: the
    /// walk is the expensive half and the original does it inside the same
    /// gate, so income becomes visible on exactly the frame it does there.
    pub fn holdings_due(&self, who: Player, frame: i64) -> bool {
        let l = &self.ledgers[who as usize];
        economy::should_recompute(frame, who, l.gather_stamp, l.dirty)
    }
}

/// What a player's income would be if it were assembled right now, in
/// sixteenths — the assembly without the frame.
///
/// Nothing in the simulation calls this; it exists so a test or the replay
/// harness can ask "what is the rate?" without waiting for the cadence.
pub fn rate_now(sim: &mut Sim, who: Player) -> [i32; economy::RESOURCES] {
    sim.assemble_holdings(who);
    economy::assemble(&sim.tuning, &sim.holdings[who as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{RATE_SCALE, RESOURCES};
    use crate::orders::{QueuePos, Worker};
    use crate::tuning::Tuning;
    use crate::world::{Cell, Player as P, Pos, Terrain, World};
    use crate::{Unit, UnitType, build::BuildType, build::flags, cost, movement};

    const TILE: i32 = crate::world::UNITS_PER_TILE;

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
    }

    fn world_sim() -> Sim {
        let mut w = World::new(16, 16);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(15, 15));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; RESOURCES];
        }
        // Both players are people: no difficulty handicap in these tests.
        for n in &mut sim.nation {
            n.human = true;
        }
        sim
    }

    fn bt(ident: Ident, flag: &str, xs: i32, ys: i32) -> BuildType {
        BuildType {
            ident,
            x_size: xs,
            y_size: ys,
            flags: flags::parse(flag),
            job_time: 150,
            hits: 400,
            price: cost::Price {
                kind: cost::Kind::Building,
                ..cost::Price::free()
            },
            ..BuildType::default()
        }
    }

    struct Types {
        village: usize,
        farm: usize,
        camp: usize,
        university: usize,
        library: usize,
        market: usize,
        granary: usize,
        oil_well: usize,
        refinery: usize,
        citizen: usize,
        scholar: usize,
    }

    fn install(sim: &mut Sim) -> Types {
        let village = sim.add_build_type(bt(Ident::Village, "ean", 7, 7));
        let mut farm = bt(Ident::Farm, "gda", 4, 4);
        farm.flags |= flags::FLAT;
        let farm = sim.add_build_type(farm);
        // `e` no city, `f` territory-exempt: a camp can stand in the wild.
        let camp = sim.add_build_type(bt(Ident::Woodcutter, "gaef", 4, 4));
        let university = sim.add_build_type(bt(Ident::University, "gja", 4, 4));
        let library = sim.add_build_type(bt(Ident::Library, "ja", 5, 5));
        let market = sim.add_build_type(bt(Ident::Market, "ja", 4, 4));
        let granary = sim.add_build_type(bt(Ident::Granary, "ja", 4, 4));
        let mut oil = bt(Ident::OilWell, "ga", 4, 4);
        oil.flags |= flags::FLAT;
        let oil_well = sim.add_build_type(oil);
        let refinery = sim.add_build_type(bt(Ident::Refinery, "ja", 4, 4));

        let citizen = sim.add_unit_type(worker_type(village, Worker::Citizen));
        let scholar = sim.add_unit_type(worker_type(village, Worker::Scholar));
        Types {
            village,
            farm,
            camp,
            university,
            library,
            market,
            granary,
            oil_well,
            refinery,
            citizen,
            scholar,
        }
    }

    fn worker_type(village: usize, worker: Worker) -> UnitType {
        UnitType {
            price: cost::Price {
                pop: 1,
                ..cost::Price::free()
            },
            hits: 40,
            worker,
            garrison: crate::garrison::UnitTraits {
                trained_at: Some(village),
                ..crate::garrison::UnitTraits::default()
            },
            ..UnitType::default()
        }
    }

    fn spawn(sim: &mut Sim, owner: P, ty: usize, pos: Pos) -> usize {
        let index = i16::try_from(sim.units.len()).unwrap();
        let hits = sim.unit_types[ty].hits;
        let mut u = Unit::new(owner, index, pos, hits);
        u.ty = Some(ty);
        u.kind = sim.unit_types[ty].kind;
        u.movement.speed = 25;
        u.movement.turning = movement::Turning {
            type_turn_speed: movement::degrees_to_angle(45).0,
            packed: false,
            instant_from_stop: true,
            wide_limit: false,
        };
        sim.add_unit(u)
    }

    fn finish(sim: &mut Sim, b: usize) {
        let mut guard = 0;
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
            guard += 1;
            assert!(guard < 10);
        }
        sim.buildings[b].helpers = 0;
    }

    fn build(sim: &mut Sim, who: P, ty: usize, tx: i32, ty_: i32) -> usize {
        let b = sim
            .place_building(who, ty, tile_pos(tx, ty_))
            .unwrap_or_else(|e| panic!("should place: {e:?}"));
        finish(sim, b);
        b
    }

    /// Walks a worker onto a gather building and returns when the arrival has
    /// registered — `been_there`, which is what `num_gatherers(1, 1)` reads.
    fn gather_until_arrived(sim: &mut Sim, u: usize, b: usize) {
        sim.add_gather_order(u, b, QueuePos::New, false);
        let mut guard = 0;
        while sim.num_gatherers(b, true, true) == 0 {
            sim.tick();
            guard += 1;
            assert!(guard < 200, "the worker never arrived");
        }
    }

    // ------------------------------------------------------------------
    // The one that matters: a farm with a citizen on it pays.
    // ------------------------------------------------------------------

    #[test]
    fn a_farm_with_one_gathering_citizen_pays_the_peasant_rate() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        let city = build(&mut sim, 0, t.village, 32, 32);
        let c = sim.buildings[city]
            .city
            .expect("a finished city has a record");
        let farm = build(&mut sim, 0, t.farm, 40, 32);
        assert_eq!(sim.buildings[farm].city, Some(c), "the farm joins the city");

        // Before anybody works it, the city is worth CITY_GATHER and no more.
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities.len(), 1);
        assert_eq!(
            sim.holdings[0].cities[0].sites,
            vec![Site {
                resource: Resource::Food,
                gatherers: 0,
                level: 1,
                nation_pct: 0,
                farm_wealth: 0,
                miner_wealth: 0,
            }],
            "the farm is a site the moment it is finished, with nobody on it"
        );
        let bare = economy::assemble(&sim.tuning, &sim.holdings[0]);
        assert_eq!(bare[0], sim.tuning.city_gather[0] * RATE_SCALE);

        // Now put a citizen on it and walk it there.
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        gather_until_arrived(&mut sim, u, farm);

        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities[0].sites[0].gatherers, 1);

        // Through `economy::process`: one gatherer is PEASANT_RATE, the city
        // is CITY_GATHER, and the pair arrive exactly once per GATHER_RATE
        // frames.
        let mut ledger = economy::Ledger {
            dirty: true,
            ..economy::Ledger::default()
        };
        for frame in 0..i64::from(sim.tuning.gather_rate) {
            economy::process(&sim.tuning, &mut ledger, &sim.holdings[0], 0, frame);
        }
        let expected = (sim.tuning.peasant_rate >> 8) + sim.tuning.city_gather[0];
        assert_eq!(expected, 20);
        assert_eq!(ledger.rate[0], expected * RATE_SCALE);
        assert_eq!(ledger.bucket[Resource::Food.index()], expected);
        // And nothing else moved.
        assert_eq!(ledger.bucket[Resource::Metal.index()], 0);
    }

    /// `BuildTypeData::calc_gather@00639e40:978`–`985`: a Japanese farm's
    /// food per gatherer is `(JAPANESE_FISHING_BOATS + 100) × rate / 100`,
    /// and it is the nation, not the player (item 3331, `docs/AI.md` §268).
    #[test]
    fn a_japanese_farmer_gathers_a_quarter_more_food() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let farm = build(&mut sim, 0, t.farm, 40, 32);
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        gather_until_arrived(&mut sim, u, farm);
        let rate = |sim: &mut Sim| {
            sim.assemble_holdings(0);
            economy::city_rates(&sim.tuning, &sim.holdings[0].cities[0])[0]
        };
        let city = sim.tuning.city_gather[0] * RATE_SCALE;
        let plain = rate(&mut sim);
        assert_eq!(plain - city, 160, "PEASANT_RATE, in sixteenths");
        sim.set_tribe(0, 15);
        assert_eq!(sim.holdings[0].cities.len(), 1);
        assert_eq!(rate(&mut sim) - city, 200, "160 × 125 / 100");
        assert_eq!(sim.holdings[0].cities[0].sites[0].nation_pct, 25);
        sim.set_tribe(0, 16);
        assert_eq!(rate(&mut sim), plain, "the Koreans farm at cost");
    }

    /// `BuildTypeData::calc_gather@00639e40:997`–`1005`: an Egyptian Farm
    /// with any gatherer pays `EGYPTIAN_FARM_WEALTH × 16` wealth, flat — not
    /// per gatherer — and an unworked one, or another nation's, nothing
    /// (item 3332).
    #[test]
    fn an_inca_mine_pays_ten_wealth_a_miner() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        let mine = sim.add_build_type(bt(Ident::Mine, "gaef", 4, 4));
        // A mountain range the cell east of the mine.
        for cy in 7..10 {
            for ty in 0..crate::world::TILES_PER_CELL {
                for tx in 0..crate::world::TILES_PER_CELL {
                    let p = Pos::new(44 + tx, cy * crate::world::TILES_PER_CELL + ty);
                    sim.world.set_tile_bits(
                        p,
                        crate::world::tile::OBJECT_MOUNTAIN | crate::gather::GATHERABLE,
                    );
                }
            }
        }
        build(&mut sim, 0, t.village, 32, 32);
        let m = build(&mut sim, 0, mine, 42, 32);
        let rates = |sim: &mut Sim| {
            sim.assemble_holdings(0);
            economy::city_rates(&sim.tuning, &sim.holdings[0].cities[0])
        };
        let w = Resource::Wealth.index();
        let city = rates(&mut sim)[w];
        sim.set_tribe(0, 2);
        assert_eq!(rates(&mut sim)[w], city, "an unworked Mine pays nothing");
        for (i, x) in [40, 42].into_iter().enumerate() {
            let u = spawn(&mut sim, 0, t.citizen, tile_pos(x, 38));
            let n = i32::try_from(i + 1).unwrap();
            sim.add_gather_order(u, m, QueuePos::New, false);
            for _ in 0..200 {
                if sim.num_gatherers(m, true, true) == n {
                    break;
                }
                sim.tick();
            }
            assert_eq!(sim.num_gatherers(m, true, true), n, "miner {n} arrived");
            assert_eq!(sim.tuning.inca_wealth_per_miner, 10);
            assert_eq!(
                rates(&mut sim)[w] - city,
                10 * RATE_SCALE * n,
                "ten wealth a miner, per head"
            );
        }
        let metal = rates(&mut sim)[Resource::Metal.index()];
        sim.set_tribe(0, 16);
        assert_eq!(rates(&mut sim)[w], city, "the Koreans' Mine pays no wealth");
        assert_eq!(
            rates(&mut sim)[Resource::Metal.index()],
            metal,
            "the metal is the same"
        );
    }

    #[test]
    fn an_egyptian_worked_farm_pays_two_wealth_flat() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let farm = build(&mut sim, 0, t.farm, 40, 32);
        let wealth = |sim: &mut Sim| {
            sim.assemble_holdings(0);
            economy::city_rates(&sim.tuning, &sim.holdings[0].cities[0])[Resource::Wealth.index()]
        };
        let city = sim.tuning.city_gather[Resource::Wealth.index()] * RATE_SCALE;
        sim.set_tribe(0, 7);
        assert_eq!(wealth(&mut sim), city, "an unworked Farm pays nothing");
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        gather_until_arrived(&mut sim, u, farm);
        assert_eq!(sim.tuning.egyptian_farm_wealth, 2);
        assert_eq!(wealth(&mut sim) - city, 2 * RATE_SCALE, "two wealth a Farm");
        let v = spawn(&mut sim, 0, t.citizen, tile_pos(44, 34));
        gather_until_arrived(&mut sim, v, farm);
        assert_eq!(
            wealth(&mut sim) - city,
            2 * RATE_SCALE,
            "flat, not per head"
        );
        sim.set_tribe(0, 16);
        assert_eq!(wealth(&mut sim), city, "the Koreans' Farm pays no wealth");
    }

    #[test]
    fn a_walking_citizen_earns_nothing_until_it_arrives() {
        // `BuildData::calc_gather` passes `num_gatherers(1, 1)` — arrived —
        // so the whole walk is unpaid. This is the difference between the
        // count the chain carries and the count the economy reads.
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let farm = build(&mut sim, 0, t.farm, 40, 32);
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        sim.add_gather_order(u, farm, QueuePos::New, false);

        assert_eq!(sim.num_gatherers(farm, false, false), 1, "on the chain");
        sim.assemble_holdings(0);
        assert_eq!(
            sim.holdings[0].cities[0].sites[0].gatherers, 0,
            "not arrived: worth nothing"
        );
        sim.tick();
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities[0].sites[0].gatherers, 0);

        gather_until_arrived(&mut sim, u, farm);
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities[0].sites[0].gatherers, 1);
    }

    #[test]
    fn the_arrival_marks_the_economy_dirty_so_income_shows_within_eight_frames() {
        // The `0x2000000` half of this module's notes, end to end: the frame
        // the citizen arrives is the frame the flag goes up, and the rate is
        // reassembled on the next multiple of eight rather than at 512.
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let farm = build(&mut sim, 0, t.farm, 40, 32);
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        sim.ledgers[0].dirty = false;
        sim.ledgers[0].gather_stamp = sim.frame;
        gather_until_arrived(&mut sim, u, farm);
        assert!(sim.ledgers[0].dirty, "the arrival raised the flag");

        let arrived = sim.frame;
        let next = (arrived..arrived + 64)
            .find(|&f| sim.holdings_due(0, f))
            .expect("a dirty economy reassembles");
        assert!(
            next - arrived <= 8,
            "waited {} frames, not the lazy 512",
            next - arrived
        );
    }

    // ------------------------------------------------------------------
    // The city's own five terms
    // ------------------------------------------------------------------

    #[test]
    fn a_market_a_library_and_a_granary_are_read_off_the_member_chain() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        let city = build(&mut sim, 0, t.village, 32, 32);
        let c = sim.buildings[city].city.unwrap();

        sim.assemble_holdings(0);
        let bare = sim.holdings[0].cities[0].clone();
        assert!(!bare.market && !bare.library && !bare.university);
        assert_eq!(bare.granary, 0);
        assert_eq!(bare.buildings, 1, "the city centre is in its own chain");

        let market = build(&mut sim, 0, t.market, 38, 32);
        let library = build(&mut sim, 0, t.library, 32, 38);
        let granary = build(&mut sim, 0, t.granary, 26, 32);
        for b in [market, library, granary] {
            assert_eq!(sim.buildings[b].city, Some(c));
        }

        sim.assemble_holdings(0);
        let full = &sim.holdings[0].cities[0];
        assert!(full.market && full.library);
        assert!(!full.university, "a library is not a university");
        assert_eq!(full.buildings, 4);
        // GRANARY_BONUS level 1, the level a player with no bonus type holds.
        assert_eq!(full.granary, sim.tuning.granary_bonus[0]);
        assert_eq!(full.lumber_mill, 0, "no lumber mill, no percentage");

        // A market is ten wealth per thirty seconds. A library on its own is
        // worth nothing — `LIBRARY_LITERACY` ships as zero, and its value is
        // that scholars can live in it — and so is a granary with nobody
        // farming under it.
        let rate = economy::assemble(&sim.tuning, &sim.holdings[0]);
        assert_eq!(rate[Resource::Wealth.index()], 10 * RATE_SCALE);
        assert_eq!(rate[Resource::Knowledge.index()], 0);
    }

    #[test]
    fn an_unfinished_building_is_not_in_the_economy_at_all() {
        // Both `count_buildings` and `num_buildings` insist on `is_active`,
        // and `BuildTypeData::calc_gather` answers zero for an inactive site.
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let market = sim.place_building(0, t.market, tile_pos(38, 32)).unwrap();
        assert!(!sim.buildings[market].active);
        let farm = sim.place_building(0, t.farm, tile_pos(32, 39)).unwrap();

        sim.assemble_holdings(0);
        let c = &sim.holdings[0].cities[0];
        assert!(!c.market, "a site under construction pays no taxes");
        assert_eq!(c.buildings, 1);
        assert!(c.sites.is_empty(), "and an unfinished farm is not a site");

        finish(&mut sim, farm);
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities[0].sites.len(), 1);
    }

    /// **`num_gatherers(_, 1)` drops a decoy from the garrison term too**
    /// (twenty-fourth pass, A7 row 28): the garrison is
    /// `count_inside(COUNT_TYPE + 2 * skip, …)`, and `COUNT_NON_DECOY_TYPE`
    /// skips a unit whose `unit_masks & 1` is set. The admission count
    /// (`skip = 0`) keeps it.
    #[test]
    fn a_decoy_scholar_is_not_the_economy_s_gatherer() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let uni = build(&mut sim, 0, t.university, 40, 32);
        let real = spawn(&mut sim, 0, t.scholar, tile_pos(44, 32));
        let decoy = spawn(&mut sim, 0, t.scholar, tile_pos(44, 33));
        sim.units[decoy].decoy = true;
        for s in [real, decoy] {
            sim.buildings[uni].garrison.push(s);
            sim.units[s].on_map = false;
            sim.units[s].inside = Some(uni);
        }
        assert_eq!(
            sim.num_gatherers(uni, true, false),
            2,
            "admission counts both"
        );
        assert_eq!(
            sim.num_gatherers(uni, true, true),
            1,
            "the economy skips the decoy"
        );
    }

    #[test]
    fn a_scholar_in_a_university_gathers_knowledge_at_the_leader_s_level() {
        // `num_gatherers` counts a *garrisoned* scholar, and the rate is
        // `SCHOLAR_RATE[university_level - 1]` off the leader, not the
        // building.
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let uni = build(&mut sim, 0, t.university, 40, 32);
        let s = spawn(&mut sim, 0, t.scholar, tile_pos(44, 32));
        sim.buildings[uni].garrison.push(s);
        sim.units[s].on_map = false;
        sim.units[s].inside = Some(uni);
        assert_eq!(sim.num_gatherers(uni, true, true), 1);

        sim.assemble_holdings_with(0, Levels::BASE);
        let site = sim.holdings[0].cities[0]
            .sites
            .iter()
            .find(|s| s.resource == Resource::Knowledge)
            .expect("the university is a knowledge site");
        assert_eq!(site.gatherers, 1);
        assert_eq!(site.level, 1);

        let base = economy::assemble(&sim.tuning, &sim.holdings[0]);
        // A university is ten knowledge of literacy plus one scholar at
        // SCHOLAR_RATE level 1.
        assert_eq!(
            base[Resource::Knowledge.index()],
            10 * RATE_SCALE + economy::per_gatherer(&sim.tuning, Resource::Knowledge, 1)
        );

        // The last university level is worth five times the first.
        sim.assemble_holdings_with(
            0,
            Levels {
                university: 6,
                ..Levels::BASE
            },
        );
        let top = economy::assemble(&sim.tuning, &sim.holdings[0]);
        assert_eq!(
            top[Resource::Knowledge.index()] - 10 * RATE_SCALE,
            25 * RATE_SCALE
        );
    }

    // ------------------------------------------------------------------
    // The player-level terms
    // ------------------------------------------------------------------

    #[test]
    fn refineries_are_counted_across_the_whole_nation() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].refineries, 0);

        build(&mut sim, 0, t.refinery, 38, 32);
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].refineries, 1);

        // Another player's refinery is not ours.
        build(&mut sim, 1, t.village, 8, 8);
        build(&mut sim, 1, t.refinery, 14, 8);
        sim.assemble_holdings(0);
        sim.assemble_holdings(1);
        assert_eq!(sim.holdings[0].refineries, 1);
        assert_eq!(sim.holdings[1].refineries, 1);
    }

    #[test]
    fn an_oil_well_pays_outside_the_city_loop_even_when_it_stands_in_a_city() {
        // The city loop skips `is(OILWELL)` and `is(OILPLATFORM)` explicitly,
        // because the `oil_wells` list already covers them. Counting them in
        // both places would double a nation's oil.
        let mut sim = world_sim();
        let t = install(&mut sim);
        let city = build(&mut sim, 0, t.village, 32, 32);
        let c = sim.buildings[city].city.unwrap();
        // An Oil Well stands only on the cell's oil (`blocked_tcoord`'s
        // `NoOil`, `docs/AI.md` §78).
        for cy in 7..=8 {
            for cx in 9..=10 {
                let at = crate::Cell::new(cx, cy);
                let mut d = sim.world.cell_data(at);
                d.flags |= crate::world::cell::OIL;
                sim.world.set_cell_data(at, d);
            }
        }
        let well = build(&mut sim, 0, t.oil_well, 40, 32);
        assert_eq!(sim.buildings[well].city, Some(c), "it is a city member");

        sim.assemble_holdings(0);
        assert!(
            sim.holdings[0].cities[0]
                .sites
                .iter()
                .all(|s| s.resource != Resource::Oil),
            "not a city site"
        );
        let outside = sim.gather_sites_outside_cities(0);
        assert_eq!(outside.len(), 1);
        assert_eq!(outside[0].resource, Resource::Oil);
    }

    /// **The oil well's stand puts the worker on `CHAR_FARM`** — and a guy
    /// already on it returns (`do_gather@005ef2a0:377`), so `Guy::move`'s
    /// arrival test (`cur_anim == CHAR_WALK`) never sees a walk and the
    /// arrival stand `Guy::move+0x19f` is never rolled (`docs/AI.md` §150,
    /// item 1549: Great Sahara in the coverage lobby, frame 720, where this
    /// crate spent one draw the original did not).
    #[test]
    fn an_oil_well_s_gatherer_stands_on_the_farm_animation_and_rolls_no_arrival() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        for cy in 7..=8 {
            for cx in 9..=10 {
                let at = crate::Cell::new(cx, cy);
                let mut d = sim.world.cell_data(at);
                d.flags |= crate::world::cell::OIL;
                sim.world.set_cell_data(at, d);
            }
        }
        let well = build(&mut sim, 0, t.oil_well, 40, 32);
        let u = spawn(&mut sim, 0, t.citizen, tile_pos(44, 32));
        sim.units[u].guys.push(crate::anim::Guy::fresh(0));
        sim.trace_phases = true;
        gather_until_arrived(&mut sim, u, well);
        sim.phase_marks.clear();
        let mut arrivals = 0;
        for _ in 0..8 {
            sim.tick();
            arrivals += sim
                .phase_marks
                .iter()
                .filter(|(l, _)| l == crate::anim::SITE_ARRIVE)
                .count();
            sim.phase_marks.clear();
        }
        assert_eq!(
            sim.units[u].guys[0].anim,
            crate::anim::FARM,
            "the worker works the well on the farm animation"
        );
        assert_eq!(arrivals, 0, "and it rolls no arrival stand");
        // **The guard**: a guy already on `CHAR_FARM` returns before the
        // turn and the relocation, so a worker nudged off its stand is not
        // put back (`do_gather:377`).
        let stand = sim.units[u].pos;
        let nudged = Pos::new(stand.x + 8, stand.y);
        let from = sim.units[u].pos;
        sim.set_new_location(u, nudged, true);
        sim.moved_to(u, from, false);
        sim.tick();
        assert_eq!(
            sim.units[u].pos, nudged,
            "the guard leaves the worker where it stands"
        );
    }

    #[test]
    fn a_camp_with_no_city_is_outside_the_city_loop() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        // No city anywhere near: `find_city` leaves the link empty.
        build(&mut sim, 0, t.village, 4, 4);
        sim.plant_camp_forest(tile_pos(56, 56));
        let camp = build(&mut sim, 0, t.camp, 56, 56);
        assert_eq!(sim.buildings[camp].city, None);

        sim.assemble_holdings(0);
        assert!(sim.holdings[0].cities[0].sites.is_empty());
        let outside = sim.gather_sites_outside_cities(0);
        assert_eq!(outside.len(), 1);
        assert_eq!(outside[0].resource, Resource::Timber);
    }

    /// A tree carrying the five `TECHBONUSES` ladders, nothing else: the
    /// rows in level order, so `roles.*_preq[i]` is the `i+1`-th rung.
    fn with_the_ladders(sim: &mut Sim) {
        use crate::tech::{Line, TechTree, TypeDef, TypeId};
        let mut tree = TechTree::new();
        let rung = |tree: &mut TechTree, n: &str| -> Option<TypeId> {
            Some(tree.add(TypeDef::epoch(n, Line::Commerce, 0)))
        };
        let g = [
            rung(&mut tree, "GRANARY2"),
            rung(&mut tree, "GRANARY3"),
            rung(&mut tree, "GRANARY4"),
            rung(&mut tree, "GRANARY5"),
        ];
        let l = [
            rung(&mut tree, "LUMBERMILL2"),
            rung(&mut tree, "LUMBERMILL3"),
            rung(&mut tree, "LUMBERMILL4"),
        ];
        let sm = [
            rung(&mut tree, "SMELTER2"),
            rung(&mut tree, "SMELTER3"),
            rung(&mut tree, "SMELTER4"),
        ];
        let u = [
            rung(&mut tree, "UNIVERSITY2"),
            rung(&mut tree, "UNIVERSITY3"),
            rung(&mut tree, "UNIVERSITY4"),
            rung(&mut tree, "UNIVERSITY5"),
            rung(&mut tree, "UNIVERSITY6"),
        ];
        let tx = [
            rung(&mut tree, "TAX_1"),
            rung(&mut tree, "TAX_2"),
            rung(&mut tree, "TAX_3"),
            rung(&mut tree, "TAX_4"),
        ];
        tree.roles.granary_preq = g;
        tree.roles.lumbermill_preq = l;
        tree.roles.smelter_preq = sm;
        tree.roles.university_preq = u;
        tree.roles.taxation_preq = tx;
        sim.set_tech_tree(tree);
    }

    /// **The five ladders answer their floor, and only their floor, with
    /// nothing held** — one everywhere, and **zero** for taxation. That
    /// asymmetry is `get_granary@006db340`'s `(held) + 1` against
    /// `get_taxation@006d6e20`'s `(uint)(held != 0)`, and it is the whole
    /// reason the territory tax is the sharp one (item 232, ECONOMY R8).
    #[test]
    fn the_five_ladders_floor_at_one_except_taxation_which_floors_at_zero() {
        let mut sim = world_sim();
        with_the_ladders(&mut sim);
        assert_eq!(Levels::for_player(&sim, 0), Levels::BASE);
        assert_eq!(Levels::BASE.taxation, 0);

        // And a tree that names none of the rows — every fixture that builds
        // its own — answers the same, so item 232 is a widening.
        let bare = world_sim();
        assert_eq!(Levels::for_player(&bare, 0), Levels::BASE);
    }

    /// **Each ladder is the highest rung the player holds**, tested most
    /// advanced first: holding rung 2 and rung 4 answers 4.
    #[test]
    fn each_ladder_answers_the_highest_rung_held() {
        let mut sim = world_sim();
        with_the_ladders(&mut sim);
        let roles = sim.tech_tree.roles.clone();
        for (rows, floor, want) in [
            (roles.granary_preq.to_vec(), 1, 5),
            (roles.lumbermill_preq.to_vec(), 1, 4),
            (roles.smelter_preq.to_vec(), 1, 4),
            (roles.university_preq.to_vec(), 1, 6),
            (roles.taxation_preq.to_vec(), 0, 4),
        ] {
            let mut s = world_sim();
            with_the_ladders(&mut s);
            // The second rung alone: floor + 2.
            s.tech[0].tech[rows[1].unwrap()] = true;
            assert_eq!(s.bonus_level(0, &rows) + floor, floor + 2);
            // The top rung as well — the ladder does not add up, it picks.
            s.tech[0].tech[rows[rows.len() - 1].unwrap()] = true;
            assert_eq!(s.bonus_level(0, &rows) + floor, want);
            // And it is per player.
            assert_eq!(s.bonus_level(1, &rows), 0);
        }
    }

    /// **The territory tax is a whole income line that could never fire**
    /// (audit R8): `Holdings::taxation` was written by nothing, so it kept
    /// its `Default` zero and `TERRITORY_TAXES[0]` — 0 per cent — for the
    /// life of every game. `assemble_holdings` writes it now.
    ///
    /// **No capture can falsify this.** The original's own territory tax is
    /// zero through every run on disk too, because neither leader reaches a
    /// taxation level inside any captured window (ECONOMY, "What run60
    /// leaves"), so the two agree before the fix and after it. This is
    /// correctness with no oracle behind it, and the check that would give
    /// it one is a capture past a `TAX_1` research with `LEADERDATA`'s
    /// `resources[2]`/`income[2]` on the frame after.
    #[test]
    fn taxation_reaches_the_territory_tax() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        with_the_ladders(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        sim.update_territory_holdings();
        assert!(sim.holdings[0].territory > 0 && sim.holdings[0].land_size > 0);

        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].taxation, 0, "no TAX_n held");
        assert_eq!(economy::territory_tax(&sim.tuning, &sim.holdings[0]), 0);

        let tax2 = sim.tech_tree.roles.taxation_preq[1].unwrap();
        sim.tech[0].tech[tax2] = true;
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].taxation, 2);
        let h = &sim.holdings[0];
        assert_eq!(
            economy::territory_tax(&sim.tuning, h),
            h.territory * sim.tuning.territory_taxes[2] * RATE_SCALE / h.land_size,
        );
        assert!(
            economy::territory_tax(&sim.tuning, h) > 0,
            "the line the audit called unfirable",
        );
    }

    #[test]
    fn the_commerce_level_is_the_commerce_epoch() {
        // `calc_resource_caps` reads `LeaderDataEncrypt + 0xf0`, which is
        // `epoch[2]`, and `Line::Commerce` is 2.
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        sim.tech[0].epoch[Line::Commerce.index()] = 3;
        sim.tech[0].epoch[Line::Science.index()] = 7;
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].commerce, 3);
        assert_eq!(
            economy::commerce_cap(&sim.tuning, &sim.holdings[0], Resource::Food),
            sim.tuning.commerce_cap[3] * RATE_SCALE
        );
    }

    #[test]
    fn only_an_ai_carries_the_difficulty_handicap() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        build(&mut sim, 1, t.village, 8, 8);
        sim.lobby.difficulty = 5;
        sim.nation[1].human = false;

        sim.assemble_holdings(0);
        sim.assemble_holdings(1);
        assert_eq!(sim.holdings[0].handicap, 0, "a human earns their rate");
        assert_eq!(sim.holdings[1].handicap, 50, "the AI earns half again");
    }

    #[test]
    fn the_fields_the_rest_of_the_simulation_owns_are_left_alone() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        sim.update_territory_holdings();
        let territory = sim.holdings[0].territory;
        let land_size = sim.holdings[0].land_size;
        assert!(land_size > 0);
        sim.holdings[0].available[Resource::Oil.index()] = false;
        sim.holdings[0].discovered[Resource::Oil.index()] = true;
        sim.holdings[0].bonus_cap[Resource::Wealth.index()] = 42;
        sim.holdings[0].taxation = 4;

        sim.assemble_holdings(0);

        let h = &sim.holdings[0];
        assert_eq!(h.territory, territory);
        assert_eq!(h.land_size, land_size);
        assert!(!h.available[Resource::Oil.index()]);
        assert!(h.discovered[Resource::Oil.index()]);
        assert_eq!(h.bonus_cap[Resource::Wealth.index()], 42);
        // `taxation` was on this list until item 232, and the hand-set 4
        // survived because nothing wrote it. It is assembled now, from the
        // player's `TAX_n` ladder, so a hand-set value is *overwritten* —
        // which is the point of the fix.
        assert_eq!(h.taxation, 0, "assembled from the ladder, not preserved");
    }

    #[test]
    fn a_lost_city_stops_paying() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        let city = build(&mut sim, 0, t.village, 32, 32);
        build(&mut sim, 0, t.farm, 40, 32);
        sim.assemble_holdings(0);
        assert_eq!(sim.holdings[0].cities.len(), 1);
        let with = economy::assemble(&sim.tuning, &sim.holdings[0]);
        assert!(with[0] > 0);

        sim.buildings[city].alive = false;
        let c = sim.buildings[city].city.unwrap();
        sim.cities[c].alive = false;
        sim.assemble_holdings(0);
        assert!(sim.holdings[0].cities.is_empty());
        let without = economy::assemble(&sim.tuning, &sim.holdings[0]);
        assert_eq!(without, [0; RESOURCES], "BASIC_GATHER ships as zero");
    }

    #[test]
    fn rate_now_is_the_assembly_without_the_cadence() {
        let mut sim = world_sim();
        let t = install(&mut sim);
        build(&mut sim, 0, t.village, 32, 32);
        let rate = rate_now(&mut sim, 0);
        assert_eq!(rate[0], sim.tuning.city_gather[0] * RATE_SCALE);
        assert_eq!(rate[1], sim.tuning.city_gather[1] * RATE_SCALE);
    }
}
