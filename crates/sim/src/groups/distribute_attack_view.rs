//! `Group::distribute_attack@00713390`: "unless at least two of us can already hit it, everyone drops what
//! they are doing and attacks it" (`docs/GROUPS.md` §14, the blind list; `docs/audit/2026-08-25-groups.md`
//! B.44), swept against the original under `tools/emu` (item 3381, `sweep::group_distribute_attack`).
//!
//! The function is two walks of the group's member ids (the shorts at `+0x8cc`, `count` at `+0xc`, read afresh
//! each step), after one `Group::normalize`. A member is *eligible* when the object's slots `+8` and `+0xbc`
//! are both true, its `+0xc0` slot says it is not an excluded plane (inline `UnitData::is_plane`: type
//! `+0x218 != 2`, or type `+0x2b4 & 0x20`; else the slot's call answers zero), and its current order (`+0xdc`
//! list head non-null, the cache `+0xd4/+0xcc/+0xd0` refreshed from the head's node, `+0xcc` non-null) answers
//! non-zero on slot `+0x18`.
//!
//! Walk one, for each eligible member: `Unit::update_order`'s slot-`+0x50` record is read (the first eligible
//! member's byte `+0x1c` and the `& 4` of the byte at `rec + 8 + [[rec+4]+4]` are kept, with its id) and the
//! member's own `is_in_range(p1, p2, x^0x63637, y^0x63637, unit, 0, 0)` counted; out of range, the record's
//! `+8` and `+0xc` are set to `-1`. Walk two runs only when fewer than two were in range and an eligible
//! member was seen: each eligible member `kill_current_order(0)`s and `add_attack_order(p1, p2, QUEUE_FIRST
//! (0), kept byte, kept flag)`s.
//!
//! **Not wired**: this crate has no `Group::distribute_attack`; `group_move::GroupMoveView::distribute_attack`
//! stays the no-op `SimGroupMove` gives it (no traced game has run it), so `Sim` does not call these steps yet.
//! The view is the swept reading; the record's Sim counterpart (`combat.target`/`mandatory` for `+8/+0xc/+0x1c`;
//! the `& 4` flag has none named) is parked.

/// `QUEUE_FIRST`, the queue position the second walk pushes.
pub const QUEUE_FIRST: i32 = 0;
/// The xor the original keeps a unit's position under (`ObjectData +0x10/+0x14`).
pub const COORD_KEY: i32 = 0x63637;

/// What `distribute_attack` reads and does, one member id at a time.
pub trait DistributeView {
    /// The record `Unit::update_order(..)`'s slot `+0x50` answers.
    type R: Copy;
    /// `Group::normalize(this)`.
    fn normalize(&mut self);
    /// `this->count` (`+0xc`), read at each loop test.
    fn count(&mut self) -> i32;
    /// The `i`th member id (the short at `+0x8cc + 2i`, sign-extended).
    fn member(&mut self, i: i32) -> i32;
    /// Both of the object's slots `+8` and `+0xbc` answer non-zero.
    fn object_live(&mut self, id: i32) -> bool;
    /// The unit's `+0xc0` gate: `true` when it is not an excluded plane.
    fn plane_gate(&mut self, id: i32) -> bool;
    /// `+0xdc` set, the cache refreshed, `+0xcc` set, and its slot `+0x18` non-zero.
    fn live_order(&mut self, id: i32) -> bool;
    /// `Unit::update_order(unit)`, then its slot `+0x50`.
    fn update_record(&mut self, id: i32) -> Self::R;
    /// The record's byte `+0x1c`.
    fn rec_byte(&mut self, r: Self::R) -> i32;
    /// The record's `& 4` flag: byte at `rec + 8 + [[rec+4]+4]`, sign-extended.
    fn rec_flag(&mut self, r: Self::R) -> i32;
    /// The record's `+8` and `+0xc` set to `-1`.
    fn rec_clear(&mut self, r: Self::R);
    /// `ObjectData::is_in_range(unit, p1, p2, x ^ KEY, y ^ KEY, unit, 0, 0)`.
    fn in_range(&mut self, id: i32, p1: i32, p2: i32) -> bool;
    /// `Unit::kill_current_order(unit, 0)`.
    fn kill_current_order(&mut self, id: i32);
    /// `Unit::add_attack_order(unit, p1, p2, queue, mandatory, flag)`.
    fn add_attack_order(
        &mut self,
        id: i32,
        p1: i32,
        p2: i32,
        queue: i32,
        mandatory: i32,
        flag: i32,
    );
}

fn eligible<V: DistributeView>(v: &mut V, id: i32) -> bool {
    v.object_live(id) && v.plane_gate(id) && v.live_order(id)
}

/// The function's whole body.
pub fn distribute_attack_steps<V: DistributeView>(v: &mut V, p1: i32, p2: i32) {
    let mut hits = 0;
    let mut kept_byte = 0;
    let mut kept_flag = 0;
    let mut first = -1;
    v.normalize();
    if v.count() <= 0 {
        return;
    }
    let mut i = 0;
    loop {
        let id = v.member(i);
        if eligible(v, id) {
            let r = v.update_record(id);
            if first < 0 {
                kept_byte = v.rec_byte(r);
                kept_flag = v.rec_flag(r);
                first = id;
            }
            if v.in_range(id, p1, p2) {
                hits += 1;
            } else {
                v.rec_clear(r);
            }
        }
        i += 1;
        if i >= v.count() {
            break;
        }
    }
    if hits < 2 && first >= 0 && v.count() > 0 {
        let mut i = 0;
        loop {
            let id = v.member(i);
            if eligible(v, id) {
                v.kill_current_order(id);
                v.add_attack_order(id, p1, p2, QUEUE_FIRST, kept_byte, kept_flag);
            }
            i += 1;
            if i >= v.count() {
                break;
            }
        }
    }
}
