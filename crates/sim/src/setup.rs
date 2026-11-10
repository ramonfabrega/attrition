//! **The setup door** (`docs/STRUCTURE.md` §5 move 9, item 4017): what a
//! harness does to a game that no player's order does — the third door,
//! named so the count of the other two is honest (`docs/STRUCTURE.md` §6).
//!
//! [`Sim::setup`] borrows the game and hands back a [`Setup`], whose methods
//! are forwards exactly as [`crate::command`]'s are: each calls the `Sim`
//! method of its name with its arguments and nothing else. The set is what
//! `rondata` calls today with `&mut Sim` outside the order door: building a
//! game from a start dump (the store, the map, the players, the figures),
//! the golden harness's console cheats, and the diff windows that step one
//! mechanic in isolation against a dump (`docs/AI.md`'s army windows, the
//! scout's think, a relocate probe). [`Sim::new`] is this door's
//! constructor and stays where it is.
//!
//! A read of internal state the original dumps — every field the diff
//! harness compares, and the `&self` queries it links and probes with — is
//! not a door: the harness reads `Sim` for it directly, and
//! `docs/journal/2026-11-09-item-4017.md` counts those reads by name.

use crate::ai::bhs::{self, Source};
use crate::cities::build;
use crate::cities::farms::AnimalSeed;
use crate::combat::Obj;
use crate::economy::gather::MountainTemplate;
use crate::tech;
use crate::units::anim::Guy;
use crate::{Cell, Player, Pos, Sim, Unit, UnitType};

/// The setup door over one borrowed game; see the module note.
pub struct Setup<'a> {
    sim: &'a mut Sim,
}

impl Sim {
    /// The setup door ([`crate::setup`]).
    pub fn setup(&mut self) -> Setup<'_> {
        Setup { sim: self }
    }
}

impl Setup<'_> {
    // The object store and the type tables.
    /// [`Sim::add_unit_type`] (`units/store.rs`).
    pub fn add_unit_type(&mut self, ty: UnitType) -> usize {
        self.sim.add_unit_type(ty)
    }

    /// [`Sim::add_build_type`] (`cities/building.rs`).
    pub fn add_build_type(&mut self, ty: build::BuildType) -> usize {
        self.sim.add_build_type(ty)
    }

    /// [`Sim::set_tech_tree`] (`tech/gain.rs`).
    pub fn set_tech_tree(&mut self, tree: tech::TechTree) {
        self.sim.set_tech_tree(tree)
    }

    /// [`Sim::add_unit`] (`units/store.rs`).
    pub fn add_unit(&mut self, unit: Unit) -> usize {
        self.sim.add_unit(unit)
    }

    /// [`Sim::init_unit`] (`units/store.rs`).
    pub fn init_unit(&mut self, who: Player, ty: usize, pos: Pos) -> usize {
        self.sim.init_unit(who, ty, pos)
    }

    /// [`Sim::scenario_add_unit`] (`ai/scenario_add.rs`).
    pub fn scenario_add_unit(
        &mut self,
        who: Player,
        ty: usize,
        (x, y): (i32, i32),
        num: i32,
    ) -> Option<Vec<usize>> {
        self.sim.scenario_add_unit(who, ty, (x, y), num)
    }

    /// [`Sim::add_building`] (`cities/building.rs`).
    pub fn add_building(&mut self, owner: Player, pos: Pos, capacity: usize) -> usize {
        self.sim.add_building(owner, pos, capacity)
    }

    /// [`Sim::init_build`] (`cities/city.rs`).
    pub fn init_build(&mut self, who: Player, ty: usize, pos: Pos, restore: bool) -> usize {
        self.sim.init_build(who, ty, pos, restore)
    }

    /// [`Sim::activate`] (`cities/city.rs`).
    pub fn activate(&mut self, b: usize, captured: bool, counted: bool) {
        self.sim.activate(b, captured, counted)
    }

    /// [`Sim::produce_building`] (`ai/ai_place.rs`): at frame 0, its
    /// arm for `Setup::small_city_buildings` and `build_civ_specific`
    /// (`docs/START.md` §2 step 7).
    pub fn produce_building(
        &mut self,
        who: Player,
        rec: usize,
        near: usize,
        city: Option<usize>,
        escrow: bool,
    ) -> bool {
        self.sim.produce_building(who, rec, near, city, escrow)
    }

    /// [`Sim::place_unit`] (`units/starting.rs`): `Setup::place_unit`'s
    /// arm for a start with a city (`docs/START.md` §2 step 7).
    pub fn place_unit(
        &mut self,
        who: Player,
        city: Option<usize>,
        ty: usize,
        at: Pos,
    ) -> Result<usize, crate::units::starting::PlaceRefused> {
        self.sim.place_unit(who, city, ty, at)
    }

    /// [`Sim::come_out_own_point`] (`units/starting.rs`): a starting
    /// citizen's `Unit::come_out(u, 0)` off its building.
    pub fn come_out_own_point(&mut self, u: usize) -> Pos {
        self.sim.come_out_own_point(u)
    }

    /// [`Sim::found_herd`] (`gaia/herd_create_view.rs`):
    /// `Herd::create_units` (`docs/START.md` §2 step 8).
    pub fn found_herd(&mut self, h: &crate::gaia::herd_create_view::HerdRec) -> (Vec<usize>, bool) {
        self.sim.found_herd(h)
    }

    /// [`Sim::close_building`] (`cities/city.rs`).
    pub fn close_building(&mut self, b: usize, silent: bool) {
        self.sim.close_building(b, silent)
    }

    /// [`Sim::die_building`] (`cities/city.rs`).
    pub fn die_building(&mut self, b: usize) {
        self.sim.die_building(b)
    }

    /// [`Sim::spawn_bird_at`] (`gaia.rs`).
    pub fn spawn_bird_at(&mut self, pos: Pos) -> Option<usize> {
        self.sim.spawn_bird_at(pos)
    }

    /// [`Sim::add_goody_item`] (`gaia/goody.rs`).
    pub fn add_goody_item(&mut self, pos: Pos, alive: bool, ever_seen: u8) {
        self.sim.add_goody_item(pos, alive, ever_seen)
    }

    /// [`Sim::farm_add_animals`] (`cities/farms.rs`).
    pub fn farm_add_animals(&mut self, b: usize, seeds: &[AnimalSeed]) -> Vec<usize> {
        self.sim.farm_add_animals(b, seeds)
    }

    // The map: its fixed features and the territory over it.
    /// [`Sim::place_mountains`] (`economy/gather.rs`).
    pub fn place_mountains(&mut self, placed: &[(Cell, usize)], templates: &[MountainTemplate]) {
        self.sim.place_mountains(placed, templates)
    }

    /// [`Sim::place_cliffs`] (`economy/cliff_mine.rs`).
    pub fn place_cliffs(&mut self, mining: &[Vec<(i32, i32)>]) {
        self.sim.place_cliffs(mining)
    }

    /// [`Sim::place_roads`] (`movement/roads.rs`).
    pub fn place_roads(&mut self, b: usize) {
        self.sim.place_roads(b)
    }

    /// [`Sim::claim_oil_from_owners`] (`economy/rares.rs`).
    pub fn claim_oil_from_owners(&mut self) {
        self.sim.claim_oil_from_owners()
    }

    /// [`Sim::update_seen_ally_build`] (`vision/seen_ally.rs`).
    pub fn update_seen_ally_build(&mut self, b: usize) -> usize {
        self.sim.update_seen_ally_build(b)
    }

    /// [`Sim::seed_new_rares_from_fog`] (`economy/rares.rs`).
    pub fn seed_new_rares_from_fog(&mut self) {
        self.sim.seed_new_rares_from_fog()
    }

    /// [`Sim::settle_borders`] (`territory/border_pass.rs`).
    pub fn settle_borders(&mut self) {
        self.sim.settle_borders()
    }

    /// [`Sim::sync_territory`] (`cities/city.rs`).
    pub fn sync_territory(&mut self) {
        self.sim.sync_territory()
    }

    // The players: the lobby, the AI, the tech tree, diplomacy.
    /// [`Sim::sync_setup_from_lobby`] (`cities/nations.rs`).
    pub fn sync_setup_from_lobby(&mut self) {
        self.sim.sync_setup_from_lobby()
    }

    /// [`Sim::set_tribe`] (`cities/nations.rs`).
    pub fn set_tribe(&mut self, who: Player, tribe: i64) {
        self.sim.set_tribe(who, tribe)
    }

    /// [`Sim::init_leader_ai`] (`ai/ai_drive.rs`).
    pub fn init_leader_ai(&mut self, who: Player, player_flags: u32) {
        self.sim.init_leader_ai(who, player_flags)
    }

    /// [`Sim::load_scripts`] (`ai/ai_host.rs`).
    pub fn load_scripts(
        &mut self,
        roots: &[&str],
        sources: &[Source],
    ) -> Result<(), bhs::CompileError> {
        self.sim.load_scripts(roots, sources)
    }

    /// [`Sim::start_techs`] (`tech/gain.rs`).
    pub fn start_techs(&mut self, who: Player) -> Vec<tech::Gained> {
        self.sim.start_techs(who)
    }

    /// [`Sim::gain_tech`] (`tech/gain.rs`).
    pub fn gain_tech(&mut self, who: Player, t: tech::TypeId) -> Vec<tech::Gained> {
        self.sim.gain_tech(who, t)
    }

    /// [`Sim::set_leader_age`] (`leaders/relations.rs`).
    pub fn set_leader_age(&mut self, who: Player, n: i32) -> Vec<tech::Gained> {
        self.sim.set_leader_age(who, n)
    }

    /// [`Sim::set_leader_epoch`] (`leaders/relations.rs`).
    pub fn set_leader_epoch(&mut self, who: Player, cat: i32, level: i32) -> Vec<tech::Gained> {
        self.sim.set_leader_epoch(who, cat, level)
    }

    /// [`Sim::set_diplo`] (`leaders/relations.rs`).
    pub fn set_diplo(&mut self, a: Player, b: Player, level: i32) {
        self.sim.set_diplo(a, b, level)
    }

    /// [`Sim::init_diplos`] (`leaders/relations.rs`).
    pub fn init_diplos(&mut self, who: Player, seats: &[crate::leaders::relations::Seat]) {
        self.sim.init_diplos(who, seats)
    }

    /// [`Sim::track_unit_type`] (`units/store.rs`).
    pub fn track_unit_type(&mut self, who: Player, ty: usize, delta: i32) {
        self.sim.track_unit_type(who, ty, delta)
    }

    /// [`Sim::init_iroquois_masks`] (`combat/cloak.rs`).
    pub fn init_iroquois_masks(&mut self, u: usize) {
        self.sim.init_iroquois_masks(u)
    }

    // The figures, seated as a dump left them.
    /// [`Sim::set_guy`] (`units/anim.rs`).
    pub fn set_guy(&mut self, u: usize, g: usize, guy: Guy) {
        self.sim.set_guy(u, g, guy)
    }

    /// [`Sim::seat_guys`] (`units/anim.rs`).
    pub fn seat_guys(&mut self, u: usize) {
        self.sim.seat_guys(u)
    }

    /// [`Sim::birth_come_out`] (`movement/collide.rs`).
    pub fn birth_come_out(&mut self, u: usize, birth: Pos, later: &[usize]) {
        self.sim.birth_come_out(u, birth, later)
    }

    /// [`Sim::reseat_animal`] (`gaia.rs`).
    pub fn reseat_animal(&mut self, u: usize, pos: Pos, goal: Option<Pos>) -> i32 {
        self.sim.reseat_animal(u, pos, goal)
    }

    // The golden harness's console cheats: input, but not orders.
    /// [`Sim::cheat_die_unit`] (`combat/fight/delivery.rs`).
    pub fn cheat_die_unit(&mut self, v: usize) {
        self.sim.cheat_die_unit(v)
    }

    /// [`Sim::cheat_finish`] (`cities/city.rs`).
    pub fn cheat_finish(&mut self, b: usize) {
        self.sim.cheat_finish(b)
    }

    /// [`Sim::cheat_finish_all`] (`cities/city.rs`).
    pub fn cheat_finish_all(&mut self, b: usize) {
        self.sim.cheat_finish_all(b)
    }

    /// [`Sim::cheat_resource`] (`tech/gain.rs`).
    pub fn cheat_resource(&mut self, who: Player, good: Option<usize>, amount: i32) {
        self.sim.cheat_resource(who, good, amount)
    }

    // One mechanic stepped alone, in a window that compares it to a dump.
    /// [`Sim::army_engagement_seed`] (`ai/army.rs`).
    pub fn army_engagement_seed(&mut self, who: Player, slot: usize) -> Option<(usize, Obj)> {
        self.sim.army_engagement_seed(who, slot)
    }

    /// [`Sim::army_is_engaged`] (`ai/army.rs`).
    pub fn army_is_engaged(&mut self, who: Player, slot: usize) -> bool {
        self.sim.army_is_engaged(who, slot)
    }

    /// [`Sim::do_mustering`] (`ai/army.rs`).
    pub fn do_mustering(&mut self, who: Player, slot: usize) {
        self.sim.do_mustering(who, slot)
    }

    /// [`Sim::find_muster_spot`] (`ai/army.rs`).
    pub fn find_muster_spot(&mut self, who: Player, slot: usize, target: Obj, flag: bool) -> bool {
        self.sim.find_muster_spot(who, slot, target, flag)
    }

    /// [`Sim::find_target`] (`ai/army.rs`).
    pub fn find_target(&mut self, who: Player, slot: usize) {
        self.sim.find_target(who, slot)
    }

    /// [`Sim::think_scout`] (`units/think/scout.rs`).
    pub fn think_scout(&mut self, u: usize) -> bool {
        self.sim.think_scout(u)
    }

    /// [`Sim::probe_relocate`] (`movement/collide.rs`).
    pub fn probe_relocate(&mut self, u: usize, to: Pos) -> bool {
        self.sim.probe_relocate(u, to)
    }
}
