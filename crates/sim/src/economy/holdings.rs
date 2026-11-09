//! `Leader::calc_gather`'s **inputs**: filling [`economy::Holdings`] from the
//! live simulation.
//!
//! `crates/sim/src/economy/mod.rs` is the arithmetic — the rate in sixteenths, the
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
//! | 6 | every idle fisherman and merchant, `Unit::do_gather` | [`Sim::gather_rares`] ([`crate::economy::rares`]) |
//! | 7 | refineries — `get_buildings(REFINERY)` | [`economy::Holdings::refineries`] |
//! | 11 | the territory tax | `Sim::update_territory_holdings` (untouched) |
//!
//! A **site** is a building. `calc_city_resources` walks the city's member
//! chain and calls `BuildData::calc_gather` on every member that is active,
//! whose type `is_gather_type` (`build_flags & 0x40`, [`build::flags::GATHERER`]),
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
//!   both are modelled ([`crate::economy::rares`]); an earlier draft of this list had
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
//! `crates/sim/src/orders/mod.rs` raises it on the arrival and on both halves of
//! the idle latch; the building half is listed as an open edit in this
//! module's notes.
//!
//! # Arithmetic
//!
//! Counting only. Every number this module produces is a count, a percentage
//! taken straight from `Tuning`, or a level; the sixteenths and the
//! truncations all live in `economy.rs`.

use crate::Sim;
use crate::cities::build::{self, Ident};
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
        debug_assert!(
            (who as usize) < sim.economy.holdings.len(),
            "no such player"
        );
        let roles = &sim.tables.tech_tree.roles;
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
/// the [`crate::economy::rares::top_held`] of `LUMBERMILL2..4` — `has_preq(i)` asked for row `i`, the top first. The
/// leader half is [`Levels::for_player`]'s `lumber_mill`; item 2090's sweep holds the gate and the asks.
pub fn lumber_level_by(city_flags: i32, has_preq: impl FnMut(usize) -> bool) -> i32 {
    if city_flags & 0x400 == 0 {
        return 0;
    }
    1 + crate::economy::rares::top_held(3, has_preq) as i32
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
    /// `crates/sim/src/economy/gather.rs` reaches the same list from the other end,
    /// where farm, oil well and oil platform are the flat ones and the camp,
    /// the mine and the university are not.
    pub fn gather_good(&self, b: usize) -> Option<Resource> {
        let t = self.buildings[b].ty?;
        if !self.tables.build_types[t].has(build::flags::GATHERER) {
            return None;
        }
        let is = |i| build::is(&self.tables.build_types, t, i);
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
            .is_some_and(|t| build::is(&self.tables.build_types, t, ident))
    }

    /// **The city a leader's CEO stands in** — `calc_city_resources@006d5530:166`
    /// –`195` (item 3516). Gated on the leader's count of the `0x165` type
    /// (`+0x59c8`, the muster's, as [`Sim::ceo_unit_range`] reads it) and on
    /// its heroes; then the first hero on the map that is the `0x165` type
    /// ends the walk, and `ObjectsData::find_city_at` at its position is the
    /// city. `is(0x165, 1)` is taken as the type itself, as the patriot tests
    /// of the radius are, and the heroes list as unit order.
    pub(crate) fn ceo_city(&self, who: Player) -> Option<usize> {
        const CEO: i32 = 0x165;
        let counted = self
            .tables
            .unit_types
            .iter()
            .position(|t| t.type_index == CEO)
            .and_then(|r| {
                self.roster
                    .muster
                    .get(who as usize)
                    .and_then(|m| m.by_type.get(r))
            })
            .is_some_and(|&n| n != 0);
        if !counted {
            return None;
        }
        let h = (0..self.units.len()).find(|&h| {
            let x = &self.units[h];
            x.owner == who && x.alive() && x.on_map && x.type_index == CEO && self.is_hero_unit(h)
        })?;
        self.find_city_at(who, self.units[h].pos, None)
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
        out.trade_val = self.cities.cities[c].trade_val;

        // `CityData::get_taxes` and `get_literacy`, which are
        // `count_buildings(MARKET | TEMPLE)` and
        // `count_buildings(UNIVERSITY | LIBRARY)`.
        out.market = any(Ident::Market);
        // `get_taxes@00737b50`: the Porcelain Tower raises a market's taxes
        // (`docs/GOLDEN.md` §96, item 2598).
        out.porcelain = out.market
            && self.wonders_held(self.cities.cities[c].owner)
                & (1 << crate::tech::wonder::PORCELAIN_TOWER)
                != 0;
        out.temple = any(Ident::Temple);
        out.university = any(Ident::University);
        out.library = any(Ident::Library);
        out.ceo = self.ceo_city(self.cities.cities[c].owner) == Some(c);

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
        let tuning = &self.tables.tuning;
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
            && self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                &self.tech[bd.owner as usize],
                crate::cities::nations::power::JAPANESE,
            ) {
            self.tables.tuning.japanese_fishing_boats
        } else {
            0
        };
        // `calc_gather:997`–`1005`: a Farm's wealth, for an Egyptian leader
        // (item 3332).
        let farm_wealth = if self.is_kind(b, Ident::Farm)
            && self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                &self.tech[bd.owner as usize],
                crate::cities::nations::power::EGYPTIANS,
            ) {
            self.tables.tuning.egyptian_farm_wealth
        } else {
            0
        };
        // `calc_gather:850`–`856`: a mine's wealth per gatherer, for an Inca
        // leader (item 3360). The branch is the mine's — metal, beside
        // `german_miners` — and the constant must be positive.
        let miner_wealth = if resource == Resource::Metal
            && self.tables.tuning.inca_wealth_per_miner > 0
            && self.tables.tech_tree.has_tribe_bonus(
                &self.tables.setup,
                &self.tech[bd.owner as usize],
                crate::cities::nations::power::INCA,
            ) {
            self.tables.tuning.inca_wealth_per_miner
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
        let line = &self.tables.tech_tree.roles.wonder_line;
        let mut held = 0u32;
        for b in &self.buildings {
            if b.owner != who || !b.alive || !b.active {
                continue;
            }
            let Some(tree) = b.ty.and_then(|t| self.tables.build_types[t].tree) else {
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
        if let Some(a) = self.ai.ai.get_mut(w) {
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
        let republic = self.government_bonus_level(who, &self.tables.tech_tree.roles.republic_preq);

        // `LeaderData::get_gather_handicap@006d66a0`: a human takes zero
        // unless the multiplayer handicap option is on, and an AI takes the
        // difficulty's percentage. `semaphore[0] & 4` — the MP handicap — is
        // not modelled, so a human is flatly zero here.
        let handicap = if self.cities.nation[w].human {
            0
        } else {
            let d = self.ai.lobby.difficulty.clamp(0, i32::from(u8::MAX));
            economy::gather_handicap(u8::try_from(d).unwrap_or(0))
        };

        let outside = self.gather_sites_outside_cities(who);
        // Step 6, and the `rare_owned` it rebuilds ([`crate::economy::rares`]).
        let (rares, rare_owned) = self.gather_rares(who);
        let lakota = self.cities.nation[w].lakota && self.tables.tuning.lakota_food != 0;
        let iroquois = self.tech.get(w).is_some_and(|p| {
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, p, crate::ai::tribe::IROQUOIS)
        });
        let dutch = self.tech.get(w).is_some_and(|p| {
            self.tables
                .tech_tree
                .has_tribe_bonus(&self.tables.setup, p, crate::ai::tribe::DUTCH)
        });
        let americans = self.tables.tuning.americans_barracks_gather != 0
            && self.tech.get(w).is_some_and(|p| {
                self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    p,
                    crate::ai::tribe::AMERICANS,
                )
            });
        let american_units = if americans {
            self.barracks_units(who) - self.ai.ai[w].census.barracks_garr
        } else {
            0
        };
        let starting = std::array::from_fn(|g| self.starting_good(who, g));
        let capitalism = self.holds_capitalism_bonus(who);
        let lakota_units = if lakota {
            self.lakota_food_units(who)
        } else {
            0
        };
        let h = &mut self.economy.holdings[w];
        h.cities = cities;
        h.outside = outside;
        h.rares = rares;
        h.rare_owned = rare_owned;
        h.refineries = refineries;
        h.commerce = commerce;
        h.republic = republic;
        h.handicap = handicap;
        h.taxation = levels.taxation;
        h.british = self.cities.nation[w].british;
        h.mongols = self.cities.nation[w].mongols;
        h.num_nations = self.territory.players.len() as i32;
        h.lakota = lakota;
        h.lakota_units = lakota_units;
        h.americans = americans;
        h.american_units = american_units;
        h.capitalism = capitalism;
        h.romans = self.cities.nation[w].romans;
        h.germans = self.cities.nation[w].germans;
        h.egyptians = self.cities.nation[w].egyptians;
        h.russians = self.cities.nation[w].russians;
        h.french = self.cities.nation[w].french;
        h.inca = self.cities.nation[w].inca;
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
        self.tables
            .tech_tree
            .roles
            .gov_bonuses
            .get(1)
            .is_some_and(|(preqs, _)| {
                preqs
                    .iter()
                    .all(|&q| self.tables.tech_tree.has_tech_p(&self.tables.setup, p, q))
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
    /// `barracks_units` (`Leader +0x9f8`): `Leader::track_unit_type@006e0dd0`
    /// moves it for every armed type (`+0x1e8`) whose trainer
    /// (`UnitTypeData +0x40`) is the Barracks (`0x1ab`) — the sum of
    /// `num_units` over them.
    pub(crate) fn barracks_units(&self, who: Player) -> i32 {
        let w = who as usize;
        (0..self.tables.unit_types.len())
            .filter(|&t| {
                self.tables.unit_types[t].combat.attack != 0
                    && self.trainer_ident(t) == Some(crate::cities::build::Ident::Barracks)
            })
            .map(|t| self.roster.muster[w].by_type[t])
            .sum()
    }

    /// `Unit::init@00612100:629` and `Unit::close@0060ee50:157`'s American
    /// arm: `has_tribe_bonus(0x14)`, `AMERICANS_BARRACKS_GATHER` set, a
    /// Barracks type (`+0x40 == 0x1ab`) and not `is(SCOUT, 0)`.
    pub(crate) fn american_barracks_unit(&self, who: Player, u: usize, ty: usize) -> bool {
        self.tables.tuning.americans_barracks_gather != 0
            && self.trainer_ident(ty) == Some(crate::cities::build::Ident::Barracks)
            && self.tech.get(who as usize).is_some_and(|p| {
                self.tables.tech_tree.has_tribe_bonus(
                    &self.tables.setup,
                    p,
                    crate::ai::tribe::AMERICANS,
                )
            })
            && !self.unit_line_is(u, 0x45)
    }

    pub(crate) fn lakota_food_units(&self, who: Player) -> i32 {
        let w = who as usize;
        let c = &self.ai.ai[w].census;
        let mut n = c.peasants + c.scouts - c.peasants_garr - c.stable_garr - c.scout_garr;
        for t in 0..self.tables.unit_types.len() {
            let k = self.roster.muster[w].by_type[t];
            if k != 0
                && self.tables.unit_types[t].combat.attack != 0
                && self.trainer_ident(t) == Some(crate::cities::build::Ident::Stable)
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
        let l = &self.economy.ledgers[who as usize];
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
    economy::assemble(&sim.tables.tuning, &sim.economy.holdings[who as usize])
}

#[cfg(test)]
mod tests;
