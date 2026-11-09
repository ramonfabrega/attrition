//! The walk views of `docs/CITIES.md`'s city calls and their steps (item 4031's cut from `city.rs`):
//! `Build::start` and `Wall::start`'s footprint walk, `City::find_buildings`, `City::check_upgrade`,
//! `Build::new_library` and `City::assimilate`. Each is a trait the original's calls go through, the
//! steps over it, and the view over a [`Sim`] that `city.rs`'s `impl Sim` runs them with; the sweeps
//! run the same steps over the emulated original's windows.

use crate::cities::build::{Ident, flags};
use crate::world::{Pos, vector_dist};
use crate::{Player, Sim};

/// What `Build::start@006273a0` reads and does, one method a call it makes.
pub(crate) trait BuildStartView {
    /// `Wall::start`; true when it answered nonzero (the site had started already).
    fn wall_start(&mut self) -> bool;
    /// The wonder test: `Build::is_wonder` — the type's index in `0x20e..=0x21e` on the stock vtables,
    /// the vslot `+0x2c` / the type's `+0x1c` otherwise.
    fn is_wonder(&mut self) -> bool;
    /// Vslot `+0x164`, `Wall::update_local_seen`.
    fn update_local_seen(&mut self);
    /// The begun-a-wonder notice and `S_WONDER_STARTED`: interface only (the sound, `play(0x78)`).
    fn announce(&mut self);
}

/// `Build::start@006273a0`. `announce` is its argument: the begun-a-wonder notice, interface only. Answers
/// the original's `eax`: 1 when `Wall::start` did, else 0. A wonder lights its grown footprint at once (vslot
/// `+0x164`), `0xff` into `seen` and `seen2`, so every player has it in sight until the next clear — and
/// the owner's next `check_ever_seen` takes those bits, which is first contact for a player who never
/// looked (`docs/VISION.md` §6.4, item 1446).
pub(crate) fn build_start_steps(v: &mut impl BuildStartView, announce: bool) -> i32 {
    if v.wall_start() {
        return 1;
    }
    if !v.is_wonder() {
        return 0;
    }
    v.update_local_seen();
    if announce {
        v.announce();
    }
    0
}

pub(crate) struct StartSite<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) b: usize,
}

impl BuildStartView for StartSite<'_> {
    fn wall_start(&mut self) -> bool {
        self.sim.wall_start(self.b)
    }
    fn is_wonder(&mut self) -> bool {
        self.sim.buildings[self.b]
            .ty
            .is_some_and(|ty| self.sim.tables.build_types[ty].wonder)
    }
    fn update_local_seen(&mut self) {
        self.sim.update_local_seen_build(self.b);
    }
    fn announce(&mut self) {}
}

/// `Wall::start@0063e810`'s footprint walk (the loop between `mask_me` and
/// `check_ever_seen`): the half-cells `(tile.x >> 1, tile.y >> 1)` whose
/// owner bit is or-ed in, in the order the original visits them. The outer
/// loop runs the type's `+0x234` (x size), the inner its `+0x238` (y
/// size); a tile off the `w × h` grid (`world+0x18`, `+0x1c`) is skipped,
/// and a tile whose half-cell is the one just written is not written again.
pub(crate) fn wall_start_seen_steps(
    corner: (i32, i32),
    xs: i32,
    ys: i32,
    w: i32,
    h: i32,
) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let (mut last_x, mut last_y) = (-1, -1);
    for u in 0..xs {
        let x = corner.0.wrapping_add(u);
        for v in 0..ys {
            let y = corner.1.wrapping_add(v);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let (hx, hy) = (x >> 1, y >> 1);
            if hx != last_x || hy != last_y {
                out.push((hx, hy));
                last_x = hx;
                last_y = hy;
            }
        }
    }
    out
}

/// What `City::check_upgrade@00738b20` asks of the world, in the order it
/// asks it (item 3272): the sweep's emulated original answers the same
/// questions from its windows, [`Sim`] from its records.
pub trait CheckUpgradeView {
    /// `CityData::ready_to_upgrade`.
    fn ready_to_upgrade(&mut self) -> bool;
    /// `CityData::get_pop_value`: 1, 3 or 5 by the city building's level.
    fn pop_value(&mut self) -> i32;
    /// `CityData::upgrades_to`: the type the building becomes.
    fn upgrades_to(&mut self) -> usize;
    /// The building's `set_type` (slot `+0x80`, keeping its damage).
    fn set_type(&mut self, next: usize);
    /// `Wall::mask_me(1)` on the building.
    fn mask_city(&mut self);
    /// `leader_flags |= 0x8000000`.
    fn flag_wall_stats(&mut self);
    /// `find_buildings`.
    fn find_buildings(&mut self);
    /// The leader's population (`+0x95c`), the world's, the region's and
    /// `Leader::calc_pop_cap`, moved by the new value less `before`.
    fn repop(&mut self, before: i32);
    /// `Region::fix_borders`.
    fn fix_borders(&mut self);
    /// The local player's message and `S_CITY_UPGRADED`.
    fn announce(&mut self);
}

/// What `City::find_buildings@007384c0` reads and does, one question or call
/// a method (item 3304): the steps are [`find_buildings_steps`], the
/// emulated original's sweep holds them against the executable and
/// [`SimFind`] runs them over the simulation. The walk is the original's
/// `leaders × (two object ranges) × index`, which the view flattens to a
/// cursor; the crate walks its building slots.
pub trait FindBuildingsView {
    /// `LeaderData::get_radius(city type)`, before the `0x40` clamp.
    fn radius(&mut self) -> i32;
    /// The cursor goes to the next object; false at the end of the walk.
    fn next_object(&mut self) -> bool;
    /// The object's slot `+0x10`: a building at all.
    fn is_building(&mut self) -> bool;
    /// The object's `flags & 0x20`.
    fn flagged(&mut self) -> bool;
    /// Slot `+0xac`'s `Build`: its city (`+0x72`) is negative.
    fn cityless(&mut self) -> bool;
    /// The object's owner is the city's.
    fn same_owner(&mut self) -> bool;
    /// The type's `flags & 0x10` (`NO_CITY`).
    fn no_city(&mut self) -> bool;
    /// `vector_dist` of the tile offsets between the city and the object.
    fn dist(&mut self) -> i32;
    /// `is(0x1b3, 0)` of the `Build`: a Library by lineage.
    fn is_library(&mut self) -> bool;
    /// `Build::new_library`.
    fn new_library(&mut self);
    /// `Build::add_to_city(city)`.
    fn add_to_city(&mut self);
    /// `Build::swap_team(who)`: the converted copy's slot, or none.
    fn swap_team(&mut self) -> bool;
    /// A failed conversion: the old building's `die(0, -1, 0)`.
    fn die_old(&mut self);
    /// The console is the city's owner or the old owner.
    fn console_involved(&mut self) -> bool;
    /// The `MessageWin` line.
    fn announce(&mut self);
    /// The converted copy's data slot `+0x1a8(0, 1, 0)`.
    fn activate_new(&mut self);
    /// The old building's slot `+0x150(0, -1, 0)`.
    fn close_old(&mut self);
    /// `Wall::mask_me(1, 2)` on the converted copy.
    fn mask_new(&mut self);
    /// `City::check_upgrade`.
    fn check_upgrade(&mut self);
}

/// `City::find_buildings@007384c0`.
pub fn find_buildings_steps<V: FindBuildingsView>(v: &mut V) {
    let r = v.radius().min(0x40);
    while v.next_object() {
        if !v.is_building() || v.flagged() || !v.cityless() {
            continue;
        }
        let own = v.same_owner();
        if !own && v.no_city() {
            continue;
        }
        if v.dist() > r {
            continue;
        }
        if own {
            if v.is_library() {
                v.new_library();
            }
            v.add_to_city();
        } else {
            if !v.swap_team() {
                // A failed conversion kills the building and **ends the
                // sweep** — the rest waits for the next trigger.
                v.die_old();
                return;
            }
            if v.console_involved() {
                v.announce();
            }
            v.activate_new();
            v.close_old();
            v.mask_new();
        }
    }
    v.check_upgrade();
}

/// [`FindBuildingsView`] over a [`Sim`]: the building slots in order, as the
/// sweep started (a converted copy is appended and not visited).
pub(crate) struct SimFind<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) c: usize,
    pub(crate) who: Player,
    pub(crate) ct: Pos,
    pub(crate) n: usize,
    pub(crate) at: usize,
    pub(crate) swapped: Option<usize>,
}

impl FindBuildingsView for SimFind<'_> {
    fn radius(&mut self) -> i32 {
        self.sim.radius_of(self.c)
    }
    fn next_object(&mut self) -> bool {
        self.at = self.at.wrapping_add(1);
        self.at < self.n
    }
    fn is_building(&mut self) -> bool {
        self.sim.buildings[self.at].alive
    }
    /// SEAM: the record's `0x20` flag has no counterpart here.
    fn flagged(&mut self) -> bool {
        false
    }
    /// Also the crate's own reading of "needs a city": a city centre is
    /// never swept, and a building with no type has nothing to join.
    fn cityless(&mut self) -> bool {
        let bd = &self.sim.buildings[self.at];
        bd.city.is_none() && !self.sim.building_is_city(self.at) && bd.ty.is_some()
    }
    fn same_owner(&mut self) -> bool {
        self.sim.buildings[self.at].owner == self.who
    }
    fn no_city(&mut self) -> bool {
        let ty = self.sim.buildings[self.at]
            .ty
            .expect("cityless names a type");
        self.sim.tables.build_types[ty].has(flags::NO_CITY)
    }
    fn dist(&mut self) -> i32 {
        let bt = self.sim.buildings[self.at].pos.tile();
        vector_dist(self.ct.x - bt.x, self.ct.y - bt.y)
    }
    fn is_library(&mut self) -> bool {
        self.sim.building_is(self.at, Ident::Library)
    }
    /// SEAM: a library's queue is the owner's first library's
    /// (`get_first_library`), read where it is used and never copied.
    fn new_library(&mut self) {}
    fn add_to_city(&mut self) {
        self.sim.add_to_city(self.at, self.c);
    }
    fn swap_team(&mut self) -> bool {
        self.swapped = self.sim.swap_team(self.at, self.who);
        self.swapped.is_some()
    }
    fn die_old(&mut self) {
        self.sim.die_building(self.at);
    }
    /// The console is the renderer's; the sim has none.
    fn console_involved(&mut self) -> bool {
        false
    }
    fn announce(&mut self) {}
    fn activate_new(&mut self) {
        let n = self.swapped.expect("swap_team answered a slot");
        self.sim.activate(n, false, false);
    }
    fn close_old(&mut self) {
        // `City::find_buildings@007384c0:222` closes the old half with
        // reason 0, which flags its cell.
        self.sim.mark_closed_cell(self.at);
        self.sim.close_building(self.at, true);
    }
    /// SEAM: the copy's wall mask is whatever `activate` leaves.
    fn mask_new(&mut self) {}
    fn check_upgrade(&mut self) {
        self.sim.check_upgrade(self.c);
    }
}

/// `City::check_upgrade@00738b20`: when the city is ready, its building
/// becomes the next level, is re-masked, the leader's wall stats are marked,
/// the city re-sweeps its buildings, the population and borders follow and
/// the local player is told.
pub fn check_upgrade_steps(v: &mut impl CheckUpgradeView) {
    if !v.ready_to_upgrade() {
        return;
    }
    let before = v.pop_value();
    let next = v.upgrades_to();
    v.set_type(next);
    v.mask_city();
    v.flag_wall_stats();
    v.find_buildings();
    v.repop(before);
    v.fix_borders();
    v.announce();
}

pub(crate) struct SimUpgrade<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) c: usize,
    pub(crate) next: Option<usize>,
}

impl CheckUpgradeView for SimUpgrade<'_> {
    fn ready_to_upgrade(&mut self) -> bool {
        self.next = self.sim.ready_to_upgrade(self.c);
        self.next.is_some()
    }
    fn pop_value(&mut self) -> i32 {
        2 * self.sim.city_level_of(self.c) - 1
    }
    fn upgrades_to(&mut self) -> usize {
        self.next.expect("ready_to_upgrade answered a type")
    }
    fn set_type(&mut self, next: usize) {
        let b = self.sim.cities.cities[self.c].building;
        self.sim.set_type(b, next);
    }
    fn mask_city(&mut self) {
        self.sim.mask_city(self.c, true);
    }
    fn flag_wall_stats(&mut self) {
        // The Senate bonus and the new hits arrive through `calc_wall_stats`
        // on the next frame.
        let owner = self.sim.cities.cities[self.c].owner as usize;
        self.sim.cities.wall_stats_dirty[owner] = true;
    }
    fn find_buildings(&mut self) {
        self.sim.find_buildings(self.c);
    }
    fn repop(&mut self, _before: i32) {
        // A recount over the live cities is the original's running delta
        // (`sync_leader_pop`).
        self.sim.sync_pop_cities();
    }
    fn fix_borders(&mut self) {
        self.sim.sync_territory();
    }
    fn announce(&mut self) {
        // SEAM: the message and the sound are the renderer's; no sim state.
    }
}

/// What `City::assimilate@00738e90` asks of the world, one method a call it
/// makes, in the order it makes them (item 3295's sweep holds the order, the
/// arguments and the short circuits against the emulated original).
///
/// A member is the building the city's chain names (`+0x74` links from the
/// city building's own record); the original reads its data record through
/// the objects table's slot `+0xac` every time, and so does not hold one.
pub trait AssimilateView {
    /// `city_flags &= 0xfeff` — the unassimilated bit, cleared first, every call.
    fn clear_unassimilated(&mut self);
    /// `+0x5f != +0x5e`: the race the city carries is not yet the owner's.
    fn race_pending(&mut self) -> bool;
    /// `+0x5f = +0x5e`.
    fn adopt_race(&mut self);
    /// The region under the city centre holds a border count under `0x40`
    /// (`World +0x134`, 28 bytes a region, the short at `+4`).
    fn region_unfixed(&mut self) -> bool;
    /// `Region::fix_borders`.
    fn fix_borders(&mut self);
    /// `has_tribe_bonus(0xe)` and `CHINESE_LARGE_CITIES`.
    fn chinese_large_cities(&mut self) -> bool;
    /// The city record's slot `+0x84` handed `(0x19f, 0)`: a Village becomes a Town.
    fn town_up(&mut self);
    /// The owner's `leader_flags |= 0x2000000`.
    fn raise_economy_flag(&mut self);
    /// The head building's `+0x74` link: the first member, if any.
    fn first_member(&mut self) -> Option<i32>;
    /// A member's `+0x74` link.
    fn next_member(&mut self, m: i32) -> Option<i32>;
    /// A member's slot `+0x4c` (`Build` vtable: the data record's `flags & 4`).
    fn member_active(&mut self, m: i32) -> bool;
    /// `is(0x1b3, 0)`: a Library by lineage.
    fn member_is_library(&mut self, m: i32) -> bool;
    /// The data record's `flags` bit 4, written (`&= 0xfb` / `|= 4`).
    fn member_set_active(&mut self, m: i32, on: bool);
    /// `Build::new_library`.
    fn new_library(&mut self, m: i32);
    /// `is(0x1b6, 0)`: a Senate by lineage.
    fn member_is_senate(&mut self, m: i32) -> bool;
    /// `LeaderData +0xa48 gov_hero_frame`.
    fn gov_hero_frame(&mut self) -> i32;
    /// `ObjectsData::find_unit(x, y, SEARCH_FRIENDLY, who, -1, 0x20000,
    /// FILTER_GOV_HERO, …)` from the member's position: a government patriot
    /// of the owner's exists.
    fn gov_hero_found(&mut self, m: i32) -> bool;
    /// `gov_hero_frame <= game->frame`.
    fn hero_frame_reached(&mut self, since: i32) -> bool;
    /// `LeaderData::get_gov_hero(since)`: the patriot type, if the government names one.
    fn gov_hero_type(&mut self, since: i32) -> Option<i32>;
    /// `Build::train(ty)`.
    fn train(&mut self, m: i32, ty: i32);
    /// `MiscAccess::console->who` is the city's owner.
    fn console_is_owner(&mut self) -> bool;
    /// The sound and the `MessageWin` line.
    fn announce(&mut self);
}

/// What `Build::new_library@00627fe0` reads and writes: the research queue
/// (a count byte at `+0x82`, entries of 0x14 bytes behind `+0x8c`) of the
/// library it is handed and of the owner's first library.
pub trait NewLibraryView {
    /// `is(0x1b3, 0)`: a Library by lineage.
    fn is_library(&mut self) -> bool;
    /// `LeaderData::get_first_library`: the object index, or negative.
    fn first_library(&mut self) -> i32;
    /// The library's own object index (the short at `+0xa`).
    fn own_index(&mut self) -> i32;
    /// The first library's queued count (the byte at `+0x82`).
    fn source_queued(&mut self, first: i32) -> u8;
    /// Entry `i` of the first library's queue copied onto this one's.
    fn copy_entry(&mut self, first: i32, i: usize);
    /// This library's own count byte.
    fn set_queued(&mut self, n: u8);
    /// The first library's count byte cleared.
    fn clear_source(&mut self, first: i32);
}

/// `Build::new_library@00627fe0`: a library that sits earlier in the object
/// table than the owner's first one takes that one's research queue and leaves
/// it empty. (The crate keeps the queue on the first library and resolves
/// `queue_home` on read, so `SimAssimilate` does not call this; the sweep
/// holds the arithmetic.)
pub fn new_library_steps<V: NewLibraryView>(v: &mut V) {
    if !v.is_library() {
        return;
    }
    let first = v.first_library();
    if first < 0 || v.own_index() >= first {
        return;
    }
    if v.source_queued(first) == 0 {
        return;
    }
    let n = v.source_queued(first);
    v.set_queued(n);
    for i in 0..usize::from(n) {
        v.copy_entry(first, i);
    }
    v.clear_source(first);
}

/// `City::assimilate@00738e90`.
pub fn assimilate_steps<V: AssimilateView>(v: &mut V) {
    v.clear_unassimilated();
    if !v.race_pending() {
        return;
    }
    v.adopt_race();
    if v.region_unfixed() {
        v.fix_borders();
    }
    if v.chinese_large_cities() {
        v.town_up();
    }
    v.raise_economy_flag();
    let mut at = v.first_member();
    while let Some(m) = at {
        if v.member_active(m) && v.member_is_library(m) {
            v.member_set_active(m, false);
            v.new_library(m);
            v.member_set_active(m, true);
        }
        if v.member_active(m) && v.member_is_senate(m) {
            let since = v.gov_hero_frame();
            if since >= 0
                && !v.gov_hero_found(m)
                && v.hero_frame_reached(since)
                && let Some(ty) = v.gov_hero_type(since)
            {
                v.train(m, ty);
            }
        }
        at = v.next_member(m);
    }
    if v.console_is_owner() {
        v.announce();
    }
}

/// [`AssimilateView`] over a [`Sim`]: the chain is the city building and
/// its `members`, taken when the call starts.
pub(crate) struct SimAssimilate<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) c: usize,
    pub(crate) chain: Vec<usize>,
}

impl SimAssimilate<'_> {
    fn owner(&self) -> Player {
        self.sim.cities.cities[self.c].owner
    }
}

impl AssimilateView for SimAssimilate<'_> {
    fn clear_unassimilated(&mut self) {
        self.sim.cities.cities[self.c].unassimilated = false;
    }
    fn race_pending(&mut self) -> bool {
        self.sim.cities.cities[self.c].race != Some(self.owner())
    }
    fn adopt_race(&mut self) {
        self.sim.cities.cities[self.c].race = Some(self.owner());
    }
    /// SEAM: the region's border count is not carried; the territory pass
    /// below rebuilds it whole every time.
    fn region_unfixed(&mut self) -> bool {
        true
    }
    fn fix_borders(&mut self) {
        self.sim.sync_territory();
    }
    fn chinese_large_cities(&mut self) -> bool {
        self.sim.cities.nation[self.owner() as usize].chinese
            && self.sim.tables.tuning.chinese_large_cities != 0
    }
    fn town_up(&mut self) {
        let b = self.sim.cities.cities[self.c].building;
        if let Some(town) = self
            .sim
            .tables
            .build_types
            .iter()
            .position(|t| t.ident == Ident::Town)
            && self.sim.building_ident(b) == Ident::Village
        {
            // Without a `mask_me`: the original leaves the radius mask
            // at the old size until the next remask.
            self.sim.set_type(b, town);
        }
    }
    fn raise_economy_flag(&mut self) {
        let who = self.owner();
        self.sim.economy_changed(who);
    }
    fn first_member(&mut self) -> Option<i32> {
        (self.chain.len() > 1).then_some(1)
    }
    fn next_member(&mut self, m: i32) -> Option<i32> {
        ((m as usize + 1) < self.chain.len()).then_some(m + 1)
    }
    fn member_active(&mut self, m: i32) -> bool {
        self.sim.buildings[self.chain[m as usize]].active
    }
    fn member_is_library(&mut self, m: i32) -> bool {
        self.sim.building_is(self.chain[m as usize], Ident::Library)
    }
    fn member_set_active(&mut self, m: i32, on: bool) {
        self.sim.buildings[self.chain[m as usize]].active = on;
    }
    /// SEAM: a library's queue is the owner's first library's
    /// (`get_first_library`), read where it is used and never copied.
    fn new_library(&mut self, _m: i32) {}
    fn member_is_senate(&mut self, m: i32) -> bool {
        self.sim.building_is(self.chain[m as usize], Ident::Senate)
    }
    fn gov_hero_frame(&mut self) -> i32 {
        self.sim.tech[self.owner() as usize].gov_hero_frame as i32
    }
    fn gov_hero_found(&mut self, _m: i32) -> bool {
        let who = self.owner();
        self.sim.gov_hero_exists(who)
    }
    fn hero_frame_reached(&mut self, since: i32) -> bool {
        i64::from(since) <= self.sim.frame
    }
    fn gov_hero_type(&mut self, _since: i32) -> Option<i32> {
        let who = self.owner() as usize;
        let hero = self
            .sim
            .tables
            .tech_tree
            .get_gov_hero(&self.sim.tables.setup, &self.sim.tech[who])?;
        self.sim.unit_record(hero).map(|r| r as i32)
    }
    fn train(&mut self, m: i32, ty: i32) {
        self.sim.build_train(self.chain[m as usize], ty as usize);
    }
    /// The console is the renderer's; the sim has none.
    fn console_is_owner(&mut self) -> bool {
        false
    }
    fn announce(&mut self) {}
}
