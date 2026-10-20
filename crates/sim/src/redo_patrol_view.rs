//! **`Group::redo_patrol_order@00706d90`'s steps** (item 3379): `finish_insert`'s
//! case `0x16`, the saved patrol given again to every member that took the
//! group's `QUEUE_FIRST` halt, as a pure walk over a [`RedoPatrolView`]
//! (`docs/ORDERS.md` §27). The swept steps read the world only through the
//! view, so the sweep's port is a view over the emulated original's windows
//! and the crate's is [`SimRedoPatrol`].
//!
//! **The second stack argument is dead.** The function is `ret 8`, but
//! nothing in its listing reads `[ebp + 0xc]`: `finish_insert`'s leftover `ecx`
//! (parked 3372f) is a `QueuePos` the function was declared with and never
//! looks at (the add is always `QUEUE_LAST`). The sweep draws it at random.

use crate::Sim;
use crate::group::Group;
use crate::orders::{Body, PatrolOrder, QueuePos};
use crate::world::Pos;

/// `obj_masks & 0x8000000`: a missile, which is not re-ordered.
pub const MISSILE: u32 = 0x800_0000;
/// `QUEUE_LAST` in the original's numbering.
pub const QUEUE_LAST: i32 = 1;

/// What `Group::redo_patrol_order` asks of the group, the objects and the
/// two records. `m` is a member's index (the group's short at `+0x8cc`).
pub trait RedoPatrolView {
    /// `Group::normalize`.
    fn normalize(&mut self);
    /// `[this + 0xc]`.
    fn count(&self) -> i32;
    /// The short `i` of the group's array at `+0x8cc`.
    fn member(&self, i: i32) -> i32;
    /// `objects[who][m]`'s slot `+8`.
    fn obj_alive(&mut self, m: i32) -> bool;
    /// Slot `+0xbc`: the member is on the map.
    fn obj_on_map(&mut self, m: i32) -> bool;
    /// Slot `+0xc0`: the member is a plane.
    fn obj_plane(&mut self, m: i32) -> bool;
    /// The object's data's word at `+0x1e4` (`obj_masks`).
    fn obj_masks(&mut self, m: i32) -> u32;
    /// The saved order's word at `off`: `+8` the point count, `+0x3c` the
    /// waypoint, `+0x44`/`+0x48` the leader, `+0x4c` the id, `+0x50` the form.
    fn p_word(&self, off: i32) -> i32;
    /// `[param_1 + 0x14][i]` (`axis` 0) or `[param_1 + 0x30][i]` (`axis` 1).
    fn p_point(&self, axis: i32, i: i32) -> i32;
    /// `div_3_table[i]`.
    fn div3(&self, i: i32) -> i32;
    /// `Unit::add_patrol_order(x1, y1, x2, y2, id, form, a, b, queue, y1 >> 4)`
    /// on the member's unit.
    fn add_patrol_order(&mut self, m: i32, a: [i32; 10]);
    /// `Unit::update_patrol_order(id)` on the member's unit: the new order.
    fn update_patrol_order(&mut self, m: i32, id: i32);
    /// The new order's word at `off`.
    fn o_word(&self, off: i32) -> i32;
    /// The new order's short at `off`, sign-extended.
    fn o_short(&self, off: i32) -> i32;
    /// Writes the new order's word at `off`.
    fn o_set(&mut self, off: i32, v: i32);
    /// `SimpleArray<WCoord>::increase_size` on the array at the new order's
    /// `off`.
    fn grow(&mut self, off: i32, n: i32);
    /// The point at byte `off` copied from the saved order's two arrays into
    /// the new order's.
    fn copy_point(&mut self, off: i32);
}

/// One `SimpleArray`'s growth: `have` the stored size, `cap` the short beside
/// it: the argument `increase_size` is given, or none.
fn growth(have: i32, i: i32, cap: i32) -> Option<i32> {
    if have >= i {
        return None;
    }
    let d = i - have;
    let limit = if cap >= 0 { cap } else { have };
    Some(if d <= limit {
        if cap >= 0 { cap } else { have }
    } else {
        d
    })
}

/// `Group::redo_patrol_order(order, _)`: the steps.
pub fn redo_patrol_order_steps<V: RedoPatrolView>(v: &mut V) {
    v.normalize();
    let mut k = 0;
    while k < v.count() {
        let m = v.member(k);
        k += 1;
        if !(v.obj_alive(m) && v.obj_on_map(m) && !v.obj_plane(m)) || v.obj_masks(m) & MISSILE != 0
        {
            continue;
        }
        let y0 = v.p_point(1, 0) >> 4;
        let a = [
            v.div3(v.p_point(0, 0) >> 4),
            v.div3(y0),
            v.div3(v.p_point(0, 1) >> 4),
            v.div3(v.p_point(1, 1) >> 4),
            v.p_word(0x4c),
            v.p_word(0x50),
            v.p_word(0x44),
            v.p_word(0x48),
            QUEUE_LAST,
            y0,
        ];
        v.add_patrol_order(m, a);
        let id = v.p_word(0x4c);
        v.update_patrol_order(m, id);
        if v.p_word(8) > 2 {
            let mut at = 8;
            let mut i = 3;
            loop {
                if let Some(n) = growth(v.o_word(0xc), i, v.o_short(0x10)) {
                    v.grow(4, n);
                }
                let c = v.o_word(8).max(i);
                v.o_set(8, c);
                if let Some(n) = growth(v.o_word(0x28), i, v.o_short(0x2c)) {
                    v.grow(0x20, n);
                }
                let c = v.o_word(0x24).max(i);
                v.o_set(0x24, c);
                v.copy_point(at);
                at += 4;
                let more = i < v.p_word(8);
                i += 1;
                if !more {
                    break;
                }
            }
        }
        let w = v.p_word(0x3c);
        v.o_set(0x3c, w);
    }
}

/// The crate's view: the group's members, the saved [`PatrolOrder`] and the
/// member's order list.
///
/// SEAM: the saved order is two points ([`PatrolOrder`]), so `p_word(8)` is 2
/// and the third-point loop's leaves answer nothing.
pub struct SimRedoPatrol<'a> {
    pub sim: &'a mut Sim,
    pub g: &'a Group,
    pub p: PatrolOrder,
    /// The unit and order-list index `update_patrol_order` found.
    cur: Option<(usize, usize)>,
}

impl<'a> SimRedoPatrol<'a> {
    pub fn new(sim: &'a mut Sim, g: &'a Group, p: PatrolOrder) -> Self {
        SimRedoPatrol {
            sim,
            g,
            p,
            cur: None,
        }
    }
}

impl RedoPatrolView for SimRedoPatrol<'_> {
    fn normalize(&mut self) {
        // SEAM: `normalize`'s prune of dead members; `obj_alive` skips them.
        self.sim.group_set_speed(self.g);
    }
    fn count(&self) -> i32 {
        self.g.list.len() as i32
    }
    fn member(&self, i: i32) -> i32 {
        self.g.list[i as usize] as i32
    }
    fn obj_alive(&mut self, m: i32) -> bool {
        self.sim.units[m as usize].alive()
    }
    fn obj_on_map(&mut self, m: i32) -> bool {
        self.sim.units[m as usize].on_map
    }
    fn obj_plane(&mut self, m: i32) -> bool {
        self.sim.is_plane(m as usize)
    }
    fn obj_masks(&mut self, m: i32) -> u32 {
        self.sim.units[m as usize]
            .ty
            .map_or(0, |t| self.sim.unit_types[t].combat.obj_masks)
    }
    fn p_word(&self, off: i32) -> i32 {
        match off {
            8 => 2,
            0x3c => self.p.waypoint as i32,
            0x44 => self.p.leader as i32,
            0x4c => self.p.id as i32,
            0x50 => self.p.form_id as i32,
            _ => 0,
        }
    }
    fn p_point(&self, axis: i32, i: i32) -> i32 {
        let q = self.p.points[i as usize];
        if axis == 0 { q.x } else { q.y }
    }
    fn div3(&self, i: i32) -> i32 {
        i / 3
    }
    fn add_patrol_order(&mut self, m: i32, a: [i32; 10]) {
        // The cells `div3` answered, back to the world units `add_patrol_order`
        // snaps again (a cell is three 16-unit steps).
        let from = Pos::new(a[0] * 48, a[1] * 48);
        let to = Pos::new(a[2] * 48, a[3] * 48);
        self.sim.add_patrol_order(
            m as usize,
            (from, to),
            i64::from(a[4]),
            a[5] as usize,
            a[6] as usize,
            QueuePos::Last,
        );
    }
    fn update_patrol_order(&mut self, m: i32, id: i32) {
        self.cur = self
            .sim
            .update_patrol_order(m as usize, Some(i64::from(id)))
            .map(|i| (m as usize, i));
    }
    fn o_word(&self, _off: i32) -> i32 {
        0
    }
    fn o_short(&self, _off: i32) -> i32 {
        0
    }
    fn o_set(&mut self, off: i32, v: i32) {
        if off == 0x3c
            && let Some((u, i)) = self.cur
            && let Body::Patrol(x) = &mut self.sim.units[u].orders[i].body
        {
            x.waypoint = v as usize;
        }
    }
    fn grow(&mut self, _off: i32, _n: i32) {}
    fn copy_point(&mut self, _off: i32) {}
}
