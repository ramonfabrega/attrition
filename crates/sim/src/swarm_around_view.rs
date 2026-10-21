//! **`Group::action_swarm_around@0070fbe0`'s steps** (item 3386): a group
//! swarming a site, as a pure walk over a [`SwarmView`]. The swept steps read
//! the world only through the view, so the sweep's port is a view over the
//! emulated original's windows and the crate's is [`SimSwarm`]
//! (`docs/GROUPS.md` §24, `docs/GOLDEN.md` §26, §49).
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked in
//! the order the listing asks it, which is what the sweep holds. `p` is the
//! group's player, `who` the site's (the command's second argument), `b` the
//! site's index in `who`'s objects and `m` a member's index in `p`'s
//! objects/units; the original indexes both arrays by one object index.
//! `order` is the `OrderIndex` (`BUILD_AT` 6, `GATHER` 7, `REPAIR` 0xd),
//! `queue` the `QueuePos` in the original's numbering (`QUEUE_FIRST` 0,
//! `QUEUE_LAST` 1, `QUEUE_NEW` 2).
//!
//! The listing's member walk is a `do … while`: a group with no members still
//! reads its first slot, and the steps keep that.

use crate::garrison_view::{GarrisonHead, garrison_head};

/// `OrderIndex::BUILD_AT`.
pub const BUILD_AT: i32 = 6;
/// `OrderIndex::GATHER`.
pub const GATHER: i32 = 7;
/// `OrderIndex::REPAIR`.
pub const REPAIR: i32 = 0xd;

/// The head of `Group::action_swarm_around`: the scratch group's clear, the
/// two refusals (the group is off the map, the site is gone), the scenario's
/// ignore-orders sweep and the group's refresh, which every entry runs, the
/// `QUEUE_FIRST` re-entry's too.
pub trait SwarmHead: GarrisonHead {
    /// `Group::clear(local scratch group, -1)`.
    fn scratch_clear(&mut self);
    /// `GroupData::is_on_map`.
    fn is_on_map(&mut self) -> bool;
    /// `objects[who][b]`'s byte `+8 & 1`: the site exists.
    fn site_exists(&self, who: i32, b: i32) -> bool;
}

/// The listing up to and including the group's refresh: false where the
/// entry returns before the sweep.
pub fn swarm_head<V: SwarmHead>(v: &mut V, b: i32, who: i32) -> bool {
    v.scratch_clear();
    if !v.is_on_map() {
        return false;
    }
    if !v.site_exists(who, b) {
        return false;
    }
    garrison_head(v);
    true
}

/// What `Group::action_swarm_around` asks of the group, the world and the
/// objects.
pub trait SwarmView: SwarmHead {
    /// `Group::add(local scratch group, m, p, 0, 0)`: a member the swarm does
    /// not take, sent to the site afterwards.
    fn scratch_add(&mut self, m: i32, p: i32);
    /// `Group::get_num(local scratch group)`.
    fn scratch_num(&mut self) -> i32;
    /// `Group::action_move_to(local scratch group, x, y, QUEUE_NEW, 0, 0,
    /// MOVE_TO, 0, -1, -1, 0)`.
    fn scratch_move_to(&mut self, x: i32, y: i32);
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
    /// `leaders[p]`'s first word: the leader's flags (bit 2 the human).
    fn leader_flags(&self, p: i32) -> i32;
    /// The site's slot `+0xc`.
    fn site_is_build(&mut self, who: i32, b: i32) -> bool;
    /// The site's slot `+0xb0` (its data), then the data's type's slot
    /// `+0x90`.
    fn site_type_flag(&mut self, who: i32, b: i32) -> bool;
    /// `GroupData::count(code, 0, 0)`.
    fn count(&mut self, code: i32) -> i32;
    /// The site's coordinates, `+0x10` and `+0x14` (each xor `0x63637`).
    fn site_pos(&self, who: i32, b: i32) -> (i32, i32);
    /// The site's slot `+0xb0`, then the word at `off` of the data's type
    /// (`+0x234`, `+0x238`: the footprint's sides).
    fn data_ty(&mut self, who: i32, b: i32, off: i32) -> i32;
    /// `units[who][b]`'s type's word `+0x244`.
    fn unit_ty_244(&self, who: i32, b: i32) -> i32;
    /// The site's slot `+0xb8`, `is(id, 0)`: the slot itself, or on the base
    /// the site's type's slot `+0x60`.
    fn site_is(&mut self, who: i32, b: i32, id: i32) -> bool;
    /// `[this + 0xc]`.
    fn members(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `objects[p][m]`'s slot `+8`.
    fn m_alive(&mut self, p: i32, m: i32) -> bool;
    /// `objects[p][m]`'s slot `+0xbc`.
    fn m_bc(&mut self, p: i32, m: i32) -> bool;
    /// `units[p][m]`'s type's `+0x218`: land 0, sea 1, air 2.
    fn m_domain(&self, p: i32, m: i32) -> i32;
    /// `UnitData::get_action`'s order's slot `+0x10`, none for a null order.
    fn m_action_kind(&mut self, p: i32, m: i32) -> Option<i32>;
    /// `UnitData::is_busy`.
    fn m_busy(&mut self, p: i32, m: i32) -> bool;
    /// `units[p][m]`'s type's `+4`.
    fn m_type(&self, p: i32, m: i32) -> i32;
    /// `ObjectData::count_inside(units[p][m], COUNT_PEASANTS, 0)`.
    fn m_count_inside(&mut self, p: i32, m: i32) -> i32;
    /// `SpellTypeData::is_castable(spelltypes[0xa4c], m, p, 0)`: the
    /// Civilian.
    fn m_castable(&mut self, p: i32, m: i32) -> bool;
    /// The site's slot `+0x1c`.
    fn site_slot_1c(&mut self, who: i32, b: i32) -> bool;
    /// The site's type's `+0x218`: land 0, water 1.
    fn site_domain(&self, who: i32, b: i32) -> i32;
    /// `ObjectData::is_peasant(units[p][m])`.
    fn m_is_peasant(&mut self, p: i32, m: i32) -> bool;
    /// `LeaderData::can_transport(leaders[p])`.
    fn can_transport(&mut self, p: i32) -> i32;
    /// `LeaderData::current_upgrade(0x140)`: the transport barge's type.
    fn current_upgrade(&mut self, p: i32) -> i32;
    /// `units[p][m]`'s byte `+0x3e` and short `+0x28`: the unit that carries
    /// it, as an owner and an index (negative: none).
    fn m_inside(&self, p: i32, m: i32) -> (i32, i32);
    /// `units[owner][idx]`'s type's `+4`.
    fn unit_type_of(&self, owner: i32, idx: i32) -> i32;
    /// `UnitType::find_nearby_spot(seeker's type, x, y, &out, radius, a5, a6,
    /// angle, FILTER_NOT_ME, m, p, 0, 0, b, who, -1)`, zero for a spot found.
    /// `which` tells the two call sites apart (the first spot, then the
    /// build's nudged one), `seeker` the type index its `ecx` is read at.
    #[allow(clippy::too_many_arguments)]
    fn find_nearby_spot(
        &mut self,
        which: u8,
        seeker: i32,
        x: i32,
        y: i32,
        radius: i32,
        a5: i32,
        a6: i32,
        angle: i32,
        m: i32,
        p: i32,
        b: i32,
        who: i32,
    ) -> Option<(i32, i32)>;
    /// `find_angle(dx, dy)`.
    fn find_angle(&self, dx: i32, dy: i32) -> i32;
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// `objects[p][m]`'s coordinates, `+0x10` and `+0x14` (each xor
    /// `0x63637`).
    fn m_pos(&self, p: i32, m: i32) -> (i32, i32);
    /// `Unit::add_move_facing_order(units[p][m], x, y, angle, kind, 0, queue,
    /// 0, -1, -1, -1, 0)`, `x`/`y` in the `div_3_table`'s units.
    #[allow(clippy::too_many_arguments)]
    fn add_move_facing(&mut self, p: i32, m: i32, x: i32, y: i32, angle: i32, kind: i32, queue: i32);
    /// `Unit::add_cast_order(units[p][m], -1, -1, x, y, 0x293, QUEUE_LAST,
    /// action)`.
    fn add_cast(&mut self, p: i32, m: i32, x: i32, y: i32, action: i32);
    /// `Unit::add_build_order(units[p][m], b, who, QUEUE_LAST, action)`.
    fn add_build(&mut self, p: i32, m: i32, b: i32, who: i32, action: i32);
    /// `Unit::add_repair_order(units[p][m], b, who, QUEUE_LAST, action)`.
    fn add_repair(&mut self, p: i32, m: i32, b: i32, who: i32, action: i32);
    /// The site's slot `+0xb0`, then `data->short[+0x60] &= 0xdfff`.
    fn clear_site_2000(&mut self, who: i32, b: i32);
    /// `::ignore_always`.
    fn ignore_always(&self) -> bool;
    /// `Error::report("groups.cpp", "ILLEGAL SWARM AROUND ORDER", …)`: true
    /// where the report asks for the break.
    fn report_illegal(&mut self) -> bool;
}

/// `Group::action_swarm_around(b, who, queue, order, action)`: `b` the site's
/// index in `who`'s objects, `action` the order's action bit. `depth` counts
/// the `QUEUE_FIRST` re-entries above this call.
pub fn action_swarm_around_steps<V: SwarmView>(
    v: &mut V,
    b: i32,
    who: i32,
    queue: i32,
    order: i32,
    action: i32,
    depth: i32,
) {
    if !swarm_head(v, b, who) {
        return;
    }
    if queue == 0 {
        // The insert dance: the group halts, takes the swarm at `QUEUE_NEW`
        // and re-issues what it was doing (`finish_insert`).
        v.set_up_insert();
        v.action_halt();
        action_swarm_around_steps(v, b, who, 2, order, action, depth + 1);
        v.finish_insert();
        v.list_clear();
        return;
    }
    let p = v.who() & 0xff;
    v.set_form();
    v.scratch_clear();
    let mut kind = 1;
    let mut idle = 0;
    if order == BUILD_AT {
        kind = (!((v.leader_flags(p) as u32) >> 1) & 2 | 1) as i32;
        if v.site_is_build(who, b) && v.site_type_flag(who, b) {
            let gathering = v.count(0xb);
            let peasants = v.count(1);
            let building = v.count(0xe);
            idle = peasants.wrapping_sub(gathering).wrapping_sub(building);
        }
    }
    let (bx, by) = v.site_pos(who, b);
    let mut radius = if v.site_is_build(who, b) {
        let a = v.data_ty(who, b, 0x238);
        let c = v.data_ty(who, b, 0x234);
        let side = if c < a {
            v.data_ty(who, b, 0x234)
        } else {
            v.data_ty(who, b, 0x238)
        };
        side.wrapping_mul(0x60).wrapping_add(0x30)
    } else {
        v.unit_ty_244(who, b)
    };
    if v.site_is(who, b, 0x1a1) && order == BUILD_AT {
        radius /= 2;
    }
    for layer in 0..2 {
        let mut i = 0;
        loop {
            let m = v.member(i);
            swarm_member(v, p, m, b, who, queue, order, action, (bx, by), radius, kind, idle, layer);
            i += 1;
            if i >= v.members() {
                break;
            }
        }
    }
    if v.scratch_num() != 0 {
        v.scratch_move_to(bx, by);
    }
}

/// One member of the walk: the listing's body from `0070ff60` to `0071060a`.
#[allow(clippy::too_many_arguments)]
fn swarm_member<V: SwarmView>(
    v: &mut V,
    p: i32,
    m: i32,
    b: i32,
    who: i32,
    queue: i32,
    order: i32,
    action: i32,
    (bx, by): (i32, i32),
    radius: i32,
    kind: i32,
    idle: i32,
    layer: i32,
) {
    if !v.m_alive(p, m) || !v.m_bc(p, m) {
        return;
    }
    let domain = v.m_domain(p, m);
    if domain != layer || domain == 2 {
        return;
    }
    if idle != 0 && v.m_action_kind(p, m) == Some(7) {
        return;
    }
    if v.m_busy(p, m) {
        return;
    }
    if order == BUILD_AT || order == REPAIR {
        let t = v.m_type(p, m);
        if t != 0x32 && t != 0x33 && v.m_count_inside(p, m) == 0 && !v.m_castable(p, m) {
            v.scratch_add(m, p);
            return;
        }
    }
    if order == GATHER {
        let t = v.m_type(p, m);
        let worker = t == 0x32 || t == 0x33 || t == 0x34 || t == 0x35 || v.m_castable(p, m);
        let mut take = false;
        if worker {
            let t = v.m_type(p, m);
            take = if t == 0x34 || t == 0x35 {
                v.site_is(who, b, 0x1a4)
            } else {
                !v.site_is(who, b, 0x1a4)
            };
        }
        if !take {
            v.scratch_add(m, p);
            return;
        }
    }
    let (mx, my) = v.m_pos(p, m);
    let sweep_angle = v.find_angle(mx.wrapping_sub(bx), my.wrapping_sub(by));
    let seeker = seeker_type(v, p, m, b, who);
    let Some((mut ox, mut oy)) = v.find_nearby_spot(
        0, seeker, bx, by, radius, 0, -1, sweep_angle, m, p, b, who,
    ) else {
        return;
    };
    let facing = v.find_angle(bx.wrapping_sub(ox), by.wrapping_sub(oy));
    if order == BUILD_AT {
        let cx = if ox > bx { ox.wrapping_add(0x30) } else { ox.wrapping_sub(0x30) };
        let cy = if oy > by { oy.wrapping_add(0x30) } else { oy.wrapping_sub(0x30) };
        if let Some((nx, ny)) = v.find_nearby_spot(
            1, seeker, cx, cy, 0, 0, 0, sweep_angle, m, p, b, who,
        ) {
            ox = nx;
            oy = ny;
        }
    }
    let mut at = queue;
    if (order == BUILD_AT || order == REPAIR)
        && queue == 1
        && v.m_action_kind(p, m) == Some(7)
        && action != 0
    {
        at = 2;
    }
    let x3 = v.div3(ox >> 4);
    let y3 = v.div3(oy >> 4);
    v.add_move_facing(p, m, x3, y3, facing, kind, at);
    if order == BUILD_AT {
        if v.m_castable(p, m) {
            let (cx, cy) = v.m_pos(p, m);
            v.add_cast(p, m, cx, cy, action);
        }
        v.add_build(p, m, b, who, action);
        v.clear_site_2000(who, b);
    } else if order == REPAIR {
        if v.m_castable(p, m) {
            let (cx, cy) = v.m_pos(p, m);
            v.add_cast(p, m, cx, cy, action);
        }
        v.add_repair(p, m, b, who, action);
    } else if !v.ignore_always() && v.report_illegal() {
        panic!("ILLEGAL SWARM AROUND ORDER");
    }
}

/// The type the ring is searched as (`00710238`–`007103a7`): a citizen
/// swarming a water building under a leader that can carry citizens is
/// searched as the leader's barge, a unit that carries citizens swarming a
/// land building as the passenger it carries, anything else as itself.
fn seeker_type<V: SwarmView>(v: &mut V, p: i32, m: i32, b: i32, who: i32) -> i32 {
    if v.site_slot_1c(who, b)
        && v.site_domain(who, b) == 1
        && !v.site_is(who, b, 0x1b0)
        && v.m_is_peasant(p, m)
        && v.can_transport(p) >= 3
    {
        return v.current_upgrade(p);
    }
    if v.site_slot_1c(who, b) && v.site_domain(who, b) == 0 && v.m_count_inside(p, m) != 0 {
        let (owner, idx) = v.m_inside(p, m);
        if idx >= 0 {
            return v.unit_type_of(owner, idx);
        }
    }
    v.m_type(p, m)
}

/// This crate's [`SwarmView`]: the group, the insert dance's copies and the
/// world as the crate keeps it. A member is a unit index and the site a
/// building index; the original's one object index is told apart by which
/// method asks. Where the crate models less than the original:
///
/// - SEAM: the scenario's ignore-orders sweep and the group's refresh are
///   not modelled (no list, no slot).
/// - SEAM: the scratch group is not modelled. The members the swarm does not
///   take (not a citizen, carrying none, no Civilian to cast) are gathered
///   by the original and sent `MOVE_TO` the site at `QUEUE_NEW`; here they
///   are left alone, as the member arm always has (no capture on file
///   swarms a mixed group).
/// - SEAM: the busy test (`is_busy`, a unit mid-cast or boarding), the
///   idle-citizens filter (the site's type's slot `+0x90` is not carried, so
///   `idle` stays 0), a unit carrying citizens (`count_inside`) and the
///   `GATHER` arm's predicate (`is(0x1a4)`; this crate swarms `Build` and
///   `Repair` only) answer no; the report of an illegal order is not
///   modelled; `BUILD_AT`'s clear of the site's `+0x60 & 0x2000` is not
///   either.
/// - The object's type indices are the original's only where a test reads
///   them: a citizen answers `0x32` and every other unit `0x7fff`, the
///   seeker's type is rebuilt by [`crate::Sim::swarm_ring`] from the
///   member and the site.
pub struct SimSwarm<'a> {
    pub sim: &'a mut crate::Sim,
    pub g: &'a crate::group::Group,
    insert: Option<crate::group::Insert>,
}

impl<'a> SimSwarm<'a> {
    pub fn new(sim: &'a mut crate::Sim, g: &'a crate::group::Group) -> Self {
        SimSwarm {
            sim,
            g,
            insert: None,
        }
    }
}

fn pos_of(queue: i32) -> crate::orders::QueuePos {
    match queue {
        0 => crate::orders::QueuePos::First,
        1 => crate::orders::QueuePos::Last,
        _ => crate::orders::QueuePos::New,
    }
}

impl GarrisonHead for SimSwarm<'_> {
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

impl SwarmHead for SimSwarm<'_> {
    fn scratch_clear(&mut self) {}
    fn is_on_map(&mut self) -> bool {
        self.sim.group_is_on_map(self.g)
    }
    fn site_exists(&self, _who: i32, b: i32) -> bool {
        self.sim
            .buildings
            .get(b as usize)
            .is_some_and(|bd| bd.alive)
    }
}

impl SwarmView for SimSwarm<'_> {
    fn scratch_add(&mut self, _m: i32, _p: i32) {}
    fn scratch_num(&mut self) -> i32 {
        0
    }
    fn scratch_move_to(&mut self, _x: i32, _y: i32) {}
    fn set_up_insert(&mut self) {
        self.insert = Some(self.sim.group_set_up_insert(self.g));
    }
    fn action_halt(&mut self) {
        self.sim.group_action_halt(self.g, 0);
    }
    fn finish_insert(&mut self) {
        if let Some(insert) = self.insert.take() {
            self.sim.group_finish_insert(self.g, insert);
        }
    }
    fn list_clear(&mut self) {}
    fn set_form(&mut self) {
        if let Some(st) = self.sim.gstate_mut(self.g) {
            st.form = -1;
        }
    }
    fn leader_flags(&self, p: i32) -> i32 {
        // `leaders[p]`'s flags: bit 2 is the human.
        if self.sim.nation[p as usize].human {
            4
        } else {
            0
        }
    }
    fn site_is_build(&mut self, _who: i32, _b: i32) -> bool {
        true
    }
    fn site_type_flag(&mut self, _who: i32, _b: i32) -> bool {
        false
    }
    fn count(&mut self, _code: i32) -> i32 {
        0
    }
    fn site_pos(&self, _who: i32, b: i32) -> (i32, i32) {
        let pos = self.sim.buildings[b as usize].pos;
        (pos.x, pos.y)
    }
    fn data_ty(&mut self, _who: i32, b: i32, off: i32) -> i32 {
        let t = self.sim.buildings[b as usize].ty;
        t.map_or(1, |t| {
            if off == 0x234 {
                self.sim.build_types[t].x_size
            } else {
                self.sim.build_types[t].y_size
            }
        })
    }
    fn unit_ty_244(&self, _who: i32, _b: i32) -> i32 {
        0
    }
    fn site_is(&mut self, _who: i32, b: i32, id: i32) -> bool {
        let b = b as usize;
        match id {
            // `is(FARM)`: the swarm of a farm halves the ring.
            0x1a1 => self.sim.building_ident(b) == crate::build::Ident::Farm,
            // `is(DOCK)`: the lineage a water site is excepted by.
            0x1b0 => self.sim.buildings[b]
                .ty
                .is_some_and(|t| crate::build::is_dock(&self.sim.build_types, t)),
            _ => false,
        }
    }
    fn members(&self) -> i32 {
        self.g.list.len() as i32
    }
    fn member(&self, i: i32) -> i32 {
        // A group with no members still reads its first slot (the walk is a
        // `do … while`); here that slot names no unit.
        self.g.list.get(i as usize).map_or(-1, |&u| u as i32)
    }
    fn m_alive(&mut self, _p: i32, m: i32) -> bool {
        m >= 0 && self.sim.units[m as usize].alive()
    }
    fn m_bc(&mut self, _p: i32, m: i32) -> bool {
        self.sim.units[m as usize].on_map
    }
    fn m_domain(&self, _p: i32, m: i32) -> i32 {
        use crate::attrition::Domain;
        let u = &self.sim.units[m as usize];
        match u.ty.map_or(Domain::Land, |t| self.sim.unit_types[t].kind.domain) {
            Domain::Land => 0,
            Domain::Sea => 1,
            _ => 2,
        }
    }
    fn m_action_kind(&mut self, _p: i32, m: i32) -> Option<i32> {
        let u = m as usize;
        self.sim.action_of(u).map(|a| {
            if self.sim.units[u].orders[a].index() == crate::orders::index::GATHER {
                7
            } else {
                0
            }
        })
    }
    fn m_busy(&mut self, _p: i32, _m: i32) -> bool {
        false
    }
    fn m_type(&self, _p: i32, m: i32) -> i32 {
        if self.sim.worker_of(m as usize) == crate::orders::Worker::Citizen {
            0x32
        } else {
            0x7fff
        }
    }
    fn m_count_inside(&mut self, _p: i32, _m: i32) -> i32 {
        0
    }
    fn m_castable(&mut self, _p: i32, m: i32) -> bool {
        self.sim
            .spell_castable(crate::orders::spell::CIVILIAN, m as usize)
    }
    fn site_slot_1c(&mut self, _who: i32, _b: i32) -> bool {
        true
    }
    fn site_domain(&self, _who: i32, b: i32) -> i32 {
        let water = self.sim.buildings[b as usize].ty.is_some_and(|t| {
            self.sim.build_types[t].domain() == crate::build::BuildDomain::Water
        });
        i32::from(water)
    }
    fn m_is_peasant(&mut self, _p: i32, m: i32) -> bool {
        self.sim.is_peasant(m as usize)
    }
    fn can_transport(&mut self, p: i32) -> i32 {
        self.sim.transport_level(p as crate::world::Player) as i32
    }
    fn current_upgrade(&mut self, p: i32) -> i32 {
        self.sim
            .barge_type(p as crate::world::Player)
            .map_or(-1, |t| t as i32)
    }
    fn m_inside(&self, _p: i32, _m: i32) -> (i32, i32) {
        (0, -1)
    }
    fn unit_type_of(&self, _owner: i32, _idx: i32) -> i32 {
        0x7fff
    }
    fn find_nearby_spot(
        &mut self,
        _which: u8,
        _seeker: i32,
        x: i32,
        y: i32,
        radius: i32,
        a5: i32,
        a6: i32,
        angle: i32,
        m: i32,
        _p: i32,
        b: i32,
        _who: i32,
    ) -> Option<(i32, i32)> {
        self.sim
            .swarm_ring(
                m as usize,
                b as usize,
                crate::Pos::new(x, y),
                radius,
                a5,
                a6,
                crate::movement::Angle(angle),
            )
            .map(|s| (s.x, s.y))
    }
    fn find_angle(&self, dx: i32, dy: i32) -> i32 {
        crate::movement::find_angle(dx, dy).0
    }
    fn div3(&self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn m_pos(&self, _p: i32, m: i32) -> (i32, i32) {
        let pos = self.sim.units[m as usize].pos;
        (pos.x, pos.y)
    }
    fn add_move_facing(
        &mut self,
        _p: i32,
        m: i32,
        x: i32,
        y: i32,
        angle: i32,
        kind: i32,
        queue: i32,
    ) {
        // The tile pair back to a point: `add_move_facing_order` snaps to
        // `SNAP` again.
        const SNAP: i32 = 0x30;
        let kind = if kind == 3 {
            crate::orders::MoveKind::ExploreTo
        } else {
            crate::orders::MoveKind::MoveTo
        };
        self.sim.add_move_facing_order(
            m as usize,
            crate::Pos::new(x * SNAP, y * SNAP),
            kind,
            pos_of(queue),
            false,
            crate::movement::Angle(angle),
            None,
            false,
        );
    }
    fn add_cast(&mut self, _p: i32, m: i32, x: i32, y: i32, action: i32) {
        self.sim.add_cast_order_on(
            m as usize,
            crate::orders::spell::CIVILIAN,
            None,
            crate::Pos::new(x, y),
            crate::orders::QueuePos::Last,
            action != 0,
        );
    }
    fn add_build(&mut self, _p: i32, m: i32, b: i32, _who: i32, action: i32) {
        self.sim.add_build_order(
            m as usize,
            b as usize,
            crate::orders::QueuePos::Last,
            action != 0,
        );
    }
    fn add_repair(&mut self, _p: i32, m: i32, b: i32, _who: i32, action: i32) {
        self.sim.add_repair_order(
            m as usize,
            b as usize,
            crate::orders::QueuePos::Last,
            action != 0,
        );
    }
    fn clear_site_2000(&mut self, _who: i32, _b: i32) {}
    fn ignore_always(&self) -> bool {
        true
    }
    fn report_illegal(&mut self) -> bool {
        false
    }
}
