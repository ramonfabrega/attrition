//! **`Unit::find_path@005fb910`** as steps over a [`FindPathView`] (item 3386,
//! parked 3348c): the straight-line planner every walking unit asks before it
//! takes a step (`docs/PATHFINDER.md` §4.6, §9).
//!
//! The steps are the listing's, call for call and read for read, in the
//! listing's order, so that the sweep (`sweep/unit_find_path.rs`) can hold the
//! port to the emulated original's stub calls as well as to its writes. The
//! crate's [`crate::Sim`] answers the view from its own units
//! (`orders.rs`, `SimFindPath`); the sweep answers it from the emulated
//! original's windows.
//!
//! What the sweep forced into the steps, none of it in the crate's older
//! hand-written march:
//!
//! * **the domain test.** Past the pull-back the original returns `0` for a
//!   type whose `domain` (`type+0x218`) is air (`2`) — before the
//!   goal-on-top-of-us arm, so a plane's goal that has walked back onto it
//!   answers `0`, not `1`, and writes no `avoid`.
//! * **the last tile checked.** The march asks `invalid_loc` only when its
//!   tile differs from the last one *asked about*, which starts at `(-1, -1)`
//!   — not from the tile it stepped off, so the first step is always asked.
//!
//! Not modelled: a `pop` of an **empty** path stack (the tail of the detour's
//! replay) reads a stale slot in the original and pushes it back; the view
//! answers `None` and the steps skip both. The sweep's view answers the stale
//! slot, as the original does, so the seam is the Sim adapter's alone.

use crate::world::Pos;

/// `PathData`: one entry of the unit's path stack, its flags a whole word as
/// the original keeps them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathEntry {
    pub to: Pos,
    pub tolerance: i32,
    pub flags: u32,
}

/// `TURN_FIRST`: the entry's "turn in place" bit.
pub const TURN_FIRST: u32 = 0x4;
/// `unit_masks & 8`, the unit's "line verified" bit.
pub const LINE_OK: u32 = 0x8;
/// More than this many cells of Manhattan distance is a plan, not a walk.
pub const STRAIGHT_LINE_CELLS: i32 = 4;
/// The recursion depth at which the original stops asking for a detour.
pub const MAX_RECURSION: u8 = 10;
/// `type+0x218 == 2`: an air type (`domain`).
pub const DOMAIN_AIR: i32 = 2;

/// What `find_path` reads and does. Every method is one call or one inline
/// read of the listing; positions are world coordinates, and the view turns
/// them into the tiles and cells the original indexes `div_3_table` with.
pub trait FindPathView {
    /// `type+0x2b4 & 0x20`: flies like a helicopter.
    fn helicopter(&mut self) -> bool;
    /// The unit's position, decoded.
    fn here(&mut self) -> Pos;
    /// `UnitData::invalid_loc(tile of at, ignore_buildings, fog_relax, 0, 0, 0)`.
    fn invalid_loc(&mut self, at: Pos, ignore_buildings: bool, fog_relax: bool) -> bool;
    /// `UnitType::find_nearby_spot(here, ..)` then `set_new_location(spot, 1, 0)`.
    fn relocate(&mut self);
    /// `update_order` and its vslot `+0x40`: the order whose waypoint
    /// (`+0x2c`) and last plan (`+0x34`) the rest reads.
    fn order_begin(&mut self);
    /// Manhattan distance, in cells, from the unit to `to`.
    fn cells_apart(&mut self, to: Pos) -> i32;
    /// `vslot +0x17c (x, y, 0)` of the unit: its step (before the floor of 3).
    fn speed(&mut self) -> i32;
    /// `find_angle(dx, dy)`.
    fn angle(&mut self, dx: i32, dy: i32) -> i32;
    /// The x component of a step: `sin_table` of the angle folded as the
    /// original folds it.
    fn sin(&mut self, angle: i32, distance: i32) -> i32;
    /// The y component: `cosx`, and in the march `sin_table(angle + 0x40000000)`.
    fn cos(&mut self, angle: i32, distance: i32) -> i32;
    /// `path_recursion += 1`, saturating at `0xff`.
    fn bump_recursion(&mut self);
    /// `path_recursion`.
    fn recursion(&mut self) -> u8;
    /// The order's waypoint (`+0x2c`, `+0x30`).
    fn waypoint(&mut self) -> Pos;
    fn set_waypoint(&mut self, to: Pos);
    /// The order's last plan (`+0x34`, `+0x38`); `(-1, -1)` for none.
    fn last_plan(&mut self) -> Pos;
    fn set_last_plan(&mut self, to: Pos);
    /// `Stack<PathData>::length` of the unit's path.
    fn path_len(&mut self) -> i32;
    /// `pop` of the unit's path; `None` is the empty stack (see the header).
    fn path_pop(&mut self) -> Option<PathEntry>;
    fn path_push(&mut self, e: PathEntry);
    /// `peek` of the unit's path.
    fn path_peek(&mut self) -> PathEntry;
    /// `avoid_x/y = at`.
    fn set_avoid(&mut self, at: Pos);
    /// `type+0x218`.
    fn domain(&mut self) -> i32;
    /// `Unit::go_around_building(at, angle, speed)`.
    fn go_around(&mut self, at: Pos, angle: i32, speed: i32);
    /// The recursive `find_path(to)`.
    fn recurse(&mut self, to: Pos) -> i32;
    /// `unit_masks & 8`.
    fn line_ok(&mut self) -> bool;
    fn set_line_ok(&mut self);
    /// `(mask(a) & 0x30 == 0x20) != (mask(b) & 0x30 == 0x20)`: the shore
    /// lies between the two tiles.
    fn shore_between(&mut self, a: Pos, b: Pos) -> bool;
}

fn flagged<V: FindPathView + ?Sized>(v: &mut V, mut e: PathEntry, other: Pos) -> PathEntry {
    if v.shore_between(e.to, other) {
        e.flags |= TURN_FIRST;
    } else {
        e.flags &= !TURN_FIRST;
    }
    e
}

/// `Unit::find_path(goal)`: `0` walk straight, `1` plan, `2` abort (only ever
/// a recursive call's answer passed up).
pub fn find_path_steps<V: FindPathView + ?Sized>(v: &mut V, goal: Pos) -> i32 {
    if v.helicopter() {
        return 0;
    }
    let here = v.here();
    if v.invalid_loc(here, true, false) {
        v.relocate();
        return 1;
    }
    if goal == here {
        return 0;
    }
    v.order_begin();
    if v.cells_apart(goal) > STRAIGHT_LINE_CELLS && !v.invalid_loc(goal, true, true) {
        return 1;
    }
    let ang = v.angle(goal.x - here.x, goal.y - here.y);
    let speed = v.speed().max(3);
    let mut back_x = v.sin(ang, speed);
    let mut back_y = v.cos(ang, speed);
    v.bump_recursion();
    let mut goal = goal;
    // THE PULL-BACK: a goal whose own tile refuses is walked back toward us.
    let mut dx = goal.x - here.x;
    let mut dy = goal.y - here.y;
    let mut blocked = v.invalid_loc(goal, false, false);
    while blocked {
        if dx.abs() < back_x.abs() {
            back_x = dx;
        }
        if dy.abs() < back_y.abs() {
            back_y = -dy;
        }
        let wp = v.waypoint();
        if wp == goal {
            v.set_waypoint(Pos::new(wp.x - back_x, wp.y + back_y));
        }
        if v.path_len() != 0
            && let Some(mut top) = v.path_pop()
        {
            if top.to == goal {
                top.to = Pos::new(top.to.x - back_x, top.to.y + back_y);
            }
            v.path_push(top);
        }
        goal = Pos::new(goal.x - back_x, goal.y + back_y);
        dx = goal.x - here.x;
        dy = goal.y - here.y;
        if back_x == 0 && back_y == 0 {
            break;
        }
        blocked = v.invalid_loc(goal, false, false);
    }
    if v.domain() == DOMAIN_AIR {
        return 0;
    }
    if dx == 0 && dy == 0 {
        v.set_avoid(goal);
        return 1;
    }
    // THE MARCH.
    let mut at = here;
    let mut manh = dx.abs() + dy.abs();
    let mut prev = manh;
    let mut checked = Pos::new(-1, -1);
    while speed < manh {
        if prev < manh {
            return 0;
        }
        prev = manh;
        let ang = v.angle(dx, dy);
        let mut sx = v.sin(ang, speed);
        let mut cy = v.cos(ang, speed);
        if dx.abs() < sx.abs() {
            sx = dx;
        }
        if dy.abs() < cy.abs() {
            cy = -dy;
        }
        let from = at;
        at = Pos::new(at.x + sx, at.y - cy);
        let tile = at.tile();
        if tile != checked {
            checked = tile;
            if v.invalid_loc(at, false, false) {
                if v.recursion() < MAX_RECURSION {
                    let n0 = v.path_len();
                    v.go_around(from, ang, speed);
                    if v.recursion() != MAX_RECURSION {
                        return detour(v, goal, n0);
                    }
                }
                return 1;
            }
        }
        dx = goal.x - at.x;
        dy = goal.y - at.y;
        if dx.abs() <= sx.abs() && dy.abs() <= cy.abs() {
            return 0;
        }
        manh = dx.abs() + dy.abs();
    }
    0
}

/// The detour's verification (`5fbfaf`..`5fc12e`, the replay `5fc134`..): the
/// entries `go_around_building` pushed are lifted off the path, the first
/// point of the detour is asked of `find_path`, and a detour that holds is
/// put back with its turn bits recomputed.
fn detour<V: FindPathView + ?Sized>(v: &mut V, goal: Pos, n0: i32) -> i32 {
    let here = v.here();
    let top = v.path_peek();
    let n1 = v.path_len();
    let mut w = top.to;
    let mut below = top.to;
    let mut saved: Vec<PathEntry> = Vec::new();
    let mut i = n0;
    while i < n1 {
        let Some(e) = v.path_pop() else {
            break;
        };
        below = e.to;
        if w == here && i < n1 - 1 {
            let p = v.path_peek();
            w = p.to;
            below = p.to;
        } else {
            saved.push(e);
        }
        i += 1;
    }
    let last = v.last_plan();
    if w == here || w == last || below == here || below == last || here == last || goal == w {
        return 1;
    }
    let r = v.recurse(w);
    if r != 0 {
        return if r == 2 { 2 } else { 1 };
    }
    if !v.line_ok() {
        while let Some(e) = saved.pop() {
            if v.path_len() != 0
                && let Some(f) = v.path_pop()
            {
                let f = flagged(v, f, e.to);
                v.path_push(f);
            }
            v.path_push(e);
        }
        // The recursive call may have moved the unit: read it afresh.
        let now = v.here();
        v.set_last_plan(now);
        if let Some(t) = v.path_pop() {
            let t = flagged(v, t, now);
            v.path_push(t);
        }
        v.set_line_ok();
    }
    0
}
