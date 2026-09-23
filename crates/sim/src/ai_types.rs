//! The type-level helpers every production-AI module shares: a tree id's
//! class and its unit or building record, the counts `LeaderData` keeps by
//! type (`num_buildings`, `num_queued`), the price and the affordability of
//! a type, and the leader's cities. `docs/AI.md` §2.10 names the vtable
//! slots these stand in for; the script host (`ai_host.rs`) has the same
//! helpers on its own `who` and stays the authority on the scripts' view.

use crate::tech::{self, Kind, TypeId};
use crate::{Player, Sim, cost, economy};

/// Which class of type a tree entry is, by the `TypeIndex` bands
/// (`docs/AI.md` §2.6: units `0x32..0x19d`, buildings `0x19e..0x21e`,
/// techs `0x220..0x274`, spells after).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Good,
    Unit,
    Build,
    Tech,
}

impl Sim {
    /// The class of a tree id.
    pub fn type_class(&self, t: TypeId) -> Class {
        match self.tech_tree.kind(t) {
            Kind::Good => Class::Good,
            Kind::Unit(_) => Class::Unit,
            Kind::Building { .. } => Class::Build,
            _ => Class::Tech,
        }
    }

    /// The unit record (an index into `unit_types`) a tree id names.
    pub fn unit_record(&self, t: TypeId) -> Option<usize> {
        self.unit_types.iter().position(|u| u.tree == Some(t))
    }

    /// The building record (an index into `build_types`) a tree id names.
    pub fn build_record(&self, t: TypeId) -> Option<usize> {
        self.build_types.iter().position(|b| b.tree == Some(t))
    }

    /// `LeaderData::num_buildings[t]`: live, finished buildings of exactly
    /// this record.
    pub fn num_buildings_of(&self, who: Player, rec: usize) -> i32 {
        self.buildings
            .iter()
            .filter(|b| b.alive && b.active && b.owner == who && b.ty == Some(rec))
            .count() as i32
    }

    /// `num_queued[t]` for a building: placed sites not yet active.
    pub fn num_sites_of(&self, who: Player, rec: usize) -> i32 {
        self.buildings
            .iter()
            .filter(|b| b.alive && !b.active && b.owner == who && b.ty == Some(rec))
            .count() as i32
    }

    /// `get_buildings(t)`: the finished buildings of this record plus those
    /// of every record it upgrades to — `docs/AI.md` §2.12, §2.14.
    pub fn buildings_of_line(&self, who: Player, rec: usize) -> i32 {
        let mut n = 0;
        let mut r = Some(rec);
        let mut guard = 0;
        while let Some(x) = r {
            n += self.num_buildings_of(who, x);
            r = self.build_types[x].to;
            guard += 1;
            if guard > 16 {
                break;
            }
        }
        n
    }

    /// The price of one of `t` for `who` — the type's own `get_cost` with
    /// no object or city context. `None` for a good.
    pub fn type_price(&self, who: Player, t: TypeId) -> Option<[i32; economy::RESOURCES]> {
        match self.type_class(t) {
            Class::Unit => self.unit_record(t).map(|r| self.price_of(who, r)),
            Class::Build => self.build_record(t).map(|r| self.building_price(who, r)),
            Class::Tech => Some(self.tech_price(who, t)),
            Class::Good => None,
        }
    }

    /// `TypeData::can_pay_cost(who, −1, −1, escrow, …)`: how many of `t`
    /// the leader can pay for; [`cost::PLENTY`] when nothing is costed.
    /// `escrow_ignored` is the original's fourth argument as the readers
    /// took it (`docs/COSTS.md`).
    pub fn type_affordable(&self, who: Player, t: TypeId, escrow_ignored: bool) -> i32 {
        match self.type_price(who, t) {
            Some(charges) => cost::affordable(
                &charges,
                &self.ledgers[who as usize],
                &self.holdings[who as usize].available,
                escrow_ignored,
            ),
            None => cost::PLENTY,
        }
    }

    /// `type_avail(t) == 4`.
    pub fn type_available(&self, who: Player, t: TypeId) -> bool {
        self.type_avail(who, t) == tech::AVAILABLE
    }

    /// The leader's live cities in slot order.
    pub fn cities_of(&self, who: Player) -> Vec<usize> {
        self.cities
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alive && c.owner == who)
            .map(|(i, _)| i)
            .collect()
    }

    /// The city's member chain, the centre first — `CityData.o` then the
    /// `city_down` links.
    pub fn city_chain(&self, c: usize) -> Vec<usize> {
        let city = &self.cities[c];
        std::iter::once(city.building)
            .chain(city.members.iter().copied())
            .collect()
    }

    /// `get_diff()`: the difficulty the AI plays at — the lobby's in a solo
    /// game (`docs/AI.md` §10).
    pub fn ai_difficulty(&self) -> i32 {
        self.lobby.difficulty
    }

    /// `LeaderData::get_mod_resource_cap@006d65b0` — the commerce cap **as
    /// the production AI reads it**, which is not the ledger's own.
    ///
    /// `docs/AI.md` §2.5.1. Three answers: `0` under
    /// `starting_resources == 8`; the raw cap for a human
    /// (`leader_flags & 4`) or a multiplayer semaphore; and otherwise the
    /// cap scaled by `get_diff()` — **half** on the easiest and three
    /// quarters on easy, so a computer leader on those settings reads a
    /// smaller cap than it holds. The original multiplies by an `f32`
    /// (`0.5f`, `0.75f`, `1.0f`) and truncates; both constants are exact
    /// in binary and the cap fits a mantissa many times over, so the
    /// integer forms here are the same arithmetic and not an
    /// approximation (`docs/DECISIONS.md` entry 16).
    ///
    /// Nine call sites read it and every one of them used the raw cap
    /// here until item 290: the goods picture's rate pass and threshold
    /// pass (§2.5), `create_buildings`' three, `create_units`' two and
    /// `research_techs`' one.
    pub fn mod_resource_cap(&self, who: Player, g: usize) -> i32 {
        if self.lobby.resources_unlimited() {
            return 0;
        }
        let cap = self.ledgers[who as usize].cap[g];
        if self.nation[who as usize].human {
            return cap;
        }
        match self.ai_difficulty() {
            0 => cap / 2,
            1 => cap * 3 / 4,
            _ => cap,
        }
    }

    /// `City::count_gather_slots@00737dc0`: over the city's chain, each
    /// gather building's `gather_max` per good — **finished or not**: the
    /// walk tests the type (`+0x90`) and nothing else, so a site that is
    /// placed and not yet started counts its slots from placement, and
    /// run150's packet reads the unstarted Mine `1/2018` holding
    /// `gather_max 3` in the chain (`docs/AI.md` §61) — how much of it is
    /// unfilled, and the total **excluding knowledge** (good 3 — the
    /// original skips it when accumulating its return while still writing
    /// it into the per-good array). Returns `(total, slots, open)`.
    pub fn count_gather_slots(
        &self,
        c: usize,
    ) -> (i32, [i32; economy::RESOURCES], [i32; economy::RESOURCES]) {
        let mut slots = [0; economy::RESOURCES];
        let mut open = [0; economy::RESOURCES];
        let mut total = 0;
        for b in self.city_chain(c) {
            let bd = &self.buildings[b];
            if !bd.alive {
                continue;
            }
            let Some(rec) = bd.ty else { continue };
            if !self.build_types[rec].has(crate::build::flags::GATHER) {
                continue;
            }
            let Some(g) = crate::ai_place::gather_good(self.build_types[rec].ident) else {
                continue;
            };
            let max = bd.gather_max.unwrap_or(0);
            slots[g] += max;
            // `param_2[g] += gather_max − BuildData::num_gatherers(0, 0)`
            // (`00737dc0`, the `param_2 != 0` arm). **`num_gatherers` is two
            // counts, not one**: the `gather_down` chain *plus* `count_inside`
            // for the two kinds whose workers sit inside — the University
            // (`0x1a4`, scholars) and the Oil Platform (`0x1a6`, citizens),
            // and not the Oil Well (`0x1a5`). Reading the chain's raw length
            // here is the defect item 438 measured (`docs/AI.md` §51.1): a
            // University's seated scholars never reduced its free slots, and
            // Great Lakes 9380 answered `k = 7` for both of the AI's cities
            // where the original answers 6 and 3. `docs/AI.md` §53.
            open[g] += (max - self.num_gatherers(b, false, false)).max(0);
            if g != 3 {
                total += max;
            }
        }
        (total, slots, open)
    }

    /// `LeaderData::village_num` — [`crate::ai::Census::village_num`].
    pub fn village_num(&self, who: Player) -> i32 {
        self.ai[who as usize].census.village_num
    }

    /// `world+0x34`, `sea_map`: the map style's sea class every producer
    /// reads ([`crate::world::World::sea_map`] — not a landmass count).
    pub fn sea_map(&self) -> i32 {
        self.world.sea_map()
    }
}
