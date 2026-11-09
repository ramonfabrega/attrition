//! **`Group::action_board_ship@00700010`'s steps** (item 3386): a group's
//! board order over a ship, as a pure walk over a [`BoardShipView`]. The
//! swept steps read the world only through the view, so the sweep's port is
//! a view over the emulated original's windows (`docs/ORDERS.md` §29,
//! `docs/TRANSPORT.md` §15).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it, which is what the sweep holds. `p` is a
//! player and `o`/`u` an index in that player's `objects`/`units` array; the
//! original indexes both arrays by one object index. Unlike the trade and
//! garrison orders the ship is the group's own player's (`objects[who][o]`
//! and `units[who][o]`, `who` = `[this + 0x4a]`): the command carries one
//! index, not a player. The head is the garrison order's, instruction for
//! instruction ([`crate::garrison_view::garrison_head`]).
//!
//! **Not wired.** The crate has no `Group::action_board_ship`:
//! `group.rs`'s `FinishInsertView::action_board_ship` is an empty stub, and
//! its callees (`Unit::add_board_order@005e4d10`,
//! `Unit::add_await_board_order@005e4c80`, `ObjectData::can_carry@006483c0`)
//! have no crate twin: no game reaches a `BoardOrder`
//! (`docs/TRANSPORT.md` §15, "built by nothing a game reaches"), and wiring
//! the stub would add boarding orders to every `finish_insert` replay, so it
//! is parked rather than done here.

pub use crate::garrison_view::{GarrisonHead, garrison_head};

/// `QueuePos`: the order goes to the front, after the others, or alone.
pub const QUEUE_FIRST: i32 = 0;
/// `QueuePos::QUEUE_LAST`.
pub const QUEUE_LAST: i32 = 1;
/// `QueuePos::QUEUE_NEW`.
pub const QUEUE_NEW: i32 = 2;

/// What `Group::action_board_ship` asks of the group, the world and the
/// objects.
pub trait BoardShipView: GarrisonHead {
    /// `Group::set_up_insert(&list)`: the `QUEUE_FIRST` re-entry's start.
    fn set_up_insert(&mut self);
    /// `Group::action_halt(0)`.
    fn action_halt(&mut self);
    /// `Group::finish_insert(&list)`.
    fn finish_insert(&mut self);
    /// The local order list's `clear`.
    fn list_clear(&mut self);
    /// `this + 0x10 = -1`: the group's formation is forgotten.
    fn set_form(&mut self);
    /// `Group::clear(-1)` on the local group the replay builds.
    fn clear_group(&mut self);
    /// `Group::add(o, who, 0, 0)` on that local group.
    fn add_to_group(&mut self, o: i32, who: i32);

    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// The ship's tile region: `WorldData::get_tregion` of the
    /// `div_3_table` lookups of `objects[p][o]`'s position (the first call
    /// site).
    fn target_tregion(&mut self, p: i32, o: i32) -> i32;
    /// The same for the member `m` (the loop's call site).
    fn member_tregion(&mut self, p: i32, m: i32) -> i32;
    /// `objects[p][o]`'s slot `+8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// `units[p][u]`'s type is a plane: its `+0x218` is 2.
    fn unit_is_plane(&self, p: i32, u: i32) -> bool;
    /// `UnitData::is_busy(units[p][u])`.
    fn unit_busy(&mut self, p: i32, u: i32) -> bool;
    /// `Region::is_coast(regions[unit_region], target_region)`.
    fn is_coast(&mut self, unit_region: i32, target_region: i32) -> bool;
    /// `ObjectData::can_carry(units[p][ship], m, p)`.
    fn can_carry(&mut self, p: i32, ship: i32, m: i32) -> bool;
    /// `Unit::add_board_order(units[p][m], ship, p, queue, …)`, queue in the
    /// original's numbering (`QUEUE_LAST` 1, `QUEUE_NEW` 2).
    fn add_board_order(&mut self, p: i32, m: i32, ship: i32, queue: i32);
    /// `Unit::clear_orders(units[p][ship])`: the ship's own orders go once.
    fn clear_orders(&mut self, p: i32, ship: i32);
    /// `Unit::add_await_board_order(units[p][ship], m, p, …, 1)`.
    fn add_await_board_order(&mut self, p: i32, ship: i32, m: i32);
    /// `MiscAccess::console->who`.
    fn console_who(&self) -> i32;
    /// `MessageWin::add_message` of the "some could not board" line, at the
    /// ship's position (`objects[p][o]`'s `+0x10`/`+0x14`, unmasked by
    /// `0x63637`).
    fn add_message(&mut self, p: i32, o: i32);
    /// The red feedback line through `MessageWin::add_feedback` and
    /// `S_INVALID_ORDER`'s sound.
    fn refuse(&mut self);
}

/// `Group::action_board_ship(o, queue)`: `o` the ship's index in the
/// group's player's objects, `queue` in the original's numbering.
pub fn action_board_ship_steps<V: BoardShipView>(v: &mut V, o: i32, queue: i32) {
    garrison_head(v);
    if queue == QUEUE_FIRST {
        // The insert dance: the group halts, boards at `QUEUE_NEW` and
        // re-issues what it was doing (`finish_insert`).
        v.set_up_insert();
        v.action_halt();
        action_board_ship_steps(v, o, QUEUE_NEW);
        v.clear_group();
        v.clear_group();
        let who = v.who() & 0xff;
        v.add_to_group(o, who);
        v.finish_insert();
        v.list_clear();
        return;
    }
    let who = v.who() & 0xff;
    v.set_form();
    let target = v.target_tregion(who, o);
    let mut failed = 0;
    let mut boarded = 0;
    if v.count() > 0 {
        let mut i = 0;
        loop {
            let m = v.member(i);
            board_member(v, (who, o, m), queue, target, &mut failed, &mut boarded);
            i += 1;
            if i >= v.count() {
                break;
            }
        }
    }
    if v.console_who() == v.who() & 0xff && failed != 0 {
        if boarded != 0 {
            v.add_message(who, o);
        } else {
            v.refuse();
        }
    }
}

/// One member of the walk: the listing's body from `00700211` to
/// `007003b4`.
fn board_member<V: BoardShipView>(
    v: &mut V,
    (who, o, m): (i32, i32, i32),
    queue: i32,
    target: i32,
    failed: &mut i32,
    boarded: &mut i32,
) {
    if !v.obj_alive(who, m) || !v.obj_bc(who, m) || v.unit_is_plane(who, m) || v.unit_busy(who, m) {
        return;
    }
    let region = v.member_tregion(who, m);
    if !v.is_coast(region, target) {
        return;
    }
    if !v.can_carry(who, o, m) {
        *failed += 1;
        return;
    }
    let q = if queue == QUEUE_LAST {
        QUEUE_LAST
    } else {
        QUEUE_NEW
    };
    v.add_board_order(who, m, o, q);
    if *boarded == 0 {
        v.clear_orders(who, o);
    }
    *boarded += 1;
    if who == v.who() & 0xff {
        v.add_await_board_order(who, o, m);
    }
}
