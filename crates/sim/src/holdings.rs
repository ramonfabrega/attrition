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
/// **These five are a seam.** `GRANARY2` is `TypeIndex` `0x2bb` and
/// `BASE_BONUSTYPES` is `0x2ac`, so all of them — `GRANARY2..5`,
/// `LUMBERMILL2..4`, `SMELTER2..4`, `UNIVERSITY2..6`, `TAX_1..4` — are
/// *bonus* types, and the tree `crates/rondata` loads holds units, buildings,
/// goods, ages and techs but not bonuses. There is nothing in `Sim` to read
/// them off yet. [`Levels::BASE`] is what the original answers for a player
/// who holds none of them, which is every player at the start of a game, so
/// it is the right default rather than a placeholder — but a game that runs
/// long enough to research Agriculture will need them. The natural home is
/// `city::Nation`, which already carries `temple_level` and
/// `fort_garrison_level` for exactly this reason.
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

    /// The levels for one player, as far as the simulation can tell.
    ///
    /// [`Levels::BASE`] today — see the type's note. This is the one function
    /// to change when the bonus types arrive.
    pub fn for_player(sim: &Sim, who: Player) -> Levels {
        debug_assert!((who as usize) < sim.holdings.len(), "no such player");
        Levels::BASE
    }
}

impl Default for Levels {
    fn default() -> Levels {
        Levels::BASE
    }
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
        out.temple = any(Ident::Temple);
        out.university = any(Ident::University);
        out.library = any(Ident::Library);

        if any(Ident::Granary) {
            out.set_enhancer(&self.tuning, Resource::Food, levels.granary.max(1) as usize);
        }
        if any(Ident::Lumbermill) {
            out.set_enhancer(
                &self.tuning,
                Resource::Timber,
                levels.lumber_mill.max(1) as usize,
            );
        }
        if any(Ident::Smelter) {
            out.set_enhancer(
                &self.tuning,
                Resource::Metal,
                levels.smelter.max(1) as usize,
            );
        }

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
        Site {
            resource,
            gatherers: self.num_gatherers(b, true, true),
            level: if resource == Resource::Knowledge {
                levels.university.max(1)
            } else {
                1
            },
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
    /// Writes `cities`, `refineries`, `commerce` and `handicap`. **Leaves
    /// alone** the four fields the simulation maintains elsewhere or has no
    /// source for: `territory` and `land_size`
    /// (`Sim::update_territory_holdings`), `available` and `discovered` (the
    /// tech layer), `bonus_cap` (scenario scripts only —
    /// `ScenarioFuncSet::set_bonus_cap` is its one writer in the executable)
    /// and `taxation` (a bonus type; see [`Levels`]).
    pub fn assemble_holdings(&mut self, who: Player) {
        let levels = Levels::for_player(self, who);
        self.assemble_holdings_with(who, levels);
    }

    /// [`Sim::assemble_holdings`] with the enhancer levels supplied.
    pub fn assemble_holdings_with(&mut self, who: Player, levels: Levels) {
        let w = who as usize;

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
        let h = &mut self.holdings[w];
        h.cities = cities;
        h.outside = outside;
        h.rares = rares;
        h.rare_owned = rare_owned;
        h.refineries = refineries;
        h.commerce = commerce;
        h.handicap = handicap;
        h.british = self.nation[w].british;
        h.egyptians = self.nation[w].egyptians;
        h.french = self.nation[w].french;
        h.inca = self.nation[w].inca;
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
        assert_eq!(h.taxation, 4, "a bonus type this module cannot read");
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
