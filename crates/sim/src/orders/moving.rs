//! The move order: `Unit::do_move`, the group move, the path and step, arrival, the nearby-spot search and the ungroup walk (STRUCTURE §5, move 7).

use super::*;

/// `Unit::repath@005e29b0`, in the original's own call order (item 2341's
/// sweep, `crate::sweep::unit_repath`): while there is a head, `get_type`
/// (vslot `+0x10`) is asked; outside the move family (`1, 2, 3, 4, 0x12,
/// 0x13, 0x15`) it is asked again and the walk stops unless that answer
/// is `CHANGE_FORM` without the action bit (dead with one answer a head,
/// but a second call all the same). In the family, `get_type` a third
/// time: a `GROUP_MOVE` of a unit whose group (`+0x80`) is not `-1` has
/// `Group::kill_group_move(groups[group], get_group_move_order()->id)`
/// (`+0x10c`, `+0xc`), anything else `kill_current_order(0)`, and the
/// walk goes round. The action bit is never read on the family's arm.
#[allow(clippy::too_many_arguments)]
pub(crate) fn repath_steps<C, G, I>(
    cx: &mut C,
    has_head: impl Fn(&mut C) -> bool,
    mut get_type: impl FnMut(&mut C) -> i32,
    action: impl Fn(&mut C) -> bool,
    group: impl Fn(&mut C) -> Option<G>,
    mut group_id: impl FnMut(&mut C) -> I,
    mut kill_group_move: impl FnMut(&mut C, G, I),
    mut kill_current: impl FnMut(&mut C),
) {
    let family = |t: i32| u8::try_from(t).is_ok_and(index::is_move_family);
    loop {
        if !has_head(cx) {
            return;
        }
        if !family(get_type(cx))
            && (!has_head(cx) || get_type(cx) != i32::from(index::CHANGE_FORM) || action(cx))
        {
            return;
        }
        if has_head(cx)
            && get_type(cx) == i32::from(index::GROUP_MOVE)
            && let Some(g) = group(cx)
        {
            let id = group_id(cx);
            kill_group_move(cx, g, id);
            continue;
        }
        kill_current(cx);
    }
}

/// `Unit::kill_group_move@005e3400`, in the original's own call order
/// (item 2368's sweep, `crate::sweep::unit_kill_group_move`). With a head
/// (`+0xdc`), the cursor starts on the first node and, for each order:
/// `is_move` (vslot `+0x14`), `get_type` (`+0x10`) not `GROUP_ATTACK_TO`
/// and `is_pathed` (`+0x24`) clear the order's `PATHED` bit and
/// `kill_current_path` (`5e345d`-`5e348a`); then `get_group_move_order`
/// (`+0x94`), its id (`+0xc`) against `id`, and the same record's
/// `is_move` and `get_type` not `GROUP_ATTACK_TO` call
/// `kill_current_order(0)` **inside the walk** (`5e3497`-`5e34ce`), which
/// moves the cursor to the new head. The cursor advances while there is a
/// head, and the walk goes round while the count (`+0xd8`) is not zero and
/// the advanced node's `+0` is not the tail (`5e34d3`-`5e351a`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn kill_group_move_steps<C, K: Copy>(
    cx: &mut C,
    id: i64,
    first: impl Fn(&mut C) -> Option<K>,
    has_head: impl Fn(&mut C) -> bool,
    next: impl Fn(&mut C, K) -> K,
    more: impl Fn(&mut C, K) -> bool,
    mut is_move: impl FnMut(&mut C, K) -> bool,
    mut get_type: impl FnMut(&mut C, K) -> i32,
    mut is_pathed: impl FnMut(&mut C, K) -> bool,
    mut unpath: impl FnMut(&mut C, K),
    mut kill_path: impl FnMut(&mut C),
    mut group_id: impl FnMut(&mut C, K) -> Option<i64>,
    mut kill_current: impl FnMut(&mut C, K) -> K,
) {
    let Some(mut k) = first(cx) else {
        return;
    };
    let gat = i32::from(index::GROUP_ATTACK_TO);
    loop {
        if is_move(cx, k) && get_type(cx, k) != gat && is_pathed(cx, k) {
            unpath(cx, k);
            kill_path(cx);
        }
        if group_id(cx, k) == Some(id) && is_move(cx, k) && get_type(cx, k) != gat {
            k = kill_current(cx, k);
        }
        if has_head(cx) {
            k = next(cx, k);
        }
        if !more(cx, k) {
            return;
        }
    }
}

/// What `Unit::ungroup_move_order@005fd140` reads and does, one method a
/// step of the original's own order (item 3268's sweep,
/// `crate::sweep::unit_ungroup_move_order`). The original keeps each
/// unit's orders in a circular list whose **head** is the order at the
/// *back* of this crate's deque and whose last link is the front, a
/// cursor walking `next` from the head; every method here is one read or
/// write of that list or one callee of the function.
pub trait UngroupView {
    /// A unit.
    type U: Copy;
    /// An order object on a unit's list.
    type O: Copy;
    /// The group order's record (`get_group_move_order`'s answer).
    type G: Copy;
    /// A fresh order object (`OrdersMemManager::get_obj`'s answer).
    type N: Copy;
    /// `UnitData::is_captain` (vslot `+0xe8`).
    fn is_captain(&mut self, u: Self::U) -> bool;
    /// `units[player][o_up]`; `None` is the port's guard, never the
    /// original's (it walks whatever the table holds).
    fn up(&mut self, u: Self::U) -> Option<Self::U>;
    /// `units[player][o_down]` when `o_down >= 0` and that unit's `+8`
    /// bit 0 is set, else the walk ends.
    fn down(&mut self, u: Self::U) -> Option<Self::U>;
    /// With a head (`+0xdc`) the cursor moves onto it (`+0xd4`, data
    /// `+0xcc`, metric `+0xd0`).
    fn start_cursor(&mut self, u: Self::U);
    /// The list's count (`+0xd8`) is not zero.
    fn has_orders(&mut self, u: Self::U) -> bool;
    /// The cursor's data (`+0xcc`), `None` when null.
    fn cursor_order(&mut self, u: Self::U) -> Option<Self::O>;
    /// vslot `+0x110` is non-null and its record's `+0x5c` is `id` —
    /// asked twice (null test, then the value) when the first is non-null.
    fn carries(&mut self, o: Self::O, id: i64) -> bool;
    /// The list has a head (`+0xdc` non-null).
    fn has_head(&mut self, u: Self::U) -> bool;
    /// `d4 = [d4]` and the cursor's data and metric follow.
    fn advance(&mut self, u: Self::U);
    /// The cursor node is the head (`+0xd4 == +0xdc`).
    fn at_head(&mut self, u: Self::U) -> bool;
    /// vslot `+0x98`: the group order's record.
    fn group_record(&mut self, o: Self::O) -> Self::G;
    /// The record names `u` itself (`+0x54` the unit's id, `+0x58` its
    /// player).
    fn is_leader(&mut self, u: Self::U, g: Self::G) -> bool;
    /// vslot `+0x10` of the record: its `OrderIndex`.
    fn kind(&mut self, g: Self::G) -> i32;
    /// `OrdersMemManager::get_obj(kind)`.
    fn get_obj(&mut self, kind: i32) -> Self::N;
    /// The `GROUP_ATTACK_TO` arm's casts and `MoveOrder::operator=`.
    fn assign_attack(&mut self, n: Self::N, g: Self::G);
    /// The `GROUP_MOVE` arm's cast and `MoveOrder::operator=`.
    fn assign_move(&mut self, n: Self::N, g: Self::G);
    /// A non-leader's new order loses `PATHED` (when `!leader`), then
    /// `dest = 0` and `orig = (x, y)`.
    fn finish(&mut self, n: Self::N, leader: bool);
    /// `LinkListBase::remove_current` on the cursor.
    fn remove_current(&mut self, u: Self::U);
    /// `OrdersMemManager::give_obj(old)`.
    fn give_obj(&mut self, o: Self::O);
    /// `LinkListBase::add(new)`.
    fn add(&mut self, u: Self::U, n: Self::N);
    /// `head = head->next`: the fix-up after the add.
    fn head_next(&mut self, u: Self::U);
    /// `Unit::kill_current_path`.
    fn kill_current_path(&mut self, u: Self::U);
}

/// The whole of `Unit::ungroup_move_order@005fd140` (item 3268). `skip_walk`
/// is the second argument (every caller passes 0): non-zero skips the walk
/// up to the captain. The walk goes up while `is_captain` is false, runs
/// [`ungroup_unit_steps`], and goes down `o_down` while the next unit
/// lives, the later units skipping the walk up.
pub fn ungroup_move_order_steps<V: UngroupView>(v: &mut V, this: V::U, id: i64, skip_walk: bool) {
    let mut at = this;
    let mut skip = skip_walk;
    loop {
        if !skip {
            loop {
                if v.is_captain(at) {
                    break;
                }
                match v.up(at) {
                    Some(n) => at = n,
                    None => break,
                }
            }
        }
        ungroup_unit_steps(v, at, id);
        match v.down(at) {
            Some(d) => {
                at = d;
                skip = true;
            }
            None => return,
        }
    }
}

/// One unit of the walk (`005fd19b`-`005fd3c6`): the cursor goes to the
/// head; while the list has a count and the cursor a datum, the orders
/// are asked from the head round (`005fd1e0`-`005fd230`) until one carries
/// `id`, or the cursor is back on the head, whose datum is asked once more
/// (`005fd235`). The record's kind `GROUP_ATTACK_TO` (`0x15`) or
/// `GROUP_MOVE` (`0x13`) builds the plain order in a fresh object, takes
/// the old one off the list, gives it back, and **adds** the new one
/// (which makes it the head and the cursor) and then sets the head to the
/// head's `next` — so the net is the new order at the list's **last**
/// link (this crate's front) with the head where it was
/// (`005fd3ab`). A non-leader loses its path whatever the kind.
pub fn ungroup_unit_steps<V: UngroupView>(v: &mut V, u: V::U, id: i64) {
    v.start_cursor(u);
    if !v.has_orders(u) {
        return;
    }
    let Some(mut o) = v.cursor_order(u) else {
        return;
    };
    let o = loop {
        if v.carries(o, id) {
            break o;
        }
        if v.has_head(u) {
            v.advance(u);
        }
        let at_head = v.at_head(u);
        let next = v.cursor_order(u);
        if at_head {
            match next {
                Some(x) => break x,
                None => return,
            }
        }
        match next {
            Some(x) => o = x,
            None => return,
        }
    };
    if !v.carries(o, id) {
        return;
    }
    let g = v.group_record(o);
    let leader = v.is_leader(u, g);
    if v.kind(g) == i32::from(index::GROUP_ATTACK_TO) {
        let n = v.get_obj(i32::from(index::ATTACK_TO));
        v.assign_attack(n, g);
        v.finish(n, leader);
        v.remove_current(u);
        v.give_obj(o);
        v.add(u, n);
        v.head_next(u);
    } else if v.kind(g) == i32::from(index::GROUP_MOVE) {
        let n = v.get_obj(i32::from(index::MOVE_TO));
        v.assign_move(n, g);
        v.finish(n, leader);
        v.remove_current(u);
        v.give_obj(o);
        v.add(u, n);
        v.head_next(u);
    }
    if !leader {
        v.kill_current_path(u);
    }
}

/// [`UngroupView`] over this crate's own lists. The ring's head is the
/// deque's back and its last link the front, so the cursor walks the deque
/// from the back towards the front and wraps; the matched order is
/// taken off in [`UngroupView::remove_current`] and pushed on the front
/// in [`UngroupView::add`] (`docs/ORDERS.md` §20.2).
pub(super) struct SimUngroup<'a> {
    pub(super) sim: &'a mut Sim,
    /// The unit being walked (the trait's `u` is passed where the
    /// original reads it, but the order-level methods carry none).
    pub(super) unit: usize,
    pub(super) cursor: usize,
    pub(super) pending: Option<Order>,
}

impl SimUngroup<'_> {
    /// The order at deque index `o`: its `OrderIndex` and group.
    pub(super) fn group_of(&self, o: usize) -> Option<(i32, GroupMove)> {
        let order = self.sim.units[self.unit].orders.get(o)?;
        match order.body {
            Body::Move(m) => m.group.map(|g| (i32::from(order.index()), g)),
            _ => None,
        }
    }
}

impl UngroupView for SimUngroup<'_> {
    type U = usize;
    type O = usize;
    type G = (i32, usize);
    type N = ();
    fn is_captain(&mut self, u: usize) -> bool {
        self.sim.units[u].captain
    }
    fn up(&mut self, u: usize) -> Option<usize> {
        match self.sim.units[u].o_up {
            Some(c) if c != u && self.sim.units[c].alive() => Some(c),
            _ => None,
        }
    }
    fn down(&mut self, u: usize) -> Option<usize> {
        match self.sim.units[u].o_down {
            Some(d) if self.sim.units[d].alive() => Some(d),
            _ => None,
        }
    }
    fn start_cursor(&mut self, u: usize) {
        self.unit = u;
        self.cursor = self.sim.units[u].orders.len().saturating_sub(1);
    }
    fn has_orders(&mut self, u: usize) -> bool {
        !self.sim.units[u].orders.is_empty()
    }
    fn cursor_order(&mut self, _u: usize) -> Option<usize> {
        Some(self.cursor)
    }
    fn carries(&mut self, o: usize, id: i64) -> bool {
        self.group_of(o).is_some_and(|(_, x)| x.id == id)
    }
    fn has_head(&mut self, u: usize) -> bool {
        !self.sim.units[u].orders.is_empty()
    }
    fn advance(&mut self, u: usize) {
        let n = self.sim.units[u].orders.len();
        self.cursor = if self.cursor == 0 {
            n - 1
        } else {
            self.cursor - 1
        };
    }
    fn at_head(&mut self, u: usize) -> bool {
        self.cursor + 1 == self.sim.units[u].orders.len()
    }
    fn group_record(&mut self, o: usize) -> (i32, usize) {
        let (k, g) = self.group_of(o).expect("the order just matched");
        (k, g.leader)
    }
    fn is_leader(&mut self, u: usize, g: (i32, usize)) -> bool {
        g.1 == u
    }
    fn kind(&mut self, g: (i32, usize)) -> i32 {
        g.0
    }
    fn get_obj(&mut self, _kind: i32) {}
    fn assign_attack(&mut self, _n: (), _g: (i32, usize)) {}
    fn assign_move(&mut self, _n: (), _g: (i32, usize)) {}
    fn finish(&mut self, _n: (), leader: bool) {
        let (u, i) = (self.unit, self.cursor);
        let order = &mut self.sim.units[u].orders[i];
        if !leader {
            order.flags &= !flag::PATHED;
        }
        if let Some(m) = order.move_mut() {
            m.group = None;
            m.has_waypoint = false;
            // `orig_x`/`orig_y` take the order's own `x`/`y` here
            // (`MoveOrder +0x44`/`+0x48`): `1/29`'s go `(39133, 21131)` →
            // `(38952, 21048)` across run100's ungroup. `finish_insert`
            // replays a copy to this point (item 738).
            m.orig = Some(m.dest);
        }
    }
    fn remove_current(&mut self, u: usize) {
        self.pending = self.sim.units[u].orders.remove(self.cursor);
    }
    fn give_obj(&mut self, _o: usize) {}
    fn add(&mut self, u: usize, _n: ()) {
        // `add@0046d5a0` prepends and the fix-up hands the head back; the
        // new order is the list's last link, the deque's front, and the
        // unit's head order changes (item 487, `docs/ORDERS.md` §20).
        let order = self.pending.take().expect("the order just removed");
        self.sim.units[u].orders.push_front(order);
        self.sim.new_head(u);
    }
    fn head_next(&mut self, _u: usize) {}
    fn kill_current_path(&mut self, u: usize) {
        self.sim.kill_current_path(u);
    }
}

/// What `Unit::do_move` reports back to `work`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Did {
    Nothing,
    Something,
}

/// `do_group_move`'s angle window (`5e815d`–`5e816d`): the wrapped
/// difference folded into half a turn and compared against `0x55555555` —
/// **a third of a turn**, 120°, not the quarter `reversing` uses.
pub(crate) fn within_third(a: Angle, b: Angle) -> bool {
    let d = (a.0 as u32).wrapping_sub(b.0 as u32);
    let d = if d > 0x8000_0000 { !d } else { d };
    d < 0x5555_5555
}

pub(super) const BEARINGS: [i32; 31] = [
    0, 1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8, 9, -9, 10, -10, 11, -11, 12, -12,
    13, -13, 14, -14, 15, -15,
];

/// Which of `find_nearby_spot`'s two collision halves a call site asks for
/// (`docs/ORDERS.md` §10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coll {
    /// `FILTER_NOT_ME` / `FILTER_CAN_COLLIDE` with a real `(o, who)` and no
    /// squad — every build, repair, gather, garrison, idle and stable site:
    /// `Objects::find_collision` and then `find_ordered_collision`.
    ///
    /// A **squad** placement wears the same name and takes the other path:
    /// [`Sim::find_nearby_spot_squad`] passes `uber = 1`, and the
    /// original's `bVar17` is written only in the arm `uber_unit != 0`
    /// jumps over (`61df3e`).
    Pairwise,
    /// Accept any passable candidate. The original's `nocoll != 0`, and —
    /// as a stated seam — its `FILTER_ALL` general path where a call site
    /// here has not yet asked for [`Coll::All`]. ~~`come_out`'s
    /// `block_radius == 0` arm, which no capture's trained type reaches~~:
    /// run1446's Attack Sub reached it, and it asks [`Coll::All`] since
    /// item 3780 (`docs/AI.md` §378.1).
    None,
    /// `FILTER_ALL` with the unit's own `(o, who)`: the **general** path
    /// (`61e37e` and `61e39c`, `docs/COLLISION.md` §5.2.1) —
    /// `find_unit_with_radius` over every player's units at `big_radius +
    /// r_coll`, `r_coll` the type's `block_radius` (`0x180` when that is
    /// 0), then its ordered twin, since `not_who >= 0`. `come_out`'s
    /// re-seat at a building point is a call site (`619360`,
    /// `docs/PRODUCTION.md` "The gather point"), and so is its ring's
    /// first sweep for a `block_radius == 0` type (`618436`, item 3780).
    ///
    /// **The seeker counts** (item 955, run338): `FILTER_ALL` skips
    /// `Search::valid_filter` (`659a1a`), the only reader of `(not_o,
    /// not_who)`, and the search argument is never read
    /// (`Search::valid_search(·, 0, …)` answers 1, `6599d4`), so every
    /// player's live, on-map unit is a candidate, the one asking too.
    /// run338's three Citizens, each re-seated with its first candidate
    /// the point it stands on, stand one candidate on (`docs/GOLDEN.md`
    /// §40). The ordered twin keeps the exemption: the seeker has no order
    /// when a re-seat asks.
    All,
}

/// Who is asking `find_nearby_spot` — `this` is always the unit **type**,
/// but every call site but one hands it a `(not_o, not_who)` pair as well,
/// and the pair is what the collision half exempts. `cast_transport` passes
/// `(-1, -1)` (`docs/TRANSPORT.md` §6), and that is [`Seeker::Type`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seeker {
    Unit(usize),
    Type(usize),
    /// A type asked **for** a unit: the block, domain and radius defaults
    /// are the type's, and the collision half is the unit's pairwise pair
    /// (`FILTER_NOT_ME` with the unit as `(not_o, not_who)`), which is how
    /// `cast_create_decoy` places a copy beside its General.
    TypeFor(usize, usize),
}

/// What [`spot_ring`] reads of the world and the units: the tile masks, the
/// footprint, and the collision half. [`SimSpot`] answers from a [`Sim`];
/// the emulator sweep answers from the original's own records.
pub trait SpotView {
    fn tile_in_bounds(&self, t: Pos) -> bool;
    fn tile_mask(&self, t: Pos) -> u16;
    fn covers_tile(&self, building: usize, t: Pos) -> bool;
    /// `is(AIRCRAFTCARRIER, 1)` of the seeker's type, asked only of a sea
    /// type whose `attack` is zero.
    fn is_carrier(&self, who: Seeker) -> bool;
    fn owner(&self, u: usize) -> Player;
    fn find_collision(&self, u: usize, at: Pos) -> bool;
    fn find_ordered_collision(&self, u: usize, at: Pos) -> bool;
    fn find_unit_with_radius(&self, r: i32, at: Pos, exempt: Option<usize>) -> bool;
    fn find_unit_ordered_with_radius(
        &self,
        r: i32,
        at: Pos,
        who: Player,
        exempt: Option<usize>,
    ) -> bool;
}

/// [`SpotView`] over a [`Sim`].
pub(super) struct SimSpot<'a>(&'a Sim);

impl SpotView for SimSpot<'_> {
    fn tile_in_bounds(&self, t: Pos) -> bool {
        self.0.world.tile_in_bounds(t)
    }
    fn tile_mask(&self, t: Pos) -> u16 {
        self.0.world.tile_mask(t)
    }
    fn covers_tile(&self, building: usize, t: Pos) -> bool {
        self.0.covers_tile(building, t)
    }
    fn is_carrier(&self, who: Seeker) -> bool {
        let ty = match who {
            Seeker::Unit(u) => self.0.units[u].ty,
            Seeker::Type(t) | Seeker::TypeFor(t, _) => Some(t),
        };
        ty.is_some_and(|t| self.0.is_aircraft_carrier(t))
    }
    fn owner(&self, u: usize) -> Player {
        self.0.units[u].owner
    }
    fn find_collision(&self, u: usize, at: Pos) -> bool {
        self.0.find_collision(u, at)
    }
    fn find_ordered_collision(&self, u: usize, at: Pos) -> bool {
        self.0.find_ordered_collision(u, at)
    }
    fn find_unit_with_radius(&self, r: i32, at: Pos, exempt: Option<usize>) -> bool {
        self.0.find_unit_with_radius(r, at, exempt)
    }
    fn find_unit_ordered_with_radius(
        &self,
        r: i32,
        at: Pos,
        who: Player,
        exempt: Option<usize>,
    ) -> bool {
        self.0.find_unit_ordered_with_radius(r, at, who, exempt)
    }
}

/// The ring sweep of `UnitType::find_nearby_spot@0061de70`, over a
/// [`SpotView`] (the body of [`Sim::spot_sweep`]). `p` is the asking
/// type's record, `transport` its `unit_flags & 0x10`, `range` is
/// `(min, max, step)`, `footprint` the building to keep off and whether a
/// farm is exempt.
#[allow(clippy::too_many_arguments)]
pub fn spot_ring(
    view: &impl SpotView,
    who: Seeker,
    p: &combat::Profile,
    transport: bool,
    centre: Pos,
    range: (i32, i32, i32),
    angle: Angle,
    footprint: Option<(usize, bool)>,
    coll: Coll,
    uber: bool,
) -> Option<Pos> {
    let (min, max, step) = range;
    let mut max = max;
    if (min > 0 && max == 0) || max < 0 {
        // `find_nearby_spot@0061de70:45`: the default is
        // `min + 4 × big_radius`, overridden to `min + 0x240` only when
        // `big_radius == 0` **and** `!(unit_flags & 0x10)` — the
        // transport-capable flag, which this crate does not model, so the
        // second condition is assumed false. It can only differ for a
        // transport-capable type whose `big_radius` is 0, and no shipped
        // row is both. Stated in §14. The squad branch (`uber_unit !=
        // 0`, the decompile's lines 52–54) is `min + 0xc0 + 4 ×
        // (((uber − 1) × guy_spacing) / 2 + big_radius)`: `come_out`'s
        // routing asks it for a squad's target with `max` −1 (item
        // 955, run338's `0/13` on 858); `go_to` passes its own `max`.
        // (`docs/audit/2026-08-21-orders.md` R7 N3.)
        max = if uber {
            min + 0xc0 + 4 * (((p.uber_size - 1) * p.guy_spacing) / 2 + p.big_radius)
        } else if p.big_radius == 0 && !transport {
            min + 0x240
        } else {
            min + 4 * p.big_radius
        };
    }
    if max < min {
        max = min;
    }
    let step = if step <= 0 {
        ((max - min) / 8).max(1)
    } else {
        step
    };
    // **The squad's own collision radius** (`61df42`–`61df58`): with
    // `uber_unit != 0` the block the general query is asked with is
    // `block_radius + ((uber_size − 1) × guy_spacing) / 2 + 0x30` —
    // half the formation's own span plus a quarter-tile, so a squad
    // asks for the room it will actually stand in.
    let uber_r = p.block_radius + ((p.uber_size - 1) * p.guy_spacing) / 2 + 0x30;
    let air = matches!(p.domain, crate::territory::attrition::Domain::Air);
    // **A warship keeps off the `0x2400` tiles** (`61df8b`–`61dfaf`,
    // the test at `61e3a1`): a sea type whose `attack` (`ObjectTypeData
    // +0x1e8`, the type record's name) is non-zero, or that strictly
    // `is(AIRCRAFTCARRIER)` (`0x15f`, `TypeIndex`), refuses an ocean
    // tile carrying either bit. East Indies' Trireme `1/32` is born
    // due east of Dock `1/2010` because the channel south of it is
    // `0x0420` (`docs/ORDERS.md` §25).
    let warship = matches!(p.domain, crate::territory::attrition::Domain::Sea)
        && (p.attack != 0 || { view.is_carrier(who) });
    let mut r = min;
    loop {
        let ks: &[i32] = if r == 0 { &[0] } else { &BEARINGS };
        for &k in ks {
            let cand = if r == 0 {
                centre
            } else {
                let half = if !(-7..=7).contains(&k) {
                    0x800_0000
                } else {
                    0
                };
                let a = Angle(
                    angle
                        .0
                        .wrapping_add(k.wrapping_mul(0x1000_0000))
                        .wrapping_add(half),
                );
                Pos::new(
                    centre.x + movement::sin_component(a, r),
                    centre.y - movement::cos_component(a, r),
                )
            };
            let c = Pos::new(
                cand.x.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
                cand.y.div_euclid(SNAP) * SNAP + SNAP_CENTRE,
            );
            if !view.tile_in_bounds(c.tile()) {
                continue;
            }
            if !air && view.tile_mask(c.tile()) & crate::world::tile::BLOCKED != 0 {
                continue;
            }
            if let Some((b, farm_ok)) = footprint
                && !farm_ok
                && view.covers_tile(b, c.tile())
            {
                continue;
            }
            // **The terrain class, by domain** (`0061e3a1`..`0061e39e`):
            // an air type takes anything; a **sea** type needs the tile's
            // surface field to be `SURFACE_OCEAN` exactly; a **land**
            // type needs it not to be. This crate asked
            // `World::accepts` here — a cell-bounds test the loop's own
            // `tile_in_bounds` has already made — so a citizen was free
            // to stand in the sea, which is what put the AI's dock
            // builder on the wrong point of the swarm ring
            // (`docs/ORDERS.md` §10, "The ocean the ring could stand
            // in").
            //
            // And the warship clause: a sea type that can fight also
            // needs `!(mask & 0x2400)` — `BAD_PATH` and the unnamed
            // `0x400` (`warship` above).
            if !air {
                let mask = view.tile_mask(c.tile());
                let ocean = mask & tile::SURFACE == tile::SURFACE_OCEAN;
                if ocean != matches!(p.domain, crate::territory::attrition::Domain::Sea) {
                    continue;
                }
                if warship && mask & WARSHIP_REFUSES != 0 {
                    continue;
                }
            }
            // The collision half, last of all: `Objects::find_collision`
            // then `Objects::find_ordered_collision`, both against
            // `(u)` as "me" (`docs/COLLISION.md` §5.2). A spot another
            // unit is standing on — or has already been sent to — is
            // taken.
            let hit = coll != Coll::None
                && match (uber, who) {
                    // **A squad placement never takes the pairwise
                    // pair.** `bVar17` — the flag that selects
                    // `find_collision` plus `find_ordered_collision` —
                    // is written only in the arm `uber_unit != 0`
                    // jumps over (`61df3e`, the write at
                    // `61df5f`–`61df7c`), so a squad goes down the general
                    // path whatever its filter says: one radius query
                    // against every player's positions, and, because
                    // `not_who` is a real player here, the *ordered*
                    // one beside it.
                    (true, _) => {
                        let exempt = match who {
                            Seeker::Unit(u) | Seeker::TypeFor(_, u) => Some(u),
                            Seeker::Type(_) => None,
                        };
                        view.find_unit_with_radius(uber_r, c, exempt)
                            || match who {
                                Seeker::Unit(u) => view.find_unit_ordered_with_radius(
                                    uber_r,
                                    c,
                                    view.owner(u),
                                    exempt,
                                ),
                                Seeker::Type(_) | Seeker::TypeFor(..) => false,
                            }
                    }
                    (false, Seeker::Unit(u)) if coll == Coll::All => {
                        let r = if p.block_radius == 0 {
                            0x180
                        } else {
                            p.block_radius
                        };
                        // The seeker counts (item 955): no exemption
                        // on the position query under `FILTER_ALL`.
                        view.find_unit_with_radius(r, c, None)
                            || view.find_unit_ordered_with_radius(r, c, view.owner(u), Some(u))
                    }
                    (false, Seeker::Unit(u) | Seeker::TypeFor(_, u)) => {
                        view.find_collision(u, c) || view.find_ordered_collision(u, c)
                    }
                    (false, Seeker::Type(_)) => view.find_unit_with_radius(p.block_radius, c, None),
                };
            if hit {
                continue;
            }
            return Some(c);
        }
        if max <= r {
            return None;
        }
        r = (r + step).min(max);
    }
}

/// A waypoint `do_move`'s tile arm unwinds before `find_tpath`: neither
/// final nor `0x20`, with a tolerance of **1 to `0x60`**, unsigned —
/// `lea eax,[ecx-1]; cmp eax,0x5f; ja` at `0x5f8ad2`. A tolerance-0
/// waypoint is an exact point and is kept: a group move's formation
/// waypoint is one, and popping it re-aims the tile search at the final
/// goal (`docs/ORDERS.md` §4.4, item 669).
pub(super) fn is_loose(t: &PathData) -> bool {
    t.flags & (path_flag::FINAL | 0x20) == 0 && (t.tolerance as u32).wrapping_sub(1) <= 0x5f
}

impl Sim {
    // ------------------------------------------------------------------
    // The move order — `Unit::do_move`
    // ------------------------------------------------------------------

    /// `Unit::do_move` (§4.4) on open ground: the planning half, the
    /// waypoint take with both arrival tests, the straight-line check, the
    /// step.
    pub(crate) fn do_move(&mut self, u: usize, frame: i64) -> Did {
        let Some(Order {
            body: Body::Move(mut mo),
            flags,
        }) = self.current_order(u).copied()
        else {
            return Did::Nothing;
        };
        let mut flags = flags;

        // **The fire on the move** (§4.4 step 1, `005f7b30:60-86`): a
        // `v` type looks for, and shoots at, a target beside its walk
        // before the step does anything else ([`crate::combat::cavarch`]).
        self.cavarch_head(u, frame);

        // **A suspended search** (§4.4 step 2, item 301). `do_move`'s
        // first block after the cavalry-archer fire, and **no step happens
        // while one is pending**: every arm below returns.
        //
        // **Its Great Lakes cost is paid** (item 304,
        // `docs/PATHFINDER.md` §18.5). Wired, this block used to cost
        // Great Lakes' long word 7679 → 6862, and the cause was upstream
        // of every line of it: run76's `1/28` degrades out of formation on
        // 6860 on the original's own frame, but this crate's
        // [`Sim::kill_current_path`] popped the path segment without the
        // `clear_partial_path` under it, so the follower kept a suspended
        // search that no order was left to serve and stood in this block
        // for the rest of the capture. With that one call in place the
        // wired measurement is East Indies 7812 → **8193** and Great Lakes
        // **7679**, unmoved.
        //
        // The clock is `frame − collide_frame` — `UnitData +0x48`, which is
        // `collide_frame` by the type record, not a second stamp — so the
        // "has the blocker gone" probe fires on the 5th, 7th, 9th … frame
        // after the collision that suspended the search, and the `repaths`
        // tick on every fourth `o + frame`.
        //
        // **The `ATTACK` retarget** (`005f7b30`, the action's `get_type ==
        // 10` arm; item 3336, `docs/AI.md` §272): a unit whose action is an
        // attack that is not `mandatory` (the action's vslot `+0xcc`, byte
        // `+0x1c`, as `find_melee_target` reads it) gives the walk up every
        // fourth frame after the collision, while its player's `repaths` is
        // non-zero: `find_new_target(this, NULL, 1)` — the move dies and a
        // target is sought standing ground — and `collide_frame` takes the
        // frame, so `collide` is not counted. Great Sahara's Egyptian
        // Fusilier `1/162` on tick 20029 (run967g), four frames after it met
        // `1/119`.
        if self.units[u].search.is_some() {
            let elapsed = frame - self.units[u].collide_frame;
            let attacking = self
                .action_of(u)
                .is_some_and(|i| self.units[u].orders[i].index() == index::ATTACK);
            if attacking {
                let who = self.units[u].owner as usize;
                if elapsed > 0
                    && elapsed % 4 == 0
                    && self.movement.repaths[who] != 0
                    && !self.units[u].combat.mandatory
                {
                    self.find_new_target(u, true);
                    self.units[u].collide_frame = frame;
                    return Did::Nothing;
                }
            }
            // **The `GATHER` park** (`docs/COLLISION.md` §15, item 698):
            // a gatherer whose search is suspended, on the frame
            // `(elapsed + 2) & 7 == 0`, within `vector_dist < 0x120` of the
            // move's own `x/y`, gives the walk up where it stands.
            // `avoid` takes the move's point (`005f7ce6`, not the unit's),
            // the gather order forgets its tile (`tx`/`ty`/`wait` −1,
            // `goto_build` 1) and the move dies, so the next frame's
            // `do_non_flat_gather` picks a tile afresh. `collide` is not
            // counted: the arm returns above the increment. Run178's `1/43`
            // on 14649, six frames after it met the standing `1/18`. The
            // `hold_doobers` cleanup beside it is presentation.
            let gathering = self
                .action_of(u)
                .is_some_and(|i| self.units[u].orders[i].index() == index::GATHER);
            if gathering
                && elapsed > 0
                && (elapsed + 2) % 8 == 0
                && vector_dist(
                    mo.dest.x - self.units[u].pos.x,
                    mo.dest.y - self.units[u].pos.y,
                ) < 0x120
            {
                self.units[u].avoid = Some(mo.dest);
                if let Some(mut g) = self.units[u].orders.iter().find_map(|o| match o.body {
                    Body::Gather(g) => Some(g),
                    _ => None,
                }) {
                    g.tile = None;
                    g.wait = -1;
                    g.goto_build = true;
                    self.store_gather(u, g);
                }
                self.kill_current_order(u);
                return Did::Nothing;
            }
            // The probe is the **quick** form (`docs/COLLISION.md` §18,
            // item 857): an occupied cell is the blocker still there, with
            // no corner rule and no write to `collide_o`. East Indies'
            // `1/71` on 19498 met `1/75`'s corner at `coll`, which the full
            // form let pass and the original counts.
            let coll = mo.coll.unwrap_or(self.units[u].pos);
            if elapsed > 3 && (elapsed - 1) % 2 == 0 && !self.blocker_still_there(u, coll) {
                // The blocker has gone: drop the search and re-plan next
                // frame off the stack as it stands.
                mo.has_waypoint = false;
                self.clear_partial_path(u);
                flags |= flag::PATHED;
                self.units[u].collide = 0;
                self.store_move(u, mo, flags);
                return Did::Nothing;
            }
            if (i64::from(self.units[u].index) + frame) % 4 == 0 {
                let who = self.units[u].owner as usize;
                self.movement.repaths[who] += 1;
            }
            self.units[u].collide += 1;
            mo.has_waypoint = false;
            self.store_move(u, mo, flags);
            self.find_upath_restore(u);
            if self.units[u].search.is_some() {
                return Did::Nothing;
            }
            if frame - 5 <= self.units[u].collide_frame {
                return Did::Nothing;
            }
            self.units[u].collide = 0;
            return Did::Nothing;
        }

        // `timer`: a self-destruct.
        if mo.timer > 0 {
            if mo.timer == 1 {
                self.kill_current_order(u);
                self.work(u, frame);
                return Did::Nothing;
            }
            mo.timer -= 1;
            self.store_move(u, mo, flags);
        }

        // The action under the move: an ATTACK whose target is in range has
        // no further use for the chase; one whose target is gone re-paths.
        //
        // **And the whole block is a RANGED attacker's** (item 405). The
        // original's test at `do_move@005f7b30:212` is
        // `if (*(int *)(*(int *)&this->field_0x18 + 0x1fc) != 0)` —
        // `SubObjectData +0x18 ptype`, `ObjectTypeData +0x1fc max_range`
        // by the type record — and **everything** the ATTACK action does
        // under a move sits inside it: the unit-target kill, the building
        // -target kill, and the melee retarget beneath them. A melee type
        // never abandons its chase because the target came into reach; it
        // walks the leg it was given and `do_attack` takes over when the
        // move ends. `docs/ORDERS.md` §4.4 has said "a ranged type" since
        // the second reading and this crate asked every type.
        //
        // It is the golden record's frames 619 and 620. `1/8` stands at
        // `(1332, 8121)` with `attack_dist` **246** to `0/7` — exactly
        // `0xf6`, the HOPLITES reach, in range by a single unit — and
        // this crate killed the move 171 short of the `(1176, 8088)` the
        // dump walks it to; `1/7` did the same a frame later at 240. The
        // original's own record says the collision half was never it:
        // `tolerance` is 0 and `collide_o`/`collide_frame` are clear on
        // every block 616..629, on both sides (`docs/RUNS.md` run110).
        //
        // SEAM: two further conjuncts of the unit-target kill are not
        // modelled, and both only make it **rarer** — the flank clause
        // (`angle` window, `flanking`, `max_range` again) and
        // `Objects::find_collision(my own spot, o, who, 1) == 0`.
        // ~~The `vector_dist < 0x481` guard on the re-path arm is the
        // third.~~ **Item 487 landed it**, below.
        if let Some(a) = self.action_of(u)
            && let Body::Attack(_) = self.units[u].orders[a].body
        {
            let me = Obj::Unit(u);
            match self.units[u].combat.target {
                // **The gate is the slot, not `valid_target`** (item 1074,
                // `docs/COMBAT.md` §77). `do_move@005f7b30`, `5f7ece`–`5f7f11`,
                // asks of the action's target only that `o` and `who` be
                // non-negative, that `objects[whom][ox]`'s `flags & 1` be
                // up, and that its `uid` match the order's — no
                // diplomacy, and no `is_seen`. A living target that has
                // walked out of its chaser's sight keeps the chase: the
                // planner walks on to the goal the order holds. This crate
                // never recycles a slot, so the `uid` test is `alive`.
                Some(t) if self.obj_alive(t) => {
                    // **The gate is a block, not a conjunct** (item 481).
                    // `do_move@005f7b30:207` opens
                    // `if (ptype->max_range != 0) { … }` and the brace
                    // closes past the captain retarget below, so *both*
                    // arms are a ranged attacker's. Writing it as a
                    // conjunct of the kill alone let a melee captain
                    // retarget: run112's hoplite captain `1/6` carries
                    // `near_o 10` from block 621 to the end of the
                    // window and this crate switched its whole squad off
                    // `0/11` at 671, where the dump has all three
                    // hoplites on `0/11` for every block of the capture.
                    // `docs/COMBAT.md` §38.
                    // **The building arm** (item 1002, `docs/COMBAT.md`
                    // §65): the target's vslot `+0x1c` (1 for a building
                    // or a wall) splits the block at `5f7f27`. A building
                    // is asked through the four-argument
                    // `is_in_range@00648d70`, which passes the sixth
                    // argument **0** — no `0x90` margin, `mandatory` or
                    // not — after an AI-driven siege type
                    // (`unit_masks & 0x40000`, `is_siege` `+0x10c`) is
                    // passed over at `5f7f34`–`5f7f4a`. Either refusal
                    // goes to the planner at `5f82c1`: the flank clause
                    // and the captain retarget below are the unit
                    // target's alone. run347's `1/26`, a ranged soldier
                    // on the human's city, stops at (5079, 31628) at
                    // `attack_dist` 1128 against a reach of 1158.
                    //
                    // **And no other unit in my block** (item 1177,
                    // `docs/COMBAT.md` §65.8): `Objects::find_collision(my
                    // own spot, o, who, 1) == 0` at `5f7f9d`, the fifth
                    // argument set, so the chain walk rather than the
                    // land grid. run426's Longbowman `1/29` stands in
                    // reach of the human's city at 2284 with its mate
                    // `1/30` two unit cells off, and walks one more step.
                    //
                    // SEAM: `valid_target@00648ba0` (`5f7f71`), between
                    // the two, is not modelled; it refuses only a
                    // capturable building.
                    if self.profile(me).max_range != 0
                        && let crate::combat::Obj::Building(_) = t
                    {
                        let siege = self.units[u].ty.is_some_and(|ty| {
                            self.tables.unit_types[ty]
                                .cols
                                .flag(crate::ai::ai_load::uflags::FLAG_SIEGE)
                        });
                        if !(siege && self.ai_driven(self.units[u].owner))
                            && self.is_in_range(me, t)
                            && !self.find_collision_forced(u, self.units[u].pos)
                        {
                            self.kill_current_order(u);
                            return Did::Something;
                        }
                    } else if self.profile(me).max_range != 0 {
                        // **`is_in_range`'s sixth argument**, and this is
                        // the executable's only caller that sets it: a
                        // non-`mandatory` chase is dropped `0x90` inside
                        // the attacker's reach rather than at its edge
                        // (`docs/COMBAT.md` §35.3).
                        //
                        // **And the target must not be running from the
                        // chaser's own heading** (item 1061, `docs/COMBAT.md`
                        // §71): `5f7fbe`–`5f7ff6`, ahead of the range test,
                        // `e = target +0x50 − my +0x50 − 0x80000000`; `e`
                        // at or past `0x2aaaaaaa` (`jb`), `flanking@0092cfe0`
                        // on `ecx` non-zero and the target's `vt+0xd8`,
                        // `UnitData::is_moving`, non-zero jump to `5f803f`,
                        // the captain's retarget, past the kill. It is
                        // `check_target_path`'s triple against the
                        // attacker's heading rather than the bearing.
                        //
                        // **And no other unit in my block**, as the
                        // building arm (item 3342, `docs/COMBAT.md`
                        // §65.8): `5f7f11`–`5f7f2c` call
                        // `Objects::find_collision(my own spot, o, who, 1)`
                        // after the range test, and a hit goes to the
                        // planner. run976's Slinger `1/22` stands in reach
                        // of `0/5` on 6561 with a mate in its block, and
                        // walks on.
                        let margin = !self.units[u].combat.mandatory;
                        let at = self.units[u].pos;
                        if !self.chase_target_flees(u, t)
                            && self.is_in_range_at_margin(me, at, t, margin)
                            && !self.find_collision_forced(u, at)
                        {
                            self.kill_current_order(u);
                            return Did::Something;
                        }
                        // **The captain's retarget**, `005f803f`-`005f8216`
                        // (`docs/COMBAT.md` §37.2). It runs only when the
                        // kill did *not* fire — so a ranged captain
                        // walking to a target it cannot yet reach asks,
                        // every frame, whether the incumbent its last
                        // search left in [`crate::Unit::near`] is one it
                        // can.
                        //
                        // The two range tests are the same function with
                        // a different sixth argument, and that is the
                        // whole of why both can be true on one frame: the
                        // kill uses the reach less `0x90`, this uses the
                        // plain reach.
                        //
                        // `mandatory` gates it — an ordered attack is
                        // never retargeted under the player — and so does
                        // `is_captain`: a squad member's target comes
                        // down the chain from here and it decides nothing
                        // itself.
                        //
                        // **And on every sixteenth `o + frame` it searches
                        // first** (`005f803f`–`005f80a6`; item 3355,
                        // `docs/AI.md` §279): `find_melee_target(this, −1,
                        // &who, 0, 0, word)` with no add, the word 1
                        // (`UNITS`) — the target's vslot `+0x1c` is 0 on
                        // this arm — or 2 for a siege type (`+0x10c`). The
                        // search writes `near` and bumps its winner's
                        // `targeted` whatever follows, and a find in range
                        // stands in for `near` below. The Persian word's
                        // 6251: the Immortal captains chasing the fleeing
                        // citizen `0/3` leave it `targeted` 3 and `0/4` 1
                        // (run984c's packet, `Object +0x3d`), so `1/10`'s
                        // re-search takes `0/4`; without the search both
                        // read 0 and it kept `0/3`.
                        //
                        // **And the candidate may be the target itself**
                        // (item 3525, `docs/AI.md` §303): nothing from
                        // `5f803f` to `5f8216` compares it with the
                        // action's own `o`/`who`, and `change_target` with
                        // the same pair rewrites it and still pops every
                        // order above the action. A captain whose chase the
                        // flank test kept from the kill drops its move the
                        // frame its `near` is in plain reach: run1153c's
                        // Destroyer `1/66` on 3815, `near` the fleeing
                        // `0/5`, stops at (2764, 8930) and fires on 3816.
                        // The Russian pair's Cruiser `1/25` is the same
                        // shape on 5815, its search finding `0/0` itself
                        // (item 3527, §304).
                        let searched = if !self.units[u].combat.mandatory
                            && self.units[u].captain
                            && self.units[u].phase(frame) % 16 == 0
                        {
                            let siege = self.units[u].ty.is_some_and(|ty| {
                                self.tables.unit_types[ty]
                                    .cols
                                    .flag(crate::ai::ai_load::uflags::FLAG_SIEGE)
                            });
                            let word = if siege {
                                crate::combat::fight::search::BUILDINGS
                            } else {
                                crate::combat::fight::search::UNITS
                            };
                            self.find_melee_target_with(u, -1, word)
                        } else {
                            None
                        };
                        if !self.units[u].combat.mandatory
                            && self.units[u].captain
                            && let Some(near) = self.units[u].near
                            && let c = searched
                                .filter(|&f| self.is_in_range(me, f))
                                .unwrap_or(near)
                            && self.active(c)
                            && self.is_in_range(me, c)
                            && !self.poor_target(me, c)
                            && matches!(c, Obj::Unit(_))
                            && self.retarget_admits(u, c)
                        {
                            // `005f820a`, and it is the fingerprint the
                            // item was found by: `in_range` is written
                            // **here**, on the deciding unit alone and a
                            // frame before `Unit::fight` writes
                            // `ever_in_range`. run112's `0/9` prints
                            // `in_range 1` with `ever 0` on block 645 and
                            // nothing else in the executable can produce
                            // that pair.
                            if let Body::Attack(x) = &mut self.units[u].orders[a].body {
                                x.in_range = true;
                            }
                            self.change_target(u, t, c);
                            return Did::Something;
                        }
                    }
                }
                // **The dead target's arm, and it is a *near* one**
                // (item 487, `docs/ORDERS.md` §20). The original reaches
                // here when the action's target is gone — a negative
                // `o`/`who`, an object whose `flags & 1` is down, or a
                // slot whose `uid` no longer matches — and it does **not**
                // re-path unconditionally. `do_move@005f7b30`,
                // `005f8221`-`005f825d`:
                //
                // ```
                // 5f8221: eax = this->y ^ 0x63637; eax -= [edi+0x8]
                // 5f822f: eax = this->x ^ 0x63637; eax -= [edi+0x4]
                // 5f8247: call vector_dist            ; (|dx|, |dy|)
                // 5f824c: cmp  eax, 0x480
                // 5f8251: jg   0x5f82c1                ; → the planner
                // 5f8256: test $0x10, 0x2b4(ptype)     ; a sea transport
                // 5f825d: jne  0x5f82c1                ; → the planner
                // 5f8261: call Unit::repath
                // ```
                //
                // `edi` is the `MoveOrder` the function opened with
                // (`5f7b4b`'s vfunc `+0x40`, stashed at `[ebp-0x18]`), and
                // `+0x4`/`+0x8` are `x`/`y` by the type record — the
                // order's own destination. So the octagonal distance from
                // the unit to **where it was walking** decides it: within
                // `0x480` the walk is pointless and the transit legs are
                // popped; beyond it the unit keeps the order and falls
                // through to the planner, dead target and all.
                //
                // run100's `1/29` is the case and it is 28,000 units out:
                // an AI raider whose farm died on sim-frame 10230, whose
                // reload keeps `do_attack` from ever reaching `fight`'s
                // own validity kill, and whose move home the original
                // plans on 10240 — 43 nodes — while this crate popped it
                // and left the unit standing in `1/27`'s way.
                _ => {
                    let d = crate::world::vector_dist(
                        (self.units[u].pos.x - mo.dest.x).abs(),
                        (self.units[u].pos.y - mo.dest.y).abs(),
                    );
                    let transport = self.units[u].ty.is_some_and(|t| {
                        self.tables.unit_types[t].cols.unit_flags
                            & crate::ai::ai_load::uflags::FLAG_TRANSPORT
                            != 0
                    });
                    if d <= 0x480 && !transport {
                        self.repath(u);
                        return Did::Nothing;
                    }
                }
            }
        }

        // **The retry delay a failed unit-grid search bought**
        // (`do_move@005f7b30:348`-`355`, `docs/PATHFINDER.md` §21). It sits
        // between the action tests and the planner, and while it stands the
        // move does nothing at all — no plan, no waypoint, no step. The
        // frame it runs out, `attempts` gains 3, which is what stops a unit
        // wedged against a neighbour buying delay after delay for ever.
        if mo.retry != 0 {
            mo.retry -= 1;
            if mo.retry == 0 {
                mo.attempts += 3;
            }
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        // **The modern-infantry pack** (`5f82df`-`5f8390`, `docs/ORDERS.md`
        // §3.2's `retry` row, `docs/GROUPS.md` §34): once every 128 frames,
        // on the unit's own phase `(o * 0x11 + frame) & 0x7f == 0`, a
        // modern-infantry unit not `in_danger` (`unit_masks & 4`, `5f8315`)
        // squares its heading to guy 0's, is re-seated on guy 0's point
        // (`set_new_location(x, y, 1, 1)`), plays `CHAR_PACK`, and waits
        // out the animation: `retry` is guy 0's `end_time` read **after**
        // the `set_anim` (`5f837b`), and `attempts` −3. Every later frame
        // is the retry branch above; the frame it runs out, `attempts`
        // goes back to 0.
        //
        // ~~SEAM: `ObjectData::has_general(0x8000, -1) < 0` (`5f832f`) is
        // taken as true~~ — **a marching General in range keeps the squad
        // walking** (`5f8326`–`5f8336`, `jns` past the pack):
        // [`Sim::near_marching_hero`]. The coverage pair's TOW `1/66`,
        // sixteen tiles from the General `1/150` on its march, walks on
        // through its phase on 2974 (`docs/AI.md` §172).
        if (i64::from(self.units[u].index) * 0x11 + self.frame) & 0x7f == 0
            && !self.units[u].in_danger
            && self.is_modern_infantry(u)
            && !self.near_marching_hero(u)
        {
            let facing = self.units[u].movement.facing;
            self.unit_set_angle(u, facing);
            let at = self.units[u].movement.body.pos;
            self.set_new_location(u, at, true);
            self.set_anim(u, crate::units::anim::ANIM_PACK, false, true);
            // A figure with no art has no length (`anim::UNKNOWN`): it
            // does not wait, where the original always has one.
            mo.retry = self.units[u]
                .guys
                .first()
                .filter(|g| g.end_time != crate::units::anim::UNKNOWN)
                .map_or(0, |g| g.end_time as i32);
            mo.attempts = -3;
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        // And the decay, on every frame that reaches the planner
        // (`005f7b30:369`).
        if mo.attempts != 0 {
            mo.attempts -= 1;
            self.store_move(u, mo, flags);
        }
        // Planning: the first time, or with an empty stack. Every fresh
        // move calls `find_wpath` exactly once; a refusal re-pushes the
        // goal and falls back to the tile grid (§4.4).
        if flags & flag::PATHED == 0 || self.units[u].path.is_empty() {
            let goal = PathData {
                to: mo.dest,
                tolerance: 0,
                flags: path_flag::FINAL,
            };
            self.units[u].path.push(goal);
            let r = self.find_wpath(u);
            if r < 1 {
                if self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
                let r2 = self.find_tpath(u);
                if r2 < 1 && self.units[u].path.is_empty() {
                    self.units[u].path.push(goal);
                }
            }
            flags |= flag::PATHED;
            self.store_move(u, mo, flags);
            if self.units[u].path.len() > 10 {
                return Did::Something;
            }
        }

        // Taking a waypoint.
        if !mo.has_waypoint {
            let Some(top) = self.units[u].path.last().copied() else {
                // Nothing to walk: the goal was already here.
                self.kill_current_order(u);
                return Did::Nothing;
            };
            mo.has_waypoint = true;
            mo.waypoint = top.to;
            self.units[u].line_ok = false;
            self.units[u].tolerance = top.tolerance;
            self.store_move(u, mo, flags);
            // `do_move@005f7b30:437`: under a `TRADE_ROUTE` action, a
            // waypoint the road was laid on asks whether the road is still
            // there (`docs/CARAVAN.md` §10).
            if top.flags & path_flag::ROAD_LAID != 0
                && self
                    .action_of(u)
                    .is_some_and(|i| self.units[u].orders[i].index() == index::TRADE_ROUTE)
            {
                self.caravan_road_step(u, top.to);
            }
            // The strip above wrote the stack; the region check below pops
            // and pushes **the stack's** top, so it carries the stripped
            // flags (`do_move@005f7b30:496`, `Stack<PathData>::pop` and
            // `pPVar8->flags | 4`): run993d's block 14943, `1/79`'s
            // `path[19].flags` 4 in theirs, ours 36 (item 3370).
            let top = self.units[u].path.last().copied().unwrap_or(top);
            // **The region check** (`5f8544`–`5f86f4`, §4.4; item 1143): a
            // **final** waypoint, or any waypoint under a `TRADE_ROUTE`
            // action, whose tile's `get_tregion@006b52e0` is not the unit's
            // own is popped and pushed back with `flags | 4` and tolerance
            // 0. The unit's own tolerance keeps the old entry's (`5f8541`
            // wrote it first). The bit is `invalid_loc`'s transport relax,
            // read off the path's top: a barge whose goal is on land keeps
            // it rather than `find_path` pulling it back to the water, and
            // walks onto the shore, where `set_new_location` disembarks it
            // (`docs/TRANSPORT.md` §6.4). East Indies' barge `1/42` stopped
            // a tile short of the shore on 6321 without it.
            let trade = self
                .action_of(u)
                .is_some_and(|i| self.units[u].orders[i].index() == index::TRADE_ROUTE);
            let top = if (top.flags & path_flag::FINAL != 0 || trade)
                && self.world.tregion_alt(top.to.tile())
                    != self.world.tregion_alt(self.units[u].pos.tile())
            {
                let flagged = PathData {
                    tolerance: 0,
                    flags: top.flags | path_flag::BOARDS_TRANSPORT,
                    ..top
                };
                self.units[u].path.pop();
                self.units[u].path.push(flagged);
                flagged
            } else {
                top
            };

            // **The waypoint's own collision test** (§4.4), the one call of
            // `detect_unit_collision` that is not `move_step`'s or
            // `resolve_unit_collision`'s. It runs once per leg — only on
            // the frame the waypoint is taken — and asks whether somebody
            // is already standing where this leg ends.
            //
            // A **final** waypoint under a `TRADE_ROUTE`, `GATHER`,
            // `ATTACK` or `BUILD_AT` action is then not worth walking at
            // all: the walk dies here and the action picks somewhere else
            // next frame (`5f8714`–`5f873f`: `top.flags & 1`, then
            // `action_type` against `0xf`, `7`, `0xa`, `6`, each to the
            // `kill_current_order(0)` at `5f8851`). That is how a farmer
            // whose re-picked cell a sibling already works stays where it
            // is (`docs/COLLISION.md` §8), and how a caravan whose last
            // node a builder stands on arrives where it is — its
            // `do_trade` runs the next frame (Great Sahara at Toughest's
            // 10391, item 1371).
            //
            // Otherwise a **parked** collider — one whose own current order
            // is not a move, so it is not going to get out of the way —
            // widens the tolerance to three of its `big_radius`
            // (`ObjectType +0x244`): give up short of it rather than walk
            // into it.
            //
            // **And a ship never asks.** The call is `(x, y, 0, 0, 0, 0, 0)`
            // on the listing (`5f86f9`–`5f8707`); with `boats` zero the
            // second arm returns 0 at `61782b`, before any scan and past the
            // exit bookkeeping (`docs/COLLISION.md` §13). East Indies' Bark
            // `1/34` took a soft hit here from Trireme `1/32` beside the
            // navy's muster and walked its first step at half speed.
            let hit = if self.takes_boat_arm(u) {
                None
            } else {
                self.detect_unit_collision(u, mo.waypoint)
            };
            if let Some(other) = hit {
                let action = self.action_of(u).map(|a| self.units[u].orders[a].index());
                if top.flags & path_flag::FINAL != 0
                    && matches!(
                        action,
                        Some(index::TRADE_ROUTE | index::GATHER | index::ATTACK | index::BUILD_AT)
                    )
                {
                    self.kill_current_order(u);
                    return Did::Nothing;
                }
                if !self.current_order(other).is_some_and(Order::is_move) {
                    let t = self.profile(Obj::Unit(other)).big_radius * 3;
                    if self.units[u].tolerance < t {
                        self.units[u].tolerance = t;
                        self.units[u].path.pop();
                        self.units[u].path.push(PathData {
                            tolerance: t,
                            ..top
                        });
                    }
                }
            }
            // `detect_unit_collision` writes `coll_x`/`coll_y` into the
            // order in place; take the copy back so the stores below keep
            // them.
            if let Some(m) = self.current_move(u) {
                mo = m;
            }

            let here = self.units[u].pos;
            // Already there: the Euclidean test before the step.
            if vector_dist(mo.waypoint.x - here.x, mo.waypoint.y - here.y)
                <= self.units[u].tolerance
            {
                mo.has_waypoint = false;
                let popped = self.units[u].path.pop();
                self.store_move(u, mo, flags);
                if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                    return Did::Something;
                }
                // `false`: `do_move`'s own arrival (`005f8844`) tests
                // `orderlist.length == 1` and nothing else — the gather
                // clause belongs to `move_step`'s snap arm alone.
                self.arrive(u, mo, false);
                return Did::Nothing;
            }
            self.store_move(u, mo, flags);
            // **A building across the leg** (`5f8844`..`5f8859`): a top the
            // tile grid flagged as crossing a footprint asks
            // [`Sim::resolve_block`], and an attack it queues ends the frame.
            if top.flags & path_flag::BLOCK != 0 && self.resolve_block(u) {
                return Did::Something;
            }
        }

        // The speed, and the straight-line check. `do_move` takes it from
        // the `+0x17c` virtual, which an animal overrides
        // ([`Sim::get_speed`]), then `× ai_speed` above 1 and the Modern
        // Infantry's `× 5 / 4`, truncating (`5f8893`..`5f88cb`; item 1109:
        // run404's Infantry steps 42 on a speed of 34).
        let mut speed = self.get_speed(u, 0);
        if self.ai.ai_speed > 1 {
            speed *= self.ai.ai_speed;
        }
        if self.is_modern_infantry(u) {
            speed = speed * 5 / 4;
        }
        // `if (masks & 8) goto STEP` (§4.4): the jump the straight-line
        // check takes when it succeeds lands **past** the pause check, so a
        // unit that verifies its line this frame steps this frame even with
        // a collision pause running — and the pause does not tick. Only the
        // re-plan's own `TAKE` comes back through `STEP_IF_MOVING`, and so
        // does a unit that already had the bit at entry.
        //
        // run10's `1/2` is the case, and it is worth a line: the collision
        // at frame 572 sets `pause = 3` and clears the bit; 573 re-verifies
        // and **steps with `pause` still 3**; 574–576 are the three still
        // frames. Ticking on 573 put this simulation a frame ahead for the
        // rest of the walk (2026-08-31, item 69).
        let mut straight_to_step = false;
        if !self.units[u].line_ok {
            self.units[u].path_recursion = 0;
            let goal = mo.waypoint;
            let r = self.find_path(u, &mut mo, goal);
            if r == 0 {
                let top = self.units[u].path.last().copied();
                match top {
                    Some(t) if t.to != self.units[u].pos => {
                        self.units[u].line_ok = true;
                        mo.has_waypoint = true;
                        mo.waypoint = t.to;
                        self.units[u].tolerance = t.tolerance;
                        self.store_move(u, mo, flags);
                    }
                    _ => self.units[u].line_ok = false,
                }
            }
            straight_to_step = self.units[u].line_ok;
            if !self.units[u].line_ok {
                // The straight line is not enough: the pathfinder's job
                // (§4.4). An unreachable goal with more orders queued kills
                // **this one** — ~~and the next~~: `5f8990`–`5f8997` jump to
                // `5f82a3`, one `kill_current_order(0)` and `return 1`. The
                // coverage pair's Transport Freighter `1/152`, whose move
                // ends inland, keeps the guard under it on block 2995 and
                // hands it ashore with `1/41` (`docs/AI.md` §172).
                let here = self.units[u].pos;
                if self.invalid_loc(u, mo.dest.tile(), true, false, false, false, true) != 0
                    && self.units[u].orders.len() > 1
                {
                    self.kill_current_order(u);
                    return Did::Something;
                }
                // The draw that chooses the grid — one `Random::get` off
                // the sync stream on the first `do_move` of any move
                // `find_path` refuses.
                self.mark(SITE_MOVE_GRID);
                let n = self.rng.roll();
                let thr = match n % 5 {
                    2 => 2 * CELL,
                    0 => 8 * CELL,
                    _ => 5 * CELL,
                };
                let far = (mo.waypoint.x - here.x).abs() + (mo.waypoint.y - here.y).abs() > thr;
                // `wflag` (which grid planned) only matters to the
                // resolve-block call below: a world plan waits a frame,
                // a near one asks [`Sim::resolve_block`].
                //
                // The length a positive return is compared against is read
                // **per arm**, and in the near arm *after* every pop: the
                // original captures `field_0xc0` beside each planner call,
                // and re-reads it once the unwind loop has run. Reading it
                // before the pops made "unchanged" nearly unreachable.
                let (r, len_before, wflag) = if far {
                    let len_before = self.units[u].path.len();
                    (self.find_wpath(u), len_before, true)
                } else {
                    // A non-final top equal to `last` is stale: pop it.
                    if let Some(t) = self.units[u].path.last().copied()
                        && t.flags & path_flag::FINAL == 0
                        && Some(t.to) == mo.last
                    {
                        self.units[u].path.pop();
                    }
                    // **`collide`, and the two grids it chooses between**
                    // (`docs/ORDERS.md` §4.4, `Unit::do_move@005f7b30`'s
                    // `field_0x88` test). A unit that has *not* been
                    // colliding drops its loose near waypoints and plans on
                    // tiles; one that has keeps them and plans on the
                    // 48-grid, which is the finer one and the only one that
                    // can get around the unit in the way.
                    //
                    // Taking the tile arm unconditionally is what put
                    // run53's `1/7` in a two-frame livelock at 5502: the
                    // sidestep `move_step` had just pushed was popped the
                    // next frame, the unit walked back into the same
                    // collider, and the pair repeated for the rest of the
                    // capture (item 204).
                    if self.units[u].collide != 0 {
                        let len_before = self.units[u].path.len();
                        (self.find_upath(u, false), len_before, false)
                    } else {
                        while let Some(t) = self.units[u].path.last().copied() {
                            if is_loose(&t) {
                                self.units[u].path.pop();
                            } else {
                                break;
                            }
                        }
                        let len_before = self.units[u].path.len();
                        (self.find_tpath(u), len_before, false)
                    }
                };
                // **The planner's roll is the order's, not the copy's**
                // (item 3637, `docs/AI.md` §333): a failed unit-grid search
                // writes `retry` into the live order
                // ([`Sim::roll_upath_retry`]), and the original goes on
                // writing through its `MoveOrder*` (`do_move@005f7b30:711`: `dest = 0`
                // in place), so every store below must carry the roll, not
                // the `mo` taken before the search. East Indies' Chinese
                // Heavy Fire Lance `1/215` on 4861 held `retry` 6 in theirs
                // and 0 in ours, and re-pathed six frames early.
                if let Some(m) = self.current_move(u) {
                    mo.retry = m.retry;
                }
                // A positive return with the stack length unchanged counts
                // as a refusal.
                let r = if r > 0 && self.units[u].path.len() == len_before {
                    0
                } else {
                    r
                };
                if r == 0 && !self.units[u].path.is_empty() {
                    let top = self.units[u].path.last().copied().expect("non-empty");
                    if top.flags & path_flag::FINAL == 0 {
                        self.units[u].path.pop();
                        // **A refused transport waypoint hands its bit
                        // down** (`do_move@005f7b30`, `005f8b89`–`005f8bb5`,
                        // item 1948): the popped top's `flags & 4`, with the
                        // stack not empty, pops the next entry and pushes it
                        // back with `flags |= 4`, tolerance kept — the walk
                        // to a shore the first plan marked still ends in a
                        // cast.
                        if top.flags & path_flag::BOARDS_TRANSPORT != 0
                            && let Some(mut next) = self.units[u].path.pop()
                        {
                            next.flags |= path_flag::BOARDS_TRANSPORT;
                            self.units[u].path.push(next);
                        }
                        mo.has_waypoint = false;
                        self.store_move(u, mo, flags);
                        return Did::Something;
                    }
                    self.units[u].path.pop();
                    flags &= !flag::PATHED;
                    self.store_move(u, mo, flags);
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                } else if r == -1 {
                    self.kill_current_order(u);
                    if self
                        .current_order(u)
                        .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Build(_)))
                    {
                        self.kill_current_order(u);
                    }
                    return Did::Something;
                }
                // TAKE: the new top is the target, and the straight line to
                // it is verified the same frame.
                let Some(top) = self.units[u].path.last().copied() else {
                    self.kill_current_order(u);
                    return Did::Something;
                };
                mo.last = None;
                mo.has_waypoint = true;
                // **The new top across a footprint** (`5f8bf0`..`5f8c05`):
                // asked before `dest_x`/`dest_y` are written, so an attack
                // it queues leaves the move's old waypoint under it.
                if top.flags & path_flag::BLOCK != 0 {
                    self.store_move(u, mo, flags);
                    if self.resolve_block(u) {
                        return Did::Something;
                    }
                    if let Some(m) = self.current_move(u) {
                        mo = m;
                    }
                }
                // `dest_x`/`dest_y` and the unit's `+0x60` tolerance are
                // **not** the new top's yet: `TAKE` hands `find_path` the
                // top itself and writes the move only once the line is
                // verified (`do_move:694`..`698`, below). A near plan whose
                // line fails keeps the old waypoint — run460's Knight at
                // peace, looping on the ring's corner from 811, holds its
                // goal (42720, 20064) and tolerance 0 (item 1209).
                self.units[u].path_recursion = 0;
                let r2 = self.find_path(u, &mut mo, top.to);
                if r2 == 0 {
                    self.units[u].line_ok = true;
                }
                if !self.units[u].line_ok {
                    // The line to the new top failed too: with a world-grid
                    // plan, wait for the next frame; a near plan asks
                    // [`Sim::resolve_block`] of whatever top the stack has
                    // now, flagged or not, and returns either way
                    // (`5f8c19`..`5f8c2f`).
                    self.store_move(u, mo, flags);
                    if !wflag {
                        self.resolve_block(u);
                    }
                    return Did::Something;
                }
                // **The verified line reads the stack again** (`5f8c3d`–
                // `5f8c5d`, `docs/GROUPS.md` §32): `peek`, then the order's
                // `dest_x`/`dest_y` and the unit's `+0x60` tolerance from
                // the top. A detour `find_path` just pushed and verified
                // (`go_around_building`, [`crate::movement::find_path_view`]) is the
                // top now, so it is what the step walks at — not the world
                // entry under it. Great Lakes' `1/40` walked past its own
                // detour for seventeen frames without this (item 795).
                if let Some(t) = self.units[u].path.last().copied() {
                    mo.waypoint = t.to;
                    self.units[u].tolerance = t.tolerance;
                }
                self.store_move(u, mo, flags);
                // `goto LAB_005f8c60` (the export's `do_move:699`): the
                // re-plan's `TAKE` lands past the pause check, as `goto
                // STEP` does — the check (`:729`) is the `else` of the
                // whole `masks & 8 == 0` block, so a unit that re-plans
                // steps with its `pause` untouched and stands no frame
                // (East Indies 16009: the Supply Wagon `1/153`, item 1383).
                straight_to_step = true;
            }
        }

        // Stepping — `STEP_IF_MOVING`, which `goto STEP` jumps past.
        if !straight_to_step && mo.pause != 0 {
            mo.pause -= 1;
            self.store_move(u, mo, flags);
            // **An unarmed attack-move stands out its pause**
            // (`do_move@005f7b30:734`–`741`): with the order's own type
            // `ATTACK_TO` or `GROUP_ATTACK_TO` and the type's attack 0, the
            // waiting frame is a `set_anim(CHAR_DEFAULT, 0, 1)` — one stand
            // per guy, each a draw (`Guy::set_anim+0x97a < Unit::set_anim <
            // Unit::do_move+0x11cf`). The pause it stands out is
            // `do_attack_to_pause`'s fifteen (`docs/ORDERS.md` §24.9).
            let t = Order {
                flags,
                body: Body::Move(mo),
            }
            .index();
            if matches!(t, index::ATTACK_TO | index::GROUP_ATTACK_TO)
                && self.profile(crate::combat::Obj::Unit(u)).attack == 0
            {
                self.mark(anim::SITE_PAUSE_STAND);
                self.set_default_anim(u);
            }
            return Did::Something;
        }
        self.unit_step(u, mo, speed)
    }

    /// `Unit::resolve_block@005fccc0` (`docs/ORDERS.md` §4.4, item 1209):
    /// what an armed walker does about a building across its near plan.
    /// It reads the path's top and nothing else:
    ///
    /// 1. the top's tile refuses the unit plainly (`invalid_loc(t, 0, 0,
    ///    0, 0, 0)`) **and** passes it under `param_5` (`invalid_loc(t, 0,
    ///    0, 1, 0, 0)`) — which is [`Sim::invalid_loc`]'s armed arm, so the
    ///    walker is armed and the tile another player's footprint;
    /// 2. `find_any_building_at(t, who, FILTER_ALL)` finds its building;
    /// 3. the owner decides (`5fcd6e`..`5fce11`):
    ///    - **not at war** (`LeaderData::is_enemy` no): the walker's
    ///      leader ors `2` into `agendas[owner]` and the move goes on
    ///      (0);
    ///    - **at war**: `add_attack_order(building, QUEUE_FIRST, 0, 0)`,
    ///      and `do_move` returns (1).
    ///
    /// The **own** arm (`5fcd72`..`5fcdb4`: `build_masks |= 1` on a
    /// non-gather type) is dead: step 1's second test refuses a building
    /// of the walker's own (`invalid_loc`'s arm answers 4 on
    /// `find_who == who`), and both finds are the same call on the same
    /// frame. It is not built. `AI.md` §13 names this the only writer of
    /// `build_masks & 1`, so the orphan check's disband arm is dead too.
    ///
    /// An empty stack reads the list's stale first slot in the original;
    /// here it answers 0 (SEAM: no caller here reaches it empty).
    pub(crate) fn resolve_block(&mut self, u: usize) -> bool {
        let Some(top) = self.units[u].path.last().copied() else {
            return false;
        };
        let t = top.to.tile();
        if self.invalid_loc(u, t, false, false, false, false, false)
            == crate::movement::path::loc::VALID
        {
            return false;
        }
        if self.invalid_loc(u, t, false, false, true, false, false)
            != crate::movement::path::loc::VALID
        {
            return false;
        }
        let who = self.units[u].owner;
        let Some(b) = self.find_any_building_at(t, who) else {
            return false;
        };
        let owner = self.buildings[b].owner;
        if owner == who {
            return false;
        }
        if !self.is_enemy(who, owner) {
            if let Some(x) = self
                .leaders
                .agendas
                .get_mut(usize::from(who))
                .and_then(|r| r.get_mut(usize::from(owner)))
            {
                *x |= 2;
            }
            return false;
        }
        self.add_attack_order(u, Obj::Building(b), QueuePos::First, false, false);
        true
    }

    /// `Unit::do_group_move@005e79a0` (`docs/ORDERS.md` §8.3) — the
    /// per-frame half of a `GroupMoveOrder`, and the reason a marching
    /// formation costs the original **one** `do_move` and not one per
    /// member.
    ///
    /// The leader steps a plain move and then rotates the group's slot
    /// table by its own heading; every follower is steered off the
    /// leader's *current* position plus its rotated offset, and takes
    /// `move_step` directly. That difference is the whole of it: three
    /// archers walking as a squad spend one `Unit::do_move+0xe84` between
    /// them, where N independent moves spend three.
    pub(super) fn do_group_move(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::Move(mo),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        let Some(gm) = mo.group else { return };
        // The head and the leader arm are [`crate::orders::group_move`]'s steps
        // (item 3376's sweep holds them to the original); the follower arm
        // is entered through its view. `this->group == -1` (`5e79f0`) — the
        // back-pointer ([`crate::Unit::group_ptr`], `docs/GROUPS.md` §23)
        // names no seat — is the view's `group_unset`.
        let g = self.group_of(u);
        let mut view = crate::orders::group_move::SimGroupMove::new(self, u, frame, g, gm);
        crate::orders::group_move::do_group_move_steps(&mut view);
    }

    /// `do_group_move`'s follower arm (`5e7c8c`–`5e8660`).
    pub(crate) fn group_move_follower(
        &mut self,
        u: usize,
        frame: i64,
        g: &crate::groups::group::Group,
        mut gm: GroupMove,
        mut mo: MoveOrder,
        flags: u8,
    ) {
        let l = gm.leader;
        // 1. The leader has to still be usable: alive, on the map, in my
        //    group, and holding a group order with **my** `id` — or a
        //    `CHANGE_FORM`, which this crate does not model.
        //    "In my group" is the two back-pointers agreeing
        //    (`5e7c8c`: `leader.+0x80 != this.+0x80`), not a list.
        let lost = !(self.units[l].alive()
            && self.units[l].on_map
            && self.units[l].group_ptr == self.units[u].group_ptr
            && self.still_group_move(l, gm.id));
        // 2. `form_id` is rewritten from the list on every frame that
        //    reaches here (`5e7ea0`). A member the list no longer holds
        //    loses its group outright — `+0x80 = −1` (`5e7ed8`), then the
        //    ungroup (`docs/GROUPS.md` §23).
        let Some(i) = g.list.iter().position(|&m| m == u) else {
            self.units[u].group_ptr = None;
            self.ungroup_move_order(u, gm.id);
            return;
        };
        if gm.form_id != i {
            gm.form_id = i;
            mo.group = Some(gm);
            self.store_move(u, mo, flags);
        }
        if lost {
            // `LAB_005e7ee2`: more than `0x5ff` from **the order's own
            // destination** — my slot, not the leader's cell — and I take
            // the formation over; otherwise the order degrades.
            let here = self.units[u].pos;
            if vector_dist(mo.dest.x - here.x, mo.dest.y - here.y) > 0x5ff {
                self.group_refresh_order(g, u);
                self.group_rewrite_leader(g, u, gm.id);
                return;
            }
            self.ungroup_move_order(u, gm.id);
            return;
        }
        // 3. An attacking follower whose target is already in range stops
        //    marching (`5e8611`).
        if let Some(a) = self.action_of(u)
            && matches!(self.units[u].orders[a].body, Body::Attack(_))
            && let Some(t) = self.units[u].combat.target
            && self.is_in_range(Obj::Unit(u), t)
        {
            self.kill_current_order(u);
            return;
        }
        // 4. My slot this frame: the leader's **current** position plus my
        //    rotated offset.
        let Some(slot) = self.group_slot_point(g, l, i) else {
            self.ungroup_move_order(u, gm.id);
            return;
        };
        // 5. Unwind my path to its goal (`5e8034`): pop entries until one
        //    carries `FINAL`, keep it, and push it back. An empty stack is
        //    the original's uninitialised read; the order's own
        //    destination stands in for it.
        let mut goal = PathData {
            to: mo.dest,
            tolerance: 0,
            flags: path_flag::FINAL,
        };
        while let Some(top) = self.units[u].path.pop() {
            goal = top;
            if top.flags & path_flag::FINAL != 0 {
                break;
            }
            if self.units[u].path.is_empty() {
                goal.flags |= path_flag::FINAL;
                break;
            }
        }
        self.units[u].path.push(goal);

        let here = self.units[u].pos;
        let d_slot = vector_dist(slot.x - here.x, slot.y - here.y);
        let d_goal = vector_dist(goal.to.x - here.x, goal.to.y - here.y);
        // 6. **The formation is over** when my slot buys me no more than
        //    `0x60`, or the goal is within `0x180`: the order becomes a
        //    plain move at the goal (`5e80fc`, `5e8105`).
        if d_goal - d_slot <= 0x60 || d_goal <= 0x180 {
            gm.in_group = false;
            mo.group = Some(gm);
            mo.has_waypoint = true;
            mo.waypoint = goal.to;
            self.store_move(u, mo, flags);
            self.ungroup_move_order(u, gm.id);
            return;
        }
        // 7. Where to walk. Straight to my slot when the leader is my own
        //    captain, or when the bearing to the slot is within 120° of
        //    the bearing to the goal (`5e8167`'s `0x55555555`); otherwise
        //    off the leader's next waypoint, or the midpoint.
        let to_slot = find_angle(slot.x - here.x, slot.y - here.y);
        let to_goal = find_angle(goal.to.x - here.x, goal.to.y - here.y);
        let straight = self.squad_captain(u) == l || within_third(to_slot, to_goal);
        let mut disagrees = false;
        if straight {
            self.units[u].path.push(PathData {
                to: slot,
                tolerance: 0,
                flags: 0,
            });
            mo.waypoint = slot;
            gm.in_group = true;
        } else {
            // `5e8173`: a slot this close is not worth chasing at all.
            if d_slot < 0x30 {
                return;
            }
            let lead = self.units[l].path.last().copied();
            match lead {
                Some(w) if gm.in_group => {
                    // The group's own formation — an army's, or a pushed
                    // group's ([`Sim::gstate`]). This read an army's list
                    // with `unwrap_or(0)`, which a pushed group (no army)
                    // indexed out of bounds: Great Lakes 18333, reached
                    // once item 695 moved the word past 14529. A slot the
                    // state does not hold stands at the leader's point.
                    let off = self
                        .gstate(g)
                        .and_then(|st| st.curr.get(i).copied())
                        .unwrap_or_default();
                    let to = Pos::new(w.to.x + off.x, w.to.y + off.y);
                    self.units[u].path.push(PathData {
                        to,
                        tolerance: 0,
                        flags: 0,
                    });
                    mo.waypoint = to;
                    disagrees = !within_third(to_slot, self.units[u].movement.heading);
                    if vector_dist(to.x - here.x, to.y - here.y) < 0x60 {
                        gm.in_group = false;
                    }
                }
                _ => {
                    mo.waypoint = Pos::new((slot.x + goal.to.x) / 2, (slot.y + goal.to.y) / 2);
                    disagrees = true;
                }
            }
        }
        mo.has_waypoint = true;
        mo.group = Some(gm);
        self.store_move(u, mo, flags);
        if mo.waypoint == here {
            return;
        }
        self.units[u].tolerance = 0;
        // 8. The speed. A follower walking with the formation is allowed a
        //    third more, capped at nine; one walking across it takes half.
        let v = if disagrees {
            // `5e83f6`: flag **0**, so a follower cutting across the
            // formation is capped by the group like any plain mover.
            self.get_speed(u, 0) / 2
        } else {
            // **And a follower in formation reports** (`5e8336`, which is
            // `Group::report_speed@00713bb0` inlined): its own
            // `UnitData::speed` — layer two, not `get_speed`, so neither
            // the ground it stands on nor the order it carries enters the
            // group's cap — drives the pair down and never up. This is
            // what puts a slow squad's speed on a fast leader, and a
            // follower that stops reporting lets the cap drift back to
            // the leader's own over the next two frames.
            //
            // Under a Forced March (`+0x4b`, set by the leader's report
            // above) only a follower `has_general(0x8000, -1)` puts near
            // a marching hero reports (`5e8303`–`5e8318`), so the march's
            // stragglers do not hold the army back.
            if !self.gstate(g).is_some_and(|st| st.march) || self.near_marching_hero(u) {
                let own = self.unit_speed(u);
                self.group_report_speed(g, own);
            }
            // `5e8355`: flag **1**. A follower keeping formation is
            // allowed its own speed and a third on top, uncapped — the
            // catch-up, and the reason the cap never stops the block
            // closing up.
            let v = self.get_speed(u, 1);
            v + (v / 3).min(9)
        };
        // 9. **A slot the world refuses is not walked into at all**
        //    (`5e838a`): the formation gives up and the order degrades,
        //    which is what keeps a follower from marching into its
        //    neighbour rather than standing blocked beside it.
        //
        //    **And a slot in a wood flushes its birds** (`5e83f0`–`5e84cb`,
        //    item 1293): within `0x300` Manhattan of the unit, a slot whose
        //    cell is `FOREST` sends up a flock from a unit in a group —
        //    [`Sim::group_flock`].
        if self.invalid_loc(u, mo.waypoint.tile(), false, false, false, false, false) != 0 {
            self.group_flock(u, mo.waypoint);
            self.ungroup_move_order(u, gm.id);
            return;
        }
        //    **Then the follower's own fire on the move** (`5e84d0`–
        //    `5e856d`, item 3336): [`Sim::cavarch_follower`].
        self.cavarch_follower(u, frame);
        // 10. **The step's own answer is the formation's end condition**
        //     (`5e856d`–`5e8660`). `move_step` returns 0 from exactly
        //     three places — blocked and still owing a turn, blocked and
        //     handed to `resolve_unit_collision`, and a tile the world
        //     refused ([`Sim::unit_step`]) — and every one of them drops
        //     the follower out of formation: the head order is re-read,
        //     and unless the attack hand-off takes it the group move
        //     degrades into N independent moves. It is the same tail the
        //     leader takes off `do_move`'s 0 in
        //     [`Sim::group_move_leader`], and `docs/ORDERS.md` §8.3 read
        //     it as "arrived", which is the one thing a 0 never means:
        //     arrival returns 1.
        //
        //     SEAM: the attack-context hand-off above it —
        //     `Group::action_attack` for a `GROUP_ATTACK_TO` whose
        //     `collide_o` is a valid target, `kill_current_order`
        //     otherwise — is the leader arm's own seam and is not
        //     modelled; no capture reaches it, because every collider in
        //     the captures on disk belongs to the colliding unit's own
        //     player.
        if self.unit_step(u, mo, v) == Did::Nothing && self.still_group_move(u, gm.id) {
            self.ungroup_move_order(u, gm.id);
        }
    }

    /// `ObjectData::get_captain` — the head of `u`'s own squad, up the
    /// `o_up` chain. `Unit::ungroup_move_order` starts there and walks
    /// back down, so ungrouping any figure ungroups the whole squad.
    pub(crate) fn squad_captain(&self, u: usize) -> usize {
        let mut at = u;
        while !self.units[at].captain {
            match self.units[at].o_up {
                Some(c) if c != at && self.units[c].alive() => at = c,
                _ => break,
            }
        }
        at
    }

    /// Is `u`'s head order still a group move carrying `id`?
    pub(crate) fn still_group_move(&self, u: usize, id: i64) -> bool {
        self.current_order(u).is_some_and(|o| match o.body {
            Body::Move(m) => m.group.is_some_and(|x| x.id == id),
            _ => false,
        })
    }

    /// `Unit::ungroup_move_order@005fd140` — replace the group order
    /// carrying `id` with the plain move it stands on, up the captain
    /// chain and then down every subordinate.
    ///
    /// `GROUP_ATTACK_TO → ATTACK_TO` and `GROUP_MOVE → MOVE_TO`; here the
    /// kind is already the plain one and dropping [`MoveOrder::group`] is
    /// the whole conversion. `dest = 0` clears the waypoint, and a member
    /// that is **not** the leader also loses [`flag::PATHED`] and its
    /// path — so the group's shared plan dies with the formation and each
    /// unit re-plans for itself.
    ///
    /// The control flow is [`ungroup_move_order_steps`] (item 3268's
    /// sweep, `crate::sweep::unit_ungroup_move_order`), run over this
    /// crate's own order lists through [`SimUngroup`].
    pub(crate) fn ungroup_move_order(&mut self, u: usize, id: i64) {
        let mut view = SimUngroup {
            sim: self,
            unit: u,
            cursor: 0,
            pending: None,
        };
        ungroup_move_order_steps(&mut view, u, id, false);
    }

    /// `Unit::kill_group_move@005e3400` over the group
    /// (`Group::kill_group_move@007123f0`): every member's orders lose the
    /// group plan, and every one carrying `id` that is **not** an
    /// attack-move is killed. A `GROUP_ATTACK_TO` survives both arms.
    pub(crate) fn kill_group_move(&mut self, g: &crate::groups::group::Group, id: i64) {
        // `007123f9`: `normalize(this)` before anything else, which is
        // where the group's speed goes back to its leader's
        // (`docs/GROUPS.md` §18) after a march's own reports have walked
        // it up to the leader's uncapped speed.
        group_member_walk(
            self,
            g.list.len(),
            |s| s.group_set_speed(g),
            |_, k| g.list[k],
            |s, u| s.units[u].alive(),
            |s, u| s.units[u].on_map,
            |s, u| s.unit_kill_group_move(u, id),
        );
    }

    /// `Unit::kill_group_move@005e3400` for one member: each pathed move
    /// but a `GROUP_ATTACK_TO` loses its plan, and a group order carrying
    /// `id` that is not a `GROUP_ATTACK_TO` kills the head where the walk
    /// stands. The walk is [`kill_group_move_steps`] (item 2368), which the
    /// sweep `sweep::unit_kill_group_move` holds to the original.
    pub(super) fn unit_kill_group_move(&mut self, u: usize, id: i64) {
        kill_group_move_steps(
            self,
            id,
            |s| (!s.units[u].orders.is_empty()).then_some(0),
            |s| !s.units[u].orders.is_empty(),
            |s, k| (k + 1) % s.units[u].orders.len(),
            // `+0xd8` and the advanced node's `prev` against the tail
            // (`5e3503`-`5e351a`): a non-empty list not wrapped to its first.
            |s, k| !s.units[u].orders.is_empty() && k != 0,
            |s, k| s.units[u].orders[k].is_move(),
            |s, k| i32::from(s.units[u].orders[k].index()),
            |s, k| s.units[u].orders[k].has(flag::PATHED),
            |s, k| s.units[u].orders[k].flags &= !flag::PATHED,
            |s| s.kill_current_path(u),
            |s, k| match s.units[u].orders[k].body {
                Body::Move(m) => m.group.map(|g| g.id),
                _ => None,
            },
            // `kill_current_order` kills the head and leaves the engine's
            // cursor on the new head (`5e2cb0`), whatever order matched.
            |s, _| {
                s.kill_current_order(u);
                0
            },
        );
    }

    /// Writes a move order's fields back to the front of the list.
    pub(super) fn store_move(&mut self, u: usize, mo: MoveOrder, flags: u8) {
        // The order being stepped is the front one — **unless**
        // `set_new_location` has just pushed a transport cast in front of
        // it (`docs/TRANSPORT.md` §6). The original writes through a
        // pointer to the move and so does not care; here the write has to
        // find it again, one place back.
        let i = usize::from(matches!(
            self.units[u].orders.front().map(|o| o.body),
            Some(Body::Cast(_))
        ));
        if let Some(order) = self.units[u].orders.get_mut(i) {
            order.flags = flags;
            if let Some(m) = order.move_mut() {
                *m = mo;
            }
        }
    }

    /// `Unit::find_path` (§4.6): 0 = walk straight, 1 = plan, 2 = abort (a
    /// recursive call's answer passed up; nothing produces it at the base).
    ///
    /// The body is [`crate::movement::find_path_view::find_path_steps`], the original's
    /// flow over a view, which the emulator sweep also runs (item 3386); the
    /// march, the pull-back and the detour's replay are described there and
    /// in `docs/PATHFINDER.md` §9.
    pub(super) fn find_path(&mut self, u: usize, mo: &mut MoveOrder, goal: Pos) -> u8 {
        let mut view = SimFindPath { sim: self, u, mo };
        crate::movement::find_path_view::find_path_steps(&mut view, goal) as u8
    }

    /// `Unit::go_around_building@005fc350` (§4.6) — the answer to a
    /// straight line that clips a *building*, which the pull-back cannot
    /// help with: the goal's own tile is clear, so there is nothing to pull
    /// back, and the obstacle sits somewhere in the middle of the march.
    ///
    /// It is a **tile-edge walk**, and its shape is the same in both axes.
    /// The step that failed crossed one tile edge, so the unit has a *lane*
    /// (the row or column it is still standing in) and a *blocked lane*
    /// (the one it tried to enter). The walk runs perpendicular to the
    /// crossing, one tile at a time in both directions from the blocked
    /// point, and stops when either the blocked lane opens (success, at
    /// that offset) or the unit's own lane closes (success only if the
    /// blocked lane happens to be open there). Off the map is a failure for
    /// that direction; both directions failing gives up.
    ///
    /// Of the two candidates the closer to `mo.waypoint` wins, with
    /// forward as the tie-break, and either is discarded if it resolves to
    /// the unit's own tile centre. A candidate more than `0x300` from the
    /// blocked point in either axis gives up — the detour would be longer
    /// than the pathfinder's own leg.
    ///
    /// Then one to three `PathData`s go on the stack, bottom first:
    /// **the turn-in point** in the blocked lane and a **`DETOUR`
    /// midpoint** between the lanes, both only when the unit's own lane is
    /// closed at the found offset; and always **the target**, in the unit's
    /// own lane, one tile back along the walk when the turn-in pair was
    /// pushed. Every one of them is placed `off % 0xc0 / 2` off its tile's
    /// low corner rather than at the centre, from the order's own
    /// `off_x/off_y` — so a formation's units do not all aim at the same
    /// point.
    ///
    /// Giving up is `mo.dest = 0; masks &= ~8; path_recursion = 10`, and
    /// the recursion counter is how the caller tells the two apart.
    pub(super) fn go_around_building(
        &mut self,
        u: usize,
        mo: &mut MoveOrder,
        last: Pos,
        ang: Angle,
        spd: i32,
    ) {
        // The body is `crate::movement::detour::go_around_building_steps`, the
        // original's flow over a view, which the emulator sweep also runs
        // (item 3330).
        let mut view = crate::movement::detour::SimDetour { sim: self, u, mo };
        crate::movement::detour::go_around_building_steps(&mut view, last, ang, spd);
    }

    /// The unit step and its arrival (§4.5): `move_step` through
    /// `docs/MOVEMENT.md`, then the Manhattan test against the waypoint's
    /// tolerance, the pop, the facing on the final waypoint, the kill.
    pub(super) fn unit_step(&mut self, u: usize, mut mo: MoveOrder, speed: i32) -> Did {
        // `move_step@005faf30:60`–`65`: a step leaves the trench —
        // `unit_masks &= ~0x2000000` and `unit_masks2 &= 0xfffdefff`
        // (item 3661).
        if self.units[u].combat.entrenched {
            self.units[u].combat.entrenched = false;
            self.units[u].unit_masks2 &= 0xfffd_efff;
        }
        // STEP (§4.4): `avoid_x/y = −1,−1` before every step.
        self.units[u].avoid = None;
        // `move_step@005faf30:66-100`, at its head: a fire-on-the-move
        // type aims its figures at what it holds, and walks under
        // `CHAR_ATTACKWALK` while its pivots bear ([`crate::combat::cavarch`]).
        let walk = self.cavarch_step_anim(u);
        let unit = &self.units[u];
        let m = unit.movement;
        let from = unit.pos;
        // `GuyData::turn_speed@005de340` reads `unit_masks & 0x80000` on the
        // **unit**, not a copy taken when the type was assigned, so the
        // pack bonus follows the boat rather than the record
        // (`docs/ORDERS.md` §6.8).
        let turning = movement::Turning {
            packed: unit.combat.packed,
            ..m.turning
        };
        let rate = movement::turn_speed(
            &self.tables.tuning,
            &turning,
            m.body.last_speed,
            m.body.avg_speed,
            movement::TurnMode::Unit,
        );
        // `local_18 & 4` at `005fb1a5`: the **current waypoint's** own
        // `TURN_FIRST`/`TRANSPORT` bit, which `shore_flagged` above writes
        // and until 2026-09-01 nothing read. A waypoint that crosses the
        // shore line is turned to before it is walked to, whatever the
        // distance.
        let turn_first = self.units[u]
            .path
            .last()
            .is_some_and(|e| e.flags & path_flag::TURN_FIRST != 0);
        // `unit_masks & 0x100000`, the one-shot half step the **soft**
        // collision arm left on this unit last frame
        // (`docs/COLLISION.md` §7). `move_step` spends it and says so;
        // the bit is the unit's, so the clearing is here.
        let half = self.units[u].half_step;
        // `GuyData::turn_speed(guy, 1)`, the second turn a helicopter's
        // step is aimed along (`movement::move_step_as`).
        let heli_rate = self.is_helicopter(u).then(|| {
            movement::turn_speed(
                &self.tables.tuning,
                &turning,
                m.body.last_speed,
                m.body.avg_speed,
                movement::TurnMode::Body,
            )
        });
        let step = movement::move_step_as(
            from,
            m.facing,
            mo.waypoint,
            speed,
            &m.turning,
            rate,
            turn_first,
            half,
            heli_rate,
        );
        if step.half_step_used {
            self.units[u].half_step = false;
        }
        // `Unit::move_step`'s own `set_angle`, which is the one call of the
        // eighteen this simulation makes — and it is where a marching
        // leader's turn-around flips its group's mirror flag
        // (`docs/GROUPS.md` §4.1, §6.3). It is passed the **heading**, before
        // and regardless of the turn: `move_step` calls it at the top, on the
        // bearing `find_angle` just returned. The facing that the step is
        // actually taken along is guy 0's, and only `Guy::do_turn` moves it.
        self.unit_set_angle(u, step.heading);
        // **The turning stand** (`docs/ANIM.md` §4.8). Both turn-in-place
        // arms hand `Guy::do_turn` a non-zero fifth argument, so a guy
        // with `guy_flags & 8` is asked for a turn animation — and a
        // packing type that has none takes the idle roll instead. The two
        // arms are two `do_turn` call sites and so two `ebp` chains.
        match step.turned_in_place {
            Some(movement::TurnArm::Near) => self.mark(crate::units::anim::SITE_TURN_NEAR),
            Some(movement::TurnArm::Far) => self.mark(crate::units::anim::SITE_TURN_FAR),
            None => {}
        }
        if step.turned_in_place.is_some() {
            self.do_turn_anim(u, m.facing, step.facing, step.heading);
        }
        let unit = &mut self.units[u];
        unit.movement.facing = step.facing;
        // **`Guy::do_turn`'s own `set_angle(guy 0, angle, 0)`** — every
        // arm that gets here calls `Guy::do_turn(guy 0, heading, facing,
        // …)` (`005fb2b4`, `005fb2e1` and the walking one), and its body
        // writes guy 0's `angle` and then runs `Guy::set_angle` on it with
        // the **new facing**, keeping only guy 0's own `des_angle`. That
        // call's crew loop tells every tracked figure its `des` from guy
        // 0's point and this angle, **after** `Unit::set_angle`'s heading
        // loop above and before the collision block. So a crew figure that
        // stood on its point last frame stands on it again when the blocked
        // stand asks, and rolls its idle: run1331's `1/126` (TI 280, a
        // Heavy Flaming Arrow), whose guy 2 the heading loop put on (32169,
        // 21229) and this one back on (32167, 21225), its body, on 3627
        // (item 3666, `docs/AI.md` §345; run1332a's packet watched both
        // writes). The loop reads guy 0's own `x` / `y`, the body's point
        // before `Guy::move` steps it; the name is not `from`, which the
        // arrival below reads as the unit's (a shadowing `from` here moved
        // fourteen other lobbies' floors).
        let guy0 = unit.movement.body.pos;
        self.crew_des(u, guy0, step.facing, false);

        // **And there the frame ends** (`docs/COLLISION.md` §8.8). Both
        // turn-in-place arms are `call Guy::do_turn(…, 0, 1); mov eax,1;
        // ret 8` on the listing — the far one returning at `005fb2b9`,
        // the near one at `005fb2e6` — so a frame spent turning reaches
        // **none** of what follows: not the four world-bounds tests, not
        // the collision probe, not `invalid_loc`, not `set_new_location`
        // and its reveal, and not the tail's arrival test.
        //
        // The probe is the one that cost a frame. `detect_unit_collision`
        // does its clearing on **every** path out (§4.1), so running it on
        // a turning unit wipes the `collide_o`/`collide_who` that
        // `do_move`'s own waypoint probe wrote three statements earlier
        // and ages `collide` to zero — the original keeps both, because it
        // never asks. run90 block 7809 is the case: `1/7` turns 44° in
        // place with `1/6`'s block over the cell its new leg ends in, and
        // the original carries `collide 1 / collide_o 6 / collide_who 1`
        // into the dump where this crate carried the fields clear.
        //
        // The store is still owed — this crate steps on a *copy* of the
        // order, and `do_move`'s waypoint block writes `coll_x`/`coll_y`
        // into the real one (§4.3's "the store is into the order").
        if step.turned_in_place.is_some() {
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            return Did::Something;
        }

        // **The collision block** (`docs/COLLISION.md` §5). The proposed
        // point is tested against the occupancy index; a blocked step
        // either snaps through onto a sidestep waypoint, waits out the
        // turn it still owes, gives up and calls the waypoint reached, or
        // goes to `resolve_unit_collision`.
        let top = self.units[u].path.last().copied();
        let mut target = step.pos;
        // `move_step`'s probe is `(x, y, 0, 1, 0, 0, 0)`: a ship takes the
        // second arm, and a push that handles the point answers no
        // collision before the land scan (`docs/COLLISION.md` §13.2).
        let hit = if self.takes_boat_arm(u) && self.detect_boat_collision(u, target, true) {
            None
        } else {
            self.detect_unit_collision(u, target)
        };
        // `detect_unit_collision` writes `coll_x`/`coll_y` **into the
        // order**, and the original walks `move_step` on a pointer to it,
        // so every later write of the move's own fields keeps the pair.
        // This crate steps on a copy, so take the pair back before any
        // `store_move` below puts the stale one back — the same reason
        // `do_move`'s waypoint test does it, and the arm that showed it is
        // the one that stores and returns while a turn is still owed.
        if let Some(m) = self.current_move(u) {
            mo.coll = m.coll;
        }
        if let Some(_other) = hit
            && step.snapped
        {
            // **The snap arm's own collision block** (`005fb3bd`,
            // `docs/COLLISION.md` §5.4). `move_step` splits on
            // `param_2 < local_28` and has a collision block on *each*
            // side; this crate had only the partial step's, and spent it
            // for both. The snap's does none of what §5 lists — no
            // sidestep snap-through, no wait on an owed turn, no
            // `resolve_unit_collision`, no widened tolerance. It clears
            // the collider the probe just named (`field_0x8a = 0xffff`,
            // `field_0xb3 = 0xff` — `collide_o` and `collide_who`),
            // consumes the waypoint where the unit stands, and falls into
            // the same tail the accepted step uses.
            //
            // Great Lakes 9134 is the frame: `1/32` walks its formation
            // slot in ~24-unit hops, arrives on each within one step, and
            // on 9134 the hop it snaps to is blocked. The original stands
            // for the frame and does everything else one frame later;
            // this crate ran the give-up chain, snapped the unit back to
            // `(42792, 22824)` and ungrouped it a frame early.
            self.units[u].collide_o = -1;
            self.units[u].collide_who = -1;
            mo.has_waypoint = false;
            let popped = self.units[u].path.pop();
            self.mark(crate::units::anim::SITE_SNAP_BLOCKED);
            self.set_default_anim(u);
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            self.units[u].movement.dest = None;
            if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                return Did::Something;
            }
            self.arrive(u, mo, true);
            return Did::Something;
        }
        let mut gave_up = false;
        if let Some(other) = hit {
            let (dx, dy) = (mo.waypoint.x - from.x, mo.waypoint.y - from.y);
            let through = top.is_some_and(|t| t.flags & path_flag::SIDESTEP != 0)
                && !self.detect_quick(u, top.expect("tested").to, false)
                && dx.abs() < 0x61
                && dy.abs() < 0x61;
            if through {
                // The final snap through a collision: walk onto the
                // waypoint itself and let the arrival test take it.
                target = mo.waypoint;
            } else {
                // The blocked stand — `move_step:281`, the call at
                // `005fb74e` and so the site `+0x823`
                // ([`crate::units::anim::SITE_BLOCKED`], `docs/COLLISION.md` §5).
                // It sits **before** all three give-up tests, so a unit
                // still owed a turn has re-rolled its idle by the time
                // `move_step` returns.
                self.mark(crate::units::anim::SITE_BLOCKED);
                self.set_default_anim(u);
                // **The two arms `move_step` answers 0 from**
                // (`005fb689` and `005fb6df`, the only `return 0`s the
                // function has besides the tile refusal below). The value
                // is not decoration: `do_group_move` reads it on **both**
                // sides of the formation — the leader's through
                // `do_move`, the follower's straight off `move_step` —
                // and a zero ungroups the whole squad
                // (`docs/ORDERS.md` §8.3). Answering `Did::Something`
                // here kept run76's Archer squad in formation for the
                // rest of its march where the original degrades it into
                // three independent moves on the frame the leading
                // Archer is squeezed onto its cell centre (item 236).
                if step.owed != 0 {
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    self.store_move(u, mo, flags);
                    return Did::Nothing;
                }
                let manh = dx.abs() + dy.abs();
                let reach = self.profile(Obj::Unit(u)).big_radius
                    + self.profile(Obj::Unit(other)).big_radius;
                let give_up = reach * 3 <= manh
                    || self.units[u].collide < 0x1a
                    || (top.is_none_or(|t| t.tolerance == 0 || t.flags & path_flag::SIDESTEP != 0)
                        && top.is_none_or(|t| t.flags & path_flag::FINAL == 0));
                if give_up {
                    self.resolve_unit_collision(u);
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    if self.current_order(u).is_some_and(Order::is_move) {
                        let mo = self.current_move(u).expect("a move order");
                        self.store_move(u, mo, flags);
                    }
                    return Did::Nothing;
                }
                // Give up on reaching it exactly: the waypoint is close
                // enough now.
                self.units[u].tolerance = manh * 2;
                gave_up = true;
            }
        }

        let mut arrived = false;
        let mut snapped_in = false;
        if gave_up && self.world.accepts(target) {
            // **The give-up does not step** (`docs/COLLISION.md` §17).
            // `005fb7b5`–`005fb7bb` write `tolerance = local_28 * 2` and
            // `jmp 005fb82c`, past the step's `invalid_loc`,
            // `set_anim(walk)` and `set_new_location` (`005fb7d1`–
            // `005fb826`), onto the tail's arrival test itself:
            // `|dest_y − y| + |dest_x − x|` from where the unit stands,
            // `jg` over `tolerance` returns 1 (`005fb82c`–`005fb85c`), and
            // otherwise `dest = 0` and the pop (`005fb862`). This crate
            // took the step first, and run218's `1/23`, blocked 26 frames
            // by The Despot, walked 26 on 16459 where the original pops
            // its waypoint in place (Great Lakes 16460).
            let (dx, dy) = (mo.waypoint.x - from.x, mo.waypoint.y - from.y);
            arrived = dx.abs() + dy.abs() <= self.units[u].tolerance;
        } else if self.world.accepts(target) {
            // **The step that changes tile is asked permission**
            // (`move_step@005faf30`, `005fb7c1`–`005fb7fd`). The original
            // compares the tile of the proposed point against the tile it
            // is standing on, and where they differ calls
            // `UnitData::invalid_loc` with all five flags clear — the same
            // call `find_path` and `go_around_building` make. A refusal
            // drops the step whole: no `set_anim`, no `set_new_location`,
            // no reveal, and the order keeps its waypoint, so the unit
            // stands where it is, turns another frame's worth, and tries
            // the step again next frame from a bearing it has turned
            // further round. It is the last of the four things
            // `docs/MOVEMENT.md`'s `move_step` section listed as not
            // modelled.
            //
            // **And it clears the verified line on the way out**
            // (`005fb7c1`-`005fb7fd`: the `& 0xfffffff7` and the
            // `return 0` are the same two lines), which is what makes the
            // next frame re-verify rather than step straight again. This
            // crate's one reader is `do_move`'s `if !line_ok` above, so
            // leaving the bit standing cost run65 a `Unit::find_path` the
            // original enters on 6207 and this crate did not.
            if target.tile() != from.tile()
                && self.invalid_loc(u, target.tile(), false, false, false, false, false) != 0
            {
                // **The snap arm consumes the waypoint anyway**
                // (`005fb41d`–`005fb491`, `docs/AI.md` §229). Its refusal
                // is `jne 005fb48e`, past the walk and `set_new_location`
                // alone and onto `dest = 0` and the pop, where the unit
                // stands, with `unit_masks & 8` untouched. Only the partial
                // step's refusal (`005fb7c1`) clears the bit and answers 0.
                // run866's Heavy Tank `1/11` snaps onto a sidestep over the
                // shore on 5314; ours kept it and re-planned a frame later
                // with a grid draw the original never spends.
                if step.snapped {
                    mo.has_waypoint = false;
                    let popped = self.units[u].path.pop();
                    let flags = self.current_order(u).map_or(0, |o| o.flags);
                    self.store_move(u, mo, flags);
                    self.units[u].movement.dest = None;
                    if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
                        return Did::Something;
                    }
                    self.arrive(u, mo, true);
                    return Did::Something;
                }
                self.units[u].line_ok = false;
                return Did::Nothing;
            }
            // **`move_step`'s own `set_anim`, and it is a clock and not
            // a picture** (`docs/ANIM.md` §4.9). Both arms of an accepted
            // step — the partial one at `move_step:304` and the Manhattan
            // snap at `:355` — call `Unit::set_anim(CHAR_WALK, 0, 1)` on
            // **every guy** immediately before `set_new_location`, so a
            // walking unit asks for its walk *twice* a frame: here, and
            // again in `Guy::move`'s own half ([`Sim::guys_follow_on`]).
            //
            // For guy 0 the pair is idempotent — a walk still inside its
            // length keeps its clock — and for a **crew figure whose
            // packet is empty** it is the whole mechanic.
            // `Guy::set_anim`'s walk arm splits on whether the resolved
            // slot is the one already playing: a *change* rescales, and
            // the rescale passes the old slot to both `get_anim_time`
            // calls, so it is `cur_time · t / t` and keeps the clock; only
            // the *same* slot takes an overrun length off it. A crew
            // figure comes off the mirror on `CHAR_SLOG` carrying guy 0's
            // clock, so this call is the change — `SLOG → WALK`, clock
            // kept — and `Guy::move`'s is then the same slot and
            // subtracts. With only the second call the figure kept the
            // mirrored clock, wrapped a frame early and paid an idle roll
            // the original does not: run73's window, Great Lakes 5571.
            //
            // **Past the turn-in-place arms**, which `return 1` at
            // `005fb2e1` and `005fb2b4`, so a frame spent turning never
            // reaches the call. This crate walks on to the collision block
            // and the store on such a frame — it steps on a *copy* of the
            // order, so the store is owed — but the animation is not.
            //
            // **And the snap arm asks the stand instead** when the
            // waypoint offsets were both zero at the top of `move_step` and
            // the order list holds at most one order (`UnitData+0xd8 < 2`,
            // `005fb44c`–`005fb474`): `UVar17` is `CHAR_DEFAULT`, and that
            // request is a draw, `Unit::set_anim+0x56 < move_step+0x549`.
            // `do_move`'s "already there" test does not take every such
            // case a step earlier: a unit standing on its final waypoint
            // while it turns in place spends the turn frames in the arms
            // above, and on the frame the turn completes it reaches the
            // snap with nothing left to walk. French East Indies 16857 is
            // the case — the General `1/79` turned six frames on
            // (6840, 13704) (`docs/AI.md` §130).
            if step.turned_in_place.is_none() {
                if step.snapped && mo.waypoint == from && self.units[u].orders.len() < 2 {
                    self.mark(crate::units::anim::SITE_SNAP_STAND);
                    self.set_default_anim(u);
                } else {
                    self.set_anim(u, walk, false, true);
                }
            }
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            if !self.set_new_location(u, target, false) {
                // The step crossed the waterline and was converted rather
                // than taken (`docs/TRANSPORT.md` §6): `move_step` returns
                // 1 on the spot, so the waypoint, the arrival test and the
                // reveal all wait for the boat. The flags are the move's
                // own, read before the cast went in front of it.
                self.store_move(u, mo, flags);
                return Did::Something;
            }
            // `Unit::set_new_location`'s half-cell test and the reveal
            // behind it (`docs/VISION.md` §6). `move_step` is the caller
            // that passes `param_3 = 0`, so this is the **ring** pass.
            self.moved_to(u, from, true);
            if step.arrived && target == step.pos {
                arrived = true;
                snapped_in = step.snapped;
            } else {
                let (dx, dy) = (mo.waypoint.x - target.x, mo.waypoint.y - target.y);
                if dx.abs() + dy.abs() <= self.units[u].tolerance {
                    arrived = true;
                }
            }
        } else {
            // A step outside the world. The original's four bounds tests
            // each `return 1` on their own and none of them touches
            // `unit_masks`, so the line stays verified and the unit tries
            // the same step again next frame; answering `Did::Nothing`
            // here ungrouped a marching formation whose leader reached
            // the edge (`group_move_leader`, 2026-09-05's R7).
            self.units[u].movement.dest = None;
            return Did::Something;
        }
        self.units[u].movement.dest = Some(mo.waypoint);
        if !arrived {
            let flags = self.current_order(u).map_or(0, |o| o.flags);
            self.store_move(u, mo, flags);
            return Did::Something;
        }
        mo.has_waypoint = false;
        let popped = self.units[u].path.pop();
        let flags = self.current_order(u).map_or(0, |o| o.flags);
        self.store_move(u, mo, flags);
        self.units[u].movement.dest = None;
        if popped.is_none_or(|p| p.flags & path_flag::FINAL == 0) {
            // Next frame takes the next waypoint and re-checks the line.
            // The clear belongs to *that* frame: the original writes
            // `MoveOrder::dest = 0` here and nothing else, and it is
            // `do_move`'s `dest == 0` arm (`005f7b30:428`) that clears
            // the bit when it lifts the next entry off the stack —
            // which is `mo.has_waypoint = false` and the clear above.
            return Did::Something;
        }
        self.arrive(u, mo, snapped_in);
        Did::Something
    }

    /// The final waypoint: `set_angle(mo->angle)` when the move is the only
    /// order — **or, on the Manhattan-snap arm only, when the action beneath
    /// is a gather**; clear the pathed bit; kill.
    ///
    /// The two arms of `move_step@005faf30` end with two different tests, and
    /// the difference is one clause: the snap's is `orderlist.length == 1 ||
    /// get_action()->get_type() == GATHER` (`005fb562`), the partial step's
    /// is `orderlist.length == 1` alone (`005fb4a8`). A farmer that lands on
    /// its cell by a partial step — its last leg still *longer* in Manhattan
    /// than one step, so the snap does not fire, while the sine and cosine
    /// components reach it exactly — keeps the bearing of that step and never
    /// faces the order's angle. run10's AI farmers do
    /// exactly that on frame 116 and the human's do not on 110
    /// (`docs/ORDERS.md` §4.5, `docs/SYNC.md` §3.12).
    pub(super) fn arrive(&mut self, u: usize, mo: MoveOrder, snapped_in: bool) {
        let only = self.units[u].orders.len() == 1;
        let gather_beneath = snapped_in
            && self
                .action_of(u)
                .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Gather(_)));
        if only || gather_beneath {
            // `set_angle(mo->angle, …, 0)` — the heading, and guy 0's
            // `des_angle` with it. The facing does not snap: the body turns
            // toward the order's angle while it stands there.
            self.unit_set_angle(u, mo.angle);
        }
        if let Some(front) = self.units[u].orders.front_mut() {
            front.flags &= !flag::PATHED;
        }
        self.kill_current_order(u);
    }

    // ------------------------------------------------------------------
    // Build, repair, garrison — §5
    // ------------------------------------------------------------------

    /// `Object::adjacent_to@00651f40` (vslot `+0x170`), through
    /// [`adjacent_steps`]: `attack_dist < 0x60`, the footprint's extent for
    /// the building and `block_radius + 0x18` for the unit — or, for a sea
    /// unit that is no transport, `attack_dist <
    /// BOAT_GARRISON_MAX_DISTANCE`, three tiles and `0x180` more for a
    /// `coll_size` under 4 (`651f7f`–`651fef`). Both slots' in-use bits
    /// first (`651f46`, `651f70`), as `in_range_from` reads them.
    pub fn adjacent_to(&self, u: usize, b: usize) -> bool {
        let bd = &self.buildings[b];
        let unit = &self.units[u];
        let dist = |_boat: bool| {
            let t_ext = match bd.ty {
                Some(ty) => (
                    self.tables.build_types[ty].x_size * HALF_TILE,
                    self.tables.build_types[ty].y_size * HALF_TILE,
                ),
                None => combat::extent(&bd.combat.unwrap_or_default(), true),
            };
            let a_ext = combat::extent(&self.profile(Obj::Unit(u)), false);
            combat::attack_dist(unit.pos, bd.pos, a_ext, t_ext, false)
        };
        adjacent_steps(
            unit.alive(),
            bd.alive,
            unit.kind.domain == crate::territory::attrition::Domain::Sea,
            // `+0x18` on a `Unit` is `return 1` (`0041e0e0`).
            || true,
            || {
                let transport = unit
                    .ty
                    .is_some_and(|t| self.tables.unit_types[t].cols.flag(uflags::FLAG_TRANSPORT));
                (transport, self.coll_size(u))
            },
            self.tables.tuning.boat_garrison_max_distance,
            dist,
        )
    }

    /// `ObjectsData::find_any_building_at(t, …, who, FILTER_ALL)`
    /// (`00659180`): the building whose footprint covers the tile, over
    /// the chains of the nine cells round it. A candidate is a player's
    /// (`owner < 8`), in use, and **started, or `who`'s own** (vslot
    /// `+0x50`, `is_started`): another player's placed site is not found.
    /// Footprints do not overlap, so the walk's order does not choose.
    pub(crate) fn find_any_building_at(&self, t: Pos, who: Player) -> Option<usize> {
        (0..self.buildings.len()).find(|&b| {
            let bd = &self.buildings[b];
            bd.alive
                && bd.owner < crate::world::PLAYER_SLOTS
                && (bd.started || bd.owner == who)
                && self.covers_tile(b, t)
        })
    }

    /// `WallData::covers_tile`: the tile is inside the footprint.
    pub fn covers_tile(&self, b: usize, tile: Pos) -> bool {
        let bd = &self.buildings[b];
        let Some(ty) = bd.ty else {
            return bd.pos.tile() == tile;
        };
        let corner = self.tile_corner(ty, bd.pos);
        let (xs, ys) = (
            self.tables.build_types[ty].x_size,
            self.tables.build_types[ty].y_size,
        );
        crate::cities::place::footprint_covers(corner, xs, ys, tile)
    }

    /// `UnitType::find_nearby_spot(x, y, min, max, step, angle, …)`
    /// (`docs/ORDERS.md` §10) under the **pairwise** filter — the
    /// `FILTER_NOT_ME` every build, repair, gather, garrison and stable
    /// call site passes.
    ///
    /// Rings of radius `min, min + step, …, max` around the centre, each
    /// swept over 31 bearings in the order `0, ±1, ±2, … ±7` sixteenths of
    /// a turn from `angle`, then the odd thirty-seconds (`17/32` twice,
    /// `1/32` never, the direct opposite never); each projected point
    /// snapped to its quarter-tile centre; the first candidate on the map,
    /// not on a blocked tile, not on `footprint_of`'s footprint (a farm
    /// excepted for a citizen), of the unit's terrain class and **free of
    /// other units** wins. `max <= 0` and `step <= 0` take the defaults.
    /// No RNG.
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
    ) -> Option<Pos> {
        self.find_nearby_spot_coll(
            u,
            centre,
            min,
            max,
            step,
            angle,
            footprint_of,
            Coll::Pairwise,
        )
    }

    /// The same sweep with the collision half named. `Coll::None` is the
    /// original's `nocoll != 0` — and, as a stated seam, its whole
    /// general path (`docs/ORDERS.md` §10, §14).
    #[allow(clippy::too_many_arguments)]
    pub fn find_nearby_spot_coll(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
        coll: Coll,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Unit(u),
            centre,
            min,
            max,
            step,
            angle,
            footprint_of,
            coll,
            false,
        )
    }

    /// The same sweep with the original's **`uber_unit`** argument set —
    /// `Unit::go_to@005f7a50`'s call, and the only one in this crate
    /// (`docs/ARMY.md` §4.3).
    ///
    /// A squad placement changes two things at once (`docs/ORDERS.md`
    /// §10): the collision block grows by half the formation's span, and
    /// the pairwise pair is not asked at all — the general radius query
    /// and its ordered twin are.
    pub fn find_nearby_spot_squad(
        &self,
        u: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Unit(u),
            centre,
            min,
            max,
            step,
            angle,
            None,
            Coll::Pairwise,
            true,
        )
    }

    /// The sweep asked by a **type** on a unit's behalf ([`Seeker::TypeFor`]):
    /// `UnitType::find_nearby_spot(…, FILTER_NOT_ME, o, who, 0, …)` with
    /// no squad argument, whose collision half is `find_collision` and
    /// `find_ordered_collision` against `(o, who)` (`bVar17`, written at `61df5f`–`61df7c`).
    /// run346's six decoy squads on 11637 stand where the General's pair
    /// allows, not the copied type's radius query (item 1302).
    pub(crate) fn find_nearby_spot_type_for(
        &self,
        ty: usize,
        me: usize,
        centre: Pos,
        min: i32,
        max: i32,
        angle: Angle,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::TypeFor(ty, me),
            centre,
            min,
            max,
            0,
            angle,
            None,
            Coll::Pairwise,
            false,
        )
    }

    /// The same sweep asked by a **type** rather than by a unit — the
    /// original's `not_o`/`not_who` of `(-1, -1)`, which is what
    /// `Unit::do_cast` and `SpellType::cast_transport` pass when they look
    /// for the water a barge is born on (`docs/TRANSPORT.md` §6).
    ///
    /// Nothing is exempt from the collision half, and the block, the domain
    /// and the radius defaults are the type's.
    pub fn find_nearby_spot_type(
        &self,
        ty: usize,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
    ) -> Option<Pos> {
        self.spot_sweep(
            Seeker::Type(ty),
            centre,
            min,
            max,
            step,
            angle,
            None,
            Coll::Pairwise,
            false,
        )
    }

    /// `is(AIRCRAFTCARRIER, 1)` on a unit type — strict, so the type
    /// itself or its graft, never a lineage by `from`. A type the tree
    /// does not carry answers no.
    pub(crate) fn is_aircraft_carrier(&self, ty: usize) -> bool {
        let Some(t) = self.tables.unit_types.get(ty).and_then(|u| u.tree) else {
            return false;
        };
        self.tables
            .tech_tree
            .types
            .iter()
            .position(|d| d.kind.is_unit() && d.name.eq_ignore_ascii_case("Aircraft Carrier"))
            .is_some_and(|root| self.tables.tech_tree.is(t, root, true))
    }

    #[allow(clippy::too_many_arguments)] // as `find_nearby_spot_coll`
    pub(super) fn spot_sweep(
        &self,
        who: Seeker,
        centre: Pos,
        min: i32,
        max: i32,
        step: i32,
        angle: Angle,
        footprint_of: Option<usize>,
        coll: Coll,
        uber: bool,
    ) -> Option<Pos> {
        let p = match who {
            Seeker::Unit(u) => self.profile(Obj::Unit(u)),
            Seeker::Type(t) | Seeker::TypeFor(t, _) => self.tables.unit_types[t].combat,
        };
        let farm_ok = footprint_of.is_some_and(|b| self.building_ident(b) == Ident::Farm)
            && matches!(who, Seeker::Unit(u) if self.worker_of(u) == Worker::Citizen);
        // `unit_flags & 0x10`, the transport-capable flag, which this
        // crate does not model: assumed clear (the ring's comment).
        spot_ring(
            &SimSpot(self),
            who,
            &p,
            false,
            centre,
            (min, max, step),
            angle,
            footprint_of.map(|b| (b, farm_ok)),
            coll,
            uber,
        )
    }
}

/// [`Sim`] behind [`crate::movement::find_path_view::FindPathView`].
pub(super) struct SimFindPath<'a> {
    pub(super) sim: &'a mut Sim,
    pub(super) u: usize,
    pub(super) mo: &'a mut MoveOrder,
}

impl crate::movement::find_path_view::FindPathView for SimFindPath<'_> {
    fn helicopter(&mut self) -> bool {
        self.sim.is_helicopter(self.u)
    }
    fn here(&mut self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn invalid_loc(&mut self, at: Pos, ignore_buildings: bool, fog_relax: bool) -> bool {
        self.sim.invalid_loc(
            self.u,
            at.tile(),
            ignore_buildings,
            fog_relax,
            false,
            false,
            false,
        ) != 0
    }
    fn relocate(&mut self) {
        let here = self.sim.units[self.u].pos;
        let spot = self
            .sim
            .find_nearby_spot(self.u, here, 0, -1, 0, Angle(0x5555_5555), None)
            .unwrap_or(here);
        self.sim.set_new_location(self.u, spot, true);
    }
    fn order_begin(&mut self) {}
    fn cells_apart(&mut self, to: Pos) -> i32 {
        let here = self.sim.units[self.u].pos;
        (to.cell().x - here.cell().x).abs() + (to.cell().y - here.cell().y).abs()
    }
    fn speed(&mut self) -> i32 {
        // `find_path`'s own `(*+0x17c)(x, y, 0)`, the same virtual the step
        // takes ([`Sim::get_speed`]).
        self.sim.get_speed(self.u, 0)
    }
    fn angle(&mut self, dx: i32, dy: i32) -> i32 {
        find_angle(dx, dy).0
    }
    fn sin(&mut self, angle: i32, distance: i32) -> i32 {
        movement::sin_component(Angle(angle), distance)
    }
    fn cos(&mut self, angle: i32, distance: i32) -> i32 {
        movement::cos_component(Angle(angle), distance)
    }
    fn bump_recursion(&mut self) {
        let r = &mut self.sim.units[self.u].path_recursion;
        *r = r.saturating_add(1);
    }
    fn recursion(&mut self) -> u8 {
        self.sim.units[self.u].path_recursion
    }
    fn waypoint(&mut self) -> Pos {
        self.mo.waypoint
    }
    fn set_waypoint(&mut self, to: Pos) {
        self.mo.waypoint = to;
    }
    fn last_plan(&mut self) -> Pos {
        self.mo.last.unwrap_or(Pos::new(-1, -1))
    }
    fn set_last_plan(&mut self, to: Pos) {
        self.mo.last = Some(to);
    }
    fn path_len(&mut self) -> i32 {
        self.sim.units[self.u].path.len() as i32
    }
    fn path_pop(&mut self) -> Option<crate::movement::find_path_view::PathEntry> {
        self.sim.units[self.u]
            .path
            .pop()
            .map(|e| crate::movement::find_path_view::PathEntry {
                to: e.to,
                tolerance: e.tolerance,
                flags: u32::from(e.flags),
            })
    }
    fn path_push(&mut self, e: crate::movement::find_path_view::PathEntry) {
        self.sim.units[self.u].path.push(PathData {
            to: e.to,
            tolerance: e.tolerance,
            flags: e.flags as u8,
        });
    }
    fn path_peek(&mut self) -> crate::movement::find_path_view::PathEntry {
        let e = self.sim.units[self.u]
            .path
            .last()
            .copied()
            .unwrap_or(PathData {
                to: Pos::new(0, 0),
                tolerance: 0,
                flags: 0,
            });
        crate::movement::find_path_view::PathEntry {
            to: e.to,
            tolerance: e.tolerance,
            flags: u32::from(e.flags),
        }
    }
    fn set_avoid(&mut self, at: Pos) {
        self.sim.units[self.u].avoid = Some(at);
    }
    fn domain(&mut self) -> i32 {
        self.sim.units[self.u]
            .ty
            .map_or(0, |t| self.sim.tables.unit_types[t].combat.domain as i32)
    }
    fn go_around(&mut self, at: Pos, angle: i32, speed: i32) {
        self.sim
            .go_around_building(self.u, self.mo, at, Angle(angle), speed);
    }
    fn recurse(&mut self, to: Pos) -> i32 {
        i32::from(self.sim.find_path(self.u, self.mo, to))
    }
    fn line_ok(&mut self) -> bool {
        self.sim.units[self.u].line_ok
    }
    fn set_line_ok(&mut self) {
        self.sim.units[self.u].line_ok = true;
    }
    fn shore_between(&mut self, a: Pos, b: Pos) -> bool {
        self.sim.world.is_tocean(a.tile()) != self.sim.world.is_tocean(b.tile())
    }
}

/// **`Group::kill_group_move@007123f0`'s walk** (item 2287): `normalize`
/// first, then each of the group's `+0xc` members (the shorts at
/// `+0x8cc`) that is in use (vslot `+8`) and, asked only then, on the map
/// (vslot `+0xbc`) is handed to `each` — the unit's own
/// `kill_group_move(id)`.
pub(crate) fn group_member_walk<C>(
    cx: &mut C,
    n: usize,
    normalize: impl FnOnce(&mut C),
    member: impl Fn(&mut C, usize) -> usize,
    mut in_use: impl FnMut(&mut C, usize) -> bool,
    mut on_map: impl FnMut(&mut C, usize) -> bool,
    mut each: impl FnMut(&mut C, usize),
) {
    normalize(cx);
    for k in 0..n {
        let u = member(cx, k);
        if in_use(cx, u) && on_map(cx, u) {
            each(cx, u);
        }
    }
}
