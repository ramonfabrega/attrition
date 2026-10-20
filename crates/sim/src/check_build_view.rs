//! **`Unit::check_build_order@00603470`** (item 3372) as a pure walk over a
//! [`CheckBuildView`]: the scan of the order list that pops the queued
//! `BUILD_AT`s, and the tail that re-adds the surviving sites, least crowded
//! first. Every leaf of the view is one virtual call, one direct callee or
//! one inline read of the listing, asked in the order the listing asks it —
//! which is what the sweep holds. The crate's view is `SimCheckBuild`
//! (`orders.rs`); the sweep's is a view over the emulated original's windows.
//!
//! **The cursor.** The original walks the unit's own order list with a
//! cursor (`+0xcc` the order, `+0xd4` its node, `+0xd0` the node's flag): a
//! scan *stops* on a site (the order stays), *steps over* a plain transit
//! move, and *kills* a finished or vanished site and starts again from the
//! front. [`CheckBuildView::cursor_to_front`] and
//! [`CheckBuildView::cursor_advance`] are those two moves.
//!
//! **The repeated asks are kept.** The listing asks `get_type` up to five
//! times in an iteration, and the order's target (`vslot +0xb4`) three
//! times — once for the exists test, once for the finished test, once for
//! the site and the city test — and each ask is one view call, so a stub
//! that answers a call at a time sees the same sequence the original's does.
//! In the crate's view they are all the same pure read.

/// What `Unit::check_build_order` asks of the unit, its orders and the
/// world, in the listing's order.
pub trait CheckBuildView {
    /// `this->field_0xd8 != 0`: the unit has orders.
    fn orders_present(&mut self) -> bool;
    /// `this->field_0xdc != 0`: the order list has its end node.
    fn list_present(&mut self) -> bool;
    /// The cursor onto the front of the list (`+0xd4 = end->next`,
    /// `+0xcc = its order`, `+0xd0 = its flag`).
    fn cursor_to_front(&mut self);
    /// The cursor one node on (`+0xd4 = +0xd4->next`, …).
    fn cursor_advance(&mut self);
    /// `this->field_0xcc != 0`: the cursor holds an order.
    fn has_order(&mut self) -> bool;
    /// The cursor order's vslot `+0x10`: `get_type`.
    fn order_type(&mut self) -> i32;
    /// The cursor order's vslot `+0xb8` object, flag byte `& 4`: the
    /// order carries the action bit.
    fn order_is_action(&mut self) -> bool;
    /// `+0xd4 == +0xdc`: the cursor is on the list's end node.
    fn cursor_at_end(&mut self) -> bool;
    /// The order's target (vslot `+0xb4`), and whether it exists: inline
    /// when the target's class is `TargetOrder` (a live object whose
    /// generation matches), its vslot `+4` otherwise.
    fn target_exists(&mut self) -> bool;
    /// The order's target again, the object it names (`get_target`,
    /// inline or its vslot `+0`), vslot `+0x4c`: the site is finished.
    fn target_finished(&mut self) -> bool;
    /// The order's target a third time, the object's `+0xa` — the site's
    /// index — which the object it holds stays the subject of
    /// [`Self::target_blocks`].
    fn target_site(&mut self) -> i32;
    /// The object [`Self::target_site`] named: flags `& 0x20`, or its
    /// type's vslot `+0xfc` or `+0x100`, which ends the scan on it.
    fn target_blocks(&mut self) -> bool;
    /// `Unit::repath@005e29b0`.
    fn repath(&mut self);
    /// `Unit::kill_current_order@005e2cb0(0)`.
    fn kill_current_order(&mut self);
    /// `Unit::add_build_order@005e5210(site, owner, QUEUE_FIRST, 1)`.
    fn add_build_order(&mut self, site: i32);
    /// The unit's owner byte (`+9`).
    fn owner(&mut self) -> i32;
    /// `Objects::find_units@0065a620` for the unit's friends in
    /// `unit_build_respond_range * 0xc0`, filter `BUILDREPAIR`: the found
    /// list the next calls index.
    fn find_units(&mut self);
    /// The found list's length (`objects +0x208`).
    fn found_count(&mut self) -> i32;
    /// Found unit `j`'s data (vslot `+0xa8`) is not this unit's own
    /// (`+0xa` differs).
    fn found_other(&mut self, j: i32) -> bool;
    /// `UnitData::get_action@00608450` of found unit `j` is an order of
    /// type 6 (`BUILD_AT`).
    fn found_builds(&mut self, j: i32) -> bool;
    /// Found unit `j`'s action again, its record (vslot `+0xd8`): the
    /// order's `(owner, site)` (`+0xc`, `+8`).
    fn found_target(&mut self, j: i32) -> (i32, i32);
}

/// `get_type` of `MOVE_TO`.
pub const MOVE_TO: i32 = 1;
/// `get_type` of `EXPLORE_TO`.
pub const EXPLORE_TO: i32 = 3;
/// `get_type` of `BUILD_AT`.
pub const BUILD_AT: i32 = 6;

/// `Unit::check_build_order`: scan, then re-add.
pub fn check_build_order_steps<V: CheckBuildView + ?Sized>(v: &mut V) {
    let mut sites: Vec<i32> = Vec::new();
    if !v.orders_present() {
        return;
    }
    if v.list_present() {
        v.cursor_to_front();
    }
    while v.has_order() {
        if v.order_type() != BUILD_AT && v.order_type() != MOVE_TO {
            break;
        }
        if v.order_type() == MOVE_TO || v.order_type() == EXPLORE_TO {
            // A transit move is stepped over — unless it carries the action
            // bit, or is the last node.
            if v.order_is_action() || v.cursor_at_end() {
                break;
            }
            if v.list_present() {
                v.cursor_advance();
            }
            continue;
        }
        if v.order_type() != BUILD_AT {
            break;
        }
        if v.target_exists() && !v.target_finished() {
            sites.push(v.target_site());
            // The site the scan stops on keeps its order.
            if v.target_blocks() {
                break;
            }
        }
        v.repath();
        v.kill_current_order();
        if !v.orders_present() {
            break;
        }
        if v.list_present() {
            v.cursor_to_front();
        }
    }
    if sites.is_empty() {
        return;
    }
    if sites.len() == 1 {
        v.add_build_order(sites[0]);
        return;
    }
    v.find_units();
    let owner = v.owner();
    let mut best_count = 999_999;
    let mut best = 0usize;
    for (k, &site) in sites.iter().enumerate() {
        let mut count = 0;
        let mut j = 0;
        while j < v.found_count() {
            if v.found_other(j) && v.found_builds(j) {
                let (o, s) = v.found_target(j);
                if o == owner && s == site {
                    count += 1;
                }
            }
            j += 1;
        }
        if count < best_count {
            best_count = count;
            best = k;
        }
    }
    sites.swap(0, best);
    for &site in sites.iter().rev() {
        v.add_build_order(site);
    }
}
