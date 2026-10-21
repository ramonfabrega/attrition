//! **`ObjectData::count_inside@0064dda0`** (item 3391): how many of the
//! figures inside a building (or a ship) satisfy a `CountIndex` test, as a
//! pure walk over a [`CountInsideView`]. The crate keeps no function of this
//! name (its callers hold the answer behind a view leaf, as
//! `build_attack_view` does), so this module is the port; it is unwired and
//! the sweep (`sweep/object_data_count_inside.rs`) is what holds it.
//!
//! The walk is the container's inside list: from the object `this` names
//! (its owner byte and index), each hop reads the current object's owner
//! and link, and the next figure is `objects[owner][link]`; the list ends
//! on a negative link. A figure that is not alive (slot `+8`) is skipped
//! before the test. Every leaf below is one virtual call, one direct callee
//! or one read of the listing, asked in the order the test asks it, which
//! is what the sweep holds. `w` is a player and `i` an index in that
//! player's `objects` (and, the same index, `units`) array; the original
//! indexes both by one object index.

/// `CountIndex` (the type record's enum, `rise.pdb`).
pub const COUNT_WORKERS: i32 = 0;
pub const COUNT_PEASANTS: i32 = 1;
pub const COUNT_ATTACK: i32 = 2;
pub const COUNT_MILITARY: i32 = 3;
pub const COUNT_DOMAIN: i32 = 9;
pub const COUNT_ON_MAP: i32 = 10;
pub const COUNT_GATHER_PEASANTS: i32 = 11;
pub const COUNT_GATHER_WORKERS: i32 = 12;
pub const COUNT_GATHER_SCHOLARS: i32 = 13;
pub const COUNT_BUILD_PEASANTS: i32 = 14;
pub const COUNT_STRICT_TYPE: i32 = 16;
pub const COUNT_TYPE: i32 = 17;
pub const COUNT_TYPE_SAME_WHO: i32 = 18;
pub const COUNT_NON_DECOY_TYPE: i32 = 19;
pub const COUNT_CATEGORY: i32 = 20;
pub const COUNT_REGION: i32 = 21;
pub const COUNT_TRADE: i32 = 22;
pub const COUNT_GARRISON_ARROWS: i32 = 26;
pub const COUNT_WHO_INSIDE: i32 = 30;

/// The two peasant type indices (`+4` of the object's type record).
const PEASANT_TYPES: [i32; 2] = [0x32, 0x33];
/// `OrderIndex` `GATHER` and `BUILD_AT`, as `UnitData::action_type` answers.
const ORDER_GATHER: i32 = 7;
const ORDER_BUILD_AT: i32 = 6;
/// `MILITIA` and the type `ObjectData::is` is asked of a garrison arrow
/// count.
const MILITIA: i32 = 0x42;
/// The type `COUNT_TRADE` asks `is` after the slot `+0xd0` test.
const TRADE_TYPE: i32 = 0x13e;

/// What `ObjectData::count_inside` asks of the objects, the units, the
/// leaders and the world.
pub trait CountInsideView {
    /// `this`'s slot `+0xbc`: on the map.
    fn this_on_map(&mut self) -> bool;
    /// The byte `this + 9`.
    fn this_who(&self) -> i32;
    /// The short `this + 0xa`.
    fn this_idx(&self) -> i32;
    /// `objects[w][i]`'s short `+0x28`: the next figure's index.
    fn link(&self, w: i32, i: i32) -> i32;
    /// `objects[w][i]`'s signed byte `+0x3e`: the owner the next hop reads.
    fn owner(&self, w: i32, i: i32) -> i32;

    /// `objects[w][i]`'s slot `+8`.
    fn alive(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0xbc`.
    fn obj_on_map(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0x18`: a unit.
    fn obj_is_unit(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0xe8`: a captain.
    fn obj_captain(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0xd0`.
    fn obj_slot_d0(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0x148`, handed 4.
    fn obj_slot_148_4(&mut self, w: i32, i: i32) -> bool;
    /// `objects[w][i]`'s slot `+0xb8` handed `(ty, strict)`: `ObjectData::is`,
    /// which on the base implementation is its type record's slot `+0x60`.
    fn obj_is(&mut self, w: i32, i: i32, ty: i32, strict: i32) -> bool;
    /// `is_worker(objects[w][i])`.
    fn obj_is_worker(&mut self, w: i32, i: i32) -> bool;
    /// `is_map_unit(objects[w][i])`.
    fn obj_is_map_unit(&mut self, w: i32, i: i32) -> bool;
    /// The word `+0x4` of the type record at `objects[w][i] + 0x18`.
    fn obj_type_index(&self, w: i32, i: i32) -> i32;
    /// The word at `+off` of `objects[w][i]` (`0x10`, `0x14`: its position).
    fn obj_pos(&self, w: i32, i: i32, off: i32) -> i32;

    /// The word at `+off` of the type record `units[w][i] + 0x18` points at
    /// (`0x14` category, `0x1e8`, `0x1fc`, `0x218` domain).
    fn unit_type_word(&self, w: i32, i: i32, off: i32) -> i32;
    /// `units[w][i]`'s byte `+0x68 & 1`: a decoy.
    fn unit_decoy(&self, w: i32, i: i32) -> bool;
    /// `is_peasant(units[w][i])`.
    fn unit_is_peasant(&mut self, w: i32, i: i32) -> bool;
    /// `is_worker(units[w][i])`.
    fn unit_is_worker(&mut self, w: i32, i: i32) -> bool;
    /// `is_scholar(units[w][i])`.
    fn unit_is_scholar(&mut self, w: i32, i: i32) -> bool;
    /// `UnitData::action_type(units[w][i])`.
    fn unit_action_type(&mut self, w: i32, i: i32) -> i32;
    /// `units[w][i]`'s slot `+0x148` handed `0x20`, which on `has_objmask`
    /// is its type record's slot `+0xf0`.
    fn unit_objmask_20(&mut self, w: i32, i: i32) -> bool;
    /// The type record's slot `+0x10c`.
    fn unit_type_slot_10c(&mut self, w: i32, i: i32) -> bool;
    /// `units[w][i]`'s slot `+0xb8` handed `(ty, 0)`: `ObjectData::is`.
    fn unit_is(&mut self, w: i32, i: i32, ty: i32) -> bool;
    /// `units[w][i]`'s slot `+0x120`.
    fn unit_slot_120(&mut self, w: i32, i: i32) -> i32;

    /// `div_3_table[idx]`.
    fn div3(&self, idx: i32) -> i32;
    /// `WorldData::get_tregion(&x, &y)`, the two table words.
    fn tregion(&mut self, x: i32, y: i32) -> i32;

    /// `LeaderData::tribe_can_type(leaders[w], types.list[..])`.
    fn tribe_can_type(&mut self, w: i32) -> bool;
    /// The type list's slot `+0x68` handed `leaders[w]`'s word `+8`.
    fn list_slot_68(&mut self, w: i32) -> i32;
    /// `LeaderData::has_tech(leaders[w], ty)`.
    fn has_tech(&mut self, w: i32, ty: i32) -> bool;
    /// `LeaderData::current_upgrade(leaders[w], ty)`: an index in the unit
    /// types.
    fn current_upgrade(&mut self, w: i32, ty: i32) -> i32;
    /// `unittypes[ix]`'s word `+0x1e8`.
    fn unittype_1e8(&self, ix: i32) -> i32;
}

/// `ObjectData::count_inside(this, kind, param)`.
pub fn count_inside<V: CountInsideView>(v: &mut V, kind: i32, param: i32) -> i32 {
    if !v.this_on_map() {
        return 0;
    }
    let who = v.this_who();
    let (mut w, mut i) = (who, v.this_idx());
    let mut n = 0i32;
    while v.link(w, i) >= 0 {
        // The next figure: this object's owner and link.
        let (nw, ni) = (v.owner(w, i), v.link(w, i));
        w = nw;
        i = ni;
        if v.alive(w, i) {
            n = n.wrapping_add(count_one(v, kind, param, who, w, i));
        }
    }
    n
}

/// The test's contribution of one live figure `objects[w][i]`.
fn count_one<V: CountInsideView>(
    v: &mut V,
    kind: i32,
    param: i32,
    who: i32,
    w: i32,
    i: i32,
) -> i32 {
    let yes = |b: bool| i32::from(b);
    match kind {
        COUNT_WORKERS => yes(v.obj_is_worker(w, i)),
        COUNT_PEASANTS => yes(PEASANT_TYPES.contains(&v.obj_type_index(w, i))),
        COUNT_ATTACK => {
            yes(v.alive(w, i) && v.unit_type_word(w, i, 0x1e8) != 0 && v.obj_captain(w, i))
        }
        COUNT_MILITARY => yes(v.alive(w, i) && v.obj_slot_148_4(w, i)),
        COUNT_DOMAIN => yes(v.alive(w, i) && v.unit_type_word(w, i, 0x218) == param),
        COUNT_ON_MAP => yes(v.obj_on_map(w, i)),
        COUNT_GATHER_PEASANTS => yes(v.obj_is_unit(w, i)
            && v.unit_is_peasant(w, i)
            && v.unit_action_type(w, i) == ORDER_GATHER),
        COUNT_GATHER_WORKERS => yes(v.obj_is_unit(w, i)
            && v.unit_is_worker(w, i)
            && v.unit_action_type(w, i) == ORDER_GATHER),
        COUNT_GATHER_SCHOLARS => yes(v.obj_is_unit(w, i)
            && v.unit_is_scholar(w, i)
            && v.unit_action_type(w, i) == ORDER_GATHER),
        COUNT_BUILD_PEASANTS => yes(PEASANT_TYPES.contains(&v.obj_type_index(w, i))
            && v.unit_action_type(w, i) == ORDER_BUILD_AT),
        COUNT_STRICT_TYPE => yes(v.obj_is(w, i, param, 1)),
        COUNT_TYPE => yes(v.obj_is(w, i, param, 0)),
        COUNT_TYPE_SAME_WHO => yes(v.obj_is(w, i, param, 0) && w == who),
        COUNT_NON_DECOY_TYPE => {
            // A decoy (a unit with `+0x68` bit 0) counts nothing; the rest
            // are counted as `COUNT_TYPE` would.
            if v.obj_is_unit(w, i) && v.unit_decoy(w, i) {
                0
            } else {
                yes(v.obj_is(w, i, param, 0))
            }
        }
        COUNT_CATEGORY => yes(v.alive(w, i) && v.unit_type_word(w, i, 0x14) == param),
        COUNT_REGION => {
            if v.obj_is_map_unit(w, i) {
                let y = v.div3((v.obj_pos(w, i, 0x14) ^ 0x63637) >> 6);
                let x = v.div3((v.obj_pos(w, i, 0x10) ^ 0x63637) >> 6);
                yes(v.tregion(x, y) == param)
            } else {
                0
            }
        }
        COUNT_TRADE => yes(v.obj_slot_d0(w, i) || v.obj_is(w, i, TRADE_TYPE, 0)),
        COUNT_GARRISON_ARROWS => arrows(v, who, w, i),
        COUNT_WHO_INSIDE => yes(w == param),
        _ => 0,
    }
}

/// `COUNT_GARRISON_ARROWS`: the arrows a figure adds to a garrisoned
/// building: a captain unit (not a decoy) that fires from inside, a worker
/// who may be drafted as militia counting the militia's attack, anything else
/// a tenth of its attack (halved when its type has no `+0x1fc`).
fn arrows<V: CountInsideView>(v: &mut V, who: i32, w: i32, i: i32) -> i32 {
    if !v.obj_is_unit(w, i) || !v.obj_captain(w, i) || v.unit_decoy(w, i) {
        return 0;
    }
    if !v.unit_objmask_20(w, i) || v.unit_type_slot_10c(w, i) {
        return 0;
    }
    // Whether the figure garrisons as militia.
    let militia = if v.unit_is_worker(w, i) {
        let ty = if v.tribe_can_type(w) {
            MILITIA
        } else {
            v.list_slot_68(w)
        };
        v.has_tech(w, ty)
    } else {
        false
    };
    if !militia && !v.unit_is(w, i, MILITIA) {
        let mut a = v.unit_slot_120(w, i) / 10;
        if v.unit_type_word(w, i, 0x1fc) == 0 {
            a = (a + 1) / 2;
        }
        return a;
    }
    let up = v.current_upgrade(who, MILITIA);
    (v.unittype_1e8(up) / 10 + 2) / 3
}
