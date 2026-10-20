//! `Unit::work@0060d180`'s prologue (`:40-87`) as steps over a view (item
//! 3348): the 32-frame clock, the attack latch, the `0x100` clear and the
//! recharging gate that returns before the dispatch. [`work_gate_steps`] is
//! the original's control flow in its order; [`WorkGateView`] is what it
//! reads and writes, which [`SimWorkGate`] answers from the crate's [`Sim`]
//! and `sweep/unit_work.rs` from the emulated original's windows. The
//! method's account is on [`Sim::work`].
//!
//! The sweep runs `Unit::work` from its entry **up to the first call after
//! the gate** (`this->vtable[0xd0]`, `is_caravan`'s slot, stubbed as the
//! call that ends the row — `sweep_gen.py --stop-stub`), so the whole
//! function is not swept: the caravan block, the move dispatch, the action
//! tail and `do_job` are what that stop leaves out.

use crate::Sim;
use crate::orders::{flag, index};

/// What the prologue reads and writes. One method is one reach into the
/// unit or its order; the original reads the order's type through the
/// order's vtable slot `0x10` once, which [`WorkGateView::order_type`] is.
pub trait WorkGateView {
    /// `(frame + (short)o) & 0x8000001f == 0` by the original's own signed
    /// arithmetic: one frame in 32, phased by the unit's object number.
    fn clock_edge(&self) -> bool;
    /// The attack latch (`+0x8` bit 7).
    fn attack_latch(&self) -> bool;
    /// `+0x40 = 0`: the visibility the unit has made itself.
    fn clear_visible(&mut self);
    /// `+0x68 &= ~4`: the in-danger latch.
    fn clear_in_danger(&mut self);
    /// The front order's type (the order's slot `0x10`), `None` for no
    /// order; asked once.
    fn order_type(&mut self) -> Option<i32>;
    /// The front order's `flags` (`+0x4`).
    fn order_flags(&self) -> u32;
    /// `+0x8 &= 0x7f`: the latch dropped.
    fn drop_attack_latch(&mut self);
    /// `+0x68 & 0x100`. SEAM: the crate carries no such bit (nothing sets
    /// it, `docs/ORDERS.md` §6), so [`SimWorkGate`] answers false.
    fn mask_100(&self) -> bool;
    /// `+0x68 &= ~0x100`.
    fn clear_mask_100(&mut self);
    /// The unit's `recharging` byte (`+0xae`).
    fn recharging(&self) -> u8;
    /// The unit type's `max_range` (`+0x1fc`).
    fn type_max_range(&self) -> i32;
    /// The unit type's `unit_flags & 0x400` (`k`, "melee AND ranged
    /// attacks").
    fn type_melee_and_ranged(&self) -> bool;
}

/// What the prologue answered: the unit goes on to the dispatch, or
/// `Unit::work` returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Go,
    Return,
}

/// `EXPLORE_TO`, the one type the `0x100` clear skips.
const EXPLORE_TO: i32 = index::EXPLORE_TO as i32;
const ATTACK: i32 = index::ATTACK as i32;

/// The prologue, in the original's order (`work@0060d180:40-87`).
pub fn work_gate_steps<V: WorkGateView>(v: &mut V) -> Gate {
    let ty = v.order_type();
    let flags = v.order_flags();
    let has_order = ty.is_some();
    let ty = ty.unwrap_or(i32::from(index::NONE));
    if v.clock_edge() {
        if !v.attack_latch() {
            v.clear_visible();
        }
        v.clear_in_danger();
    }
    if ty != ATTACK {
        v.drop_attack_latch();
        if v.mask_100() && has_order && flags & u32::from(flag::ACTION) != 0 && ty != EXPLORE_TO {
            v.clear_mask_100();
        }
        if v.recharging() != 0
            && v.type_max_range() == 0
            && !v.type_melee_and_ranged()
            && (!has_order || flags & u32::from(flag::ACTION) == 0)
        {
            return Gate::Return;
        }
    }
    Gate::Go
}

/// [`WorkGateView`] over the crate's unit.
pub struct SimWorkGate<'a> {
    sim: &'a mut Sim,
    u: usize,
    frame: i64,
}

impl<'a> SimWorkGate<'a> {
    pub fn new(sim: &'a mut Sim, u: usize, frame: i64) -> Self {
        Self { sim, u, frame }
    }
}

impl WorkGateView for SimWorkGate<'_> {
    fn clock_edge(&self) -> bool {
        self.sim.units[self.u].phase(self.frame).rem_euclid(32) == 0
    }
    fn attack_latch(&self) -> bool {
        self.sim.units[self.u].attacking
    }
    fn clear_visible(&mut self) {
        self.sim.units[self.u].visible = 0;
    }
    fn clear_in_danger(&mut self) {
        self.sim.units[self.u].in_danger = false;
    }
    fn order_type(&mut self) -> Option<i32> {
        self.sim.current_order(self.u).map(|o| i32::from(o.index()))
    }
    fn order_flags(&self) -> u32 {
        self.sim
            .current_order(self.u)
            .map_or(0, |o| u32::from(o.flags))
    }
    fn drop_attack_latch(&mut self) {
        self.sim.units[self.u].attacking = false;
    }
    fn mask_100(&self) -> bool {
        false
    }
    fn clear_mask_100(&mut self) {}
    fn recharging(&self) -> u8 {
        // An animal has no gate: `Animal::work@005d7330` is its own entry.
        if self.sim.units[self.u].is_gaia() {
            0
        } else {
            self.sim.units[self.u].combat.recharging
        }
    }
    fn type_max_range(&self) -> i32 {
        self.sim.profile(crate::combat::Obj::Unit(self.u)).max_range
    }
    fn type_melee_and_ranged(&self) -> bool {
        self.sim.units[self.u].ty.is_some_and(|t| {
            self.sim.unit_types[t]
                .cols
                .flag(crate::ai_load::uflags::MELEE_AND_RANGED)
        })
    }
}
