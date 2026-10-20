//! **`Group::action_alarm@0070ec30`'s steps** (item 3364): the City's alarm
//! over a group of buildings, as a pure walk over an [`AlarmView`]. The
//! swept steps read the world only through the view, so the sweep's port is
//! a view over the emulated original's windows and the crate's is
//! [`SimAlarm`] (`docs/GOLDEN.md` §49 for the bell and the all-clear).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it, which is what the sweep holds. `p` is a
//! player and `o`/`u` an index in that player's `objects`/`units` array;
//! the original indexes both arrays by one object index.

use crate::ai_place::circle;
use crate::world::vector_dist;

/// `SoundGlobal::play(S_SOUND_ALARM)`'s argument, the bell's.
pub const SOUND_BELL: i32 = 0x67;
/// The all-clear's.
pub const SOUND_ALL_CLEAR: i32 = 4;
/// `CityData` flags `+4`: the city is live.
pub const CITY_LIVE: i32 = 0x01;
/// `CityData` flags `+4`: the bell is rung.
pub const CITY_BELL: i32 = 0x40;
/// `Object` flags `+8`: a city building.
pub const OBJ_CITY: i32 = 0x20;

/// The head of `Group::action_alarm`: what it asks of the group before the
/// walk of its members.
pub trait AlarmHead {
    /// `[this + 0x4a]`.
    fn who(&self) -> i32;
    /// The stack group's `Group::clear(-1)@00713e80`.
    fn group_clear(&mut self);
    /// `ScenarioData::ignore_orders != 0`.
    fn ignore_orders(&self) -> bool;
    /// `ScenarioData::objects_ignoring_orders[who]`'s count.
    fn ignoring_count(&self, who: i32) -> i32;
    /// Its element `i`.
    fn ignoring_at(&self, who: i32, i: i32) -> i32;
    /// The group's slot `+0x10(o, who, 0, 0)`.
    fn drop_ignored(&mut self, o: i32, who: i32);
    /// `[this + 0x49] != 0`.
    fn armed(&self) -> bool;
    /// `Group::action_alarm_peasant@006fd980`.
    fn peasant_alarm(&mut self);
    /// The group's slot `+0x14()`.
    fn refresh(&mut self);
}

/// What `Group::action_alarm` asks of the group, the world and the objects.
pub trait AlarmView: AlarmHead {
    /// The leader's `0x2000000` bit, the economy's dirty bit.
    fn mark_economy(&mut self, who: i32);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;

    /// `objects[p][o]`'s slot `+0x10`.
    fn obj_slot10(&mut self, p: i32, o: i32) -> i32;
    /// `ignore_always`.
    fn ignore_always(&self) -> bool;
    /// "Find Pancake, there's a problem with action_alarm." through
    /// `Error::report`; true is the `int3` (a debugger break).
    fn report_pancake(&mut self) -> bool;
    /// `objects[p][o]`'s record `+8`.
    fn obj_flags8(&self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the data's short `+0x72`.
    fn data_city(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the data is finished (`Build` byte `+8 & 4`, else
    /// its slot `+0x4c`).
    fn data_finished(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xac`, then the data's short `+0x28`.
    fn data_first_inside(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the data's signed byte `+0x3e`.
    fn data_inside_player(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the data's short `+0x74`.
    fn data_next_building(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the data's short `+0x70`.
    fn data_first_unit(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then `ObjectData::num_inside(data, 0)`.
    fn data_num_inside(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then the garrison limit (`Build`: its type's
    /// `get_garrison_limit(level)`, else slot `+0x178`).
    fn data_limit(&mut self, p: i32, o: i32) -> i32;
    /// Slot `+0xac`, then slot `+0x90` of the data's type.
    fn data_type_slot90(&mut self, p: i32, o: i32) -> bool;
    /// Slot `+0xac`, then `Object::eject_contents(0, PEASANTS, 0, 0)`
    /// (`PEASANTS` is 0x32).
    fn eject_peasants(&mut self, p: i32, o: i32);
    /// `objects[p][o]`'s slot `+8`.
    fn obj_alive(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0xbc`.
    fn obj_bc(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s slot `+0x1c`.
    fn obj_slot1c(&mut self, p: i32, o: i32) -> bool;
    /// `objects[p][o]`'s record `+0x18`'s type index `+4`.
    fn obj_type(&self, p: i32, o: i32) -> i32;
    /// The tile `objects[p][o]` stands on: `div_3_table[(pos ^ 0x63637) >> 6]`
    /// for x and y.
    fn obj_tile(&self, p: i32, o: i32) -> (i32, i32);
    /// The short `+0x2c` and `+0x2e` of `objects[p][o]`: the next object on
    /// its cell's chain and its player.
    fn obj_chain(&self, p: i32, o: i32) -> (i32, i32);

    /// `units[p][u]`'s type index.
    fn unit_type(&self, p: i32, u: i32) -> i32;
    /// `units[p][u]`'s population: 0 for a decoy (`+0x68 & 1`), else its
    /// type's `+0x2f0`.
    fn unit_pop(&self, p: i32, u: i32) -> i32;
    /// `units[p][u]`'s short `+0x92`: the next unit inside the building.
    fn unit_next_inside(&self, p: i32, u: i32) -> i32;
    /// `units[p][u]`'s short `+0x28` and signed byte `+0x3e`.
    fn unit_next_civ(&self, p: i32, u: i32) -> (i32, i32);
    /// `is(MILITIA, 0)`: slot `+0xb8`, or the type's slot `+0x60`.
    fn unit_is_militia(&mut self, p: i32, u: i32) -> bool;
    /// `SpellType::cast_civilian(u, p)`.
    fn cast_civilian(&mut self, p: i32, u: i32);
    /// `UnitData::get_action`'s order's slot `+0x10`, none for a null order.
    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32>;
    /// `Unit::repath`.
    fn repath(&mut self, p: i32, u: i32);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self, p: i32, u: i32);

    /// `cities[who][c]`'s flags `+4`.
    fn city_flags(&self, who: i32, c: i32) -> i32;
    /// Sets them.
    fn set_city_flags(&mut self, who: i32, c: i32, v: i32);
    /// `cities[who][c]`'s short `+8`: the city building.
    fn city_building(&self, who: i32, c: i32) -> i32;
    /// `CityData::get_radius`.
    fn city_radius(&mut self, who: i32, c: i32) -> i32;
    /// The city's cell: `div_3_table[pos >> 8]` for x and y.
    fn city_cell(&self, who: i32, c: i32) -> (i32, i32);
    /// The city's tile: `div_3_table[pos >> 6]` for x and y.
    fn city_tile(&self, who: i32, c: i32) -> (i32, i32);
    /// `MiscAccess::console->who`.
    fn console_who(&self) -> i32;
    /// `SoundGlobal::play(who, id)`.
    fn play(&mut self, who: i32, id: i32);
    /// `world->w`, `world->h`.
    fn world_wh(&self) -> (i32, i32);
    /// The cell's first object and its player (the shorts `+8`, `+0xa`).
    fn cell_first(&self, x: i32, y: i32) -> (i32, i32);

    /// `LeaderData::tribe_can_type(leader, types[0x32])`.
    fn tribe_can_type(&mut self, who: i32) -> bool;
    /// The type's slot `+0x68(leader.who)`.
    fn type_slot68(&mut self, who: i32) -> i32;
    /// `UnitTypeData::can_garrison(unittypes[k], t)`.
    fn can_garrison(&mut self, k: i32, t: i32) -> bool;
    /// `ObjectsData::find_city_at(x, y, who, -1, 0)`.
    fn find_city_at(&mut self, x: i32, y: i32, who: i32) -> i32;
    /// `WorldData::get_down(x, y)`: the first object and its player.
    fn get_down(&mut self, x: i32, y: i32) -> (i32, i32);
    /// `GroupData::member(stack, u, who, 1)`.
    fn group_member(&mut self, u: i32, who: i32) -> bool;
    /// `Group::add(stack, u, who, 0, 0)`.
    fn group_add(&mut self, u: i32, who: i32);
    /// The stack group's count, `+0xc`.
    fn group_count(&self) -> i32;
    /// `Group::action_garrison(stack, b, who, QUEUE_NEW, 1)` (`QUEUE_NEW` is 2).
    fn action_garrison(&mut self, b: i32, who: i32);
}

/// `(a + 3) / 4` the way `70ef05` computes it, truncating toward zero.
fn cells_of(r: i32) -> i32 {
    (r + 3).wrapping_add(((r + 3) >> 31) & 3) >> 2
}

/// The first part of the listing, up to and including the group's refresh
/// (`70ed15`): the stack group cleared, the scenario's ignore-orders sweep, a
/// group that is not armed handed to `action_alarm_peasant`. True when the
/// walk goes on.
pub fn alarm_head<V: AlarmHead>(v: &mut V) -> bool {
    v.group_clear();
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
    if !v.armed() {
        v.peasant_alarm();
        return false;
    }
    v.refresh();
    true
}

/// The whole of `Group::action_alarm`.
pub fn action_alarm_steps<V: AlarmView>(v: &mut V) {
    if !alarm_head(v) {
        return;
    }
    let who = v.who() & 0xff;
    v.mark_economy(who);
    let mut rang = false;
    let mut cleared = false;
    let mut i = 0;
    while i < v.count() {
        i += 1;
        let o = v.member(i - 1);
        if v.obj_slot10(who, o) == 0 {
            if !v.ignore_always() && v.report_pancake() {
                return;
            }
            continue;
        }
        if v.obj_flags8(who, o) & OBJ_CITY == 0 {
            continue;
        }
        if v.data_city(who, o) < 0 {
            continue;
        }
        let c = v.data_city(who, o);
        let flags = v.city_flags(who, c);
        if flags & CITY_LIVE == 0 {
            continue;
        }
        if flags & CITY_BELL != 0 {
            if rang {
                continue;
            }
            all_clear(v, who, c, &mut cleared);
        } else {
            if cleared {
                continue;
            }
            bell(v, who, c, &mut rang);
        }
    }
}

/// The `0x40`-set arm (`70ee98`..`70f271`).
fn all_clear<V: AlarmView>(v: &mut V, who: i32, c: i32, cleared: &mut bool) {
    if !*cleared && v.console_who() == who {
        v.play(who, SOUND_ALL_CLEAR);
    }
    *cleared = true;
    let (cx, cy) = v.city_cell(who, c);
    let r = v.city_radius(who, c);
    let rc = cells_of(r);
    let circ = circle();
    let (w, h) = v.world_wh();
    for k in 0..circ.radius[rc as usize] {
        let x = cx + circ.x[k];
        let y = cy + circ.y[k];
        if x < 0 || y < 0 || x >= w || y >= h {
            continue;
        }
        let (mut idx, mut p) = v.cell_first(x, y);
        while idx >= 0 {
            if p == who && v.obj_alive(who, idx) && v.obj_bc(who, idx) {
                let t = v.unit_type(who, idx);
                if t == 0x32 || t == 0x33 {
                    let (ox, oy) = v.obj_tile(who, idx);
                    let (tx, ty) = v.city_tile(who, c);
                    if vector_dist((ox - tx).abs(), (oy - ty).abs()) <= r
                        && v.unit_order_kind(who, idx) == Some(0x1a)
                    {
                        v.repath(who, idx);
                        v.kill_current_order(who, idx);
                    }
                }
            }
            (idx, p) = v.obj_chain(p, idx);
        }
    }
    let mut b = v.city_building(who, c);
    while b >= 0 {
        if v.data_finished(who, b) {
            let mut u = v.data_first_inside(who, b);
            let mut pl = v.data_inside_player(who, b);
            while u >= 0 {
                if v.unit_is_militia(pl, u) {
                    v.cast_civilian(pl, u);
                }
                (u, pl) = v.unit_next_civ(pl, u);
            }
            v.eject_peasants(who, b);
        }
        b = v.data_next_building(who, b);
    }
    let flags = v.city_flags(who, c);
    v.set_city_flags(who, c, flags & !CITY_BELL);
}

/// The `0x40`-clear arm (`70f276`..`70f866`).
fn bell<V: AlarmView>(v: &mut V, who: i32, c: i32, rang: &mut bool) {
    if !*rang && v.console_who() == who {
        v.play(who, SOUND_BELL);
    }
    *rang = true;
    let cb = v.city_building(who, c);
    let mut room = v.data_num_inside(who, cb);
    let mut limit = v.data_limit(who, cb);
    let mut b = v.data_next_building(who, cb);
    let tribe = if v.tribe_can_type(who) {
        0x32
    } else {
        v.type_slot68(who)
    };
    while b >= 0 {
        let t = v.obj_type(who, b);
        if v.can_garrison(tribe, t) {
            room = room.wrapping_add(v.data_num_inside(who, b));
            limit = limit.wrapping_add(v.data_limit(who, b));
        }
        b = v.data_next_building(who, b);
    }
    let (cx, cy) = v.city_cell(who, c);
    let r = v.city_radius(who, c);
    let rc = cells_of(r);
    v.group_clear();
    let circ = circle();
    let (w, h) = v.world_wh();
    for k in 0..circ.radius[rc as usize] {
        let x = cx + circ.x[k];
        let y = cy + circ.y[k];
        if x < 0 || y < 0 || x >= w || y >= h {
            continue;
        }
        if v.find_city_at(x * 0x300 + 0x180, y * 0x300 + 0x180, who) != c {
            continue;
        }
        let (mut idx, mut p) = v.get_down(x, y);
        if idx < 0 {
            continue;
        }
        while room < limit {
            if p == who {
                if !v.obj_alive(who, idx) || !v.obj_bc(who, idx) {
                    if v.obj_slot1c(who, idx) && v.data_type_slot90(who, idx) {
                        let mut u = v.data_first_unit(who, idx);
                        while u >= 0 && room < limit {
                            let t = v.unit_type(who, u);
                            if (t == 0x32 || t == 0x33) && !v.group_member(u, who) {
                                v.group_add(u, who);
                                room = room.wrapping_add(v.unit_pop(who, u));
                            }
                            u = v.unit_next_inside(who, u);
                        }
                    }
                } else {
                    let t = v.unit_type(who, idx);
                    if t == 0x32 || t == 0x33 {
                        let (ox, oy) = v.obj_tile(who, idx);
                        let (tx, ty) = v.city_tile(who, c);
                        if vector_dist((ox - tx).abs(), (oy - ty).abs()) <= r
                            && !v.group_member(idx, who)
                        {
                            v.group_add(idx, who);
                            room = room.wrapping_add(v.unit_pop(who, idx));
                        }
                    }
                }
            }
            (idx, p) = v.obj_chain(p, idx);
            if idx < 0 {
                break;
            }
        }
    }
    if v.group_count() != 0 {
        v.action_garrison(cb, who);
        let flags = v.city_flags(who, c);
        v.set_city_flags(who, c, flags | CITY_BELL);
    }
}

/// This crate's [`AlarmView`]: `who`'s buildings `list`, the stack group the
/// bell builds, and the world as the crate keeps it. A member and a
/// city-chain building are building indices, a cell's chain and a building's
/// garrison unit indices; the original's one object index is told apart by
/// which method asks. Where the crate models less than the original:
///
/// - SEAM: the scenario's ignore-orders sweep is not modelled (no list, so
///   the head's loop never runs), nor the group's armed flag (a group here is
///   always armed), the group's refresh slot, the peasant variant, the
///   console (no sound) or the error report (the object's slot 0x10 is the
///   member being alive and the player's own).
/// - SEAM: a cell's chain holds units only, so the bell's
///   building-on-the-chain arm never runs, and the Citizens inside
///   such a building are not taken.
/// - SEAM: the tribe's Citizen substitution (the type's slot `+0x68`) is not
///   modelled: the Citizen is `PEASANTS`.
pub struct SimAlarm<'a> {
    pub sim: &'a mut crate::Sim,
    pub who: crate::world::Player,
    pub list: &'a [usize],
    pub group: crate::group::Group,
    /// The building whose garrison `unit_next_civ` walks.
    pub inside_of: usize,
}

impl<'a> SimAlarm<'a> {
    pub fn new(sim: &'a mut crate::Sim, who: crate::world::Player, list: &'a [usize]) -> Self {
        SimAlarm {
            sim,
            who,
            list,
            group: crate::group::Group::stack(who),
            inside_of: 0,
        }
    }

    fn first_of_cell(&self, x: i32, y: i32) -> (i32, i32) {
        match self.sim.cell_chain(crate::Cell::new(x, y)).first() {
            Some(&u) => (u as i32, i32::from(self.sim.units[u].owner)),
            None => (-1, -1),
        }
    }
}

impl AlarmHead for SimAlarm<'_> {
    fn who(&self) -> i32 {
        i32::from(self.who)
    }
    fn group_clear(&mut self) {
        self.group = crate::group::Group::stack(self.who);
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
    fn armed(&self) -> bool {
        true
    }
    fn peasant_alarm(&mut self) {}
    fn refresh(&mut self) {}
}

impl AlarmView for SimAlarm<'_> {
    fn mark_economy(&mut self, who: i32) {
        self.sim.economy_changed(who as crate::world::Player);
    }
    fn count(&self) -> i32 {
        self.list.len() as i32
    }
    fn member(&self, i: i32) -> i32 {
        self.list[i as usize] as i32
    }
    fn obj_slot10(&mut self, p: i32, o: i32) -> i32 {
        let bd = &self.sim.buildings[o as usize];
        i32::from(bd.alive && i32::from(bd.owner) == p)
    }
    fn ignore_always(&self) -> bool {
        true
    }
    fn report_pancake(&mut self) -> bool {
        false
    }
    fn obj_flags8(&self, _p: i32, o: i32) -> i32 {
        if self.sim.building_is_city(o as usize) {
            OBJ_CITY
        } else {
            0
        }
    }
    fn data_city(&mut self, _p: i32, o: i32) -> i32 {
        self.sim.buildings[o as usize].city.map_or(-1, |c| c as i32)
    }
    fn data_finished(&mut self, _p: i32, o: i32) -> bool {
        let bd = &self.sim.buildings[o as usize];
        bd.alive && bd.active
    }
    fn data_first_inside(&mut self, _p: i32, o: i32) -> i32 {
        self.inside_of = o as usize;
        self.sim.buildings[o as usize]
            .garrison
            .first()
            .map_or(-1, |&u| u as i32)
    }
    fn data_inside_player(&mut self, _p: i32, _o: i32) -> i32 {
        i32::from(self.who)
    }
    fn data_next_building(&mut self, _p: i32, o: i32) -> i32 {
        let Some(c) = self.sim.buildings[o as usize].city else {
            return -1;
        };
        let chain = self.sim.city_chain(c);
        match chain.iter().position(|&b| b == o as usize) {
            Some(i) => chain.get(i + 1).map_or(-1, |&b| b as i32),
            None => -1,
        }
    }
    fn data_first_unit(&mut self, _p: i32, _o: i32) -> i32 {
        -1
    }
    fn data_num_inside(&mut self, _p: i32, o: i32) -> i32 {
        self.sim.num_inside(o as usize)
    }
    fn data_limit(&mut self, _p: i32, o: i32) -> i32 {
        self.sim.garrison_limit(o as usize)
    }
    fn data_type_slot90(&mut self, _p: i32, _o: i32) -> bool {
        false
    }
    fn eject_peasants(&mut self, _p: i32, o: i32) {
        self.sim.eject_citizens_now(o as usize);
    }
    fn obj_alive(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].alive()
    }
    fn obj_bc(&mut self, _p: i32, o: i32) -> bool {
        self.sim.units[o as usize].on_map
    }
    fn obj_slot1c(&mut self, _p: i32, _o: i32) -> bool {
        false
    }
    fn obj_type(&self, _p: i32, o: i32) -> i32 {
        o
    }
    fn obj_tile(&self, _p: i32, o: i32) -> (i32, i32) {
        let t = self.sim.units[o as usize].pos.tile();
        (t.x, t.y)
    }
    fn obj_chain(&self, _p: i32, o: i32) -> (i32, i32) {
        match self.sim.units[o as usize].down {
            Some(d) => (d as i32, i32::from(self.sim.units[d].owner)),
            None => (-1, -1),
        }
    }
    fn unit_type(&self, _p: i32, u: i32) -> i32 {
        self.sim.units[u as usize].type_index
    }
    fn unit_pop(&self, _p: i32, u: i32) -> i32 {
        self.sim.control_cost(u as usize)
    }
    fn unit_next_inside(&self, _p: i32, _u: i32) -> i32 {
        -1
    }
    fn unit_next_civ(&self, p: i32, u: i32) -> (i32, i32) {
        let g = &self.sim.buildings[self.inside_of].garrison;
        match g.iter().position(|&x| x == u as usize) {
            Some(i) => (g.get(i + 1).map_or(-1, |&n| n as i32), p),
            None => (-1, p),
        }
    }
    fn unit_is_militia(&mut self, _p: i32, u: i32) -> bool {
        self.sim.unit_is_index(u as usize, crate::cast::MILITIA)
    }
    fn cast_civilian(&mut self, _p: i32, u: i32) {
        self.sim.cast_civilian(u as usize);
    }
    fn unit_order_kind(&mut self, _p: i32, u: i32) -> Option<i32> {
        let u = u as usize;
        self.sim
            .action_of(u)
            .map(|a| i32::from(self.sim.units[u].orders[a].index()))
    }
    fn repath(&mut self, _p: i32, u: i32) {
        self.sim.repath(u as usize);
    }
    fn kill_current_order(&mut self, _p: i32, u: i32) {
        self.sim.kill_current_order(u as usize);
    }
    fn city_flags(&self, _who: i32, c: i32) -> i32 {
        let city = &self.sim.cities[c as usize];
        i32::from(city.alive) * CITY_LIVE + i32::from(city.alarm) * CITY_BELL
    }
    fn set_city_flags(&mut self, _who: i32, c: i32, v: i32) {
        self.sim.cities[c as usize].alarm = v & CITY_BELL != 0;
    }
    fn city_building(&self, _who: i32, c: i32) -> i32 {
        self.sim.cities[c as usize].building as i32
    }
    fn city_radius(&mut self, _who: i32, c: i32) -> i32 {
        self.sim.radius_of(c as usize)
    }
    fn city_cell(&self, _who: i32, c: i32) -> (i32, i32) {
        let cell = self.sim.cities[c as usize].pos.cell();
        (cell.x, cell.y)
    }
    fn city_tile(&self, _who: i32, c: i32) -> (i32, i32) {
        let t = self.sim.cities[c as usize].pos.tile();
        (t.x, t.y)
    }
    fn console_who(&self) -> i32 {
        -1
    }
    fn play(&mut self, _who: i32, _id: i32) {}
    fn world_wh(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn cell_first(&self, x: i32, y: i32) -> (i32, i32) {
        self.first_of_cell(x, y)
    }
    fn tribe_can_type(&mut self, _who: i32) -> bool {
        true
    }
    fn type_slot68(&mut self, _who: i32) -> i32 {
        0
    }
    fn can_garrison(&mut self, _k: i32, t: i32) -> bool {
        let citizen = self.sim.unit_rec_of_index(crate::cast::PEASANTS);
        citizen
            .zip(self.sim.buildings[t as usize].ty)
            .is_some_and(|(ut, bt)| self.sim.can_garrison(ut, bt))
    }
    fn find_city_at(&mut self, x: i32, y: i32, who: i32) -> i32 {
        self.sim
            .find_city_at(
                who as crate::world::Player,
                crate::world::Pos::new(x, y),
                None,
            )
            .map_or(-1, |c| c as i32)
    }
    fn get_down(&mut self, x: i32, y: i32) -> (i32, i32) {
        self.first_of_cell(x, y)
    }
    fn group_member(&mut self, u: i32, _who: i32) -> bool {
        self.group.list.contains(&(u as usize))
    }
    fn group_add(&mut self, u: i32, _who: i32) {
        self.sim.group_add(&mut self.group, u as usize);
    }
    fn group_count(&self) -> i32 {
        self.group.list.len() as i32
    }
    fn action_garrison(&mut self, b: i32, _who: i32) {
        let g = self.group.clone();
        self.sim
            .group_action_garrison_search(&g, b as usize, crate::orders::QueuePos::New, true);
    }
}
