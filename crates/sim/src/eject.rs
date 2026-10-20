//! `Object::eject_contents@0064cd20` as steps over a view (item 3356): every
//! passenger of a container comes out — a transport's, a building's — and the
//! unit that comes out is put to rights: its orders cleared or its path
//! carried over from the transport, a builder sent to the nearest friendly
//! city, a caravan's cargo handed on, the player's selection and hot unit
//! moved to it. [`eject_steps`] is the original's control flow in its order;
//! [`EjectView`] is what it reads and the callees it makes, which
//! `sweep/object_eject_contents.rs` answers from the emulated original's
//! windows. It sits beside [`crate::disband`], which is built the same way.
//!
//! **Arguments.** `kill` (the first) is "a passenger that will not come out
//! dies" (the second `eject_contents` argument of `transport.rs` and
//! `build.rs`); `ty` is the type to eject or `-1` for every passenger;
//! `carry` is "the passenger takes over the transport's orders and path";
//! `p4` is "clear the passenger's orders first" (the original passes it from
//! the transport's dying side). The names are the crate's reading of the call
//! sites, not the symbols', which the export does not keep for a parameter.
//!
//! **The loop.** The container's chain head is re-read every pass, because
//! `Unit::come_out` (or the death after a refusal) unlinks the passenger that
//! was just handled. A passenger the type filter skips is not unlinked:
//! the walk *moves into it* (`container := passenger`) and goes on down its
//! own chain, which is what the original does and what no crate method that
//! walks a garrison does.
//!
//! **What the crate does not carry.** The crate's `eject_contents`s
//! (`garrison.rs`, `build.rs`, `city.rs`) are approximations on its own data
//! model; none is this function. There is no `SimEject`, so this module has
//! no caller in the simulation yet: the crate's building ejection is
//! `garrison.rs::eject_contents`, one squad a frame, and `transport.rs`
//! models the transport arm in place.

/// What `Object::eject_contents` reads of the world and the callees it makes.
/// A passenger or container is named by `(w, s)`: the owner's index and its
/// slot in `units[w]` / `objects[w]`, which are the same records.
pub trait EjectView {
    /// `this->who` (the byte at `+9`).
    fn who(&mut self) -> i32;
    /// `this->index` (the short at `+0xa`).
    fn index(&mut self) -> i32;
    /// Vslot `0xbc`: the object can hold anything at all.
    fn can_hold(&mut self) -> bool;
    /// Vslot `0x20`: a building that is not a wall.
    fn is_plain_building(&mut self) -> bool;
    /// Vslot `0x18`: a unit.
    fn is_unit(&mut self) -> bool;
    /// `game->semaphore.ptr[1] & 8`.
    fn game_flag_8(&mut self) -> bool;
    /// `ObjectData::can_carry(this, kind)`.
    fn can_carry(&mut self, kind: i32) -> bool;
    /// `this->vtable[0xac]()` (the data record), then `+0x60 |= 0x4000`.
    fn mark_data_4000(&mut self);
    /// `(short) objects[w][s]->+0x28`: the head of its passenger chain.
    fn container_head(&mut self, w: i32, s: i32) -> i32;
    /// `(char) objects[w][s]->+0x3e`: the owner its passengers carry.
    fn container_owner(&mut self, w: i32, s: i32) -> i32;
    /// `objects[w][s]->is(ty, 0)`: vslot `0xb8`, which `ObjectData::is`
    /// answers by its type record's vslot `0x60`.
    fn obj_is(&mut self, w: i32, s: i32, ty: i32) -> bool;
    /// `this->is(ty, 0)`, the same on the object itself.
    fn this_is(&mut self, ty: i32) -> bool;
    /// `types[ty]->index` (`+4`).
    fn type_index(&mut self, ty: i32) -> i32;
    /// `UnitData::get_action(units[w][s])` is not null.
    fn get_action(&mut self, w: i32, s: i32) -> bool;
    /// `Unit::repath`.
    fn repath(&mut self, w: i32, s: i32);
    /// `SpellTypeData::is_castable(spell, s, w, 0)`.
    fn is_castable(&mut self, w: i32, s: i32) -> bool;
    /// `Unit::add_cast_order(u, -1, -1, x ^ 0x63637, y ^ 0x63637, 0x293, 0, 1)`
    /// with `this`'s position.
    fn add_cast_order(&mut self, w: i32, s: i32);
    /// `u->+0x68 &= ~0x4000000; u->+0xc0 = 0`.
    fn unit_reset_flags(&mut self, w: i32, s: i32);
    /// `Unit::close_orders(u, 0)`.
    fn close_orders(&mut self, w: i32, s: i32);
    /// `Unit::clear_partial_path`.
    fn clear_partial_path(&mut self, w: i32, s: i32);
    /// `Unit::update_action`.
    fn update_action(&mut self, w: i32, s: i32);
    /// `Unit::come_out(u, 0)` refused (non-zero).
    fn come_out(&mut self, w: i32, s: i32) -> bool;
    /// `u->vtable[0x158](0, -1, 0)`: `die`, directly or through the slot.
    fn die(&mut self, w: i32, s: i32);
    /// `units[w][s]->type->index`.
    fn unit_type(&mut self, w: i32, s: i32) -> i32;
    /// `units[w][s]->type->+0x308`.
    fn unit_type_308(&mut self, w: i32, s: i32) -> i32;
    /// `this->vtable[0xac]()->+0x72`, the city the container belongs to.
    fn data_city(&mut self) -> i32;
    /// `LeaderData::tribe_can_type(leaders[w], types[0x42])`.
    fn tribe_can_type(&mut self, w: i32) -> bool;
    /// `types[0x42]->vtable[0x68](leaders[w]->+8)`.
    fn leader_type68(&mut self, w: i32) -> i32;
    /// The leader's type bitset at `+0x6c18`, bit `r`.
    fn leader_bit(&mut self, w: i32, r: i32) -> bool;
    /// `LeaderData::current_upgrade(leaders[w], ty)`.
    fn current_upgrade(&mut self, w: i32, ty: i32) -> i32;
    /// `Unit::set_type` of the passenger, through its slot (`0x80`, or `0x84`
    /// when the slot holds the base), handed `(ty, 0)`.
    fn unit_set_type(&mut self, w: i32, s: i32, ty: i32);
    /// `Unit::clear_orders`.
    fn unit_clear_orders(&mut self, w: i32, s: i32);
    /// `ObjectsData::find_city(l20, x, y, 1, w, l20, 0x200, 3, city_id, who)`
    /// from `this`'s position; the answer is a city index or negative.
    fn find_city(&mut self, w: i32, l20: i32) -> i32;
    /// `Unit::add_move_order` to `cities[w][city]`'s position.
    fn add_move_order(&mut self, w: i32, s: i32, city: i32);
    /// `Unit::add_strafe_order(u, -1, -1, -1, -1, 1, 2, 0)`.
    fn add_strafe_order(&mut self, w: i32, s: i32);
    /// The transport's own unit is on screen (vslot `0x70`, or the byte at
    /// `+0x1c` when the slot holds `SubObjectOut::is_on_screen`).
    fn me_on_screen(&mut self) -> bool;
    /// The transport's unit's vslot `0x48(console->who, 0)`.
    fn me_pred(&mut self) -> bool;
    /// `SoundGlobal::play(0x134)`.
    fn sound_unload(&mut self);
    /// `Unit::same_damage(u, this->index, this->who)`.
    fn same_damage(&mut self, w: i32, s: i32);
    /// The transport's unit is a caravan (vslot `0xd0`).
    fn me_is_caravan(&mut self) -> bool;
    /// The caravan's cargo handed to the passenger: `+0x86` copied, the
    /// caravan record pointed at it, `+0x86 := -1`, flag `0x200` copied.
    fn hand_over_caravan(&mut self, w: i32, s: i32);
    // The orders list of the transport's unit, moved to the passenger.
    /// `me->+0xd8`: the list's length.
    fn me_list_len(&mut self) -> i32;
    /// `me->+0xdc != 0`: the list has a head node; when it has, the current
    /// node, data and flag (`+0xd4`, `+0xcc`, `+0xd0`) are set from it.
    fn me_list_advance(&mut self);
    /// `me->+0xcc`: the current node's data.
    fn me_list_data(&mut self) -> i32;
    /// `LinkListBase::remove_current` on `me->+0xcc`.
    fn me_list_remove(&mut self);
    /// `LinkListBase::add` on the passenger's `+0xcc`.
    fn list_add(&mut self, w: i32, s: i32, data: i32);
    // The path stack (`Stack<PathData>`, 16-byte records) at `+0xb8`.
    /// `Stack<PathData>::invert` on the transport's unit.
    fn me_stack_invert(&mut self);
    /// The transport's unit's `+0xc0`.
    fn me_stack_count(&mut self) -> i32;
    /// Set the transport's unit's `+0xc0`.
    fn me_stack_set_count(&mut self, n: i32);
    /// Record `i` of the transport's stack.
    fn me_stack_get(&mut self, i: i32) -> [i32; 4];
    /// The passenger's `+0xc0` and `+0xbc` (the stack's count and capacity).
    fn stack_count(&mut self, w: i32, s: i32) -> i32;
    fn stack_cap(&mut self, w: i32, s: i32) -> i32;
    fn stack_set_count(&mut self, w: i32, s: i32, n: i32);
    fn stack_get(&mut self, w: i32, s: i32, i: i32) -> [i32; 4];
    fn stack_set(&mut self, w: i32, s: i32, i: i32, rec: [i32; 4]);
    /// `Stack<PathData>::increase_size(stack, arg)`.
    fn stack_increase(&mut self, w: i32, s: i32);
    /// `Stack<PathData>::push(stack, &rec)`.
    fn stack_push(&mut self, w: i32, s: i32, rec: [i32; 4]);
    /// `Unit::reset_move_orders(me)`.
    fn me_reset_move_orders(&mut self);
    /// `u->+0x68`.
    fn unit_flags68(&mut self, w: i32, s: i32) -> i32;
    /// `Object::get_army(u)`.
    fn get_army(&mut self, w: i32, s: i32) -> i32;
    /// The group the passenger is put in: `Group::clear`, `add`, `set_up_insert`,
    /// `kill`, `add`, `push_group`, `finish_insert`, the list's `clear`.
    fn group_insert(&mut self, w: i32, s: i32);
    /// `leaders[this->who].flags`.
    fn leader_flags(&mut self) -> i32;
    /// `UnitData::transport_type(u)`.
    fn transport_type(&mut self, w: i32, s: i32) -> i32;
    /// `units[w][s]->is(ty, 0)` through its slot (the same record as the
    /// object's).
    fn unit_is(&mut self, w: i32, s: i32, ty: i32) -> bool;
    /// `leader_options[this->who].+0x1c & 2`.
    fn leader_option_2(&mut self) -> bool;
    /// `u->+0x68 |= 0x800000`.
    fn unit_set_flag_800000(&mut self, w: i32, s: i32);
    /// `GroupData::member(selection, this->index, this->who, 1)`.
    fn selection_member(&mut self) -> bool;
    /// The selection's slots `0x10(index, who, 0, 0)` and `0x18(s, w, 0)`.
    fn selection_swap(&mut self, w: i32, s: i32);
    /// `Object::replace_hotunit(this, s, w)`.
    fn replace_hotunit(&mut self, w: i32, s: i32);
}

/// How a passenger is made ready to come out.
#[derive(Clone, Copy)]
enum Ready {
    /// Its action repathed, a castable spell queued for a unit that casts.
    Repath,
    /// Its busy flag and orders cleared (`p4`).
    Clear,
}

/// The original's control flow. `kill`, `ty`, `carry`, `p4` are its four
/// arguments in order.
pub fn eject_steps<V: EjectView>(v: &mut V, kill: i32, ty: i32, carry: i32, p4: i32) {
    if !v.can_hold() {
        return;
    }
    // A building that carries nothing the game would let out is only marked,
    // once, for the next pass.
    if ty < 0 && v.is_plain_building() && !v.game_flag_8() && !v.can_carry(2) {
        v.mark_data_4000();
        return;
    }
    let (mut cw, mut ci) = (v.who(), v.index());
    loop {
        let head = v.container_head(cw, ci);
        if head < 0 {
            break;
        }
        let (w, s) = (v.container_owner(cw, ci), head);
        let ready = if ty < 0 || v.obj_is(w, s, ty) {
            Some(if p4 == 0 { Ready::Repath } else { Ready::Clear })
        } else if matches!(v.type_index(ty), 0x32 | 0x33) && v.obj_is(w, s, 0x42) && p4 == 0 {
            Some(Ready::Repath)
        } else {
            None
        };
        let Some(ready) = ready else {
            // Not this one: the walk goes into it.
            cw = w;
            ci = s;
            continue;
        };
        match ready {
            Ready::Repath => {
                if v.get_action(w, s) {
                    v.repath(w, s);
                }
                if v.obj_is(w, s, 0x42) && v.is_castable(w, s) {
                    v.add_cast_order(w, s);
                }
            }
            Ready::Clear => {
                v.unit_reset_flags(w, s);
                v.close_orders(w, s);
                v.clear_partial_path(w, s);
                v.update_action(w, s);
            }
        }
        if v.come_out(w, s) {
            // It would not come out.
            if kill == 0 {
                return;
            }
            v.die(w, s);
            continue;
        }
        placed(v, kill, carry, w, s);
    }
}

/// What is done for a passenger that has come out: `kill` is the first
/// argument and `carry` the third.
fn placed<V: EjectView>(v: &mut V, kill: i32, carry: i32, w: i32, s: i32) {
    if v.is_plain_building() && matches!(v.unit_type(w, s), 0x32..=0x35) && kill != 0 {
        builder_goes_home(v, w, s);
    }
    if v.this_is(0x1bf) {
        v.add_strafe_order(w, s);
    }
    if !v.is_unit() {
        return;
    }
    if v.me_on_screen() && v.me_pred() {
        v.sound_unload();
    }
    v.same_damage(w, s);
    if v.me_is_caravan() {
        v.hand_over_caravan(w, s);
    }
    if carry != 0 {
        carry_orders(v, w, s);
    }
    // The passenger is a transport's if the owner's flags let it be.
    let lf = v.leader_flags();
    let limit = if lf & 0x100 != 0 {
        3
    } else if lf & 0x200 != 0 {
        2
    } else {
        (lf >> 10) & 1
    };
    if v.transport_type(w, s) <= limit && (!v.unit_is(w, s, 0x45) || v.leader_option_2()) {
        v.unit_set_flag_800000(w, s);
    }
    if v.selection_member() {
        v.selection_swap(w, s);
    }
    v.replace_hotunit(w, s);
}

/// A builder that came out of a building goes back to work: a Lakota-style
/// upgrade of its type when the owner has one, else to the nearest friendly
/// city.
fn builder_goes_home<V: EjectView>(v: &mut V, w: i32, s: i32) {
    let l20 = v.data_city();
    if matches!(v.unit_type(w, s), 0x32 | 0x33) {
        let r = if v.tribe_can_type(w) {
            0x42
        } else {
            v.leader_type68(w)
        };
        if v.leader_bit(w, r) {
            let t = if v.tribe_can_type(w) {
                0x42
            } else {
                v.leader_type68(w)
            };
            let up = v.current_upgrade(w, t);
            v.unit_set_type(w, s, up);
            v.unit_clear_orders(w, s);
            return;
        }
    }
    if l20 >= 0 {
        let city = v.find_city(w, l20);
        if city >= 0 {
            v.add_move_order(w, s, city);
        }
    }
}

/// The passenger takes over the transport's orders and path (`carry`).
fn carry_orders<V: EjectView>(v: &mut V, w: i32, s: i32) {
    if v.unit_type_308(w, s) == 1 {
        // The orders list moves node by node; the path stack is inverted and
        // moved record by record, and the top one comes back with its flag 4
        // cleared.
        while v.me_list_len() != 0 {
            v.me_list_advance();
            let data = v.me_list_data();
            v.me_list_remove();
            v.list_add(w, s, data);
        }
        v.me_stack_invert();
        while v.me_stack_count() != 0 {
            let mut n = v.me_stack_count();
            if n < 1 {
                n = 1;
            }
            n -= 1;
            v.me_stack_set_count(n);
            let rec = v.me_stack_get(n);
            if v.stack_cap(w, s) <= v.stack_count(w, s) {
                v.stack_increase(w, s);
            }
            let at = v.stack_count(w, s);
            v.stack_set(w, s, at, rec);
            v.stack_set_count(w, s, at.wrapping_add(1));
        }
        if v.stack_count(w, s) != 0 {
            let mut n = v.stack_count(w, s);
            if n < 1 {
                n = 1;
            }
            n -= 1;
            v.stack_set_count(w, s, n);
            let mut rec = v.stack_get(w, s, n);
            rec[3] &= !4;
            v.stack_push(w, s, rec);
        }
    } else {
        v.me_reset_move_orders();
        if v.unit_flags68(w, s) & 0x40000 == 0 || v.get_army(w, s) < 0 {
            v.group_insert(w, s);
        }
    }
}
