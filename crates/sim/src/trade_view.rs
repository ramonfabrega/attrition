//! **`Group::action_trade@00701cc0`'s steps** (item 3381): a group's trade
//! order over a trade building, as a pure walk over a [`TradeView`]. The
//! swept steps read the world only through the view, so the sweep's port is
//! a view over the emulated original's windows (`docs/ORDERS.md` §29).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it, which is what the sweep holds. `p` is a
//! player and `o`/`u` an index in that player's `objects`/`units` array; the
//! original indexes both arrays by one object index. `who` is the building's
//! player (the command's second argument), `p` the group's own. The head is
//! the garrison order's, instruction for instruction
//! ([`crate::garrison_view::garrison_head`]).
//!
//! **Not wired.** The crate has no `Group::action_trade`: `group.rs`'s
//! `FinishInsertView::action_trade` is an empty stub, and the callee at the
//! walk's end, `Unit::add_trade_order@005e4dc0` with its far-city arguments,
//! is not the crate's `caravan.rs` `add_trade_order(u, home)` (the AI's
//! `QUEUE_NEW` reduction). A `SimTrade` adapter would be dead code until that
//! callee has its own sweep, and wiring the stub would add trade orders to
//! every capture whose human clicks a caravan onto a market, so it is parked
//! rather than done here.

pub use crate::garrison_view::{GarrisonHead, garrison_head};

/// `GroupData::count`'s first argument: `COUNT_TYPE`.
pub const COUNT_TYPE: i32 = 0x11;
/// The caravan's type (`0x3b`).
pub const TYPE_CARAVAN: i32 = 0x3b;
/// The second type a group may trade with (`0x13e`), which also asks the
/// unit's coast test.
pub const TYPE_TRADE_SHIP: i32 = 0x13e;

/// What `Group::action_trade` asks of the group, the world and the objects.
pub trait TradeView: GarrisonHead {
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

    /// `objects[p][o]`'s slot `+0xc`.
    fn obj_slot0c(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x4c`.
    fn obj_slot4c(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xb0`, then the data's `is_trade` (slot `+0x24`, or
    /// `WallData::is_trade` inline: `is(0x19e, 0)`).
    fn data_is_trade(&mut self, p: i32, o: i32) -> bool;
    /// `GroupData::count(kind, ty, 0)`, `kind` [`COUNT_TYPE`].
    fn count_type(&mut self, kind: i32, ty: i32) -> i32;
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// The building's tile region: `WorldData::get_tregion` of the
    /// `div_3_table` lookups of its position (the first call site).
    fn target_tregion(&mut self, p: i32, o: i32) -> i32;
    /// The same for the member `m` of the group's own player (the loop's
    /// call site).
    fn member_tregion(&mut self, p: i32, m: i32) -> i32;
    /// `objects[p][o]`'s slot `+8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x24`: the building trades by land.
    fn obj_is_trade(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x28`: the building trades by sea.
    fn obj_is_sea_trade(&mut self, p: i32, o: i32) -> bool;
    /// `units[p][u]`'s slot `+0xd0` (`UnitData::is_caravan`).
    fn unit_is_caravan(&mut self, p: i32, u: i32) -> bool;
    /// `units[p][u]`'s `is(0x13e, 0)`.
    fn unit_is_ship(&mut self, p: i32, u: i32) -> bool;
    /// `Region::is_coast(regions[unit_region], target_region)`.
    fn is_coast(&mut self, unit_region: i32, target_region: i32) -> bool;
    /// `Unit::add_trade_order(b, who, c, d, queue, 1)`, queue in the
    /// original's numbering (`QUEUE_FIRST` 0, `QUEUE_LAST` 1, `QUEUE_NEW` 2).
    fn add_trade_order(&mut self, p: i32, u: i32, args: [i32; 4], queue: i32);
}

/// `Group::action_trade(b, who, c, d, queue)`: `b` the building's index in
/// `who`'s objects, `queue` in the original's numbering.
pub fn action_trade_steps<V: TradeView>(
    v: &mut V,
    b: i32,
    who: i32,
    c: i32,
    d: i32,
    queue: i32,
) {
    garrison_head(v);
    if !v.obj_slot0c(who, b) || !v.obj_slot4c(who, b) || !v.data_is_trade(who, b) {
        return;
    }
    if v.count_type(COUNT_TYPE, TYPE_CARAVAN) == 0 && v.count_type(COUNT_TYPE, TYPE_TRADE_SHIP) == 0 {
        return;
    }
    if queue == 0 {
        // The insert dance: the group halts, takes the trade at `QUEUE_NEW`
        // and re-issues what it was doing (`finish_insert`).
        v.set_up_insert();
        v.action_halt();
        action_trade_steps(v, b, who, c, d, 2);
        v.finish_insert();
        v.list_clear();
        return;
    }
    v.set_form();
    let p = v.who() & 0xff;
    let target = v.target_tregion(who, b);
    if v.count() <= 0 {
        return;
    }
    let mut i = 0;
    loop {
        let m = v.member(i);
        trade_member(v, p, m, [b, who, c, d], queue, target);
        i += 1;
        if i >= v.count() {
            break;
        }
    }
}

/// One member of the walk: the listing's body from `00701f10` to
/// `0070209a`.
fn trade_member<V: TradeView>(
    v: &mut V,
    p: i32,
    m: i32,
    args: [i32; 4],
    queue: i32,
    target: i32,
) {
    if !v.obj_alive(p, m) || !v.obj_bc(p, m) {
        return;
    }
    let region = v.member_tregion(p, m);
    let [b, who, ..] = args;
    if v.obj_is_trade(who, b) {
        // A land trade building takes a caravan.
        if !v.unit_is_caravan(p, m) {
            return;
        }
    } else if v.obj_is_sea_trade(who, b) {
        // A sea one takes the other type, and only from the coast.
        if !v.unit_is_ship(p, m) || !v.is_coast(region, target) {
            return;
        }
    }
    // Neither: any member goes (`00702064`).
    let q = if queue == 1 { 1 } else { 2 };
    v.add_trade_order(p, m, args, q);
}
