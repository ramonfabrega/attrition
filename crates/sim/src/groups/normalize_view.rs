//! **`Group::normalize@00711540`'s steps** (item 3387): the prune of a group's
//! member list, last to first, then `find_role` and the speed tail, as a pure
//! walk over a [`NormalizeView`] (`docs/GROUPS.md` §4.3, §18.1). The sweep's
//! port is a view over the emulated original's windows
//! (`sweep::group_normalize`).
//!
//! **Not wired.** The crate's `Sim::group_normalize` and `Sim::seat_normalize`
//! are namesakes: they prune a `Vec<usize>` of unit indices by `alive` (and,
//! seated, the back-pointer) and keep three parallel arrays, where the original
//! keeps five (member short, a byte and four words a member) and tests the
//! object's slots `+0x18`, `+0x20` and the group's `id` and `+0x30`. These
//! steps are the original's reading, not the crate's behaviour.
//!
//! A member is dropped when its short is negative, or when the object's byte
//! `+8` has bit 0 clear, or when all of: `+0x30 == 0`, `id >= 0`, slot `+0x18`
//! answers 0 or the unit's short `+0x80` is not `id`, and slot `+0x20` answers 0.
//! Each drop shifts the later entries of all five arrays down and decrements
//! the count. The tail is `find_role`; then, with `+0x49 == 0`, members left
//! and a leader (`find_leader(NULL)` at least 0), both speeds are the leader's
//! `UnitData::speed`, else 0.

/// What `normalize` reads and does. `m` is a member id (the short at `+0x8cc`).
pub trait NormalizeView {
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// Writes `[this + 0xc]`.
    fn set_count(&mut self, n: i32);
    /// The short `i` of the list at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `objects[who][m]`'s byte `+8`, bit 0.
    fn obj_valid(&mut self, m: i32) -> bool;
    /// `[this + 0x30]`.
    fn f30(&self) -> i32;
    /// `[this + 4]`, the group's id.
    fn id(&self) -> i32;
    /// The object's vtable slot `+0x18`.
    fn obj_slot18(&mut self, m: i32) -> i32;
    /// The object's vtable slot `+0x20`.
    fn obj_slot20(&mut self, m: i32) -> i32;
    /// `units[who][m]`'s short `+0x80`.
    fn unit_group(&mut self, m: i32) -> i32;
    /// Entry `j` of all five arrays takes entry `j + 1`'s.
    fn shift_down(&mut self, j: i32);
    /// `Group::find_role`.
    fn find_role(&mut self);
    /// The byte `+0x49`.
    fn f49(&self) -> i32;
    /// `GroupData::find_leader(NULL)`.
    fn find_leader(&mut self) -> i32;
    /// `UnitData::speed` of `units[who][l]`.
    fn unit_speed(&mut self, l: i32) -> i32;
    /// Writes the pair `+0x3c` and `+0x40`.
    fn set_speed(&mut self, v: i32);
}

/// `Group::normalize(this)`: the steps.
pub fn normalize_steps<V: NormalizeView>(v: &mut V) {
    let mut i = v.count() - 1;
    while i >= 0 {
        let m = v.member(i);
        let drop = if m < 0 || !v.obj_valid(m) {
            true
        } else {
            v.f30() == 0
                && v.id() >= 0
                && (v.obj_slot18(m) == 0 || v.unit_group(m) != v.id())
                && v.obj_slot20(m) == 0
        };
        if drop {
            let mut j = i;
            while j < v.count() - 1 {
                v.shift_down(j);
                j += 1;
            }
            let n = v.count() - 1;
            v.set_count(n);
        }
        i -= 1;
    }
    v.find_role();
    if v.f49() == 0 && v.count() != 0 && v.count() > 0 {
        let l = v.find_leader();
        if l >= 0 {
            let s = v.unit_speed(l);
            v.set_speed(s);
            return;
        }
    }
    v.set_speed(0);
}
