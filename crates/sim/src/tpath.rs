//! `PathFinder::find_tpath@006897d0` as steps over a view (item 3337,
//! `docs/PATHFINDER.md`): the tile-grid planner. [`find_tpath_steps`] is the
//! original's control flow in its order; [`TpathView`] is the state it reads
//! and the callees it makes, which [`SimTpath`] answers from the crate's
//! [`Sim`] and `sweep/path_finder_find_tpath_006897d0.rs` from the emulated
//! original's windows. The method's account is on [`Sim::find_tpath`].

use crate::Sim;
use crate::movement;
use crate::orders::{PathData, path_flag};
use crate::world::Pos;

/// What `find_tpath` reads of the unit, the world and its path stack, and the
/// callees it makes.
pub trait TpathView {
    /// The pathfinder's start tile (`pathfinder +0x58/+0x5c`), written before
    /// anything else.
    fn begin(&mut self, start: Pos);
    /// `Stack<PathData>` pop of the goal; `None` on an empty stack.
    fn pop(&mut self) -> Option<PathData>;
    /// `Stack<PathData>::push`.
    fn push(&mut self, e: PathData);
    /// The stack emptied (`length = 0`).
    fn clear(&mut self);
    /// The stack's length now.
    fn depth(&self) -> i32;
    /// Whether the tile lies inside the world (`world +0x18/+0x1c`).
    fn tile_in_bounds(&self, t: Pos) -> bool;
    /// The unit type's helicopter bit (`type +0x2b4 & 0x20`).
    fn is_helicopter(&self) -> bool;
    /// The unit type's domain below 2 (`type +0x218 < 2`): not an aircraft.
    fn domain_below_2(&self) -> bool;
    /// `UnitData::can_transport`.
    fn can_transport(&mut self) -> bool;
    /// `WorldData::get_tregion` of a tile (`-1` for none: the crate's
    /// unregioned cell).
    fn tregion(&mut self, t: Pos) -> i32;
    /// `PathFinder::valid_tcoord` of a position.
    fn valid_tcoord(&mut self, p: Pos) -> bool;
    /// The current order's move destination offset (`+0x4c/+0x4e` shorts),
    /// asked through the unit's `update_order` chain; `None` when the unit
    /// has no such order.
    fn order_offset(&mut self) -> Option<Pos>;
    /// The pathfinder's tile offset (`pathfinder +0x68/+0x6c`) the search
    /// reads, written before it.
    fn set_offset(&mut self, off: Pos);
    /// `PathFinder::astar_path` on the tile grid, then `kill_lists`; the
    /// search's verdict.
    fn search(&mut self) -> i32;
    /// The stack's top, dropped unless it carries `FINAL`.
    fn drop_unfinal_top(&mut self);
}

/// The original's control flow. `here` is the planner's from-position.
pub fn find_tpath_steps<V: TpathView>(v: &mut V, here: Pos) -> i32 {
    let ht = here.tile();
    v.begin(ht);
    let Some(mut goal_e) = v.pop() else {
        return 0;
    };
    let gt = goal_e.to.tile();
    if !v.tile_in_bounds(gt) {
        v.clear();
        return -1;
    }
    // The same tile, or a helicopter (`68990d`, item 1048).
    if ht == gt || v.is_helicopter() {
        goal_e.tolerance = 0;
        v.push(goal_e);
        return v.depth();
    }
    // The pull-back walk on tiles — gated as `find_upath`'s is
    // (`006897d0`, listing `68994c`–`689960`): `cmp [type + 0x218], 2; jge`
    // and `UnitData::can_transport@0046f960`, `jne`, both to the near test
    // at `689aa6`. So only a unit that is not an aircraft and cannot
    // transport walks its goal back toward its own tile region; a transport
    // keeps the goal it was given. run710's Freighter `1/52`
    // (`unit_masks & 0x800000`) stands on 1685 on a tile whose region is
    // not its goal's, and without the gate its walk came all the way home
    // and the plan was refused (`docs/AI.md` §163).
    let mut goal = goal_e.to;
    let pulls_back = v.domain_below_2() && !v.can_transport();
    if pulls_back {
        loop {
            let goal_region = v.tregion(goal.tile());
            let here_region = v.tregion(ht);
            if goal_region == here_region && v.valid_tcoord(goal) {
                break;
            }
            let (dx, dy) = (here.x.wrapping_sub(goal.x), here.y.wrapping_sub(goal.y));
            if dx.wrapping_abs() < 0x60 && dy.wrapping_abs() < 0x60 {
                if goal_e.flags & path_flag::FINAL == 0 {
                    return 0;
                }
                // A final goal given up on is pushed as it stands —
                // `test [ebp-0x28], 1; jne 689a75`, the plain push — past
                // the near test and the search, its tolerance kept
                // (`docs/AI.md` §231, parked 2739).
                v.push(goal_e);
                return v.depth();
            }
            let ang = movement::find_angle(dx, dy);
            // `0x60`, the same constant as the give-up test above, and no
            // caller-level fold — `mov ebx, 0x60` at `0x6899da`, negated to
            // `0xffffffa0` only inside the sine's own fold.
            let s = 0x60;
            let sx = movement::sin_component(ang, s);
            let cy = movement::cos_component(ang, s);
            goal = Pos::new(goal.x.wrapping_add(sx), goal.y.wrapping_sub(cy));
            goal_e.to = goal;
            if goal.tile() == ht {
                v.push(goal_e);
                return v.depth();
            }
            if sx == 0 && cy == 0 {
                break;
            }
        }
    }
    let gt = goal.tile();
    let td = (ht.x - gt.x).abs() + (ht.y - gt.y).abs();
    if td < 2 {
        goal_e.tolerance = 0;
        v.push(goal_e);
        return v.depth();
    }
    let kept = goal_e.tolerance;
    v.push(goal_e);
    let centre_t = |t: Pos| Pos::new(t.x * 0xc0 + 0x60, t.y * 0xc0 + 0x60);
    v.push(PathData {
        to: centre_t(gt),
        tolerance: if kept == 0x180 { 0x180 } else { 0x60 },
        flags: 0,
    });
    v.push(PathData {
        to: centre_t(ht),
        tolerance: 0,
        flags: 0,
    });
    let off = v.order_offset().map_or(Pos::new(0, 0), |o| {
        Pos::new(o.x % 0xc0 - 0x60, o.y % 0xc0 - 0x60)
    });
    v.set_offset(off);
    if v.search() < 1 {
        v.drop_unfinal_top();
        return 0;
    }
    v.depth()
}

/// The crate's [`Sim`] as the view: unit `u`.
pub(crate) struct SimTpath<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) u: usize,
}

impl TpathView for SimTpath<'_> {
    fn begin(&mut self, start: Pos) {
        self.sim.pf_start = start;
    }
    fn pop(&mut self) -> Option<PathData> {
        self.sim.units[self.u].path.pop()
    }
    fn push(&mut self, e: PathData) {
        self.sim.units[self.u].path.push(e);
    }
    fn clear(&mut self) {
        self.sim.units[self.u].path.clear();
    }
    fn depth(&self) -> i32 {
        self.sim.units[self.u].path.len() as i32
    }
    fn tile_in_bounds(&self, t: Pos) -> bool {
        self.sim.world.tile_in_bounds(t)
    }
    fn is_helicopter(&self) -> bool {
        self.sim.is_helicopter(self.u)
    }
    fn domain_below_2(&self) -> bool {
        self.sim.unit_domain_of(self.u) != crate::attrition::Domain::Air
    }
    fn can_transport(&mut self) -> bool {
        self.sim.unit_can_transport(self.u)
    }
    fn tregion(&mut self, t: Pos) -> i32 {
        self.sim.world.tregion_alt(t).map_or(-1, i32::from)
    }
    fn valid_tcoord(&mut self, p: Pos) -> bool {
        self.sim.valid_tcoord(self.u, p)
    }
    fn order_offset(&mut self) -> Option<Pos> {
        self.sim.toff(self.u).map(|(x, y)| Pos::new(x, y))
    }
    fn set_offset(&mut self, _off: Pos) {
        // `astar_path` re-reads the order's offset itself (`toff`), so the
        // pathfinder's `+0x68/+0x6c` has no field of its own here.
    }
    fn search(&mut self) -> i32 {
        self.sim.tpath_search(self.u)
    }
    fn drop_unfinal_top(&mut self) {
        if self.sim.units[self.u]
            .path
            .last()
            .is_some_and(|p| p.flags & path_flag::FINAL == 0)
        {
            self.sim.units[self.u].path.pop();
        }
    }
}
