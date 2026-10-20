//! **`Group::action_attack@00712490`'s steps** (item 3362): the group order
//! that sends every member at one target, as a walk over an [`AttackView`].
//! The control flow is the original's, call for call and in its order; the
//! view is the state it reads and the callees it makes, which [`SimAttack`]
//! answers from the crate's [`Sim`] and `sweep/group_action_attack.rs` from
//! the emulated original's windows (`docs/COMBAT.md` §10, §66).
//!
//! ```text
//! head      is_on_map, whom >= 0; the scenario's ignore-orders list (group slot +0x10);
//!           count > 0; group slot +0x14
//! building  group flag +0x49: each member's Build marked (|= 4, +0x7c = whom, +0x81 = who of whom);
//!           the console hears one refusal; the order ends
//! First     (QUEUE_FIRST) set aside the leader's group orders, halt, attack at QUEUE_NEW, put them back
//! leader    find_leader, its position (QUEUE_LAST: get_loc_to); one find_attack_pos when the
//!           leader cannot already shoot, a refusal for a wall the console's group cannot reach
//! capture   an active Build target a capture is open on: check_capture, then a move to it
//! passes    three, by domain; each member: prefilters (alive, orderable, ignore mask), planes
//!           (the strafe order's retarget), a member already attacking this target, then
//!           packing / building / melee-retarget arms
//! ```

use crate::Sim;
use crate::combat::Obj;
use crate::group::Group;
use crate::orders::{MoveKind, QueuePos};
use crate::world::Pos;

/// `QUEUE_FIRST`, `QUEUE_LAST`, `QUEUE_NEW` as the original's `QueuePos` words.
pub(crate) const QUEUE_FIRST: i32 = 0;
pub(crate) const QUEUE_LAST: i32 = 1;
pub(crate) const QUEUE_NEW: i32 = 2;
/// `OrderIndex`: the head order of a member already on the target.
pub(crate) const ORDER_ATTACK: i32 = 10;
/// `OrderIndex::STRAFE`.
pub(crate) const ORDER_STRAFE: i32 = 0x10;
/// The cast ids the packing arms add (`add_cast_order(…, id, QUEUE_NEW, 0)`):
/// `spell::PACK` and `spell::UNPACK`.
pub(crate) const CAST_PACK: i32 = 0x28b;
pub(crate) const CAST_UNPACK: i32 = 0x28c;
/// The refusals the console hears (`loc_str_array_orig + OFFSET`).
pub(crate) const MSG_BUILD_NOT_IN_RANGE: i32 = 0x837c;
pub(crate) const MSG_CANNOT_REACH: i32 = 0x8390;

/// The head of `Group::action_attack`: what it asks before the group's
/// refresh slot (the part of the view the head's own sweep implements).
pub(crate) trait AttackHead {
    /// `GroupData::is_on_map@0070c450`.
    fn is_on_map(&mut self) -> bool;
    /// `ScenarioData::ignore_orders != 0`.
    fn ignore_orders_on(&self) -> bool;
    /// `objects_ignoring_orders[who]`'s count, read again at every step.
    fn ignored_count(&self, who: u8) -> i32;
    /// Its `i`th entry.
    fn ignored_nth(&self, who: u8, i: i32) -> i32;
    /// The group's slot `+0x10(o, who, 0, 0)`.
    fn ignore_call(&mut self, o: i32, who: u8);
    /// The group's player (`+0x4a`).
    fn who(&self) -> u8;
    /// The group's count (`+0xc`).
    fn count(&self) -> i32;
    /// The group's slot `+0x14`.
    fn refresh(&mut self);
}

/// What `Group::action_attack` asks of the group, its members and the target.
/// `H` names a member (the original's short in the group's array); the
/// target is `objects[whom_who][whom]`, and every `t_*` is asked of it.
pub(crate) trait AttackView: AttackHead {
    type H: Copy;
    /// A `get_order` / `get_action` / `update_*` result: an order record.
    type O: Copy;

    /// The group's byte `+0x49`.
    fn building_group(&self) -> bool;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> Self::H;
    /// A member handle from the word a callee answered (`find_leader`).
    fn h_of(&self, x: i32) -> Self::H;
    /// The word a member handle is (`check_capture`'s first argument).
    fn h_idx(&self, h: Self::H) -> i32;
    /// `this->field_0x2c += 1`.
    fn bump_order_num(&mut self);
    /// `MiscAccess::console->who == the group's player`.
    fn console_is_who(&mut self) -> bool;
    /// `String::String(loc_str + id)`, then `add_feedback` and the sound.
    fn feedback(&mut self, id: i32);

    // ---- QUEUE_FIRST: the leader's group orders set aside
    /// `OrderList::OrderList(&list)`.
    fn order_list_new(&mut self);
    /// `Group::set_up_insert(&list)`.
    fn set_up_insert(&mut self);
    /// `Group::action_halt(ignore)`.
    fn halt(&mut self, mask: i32);
    /// `Group::finish_insert(&list)`.
    fn finish_insert(&mut self);
    /// `LinkListBase::clear` of the list.
    fn order_list_clear(&mut self);
    /// The recursion's frame: `down` entering `action_attack(… QUEUE_NEW …)`,
    /// not down on returning (the sweep's stack-address tokens are the
    /// callee's frame's).
    fn nest(&mut self, down: bool);

    // ---- a building group
    /// The member object's slot `+0x20`.
    fn obj_is_build(&mut self, h: Self::H) -> bool;
    /// `Error::report("Buildings and units in a group?  You can't do that.")`,
    /// unless `ignore_always`; its answer is never read here.
    fn report_not_build(&mut self);
    /// The member's `Build` (slot `+0xac`): `|= 4` at `+0x60`, the short
    /// `+0x7c = whom`, the byte `+0x81 = whom_who`.
    fn mark_build(&mut self, h: Self::H, whom: i32, whom_who: i32);
    /// The member's `Build` (slot `+0xac` again), `is_in_range(whom, whom_who,
    /// x, y, x, 0, 0)` with its own position.
    fn build_in_range(&mut self, h: Self::H, whom: i32, whom_who: i32) -> bool;

    // ---- the leader
    /// `GroupData::find_leader@0070ccb0(0)`.
    fn find_leader(&mut self) -> i32;
    /// The member object's position (`+0x10`, `+0x14`, `^ 0x63637`).
    fn obj_pos(&mut self, h: Self::H) -> (i32, i32);
    /// `GroupData::get_loc_to@0070c5d0(&x, &y, objects)`: the new `(x, y)`.
    fn get_loc_to(&mut self, x: i32, y: i32) -> (i32, i32);
    /// `ObjectData::is_in_range@006486b0` of the leader's unit: `(whom,
    /// whom_who, x, y, &units[h], 0, 0)`.
    fn lead_in_range(&mut self, h: Self::H, whom: i32, whom_who: i32, x: i32, y: i32) -> bool;
    /// `Unit::find_attack_pos@00601280(whom, whom_who, 1, &, &, x, y, 1)`.
    fn find_attack_pos(&mut self, h: Self::H, whom: i32, whom_who: i32, x: i32, y: i32) -> bool;

    // ---- the target (`objects[whom_who][whom]`)
    /// The target's slot `+0x10`.
    fn t_slot10(&mut self) -> bool;
    /// The target's slot `+0x18` (an empty call).
    fn t_slot18(&mut self);
    /// The target's slot `+0x20`.
    fn t_is_build(&mut self) -> bool;
    /// `ObjectData::is_active_build@0046fb40`.
    fn t_is_active_build(&mut self) -> bool;
    /// `BuildData::check_capture_eligible@0062d1d0` of its slot-`+0xac` data.
    fn t_capture_eligible(&mut self) -> bool;
    /// `Build::check_capture@006276a0(leader, who)` on its slot-`+0xac` data.
    fn t_check_capture(&mut self, leader: i32, who: u8);
    /// The target's slot `+0x48(who, 0)`.
    fn t_slot48(&mut self, who: u8) -> i32;
    /// The target's position.
    fn t_pos(&mut self) -> (i32, i32);
    /// The target's slot `+0x1c` (a wall).
    fn t_slot1c(&mut self) -> bool;
    /// `Group::action_move_to@0070fba0(x, y, queue, 0, 0, MOVE_TO, 1, -1, -1, 0)`.
    fn action_move_to(&mut self, x: i32, y: i32, queue: i32);

    // ---- a member
    /// The member object's slot `+0x8`.
    fn o_alive(&mut self, h: Self::H) -> bool;
    /// The member object's slot `+0xbc`.
    fn o_orderable(&mut self, h: Self::H) -> bool;
    /// The member's type's slot `+0x10c` (the `ignore & 4` test).
    fn type_slot10c(&mut self, h: Self::H) -> bool;
    /// The member's slot `+0xd4`: `UnitData::is_special@0046cea0` inline
    /// (`type +0x2b8 & 0x10`) or the slot (`ignore & 2`).
    fn u_is_special(&mut self, h: Self::H) -> bool;
    /// The member's slot `+0xb8(0x3a, 0)` (`ignore & 1`).
    fn u_is_spy(&mut self, h: Self::H) -> bool;
    /// The member's slot `+0xc0`: `UnitData::is_plane@0046ce40` inline
    /// (`type +0x218 == 2 && !(type +0x2b4 & 0x20)`) or the slot.
    fn u_is_plane(&mut self, h: Self::H) -> bool;
    /// `UnitData::mana@00609a50 - mana_burn (short +0x96)`, floored at 0.
    fn u_mana_left(&mut self, h: Self::H) -> i32;
    /// `UnitData::order_type@00616e80`.
    fn u_order_type(&mut self, h: Self::H) -> i32;
    /// The strafe order's retarget: `Unit::update_order@006179d0`, its slot
    /// `+0x88`'s record `{+8 = whom, +0xc = whom_who, short +0x10 = the
    /// target's +0x30, +0x3c = 0}`.
    fn strafe_retarget(&mut self, h: Self::H, whom: i32, whom_who: i32);
    /// The member's type's `+0x218` (its domain).
    fn type_domain(&mut self, h: Self::H) -> i32;
    /// `UnitData::get_action@00608450`.
    fn get_action(&mut self, h: Self::H) -> Option<Self::O>;
    /// The order's slot `+0x10`.
    fn ord_type(&mut self, o: Self::O) -> i32;
    /// `Unit::update_action@0060a870`'s slot `+0x50` record:
    /// `(+8, +0xc, byte +0x1c)`.
    fn update_action_rec(&mut self, h: Self::H) -> (i32, i32, i32);
    /// The member's `+0xd8` (orders queued).
    fn u_d8(&mut self, h: Self::H) -> i32;
    /// `ObjectData::is_in_range@00648d70(whom, whom_who, &units[h], 0)`.
    fn in_range4(&mut self, h: Self::H, whom: i32, whom_who: i32) -> bool;
    /// `UnitData::get_order@0060b030`.
    fn get_order(&mut self, h: Self::H) -> Self::O;
    /// The order's slot `+0x14`.
    fn ord_slot14(&mut self, o: Self::O) -> bool;
    /// The order's slot `+0xb8` record: `(+4, +8)`.
    fn ord_b8(&mut self, o: Self::O) -> (i32, i32);
    /// `is_in_range(whom, whom_who, a, b, who * 7, 0, 0)` of the member.
    fn approach_in_range(&mut self, h: Self::H, whom: i32, whom_who: i32, a: i32, b: i32) -> bool;
    /// `UnitData::is_packing_or_unpacking@0060a410`.
    fn u_packing_or_unpacking(&mut self, h: Self::H) -> bool;
    /// `UnitData::is_packing@0060aa60`.
    fn u_is_packing(&mut self, h: Self::H) -> bool;
    /// `Unit::clear_orders@005e3860`.
    fn clear_orders(&mut self, h: Self::H);
    /// `ObjectData::is_in_range(whom, whom_who, x, y, &units[h], 0, 0)` with
    /// the member's own position.
    fn self_in_range(&mut self, h: Self::H, whom: i32, whom_who: i32) -> bool;
    /// `Unit::add_cast_order@005e4a60(-1, -1, -1, -1, id, QUEUE_NEW, 0)`.
    fn add_cast_order(&mut self, h: Self::H, id: i32);
    /// `Unit::add_move_order@00616ed0(x, y, 1, 0, queue, 0, &units[h], -1, -1)`.
    fn add_move_order(&mut self, h: Self::H, x: i32, y: i32, queue: i32);
    /// `Unit::add_attack_order@005e5410(whom, whom_who, queue, mandatory, 1)`.
    fn add_attack_order(
        &mut self,
        h: Self::H,
        whom: i32,
        whom_who: i32,
        queue: i32,
        mandatory: i32,
    );
    /// `Unit::find_melee_target@005ff9c0(maxd, &whom_who, 0, 0, word)`:
    /// `(answer, the out-parameter)`.
    fn find_melee_target(&mut self, h: Self::H, maxd: i32, whom_who: i32, word: i32) -> (i32, i32);
    /// `constants->unit_respond_range`.
    fn respond_range(&mut self) -> i32;
}

/// The head: `false` when the order ends before the group's refresh.
pub(crate) fn attack_head<V: AttackHead>(v: &mut V, whom: i32) -> bool {
    if !v.is_on_map() || whom < 0 {
        return false;
    }
    let who = v.who();
    if v.ignore_orders_on() && who < 8 && v.ignored_count(who) > 0 {
        let mut i = 0;
        loop {
            let o = v.ignored_nth(who, i);
            if o >= 0 {
                v.ignore_call(o, who);
            }
            i += 1;
            if i >= v.ignored_count(who) {
                break;
            }
        }
    }
    if v.count() <= 0 {
        return false;
    }
    v.refresh();
    true
}

/// `Group::action_attack@00712490(whom, whom_who, mandatory, queue, ignore)`.
pub(crate) fn action_attack_steps<V: AttackView>(
    v: &mut V,
    whom: i32,
    whom_who: i32,
    mandatory: i32,
    queue: i32,
    mask: i32,
) {
    if !attack_head(v, whom) {
        return;
    }
    if v.building_group() {
        let mut flagged = false;
        let mut i = 0;
        loop {
            let h = v.member(i);
            if !v.obj_is_build(h) {
                v.report_not_build();
            } else {
                v.mark_build(h, whom, whom_who);
                if v.console_is_who() && !flagged && !v.build_in_range(h, whom, whom_who) {
                    v.feedback(MSG_BUILD_NOT_IN_RANGE);
                    flagged = true;
                }
            }
            i += 1;
            if i >= v.count() {
                break;
            }
        }
        return;
    }
    if queue == QUEUE_FIRST {
        v.order_list_new();
        v.set_up_insert();
        v.halt(mask);
        v.nest(true);
        action_attack_steps(v, whom, whom_who, mandatory, QUEUE_NEW, mask);
        v.nest(false);
        v.finish_insert();
        v.order_list_clear();
        return;
    }
    if v.count() <= 0 {
        return;
    }
    let l1 = v.find_leader();
    if l1 < 0 {
        return;
    }
    let who = v.who();
    let lh = v.h_of(l1);
    let (mut x, mut y) = v.obj_pos(lh);
    if queue == QUEUE_LAST {
        (x, y) = v.get_loc_to(x, y);
    }
    let l2 = if v.count() < 1 { -1 } else { v.find_leader() };
    let l2h = v.h_of(l2);
    if !v.lead_in_range(l2h, whom, whom_who, x, y)
        && !v.find_attack_pos(l2h, whom, whom_who, x, y)
        && v.t_slot10()
        && v.console_is_who()
    {
        v.feedback(MSG_CANNOT_REACH);
    }
    v.t_slot18();
    if v.t_is_build() && v.t_is_active_build() && v.t_capture_eligible() {
        v.t_check_capture(l1, who);
        let (tx, ty) = v.t_pos();
        v.action_move_to(tx, ty, queue);
        return;
    }
    let goes = v.t_slot48(who) == 0;
    for pass in 0..3 {
        let mut i = 0;
        loop {
            let h = v.member(i);
            member_visit(
                v, h, whom, whom_who, mandatory, queue, mask, pass, goes, who,
            );
            i += 1;
            if i >= v.count() {
                break;
            }
        }
    }
    v.bump_order_num();
}

#[allow(clippy::too_many_arguments)]
fn member_visit<V: AttackView>(
    v: &mut V,
    h: V::H,
    whom: i32,
    wwho: i32,
    mandatory: i32,
    queue: i32,
    mask: i32,
    pass: i32,
    goes: bool,
    who: u8,
) {
    if !v.o_alive(h) || !v.o_orderable(h) {
        return;
    }
    if mask & 4 != 0 && v.type_slot10c(h) {
        return;
    }
    if mask & 2 != 0 && v.u_is_special(h) {
        return;
    }
    if mask & 1 != 0 && v.u_is_spy(h) {
        return;
    }
    if v.u_is_plane(h) {
        // A plane is never ordered; a strafing one with mana left is
        // pointed at the new target instead.
        if v.u_mana_left(h) != 0 && v.u_order_type(h) == ORDER_STRAFE {
            v.strafe_retarget(h, whom, wwho);
        }
        return;
    }
    if v.type_domain(h) != pass {
        return;
    }
    if let Some(a) = v.get_action(h)
        && v.ord_type(a) == ORDER_ATTACK
    {
        let (rw, rt, rm) = v.update_action_rec(h);
        if rw == whom && rt == wwho && rm != 0 && v.u_d8(h) <= 2 && !v.in_range4(h, whom, wwho) {
            if v.u_d8(h) == 1 {
                return;
            }
            let o = v.get_order(h);
            if v.ord_slot14(o) {
                let o1 = v.get_order(h);
                let o2 = v.get_order(h);
                let (_, b1) = v.ord_b8(o1);
                let (b2, _) = v.ord_b8(o2);
                if v.approach_in_range(h, whom, wwho, b2, b1) {
                    return;
                }
            }
        }
    }
    let packing = v.u_packing_or_unpacking(h);
    if !packing || queue != QUEUE_NEW {
        if v.t_is_build() {
            if !goes {
                v.add_attack_order(h, whom, wwho, queue, mandatory);
            } else {
                if v.in_range4(h, whom, wwho) {
                    let hi = v.h_idx(h);
                    v.t_check_capture(hi, who);
                }
                let (tx, ty) = v.t_pos();
                v.add_move_order(h, tx, ty, queue);
            }
            return;
        }
        if goes {
            let (tx, ty) = v.t_pos();
            v.add_move_order(h, tx, ty, queue);
        }
        let (mut aw, mut at) = (whom, wwho);
        if mandatory == 0 {
            let (ox, oy) = v.obj_pos(h);
            let (tx, ty) = v.t_pos();
            let d = crate::world::vector_dist(
                tx.wrapping_sub(ox).wrapping_abs(),
                ty.wrapping_sub(oy).wrapping_abs(),
            );
            let cap = v.respond_range().wrapping_mul(0x240);
            let maxd = d.wrapping_add(0xc0).min(cap);
            let word = if v.t_slot1c() { 2 } else { 1 };
            let (r, out) = v.find_melee_target(h, maxd, wwho, word);
            if r >= 0 && out >= 0 {
                aw = r;
                at = out;
            }
        }
        v.add_attack_order(h, aw, at, queue, mandatory);
        return;
    }
    // Packing at QUEUE_NEW.
    if !goes {
        let packed = v.u_is_packing(h);
        v.clear_orders(h);
        let inr = v.self_in_range(h, whom, wwho);
        if packed != inr {
            v.add_cast_order(h, if packed { CAST_PACK } else { CAST_UNPACK });
        }
        v.add_attack_order(h, whom, wwho, QUEUE_LAST, mandatory);
    } else {
        if v.t_is_build() && v.in_range4(h, whom, wwho) {
            let hi = v.h_idx(h);
            v.t_check_capture(hi, who);
        }
        if v.u_is_packing(h) {
            v.add_cast_order(h, CAST_PACK);
        }
        let (tx, ty) = v.t_pos();
        v.add_move_order(h, tx, ty, QUEUE_LAST);
    }
}

/// The index bit a building's key carries (the crate keeps units and
/// buildings in two arrays; the original has one object array a player).
const BUILDING_KEY: i32 = 1 << 24;

/// A target's `(whom, whom_who)` words as the steps carry them.
pub(crate) fn obj_key(sim: &Sim, o: Obj) -> (i32, i32) {
    let who = i32::from(sim.owner_of(o));
    match o {
        Obj::Unit(u) => (u as i32, who),
        Obj::Building(b) => (BUILDING_KEY | b as i32, who),
    }
}

fn key_obj(k: i32) -> Obj {
    if k & BUILDING_KEY != 0 {
        Obj::Building((k & !BUILDING_KEY) as usize)
    } else {
        Obj::Unit(k as usize)
    }
}

const fn queue_pos(q: i32) -> QueuePos {
    match q {
        QUEUE_FIRST => QueuePos::First,
        QUEUE_LAST => QueuePos::Last,
        _ => QueuePos::New,
    }
}

/// This crate's [`AttackView`]: `h` is a unit index, the target an
/// [`Obj`]. Where the crate models less than the original: no scenario
/// ignore-orders list, no building groups (`+0x49`), no console (no
/// refusal is heard), no mana (a strafing plane is skipped like any plane),
/// no spy type (`ignore & 1` matches nothing), and the target's slot `+0x10`
/// (a refusal's condition) answers false.
pub(crate) struct SimAttack<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) g: &'a Group,
    pub(crate) target: Obj,
    pub(crate) insert: Option<crate::group::Insert>,
}

impl SimAttack<'_> {
    fn build(&self) -> Option<usize> {
        match self.target {
            Obj::Building(b) => Some(b),
            Obj::Unit(_) => None,
        }
    }
}

impl AttackHead for SimAttack<'_> {
    fn is_on_map(&mut self) -> bool {
        self.sim.group_is_on_map(self.g)
    }
    fn ignore_orders_on(&self) -> bool {
        false
    }
    fn ignored_count(&self, _who: u8) -> i32 {
        0
    }
    fn ignored_nth(&self, _who: u8, _i: i32) -> i32 {
        -1
    }
    fn ignore_call(&mut self, _o: i32, _who: u8) {}
    fn who(&self) -> u8 {
        self.g.who
    }
    fn count(&self) -> i32 {
        self.g.list.len() as i32
    }
    fn refresh(&mut self) {}
}

impl AttackView for SimAttack<'_> {
    type H = usize;
    type O = Option<usize>;

    fn building_group(&self) -> bool {
        false
    }
    fn member(&self, i: i32) -> usize {
        self.g.list[i as usize]
    }
    fn h_of(&self, x: i32) -> usize {
        usize::try_from(x).unwrap_or(0)
    }
    fn h_idx(&self, h: usize) -> i32 {
        h as i32
    }
    fn bump_order_num(&mut self) {
        self.sim.bump_order_num(self.g);
    }
    fn console_is_who(&mut self) -> bool {
        false
    }
    fn feedback(&mut self, _id: i32) {}
    fn order_list_new(&mut self) {}
    fn set_up_insert(&mut self) {
        self.insert = Some(self.sim.group_set_up_insert(self.g));
    }
    fn halt(&mut self, mask: i32) {
        self.sim.group_action_halt(self.g, mask);
    }
    fn finish_insert(&mut self) {
        if let Some(i) = self.insert.take() {
            self.sim.group_finish_insert(self.g, i);
        }
    }
    fn order_list_clear(&mut self) {}
    fn nest(&mut self, _down: bool) {}
    fn obj_is_build(&mut self, _h: usize) -> bool {
        false
    }
    fn report_not_build(&mut self) {}
    fn mark_build(&mut self, _h: usize, _whom: i32, _wwho: i32) {}
    fn build_in_range(&mut self, _h: usize, _whom: i32, _wwho: i32) -> bool {
        true
    }
    fn find_leader(&mut self) -> i32 {
        self.sim.group_find_leader(self.g).map_or(-1, |u| u as i32)
    }
    fn obj_pos(&mut self, h: usize) -> (i32, i32) {
        let p = self.sim.units[h].pos;
        (p.x, p.y)
    }
    fn get_loc_to(&mut self, x: i32, y: i32) -> (i32, i32) {
        let p = self.sim.group_loc_to(self.g).unwrap_or(Pos::new(x, y));
        (p.x, p.y)
    }
    fn lead_in_range(&mut self, h: usize, _whom: i32, _wwho: i32, x: i32, y: i32) -> bool {
        self.sim
            .is_in_range_at(Obj::Unit(h), Pos::new(x, y), self.target)
    }
    fn find_attack_pos(&mut self, h: usize, _whom: i32, _wwho: i32, x: i32, y: i32) -> bool {
        self.sim
            .find_attack_pos(
                h,
                self.target,
                Pos::new(x, y),
                crate::fight::SITE_ATTACK_POS_GROUP,
            )
            .is_some()
    }
    fn t_slot10(&mut self) -> bool {
        false
    }
    fn t_slot18(&mut self) {}
    fn t_is_build(&mut self) -> bool {
        matches!(self.target,
            Obj::Building(b) if self.sim.buildings[b].index < crate::WALL_BASE)
    }
    fn t_is_active_build(&mut self) -> bool {
        self.build()
            .is_some_and(|b| self.sim.buildings[b].alive && self.sim.buildings[b].active)
    }
    fn t_capture_eligible(&mut self) -> bool {
        self.build().is_some_and(|b| self.sim.capture_eligible(b))
    }
    fn t_check_capture(&mut self, leader: i32, _who: u8) {
        if let Some(b) = self.build() {
            let _ = self.sim.check_capture(b, leader as usize);
        }
    }
    fn t_slot48(&mut self, who: u8) -> i32 {
        i32::from(self.sim.target_seen_by(who, self.target))
    }
    fn t_pos(&mut self) -> (i32, i32) {
        let p = self.sim.pos_of(self.target);
        (p.x, p.y)
    }
    fn t_slot1c(&mut self) -> bool {
        matches!(self.target, Obj::Building(_))
    }
    fn action_move_to(&mut self, x: i32, y: i32, queue: i32) {
        self.sim.group_action_move_to(
            self.g,
            Pos::new(x, y),
            queue_pos(queue),
            false,
            crate::movement::Angle(0),
            MoveKind::MoveTo,
            true,
        );
    }
    fn o_alive(&mut self, h: usize) -> bool {
        self.sim.units[h].alive()
    }
    fn o_orderable(&mut self, h: usize) -> bool {
        self.sim.units[h].on_map
    }
    fn type_slot10c(&mut self, h: usize) -> bool {
        self.sim.is_siege_unit(h)
    }
    fn u_is_special(&mut self, h: usize) -> bool {
        self.sim.is_special_unit(h)
    }
    fn u_is_spy(&mut self, _h: usize) -> bool {
        false
    }
    fn u_is_plane(&mut self, h: usize) -> bool {
        self.sim.is_plane(h)
    }
    fn u_mana_left(&mut self, _h: usize) -> i32 {
        0
    }
    fn u_order_type(&mut self, h: usize) -> i32 {
        i32::from(self.sim.order_type(h))
    }
    fn strafe_retarget(&mut self, _h: usize, _whom: i32, _wwho: i32) {}
    fn type_domain(&mut self, h: usize) -> i32 {
        self.sim.group_domain(h) as i32
    }
    fn get_action(&mut self, h: usize) -> Option<Option<usize>> {
        self.sim.action_of(h).map(Some)
    }
    fn ord_type(&mut self, o: Option<usize>) -> i32 {
        o.map_or(-1, |_| ORDER_ATTACK)
    }
    fn update_action_rec(&mut self, h: usize) -> (i32, i32, i32) {
        match self.sim.units[h].combat.target {
            Some(t) => {
                let (w, ww) = obj_key(self.sim, t);
                (w, ww, i32::from(self.sim.units[h].combat.mandatory))
            }
            None => (-2, -2, 0),
        }
    }
    fn u_d8(&mut self, h: usize) -> i32 {
        self.sim.units[h].orders.len() as i32
    }
    fn in_range4(&mut self, h: usize, _whom: i32, _wwho: i32) -> bool {
        self.sim.is_in_range(Obj::Unit(h), self.target)
    }
    fn get_order(&mut self, h: usize) -> Option<usize> {
        self.sim.current_order(h).map(|_| h)
    }
    fn ord_slot14(&mut self, o: Option<usize>) -> bool {
        o.is_some_and(|h| {
            self.sim
                .current_order(h)
                .is_some_and(|o| o.move_dest().is_some())
        })
    }
    fn ord_b8(&mut self, o: Option<usize>) -> (i32, i32) {
        o.and_then(|h| self.sim.current_order(h).and_then(|o| o.move_dest()))
            .map_or((0, 0), |p| (p.x, p.y))
    }
    fn approach_in_range(&mut self, h: usize, _whom: i32, _wwho: i32, a: i32, b: i32) -> bool {
        self.sim
            .is_in_range_at(Obj::Unit(h), Pos::new(a, b), self.target)
    }
    fn u_packing_or_unpacking(&mut self, h: usize) -> bool {
        self.sim.is_packing(h) || self.sim.is_unpacking(h)
    }
    fn u_is_packing(&mut self, h: usize) -> bool {
        self.sim.is_packing(h)
    }
    fn clear_orders(&mut self, h: usize) {
        self.sim.clear_orders(h);
    }
    fn self_in_range(&mut self, h: usize, _whom: i32, _wwho: i32) -> bool {
        let at = self.sim.units[h].pos;
        self.sim.is_in_range_at(Obj::Unit(h), at, self.target)
    }
    fn add_cast_order(&mut self, h: usize, id: i32) {
        self.sim.add_cast_order_at(h, id, QueuePos::New);
    }
    fn add_move_order(&mut self, h: usize, x: i32, y: i32, queue: i32) {
        self.sim
            .add_move_order(h, Pos::new(x, y), MoveKind::MoveTo, queue_pos(queue), false);
    }
    fn add_attack_order(&mut self, h: usize, whom: i32, _wwho: i32, queue: i32, mandatory: i32) {
        let t = if whom == obj_key(self.sim, self.target).0 {
            self.target
        } else {
            key_obj(whom)
        };
        self.sim
            .add_attack_order(h, t, queue_pos(queue), mandatory != 0, true);
    }
    fn find_melee_target(&mut self, h: usize, maxd: i32, wwho: i32, word: i32) -> (i32, i32) {
        let word = if word == 2 {
            crate::fight::search::BUILDINGS
        } else {
            crate::fight::search::UNITS
        };
        let found = match self.sim.melee_squad_head(h) {
            crate::orders::SquadHead::Nothing => None,
            crate::orders::SquadHead::Captain(t, _) => Some(t),
            crate::orders::SquadHead::Search => self.sim.find_melee_target_with(h, maxd, word),
        };
        match found {
            Some(t) => obj_key(self.sim, t),
            None => (-1, wwho),
        }
    }
    fn respond_range(&mut self) -> i32 {
        self.sim.tuning.unit_respond_range
    }
}
