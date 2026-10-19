//! `PathFinder::find_upath@00682f30` as steps over a view (item 3339,
//! `docs/PATHFINDER.md`): the 48-grid planner. [`find_upath_steps`] is the
//! original's control flow in its order; [`UpathView`] is the state it reads
//! and the callees it makes, which `Sim`'s `SimUpath` (in `path.rs`, beside
//! the pathfinder's private parts) answers from the crate's state and
//! `sweep/path_finder_find_upath_00682f30.rs` from the emulated original's
//! windows. Modelled on [`crate::tpath`], its tile-grid sibling.

use crate::movement;
use crate::orders::{PathData, path_flag};
use crate::world::Pos;

/// What `find_upath` reads of the unit, the world, its path stack and the
/// kept-waypoint stack, and the callees it makes.
pub trait UpathView {
    /// The prologue's writes, before the `saving` gate: the unit pointer
    /// (`pathfinder +0x54`), the start tile (`+0x58/+0x5c`, the tile table
    /// at the position `>> 6`), and the zeroed `+0x68/+0x6c/+0x94`.
    fn begin(&mut self, here: Pos);
    /// `pathfinder +0x84`, read at entry: set, the pre-search block is
    /// skipped (`find_upath_restore`).
    fn saving(&self) -> bool;
    /// `div_3_table[c >> 4]`: the 48-grid cell a coordinate lies in.
    fn cell48(&self, c: i32) -> i32;
    /// The world's `+0/+4`, in 16-tile blocks.
    fn world_dims(&self) -> (i32, i32);
    /// `Stack<PathData>` pop. An empty stack pops `list[0]` and stays empty
    /// in the original; the crate has no such entry, so its view answers
    /// `None` and the call returns 0.
    fn pop(&mut self) -> Option<PathData>;
    /// `Stack<PathData>::peek` (the top, `list[0]` when empty).
    fn peek(&mut self) -> PathData;
    /// `Stack<PathData>::push`.
    fn push(&mut self, e: PathData);
    /// The stack emptied (`length = 0`).
    fn clear(&mut self);
    /// The stack's length now.
    fn depth(&self) -> i32;
    /// The unit type's domain below 2 (`type +0x218 < 2`): not an aircraft.
    fn domain_below_2(&self) -> bool;
    /// `UnitData::can_transport`.
    fn can_transport(&mut self) -> bool;
    /// `WorldData::get_tregion` of a (cell `>> 2`) pair.
    fn tregion(&mut self, cx4: i32, cy4: i32) -> i32;
    /// `PathFinder::valid_ucoord` of a position and the cell metric.
    fn valid_ucoord(&mut self, p: Pos, metric: i32) -> bool;
    /// `PathFinder::astar_path` on the unit grid (`step 0x30`); the verdict.
    fn search(&mut self, anti: i32) -> i32;
    /// `pathfinder +0x84` read **after** the search — `astar_path` writes 1
    /// on its suspend and 0 on every failure tail — given the verdict.
    fn suspended(&self, r: i32) -> bool;
    /// The stack's top, dropped unless it carries `FINAL`.
    fn drop_unfinal_top(&mut self);
    /// The current order is a move (`update_order`, vslot `+0x14`).
    fn order_is_move(&mut self) -> bool;
    /// The current order's move data `retry` (`update_order`, vslot `+0x40`,
    /// field `+0x1c`).
    fn move_retry(&mut self) -> i32;
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// `PathFinder::kill_lists`.
    fn kill_lists(&mut self);
    /// The unit's position as the object stores it (`+0x10/+0x14`, xor
    /// `0x63637`).
    fn unit_pos(&self) -> Pos;
    /// The function-static `keep` stack emptied (and its guard passed).
    fn keep_begin(&mut self);
    /// `keep.push`.
    fn keep_push(&mut self, e: PathData);
    /// `keep` popped; `None` once it is empty.
    fn keep_pop(&mut self) -> Option<PathData>;
}

/// The original's control flow. `here` is the planner's from-position,
/// `anti` the sixth argument (`astar_path`'s fourth).
pub fn find_upath_steps<V: UpathView>(v: &mut V, here: Pos, anti: i32) -> i32 {
    v.begin(here);
    if !v.saving() {
        let hcx = v.cell48(here.x);
        let hcy = v.cell48(here.y);
        let Some(mut goal_e) = v.pop() else {
            return 0;
        };
        let mut gcx = v.cell48(goal_e.to.x);
        let mut gcy = v.cell48(goal_e.to.y);
        let (w, h) = v.world_dims();
        if gcx < 0 || gcy < 0 || w.wrapping_mul(16) <= gcx || h.wrapping_mul(16) <= gcy {
            v.clear();
            return -1;
        }
        // The same cell: the tolerance is zeroed and the goal pushed as is.
        if hcx == gcx && hcy == gcy {
            goal_e.tolerance = 0;
            v.push(goal_e);
            return v.depth();
        }
        let mut goal = goal_e.to;
        // **The pre-walk is gated**: the type's domain `+0x218 < 2` and
        // `UnitData::can_transport` — a conjunction with no vfunc `+0x8`
        // disjunct (item 301, `docs/PATHFINDER.md` §18.1).
        let pulls_back = v.domain_below_2() && !v.can_transport();
        if pulls_back {
            loop {
                // Both regions are asked, goal first, whatever the first says.
                let goal_region = v.tregion(gcx >> 2, gcy >> 2);
                let here_region = v.tregion(hcx >> 2, hcy >> 2);
                let metric = w.wrapping_mul(gcy).wrapping_mul(16).wrapping_add(gcx);
                if goal_region == here_region && v.valid_ucoord(goal, metric) {
                    break;
                }
                let (dx, dy) = (here.x.wrapping_sub(goal.x), here.y.wrapping_sub(goal.y));
                if dx.wrapping_abs() < 0x18 && dy.wrapping_abs() < 0x18 {
                    if goal_e.flags & path_flag::FINAL == 0 {
                        return 0;
                    }
                    // A final goal given up on is pushed as it stands, its
                    // tolerance kept (`testb $1, [ebp-0x34]; jne 683082`,
                    // `docs/AI.md` §231).
                    v.push(goal_e);
                    return v.depth();
                }
                let ang = movement::find_angle(dx, dy);
                // `0x18` — `mov edi, 0x18` at `0x68318b`.
                let s = 0x18;
                let sx = movement::sin_component(ang, s);
                let cy = movement::cos_component(ang, s);
                goal = Pos::new(goal.x.wrapping_add(sx), goal.y.wrapping_sub(cy));
                goal_e.to = goal;
                gcx = v.cell48(goal.x);
                gcy = v.cell48(goal.y);
                if gcx == hcx && gcy == hcy {
                    v.push(goal_e);
                    return v.depth();
                }
                // The crate's guard against a step of nothing; the original
                // would ask the same two callees for ever.
                if sx == 0 && cy == 0 {
                    break;
                }
            }
        }
        // **The near test zeroes the tolerance**, as the same-cell test does
        // (`cmp ecx, 1; jg` at `683273`, `mov [ebp-0x3c], 0`).
        let md = hcx
            .wrapping_sub(gcx)
            .wrapping_abs()
            .wrapping_add(hcy.wrapping_sub(gcy).wrapping_abs());
        if md <= 1 {
            goal_e.tolerance = 0;
            v.push(goal_e);
            return v.depth();
        }
        let centre = |c: i32| c.wrapping_mul(0x30).wrapping_add(0x18);
        let kept = goal_e.tolerance;
        v.push(goal_e);
        // The goal's cell: `[ebp-0x38]` (flags) zeroed and the position
        // rewritten, **the tolerance left as the goal had it** — the
        // listing at `6832aa`–`6832cf` writes no `[ebp-0x3c]`.
        v.push(PathData {
            to: Pos::new(centre(gcx), centre(gcy)),
            tolerance: kept,
            flags: 0,
        });
        v.push(PathData {
            to: Pos::new(centre(hcx), centre(hcy)),
            tolerance: 0,
            flags: 0,
        });
    }
    let r = v.search(anti);
    if r < 1 {
        if v.suspended(r) {
            v.kill_lists();
            return r;
        }
        v.drop_unfinal_top();
        // **The kill, and what spares it** (`find_upath@00682f30:187`–
        // `195`): a current order that is a move whose move data carries a
        // non-zero `retry`.
        let spared = v.order_is_move() && v.move_retry() != 0;
        if !spared {
            v.kill_current_order();
        }
        v.kill_lists();
        return 0;
    }
    // Success: drop a top equal to the unit's position, then compact
    // collinear side-steps.
    v.keep_begin();
    if v.depth() > 3 {
        let Some(mut first) = v.pop() else {
            return 0;
        };
        if first.to == v.unit_pos()
            && first.flags & path_flag::FINAL == 0
            && let Some(second) = v.pop()
        {
            first = second;
        }
        v.keep_push(first);
        if first.flags & path_flag::SIDESTEP != 0 && v.depth() > 2 {
            // The reference point advances to the examined waypoint on a
            // keep and on a collinear drop, but not on a one-step-jump
            // drop — the decompile's comma-expression side effects,
            // followed exactly.
            let (mut ax, mut ay) = (first.to.x, first.to.y);
            let mut mid = v.pop().expect("a third entry");
            let mut next = v.peek();
            while mid.flags & path_flag::SIDESTEP != 0
                && next.flags & path_flag::SIDESTEP != 0
                && v.depth() > 1
            {
                let collinear = ax.wrapping_sub(mid.to.x) == mid.to.x.wrapping_sub(next.to.x)
                    && ay.wrapping_sub(mid.to.y) == mid.to.y.wrapping_sub(next.to.y);
                let one_step = ax.wrapping_sub(next.to.x).wrapping_abs() == 0x30
                    && ay.wrapping_sub(next.to.y).wrapping_abs() == 0x30;
                if mid.flags & path_flag::TRANSPORT != 0 || (!collinear && !one_step) {
                    v.keep_push(mid);
                    ax = mid.to.x;
                    ay = mid.to.y;
                } else if collinear {
                    ax = mid.to.x;
                    ay = mid.to.y;
                }
                mid = v.pop().expect("a chain entry");
                next = v.peek();
            }
            v.push(mid);
        }
        while let Some(k) = v.keep_pop() {
            v.push(k);
        }
    }
    let n = v.depth();
    v.kill_lists();
    n
}
