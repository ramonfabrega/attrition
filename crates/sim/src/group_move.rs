//! **`Unit::do_group_move@005e79a0`**, by slices (item 3376; `docs/ORDERS.md`
//! §8.3).
//!
//! The function is seventeen thousand bytes: a head that finds the order and
//! the leader, a **leader arm** that steps a plain move and publishes the
//! pass, and a follower arm that steers off the leader. The head and the
//! leader arm are [`do_group_move_steps`] over a [`GroupMoveView`] that names
//! every access they make in the order they make them. [`SimGroupMove`]
//! answers from this crate's state; the sweep
//! (`crates/sim/src/sweep/unit_do_group_move_leader.rs`) answers from the
//! emulated original's windows, and ends the call where the follower arm
//! first asks for the leader's object (`--stop-stub`).
//!
//! The follower arm is [`Sim::group_move_follower`], still the crate's own
//! reading: it is entered through [`GroupMoveView::follower_arm`] and the
//! slices that sweep it come after.
//!
//! **Not established.** The leader arm's attack hand-off — a leader whose
//! step gave up while something stands in its way — ends in the group's
//! `distribute_attack`, which this crate does not model (no traced game
//! reaches it, `docs/GROUPS.md` §14); [`SimGroupMove`] answers the lookups
//! and does nothing for the call. A unit's index is compared as the original's
//! sign-extended short, which the sweep draws only from 0 and 1 (a negative
//! index never names a live unit).

use crate::Sim;
use crate::combat::Obj;
use crate::group::Group;
use crate::orders::{Body, Did, GroupMove, QueuePos, index};
use crate::world::vector_dist;

/// What `do_group_move`'s head and leader arm read and do, in the order
/// they do it. A name that is a call of the original's says so; the rest are
/// reads of the unit, its order, its group and the leaders table.
pub trait GroupMoveView {
    /// The order's vslot `+0x98`: the move order's record (`GroupMoveOrder`
    /// handed back). Asked at the top and again before the last ungroup.
    fn order_record(&mut self);
    /// `this->group == -1`: the unit has no seat.
    fn group_unset(&mut self) -> bool;
    /// `Unit::ungroup_move_order(this, order.id, 0)`.
    fn ungroup_move_order(&mut self);
    /// The order names this unit as the leader: its `+0x54` against the
    /// sign-extended index and `+0x58` against the owner.
    fn leader_is_me(&mut self) -> bool;
    /// The leader's object, asked through its vslot `+0x8`: the follower
    /// arm's first act. The sweep ends the call here.
    fn follower_arm(&mut self);
    /// `UnitData::get_action` answered something.
    fn get_action(&mut self) -> bool;
    /// The action order's vslot `+0x10`: its type.
    fn action_type_slot(&mut self) -> i32;
    /// `(frame + index) & 31 == 0`: this unit's turn, one frame in thirty-two.
    fn frame_tick(&mut self) -> bool;
    /// `Unit::update_action`.
    fn update_action(&mut self);
    /// The update's action target's vslot `+0x50`, and the distance from the
    /// unit to what it names (`vector_dist` of the two absolute differences);
    /// `i32::MAX` when it names nothing.
    fn action_target_dist(&mut self) -> i32;
    /// `Group::kill_group_move(group, order.id)`.
    fn kill_group_move(&mut self);
    /// `Unit::do_move(this, order)` answered non-zero.
    fn do_move(&mut self) -> bool;
    /// `Unit::update_order(this) == order`: the move just stepped is still
    /// the head.
    fn update_order_is_param(&mut self) -> bool;
    /// The unit's vslot `+0x17c` at its own point with flag 1: its uncapped
    /// speed.
    fn leader_speed(&mut self) -> i32;
    /// The group's accumulator `+0x3c` moves to `+0x40`, takes the speed,
    /// and `+0x4b` clears.
    fn leader_report(&mut self, speed: i32);
    /// The leaders table's flags for the owner carry `0x8000`.
    fn leader_flag_8000(&mut self) -> bool;
    /// `ObjectData::has_general(this, 0x8000, -1) >= 0`.
    fn has_general(&mut self) -> bool;
    /// The group's `+0x4b = 1`.
    fn set_march(&mut self);
    /// `Group::update_positions(group, index, owner)`.
    fn update_positions(&mut self);
    /// `UnitData::action_type`.
    fn action_type(&mut self) -> i32;
    /// The order's own vslot `+0x10` (through its virtual base).
    fn mo_type(&mut self) -> i32;
    /// `this->collide != 0`.
    fn collide_set(&mut self) -> bool;
    /// `Object::valid_target(this, collide_o, collide_who, 0)`.
    fn valid_target(&mut self) -> bool;
    /// `this->group < 0`.
    fn group_negative(&mut self) -> bool;
    /// `Group::action_attack(group, collide_o, collide_who, 0, QUEUE_FIRST, 0)`.
    fn action_attack(&mut self);
    /// The action's two lookups (`get_action` twice, the action's vslot
    /// `+0xcc` twice) and `Group::distribute_attack(group, ..)` on what they
    /// name.
    fn distribute_attack(&mut self);
    /// `UnitData::order_type`.
    fn order_type(&mut self) -> i32;
}

/// `OrderIndex::ATTACK`.
const ATTACK: i32 = 10;
/// `OrderIndex::GROUP_ATTACK`.
const GROUP_ATTACK: i32 = 20;
/// `OrderIndex::GROUP_MOVE`.
const GROUP_MOVE: i32 = 19;
/// `OrderIndex::GROUP_ATTACK_TO`.
const GROUP_ATTACK_TO: i32 = 21;

/// `do_group_move`, from the entry to the leader arm's last `ret`.
pub fn do_group_move_steps<V: GroupMoveView>(v: &mut V) {
    v.order_record();
    // `5e79cd`: no seat — the group order degrades on the spot.
    if v.group_unset() {
        v.ungroup_move_order();
        return;
    }
    if !v.leader_is_me() {
        v.follower_arm();
        return;
    }
    // Every 32nd frame, phased by the index, an attacking leader whose
    // target has come within `0x900` drops the whole group move.
    if v.get_action() && v.action_type_slot() == ATTACK && v.frame_tick() {
        v.update_action();
        if v.action_target_dist() < 0x900 {
            v.kill_group_move();
            return;
        }
    }
    if v.do_move() {
        if !v.update_order_is_param() {
            return;
        }
        // The leader publishes the pass, and a marching hero's army sets the
        // forced-march flag under it.
        let speed = v.leader_speed();
        v.leader_report(speed);
        if v.leader_flag_8000() && v.has_general() {
            v.set_march();
        }
        v.update_positions();
        return;
    }
    // `do_move` gave up.
    if v.update_order_is_param() && attack_hand_off(v) {
        return;
    }
    // `5e7c50`: a group order of either kind degrades; anything else stands.
    if v.order_type() != GROUP_MOVE && v.order_type() != GROUP_ATTACK_TO {
        return;
    }
    if !v.update_order_is_param() {
        return;
    }
    v.order_record();
    v.ungroup_move_order();
}

/// `5e7b00`–`5e7c4d`: the leader's step gave up while its action is an
/// attack (or the order is an attack-move) and something it may attack
/// stands in the way. `true` when the call ends here.
fn attack_hand_off<V: GroupMoveView>(v: &mut V) -> bool {
    let attacking = v.action_type() == ATTACK
        || v.action_type() == GROUP_ATTACK
        || v.mo_type() == GROUP_ATTACK_TO;
    if !(attacking && v.collide_set() && v.valid_target() && !v.group_negative()) {
        return false;
    }
    if v.mo_type() == GROUP_ATTACK_TO {
        v.action_attack();
        return true;
    }
    v.kill_group_move();
    if v.action_type() == ATTACK || v.action_type() == GROUP_ATTACK {
        v.distribute_attack();
    }
    true
}

/// This crate's [`GroupMoveView`]: `u` is the unit, `g` its seat's group.
pub struct SimGroupMove<'a> {
    pub sim: &'a mut Sim,
    pub u: usize,
    pub frame: i64,
    pub g: Option<Group>,
    pub gm: GroupMove,
    /// The action [`Self::update_action`] answered.
    action: Option<usize>,
}

impl<'a> SimGroupMove<'a> {
    pub fn new(sim: &'a mut Sim, u: usize, frame: i64, g: Option<Group>, gm: GroupMove) -> Self {
        Self {
            sim,
            u,
            frame,
            g,
            gm,
            action: None,
        }
    }
}

impl GroupMoveView for SimGroupMove<'_> {
    fn order_record(&mut self) {}
    fn group_unset(&mut self) -> bool {
        self.g.is_none()
    }
    fn ungroup_move_order(&mut self) {
        self.sim.ungroup_move_order(self.u, self.gm.id);
    }
    fn leader_is_me(&mut self) -> bool {
        self.gm.leader == self.u
    }
    fn follower_arm(&mut self) {
        let Some(order) = self.sim.current_order(self.u).copied() else {
            return;
        };
        let Body::Move(mo) = order.body else { return };
        self.sim.group_move_follower(
            self.u,
            self.frame,
            self.g.as_ref().expect("a seated group"),
            self.gm,
            mo,
            order.flags,
        );
    }
    fn get_action(&mut self) -> bool {
        self.sim.action_of(self.u).is_some()
    }
    fn action_type_slot(&mut self) -> i32 {
        match self.sim.action_of(self.u) {
            Some(a) => i32::from(self.sim.units[self.u].orders[a].index()),
            None => i32::from(index::NONE),
        }
    }
    fn frame_tick(&mut self) -> bool {
        let idx = i32::from(self.sim.units[self.u].index);
        (self.frame as i32).wrapping_add(idx) & 31 == 0
    }
    fn update_action(&mut self) {
        self.action = self.sim.update_action(self.u);
    }
    fn action_target_dist(&mut self) -> i32 {
        let attacking = self
            .action
            .is_some_and(|a| matches!(self.sim.units[self.u].orders[a].body, Body::Attack(_)));
        match self.sim.units[self.u].combat.target {
            Some(t) if attacking => {
                let p = self.sim.pos_of(t);
                let here = self.sim.units[self.u].pos;
                vector_dist(p.x - here.x, p.y - here.y)
            }
            _ => i32::MAX,
        }
    }
    fn kill_group_move(&mut self) {
        self.sim
            .kill_group_move(self.g.as_ref().expect("a seated group"), self.gm.id);
    }
    fn do_move(&mut self) -> bool {
        self.sim.do_move(self.u, self.frame) == Did::Something
    }
    fn update_order_is_param(&mut self) -> bool {
        self.sim.still_group_move(self.u, self.gm.id)
    }
    fn leader_speed(&mut self) -> i32 {
        self.sim.get_speed(self.u, 1)
    }
    fn leader_report(&mut self, speed: i32) {
        self.sim
            .group_leader_report_speed(self.g.as_ref().expect("a seated group"), speed);
    }
    fn leader_flag_8000(&mut self) -> bool {
        self.sim.leader_marching(self.u)
    }
    fn has_general(&mut self) -> bool {
        self.sim.near_marching_hero(self.u)
    }
    fn set_march(&mut self) {
        self.sim
            .group_set_march(self.g.as_ref().expect("a seated group"));
    }
    fn update_positions(&mut self) {
        self.sim
            .group_update_positions(self.g.as_ref().expect("a seated group"), self.u);
    }
    fn action_type(&mut self) -> i32 {
        self.action_type_slot()
    }
    fn mo_type(&mut self) -> i32 {
        i32::from(self.sim.order_type(self.u))
    }
    fn collide_set(&mut self) -> bool {
        self.sim.units[self.u].collide != 0
    }
    fn valid_target(&mut self) -> bool {
        match self.sim.collider_of(self.u) {
            Some(t) => self.sim.valid_target(Obj::Unit(self.u), Obj::Unit(t)),
            None => false,
        }
    }
    fn group_negative(&mut self) -> bool {
        false
    }
    fn action_attack(&mut self) {
        if let Some(t) = self.sim.collider_of(self.u) {
            self.sim.group_action_attack(
                self.g.as_ref().expect("a seated group"),
                Obj::Unit(t),
                false,
                QueuePos::First,
                0,
            );
        }
    }
    fn distribute_attack(&mut self) {}
    fn order_type(&mut self) -> i32 {
        i32::from(self.sim.order_type(self.u))
    }
}
