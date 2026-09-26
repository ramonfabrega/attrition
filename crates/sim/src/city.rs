//! Cities and the building lifecycle — `docs/CITIES.md` §3–§5, §7–§9.
//!
//! A city is a building with a record behind it: who owns it, whose nation it
//! is assimilated to, which buildings belong to it, and the stamps that gate
//! capture and assimilation. This file holds the record, the lifecycle every
//! typed building goes through (placed → started → active → closed), the
//! level-up, the capture test and hand-over, the assimilation tick, the city
//! heal, plunder, and what losing the last city does. The arithmetic it leans
//! on is `build.rs`'s; the placement predicates are `place.rs`'s; the
//! garrison `garrison.rs`'s.

use crate::build::{self, Ident, flags};
use crate::combat::Obj;
use crate::cost;
use crate::economy::RESOURCES;
use crate::place::Blocked;
use crate::territory;
use crate::world::{Owner, Pos, UNITS_PER_TILE, tile, vector_dist};
use crate::{Building, Player, Sim};

/// The nation, wonder and tech inputs this mechanic reads per player — the
/// `has_tribe_bonus`, `has_wonder` and `has_preq` answers, as inputs until
/// the layers that produce them exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nation {
    /// `leader_flags & 4`: a human player. The 75-frame lost-city grace on
    /// city placement applies to humans only.
    pub human: bool,
    pub lakota: bool,
    pub dutch: bool,
    pub egyptians: bool,
    pub indians: bool,
    pub bantu: bool,
    pub turks: bool,
    pub chinese: bool,
    pub koreans: bool,
    pub russians: bool,
    pub aztecs: bool,
    pub germans: bool,
    pub maya: bool,
    pub romans: bool,
    pub british: bool,
    pub nubians: bool,
    pub french: bool,
    pub inca: bool,
    /// The Pyramids, Versailles, the Taj Mahal, the Red Fort, Tikal.
    pub pyramids: bool,
    pub versailles: bool,
    pub taj_mahal: bool,
    pub red_fort: bool,
    pub tikal: bool,
    /// `BUILDINGS_CREATED_FASTER`; `GLOBAL_GOVERNMENT_BONUS`.
    /// `COLONIZE_BONUS` was here too until 2026-09-02 and is not a
    /// nation's at all — it is a technology's prerequisite, and lives in
    /// `Roles::colonize_preq`. **The Tobacco rare left on 2026-09-07**
    /// (item 261): it is not a nation flag at all, nothing ever wrote this
    /// field, and the clock reads [`crate::Sim::has_rare`] now.
    pub created_faster: bool,
    pub global_government: bool,
    /// `get_building_speed_upgrade`, `get_building_hp_upgrade`, 0..3.
    pub speed_upgrade: i32,
    pub hp_upgrade: i32,
    /// `get_heal_level`, 0..3.
    pub heal_level: i32,
    /// The temple border tech level 1..4 for a city with a temple; 0 none.
    pub temple_level: i32,
    /// `FORTGARRISON2..4`: 1..4.
    pub fort_garrison_level: i32,
    /// `disable_building_attrition`.
    pub disable_building_attrition: bool,
    /// Whether the player has a Despot or Spitamenes patriot standing by the
    /// city — the hero cut on plunder.
    pub plunder_hero: bool,
}

impl Default for Nation {
    fn default() -> Nation {
        Nation {
            human: true,
            lakota: false,
            dutch: false,
            egyptians: false,
            indians: false,
            bantu: false,
            turks: false,
            chinese: false,
            koreans: false,
            russians: false,
            aztecs: false,
            germans: false,
            maya: false,
            romans: false,
            british: false,
            nubians: false,
            french: false,
            inca: false,
            pyramids: false,
            versailles: false,
            taj_mahal: false,
            red_fort: false,
            tikal: false,
            created_faster: false,
            global_government: false,
            speed_upgrade: 0,
            hp_upgrade: 0,
            heal_level: 0,
            temple_level: 0,
            fort_garrison_level: 1,
            disable_building_attrition: false,
            plunder_hero: false,
        }
    }
}

/// `LeaderData::cities_built`, `cities_captured`, `cities_lost`, and the
/// frame the capital was last lost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub built: i32,
    pub captured: i32,
    pub lost: i32,
    pub lost_capital_frame: Option<i64>,
    /// `leader_flags & 0x400000`: the first capital loss has been plundered.
    pub capital_plundered: bool,
}

/// A city record — `CityData`, as far as this mechanic reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct City {
    /// `city_flags & 1`.
    pub alive: bool,
    pub owner: Player,
    /// `race`: the nation the city is assimilated to. `None` is the −1 of a
    /// record that has not been founded; `!= Some(owner)` is unassimilated.
    pub race: Option<Player>,
    pub founder: Player,
    /// The city building — the head of the member chain.
    pub building: usize,
    /// The other members, in the order they joined.
    pub members: Vec<usize>,
    pub reg: Option<u16>,
    pub pos: Pos,
    /// `city_flags & 0x10`, `0x4000`, `0x8000`, `0x100`, `0x2`, `0x40`.
    pub capital: bool,
    pub founding_capital: bool,
    pub was_founding_capital: bool,
    pub unassimilated: bool,
    pub no_heal: bool,
    pub alarm: bool,
    /// `city_flags & 0x2000`: the army's "no muster spot here" mark —
    /// set when `find_muster_spot`'s ring search finds no cell at an
    /// active, untroubled city of the owner's own, cleared when it does
    /// (`docs/ARMY.md` §13); `find_target` divides the city's score by 20
    /// while it stands (§12).
    pub no_muster: bool,
    /// `was_capital_flags`, a bit per player.
    pub was_capital: u64,
    /// `capture_stamp`, `assimilation_timer`, `attack_stamp` — frames; zero
    /// on founding, the current frame on a transfer.
    pub capture_stamp: i64,
    pub assimilation_timer: i64,
    pub attack_stamp: i64,
    pub capture_strength: i32,
    /// `CityData::pop` (+0x5d): 1 at init, copied on capture, not the pop
    /// value (`get_pop_value` is computed from the level) and not touched by
    /// a level-up.
    pub pop: i32,
    /// The Citizen patriot is in this city.
    pub has_citizen: bool,
    /// This city's entry in [`Sim::sources`], while it projects territory.
    pub source: Option<usize>,
    /// `trade_val` (+0x52): what the trade routes ending here add to the
    /// owner's **wealth** rate, in sixteenths.
    /// `City::compute_trade@00739640` is its only writer
    /// (`docs/CARAVAN.md` §7.2).
    pub trade_val: i32,
    /// `traded_with[8]` (+0x2c): a bit per partner city, per leader — the
    /// mark that stops `City::new_caravan`'s one-off being paid twice
    /// (`docs/CARAVAN.md` §7.3).
    pub traded_with: [u32; 8],
}

/// Why a placement order was refused — `Group::action_build`'s three gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceFail {
    Blocked(Blocked),
    CityLimit,
    Cost,
}

/// What a unit counts for in a capture — `UnitData::get_capture_value`: a
/// decoy or siege type counts nothing; anything else at least one.
pub const fn capture_value(siege: bool, squad: i32, figures: i32) -> i32 {
    if siege {
        return 0;
    }
    let v = if squad < figures { squad } else { figures };
    if v < 1 { 1 } else { v }
}

/// `ObjectData::health_level`: 0 at ≥ 90 %, 1 ≥ 75 %, 2 ≥ 50 %, 3 ≥ 25 %,
/// 4 ≥ 10 %, 5 ≥ 1 hit, 6 none left.
pub const fn health_level(hits: i32, damage: i32) -> i32 {
    let mut left = hits - damage;
    if left < 0 {
        left = 0;
    }
    if left > hits {
        left = hits;
    }
    if left >= hits * 90 / 100 {
        0
    } else if left >= hits * 75 / 100 {
        1
    } else if left >= hits * 50 / 100 {
        2
    } else if left >= hits * 25 / 100 {
        3
    } else if left >= hits * 10 / 100 {
        4
    } else if left >= 1 {
        5
    } else {
        6
    }
}

/// `LeaderData::get_radius` without the sim: the level's radius in tiles.
pub const fn radius_for_level(t: &crate::Tuning, level: i32, indians: bool) -> i32 {
    let mut r = t.city_center_radius + (level - 1) * t.city_center_pop_radius;
    if indians {
        r += t.indians_city_radius;
    }
    if r > 64 { 64 } else { r }
}

/// `CityData::get_pop_value`: 1, 3, 5 by level.
pub const fn pop_value(level: i32) -> i32 {
    match level {
        2 => 3,
        3 => 5,
        _ => 1,
    }
}

/// How close a builder must stand: its tile within one tile of the
/// footprint (on it counts too, since movement here does not path around
/// buildings) — `docs/CITIES.md` §11.
impl Sim {
    // ------------------------------------------------------------------
    // Small accessors
    // ------------------------------------------------------------------

    /// The ident of a building's type, `Other` for the bare object.
    pub fn building_ident(&self, b: usize) -> Ident {
        self.buildings[b]
            .ty
            .map_or(Ident::Other, |t| self.build_types[t].ident)
    }

    pub(crate) fn building_is(&self, b: usize, ident: Ident) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|t| build::is(&self.build_types, t, ident))
    }

    /// Whether a building is a city — `flags & 0x20`.
    pub fn building_is_city(&self, b: usize) -> bool {
        self.building_is(b, Ident::Village)
    }

    /// `LeaderData::get_radius(who, type)`: in tiles, capped at 64.
    pub fn city_radius(&self, who: Player, ty: Option<usize>) -> i32 {
        let level = ty.map_or(1, |t| build::city_level(&self.build_types, t).max(1));
        radius_for_level(&self.tuning, level, self.nation[who as usize].indians)
    }

    /// `CityData::get_level`.
    pub fn city_level_of(&self, c: usize) -> i32 {
        self.buildings[self.cities[c].building]
            .ty
            .map_or(1, |t| build::city_level(&self.build_types, t).max(1))
    }

    /// The city's radius, `CityData::get_radius`.
    pub fn radius_of(&self, c: usize) -> i32 {
        let city = &self.cities[c];
        self.city_radius(city.owner, self.buildings[city.building].ty)
    }

    /// `CityData::count_buildings(t, exact = 0, active_only)`: members,
    /// including the city building, whose type `is(t, 0)`.
    pub fn count_buildings(&self, c: usize, ident: Ident, active_only: bool) -> i32 {
        let city = &self.cities[c];
        std::iter::once(city.building)
            .chain(city.members.iter().copied())
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive && (!active_only || bd.active) && self.building_is(b, ident)
            })
            .count() as i32
    }

    /// `CityData::num_buildings`: alive, active members — the city building
    /// counts itself.
    pub fn num_buildings(&self, c: usize) -> i32 {
        let city = &self.cities[c];
        std::iter::once(city.building)
            .chain(city.members.iter().copied())
            .filter(|&b| self.buildings[b].alive && self.buildings[b].active)
            .count() as i32
    }

    /// `CityData::num_wonders@007382b0(exclude_city)`: wonders on the chain,
    /// **finished or not**, never the Red Fort, and not the city building
    /// itself when asked. The test is `BuildData::is_wonder` (Build vslot
    /// `+0x2c`), a type range, and nothing reads `is_active`: a wonder site
    /// counts from the frame it joins the city (`docs/AI.md` §70). That is
    /// why `Wall::do_construct`'s bound is `<= 1 + egyptians` — the site
    /// counts itself — and `blocked_location`'s is `> egyptians`.
    pub fn num_wonders(&self, c: usize, exclude_city: bool) -> i32 {
        let city = &self.cities[c];
        std::iter::once(city.building)
            .chain(city.members.iter().copied())
            .filter(|&b| {
                let bd = &self.buildings[b];
                bd.alive
                    && bd.ty.is_some_and(|t| self.build_types[t].wonder)
                    && !self.building_is(b, Ident::RedFort)
                    && !(exclude_city && self.building_is_city(b))
            })
            .count() as i32
    }

    /// `CityData::get_farm_limit`.
    pub fn farm_limit(&self, c: usize) -> i32 {
        let n = &self.nation[self.cities[c].owner as usize];
        let base = if n.egyptians {
            self.tuning.egyptian_farms_per_city_base
        } else {
            self.tuning.farms_per_city_base
        };
        base + (self.city_level_of(c) - 1) * self.tuning.farms_per_city_level
    }

    /// Whether the city holds an active building of a kind — the
    /// `city_flags` bits `0x80`/`0x200`/`0x400`/`0x800`, derived.
    pub fn city_has(&self, c: usize, ident: Ident) -> bool {
        self.count_buildings(c, ident, true) > 0
    }

    /// `BuildData::is_unassimilated`: in a city whose race is not the owner,
    /// and either the city itself or a type that needs a city.
    pub fn building_unassimilated(&self, b: usize) -> bool {
        let bd = &self.buildings[b];
        let Some(c) = bd.city else {
            return false;
        };
        let city = &self.cities[c];
        city.alive
            && city.race != Some(bd.owner)
            && (self.building_is_city(b)
                || bd
                    .ty
                    .is_some_and(|t| !self.build_types[t].has(flags::NO_CITY)))
    }

    /// `CityData::is_unassimilated`.
    pub fn city_unassimilated(&self, c: usize) -> bool {
        let city = &self.cities[c];
        city.race != Some(city.owner)
    }

    // ------------------------------------------------------------------
    // The record
    // ------------------------------------------------------------------

    /// `Cities::init_city` → `City::init`: a record for a city building.
    fn init_city(&mut self, who: Player, b: usize, transfer: bool, capital: bool) -> usize {
        let pos = self.buildings[b].pos;
        let stamp = if transfer { self.frame } else { 0 };
        let reg = self.world.region_of(pos.cell());
        let city = City {
            alive: true,
            owner: who,
            race: None,
            founder: who,
            building: b,
            members: Vec::new(),
            reg,
            pos,
            capital,
            founding_capital: capital,
            was_founding_capital: false,
            unassimilated: false,
            no_heal: false,
            alarm: false,
            no_muster: false,
            was_capital: 0,
            capture_stamp: stamp,
            assimilation_timer: stamp,
            attack_stamp: stamp,
            capture_strength: 0,
            pop: 0,
            has_citizen: false,
            source: None,
            trade_val: 0,
            traded_with: [0; 8],
        };
        let c = match self.cities.iter().position(|c| !c.alive) {
            Some(i) => {
                self.cities[i] = city;
                i
            }
            None => {
                self.cities.push(city);
                self.cities.len() - 1
            }
        };
        self.cities[c].pop = 1;
        self.buildings[b].city = Some(c);
        // `LeaderData::home_reg`: the capital's region, which the AI's
        // sweep reads (`docs/AI.md` §2.3 step 15) and nothing else wrote.
        if capital && let Some(r) = reg {
            self.ai[who as usize].census.home_reg = i32::from(r);
        }
        self.fix_world_vals(c);
        c
    }

    /// `City::fix_world_vals@00735aa0`, the last thing `City::init` does,
    /// on every city it records — a founding and a capture alike
    /// (`docs/AI.md` §67). The site value byte `WData.val` of every cell
    /// in the circle round the centre is **quartered** out to ring `k`
    /// and **halved** on the three rings beyond it, where `k` is the
    /// city's radius in tiles, capped at 64, plus three, over four (the
    /// circle's `radius[r]` counts rings `0..=r`) — so a spiral
    /// or a site score read after a city stands sees the map maker's value
    /// worn down round it, and a second city on the same ground wears it
    /// down again. `City::close` gives nothing back.
    fn fix_world_vals(&mut self, c: usize) {
        let centre = self.cities[c].pos.cell();
        let r = self.radius_of(c).min(0x40);
        let k = ((r + 3) / 4).min(0x3d) as usize;
        let circle = crate::ai_place::circle();
        for i in 0..circle.radius[k + 3] {
            let cell = crate::world::Cell::new(centre.x + circle.x[i], centre.y + circle.y[i]);
            if !self.world.contains(cell) {
                continue;
            }
            let mut d = self.world.cell_data(cell);
            d.val >>= if i < circle.radius[k] { 2 } else { 1 };
            self.world.set_cell_data(cell, d);
        }
    }

    /// `City::close`: the record dies; members re-home.
    fn close_city(&mut self, c: usize, captor: Option<Player>) {
        let owner = self.cities[c].owner;
        let building = self.cities[c].building;
        self.armies_city_closed(c, building);
        let was_capital = self.cities[c].capital;
        self.cities[c].alive = false;
        self.lost_city_stamp[owner as usize] = Some(self.frame);
        self.remove_source_of_city(c);
        let members = std::mem::take(&mut self.cities[c].members);
        for m in members {
            self.buildings[m].city = None;
            if self.buildings[m].alive && !self.building_is_city(m) {
                self.find_city(m);
            }
        }
        self.lost_a_city(owner, was_capital, captor);
        self.sync_pop_cities();
        self.sync_territory();
    }

    /// `Leader::lost_a_city` in the default elimination mode: a player with no
    /// city left is defeated.
    fn lost_a_city(&mut self, who: Player, _was_capital: bool, captor: Option<Player>) {
        if self.city_num(who) == 0 {
            self.defeat(who, captor);
        }
    }

    /// `Leader::defeat` → `defeat_by`: the loser's cities go to the captor
    /// (or to their living allied founder), their military trainers inside
    /// the captor's cities too, everything else dies; queues and orders are
    /// cleared.
    pub fn defeat(&mut self, who: Player, by: Option<Player>) {
        if self.defeated[who as usize] {
            return;
        }
        self.defeated[who as usize] = true;
        self.armies_leader_defeated(who);
        if let Some(by) = by {
            for c in 0..self.cities.len() {
                if !self.cities[c].alive || self.cities[c].owner != who {
                    continue;
                }
                let founder = self.cities[c].founder;
                let race = self.cities[c].race;
                let taker = if founder != who
                    && founder != by
                    && self.is_ally(by, founder)
                    && !self.defeated[founder as usize]
                {
                    founder
                } else if let Some(r) = race
                    && r != who
                    && r != by
                    && self.is_ally(by, r)
                    && !self.defeated[r as usize]
                {
                    r
                } else {
                    by
                };
                self.capture_city(taker, c, who);
            }
        }
        for b in 0..self.buildings.len() {
            if !self.buildings[b].alive || self.buildings[b].owner != who {
                continue;
            }
            if !self.buildings[b].active {
                self.die_building(b);
                continue;
            }
            let trainer = self.buildings[b].ty.is_some_and(|t| {
                matches!(
                    self.build_types[t].ident,
                    Ident::Barracks
                        | Ident::Stable
                        | Ident::AutoPlant
                        | Ident::SiegeFactory
                        | Ident::Factory
                        | Ident::Dock
                        | Ident::Airbase
                )
            });
            if let Some(by) = by
                && trainer
                && self.find_city_at(by, self.buildings[b].pos, None).is_some()
                && let Some(n) = self.swap_team(b, by)
            {
                self.activate(n, false, false);
                self.close_building(b, true);
                continue;
            }
            self.die_building(b);
        }
        for u in 0..self.units.len() {
            if self.units[u].owner == who {
                self.clear_orders(u);
            }
        }
    }

    // ------------------------------------------------------------------
    // Membership
    // ------------------------------------------------------------------

    /// `Build::find_city`: a non-city building joins its owner's nearest
    /// covering city, through `get_town`.
    pub fn find_city(&mut self, b: usize) -> Option<usize> {
        if self.building_is_city(b) {
            return self.buildings[b].city;
        }
        if self.buildings[b].city.is_some() {
            self.remove_from_city(b);
        }
        let (who, pos, ty) = {
            let bd = &self.buildings[b];
            (bd.owner, bd.pos, bd.ty?)
        };
        let c = self.get_town(who, ty, pos)?;
        self.add_to_city(b, c);
        Some(c)
    }

    /// `Build::add_to_city`.
    pub fn add_to_city(&mut self, b: usize, c: usize) {
        self.buildings[b].city = Some(c);
        if self.building_is_city(b) {
            return;
        }
        if !self.cities[c].members.contains(&b) {
            self.cities[c].members.push(b);
        }
        if self.buildings[b].active && self.building_is(b, Ident::Temple) {
            self.sync_territory();
        }
    }

    /// `Build::remove_from_city@00622030`.
    ///
    /// The whole body sits under one gate — `(flags & 0x20) == 0 && city >=
    /// 0`, a city centre and an unattached building do nothing — and both
    /// callers here already hold it, so it is the caller's.
    ///
    /// **`City::regen_roads` is called from here too** (`crate::roads` §1),
    /// and it is the writer Great Lakes 10230 reaches: the building is
    /// unlinked from the `city_down` chain *first*, so what gets flagged is
    /// the city's **remaining** buildings, and each of them then replans on
    /// its own `(frame + o) % 16` slot over the sixteen frames that follow.
    /// The original clears `city` after the call rather than before; the
    /// order is immaterial because the flag goes on the survivors either
    /// way, and taking it first is what keeps the borrow local here.
    pub fn remove_from_city(&mut self, b: usize) {
        if let Some(c) = self.buildings[b].city.take() {
            self.cities[c].members.retain(|&m| m != b);
            if self.buildings[b].active && self.building_is(b, Ident::Temple) {
                self.sync_territory();
            }
            self.city_regen_roads(c);
        }
    }

    /// `City::find_buildings`: the radius sweep — own city-bound buildings
    /// inside the radius join; foreign ones that need a city are converted.
    /// Ends in `check_upgrade`.
    pub fn find_buildings(&mut self, c: usize) {
        let (who, pos) = (self.cities[c].owner, self.cities[c].pos);
        let r = self.radius_of(c).min(64);
        let ct = pos.tile();
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive || bd.city.is_some() || self.building_is_city(b) {
                continue;
            }
            let Some(ty) = bd.ty else {
                continue;
            };
            let p = bd.owner;
            if p != who && self.build_types[ty].has(flags::NO_CITY) {
                continue;
            }
            let bt = bd.pos.tile();
            if vector_dist(ct.x - bt.x, ct.y - bt.y) > r {
                continue;
            }
            if p == who {
                self.add_to_city(b, c);
            } else if let Some(n) = self.swap_team(b, who) {
                self.activate(n, false, false);
                self.close_building(b, true);
            } else {
                // A failed conversion kills the building and **ends the
                // sweep** — the rest waits for the next trigger.
                self.die_building(b);
                return;
            }
        }
        self.check_upgrade(c);
    }

    /// `Leader::gain_tech@006dcb60:1324–1356`, the Civic library level's
    /// arm: every live city of `who`, in slot order, is re-masked
    /// (`Wall::mask_city` at `CityData::get_radius`) and then swept by
    /// [`Sim::find_buildings`]. The gate is the gained type's
    /// `is_epoch_type` (vslot `+0x38`) and `TechTypeData +0x14 == 1`, the
    /// line the same function reads as `0` for Military's `calc_pop_cap`
    /// and `3` for Science's library re-price.
    ///
    /// The radius itself does not read the Civic level
    /// (`LeaderData::get_radius@006db790` is the city's level and the
    /// Indian bonus), so what the arm buys is the **sweep**: placement's
    /// `get_town` wants every footprint tile inside the city mask, and
    /// `find_buildings` only the centre tile within the radius. A building
    /// that straddled the mask's edge when it was placed stands cityless
    /// until a sweep runs, and a Civic level is the sweep a settled
    /// leader meets most. Great Lakes' who=1 Barracks and Stable stood
    /// so from their placing until block 8734, when a Civic level swept
    /// both into Norwich — and a trade route's worth is that chain's
    /// length (`docs/AI.md` §63).
    pub(crate) fn civic_epoch_sweep(&mut self, who: Player) {
        for c in self.cities_of(who) {
            self.mask_city(c, true);
            self.find_buildings(c);
        }
    }

    /// `CityData::enough_kinds(n)`: at least `n` distinct completed building
    /// types on the chain, the city itself first.
    pub fn enough_kinds(&self, c: usize, n: i32) -> bool {
        let n = n.min(24);
        let city = &self.cities[c];
        let mut kinds: Vec<usize> = Vec::new();
        for b in std::iter::once(city.building).chain(city.members.iter().copied()) {
            let bd = &self.buildings[b];
            if !bd.alive || !bd.active {
                continue;
            }
            let Some(t) = bd.ty else { continue };
            if !kinds.contains(&t) {
                kinds.push(t);
                if kinds.len() as i32 >= n {
                    return true;
                }
            }
        }
        false
    }

    /// `CityData::num_kinds`: the same count, unbounded.
    pub fn num_kinds(&self, c: usize) -> i32 {
        let city = &self.cities[c];
        let mut kinds: Vec<usize> = Vec::new();
        for b in std::iter::once(city.building).chain(city.members.iter().copied()) {
            let bd = &self.buildings[b];
            if bd.alive
                && bd.active
                && let Some(t) = bd.ty
                && !kinds.contains(&t)
            {
                kinds.push(t);
            }
        }
        kinds.len() as i32
    }

    /// `CityData::ready_to_upgrade`: not at the top, the next level
    /// available to the owner, and enough kinds.
    pub fn ready_to_upgrade(&self, c: usize) -> Option<usize> {
        let city = &self.cities[c];
        let t = self.buildings[city.building].ty?;
        let next = build::upgrades_to(&self.build_types, t)?;
        if let Some(id) = self.build_types[next].tree
            && self.type_avail(city.owner, id) != crate::tech::AVAILABLE
        {
            return None;
        }
        let needed = match self.build_types[next].ident {
            Ident::Town => self.tuning.city_buildings + 1,
            _ => self.tuning.metro_buildings + 1,
        };
        self.enough_kinds(c, needed).then_some(next)
    }

    /// `City::check_upgrade`: the automatic level-up.
    pub fn check_upgrade(&mut self, c: usize) {
        let Some(next) = self.ready_to_upgrade(c) else {
            return;
        };
        let b = self.cities[c].building;
        let owner = self.cities[c].owner;
        self.set_type(b, next);
        self.mask_city(c, true);
        // `leader_flags |= 0x8000000`: the Senate bonus and the new hits
        // arrive through `calc_wall_stats` on the next frame.
        self.wall_stats_dirty[owner as usize] = true;
        self.find_buildings(c);
        self.sync_pop_cities();
        self.sync_territory();
    }

    /// `Wall::set_type`: the building becomes another type of the same
    /// lineage; its damage is kept.
    fn set_type(&mut self, b: usize, ty: usize) {
        self.buildings[b].ty = Some(ty);
        self.buildings[b].combat = Some(self.build_types[ty].combat.unwrap_or_default());
        self.update_hits(b);
    }

    // ------------------------------------------------------------------
    // Territory and the population cap
    // ------------------------------------------------------------------

    /// The territory sources the cities and forts project, rebuilt from the
    /// records — and the borders recomputed wholesale (`docs/ATTRITION.md`'s
    /// stated simplification).
    pub fn sync_territory(&mut self) {
        self.fix_borders();
        // `World::compute_reg_territory` rebuilds its whole per-player table
        // at the top of every region pass (`006b0bb0` lines 125–260) rather
        // than caching it, so the Civic level a leader holds *now* is what
        // its cities project. This crate cached the table at `add_player`
        // and never rewrote it, which cost the AI its `CIVIC_UPGRADE_TERR`
        // step for the whole of a game (item 113).
        for w in 0..self.players.len() {
            self.borders[w] = self.player_borders(w as Player);
        }
        // Drop every source a city or fort owns, keep the ones tests put in
        // by hand, then add the live ones back.
        let mut owned: Vec<usize> = self
            .cities
            .iter()
            .filter_map(|c| c.source)
            .chain(self.buildings.iter().filter_map(|b| b.fort_source))
            .collect();
        owned.sort_unstable();
        owned.dedup();
        for i in owned.into_iter().rev() {
            self.sources.remove(i);
        }
        for c in &mut self.cities {
            c.source = None;
        }
        for b in &mut self.buildings {
            b.fort_source = None;
        }
        for c in 0..self.cities.len() {
            if !self.cities[c].alive {
                continue;
            }
            let city = &self.cities[c];
            let owner = city.owner;
            let level = (self.city_level_of(c) - 1).clamp(0, 2) as usize;
            let src = territory::city_source(
                &self.tuning,
                owner,
                &self.borders[owner as usize],
                &territory::City {
                    pos: city.pos,
                    level,
                    capital: city.capital,
                    temple: self.city_has(c, Ident::Temple),
                    owner_agrees: true,
                },
            );
            self.sources.push(src);
            self.cities[c].source = Some(self.sources.len() - 1);
        }
        for b in 0..self.buildings.len() {
            let bd = &self.buildings[b];
            if !bd.alive
                || !bd.active
                || !bd.ty.is_some_and(|t| build::is_fort(&self.build_types, t))
            {
                continue;
            }
            let src = territory::fort_source(
                &self.tuning,
                bd.owner,
                &self.borders[bd.owner as usize],
                &territory::Fort {
                    pos: bd.pos,
                    red_fort: self.building_ident(b) == Ident::RedFort,
                },
            );
            self.sources.push(src);
            self.buildings[b].fort_source = Some(self.sources.len() - 1);
        }
        self.recompute_territory();
        self.update_territory_holdings();
    }

    /// `Region::fix_borders@00680f60` (and `Regions::fix_all_borders`,
    /// the same loop): every region's resume index (`Region::borders`,
    /// `+0x2c`) goes to 0. Its one effect this crate keeps beyond the
    /// wholesale recompute is the **zero of `reg_known_rares`**:
    /// `GameDaemon::check_borders@00732060` runs every frame and calls
    /// `World::compute_reg_territory@006b0bb0` on each unfinished region,
    /// and that call, when the region's index is 0, zeroes `reg_terr[r]`
    /// and `reg_known_rares[r]` for every live leader **before** its
    /// 256-cell budget test (`:76–105`). So the first `check_borders`
    /// after a fix zeroes the whole array, and only the census's step 9
    /// counts it again. A `calc_gather` recompute in between sums 0 into
    /// `known_rares`, and `create_units`' Merchant arm reads that sum
    /// (`docs/AI.md` §76). `reg_terr`'s zero is the transient this crate's
    /// wholesale recompute does not model (parked 568).
    ///
    /// Every [`Sim::sync_territory`] is a fix: its call sites are the
    /// original's (`Build::activate`, `Build::finished`, `Build::close`,
    /// `City::check_upgrade`, `City::assimilate`, `gain_tech`'s Civic arm,
    /// `calc_gather`'s rare arm). The fix only raises
    /// [`Sim::borders_fixed`]; the zero is [`Sim::check_borders`]', at the
    /// daemon's point in the tick. A fix raised by an object therefore
    /// zeroes on the next frame, after that frame's `calc_gather` and
    /// census, and East Indies' run227 shows it there: the original's
    /// `reg_known_rares[7]` and `[11]` go to 0 on block 16529, one after
    /// the fix's own.
    pub(crate) fn fix_borders(&mut self) {
        self.borders_fixed = true;
    }

    /// `GameDaemon::check_borders@00732060`'s one modelled effect: after
    /// a fix, every leader's `reg_known_rares` is zeroed. The original
    /// zeroes a region again on every frame its pass has not yet reached
    /// under the 256-cell budget; this crate zeroes once (§76.5).
    pub(crate) fn check_borders(&mut self) {
        if !self.borders_fixed {
            return;
        }
        self.borders_fixed = false;
        for a in &mut self.ai {
            a.census.reg_known_rares.iter_mut().for_each(|s| *s = 0);
        }
    }

    fn remove_source_of_city(&mut self, c: usize) {
        // Rebuilt wholesale by `sync_territory`; nothing to do here beyond
        // marking, which the `alive` flag already does.
        let _ = c;
    }

    /// `Leader::calc_pop_cap`'s city term: every live city's level into the
    /// owner's [`cost::PopBonuses::cities`] — and, on the same walk,
    /// [`Sim::sync_leader_pop`], because the original updates the two
    /// together.
    pub fn sync_pop_cities(&mut self) {
        for m in &mut self.muster {
            m.bonuses.cities.clear();
        }
        for c in 0..self.cities.len() {
            if !self.cities[c].alive {
                continue;
            }
            let level = match self.city_level_of(c) {
                2 => cost::CityLevel::Town,
                3 => cost::CityLevel::Metropolis,
                _ => cost::CityLevel::Village,
            };
            let owner = self.cities[c].owner as usize;
            self.muster[owner].bonuses.cities.push(level);
        }
        self.sync_leader_pop();
        self.recompute_pop_caps();
    }

    /// `LeaderData::pop` (`+0x95c`) and `reg_pop` (`+0xe62`), which
    /// `docs/AI.md` §2.3 records as kept by the city lifecycle and which
    /// nothing here wrote — so `create_units`' `base` and `research_techs`'
    /// were identically zero (`docs/AI.md` §27).
    ///
    /// Both are the sum of `CityData::get_pop_value@00738450` over the
    /// leader's live cities: **1** for a Small City, **3** for a Large City
    /// (`TOWN`, `0x19f`) and **5** for a Major City or the Forbidden City
    /// (`0x1a0`, `0x213`) — `2 · city_level − 1`. The original keeps the
    /// running sum incrementally and every writer is that function of one
    /// city (`City::init@00737050`, `City::close@00737550`,
    /// `City::capture@00736c40`, `City::check_upgrade@00738b20` and
    /// `Build::finished@00628490`'s upgrade delta), so a recount over the
    /// live cities is the same number, and every one of those five writers
    /// is a call site of [`Sim::sync_pop_cities`] here.
    ///
    /// `reg_pop` is written only for a region below `0x40`, which is the
    /// original's own bound on the array; a city off the region table
    /// contributes to `pop` and to no region.
    pub(crate) fn sync_leader_pop(&mut self) {
        let regions = self.world.region_count();
        for a in &mut self.ai {
            a.census.pop = 0;
            a.census.reg_pop.clear();
            a.census.reg_pop.resize(regions, 0);
        }
        for c in 0..self.cities.len() {
            if !self.cities[c].alive {
                continue;
            }
            let owner = self.cities[c].owner as usize;
            let value = 2 * self.city_level_of(c) - 1;
            // The city building's cell, not `CityData::reg` — the same
            // source step 8's `reg_cities` recount reads, and the one that
            // is already there when a city is stood up from a dump.
            let reg = self
                .world
                .region_of(self.buildings[self.cities[c].building].pos.cell());
            let Some(a) = self.ai.get_mut(owner) else {
                continue;
            };
            a.census.pop += value;
            if let Some(r) = reg
                && r < 0x40
                && let Some(slot) = a.census.reg_pop.get_mut(r as usize)
            {
                *slot += value;
            }
        }
    }

    // ------------------------------------------------------------------
    // Placement and construction
    // ------------------------------------------------------------------

    /// What a building of a type costs a player now — `get_cost` through
    /// `docs/COSTS.md`'s ramp, with the count of this type the player has
    /// placed.
    pub fn building_price(&self, who: Player, ty: usize) -> [i32; RESOURCES] {
        let of_type = self
            .buildings
            .iter()
            .filter(|b| b.alive && b.owner == who && b.ty == Some(ty))
            .count() as i32;
        let holdings = &self.holdings[who as usize];
        cost::charges(
            &self.tuning,
            &self.build_types[ty].price,
            cost::Counts {
                of_type,
                of_group: 0,
            },
            &cost::Modifiers::default(),
            &holdings.available,
            &holdings.discovered,
            &self.redirects,
        )
    }

    /// `Group::action_build`: the player's order — the site, the city limit,
    /// the price, then the building. Returns its index.
    pub fn place_building(&mut self, who: Player, ty: usize, pos: Pos) -> Result<usize, PlaceFail> {
        let r = self.blocked_site(Some(who), ty, pos, None);
        if r != Blocked::Clear {
            return Err(PlaceFail::Blocked(r));
        }
        if build::is_city(&self.build_types, ty)
            && self.build_types[ty].ident != Ident::ForbiddenCity
            && self.at_city_limit(who)
        {
            return Err(PlaceFail::CityLimit);
        }
        let charges = self.building_price(who, ty);
        let available = self.holdings[who as usize].available;
        if !cost::can_pay(&charges, &self.ledgers[who as usize], &available, 1) {
            return Err(PlaceFail::Cost);
        }
        cost::pay(&charges, &mut self.ledgers[who as usize], &available, false);
        self.economy_changed(who);
        Ok(self.init_build(who, ty, pos, false))
    }

    /// `Objects::init_build` → `Build::init` → `Wall::init`: the building
    /// exists, placed and unstarted; its footprint is reserved, its
    /// construction time frozen, its hit points at the site's first frame,
    /// and — if it is not a city — its city chosen. `restore` is the
    /// "re-creating, not building" flag `Wall::swap_team` passes; besides the
    /// build particle and the `buildings_built` tally the original skips, it
    /// is the gate on [`Sim::farms_add`] (`Build::init` line 247), so a farm
    /// changing hands keeps the record it already had.
    pub fn init_build(&mut self, who: Player, ty: usize, pos: Pos, restore: bool) -> usize {
        let pos = self.snap_center(ty, pos);
        let bt = self.build_types[ty].clone();
        let capacity = crate::build::queue_capacity(&self.build_types, ty);
        let b = self.buildings.len();
        let index = self
            .find_free(who, crate::BUILD_BASE, crate::WALL_BASE)
            .expect("a player's building band is full");
        self.buildings.push(Building {
            owner: who,
            index,
            pos,
            queue: crate::production::Queue::new(capacity),
            is_library: bt.ident == Ident::Library,
            combat: Some(bt.combat.unwrap_or_default()),
            hits: 0,
            health: 0,
            damage_frac: 0,
            active: false,
            garrison_attack: 0,
            recharging: 0,
            target: None,
            ordered: false,
            targeted: 0,
            ty: Some(ty),
            orig_ty: Some(ty),
            infiltrated: 0,
            alive: true,
            started: false,
            activated: false,
            regen_roads: false,
            damage: 0,
            job_counter: 0,
            job_counter_2: 0,
            constr_time: 0,
            construct_hits: 0,
            helpers: 0,
            under_attack: 0,
            eject_pending: false,
            hold_frames: 0,
            city: None,
            garrison: Vec::new(),
            founder: who,
            clock: build::ClockMods::default(),
            hit_frame: None,
            fort_source: None,
            gatherers: Vec::new(),
            gather_max: None,
            gather_from: Vec::new(),
            gather_bumped: false,
            dock_slot: None,
            farm: crate::Farm::default(),
            ever_seen: 0,
            ever_seen_completed: 0,
        });
        // `start_me(1)`: reserve the footprint.
        let corner = self.tile_corner(ty, pos);
        for t in self.footprint(ty, corner) {
            if self.world.tile_mask(t) & tile::PLACED != 0 {
                self.world.set_tile_bits(t, tile::PLACED_TWICE);
            } else {
                self.world.set_tile_bits(t, tile::PLACED);
            }
        }
        // The clock — baked now, and again on every `calc_wall_stats`.
        let mods = self.build_mods(who);
        self.buildings[b].constr_time =
            build::construct_base(&self.tuning, &self.build_types, ty, &mods);
        self.update_hits(b);
        // **The flattening, at placement** — `Wall::init@0063e9b0:70`,
        // after `check_ever_seen` and under `param_6 == 0`, which is
        // `restore`. It moved here from `Wall::start` on run72
        // (`docs/ROADS.md` §7.6): the two calls land on the same frame for
        // a building placed and started at once — run32's two cheat-channel
        // enhancers, which is why no capture before could tell them apart —
        // and **226 frames apart** for one the AI builds. The AI's Market
        // `1/2015` is placed on Great Lakes' frame 4577 and finishes on
        // 4803, and the original's height grid is already flat under it at
        // 4802 where this crate's was still the map generator's on 62
        // tiles.
        //
        // **A farm never terraforms.** `Wall::init`'s test is
        // `TVar10 != FARM`, an identity on the type rather than a lineage,
        // and it takes `Terrain::refresh_good_z` and
        // `Farms::recalc_heights` out with it. run64's frame 6166 is the
        // measurement: the AI's three farms had moved 182 tiles of height
        // that the original leaves exactly where the map generator put
        // them.
        if !restore && self.build_types[ty].ident != Ident::Farm {
            let (bw, bh) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
            self.terraform_for_building(corner, bw, bh);
        }
        // Membership is decided at placement, restoring or not.
        if !build::is_city(&self.build_types, ty) {
            self.find_city(b);
        }
        // `Build::init` line 249, after `find_city` and before the gather
        // survey: a farm's record is created with the *site*, and the city
        // it just joined is what `Farms::add` counts.
        if !restore && self.build_types[ty].ident == Ident::Farm {
            self.farms_add(b);
        }
        // `Build::init@00629740` line 256 onwards, and the order is the
        // whole of it: the mining list is cleared, and a gather type that is
        // **not flat and not the university** goes to
        // `Build::find_gather_tiles` — which fills the list, marks its
        // tiles, shuffles it off the sync stream and only then computes
        // `gather_max` from it. Everything else takes the plain survey at
        // line 275, where a flat type answers 1 without looking at the map.
        //
        // Until this branch existed every building took the second path, so
        // a camp placed during a run surveyed its own still-empty list and
        // activated with **zero** slots (`docs/QUEUE.md` item 85). A camp
        // stood up from a dump was fine only because the dump handed it a
        // list.
        // No `restore` guard: `Build::init`'s farm arm is the one gated on
        // its sixth argument, and this arm is not. A captured camp runs the
        // walk again and finds every candidate cell already carrying
        // `0x1000` — its own former tiles — so it costs no draw and lands an
        // empty list, which is what the original leaves it with too.
        let t = &self.build_types[ty];
        if t.has(build::flags::GATHER) && !t.has(build::flags::FLAT) && t.ident != Ident::University
        {
            self.find_gather_tiles(b);
        } else {
            self.buildings[b].gather_max = Some(self.max_gatherers(b));
        }
        b
    }

    /// `BuildData::construct_time(0)` for a building.
    pub fn construct_time_of(&self, b: usize) -> i32 {
        let bd = &self.buildings[b];
        match bd.ty {
            Some(t) => build::construct_time(
                &self.tuning,
                &self.build_types[t],
                bd.constr_time,
                &bd.clock,
            ),
            None => 1,
        }
    }

    /// `Wall::update_hits`: the full figure and the site's growing one.
    pub fn update_hits(&mut self, b: usize) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let bd = &self.buildings[b];
        let owner = bd.owner as usize;
        let n = &self.nation[owner];
        let (city_level, has_temple) = match bd.city {
            Some(c) if self.cities[c].alive => {
                (self.city_level_of(c), self.city_has(c, Ident::Temple))
            }
            _ => (0, false),
        };
        let m = build::HitsMods {
            maya: n.maya,
            romans: n.romans,
            hp_upgrade: n.hp_upgrade,
            taj_mahal: n.taj_mahal,
            red_fort: n.red_fort,
            nubians: n.nubians,
            tikal: n.tikal,
            temple_level: n.temple_level,
        };
        let full = build::full_hits(
            &self.tuning,
            &self.build_types,
            ty,
            bd.active,
            city_level,
            has_temple,
            &m,
        );
        let ct = self.construct_time_of(b);
        let bd = &mut self.buildings[b];
        bd.hits = full;
        bd.construct_hits = if bd.active {
            full
        } else {
            build::site_hits(full, bd.job_counter, ct, self.build_types[ty].wonder)
        };
        bd.sync_health();
        if bd.construct_hits <= bd.damage && !bd.garrison.is_empty() {
            bd.eject_pending = true;
        }
    }

    /// `Leader::calc_wall_stats`, the `leader_flags & 0x8000000` answer: every
    /// unfinished building of the player has its clock re-baked
    /// (`update_construct_time`) and every building its hit points refreshed.
    pub(crate) fn calc_wall_stats(&mut self, who: Player) {
        self.wall_stats_dirty[who as usize] = false;
        for b in 0..self.buildings.len() {
            if !self.buildings[b].alive || self.buildings[b].owner != who {
                continue;
            }
            if !self.buildings[b].active
                && let Some(ty) = self.buildings[b].ty
            {
                let mods = self.build_mods(who);
                self.buildings[b].constr_time =
                    build::construct_base(&self.tuning, &self.build_types, ty, &mods);
            }
            self.update_hits(b);
        }
    }

    /// The nation, wonder and tech layer of `update_construct_time`, read off
    /// the player's inputs and their city count now.
    fn build_mods(&self, who: Player) -> build::BuildMods {
        let n = &self.nation[who as usize];
        build::BuildMods {
            maya: n.maya,
            created_faster: n.created_faster,
            versailles: n.versailles,
            // **The rare, not a nation flag** — `Nation::tobacco` was
            // never written by anything, so this term was dead until item
            // 261. Great Lakes' AI collects Tobacco before it places its
            // Tower, and the original's `constr_time` for that Tower is
            // **90909** where `job_time * 100` is 100000: exactly
            // `× 100 / (10 + 100)`. Every earlier building of the same
            // game carries a round number, because the clock is baked at
            // placement and re-baked only for the **unfinished**
            // (`calc_wall_stats`).
            tobacco: self.has_rare(who, crate::economy::TOBACCO),
            british: n.british,
            dutch: n.dutch,
            romans: n.romans,
            no_city: self.city_num(who) == 0,
            speed_upgrade: n.speed_upgrade,
        }
    }

    /// `Wall::start` → `Build::start`: the site is committed.
    pub fn start_building(&mut self, b: usize) {
        if self.buildings[b].started {
            return;
        }
        let Some(ty) = self.buildings[b].ty else {
            self.buildings[b].started = true;
            return;
        };
        self.buildings[b].started = true;
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        // `kill_competing_buildings`: every other not-started building on a
        // double-placed footprint tile goes back, refunded in full.
        let tiles = self.footprint(ty, corner);
        let mut victims = Vec::new();
        for t in &tiles {
            if self.world.tile_mask(*t) & tile::PLACED_TWICE == 0 {
                continue;
            }
            for o in 0..self.buildings.len() {
                if o == b || !self.buildings[o].alive || self.buildings[o].started {
                    continue;
                }
                let Some(oty) = self.buildings[o].ty else {
                    continue;
                };
                let oc = self.tile_corner(oty, self.buildings[o].pos);
                let ot = &self.build_types[oty];
                if t.x >= oc.x
                    && t.x < oc.x + ot.x_size
                    && t.y >= oc.y
                    && t.y < oc.y + ot.y_size
                    && !victims.contains(&o)
                {
                    victims.push(o);
                }
            }
        }
        for o in victims {
            self.disband_building(o, true);
        }
        // **The flattening is not here.** `Wall::start@0063e810`'s own
        // statement at this point is `Terrain::object_placed@00850c40`,
        // which is the **renderer's** (`spot_update_land`,
        // `invalidate_wcoord`) and moves no height; the terraform belongs
        // to `Wall::init@0063e9b0:70`, which runs when the *site* is
        // created ([`Sim::init_build`], `docs/ROADS.md` §7.4).
        self.mask_building(b, true);
        // `Wall::start@0063e810` passes **`REGEN_FORCE`** to `mask_me`, and
        // `BuildType::mask_me@006312a0`'s tail is `place_roads` — so a
        // building lays its ring and plans its road the moment it *starts*,
        // not only when `Build::process` comes round to its flag. run32 is
        // the capture: two enhancers dropped at sim-frame 100 spend 2,913
        // road-cost draws in that frame, before phase 1, and the roads are
        // on the map four frames before their scheduled replans
        // (`docs/ROADS.md` §1).
        self.place_roads(b);
        // **The owner's own bit over the footprint** (`Wall::start@0063e810`,
        // the loop between `mask_me` and `check_ever_seen`): every tile of
        // the `x_size × y_size` rectangle — not grown, as
        // `update_local_seen`'s is — on the map ors `1 << who` into
        // `seen2` at its half-cell, `tile >> 1`, and into the cell's `WData
        // +0x14`, which this world does not keep. A raw byte write, so no
        // `set_seen` side effect follows. It matters where the owner's line
        // of sight does not reach: the AI places a city out of its own
        // sight, and its scout's search then prices that footprint as seen
        // ground (run157's 1076, `docs/VISION.md` §6.1, `docs/SCOUT.md`
        // §14).
        let who = self.buildings[b].owner;
        if who < 8 && self.world.has_fog() {
            for t in self.footprint(ty, corner) {
                if self.world.tile_in_bounds(t) {
                    self.world.set_seen2_only(t.x >> 1, t.y >> 1, 1 << who);
                }
            }
        }
        // `Wall::start@0063e810`'s own last-but-one statement.
        self.check_ever_seen(b, false);
    }

    /// `Wall::mask_me` → `BuildType::mask_me`: the footprint marked (or
    /// unmarked) and, for a city, its radius.
    fn mask_building(&mut self, b: usize, on: bool) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        // `mask_me@006312a0`'s **first** write, before the footprint walk:
        // the flag on the cell holding the building's own position
        // (`cells[y / 0x300 * xs + x / 0x300].flags |= 0x4000`, and
        // `&= 0xbfff` on the unmask). `docs/CITIES.md` §3.6 named it at the
        // first reading and nothing set it until item 352 —
        // `Army::find_muster_spot`'s ring score is its one reader in the
        // export, and with the flag missing every cell of the AI's own town
        // scored as open ground (`docs/ARMY.md` §13).
        let centre = self.buildings[b].pos.cell();
        let mut d = self.world.cell_data(centre);
        if on {
            d.flags |= crate::world::cell::BUILDING;
        } else {
            d.flags &= !crate::world::cell::BUILDING;
        }
        self.world.set_cell_data(centre, d);
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        // `set_blocked_at` runs only where the type's per-tile mask
        // template byte is 1 (`mask_me@006312a0`), and **clears** the bit
        // everywhere else on the footprint — so the object field covers the
        // whole rectangle while the blocked bit covers only what the
        // graphic's mask says. `BuildType::blocks` is that template
        // (`masks.txt`, `docs/DATALAYER.md`); without one loaded it falls
        // back on the old flat/non-flat line.
        let (xs, ys) = (
            self.build_types[ty].x_size.max(1),
            self.build_types[ty].y_size.max(1),
        );
        // `is_city || connects_to_roads` — the one predicate both of
        // `mask_me`'s road arms read, from opposite sides.
        let lays = build::is_city(&self.build_types, ty)
            || crate::roads::connects_to_roads(&self.build_types, ty);
        for (i, t) in self.footprint(ty, corner).into_iter().enumerate() {
            let (u, v) = (i as i32 % xs, i as i32 / xs);
            debug_assert!(v < ys);
            if on {
                self.world
                    .clear_tile_bits(t, tile::PLACED | tile::PLACED_TWICE);
                self.world
                    .set_tile_field(t, tile::OBJECT, tile::OBJECT_BUILDING);
                // **`mask_me`'s two road arms, and they are one arm.** A
                // footprint tile of a type that does not connect to roads
                // has its road **taken away** (`006312a0`'s
                // `set_road_at(…, 0, 0, 0)`, gated on
                // `!is_city && (is_gather_type || NO_CITY) && !is(UNIVERSITY)`
                // — which is exactly `!lays`); a tile of one that does, and
                // whose template byte is not 1, has one **laid**. Both go
                // through the mesh door, `param_5 = 0`.
                if !lays {
                    self.world_set_road_at(t, false, 0, 0);
                }
                // Through `set_blocked_at`, never by hand: the bit is only
                // half of it — the containing cell's `blocked`/`solid`
                // counts are the pathfinder's terrain cost
                // (`docs/PATHFINDER.md` §5), and they move nowhere else.
                let blocks = self.build_types[ty].blocks(u, v);
                self.world.set_blocked_at(t, blocks);
                if !blocks && lays {
                    self.world_set_road_at(t, true, 0, 0);
                }
            } else {
                self.world.set_tile_field(t, tile::OBJECT, 0);
                self.world.set_blocked_at(t, false);
                self.world
                    .clear_tile_bits(t, tile::PLACED | tile::PLACED_TWICE);
            }
        }
        if build::is_dock(&self.build_types, ty) {
            self.mask_dock_water(b, on);
            if !on {
                // `Docks::remask_docks@00740c90`: every other live dock,
                // every player's, lays its margin again, so an overlap
                // survives the unmask.
                for o in 0..self.buildings.len() {
                    let other = &self.buildings[o];
                    if o != b
                        && other.alive
                        && other.started
                        && other
                            .ty
                            .is_some_and(|t| build::is_dock(&self.build_types, t))
                    {
                        self.mask_dock_water(o, true);
                    }
                }
            }
        }
        if build::is_city(&self.build_types, ty) {
            let r = match self.buildings[b].city {
                Some(c) if self.cities[c].alive => self.radius_of(c),
                _ => self.city_radius(self.buildings[b].owner, Some(ty)),
            };
            let pos = self.buildings[b].pos;
            for t in self.city_mask_tiles(pos, r) {
                if on {
                    self.world.set_tile_bits(t, tile::CITY_RADIUS);
                } else {
                    self.world.clear_tile_bits(t, tile::CITY_RADIUS);
                }
            }
            if !on {
                // `Build::close`: every other alive started city re-lays its
                // mask, so an overlap survives.
                for o in 0..self.cities.len() {
                    if self.cities[o].alive && o != self.buildings[b].city.unwrap_or(usize::MAX) {
                        self.mask_city(o, true);
                    }
                }
            }
        }
    }

    /// **A dock marks the water around it bad** — `BuildType::mask_me@
    /// 006312a0`'s `is(DOCK)` arm, after the footprint walk: every
    /// **ocean** tile (`mask & 0x30 == 0x20`) of the footprint grown by
    /// three tiles on every side takes [`tile::BAD_PATH`] through
    /// `World::set_bad_path`, and loses it on the unmask. A warship may not
    /// be placed on such a tile (`find_nearby_spot`'s `0x2400` test), so a
    /// dock's own warships are born at least three tiles out: East Indies'
    /// Trireme `1/32` lands due east of Dock `1/2010`, the first bearing of
    /// its ring that clears the margin (`docs/ORDERS.md` §25).
    pub(crate) fn mask_dock_water(&mut self, b: usize, on: bool) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let (xs, ys) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        for dx in -3..xs + 3 {
            for dy in -3..ys + 3 {
                let t = Pos::new(corner.x + dx, corner.y + dy);
                if !self.world.tile_in_bounds(t) {
                    continue;
                }
                if self.world.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN {
                    self.world.set_bad_path(t, on);
                }
            }
        }
    }

    /// `Wall::mask_city` for a city record.
    fn mask_city(&mut self, c: usize, on: bool) {
        let r = self.radius_of(c);
        let pos = self.cities[c].pos;
        for t in self.city_mask_tiles(pos, r) {
            if on {
                self.world.set_tile_bits(t, tile::CITY_RADIUS);
            } else {
                self.world.clear_tile_bits(t, tile::CITY_RADIUS);
            }
        }
    }

    /// `Wall::do_construct(amount)`: one builder's contribution this frame.
    /// Returns whether the building finished.
    pub fn do_construct(&mut self, b: usize, amount: i32) -> bool {
        if !self.buildings[b].alive {
            return false;
        }
        if !self.buildings[b].started {
            let (who, ty, pos) = {
                let bd = &self.buildings[b];
                (bd.owner, bd.ty, bd.pos)
            };
            if let Some(ty) = ty {
                let r = self.blocked_site(Some(who), ty, pos, Some(b));
                let wonder_ok = r == Blocked::Wonder
                    && self.buildings[b].city.is_some_and(|c| {
                        self.num_wonders(c, true)
                            <= 1 + i32::from(self.nation[who as usize].egyptians)
                    });
                let ok = matches!(
                    r,
                    Blocked::Clear
                        | Blocked::One
                        | Blocked::OneOther
                        | Blocked::Farm
                        | Blocked::NeedWall
                ) || wonder_ok;
                if !ok {
                    self.disband_building(b, true);
                    return false;
                }
            }
            self.start_building(b);
        }
        if self.buildings[b].active {
            return false;
        }
        let ct = self.construct_time_of(b);
        let bd = &mut self.buildings[b];
        let share = build::contribution(amount, bd.helpers);
        bd.helpers += 1;
        bd.job_counter_2 += share;
        bd.job_counter += share;
        if bd.job_counter >= ct {
            self.activate(b, false, true);
            return true;
        }
        false
    }

    /// **`Build::activate@00623e20`'s Senate arm** (`docs/CITIES.md` §15):
    /// a Senate finished, not captured, in a live city of the owner's own
    /// race, **moves the capital here** when the owner's capital
    /// (`LeaderData::find_capital`'s first arm, the first live own city
    /// with `0x10`) holds no finished Senate — `CityData::count_buildings(
    /// SENATE, 0, 1)`. Every own live city loses `0x10` (with tribe bonus
    /// `0x17`, only one that is not the founding capital, `0x4000`), this
    /// one takes it and `founder = who`, and `Region::fix_borders` runs,
    /// because a capital projects `CAPITAL_TERRITORY_BONUS`.
    ///
    /// Drawless: Great Lakes' Senate `1/2024` moves the capital from
    /// London to Norwich on tick 14528 and spends nothing on it.
    ///
    /// SEAM: the arm's head — the government hero trained when
    /// `gov_hero_frame` is set and none is near (the field is carried since
    /// item 706, `docs/TECH.md` §"The government patriot") — and
    /// `senates_built`, which no reader here carries.
    fn senate_moves_capital(&mut self, b: usize) {
        let who = self.buildings[b].owner;
        let Some(c) = self.buildings[b].city.filter(|&c| self.cities[c].alive) else {
            return;
        };
        if self.cities[c].race != Some(who) {
            return;
        }
        let own = |k: usize, s: &Self| s.cities[k].alive && s.cities[k].owner == who;
        let Some(cap) = (0..self.cities.len()).find(|&k| own(k, self) && self.cities[k].capital)
        else {
            return;
        };
        if self.count_buildings(cap, Ident::Senate, true) != 0 {
            return;
        }
        let keeps_founding =
            self.tech_tree
                .has_tribe_bonus(&self.setup, &self.tech[who as usize], 0x17);
        for k in 0..self.cities.len() {
            if own(k, self) && !(keeps_founding && self.cities[k].founding_capital) {
                self.cities[k].capital = false;
            }
        }
        self.cities[c].capital = true;
        self.cities[c].founder = who;
        self.sync_territory();
    }

    /// `Build::activate(captured, announce, counted)` — `docs/CITIES.md` §4.
    pub fn activate(&mut self, b: usize, captured: bool, counted: bool) {
        if !self.buildings[b].started {
            self.start_building(b);
        }
        let who = self.buildings[b].owner;
        // `Build::activate@00623e20` lines 398/402/443: the economy's dirty
        // flag, so a finished farm pays within eight frames, not 512.
        self.economy_changed(who);
        let Some(ty) = self.buildings[b].ty else {
            self.buildings[b].active = true;
            self.buildings[b].activated = true;
            self.update_seen_build(b);
            return;
        };
        // Chinese cities are founded as Large Cities.
        if build::is_city(&self.build_types, ty)
            && !captured
            && self.nation[who as usize].chinese
            && self.tuning.chinese_large_cities != 0
            && self.build_types[ty].ident != Ident::ForbiddenCity
            && let Some(town) = self.build_types.iter().position(|t| t.ident == Ident::Town)
        {
            self.buildings[b].ty = Some(town);
            self.buildings[b].combat = Some(self.build_types[town].combat.unwrap_or_default());
        }
        // The `Farms` list is joined at *placement* (`Sim::farms_add`), not
        // here — `Farms::add` runs from `Build::init`. This catches a farm
        // the harness stood up from a dump without going through it.
        if self.build_types[ty].ident == Ident::Farm && !self.farm_order.contains(&b) {
            self.farm_order.push(b);
        }
        {
            let bd = &mut self.buildings[b];
            bd.active = true;
            bd.activated = true;
            bd.job_counter = 0;
            bd.job_counter_2 = 0;
        }
        // `Wall::activate@0063e4b0` line 62, immediately after the active
        // flag: an active military trainer joins the leader's own list
        // (`docs/AI.md` §29). The guard the original spells at the call site
        // — `NO_CITY`, or complete and in a city — is inside.
        self.mil_trainer_open(b);
        // `leader_flags |= 0x2000000 | 0x8000000`: the wall stats go stale —
        // which is how a nomad's other sites lose the ×3 once the first city
        // stands.
        self.wall_stats_dirty[who as usize] = true;
        // `Build::activate` line 560: a dock (not a fort) joins the docks
        // registry — `reg_docks`, the gull's two draws (`docs/TRANSPORT.md`
        // §5.1–§5.2).
        let dock = build::is_dock(&self.build_types, ty);
        if dock && !build::is_fort(&self.build_types, ty) {
            self.dock_open(b);
        }
        if self.building_is_city(b) {
            let capital = !captured && self.city_num(who) == 0;
            let c = self.init_city(who, b, captured, capital);
            if !captured {
                self.cities[c].race = Some(who);
                self.cities[c].founder = who;
            }
            if counted {
                self.city_tally[who as usize].built += 1;
            }
            self.mask_city(c, true);
            self.find_buildings(c);
            self.sync_pop_cities();
            self.sync_territory();
        } else {
            if let Some(c) = self.buildings[b].city
                && self.cities[c].alive
                && self.building_is(b, Ident::Temple)
            {
                self.sync_territory();
            }
            if build::is_fort(&self.build_types, ty) {
                self.sync_territory();
            }
            if !captured && self.building_is(b, Ident::Senate) {
                self.senate_moves_capital(b);
            }
        }
        self.update_hits(b);
        if !self.building_is_city(b)
            && let Some(c) = self.buildings[b].city
            && self.cities[c].alive
        {
            self.check_upgrade(c);
        }
        // `Build::activate` line 1978: a finished dock may grant the
        // transport level (`docs/TRANSPORT.md` §4).
        if dock {
            self.check_transport(who);
        }
        // `Build::activate@00623e20+0x744`: the city's roads want
        // replanning, which is what flags every one of its buildings
        // (`crate::roads` §1). ~~It is the only writer a traced game
        // reaches.~~ **Not so**: `Build::remove_from_city` is the other,
        // and Great Lakes 10230 reaches it when a farm finally falls
        // (`docs/ROADS.md` §1.2, item 478).
        if let Some(c) = self.buildings[b].city
            && self.cities[c].alive
        {
            self.city_regen_roads(c);
        }
        // `Build::activate@00623e20` line 590, and it comes *before* the
        // gather block: a dock, a market or a temple is a wealth slot
        // (`docs/ECONOMY.md`, "The wealth slot, and the thirty it pays").
        self.claim_commerce_slot(b, captured, counted);
        // `Build::activate@00623e20` lines 603–1150, between the two: the
        // building high-water mark, and the nation's free units for a
        // first-of-its-kind building (`crate::nations`, §4.3).
        self.claim_building_high(b, captured, counted);
        // `Build::activate@00623e20` lines 1151–1205: the gather slots, and
        // the bonus for the ones the player has never held.
        self.claim_gather_slots(b, captured, counted);
        // The function's **last** statement, vtable `+0x174` with `ring =
        // 0`: the finished building lights its whole fog disc
        // (`docs/VISION.md` §2.1, §6).
        self.update_seen_build(b);
    }

    /// **`Build::activate@00623e20` line 590 — the wealth slot, and the
    /// thirty it pays.** This is the block `docs/ECONOMY.md` used to call a
    /// third `do_bonus` "off a separate pair of counters at `+0x8ac` and
    /// `+0x8dc`". They are not a separate pair: `LeaderData +0x8a4` is
    /// `gather_slots` and `+0x8d4` is `gather_slots_high`, so `+0x8ac` is
    /// `gather_slots[2]` and `+0x8dc` its high-water mark. **The wealth slot
    /// nothing here wrote and the thirty wealth nobody paid are one line.**
    ///
    /// The kinds are three, and the first is a vtable call the decompiler
    /// leaves as `(**(code **)(**(int **)&this->field_0x18 + 0x108))()`:
    /// `ObjectData::is_dock@004711e0` is that call and nothing else, so slot
    /// `+0x108` on the type is `is_dock`. Then `is(MARKET)` and
    /// `is(TEMPLE)`, spelled out.
    ///
    /// The shape is the gather block's exactly — claim always, pay only past
    /// the mark and only away from frame 0 — and `Build::close@00628980`
    /// line 107 is the mirror, inside the same guard as the gather slots'.
    fn claim_commerce_slot(&mut self, b: usize, captured: bool, counted: bool) {
        if !self.commerce_slot(b) {
            return;
        }
        let who = self.buildings[b].owner as usize;
        let i = crate::economy::Resource::Wealth.index();
        let ledger = &mut self.ledgers[who];
        ledger.gather_slots[i] += 1;
        if ledger.gather_slots[i] > ledger.gather_slots_high[i] {
            ledger.gather_slots_high[i] = ledger.gather_slots[i];
            if self.frame != 0 && !captured && counted {
                // `do_bonus(this, 2, 0x1e)` — a literal thirty, not a
                // `Constants` slot, through the same German multiplier the
                // gather bonuses take.
                let mut amount = 30;
                if self.nation[who].germans {
                    amount = (self.tuning.german_completion_bonus + 100) * amount / 100;
                }
                self.ledgers[who].bucket[i] += amount;
            }
        }
    }

    /// The three kinds `claim_commerce_slot` counts: a dock, a market or a
    /// temple.
    fn commerce_slot(&self, b: usize) -> bool {
        let Some(ty) = self.buildings[b].ty else {
            return false;
        };
        build::is_dock(&self.build_types, ty)
            || matches!(self.build_types[ty].ident, Ident::Market | Ident::Temple)
    }

    /// `Build::activate`'s tail: a finished gather building's slots join the
    /// leader's running count, and whatever part of them is past the
    /// high-water mark is paid for — `Build::do_bonus`
    /// (`docs/ECONOMY.md`, "What a finished gather building pays").
    ///
    /// The slots are claimed whatever the frame and whoever asked, because
    /// the counter is the player's inventory; only the *payment* is gated,
    /// on `frame != 0`, on the building not arriving by capture, and on the
    /// caller counting it — which is what keeps a dump's own buildings and
    /// the frame-0 setup from paying.
    fn claim_gather_slots(&mut self, b: usize, captured: bool, counted: bool) {
        let Some(r) = self.gather_good(b) else { return };
        let who = self.buildings[b].owner as usize;
        let slots = self.buildings[b].gather_max.unwrap_or(0);
        let i = r.index();
        let ledger = &mut self.ledgers[who];
        ledger.gather_slots[i] += slots;
        let fresh = ledger.gather_slots[i] - ledger.gather_slots_high[i];
        if fresh > 0 {
            ledger.gather_slots_high[i] = ledger.gather_slots[i];
            if self.frame != 0 && !captured && counted {
                let mut amount = crate::economy::completion_bonus(&self.tuning, r, fresh);
                if self.nation[who].germans {
                    amount = (self.tuning.german_completion_bonus + 100) * amount / 100;
                }
                self.ledgers[who].bucket[i] += amount;
            }
        }
        // **And the pasture is stocked.** `LAB_00625a36` — the fall-through
        // from `do_bonus(0, FOOD_BONUS_FOR_FARM)` — and the `iVar18 == 0`
        // arm of `switchD_006259b5_caseD_2`, the label every *un*paid exit
        // above jumps to, are the same call: whatever the bonus did or did
        // not do, a finished **food** gather building runs
        // `Farms::add_animals`, and there a pasture costs twenty draws
        // (`crate::farms::Sim::farm_stock_pasture`).
        //
        // **Not at frame 0**, which is where this crate parts from the
        // original and does so knowingly: the original's starting farms are
        // stood up inside `Setup::build_empire` with `Game::frame` still 0 —
        // the same test the bonus is gated on one line above — and they do
        // stock there. The harness does not replay the setup stream, so a
        // pasture that already exists at frame 0 is stood up from the
        // capture's own trace instead (`Sim::farm_add_animals`, `docs/SYNC.md`
        // §3.11) and this call would double it.
        if matches!(r, crate::economy::Resource::Food) && self.frame != 0 {
            self.farm_stock_pasture(b);
        }
    }

    /// `Object::disband(full)`: the building goes back; the refund is the
    /// fraction not yet built, or the whole price.
    pub fn disband_building(&mut self, b: usize, full: bool) {
        if !self.buildings[b].alive {
            return;
        }
        let who = self.buildings[b].owner;
        // The builders' orders die in `Unit::work`'s liveness test, not here.
        let mut full = full;
        if self.nation[who as usize].lakota && self.tuning.lakota_raze_price != 0 {
            full = true;
        }
        if let Some(ty) = self.buildings[b].ty {
            let bd = &self.buildings[b];
            let ct = self.construct_time_of(b);
            let partial =
                !full && !bd.active && bd.job_counter < self.build_types[ty].job_time * 100;
            if partial || full {
                let price = self.building_price(who, ty);
                let ledger = &mut self.ledgers[who as usize];
                for (g, p) in price.iter().enumerate() {
                    let r = if full {
                        *p
                    } else {
                        build::refund(*p, ct, bd.job_counter_2)
                    };
                    ledger.bucket[g] += r;
                }
            }
        }
        if !self.buildings[b].started {
            self.close_building(b, false);
        } else {
            self.die_building(b);
        }
    }

    /// `Object::die` on a building: close it and keep the slot while the
    /// garrison streams out.
    pub fn die_building(&mut self, b: usize) {
        self.close_building(b, false);
        self.buildings[b].hold_frames = 1;
    }

    /// `Build::close` / `Wall::close`. `silent` is reason 5, a transfer:
    /// the footprint stays marked for the copy that replaces it.
    pub fn close_building(&mut self, b: usize, silent: bool) {
        if !self.buildings[b].alive {
            return;
        }
        let who = self.buildings[b].owner;
        // `Build::close@00628980` line 95: the economy's dirty flag.
        self.economy_changed(who);
        // `clean_queue(refund)`.
        {
            let mut ledger = std::mem::take(&mut self.ledgers[who as usize]);
            while !self.buildings[b].queue.items.is_empty() {
                if let Some(item) = self.buildings[b].queue.unqueue(0, !silent, &mut ledger) {
                    self.muster[who as usize].queued_by_type[item.ty] -= 1;
                    self.track_tree_queued(who, item.ty, -1);
                }
            }
            self.ledgers[who as usize] = ledger;
        }
        if self.buildings[b].started && !silent {
            self.mask_building(b, false);
        } else if let Some(ty) = self.buildings[b].ty {
            // `start_me(0)`: release the reservation.
            let corner = self.tile_corner(ty, self.buildings[b].pos);
            for t in self.footprint(ty, corner) {
                self.world
                    .clear_tile_bits(t, tile::PLACED | tile::PLACED_TWICE);
            }
        }
        if !self.buildings[b].garrison.is_empty() {
            self.buildings[b].eject_pending = true;
        }
        // `Build::close` line 227: a dock leaves the registry while the
        // object is still flagged in use (`docs/TRANSPORT.md` §5.3).
        self.dock_close(b);
        // `Build::close@00628980` line 99, inside the same `flags & 4` guard:
        // an active military trainer leaves the leader's list
        // (`docs/AI.md` §29).
        if self.buildings[b].active {
            self.mil_trainer_close(b);
        }
        // `Build::close@00628980` line 104, inside the same `flags & 4`
        // guard that gates the wall stats: an **active** gather building
        // gives its slots back. The high-water mark does not follow it
        // down, which is what makes a rebuild free of bonus.
        if self.buildings[b].active
            && let Some(r) = self.gather_good(b)
        {
            let slots = self.buildings[b].gather_max.unwrap_or(0);
            self.ledgers[who as usize].gather_slots[r.index()] -= slots;
        }
        // `Build::close@00628980` line 107, three lines below the gather
        // slots and inside the same guard: a dock, a market or a temple
        // gives its wealth slot back too, and the mark stays where it is.
        if self.buildings[b].active && self.commerce_slot(b) {
            let i = crate::economy::Resource::Wealth.index();
            self.ledgers[who as usize].gather_slots[i] -= 1;
        }
        self.buildings[b].alive = false;
        self.buildings[b].damage = self.buildings[b].hits_now();
        self.buildings[b].sync_health();
        self.wall_stats_dirty[who as usize] = true;
        self.removed.push(b);
        self.forget(Obj::Building(b));
        if self.building_is_city(b) {
            if let Some(c) = self.buildings[b].city
                && self.cities[c].alive
                && self.cities[c].building == b
            {
                self.close_city(c, None);
            }
        } else {
            self.remove_from_city(b);
        }
        if self.buildings[b]
            .ty
            .is_some_and(|t| build::is_fort(&self.build_types, t))
        {
            self.sync_territory();
        }
        self.economy_changed(who);
        // `Build::close` line 562: the last dock lost revokes the level
        // (`docs/TRANSPORT.md` §4).
        if self.buildings[b]
            .ty
            .is_some_and(|t| build::is_dock(&self.build_types, t))
        {
            self.check_transport(who);
        }
    }

    // ------------------------------------------------------------------
    // Every frame
    // ------------------------------------------------------------------

    /// `Wall::process` and the Build-specific tail this mechanic owns:
    /// the under-attack decay, the helpers reset, the building's own
    /// attrition, deferred ejection, the capture re-test, the assimilation
    /// tick and the city heal. Queues and combat run from `Sim::tick`.
    pub(crate) fn process_building(&mut self, b: usize, frame: i64) {
        if !self.buildings[b].alive {
            if self.buildings[b].hold_frames > 0 {
                if self.buildings[b].garrison.is_empty() {
                    self.buildings[b].hold_frames = 0;
                } else {
                    self.process_ejection(b);
                }
            }
            return;
        }
        // **`Wall::process@00640450`'s first statement**, on the game's own
        // frame and not the building's phase: every eighth frame, on the one
        // whose low three bits are the owner's player number, a building
        // asks who has looked at it (`docs/VISION.md` §6.1). The `targeted`
        // decay that shares the branch is a seam.
        if frame != 0 && frame & 7 == i64::from(self.buildings[b].owner) {
            self.check_ever_seen(b, false);
        }
        let phase = self.buildings[b].phase(frame);
        if phase & 31 == 0 {
            let bd = &mut self.buildings[b];
            if bd.under_attack & 0x1 != 0 {
                bd.under_attack &= !0x1;
            } else {
                bd.under_attack &= !0x2;
            }
        }
        // The site recruiter (`crate::site_recruit`, `docs/AI.md` §69), in
        // the same 32-frame block as the decay and **ahead of** the helpers
        // reset below, so it reads the builders the site had last frame.
        if phase & 31 == 0
            && !self.buildings[b].active
            && !self.nation[self.buildings[b].owner as usize].human
        {
            self.site_recruit(b);
        }
        self.buildings[b].helpers = 0;
        // A building in enemy territory bleeds.
        let who = self.buildings[b].owner;
        if phase % build::ENEMY_TERRITORY_PERIOD == 0
            && !self.nation[who as usize].disable_building_attrition
            && let Owner::Player(t) = self.world.owner_at(self.buildings[b].pos)
            && t != who
            && !self.is_ally(who, t)
        {
            if !self.buildings[b].started {
                self.disband_building(b, false);
                return;
            }
            self.building_attrition(b, frame);
            if !self.buildings[b].alive {
                return;
            }
        }
        if !self.buildings[b].active {
            return;
        }
        if self.buildings[b].eject_pending {
            self.process_ejection(b);
        }
        // The capture re-test, twice a second, with the nearest enemy land
        // unit within sixteen tiles.
        if phase & 63 == 0 && self.capture_eligible(b) {
            let pos = self.buildings[b].pos;
            let mut best: Option<(i32, usize)> = None;
            for (i, u) in self.units.iter().enumerate() {
                // A `find_unit`, so gaia is out of its space
                // (`world::PLAYER_SLOTS`) — and a sheep cannot capture.
                if !u.alive() || !u.on_map || u.is_gaia() || !self.is_enemy(who, u.owner) {
                    continue;
                }
                if !matches!(u.kind.domain, crate::attrition::Domain::Land) {
                    continue;
                }
                let d = vector_dist(u.pos.x - pos.x, u.pos.y - pos.y);
                if d <= 16 * UNITS_PER_TILE && best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, i));
                }
            }
            if let Some((_, u)) = best
                && self.check_capture(b, u)
            {
                return;
            }
        }
        if self.building_is_city(b)
            && let Some(c) = self.buildings[b].city
            && self.cities[c].alive
        {
            self.assimilation_tick(c, frame);
            // The city heal.
            let bd = &self.buildings[b];
            if bd.damage != 0
                && self.cities[c].race == Some(bd.owner)
                && !self.cities[c].no_heal
                && self.tuning.city_heal_rate > 0
                && phase % i64::from(self.tuning.city_heal_rate) == 0
            {
                let level = self.city_level_of(c);
                self.repair_building(b, level);
            }
        }
        // **`Build::process@0061edf0`'s last statement**, and it is below
        // the `is_active` gate — which is the function's *second*, inlined
        // as `field_0x8 & 4` (`WallData::is_active@00472350` is exactly
        // that expression) and taken the moment `Wall::process` returns. A
        // building that is still a site therefore never replans its roads,
        // however the flag was set; the flag simply stays until the site
        // finishes. This crate ran the replan beside the helpers reset,
        // above the gate, and East Indies 8193 is the frame that says so
        // (`crate::roads` §1, `docs/ROADS.md` §1.1).
        self.regen_roads_due(b, frame);
    }

    /// `Wall::process`'s enemy-territory hit: eight hits through
    /// `take_damage` with the attrition flag.
    fn building_attrition(&mut self, b: usize, frame: i64) {
        let hit = crate::combat::Sixteenths {
            whole: build::ENEMY_TERRITORY_DAMAGE,
            frac: 0,
        };
        self.damage_building(b, hit, None, frame, true);
    }

    /// `Build::repair_damage(amount)`.
    pub fn repair_building(&mut self, b: usize, amount: i32) {
        let bd = &mut self.buildings[b];
        let hits = bd.hits_now();
        bd.damage = bd.damage.clamp(0, hits);
        bd.damage_frac = 0;
        bd.damage = (bd.damage - amount).max(0);
        bd.sync_health();
    }

    /// The sites' hit points refreshed from their progress — `Wall::inc_time`
    /// after every object has been processed.
    pub(crate) fn refresh_site_hits(&mut self) {
        for b in 0..self.buildings.len() {
            if self.buildings[b].alive
                && !self.buildings[b].active
                && self.buildings[b].ty.is_some()
            {
                self.update_hits(b);
            }
        }
    }

    /// The repair arithmetic of `Unit::do_repair` once the repairer stands
    /// adjacent — `docs/CITIES.md` §9.3: the period, the helper count, the
    /// amount, the price, `Build::repair_damage`. The order side is
    /// `orders::do_repair`.
    pub(crate) fn repair_step(&mut self, i: usize, at: usize, frame: i64) {
        let who = self.units[i].owner;
        let Some(ty) = self.buildings[at].ty else {
            self.kill_current_order(i);
            return;
        };
        let bd = &self.buildings[at];
        let (city, captured) = match bd.city {
            Some(c) if self.building_is_city(at) && self.cities[c].alive => {
                (true, self.cities[c].race != Some(bd.owner))
            }
            _ => (false, false),
        };
        let target = build::RepairTarget {
            helpers: bd.helpers,
            construct_time: self.construct_time_of(at),
            hits: bd.hits_now(),
            city,
            captured,
            under_attack: bd.is_under_attack(),
            fresh_hit: bd.under_attack & 0x1 != 0,
        };
        let period = build::repair_period(&self.tuning, &target, self.nation[who as usize].koreans);
        self.buildings[at].helpers += 1;
        let amount = build::repair_amount(frame, period, self.buildings[at].damage);
        if amount == 0 {
            return;
        }
        // The price, again: one unit of a good whenever the repaired
        // figure crosses a multiple of `hits / cost`. Pinned as the
        // exact rational.
        let hits = i64::from(self.buildings[at].hits_now());
        let left = i64::from(hits as i32 - self.buildings[at].damage);
        let price = self.building_price(who, ty);
        let mut due = [0i32; RESOURCES];
        for (g, p) in price.iter().enumerate() {
            if *p <= 0 || hits <= 0 {
                continue;
            }
            let before = left * i64::from(*p) / hits;
            let after = (left + i64::from(amount)) * i64::from(*p) / hits;
            if after > before {
                due[g] = 1;
            }
        }
        let ledger = &mut self.ledgers[who as usize];
        if due.iter().enumerate().any(|(g, d)| *d > ledger.bucket[g]) {
            self.kill_current_order(i);
            return;
        }
        for (g, d) in due.iter().enumerate() {
            ledger.bucket[g] -= d;
        }
        self.repair_building(at, amount);
    }

    // ------------------------------------------------------------------
    // Capture
    // ------------------------------------------------------------------

    /// `BuildData::check_capture_eligible`: a city building, active, at zero.
    pub fn capture_eligible(&self, b: usize) -> bool {
        let bd = &self.buildings[b];
        bd.alive
            && bd.active
            && self.building_is_city(b)
            && health_level(bd.hits_now(), bd.damage) > 5
    }

    /// `Build::check_capture(B, o, who)` with an enemy unit as the would-be
    /// captor. Returns whether ownership changed. `docs/CITIES.md` §7.2.
    pub fn check_capture(&mut self, b: usize, unit: usize) -> bool {
        if !self.capture_eligible(b) {
            return false;
        }
        if !self.buildings[b].garrison.is_empty() {
            self.buildings[b].eject_pending = true;
            return false;
        }
        let owner = self.buildings[b].owner;
        let u = &self.units[unit];
        let a = u.owner;
        if (a == owner || self.is_ally(owner, a)) && !self.defeated[owner as usize] {
            return false;
        }
        if self.defeated[a as usize] {
            return false;
        }
        if !matches!(u.kind.domain, crate::attrition::Domain::Land) || !u.alive() {
            return false;
        }
        if self.attack_of(Obj::Unit(unit)) == 0 {
            return false;
        }
        let Some(c) = self.buildings[b].city else {
            return false;
        };
        if self.frame - self.cities[c].capture_stamp <= 74 {
            return false;
        }
        let radius = self.tuning.city_capture_radius;
        let dist_max = radius * UNITS_PER_TILE;
        let bpos = self.buildings[b].pos;
        let land = self.world.region_of(bpos.cell());
        let def_base = if self.frame - self.cities[c].capture_stamp < 900 {
            self.cities[c].capture_strength
        } else {
            2
        };
        let mine = self.unit_capture_value(unit);
        let mut per_player = vec![0i32; self.players.len()];
        per_player[a as usize] = mine;
        per_player[owner as usize] += def_base;
        let mut attackers = mine;
        let mut defenders = def_base;
        let side = |s: &Sim, p: Player, v: i32, attackers: &mut i32, defenders: &mut i32| {
            if p == a || (s.is_ally(a, p) && s.is_enemy(owner, p)) {
                *attackers += v;
            } else if p == owner || (s.is_ally(owner, p) && s.is_enemy(a, p)) {
                *defenders += v;
            }
        };
        for (i, x) in self.units.iter().enumerate() {
            if i == unit || !x.alive() || !x.on_map {
                continue;
            }
            // Gaia is out of the tally (`world::PLAYER_SLOTS`); see
            // `docs/CITIES.md` §7.2 for what the original's own tally does
            // instead, which is *not* a leader bound.
            if x.is_gaia() {
                continue;
            }
            if !matches!(x.kind.domain, crate::attrition::Domain::Land) {
                continue;
            }
            if self.world.region_of(x.pos.cell()) != land {
                continue;
            }
            if vector_dist(x.pos.x - bpos.x, x.pos.y - bpos.y) > dist_max {
                continue;
            }
            let v = self.unit_capture_value(i);
            per_player[x.owner as usize] += v;
            side(self, x.owner, v, &mut attackers, &mut defenders);
        }
        for (i, x) in self.buildings.iter().enumerate() {
            if i == b || !x.alive {
                continue;
            }
            if self.world.region_of(x.pos.cell()) != land {
                continue;
            }
            if vector_dist(x.pos.x - bpos.x, x.pos.y - bpos.y) > dist_max {
                continue;
            }
            let mut v = 1;
            if x.owner == owner {
                let fort = x.ty.is_some_and(|t| build::is_fort(&self.build_types, t));
                v += x.garrison.len() as i32 + if fort { 12 } else { 6 };
            }
            per_player[x.owner as usize] += v;
            side(self, x.owner, v, &mut attackers, &mut defenders);
        }
        if defenders >= attackers {
            return false;
        }
        // The captor: the allied player with the most in the radius.
        let mut taker = a;
        let mut best = 0;
        for p in 0..self.players.len() {
            let p = p as Player;
            if self.defeated[p as usize] || !(p == a || self.is_ally(p, a)) {
                continue;
            }
            if per_player[p as usize] > best {
                best = per_player[p as usize];
                taker = p;
            }
        }
        let founder = self.cities[c].founder;
        let race = self.cities[c].race;
        if founder != owner
            && founder != taker
            && self.is_ally(taker, founder)
            && !self.defeated[founder as usize]
        {
            taker = founder;
        } else if let Some(r) = race
            && r != owner
            && r != taker
            && self.is_ally(taker, r)
            && !self.defeated[r as usize]
        {
            taker = r;
        }
        if let Some(newcity) = self.capture_city(taker, c, owner) {
            self.cities[newcity].capture_strength = mine;
        }
        true
    }

    /// `UnitData::get_capture_value` for one figure.
    fn unit_capture_value(&self, i: usize) -> i32 {
        let u = &self.units[i];
        let siege = u.kind.siege;
        capture_value(siege, u.squad_size, 1)
    }

    /// `Cities::capture_city(A, c, O)`: the hand-over. Returns the new city's
    /// index. `docs/CITIES.md` §7.3.
    pub fn capture_city(&mut self, a: Player, c: usize, o: Player) -> Option<usize> {
        if !self.cities[c].alive {
            return None;
        }
        let old_building = self.cities[c].building;
        let was_capital = self.cities[c].capital;
        self.city_tally[o as usize].lost += 1;
        self.city_tally[a as usize].captured += 1;
        let mut base;
        let mut to_close: Vec<usize> = vec![old_building];
        let newcity = match self.swap_team(old_building, a) {
            None => {
                base = 0;
                for m in self.cities[c].members.clone() {
                    if self.buildings[m]
                        .ty
                        .is_some_and(|t| !self.build_types[t].has(flags::NO_CITY))
                    {
                        to_close.push(m);
                        base += 25;
                    }
                }
                None
            }
            Some(n) => {
                self.activate(n, true, false);
                let newcity = self.buildings[n]
                    .city
                    .expect("a city building activates with a record");
                base = self.tuning.city_plunder_per_level * (self.city_level_of(newcity) - 1);
                self.city_capture_record(newcity, c, a, n);
                self.armies_update_city(Obj::Building(old_building), Obj::Building(n), a);
                for m in self.cities[c].members.clone() {
                    let Some(t) = self.buildings[m].ty else {
                        continue;
                    };
                    if self.build_types[t].has(flags::NOT_CIVILIAN) {
                        continue;
                    }
                    to_close.push(m);
                    let farmish = matches!(self.build_types[t].ident, Ident::Farm | Ident::Granary);
                    if farmish && self.nation[a as usize].lakota {
                        continue;
                    }
                    if self.buildings[m].active
                        && let Some(nm) = self.swap_team(m, a)
                    {
                        self.activate(nm, true, false);
                        base += 25;
                    }
                }
                self.find_buildings(newcity);
                Some(newcity)
            }
        };
        self.plunder_on_capture(a, o, c, base, was_capital);
        for b in to_close {
            if self.buildings[b].alive {
                if !self.buildings[b].active {
                    self.disband_building(b, false);
                } else {
                    self.close_building(b, true);
                }
            }
        }
        if was_capital && self.city_tally[o as usize].lost_capital_frame.is_none() {
            self.city_tally[o as usize].lost_capital_frame = Some(self.frame);
        }
        if let Some(nc) = newcity
            && self.cities[nc].capital
        {
            self.city_tally[a as usize].lost_capital_frame = None;
        }
        // The old record.
        if self.cities[c].alive {
            self.cities[c].members.clear();
            self.close_city(c, Some(a));
        }
        if let Some(nc) = newcity {
            let nb = self.cities[nc].building;
            self.update_hits(nb);
            let bd = &mut self.buildings[nb];
            bd.damage = bd.hits_now() - 10;
            bd.sync_health();
            self.sync_pop_cities();
            self.sync_territory();
        }
        newcity
    }

    /// `City::capture(new, old, c, A, o)`: the record hand-over.
    fn city_capture_record(&mut self, newcity: usize, old: usize, a: Player, n: usize) {
        let o = self.cities[old].owner;
        let oldc = self.cities[old].clone();
        let friendly = o == a || self.is_ally(a, o);
        let assimilate_now = friendly || self.nation[a as usize].global_government;
        let nc = &mut self.cities[newcity];
        nc.owner = a;
        nc.building = n;
        nc.attack_stamp = self.frame;
        nc.reg = oldc.reg;
        nc.founder = oldc.founder;
        nc.race = if assimilate_now { Some(a) } else { oldc.race };
        nc.was_capital = oldc.was_capital;
        nc.pop = oldc.pop;
        nc.unassimilated = !friendly;
        nc.no_heal = oldc.no_heal;
        if oldc.capital {
            nc.was_capital |= 1 << o;
            if oldc.founding_capital {
                nc.was_founding_capital = true;
            }
        }
        nc.capital = false;
        nc.founding_capital = false;
        if nc.was_capital & (1 << a) != 0 {
            nc.capital = true;
            nc.was_capital &= !(1 << a);
            if nc.was_founding_capital && a == nc.founder {
                nc.founding_capital = true;
                nc.was_founding_capital = false;
            }
            nc.race = Some(a);
        }
        if nc.race == Some(a) {
            nc.unassimilated = false;
        }
    }

    /// `Build::swap_team(B, A)` → `Wall::swap_team`: the garrison out, the
    /// queue cleared, a copy of the building under the new owner with the
    /// progress and damage carried over. Returns the new index; the old one
    /// is the caller's to close.
    pub fn swap_team(&mut self, b: usize, a: Player) -> Option<usize> {
        let ty = self.buildings[b].ty?;
        if !self.buildings[b].garrison.is_empty() {
            self.eject_contents(b, true);
        }
        if !self.buildings[b].is_library {
            let who = self.buildings[b].owner;
            let mut ledger = std::mem::take(&mut self.ledgers[who as usize]);
            while !self.buildings[b].queue.items.is_empty() {
                if let Some(item) = self.buildings[b].queue.unqueue(0, true, &mut ledger) {
                    self.muster[who as usize].queued_by_type[item.ty] -= 1;
                    self.track_tree_queued(who, item.ty, -1);
                }
            }
            self.ledgers[who as usize] = ledger;
        }
        let pos = self.buildings[b].pos;
        let n = self.init_build(a, ty, pos, true);
        let old = self.buildings[b].clone();
        let bd = &mut self.buildings[n];
        bd.started = old.started;
        bd.active = old.active;
        bd.activated = old.activated;
        bd.under_attack = old.under_attack;
        bd.job_counter = old.job_counter;
        bd.job_counter_2 = old.job_counter_2;
        bd.constr_time = old.constr_time;
        bd.construct_hits = old.construct_hits;
        bd.damage = old.damage;
        bd.damage_frac = old.damage_frac;
        bd.founder = old.founder;
        bd.recharging = old.recharging;
        bd.clock = old.clock;
        self.update_hits(n);
        Some(n)
    }

    // ------------------------------------------------------------------
    // Assimilation, plunder
    // ------------------------------------------------------------------

    /// The assimilation tick of `Build::process` for an unassimilated city.
    fn assimilation_tick(&mut self, c: usize, frame: i64) {
        let owner = self.cities[c].owner;
        if self.cities[c].race == Some(owner) {
            return;
        }
        if self.cities[c].has_citizen {
            self.cities[c].assimilation_timer +=
                1 - i64::from(self.tuning.thecitizen_assimilation_speed);
        }
        let mut e = frame - self.cities[c].assimilation_timer;
        if self.nation[owner as usize].turks {
            e = i64::from(self.tuning.turk_assimilate + 100) * e / 100;
        }
        if self.cities[c].founder == owner {
            e = i64::from(self.tuning.reassimilation + 100) * e / 100;
        }
        if e >= i64::from(self.tuning.assimilation_timer) {
            self.assimilate(c);
        }
    }

    /// `City::assimilate`.
    pub fn assimilate(&mut self, c: usize) {
        let owner = self.cities[c].owner;
        self.cities[c].unassimilated = false;
        if self.cities[c].race != Some(owner) {
            self.cities[c].race = Some(owner);
            if self.nation[owner as usize].chinese
                && self.tuning.chinese_large_cities != 0
                && let Some(town) = self.build_types.iter().position(|t| t.ident == Ident::Town)
                && self.building_ident(self.cities[c].building) == Ident::Village
            {
                // Without a `mask_me`: the original leaves the radius mask
                // at the old size until the next remask.
                let b = self.cities[c].building;
                self.set_type(b, town);
            }
            self.sync_territory();
        }
    }

    /// The plunder of `Cities::capture_city` — `docs/CITIES.md` §8.3.
    fn plunder_on_capture(&mut self, a: Player, o: Player, c: usize, base: i32, was_capital: bool) {
        if base == 0 && !was_capital {
            return;
        }
        let old = &self.cities[c];
        if old.unassimilated || (old.capture_stamp != 0 && self.frame - old.capture_stamp < 0x1195)
        {
            return;
        }
        let hero = self.nation[a as usize].plunder_hero;
        let russian = self.nation[o as usize].russians && self.tuning.russian_plunder_steal != 0;
        let first_capital = was_capital && !self.city_tally[o as usize].capital_plundered;
        let usable: Vec<usize> = (0..5)
            .filter(|g| {
                self.holdings[a as usize].available[*g] && self.holdings[o as usize].available[*g]
            })
            .collect();
        if !first_capital {
            let (take, steal) = if !russian {
                (
                    if hero {
                        self.tuning.thedespot_plunder * base / 100
                    } else {
                        base
                    },
                    0,
                )
            } else {
                (if hero { base } else { 0 }, base)
            };
            if take > 0
                && let Some(&g) = usable
                    .iter()
                    .min_by_key(|g| self.ledgers[a as usize].bucket[**g])
            {
                self.ledgers[a as usize].bucket[g] += take;
            }
            if steal > 0
                && let Some(&g) = usable
                    .iter()
                    .min_by_key(|g| self.ledgers[o as usize].bucket[**g])
            {
                self.ledgers[o as usize].bucket[g] += steal;
            }
        } else {
            self.city_tally[o as usize].capital_plundered = true;
            let base = base.max(self.tuning.capital_plunder);
            let (take, steal) = if !russian {
                (
                    if hero {
                        self.tuning.thedespot_plunder * base / 100
                    } else {
                        base
                    },
                    0,
                )
            } else {
                (if hero { base } else { 0 }, base)
            };
            for &g in &usable {
                self.ledgers[a as usize].bucket[g] += take;
                self.ledgers[o as usize].bucket[g] += steal;
            }
        }
    }

    /// `Build::plunder(B, o, who)` on a building's death by `killer`.
    pub(crate) fn plunder_kill(&mut self, b: usize, killer: Player) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let owner = self.buildings[b].owner;
        let no = &self.nation[owner as usize];
        if killer == owner && no.lakota && self.tuning.lakota_raze_price != 0 {
            return;
        }
        let bt = &self.build_types[ty];
        let mut v = bt.plunder_value;
        let Some(g) = bt.plunder_good else { return };
        if v == 0 || !self.holdings[killer as usize].available[g] {
            return;
        }
        if no.germans && bt.ident == Ident::Mine {
            v = (100 - self.tuning.german_mine_cost) * v / 100;
            if v == 0 {
                return;
            }
        }
        let bd = &self.buildings[b];
        if killer == owner {
            if !bd.activated {
                return;
            }
            if bt.ident == Ident::Temple && no.tikal {
                v = (100 - self.tuning.tikal_temple_cost) * v / 100;
            }
        }
        if !bd.activated {
            let ct = self.construct_time_of(b).max(1);
            let jc2 = bd.job_counter_2.min(ct);
            v = ((i64::from(v) * i64::from(jc2)) / i64::from(ct)) as i32;
        } else if killer == owner {
            let foreign = bd
                .city
                .is_some_and(|c| self.cities[c].alive && self.cities[c].race != Some(owner));
            if foreign {
                v = self.tuning.plunder * v / 100;
            }
        } else {
            v = self.tuning.plunder * v / 100;
        }
        if v == 0 {
            return;
        }
        if self.tuning.aztec_plunder != 0 && killer != owner && self.nation[killer as usize].aztecs
        {
            v = (self.tuning.aztec_plunder + 100) * v / 100;
        }
        let hero = if killer != owner && self.nation[killer as usize].plunder_hero {
            self.tuning.thedespot_plunder * v / 100
        } else {
            0
        };
        if no.russians && self.tuning.russian_plunder_steal != 0 && killer != owner {
            self.ledgers[owner as usize].bucket[g] += v;
            self.ledgers[killer as usize].bucket[g] += hero;
        } else {
            self.ledgers[killer as usize].bucket[g] += v + hero;
        }
    }
}
