//! `Object::disband@006455c0` as steps over a view (item 3346,
//! `docs/CITIES.md` §9.2): the builders let go, the Lakota raze, the plunder
//! of a captain's kill, the close or the death, and the goods that come back.
//! [`disband_steps`] is the original's control flow in its order;
//! [`DisbandView`] is what it reads and the callees it makes, which
//! [`SimDisband`] answers from the crate's [`Sim`] for a building and
//! `sweep/object_disband.rs` from the emulated original's windows.
//!
//! Every method is **one reach into the original**: a virtual slot of the
//! object, a field of the data record behind its `+0xb0` slot, a callee. The
//! sweep's stubs answer them in the order the original asks.
//!
//! **What the crate's building carries and what it does not.** The original
//! is one function for every object; the crate disbands buildings. So
//! [`SimDisband`] answers the unit arms as the building arm reaches them
//! (never: a building is not a unit, so there is no captain to plunder, no
//! garrison to disband in the sweep's sense, no random draw), and the
//! builders' loop as "no builders to let go" — the crate's builders drop
//! their order in `Unit::work`'s liveness test instead, which is where
//! `docs/CITIES.md` §9.2 puts the same effect. The Lakota message
//! (`console->who == who`) is the original's user interface and the crate
//! has none.

use crate::Sim;
use crate::num::single::Single;

/// The goods the loop walks: `TypeIndex` `BASE_GOODTYPES` up to six.
const GOODS: i32 = 6;

/// What `Object::disband` reads of the object, the world and the leaders,
/// and the callees it makes. Methods that name a unit take its index in the
/// owner's unit table; `Order` is a `UnitOrder` the unit answered.
pub trait DisbandView {
    type Order: Copy;
    /// `this->who` (the byte at `+9`).
    fn who(&mut self) -> i32;
    /// `this->index` (the short at `+0xa`).
    fn index(&mut self) -> i32;
    /// Vslot `0xc`: `+8 & 1`, the record is in use.
    fn in_use(&mut self) -> bool;
    /// Vslot `0x4c`: `+8 & 4`.
    fn active(&mut self) -> bool;
    /// Vslot `0x50`: `+8 & 2`.
    fn started(&mut self) -> bool;
    /// Vslot `0x1c`: a building or a wall.
    fn is_structure(&mut self) -> bool;
    /// Vslot `0x18`: a unit.
    fn is_unit(&mut self) -> bool;
    /// Vslot `0xe8`: `UnitData::is_captain` on a unit.
    fn is_captain(&mut self) -> bool;
    /// Vslot `0x20`: a building that is not a wall.
    fn is_plain_building(&mut self) -> bool;
    /// Vslot `0xbc`: `UnitData::is_on_map` on a unit.
    fn on_map(&mut self) -> bool;
    /// `objects[who][index]->vtable[0xb0]()->[+0x18]->vtable[0x90]()`: the
    /// gate on the builders' loop.
    fn builders_gate(&mut self) -> bool;
    /// `objects->count[who]` (`+0x15c + who * 4`), read every step of the loop.
    fn unit_count(&mut self) -> i32;
    /// `units[who][i]->flags & 1`.
    fn unit_in_use(&mut self, i: i32) -> bool;
    /// `units[who][i]->type->index` (`+0x18` then `+4`).
    fn unit_type(&mut self, i: i32) -> i32;
    /// `UnitData::get_action`: the order, or none.
    fn unit_action(&mut self, i: i32) -> Option<Self::Order>;
    /// The order's vslot `0x10`: its kind.
    fn order_kind(&mut self, o: Self::Order) -> i32;
    /// `Unit::update_action`'s order's vslot `0x6c` record: the target's
    /// index (`+8`) and owner (`+0xc`).
    fn updated_order_target(&mut self, i: i32) -> (i32, i32);
    /// `Unit::clear_orders`.
    fn unit_clear_orders(&mut self, i: i32);
    /// `LeaderData::has_tribe_bonus(n)` of leader `who`.
    fn has_tribe_bonus(&mut self, who: i32, n: i32) -> bool;
    /// `GameAccess::constants->lakota_raze_price`.
    fn lakota_raze_price(&mut self) -> i32;
    /// `MiscAccess::console->who`.
    fn console_who(&mut self) -> i32;
    /// `objects[who][index]->vtable[0xb0]()->vtable[0xa4](who)`: the build time.
    fn time(&mut self) -> u32;
    /// The data's `+0x48`.
    fn job_counter(&mut self) -> u32;
    /// The data's vslot `0x18c` with argument 0: `construct_time(0)`.
    fn construct_time(&mut self) -> i32;
    /// The data's `+0x4c`.
    fn job_counter_2(&mut self) -> i32;
    /// `units[who][index]->+0x68`.
    fn unit_flags68(&mut self) -> i32;
    /// `(char) units[who][index]->+0xa9`, sign-extended: the killer.
    fn unit_killer(&mut self) -> i32;
    /// `(short) units[who][index]->+0xa4`.
    fn unit_killed_obj(&mut self) -> i32;
    /// `leaders[who].leader_flags`.
    fn leader_flags(&mut self, who: i32) -> i32;
    /// `Unit::plunder(units[who][index], a, killer, 1, 0)`.
    fn plunder(&mut self, a: i32, killer: i32, c: i32, d: i32);
    /// `ScenarioData::last_razed[who] = index`.
    fn set_last_razed(&mut self, who: i32, index: i32);
    /// `ObjectData::garrison_count(this, 0)`.
    fn garrison_count(&mut self) -> i32;
    /// The short at `+0x28`: the head of the garrison chain.
    fn inside_head(&mut self) -> i32;
    /// `Object::disband(objects[who][s], 0)` on a garrisoned object.
    fn disband_inside(&mut self, s: i32);
    /// `Random::get(game_random, lo, hi)`.
    fn random(&mut self, lo: i32, hi: i32) -> i32;
    /// `this->vtable[0x158](a, b, c)`: `die`.
    fn die(&mut self, a: i32, b: i32, c: i32);
    /// `this->vtable[0x150](a, b, c)`: `close`.
    fn close(&mut self, a: i32, b: i32, c: i32);
    /// `LeaderData::type_avail(who, good, strict) != 0`.
    fn type_avail(&mut self, who: i32, good: i32, strict: i32) -> bool;
    /// The data's vslot `0xac` record's `+0x6c`: the original type.
    fn orig_type(&mut self) -> i32;
    /// `types[ty]->vtable[0x78](good, who, -1, -1, 0, 1, orig)`: the price
    /// of one good, the seven arguments in the order they are pushed.
    fn price(&mut self, args: [i32; 7]) -> i32;
    /// The leader's encrypted stockpile word for `good`, `+ amount`.
    fn add_good(&mut self, who: i32, good: i32, amount: i32);
    /// The message line for one good with a refund (the Lakota owner's
    /// screen); `first` is the first line built.
    fn message_good(&mut self, good: i32, amount: i32, first: bool);
    /// The message posted once the loop ends, with its sound.
    fn message_end(&mut self);
}

/// `(float) price × (float) (ct − jc2) / (float) ct`, truncated: the partial
/// refund of an unfinished building, in the original's single-precision
/// order (`cvtdq2ps`, `mulss`, `divss`, `cvttss2si`).
pub fn partial_refund(price: i32, ct: i32, jc2: i32) -> i32 {
    Single::from_i32(price)
        .mulss(Single::from_i32(ct.wrapping_sub(jc2)))
        .divss(Single::from_i32(ct))
        .to_i32()
}

/// The original's control flow. `param` is `disband`'s one argument, `full`.
pub fn disband_steps<V: DisbandView>(v: &mut V, param: i32) {
    let mut refund_arm = false;
    let mut message = false;

    // The builders whose BUILD order targets this object let it go.
    if v.in_use() && v.active() && v.builders_gate() {
        let mut i = 0;
        while i < v.unit_count() {
            if v.unit_in_use(i)
                && matches!(v.unit_type(i), 0x32..=0x35)
                && let Some(order) = v.unit_action(i)
                && v.order_kind(order) == 7
            {
                let (target, owner) = v.updated_order_target(i);
                if target == v.index() && owner == v.who() {
                    v.unit_clear_orders(i);
                }
            }
            i += 1;
        }
    }

    // The Lakota raze: everything comes back.
    let mut full = param;
    if v.is_structure() {
        let who = v.who();
        if v.has_tribe_bonus(who, 0x13) && v.lakota_raze_price() != 0 {
            full = 1;
            message = v.console_who() == who;
        }
    }

    let (mut ct, mut jc2) = (0, 0);
    if full == 0 {
        if v.in_use() && !v.active() {
            let time = v.time();
            if v.job_counter() < time {
                refund_arm = true;
                ct = v.construct_time();
                jc2 = v.job_counter_2();
            }
        }
    } else {
        refund_arm = true;
    }

    // A captain's kill is plundered by the killer's tribe.
    if v.is_unit() && v.is_captain() && v.unit_flags68() & 0x4000 != 0 {
        let killer = v.unit_killer();
        if (killer as u32) < 8 && v.leader_flags(killer) & 3 == 3 && v.has_tribe_bonus(killer, 0) {
            let obj = v.unit_killed_obj();
            v.plunder(obj, killer, 1, 0);
        }
    }

    if v.in_use() && v.active() {
        let (who, index) = (v.who(), v.index());
        v.set_last_razed(who, index);
    }

    // Close it, or let it die.
    if v.in_use() && !v.started() {
        v.close(0, -1, 0);
    } else if !v.is_plain_building() && v.on_map() {
        if v.garrison_count() != 0 {
            let mut s = v.inside_head();
            while s >= 0 {
                v.disband_inside(s);
                s = v.inside_head();
            }
        }
        let r = v.random(0, 0xffff);
        v.die(i32::from(((r % 3) as u8).wrapping_add(1)), -1, 0);
    } else {
        v.die(1, -1, 0);
    }

    // The goods that come back.
    if refund_arm {
        let who = v.who();
        let mut built = false;
        for good in 0..GOODS {
            if !v.type_avail(who, good, 1) {
                continue;
            }
            let mut amount = 0;
            if full != 0 {
                amount = v.price([good, who, -1, -1, 0, 1, -1]);
                v.add_good(who, good, amount);
            } else if ct != 0 && jc2 < ct {
                let orig = v.orig_type();
                let price = v.price([good, who, -1, -1, 0, 1, orig]);
                amount = partial_refund(price, ct, jc2);
                v.add_good(who, good, amount);
            }
            if message && amount != 0 {
                v.message_good(good, amount, !built);
                built = true;
            }
        }
        if built && message {
            v.message_end();
        }
    }
}

/// [`DisbandView`] for one of the crate's buildings. **The price is asked
/// when the steps ask it, after the close** (item 3516): `disband@006455c0`
/// reaches `close` (vslot `0x150`, line 181) or `die` (`0x158`, which is
/// `Object::die` and closes through `0x150`) before either price read
/// (`0x78`, lines 211 and 221), so the site no longer counts in its own
/// type's ramp — run1162c's who=1 site `1/2038` (type 422) refunds 160
/// timber and 110 metal, not the 180 and 130 its live count prices.
pub struct SimDisband<'a> {
    pub sim: &'a mut Sim,
    pub b: usize,
}

impl DisbandView for SimDisband<'_> {
    type Order = ();
    fn who(&mut self) -> i32 {
        i32::from(self.sim.buildings[self.b].owner)
    }
    fn index(&mut self) -> i32 {
        self.b as i32
    }
    fn in_use(&mut self) -> bool {
        self.sim.buildings[self.b].alive
    }
    fn active(&mut self) -> bool {
        self.sim.buildings[self.b].active
    }
    fn started(&mut self) -> bool {
        self.sim.buildings[self.b].started
    }
    fn is_structure(&mut self) -> bool {
        true
    }
    fn is_unit(&mut self) -> bool {
        false
    }
    fn is_captain(&mut self) -> bool {
        false
    }
    fn is_plain_building(&mut self) -> bool {
        true
    }
    fn on_map(&mut self) -> bool {
        false
    }
    fn builders_gate(&mut self) -> bool {
        // The builders drop their order in `Unit::work`'s liveness test.
        false
    }
    fn unit_count(&mut self) -> i32 {
        0
    }
    fn unit_in_use(&mut self, _i: i32) -> bool {
        false
    }
    fn unit_type(&mut self, _i: i32) -> i32 {
        0
    }
    fn unit_action(&mut self, _i: i32) -> Option<()> {
        None
    }
    fn order_kind(&mut self, _o: ()) -> i32 {
        0
    }
    fn updated_order_target(&mut self, _i: i32) -> (i32, i32) {
        (-1, -1)
    }
    fn unit_clear_orders(&mut self, _i: i32) {}
    fn has_tribe_bonus(&mut self, who: i32, n: i32) -> bool {
        n == 0x13 && self.sim.cities.nation[who as usize].lakota
    }
    fn lakota_raze_price(&mut self) -> i32 {
        self.sim.tables.tuning.lakota_raze_price
    }
    fn console_who(&mut self) -> i32 {
        -1
    }
    fn time(&mut self) -> u32 {
        let ty = self.sim.buildings[self.b].ty;
        ty.map_or(0, |t| {
            (self.sim.tables.build_types[t].job_time * 100) as u32
        })
    }
    fn job_counter(&mut self) -> u32 {
        self.sim.buildings[self.b].job_counter as u32
    }
    fn construct_time(&mut self) -> i32 {
        self.sim.construct_time_of(self.b)
    }
    fn job_counter_2(&mut self) -> i32 {
        self.sim.buildings[self.b].job_counter_2
    }
    fn unit_flags68(&mut self) -> i32 {
        0
    }
    fn unit_killer(&mut self) -> i32 {
        -1
    }
    fn unit_killed_obj(&mut self) -> i32 {
        0
    }
    fn leader_flags(&mut self, _who: i32) -> i32 {
        0
    }
    fn plunder(&mut self, _a: i32, _killer: i32, _c: i32, _d: i32) {}
    fn set_last_razed(&mut self, _who: i32, _index: i32) {}
    fn garrison_count(&mut self) -> i32 {
        0
    }
    fn inside_head(&mut self) -> i32 {
        -1
    }
    fn disband_inside(&mut self, _s: i32) {}
    fn random(&mut self, _lo: i32, _hi: i32) -> i32 {
        0
    }
    fn die(&mut self, _a: i32, _b: i32, _c: i32) {
        self.sim.die_building(self.b);
    }
    fn close(&mut self, _a: i32, _b: i32, _c: i32) {
        self.sim.close_building(self.b, false);
    }
    fn type_avail(&mut self, who: i32, good: i32, _strict: i32) -> bool {
        // A tree without the goods at the front (a test's) lets every good
        // through, as the crate did before this was asked.
        let g = good as usize;
        g >= self.sim.tables.tech_tree.types.len() || self.sim.type_avail(who as u8, g) != 0
    }
    fn orig_type(&mut self) -> i32 {
        -1
    }
    fn price(&mut self, args: [i32; 7]) -> i32 {
        let bd = &self.sim.buildings[self.b];
        bd.ty.map_or(0, |ty| {
            self.sim.building_price(bd.owner, ty)[args[0] as usize]
        })
    }
    fn add_good(&mut self, who: i32, good: i32, amount: i32) {
        self.sim.economy.ledgers[who as usize].bucket[good as usize] += amount;
    }
    fn message_good(&mut self, _good: i32, _amount: i32, _first: bool) {}
    fn message_end(&mut self) {}
}

/// `Object::disband+0x415`: the unit arm's one `Random::get(0, 0xffff)`,
/// read `% 3` for the `dtype` of the `die` after it.
pub const SITE_DISBAND_DIE: &str = "Object::disband+0x415";

impl Sim {
    /// `Group::action_disband@0070e260` on a group of units (item 3914,
    /// `docs/GOLDEN.md` §161): `OPTION_DISBAND` (`all` 0) or
    /// `OPTION_DISBAND_ALL` (`all` 1) with the seat's units selected.
    /// `GroupOut::validate_disband@0070a4a0` answers 0 for a unit group
    /// (`+0x49` clear), so the walk runs from the last member down: a live
    /// unit (vslot `0x20`, a plain building, answers 0) goes to
    /// `Object::disband(o, 0)` ([`Sim::disband_unit`]), and with `all` 0
    /// the walk stops after the first. Returns the units disbanded.
    ///
    /// The building arms are [`Sim::group_action_disband_buildings`].
    ///
    /// SEAM: the scenario's ignore-orders sweep; and the "already given"
    /// feedback, the user interface's.
    pub fn group_action_disband(&mut self, g: &crate::groups::group::Group, all: bool) -> usize {
        let mut n = 0;
        for &u in g.list.iter().rev() {
            if !self.units[u].alive() {
                continue;
            }
            self.disband_unit(u);
            n += 1;
            if !all {
                break;
            }
        }
        n
    }

    /// **`Group::action_disband@0070e260` on a group of buildings** (item
    /// 3994, `docs/GOLDEN.md` §187): `OPTION_DISBAND` with the seat's
    /// buildings selected. [`Sim::disband_refused`] (`validate_disband`'s
    /// building arms) refuses the whole command; then the walk runs from
    /// the last member down, and for each live one (`+8 & 1`):
    ///
    /// - not active (vslot `0x4c`: an unfinished site) → `Object::disband(o,
    ///   0)` ([`Sim::disband_building`], the refund the fraction not built),
    ///   and with `all` 0 the walk stops;
    /// - active, holding neither a `DISBAND` nor a `0x286` entry
    ///   (`count_queue(1, 0x29a)`, then `count_queue(1, 0x286)`, the
    ///   listing's pushes at `0x70e38e` and `0x70e3c6`: the decompile
    ///   prints neither argument) → `Build::queue_up(DISBAND, 0)`
    ///   ([`Sim::queue_disband`]; the second argument reaches only the
    ///   library forward, which `DISBAND` skips). The walk goes on.
    ///
    /// Returns the members disbanded or queued (`local_14`).
    ///
    /// SEAM: a `0x286` entry, which this crate's queue never holds.
    pub fn group_action_disband_buildings(&mut self, list: &[usize], all: bool) -> usize {
        if self.disband_refused(list) {
            return 0;
        }
        let mut n = 0;
        for &b in list.iter().rev() {
            if !self.buildings[b].alive {
                continue;
            }
            if !self.buildings[b].active {
                self.disband_building(b, false);
                n += 1;
                if !all {
                    break;
                }
            } else if self.buildings[b].queue.count_disband() == 0 {
                self.queue_disband(b);
                n += 1;
            }
        }
        n
    }

    /// `Object::disband(o, 0)@006455c0` on a unit. The export's
    /// `Unit::vftable` folds vslot `0xc` to `Window::get_button` (0), so
    /// a unit skips the builders' loop, the refund and `last_razed`; it is
    /// not a structure (`0x1c`, 0), so no Lakota raze; it is not a plain
    /// building (`0x20`, 0), so on the map it disbands what it holds and
    /// then `die`s with `dtype` `(Random::get(0, 0xffff) % 3) + 1`
    /// ([`Sim::die_with`]) — whose `Unit::close` spends the death
    /// animation's draw after this one.
    ///
    /// SEAM: the captain's plunder (`+0x68 & 0x4000`, the killer at
    /// `+0xa9`), which a living unit disbanded by its owner never carries
    /// in any capture; and a unit off the map, which takes `die(1, …)`
    /// with no draw.
    pub(crate) fn disband_unit(&mut self, u: usize) {
        if !self.units[u].on_map {
            self.die_with(u, 1);
            return;
        }
        // `garrison_count(0)`, then `disband` on the chain's head until it
        // is empty.
        while let Some(r) = (0..self.units.len())
            .find(|&r| self.units[r].inside_unit == Some(u) && self.units[r].alive())
        {
            self.units[r].inside_unit = None;
            self.disband_unit(r);
        }
        self.mark(SITE_DISBAND_DIE);
        let r = self.rng.roll();
        self.die_with(u, r % 3 + 1);
    }
}

#[cfg(test)]
mod tests;
