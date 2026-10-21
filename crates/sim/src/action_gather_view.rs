//! **`Group::action_gather@00700b90`'s steps** (item 3386): a group's order to
//! gather at a building, as a walk over a [`GatherView`]. The control flow is
//! the original's, call for call and in its order; the view is the state it
//! reads and the callees it makes, which `sweep/group_action_gather.rs`
//! answers from the emulated original's windows.
//!
//! **Not wired.** `group.rs`'s `FinishInsertView::action_gather` is an empty
//! stub, and the crate has no `Group::add`-into-a-scratch-group nor
//! `Group::action_move_near` to hand the scholars to; wiring it would add
//! gather and move orders to every capture whose human clicks a group onto a
//! building, so it is parked rather than done here. The chain pieces the crate
//! does have (`Sim::add_gather_order`, `rally::action_gather_point`) are the
//! callees' twins, not this function.
//!
//! ```text
//! head      a scratch group is cleared; the scenario's ignore-orders sweep and the group's refresh
//! counts    the building asks `is(0x1a4)`: scholars are counted (type 0x34) when it does, the gatherers
//!           that can gather less the workers and the scholars when it does not; the building's data kind
//!           picks a good the leader must have available (`type_avail`), else the order ends
//! members   from the last: alive and `+0xbc`; a unit that is not the building's kind goes to the scratch
//!           group; one that is, and (unless every gatherer is busy at kind 7) stands in the building's
//!           tile region, is sent to the first free object of its own along a spiral of 0x78 steps
//!           (`MOVE_289`, 0x300 apart) from the building: the gather order, the spell cast first or after,
//!           and the editor's instant garrison when the game's semaphore says so
//! spiral    a step off the map, in another region, or on a tile with no usable object retries the same
//!           member; the 0x79th hands every member left to the scratch group
//! tail      the scratch group, if it has anyone, walks near the building (`action_move_near`)
//! ```

pub use crate::garrison_view::{GarrisonHead, garrison_head};

/// The key the original xors into every stored coordinate.
pub const POS_KEY: i32 = 0x63637;
/// `GroupData::count`'s kinds, as the listing pushes them.
pub const COUNT_TYPE: i32 = 0x11;
/// The type the scholar branch counts.
pub const TYPE_SCHOLAR_COUNT: i32 = 0x34;
/// The spiral's last usable step is `SPIRAL_STEPS - 1`; the next hands the rest over.
pub const SPIRAL_STEPS: i32 = 0x79;

/// What `Group::action_gather` asks of the group, the world and the objects.
/// `p` is a player and `o`/`u` an index in that player's `objects`/`units`
/// array (the original indexes both by one object index).
pub trait GatherView: GarrisonHead {
    /// `Group::clear(scratch, -1)`.
    fn list_clear(&mut self);
    /// `this + 0x10 = -1`: the group's formation is forgotten.
    fn set_form(&mut self);
    /// `[this + 0xc]`, read afresh each time.
    fn count(&self) -> i32;
    /// The sign-extended short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `GroupData::count(kind, ty, 0)`.
    fn count_kind(&mut self, kind: i32, ty: i32) -> i32;

    /// `[world]`: the width in tiles.
    fn world_w(&self) -> i32;
    /// `[world + 0x18]`, the bound on x in units of `0xc0`.
    fn world_dim_x(&self) -> i32;
    /// `[world + 0x1c]`, the bound on y.
    fn world_dim_y(&self) -> i32;
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// The sign-extended short at `off` of tile cell `idx` (stride `0x1c`).
    fn cell_i16(&self, idx: i32, off: i32) -> i32;
    /// The byte of the world's second array (`+0x138`) at element `i` (stride 2).
    fn ext_byte(&self, i: i32) -> i32;
    /// `WorldData::get_tregion(&div3[x >> 6], &div3[y >> 6])`; the arguments
    /// are the table's words.
    fn get_tregion(&mut self, tx: i32, ty: i32) -> i32;

    /// `objects[p][o]`'s `+0x10` and `+0x14`, before the xor.
    fn obj_pos_raw(&self, p: i32, o: i32) -> (i32, i32);
    /// Slot `+0x8` of `objects[p][o]`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xbc` of `objects[p][o]`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]` `is(ty, 0)`: the object's slot `+0xb8`, or on a
    /// plain `ObjectData::is` its type's slot `+0x60`.
    fn obj_is(&mut self, p: i32, o: i32, ty: i32) -> bool;
    /// Slot `+0xac` of `objects[p][o]`, then the data's `+0x18` record's `+4`.
    fn obj_data_kind(&mut self, p: i32, o: i32) -> i32;
    /// `LeaderData::type_avail(leaders[p], kind, 1)`.
    fn type_avail(&mut self, p: i32, kind: i32) -> bool;
    /// `leaders[p]`'s first word has a bit of `0x700`.
    fn leader_flag(&self, p: i32) -> bool;

    /// `units[p][u]`'s `+0x54`.
    fn unit_type_override(&self, p: i32, u: i32) -> i32;
    /// `units[p][u]`'s `+0x18` record's `+4`.
    fn unit_kind(&self, p: i32, u: i32) -> i32;
    /// `types.list[ty]`'s `+4`.
    fn type_kind(&self, ty: i32) -> i32;
    /// `SpellTypeData::is_castable(spelltypes[+0xa4c], u, p, 0)`.
    fn is_castable(&mut self, p: i32, u: i32) -> bool;
    /// `ObjectData::count_inside(units[p][u], kind, ty)`.
    fn count_inside(&mut self, p: i32, u: i32, kind: i32, ty: i32) -> i32;
    /// `Group::add(scratch, u, p, 0, 0)`.
    fn add_local(&mut self, u: i32, p: i32);
    /// `UnitData::get_action`'s order's slot `+0x10`, none for a null order.
    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32>;

    /// The chain link of `objects[p][o]`: its shorts `+0x2c` and `+0x2e`.
    fn obj_next(&self, p: i32, o: i32) -> (i32, i32);
    /// Slot `+0xc` of `objects[p][o]`.
    fn obj_slot_c(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0x4c` of `objects[p][o]`.
    fn obj_slot_4c(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xb0`, then slot `+0x90` of the data's `+0x18` record.
    fn obj_data_slot_90(&mut self, p: i32, o: i32) -> bool;

    /// `units[p][u]`'s short `+0x80` set to `-1`.
    fn unit_clear_80(&mut self, p: i32, u: i32);
    /// `Unit::add_cast_order(units[p][u], -1, -1, x, y, 0x293, queue, 1)`.
    fn add_cast_order(&mut self, p: i32, u: i32, x: i32, y: i32, queue: i32);
    /// `Unit::add_gather_order(units[p][u], target, queue, 1)`.
    fn add_gather_order(&mut self, p: i32, u: i32, target: i32, queue: i32);
    /// `Game::semaphore.ptr[1] & 8`.
    fn semaphore(&self) -> bool;
    /// `objects[p][o]`'s short `+0x28`.
    fn obj_link28(&self, p: i32, o: i32) -> i32;
    /// `ObjectData::can_carry(units[p][u], GROUND)`.
    fn can_carry_ground(&mut self, p: i32, u: i32) -> bool;
    /// `units[p][u]`'s signed byte `+0x3e` and short `+0x28`.
    fn unit_carrier(&self, p: i32, u: i32) -> (i32, i32);
    /// `Unit::same_damage(units[owner][cont], u, p)`.
    fn same_damage(&mut self, owner: i32, cont: i32, u: i32, p: i32);
    /// `Unit::clear_orders(units[p][u])`.
    fn clear_orders(&mut self, p: i32, u: i32);
    /// `Unit::go_inside(units[p][u], target, tp, 0)`.
    fn go_inside(&mut self, p: i32, u: i32, target: i32, tp: i32);
    /// The unit's slot `+0x158`: `Unit::die` (`Object::die(0, -1, 0)`) or
    /// whatever else the vtable holds.
    fn die(&mut self, p: i32, u: i32);
    /// Slot `+0xac` of `objects[p][o]`, then `Build::check_gatherers`.
    fn check_gatherers(&mut self, p: i32, o: i32);
    /// The group's slot `+0x10(u, p, 0, 0)`.
    fn group_slot10(&mut self, u: i32, p: i32);

    /// `Group::get_num(scratch)`.
    fn get_num(&mut self) -> i32;
    /// `Group::action_move_near(scratch, x, y, 0xc0, QUEUE_NEW, 0, 0, MOVE_TO, 0, -1, -1, 0)`.
    fn move_near(&mut self, x: i32, y: i32);
}

/// The tile region the listing reads inline for a point: the cell's region,
/// or its second when the world's second array marks the point.
fn tile_region<V: GatherView>(v: &V, x: i32, y: i32) -> i32 {
    let dy = v.div3(y >> 6);
    let dx = v.div3(x >> 6);
    let idx = (dy >> 2).wrapping_mul(v.world_w()).wrapping_add(dx >> 2);
    if v.cell_i16(idx, 0) & 0x100 == 0 {
        v.cell_i16(idx, 4)
    } else {
        let at = v.world_dim_x().wrapping_mul(dy).wrapping_add(dx);
        if v.ext_byte(at) & 0x30 == 0x20 {
            v.cell_i16(idx, 6)
        } else {
            v.cell_i16(idx, 4)
        }
    }
}

/// The good the building's data kind (`0x1a1..=0x1a6`) needs the leader to
/// have, or none for any other kind.
fn needed_good(kind: i32) -> Option<i32> {
    match kind {
        0x1a1 => Some(0),
        0x1a2 => Some(1),
        0x1a3 => Some(4),
        0x1a4 => Some(3),
        0x1a5 | 0x1a6 => Some(5),
        _ => None,
    }
}

/// A scholar's type, by the unit's own kind or by the type its override names.
fn scholar<V: GatherView>(v: &V, p: i32, u: i32, ty: i32) -> bool {
    let kind = v.unit_kind(p, u);
    kind == 0x34 || kind == 0x35 || {
        let k = v.type_kind(ty);
        k == 0x34 || k == 0x35
    }
}

/// `Group::action_gather(b, queue)`: `b` the building's index in the
/// group's player's objects, `queue` in the original's numbering
/// (`QUEUE_FIRST` 0 reads as `QUEUE_NEW`).
pub fn action_gather_steps<V: GatherView>(v: &mut V, b: i32, queue: i32) {
    v.list_clear();
    garrison_head(v);
    v.set_form();
    let q = if queue != 0 { queue } else { 2 };
    let p = v.who() & 0xff;
    let (rx, ry) = v.obj_pos_raw(p, b);
    let (bx, by) = (rx ^ POS_KEY, ry ^ POS_KEY);
    let region = tile_region(v, bx, by);
    v.list_clear();
    // How many members still gather: the listing counts one of two ways.
    let free = if !v.obj_is(p, b, 0x1a4) {
        let a = v.count_kind(0xf, 0);
        a.wrapping_sub(v.count_kind(0xc, 0))
            .wrapping_sub(v.count_kind(0x1f, 0))
    } else {
        let a = v.count_kind(COUNT_TYPE, TYPE_SCHOLAR_COUNT);
        a.wrapping_sub(v.count_kind(0xd, 0))
    };
    let kind = v.obj_data_kind(p, b);
    if let Some(good) = needed_good(kind)
        && !v.type_avail(p, good)
    {
        return;
    }
    let mut i = v.count() - 1;
    let mut spiral = 0usize;
    // The building the loop asks about is two locals the chain walk below reuses for the object it is
    // looking at (`-0x1c` and `-0x20` of the frame), so after a scan it is the last object visited.
    let (mut bp, mut bo) = (p, b);
    'members: while i >= 0 {
        let m = v.member(i);
        if !v.obj_alive(p, m) || !v.obj_bc(p, m) {
            i -= 1;
            continue;
        }
        let mut ty = v.unit_type_override(p, m);
        if ty < 0 {
            ty = v.unit_kind(p, m);
        }
        if !v.obj_is(bp, bo, 0x1a4) {
            if scholar(v, p, m, ty) {
                v.add_local(m, p);
                i -= 1;
                continue;
            }
            let kind = v.unit_kind(p, m);
            let go = kind == 0x32
                || kind == 0x33
                || v.is_castable(p, m)
                || (v.leader_flag(p) && v.count_inside(p, m, COUNT_TYPE, 0x32) != 0);
            if !go {
                v.add_local(m, p);
                i -= 1;
                continue;
            }
        } else if !scholar(v, p, m, ty) {
            v.add_local(m, p);
            i -= 1;
            continue;
        }
        // Busy gatherers (an order of kind 7) are left alone.
        if free != 0 && v.unit_order_kind(p, m) == Some(7) {
            i -= 1;
            continue;
        }
        let (mrx, mry) = v.obj_pos_raw(p, m);
        let mreg = tile_region(v, mrx ^ POS_KEY, mry ^ POS_KEY);
        if !v.leader_flag(p) && mreg != region {
            i -= 1;
            continue;
        }
        let (dx, dy) = crate::world::MOVE_289[spiral];
        let x = dx.wrapping_mul(0x300).wrapping_add(bx);
        let y = dy.wrapping_mul(0x300).wrapping_add(by);
        spiral += 1;
        if spiral >= SPIRAL_STEPS as usize {
            // Everyone left goes to the scratch group, unless busy.
            let mut k = i;
            while k >= 0 {
                let m2 = v.member(k);
                k -= 1;
                if v.obj_alive(p, m2)
                    && v.obj_bc(p, m2)
                    && (free == 0 || v.unit_order_kind(p, m2) != Some(7))
                {
                    v.add_local(m2, p);
                }
            }
            break 'members;
        }
        let bound_x = v.world_dim_x().wrapping_mul(0xc0);
        let bound_y = v.world_dim_y().wrapping_mul(0xc0);
        if x < 0 || y < 0 || x >= bound_x || y >= bound_y {
            continue;
        }
        if !v.leader_flag(p) {
            let ty_ = v.div3(y >> 6);
            let tx_ = v.div3(x >> 6);
            if v.get_tregion(tx_, ty_) != region {
                continue;
            }
        }
        let idx = v
            .div3(y >> 8)
            .wrapping_mul(v.world_w())
            .wrapping_add(v.div3(x >> 8));
        let mut o = v.cell_i16(idx, 8);
        let mut op = v.cell_i16(idx, 10);
        while o >= 0 {
            let (next, next_p) = v.obj_next(op, o);
            (bp, bo) = (op, o);
            if op == p && v.obj_slot_c(op, o) && v.obj_slot_4c(op, o) && v.obj_data_slot_90(op, o) {
                gather_body(v, p, m, op, o, q);
                i -= 1;
                spiral = 0;
                continue 'members;
            }
            o = next;
            op = next_p;
        }
    }
    if v.get_num() != 0 {
        v.move_near(bx, by);
    }
}

/// The listing from `007012f4`: the member `m` takes the object `o` of `tp`.
fn gather_body<V: GatherView>(v: &mut V, p: i32, m: i32, tp: i32, o: i32, q: i32) {
    v.unit_clear_80(p, m);
    let (mrx, mry) = v.obj_pos_raw(p, m);
    let (cx, cy) = (mrx ^ POS_KEY, mry ^ POS_KEY);
    if q == 1 {
        if v.is_castable(p, m) {
            v.add_cast_order(p, m, cx, cy, 1);
        }
        v.add_gather_order(p, m, o, 1);
    } else {
        v.add_gather_order(p, m, o, 2);
        if v.is_castable(p, m) {
            v.add_cast_order(p, m, cx, cy, 0);
        } else if v.semaphore() && v.obj_is(tp, o, 0x1a6) && v.obj_link28(tp, o) < 0 && tp == p {
            if !v.can_carry_ground(p, m) {
                v.clear_orders(p, m);
                v.go_inside(p, m, o, tp);
            } else {
                let (owner, cont) = v.unit_carrier(p, m);
                if cont >= 0 {
                    v.same_damage(owner, cont, m, p);
                    v.clear_orders(owner, cont);
                }
                v.clear_orders(p, m);
                v.go_inside(p, m, o, tp);
                v.die(p, m);
            }
            v.check_gatherers(tp, o);
        }
    }
    v.group_slot10(m, p);
}
