//! **`Build::activate@00623e20`**, by slices (item 3364; `docs/CITIES.md` §4).
//!
//! The function is two thousand lines: the head, up to its call to
//! `Wall::activate`, is [`activate_head_steps`] over an [`ActivateHeadView`]
//! that names every access it makes in the order it makes them. [`SimHead`]
//! answers from this crate's state; the sweep
//! (`crates/sim/src/sweep/build_activate_head.rs`) answers from the emulated
//! original's windows and ends the call where `Wall::activate` is first
//! entered (`--stop-stub`).
//!
//! **Not established.** `Build::new_library` has no counterpart here (the
//! crate reads a library's queue from the first library where it is used,
//! `docs/CITIES.md` §4); the per-leader `last_build_built` tables the head
//! writes (`leaders +0x59f8`, `ScenarioData::last_build_built_o`) are read by
//! nothing in this crate; `Wall::mask_me(1)` on the converted city is the
//! city arm's own `mask_city`, later.

use crate::build::Ident;
use crate::{Player, Sim};

/// What `Build::activate`'s head reads and does.
pub trait ActivateHeadView {
    /// The building's `is(id, 0)` (`0x1b3` a Library, `0x213` the Forbidden
    /// City).
    fn is_type(&mut self, id: i32) -> bool;
    /// `Build::new_library`.
    fn new_library(&mut self);
    /// `+0x7a = 0`: the `recharging` count a site collected while built.
    fn clear_recharging(&mut self);
    /// The building's `flags & 0x20`: a city.
    fn city_flag(&mut self) -> bool;
    /// The leader's `+0x3f8` (its city count) moved by `d`.
    fn cities_add(&mut self, d: i32);
    /// `LeaderData::has_tribe_bonus(bonus)` of the owner.
    fn tribe_bonus(&mut self, bonus: i32) -> bool;
    /// `GameAccess::constants->chinese_large_cities != 0`.
    fn chinese_large_cities(&mut self) -> bool;
    /// The building's slot `+0x84(id, 0)`: its type becomes `id`.
    fn set_type(&mut self, id: i32);
    /// `Wall::mask_me(1, REGEN_NONE)`.
    fn mask_me(&mut self);
    /// `leader_flags |= 0x8000000`.
    fn flag_wall_stats(&mut self);
    /// The two `last_build_built` writes: `leaders[who] +0x59f8[type idx]`
    /// and `ScenarioData::last_build_built_o[who]`, the building's `+0xa`.
    fn record_last_built(&mut self);
    /// `Wall::activate(captured, announce, 0)`: where the slice ends.
    fn wall_activate(&mut self, captured: i32, announce: i32);
}

/// `Build::activate@00623e20`, from its entry to its call of
/// `Wall::activate` (lines 121–159).
pub fn activate_head_steps<V: ActivateHeadView>(
    v: &mut V,
    captured: i32,
    announce: i32,
    counted: i32,
) {
    if v.is_type(0x1b3) {
        v.new_library();
    }
    v.clear_recharging();
    if v.city_flag() {
        v.cities_add(1);
        // Chinese cities are founded as Large Cities (a captured one keeps
        // its level), unless it is the Forbidden City.
        if v.tribe_bonus(0xe) && captured == 0 && v.chinese_large_cities() && !v.is_type(0x213) {
            v.set_type(0x19f);
            v.mask_me();
            v.flag_wall_stats();
        }
        v.cities_add(-1);
    }
    if captured == 0 && counted != 0 {
        v.record_last_built();
    }
    v.wall_activate(captured, announce);
}

/// [`ActivateHeadView`] over a [`Sim`]. The head's `Wall::activate` is the
/// rest of [`Sim::activate`], so [`ActivateHeadView::wall_activate`] does
/// nothing here.
pub(crate) struct SimHead<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) b: usize,
    pub(crate) who: Player,
}

impl ActivateHeadView for SimHead<'_> {
    fn is_type(&mut self, id: i32) -> bool {
        let ident = match id {
            0x1b3 => Ident::Library,
            0x213 => Ident::ForbiddenCity,
            _ => unreachable!("the head asks for 0x1b3 and 0x213 only"),
        };
        self.sim.building_is(self.b, ident)
    }
    /// SEAM: see the module's "not established".
    fn new_library(&mut self) {}
    /// `Sim::activate` clears it with the rest of the economy's reset
    /// ahead of the head (the order is unobserved: nothing between reads it).
    fn clear_recharging(&mut self) {
        self.sim.buildings[self.b].recharging = 0;
    }
    fn city_flag(&mut self) -> bool {
        self.sim.building_is_city(self.b)
    }
    /// SEAM: the count is bumped and dropped again around the conversion;
    /// nothing between reads it here (`mask_me` is the city arm's).
    fn cities_add(&mut self, _d: i32) {}
    fn tribe_bonus(&mut self, bonus: i32) -> bool {
        debug_assert_eq!(bonus, 0xe, "the head asks the Chinese bonus only");
        self.sim.nation[self.who as usize].chinese
    }
    fn chinese_large_cities(&mut self) -> bool {
        self.sim.tuning.chinese_large_cities != 0
    }
    fn set_type(&mut self, id: i32) {
        debug_assert_eq!(id, 0x19f, "the head converts to a Town");
        if let Some(town) = self
            .sim
            .build_types
            .iter()
            .position(|t| t.ident == Ident::Town)
        {
            let b = self.b;
            self.sim.buildings[b].ty = Some(town);
            self.sim.buildings[b].combat =
                Some(self.sim.build_types[town].combat.unwrap_or_default());
        }
    }
    /// SEAM: the city arm's `mask_city` masks the converted centre.
    fn mask_me(&mut self) {}
    fn flag_wall_stats(&mut self) {
        self.sim.wall_stats_dirty[self.who as usize] = true;
    }
    /// SEAM: nothing here reads the tables.
    fn record_last_built(&mut self) {}
    fn wall_activate(&mut self, _captured: i32, _announce: i32) {}
}

/// What `Build::activate`'s non-city arm reads and does (lines 160–445 and
/// `LAB_00624564`): the dispatch on the type's lineage after `Wall::activate`,
/// and the Senate's capital move.
pub trait ActivateArmsView: ActivateHeadView {
    /// The building's `+0x72` short: its city's index, negative for none.
    fn city_index(&mut self) -> i32;
    /// `cities[owner][city].city_flags |= mask`.
    fn city_flags_or(&mut self, mask: u16);
    /// A Granary's `CityData::granary_level` and the byte at city `+0x56`
    /// taken from `constantsc`'s table.
    fn granary_bonus(&mut self);
    /// A Lumbermill's `lumber_level` and the byte at city `+0x57`.
    fn lumber_bonus(&mut self);
    /// The owner's `leader_flags |= mask`.
    fn leader_flags_or(&mut self, mask: u32);
    /// The region under the building's tile, `Region::fix_borders` when its
    /// `+4` short is not negative.
    fn fix_borders(&mut self);
    /// `LeaderData +0xa48 gov_hero_frame` is not negative.
    fn gov_hero_frame_set(&mut self) -> bool;
    /// `ObjectsData::find_unit(…FILTER_GOV_HERO…)` finds a patriot.
    fn gov_hero_near(&mut self) -> bool;
    /// `gov_hero_frame <= game->frame`.
    fn gov_hero_due(&mut self) -> bool;
    /// `LeaderData::get_gov_hero`: the type, negative for none.
    fn get_gov_hero(&mut self) -> i32;
    /// `Build::train(ty)` on the activating building.
    fn train(&mut self, ty: i32);
    /// `LeaderData +0x85c` (senates built) `+= 1`.
    fn senates_built_inc(&mut self);
    /// The building's city's `+0x5f` (race) byte is the owner.
    fn city_race_is_owner(&mut self) -> bool;
    /// `LeaderData::find_capital(-1, -1)`: the leader's city index of a
    /// capital the leader holds, if the answer names the owner.
    fn find_capital(&mut self) -> Option<i32>;
    /// `CityData::count_buildings(0x1b6, 0, 1)` of the leader's city `i`.
    fn count_senates(&mut self, city: i32) -> i32;
    /// `LeaderData +0x408`: the leader's city count.
    fn own_cities(&mut self) -> i32;
    /// The `city_flags` short of the leader's city `i`.
    fn own_city_flags(&mut self, i: i32) -> u16;
    /// … written.
    fn set_own_city_flags(&mut self, i: i32, f: u16);
    /// The building's city: `city_flags |= 0x10`, `+0x60 = owner`.
    fn city_make_capital(&mut self);
    /// The console's sound and message (`announce` set).
    fn senate_announce(&mut self);
    /// `City::regen_roads` of the building's city.
    fn regen_roads(&mut self);
    /// The type's slot `+0xfc` (`is_fort`): the next call after the arm.
    fn fort_probe(&mut self);
}

/// `Build::activate@00623e20`, from its second look at the city flag to the
/// type's `is_fort` (lines 160–445 and 551–556): which arm the type's
/// lineage takes, and the Senate's. The city arm is not here.
pub fn activate_arms_steps<V: ActivateArmsView>(v: &mut V, captured: i32, announce: i32) {
    if v.city_flag() {
        return;
    }
    if v.is_type(0x1a7) {
        // The original ORs the flag and then tests it: always set.
        if v.city_index() >= 0 {
            v.city_flags_or(0x200);
            v.granary_bonus();
            v.leader_flags_or(0x200_0000);
        }
    } else if v.is_type(0x1a8) {
        v.leader_flags_or(0x200_0000);
        if v.city_index() >= 0 {
            v.city_flags_or(0x400);
            v.lumber_bonus();
        }
    } else if v.is_type(0x1a9) || v.is_type(0x1aa) {
        v.leader_flags_or(0x200_0000);
    } else if v.is_type(0x1b5) {
        if v.city_index() >= 0 {
            v.fix_borders();
            v.city_flags_or(0x80);
        }
    } else if v.is_type(0x1b4) {
        if v.city_index() >= 0 {
            v.city_flags_or(0x800);
        }
    } else if v.is_type(0x1b6) {
        senate_arm_steps(v, captured, announce);
    }
    if v.city_index() >= 0 {
        v.regen_roads();
    }
    v.fort_probe();
}

/// **`Build::activate@00623e20`'s Senate arm** (lines 209–373,
/// `docs/CITIES.md` §15): a Senate finished, not captured, in a live city of
/// the owner's own race **moves the capital here** when the owner's capital
/// (`LeaderData::find_capital`'s first arm, the first live own city with
/// `0x10`) holds no finished Senate — `CityData::count_buildings(SENATE, 0,
/// 1)`. Every own live city loses `0x10` (with tribe bonus `0x17`, only one
/// that is not the founding capital, `0x4000`), this one takes it and
/// `founder = who`, and `Region::fix_borders` runs, because a capital
/// projects `CAPITAL_TERRITORY_BONUS`. Drawless: Great Lakes' Senate `1/2024`
/// moves the capital from London to Norwich on tick 14528 and spends
/// nothing on it.
///
/// The arm's head trains the government hero when `gov_hero_frame` is set,
/// none is near and the frame has come (item 3364: [`SimArms`] runs it; the
/// crate had left it a SEAM).
pub fn senate_arm_steps<V: ActivateArmsView>(v: &mut V, captured: i32, announce: i32) {
    if v.gov_hero_frame_set() && captured == 0 && !v.gov_hero_near() && v.gov_hero_due() {
        let hero = v.get_gov_hero();
        if hero >= 0 {
            v.train(hero);
        }
    }
    if v.city_index() < 0 || captured != 0 {
        return;
    }
    v.senates_built_inc();
    if !v.city_race_is_owner() {
        return;
    }
    let Some(cap) = v.find_capital() else {
        return;
    };
    if v.count_senates(cap) != 0 {
        return;
    }
    for i in 0..v.own_cities() {
        let f = v.own_city_flags(i);
        if f & 1 != 0 {
            let keeps = v.tribe_bonus(0x17);
            let clear = if keeps {
                f & 0x4010 == 0x10
            } else {
                f & 0x10 != 0
            };
            if clear {
                v.set_own_city_flags(i, f & 0xffef);
            }
        }
    }
    v.city_make_capital();
    v.fix_borders();
    if announce != 0 {
        v.senate_announce();
    }
}

/// [`ActivateArmsView`] over a [`Sim`]. The leader's cities are the crate's
/// `cities` whose owner is the building's, in order.
pub(crate) struct SimArms<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) b: usize,
    pub(crate) who: Player,
    own: Vec<usize>,
}

impl<'a> SimArms<'a> {
    pub(crate) fn new(sim: &'a mut Sim, b: usize, who: Player) -> Self {
        let own = (0..sim.cities.len())
            .filter(|&k| sim.cities[k].owner == who)
            .collect();
        SimArms { sim, b, who, own }
    }
    fn city(&self) -> Option<usize> {
        self.sim.buildings[self.b]
            .city
            .filter(|&c| self.sim.cities[c].alive)
    }
}

impl ActivateHeadView for SimArms<'_> {
    fn is_type(&mut self, id: i32) -> bool {
        let ident = match id {
            0x1a7 => Ident::Granary,
            0x1a8 => Ident::Lumbermill,
            0x1a9 => Ident::Smelter,
            0x1aa => Ident::Refinery,
            0x1b4 => Ident::Market,
            0x1b5 => Ident::Temple,
            0x1b6 => Ident::Senate,
            _ => unreachable!("the arms ask for the seven lineages only"),
        };
        self.sim.building_is(self.b, ident)
    }
    fn new_library(&mut self) {}
    fn clear_recharging(&mut self) {}
    fn city_flag(&mut self) -> bool {
        self.sim.building_is_city(self.b)
    }
    fn cities_add(&mut self, _d: i32) {}
    fn tribe_bonus(&mut self, bonus: i32) -> bool {
        debug_assert_eq!(bonus, 0x17, "the arms ask the Persian bonus only");
        self.sim
            .tech_tree
            .has_tribe_bonus(&self.sim.setup, &self.sim.tech[self.who as usize], 0x17)
    }
    fn chinese_large_cities(&mut self) -> bool {
        false
    }
    fn set_type(&mut self, _id: i32) {}
    fn mask_me(&mut self) {}
    fn flag_wall_stats(&mut self) {}
    fn record_last_built(&mut self) {}
    fn wall_activate(&mut self, _captured: i32, _announce: i32) {}
}

impl ActivateArmsView for SimArms<'_> {
    fn city_index(&mut self) -> i32 {
        self.city().map_or(-1, |c| c as i32)
    }
    /// SEAM: the `0x80`/`0x200`/`0x400`/`0x800` bits are derived from the
    /// city's buildings (`Sim::city_has`), not carried.
    fn city_flags_or(&mut self, _mask: u16) {}
    /// SEAM: the granary and lumber bonus bytes (`+0x56`, `+0x57`) are not
    /// carried.
    fn granary_bonus(&mut self) {}
    fn lumber_bonus(&mut self) {}
    /// `0x2000000` is the economy's dirty flag: `Sim::activate` raises it
    /// for every type at its head.
    fn leader_flags_or(&mut self, _mask: u32) {}
    fn fix_borders(&mut self) {
        // SEAM: the region's border count is not carried; the territory
        // pass rebuilds it whole.
        self.sim.sync_territory();
    }
    fn gov_hero_frame_set(&mut self) -> bool {
        self.sim.tech[self.who as usize].gov_hero_frame >= 0
    }
    fn gov_hero_near(&mut self) -> bool {
        self.sim.gov_hero_exists(self.who)
    }
    fn gov_hero_due(&mut self) -> bool {
        self.sim.tech[self.who as usize].gov_hero_frame <= self.sim.frame
    }
    fn get_gov_hero(&mut self) -> i32 {
        let who = self.who as usize;
        self.sim
            .tech_tree
            .get_gov_hero(&self.sim.setup, &self.sim.tech[who])
            .and_then(|h| self.sim.unit_record(h))
            .map_or(-1, |r| r as i32)
    }
    fn train(&mut self, ty: i32) {
        self.sim.build_train(self.b, ty as usize);
    }
    /// SEAM: no reader here carries `senates_built`.
    fn senates_built_inc(&mut self) {}
    fn city_race_is_owner(&mut self) -> bool {
        self.city()
            .is_some_and(|c| self.sim.cities[c].race == Some(self.who))
    }
    /// SEAM: the crate's search is the leader's first live capital; the
    /// original's `find_capital(-1, -1)` also reads another leader's
    /// former capitals (`docs/AI.md` §149).
    fn find_capital(&mut self) -> Option<i32> {
        self.own
            .iter()
            .position(|&k| self.sim.cities[k].alive && self.sim.cities[k].capital)
            .map(|i| i as i32)
    }
    fn count_senates(&mut self, city: i32) -> i32 {
        self.sim
            .count_buildings(self.own[city as usize], Ident::Senate, true)
    }
    fn own_cities(&mut self) -> i32 {
        self.own.len() as i32
    }
    fn own_city_flags(&mut self, i: i32) -> u16 {
        let c = &self.sim.cities[self.own[i as usize]];
        u16::from(c.alive) | u16::from(c.capital) << 4 | u16::from(c.founding_capital) << 14
    }
    fn set_own_city_flags(&mut self, i: i32, f: u16) {
        let k = self.own[i as usize];
        self.sim.cities[k].capital = f & 0x10 != 0;
    }
    fn city_make_capital(&mut self) {
        if let Some(c) = self.city() {
            self.sim.cities[c].capital = true;
            self.sim.cities[c].founder = self.who;
        }
    }
    /// The console is the renderer's; the sim has none.
    fn senate_announce(&mut self) {}
    /// SEAM: `Sim::activate` flags the city's roads once, after both arms.
    fn regen_roads(&mut self) {}
    fn fort_probe(&mut self) {}
}
