//! `SpellType::cast_transport@00670db0` as steps over a view (item 3330):
//! a land unit at the shore is put into a boat the cast builds. The steps
//! are the original's control flow, call for call and in its order;
//! [`TransportView`] is the state it reads and the callees it makes, which
//! [`SimTransport`] answers from the crate's [`Sim`] and
//! `sweep/spell_type_cast_transport.rs` from the emulated original's windows.
//!
//! ```text
//! g alive and its vslot +0xbc          else: nothing
//! g's type domain is sea (1) or air (2): nothing
//! base = MERCHANTFLEET when g is a caravan, else TRANSPORTBARGE
//! ty   = current_upgrade(base), or base when negative
//! find_nearby_spot(ty, g's point, board distance, bearing 0x55555555) nonzero:
//!        the console's own cast says "not near ocean"; nothing
//! b = init_unit(who, ty, g's point)    negative: nothing
//! g on screen and the console's: the load sound
//! MARINES: b |= 0x200 (unit_masks2), vslot +0x15c(0), update_speed
//! same_damage(b, g); set_new_location(b, spot, 1, 1); set_angle(b, g's)
//! caravan: b takes g's slot, the caravan list names b, the 0x200 bit follows
//! the order list moves g -> b one at a time; kill_current_order(b)
//! g's path stack inverted, popped onto b (so b's reads as g's did);
//! b |= 0x800000; b's top is popped, its TURN_FIRST bit dropped when its
//!        region is the spot's, and pushed back
//! clear_orders(g); the selection group swaps g for b; replace_hotunit;
//! go_inside(g, b)
//! ```
//!
//! Still seams, in [`SimTransport`]: the sounds, `MARINES` (`unit_masks2`
//! and `update_speed`), the caravan's `0x200` bit, the selection group's
//! swap and `replace_hotunit` — none of them is state this crate keeps.
//! ~~The caravan's slot hand-over~~ is built (item 3370, `docs/AI.md`
//! §286): the slot is [`crate::Unit::caravan`].

use crate::orders::PathData;
use crate::{Pos, Sim};

/// `TypeIndex::TRANSPORTBARGE` and `MERCHANTFLEET`.
const TRANSPORTBARGE: i32 = 0x140;
const MERCHANTFLEET: i32 = 0x13e;
/// `ObjectData::is(MARINES)`.
pub(crate) const MARINES: i32 = 0x77;
/// `S_NOT_NEAR_OCEAN_FOR_TRANSPORT` and `S_TRANSPORTLOAD`.
pub(crate) const S_NOT_NEAR_OCEAN: i32 = 0x4e;
pub(crate) const S_TRANSPORTLOAD: i32 = 0x133;
/// `PathData::flags & TURN_FIRST` (`0xfffffffb` clears it).
const TURN_FIRST: i32 = 4;

/// What `cast_transport` reads and does; `g` is the caster, `b` the boat.
pub trait TransportView {
    /// The object's vslot `+0x8`.
    fn alive(&mut self, g: i32) -> bool;
    /// The object's vslot `+0xbc`.
    fn on_map(&mut self, g: i32) -> bool;
    /// The type's `+0x218`: 0 land, 1 sea, 2 air.
    fn domain(&self, g: i32) -> i32;
    /// `UnitData::is_caravan`, at the unit's vslot `+0xd0` or the type's `+0x130`.
    fn is_caravan(&mut self, g: i32) -> bool;
    /// `LeaderData::current_upgrade(base)` of the caster's leader.
    fn current_upgrade(&mut self, base: i32) -> i32;
    /// `UnitType::find_nearby_spot@0061de70` of type `ty` at the caster's point.
    fn find_spot(&mut self, g: i32, ty: i32) -> Option<Pos>;
    /// `Objects::init_unit@0065e0c0(who, ty, g's point, -1, -1, -1)`.
    fn init_unit(&mut self, who: i32, ty: i32, g: i32) -> i32;
    /// The console's player (`console->who`).
    fn console_who(&self) -> i32;
    /// The caster's `is_on_screen` (vslot `+0x70`).
    fn on_screen(&mut self, g: i32) -> bool;
    /// The caster's vslot `+0x48(console who, 0)`.
    fn console_sees(&mut self, g: i32) -> bool;
    /// `SoundGlobal::play`.
    fn sound(&mut self, id: i32);
    /// `ObjectData::is(ty, 0)` of the caster.
    fn is(&mut self, g: i32, ty: i32) -> bool;
    /// `unit_masks2 |= 0x200`, vslot `+0x15c(0)` and `update_speed` of `b`.
    fn marines(&mut self, b: i32);
    /// `Unit::same_damage(b; g, who)`.
    fn same_damage(&mut self, b: i32, g: i32, who: i32);
    /// `Unit::set_new_location(b; x, y, 1, 1)`.
    fn set_new_location(&mut self, g: i32, b: i32, to: Pos);
    /// The caster's `+0x50`.
    fn heading(&self, g: i32) -> i32;
    /// `Unit::set_angle(b; angle, 1)`.
    fn set_angle(&mut self, b: i32, angle: i32);
    /// The short at `+0x86`.
    fn caravan_slot(&self, o: i32) -> i32;
    fn set_caravan_slot(&mut self, o: i32, s: i32);
    /// The caravan list of `who`: its entry `s` names `b` (a short at `+0xa`).
    fn set_caravan_unit(&mut self, who: i32, s: i32, b: i32);
    /// `unit_masks2 & 0x200`.
    fn has_200(&self, o: i32) -> bool;
    fn set_200(&mut self, o: i32, on: bool);
    /// The order list's count (`+0xd8`).
    fn orders_left(&self, g: i32) -> i32;
    /// The list's tail pointer (`+0xdc`) is set: the current node is re-read.
    fn orders_tail(&self, g: i32) -> bool;
    /// The re-read of the current node (`+0xd4`, `+0xcc`, `+0xd0`).
    fn orders_sync(&mut self, g: i32);
    /// The current order (`+0xcc`).
    fn order_head(&mut self, g: i32) -> i32;
    /// `LinkListBase::remove_current`.
    fn order_remove(&mut self, g: i32);
    /// `LinkListBase::add(order)`.
    fn order_add(&mut self, b: i32, order: i32);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self, b: i32);
    /// `Stack<PathData>::invert`.
    fn path_invert(&mut self, g: i32);
    /// The stack's count (`+0xc0`).
    fn path_len(&self, o: i32) -> i32;
    /// The inline pop: the count clamped to one and decremented, the record read.
    fn path_take(&mut self, g: i32) -> [i32; 4];
    /// The inline push: `increase_size` when full, the record written, the count raised.
    fn path_put(&mut self, b: i32, pd: [i32; 4]);
    /// `unit_masks |= 0x800000`.
    fn mark_auto_transport(&mut self, b: i32);
    /// `Stack<PathData>::pop`.
    fn path_pop(&mut self, b: i32) -> [i32; 4];
    /// `div_3_table[k]`.
    fn div3(&self, k: i32) -> i32;
    /// `WorldData::get_tregion(&x, &y)` on tile coordinates.
    fn tregion(&mut self, x: i32, y: i32) -> i32;
    /// `Stack<PathData>::push`.
    fn path_push(&mut self, b: i32, pd: [i32; 4]);
    /// `Unit::clear_orders`.
    fn clear_orders(&mut self, g: i32);
    /// `GroupData::member(g, who, 1)` of the console's selection.
    fn member(&mut self, g: i32, who: i32) -> bool;
    /// The group's vslots `+0x10(g, who, 0, 0)` and `+0x18(b, who, 0)`.
    fn group_swap(&mut self, g: i32, b: i32, who: i32);
    /// `Object::replace_hotunit(b, who)` of the caster.
    fn replace_hotunit(&mut self, g: i32, b: i32, who: i32);
    /// `Unit::go_inside(g; b, who, 0)`.
    fn go_inside(&mut self, g: i32, b: i32, who: i32);
}

/// The original's body over a view.
pub fn cast_transport_steps<V: TransportView>(v: &mut V, g: i32, who: i32) {
    if !v.alive(g) || !v.on_map(g) {
        return;
    }
    let domain = v.domain(g);
    if domain == 2 || domain == 1 {
        return;
    }
    let base = if v.is_caravan(g) {
        MERCHANTFLEET
    } else {
        TRANSPORTBARGE
    };
    let mut ty = v.current_upgrade(base);
    if ty < 0 {
        ty = base;
    }
    let Some(spot) = v.find_spot(g, ty) else {
        if who == v.console_who() {
            v.sound(S_NOT_NEAR_OCEAN);
        }
        return;
    };
    let b = v.init_unit(who, ty, g);
    if b < 0 {
        return;
    }
    if v.on_screen(g) && v.console_sees(g) {
        v.sound(S_TRANSPORTLOAD);
    }
    if v.is(g, MARINES) {
        v.marines(b);
    }
    v.same_damage(b, g, who);
    v.set_new_location(g, b, spot);
    let angle = v.heading(g);
    v.set_angle(b, angle);
    if v.is_caravan(g) {
        let s = v.caravan_slot(g);
        v.set_caravan_slot(b, s);
        v.set_caravan_unit(who, s, b);
        let on = v.has_200(g);
        v.set_200(b, on);
    }
    while v.orders_left(g) != 0 {
        if v.orders_tail(g) {
            v.orders_sync(g);
        }
        let head = v.order_head(g);
        v.order_remove(g);
        v.order_add(b, head);
    }
    v.kill_current_order(b);
    v.path_invert(g);
    while v.path_len(g) != 0 {
        let pd = v.path_take(g);
        v.path_put(b, pd);
    }
    v.mark_auto_transport(b);
    if v.path_len(b) != 0 {
        let mut pd = v.path_pop(b);
        let flags = pd[3];
        let (tx, ty_) = (v.div3(pd[0] >> 6), v.div3(pd[1] >> 6));
        let (sx, sy) = (v.div3(spot.x >> 6), v.div3(spot.y >> 6));
        let there = v.tregion(tx, ty_);
        let here = v.tregion(sx, sy);
        if here == there {
            pd[3] = flags & !TURN_FIRST;
        }
        v.path_push(b, pd);
    }
    v.clear_orders(g);
    if v.member(g, who) {
        v.group_swap(g, b, who);
    }
    v.replace_hotunit(g, b, who);
    v.go_inside(g, b, who);
}

/// The crate's [`Sim`] as the view: `g` and `b` are unit ids.
pub(crate) struct SimTransport<'a> {
    pub(crate) sim: &'a mut Sim,
    who: usize,
    /// The order in hand between `order_head` and `order_add`.
    held: Option<crate::orders::Order>,
}

impl<'a> SimTransport<'a> {
    pub(crate) fn new(sim: &'a mut Sim, who: usize) -> Self {
        SimTransport {
            sim,
            who,
            held: None,
        }
    }

    /// The unit-type index a type id names.
    fn index_of(&self, id: i32) -> Option<usize> {
        let id = usize::try_from(id).ok()?;
        self.sim.unit_types.iter().position(|t| t.tree == Some(id))
    }
}

fn pd_words(p: &PathData) -> [i32; 4] {
    [p.to.x, p.to.y, p.tolerance, i32::from(p.flags)]
}

fn pd_from(w: [i32; 4]) -> PathData {
    PathData {
        to: Pos::new(w[0], w[1]),
        tolerance: w[2],
        flags: w[3] as u8,
    }
}

impl TransportView for SimTransport<'_> {
    fn alive(&mut self, g: i32) -> bool {
        self.sim.units[g as usize].alive()
    }
    fn on_map(&mut self, g: i32) -> bool {
        self.sim.units[g as usize].on_map
    }
    fn domain(&self, g: i32) -> i32 {
        match self.sim.unit_domain_of(g as usize) {
            crate::attrition::Domain::Land => 0,
            crate::attrition::Domain::Sea => 1,
            crate::attrition::Domain::Air => 2,
        }
    }
    fn is_caravan(&mut self, g: i32) -> bool {
        self.sim.unit_is(g as usize, crate::transport::ty::CARAVAN)
    }
    fn current_upgrade(&mut self, base: i32) -> i32 {
        self.sim.current_upgrade_id(self.who, base as usize) as i32
    }
    fn find_spot(&mut self, g: i32, ty: i32) -> Option<Pos> {
        let ty = self.index_of(ty)?;
        let at = self.sim.units[g as usize].pos;
        self.sim.find_nearby_spot_type(
            ty,
            at,
            0,
            self.sim.tuning.unit_board_distance,
            0,
            crate::transport::BOARD_BEARING,
        )
    }
    fn init_unit(&mut self, who: i32, ty: i32, g: i32) -> i32 {
        let Some(ty) = self.index_of(ty) else {
            return -1;
        };
        let at = self.sim.units[g as usize].pos;
        self.sim
            .transport_boat(who as crate::Player, ty, at)
            .map_or(-1, |b| b as i32)
    }
    fn console_who(&self) -> i32 {
        -1
    }
    fn on_screen(&mut self, _g: i32) -> bool {
        false
    }
    fn console_sees(&mut self, _g: i32) -> bool {
        false
    }
    fn sound(&mut self, _id: i32) {}
    fn is(&mut self, _g: i32, _ty: i32) -> bool {
        false
    }
    fn marines(&mut self, _b: i32) {}
    fn same_damage(&mut self, b: i32, g: i32, _who: i32) {
        self.sim.same_damage(b as usize, g as usize);
    }
    fn set_new_location(&mut self, g: i32, b: i32, to: Pos) {
        // The boat is born on the caster's land tile and walked to the
        // water; the move onto the water then reveals (`docs/VISION.md` §11).
        let at = self.sim.units[g as usize].pos;
        self.sim.set_new_location(b as usize, to, true);
        self.sim.moved_to(b as usize, at, false);
    }
    fn heading(&self, g: i32) -> i32 {
        self.sim.units[g as usize].movement.heading.0
    }
    fn set_angle(&mut self, b: i32, angle: i32) {
        self.sim.units[b as usize]
            .movement
            .set_facing(crate::movement::Angle(angle));
    }
    // **The caravan's slot goes with it** (`cast_transport@00670db0:117`–
    // `121`, item 3370): the Merchant Fleet's `+0x86` is the caravan's, so
    // the fleet is the route's walker at sea — its `do_move` verifies the
    // route's road and its `do_trade` runs on the route's record. The
    // caravan keeps its own copy; `eject_contents` hands it back
    // ([`Sim::disembark`]).
    fn caravan_slot(&self, o: i32) -> i32 {
        self.sim.units[o as usize].caravan.map_or(-1, |v| v as i32)
    }
    fn set_caravan_slot(&mut self, o: i32, s: i32) {
        self.sim.units[o as usize].caravan = usize::try_from(s).ok();
    }
    fn set_caravan_unit(&mut self, who: i32, s: i32, b: i32) {
        if let Some(van) = usize::try_from(s)
            .ok()
            .and_then(|s| self.sim.caravans[who as usize].slots.get_mut(s))
        {
            van.unit = Some(b as usize);
        }
    }
    fn has_200(&self, _o: i32) -> bool {
        false
    }
    fn set_200(&mut self, _o: i32, _on: bool) {}
    fn orders_left(&self, g: i32) -> i32 {
        self.sim.units[g as usize].orders.len() as i32
    }
    fn orders_tail(&self, _g: i32) -> bool {
        false
    }
    fn orders_sync(&mut self, _g: i32) {}
    fn order_head(&mut self, g: i32) -> i32 {
        self.held = self.sim.units[g as usize].orders.pop_front();
        0
    }
    fn order_remove(&mut self, _g: i32) {}
    fn order_add(&mut self, b: i32, _order: i32) {
        if let Some(o) = self.held.take() {
            self.sim.units[b as usize].orders.push_back(o);
        }
    }
    fn kill_current_order(&mut self, b: i32) {
        self.sim.kill_current_order(b as usize);
    }
    fn path_invert(&mut self, g: i32) {
        self.sim.units[g as usize].path.reverse();
    }
    fn path_len(&self, o: i32) -> i32 {
        self.sim.units[o as usize].path.len() as i32
    }
    fn path_take(&mut self, g: i32) -> [i32; 4] {
        let p = self.sim.units[g as usize].path.pop().expect("a path entry");
        pd_words(&p)
    }
    fn path_put(&mut self, b: i32, pd: [i32; 4]) {
        self.sim.units[b as usize].path.push(pd_from(pd));
    }
    fn mark_auto_transport(&mut self, b: i32) {
        self.sim.units[b as usize].auto_transport = true;
    }
    fn path_pop(&mut self, b: i32) -> [i32; 4] {
        let p = self.sim.units[b as usize].path.pop().expect("a path entry");
        pd_words(&p)
    }
    fn div3(&self, k: i32) -> i32 {
        k.div_euclid(3)
    }
    fn tregion(&mut self, x: i32, y: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(x, y))
            .map_or(-1, i32::from)
    }
    fn path_push(&mut self, b: i32, pd: [i32; 4]) {
        self.sim.units[b as usize].path.push(pd_from(pd));
    }
    fn clear_orders(&mut self, g: i32) {
        self.sim.clear_orders(g as usize);
    }
    fn member(&mut self, _g: i32, _who: i32) -> bool {
        false
    }
    fn group_swap(&mut self, _g: i32, _b: i32, _who: i32) {}
    fn replace_hotunit(&mut self, _g: i32, _b: i32, _who: i32) {}
    fn go_inside(&mut self, g: i32, b: i32, _who: i32) {
        self.sim.board(g as usize, b as usize);
    }
}
