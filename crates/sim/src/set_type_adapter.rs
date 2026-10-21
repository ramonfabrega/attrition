//! [`crate::Sim::unit_set_type`]'s adapter to [`crate::unit_set_type_view`]
//! (item 3402): the crate's collapsed state answers the view, and what the
//! crate keeps no state for answers as the old method did — nothing.
//!
//! The folds, each the crate's own before the view existed:
//!
//! - **the recycler** (`kill_guy`'s pool push, `Guy::clear`, the pops) has no
//!   state; the kill loops call [`Sim::coll_kill_guy`] for the figures past the
//!   old squad, which is what the old crew loop did in one call;
//! - **`guy_mark` and `squad_size` are [`SQUAD_SIZE`]** (written 1 for every
//!   type), so the first kill loop and the zero loop never run, and the first
//!   re-init loop is guy 0 alone;
//! - **`install_type` is the swap block** (type, hits, kind, derived stats)
//!   and `reinit_guys_begin`, both before the guys' re-init as before;
//! - **`seat_guy(0)` is [`Sim::seat_guys`]**, which seats the whole squad as
//!   `Guy::set_new_location(guy 0, x, y, 1)` does;
//! - **`track_unit_type` carries `control` and `active`**, and `caras` with
//!   it (behind the captain gate), so `leader_totals` and `leader_caras` are
//!   no-ops.
//!
//! SEAM: `is(0x134)` and `is_gov_hero` of the table's unit answer no
//! ([`crate::Sim::counts_in_muster`]'s own seam), the `+0x6c` flags, the CEO
//! bit, `is(0x77)`, the vslots `+0x15c`/`+0x160`, `update_armor`/`update_speed`
//! and the supply tail (`game.version` is not modelled) have no state here.

use crate::anim::SQUAD_SIZE;
use crate::unit_set_type_view::UnitSetTypeView;
use crate::{Player, Sim};

pub(crate) struct SimSetType<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) u: usize,
    /// The new type's record.
    pub(crate) rec: usize,
    /// `+0xe8` once `set_guy_total` has written it.
    total: Option<i32>,
    /// `+0x6c`, which nothing here reads back but the view.
    flags: i32,
}

impl<'a> SimSetType<'a> {
    pub(crate) fn new(sim: &'a mut Sim, u: usize, rec: usize) -> Self {
        SimSetType {
            sim,
            u,
            rec,
            total: None,
            flags: 0,
        }
    }

    fn who(&self) -> Player {
        self.sim.units[self.u].owner
    }

    /// The unit's current type record.
    fn cur(&self) -> Option<usize> {
        self.sim.units[self.u].ty
    }
}

impl UnitSetTypeView for SimSetType<'_> {
    fn is_captain(&mut self) -> bool {
        self.sim.units[self.u].captain
    }
    fn decoy_bit(&mut self) -> bool {
        self.sim.units[self.u].decoy
    }
    fn type_pop(&mut self) -> i32 {
        self.cur().map_or(0, |t| self.sim.unit_types[t].price.pop)
    }
    fn table_unit_is_134(&mut self) -> bool {
        false
    }
    fn table_unit_is_gov_hero(&mut self) -> bool {
        false
    }
    fn old_type_index(&mut self) -> i32 {
        self.cur().map_or(0, |t| t as i32)
    }
    fn track_unit_type(&mut self, _ty: i32, delta: i32) {
        if let Some(t) = self.cur() {
            let who = self.who();
            self.sim.track_unit_type(who, t, delta);
        }
    }
    fn leader_totals(&mut self, _delta: i32, _pop: i32) {}
    fn is_caravan(&mut self) -> bool {
        self.cur().is_some_and(|t| {
            self.sim.unit_types[t]
                .cols
                .flag2(crate::ai_load::uflags2::CARAVAN)
        })
    }
    fn leader_caras(&mut self, _delta: i32) {}

    fn guy_total(&mut self) -> i32 {
        self.total
            .unwrap_or_else(|| self.sim.units[self.u].guys.len() as i32)
    }
    fn guy_mark(&mut self) -> i32 {
        SQUAD_SIZE as i32
    }
    fn set_guy_mark(&mut self, _n: i32) {}
    fn has_type(&mut self) -> bool {
        true
    }
    fn squad_size(&mut self, _ty: i32) -> i32 {
        SQUAD_SIZE as i32
    }
    fn crew_size(&mut self, ty: i32) -> i32 {
        self.sim.unit_types[ty as usize].combat.crew_size
    }
    fn kill_and_recycle(&mut self, i: i32, _entry: Option<i32>) {
        if let Ok(i) = usize::try_from(i)
            && i < self.sim.units[self.u].guys.len()
        {
            self.sim.coll_kill_guy(self.u, i);
        }
    }
    fn guy_capacity(&mut self) -> i32 {
        0
    }
    fn guy_hint(&mut self) -> i32 {
        0
    }
    fn grow_guys(&mut self, _n: i32) {}
    fn zero_guy_slot(&mut self, _i: i32) {}
    fn set_guy_total(&mut self, n: i32) {
        self.total = Some(n);
    }
    fn pop_guy(&mut self, _i: i32) {}
    fn install_type(&mut self, _ty: i32) {
        let (u, rec) = (self.u, self.rec);
        let who = self.who();
        let damage = self.sim.units[u].max_health - self.sim.units[u].health;
        let hits = self.sim.unit_hits(who, rec);
        {
            let unit = &mut self.sim.units[u];
            unit.ty = Some(rec);
            unit.max_health = hits;
            unit.health = (hits - damage).max(1);
        }
        self.sim.units[u].kind = self.sim.unit_types[rec].kind;
        self.sim.units[u].type_index = self.sim.unit_types[rec].type_index;
        self.sim.units[u].movement.speed = self.sim.type_speed(who, rec);
        // `Unit::set_type@00612fa0:235`: `update_hits`, then `update_los`,
        // `update_armor` and `update_speed` on the new type.
        self.sim.update_los(u);
        self.sim.units[u].movement.turning = self.sim.turning_for(rec);
        self.sim.reinit_guys_begin(u, rec);
    }
    fn init_kept(&mut self, i: i32, _ty: i32) {
        if let Ok(i) = usize::try_from(i)
            && i < self.guy_count()
        {
            self.sim.reinit_guy(self.u, self.rec, i);
        }
    }
    fn init_new(&mut self, i: i32, _ty: i32) {
        if let Ok(i) = usize::try_from(i) {
            self.sim.reinit_guy(self.u, self.rec, i);
        }
    }

    fn sub_object_set_type(&mut self, _ty: i32, _mode: i32) {}
    fn is_ceo(&mut self) -> bool {
        false
    }
    fn flags6c(&mut self) -> i32 {
        self.flags
    }
    fn set_flags6c(&mut self, f: i32) {
        self.flags = f;
    }
    fn update_ceo_position(&mut self) {}
    fn is_77(&mut self) -> bool {
        false
    }
    fn slot_15c(&mut self) {}
    fn slot_160(&mut self) {}
    fn update_armor(&mut self) {}
    fn update_speed(&mut self) {}
    fn update_gpiece(&mut self) {}
    fn seat_guy(&mut self, i: i32) {
        if i == 0 {
            self.sim.seat_guys(self.u);
        }
    }
    fn version_newer(&mut self) -> bool {
        false
    }
    fn is_supply(&mut self) -> bool {
        false
    }
    fn supply_id(&mut self) -> i32 {
        -1
    }
    fn close_supply(&mut self, _id: i32) {}
    fn open_supply(&mut self) {}
}

impl SimSetType<'_> {
    /// The figures the new type holds, which `reinit_guy` fills.
    fn guy_count(&self) -> usize {
        usize::try_from(self.total.unwrap_or(0)).unwrap_or(0)
    }
}
