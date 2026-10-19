//! `PathFinder::find_wpath@00688fc0` as steps over a view (item 3339,
//! `docs/PATHFINDER.md` §3): the world-cell planner. [`find_wpath_steps`] is
//! the original's control flow in its order; [`WpathView`] is the state it
//! reads and the callees it makes, which [`SimWpath`] answers from the
//! crate's [`Sim`] and `sweep/path_finder_find_wpath_00688fc0.rs` from the
//! emulated original's windows. The method's account is on
//! [`Sim::find_wpath_from`]. The sibling is [`crate::tpath`].

use crate::Sim;
use crate::movement;
use crate::orders::{Body, MoveKind, PathData, path_flag};
use crate::world::{Cell, Pos};

/// Which of `find_wpath`'s two `get_tregion` pairs a call is, by the locals
/// the original hands it (the human variant's pop loop asks the goal cell's
/// centre tile and the start cell's; the walk asks the goal's tile and the
/// start's). A view that answers from the crate ignores it; the emulated
/// original's records name the locals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TregionSite {
    LoopGoal,
    LoopStart,
    WalkGoal,
    WalkStart,
}

/// Which of the two `WorldData::was_seen` call sites a call is: inside the
/// human pop loop, or the single test after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeenSite {
    Loop,
    After,
}

/// What `find_wpath` reads of the unit, the world and its path stack, and the
/// callees it makes.
pub trait WpathView {
    /// The pathfinder's start tile (`pathfinder +0x58/+0x5c`), written with
    /// the unit pointer, the `scouting` flag (`+0x94`) cleared and the tile
    /// offset (`+0x68/+0x6c`) zeroed, before anything else.
    fn begin(&mut self, start: Pos);
    /// `Stack<PathData>` inline pop of the goal; `None` on an empty stack.
    fn pop(&mut self) -> Option<PathData>;
    /// `Stack<PathData>::push`, the call.
    fn push(&mut self, e: PathData);
    /// The same-cell push, written inline (after `increase_size` when the
    /// stack is full).
    fn push_inline(&mut self, e: PathData);
    /// The stack emptied (`length = 0`).
    fn clear(&mut self);
    /// The stack's length now.
    fn depth(&self) -> i32;
    /// Whether the cell lies inside the world (`world +0/+4`, in cells).
    fn cell_in_bounds(&self, c: Cell) -> bool;
    /// The unit type's helicopter bit (`type +0x2b4 & 0x20`).
    fn is_helicopter(&self) -> bool;
    /// `leaders.list[class].flags & 4`: the owner is a human.
    fn human(&self) -> bool;
    /// The object's slot `+8`: the unit stands on the map.
    fn is_on_map(&mut self) -> bool;
    /// `UnitData::can_transport`.
    fn can_transport(&mut self) -> bool;
    /// The object type's domain (`type +0x218`): 0 land, 1 sea, 2 and up air.
    fn domain(&self) -> i32;
    /// `WorldData::get_tregion` of a tile (`-1` for none: the crate's
    /// unregioned cell).
    fn tregion(&mut self, t: Pos, site: TregionSite) -> i32;
    /// `WorldData::was_seen` of a cell's centre half-cell (`2c + 1`) for the
    /// unit's own leader.
    fn was_seen(&mut self, c: Cell, site: SeenSite) -> bool;
    /// `UnitData::invalid_loc` of a tile for the sea walk (flags `0,1,1,1,0`).
    fn invalid_loc(&mut self, t: Pos) -> bool;
    /// The unit type's scout role (`type +0x2c8 & 0x10`).
    fn is_scout_role(&self) -> bool;
    /// `UnitData::order_type == EXPLORE_TO`.
    fn order_is_explore_to(&mut self) -> bool;
    /// `Unit::update_order`, then the clause `order.flags & 4 == 0 ||
    /// unit +0x68 & 0x40100`.
    fn explore_gate(&mut self) -> bool;
    /// The order's slot `+0x10` answering 3 (an explore).
    fn explore_kind_is_explore(&mut self) -> bool;
    /// `pathfinder +0x94 = 1`.
    fn set_scouting(&mut self);
    /// The unit type's attack word is non-zero (`type +0x1e8`).
    fn has_attack(&self) -> bool;
    /// `UnitData::is_supply` (the unit data's slot `+0xcc`; or its inline
    /// body, `type +0x2b8 & 0x40`).
    fn is_supply(&mut self) -> bool;
    /// `ObjectData::is_worker`.
    fn is_worker(&mut self) -> bool;
    /// `UnitData::is_attacking`.
    fn is_attacking(&mut self) -> bool;
    /// The world cell under the unit carries the river flag (`& 0x100`).
    fn on_river(&self) -> bool;
    /// `pathfinder +0x70 = 1` (army mode).
    fn set_army(&mut self);
    /// `pathfinder +0x78 = 1` (worker mode).
    fn set_worker(&mut self);
    /// `PathFinder::astar_path` on the world grid; the search's verdict.
    fn search(&mut self) -> i32;
    /// `Stack<PathData>::pop` after a failed search: whether the popped
    /// entry carries `FINAL`.
    fn pop_top_is_final(&mut self) -> bool;
    /// The three mode words (`+0x70/+0x74/+0x78`) cleared, then `kill_lists`.
    fn kill_lists(&mut self);
}

/// The original's control flow. `here` is the planner's from-position.
pub fn find_wpath_steps<V: WpathView>(v: &mut V, here: Pos) -> i32 {
    let ht = here.tile();
    let hc = here.cell();
    v.begin(ht);
    let Some(mut e) = v.pop() else {
        return 0;
    };
    let mut gc = e.to.cell();
    if !v.cell_in_bounds(gc) {
        v.clear();
        return -1;
    }
    // The same cell, or a helicopter (`689110`, item 1048): pushed back as
    // it stands, tolerance kept.
    if hc == gc || v.is_helicopter() {
        v.push_inline(e);
        return v.depth();
    }
    // **A human's leader takes the other variant first** (`leaders & 4`,
    // `00689109`-`006892e4`; item 676, `docs/GOLDEN.md` §17): it does not
    // drag the goal back toward the start, it **pops**. While the goal
    // cell's centre tile is in another region from the start cell's, or its
    // centre half-cell has never been seen, the goal is dropped for the
    // entry under it — stopping on a final entry or an empty stack. Only a
    // goal it **has** seen — or a sea unit — then goes on to the AI's walk.
    let mut walk = true;
    if v.human() {
        if !v.is_on_map() || !v.can_transport() {
            loop {
                let goal_region = v.tregion(gc.centre_tile(), TregionSite::LoopGoal);
                let here_region = v.tregion(hc.centre_tile(), TregionSite::LoopStart);
                if goal_region == here_region && v.was_seen(gc, SeenSite::Loop) {
                    break;
                }
                if e.flags & path_flag::FINAL != 0 || v.depth() == 0 {
                    break;
                }
                let Some(p) = v.pop() else {
                    break;
                };
                e = p;
                gc = e.to.cell();
                if !v.cell_in_bounds(gc) {
                    v.clear();
                    return -1;
                }
            }
        }
        walk = v.was_seen(gc, SeenSite::After) || v.domain() == 1;
    }
    // The pull-back walk: step the goal toward the start until its tile
    // region matches the start's. **A unit that can board skips it whole**
    // (`00689375`: `domain < 2 && (!is_on_map() || !can_transport())`), and
    // that gate is the whole of transport pathing (`docs/TRANSPORT.md` §7).
    // **A sea unit's break test has a second clause** (item 1868): the
    // matched region must also pass `invalid_loc(t, 0, 1, 1, 1, 0)`, so a
    // goal on the shore is stepped back over water even when its region
    // reads as the boat's.
    if walk && v.domain() < 2 && (!v.is_on_map() || !v.can_transport()) {
        let mut goal = e.to;
        loop {
            let gt = goal.tile();
            let goal_region = v.tregion(gt, TregionSite::WalkGoal);
            let here_region = v.tregion(ht, TregionSite::WalkStart);
            if goal_region == here_region && (v.domain() != 1 || !v.invalid_loc(gt)) {
                break;
            }
            let (dx, dy) = (here.x.wrapping_sub(goal.x), here.y.wrapping_sub(goal.y));
            let (adx, ady) = (dx.wrapping_abs(), dy.wrapping_abs());
            let s = if ady.wrapping_add(adx) >= 0x300 {
                0x180
            } else {
                0x30
            };
            // The give-up exit: a remainder smaller than the step on both
            // axes takes the goal where it stands **without running A\***
            // (audit V17) — the same push-and-return as reaching the
            // start's cell.
            if adx < s && ady < s {
                v.push(e);
                return v.depth();
            }
            let ang = movement::find_angle(dx, dy);
            // **No sign flip here.** The decompiler's `if (angle < 0) step =
            // -step` before the two `sin_table` calls is the *inlined* fold
            // that [`movement::sin_component`] already performs — and the
            // listing settles it (`0x6894c3`): the cosine's distance is
            // reloaded from the un-negated `s` and negated again only on
            // the sign of `angle + 0x40000000`. Doing it twice cancels the
            // fold and sends the pull-back away from the start
            // (`docs/PATHFINDER.md` §13).
            let sx = movement::sin_component(ang, s);
            let cy = movement::cos_component(ang, s);
            goal = Pos::new(goal.x.wrapping_add(sx), goal.y.wrapping_sub(cy));
            e.to = goal;
            gc = goal.cell();
            if gc == hc {
                v.push(e);
                return v.depth();
            }
        }
    }
    v.push(e);
    // The near test uses the (possibly pulled-back) goal's cell.
    if (hc.y - gc.y).abs() + (hc.x - gc.x).abs() <= 2 {
        return v.depth();
    }
    // `scouting` is read **for a human too**: `type +0x2c8 & 0x10`, an
    // `EXPLORE_TO` order, and the clause that can only turn it off for a
    // scout whose explore order carries `ACTION`.
    if v.is_scout_role()
        && v.order_is_explore_to()
        && v.explore_gate()
        && v.explore_kind_is_explore()
    {
        v.set_scouting();
    }
    let centre = |c: Cell| Pos::new(c.x * 0x300 + 0x180, c.y * 0x300 + 0x180);
    v.push(PathData {
        to: centre(gc),
        tolerance: 0x180,
        flags: 0,
    });
    v.push(PathData {
        to: centre(hc),
        tolerance: 0,
        flags: 0,
    });
    // `army`/`worker` are **AI-only**: a human's `find_wpath` jumps straight
    // to the search past the whole mode block (audit V14).
    if !v.human() {
        let armed = v.has_attack() || v.is_supply();
        if armed && !v.is_worker() && !v.is_attacking() && !v.on_river() {
            v.set_army();
        }
        // The original asks `is_worker` again here, whatever the army
        // block answered.
        if v.is_worker() {
            v.set_worker();
        }
    }
    let r = if v.search() == 0 {
        -i32::from(v.pop_top_is_final())
    } else {
        v.depth()
    };
    v.kill_lists();
    r
}

/// The crate's [`Sim`] as the view: unit `u`.
pub(crate) struct SimWpath<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) u: usize,
    /// The group's own plan forces army mode (`docs/GROUPS.md` §6.7).
    pub(crate) army_hint: bool,
    scouting: bool,
    army: bool,
    worker: bool,
}

impl<'a> SimWpath<'a> {
    pub(crate) fn new(sim: &'a mut Sim, u: usize, army_hint: bool) -> Self {
        SimWpath {
            sim,
            u,
            army_hint,
            scouting: false,
            army: false,
            worker: false,
        }
    }
}

impl WpathView for SimWpath<'_> {
    fn begin(&mut self, start: Pos) {
        self.sim.pf_start = start;
    }
    fn pop(&mut self) -> Option<PathData> {
        self.sim.units[self.u].path.pop()
    }
    fn push(&mut self, e: PathData) {
        self.sim.units[self.u].path.push(e);
    }
    fn push_inline(&mut self, e: PathData) {
        self.sim.units[self.u].path.push(e);
    }
    fn clear(&mut self) {
        self.sim.units[self.u].path.clear();
    }
    fn depth(&self) -> i32 {
        self.sim.units[self.u].path.len() as i32
    }
    fn cell_in_bounds(&self, c: Cell) -> bool {
        self.sim.world.contains(c)
    }
    fn is_helicopter(&self) -> bool {
        self.sim.is_helicopter(self.u)
    }
    fn human(&self) -> bool {
        // **Gaia is a leader and it is a computer one.** `self.nation` has
        // a row per *player*; the original's `leaders.list` has ten, and
        // run58's own dump prints owner 8 with `leader_flags 33554439` —
        // the same `0x2000007` the AI player carries, `& 4` set — against
        // the human's `0x800113`. So an owner with no row here is not
        // human, and a wandering animal takes the AI's mode block.
        self.sim
            .nation
            .get(self.sim.units[self.u].owner as usize)
            .is_some_and(|n| n.human)
    }
    fn is_on_map(&mut self) -> bool {
        self.sim.units[self.u].on_map
    }
    fn can_transport(&mut self) -> bool {
        self.sim.unit_can_transport(self.u)
    }
    fn domain(&self) -> i32 {
        match self.sim.unit_domain_of(self.u) {
            crate::attrition::Domain::Land => 0,
            crate::attrition::Domain::Sea => 1,
            crate::attrition::Domain::Air => 2,
        }
    }
    fn tregion(&mut self, t: Pos, _site: TregionSite) -> i32 {
        self.sim.world.tregion_alt(t).map_or(-1, i32::from)
    }
    fn was_seen(&mut self, c: Cell, _site: SeenSite) -> bool {
        let owner = self.sim.units[self.u].owner;
        self.sim.was_seen_fog(2 * c.x + 1, 2 * c.y + 1, owner)
    }
    fn invalid_loc(&mut self, t: Pos) -> bool {
        self.sim
            .invalid_loc(self.u, t, false, true, true, true, false)
            != 0
    }
    fn is_scout_role(&self) -> bool {
        // `scouting = 1` iff the **type** is a scout — `role & 0x10`,
        // `is(SCOUT)` on land and `is(BARK)` at sea
        // (`UnitType::determine_roles@0061c320`) — **and** the order is
        // `EXPLORE_TO` (§3). `scouting` prices seen ground at `0x400`
        // against unseen `8`, so any unit given an `EXPLORE_TO` walked
        // toward the fog before the type half was read (run10's AI citizen
        // `1/1`).
        self.sim.units[self.u]
            .ty
            .is_some_and(|t| self.sim.unit_types[t].cols.is(crate::ai_load::role::SCOUT))
    }
    fn order_is_explore_to(&mut self) -> bool {
        self.sim
            .current_order(self.u)
            .is_some_and(|o| matches!(o.body, Body::Move(mo) if mo.kind == MoveKind::ExploreTo))
    }
    fn explore_gate(&mut self) -> bool {
        // SEAM: the clause `order.flags & 4 == 0 || unit_masks & 0x40100`,
        // which can only ever turn scouting *off* for a scout whose explore
        // order carries `ACTION`. Left out until the two mask bits are
        // modelled; every explore order in the corpus has `flags 1`.
        true
    }
    fn explore_kind_is_explore(&mut self) -> bool {
        // The order's own kind is the one the `order_type` test just read.
        true
    }
    fn set_scouting(&mut self) {
        self.scouting = true;
    }
    fn has_attack(&self) -> bool {
        self.sim.units[self.u]
            .ty
            .is_some_and(|t| self.sim.unit_types[t].combat.attack > 0)
    }
    fn is_supply(&mut self) -> bool {
        self.sim.units[self.u].ty.is_some_and(|t| {
            self.sim.unit_types[t]
                .cols
                .flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
        })
    }
    fn is_worker(&mut self) -> bool {
        self.sim.units[self.u]
            .ty
            .is_some_and(|t| self.sim.unit_types[t].worker != crate::orders::Worker::None)
    }
    fn is_attacking(&mut self) -> bool {
        // `is_attacking`: the current order's `is_attack` (`006896b9`).
        self.sim
            .current_order(self.u)
            .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Strafe(_)))
    }
    fn on_river(&self) -> bool {
        self.sim
            .world
            .cell_data(self.sim.units[self.u].pos.cell())
            .flags
            & crate::world::cell::HALFLAND
            != 0
    }
    fn set_army(&mut self) {
        self.army = true;
    }
    fn set_worker(&mut self) {
        self.worker = true;
    }
    fn search(&mut self) -> i32 {
        self.sim.wpath_search(
            self.u,
            self.scouting,
            self.army_hint || self.army,
            self.worker,
        )
    }
    fn pop_top_is_final(&mut self) -> bool {
        self.sim.units[self.u]
            .path
            .pop()
            .is_some_and(|p| p.flags & path_flag::FINAL != 0)
    }
    fn kill_lists(&mut self) {
        self.sim.kill_lists();
    }
}
