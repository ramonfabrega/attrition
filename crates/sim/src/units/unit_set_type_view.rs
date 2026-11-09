//! **`Unit::set_type@00612fa0`** (item 3402): a unit changes its type, as a
//! walk over a [`UnitSetTypeView`].
//!
//! **The port is [`crate::Sim::unit_set_type`]**, which runs through
//! [`set_type_steps`] by an adapter (`SimSetType`, in `lib.rs`); the sweep
//! `sweep/unit_set_type_normal.rs` runs the same function over the original's
//! own bytes. The crate carries the guy stack as a `Vec<Guy>` and keeps no
//! recycler, so the adapter folds several of the original's steps into the
//! calls the crate always made (each fold is said on the view's method).
//!
//! In the order it runs, for `SET_TYPE_NORMAL` (`mode 0`; `SET_TYPE_LOAD` is
//! `2`, and any other mode is the "no guy loops" arm):
//!
//! 1. **the leader's counts come off** for the old type — [`count_leader`]:
//!    only a captain (`vslot 0xe8`) whose type has population (`+0x2f0`), or
//!    whose table object is `is(0x134)` or a governor hero, and only with
//!    `+0x68 & 1` clear, moves `track_unit_type`, `control` and `active`; a
//!    caravan (`vslot 0xd0`) moves `caras` whatever the captain gate said;
//! 2. **the guys past the new squad go**: `kill_guy`, `Guy::clear` and a push
//!    on the recycler, from the new `squad_size` to `guy_mark`, then (with a
//!    type) from the old type's `squad_size` to the old `guy_total`;
//! 3. **the guy array grows** to `max(need, bound)` when the new total
//!    outgrows `+0xec`, **the slots between the old total and the new squad
//!    are zeroed**, the total becomes `crew_size + squad_size`, and every slot
//!    from the new squad to the new total pops a fresh guy and clears it;
//! 4. **`+0x18` is the new object type**, `guy_mark` is clamped (signed
//!    `min`) to the new `squad_size`, and the guys are re-initialised: the
//!    kept ones `0 .. guy_mark` (new type, `init_real(g, 1)`, `+0xa2 = i`),
//!    then `squad_size .. guy_total` (new type, `+0xa2`, owner `+0xa1`, index
//!    `+0x8c`, `init_real(g, 1)`);
//! 5. **`SubObject::set_type(ty, mode)`** — every mode;
//! 6. **unless `LOAD`**: the CEO bit (`is(0x165, 1)` into `+0x6c & 0x10000`,
//!    and `update_ceo_position` through `div_3_table`), `is(0x77, 0)` into
//!    `+0x6c |= 0x800`, vslots `+0x15c(0)` and `+0x160`, `update_armor` and
//!    `update_speed`;
//! 7. **`NORMAL` runs `update_gpiece`**; `LOAD` returns here;
//! 8. **the leader's counts go on** for the new type;
//! 9. **`NORMAL` only**: `Guy::set_new_location(guy i, x, y, 1)` over
//!    `0 .. guy_mark`, and — when `game.version` is greater than
//!    `xpack_rtm_version`, byte by byte from the lowest — the unit's supply:
//!    closed when the type is not `is_supply` and `+0x84 >= 0`, opened when it
//!    is and `+0x84 < 0`.
//!
//! **Contract** (not established beyond it, and the sweep draws no other):
//! the guy list holds a valid guy for every index below `guy_mark` and below
//! the old total; the old and new type indices are tables' rows. A null
//! `+0x18` is the original's own `if (this->type != 0)` and the sweep cannot
//! draw it (the leader's block above it reads through the pointer).

/// `SET_TYPE_NORMAL`: the guy loops run and `update_gpiece` follows.
pub const SET_TYPE_NORMAL: i32 = 0;
/// `SET_TYPE_LOAD`: a saved game's type; the tail returns before the counts.
pub const SET_TYPE_LOAD: i32 = 2;

/// The state `Unit::set_type` reads, and the calls it makes.
pub trait UnitSetTypeView {
    // ---- the leader's counts
    /// The unit's `vslot 0xe8` (`is_captain`: `+0x8e` bit 15).
    fn is_captain(&mut self) -> bool;
    /// `+0x68 & 1` (the decoy bit).
    fn decoy_bit(&mut self) -> bool;
    /// The unit's **current** type record's `+0x2f0` (`control_cost`, the
    /// population): the old type before step 4, the new one after it.
    fn type_pop(&mut self) -> i32;
    /// `units[who][idx].is(0x134, 0)`: the table's object for this unit.
    fn table_unit_is_134(&mut self) -> bool;
    /// `units[who][idx].is_gov_hero()`.
    fn table_unit_is_gov_hero(&mut self) -> bool;
    /// The index (`+0x4`) of the unit's **current** type record: what
    /// `track_unit_type` is handed when the counts come off.
    fn old_type_index(&mut self) -> i32;
    /// `Leader::track_unit_type(leader, ty, delta, idx)`. In the crate this is
    /// `Sim::track_unit_type`, which moves `control` and `active` too.
    fn track_unit_type(&mut self, ty: i32, delta: i32);
    /// The leader's `control += delta * pop` and `active += delta`
    /// (`+0x940`, `+0x93c`). The crate folds them into `track_unit_type`.
    fn leader_totals(&mut self, delta: i32, pop: i32);
    /// The unit's `vslot 0xd0` (`is_caravan`).
    fn is_caravan(&mut self) -> bool;
    /// The leader's `caras += delta` (`+0x980`). The crate folds it into
    /// `track_unit_type`'s civilian arm (behind the captain gate).
    fn leader_caras(&mut self, delta: i32);

    // ---- the guy stack
    /// `+0xe8`: the number of guys now.
    fn guy_total(&mut self) -> i32;
    /// `(char) +0xb5`.
    fn guy_mark(&mut self) -> i32;
    /// Writes `+0xb5`.
    fn set_guy_mark(&mut self, n: i32);
    /// Whether `+0x18` holds a type.
    fn has_type(&mut self) -> bool;
    /// `unittypes[ty] +0x304`.
    fn squad_size(&mut self, ty: i32) -> i32;
    /// `unittypes[ty] +0x30c`.
    fn crew_size(&mut self, ty: i32) -> i32;
    /// `Objects::kill_guy(who, idx, i, 0, -1, 0.0, 1)`, `Guy::clear(list[i])`,
    /// the push on the recycler's pool, and `list[i] = 0`. `entry` is the
    /// register the first call of a loop inherits (the type index the loop was
    /// reached with); the instrumentation sees it.
    fn kill_and_recycle(&mut self, i: i32, entry: Option<i32>);
    /// `+0xec`.
    fn guy_capacity(&mut self) -> i32;
    /// The short at `+0xf0`.
    fn guy_hint(&mut self) -> i32;
    /// `ArrayBaseSimpleCopy::increase_size(this + 0xe4, n)`.
    fn grow_guys(&mut self, n: i32);
    /// `list[i] = 0`.
    fn zero_guy_slot(&mut self, i: i32);
    /// Writes `+0xe8`.
    fn set_guy_total(&mut self, n: i32);
    /// `list[i] = Recycler<Guy>::pop()`, then `Guy::clear(list[i])`.
    fn pop_guy(&mut self, i: i32);
    /// `+0x18 = objecttypes[ty]` — the crate swaps the type and every derived
    /// stat here (`Sim::unit_set_type`'s swap block), before the guys.
    fn install_type(&mut self, ty: i32);
    /// A kept guy: `list[i] +8 = ty`, `Guy::init_real(list[i], 1)`,
    /// `+0xa2 = i`.
    fn init_kept(&mut self, i: i32, ty: i32);
    /// A new guy: `+8 = ty`, `+0xa2 = i`, `+0xa1 = who`, `+0x8c = idx`,
    /// `Guy::init_real(list[i], 1)`.
    fn init_new(&mut self, i: i32, ty: i32);

    // ---- the tail
    /// `SubObject::set_type(this, ty, mode)`.
    fn sub_object_set_type(&mut self, ty: i32, mode: i32);
    /// `this.is(0x165, 1)`.
    fn is_ceo(&mut self) -> bool;
    /// `+0x6c`.
    fn flags6c(&mut self) -> i32;
    /// Writes `+0x6c`.
    fn set_flags6c(&mut self, f: i32);
    /// `update_ceo_position(this, -1, -1, div3[(y^K)>>6], div3[(x^K)>>6])`.
    fn update_ceo_position(&mut self);
    /// `this.is(0x77, 0)`.
    fn is_77(&mut self) -> bool;
    /// The unit's `vslot 0x15c(0)`.
    fn slot_15c(&mut self);
    /// The unit's `vslot 0x160`.
    fn slot_160(&mut self);
    /// `Unit::update_armor`.
    fn update_armor(&mut self);
    /// `Unit::update_speed`.
    fn update_speed(&mut self);
    /// `Unit::update_gpiece`.
    fn update_gpiece(&mut self);
    /// `Guy::set_new_location(list[i], x ^ K, y ^ K, 1)`.
    fn seat_guy(&mut self, i: i32);
    /// `game.version > xpack_rtm_version`, byte by byte from the lowest.
    fn version_newer(&mut self) -> bool;
    /// The unit's `vslot 0xcc` (`is_supply`: type `+0x2b8 & 0x40`).
    fn is_supply(&mut self) -> bool;
    /// The short at `+0x84` (the unit's supply).
    fn supply_id(&mut self) -> i32;
    /// `Supplies::close_supply(who, id)`, then `+0x84 = -1`.
    fn close_supply(&mut self, id: i32);
    /// `+0x84 = Supplies::init_supply(who, idx)`.
    fn open_supply(&mut self);
}

/// The leader's counts for one direction: `delta` is `-1` before the swap,
/// `+1` after it, and `ty` is the new type's index.
fn count_leader<V: UnitSetTypeView>(v: &mut V, ty: i32, delta: i32) {
    if v.is_captain() {
        let counts = v.type_pop() != 0 || v.table_unit_is_134() || v.table_unit_is_gov_hero();
        if counts && !v.decoy_bit() {
            let t = if delta < 0 { v.old_type_index() } else { ty };
            v.track_unit_type(t, delta);
            let pop = if v.decoy_bit() { 0 } else { v.type_pop() };
            v.leader_totals(delta, pop);
        }
    }
    if v.is_caravan() {
        v.leader_caras(delta);
    }
}

/// `Unit::set_type(this, ty, mode)`.
pub fn set_type_steps<V: UnitSetTypeView>(v: &mut V, ty: i32, mode: i32) {
    if mode == SET_TYPE_NORMAL {
        let old_total = v.guy_total();
        count_leader(v, ty, -1);
        // The guys past the new squad, up to `guy_mark`.
        let mut i = v.squad_size(ty);
        let mut entry = Some(ty);
        while i < v.guy_mark() {
            v.kill_and_recycle(i, entry.take());
            i += 1;
        }
        // The old type's crew, up to the old total.
        if v.has_type() {
            let old = v.old_type_index();
            let mut i = v.squad_size(old);
            let mut entry = Some(old);
            while i < v.guy_total() {
                v.kill_and_recycle(i, entry.take());
                i += 1;
            }
        }
        // The array grows to the new total.
        let need = v
            .crew_size(ty)
            .wrapping_add(v.squad_size(ty))
            .wrapping_sub(v.guy_capacity());
        if need > 0 {
            let hint = v.guy_hint();
            let bound = if hint >= 0 { hint } else { v.guy_capacity() };
            v.grow_guys(if need > bound { need } else { bound });
        }
        let mut i = old_total;
        while i < v.squad_size(ty) {
            v.zero_guy_slot(i);
            i += 1;
        }
        let total = v.crew_size(ty).wrapping_add(v.squad_size(ty));
        v.set_guy_total(total);
        let mut i = v.squad_size(ty);
        while i < v.guy_total() {
            v.pop_guy(i);
            i += 1;
        }
        v.install_type(ty);
        let mark = v.guy_mark().min(v.squad_size(ty));
        v.set_guy_mark(mark);
        let mut i = 0;
        while i < v.guy_mark() {
            v.init_kept(i, ty);
            i += 1;
        }
        let mut i = v.squad_size(ty);
        while i < v.guy_total() {
            v.init_new(i, ty);
            i += 1;
        }
    }
    v.sub_object_set_type(ty, mode);
    if mode != SET_TYPE_LOAD {
        let ceo = v.is_ceo();
        let f = v.flags6c();
        v.set_flags6c(if ceo { f | 0x10000 } else { f & !0x10000 });
        if v.flags6c() & 0x10000 != 0 {
            v.update_ceo_position();
        }
        if v.is_77() {
            let f = v.flags6c();
            v.set_flags6c(f | 0x800);
        }
        v.slot_15c();
        v.slot_160();
        v.update_armor();
        v.update_speed();
    }
    if mode == SET_TYPE_LOAD {
        return;
    }
    if mode == SET_TYPE_NORMAL {
        v.update_gpiece();
    }
    count_leader(v, ty, 1);
    if mode != SET_TYPE_NORMAL {
        return;
    }
    let mut i = 0;
    while i < v.guy_mark() {
        v.seat_guy(i);
        i += 1;
    }
    if !v.version_newer() {
        return;
    }
    if !v.is_supply() {
        let id = v.supply_id();
        if id >= 0 {
            v.close_supply(id);
        }
    }
    if v.is_supply() && v.supply_id() < 0 {
        v.open_supply();
    }
}
