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
}
