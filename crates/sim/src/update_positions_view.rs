//! **`Group::update_positions@00713810`** (item 3379): the slot table rotated
//! once a frame from `do_group_move`'s leader arm, as a pure walk over an
//! [`UpdatePositionsView`]. The sweep's port is a view over the emulated
//! original's windows; the crate's is the view `group.rs` lays over a
//! `GroupState` and the leader unit (`docs/GROUPS.md`).
//!
//! The listing's order: the angle starts as the leader's heading (`+0x50`),
//! is replaced by `find_angle(dest − pos)` when the leader's head order
//! answers a record whose `+0x10` is set, and every slot below `form_num`
//! then gets `curr = rotate(off × 48, angle)`.

use crate::Sim;
use crate::movement::{Angle, find_angle};
use crate::world::Pos;

/// What `update_positions` reads and writes.
pub trait UpdatePositionsView {
    /// The leader's heading, `UnitData::angle`.
    fn heading(&mut self) -> i32;
    /// The leader's head order's move record when it carries a waypoint (its
    /// `+0x10` set): the point, `+0x2c`/`+0x30`.
    fn waypoint(&mut self) -> Option<(i32, i32)>;
    /// The leader's own point.
    fn leader_point(&mut self) -> (i32, i32);
    /// The group's `form_num`, the loop's bound.
    fn form_num(&mut self) -> i32;
    /// Slot `i`'s offset, `+0x4c`/`+0x24c`.
    fn slot_offset(&mut self, i: usize) -> (i32, i32);
    /// Slot `i`'s `curr`, `+0x44c`/`+0x64c`.
    fn set_curr(&mut self, i: usize, p: Pos);
}

/// The whole function.
pub fn update_positions_steps<V: UpdatePositionsView>(v: &mut V) {
    let mut theta = v.heading();
    if let Some((wx, wy)) = v.waypoint() {
        let (px, py) = v.leader_point();
        theta = find_angle(wx.wrapping_sub(px), wy.wrapping_sub(py)).0;
    }
    let n = v.form_num();
    for i in 0..n.max(0) as usize {
        let off = v.slot_offset(i);
        let curr = Sim::form_update_positions(&[off], Angle(theta));
        v.set_curr(i, curr[0]);
    }
}
