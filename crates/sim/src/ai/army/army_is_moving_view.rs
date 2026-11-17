//! **`Army::is_moving@006f5470`'s steps**, as a walk over an
//! [`ArmyIsMovingView`]: [`crate::Sim::army_is_moving`] is this walk over the
//! army's slot, and the sweep's port is this walk over the emulated
//! original's windows (`Sim::is_moving` of `attack_pos.rs`, the
//! `UnitData::is_moving` of one unit's head order, is a namesake and not
//! this).
//!
//! The army's `count` units are visited in `get_unit` order. A unit that is
//! not active and on the map, or has no head order, adds nothing. Otherwise
//! the head order's `is_move` decides: a unit **not** walking counts when it
//! stands outside the army's region, and a unit walking counts when it is
//! within `0xf00` of the centre of gravity (asked once, on the first such
//! unit). More than a sixth of `count` is moving.

use crate::world::Pos;

/// What `Army::is_moving` asks of the army, the units and the world.
pub trait ArmyIsMovingView {
    /// The word at `+0x10`: the army's unit count (loop bound and divisor).
    fn count(&self) -> i32;
    /// `ArmyData::get_unit` — the unit at position `i`, negative for none.
    fn unit(&mut self, i: i32) -> i32;
    /// The unit's vslot `+0x8`.
    fn is_active(&mut self, u: i32) -> bool;
    /// The unit's vslot `+0xbc`.
    fn is_on_map(&mut self, u: i32) -> bool;
    /// The head order's `is_move` (vslot `+0x14`), `None` when the unit has
    /// no head order.
    fn head_is_move(&mut self, u: i32) -> Option<bool>;
    /// Whether the unit's region is not the army's region (`+0x8`).
    fn outside_region(&mut self, u: i32) -> bool;
    /// `Army::center_of_gravity`, as a point.
    fn centre_of_gravity(&mut self) -> Pos;
    /// `vector_dist` from the unit to `c`.
    fn dist_to(&mut self, u: i32, c: Pos) -> i32;
}

/// `Army::is_moving@006f5470`.
pub fn army_is_moving<V: ArmyIsMovingView>(v: &mut V) -> bool {
    let count = v.count();
    let mut cog: Option<Pos> = None;
    let mut moving = 0;
    for i in 0..count.max(0) {
        let u = v.unit(i);
        if u < 0 {
            continue;
        }
        if !v.is_active(u) || !v.is_on_map(u) {
            continue;
        }
        let Some(is_move) = v.head_is_move(u) else {
            continue;
        };
        if !is_move {
            if v.outside_region(u) {
                moving += 1;
            }
            continue;
        }
        let c = match cog {
            Some(c) => c,
            None => *cog.insert(v.centre_of_gravity()),
        };
        if v.dist_to(u, c) <= 0xf00 {
            moving += 1;
        }
    }
    count / 6 < moving
}
