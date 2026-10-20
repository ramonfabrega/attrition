//! `PathFinder::astar_path@00683770` — its **entry**, its **fresh start** and
//! its **suspend**, as steps over an [`AstarView`] (item 3381, parked 3362a).
//!
//! The function is too large to sweep whole: it keeps its open and closed
//! sets in five pointer-linked trees swapped with the unit's, and a stubbed
//! callee cannot stand in for a search. What the sweep can hold is the part
//! that reads the unit, the order list, the world and the path stack and
//! writes the pathfinder's own words: the work cap, the stride, the
//! direction increment and `toff` (the entry, run for a resume too), the
//! two pops and the root node (the fresh arm), and the block that hands the
//! search to the unit when a unit-grid search runs over its limit (the
//! suspend). [`entry_steps`], [`fresh_steps`] and [`suspend_steps`] are that
//! control flow in the listing's order; [`AstarView`] and [`SuspendView`]
//! are the state they read and the callees they make, which
//! [`crate::path`]'s `SimAstar` answers from the crate's [`crate::Sim`] and
//! `sweep/path_finder_astar_suspend.rs` from the emulated original's windows.
//!
//! The search loop itself (the wheel, `calc_cost`, the closed set, the
//! reconstruction) is **not** here: it stays in `Sim::astar_path` over the
//! crate's own containers, and what it shares with the sweep is
//! [`arrive`] and [`top_of_loop`], the two pure tests the loop's head makes.

use crate::orders::PathData;
use crate::world::{Pos, vector_dist};

/// The three grids' steps, as `param_2`.
pub use crate::path::{STEP_TILE, STEP_UNIT, STEP_WORLD};

/// The `UnitOrder::get_type` of an attack (`ATTACK`), which `get_action`
/// answers for an attacking unit.
pub const ACTION_ATTACK: i32 = 10;
/// `OrderIndex::ATTACK_TO` and `GROUP_ATTACK_TO` (`UnitData::order_type`).
pub const ORDER_ATTACK_TO: i32 = 2;
pub const ORDER_GROUP_ATTACK_TO: i32 = 0x15;

/// What `no_danger` (`PathFinder +0x7c`) reads: the unit's action, its
/// owner and its current order's index.
pub trait NoDangerView {
    /// `UnitData::get_action@00608450`, then its vslot `+0x10` (`get_type`):
    /// `None` for no action.
    fn action_type(&mut self) -> Option<i32>;
    /// The unit's owner byte (`+9`).
    fn owner(&mut self) -> i32;
    /// `UnitData::order_type@00616e80`.
    fn order_type(&mut self) -> i32;
}

/// `astar_path@00683770:118-128`: the danger map is ignored under an attack
/// action, for gaia (`who >= 8`), and under `ATTACK_TO`/`GROUP_ATTACK_TO`.
/// Short-circuits as the listing does: `get_action`, its type, the owner,
/// then `order_type` once or twice.
pub fn no_danger_steps<V: NoDangerView + ?Sized>(v: &mut V) -> bool {
    let attacking = v.action_type() == Some(ACTION_ATTACK);
    !(!attacking
        && v.owner() < 8
        && v.order_type() != ORDER_ATTACK_TO
        && v.order_type() != ORDER_GROUP_ATTACK_TO)
}

/// What `astar_path` reads of the unit, its orders, the world and its path
/// stack, and the pathfinder words it writes, in the listing's order.
pub trait AstarView: NoDangerView {
    // ---- the unit and its type --------------------------------------
    /// The unit's `unit_masks2` (`+0x6c`), read through `units`.
    fn masks2(&mut self) -> i32;
    /// The unit type's `+0x248`, the collision size.
    fn collision(&mut self) -> i32;
    /// The unit type's `+0x2b4 & 0x10`: the transport bit.
    fn transport_type(&mut self) -> bool;
    /// The unit's `unit_masks & 0x40000`: computer-controlled.
    fn ai_bit(&mut self) -> bool;
    // ---- the order list ---------------------------------------------
    /// `+0xdc != 0`: the order list has its end node.
    fn list_present(&mut self) -> bool;
    /// The cursor onto the front of the list (`+0xd4 = end->next`,
    /// `+0xcc = its order`, `+0xd0 = its flag`).
    fn cursor_to_front(&mut self);
    /// `+0xcc != 0`: the cursor holds an order.
    fn has_order(&mut self) -> bool;
    /// The cursor order's vslot `+0x14`, `is_move`.
    fn order_is_move(&mut self) -> bool;
    /// `Unit::update_order`, its vslot `+0x40`, the shorts at `+0x4c` and
    /// `+0x4e`: the move order's own offset.
    fn move_offset(&mut self) -> (i32, i32);
    /// `UnitData::get_order@0060b030` is non-null.
    fn order_present(&mut self) -> bool;
    /// `get_order` again, its flag byte (`+4`); 0 for none.
    fn order_flags(&mut self) -> i32;
    // ---- the world ---------------------------------------------------
    /// `world +0`: the width in cells.
    fn world_width(&mut self) -> i32;
    /// `world +0x18`: the width in tiles.
    fn world_width_tiles(&mut self) -> i32;
    /// `div_3_table[i]`.
    fn div3(&mut self, i: i32) -> i32;
    /// The region short (`+4`) of the world cell `(cx, cy)`.
    fn region_of_cell(&mut self, cx: i32, cy: i32) -> i32;
    /// `WorldData::get_tregion` of the tile `(tx, ty)`.
    fn tregion(&mut self, tx: i32, ty: i32) -> i32;
    /// `WorldData::is_ocean` of the cell `(cx, cy)`.
    fn is_ocean_cell(&mut self, cx: i32, cy: i32) -> bool;
    /// The tile mask word of the tile `(tx, ty)`.
    fn tile_mask(&mut self, tx: i32, ty: i32) -> i32;
    // ---- the path stack ---------------------------------------------
    /// `Stack<PathData>` pop (an empty stack answers its first entry, as
    /// the listing clamps `length` to 1 first).
    fn pop_path(&mut self) -> PathData;
    /// The tolerance of the entry now on top of the stack; 0 for none.
    fn top_tolerance(&mut self) -> i32;
    // ---- the pathfinder's own words ---------------------------------
    /// `+0x74 = unit_masks2 & 0x4000`.
    fn set_iroquois(&mut self, word: i32);
    /// `+0x7c`.
    fn set_no_danger(&mut self, on: bool);
    /// `+0x90 = 0` (`valid_hit`), the one word a fresh search resets.
    fn reset_valid_hit(&mut self);
    /// `Recycler<PathNode>::pop`, the root's fields, and the two inserts
    /// that put it on the open list (`Tree::ordered_insert`, then
    /// `BRTree::ordered_insert` of the tree's current node by `metric`).
    fn seed_open(&mut self, start: Pos, metric: i32, estimate: i32);
    /// `+0x88` and `+0x8c`.
    fn set_avoid(&mut self, land: i32, sea: i32);
}

/// What the entry derives from the arguments and the unit, for a fresh
/// search and a resume alike (`00683770` runs it before it looks at
/// `saving`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// `+0x74 != 0`: the unit walks forests.
    pub iroquois: bool,
    /// `+0x7c`.
    pub no_danger: bool,
    /// `local_68`: the unit grid's `max(1, (collision + 1) / 2)`; 1 elsewhere.
    pub su: i32,
    /// `local_64`: the row width of the grid's `metric`.
    pub row: i32,
    /// `local_80`: the wheel's increment (2 on a unit-grid `anti` search).
    pub dinc: i32,
    /// `local_84 * 0x40`: the work cap.
    pub work_cap: i32,
    /// `local_7c`: `su * step`.
    pub stride: i32,
    /// `toff`, `(0, 0)` unless the current order is a move.
    pub toff: (i32, i32),
}

/// `astar_path@00683770:84-161`.
pub fn entry_steps<V: AstarView + ?Sized>(v: &mut V, step: i32, anti: i32) -> Entry {
    let mut su: i32 = 1;
    let mut cap: i32 = 50;
    let mut dinc: i32 = 1;
    let iroq = v.masks2() & 0x4000;
    v.set_iroquois(iroq);
    let row = if step == STEP_UNIT {
        cap = 500;
        // `(x + 1) / 2`, truncating; the listing clamps at 1.
        su = (v.collision().wrapping_add(1) / 2).max(1);
        dinc = i32::from(anti != 0) + 1;
        v.world_width().wrapping_shl(4)
    } else if step == STEP_TILE {
        v.world_width_tiles()
    } else {
        v.world_width()
    };
    let stride = su.wrapping_mul(step);
    let no_danger = no_danger_steps(v);
    v.set_no_danger(no_danger);
    let mut toff = (0, 0);
    if v.list_present() {
        v.cursor_to_front();
        if v.has_order() {
            // The listing re-reads the list and takes the cursor again
            // before it calls the order.
            v.cursor_to_front();
            if v.order_is_move() {
                toff = v.move_offset();
            }
        }
    }
    Entry {
        iroquois: iroq != 0,
        no_danger,
        su,
        row,
        dinc,
        work_cap: cap * 0x40,
        stride,
        toff,
    }
}

/// `PathFinderData::get_estimate`: the heuristic, 10 a unit on the unit
/// grid, 60 over the step elsewhere.
pub fn estimate(dx: i32, dy: i32, step: i32) -> i32 {
    let d = vector_dist(dx, dy);
    if step == STEP_UNIT {
        d.wrapping_mul(10)
    } else {
        d.wrapping_mul(60) / step
    }
}

/// The grid-cell index of a position on the search's grid — the `metric`,
/// `div_3_table` read as the listing reads it.
pub fn metric_of<V: AstarView + ?Sized>(v: &mut V, step: i32, p: Pos) -> i32 {
    match step {
        STEP_WORLD => {
            let (a, b) = (v.div3(p.x >> 8), v.div3(p.y >> 8));
            let w = v.world_width();
            a.wrapping_add(b.wrapping_mul(w))
        }
        STEP_TILE => {
            let (a, b) = (v.div3(p.x >> 6), v.div3(p.y >> 6));
            let w = v.world_width_tiles();
            a.wrapping_add(b.wrapping_mul(w))
        }
        _ => {
            let (a, b) = (v.div3(p.x >> 4), v.div3(p.y >> 4));
            let w = v.world_width();
            a.wrapping_add(b.wrapping_mul(w).wrapping_mul(0x10))
        }
    }
}

/// What the fresh arm hands the loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fresh {
    pub start: Pos,
    pub goal: Pos,
    /// The arrival tolerance: the *final* goal's, the entry now on top.
    pub tol: i32,
    /// `(avoid_land, avoid_sea)`, `+0x88`/`+0x8c`.
    pub avoid: (i32, i32),
    pub root_metric: i32,
    pub root_estimate: i32,
    /// The wheel's start: cardinal facing the goal.
    pub pref: i32,
    /// The start-to-goal Manhattan.
    pub start_dist: i32,
}

/// `astar_path@00683770:303-461`: pop the start and the goal, seed the open
/// list, ask the two regions, and set the search's avoid flags.
pub fn fresh_steps<V: AstarView + ?Sized>(v: &mut V, step: i32) -> Fresh {
    v.reset_valid_hit();
    let start = v.pop_path().to;
    let goal = v.pop_path().to;
    let tol = v.top_tolerance();
    let root_metric = metric_of(v, step, start);
    let root_estimate = estimate(
        start.x.wrapping_sub(goal.x),
        start.y.wrapping_sub(goal.y),
        step,
    );
    v.seed_open(start, root_metric, root_estimate);
    let avoid = avoid_steps(v, step, start, goal);
    v.set_avoid(avoid.0, avoid.1);
    let (dx, dy) = (start.x.wrapping_sub(goal.x), start.y.wrapping_sub(goal.y));
    let start_dist = dx.wrapping_abs().wrapping_add(dy.wrapping_abs());
    let pref = if dy.wrapping_abs() < dx.wrapping_abs() {
        i32::from(goal.x < start.x) * 4 + 3
    } else {
        i32::from(start.y <= goal.y) * 4 + 1
    };
    Fresh {
        start,
        goal,
        tol,
        avoid,
        root_metric,
        root_estimate,
        pref,
        start_dist,
    }
}

/// `astar_path@00683770:371-450`: `avoid_land`/`avoid_sea`, off the start's
/// terrain. The world grid reads the two cells' region shorts straight out
/// of the array and compares them; the tile and unit grids ask
/// `get_tregion` — the goal's first, then the start's.
pub fn avoid_steps<V: AstarView + ?Sized>(
    v: &mut V,
    step: i32,
    start: Pos,
    goal: Pos,
) -> (i32, i32) {
    let same_region = if step == STEP_WORLD {
        let (sx, sy) = (v.div3(start.x >> 8), v.div3(start.y >> 8));
        let (gx, gy) = (v.div3(goal.x >> 8), v.div3(goal.y >> 8));
        v.region_of_cell(sx, sy) == v.region_of_cell(gx, gy)
    } else {
        let (sy, sx) = (v.div3(start.y >> 6), v.div3(start.x >> 6));
        let (gy, gx) = (v.div3(goal.y >> 6), v.div3(goal.x >> 6));
        let goal_region = v.tregion(gx, gy);
        v.tregion(sx, sy) == goal_region
    };
    if !same_region {
        // The pair is cleared; a current order that says so (flag `0x20`)
        // sets the land word.
        if v.order_present() && v.order_flags() & 0x20 != 0 {
            return (1, 0);
        }
        return (0, 0);
    }
    let on_water = if step == STEP_WORLD {
        let (cx, cy) = (v.div3(start.x >> 8), v.div3(start.y >> 8));
        v.is_ocean_cell(cx, cy)
    } else {
        let (ty, tx) = (v.div3(start.y >> 6), v.div3(start.x >> 6));
        v.tile_mask(tx, ty) & 0x30 == 0x20
    };
    // A computer-controlled transport crosses either terrain without the
    // same-region preference (`docs/PATHFINDER.md` §31).
    if v.transport_type() && v.ai_bit() {
        return (0, 0);
    }
    if on_water {
        return (1, 0);
    }
    (
        0,
        if v.action_type() == Some(ACTION_ATTACK) {
            2
        } else {
            1
        },
    )
}

/// `local_60 / 2 + local_7c`: how near the goal ends the search.
pub fn arrive(tol: i32, stride: i32) -> i32 {
    (tol / 2).wrapping_add(stride)
}

/// What the loop's head decides about the node it has just taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Top {
    /// Expand the node.
    Expand,
    /// A unit-grid search over its node limit, short of the goal, not
    /// `anti`: hand the search to the unit (`-1`).
    Suspend,
    /// Arrived, or out of work: end the search on this node.
    Finish,
}

/// `astar_path@00683770:469-479` — the head of the loop. `probes` is
/// `local_58`, `traversed` `local_70`; `anti_unit` and `limit` the
/// pathfinder's `+0x64` and `+0x80`.
#[allow(clippy::too_many_arguments)]
pub fn top_of_loop(
    manh: i32,
    arrive: i32,
    traversed: i32,
    probes: i32,
    work_cap: i32,
    anti_unit: bool,
    limit: i32,
    anti: i32,
) -> Top {
    let over_limit = anti_unit && limit < probes;
    if manh <= arrive || traversed.wrapping_add(probes) >= work_cap || over_limit {
        if over_limit && manh > arrive && anti == 0 {
            Top::Suspend
        } else {
            Top::Finish
        }
    } else {
        Top::Expand
    }
}

/// The scalars a suspend hands the unit (`+0x128..+0x148`, `+0x138`/`+0x13c`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stash {
    pub tol: i32,
    pub pref: i32,
    pub goal: Pos,
    pub start_dist: i32,
    /// `local_70 + local_58`.
    pub traversed: i32,
    pub avoid: (i32, i32),
}

/// What the suspend does to the search's containers and the unit.
pub trait SuspendView {
    /// The node the loop stopped on goes back on the open list
    /// (`Tree::ordered_insert` by its `value`, `BRTree::ordered_insert` of
    /// the tree's current node by its `metric`).
    fn reinsert_current(&mut self);
    /// `+0x84 = 1`, the five containers handed to the unit and fresh ones
    /// taken from the recyclers, and the scalars stored on the unit.
    fn hand_over(&mut self, st: &Stash);
}

/// `astar_path@00683770:480-526`.
pub fn suspend_steps<V: SuspendView + ?Sized>(v: &mut V, st: &Stash) {
    v.reinsert_current();
    v.hand_over(st);
}
