//! `Unit::go_around_building@005fc350` as steps over a view (item 3330,
//! `docs/ORDERS.md` §4.6): the tile-edge walk that answers a straight line
//! clipping a building. [`go_around_building_steps`] is the original's
//! control flow in its order; [`DetourView`] is the state it reads and the
//! callees it makes, which [`SimDetour`] answers from the crate's [`Sim`]
//! and `sweep/unit_go_around_building.rs` from the emulated original's
//! windows. The method's own account is on [`Sim::go_around_building`].

use crate::movement::{self, Angle};
use crate::orders::{MoveOrder, PathData, path_flag};
use crate::{Pos, Sim};

/// What `go_around_building` reads of the unit and its move order, and the
/// callees it makes.
pub trait DetourView {
    /// `UnitData::invalid_loc(tile, 0, 0, 0, 0, 0)` nonzero — may this unit
    /// not stand on the tile.
    fn refuses(&mut self, tile: Pos) -> bool;
    /// The unit's own position (`+0x10`/`+0x14`, descrambled).
    fn here(&self) -> Pos;
    /// The world's `+0x18`/`+0x1c`: its extent in tiles.
    fn world_tiles(&self) -> (i32, i32);
    /// The move order's `+0x2c`/`+0x30` waypoint.
    fn waypoint(&self) -> Pos;
    /// The move order's `+0x4c`/`+0x4e` shorts: the destination's offset
    /// inside its world cell.
    fn off(&self) -> Pos;
    /// `Stack<PathData>::push` on the unit's path.
    fn push(&mut self, p: PathData);
    /// The refusal: the order's `+0x10` zeroed, the verified-line bit
    /// (`unit_masks & 8`) cleared, `path_recursion = 10`.
    fn gave_up(&mut self);
}

/// The `DETOUR` midpoint between the two lanes: the mean of the two tile
/// centres on each axis, biased one unit *down* whenever the turn-in centre
/// is the larger, so the point falls inside the tile the unit is leaving
/// rather than on the edge between them.
fn detour_mid(cx: i32, cy: i32, other_x: i32, other_y: i32) -> Pos {
    let mut m = Pos::new((other_x + cx) / 2, (other_y + cy) / 2);
    if cx != other_x && cx - other_x >= 0 {
        m.x -= 1;
    }
    if cy != other_y && cy - other_y >= 0 {
        m.y -= 1;
    }
    m
}

/// One direction of the edge walk. Returns whether the blocked lane was
/// found open, and the point the walk stopped at.
fn edge_walk<V: DetourView>(v: &mut V, from: Pos, step: Pos, lt: Pos) -> (bool, Pos) {
    let (tw, th) = v.world_tiles();
    let (w, h) = (tw * 0xc0, th * 0xc0);
    let mut at = from;
    loop {
        let next = Pos::new(at.x + step.x * 0xc0, at.y + step.y * 0xc0);
        if next.x < 0 || next.y < 0 || next.x >= w || next.y >= h {
            return (false, at);
        }
        at = next;
        let cur = at.tile();
        // The unit's own lane keeps the coordinate the walk does not move;
        // the blocked lane is the tile the walk is standing on.
        let lane = if step.x == 0 {
            Pos::new(lt.x, cur.y)
        } else {
            Pos::new(cur.x, lt.y)
        };
        if v.refuses(lane) {
            return (!v.refuses(cur), at);
        }
        if !v.refuses(cur) {
            return (true, at);
        }
    }
}

/// `go_around_building(last, ang, spd)`; see [`Sim::go_around_building`].
pub fn go_around_building_steps<V: DetourView>(v: &mut V, last: Pos, ang: Angle, spd: i32) {
    let lt = last.tile();
    // The blocked point, recomputed here from an *unclamped* step: the march
    // clamps each component to its remainder, this does not, so the two need
    // not agree. That is the original's, not a slip.
    let sx = movement::sin_component(ang, spd);
    let sy = -movement::cos_component(ang, spd);
    let blocked = Pos::new(last.x + sx, last.y + sy);
    let bt = blocked.tile();

    // Which lane is which. `cmp` is the column the diagonal case keeps;
    // `settled` is the original's `goto LAB_005fc4c0`.
    let mut cmp = bt.x;
    let mut settled = lt.x == bt.x;
    if !settled && lt.y != bt.y {
        // A diagonal entry. More than two tiles at once is not a step this
        // can reason about.
        if (lt.y - bt.y).abs() + (lt.x - bt.x).abs() > 2 {
            return v.gave_up();
        }
        if v.refuses(Pos::new(lt.x, bt.y)) {
            if v.refuses(Pos::new(bt.x, lt.y)) {
                // Both corners of the diagonal refuse: shorten the step and
                // ask again, which may land orthogonally instead.
                cmp = bt.x;
                if spd > 1 {
                    return go_around_building_steps(v, last, ang, spd - 1);
                }
            } else {
                cmp = lt.x;
            }
            settled = true;
        }
    }
    let vertical = !settled || lt.y == bt.y || (lt.x != cmp && sx.abs() < sy.abs());
    let step = if vertical {
        Pos::new(0, if sy > 0 { 1 } else { -1 })
    } else {
        Pos::new(if sx > 0 { 1 } else { -1 }, 0)
    };
    let back = Pos::new(-step.x, -step.y);
    let (fwd_ok, fwd) = edge_walk(v, blocked, step, lt);
    let (bwd_ok, bwd) = edge_walk(v, blocked, back, lt);

    let here = v.here();
    let centre = |t: i32| t * 0xc0 + 0x60;
    let at_unit = |p: Pos| {
        let t = p.tile();
        here.x == centre(t.x) && here.y == centre(t.y)
    };
    let (pick, pstep) = if !fwd_ok {
        if !bwd_ok {
            return v.gave_up();
        }
        (bwd, back)
    } else if !bwd_ok {
        (fwd, step)
    } else if at_unit(fwd) {
        (bwd, back)
    } else if at_unit(bwd) {
        (fwd, step)
    } else {
        let wp = v.waypoint();
        let reach = |p: Pos| (p.x - wp.x).abs() + (p.y - wp.y).abs();
        if reach(bwd) < reach(fwd) {
            (bwd, back)
        } else {
            (fwd, step)
        }
    };

    if (pick.x - last.x).abs() >= 0x301 || (pick.y - last.y).abs() >= 0x301 {
        return v.gave_up();
    }

    // The waypoints. `off` is the order's own destination offset inside its
    // world cell (§4.1); the halved tile part of it is what places every
    // detour point off its tile's centre.
    let off = v.off();
    let skew = |x: i32| (x % 0xc0) / 2;
    let ct = pick.tile();
    let mut pushes: Vec<PathData> = Vec::new();
    let target = if pstep.x == 0 {
        let mut ty = ct.y;
        if v.refuses(Pos::new(lt.x, ty)) {
            let (cx, cy) = (centre(ct.x), centre(ty));
            pushes.push(PathData {
                to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                tolerance: 0,
                flags: 0,
            });
            ty -= pstep.y;
            let (lane_cx, back_cy) = (centre(lt.x), centre(ty));
            pushes.push(PathData {
                to: detour_mid(cx, cy, lane_cx, back_cy),
                tolerance: 0,
                flags: path_flag::DETOUR,
            });
        }
        Pos::new(lt.x, ty)
    } else {
        let mut tx = ct.x;
        if v.refuses(Pos::new(tx, lt.y)) {
            let (cx, cy) = (centre(tx), centre(ct.y));
            pushes.push(PathData {
                to: Pos::new(skew(off.x) - 0x30 + cx, skew(off.y) - 0x30 + cy),
                tolerance: 0,
                flags: 0,
            });
            tx -= pstep.x;
            let (back_cx, lane_cy) = (centre(tx), centre(lt.y));
            pushes.push(PathData {
                to: detour_mid(cx, cy, back_cx, lane_cy),
                tolerance: 0,
                flags: path_flag::DETOUR,
            });
        }
        Pos::new(tx, lt.y)
    };
    pushes.push(PathData {
        to: Pos::new(
            skew(off.x) + target.x * 0xc0 + 0x30,
            skew(off.y) + target.y * 0xc0 + 0x30,
        ),
        tolerance: 0,
        flags: 0,
    });
    for p in pushes {
        v.push(p);
    }
}

/// The crate's [`Sim`] as the view: unit `u` and its move order `mo`.
pub(crate) struct SimDetour<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) u: usize,
    pub(crate) mo: &'a mut MoveOrder,
}

impl DetourView for SimDetour<'_> {
    fn refuses(&mut self, tile: Pos) -> bool {
        self.sim
            .invalid_loc(self.u, tile, false, false, false, false, false)
            != 0
    }
    fn here(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn world_tiles(&self) -> (i32, i32) {
        (
            self.sim.world.width() * crate::world::TILES_PER_CELL,
            self.sim.world.height() * crate::world::TILES_PER_CELL,
        )
    }
    fn waypoint(&self) -> Pos {
        self.mo.waypoint
    }
    fn off(&self) -> Pos {
        Pos::new(
            self.mo.dest.x.rem_euclid(0x300),
            self.mo.dest.y.rem_euclid(0x300),
        )
    }
    fn push(&mut self, p: PathData) {
        self.sim.units[self.u].path.push(p);
    }
    fn gave_up(&mut self) {
        self.mo.has_waypoint = false;
        self.sim.units[self.u].line_ok = false;
        self.sim.units[self.u].path_recursion = 10;
    }
}
