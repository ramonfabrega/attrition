//! **`Group::action_garrison@00700490`'s steps** (item 3376): a group's
//! garrison order over a building, as a pure walk over a [`GarrisonView`].
//! The swept steps read the world only through the view, so the sweep's port
//! is a view over the emulated original's windows and the crate's is
//! [`SimGarrison`] (`docs/ORDERS.md` §29, `docs/GOLDEN.md` §49).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it, which is what the sweep holds. `p` is a
//! player and `o`/`u` an index in that player's `objects`/`units` array; the
//! original indexes both arrays by one object index. `who` is the
//! building's player (the command's second argument), `p` the group's own.

/// The head of `Group::action_garrison`: the scenario's ignore-orders sweep
/// and the group's refresh, which every entry runs, the `QUEUE_FIRST`
/// re-entry's too.
pub trait GarrisonHead {
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
    /// The group's slot `+0x14()`.
    fn refresh(&mut self);
}

/// What `Group::action_garrison` asks of the group, the world and the
/// objects.
pub trait GarrisonView: GarrisonHead {
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

    /// `leaders[p]`'s `+8`: the player's own leader index.
    fn leader_self(&self, p: i32) -> i32;
    /// `leaders[p].diplos[q]`.
    fn diplo(&self, p: i32, q: i32) -> i32;
    /// `leadersc->list[q].diplos[f]`.
    fn diploc(&self, q: i32, f: i32) -> i32;
    /// `MiscAccess::console->who`.
    fn console_who(&self) -> i32;
    /// The red feedback line `msg` (the string table's offset) through
    /// `MessageWin::add_feedback` and `S_INVALID_ORDER`'s sound. `depth` is
    /// the `QUEUE_FIRST` re-entry's: its frame sits that much deeper.
    fn refuse(&mut self, msg: i32, depth: i32);

    /// `objects[p][o]`'s slot `+0x10`.
    fn obj_slot10(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the garrison limit (`Build`: its type's
    /// `get_garrison_limit(level)`, else slot `+0x178`).
    fn data_limit(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then `is(0x1bf, 0)`: the data's slot `+0xb8`, or on a
    /// `Build` its type's slot `+0x60`.
    fn data_is_1bf(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xac`, then `BuildData::is_unassimilated`.
    fn data_unassimilated(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xac`, then `ObjectData::num_inside(data, 0)`.
    fn data_num_inside(&mut self, p: i32, o: i32) -> i32;
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `objects[p][o]`'s slot `+8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// The unit's type is not a plane: its `+0x218` is not 2, or its
    /// `+0x2b4 & 0x20`.
    fn unit_not_plane(&self, p: i32, u: i32) -> bool;
    /// `UnitData::is_entering_or_exiting`.
    fn unit_busy(&mut self, p: i32, u: i32) -> bool;
    /// `UnitData::get_action`'s order's slot `+0x10`, none for a null order.
    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32>;
    /// `Unit::repath`.
    fn repath(&mut self, p: i32, u: i32);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self, p: i32, u: i32);
    /// Slot `+0xac` of the building `b` of `who`, then
    /// `UnitTypeData::can_garrison(unit's type, the data's type index)`.
    fn can_garrison(&mut self, p: i32, u: i32, who: i32, b: i32) -> bool;
    /// Slot `+0xac` of the building, its short `+0x72`, then
    /// `Unit::find_garrison_build(city, who)`.
    fn find_garrison_build(&mut self, p: i32, u: i32, who: i32, b: i32) -> i32;
    /// `Game::semaphore.ptr[1] & 8`: the editor's instant garrison.
    fn semaphore(&self) -> bool;
    /// `units[p][u]`'s population: 0 for a decoy (`+0x68 & 1`), else its
    /// type's `+0x2f0`.
    fn unit_pop(&self, p: i32, u: i32) -> i32;
    /// `Unit::go_inside(target, who, 0)`.
    fn go_inside(&mut self, p: i32, u: i32, target: i32, who: i32);
    /// `ObjectData::is_worker`.
    fn unit_is_worker(&mut self, p: i32, u: i32) -> bool;
    /// The unit's type `+0x2b8 & 4`: a type that packs.
    fn unit_packs(&self, p: i32, u: i32) -> bool;
    /// `UnitData::is_packing`.
    fn unit_is_packing(&mut self, p: i32, u: i32) -> bool;
    /// `UnitData::is_unpacking`.
    fn unit_is_unpacking(&mut self, p: i32, u: i32) -> bool;
    /// The unit's order list's length, `+0xd8`.
    fn list_len(&self, p: i32, u: i32) -> i32;
    /// The list's `tail()`.
    fn list_tail(&mut self, p: i32, u: i32);
    /// The list's current data, the word at `+0xcc`.
    fn list_current(&self, p: i32, u: i32) -> i32;
    /// The list's `remove_current()`.
    fn list_remove_current(&mut self, p: i32, u: i32);
    /// `Unit::clear_orders`.
    fn clear_orders(&mut self, p: i32, u: i32);
    /// The list's `add(data)`.
    fn list_add(&mut self, p: i32, u: i32, data: i32);
    /// `Unit::update_gpiece`.
    fn update_gpiece(&mut self, p: i32, u: i32);
    /// `Unit::add_garrison_order(target, who, search, queue, 1)`, queue in
    /// the original's numbering (`QUEUE_FIRST` 0, `QUEUE_LAST` 1,
    /// `QUEUE_NEW` 2).
    fn add_garrison_order(
        &mut self,
        p: i32,
        u: i32,
        target: i32,
        who: i32,
        search: i32,
        queue: i32,
    );
}

/// The first part of the listing, up to and including the group's refresh
/// (`00700523`): the scenario's ignore-orders sweep. Runs on every entry,
/// the re-entry's too.
pub fn garrison_head<V: GarrisonHead>(v: &mut V) {
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
    v.refresh();
}

/// The string table's offset of the refusal an ally check gives.
pub const MSG_NOT_ALLY: i32 = 0x84e4;
/// The refusal a building not yet assimilated gives.
pub const MSG_UNASSIMILATED: i32 = 0x84f8;

/// `Group::action_garrison(b, who, queue, search)`: `b` the building's
/// index in `who`'s objects, `queue` in the original's numbering and
/// `search` the City's alarm's argument. `depth` counts the
/// `QUEUE_FIRST` re-entries above this call.
pub fn action_garrison_steps<V: GarrisonView>(
    v: &mut V,
    b: i32,
    who: i32,
    queue: i32,
    search: i32,
    depth: i32,
) {
    garrison_head(v);
    if queue == 0 {
        // The insert dance: the group halts, takes the garrison at
        // `QUEUE_NEW` and re-issues what it was doing (`finish_insert`).
        v.set_up_insert();
        v.action_halt();
        action_garrison_steps(v, b, who, 2, 0, depth + 1);
        v.finish_insert();
        v.list_clear();
        return;
    }
    let p = v.who() & 0xff;
    v.set_form();
    let own = v.leader_self(p);
    if who != own && !(v.diplo(p, who) == 2 && v.diploc(who, own) == 2) {
        if p == v.console_who() {
            v.refuse(MSG_NOT_ALLY, depth);
        }
        return;
    }
    if v.obj_slot10(who, b) == 0 {
        return;
    }
    if v.data_limit(who, b) == 0 && !v.data_is_1bf(who, b) {
        return;
    }
    if v.data_unassimilated(who, b) {
        if p == v.console_who() {
            v.refuse(MSG_UNASSIMILATED, depth);
        }
        return;
    }
    if v.count() <= 0 {
        return;
    }
    let mut i = 0;
    loop {
        let m = v.member(i);
        garrison_member(v, p, m, b, who, queue, search);
        i += 1;
        if i >= v.count() {
            break;
        }
    }
}

/// One member of the walk: the listing's body from `007007a0` to
/// `00700b63`.
fn garrison_member<V: GarrisonView>(
    v: &mut V,
    p: i32,
    m: i32,
    b: i32,
    who: i32,
    queue: i32,
    search: i32,
) {
    if !v.obj_alive(p, m) || !v.obj_bc(p, m) {
        return;
    }
    if !v.unit_not_plane(p, m) || v.unit_busy(p, m) {
        return;
    }
    if v.unit_order_kind(p, m) == Some(0x1a) {
        v.repath(p, m);
        v.kill_current_order(p, m);
    }
    if !v.can_garrison(p, m, who, b) {
        return;
    }
    // `edi` carries the last target between members; a negative one, and
    // any call that does not search, lands on the building itself.
    let mut target = if search != 0 {
        v.find_garrison_build(p, m, who, b)
    } else {
        b
    };
    if target < 0 {
        target = b;
    }
    if queue == 2 {
        if v.semaphore() {
            let limit = v.data_limit(who, target);
            let pop = v.unit_pop(p, m);
            if v.data_num_inside(who, target).wrapping_add(pop) <= limit {
                v.go_inside(p, m, target, who);
            }
            return;
        }
        if v.unit_is_worker(p, m) {
            v.add_garrison_order(p, m, target, who, search, 0);
            return;
        }
        if v.unit_packs(p, m) {
            let packing = v.unit_is_packing(p, m);
            if packing || v.unit_is_unpacking(p, m) {
                if v.unit_is_unpacking(p, m) {
                    v.add_garrison_order(p, m, target, who, search, 2);
                } else {
                    // A packing unit keeps its head order, and the garrison
                    // goes in behind it.
                    if v.list_len(p, m) >= 1 {
                        v.list_tail(p, m);
                        let cur = v.list_current(p, m);
                        v.list_remove_current(p, m);
                        v.clear_orders(p, m);
                        v.list_add(p, m, cur);
                        v.update_gpiece(p, m);
                    }
                    v.add_garrison_order(p, m, target, who, search, 1);
                }
                return;
            }
        }
    }
    v.add_garrison_order(p, m, target, who, search, queue);
}

/// This crate's [`GarrisonView`]: the group, and the world as the crate
/// keeps it. A member is a unit index and the building a building index;
/// the original's one object index is told apart by which method asks.
/// Where the crate models less than the original:
///
/// - SEAM: the scenario's ignore-orders sweep and the group's refresh are
///   not modelled (no list, no slot), nor the console's feedback line and
///   its sound (no console).
/// - SEAM: the insert dance of the first queue position runs as its walk
///   alone: the group has no insertion list here, so the set-up, the halt,
///   the finish and the list's clear are nothing.
/// - SEAM: the zero-limit arm's second test is taken as a refusal; the
///   editor's instant garrison (the game's semaphore bit) is not modelled
///   and neither is the busy test (a unit entering or leaving a building);
///   a unit type that packs takes the plain arm.
pub struct SimGarrison<'a> {
    pub sim: &'a mut crate::Sim,
    pub g: &'a crate::group::Group,
}

fn pos_of(queue: i32) -> crate::orders::QueuePos {
    match queue {
        0 => crate::orders::QueuePos::First,
        1 => crate::orders::QueuePos::Last,
        _ => crate::orders::QueuePos::New,
    }
}

impl GarrisonHead for SimGarrison<'_> {
    fn who(&self) -> i32 {
        i32::from(self.g.who)
    }
    fn ignore_orders(&self) -> bool {
        false
    }
    fn ignoring_count(&self, _who: i32) -> i32 {
        0
    }
    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
        -1
    }
    fn drop_ignored(&mut self, _o: i32, _who: i32) {}
    fn refresh(&mut self) {}
}

impl GarrisonView for SimGarrison<'_> {
    fn set_up_insert(&mut self) {}
    fn action_halt(&mut self) {}
    fn finish_insert(&mut self) {}
    fn list_clear(&mut self) {}
    fn set_form(&mut self) {
        if let Some(st) = self.sim.gstate_mut(self.g) {
            st.form = -1;
        }
    }
    fn leader_self(&self, p: i32) -> i32 {
        p
    }
    fn diplo(&self, p: i32, q: i32) -> i32 {
        if self
            .sim
            .allied_with(p as crate::world::Player, q as crate::world::Player)
        {
            2
        } else {
            0
        }
    }
    fn diploc(&self, q: i32, f: i32) -> i32 {
        self.diplo(q, f)
    }
    fn console_who(&self) -> i32 {
        -1
    }
    fn refuse(&mut self, _msg: i32, _depth: i32) {}
    fn obj_slot10(&mut self, _p: i32, o: i32) -> i32 {
        let bd = &self.sim.buildings[o as usize];
        i32::from(bd.alive && bd.ty.is_some())
    }
    fn data_limit(&mut self, _p: i32, o: i32) -> i32 {
        self.sim.garrison_limit(o as usize)
    }
    fn data_is_1bf(&mut self, _p: i32, _o: i32) -> bool {
        false
    }
    fn data_unassimilated(&mut self, _p: i32, o: i32) -> bool {
        // `BuildData::is_unassimilated`: a city whose race is not its owner.
        let o = o as usize;
        self.sim.building_is_city(o)
            && self.sim.buildings[o].city.is_some_and(|c| {
                self.sim.cities[c].alive
                    && self.sim.cities[c].race != Some(self.sim.buildings[o].owner)
            })
    }
    fn data_num_inside(&mut self, _p: i32, o: i32) -> i32 {
        self.sim.num_inside(o as usize)
    }
    fn count(&self) -> i32 {
        self.g.list.len() as i32
    }
    fn member(&self, i: i32) -> i32 {
        self.g.list[i as usize] as i32
    }
    fn obj_alive(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].alive()
    }
    fn obj_bc(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].on_map
    }
    fn unit_not_plane(&self, _p: i32, u: i32) -> bool {
        !self.sim.is_plane(u as usize)
    }
    fn unit_busy(&mut self, _p: i32, _u: i32) -> bool {
        false
    }
    fn unit_order_kind(&mut self, _p: i32, u: i32) -> Option<i32> {
        let u = u as usize;
        self.sim.action_of(u).map(|a| {
            if matches!(
                self.sim.units[u].orders[a].body,
                crate::orders::Body::Garrison { .. }
            ) {
                0x1a
            } else {
                0
            }
        })
    }
    fn repath(&mut self, _p: i32, u: i32) {
        self.sim.repath(u as usize);
    }
    fn kill_current_order(&mut self, _p: i32, u: i32) {
        self.sim.kill_current_order(u as usize);
    }
    fn can_garrison(&mut self, _p: i32, u: i32, _who: i32, b: i32) -> bool {
        self.sim.units[u as usize]
            .ty
            .zip(self.sim.buildings[b as usize].ty)
            .is_some_and(|(ut, bt)| self.sim.can_garrison(ut, bt))
    }
    fn find_garrison_build(&mut self, _p: i32, u: i32, who: i32, b: i32) -> i32 {
        let city = self.sim.buildings[b as usize].city;
        self.sim
            .find_garrison_build(u as usize, city, who as crate::world::Player)
            .map_or(-1, |t| t as i32)
    }
    fn semaphore(&self) -> bool {
        false
    }
    fn unit_pop(&self, _p: i32, u: i32) -> i32 {
        self.sim.control_cost(u as usize)
    }
    fn go_inside(&mut self, _p: i32, u: i32, target: i32, _who: i32) {
        self.sim.go_inside(u as usize, target as usize);
    }
    fn unit_is_worker(&mut self, _p: i32, u: i32) -> bool {
        self.sim.is_worker(u as usize)
    }
    fn unit_packs(&self, _p: i32, _u: i32) -> bool {
        false
    }
    fn unit_is_packing(&mut self, _p: i32, _u: i32) -> bool {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn unit_is_unpacking(&mut self, _p: i32, _u: i32) -> bool {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn list_len(&self, _p: i32, _u: i32) -> i32 {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn list_tail(&mut self, _p: i32, _u: i32) {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn list_current(&self, _p: i32, _u: i32) -> i32 {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn list_remove_current(&mut self, _p: i32, _u: i32) {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn clear_orders(&mut self, _p: i32, _u: i32) {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn list_add(&mut self, _p: i32, _u: i32, _data: i32) {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn update_gpiece(&mut self, _p: i32, _u: i32) {
        unreachable!("a packing unit type takes the plain arm here")
    }
    fn add_garrison_order(
        &mut self,
        _p: i32,
        u: i32,
        target: i32,
        _who: i32,
        search: i32,
        queue: i32,
    ) {
        self.sim.add_garrison_order(
            u as usize,
            target as usize,
            search != 0,
            pos_of(queue),
            true,
        );
    }
}
