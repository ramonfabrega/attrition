//! **`Army::normalize@006f9b50`'s steps** (item 3387): the army's recount
//! over its group slots and the sort of those slots by the leaders' type
//! field, as a pure walk over an [`ArmyNormalizeView`]. The sweep's port is
//! a view over the emulated original's windows.
//!
//! The crate's [`crate::Sim::army_normalize`] is a **namesake**: it keeps the
//! army as a list of units and recounts those, with no group slots, no
//! `remove_group` early return and no slot sort. This walk is the original's
//! own, call for call, and is **not wired**.
//!
//! The army holds `count` group ids (`i16` at `+0x96`) from `+0x54`, and five
//! totals at `+0xc..+0x20`. Pass one visits the slots from the last down:
//! `Group::normalize`, then a group with no members or with buildings is
//! removed from the army and the whole function returns (so pass two never
//! meets a group with buildings or none); otherwise the totals take its
//! counts. Pass two is an insertion sort of the slots by the leader unit's
//! type record `+0x14`, descending, whose outer index restarts just above
//! the last slot a swap landed in.

/// `GroupData::count`'s first argument: the decoys.
pub const COUNT_DECOYS: i32 = 6;
/// The casters.
pub const COUNT_CASTERS: i32 = 5;
/// The non-decoy types (the supply wagon `0x3f`, the AA gun `0x119`).
pub const COUNT_NON_DECOY_TYPE: i32 = 19;

/// What `Army::normalize` asks of the army, the groups and the units.
pub trait ArmyNormalizeView {
    /// The short at `+0x94`: the army's player.
    fn who(&self) -> i32;
    /// The short at `+0x96`: the number of group slots.
    fn count(&self) -> i32;
    /// The id in slot `s` (`+0x54 + 4 s`).
    fn slot(&self, s: i32) -> i32;
    /// Write the id in slot `s`.
    fn set_slot(&mut self, s: i32, id: i32);
    /// Total `t` (0 role mask, 1 members, 2 speed word, 3 standard, 4 decoys:
    /// `+0xc`, `+0x10`, `+0x14`, `+0x18`, `+0x1c`).
    fn total(&self, t: usize) -> i32;
    /// Write total `t`.
    fn set_total(&mut self, t: usize, v: i32);
    /// `Group::normalize@00711540` on group `g`.
    fn group_normalize(&mut self, g: i32);
    /// The group's `num` (`+0xc`).
    fn num(&self, g: i32) -> i32;
    /// The group's `buildings` byte (`+0x49`).
    fn buildings(&self, g: i32) -> u8;
    /// The group's `role` word (`+0x34`).
    fn role(&self, g: i32) -> i32;
    /// The group's `list[0]`, sign-extended.
    fn list0(&self, g: i32) -> i32;
    /// The group's virtual slot `+8` (its speed word).
    fn vcall8(&mut self, g: i32) -> i32;
    /// `GroupData::count(g, kind, a, b)`.
    fn group_count(&mut self, g: i32, kind: i32, a: i32, b: i32) -> i32;
    /// `GroupData::find_leader(g, 0)`.
    fn find_leader(&mut self, g: i32) -> i32;
    /// `Army::remove_group(id)`.
    fn remove_group(&mut self, id: i32);
    /// `units[who][unit]->type->+0x14`.
    fn unit_type_field(&self, who: i32, unit: i32) -> i32;
}

/// The leader the sort compares: -1 for a negative `num`, `list[0]` for a
/// group of buildings, else `find_leader`.
fn leader<V: ArmyNormalizeView>(v: &mut V, g: i32) -> i32 {
    if v.num(g) < 1 {
        -1
    } else if v.buildings(g) == 0 {
        v.find_leader(g)
    } else {
        v.list0(g)
    }
}

/// `Army::normalize@006f9b50`.
pub fn army_normalize_steps<V: ArmyNormalizeView>(v: &mut V) {
    for t in 0..5 {
        v.set_total(t, 0);
    }
    let mut s = v.count() - 1;
    while s >= 0 {
        let id = v.slot(s);
        if id >= 0 {
            v.group_normalize(id);
            let n = v.num(id);
            if n == 0 || v.buildings(id) != 0 {
                v.remove_group(id);
                return;
            }
            v.set_total(1, v.total(1).wrapping_add(n));
            let speed = v.vcall8(id);
            v.set_total(2, v.total(2).wrapping_add(speed));
            let decoys = v.group_count(id, COUNT_DECOYS, 0, 0);
            v.set_total(4, v.total(4).wrapping_add(decoys));
            let casters = v.group_count(id, COUNT_CASTERS, 0, 0);
            let before = v.total(3);
            let wagons = v.group_count(id, COUNT_NON_DECOY_TYPE, 0x3f, 0);
            let decoys2 = v.group_count(id, COUNT_DECOYS, 0, 0);
            let aa = v.group_count(id, COUNT_NON_DECOY_TYPE, 0x119, 0);
            let speed2 = v.vcall8(id);
            let standard = before
                .wrapping_sub(casters)
                .wrapping_sub(wagons)
                .wrapping_sub(decoys2)
                .wrapping_sub(aa);
            v.set_total(3, speed2.wrapping_add(standard));
            v.set_total(0, v.total(0) | v.role(id));
        }
        s -= 1;
    }
    let who = v.who();
    let mut outer = 1;
    while outer < v.count() {
        let g = v.slot(outer);
        if g >= 0 && v.num(g) != 0 {
            let mine = leader(v, g);
            let mut carried = outer;
            let mut i = outer - 1;
            while i >= 0 {
                let h = v.slot(i);
                if v.num(h) != 0 {
                    let theirs = leader(v, h);
                    if v.unit_type_field(who, theirs) <= v.unit_type_field(v.who(), mine) {
                        break;
                    }
                    let (a, b) = (v.slot(i), v.slot(carried));
                    v.set_slot(i, b);
                    v.set_slot(carried, a);
                    carried = i;
                    outer = i;
                }
                i -= 1;
            }
        }
        outer += 1;
    }
}
