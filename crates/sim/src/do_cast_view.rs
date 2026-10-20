//! **`Unit::do_cast@005ebfe0`'s untargeted half** (item 3376), as a pure
//! walk over a [`DoCastClock`]: the head (the craft and the once-only
//! cost), the first frame's animation (pack, unpack, the hero's attacks, the
//! boat's berth) and the clock up to the craft's job time. The sweeps' ports
//! are views over the emulated original's windows; the crate's is
//! [`SimDoCast`], which `Sim::do_cast` runs.
//!
//! Every leaf is one virtual call, one direct callee or one inline read of
//! the listing, asked in the listing's order, which is what the sweeps hold.
//! The targeted half is [`crate::cast`]'s, dispatched before this one.
//!
//! The craft the rest acts on is **not** the order's own index when its type
//! is not a spell type: the original keeps the order's index for the cost
//! payment and for every question it asks the *type* (pack, unpack, the
//! board), and swaps in `0x296` for the spell-type row it reads the flags,
//! the job time and the cast from.

use crate::orders::spell;

/// `SpellType` row substituted for an order whose type is not a spell type.
pub const NOT_A_SPELL: i32 = 0x296;

/// `Locale` offsets of the two messages this function posts.
pub const MSG_REFUSED: i32 = 0x11fbc;
/// The rare collector's refusal to unpack where it stands.
pub const MSG_NO_SPOT: i32 = 0x11fd0;

/// What the head asks of the order, the type table and the caster.
pub trait DoCastHead {
    /// The order's own record, slot `+0x7c`'s answer: its `+0x20` index.
    fn order_spell(&mut self) -> i32;
    /// The same record's `+0x1c`: the cost is already paid.
    fn order_paid(&self) -> bool;
    /// `types.list[idx]->is_spell_type()`, the virtual slot `+0x48`.
    fn is_spell_type(&mut self, idx: i32) -> bool;
    /// `[this + 0xa]`, the caster's index in its player's units.
    fn caster_index(&self) -> i32;
    /// `[this + 9]`, the caster's player.
    fn caster_owner(&self) -> i32;
    /// `SpellType::pay_cast_costs(index, owner)` on the spell-type row at
    /// `idx`: nonzero refuses.
    fn pay_cast_costs(&mut self, idx: i32, index: i32, owner: i32) -> bool;
    /// The order record's `+0x1c = 1`.
    fn mark_paid(&mut self);
}

/// Where the head leaves the function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastHead {
    /// The cost was refused: the function ends (the feedback arm).
    Refused,
    /// On with the craft `craft` (the order's index, or [`NOT_A_SPELL`]),
    /// the original index being `idx`.
    Go { idx: i32, craft: i32 },
}

/// `005ebfe0`..`005ec0a0`: the craft, then the once-only cost.
pub fn do_cast_head_steps(v: &mut impl DoCastHead) -> CastHead {
    let idx = v.order_spell();
    let craft = if v.is_spell_type(idx) {
        idx
    } else {
        NOT_A_SPELL
    };
    if !v.order_paid() {
        let (index, owner) = (v.caster_index(), v.caster_owner());
        if v.pay_cast_costs(idx, index, owner) {
            return CastHead::Refused;
        }
        v.mark_paid();
    }
    CastHead::Go { idx, craft }
}

/// What the animation and the clock ask of the caster and its type.
pub trait DoCastClock: DoCastHead {
    /// The spell-type row at `craft`: `flags & 0xe == 0`, the craft is
    /// untargeted. Never asked of [`NOT_A_SPELL`].
    fn untargeted(&mut self, craft: i32) -> bool;
    /// The short at `this + 0x98`.
    fn spell_time(&self) -> i32;
    /// Its store.
    fn set_spell_time(&mut self, t: i32);
    /// `types.list[idx]->is_pack()`, the slot `+0x50`.
    fn type_is_pack(&mut self, idx: i32) -> bool;
    /// `types.list[idx]->is_unpack()`, the slot `+0x54`.
    fn type_is_unpack(&mut self, idx: i32) -> bool;
    /// `unit_masks & 0x80000`: the unit is packed.
    fn packed(&self) -> bool;
    /// `UnitData::is_merchant`.
    fn is_merchant(&mut self) -> bool;
    /// `scene->recalc_builds = 1`, then `Unit::update_gpiece` on the unit.
    fn merchant_repiece(&mut self);
    /// `Unit::set_angle(guy 0's angle, .., 0)`.
    fn face_guy0(&mut self);
    /// The caster's slot `+0xc4`, `is_hero`.
    fn caster_is_hero(&mut self) -> bool;
    /// `Unit::set_anim(anim, 0, p3)`.
    fn set_anim(&mut self, anim: i32, p3: bool);
    /// `options->rebuild = 1`.
    fn rebuild_options(&mut self);
    /// `UnitData::is_rare_collector`.
    fn is_rare_collector(&mut self) -> bool;
    /// `UnitData::good_merchant_spot` on the tile the unit stands on.
    fn good_merchant_spot_here(&mut self) -> bool;
    /// `Unit::set_new_location(cell centre, 1, 1)`, the cell the unit is in.
    fn seat_on_cell(&mut self);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// The caster's player is the console's.
    fn console_is_owner(&self) -> bool;
    /// `SoundGlobal::play(S_INVALID_ORDER)`, the message `msg` of the
    /// locale, `MessageWin::add_feedback` red.
    fn feedback(&mut self, msg: i32);
    /// `UnitType::find_nearby_spot` for the transport's berth answers
    /// nonzero: no place to board.
    fn board_spot_blocked(&mut self) -> bool;
    /// The unit type's slot `+0x10c`.
    fn unit_type_gate(&mut self) -> bool;
    /// `ObjectData::has_general(0, 0x162)`.
    fn has_general_162(&mut self) -> i32;
    /// The crew's hold: every figure's `+0x74` one up, held under `+0x78`.
    fn crew_hold_bump(&mut self);
    /// `SpellTypeData::get_job_time(index, owner)` of the row at `craft`.
    fn job_time(&mut self, craft: i32) -> i32;
}

/// `0x28a`, the transport craft: the one whose berth is asked.
const TRANSPORT: i32 = 0x28a;

/// Where the clock leaves the function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoCastFlow {
    /// The function returned: the order killed, or the clock still running.
    Ended,
    /// The clock reached the craft's job time on `idx`/`craft`.
    Due { idx: i32, craft: i32 },
    /// The craft is targeted: `crate::cast`'s half.
    Targeted { idx: i32, craft: i32 },
}

/// `005ebfe0`..`005ecdc1`: the head, the animation and the clock.
pub fn do_cast_clock_steps(v: &mut impl DoCastClock) -> DoCastFlow {
    let (idx, craft) = match do_cast_head_steps(v) {
        CastHead::Refused => {
            if v.console_is_owner() {
                v.feedback(MSG_REFUSED);
            }
            v.kill_current_order();
            return DoCastFlow::Ended;
        }
        CastHead::Go { idx, craft } => (idx, craft),
    };
    if craft != NOT_A_SPELL && !v.untargeted(craft) {
        return DoCastFlow::Targeted { idx, craft };
    }
    if v.spell_time() == 0 {
        let (anim, p3) = if v.type_is_pack(idx) {
            if v.packed() {
                v.kill_current_order();
                return DoCastFlow::Ended;
            }
            if v.is_merchant() {
                v.merchant_repiece();
            }
            v.face_guy0();
            (crate::anim::PACK, true)
        } else if v.type_is_unpack(idx) {
            v.rebuild_options();
            if !v.packed() {
                v.kill_current_order();
                return DoCastFlow::Ended;
            }
            if v.is_rare_collector() {
                if !v.good_merchant_spot_here() {
                    v.kill_current_order();
                    if v.console_is_owner() {
                        v.feedback(MSG_NO_SPOT);
                    }
                    return DoCastFlow::Ended;
                }
                v.seat_on_cell();
            }
            v.face_guy0();
            (crate::anim::UNPACK, true)
        } else if !v.caster_is_hero() {
            (crate::anim::DEFAULT, true)
        } else {
            // The hero's crafts are matched on the **craft** (`edi`), not on
            // the order's own index the type questions were asked of.
            match craft {
                spell::AMBUSH => (crate::anim::ATTACK1, false),
                spell::FORCED_MARCH => (crate::anim::ATTACK2, false),
                spell::RALLY => (crate::anim::ATTACK3, false),
                _ => (crate::anim::DEFAULT, true),
            }
        };
        v.set_anim(i32::from(anim), p3);
        if v.spell_time() == 0 && idx == TRANSPORT && v.board_spot_blocked() {
            v.kill_current_order();
            return DoCastFlow::Ended;
        }
    }
    if (v.type_is_pack(idx) || v.type_is_unpack(idx))
        && v.unit_type_gate()
        && v.has_general_162() >= 0
    {
        let t = v.spell_time();
        v.set_spell_time(t.wrapping_add(1));
        v.crew_hold_bump();
    }
    let t = i32::from((v.spell_time() as i16).wrapping_add(1));
    v.set_spell_time(t);
    let due = v.job_time(craft);
    if (t as i16) < (due as i16) {
        return DoCastFlow::Ended;
    }
    DoCastFlow::Due { idx, craft }
}

/// The crate's view: the order the unit holds at its head, and the craft's
/// mana onto `mana_burn` as `pay_cast_costs@00676c40` does for a unit caster.
pub struct SimDoCast<'a> {
    pub sim: &'a mut crate::Sim,
    pub u: usize,
    pub order: crate::orders::CastOrder,
}

impl DoCastHead for SimDoCast<'_> {
    fn order_spell(&mut self) -> i32 {
        self.order.spell
    }
    fn order_paid(&self) -> bool {
        self.order.paid
    }
    fn is_spell_type(&mut self, idx: i32) -> bool {
        (spell::FIRST..=spell::LAST).contains(&idx)
    }
    fn caster_index(&self) -> i32 {
        self.u as i32
    }
    fn caster_owner(&self) -> i32 {
        i32::from(self.sim.units[self.u].owner)
    }
    fn pay_cast_costs(&mut self, idx: i32, _index: i32, _owner: i32) -> bool {
        // `pay_cast_costs@00676c40`: a unit caster's `mana_burn` takes the
        // craft's `MANA` (`+0x1d0`) — Create Decoys' 1,000 on the General
        // (`docs/GOLDEN.md` §48); the pack, unpack and transport crafts ask
        // none. The refusal arm is not modelled: nothing refuses here.
        let mana = self.sim.spell(idx).map_or(0, |d| d.mana);
        let u = self.u;
        self.sim.units[u].mana_burn = self.sim.units[u]
            .mana_burn
            .saturating_add(i16::try_from(mana).unwrap_or(i16::MAX));
        false
    }
    fn mark_paid(&mut self) {
        if let Some(front) = self.sim.units[self.u].orders.front_mut()
            && let crate::orders::Body::Cast(c) = &mut front.body
        {
            c.paid = true;
        }
    }
}

impl DoCastClock for SimDoCast<'_> {
    fn untargeted(&mut self, _craft: i32) -> bool {
        // The targeted crafts were handed to `crate::cast` before this runs.
        true
    }
    fn spell_time(&self) -> i32 {
        i32::from(self.sim.units[self.u].spell_time)
    }
    fn set_spell_time(&mut self, t: i32) {
        self.sim.units[self.u].spell_time = t as i16;
    }
    fn type_is_pack(&mut self, idx: i32) -> bool {
        spell::is_pack(idx)
    }
    fn type_is_unpack(&mut self, idx: i32) -> bool {
        spell::is_unpack(idx)
    }
    fn packed(&self) -> bool {
        self.sim.units[self.u].combat.packed
    }
    fn is_merchant(&mut self) -> bool {
        self.sim.is_merchant(self.u)
    }
    fn merchant_repiece(&mut self) {
        // `recalc_builds` is the interface's; the figure is re-pieced as a
        // merchant reads packed while its current order is a pack
        // (`005d8530`, item 3369, `docs/AI.md` §285).
        self.sim.update_gpiece(self.u);
    }
    fn face_guy0(&mut self) {
        let facing = self.sim.units[self.u].movement.facing;
        self.sim.unit_set_angle(self.u, facing);
    }
    fn caster_is_hero(&mut self) -> bool {
        self.sim.is_hero_unit(self.u)
    }
    fn set_anim(&mut self, anim: i32, p3: bool) {
        self.sim.mark(crate::anim::SITE_CAST);
        self.sim.set_anim(self.u, anim as i8, false, p3);
    }
    fn rebuild_options(&mut self) {}
    fn is_rare_collector(&mut self) -> bool {
        self.sim.is_rare_collector(self.u)
    }
    fn good_merchant_spot_here(&mut self) -> bool {
        let tile = self.sim.units[self.u].pos.tile();
        self.sim.good_merchant_spot(self.u, tile)
    }
    fn seat_on_cell(&mut self) {
        let seat = crate::collide::ucell_centre(crate::collide::ucell(self.sim.units[self.u].pos));
        self.sim.set_new_location(self.u, seat, true);
    }
    fn kill_current_order(&mut self) {
        self.sim.kill_current_order(self.u);
    }
    fn console_is_owner(&self) -> bool {
        false
    }
    fn feedback(&mut self, _msg: i32) {}
    fn board_spot_blocked(&mut self) -> bool {
        let u = self.u;
        let barge = self.sim.transport_type_for(u);
        let spot = barge.and_then(|b| {
            self.sim.find_nearby_spot_type(
                b,
                self.sim.units[u].pos,
                0,
                self.sim.tuning.unit_board_distance,
                0,
                crate::transport::BOARD_BEARING,
            )
        });
        spot.is_none()
    }
    fn unit_type_gate(&mut self) -> bool {
        // Not modelled: the General's `has_general(0, 0x162)` clock step.
        false
    }
    fn has_general_162(&mut self) -> i32 {
        -1
    }
    fn crew_hold_bump(&mut self) {}
    fn job_time(&mut self, craft: i32) -> i32 {
        i32::from(self.sim.cast_job_time(self.u, craft))
    }
}
