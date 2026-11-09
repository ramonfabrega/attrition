//! **`Group::action_attack_ground@00704520`'s steps** (item 3381): the group
//! order that sends every member at a point, as a walk over an
//! [`AttackGroundView`]. The control flow is the original's, call for call and
//! in its order; the view is the state it reads and the callees it makes, which
//! `sweep/group_action_attack_ground.rs` answers from the emulated original's
//! windows. The crate has no group-level method of this name yet: the
//! [`crate::group`] adapter's `action_attack_ground` is an empty arm, and
//! wiring it wants `Group::add` and `action_air_attack_ground`, neither built.
//!
//! ```text
//! head      group slot +0x14 (refresh); the scenario's ignore-orders list (slot +0x10); count == 0 ends
//! clamp     both coordinates into 0..(world cells * 0x300 - 1)
//! peace     the tile's owner (>= 0, not this player) at peace with this player: the console's
//!           group hears one refusal, any other's ends silently
//! leader    count < 1 ends; find_leader (or the first member when the group is flagged +0x49)
//! ground    unflagged group, leader cannot carry air, slot +0xc0 false: on the map, then each
//!           member that is alive, can act, is not busy and can attack ground gets one order; ends
//! air       slot +0xc0 true: straight to the air order
//! carriers  otherwise: every member flagged +8 & 1 follows its +0x28 chain into a scratch group,
//!           this group is cleared, the scratch members are re-added (slot +0xc), then the air order
//! ```

/// What `Group::action_attack_ground` asks of the group, the world and the objects.
pub trait AttackGroundView {
    /// The group's slot `+0x14()`.
    fn refresh(&mut self);
    /// `[this + 0x4a]`.
    fn who(&self) -> i32;
    /// `ScenarioData::ignore_orders != 0`.
    fn ignore_orders(&self) -> bool;
    /// `ScenarioData::objects_ignoring_orders[who]`'s count.
    fn ignoring_count(&self, who: i32) -> i32;
    /// Its element `i`.
    fn ignoring_at(&self, who: i32, i: i32) -> i32;
    /// The group's slot `+0x10(o, who, 0, 0)`.
    fn drop_ignored(&mut self, o: i32, who: i32);
    /// `[this + 0xc]`, read afresh each time.
    fn count(&self) -> i32;
    /// The world's width and height in cells (`[world]`, `[world + 4]`).
    fn world_dims(&self) -> (i32, i32);
    /// The signed owner byte of the tile at the clamped point.
    fn tile_owner(&self, x: i32, y: i32) -> i32;
    /// `LeaderData::is_peace(leaders[who], owner)`.
    fn is_peace(&mut self, who: i32, owner: i32) -> bool;
    /// `console->who`.
    fn console_who(&self) -> i32;
    /// The refusal: the string, `add_feedback` in red, the invalid-order sound.
    fn refuse(&mut self);
    /// `[this + 0x49] != 0`.
    fn flagged(&self) -> bool;
    /// The sign-extended short `i` of the member list (`+0x8cc`).
    fn member(&self, i: i32) -> i32;
    /// `GroupData::find_leader(0)`.
    fn find_leader(&mut self) -> i32;
    /// `ObjectData::can_carry(objects[who][o], AIR)`.
    fn can_carry_air(&mut self, who: i32, o: i32) -> bool;
    /// Slot `+0xc0` of `objects[who][o]`.
    fn obj_c0(&mut self, who: i32, o: i32) -> bool;
    /// `GroupData::is_on_map`.
    fn is_on_map(&mut self) -> bool;
    /// Slot `+0x8` of `objects[who][o]`.
    fn obj_slot8(&mut self, who: i32, o: i32) -> bool;
    /// Slot `+0xbc` of `objects[who][o]`.
    fn obj_slot_bc(&mut self, who: i32, o: i32) -> bool;
    /// Slot `+0xc0` of `units[who][o]`.
    fn unit_c0(&mut self, who: i32, o: i32) -> bool;
    /// `UnitTypeData::can_attack_ground(units[who][o].type)`.
    fn unit_can_attack_ground(&mut self, who: i32, o: i32) -> bool;
    /// `Unit::add_attack_ground_order(x, y, queue, 1, 0, 0)` on `units[who][o]`.
    fn add_attack_ground_order(&mut self, who: i32, o: i32, x: i32, y: i32, queue: i32);
    /// `objects[who][o]`'s flag byte `+8`, bit 0.
    fn obj_flag1(&self, who: i32, o: i32) -> bool;
    /// `objects[who][o]`'s short `+0x28`.
    fn obj_link(&self, who: i32, o: i32) -> i32;
    /// `objects[who][o]`'s signed byte `+0x3e`.
    fn obj_link_who(&self, who: i32, o: i32) -> i32;
    /// `Group::clear(scratch, -1)`.
    fn scratch_clear(&mut self);
    /// `Group::add(scratch, id, who, 0, 0)`.
    fn scratch_add(&mut self, id: i32, who: i32);
    /// The scratch group's count, read afresh each time.
    fn scratch_count(&self) -> i32;
    /// Its member `k`: the id and the player.
    fn scratch_member(&self, k: i32) -> (i32, i32);
    /// `Group::clear(this, -1)`.
    fn clear(&mut self);
    /// The group's slot `+0xc(id, who, 1, 1)`.
    fn add_member(&mut self, id: i32, who: i32);
    /// `Group::action_air_attack_ground(x, y, queue)`.
    fn air_attack_ground(&mut self, x: i32, y: i32, queue: i32);
}

/// The whole of the listing at `00704520`.
pub fn action_attack_ground_steps<V: AttackGroundView>(v: &mut V, a: [i32; 3]) {
    v.refresh();
    if v.ignore_orders() && (v.who() & 0xff) < 8 {
        let who = v.who() & 0xff;
        if v.ignoring_count(who) > 0 {
            let mut i = 0;
            loop {
                let o = v.ignoring_at(v.who() & 0xff, i);
                if o >= 0 {
                    v.drop_ignored(o, v.who() & 0xff);
                }
                i += 1;
                if i >= v.ignoring_count(v.who() & 0xff) {
                    break;
                }
            }
        }
    }
    if v.count() == 0 {
        return;
    }
    let (w, h) = v.world_dims();
    let (mut x, mut y, queue) = (a[0].max(0), a[1].max(0), a[2]);
    let (xmax, ymax) = (w.wrapping_mul(0x300), h.wrapping_mul(0x300));
    if x >= xmax {
        x = xmax.wrapping_sub(1);
    }
    if y >= ymax {
        y = ymax.wrapping_sub(1);
    }
    let owner = v.tile_owner(x, y);
    let who = v.who() & 0xff;
    if owner >= 0 && owner != who && v.is_peace(who, owner) {
        if who == v.console_who() {
            v.refuse();
        }
        return;
    }
    if v.count() < 1 {
        return;
    }
    let leader = if v.flagged() {
        v.member(0)
    } else {
        v.find_leader()
    };
    if leader < 0 {
        return;
    }
    if !v.flagged() && !v.can_carry_air(v.who() & 0xff, leader) {
        if !v.obj_c0(v.who() & 0xff, leader) {
            if !v.is_on_map() {
                return;
            }
            let mut i = 0;
            if v.count() <= i {
                return;
            }
            loop {
                let (who, o) = (v.who() & 0xff, v.member(i));
                if v.obj_slot8(who, o)
                    && v.obj_slot_bc(who, o)
                    && !v.unit_c0(who, o)
                    && v.unit_can_attack_ground(who, o)
                {
                    v.add_attack_ground_order(who, o, x, y, queue);
                }
                i += 1;
                if i >= v.count() {
                    break;
                }
            }
            return;
        }
        v.air_attack_ground(x, y, queue);
        return;
    }
    v.scratch_clear();
    let mut i = 0;
    while i < v.count() {
        let (mut who, mut o) = (v.who() & 0xff, v.member(i));
        if v.obj_flag1(who, o) {
            while v.obj_link(who, o) >= 0 {
                let (nw, nid) = (v.obj_link_who(who, o), v.obj_link(who, o));
                v.scratch_add(nid, nw);
                who = nw;
                o = nid;
            }
        }
        i += 1;
    }
    v.clear();
    let mut k = 0;
    while k < v.scratch_count() {
        let (id, w) = v.scratch_member(k);
        v.add_member(id, w);
        k += 1;
    }
    v.air_attack_ground(x, y, queue);
}
