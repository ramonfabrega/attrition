//! **`Unit::come_out@00617c10`'s exit-spot steps** (item 3362): the first
//! half of the function — from its entry to `set_new_location`'s call at
//! `6186bc` — as a pure walk over a [`ComeOutView`]. The swept steps read the
//! world only through the view, so the sweep's port is a view over the
//! emulated original's windows and the crate's is [`SimComeOut`]
//! (`docs/CITIES.md` §6.5, `docs/PRODUCTION.md` "The gather point").
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call, one direct callee or one inline read of the listing, asked
//! in the order the listing asks it — which is what the sweep holds. The
//! function's tail (`6186c1` on: the ceo position, the animation, the
//! path wipe, the squad's push) is not here.
//!
//! What it models, by arm of the listing:
//!
//! - **no host** (`get_inside < 0`, `617d27`): the unit is swept round its own
//!   point from its type's `block_radius` out `UNIT_DISEMBARK_DISTANCE`, under
//!   `FILTER_NOT_ME`, then again with the collision flag; both failing, it
//!   stays where it is. The crate's [`Sim::come_out`] returns early for a unit
//!   that is inside nothing, so this arm has no caller in the port (not modelled).
//! - **the barge** (`617e5d`..`617f97`): a host that `is(0x1a6)` for a land
//!   unit spawns a transport barge. Only its refusal (`init_unit < 0`,
//!   `return 1`) is modelled; the success arm recurses into `come_out` and
//!   is [`Exit::Barge`], unmodelled.
//! - **a missile** (`617fc7`): the host's own point, the figure's `+0x40` the
//!   float 90.0.
//! - **everything else** (`61800f`..`618657`): the captain redirect, the
//!   leader flag, the gather-point block, and the ring round the host.

/// `FILTER_ALL` as `find_nearby_spot` is handed it.
pub const FILTER_ALL: i32 = 0;
/// `FILTER_NOT_ME`.
pub const FILTER_NOT_ME: i32 = 3;
/// `is(0x1a4, 0)`.
pub const IS_A4: i32 = 0x1a4;
/// `is(0x1a6, 0)`.
pub const IS_A6: i32 = 0x1a6;
/// The default sweep bearing, due south: `Angle(i32::MIN)`.
pub const SOUTH: i32 = i32::MIN;
/// The float `90.0` a missile's figure is given at `+0x40`.
pub const MISSILE_FIGURE_90: u32 = 0x42b4_0000;
/// The gather block's reach (`0x600`) and its bearing (`0x55555555`).
pub const GATHER_REACH: i32 = 0x600;
pub const GATHER_BIAS: i32 = 0x5555_5555;
/// The tail's `set_new_location(·, ·, 1, 1)`.
pub const PLACED: (i32, i32) = (1, 1);

/// One `UnitType::find_nearby_spot@0061de70` call, the arguments it varies:
/// the centre, the ring `[min, max]`, the bearing, the filter and the two
/// flags before the constant `-1, 0, -1` (`a12` is the collision-off flag).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Find {
    pub x: i32,
    pub y: i32,
    pub min: i32,
    pub max: i32,
    pub angle: i32,
    pub filter: i32,
    pub a12: i32,
    pub a13: i32,
    /// The gather block's call: its out-words are other locals of the
    /// original's frame (the sweep's recorded stack words differ), and
    /// nothing else of it.
    pub gather: bool,
}

/// What `find_nearby_spot` answered: its return (0 is a spot found) and the
/// two out-words it wrote, which are read whatever the return was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub code: i32,
    pub x: i32,
    pub y: i32,
}

/// The head node of a building's gather list: its kind byte and the two
/// words (`+4`, `+8`): a point, or an object's `(index, player)` for kind 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: i32,
    pub a: i32,
    pub b: i32,
}

/// Where the walk ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// `param_1 == 0` and not the captain: `come_out(captain, 0)` (`617cf7`).
    /// Not walked.
    Recurse,
    /// The function returned `1` before placing (a refused sweep, a refused
    /// barge).
    Ret(i32),
    /// The barge was made (`init_unit >= 0`): recurses and dies. Not walked.
    Barge,
    /// `set_new_location(this, x, y, 1, 1)` is reached.
    Place { x: i32, y: i32 },
}

/// What `Unit::come_out` asks of the unit, its host and the world. A host
/// is named by `(player, index)` into the objects arrays, as the original
/// names it. Every method is one leaf of the listing.
pub trait ComeOutView {
    /// `Group::clear` on the local group (`617c66`).
    fn group_clear(&mut self);
    /// The signed byte at `+0xb5`: how many figures the unit has.
    fn member_count(&self) -> i32;
    /// `figures[i]->+0xd0 = 0` (`617c8f`).
    fn clear_member(&mut self, i: i32);
    /// `this->is_captain()` (vslot `0xe8`, inlined when it is the base).
    fn is_captain(&mut self) -> bool;
    /// `ObjectData::get_inside(this, &player)`: the host index (negative: none)
    /// and the player the out-word was given.
    fn get_inside(&mut self) -> (i32, i32);
    /// The unit's own point (`+0x10`, `+0x14`, `^ 0x63637`).
    fn pos(&self) -> (i32, i32);
    /// The unit's owner (`+0x9`).
    fn owner(&self) -> i32;
    /// Its type's `block_radius` (`+0x240`).
    fn block_radius(&self) -> i32;
    /// Its type's domain (`+0x218`).
    fn domain(&self) -> i32;
    /// Its type's `big_radius` (`+0x244`).
    fn big_radius(&self) -> i32;
    /// Its type's missile flag (`+0x1e4 & 0x8000000`).
    fn missile(&self) -> bool;
    /// `constants->unit_disembark_distance`.
    fn disembark(&self) -> i32;
    /// `UNIT_TRAIN_DISTANCE`, `UNIT_TRAIN_MAX_DISTANCE`.
    fn train(&self) -> (i32, i32);
    /// `BOAT_TRAIN_DISTANCE`, `BOAT_TRAIN_MAX_DISTANCE`.
    fn boat_train(&self) -> (i32, i32);
    /// `UnitType::find_nearby_spot` with this unit's own type, index and owner.
    fn find(&mut self, f: Find) -> Found;
    /// `objects[player][h]->is(id, 0)`.
    fn host_is(&mut self, player: i32, h: i32, id: i32) -> bool;
    /// `LeaderData::current_upgrade(…, TRANSPORTBARGE)` then
    /// `Objects::init_unit` at the host's point; the second's answer.
    fn barge(&mut self, player: i32, h: i32) -> i32;
    /// An object's point (`+0x10`, `+0x14`, `^ 0x63637`).
    fn object_pos(&self, player: i32, h: i32) -> (i32, i32);
    /// `this->guys[0]->+0x40 = 90.0`.
    fn set_figure_90(&mut self);
    /// The host's vslot `0x20` (is a building, as the gather block asks).
    fn v20(&mut self, player: i32, h: i32) -> bool;
    /// The host's vslot `0x34`, whose answer the function never reads.
    fn v34(&mut self, player: i32, h: i32);
    /// The host's vslot `0x1c`: 1 for a `Build`, 0 for a `Unit`.
    fn v1c(&mut self, player: i32, h: i32) -> bool;
    /// The host's vslot `0xe4`, for a unit that is not its squad's captain.
    fn get_captain(&mut self) -> i32;
    /// `leaders[owner].flags |= 0x2000000`.
    fn set_leader_flag(&mut self);
    /// The host's vslot `0xac`: its `BuildData`, an opaque handle.
    fn build_data(&mut self, player: i32, h: i32) -> i32;
    /// `build_data->+0xcc != 0`.
    fn bd_has_list(&self, bd: i32) -> bool;
    /// `BuildData::gather_inside` (`46f180`).
    fn bd_gather_inside(&mut self, bd: i32) -> bool;
    /// `LinkListBase::head` of the list at `+0xbc` (`46f0c0`).
    fn bd_head(&mut self, bd: i32);
    /// The head node, `[bd + 0xbc]`.
    fn bd_node(&self, bd: i32) -> Node;
    /// `HotKeyGroups::copy(a, b, this->index, owner)` (`714fd0`).
    fn copy_hotkeys(&mut self, a: i32, b: i32);
    /// `find_angle(dx, dy)` (`92d130`, both in registers).
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32;
    /// `units[owner][this->index]->angle = a` (`+0x50`, a bare store).
    fn set_unit_angle(&mut self, a: i32);
    /// An object's `+8 & 1`.
    fn object_alive(&self, player: i32, h: i32) -> bool;
    /// A building object's type `x_size`, `y_size` (`+0x234`, `+0x238`).
    fn object_size(&self, player: i32, h: i32) -> (i32, i32);
    /// A unit host's angle (`units[player][h]->+0x50`).
    fn host_angle(&self, player: i32, h: i32) -> i32;
    /// A unit host's type `block_radius`.
    fn host_block_radius(&self, player: i32, h: i32) -> i32;
    /// `this->pos = (x, y)` (stored `^ 0x63637`).
    fn set_pos(&mut self, x: i32, y: i32);
    /// `TerrainOut::find_tcoord_z` of `(div3[x >> 6], div3[y >> 6])`, its out
    /// word stored `^ 0x63637` at `+0xc`.
    fn find_tcoord_z(&mut self, x: i32, y: i32);
    /// `Object::remove_from_inside(this)`.
    fn remove_from_inside(&mut self);
}

/// The tail every placing arm shares (`618657`..`6186bc`).
fn finish<V: ComeOutView>(v: &mut V, x: i32, y: i32) -> Exit {
    v.set_pos(x, y);
    v.find_tcoord_z(x, y);
    v.remove_from_inside();
    Exit::Place { x, y }
}

/// `Unit::come_out@00617c10` from its entry to `set_new_location`.
pub fn come_out_steps<V: ComeOutView>(v: &mut V, param1: i32) -> Exit {
    v.group_clear();
    for i in 0..v.member_count() {
        v.clear_member(i);
    }
    if param1 == 0 && !v.is_captain() {
        return Exit::Recurse;
    }
    let (host, player) = v.get_inside();
    if host < 0 {
        // `617d27`: the unit's own point, ringed from its own block radius.
        let block = v.block_radius();
        let max = v.disembark().wrapping_add(block);
        let (x, y) = v.pos();
        let at = |a12| Find {
            x,
            y,
            min: block,
            max,
            angle: SOUTH,
            filter: FILTER_NOT_ME,
            a12,
            a13: 0,
            gather: false,
        };
        let r = v.find(at(0));
        if r.code == 0 {
            return Exit::Place { x: r.x, y: r.y };
        }
        let r = v.find(at(1));
        if r.code == 0 {
            return Exit::Place { x: r.x, y: r.y };
        }
        return Exit::Place { x, y };
    }
    if v.host_is(player, host, IS_A6) && v.domain() == 0 {
        // `617e83`: a barge. A refused one is `return 1`.
        return if v.barge(player, host) < 0 {
            Exit::Ret(1)
        } else {
            Exit::Barge
        };
    }
    if v.missile() {
        let (x, y) = v.object_pos(player, host);
        v.set_figure_90();
        return finish(v, x, y);
    }
    let (mut host, mut player) = (host, player);
    if !v.is_captain() {
        host = v.get_captain();
        player = v.owner();
    }
    if v.v20(player, host) {
        let r = v.host_is(player, host, IS_A4) || v.host_is(player, host, IS_A6);
        if r {
            v.set_leader_flag();
        }
    }
    v.v34(player, host);
    let mut angle = SOUTH;
    if host >= 0 && v.v20(player, host) {
        let bd = v.build_data(player, host);
        if v.bd_has_list(bd) {
            let bd = v.build_data(player, host);
            if !v.bd_gather_inside(bd) {
                let bd = v.build_data(player, host);
                v.bd_head(bd);
                let bd = v.build_data(player, host);
                let node = v.bd_node(bd);
                let (x, y) = if node.kind == 3 {
                    let (idx, pl) = (node.a, node.b);
                    if idx >= 0 && v.is_captain() && pl < 8 {
                        v.copy_hotkeys(idx, pl);
                    }
                    v.object_pos(pl, idx)
                } else {
                    (node.a, node.b)
                };
                let f = if v.block_radius() == 0 {
                    Find {
                        x,
                        y,
                        min: 0,
                        max: GATHER_REACH,
                        angle: GATHER_BIAS,
                        filter: FILTER_ALL,
                        a12: 0,
                        a13: 1,
                        gather: true,
                    }
                } else {
                    Find {
                        x,
                        y,
                        min: 0,
                        max: GATHER_REACH,
                        angle: GATHER_BIAS,
                        filter: FILTER_NOT_ME,
                        a12: 0,
                        a13: 0,
                        gather: true,
                    }
                };
                let r = v.find(f);
                if r.code == 0 {
                    let (hx, hy) = v.object_pos(player, host);
                    angle = v.find_angle(r.x.wrapping_sub(hx), r.y.wrapping_sub(hy));
                    v.set_unit_angle(angle);
                }
            }
        }
    }
    // `618381`: the ring. A building host's is its footprint plus the train
    // distance; a unit host's is its own radius out the disembark distance.
    let (min, max);
    if v.v1c(player, host) {
        let (xs, ys) = v.object_size(player, host);
        let foot = xs.wrapping_add(ys).wrapping_mul(0x30);
        let (ring, far, near) = if v.domain() == 1 {
            let (near, far) = v.boat_train();
            (
                foot.wrapping_add(v.big_radius()).wrapping_add(near),
                far,
                near,
            )
        } else {
            let (near, far) = v.train();
            (foot.wrapping_add(near), far, near)
        };
        max = ring.wrapping_add(far.wrapping_sub(near));
        min = if v.object_alive(player, host) {
            ring
        } else {
            0
        };
    } else {
        angle = v.host_angle(player, host);
        min = v.host_block_radius(player, host);
        max = v.disembark().wrapping_add(min);
    }
    let (hx, hy) = v.object_pos(player, host);
    let ring = |min, max, a12, filter| Find {
        x: hx,
        y: hy,
        min,
        max,
        angle,
        filter,
        a12,
        a13: 0,
        gather: false,
    };
    if v.block_radius() == 0 {
        let r = v.find(ring(min, max, 0, FILTER_ALL));
        if r.code == 0 {
            return finish(v, r.x, r.y);
        }
        let (hx, hy) = v.object_pos(player, host);
        let r = v.find(Find {
            x: hx,
            y: hy,
            ..ring(min.wrapping_mul(2), max.wrapping_mul(2), 1, FILTER_ALL)
        });
        if r.code == 0 {
            return finish(v, r.x, r.y);
        }
        let (x, y) = v.object_pos(player, host);
        finish(v, x, y)
    } else {
        let r = v.find(ring(min, max, 0, FILTER_NOT_ME));
        if r.code == 0 {
            return finish(v, r.x, r.y);
        }
        let (hx, hy) = v.object_pos(player, host);
        let r = v.find(Find {
            x: hx,
            y: hy,
            ..ring(min, max, 1, FILTER_NOT_ME)
        });
        if r.code == 0 {
            finish(v, r.x, r.y)
        } else {
            Exit::Ret(1)
        }
    }
}

// ---------------------------------------------------------------------------
// The crate's view
// ---------------------------------------------------------------------------

use crate::Sim;
use crate::attrition::Domain;
use crate::combat::Obj;
use crate::movement::{Angle, find_angle};
use crate::orders::Coll;
use crate::world::Pos;

/// A host handle that names a unit, not a building: the original's one
/// objects array holds both, the crate's two vectors do not.
const UNIT_HOST: i32 = 1 << 30;

/// [`ComeOutView`] over a [`Sim`] for the two hosts the port walks: a
/// building a captain leaves ([`SimComeOut::building`]) and the unit a squad
/// member or a passenger is seated round ([`SimComeOut::unit_host`]), the
/// listing's arm after the captain redirect.
///
/// Not modelled, each a leaf the port has no state for: the figures' `+0xd0` word,
/// the leader's `0x2000000` flag, the figure's `+0x40` 90.0 for a missile,
/// the barge a land unit leaving an Oil Platform gets (`host_is(0x1a6)` is
/// false here), and a gather point on an object (`kind 3`, which no capture
/// sets). The placement itself — `set_pos`, `find_tcoord_z`,
/// `remove_from_inside` — is [`Sim::come_out_place`]'s, after the walk.
pub(crate) struct SimComeOut<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) unit: usize,
    host: i32,
    /// The gather block's spot and the bearing it turned the unit to.
    pub(crate) gather: Option<(Pos, Angle)>,
}

impl<'a> SimComeOut<'a> {
    /// `unit` (a squad's captain) leaving building `b`.
    pub(crate) fn building(sim: &'a mut Sim, unit: usize, b: usize) -> Self {
        Self {
            sim,
            unit,
            host: b as i32,
            gather: None,
        }
    }

    /// `unit` seated round the unit `host`.
    pub(crate) fn unit_host(sim: &'a mut Sim, unit: usize, host: usize) -> Self {
        Self {
            sim,
            unit,
            host: UNIT_HOST | host as i32,
            gather: None,
        }
    }

    /// The walk's answer as the port's spot: where `set_new_location` is
    /// reached, or none for a refused sweep.
    pub(crate) fn run(&mut self) -> Option<Pos> {
        match come_out_steps(self, 1) {
            Exit::Place { x, y } => Some(Pos::new(x, y)),
            _ => None,
        }
    }

    fn building_of(&self, h: i32) -> usize {
        debug_assert!(h & UNIT_HOST == 0);
        h as usize
    }
}

impl ComeOutView for SimComeOut<'_> {
    fn group_clear(&mut self) {}
    fn member_count(&self) -> i32 {
        0
    }
    fn clear_member(&mut self, _i: i32) {}
    fn is_captain(&mut self) -> bool {
        true
    }
    fn get_inside(&mut self) -> (i32, i32) {
        (self.host, i32::from(self.sim.units[self.unit].owner))
    }
    fn pos(&self) -> (i32, i32) {
        let p = self.sim.units[self.unit].pos;
        (p.x, p.y)
    }
    fn owner(&self) -> i32 {
        i32::from(self.sim.units[self.unit].owner)
    }
    fn block_radius(&self) -> i32 {
        self.sim.profile(Obj::Unit(self.unit)).block_radius
    }
    fn domain(&self) -> i32 {
        match self.sim.unit_domain_of(self.unit) {
            Domain::Land => 0,
            Domain::Sea => 1,
            _ => 2,
        }
    }
    fn big_radius(&self) -> i32 {
        self.sim.profile(Obj::Unit(self.unit)).big_radius
    }
    fn missile(&self) -> bool {
        self.sim
            .profile(Obj::Unit(self.unit))
            .has(crate::combat::mask::MISSILE)
    }
    fn disembark(&self) -> i32 {
        self.sim.tuning.unit_disembark_distance
    }
    fn train(&self) -> (i32, i32) {
        (
            self.sim.tuning.unit_train_distance,
            self.sim.tuning.unit_train_max_distance,
        )
    }
    fn boat_train(&self) -> (i32, i32) {
        (
            self.sim.tuning.boat_train_distance,
            self.sim.tuning.boat_train_max_distance,
        )
    }
    fn find(&mut self, f: Find) -> Found {
        let at = Pos::new(f.x, f.y);
        let angle = Angle(f.angle);
        // **`FILTER_ALL`'s general test is a seam** (`docs/CITIES.md` §11,
        // `docs/ORDERS.md` §10): both of its sweeps run collision-off, and
        // `FILTER_NOT_ME`'s second sweep is the collision-off flag.
        let spot = if f.filter == FILTER_ALL || f.a12 != 0 {
            self.sim
                .find_nearby_spot_coll(self.unit, at, f.min, f.max, 0, angle, None, Coll::None)
        } else {
            self.sim
                .find_nearby_spot(self.unit, at, f.min, f.max, 0, angle, None)
        };
        if f.gather {
            // Kept for the routing, which is handed the gather block's spot.
            self.gather = spot.map(|s| (s, angle));
        }
        match spot {
            Some(s) => Found {
                code: 0,
                x: s.x,
                y: s.y,
            },
            None => Found {
                code: 1,
                x: 0,
                y: 0,
            },
        }
    }
    fn host_is(&mut self, _player: i32, _h: i32, _id: i32) -> bool {
        false
    }
    fn barge(&mut self, _player: i32, _h: i32) -> i32 {
        -1
    }
    fn object_pos(&self, _player: i32, h: i32) -> (i32, i32) {
        let p = if h & UNIT_HOST != 0 {
            self.sim.units[(h & !UNIT_HOST) as usize].pos
        } else {
            self.sim.buildings[self.building_of(h)].pos
        };
        (p.x, p.y)
    }
    fn set_figure_90(&mut self) {}
    fn v20(&mut self, _player: i32, h: i32) -> bool {
        h & UNIT_HOST == 0
    }
    fn v34(&mut self, _player: i32, _h: i32) {}
    fn v1c(&mut self, _player: i32, h: i32) -> bool {
        h & UNIT_HOST == 0
    }
    fn get_captain(&mut self) -> i32 {
        UNIT_HOST | self.sim.captain_of(self.unit) as i32
    }
    fn set_leader_flag(&mut self) {}
    fn build_data(&mut self, _player: i32, h: i32) -> i32 {
        h
    }
    fn bd_has_list(&self, bd: i32) -> bool {
        // Not modelled: a point on an object (`kind 3`).
        self.sim
            .first_gather(self.building_of(bd))
            .is_some_and(|p| p.action != 3)
    }
    fn bd_gather_inside(&mut self, bd: i32) -> bool {
        self.sim.gather_inside(self.building_of(bd))
    }
    fn bd_head(&mut self, _bd: i32) {}
    fn bd_node(&self, bd: i32) -> Node {
        let p = self
            .sim
            .first_gather(self.building_of(bd))
            .expect("a head the walk checked");
        Node {
            kind: i32::from(p.action),
            a: p.pos.x,
            b: p.pos.y,
        }
    }
    fn copy_hotkeys(&mut self, _a: i32, _b: i32) {}
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
        find_angle(dx, dy).0
    }
    fn set_unit_angle(&mut self, a: i32) {
        self.sim.units[self.unit].movement.heading = Angle(a);
        self.gather = self.gather.map(|(s, _)| (s, Angle(a)));
    }
    fn object_alive(&self, _player: i32, h: i32) -> bool {
        h & UNIT_HOST != 0 || self.sim.buildings[self.building_of(h)].alive
    }
    fn object_size(&self, _player: i32, h: i32) -> (i32, i32) {
        self.sim.buildings[self.building_of(h)]
            .ty
            .map_or((0, 0), |t| {
                (
                    self.sim.build_types[t].x_size,
                    self.sim.build_types[t].y_size,
                )
            })
    }
    fn host_angle(&self, _player: i32, h: i32) -> i32 {
        self.sim.units[(h & !UNIT_HOST) as usize].movement.heading.0
    }
    fn host_block_radius(&self, _player: i32, h: i32) -> i32 {
        self.sim
            .profile(Obj::Unit((h & !UNIT_HOST) as usize))
            .block_radius
    }
    fn set_pos(&mut self, _x: i32, _y: i32) {}
    fn find_tcoord_z(&mut self, _x: i32, _y: i32) {}
    fn remove_from_inside(&mut self) {}
}
