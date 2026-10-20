//! **`Group::action_gather_point@006ff1b0`'s steps** (item 3358): the
//! rally-point command over a group of buildings, as a pure walk over a
//! [`GatherView`] (the swept steps read the world only through it, so the
//! sweep's port is a view over the emulated original's windows and the
//! crate's is [`SimGather`]; `docs/PRODUCTION.md`, "The gather point").
//!
//! The control flow is the original's, call for call: every leaf is one
//! virtual call (or one direct callee, or one inline read) of the listing,
//! asked in the order the listing asks it — which is what the sweep holds.

use crate::Sim;
use crate::build::{self, Ident};
use crate::rally::GatherPoint;
use crate::world::{Player, Pos, UNITS_PER_CELL};

/// `is(0x1a4, 0)`: a University (never a member).
pub(crate) const IS_UNIVERSITY: i32 = 0x1a4;
/// `is(0x208, 0)`: a Missile Silo (never a member).
pub(crate) const IS_MISSILE_SILO: i32 = 0x208;
/// `is(0x1bf, 0)`: an Airbase.
pub(crate) const IS_AIRBASE: i32 = 0x1bf;
/// `is(0x1b6, 0)`: a Senate.
pub(crate) const IS_SENATE: i32 = 0x1b6;
/// `is(0x211, 0)`: a wonder that takes the point and clears (the Kremlin or
/// the Terracotta Army; no ident here).
pub(crate) const IS_WONDER_A: i32 = 0x211;
/// `is(0x21a, 0)`: its sibling.
pub(crate) const IS_WONDER_B: i32 = 0x21a;
/// The City centre `GroupData::count(COUNT_TYPE, …)` and the snap ask for.
pub(crate) const VILLAGE: i32 = 0x19e;
/// The Woodcutter the snap asks for on a forest tile.
pub(crate) const WOODCUTTER: i32 = 0x1a2;
/// The Mine the snap asks for on a mountain tile.
pub(crate) const MINE: i32 = 0x1a3;
/// `QUEUE_NEW`; `QUEUE_NEW - 1` is the add-to-end (`QUEUE_LAST`).
pub(crate) const QUEUE_NEW: i32 = 2;

/// The messages the command prints (`loc_str_array_orig + OFFSET`).
pub(crate) const MSG_CLEARED: i32 = 0x8548;
pub(crate) const MSG_INSIDE: i32 = 0x855c;
pub(crate) const MSG_OBJECT: i32 = 0x8570;
pub(crate) const MSG_GROUND: i32 = 0x8584;

/// What `Group::action_gather_point` asks of the group, its members and the
/// world. `H` names a member (the original's short in the group's array).
pub(crate) trait GatherView {
    type H: Copy;

    /// The first line: `(**this + 0x14)()`, its answer ignored.
    fn refresh(&mut self);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> Self::H;
    /// `[this + 0x49] != 0`.
    fn armed(&self) -> bool;
    /// `[objects + who * 0x1c + 0x14]`'s record `+8 & 1`: no call.
    fn alive_flag(&mut self, h: Self::H) -> bool;
    /// The member's slot `+0xc`.
    fn alive_slot(&mut self, h: Self::H) -> bool;
    /// The member's slot `+0x20`.
    fn is_building(&mut self, h: Self::H) -> bool;
    /// `is(id, 0)` on the member's data (`slot 0xac`, then the `Build`'s
    /// type slot `+0x60` or the data's own `+0xb8`).
    fn is_ident(&mut self, h: Self::H, id: i32) -> bool;
    /// `BuildData::gather_inside@0046f180` on the member's data.
    fn gather_inside(&mut self, h: Self::H) -> bool;
    /// `Build::clear_gather@00623180`.
    fn clear_gather(&mut self, h: Self::H);
    /// `Build::add_gather_point@00622e70(x, y, action, pos)`.
    fn add_gather_point(&mut self, h: Self::H, x: i32, y: i32, action: i32, pos: i32);
    /// `world->w * 0x300`, `world->h * 0x300`.
    fn world_extent(&self) -> (i32, i32);
    /// The tile word at the position (`div_3_table[y >> 6] * stride +
    /// div_3_table[x >> 6]`).
    fn tile_mask(&mut self, x: i32, y: i32) -> i32;
    /// `WallData::covers_tile@006439b0` on the member's data, handed the
    /// position's two tile coordinates.
    fn covers_tile(&mut self, h: Self::H, x: i32, y: i32) -> bool;
    /// `GroupData::count(COUNT_TYPE, id, 0)@00711720`.
    fn count_type(&mut self, id: i32) -> i32;
    /// The group's slot `+4`.
    fn group_num(&mut self) -> i32;
    /// `ObjectsData::find_building@0065d260(x, y, SEARCH_FRIENDLY, who,
    /// 0x600, 0, FILTER_TYPE, id, 0)`: an object index, or negative.
    fn find_building(&mut self, x: i32, y: i32, id: i32) -> i32;
    /// The found object's `+0x10` and `+0x14`, `^ 0x63637`.
    fn object_pos(&mut self, idx: i32) -> (i32, i32);
    /// `console->who == the group's player`.
    fn console_is_owner(&mut self) -> bool;
    /// `Console::ping_target(idx, who, GREEN)`.
    fn ping_target(&mut self, idx: i32);
    /// The member's type is a training building: `buildtypes[basic_type]
    /// .+0x2c0 < 0` (`is_training_building@00639de0`, inline).
    fn trains(&mut self, h: Self::H) -> bool;
    /// `BuildTypeData::get_garrison_limit` of the member's data (the `Build`
    /// arm direct, the other by slot `+0x178`).
    fn garrison_limit(&mut self, h: Self::H) -> i32;
    /// `Group::add(temp, h, who, 0, 0)`: the airbase joins the flight.
    fn flight_add(&mut self, h: Self::H);
    /// The flight's member count (`[temp + 0xc]`).
    fn flight_count(&mut self) -> i32;
    /// `Group::action_flight(temp, x, y, ATTACK, 0, 0, 0)`.
    fn flight_go(&mut self, x: i32, y: i32);
    /// `String::String(loc_str + id)` then `MessageWin::add_feedback`.
    fn feedback(&mut self, id: i32);
}

/// A live building member, as every loop asks it: `alive_slot` then
/// `is_building`, then neither a University nor a Missile Silo. `flag` picks
/// the inside scan's `+8 & 1` read for the first.
fn member_ok<V: GatherView>(v: &mut V, h: V::H, flag: bool) -> bool {
    let alive = if flag {
        v.alive_flag(h)
    } else {
        v.alive_slot(h)
    };
    alive && v.is_building(h) && !v.is_ident(h, IS_UNIVERSITY) && !v.is_ident(h, IS_MISSILE_SILO)
}

/// `Group::action_gather_point@006ff1b0(x, y, action, add_to_end)`.
pub(crate) fn action_gather_point_steps<V: GatherView>(
    v: &mut V,
    mut x: i32,
    mut y: i32,
    action: i32,
    add_to_end: i32,
) {
    v.refresh();
    if x < 0 || y < 0 {
        // The Clear (`6ffeba`).
        let n = v.count();
        for i in 0..n {
            let h = v.member(i);
            if member_ok(v, h, false) {
                v.clear_gather(h);
            }
        }
        return;
    }
    let (ex, ey) = v.world_extent();
    if x >= ex {
        x = ex.wrapping_sub(1);
    }
    if y >= ey {
        y = ey.wrapping_sub(1);
    }
    // A member whose head point is "inside" turns add-to-end off.
    let mut add = add_to_end;
    let n = v.count();
    for i in 0..n {
        let h = v.member(i);
        if member_ok(v, h, true) && v.gather_inside(h) {
            add = 0;
            break;
        }
    }
    let mut inside = false;
    if action == 3 {
        // The Airbases take the point and launch one plane at it.
        let n = v.count();
        for i in 0..n {
            let h = v.member(i);
            if v.alive_slot(h) && v.is_building(h) && v.is_ident(h, IS_AIRBASE) {
                v.add_gather_point(h, x, y, 3, QUEUE_NEW - i32::from(add != 0));
                v.flight_add(h);
            }
        }
        if n > 0 && v.flight_count() != 0 {
            v.flight_go(x, y);
        }
    } else {
        // The City centres' snap (`6ff357`..`6ff4a6`).
        if v.armed() {
            let c = v.count_type(VILLAGE);
            if c == v.group_num() {
                let mask = v.tile_mask(x, y);
                let want = if mask & 0x30 == 0x30 {
                    Some(WOODCUTTER)
                } else if mask & 3 == 2 {
                    Some(MINE)
                } else {
                    None
                };
                if let Some(id) = want {
                    let idx = v.find_building(x, y, id);
                    if idx >= 0 {
                        (x, y) = v.object_pos(idx);
                        if v.console_is_owner() {
                            v.ping_target(idx);
                        }
                    }
                }
            }
        }
        // A click on a member's own footprint (`6ff4a6`..`6ff750`).
        let n = v.count();
        if v.tile_mask(x, y) & 3 == 3 && add == 0 && n > 0 {
            for i in 0..n {
                let h = v.member(i);
                if !(v.alive_slot(h)
                    && v.is_building(h)
                    && !v.is_ident(h, IS_UNIVERSITY)
                    && !v.is_ident(h, IS_MISSILE_SILO)
                    && v.covers_tile(h, x, y))
                {
                    continue;
                }
                inside = true;
                if v.is_ident(h, IS_WONDER_A)
                    || v.is_ident(h, IS_WONDER_B)
                    || v.is_ident(h, IS_SENATE)
                {
                    // The first member is cleared instead, and the command is
                    // done (`6ff736`..`6ff89b`, `6ffe95`).
                    let n = v.count();
                    for j in 0..n {
                        let k = v.member(j);
                        if member_ok(v, k, false) {
                            v.clear_gather(k);
                            v.feedback(MSG_CLEARED);
                            return;
                        }
                    }
                }
                break;
            }
        }
    }
    // Every member that takes the point (`6ffa76`..`6ffdea`).
    let mut any = false;
    let n = v.count();
    for i in 0..n {
        let h = v.member(i);
        if !member_ok(v, h, false) {
            continue;
        }
        if v.is_ident(h, IS_AIRBASE) && action == 3 {
            continue;
        }
        let takes = v.trains(h)
            || v.garrison_limit(h) != 0
            || v.is_ident(h, IS_WONDER_A)
            || v.is_ident(h, IS_WONDER_B)
            || v.is_ident(h, IS_SENATE);
        if !takes {
            continue;
        }
        if inside {
            v.add_gather_point(h, -1, -1, 0, QUEUE_NEW);
        } else {
            v.add_gather_point(h, x, y, action, QUEUE_NEW - i32::from(add != 0));
        }
        any = true;
    }
    if !(any && v.console_is_owner()) {
        return;
    }
    if inside {
        v.feedback(MSG_INSIDE);
    } else if action == 3 {
        v.feedback(MSG_OBJECT);
    } else if action == 0 {
        v.feedback(MSG_GROUND);
    }
}

/// This crate's [`GatherView`]: `who`'s `members` (building indices), and the
/// Airbases the flight has taken. Where the crate models less than the
/// original: no console (the command prints nothing and pings nothing), no
/// group `armed` flag (a group here is always armed), the group's refresh
/// slot is empty, and the two wonders beside the Senate have no ident
/// (`SEAM`; no capture reaches them).
pub(crate) struct SimGather<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) who: Player,
    pub(crate) members: &'a [usize],
    pub(crate) flight: Vec<usize>,
    pub(crate) at: Pos,
}

impl GatherView for SimGather<'_> {
    type H = usize;

    fn refresh(&mut self) {}
    fn count(&self) -> i32 {
        self.members.len() as i32
    }
    fn member(&self, i: i32) -> usize {
        self.members[i as usize]
    }
    fn armed(&self) -> bool {
        true
    }
    fn alive_flag(&mut self, h: usize) -> bool {
        self.sim.buildings[h].alive
    }
    fn alive_slot(&mut self, h: usize) -> bool {
        self.sim.buildings[h].alive
    }
    fn is_building(&mut self, _h: usize) -> bool {
        true
    }
    fn is_ident(&mut self, h: usize, id: i32) -> bool {
        let ident = self.sim.building_ident(h);
        match id {
            IS_UNIVERSITY => ident == Ident::University,
            IS_MISSILE_SILO => ident == Ident::MissileSilo,
            IS_AIRBASE => ident == Ident::Airbase,
            IS_SENATE => ident == Ident::Senate,
            _ => false,
        }
    }
    fn gather_inside(&mut self, h: usize) -> bool {
        self.sim.gather_inside(h)
    }
    fn clear_gather(&mut self, h: usize) {
        self.sim.clear_gather(h);
    }
    fn add_gather_point(&mut self, h: usize, x: i32, y: i32, action: i32, pos: i32) {
        let p = GatherPoint {
            pos: Pos::new(x, y),
            action: action as u8,
        };
        self.sim.add_gather_point(h, p, pos == QUEUE_NEW);
    }
    fn world_extent(&self) -> (i32, i32) {
        (
            self.sim.world.width() * UNITS_PER_CELL,
            self.sim.world.height() * UNITS_PER_CELL,
        )
    }
    fn tile_mask(&mut self, x: i32, y: i32) -> i32 {
        i32::from(self.sim.world.tile_mask(Pos::new(x, y).tile()))
    }
    fn covers_tile(&mut self, h: usize, x: i32, y: i32) -> bool {
        self.sim.covers_tile(h, Pos::new(x, y).tile())
    }
    fn count_type(&mut self, _id: i32) -> i32 {
        let types = &self.sim.build_types;
        self.members
            .iter()
            .filter(|&&b| {
                self.sim.buildings[b]
                    .ty
                    .is_some_and(|t| build::is(types, t, Ident::Village))
            })
            .count() as i32
    }
    fn group_num(&mut self) -> i32 {
        self.members.len() as i32
    }
    fn find_building(&mut self, x: i32, y: i32, id: i32) -> i32 {
        let ident = if id == WOODCUTTER {
            Ident::Woodcutter
        } else {
            Ident::Mine
        };
        self.sim
            .nearest_friendly_building(self.who, Pos::new(x, y), ident, crate::rally::REACH)
            .map_or(-1, |b| b as i32)
    }
    fn object_pos(&mut self, idx: i32) -> (i32, i32) {
        let p = self.sim.buildings[idx as usize].pos;
        (p.x, p.y)
    }
    fn console_is_owner(&mut self) -> bool {
        false
    }
    fn ping_target(&mut self, _idx: i32) {}
    fn trains(&mut self, h: usize) -> bool {
        self.sim.buildings[h]
            .ty
            .is_some_and(|t| build::is_training_building(&self.sim.build_types, t))
    }
    fn garrison_limit(&mut self, h: usize) -> i32 {
        self.sim.garrison_limit(h)
    }
    fn flight_add(&mut self, h: usize) {
        self.flight.push(h);
    }
    fn flight_count(&mut self) -> i32 {
        self.flight.len() as i32
    }
    fn flight_go(&mut self, x: i32, y: i32) {
        self.at = Pos::new(x, y);
        let flight = std::mem::take(&mut self.flight);
        self.sim.gather_flight(self.who, &flight, self.at);
    }
    fn feedback(&mut self, _id: i32) {}
}
