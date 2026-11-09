//! **The order door** (`docs/STRUCTURE.md` §5 move 9, item 4017): every way
//! a player's orders reach a running game, and the tick that carries them
//! out.
//!
//! [`Sim::command`] borrows the game and hands back a [`Command`], whose
//! methods are forwards — each calls the `Sim` method of its name with its
//! arguments, and nothing else, so a caller moved onto this door runs the
//! statements it ran before, in the order it ran them. The set is what is
//! issued today, not a design: the order replay (`rondata::input::Stream`,
//! one `CommandPackage::process_*` each), the golden harness's order
//! staging, and the soak's generated orders (`soak::issue`). An order
//! nobody issues is not here.
//!
//! The other two doors: [`crate::observe`] is the read surface, and
//! [`crate::setup`] holds what a harness does to stage a game that no
//! player's order does.

use crate::air::airbase::Keys;
use crate::combat::Obj;
use crate::groups::group::{Flight, Group};
use crate::movement::Angle;
use crate::orders::{MoveKind, QueuePos};
use crate::{Player, Pos, Sim, Tick};

/// The order door over one borrowed game; see the module note.
pub struct Command<'a> {
    sim: &'a mut Sim,
}

impl Sim {
    /// The order door ([`crate::command`]).
    pub fn command(&mut self) -> Command<'_> {
        Command { sim: self }
    }
}

impl Command<'_> {
    // The selection: `CommandPackage::process_group`, and the building
    // group a command on buildings pushes first.
    /// [`Sim::push_group`] (`groups/group/pool.rs`).
    pub fn push_group(&mut self, g: &mut Group, force: bool) -> bool {
        self.sim.push_group(g, force)
    }

    /// [`Sim::push_command_buildings`] (`units/garrison.rs`).
    pub fn push_command_buildings(&mut self, who: crate::world::Player, list: &[usize]) {
        self.sim.push_command_buildings(who, list)
    }

    // One unit's orders: `Unit::add_*_order`, as the replay and the soak give them.
    /// [`Sim::add_move_order`] (`orders/order_list.rs`).
    pub fn add_move_order(
        &mut self,
        u: usize,
        to: Pos,
        kind: MoveKind,
        pos: QueuePos,
        action: bool,
    ) {
        self.sim.add_move_order(u, to, kind, pos, action)
    }

    /// [`Sim::add_build_order`] (`orders/order_list.rs`).
    pub fn add_build_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.sim.add_build_order(u, b, pos, action)
    }

    /// [`Sim::add_gather_order`] (`orders/order_list.rs`).
    pub fn add_gather_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.sim.add_gather_order(u, b, pos, action)
    }

    /// [`Sim::add_attack_order`] (`orders/order_list.rs`).
    pub fn add_attack_order(
        &mut self,
        u: usize,
        target: Obj,
        pos: QueuePos,
        mandatory: bool,
        action: bool,
    ) {
        self.sim.add_attack_order(u, target, pos, mandatory, action)
    }

    /// [`Sim::add_garrison_order`] (`orders/order_list.rs`).
    pub fn add_garrison_order(
        &mut self,
        u: usize,
        b: usize,
        search: bool,
        pos: QueuePos,
        action: bool,
    ) {
        self.sim.add_garrison_order(u, b, search, pos, action)
    }

    /// [`Sim::add_repair_order`] (`orders/order_list.rs`).
    pub fn add_repair_order(&mut self, u: usize, b: usize, pos: QueuePos, action: bool) {
        self.sim.add_repair_order(u, b, pos, action)
    }

    /// [`Sim::clear_orders`] (`orders/order_list.rs`).
    pub fn clear_orders(&mut self, u: usize) {
        self.sim.clear_orders(u)
    }

    /// [`Sim::unit_action_unqueue`] (`air/carrier_queue.rs`).
    pub fn unit_action_unqueue(&mut self, u: usize, refund: bool) {
        self.sim.unit_action_unqueue(u, refund)
    }

    // A group's orders: `Group::action_*`, one `process_*` each.
    /// [`Sim::group_action_move_to`] (`groups/group/actions.rs`).
    #[allow(clippy::too_many_arguments)]
    pub fn group_action_move_to(
        &mut self,
        g: &Group,
        to: Pos,
        queue: QueuePos,
        set_angle: bool,
        angle: Angle,
        kind: MoveKind,
        action: bool,
    ) {
        self.sim
            .group_action_move_to(g, to, queue, set_angle, angle, kind, action)
    }

    /// [`Sim::group_action_patrol`] (`groups/group/actions.rs`).
    pub fn group_action_patrol(&mut self, g: &Group, to: Pos, queue: QueuePos) {
        self.sim.group_action_patrol(g, to, queue)
    }

    /// [`Sim::group_action_attack`] (`groups/group/actions.rs`).
    pub fn group_action_attack(
        &mut self,
        g: &Group,
        target: Obj,
        mandatory: bool,
        queue: QueuePos,
        ignore: i32,
    ) {
        self.sim
            .group_action_attack(g, target, mandatory, queue, ignore)
    }

    /// [`Sim::group_action_air_attack_ground`] (`groups/group/actions.rs`).
    pub fn group_action_air_attack_ground(&mut self, g: &Group, at: Pos, queue: QueuePos) {
        self.sim.group_action_air_attack_ground(g, at, queue)
    }

    /// [`Sim::group_action_follow`] (`groups/group/actions.rs`).
    pub fn group_action_follow(&mut self, g: &Group, target: usize, queue: QueuePos) {
        self.sim.group_action_follow(g, target, queue)
    }

    /// [`Sim::group_action_guard`] (`groups/group/actions.rs`).
    pub fn group_action_guard(
        &mut self,
        g: &Group,
        target: usize,
        queue: QueuePos,
        no_siege: bool,
    ) {
        self.sim.group_action_guard(g, target, queue, no_siege)
    }

    /// [`Sim::group_action_garrison`] (`groups/group/actions.rs`).
    pub fn group_action_garrison(&mut self, g: &Group, b: usize, queue: QueuePos) {
        self.sim.group_action_garrison(g, b, queue)
    }

    /// [`Sim::group_action_form`] (`groups/form.rs`).
    pub fn group_action_form(&mut self, g: &Group, form: i32, rotate: i32, queue: QueuePos) {
        self.sim.group_action_form(g, form, rotate, queue)
    }

    /// [`Sim::group_action_stance`] (`groups/group/actions.rs`).
    pub fn group_action_stance(&mut self, g: &Group, s: i32) {
        self.sim.group_action_stance(g, s)
    }

    /// [`Sim::group_action_build`] (`groups/group/actions.rs`).
    pub fn group_action_build(
        &mut self,
        g: &Group,
        at: Pos,
        ty: usize,
        pos: QueuePos,
    ) -> Option<usize> {
        self.sim.group_action_build(g, at, ty, pos)
    }

    /// [`Sim::group_action_spell`] (`spells/cast.rs`).
    pub fn group_action_spell(&mut self, g: &Group, s: i32, target: Option<Obj>, at: Pos) -> usize {
        self.sim.group_action_spell(g, s, target, at)
    }

    /// [`Sim::group_action_stop_spell`] (`spells/cast.rs`).
    pub fn group_action_stop_spell(&mut self, g: &Group) -> usize {
        self.sim.group_action_stop_spell(g)
    }

    /// [`Sim::group_action_disband`] (`units/disband.rs`).
    pub fn group_action_disband(&mut self, g: &crate::groups::group::Group, all: bool) -> usize {
        self.sim.group_action_disband(g, all)
    }

    /// [`Sim::group_action_disband_buildings`] (`units/disband.rs`).
    pub fn group_action_disband_buildings(&mut self, list: &[usize], all: bool) -> usize {
        self.sim.group_action_disband_buildings(list, all)
    }

    /// [`Sim::group_action_flight`] (`groups/group/actions.rs`).
    pub fn group_action_flight(&mut self, g: &Group, target: Obj, kind: Flight) {
        self.sim.group_action_flight(g, target, kind)
    }

    /// [`Sim::group_action_recall`] (`groups/group/actions.rs`).
    pub fn group_action_recall(&mut self, g: &Group) {
        self.sim.group_action_recall(g)
    }

    /// [`Sim::group_action_city_gather`] (`cities/rally.rs`).
    pub fn group_action_city_gather(&mut self, buildings: &[usize], kind: i32) {
        self.sim.group_action_city_gather(buildings, kind)
    }

    /// [`Sim::group_gather`] (`groups/alarm.rs`).
    pub fn group_gather(&mut self, g: &Group, b: usize, queued: QueuePos) {
        self.sim.group_gather(g, b, queued)
    }

    /// [`Sim::group_swarm_around`] (`groups/swarm.rs`).
    pub fn group_swarm_around(
        &mut self,
        g: &Group,
        b: usize,
        queued: QueuePos,
        orders: u8,
    ) -> bool {
        self.sim.group_swarm_around(g, b, queued, orders)
    }

    /// [`Sim::set_transport`] (`ai/transport/mod.rs`).
    pub fn set_transport(&mut self, who: Player, units: &[usize], on: bool) {
        self.sim.set_transport(who, units, on)
    }

    /// [`Sim::action_alarm_peasant`] (`groups/alarm.rs`).
    pub fn action_alarm_peasant(&mut self, g: &Group) {
        self.sim.action_alarm_peasant(g)
    }

    // Orders to buildings.
    /// [`Sim::group_action_launch_flight`] (`air/airbase.rs`).
    pub fn group_action_launch_flight(
        &mut self,
        who: Player,
        buildings: &[usize],
        target: Obj,
        kind: Flight,
        keys: Keys,
    ) {
        self.sim
            .group_action_launch_flight(who, buildings, target, kind, keys)
    }

    /// [`Sim::group_action_launch_patrol`] (`air/airbase.rs`).
    pub fn group_action_launch_patrol(
        &mut self,
        buildings: &[usize],
        at: Pos,
        queue: QueuePos,
        keys: Keys,
    ) {
        self.sim
            .group_action_launch_patrol(buildings, at, queue, keys)
    }

    /// [`Sim::group_action_recall_buildings`] (`air/airbase.rs`).
    pub fn group_action_recall_buildings(&mut self, who: Player, buildings: &[usize]) {
        self.sim.group_action_recall_buildings(who, buildings)
    }

    /// [`Sim::group_action_scramble`] (`air/airbase.rs`).
    pub fn group_action_scramble(&mut self, buildings: &[usize]) {
        self.sim.group_action_scramble(buildings)
    }

    /// [`Sim::action_alarm`] (`groups/alarm.rs`).
    pub fn action_alarm(&mut self, who: Player, list: &[usize]) {
        self.sim.action_alarm(who, list)
    }

    /// [`Sim::action_buildmask`] (`air/mod.rs`).
    pub fn action_buildmask(&mut self, buildings: &[usize], mask: i32) -> usize {
        self.sim.action_buildmask(buildings, mask)
    }

    /// [`Sim::action_eject_all`] (`groups/group/actions.rs`).
    pub fn action_eject_all(
        &mut self,
        who: Player,
        buildings: &[usize],
        eject_who: i32,
        cmd_who: i32,
    ) {
        self.sim
            .action_eject_all(who, buildings, eject_who, cmd_who)
    }

    /// [`Sim::action_gather_point`] (`cities/rally.rs`).
    pub fn action_gather_point(
        &mut self,
        who: Player,
        members: &[usize],
        at: crate::Pos,
        action: i32,
        add_to_end: bool,
    ) {
        self.sim
            .action_gather_point(who, members, at, action, add_to_end)
    }

    /// [`Sim::action_queue_disband`] (`cities/queue_disband.rs`).
    pub fn action_queue_disband(&mut self, buildings: &[usize]) -> usize {
        self.sim.action_queue_disband(buildings)
    }

    /// [`Sim::action_queue_research`] (`cities/production.rs`).
    pub fn action_queue_research(&mut self, buildings: &[usize], t: crate::tech::TypeId) -> usize {
        self.sim.action_queue_research(buildings, t)
    }

    /// [`Sim::action_queue_up`] (`cities/production.rs`).
    pub fn action_queue_up(&mut self, buildings: &[usize], ty: usize, num: i32) -> usize {
        self.sim.action_queue_up(buildings, ty, num)
    }

    /// [`Sim::action_unqueue`] (`cities/production.rs`).
    pub fn action_unqueue(&mut self, at: usize, p: i32) -> usize {
        self.sim.action_unqueue(at, p)
    }

    // A player's standing.
    /// [`Sim::declare_war`] (`leaders/relations.rs`).
    pub fn declare_war(&mut self, a: Player, b: Player) {
        self.sim.declare_war(a, b)
    }

    /// [`Sim::resign`] (`leaders/resign.rs`).
    pub fn resign(&mut self, who: Player) {
        self.sim.resign(who)
    }

    // The clock: one frame, every mechanic, in the original's order.
    /// [`Sim::tick`] (`core/frame.rs`).
    pub fn tick(&mut self) -> Vec<Tick> {
        self.sim.tick()
    }
}
