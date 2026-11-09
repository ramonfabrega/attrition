//! **`Group::action_halt@0070d0c0`'s steps** (item 3379): a group's halt, as a
//! pure walk over a [`HaltView`]. The swept steps read the world only through
//! the view, so the sweep's port is a view over the emulated original's
//! windows and the crate's is [`SimHalt`] (`docs/GROUPS.md` §7).
//!
//! The control flow is the listing's, leaf for leaf: the scenario's
//! ignore-orders sweep and the group's refresh ([`garrison_head`], which the
//! garrison order shares), then, for a unit group only, the formation is
//! forgotten and each member is walked. A member is halted unless it is not
//! on the map, is a plane, is entering or leaving a building, or the mask
//! exempts it (`4` siege through its type's slot `+0x10c`, `2` special, `1`
//! the type test `is(0x3a)`).

use crate::garrison_view::{GarrisonHead, garrison_head};

/// What `Group::action_halt` asks of the group, the world and the objects.
pub trait HaltView: GarrisonHead {
    /// `[this + 0x49] == 0`: a unit group (a building group is left alone).
    fn unit_group(&self) -> bool;
    /// `this + 0x10 = -1`: the group's formation is forgotten.
    fn set_form(&mut self);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `objects[p][o]`'s slot `+8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// The unit is not a plane: slot `+0xc0` of `units[p][u]`, which
    /// `UnitData::is_plane` answers inline (`+0x218` is not 2, or `+0x2b4 &
    /// 0x20`) and any other function by a call.
    fn unit_not_plane(&mut self, p: i32, u: i32) -> bool;
    /// `UnitData::is_entering_or_exiting`.
    fn unit_busy(&mut self, p: i32, u: i32) -> bool;
    /// The unit's type slot `+0x10c`: what mask bit `4` exempts.
    fn unit_siege(&mut self, p: i32, u: i32) -> bool;
    /// Slot `+0xd4`, `UnitData::is_special` (`type +0x2b8 & 0x10`) inline:
    /// what mask bit `2` exempts.
    fn unit_special(&mut self, p: i32, u: i32) -> bool;
    /// Slot `+0xb8`, `ObjectData::is(0x3a, 0)`, inline through the type's
    /// slot `+0x60`: what mask bit `1` exempts.
    fn unit_is_3a(&mut self, p: i32, u: i32) -> bool;
    /// `Unit::clear_orders` as the listing inlines it: `+0x68 &= ~0x4000000`,
    /// `+0xc0 = 0`, `close_orders(0)`, `clear_partial_path`, `update_action`.
    fn clear_orders(&mut self, p: i32, u: i32);
    /// `+0x68 &= ~0x100`, the listing's last write to the member.
    fn clear_mask_bit(&mut self, p: i32, u: i32);
}

/// `Group::action_halt(mask)`.
pub fn action_halt_steps<V: HaltView>(v: &mut V, mask: i32) {
    garrison_head(v);
    if !v.unit_group() {
        return;
    }
    v.set_form();
    if v.count() <= 0 {
        return;
    }
    let p = v.who() & 0xff;
    let mut i = 0;
    loop {
        let m = v.member(i);
        halt_member(v, p, m, mask);
        i += 1;
        if i >= v.count() {
            break;
        }
    }
}

/// One member: the listing's body from `0070d198` to `0070d32f`.
fn halt_member<V: HaltView>(v: &mut V, p: i32, m: i32, mask: i32) {
    if !v.obj_alive(p, m) || !v.obj_bc(p, m) {
        return;
    }
    if !v.unit_not_plane(p, m) || v.unit_busy(p, m) {
        return;
    }
    if mask & 4 != 0 && v.unit_siege(p, m) {
        return;
    }
    if mask & 2 != 0 && v.unit_special(p, m) {
        return;
    }
    if mask & 1 != 0 && v.unit_is_3a(p, m) {
        return;
    }
    v.clear_orders(p, m);
    v.clear_mask_bit(p, m);
}

/// The crate's view over a [`crate::Sim`] and a [`crate::group::Group`].
///
/// Where the crate models less than the original:
///
/// - SEAM: the scenario's ignore-orders sweep and the group's refresh are
///   not modelled (as in [`crate::garrison_view::SimGarrison`]).
/// - SEAM: `is_entering_or_exiting` is not modelled, so no member is ever
///   busy; `+0x68 & 0x100` is not modelled either, so its clear is nothing.
/// - SEAM: there is no spy type, so mask bit `1` exempts no one.
/// - Every group in the simulation is a unit group.
pub struct SimHalt<'a> {
    pub sim: &'a mut crate::Sim,
    pub g: &'a crate::group::Group,
}

impl GarrisonHead for SimHalt<'_> {
    fn who(&self) -> i32 {
        i32::from(self.g.who)
    }
    fn ignore_orders(&self) -> bool {
        false
    }
    fn ignoring_count(&self, _who: i32) -> i32 {
        0
    }
    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
        -1
    }
    fn drop_ignored(&mut self, _o: i32, _who: i32) {}
    fn refresh(&mut self) {}
}

impl HaltView for SimHalt<'_> {
    fn unit_group(&self) -> bool {
        true
    }
    fn set_form(&mut self) {
        // `0070d0c0:29`: the group's `form` is cleared once, before any
        // member is examined; no member's `+0xaa` is touched.
        if let Some(st) = self.sim.gstate_mut(self.g) {
            st.form = -1;
        }
    }
    fn count(&self) -> i32 {
        self.g.list.len() as i32
    }
    fn member(&self, i: i32) -> i32 {
        self.g.list[i as usize] as i32
    }
    fn obj_alive(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].alive()
    }
    fn obj_bc(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].on_map
    }
    fn unit_not_plane(&mut self, _p: i32, u: i32) -> bool {
        !self.sim.is_plane(u as usize)
    }
    fn unit_busy(&mut self, _p: i32, _u: i32) -> bool {
        false
    }
    fn unit_siege(&mut self, _p: i32, u: i32) -> bool {
        self.sim.is_siege_unit(u as usize)
    }
    fn unit_special(&mut self, _p: i32, u: i32) -> bool {
        self.sim.is_special_unit(u as usize)
    }
    fn unit_is_3a(&mut self, _p: i32, _u: i32) -> bool {
        false
    }
    fn clear_orders(&mut self, _p: i32, u: i32) {
        self.sim.clear_orders(u as usize);
    }
    fn clear_mask_bit(&mut self, _p: i32, _u: i32) {}
}
