//! [`crate::Sim::unit_set_type`]'s adapter to [`crate::units::unit_set_type_view`]
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
//! Not carried by this adapter: the table unit's `is(0x134)` (read yes, the
//! crate's old count) and governor-hero answers (no, as the muster count's gap), the `+0x6c`
//! flags but the marine mark and the CEO bit (item 4502), the vslots `+0x15c`/`+0x160`, the armor and
//! speed refreshes the original runs here, and the supply tail
//! (`game.version` is not modelled): the crate keeps no state for them.

use crate::units::anim::SQUAD_SIZE;
use crate::units::unit_set_type_view::UnitSetTypeView;
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
        let flags = sim.units[u].unit_masks2 as i32;
        SimSetType {
            sim,
            u,
            rec,
            total: None,
            flags,
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
        self.cur()
            .map_or(0, |t| self.sim.tables.unit_types[t].price.pop)
    }
    /// `is(0x134)` on the unit's own type: the population-0 arm of the gate
    /// both count blocks share with `Unit::close` and the birth
    /// ([`Sim::type_counts_in_muster`]). It answered yes until item 4159's
    /// eleventh residue, which counted every captain whatever its
    /// population: a Transport Barge (`POP` 0) is born uncounted, and its
    /// age upgrade to a Transport Galleon took it off `num_units[270]` and
    /// put it on `[271]` — run600's who=1 held −2 and 2 from 11282, where
    /// the original holds 0 and 0.
    ///
    /// SEAM: answers no, as [`Sim::close_count`] and the birth do; no
    /// population-0 type of the Fighter Bomber's line is set here.
    fn table_unit_is_134(&mut self) -> bool {
        false
    }
    /// `UnitData::is_gov_hero@0046ce70` on the unit's own type
    /// ([`Sim::type_is_hero`]): a government patriot (`POP` 0) is counted
    /// on both sides of a type change, as it is at birth and at its close.
    fn table_unit_is_gov_hero(&mut self) -> bool {
        self.cur().is_some_and(|t| self.sim.type_is_hero(t))
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
            self.sim.tables.unit_types[t]
                .cols
                .flag2(crate::ai::ai_load::uflags2::FLAG2_CARAVAN)
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
        self.sim.tables.unit_types[ty as usize].combat.crew_size
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
        let damage = self.sim.units[u].max_health - self.sim.units[u].health;
        let hits = self.sim.own_hits(u, rec);
        {
            let unit = &mut self.sim.units[u];
            unit.ty = Some(rec);
            unit.max_health = hits;
            unit.health = (hits - damage).max(1);
        }
        self.sim.units[u].kind = self.sim.tables.unit_types[rec].kind;
        self.sim.units[u].type_index = self.sim.tables.unit_types[rec].type_index;
        self.sim.units[u].movement.speed = self.sim.own_speed(u, rec);
        self.sim.update_armor(u);
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
        self.sim.speed_is(self.u, 0x165, true)
    }
    fn flags6c(&mut self) -> i32 {
        self.flags
    }
    fn set_flags6c(&mut self, f: i32) {
        self.flags = f;
        // The marine mark (`is(0x77)`'s `| 0x800`) is the one bit of the
        // word this crate keeps from here (item 3661).
        if f & crate::combat::umask2::MARINE as i32 != 0 {
            self.sim.units[self.u].unit_masks2 |= crate::combat::umask2::MARINE;
        }
        // The CEO bit, set and cleared (item 4502).
        let ceo = crate::combat::umask2::CEO_BIT;
        if f as u32 & ceo != 0 {
            self.sim.units[self.u].unit_masks2 |= ceo;
        } else {
            self.sim.units[self.u].unit_masks2 &= !ceo;
        }
    }
    fn update_ceo_position(&mut self) {
        let here = self.sim.units[self.u].pos.tile();
        self.sim.update_ceo_position(self.u, None, Some(here));
    }
    fn is_77(&mut self) -> bool {
        self.sim
            .speed_is(self.u, crate::spells::cast_transport::MARINES, false)
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
        // **The seating writes the figures' `last`** (item 4263): the
        // call is `Guy::set_new_location(guy i, x, y, 1)`, whose
        // `param_3` copies the point into `last_x`/`last_y` (`005d86f0:66–
        // 70`) and writes no angle, so guy `i` keeps its `last_angle`. On
        // guy 0 the same call seats each crew figure past the squad with
        // `Guy::set_angle(crew, guy 0's angle, 1)` (`+0x18` and `+0x1c`) and
        // `set_new_location(crew, its point, 1)`, so the crew's `last` is
        // its seated point and guy 0's facing. A unit upgraded on a frame
        // it walked ends it with `last` on its point, not one step behind
        // (run1597, frame 11884: the Hoplites `1/56..1/58` to Phalanx).
        let Ok(i) = usize::try_from(i) else { return };
        let unit = &mut self.sim.units[self.u];
        let (pos, facing) = (unit.pos, unit.movement.facing);
        if let Some(g) = unit.guys.get_mut(i) {
            let angle = g.last.map_or(facing, |(_, a)| a);
            g.last = Some((pos, angle));
            g.seat_heights();
        }
        if i == 0 {
            for g in unit.guys.iter_mut().skip(SQUAD_SIZE) {
                let at = g.follow.map_or(pos, |f| f.body.pos);
                g.last = Some((at, facing));
                g.seat_heights();
            }
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

impl Sim {
    /// **`Unit::set_type`'s crew loop, on the occupancy index**
    /// (`docs/AI.md` §120). `Unit::set_type@00612fa0`'s second kill loop
    /// walks the figures from the **old** type's `squad_size` to the
    /// unit's whole count and calls `Objects::kill_guy@00659410` on each,
    /// and `kill_guy`'s last arm clears `radius[coll_size]` around the
    /// *figure's own cell* — `CollBlock::set(…, 0)` over the same disc
    /// `add_to_world` set, by the unit's **old** type, since
    /// `set_type` swaps `+0x18` only after the loop. Nothing puts the
    /// cells back but the sixty-fourth-frame repaint, which marks the
    /// *new* disc: a Trebuchet (size 3, trackless crew standing on guy 0)
    /// that becomes a Bombard (size 2) leaves no ring of its old block
    /// and a hole in its new one until the next repaint.
    ///
    /// SEAM: `kill_guy`'s own gates before the clear — the figure's type
    /// is not the crash kind (`+0x14 == 8`), its domain is not air, and the
    /// object type's vslot `+0x120` answers 0 — are read as true;
    /// [`Self::coll_paint_in`] gates air and a size of 0.
    ///
    /// **The region gate is the figure's world cell's plain `region`**
    /// (item 4567, `docs/COLLISION.md` §2): `Objects::kill_guy@00659410`
    /// reads `WData +0x4` of the cell under the figure's unit cell and
    /// clears a disc cell only where that cell's `region` is the same (or
    /// the figure's is negative). It asks no `get_tregion`, so a figure on
    /// the ocean half of a coastal cell clears by the cell's land region,
    /// where `coll_paint`'s gate answers the sea's `region2` and clears
    /// nothing (run1783's President `1/14`, aboard on 2983: figure 2 at
    /// unit cell (299, 640)).
    ///
    /// One figure of the crew loop: `Objects::kill_guy`'s clear around
    /// figure `i`'s own cell, called for each figure from the squad to the
    /// old total (`unit_set_type_view`) — and for every figure of a unit
    /// that dies, by `close_unit` (`Unit::close`'s two loops, item 3787,
    /// `docs/COLLISION.md` §2).
    pub(crate) fn coll_kill_guy(&mut self, u: usize, i: usize) {
        use crate::movement::collide::{UCELLS_PER_CELL, ucell};
        let body = self.units[u].movement.body.pos;
        let c = ucell(self.units[u].guys[i].follow.map_or(body, |f| f.body.pos));
        let n = UCELLS_PER_CELL;
        let cell = crate::world::Cell::new(c.x.div_euclid(n), c.y.div_euclid(n));
        let region = self.world.region_of(cell);
        self.coll_paint_in(u, c, region, false);
    }
}
