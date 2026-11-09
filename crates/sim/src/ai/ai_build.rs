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
//! - ~~`CityData.bordering` (+0x65)~~: written by [`Sim::advance_border_pass`]
//!   since item 1600 (`docs/AI.md` §167) and read by the temple and the
//!   tower and fort arms. `city_flags & 0x8` / `& 0x1000`: no known setter
//!   (`create-buildings.md` §9) — read as 0; the `0x1000` arm the border
//!   pass sets beside `bordering` (a cell within four of the city) is not
//!   modelled.
//! - `CityData.ocean_filled` (+0x66): no field — `ocean_open` is
//!   [`crate::ai::CityAi::ocean`].
//! - `LeaderData::city_num` / `village_num`: the census does not keep them —
//!   the leader's live city count, and 0.
//! - `world+0x34`, the landmass count the docks read: 1.
//! - the leader's known oil patches: none, so `oil_ok` is never true.
//! - ~~`GoodTypeData::largest_gather` (+0x2ec): 0.~~ Computed since item
//!   1451 ([`crate::world::largest_gather`]): 1 on the shipped table.
//! - the wonder bookkeeping — team, enemy and unbuilt wonder value,
//!   `Game::wonder_winning`, the wonder-win row's target and a wonder
//!   type's value factor (vslot `+0x118`) — ~~0, −1, absent, 1~~ carried
//!   since §124 ([`Sim::wonder_winning`], `BuildType::wonder_points`).
//!   `wonder_mark` has its writer ([`Sim::note_wonders`], §75).
//! - `TERRACOTTA` / `STATUEOFLIBERTY` / `SPACEPROGRAM` have no [`Ident`], so
//!   the tech-race lobby's ÷1000 never fires.
//! - ~~`WorldData::danger` is per region here, not per half-cell.~~ It is
//!   the half-cell grid now, and [`crate::danger`] writes it.
//! - ~~`build_flags & 0x8000000` is `DEEP_QUEUE` and no shipped row carries
//!   it, so the civic block is dead code.~~ **Wrong, corrected 2026-08-25**
//!   (`docs/DATALAYER.md`, "The derived words no column carries"): the bit is
//!   [`flags::RESEARCH_HERE`] and it is *derived*, not written —
//!   `TechType::set_research` marks every building in the lineage of a tech's
//!   `WHERE`, which is the Granary, Lumber Mill, Smelter, University,
//!   Library, Temple, Senate and the whole Tower and Fort lines. The civic
//!   block runs.
//! - `produce_upgrade` of a *building* type: the production queue holds
//!   units and technologies, so a building upgrade has nowhere to go.
//! - spells: the simulation has none, so `produce_spell` never casts.

use crate::ai::{Census, MakeObject};
use crate::ai_place::{enhancing_good, gather_good, is_enhancer};
use crate::ai_types::Class;
use crate::build::{self, BuildDomain, Ident, flags};
use crate::economy::RESOURCES;
use crate::tech::{self, TypeId};
use crate::{Player, Sim};

/// The wonder arm's two draw sites, under the original's own offsets: the
/// `% 1000` and the `% 300` that scale a wonder's value when no wonder
/// victory is on and nobody is winning one. They are the return addresses
/// of two consecutive `call Random::get` in the listing (`6c2bd6` and
/// `6c2bf2`, five bytes each), and the marks are what let the trace's
/// sequence name them rather than the sweep they run under
/// (`rondata::trace::SITES`).
pub const SITE_WONDER_MOD: &str = "Leader::create_buildings+0xffb";
pub const SITE_WONDER_SCALE: &str = "Leader::create_buildings+0x1017";

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
            cur = self.tables.build_types[i].from;
            guard += 1;
            if guard > self.tables.build_types.len() {
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
            cur = self.tables.build_types[i].to;
            guard += 1;
            if guard > self.tables.build_types.len() {
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

    /// `BuildTypeData::is_training_building@00639de0` — `build_flags &
    /// 0x80000000` read off the **root** of the `FROM` chain (`this+0x3c`,
    /// then vslot `+0xe4`), exactly as `is_military_trainer` is: the
    /// Redoubt's own record has no such bit and its root has (item 1738,
    /// run781's packet: the divisor 7 is `(reg + (queued + 2·have)·2)·2 + 1`).
    fn is_training_building(&self, rec: usize) -> bool {
        build::is_training_building(&self.tables.build_types, rec)
    }

    /// `BuildTypeData::is_military_trainer@0063bcf0`, the derived flag on the
    /// root of the `FROM` chain (`build::init_derived_flags`): every building
    /// that trains a military unit — the Barracks, Stable and Siege Factory
    /// lines, the Dock, the **Airbase** and the Missile Silo. The five idents
    /// this crate listed here before item 1496 were all an Ancient-to-Industrial
    /// lobby ever offered (`docs/AI.md` §129).
    fn is_trainer_type(&self, rec: usize) -> bool {
        build::is_military_trainer(&self.tables.build_types, rec)
    }

    /// `BuildTypeData::is_defensive` — a tower, a fort, an Airbase, or
    /// anything that shoots ([`build::is_defensive`]).
    fn is_defensive_type(&self, rec: usize) -> bool {
        build::is_defensive(&self.tables.build_types, rec)
    }

    /// The tree id of the `g`th good, in `goodrules.xml` order.
    fn good_type(&self, g: usize) -> Option<TypeId> {
        self.tables
            .tech_tree
            .types
            .iter()
            .enumerate()
            .filter(|(_, d)| matches!(d.kind, tech::Kind::Good))
            .map(|(i, _)| i)
            .nth(g)
    }

    /// The tree id of the first build record with this identity.
    fn ident_tree(&self, ident: Ident) -> Option<TypeId> {
        self.tables
            .build_types
            .iter()
            .find(|b| b.ident == ident)
            .and_then(|b| b.tree)
    }

    /// `LeaderData::has_tech(t)` for a named building type.
    fn has_building_tech(&self, who: Player, ident: Ident) -> bool {
        self.ident_tree(ident).is_some_and(|t| {
            self.tables
                .tech_tree
                .has_tech(&self.tables.setup, &self.tech[who as usize], t)
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
                for rec in 0..self.tables.build_types.len() {
                    let Some(l) = self.building_value(who, &f, rec, pass) else {
                        continue;
                    };
                    let Some(t) = self.tables.build_types[rec].tree else {
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
            free: i32::from(ai.free),
            busy: ai.busy,
            gatherers: i32::from(ai.gatherers),
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
        let bt = self.tables.build_types[rec].clone();
        let ident = bt.ident;
        if build::is_city(&self.tables.build_types, rec) {
            return None;
        }
        let gather = bt.has(flags::GATHERER);
        if (pass == 1) != gather {
            return None;
        }
        let tree = bt.tree?;
        if self.type_avail(who, tree) != tech::AVAILABLE {
            return None;
        }
        // The capital countdown: with my capital in someone else's hands,
        // only military trainers are listed.
        if self.lobby.elimination == 1 && self.capital_lost(who) && !self.is_trainer_type(rec) {
            return None;
        }
        let n = (bt.x_size.max(bt.y_size) - 2).clamp(0, 2) as usize;
        let fit = f.space[n];
        let dock = build::is_dock(&self.tables.build_types, rec);
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
            let where_rec = self.tables.tech_tree.types[tree]
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
        if self.lobby.rush_rules == 8 && build::is(&self.tables.build_types, rec, Ident::Lookout) {
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
            && !build::is(&self.tables.build_types, rec, Ident::Temple)
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
                // `CityData.bordering`: a city a rival's border touches
                // wants its temple a hundred times more
                // (`create_buildings:324`–`326`, `docs/AI.md` §167).
                if self.ai[w]
                    .city_ai
                    .get(f.c)
                    .is_some_and(|r| r.bordering != 0)
                {
                    t = t.wrapping_mul(100);
                }
                // `city_flags & 0x1000`: ×2, or ×10 while the city is not
                // `no_heal` (`create_buildings:328`).
                if self.ai[w].city_ai.get(f.c).is_some_and(|r| r.near_border) {
                    t = t.wrapping_mul(if self.cities[f.c].no_heal { 2 } else { 10 });
                }
                v = t;
            }
            if build::is(&self.tables.build_types, rec, Ident::Lookout) {
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
            let g = enhancing_good(ident)?;
            let mut nfilled = f.slots[g] - f.open_slots[g];
            if g == 5 {
                nfilled = self.ledgers[w].gather_slots[5] * 2;
            }
            let avail = self
                .good_type(g)
                .is_some_and(|t| self.type_available(who, t));
            let cap = self.mod_resource_cap(who, g);
            let lakota =
                self.tables
                    .tech_tree
                    .has_tribe_bonus(&self.tables.setup, &self.tech[w], 0x13);
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
        let deep = bt.has(flags::RESEARCH_HERE);
        if !deep && !dock && self.is_trainer_type(rec) {
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
                let bs = build::is(&self.tables.build_types, rec, Ident::Barracks)
                    || build::is(&self.tables.build_types, rec, Ident::Stable)
                    || (build::is(&self.tables.build_types, rec, Ident::SiegeFactory)
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
            let quad = if build::is(&self.tables.build_types, rec, Ident::Barracks) {
                tribe == 0 || tribe == 6
            } else if build::is(&self.tables.build_types, rec, Ident::Stable) {
                tribe == 0x11
            } else if build::is(&self.tables.build_types, rec, Ident::SiegeFactory) {
                tribe == 8
            } else if build::is(&self.tables.build_types, rec, Ident::Airbase) {
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
            let is_market = build::is(&self.tables.build_types, rec, Ident::Market);
            let is_temple = build::is(&self.tables.build_types, rec, Ident::Temple);
            let is_library = build::is(&self.tables.build_types, rec, Ident::Library);
            let is_senate = build::is(&self.tables.build_types, rec, Ident::Senate);
            let civic = (deep
                && !build::is(&self.tables.build_types, rec, Ident::University)
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
            let tower = build::is(&self.tables.build_types, rec, Ident::Tower);
            let fort = build::is_fort(&self.tables.build_types, rec);
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
                // `CityData.bordering` (`create_buildings:821`–`841`): a
                // city a rival's border touches is worth ten times, and
                // one it does not touch a hundredth for a fort, a tower
                // unchanged (`docs/AI.md` §167).
                if self.ai[w]
                    .city_ai
                    .get(f.c)
                    .is_some_and(|r| r.bordering != 0)
                {
                    x = x.wrapping_mul(10);
                } else if fort {
                    x /= 100;
                }
                // `city_flags & 0x1000`: ×2, or ×10 while not `no_heal`
                // (`create_buildings:847`). `& 0x8` still reads 0, so the
                // halving always applies.
                if self.ai[w].city_ai.get(f.c).is_some_and(|r| r.near_border) {
                    x = x.wrapping_mul(if self.cities[f.c].no_heal { 2 } else { 10 });
                }
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
            if build::is(&self.tables.build_types, rec, Ident::Airbase) {
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
            && (!self.is_trainer_type(rec) || self.city_count_line(f.c, rec, true) != 0)
        {
            v /= 100;
        }

        // ---- §3.10 the divisor and the head check ----
        if !gather {
            let queued = self.queued_of(who, rec);
            let reg = self.reg_buildings_of_line(who, f.reg, rec);
            let d = if dock {
                (queued + 2 * have) * 4 + 1 + reg
            } else if self.is_trainer_type(rec) {
                (have + queued) * 4 + 1 + reg
            } else if self.is_training_building(rec) {
                (reg + (queued + 2 * have) * 2) * 2 + 1
            } else if build::is(&self.tables.build_types, rec, Ident::Tower) {
                (queued + have) * 4 + 1
            } else if build::is_fort(&self.tables.build_types, rec) {
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
                        .is_some_and(|r| self.tables.build_types[r].has(flags::GATHERER));
                if !head_gather {
                    let mine = gather_good(ident);
                    for g in 0..RESOURCES {
                        let avail = self
                            .good_type(g)
                            .is_some_and(|t| self.type_available(who, t));
                        let cost = self.type_price(who, ht).map_or(0, |c| c[g]);
                        if mine == Some(g) && avail && self.ledgers[w].bucket[g] < cost {
                            v = v.wrapping_mul(2).min(head.val);
                            break;
                        }
                    }
                }
            }
            // **Every exit of the head test escrows** (item 890, listing
            // `006c3cbb`..`006c3db0`): the `jge`, the `js` and the
            // `can_pay` refusal jump to `006c3da8`, the flag test and the
            // goods walk land there or at `006c3d8a`, and each writes 1 to
            // the offer's escrow (`-0xc(%ebp)`, pushed to `make_me` at
            // `006c3fa8`). A gather offer is always escrowed: East Indies'
            // Farm on 23181 was the head, the test failed on its first
            // compare, and the original listed it at `escrow 1`.
            escrow = 1;
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
        if t >= self.tables.tech_tree.types.len() {
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
        let ident = self.tables.build_types[rec].ident;
        let good = gather_good(ident)?;
        let cen = &self.ai[w].census;
        if cen.filled_gather_slots[good] < self.ledgers[w].gather_slots[good] * 3 / 4 {
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
                && f.nb < self.tables.tuning.city_buildings + 1
            {
                base = base.wrapping_mul(2);
            }
            if f.level == 2
                && self.has_building_tech(who, Ident::Metropolis)
                && f.nb < self.tables.tuning.metro_buildings + 1
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
        let free_peasants = Census::reg_u16(&cen.reg_free_peasants, f.reg);
        if free_peasants == 0 && self.muster[w].cap - self.muster[w].control <= 1 {
            return None;
        }
        let worst = self.ai[w].worst_good;
        let best = self.ai[w].best_good;
        if free_peasants > 2 && self.ai[w].rate[worst] < self.mod_resource_cap(who, worst) / 32 {
            escrow = 1;
        }
        for g in 0..RESOURCES {
            let avail = self
                .good_type(g)
                .is_some_and(|t| self.type_available(who, t));
            if !(good == g
                && avail
                && self.ledgers[w].income[g] < self.mod_resource_cap(who, g)
                && f.open_slots[g] < 2)
            {
                continue;
            }
            // `oil_patches.count` (`+0x6e8c`): item 1505, [`Leader::oil_patches`].
            let mut oil_ok = false;
            let mut m0 = 0x100;
            if g == 5 {
                oil_ok = f.ci == 0
                    && self.ledgers[w].gather_slots[5] < self.ai[w].oil_patches.len() as i32;
                base = base.wrapping_mul(10);
            }
            let ter = f.ter[g];
            // `max(3 − GoodTypeData::largest_gather, ter)`, in both arms of
            // `Leader::create_buildings@006c1be0`'s gather multiplier, the
            // worst good's and the rest: `largest_gather` is never 0
            // ([`crate::world::largest_gather`]), so on the shipped land
            // table this is `max(2, ter)`. It was read as 0 — `max(3, ter)`
            // — until item 1451, and run612's farm offer on 8184 (45156,
            // ours 52593) is what the third reading missed.
            let floor = (3 - crate::world::largest_gather(g)).max(ter);
            let mut m;
            if g == worst {
                if !(ter != 0 || oil_ok) {
                    continue;
                }
                m = if g == 5 { 0x800 } else { (floor + 2) * 0x100 };
            } else {
                if !(ter != 0 || oil_ok) {
                    continue;
                }
                if (g == 0 || g == 1 || g == 2) && self.city_num(who) + self.village_num(who) < 5 {
                    m0 = 0x200;
                }
                let n = (self.tables.build_types[rec]
                    .x_size
                    .max(self.tables.build_types[rec].y_size)
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
                m = (k + floor).wrapping_mul(m0) / k;
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
        for p in 0..self.territory.players.len() {
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
            // `LeaderData::wonder_mark` (+0x424): one past the leader's
            // highest wonder entry, so an easy AI outside a wonder victory
            // wants no wonder once one of its own has activated (§75).
            let wonder_mark = self.ai[w].census.wonder_mark;
            if diff < 2 {
                let ok = if self.lobby.victory == 6 {
                    self.team_wonder_points(who) < self.enemy_wonder_points(who) + 1
                } else {
                    wonder_mark == 0
                };
                if !ok {
                    return None;
                }
            }
            // `LAB_006c2b6d`: who is winning, and the enemies' wonders,
            // standing and unbuilt (`local_20`), §124.
            let winning = self.wonder_winning();
            let ev = self
                .enemy_wonder_points(who)
                .wrapping_add(self.enemy_unbuilt_wonder_points(who));
            let mut val = if self.lobby.victory == 6 {
                *escrow = 1;
                (ev + 1).wrapping_mul(100_000)
            } else if (winning >= 0 && winning != who as i32)
                || self.lobby.wonder_win_points / 2 <= ev
                || ev > 4
            {
                *escrow = 1;
                2_000_000
            } else {
                self.mark(SITE_WONDER_MOD);
                let r1 = self.rng.roll();
                self.mark(SITE_WONDER_SCALE);
                let r2 = self.rng.roll();
                (self.team_wonder_points(who) + 1)
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
        // `type.vslot(+0x118)` (`6c2c76`) is `get_wonder_value`, the
        // type's own `WONDER_VAL` (§124): Tikal's 2 doubles it.
        let points = self.tables.build_types[rec].wonder_points();
        Some(
            self.city_level_of(f.c)
                .wrapping_mul(points)
                .wrapping_mul(val),
        )
    }

    /// `LeaderData::get_wonder_value@006ebb90`: the points of every wonder
    /// in `who`'s list under its `wonder_mark` — each one's type's
    /// [`BuildType::wonder_points`], through `ObjectData::get_wonder_value`
    /// (`Build`'s slot `+0x14c` is that function, read off the PE).
    ///
    /// [`BuildType::wonder_points`]: crate::build::BuildType::wonder_points
    pub fn wonder_points(&self, who: Player) -> i32 {
        let Some(a) = self.ai.get(who as usize) else {
            return 0;
        };
        let mark = usize::try_from(a.census.wonder_mark).unwrap_or(0);
        a.census
            .wonder_slots
            .iter()
            .take(mark)
            .flatten()
            .filter_map(|&b| self.buildings.get(b)?.ty)
            .map(|t| self.tables.build_types[t].wonder_points())
            .sum()
    }

    /// `LeaderData::get_unbuilt_wonder_value@006d5ee0`: the points of
    /// `who`'s unbuilt wonder sites — the list `add_unbuilt_wonder` keeps,
    /// read here as the leader's alive, inactive wonder buildings, the
    /// same reading `get_unbuilt_wonders` takes for the price.
    pub fn unbuilt_wonder_points(&self, who: Player) -> i32 {
        self.buildings
            .iter()
            .filter(|b| b.alive && !b.active && b.owner == who)
            .filter_map(|b| b.ty)
            .map(|t| self.tables.build_types[t].wonder_points())
            .sum()
    }

    /// The leaders the three sums walk: the slots whose `flags & 1` is set,
    /// read as every player not defeated.
    fn wonder_leaders(&self) -> impl Iterator<Item = Player> + '_ {
        (0..self.territory.players.len())
            .filter(|&p| !self.defeated.get(p).copied().unwrap_or(true))
            .map(|p| p as Player)
    }

    /// `LeaderData::get_team_wonder_value@006da990`: `who`'s points and
    /// every mutual ally's.
    pub fn team_wonder_points(&self, who: Player) -> i32 {
        self.wonder_leaders()
            .filter(|&p| self.is_ally(who, p))
            .map(|p| self.wonder_points(p))
            .sum()
    }

    /// `LeaderData::get_enemy_wonder_value@006da8f0`: every other leader
    /// that is not a mutual ally.
    pub fn enemy_wonder_points(&self, who: Player) -> i32 {
        self.wonder_leaders()
            .filter(|&p| !self.is_ally(who, p))
            .map(|p| self.wonder_points(p))
            .sum()
    }

    /// `LeaderData::get_enemy_unbuilt_wonder_value@006d5e40`: the same
    /// leaders' unbuilt sites.
    pub fn enemy_unbuilt_wonder_points(&self, who: Player) -> i32 {
        self.wonder_leaders()
            .filter(|&p| !self.is_ally(who, p))
            .map(|p| self.unbuilt_wonder_points(p))
            .sum()
    }

    /// `LeaderData::get_wonder_net@006ebb10`: the team's points less the
    /// best enemy team's, floored at 0.
    pub fn wonder_net(&self, who: Player) -> i32 {
        let mine = self.team_wonder_points(who);
        let best = self
            .wonder_leaders()
            .filter(|&p| !self.is_ally(who, p))
            .map(|p| self.team_wonder_points(p))
            .fold(0, i32::max);
        (mine - best).max(0)
    }

    /// `Game::wonder_winning@005948a0`: the leader whose net reaches the
    /// wonder victory's points, or −1. A tie with the holder goes to an
    /// ally with more points of its own; a tie with a non-ally clears the
    /// holder and hands the lead to the later leader if it reaches the
    /// points (`0059490e`).
    pub fn wonder_winning(&self) -> i32 {
        wonder_lead(
            self.lobby.wonder_win_points,
            self.wonder_leaders(),
            |p| self.wonder_net(p),
            |p, q| self.is_ally(p, q),
            |p| self.wonder_points(p),
        )
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
        let Some(where_t) = self.tables.tech_tree.types[t].where_ else {
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
            let danger = self.world.danger_at(who, bd.pos).max(0);
            score /= danger + 1;
            if best.is_none_or(|(v, _)| v < score) {
                best = Some((score, b));
            }
        }
        let Some((_, b)) = best else { return false };
        match self.type_class(t) {
            Class::Tech => self.queue_tech_with(b, t, escrow != 0).is_ok(),
            Class::Unit => self
                .unit_record(t)
                .is_some_and(|r| self.queue_up_with(b, r, escrow != 0).is_ok()),
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

/// `Game::wonder_winning@005948a0`'s walk over the active leaders in slot
/// order: `net` asked of each, and on a tie with a holder `ally(p, holder)`,
/// then `points(holder)` and `points(p)` in that order — the calls the
/// original makes, in its order, which item 2247's sweep answers a call at a
/// time. A leader takes the lead when its net reaches `target`.
pub(crate) fn wonder_lead(
    target: i32,
    leaders: impl IntoIterator<Item = Player>,
    mut net: impl FnMut(Player) -> i32,
    mut ally: impl FnMut(Player, Player) -> bool,
    mut points: impl FnMut(Player) -> i32,
) -> i32 {
    let (mut best, mut win) = (0, -1i32);
    for p in leaders {
        let n = net(p);
        if n < best {
            continue;
        }
        if n == best {
            if win < 0 {
                continue;
            }
            if ally(p, win as Player) {
                let held = points(win as Player);
                if held < points(p) {
                    win = i32::from(p);
                }
                continue;
            }
            win = -1;
        }
        if target <= n {
            best = n;
            win = i32::from(p);
        }
    }
    win
}

#[cfg(test)]
mod tests;
