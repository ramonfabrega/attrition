//! Buildings — `Leader::create_buildings@006c1be0`,
//! `Leader::produce_upgrade@006cb5d0`, `Leader::produce_spell@006ca720`;
//! the specification is `~/ghidra-projects/reports/ai/create-buildings.md`
//! §1–§3, §5–§7 (ratified in `docs/AI.md` §11). `produce_building` itself
//! is [`crate::ai_place`].
//!
//! `create_buildings` is two passes (non-gather, then gather) over every
//! city of the leader, and inside each city over every build type: a stack
//! of gates, then a value built family by family, then a divisor that
//! punishes what the leader already owns, then `check_income`'s
//! affordability factor, then one `make_me`. Nothing is bought here — the
//! function only fills the shopping list.
//!
//! Every multiplication is the original's, in the original's order, and
//! deliberately **wrapping**: `v` overflows in the shipped game (a Temple
//! with no Temple owned is `level · 1000 · 10000 · 100`), and the common
//! tail catches the negative and pins it at 9,999,999. A checked multiply
//! here would panic where the original wraps, so `wrapping_mul` is the
//! faithful operator and not a shortcut.
//!
//! **Draws.** The only sync-stream draws on this path are the wonder
//! branch's two, per (city, wonder type) that reaches the no-shortcut arm
//! (`create-buildings.md` §7 rows 1–2). They are taken here at exactly that
//! point, in that order.
//!
//! **Seams.** What the sim does not carry yet, each answering as an empty
//! world would, and each named at its use:
//!
//! - `CityData.bordering` (+0x65) and `city_flags & 0x8` / `& 0x1000`: no
//!   field and no known setter (`create-buildings.md` §9) — read as 0.
//! - `CityData.ocean_filled` (+0x66): no field — `ocean_open` is
//!   [`crate::ai::CityAi::ocean`].
//! - `LeaderData::city_num` / `village_num`: the census does not keep them —
//!   the leader's live city count, and 0.
//! - `world+0x34`, the landmass count the docks read: 1.
//! - the leader's known oil patches: none, so `oil_ok` is never true.
//! - `GoodTypeData::largest_gather` (+0x2ec): 0.
//! - the wonder bookkeeping — team, enemy and unbuilt wonder value,
//!   `Game::wonder_winning`, `wonder_mark`, the wonder-win row's target and
//!   a wonder type's value factor (vslot `+0x118`): 0, −1, 0, absent, 1.
//! - `TERRACOTTA` / `STATUEOFLIBERTY` / `SPACEPROGRAM` have no [`Ident`], so
//!   the tech-race lobby's ÷1000 never fires.
//! - `WorldData::danger` is per region here, not per half-cell.
//! - `build_flags & 0x8000000` is [`flags::DEEP_QUEUE`] and **no shipped
//!   row carries it** — verified against `buildingrules.xml`'s 129
//!   `BUILD_FLAGS` strings and against every writer in the export. It is
//!   implemented literally anyway, so the day a loader sets it the civic
//!   block comes alive on its own.
//! - `produce_upgrade` of a *building* type: the production queue holds
//!   units and technologies, so a building upgrade has nowhere to go.
//! - spells: the simulation has none, so `produce_spell` never casts.

use crate::ai::{Census, MakeObject};
use crate::ai_place::{gather_good, is_enhancer, is_military_trainer};
use crate::ai_types::Class;
use crate::build::{self, BuildDomain, Ident, flags};
use crate::economy::RESOURCES;
use crate::tech::{self, TypeId};
use crate::{Player, Sim};

/// What one type's value pipeline produced for one city, before
/// `check_income` and `make_me`.
struct Listing {
    val: i32,
    escrow: i32,
    cat: i32,
    up: i32,
    /// `check_income`'s multiplier before the `<< 8`: 4, or 2 for a gather
    /// type.
    mult: i32,
}

/// The per-city facts every type in the sweep reads — the head of
/// `create_buildings`' city loop (`create-buildings.md` §2.1).
struct Facts {
    /// The sim's city index; what `MakeObject.city` carries.
    c: usize,
    /// The leader's own city slot, 0-based. Only the oil arm reads it.
    ci: usize,
    /// The city's region, for the per-region census arrays.
    reg: u16,
    /// 1 Small City, 2 Large City, 3 Major City or Forbidden City.
    level: i32,
    /// `CityData::num_buildings` — active members, the centre included.
    nb: i32,
    /// `land − filled`, and `ocean − ocean_filled`.
    open: i32,
    ocean_open: i32,
    /// `CityAi::space`, `CityAi::dock_tile`.
    space: [i32; 3],
    dock_tile: i32,
    /// `City::count_gather_slots`: per good, and the total excluding
    /// universities.
    slots: [i32; RESOURCES],
    open_slots: [i32; RESOURCES],
    total_slots: i32,
    /// The city's own citizen picture, from the census.
    free: i32,
    busy: i32,
    gatherers: i32,
    peasant_dist: i32,
    ter: [i32; RESOURCES],
}

impl Sim {
    // ------------------------------------------------------------------
    // Small helpers the sweep leans on
    // ------------------------------------------------------------------

    /// `type.is(root, 0)` over build records: `t` itself or anything on its
    /// `from` chain.
    fn build_is_rec(&self, t: usize, root: usize) -> bool {
        let mut cur = Some(t);
        let mut guard = 0;
        while let Some(i) = cur {
            if i == root {
                return true;
            }
            cur = self.build_types[i].from;
            guard += 1;
            if guard > self.build_types.len() {
                break;
            }
        }
        false
    }

    /// `CityData::count_buildings(t, exact = 1, active_only)` — the city's
    /// own count of exactly this record, sites included unless `active_only`.
    fn city_count_exact(&self, c: usize, rec: usize, active_only: bool) -> i32 {
        self.city_chain(c)
            .into_iter()
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive && (!active_only || bd.active) && bd.ty == Some(rec)
            })
            .count() as i32
    }

    /// `CityData::count_buildings(t, exact = 0, active_only)` — the same over
    /// the lineage.
    fn city_count_line(&self, c: usize, rec: usize, active_only: bool) -> i32 {
        self.city_chain(c)
            .into_iter()
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive
                    && (!active_only || bd.active)
                    && bd.ty.is_some_and(|t| self.build_is_rec(t, rec))
            })
            .count() as i32
    }

    /// `get_queued(t)` for a **building**: `LeaderData::num_queued[t]` alone.
    /// The recursion into grafted types inside `get_queued@006e0d50` is
    /// gated on `is_unit_type`, so it never runs for a build type — placed
    /// sites of exactly this record are the whole answer.
    fn queued_of(&self, who: Player, rec: usize) -> i32 {
        self.num_sites_of(who, rec)
    }

    /// `get_reg_buildings(reg, t)`: finished buildings of this record and of
    /// everything it upgrades to, standing in one region. The original keeps
    /// a `[64][129]` table; here it is counted, which is the same number.
    fn reg_buildings_of_line(&self, who: Player, reg: u16, rec: usize) -> i32 {
        self.buildings
            .iter()
            .filter(|b| {
                b.alive
                    && b.active
                    && b.owner == who
                    && b.ty.is_some_and(|t| self.reg_line_member(t, rec))
                    && self.world.region_of(b.pos.cell()) == Some(reg)
            })
            .count() as i32
    }

    /// Whether `t` is `rec` or on `rec`'s `to` chain — `get_buildings`' walk.
    fn reg_line_member(&self, t: usize, rec: usize) -> bool {
        let mut cur = Some(rec);
        let mut guard = 0;
        while let Some(i) = cur {
            if i == t {
                return true;
            }
            cur = self.build_types[i].to;
            guard += 1;
            if guard > self.build_types.len() {
                break;
            }
        }
        false
    }

    /// `reg_buildings[reg][t]` alone.
    fn reg_buildings_raw(&self, who: Player, reg: u16, rec: usize) -> i32 {
        self.buildings
            .iter()
            .filter(|b| {
                b.alive
                    && b.active
                    && b.owner == who
                    && b.ty == Some(rec)
                    && self.world.region_of(b.pos.cell()) == Some(reg)
            })
            .count() as i32
    }

    /// `BuildTypeData::is_training_building` — `build_flags & 0x80000000`,
    /// which no `BUILD_FLAGS` string carries: the loader must set it from the
    /// unit tables' `WHERE`, so it is derived here the same way.
    fn is_training_building(&self, rec: usize) -> bool {
        let Some(tree) = self.build_types[rec].tree else {
            return false;
        };
        self.tech_tree
            .types
            .iter()
            .any(|d| d.kind.is_unit() && d.where_ == Some(tree))
    }

    /// `BuildTypeData::is_defensive` — a tower, a fort, an Airbase, or
    /// anything that shoots.
    fn is_defensive_type(&self, rec: usize) -> bool {
        build::is_tower(&self.build_types, rec)
            || build::is_fort(&self.build_types, rec)
            || build::is(&self.build_types, rec, Ident::Airbase)
            || self.build_types[rec].attack != 0
    }

    /// `BuildTypeData::get_enhancing_good@00639880` — an exact-type switch,
    /// not a lineage test.
    fn enhancing_good(ident: Ident) -> Option<usize> {
        Some(match ident {
            Ident::Granary => 0,
            Ident::Lumbermill => 1,
            Ident::Smelter => 4,
            Ident::Refinery => 5,
            _ => return None,
        })
    }

    /// The tree id of the `g`th good, in `goodrules.xml` order.
    fn good_type(&self, g: usize) -> Option<TypeId> {
        self.tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .nth(g)
    }

    /// The tree id of the first build record with this identity.
    fn ident_tree(&self, ident: Ident) -> Option<TypeId> {
        self.build_types
            .iter()
            .find(|b| b.ident == ident)
            .and_then(|b| b.tree)
    }

    /// `LeaderData::has_tech(t)` for a named building type.
    fn has_building_tech(&self, who: Player, ident: Ident) -> bool {
        self.ident_tree(ident).is_some_and(|t| {
            self.tech_tree
                .has_tech(&self.setup, &self.tech[who as usize], t)
        })
    }

    /// `LeaderData::get_highest_epoch` — the largest of the four lines, never
    /// below zero.
    fn highest_epoch(&self, who: Player) -> i32 {
        self.tech[who as usize]
            .epoch
            .iter()
            .fold(0, |a, &b| a.max(b))
    }

    /// `find_capital(−1, −1)` answering someone else: the leader has no live
    /// capital of its own and another leader holds a city that was one.
    fn capital_lost(&self, who: Player) -> bool {
        if self
            .cities
            .iter()
            .any(|c| c.alive && c.owner == who && c.capital)
        {
            return false;
        }
        self.cities
            .iter()
            .any(|c| c.alive && c.owner != who && c.was_capital & (1u64 << (who as u32)) != 0)
    }

    /// `City::count_gather_slots@00737dc0`: per good, the chain's gather
    /// capacity and how much of it is unfilled, and the total capacity
    /// **excluding universities** (good 3).
    fn city_gather_slots(&self, c: usize) -> ([i32; RESOURCES], [i32; RESOURCES], i32) {
        let (total, slots, open) = self.count_gather_slots(c);
        (slots, open, total)
    }

    // ------------------------------------------------------------------
    // `create_buildings`
    // ------------------------------------------------------------------

    /// `create_buildings`: every building type the leader could place,
    /// valued per city and offered to the make list.
    pub fn create_buildings(&mut self, who: Player) {
        let w = who as usize;
        if w >= self.ai.len() {
            return;
        }
        let cities = self.cities_of(who);
        for pass in 0..2u32 {
            for (ci, &c) in cities.iter().enumerate() {
                let Some(f) = self.city_facts(who, c, ci) else {
                    continue;
                };
                for rec in 0..self.build_types.len() {
                    let Some(l) = self.building_value(who, &f, rec, pass) else {
                        continue;
                    };
                    let Some(t) = self.build_types[rec].tree else {
                        continue;
                    };
                    // `check_income(t, mult << 8, o = −1, escrow, city, 1,
                    // NULL)`, then `val = income · v / 256`.
                    let inc = self.check_income(
                        who,
                        t,
                        l.mult << 8,
                        Some(c),
                        l.escrow != 0,
                        -1,
                        1,
                        l.escrow,
                    );
                    let val = inc.wrapping_mul(l.val) / 256;
                    self.ai[w]
                        .make_list
                        .make_me(t as i32, val, l.escrow, l.cat, c as i32, l.up, 1, 0, 0);
                }
            }
        }
    }

    /// The city loop's head — `create-buildings.md` §2.1. `None` when the
    /// city is skipped outright.
    fn city_facts(&self, who: Player, c: usize, ci: usize) -> Option<Facts> {
        let city = &self.cities[c];
        let b = city.building;
        // "No buildings for a city still assimilating after capture": the
        // record's `race` must already be the city building's owner.
        if city.race != Some(self.buildings[b].owner) {
            return None;
        }
        let level = self.city_level_of(c).max(1);
        let nb = self.num_buildings(c);
        let ai = self.ai[who as usize].city_ai.get(c)?;
        let open = ai.land - ai.filled;
        // `ocean_filled` (+0x66) has no field here.
        let ocean_open = ai.ocean;
        if ai.space[0] == 0 && ocean_open == 0 {
            return None;
        }
        let (slots, open_slots, total_slots) = self.city_gather_slots(c);
        Some(Facts {
            c,
            ci,
            reg: city.reg?,
            level,
            nb,
            open,
            ocean_open,
            space: ai.space,
            dock_tile: ai.dock_tile,
            slots,
            open_slots,
            total_slots,
            free: ai.free,
            busy: ai.busy,
            gatherers: ai.gatherers,
            peasant_dist: ai.peasant_dist,
            ter: ai.ter,
        })
    }

    /// One type's whole value pipeline for one city — §2.2 gates, then §3.1
    /// or §3.2, then the common tails, then the divisor. `None` is the
    /// original's `goto LAB_006c3fbe`: the type is not listed.
    ///
    /// Takes `&mut self` for one reason: the wonder branch draws.
    #[allow(clippy::too_many_lines)]
    fn building_value(&mut self, who: Player, f: &Facts, rec: usize, pass: u32) -> Option<Listing> {
        let w = who as usize;
        let bt = self.build_types[rec].clone();
        let ident = bt.ident;
        if build::is_city(&self.build_types, rec) {
            return None;
        }
        let gather = bt.has(flags::GATHER);
        if (pass == 1) != gather {
            return None;
        }
        let tree = bt.tree?;
        if self.type_avail(who, tree) != tech::AVAILABLE {
            return None;
        }
        // The capital countdown: with my capital in someone else's hands,
        // only military trainers are listed.
        if self.lobby.elimination == 1 && self.capital_lost(who) && !is_military_trainer(ident) {
            return None;
        }
        let n = (bt.x_size.max(bt.y_size) - 2).clamp(0, 2) as usize;
        let fit = f.space[n];
        let dock = build::is_dock(&self.build_types, rec);
        let water = bt.domain() == BuildDomain::Water;
        if water {
            if f.ocean_open == 0 {
                return None;
            }
            if dock && f.dock_tile == 0 {
                return None;
            }
        } else if fit == 0 {
            return None;
        }
        let mut cat = 8;
        let mut escrow = 0;
        let mut up = 0;
        // A type with a `FROM` that does not upgrade in place is queued at
        // the building its `WHERE` names, which must already stand here.
        if bt.from.is_some() && !bt.has(flags::UPGRADES) {
            up = 1;
            let where_rec = self.tech_tree.types[tree]
                .where_
                .and_then(|x| self.build_record(x))?;
            if self.city_count_exact(f.c, where_rec, true) == 0 {
                return None;
            }
        }
        let mut mult = 4;
        // Water types other than docks are never listed; no Lookout under
        // rush rules; a one-per-city type not already here.
        if water && !dock {
            return None;
        }
        if self.lobby.rush_rules == 8 && build::is(&self.build_types, rec, Ident::Lookout) {
            return None;
        }
        if bt.has(flags::ONE_PER_CITY) && self.city_count_line(f.c, rec, false) != 0 {
            return None;
        }
        // The three named civic gates, on exact identity.
        let mut check_senate = true;
        if ident == Ident::University {
            if f.nb < 3 {
                return None;
            }
        } else if ident == Ident::Library {
            if self.buildings_of_line(who, rec) != 0 {
                if f.nb < 3 {
                    return None;
                }
            } else {
                check_senate = false;
            }
        }
        if check_senate
            && ident == Ident::Senate
            && self.buildings_of_line(who, rec) + self.queued_of(who, rec) > 0
        {
            return None;
        }
        // At most three of a non-military, non-gather, non-Temple kind.
        if !gather
            && !is_enhancer(ident)
            && bt.attack == 0
            && !build::is(&self.build_types, rec, Ident::Temple)
            && self.buildings_of_line(who, rec) + self.queued_of(who, rec) > 2
        {
            return None;
        }
        let have = self.city_count_exact(f.c, rec, false);
        if ident == Ident::Farm && have >= self.farm_limit(f.c) {
            return None;
        }

        let mut v: i32;
        let mut wonder_branch = false;
        if !gather {
            // ---- §3.1 the non-gather base ----
            v = f.level.wrapping_mul(1000);
            let reg_cities = Census::reg(&self.ai[w].census.reg_cities, f.reg);
            if reg_cities <= self.reg_buildings_raw(who, f.reg, rec) + self.queued_of(who, rec) {
                v /= reg_cities + 1;
            }
            if ident == Ident::Temple {
                if !self.cities[f.c].capital && self.ai_difficulty() < 3 {
                    return None;
                }
                let mut t = v.wrapping_mul(500);
                escrow = 1;
                if self.num_buildings_of(who, rec) == 0 {
                    t = v.wrapping_mul(10000);
                }
                // `CityData.bordering` has no field here.
                v = t;
            }
            if build::is(&self.build_types, rec, Ident::Lookout) {
                if self.tech[w].epoch[3] < 4 {
                    return None;
                }
                if self.tech[w].ages < 4 {
                    if have != 0 {
                        return None;
                    }
                    v = v.wrapping_mul(100);
                    escrow = 1;
                } else if have < 2 {
                    v = v.wrapping_mul(100);
                    escrow = 1;
                } else if have > 2 {
                    return None;
                }
            }
            if bt.wonder {
                wonder_branch = true;
            } else if self.ai[w].wonder_mod != 0 {
                // A wonder is wanted: everything else is worth a hundredth.
                v /= 100;
            }
        } else {
            // ---- §3.2 the gather types ----
            let (nv, ncat, nmult) = self.gather_value(who, f, rec, up, &mut escrow)?;
            v = nv;
            cat = ncat;
            mult = nmult;
        }

        if wonder_branch {
            v = self.wonder_value(who, f, rec, &mut escrow)?;
        }

        // ---- §3.3 the common tail, every family ----
        if up == 0 {
            if f.peasant_dist > 4 {
                v /= (f.peasant_dist + 5) / 5;
            }
            if f.free == 0 && Census::reg(&self.ai[w].census.reg_free_peasants, f.reg) != 0 {
                v = v.wrapping_mul(3) / 4;
                if f.gatherers == 0 && f.busy != 0 {
                    v /= 2;
                }
            }
            if !water && !bt.has(flags::NO_CITY) {
                if f.open < 2 && fit < 2 {
                    return None;
                }
                let land = self.ai[w].city_ai[f.c].land;
                if land * 7 / 8 <= f.open {
                    v = v.wrapping_mul(4) / 3;
                }
                if f.open < 2 && fit < 3 {
                    v = v.wrapping_mul(3) / 4;
                }
            }
        }

        // ---- §3.4 the gather enhancers ----
        if is_enhancer(ident) {
            if self.lobby.starting_resources == 8 {
                return None;
            }
            if self.lobby.victory == 8 {
                v = v.wrapping_mul(100);
            }
            v = v.wrapping_mul(self.ai[w].infra_mod) / 256;
            cat = 4;
            let g = Self::enhancing_good(ident)?;
            let mut nfilled = f.slots[g] - f.open_slots[g];
            if g == 5 {
                nfilled = self.ai[w].census.gather_slots[5] * 2;
            }
            let avail = self
                .good_type(g)
                .is_some_and(|t| self.type_available(who, t));
            let cap = self.ledgers[w].cap[g];
            let lakota = self
                .tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[w], 0x13);
            if !(avail
                && self.ledgers[w].income[g] < cap
                && (nfilled > 2 || g == 5)
                && (ident != Ident::Granary || !lakota)
                && (f.open > 3 || g == 5))
            {
                return None;
            }
            let econ = self.ai[w].econ[g];
            v = nfilled.wrapping_mul(nfilled).wrapping_mul(v);
            if econ & 4 != 0 {
                v = v.wrapping_mul(2);
            }
            if econ & 2 != 0 {
                v = v.wrapping_mul(3);
                escrow = 1;
            }
            if econ & 1 != 0 {
                v = v.wrapping_mul(4);
                escrow = 1;
            }
        }

        // ---- §3.5 military trainers, then the dock family ----
        let deep = bt.has(flags::DEEP_QUEUE);
        if !deep && !dock && is_military_trainer(ident) {
            if self.lobby.rush_rules == 8 || !(have == 0 || up != 0) {
                return None;
            }
            cat = 7;
            let cen = &self.ai[w].census;
            let k = cen.wars
                + Census::reg(&cen.reg_neutrals, f.reg)
                + Census::reg(&cen.reg_wars, f.reg)
                + 3;
            v = v.wrapping_mul(k);
            let raw = self.num_buildings_of(who, rec);
            let rawq = self.queued_of(who, rec);
            if self.reg_buildings_of_line(who, f.reg, rec) == 0 && rawq == 0 {
                let doubled = v.wrapping_mul(2);
                let bs = build::is(&self.build_types, rec, Ident::Barracks)
                    || build::is(&self.build_types, rec, Ident::Stable)
                    || (build::is(&self.build_types, rec, Ident::SiegeFactory)
                        && raw == 0
                        && rawq == 0);
                v = if bs { v.wrapping_mul(400) } else { doubled };
            }
            if raw == 0 && rawq == 0 {
                escrow = 1;
            }
            let age = self.tech[w].ages;
            let total = rawq + raw;
            if total < (age + 2) / 2 {
                v = v.wrapping_mul(100);
            }
            if !(self.lobby.starting_resources == 8 || total <= age) {
                return None;
            }
            // The nation's favourite trainer. The Dock arm of this chain is
            // unreachable: a dock never gets here.
            let tribe = self.tech[w].tribe;
            let quad = if build::is(&self.build_types, rec, Ident::Barracks) {
                tribe == 0 || tribe == 6
            } else if build::is(&self.build_types, rec, Ident::Stable) {
                tribe == 0x11
            } else if build::is(&self.build_types, rec, Ident::SiegeFactory) {
                tribe == 8
            } else if build::is(&self.build_types, rec, Ident::Airbase) {
                tribe == 0xc
            } else {
                false
            };
            if quad {
                v = v.wrapping_mul(4);
            }
        }
        if dock {
            // The dock family, and everything `0x8000000` would add to it.
            let age = self.tech[w].ages;
            let rawq = self.queued_of(who, rec);
            let raw = self.num_buildings_of(who, rec);
            if !((have == 0 || up != 0)
                && (self.lobby.starting_resources == 8 || rawq + raw <= age))
            {
                return None;
            }
            let base = v.wrapping_mul(self.ai[w].sea_mod) / 256;
            let mut d = base;
            if self.buildings_of_line(who, rec) == 0 && rawq == 0 {
                d = base.wrapping_mul(4);
                if self.sea_map() > 2 {
                    escrow = 1;
                    d = base.wrapping_mul(0x50);
                }
            }
            if !(self.sea_map() > 2 || self.city_num(who) > 1) {
                return None;
            }
            if self.reg_buildings_of_line(who, f.reg, rec) == 0 {
                d = d.wrapping_mul(8);
            }
            let cen = &self.ai[w].census;
            d = if Census::reg(&cen.reg_wars, f.reg) == 0 {
                if cen.active_wars != 0 {
                    d.wrapping_mul(4)
                } else {
                    d.wrapping_mul(2)
                }
            } else {
                d / 2
            };
            // The landmass count multiplies the dock's value — settled in
            // the listing (`imull %eax, %edi` at 0x6c331d), which the first
            // reading's §3.5 does not have.
            v = self.sea_map().wrapping_mul(d);
        }

        // ---- §3.6 the civic block onward. A dock reaches it too: its own
        // branch ends in `goto LAB_006c3320`, which is this block's head.
        {
            let is_market = build::is(&self.build_types, rec, Ident::Market);
            let is_temple = build::is(&self.build_types, rec, Ident::Temple);
            let is_library = build::is(&self.build_types, rec, Ident::Library);
            let is_senate = build::is(&self.build_types, rec, Ident::Senate);
            let civic = (deep
                && !build::is(&self.build_types, rec, Ident::University)
                && !is_enhancer(ident))
                || is_market;
            if civic {
                let owned = self.buildings_of_line(who, rec);
                let queued = self.queued_of(who, rec);
                let city_num = self.city_num(who);
                if !is_temple {
                    if up == 0 && city_num <= 2 * (owned + queued) {
                        return None;
                    }
                    if self.lobby.starting_resources != 8
                        && self.tech[w].ages < 2 * (owned + queued)
                    {
                        return None;
                    }
                }
                if self.reg_buildings_of_line(who, f.reg, rec) == 0 {
                    let mut m = 10;
                    if is_temple {
                        m = 18;
                        v = v.wrapping_mul(self.ai[w].infra_mod) / 256;
                        let cen = &self.ai[w].census;
                        if cen.min_other_team_terr <= cen.my_team_terr {
                            m = 9;
                        }
                        if self.ai[w].econ[2] & 3 != 0 {
                            m *= 2;
                        }
                    }
                    if is_market {
                        m += 1;
                        if self.ai[w].econ[2] & 3 != 0 {
                            m *= 2;
                        }
                    }
                    if queued == 0 {
                        if owned == 0 {
                            if is_library {
                                m *= 2;
                            }
                            m *= 2;
                        }
                    } else {
                        m >>= 1;
                    }
                    if city_num == 1 {
                        m >>= 2;
                    } else if city_num == 2 {
                        m >>= 1;
                    } else if city_num > 2 && self.num_buildings_of(who, rec) == 0 {
                        escrow = 1;
                        m = city_num * m / 2;
                    }
                    if m > 1 {
                        v = v.wrapping_mul(m);
                    }
                }
                if is_library && self.buildings_of_line(who, rec) < 2 && city_num > 3 {
                    v = v.wrapping_mul(self.ai[w].infra_mod).wrapping_mul(10000) / 256;
                }
                if is_market && self.buildings_of_line(who, rec) < 1 {
                    v = v.wrapping_mul(self.ai[w].infra_mod).wrapping_mul(10000) / 256;
                }
                if is_senate {
                    if self.cities[f.c].no_heal || f.nb < 5 {
                        return None;
                    }
                    v = if self.highest_epoch(who) > 2 {
                        v.wrapping_mul(100)
                    } else {
                        v.wrapping_mul(10)
                    };
                }
            }

            // ---- §3.7 towers, forts, airbases ----
            let tower = build::is(&self.build_types, rec, Ident::Tower);
            let fort = build::is_fort(&self.build_types, rec);
            if tower || fort {
                let dmod = self.ai[w].defense_mod;
                let d = dmod.wrapping_mul(v);
                if self.lobby.rush_rules == 8 {
                    let cen = &self.ai[w].census;
                    if !fort || cen.other_team_terr < cen.my_team_terr {
                        return None;
                    }
                }
                let diff = self.ai_difficulty();
                let age = self.tech[w].ages;
                if dmod <= 0x100
                    && (diff == 0
                        || (diff == 1
                            && age < self.queued_of(who, rec) + self.num_buildings_of(who, rec)))
                {
                    return None;
                }
                let k = if f.nb - have < 4 {
                    1
                } else {
                    (f.nb - have) / 2
                };
                let mut x = (d / 256).wrapping_mul(k);
                // `CityData.bordering` reads 0 here: a fort is worth a
                // hundredth, a tower unchanged.
                if fort {
                    x /= 100;
                }
                // `city_flags & 0x1000` reads 0; `& 0x8` reads 0, so the
                // halving always applies.
                x /= 2;
                if self.cities[f.c].founder != who {
                    x = x.wrapping_mul(3);
                }
                if self.city_unassimilated(f.c) {
                    x = x.wrapping_mul(3) / 4;
                }
                if Census::reg(&self.ai[w].census.reg_wars, f.reg) == 0 {
                    x /= 2;
                }
                if !fort {
                    v = x / (have / 2 + 1);
                    if have == 0 {
                        if diff > 2 {
                            escrow = 1;
                            v = v.wrapping_mul(10);
                        }
                        if self.cities[f.c].capital {
                            escrow = 1;
                        }
                    }
                } else {
                    let cen = &self.ai[w].census;
                    if cen.my_team_terr < cen.min_other_team_terr {
                        x = x.wrapping_mul(2);
                    }
                    let base = x.wrapping_mul(3) / (have + 1);
                    if self.buildings_of_line(who, rec) + self.queued_of(who, rec) == 0 {
                        escrow = 1;
                    }
                    // Central cities want forts.
                    let cell = self.cities[f.c].pos.cell();
                    let (xs, ys) = (self.world.width(), self.world.height());
                    let dx = xs / 2 - cell.x;
                    let dy = ys / 2 - cell.y;
                    let reach = (xs / 2 + ys / 2 - dx.abs() - dy.abs()) / 10 + 1;
                    v = reach.wrapping_mul(base);
                }
            }
            if build::is(&self.build_types, rec, Ident::Airbase) {
                if self.lobby.rush_rules == 8 {
                    return None;
                }
                let n = self.queued_of(who, rec) + self.reg_buildings_of_line(who, f.reg, rec);
                let mut x = if n == 0 { v.wrapping_mul(10) } else { v / n };
                if Census::reg(&self.ai[w].census.reg_wars, f.reg) == 0 {
                    x /= 2;
                }
                if n == 0 {
                    escrow = 1;
                }
                v = x.wrapping_mul(self.ai[w].air_mod) / 256;
            }
        }

        // ---- §3.8 the common tail, everything ----
        if Census::reg(&self.ai[w].census.reg_free_peasants, f.reg) == 0 {
            v = v.wrapping_mul(4) / 5;
        }
        if v < 0 {
            v = if bt.wonder { 9_999_075 } else { 9_999_999 };
        }
        if self.cities[f.c].no_heal
            && !self.is_defensive_type(rec)
            && (!is_military_trainer(ident) || self.city_count_line(f.c, rec, true) != 0)
        {
            v /= 100;
        }

        // ---- §3.10 the divisor and the head check ----
        if !gather {
            let queued = self.queued_of(who, rec);
            let reg = self.reg_buildings_of_line(who, f.reg, rec);
            let d = if dock {
                (queued + 2 * have) * 4 + 1 + reg
            } else if is_military_trainer(ident) {
                (have + queued) * 4 + 1 + reg
            } else if self.is_training_building(rec) {
                (reg + (queued + 2 * have) * 2) * 2 + 1
            } else if build::is(&self.build_types, rec, Ident::Tower) {
                (queued + have) * 4 + 1
            } else if build::is_fort(&self.build_types, rec) {
                (queued + 2 * have) * 4 + 1
            } else {
                (self.buildings_of_line(who, rec) + 2 * have + queued) * 4 + 1
            };
            v /= d;
        } else {
            v /= self.queued_of(who, rec) + 1;
            let head: MakeObject = self.ai[w].make_list.list[0];
            if v < head.val && head.t >= 0 && !self.head_affordable(who, &head) {
                let ht = head.t as usize;
                let head_gather = matches!(self.type_class(ht), Class::Build)
                    && self
                        .build_record(ht)
                        .is_some_and(|r| self.build_types[r].has(flags::GATHER));
                if head_gather {
                    escrow = 1;
                } else {
                    let mine = gather_good(ident);
                    let mut took = false;
                    for g in 0..RESOURCES {
                        let avail = self
                            .good_type(g)
                            .is_some_and(|t| self.type_available(who, t));
                        let cost = self.type_price(who, ht).map_or(0, |c| c[g]);
                        if mine == Some(g) && avail && self.ledgers[w].bucket[g] < cost {
                            v = v.wrapping_mul(2).min(head.val);
                            escrow = 1;
                            took = true;
                            break;
                        }
                    }
                    if !took {
                        escrow = 1;
                    }
                }
            }
        }
        Some(Listing {
            val: v,
            escrow,
            cat,
            up,
            mult,
        })
    }

    /// `Leader::can_pay(0)`: the head entry's type, at its own escrow
    /// permission, is affordable `num` times over.
    fn head_affordable(&self, who: Player, head: &MakeObject) -> bool {
        if head.t < 0 {
            return false;
        }
        let t = head.t as usize;
        if t >= self.tech_tree.types.len() {
            return false;
        }
        self.type_affordable(who, t, head.escrow != 0) >= head.num
    }

    /// §3.2, the gather families. Returns `(v, cat, mult)` and writes the
    /// escrow flag through.
    #[allow(clippy::too_many_lines)]
    fn gather_value(
        &self,
        who: Player,
        f: &Facts,
        rec: usize,
        up: i32,
        escrow_in: &mut i32,
    ) -> Option<(i32, i32, i32)> {
        let w = who as usize;
        let mut escrow = *escrow_in;
        if self.lobby.starting_resources == 8 {
            return None;
        }
        let ident = self.build_types[rec].ident;
        let good = gather_good(ident)?;
        let cen = &self.ai[w].census;
        if cen.filled_gather_slots[good] < cen.gather_slots[good] * 3 / 4 {
            return None;
        }
        let cv = (self.city_num(who) + self.village_num(who)).max(1);
        let mut base = (cen.pop.wrapping_mul(7000) / cv
            + f.level.wrapping_mul(3000)
            + f.open.wrapping_mul(4000))
        .wrapping_mul(self.ai[w].infra_mod)
            / 256;
        if self.lobby.victory == 8 {
            base = base.wrapping_mul(100);
        }
        if self.num_buildings_of(who, rec) == 0 {
            base = base.wrapping_mul(5);
        }
        if ident == Ident::University {
            let raw = self.num_buildings_of(who, rec);
            base = if raw == 0 || raw * 3 < cen.scholars {
                base.wrapping_mul(10)
            } else {
                base / 10
            };
        }
        if self.cities[f.c].members.is_empty() {
            // The city building has no member below it yet.
            base = base.wrapping_mul(10);
        } else {
            if f.level == 1
                && self.has_building_tech(who, Ident::Town)
                && f.nb < self.tuning.city_buildings + 1
            {
                base = base.wrapping_mul(2);
            }
            if f.level == 2
                && self.has_building_tech(who, Ident::Metropolis)
                && f.nb < self.tuning.metro_buildings + 1
            {
                base = base.wrapping_mul(2);
            }
        }
        if self.cities[f.c].capital {
            base /= 2;
        }
        let cat = 4;
        let mut found = false;
        let mut cap = 0x100;
        let free_peasants = Census::reg(&cen.reg_free_peasants, f.reg);
        if free_peasants == 0 && self.muster[w].cap - self.muster[w].control <= 1 {
            return None;
        }
        let worst = self.ai[w].worst_good;
        let best = self.ai[w].best_good;
        if free_peasants > 2 && self.ai[w].rate[worst] < self.ledgers[w].cap[worst] / 32 {
            escrow = 1;
        }
        for g in 0..RESOURCES {
            let avail = self
                .good_type(g)
                .is_some_and(|t| self.type_available(who, t));
            if !(good == g
                && avail
                && self.ledgers[w].income[g] < self.ledgers[w].cap[g]
                && f.open_slots[g] < 2)
            {
                continue;
            }
            // `oil_patches.count` reads 0, so `oil_ok` is never true.
            let mut oil_ok = false;
            let mut m0 = 0x100;
            if g == 5 {
                oil_ok = f.ci == 0 && self.ai[w].census.gather_slots[5] < Self::OIL_PATCHES;
                base = base.wrapping_mul(10);
            }
            let ter = f.ter[g];
            let mut m;
            if g == worst {
                if !(ter != 0 || oil_ok) {
                    continue;
                }
                m = if g == 5 {
                    0x800
                } else {
                    // `largest_gather` reads 0.
                    (3.max(ter) + 2) * 0x100
                };
            } else {
                if !(ter != 0 || oil_ok) {
                    continue;
                }
                if (g == 0 || g == 1 || g == 2) && self.city_num(who) + self.village_num(who) < 5 {
                    m0 = 0x200;
                }
                let n = (self.build_types[rec]
                    .x_size
                    .max(self.build_types[rec].y_size)
                    - 2)
                .clamp(0, 2) as usize;
                let fit = f.space[n];
                if up == 0 {
                    if fit < 2 {
                        continue;
                    }
                    if fit < 3 && self.ai[w].econ[g] & 2 == 0 {
                        m0 >>= 1;
                    }
                    if fit < 4 && ter < 2 && g != 5 {
                        m0 >>= 1;
                    }
                }
                let econ = self.ai[w].econ[g];
                let k = if econ & 8 != 0 {
                    4
                } else if econ & 2 != 0 {
                    2
                } else {
                    3
                };
                m = (k + 3.max(ter)).wrapping_mul(m0) / k;
            }
            found = true;
            if self.ai[w].rate[g] < self.cities[f.c].pop * 20 {
                m = m.wrapping_mul(2);
            }
            if g == worst {
                m = m.wrapping_mul(2);
            }
            if g == best {
                m /= 2;
            }
            let econ = self.ai[w].econ[g];
            if econ & 8 != 0 {
                m /= 2;
            }
            if ter > 1 || g == 5 {
                if econ & 2 != 0 {
                    m = m.wrapping_mul(2);
                }
                if econ & 1 != 0 {
                    m = m.wrapping_mul(2);
                }
                if econ & 4 != 0 {
                    m = m.wrapping_mul(3) / 2;
                }
            }
            if g == 0 {
                m = m.wrapping_mul(2);
            }
            let fb = f.free + f.busy;
            if fb * 2 < f.total_slots {
                m = m.wrapping_mul(2) / 3;
            }
            if fb * 3 < f.total_slots {
                m = m.wrapping_mul(2) / 3;
            }
            if f.free != 0 {
                if f.total_slots < f.gatherers + f.free {
                    m = m.wrapping_mul(4) / 3;
                }
                if f.total_slots == 0 {
                    m = m.wrapping_mul(3);
                }
            }
            if econ & 8 == 0
                && (ter > 1 || g == 5)
                && self.city_num(who) + self.village_num(who) > 1
                && self.city_count_line(f.c, rec, true) == 0
            {
                m = m.wrapping_mul(3) / 2;
            }
            // `cmpl %esi, -0x14(%ebp); cmovll` at 0x6c2762: `m = min(m,
            // cap)`, and the cap then ratchets down to `m`. So the whole
            // multiplier saturates at ×1 and the doublings decide only
            // whether it gets there.
            if cap < m {
                m = cap;
            }
            cap = m;
        }
        if !found {
            return None;
        }
        let mut v = base.wrapping_mul(cap) / 256;
        if v < 0 {
            v = 9_999_999;
        }
        *escrow_in = escrow;
        Some((v, cat, 2))
    }

    /// `oil_patches.count` — the leader's known oil patches; none are
    /// modelled, so an oil well is never the reason a good is wanted.
    const OIL_PATCHES: i32 = 0;

    /// §3.9, the wonder branch. Draws twice from the sync stream on the
    /// no-shortcut arm, and only there.
    fn wonder_value(
        &mut self,
        who: Player,
        f: &Facts,
        rec: usize,
        escrow: &mut i32,
    ) -> Option<i32> {
        let w = who as usize;
        if self.lobby.conquest
            || self.num_wonders(f.c, true) != 0
            || (self.lobby.starting_resources == 7 && self.city_num(who) <= 1)
        {
            return None;
        }
        // Every live leader: an ally (myself included) already building it
        // drops the type; a rival more than a quarter of the way through
        // does too. Both arms read `num_queued[t]`, which for a building is
        // its unfinished sites — a wonder already standing is somebody
        // else's problem. The original's `job_counter / construct_time >
        // 0.25` is a compare of two integers widened to a float; `4 · job >
        // time` is the same verdict without one.
        for p in 0..self.players.len() {
            let other = p as Player;
            if self.defeated.get(p).copied().unwrap_or(false) {
                continue;
            }
            let sites: Vec<usize> = self
                .buildings
                .iter()
                .enumerate()
                .filter(|(_, b)| b.alive && !b.active && b.owner == other && b.ty == Some(rec))
                .map(|(i, _)| i)
                .collect();
            if self.is_ally(who, other) || other == who {
                if !sites.is_empty() {
                    return None;
                }
                continue;
            }
            for b in sites {
                let bd = &self.buildings[b];
                if bd.job_counter.wrapping_mul(4) > bd.constr_time {
                    return None;
                }
            }
        }
        let val = if self.ai[w].wonder_mod == 0 {
            let diff = self.ai_difficulty();
            // The wonder bookkeeping reads empty: no team value, no enemy
            // value, nobody winning, and no wonder-win row to halve.
            let team_value: i32 = 0;
            let ev: i32 = 0;
            let winning: i32 = -1;
            // `LeaderData::wonder_mark` (+0x424) has no field here and reads
            // zero, so an easy AI's non-wonder-victory gate always passes.
            let wonder_mark = 0;
            if diff < 2 {
                let ok = if self.lobby.victory == 6 {
                    team_value < ev + 1
                } else {
                    wonder_mark == 0
                };
                if !ok {
                    return None;
                }
            }
            let mut val = if self.lobby.victory == 6 {
                *escrow = 1;
                (ev + 1).wrapping_mul(100_000)
            } else if (winning >= 0 && winning != who as i32) || ev > 4 {
                *escrow = 1;
                2_000_000
            } else {
                let r1 = self.rng.roll();
                let r2 = self.rng.roll();
                (team_value + 1)
                    .wrapping_mul(ev + 1)
                    .wrapping_mul(r2 % 300)
                    .wrapping_mul(r1 % 1000)
            };
            if diff == 0 {
                val /= 100;
            }
            if diff == 1 {
                val /= 10;
            }
            val
        } else {
            *escrow = 1;
            100_000_000
        };
        // `type.vslot(+0x118)`, a wonder type's value factor, reads 1.
        Some(self.city_level_of(f.c).wrapping_mul(val))
    }

    // ------------------------------------------------------------------
    // The two producers
    // ------------------------------------------------------------------

    /// `produce_upgrade(t, city, escrow)`: `true` when queued.
    ///
    /// The city's chain is walked from the centre; a member that is the
    /// building `t`'s `WHERE` names, is active, is not neutralized and does
    /// not already hold `t` in its queue, scores
    /// `level · 1,000,000 / (queued + 1)`, halved in an unassimilated city
    /// and divided by the local danger plus one. The best takes the order;
    /// ties go to the member met first. No draws.
    pub fn produce_upgrade(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        escrow: i32,
    ) -> bool {
        let _ = escrow;
        let Some(c) = city.filter(|&c| c < self.cities.len() && self.cities[c].alive) else {
            return false;
        };
        let level = self.city_level_of(c).max(1);
        let strict = matches!(self.type_class(t), Class::Build);
        let Some(where_t) = self.tech_tree.types[t].where_ else {
            return false;
        };
        let Some(where_rec) = self.build_record(where_t) else {
            return false;
        };
        let mut best: Option<(i32, usize)> = None;
        for b in self.city_chain(c) {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active || bd.owner != who {
                continue;
            }
            let Some(rec) = bd.ty else { continue };
            let matches_where = if strict {
                rec == where_rec
            } else {
                self.build_is_rec(rec, where_rec)
            };
            if !matches_where {
                continue;
            }
            if self.queue_holds(b, t) {
                continue;
            }
            let mut score = level.wrapping_mul(1_000_000) / (bd.queue.items.len() as i32 + 1);
            if self.building_unassimilated(b) {
                score /= 2;
            }
            let danger = self
                .world
                .region_of(bd.pos.cell())
                .map_or(0, |r| self.world.danger(who, r))
                .max(0);
            score /= danger + 1;
            if best.is_none_or(|(v, _)| v < score) {
                best = Some((score, b));
            }
        }
        let Some((_, b)) = best else { return false };
        match self.type_class(t) {
            Class::Tech => self.queue_tech(b, t).is_ok(),
            Class::Unit => self
                .unit_record(t)
                .is_some_and(|r| self.queue_up(b, r).is_ok()),
            // A building upgrade has nowhere to go: the production queue
            // holds units and technologies. Named in the module header.
            _ => false,
        }
    }

    /// `count_queue(1, t)`: whether this building's queue already holds `t`.
    fn queue_holds(&self, b: usize, t: TypeId) -> bool {
        let unit = self.unit_record(t);
        self.buildings[b].queue.items.iter().any(|i| match i.tech {
            Some(x) => x == t,
            None => Some(i.ty) == unit,
        })
    }

    /// `produce_spell(t, city, escrow)`: `true` when cast.
    ///
    /// The original walks every live city holding the spell's `WHERE`
    /// building, scores each candidate the way `produce_upgrade` does and
    /// queues the best. **This simulation has no spells** — nothing in the
    /// tree is a spell type and no building can hold one — so the search has
    /// no candidates and the call never casts. It stays here under its own
    /// name so the make list's dispatch is complete.
    pub fn produce_spell(
        &mut self,
        who: Player,
        t: TypeId,
        city: Option<usize>,
        escrow: i32,
    ) -> bool {
        let _ = (who, t, city, escrow);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::BuildType;
    use crate::tech::{TechTree, TypeDef};
    use crate::world::{Terrain, UNITS_PER_TILE};
    use crate::{Cell, Pos, Tuning, World};

    const TILE: i32 = UNITS_PER_TILE;

    fn tile_pos(tx: i32, ty: i32) -> Pos {
        Pos::new(tx * TILE + TILE / 2, ty * TILE + TILE / 2)
    }

    struct Types {
        village: usize,
        farm: usize,
        barracks: usize,
        library: usize,
        market: usize,
        temple: usize,
        tower: usize,
        granary: usize,
        silo: usize,
        wonder: usize,
    }

    fn bt(ident: Ident, flag: &str, xs: i32, ys: i32) -> BuildType {
        BuildType {
            ident,
            x_size: xs,
            y_size: ys,
            flags: flags::parse(flag),
            job_time: 100,
            hits: 500,
            ..BuildType::default()
        }
    }

    /// A flat land world, two leaders, every type in the tree and available.
    /// The leader's nation is the Maya — nation 0 is the Aztecs, whose ×4 on
    /// a Barracks overflows the listing arithmetic (see
    /// `the_original_s_own_thirty_two_bit_product_wraps`).
    fn sim() -> (Sim, Types) {
        let mut w = World::new(24, 24);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(23, 23));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; RESOURCES];
            l.income = [100 * 16; RESOURCES];
            l.cap = [1000 * 16; RESOURCES];
        }
        let village = sim.add_build_type(bt(Ident::Village, "ean", 7, 7));
        let mut farm = bt(Ident::Farm, "gda", 4, 4);
        farm.flags |= flags::FLAT;
        let farm = sim.add_build_type(farm);
        let barracks = sim.add_build_type(bt(Ident::Barracks, "ean", 4, 4));
        let library = sim.add_build_type(bt(Ident::Library, "jam", 5, 5));
        let market = sim.add_build_type(bt(Ident::Market, "jam", 4, 4));
        let temple = sim.add_build_type(bt(Ident::Temple, "jam", 4, 4));
        let mut tower = bt(Ident::Tower, "ean", 2, 2);
        tower.attack = 12;
        let tower = sim.add_build_type(tower);
        let granary = sim.add_build_type(bt(Ident::Granary, "ja", 5, 5));
        let silo = sim.add_build_type(bt(Ident::MissileSilo, "ean", 4, 4));
        let mut wonder = bt(Ident::Wonder, "ean", 5, 5);
        wonder.wonder = true;
        let wonder = sim.add_build_type(wonder);

        // The tree: six goods in `goodrules.xml` order, then one entry per
        // building record.
        let mut tree = TechTree::new().with_tuning(&Tuning::RON);
        for name in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            let mut d = TypeDef::good(name);
            d.tribe_mask = u32::MAX;
            tree.add(d);
        }
        let recs = [
            village, farm, barracks, library, market, temple, tower, granary, silo, wonder,
        ];
        let names = [
            "Village", "Farm", "Barracks", "Library", "Market", "Temple", "Tower", "Granary",
            "Silo", "Wonder",
        ];
        let ids: Vec<_> = names
            .iter()
            .map(|name| {
                let mut d = TypeDef::building(name);
                d.tribe_mask = u32::MAX;
                tree.add(d)
            })
            .collect();
        sim.set_tech_tree(tree);
        for (rec, id) in recs.iter().zip(ids.iter()) {
            sim.build_types[*rec].tree = Some(*id);
        }
        for t in &mut sim.tech {
            t.tribe = crate::ai::tribe::MAYA;
        }
        (
            sim,
            Types {
                village,
                farm,
                barracks,
                library,
                market,
                temple,
                tower,
                granary,
                silo,
                wonder,
            },
        )
    }

    fn finish(sim: &mut Sim, b: usize) {
        let mut guard = 0;
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
            guard += 1;
            assert!(guard < 10, "a million a frame finishes anything");
        }
    }

    /// Founds a finished city of `who` and gives its AI record a generous
    /// picture: room for anything, four free citizens, terrain everywhere.
    fn city(sim: &mut Sim, t: &Types, who: Player, tx: i32, ty: i32) -> usize {
        // `init_build` rather than `place_building`: the city-count limit is
        // a tech rule and these tests are about the listing, not about it.
        let b = sim.init_build(who, t.village, tile_pos(tx, ty), false);
        finish(sim, b);
        let c = sim.buildings[b].city.expect("a finished city has a record");
        let w = who as usize;
        let regions = sim.world.region_count();
        let players = sim.players.len();
        sim.ai[w].census.resize(regions, players);
        sim.ai[w]
            .city_ai
            .resize(sim.cities.len(), crate::ai::CityAi::default());
        sim.ai[w].city_ai[c] = crate::ai::CityAi {
            free: 4,
            busy: 0,
            gatherers: 0,
            peasant_dist: 1,
            in_port: 0,
            ocean: 0,
            land: 40,
            filled: 1,
            dock_tile: 0,
            space: [8, 8, 8],
            ter: [2; RESOURCES],
            ..Default::default()
        };
        sim.ai[w].census.pop = 10;
        if let Some(r) = sim.cities[c].reg {
            let i = r as usize;
            sim.ai[w].census.reg_free_peasants[i] = 4;
            sim.ai[w].census.reg_cities[i] = 1;
        }
        c
    }

    /// One type's value in one city, straight out of the pipeline — before
    /// `check_income` and `make_me`.
    fn value(sim: &mut Sim, who: Player, c: usize, rec: usize) -> Option<Listing> {
        let f = sim.city_facts(who, c, 0)?;
        let pass = u32::from(sim.build_types[rec].has(flags::GATHER));
        sim.building_value(who, &f, rec, pass)
    }

    fn listed(sim: &Sim, who: Player, rec: usize) -> Option<MakeObject> {
        let id = sim.build_types[rec].tree.expect("in the tree") as i32;
        sim.ai[who as usize]
            .make_list
            .list
            .iter()
            .find(|m| m.t == id)
            .copied()
    }

    // ------------------------------------------------------------------
    // The sweep end to end
    // ------------------------------------------------------------------

    #[test]
    fn the_sweep_lists_what_a_first_city_can_build_and_nothing_else() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        assert_eq!(sim.cities[c].race, Some(0));
        sim.create_buildings(0);
        // The city type itself is never listed: `is_city` is the first gate.
        assert!(listed(&sim, 0, t.village).is_none());
        // A barracks is, in the military category.
        let b = listed(&sim, 0, t.barracks).expect("a barracks is listed");
        assert_eq!(b.cat, 7);
        assert_eq!(b.escrow, 1, "the first trainer is worth saving for");
        // A library is a civic entry in the default category.
        let l = listed(&sim, 0, t.library).expect("a library is listed");
        assert_eq!(l.cat, 8);
        // A farm is a gather entry in the economy category, and carries its
        // city.
        let f = listed(&sim, 0, t.farm).expect("a farm is listed");
        assert_eq!(f.cat, 4);
        assert_eq!(f.city, c as i32);
        assert_eq!(f.num, 1);
        assert_eq!(f.o, -1, "the site is chosen at purchase time");
        // The farm out-values everything here, so it heads the list too.
        assert_eq!(sim.ai[0].make_list.list[0].t, f.t);
    }

    #[test]
    fn nothing_is_listed_for_a_city_still_assimilating() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.cities[c].race = Some(1);
        sim.create_buildings(0);
        assert_eq!(sim.ai[0].make_list, crate::ai::MakeList::new());
    }

    #[test]
    fn the_original_s_own_thirty_two_bit_product_wraps() {
        // `imull %edi, %eax; cltd; and edx,0xff; add; sar $8` at 0x6c3f99:
        // the listing's value is a **signed** 32-bit product. An Aztec's
        // first Barracks is 1000 × 3 × 400 × 100 × 4 = 480,000,000, and
        // 480,000,000 × 0x100 does not fit, so the entry that should be the
        // best in the game comes out negative and loses its own slot.
        let (mut sim, t) = sim();
        for x in &mut sim.tech {
            x.tribe = crate::ai::tribe::AZTECS;
        }
        let c = city(&mut sim, &t, 0, 40, 40);
        let l = value(&mut sim, 0, c, t.barracks).expect("valued");
        assert_eq!(l.val, 480_000_000);
        sim.create_buildings(0);
        assert!(
            listed(&sim, 0, t.barracks).is_none(),
            "the product wraps negative and `make_me` refuses it"
        );
        // Without the nation's ×4 the same entry survives, wrapped but
        // positive: 120,000,000 × 256 mod 2^32 = 655,228,928, over 256.
        for x in &mut sim.tech {
            x.tribe = crate::ai::tribe::MAYA;
        }
        sim.ai[0].make_list = crate::ai::MakeList::new();
        sim.create_buildings(0);
        let b = listed(&sim, 0, t.barracks).expect("listed");
        assert_eq!(b.val, 2_559_488);
    }

    // ------------------------------------------------------------------
    // The gates
    // ------------------------------------------------------------------

    #[test]
    fn a_university_and_a_second_library_need_three_buildings_in_the_city() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        assert_eq!(sim.num_buildings(c), 1);
        assert!(
            value(&mut sim, 0, c, t.library).is_some(),
            "a first library is exempt"
        );
        let b = sim
            .place_building(0, t.library, tile_pos(48, 40))
            .expect("a library places");
        finish(&mut sim, b);
        assert_eq!(sim.num_buildings(c), 2);
        assert!(
            value(&mut sim, 0, c, t.library).is_none(),
            "a second library waits for a third building"
        );
    }

    #[test]
    fn a_temple_is_the_capital_s_alone_on_an_easy_difficulty() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 0;
        sim.cities[c].capital = false;
        assert!(value(&mut sim, 0, c, t.temple).is_none());
        sim.cities[c].capital = true;
        let l = value(&mut sim, 0, c, t.temple).expect("the capital wants a temple");
        assert_eq!(l.escrow, 1, "a temple is always worth saving for");
        // 1000 × 10000 for a first temple, then the open-ground ×4/3.
        assert_eq!(l.val, 1000 * 10000 * 4 / 3);
        // On a hard difficulty it is wanted anywhere.
        sim.cities[c].capital = false;
        sim.lobby.difficulty = 3;
        assert!(value(&mut sim, 0, c, t.temple).is_some());
    }

    #[test]
    fn the_capital_countdown_leaves_only_military_trainers() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.elimination = 1;
        sim.cities[c].capital = false;
        // With no capital of my own and nobody else holding one, the
        // original's `find_capital` still answers "me", so nothing changes.
        assert!(value(&mut sim, 0, c, t.market).is_some());
        // Player 1 holds a city that was mine.
        let c2 = city(&mut sim, &t, 1, 80, 40);
        sim.cities[c2].was_capital |= 1;
        assert!(value(&mut sim, 0, c, t.barracks).is_some());
        assert!(value(&mut sim, 0, c, t.market).is_none());
        assert!(value(&mut sim, 0, c, t.farm).is_none());
        // And a capital of my own puts everything back.
        sim.cities[c].capital = true;
        assert!(value(&mut sim, 0, c, t.market).is_some());
    }

    #[test]
    fn a_one_per_city_type_is_not_offered_twice() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        assert!(sim.build_types[t.market].has(flags::ONE_PER_CITY));
        assert!(value(&mut sim, 0, c, t.market).is_some());
        let b = sim
            .place_building(0, t.market, tile_pos(48, 40))
            .expect("a market places");
        assert_eq!(sim.buildings[b].city, Some(c));
        assert!(!sim.buildings[b].active, "an unfinished site still counts");
        assert!(value(&mut sim, 0, c, t.market).is_none());
    }

    #[test]
    fn at_most_three_of_a_plain_kind() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        // The Silo is not a military trainer, not a gather building, not an
        // enhancer, has no attack and is not the Temple: the leader may hold
        // three of it and no more.
        assert!(value(&mut sim, 0, c, t.silo).is_some());
        for i in 0..2 {
            let b = sim
                .place_building(0, t.silo, tile_pos(48, 32 + i * 6))
                .expect("a silo places");
            finish(&mut sim, b);
        }
        assert_eq!(sim.buildings_of_line(0, t.silo), 2);
        assert!(value(&mut sim, 0, c, t.silo).is_some(), "two is still fine");
        let b = sim
            .place_building(0, t.silo, tile_pos(48, 44))
            .expect("a third silo places");
        finish(&mut sim, b);
        assert_eq!(sim.buildings_of_line(0, t.silo), 3);
        assert!(value(&mut sim, 0, c, t.silo).is_none());
    }

    #[test]
    fn a_farm_stops_at_the_city_s_farm_limit() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        let limit = sim.farm_limit(c);
        assert!(limit > 0);
        for i in 0..limit {
            let b = sim
                .place_building(0, t.farm, tile_pos(48, 30 + i * 5))
                .expect("a farm places");
            finish(&mut sim, b);
        }
        assert_eq!(sim.city_count_exact(c, t.farm, false), limit);
        assert!(value(&mut sim, 0, c, t.farm).is_none());
    }

    // ------------------------------------------------------------------
    // The families
    // ------------------------------------------------------------------

    #[test]
    fn an_enhancer_needs_three_filled_slots_of_its_own_good() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        assert!(
            value(&mut sim, 0, c, t.granary).is_none(),
            "no farms, no granary"
        );
        for i in 0..3 {
            let b = sim
                .place_building(0, t.farm, tile_pos(48 + i * 5, 40))
                .expect("a farm places");
            finish(&mut sim, b);
            sim.buildings[b].gather_max = Some(1);
            sim.buildings[b].gatherers.push(0);
        }
        assert_eq!(sim.cities[c].members.len(), 3);
        let g = value(&mut sim, 0, c, t.granary).expect("three filled slots want a granary");
        assert_eq!(g.cat, 4, "an enhancer is an economy entry");
        // v = 1000 (level) × 4/3 (open ground) × n² with n = 3 filled slots;
        // the divisor for a plain type is one.
        assert_eq!(g.val, 1000 * 4 / 3 * 9);
    }

    #[test]
    fn a_market_takes_the_civic_multiplier_and_the_infrastructure_mod() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        // A sixteenth of the usual infrastructure weight, so the ×10000
        // stays inside a signed thirty-two-bit product.
        sim.ai[0].infra_mod = 0x10;
        let m = value(&mut sim, 0, c, t.market).expect("a market is listed");
        // m = 10 + 1, doubled for the first of its kind, then >>2 for a
        // one-city empire = 5; v = 1000 × 5, then × infra × 10000 / 256, then
        // the open-ground ×4/3.
        assert_eq!(m.val, 1000 * 4 / 3 * 5 * 0x10 * 10000 / 256);
        assert_eq!(m.cat, 8);
    }

    #[test]
    fn a_tower_is_dropped_on_the_easiest_difficulty_and_wanted_on_the_hardest() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 0;
        assert!(value(&mut sim, 0, c, t.tower).is_none());
        sim.lobby.difficulty = 4;
        let tw = value(&mut sim, 0, c, t.tower).expect("a hard AI wants a tower");
        assert_eq!(tw.escrow, 1, "the first tower is worth saving for");
        // A raised defence mod lifts the easy-difficulty drop entirely.
        sim.lobby.difficulty = 0;
        sim.ai[0].defense_mod = 0x200;
        assert!(value(&mut sim, 0, c, t.tower).is_some());
    }

    #[test]
    fn the_gather_multiplier_saturates_at_one() {
        // `cmpl %esi, -0x14(%ebp); cmovll` at 0x6c2762 is a minimum against
        // 0x100, so the farm's value never exceeds its own base however many
        // doublings apply.
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.ai[0].econ = [1; RESOURCES];
        sim.ai[0].worst_good = 0;
        sim.ai[0].best_good = 5;
        let f = value(&mut sim, 0, c, t.farm).expect("a farm is listed");
        assert_eq!(f.mult, 2, "a gather type discounts at half the rate");
        // base = ((pop·7000)/cities + level·3000 + open·4000) × infra/256,
        // ×5 for the first farm and ×10 for a city with no members yet, then
        // the tail's ×4/3 and a divisor of one.
        assert!(sim.cities[c].capital, "the first city is the capital");
        let open = 40 - 1;
        let base = (10 * 7000 + 3000 + open * 4000) * 5 * 10 / 2;
        assert_eq!(f.val, base * 4 / 3);
        // Better terrain raises the multiplier and changes nothing at all:
        // it was already over the cap.
        sim.ai[0].city_ai[c].ter = [5; RESOURCES];
        let g = value(&mut sim, 0, c, t.farm).expect("still listed");
        assert_eq!(g.val, f.val);
    }

    #[test]
    fn a_gather_type_waits_until_its_slots_are_three_quarters_full() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.ai[0].census.gather_slots[0] = 8;
        sim.ai[0].census.filled_gather_slots[0] = 5;
        assert!(value(&mut sim, 0, c, t.farm).is_none(), "5 < 8·3/4");
        sim.ai[0].census.filled_gather_slots[0] = 6;
        assert!(value(&mut sim, 0, c, t.farm).is_some(), "6 == 8·3/4");
    }

    #[test]
    fn a_gather_type_needs_terrain_or_it_is_not_listed() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.ai[0].city_ai[c].ter = [0; RESOURCES];
        assert!(value(&mut sim, 0, c, t.farm).is_none());
    }

    // ------------------------------------------------------------------
    // The wonder branch and its draws
    // ------------------------------------------------------------------

    #[test]
    fn the_wonder_branch_draws_twice_per_city_and_wonder() {
        let (mut sim, t) = sim();
        let _c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 3;
        let before = sim.rng;
        sim.create_buildings(0);
        let mut probe = before;
        let mut n = 0;
        while probe != sim.rng {
            probe.roll();
            n += 1;
            assert!(n < 20, "the wonder branch draws more than it should");
        }
        assert_eq!(n, 2, "one wonder type, one city, one pass");
        assert!(listed(&sim, 0, t.wonder).is_some());
    }

    #[test]
    fn two_cities_draw_twice_each() {
        let (mut sim, t) = sim();
        let _a = city(&mut sim, &t, 0, 40, 40);
        let _b = city(&mut sim, &t, 0, 80, 40);
        sim.lobby.difficulty = 3;
        let before = sim.rng;
        sim.create_buildings(0);
        let mut probe = before;
        let mut n = 0;
        while probe != sim.rng {
            probe.roll();
            n += 1;
            assert!(n < 20, "too many draws");
        }
        assert_eq!(n, 4);
    }

    #[test]
    fn a_wonder_already_standing_in_the_city_blocks_the_next() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 3;
        let b = sim
            .place_building(0, t.wonder, tile_pos(48, 40))
            .expect("a wonder places");
        finish(&mut sim, b);
        assert_eq!(sim.buildings[b].city, Some(c));
        assert_eq!(sim.num_wonders(c, true), 1);
        let before = sim.rng;
        sim.create_buildings(0);
        assert_eq!(sim.rng, before, "the branch is refused before it draws");
        assert!(listed(&sim, 0, t.wonder).is_none());
    }

    #[test]
    fn a_wonder_a_rival_is_a_quarter_through_is_abandoned() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        let _c2 = city(&mut sim, &t, 1, 80, 40);
        sim.lobby.difficulty = 3;
        let b = sim
            .place_building(1, t.wonder, tile_pos(74, 40))
            .expect("the rival's wonder places");
        // A fifth of the way: still worth racing.
        sim.buildings[b].job_counter = sim.buildings[b].constr_time / 5;
        assert!(value(&mut sim, 0, c, t.wonder).is_some());
        // A third of the way: give up.
        sim.buildings[b].job_counter = sim.buildings[b].constr_time / 3;
        assert!(value(&mut sim, 0, c, t.wonder).is_none());
    }

    #[test]
    fn my_own_site_blocks_the_wonder_however_far_along_it_is() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 3;
        let b = sim
            .place_building(0, t.wonder, tile_pos(48, 48))
            .expect("my wonder places");
        sim.buildings[b].job_counter = 0;
        assert!(value(&mut sim, 0, c, t.wonder).is_none());
    }

    #[test]
    fn the_wonder_mod_wants_a_wonder_and_devalues_everything_else() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        sim.lobby.difficulty = 3;
        sim.ai[0].wonder_mod = 0x100;
        let before = sim.rng;
        let wd = value(&mut sim, 0, c, t.wonder).expect("a wonder is valued");
        assert_eq!(sim.rng, before, "the shortcut arm takes no draw");
        assert_eq!(wd.escrow, 1);
        // 100,000,000 × the city's level. The type carries `e`, so the
        // common tail's open-ground clause does not touch it, and the
        // divisor for a first wonder is one.
        assert_eq!(wd.val, 100_000_000);
        // And a library is worth a hundredth of what it was.
        let dulled = value(&mut sim, 0, c, t.library).expect("still valued").val;
        sim.ai[0].wonder_mod = 0;
        let full = value(&mut sim, 0, c, t.library).expect("still valued").val;
        assert!(dulled * 90 < full, "{dulled} vs {full}");
    }

    // ------------------------------------------------------------------
    // The producers
    // ------------------------------------------------------------------

    fn tech_at(sim: &mut Sim, at: usize) -> TypeId {
        let lib_t = sim.build_types[at].tree.expect("in the tree");
        let mut tree = sim.tech_tree.clone();
        let mut d = TypeDef::plain("Writing", 1);
        d.where_ = Some(lib_t);
        d.tribe_mask = u32::MAX;
        let writing = tree.add(d);
        sim.set_tech_tree(tree);
        writing
    }

    fn writing_tech(sim: &mut Sim, t: &Types) -> TypeId {
        tech_at(sim, t.library)
    }

    #[test]
    fn produce_upgrade_queues_a_technology_at_the_best_member() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        let writing = writing_tech(&mut sim, &t);
        let b = sim
            .place_building(0, t.library, tile_pos(48, 40))
            .expect("a library places");
        finish(&mut sim, b);
        assert_eq!(sim.buildings[b].city, Some(c));
        assert!(sim.produce_upgrade(0, writing, Some(c), 0));
        assert_eq!(sim.buildings[b].queue.count_tech(writing), 1);
        // The same call again finds that member already holding it.
        assert!(!sim.produce_upgrade(0, writing, Some(c), 0));
    }

    #[test]
    fn produce_upgrade_refuses_a_city_with_no_such_building() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        let writing = writing_tech(&mut sim, &t);
        assert!(!sim.produce_upgrade(0, writing, Some(c), 0));
        assert!(!sim.produce_upgrade(0, writing, None, 0));
    }

    #[test]
    fn produce_upgrade_skips_a_site_and_takes_the_finished_building() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        // A drill queued at the Barracks, of which a city may hold several.
        let drill = tech_at(&mut sim, t.barracks);
        let writing = drill;
        let site = sim
            .place_building(0, t.barracks, tile_pos(48, 40))
            .expect("a barracks places");
        let done = sim
            .place_building(0, t.barracks, tile_pos(48, 48))
            .expect("a second barracks places");
        finish(&mut sim, done);
        assert!(sim.produce_upgrade(0, writing, Some(c), 0));
        assert_eq!(sim.buildings[site].queue.count_tech(writing), 0);
        assert_eq!(sim.buildings[done].queue.count_tech(writing), 1);
    }

    #[test]
    fn produce_spell_never_casts() {
        let (mut sim, t) = sim();
        let c = city(&mut sim, &t, 0, 40, 40);
        let id = sim.build_types[t.temple].tree.expect("in the tree");
        assert!(!sim.produce_spell(0, id, Some(c), 1));
    }
}
