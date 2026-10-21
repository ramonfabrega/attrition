//! **`Group::action_repair@007020c0`'s steps** (item 3411): the `repair`
//! command's group order, as a pure walk over a [`RepairView`]. The command
//! (`CommandPackage::process_repair@00948cb0`, type `0x10`) is issued by
//! nothing a player does (`docs/GOLDEN.md` §29), so the crate has no caller:
//! this port is **unwired**, the view of the swept original's windows being
//! its only implementor.
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it, which is what the sweep holds. `b` is
//! the building's index in `who`'s object array (`param_1`, `param_2`); the
//! group's own player is [`GarrisonHead::who`], which the member walk indexes
//! the objects and units by (the first tregion and the final unit reset index
//! them by the *building's* player, as the listing does).
//!
//! `queue` is the original's numbering: `0` `QUEUE_FIRST` (the re-entry),
//! `1` `QUEUE_LAST`, anything else the `QUEUE_NEW` arm.

use crate::garrison_view::{GarrisonHead, garrison_head};

/// What `Group::action_repair` asks of the group, the world and the objects.
/// `depth` is the `QUEUE_FIRST` re-entry's: its frame sits `0x64` bytes
/// deeper, which the swept stubs record in the stack locals' addresses.
pub trait RepairView: GarrisonHead {
    /// `Group::set_up_insert(&list)`.
    fn set_up_insert(&mut self);
    /// `Group::action_halt(0)`.
    fn action_halt(&mut self);
    /// `Group::finish_insert(&list)`.
    fn finish_insert(&mut self);
    /// The local order list's `clear`.
    fn list_clear(&mut self);
    /// `this + 0x10 = -1`: the group's formation is forgotten.
    fn set_form(&mut self);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `WorldData::get_tregion` of `objects[p][o]`'s coordinates (each
    /// `(field ^ 0x63637) >> 6` through `div_3_table`); the `first` call is
    /// the building's, the others the members'.
    fn tregion(&mut self, p: i32, o: i32, first: bool, depth: i32) -> i32;
    /// `objects[who][m]`'s slot `+0x8`.
    fn object_slot8(&mut self, who: i32, m: i32) -> i32;
    /// `objects[who][m]`'s slot `+0xbc`.
    fn object_slot_bc(&mut self, who: i32, m: i32) -> i32;
    /// `*(units[who][m] + 0x18) + 4`: the unit's type index.
    fn unit_type(&self, who: i32, m: i32) -> i32;
    /// `Region::is_coast(regions[member_tregion], building_tregion)`.
    fn is_coast(&mut self, member_tregion: i32, building_tregion: i32) -> i32;
    /// `SpellTypeData::is_castable(spelltypes[0xa4c], m, who, 0)`.
    fn is_castable(&mut self, m: i32, who: i32) -> i32;
    /// `Unit::add_cast_order(units[who][m], -1, -1, x, y, 0x293, queue, 1)`,
    /// `x`, `y` the member's own object's coordinates `^ 0x63637`.
    fn add_cast_order(&mut self, who: i32, m: i32, queue: i32);
    /// `Unit::add_repair_order(units[who][m], b, bwho, queue, 1)`.
    fn add_repair_order(&mut self, who: i32, m: i32, b: i32, bwho: i32, queue: i32);
    /// The reset of `units[bwho][b]` (the building's unit, as the listing
    /// has it): `+0x68 &= 0xfbffffff`, `+0xc0 = 0`, `close_orders(0)`,
    /// `clear_partial_path`, `update_action`.
    fn reset_unit(&mut self, bwho: i32, b: i32);
}

/// `Group::action_repair(b, bwho, queue)`. `depth` counts the `QUEUE_FIRST`
/// re-entries above this call.
pub fn action_repair_steps<V: RepairView>(v: &mut V, b: i32, bwho: i32, queue: i32, depth: i32) {
    garrison_head(v);
    if queue == 0 {
        // The insert dance: the group halts, takes the repair at `QUEUE_NEW`
        // and re-issues what it was doing (`finish_insert`).
        v.set_up_insert();
        v.action_halt();
        action_repair_steps(v, b, bwho, 2, depth + 1);
        v.finish_insert();
        v.list_clear();
        return;
    }
    v.set_form();
    let building = v.tregion(bwho, b, true, depth);
    let mut i = 0;
    while i < v.count() {
        let m = v.member(i);
        let who = v.who() & 0xff;
        if v.object_slot8(who, m) != 0 && v.object_slot_bc(who, m) != 0 {
            let t = v.unit_type(who, m);
            if t == 0x32 || t == 0x33 {
                let mt = v.tregion(who, m, false, depth);
                if v.is_coast(mt, building) != 0 {
                    if queue == 1 {
                        if v.is_castable(m, who) != 0 {
                            v.add_cast_order(who, m, 1);
                        }
                        v.add_repair_order(who, m, b, bwho, 1);
                    } else {
                        v.add_repair_order(who, m, b, bwho, 2);
                        if v.is_castable(m, who) != 0 {
                            v.add_cast_order(who, m, 0);
                        }
                    }
                    v.reset_unit(bwho, b);
                }
            }
        }
        i += 1;
    }
}
