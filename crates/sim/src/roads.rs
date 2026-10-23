//! Roads — the ring a building lays and the road it plans back to its city.
//!
//! `docs/ROADS.md`. The mechanic is named for the caravan because the search
//! is `PathFinder::astar_caravan_road`, but no caravan is involved in what a
//! traced game actually runs: the caller is a **building**, and the road is
//! the one that joins it to its city centre.
//!
//! Three pieces, in the order a frame reaches them:
//!
//! 1. **The schedule** (§1). `Build::activate` flags every building of its
//!    city through `City::regen_roads`; `Build::process` fires one when
//!    `(frame + o) % 16 == 0` and clears the flag. Diff-backed: the
//!    `build_masks` bit is in every dump and its lifetime on run14 is
//!    exactly this rule.
//! 2. **`BuildType::place_roads`** (§3) — the ring of road tiles around the
//!    footprint, then, for a type that connects to roads and stands in a
//!    city, the road to the centre.
//! 3. **The search** (§5) — a four-connected A\* on the **tile** grid whose
//!    cost function takes **one `game_random` draw a node** (`% 20`, a
//!    jitter). That draw is the whole of this mechanic's presence in
//!    `docs/SYNC.md`'s stream, and its count is the search's expansion
//!    count, so the road is either reproduced exactly or not at all.
//!
//! A **caravan's** road is the same search with three things turned on
//! (§8): all eight directions rather than the four cardinals, a heuristic
//! sixteen times larger, and — because a trade route is long — the right to
//! **stop at the budget and carry on next frame**. `crate::caravan` is the
//! caller; this module owns the search.

use crate::ai_place::{MOVE_X, MOVE_Y};
use crate::build::{self, Ident, flags};
use crate::world::{Cell, Owner, Pos, UNITS_PER_TILE, World, cell, tile, vector_dist};
use crate::{Player, Sim};
use std::cmp::Reverse;
use std::collections::BTreeMap;

/// `PathFinder::calc_road_cost+0x46` — the cost jitter, one draw a node.
pub const SITE_COST: &str = "PathFinder::calc_road_cost+0x46";

/// The nine `road_*` weights `PathFinder::init@00689ec0` copies out of two
/// sixteen-byte `.rdata` literals at `00b69a30` and `00b69a60` with a pair of
/// `movaps`, read from the PE rather than from the decompiler's names for
/// them. `PathFinder+0xa0` through `+0xc0`; the last, 8, is not read by the
/// cost function.
pub mod weight {
    /// `+0xa0` — every node's base.
    pub const BASE: i32 = 55;
    /// `+0xa4` — an ocean tile while `avoid_sea`.
    pub const SEA_AVOIDED: i32 = 100;
    /// `+0xa8` — stepping from land onto ocean.
    pub const SEA_ENTER: i32 = 100;
    /// `+0xac` — a cell owned by somebody who is not a friend.
    pub const FOREIGN: i32 = 540;
    /// `+0xb0` — a cell nobody owns.
    pub const UNOWNED: i32 = 240;
    /// `+0xb4` — the plain-ground term: `×4` off a road, `>>3` on one.
    pub const GROUND: i32 = 200;
    /// `+0xb8` — rough going: `×2` for rock, `×1` for a river tile.
    pub const ROUGH: i32 = 60;
    /// `+0xbc` — the climb's clamp, before `×3`.
    pub const CLIMB: i32 = 600;
}

/// The work cap, in nodes costed: `traversed >= 0xc80` ends the search.
const WORK_CAP: i32 = 0xc80;

/// The stagger: a building replans its road on the frame where
/// `(frame + o) % ROTATION == 0`.
pub const ROTATION: i64 = 16;

/// One node the road search priced, in the order it priced them —
/// `PathFinder::calc_road_cost`'s own arguments and its answer. The
/// original can be asked the same question since run62:
/// `rontrace.cfg`'s `callwin` proxies `calc_road_cost` and the
/// `valid_roadcoord` that admitted the tile, so the sequence is
/// comparable node for node (`docs/ROADS.md` §7.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoadCostMark {
    /// The candidate tile, in world units — the `PathNode` the original
    /// hands the function, whose coordinate only the gate's record has.
    pub to: (i32, i32),
    /// The node it was expanded from, likewise.
    pub from: (i32, i32),
    /// The wheel index, as the original's `dir`.
    pub dir: i32,
    /// What the function answered, **before** the goal neighbour's halving
    /// — which the search does, not the cost.
    pub cost: i32,
}

/// One node of the road search — the original's `PathNode`, minus the fields
/// only the unit grid writes.
#[derive(Clone, Copy, Debug)]
struct Node {
    x: i32,
    y: i32,
    /// `g`, the accumulated `calc_road_cost`.
    length: i32,
    value: i32,
    metric: i64,
    /// `PathNode::z_val`: the tile's height, clamped at zero, and **negated**
    /// when the tile is a road. The sign is what the next node's cost reads
    /// to tell "my parent was already on the road" from "my parent was not".
    z_val: i32,
    parent: Option<u32>,
}

/// The half of one road plan that survives a frame — `docs/ROADS.md` §8.3.
///
/// `astar_caravan_road@00685990` parks its three containers in the
/// `CaravanData` when the budget trips (`+0x28` open, `+0x2c` its refs,
/// `+0x30` closed), together with `offset` (+0x34, the wheel preference)
/// and `endx`/`endy` (+0x38, +0x3c). It writes `traversed` (+0x40) too and
/// **never reads it back**, so every resumed frame starts the budget again
/// at zero — which is why one plan costs 3,200-odd nodes a frame for as
/// many frames as it takes.
#[derive(Clone, Debug, Default)]
pub struct RoadSearch {
    nodes: Vec<Node>,
    open: BTreeMap<(i32, Reverse<u64>), u32>,
    open_by_metric: BTreeMap<i64, (u64, i32, u32)>,
    closed: BTreeMap<i64, u32>,
    seq: u64,
    /// `offset` (+0x34): the wheel index the expansion starts one past.
    pref: i32,
    /// `endx`/`endy` (+0x38, +0x3c), in world units.
    goal: (i32, i32),
}

/// What one call of the search came back with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoadPlan {
    /// The goal tile was popped: the tiles to lay, near-goal end first.
    Road(Vec<Pos>),
    /// `traversed >= 0xc80`. A building's road is simply not laid; a
    /// caravan's [`RoadSearch`] is kept and resumed next frame.
    Budget,
    /// The open list ran dry — there is no road between these two.
    None,
}

impl Sim {
    // ------------------------------------------------------------------
    // §1 — the schedule
    // ------------------------------------------------------------------

    /// `City::regen_roads@00738aa0` — flag every building of the city,
    /// starting at the centre and walking `BuildData::city_down`. Called from
    /// `Build::activate` and `Build::remove_from_city`, and it is what puts
    /// the flag on a farm or a city centre, neither of which `place_roads`
    /// would ever flag itself.
    pub(crate) fn city_regen_roads(&mut self, c: usize) {
        if !self.cities[c].alive {
            return;
        }
        let head = self.cities[c].building;
        self.buildings[head].regen_roads = true;
        let members = self.cities[c].members.clone();
        for b in members {
            self.buildings[b].regen_roads = true;
        }
    }

    /// `Build::process@0061edf0+0x1377` — the deferred regeneration. The
    /// building's own object number staggers the work across sixteen frames,
    /// so a city's seven buildings replan on seven different ones.
    ///
    /// The **caller** owns the other half of the rule: this is
    /// `Build::process`'s last statement and the function returns on
    /// `is_active` at its head, so a site never gets here and its flag
    /// simply waits (`docs/ROADS.md` §1.1, East Indies 8193).
    pub(crate) fn regen_roads_due(&mut self, b: usize, frame: i64) {
        if !self.buildings[b].regen_roads {
            return;
        }
        if (frame + i64::from(self.buildings[b].index)).rem_euclid(ROTATION) != 0 {
            return;
        }
        self.buildings[b].regen_roads = false;
        self.place_roads(b);
    }

    // ------------------------------------------------------------------
    // §3 — `BuildType::place_roads@0063c580`, the `REGEN_FORCE` call
    // ------------------------------------------------------------------

    /// The ring, and then the road to the city centre.
    ///
    /// SEAM: only the `REGEN_FORCE` call `Build::process` makes is modelled,
    /// and the teardown ([`Sim::unplace_roads`]). `BuildType::mask_me`'s
    /// `REGEN_TOTAL` — which sets the flag rather than searching — is
    /// unmodelled; `City::regen_roads` is what actually flags a building
    /// here.
    pub fn place_roads(&mut self, b: usize) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let (w, h) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        let corner = self.tile_corner(ty, self.buildings[b].pos);

        if build::is_city(&self.build_types, ty) {
            // A city lays its own ring and stops: it *is* the destination.
            self.road_ring(corner, w, h, 0, true);
            return;
        }
        if connects_to_roads(&self.build_types, ty) {
            self.road_ring(corner, w, h, 0, true);
        } else {
            // A gatherer gets nothing at all — not the ring, not the road.
            // This is why the four farms and the woodcutter fire their flag
            // on run14's frames 12 to 15 and draw nothing there.
            if self.build_types[ty].has(flags::GATHER) {
                return;
            }
            if self.buildings[b].city.is_none() {
                return;
            }
            // The one arm whose ring runs a tile wider on each side.
            self.road_ring(corner, w, h, 1, true);
        }
        if !self.plan_roads {
            return;
        }
        let Some(c) = self.buildings[b].city else {
            return;
        };
        if !self.cities[c].alive {
            return;
        }
        let centre = self.cities[c].building;
        let road = self.find_road(b, centre);
        // `reconstruct` appends goal-end first, as `astar_caravan_road`
        // does (`00685990`), and `place_roads@0063c580` pops that stack from
        // the top: the road is laid near-*start* end first, and since §9's
        // mesh the order reaches the world (`docs/ROADS.md` §5; the
        // docs-versus-code pass of 2026-09-05 found this walking the other
        // way). Blocked tiles were never pushed.
        for p in road.into_iter().rev() {
            let t = p.tile();
            if self.world.tile_mask(t) & tile::BLOCKED == 0 {
                self.set_road_at(t);
            }
        }
    }

    /// **The teardown: a footprint's ring road goes with it** (item 3314,
    /// `docs/ROADS.md` §13). `BuildType::mask_me@006312a0`'s last
    /// statement on the unmask (`param_4 == 0`, from `Wall::close`) is
    /// `place_roads(…, 0, REGEN_SIMPLE)`: the same three arms and the same
    /// ring, each tile handed to `set_road_at(x, y, param_5 = 0, 0, 0)` —
    /// the mesh door's removal — and no road to the centre, which only
    /// `param_5 != 0` reaches. `Wall::close` runs ahead of
    /// `remove_from_city` in `Build::close@00628980` (lines 144 and 165),
    /// so the third arm's `city >= 0` still holds.
    pub(crate) fn unplace_roads(&mut self, b: usize) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let (w, h) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        if build::is_city(&self.build_types, ty) || connects_to_roads(&self.build_types, ty) {
            self.road_ring(corner, w, h, 0, false);
        } else if !self.build_types[ty].has(flags::GATHER) && self.buildings[b].city.is_some() {
            self.road_ring(corner, w, h, 1, false);
        }
    }

    /// The footprint's ring. `pad` is the one asymmetry between
    /// `place_roads`' three ring loops: the city arm and the
    /// `connects_to_roads` arm walk `corner−1 ..= corner−1+size`, the third
    /// one column and row further. `set` is `place_roads`' `param_5`: a
    /// road laid, or — on the teardown — taken away.
    fn road_ring(&mut self, corner: Pos, w: i32, h: i32, pad: i32, set: bool) {
        let (sx, sy) = (corner.x - 1, corner.y - 1);
        let (ex, ey) = (sx + w + pad, sy + h + pad);
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        for ix in sx..=ex {
            for iy in sy..=ey {
                let ring = ix == sx || ix == ex || iy == sy || iy == ey;
                if !ring || ix < 0 || iy < 0 || ix >= tw || iy >= th {
                    continue;
                }
                let t = Pos::new(ix, iy);
                let m = self.world.tile_mask(t);
                let occupied = m & tile::OBJECT == tile::OBJECT_BUILDING || m & tile::PLACED != 0;
                if m & tile::BLOCKED != 0
                    || (occupied && (ix == sx || iy == sy))
                    || m & tile::SURFACE == tile::SURFACE_OCEAN
                {
                    continue;
                }
                self.world_set_road_at(t, set, 0, 0);
            }
        }
    }

    /// `World::set_road_at@006b43b0` with `set != 0`, as `place_roads` calls
    /// it — the tile's surface becomes road, the cell is marked as carrying
    /// one, and a tile that was **not** already a road goes on to the mesh
    /// (`crate::mesh`, `docs/ROADS.md` §9), which is where a road can grow a
    /// seventeenth tile out of sixteen.
    pub(crate) fn set_road_at(&mut self, t: Pos) {
        self.world_set_road_at(t, true, 0, 0);
    }

    // ------------------------------------------------------------------
    // §4 — `PathFinder::find_road@00688a40`
    // ------------------------------------------------------------------

    /// The endpoints, and the search between them. Returns the road's tiles
    /// as positions, empty when there is no road to lay.
    ///
    /// This is `find_road` with `param_6 < 0` — a building's, cardinal-only
    /// and unresumable. [`Sim::find_caravan_road`] is the other arm.
    pub fn find_road(&mut self, from: usize, to: usize) -> Vec<Pos> {
        let Some(mut st) = self.start_road(from, to) else {
            return Vec::new();
        };
        let plan = self.step_road(&mut st, (from, to), false, false);
        self.kill_lists();
        match plan {
            RoadPlan::Road(r) => r,
            _ => Vec::new(),
        }
    }

    /// `find_road`'s setup, both arms: the two endpoint tiles, the map and
    /// coincidence guards, and the root node. `None` is the `-1` the
    /// original returns without searching.
    ///
    /// `PathFinder::find_road_restore@00685950` is this step **skipped** —
    /// it sets `pathfinder+0x84` so that neither endpoint is pushed and the
    /// parked search is picked up where it stopped.
    pub(crate) fn start_road(&mut self, from: usize, to: usize) -> Option<RoadSearch> {
        let (Some(fty), Some(tty)) = (self.buildings[from].ty, self.buildings[to].ty) else {
            return None;
        };
        let start = self.road_end(from, fty);
        let goal = self.road_end(to, tty);
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        let inside = |p: Pos| p.x >= 0 && p.y >= 0 && p.x < tw && p.y < th;
        if !inside(start) || !inside(goal) || start == goal {
            return None;
        }
        Some(self.road_root(centre_of(start), centre_of(goal)))
    }

    /// One endpoint's tile: a **city**'s own tile, and for anything else the
    /// far corner of its footprint — `corner + size − 1`, which for an even
    /// footprint is the object's own tile again and for an odd one is half a
    /// footprint past it.
    pub(crate) fn road_end(&self, b: usize, ty: usize) -> Pos {
        if build::is_city(&self.build_types, ty) {
            return self.buildings[b].pos.tile();
        }
        let corner = self.tile_corner(ty, self.buildings[b].pos);
        let (w, h) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        Pos::new(corner.x - 1 + w, corner.y - 1 + h)
    }

    // ------------------------------------------------------------------
    // §5 — `PathFinder::astar_caravan_road@00685990`
    // ------------------------------------------------------------------

    /// The root node, the wheel preference and the goal — everything the
    /// search needs before its first pop. The original builds these in
    /// `astar_caravan_road`'s **fresh** arm, off the two `PathData` it pops
    /// from the stack `find_road` pushed.
    fn road_root(&self, start: Pos, goal: Pos) -> RoadSearch {
        let row = i64::from(self.world.width()) * 4;
        let mut st = RoadSearch {
            goal: (goal.x, goal.y),
            ..RoadSearch::default()
        };
        // The root's estimate is the shared `get_estimate` at the tile
        // stride — `d × 60 / 0xc0` — where every later node's is
        // `d × 60 / 0x180` (or, for a caravan, `d × 40 / 16`). The
        // inconsistency is real (both were read off the listing's two
        // divide-by-magic sequences) and harmless: the root is the only
        // node in the open list when it is popped.
        let root_h = vector_dist(start.x - goal.x, start.y - goal.y) * 60 / 0xc0;
        st.nodes.push(Node {
            x: start.x,
            y: start.y,
            length: 0,
            value: root_h,
            metric: i64::from(start.x.div_euclid(UNITS_PER_TILE))
                + i64::from(start.y.div_euclid(UNITS_PER_TILE)) * row,
            // Unclamped and never negated: a search that starts on a road
            // does not get the road bonus on its first step.
            z_val: self.world.tile_z(start.tile()),
            parent: None,
        });
        st.open.insert((root_h, Reverse(st.seq)), 0);
        st.open_by_metric
            .insert(st.nodes[0].metric, (st.seq, root_h, 0));
        st.seq += 1;
        // The wheel's preference — a diagonal index, so `pref + 1` is the
        // cardinal facing the goal (`docs/PATHFINDER.md` §4.2). A building
        // expands only the cardinals, so its order is `pref+1, +3, +5, +7`;
        // a caravan takes all eight.
        let (dx0, dy0) = (start.x - goal.x, start.y - goal.y);
        st.pref = if dy0.abs() < dx0.abs() {
            if goal.x < start.x { 7 } else { 3 }
        } else if goal.y < start.y {
            1
        } else {
            5
        };
        st
    }

    /// One frame's worth of the search — `astar_caravan_road`'s loop, from
    /// the first pop to the goal, the budget or an empty open list.
    ///
    /// `caravan` is `param_6 >= 0`: it opens the four diagonals
    /// (`local_18 = param_6 >> 31` gates the odd wheel indices out for a
    /// building and not for a caravan) and swaps the heuristic. Positions
    /// are in world units and the stride is one tile.
    fn step_road(
        &mut self,
        st: &mut RoadSearch,
        ends: (usize, usize),
        caravan: bool,
        can_transport: bool,
    ) -> RoadPlan {
        let row = i64::from(self.world.width()) * 4;
        let goal = Pos::new(st.goal.0, st.goal.1);
        let who = (self.buildings[ends.0].owner, self.buildings[ends.1].owner);
        // `pathfinder+0x8c`: both endpoints in one region. The cost
        // function reads it on an ocean tile only, and no ocean tile is
        // valid while `can_transport` is zero — so this is recomputed
        // rather than parked, exactly as the original recomputes it on
        // every call including a restore.
        let ea = self.buildings[ends.0]
            .ty
            .map(|ty| self.road_end(ends.0, ty));
        let eb = self.buildings[ends.1]
            .ty
            .map(|ty| self.road_end(ends.1, ty));
        let avoid_sea = match (ea, eb) {
            (Some(a), Some(b)) => {
                self.world.tregion(a).is_some() && self.world.tregion(a) == self.world.tregion(b)
            }
            _ => false,
        };

        let mut traversed: i32 = 0;
        while let Some((&key, &cur_id)) = st.open.first_key_value() {
            st.open.remove(&key);
            let cur = st.nodes[cur_id as usize];
            // `first_open_node` tombstones the refs entry for the popped
            // node's metric; removing it is the same thing.
            if st
                .open_by_metric
                .get(&cur.metric)
                .is_some_and(|&(s, _, _)| s == key.1.0)
            {
                st.open_by_metric.remove(&cur.metric);
            }
            if (cur.x - goal.x).abs() + (cur.y - goal.y).abs() < 1 {
                return RoadPlan::Road(reconstruct(&self.world, &st.nodes, cur_id));
            }
            if traversed >= WORK_CAP {
                // The budget. A **caravan's** popped node goes back on the
                // open list before the search is parked, which is what
                // makes a resumed frame carry on rather than lose a node;
                // a building's is recycled and the road is simply not laid.
                if caravan {
                    st.open.insert((cur.value, Reverse(st.seq)), cur_id);
                    st.open_by_metric
                        .insert(cur.metric, (st.seq, cur.value, cur_id));
                    st.seq += 1;
                }
                return RoadPlan::Budget;
            }

            for k in (st.pref + 1..).take(8) {
                let d = if k < 9 { k } else { k - 8 };
                if !caravan && d % 2 != 0 {
                    continue;
                }
                let d = usize::try_from(d).expect("wheel index");
                let nx = cur.x + MOVE_X[d] * UNITS_PER_TILE;
                let ny = cur.y + MOVE_Y[d] * UNITS_PER_TILE;
                let metric = cur.metric + i64::from(MOVE_X[d]) + i64::from(MOVE_Y[d]) * row;
                let p = Pos::new(nx, ny);
                if !self.valid_roadcoord(p, Pos::new(cur.x, cur.y), ends, can_transport) {
                    continue;
                }
                traversed += 1;
                let from = Pos::new(cur.x, cur.y);
                let (mut cost, z_val) = self.calc_road_cost(p, from, cur.z_val, who, d, avoid_sea);
                if self.trace_costs {
                    self.road_marks.push(RoadCostMark {
                        to: (nx, ny),
                        from: (cur.x, cur.y),
                        dir: i32::try_from(d).expect("wheel index"),
                        cost,
                    });
                }
                if nx == goal.x && ny == goal.y {
                    cost /= 2;
                }
                let g = cur.length + cost;
                if st.closed.contains_key(&metric) {
                    continue;
                }
                if let Some(&(old_seq, old_value, old_id)) = st.open_by_metric.get(&metric) {
                    if st.nodes[old_id as usize].length <= g {
                        continue;
                    }
                    st.open.remove(&(old_value, Reverse(old_seq)));
                    st.open_by_metric.remove(&metric);
                }
                // **The two heuristics, and the caravan's is wrong on
                // purpose — the original's purpose, not ours.** A building's
                // is `vector_dist(dx, dy) × 60 / 0x180`. A caravan's is the
                // listing's other arm at `00685fd9`, and it calls
                // `vector_dist` with `ecx = node.x − goal.x` and
                // `edx = −goal.y`: the goal's **own y coordinate** stands
                // where the y *difference* belongs, and `vector_dist` takes
                // the absolute value of both, so the sign is lost too. The
                // estimate is therefore all but constant — it varies only
                // with `dx²/(2·goal.y)` — and two nodes in the same column
                // score identically whatever their y.
                //
                // That is the whole reason a trade route costs thirteen
                // thousand nodes where a building's road costs three
                // hundred: with no pull along y the search is a breadth-
                // first flood in that axis. run64's frame 6166 is the
                // measurement — the second node popped is the **west**
                // neighbour at cost 86, not the north-west one at 135,
                // and it can only be if the two carry the same estimate.
                let dist = if caravan {
                    vector_dist(nx - goal.x, goal.y)
                } else {
                    vector_dist(nx - goal.x, ny - goal.y)
                };
                let h = if caravan {
                    dist * 40 / 16
                } else {
                    dist * 60 / 0x180
                };
                let node = Node {
                    x: nx,
                    y: ny,
                    length: g,
                    value: g + h,
                    metric,
                    z_val,
                    parent: Some(cur_id),
                };
                let id = u32::try_from(st.nodes.len()).expect("road nodes");
                st.nodes.push(node);
                st.open.insert((node.value, Reverse(st.seq)), id);
                st.open_by_metric.insert(metric, (st.seq, node.value, id));
                st.seq += 1;
            }
            st.closed.insert(cur.metric, cur_id);
        }
        RoadPlan::None
    }

    /// The caravan arm of the search, for `crate::caravan`: eight
    /// directions and the larger heuristic, resumable across frames.
    pub(crate) fn step_road_caravan(
        &mut self,
        st: &mut RoadSearch,
        ends: (usize, usize),
        can_transport: bool,
    ) -> RoadPlan {
        let plan = self.step_road(st, ends, true, can_transport);
        // `find_road@00688a40:46`: `kill_lists` after the caravan search,
        // parked or not.
        self.kill_lists();
        plan
    }

    /// `PathFinderData::valid_roadcoord@00688740`: the tile test, and then
    /// — for a **diagonal** step — the corner rule at its tail.
    ///
    /// A diagonal candidate is admitted only if at least one of the two
    /// tiles that share a side with both it and its parent is itself
    /// admissible, so a road never cuts between two blocked corners. The
    /// recursion is one deep: each corner call has an axis in common with
    /// its parent, so its own diagonal test is false. It was dead until the
    /// caravan arm existed, the four cardinals being all a building expands.
    ///
    /// `can_transport` is `pathfinder+0x98`, which `find_road` sets from
    /// the **caravan unit's** own `UnitData::can_transport@0046f960` and
    /// leaves at zero for a building. With it an ocean tile is admitted,
    /// which is the only thing that makes `calc_road_cost`'s ocean arm
    /// reachable — and East Indies' caravan has it, so its route walks
    /// straight over the water between two coasts.
    pub fn valid_roadcoord(
        &self,
        p: Pos,
        from: Pos,
        ends: (usize, usize),
        can_transport: bool,
    ) -> bool {
        if !self.roadcoord_tile(p, ends, can_transport) {
            return false;
        }
        if p.x != from.x && p.y != from.y {
            return self.valid_roadcoord(Pos::new(p.x, from.y), from, ends, can_transport)
                || self.valid_roadcoord(Pos::new(from.x, p.y), from, ends, can_transport);
        }
        true
    }

    /// The tile half of `valid_roadcoord`, above its diagonal tail.
    fn roadcoord_tile(&self, p: Pos, ends: (usize, usize), can_transport: bool) -> bool {
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        if p.x < 0 || p.y < 0 || p.x >= tw * UNITS_PER_TILE || p.y >= th * UNITS_PER_TILE {
            return false;
        }
        let t = p.tile();
        let m = self.world.tile_mask(t);
        if !can_transport && m & tile::SURFACE == tile::SURFACE_OCEAN {
            return false;
        }
        let blocked = m & tile::BLOCKED != 0;
        let occupied = m & tile::OBJECT == tile::OBJECT_BUILDING || m & tile::PLACED != 0;
        if !(blocked || (occupied && m & tile::SURFACE != tile::SURFACE_ROAD)) {
            return true;
        }
        if !occupied {
            return false;
        }
        // An occupied tile is passable only when it belongs to one of the
        // two endpoints **and that endpoint is a city** — the listing tests
        // `is_city` separately before each `covers_tile`. So a road may
        // cross the city's own footprint and never the building's.
        self.road_end_covers(ends.0, t) || self.road_end_covers(ends.1, t)
    }

    /// One `valid_roadcoord` exemption: the endpoint is a city and the tile
    /// is inside its footprint.
    fn road_end_covers(&self, b: usize, t: Pos) -> bool {
        self.buildings[b]
            .ty
            .is_some_and(|ty| build::is_city(&self.build_types, ty))
            && self.covers_tile(b, t)
    }

    /// `PathFinder::calc_road_cost@00686300`, read off the listing rather
    /// than the decompiler. Returns the cost and the node's `z_val`; the
    /// draw is the first thing it does, so a frame's draw count is exactly
    /// its count of nodes costed. The arithmetic is [`road_cost_steps`]
    /// (`sweep::path_finder_calc_road_cost` holds it to the original); this
    /// reads the world and answers its callees.
    fn calc_road_cost(
        &mut self,
        p: Pos,
        from: Pos,
        parent_z: i32,
        who: (Player, Player),
        dir: usize,
        avoid_sea: bool,
    ) -> (i32, i32) {
        let t = p.tile();
        self.mark(SITE_COST);
        // `Random::get(0, 0xffff)` — the jitter that makes two roads
        // between the same two buildings different roads.
        let roll = self.rng.roll();
        let c = Cell::new(t.x.div_euclid(4), t.y.div_euclid(4));
        let owner = match self.world.owner(c) {
            Owner::None => -1,
            // The original writes `-2` here and every consumer treats a
            // negative byte as unowned (`00686438`, `js`).
            Owner::Ambiguous => -2,
            Owner::Player(o) => i32::from(o),
        };
        let this = &*self;
        let alliance = |a: Player, b: Player| if this.allied_with(a, b) { 2 } else { 0 };
        let out = road_cost_steps(
            &SHIPPED_WEIGHTS,
            avoid_sea,
            !dir.is_multiple_of(2),
            RoadTile {
                at: (t.x, t.y),
                mask: this.world.tile_mask(t),
                parent_mask: this.world.tile_mask(from.tile()),
                cell_flags: this.world.cell_data(c).flags,
                owner,
                parent_z,
            },
            (i32::from(who.0), i32::from(who.1)),
            || roll,
            // `leaders.list[owner]`'s `who`, its `diplos[who.0]` and the
            // `leadersc` row's `diplos[owner]`: `2` is an alliance.
            |o| {
                let o = Player::try_from(o).expect("an owned cell");
                Leader {
                    who: i32::from(o),
                    fwd: alliance(o, who.0),
                    back: alliance(who.0, o),
                }
            },
            |a, b| {
                let player = |x: i32| Player::try_from(x).unwrap_or(Player::MAX);
                this.is_ally(player(a), player(b))
            },
            || this.world.tile_z(t),
            |_, _| this.road_was_seen(t, who.0),
        );
        if std::env::var("RON_PROBE_3524").is_ok() && t.x == 174 && t.y == 238 {
            eprintln!(
                "PROBE3524 cost f{} p {:?} from {:?} parent_z {parent_z} who {who:?} dir {dir} roll {roll} cell {:?} owner {owner} mask {:#x} pmask {:#x} flags {:#x} z {} -> {out:?}",
                self.frame,
                (p.x, p.y),
                (from.x, from.y),
                (c.x, c.y),
                self.world.tile_mask(t),
                self.world.tile_mask(from.tile()),
                self.world.cell_data(c).flags,
                self.world.tile_z(t)
            );
        }
        out
    }

    /// `WorldData::was_seen@006b53f0` at the tile's fog cell — the one
    /// **with** the ally-territory shortcut, which is a different function
    /// from `was_really_seen@006b54f0` with one letter between their names
    /// (`crate::scout`). This read the bare one until run62, and the six
    /// nodes it doubled that the original did not are what said so
    /// (`docs/ROADS.md` §7.3): every one is a cell of the searcher's own
    /// territory whose `seen2` bit is clear, and the original calls it
    /// seen because the owner holds a city in the cell's region.
    fn road_was_seen(&self, t: Pos, who: Player) -> bool {
        let r = self.was_seen_fog(t.x >> 1, t.y >> 1, who);
        if std::env::var("RON_PROBE_3524").is_ok() && t.x == 174 && t.y == 238 {
            let cell = Cell::new(t.x >> 2, t.y >> 2);
            let reg = self.world.region_of(cell);
            let owner = self.world.owner(cell);
            let rc: Vec<_> = (0..2u8)
                .map(|o| reg.map(|r| (self.leader_reg_cities(o, r), self.reg_forts(o, r))))
                .collect();
            eprintln!(
                "PROBE3524 f{} tile {:?} who {who} seen {r} owner {owner:?} region {reg:?} reg_cities/forts {rc:?} seen2 {:?} every {}",
                self.frame,
                (t.x, t.y),
                self.world.seen2(t.x >> 1, t.y >> 1),
                self.sees_every_cell(who)
            );
        }
        r
    }
}

/// The weights `PathFinder+0xa0..` the cost function reads, in the order it
/// reads them; [`weight`] holds the shipped values.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RoadWeights {
    /// `+0xa0`.
    pub base: i32,
    /// `+0xa4`.
    pub sea_avoided: i32,
    /// `+0xa8`.
    pub sea_enter: i32,
    /// `+0xac`.
    pub foreign: i32,
    /// `+0xb0`.
    pub unowned: i32,
    /// `+0xb4`.
    pub ground: i32,
    /// `+0xb8`.
    pub rough: i32,
    /// `+0xbc`.
    pub climb: i32,
}

/// [`weight`] as the struct the arithmetic takes.
pub(crate) const SHIPPED_WEIGHTS: RoadWeights = RoadWeights {
    base: weight::BASE,
    sea_avoided: weight::SEA_AVOIDED,
    sea_enter: weight::SEA_ENTER,
    foreign: weight::FOREIGN,
    unowned: weight::UNOWNED,
    ground: weight::GROUND,
    rough: weight::ROUGH,
    climb: weight::CLIMB,
};

/// What the cost function reads of the node's tile.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RoadTile {
    /// The tile's coordinates.
    pub at: (i32, i32),
    /// Its mask word.
    pub mask: u16,
    /// Its parent's mask word.
    pub parent_mask: u16,
    /// The cell record's first word (`& 8` is rock).
    pub cell_flags: u16,
    /// The cell record's byte `+0xf`, signed: the owner, negative for none.
    pub owner: i32,
    /// The parent node's `z_val` (a short; the sign carries "on the road").
    pub parent_z: i32,
}

/// `leaders.list[owner]` and the `leadersc` row, as `00686300` reads them.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Leader {
    /// `+0x8`, the leader's own player.
    pub who: i32,
    /// `diplos[param_2]` of the owner.
    pub fwd: i32,
    /// `leadersc.list[param_2].diplos[owner.who]`.
    pub back: i32,
}

/// `PathFinder::calc_road_cost@00686300`'s arithmetic, its callees handed
/// in: `roll` is `Random::get(0, 0xffff)`, the first thing it does;
/// `leader` reads the owner's records and `is_ally` is
/// `LeaderData::is_ally@006edb50` (owner, `who.1`); `tile_z` is
/// `find_tcoord_z`'s answer for a land tile and `seen` is
/// `WorldData::was_seen` at the half-tile — each called only where the
/// original calls it. Returns the cost and the node's new `z_val`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn road_cost_steps(
    w: &RoadWeights,
    avoid_sea: bool,
    diagonal: bool,
    t: RoadTile,
    who: (i32, i32),
    roll: impl FnOnce() -> i32,
    leader: impl FnOnce(i32) -> Leader,
    is_ally: impl FnOnce(i32, i32) -> bool,
    tile_z: impl FnOnce() -> i32,
    seen: impl FnOnce(i32, i32) -> bool,
) -> (i32, i32) {
    let jitter = roll() % 0x14;
    let mut removable = jitter;
    let mut total = jitter + w.base;
    let m = t.mask;
    let surface = m & tile::SURFACE;
    let mut z_val = 0;

    if surface == tile::SURFACE_OCEAN {
        // `avoid_sea` is `pathfinder +0x8c`: with it every ocean tile pays
        // `+0xa4`; without it only the step off land pays `+0xa8`, when
        // the **parent's** tile is not ocean (`00686392`).
        if avoid_sea {
            total += w.sea_avoided;
        } else if t.parent_mask & tile::SURFACE != tile::SURFACE_OCEAN {
            total += w.sea_enter;
        }
    } else {
        // Friendly: the owner is the searcher or its ally both ways, and
        // an ally of `who.1` (`0068643a`..`00686477`).
        let mut friendly = false;
        if t.owner < 0 {
            total += w.unowned;
        } else {
            let l = leader(t.owner);
            if (who.0 == l.who || (l.fwd == 2 && l.back == 2)) && is_ally(t.owner, who.1) {
                friendly = true;
            } else {
                total += w.foreign;
            }
        }
        if m & (tile::BAD_PATH | tile::BLOCKED) == tile::BAD_PATH && surface != tile::SURFACE_ROAD {
            total += w.ground;
        }
        if t.cell_flags & cell::ROCK != 0 {
            total += w.rough * 2;
            removable += w.rough * 2;
        }
        if m & tile::RIVER != 0 {
            total += w.rough;
            removable += w.rough;
        }
        // The height is a short, floored at zero.
        let z = i32::from(tile_z() as i16).max(0);
        z_val = z;
        let climb = (z - i32::from(t.parent_z as i16).abs()).abs().min(w.climb);
        total += climb * 3;
        removable += climb * 3;

        let on_road = surface == tile::SURFACE_ROAD;
        let parent_below = (t.parent_z as i16) < 0;
        // `ground >> 3`, rounded toward zero (`cdq; and 7; add; sar 3`).
        let eighth = (w.ground + ((w.ground >> 31) & 7)) >> 3;
        if !friendly {
            if on_road {
                if parent_below {
                    total -= removable;
                } else {
                    total += w.ground * 4;
                }
                z_val = -z;
            } else {
                total += w.ground * 4;
            }
        } else if on_road {
            if parent_below {
                total = (total - removable) / 2;
            } else {
                total += eighth;
            }
            z_val = -z;
        } else {
            total += eighth;
        }
    }

    if !seen(t.at.0 >> 1, t.at.1 >> 1) {
        total *= 2;
    }
    if diagonal {
        total = total * 7 / 5;
    }
    (total, z_val)
}

/// `BuildTypeData::connects_to_roads@006398c0`: a gatherer, or a type that
/// needs no city, connects only if it is a University; everything else
/// connects.
pub fn connects_to_roads(types: &[build::BuildType], t: usize) -> bool {
    if types[t].has(flags::GATHER) || types[t].has(flags::NO_CITY) {
        return build::is(types, t, Ident::University);
    }
    true
}

/// The tile's centre, in world units.
fn centre_of(t: Pos) -> Pos {
    Pos::new(
        t.x * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        t.y * UNITS_PER_TILE + UNITS_PER_TILE / 2,
    )
}

/// The walk back up the parent chain. The original starts at the arrived
/// node's **parent** and stops before the root, so neither endpoint's own
/// tile is laid — the ring already covers those.
///
/// **A blocked tile never enters the stack** (`astar_caravan_road@00685990`
/// `+0x996`: `mask & 0x4000` skips the push). It made no difference while
/// the answer was only ever laid — `place_roads` refuses a blocked tile on
/// the way out — but a caravan *walks* its own road, and a waypoint it
/// cannot stand on is not the same thing as one that simply carries no
/// tarmac (`docs/CARAVAN.md` §5.3).
fn reconstruct(world: &World, nodes: &[Node], end: u32) -> Vec<Pos> {
    let mut out = Vec::new();
    let mut cur = nodes[end as usize].parent;
    while let Some(id) = cur {
        let n = nodes[id as usize];
        if n.parent.is_none() {
            break;
        }
        let p = Pos::new(n.x, n.y);
        if world.tile_mask(p.tile()) & tile::BLOCKED == 0 {
            out.push(p);
        }
        cur = n.parent;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::build::BuildType;
    use crate::world::{Terrain, World};

    /// A 20×20-cell land world with a finished 7×7 city of player 0 at
    /// `centre` (in tiles) and a 3×3 library type ready to place.
    fn town(centre: Pos) -> (Sim, usize, usize, usize) {
        let mut w = World::new(20, 20);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(19, 19));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; crate::economy::RESOURCES];
        }
        // The city's own `7x7 extra space` mask: it stands on seven tiles
        // each way and blocks only the inner five, which is what leaves its
        // ring's outer row and column free for a road (`docs/ROADS.md` §3).
        let mut mask = vec![0u8; 49];
        for v in 1..6 {
            for u in 1..6 {
                mask[v * 7 + u] = 1;
            }
        }
        let city_ty = sim.add_build_type(BuildType {
            ident: Ident::Village,
            flags: flags::parse("ean"),
            x_size: 7,
            y_size: 7,
            job_time: 1,
            hits: 1200,
            block_mask: mask,
            ..BuildType::default()
        });
        let lib_ty = sim.add_build_type(BuildType {
            ident: Ident::Library,
            flags: flags::parse("a"),
            x_size: 3,
            y_size: 3,
            job_time: 1,
            hits: 400,
            ..BuildType::default()
        });
        let city = sim
            .place_building(0, city_ty, centre_of(centre))
            .unwrap_or_else(|e| panic!("the city should place: {e:?}"));
        finish(&mut sim, city);
        (sim, city, city_ty, lib_ty)
    }

    fn finish(sim: &mut Sim, b: usize) {
        let mut guard = 0;
        while !sim.buildings[b].active && sim.buildings[b].alive {
            sim.do_construct(b, 1_000_000);
            guard += 1;
            assert!(guard < 10, "a million a frame should finish anything");
        }
        sim.buildings[b].helpers = 0;
    }

    fn library_at(sim: &mut Sim, lib_ty: usize, t: Pos) -> usize {
        let b = sim
            .place_building(0, lib_ty, centre_of(t))
            .unwrap_or_else(|e| panic!("the library should place: {e:?}"));
        finish(sim, b);
        b
    }

    fn is_road(sim: &Sim, x: i32, y: i32) -> bool {
        sim.world.tile_mask(Pos::new(x, y)) & tile::SURFACE == tile::SURFACE_ROAD
    }

    #[test]
    fn a_gatherer_neither_rings_nor_roads_and_a_university_does_both() {
        let types = vec![
            BuildType {
                ident: Ident::Farm,
                flags: flags::GATHER,
                ..BuildType::default()
            },
            BuildType {
                ident: Ident::University,
                flags: flags::GATHER,
                ..BuildType::default()
            },
            BuildType {
                ident: Ident::Library,
                ..BuildType::default()
            },
            BuildType {
                ident: Ident::Other,
                flags: flags::NO_CITY,
                ..BuildType::default()
            },
        ];
        assert!(!connects_to_roads(&types, 0), "a farm does not connect");
        assert!(connects_to_roads(&types, 1), "a university does");
        assert!(connects_to_roads(&types, 2), "a library does");
        assert!(!connects_to_roads(&types, 3), "a no-city type does not");
    }

    /// `Wall::start@0063e810` passes `REGEN_FORCE` to `mask_me`, whose
    /// tail is `place_roads` — so the ring and the road are laid the moment
    /// a building **starts**, and the flag `City::regen_roads` sets is what
    /// makes it happen *again* later. Before run32 nothing here modelled
    /// that, because no traced game had placed a non-farm building: the
    /// setup's own go up before the first frame and the AI never got past
    /// its citizens (`docs/QUEUE.md` item 74).
    #[test]
    fn a_building_lays_its_road_when_it_starts_not_when_its_flag_comes_round() {
        let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let before = sim.rng.seed;
        // Placed, not yet started: nothing is laid and nothing is drawn.
        let b = sim
            .place_building(0, lib_ty, centre_of(Pos::new(40, 52)))
            .expect("the library places");
        assert!(!sim.buildings[b].started, "a site is not started");
        assert!(!is_road(&sim, 40, 52), "an unstarted site has laid no ring");
        assert_eq!(sim.rng.seed, before, "and taken no draw");
        // Starting it lays the ring and plans the road in one go.
        sim.start_building(b);
        assert!(is_road(&sim, 38, 50), "the ring is down");
        assert!(
            sim.rng.seed != before,
            "and the search has drawn its jitter a node"
        );
        // The road is a *path* between the two rings, not a column: since
        // 2026-09-01 a blocked tile puts `BAD_PATH` on its eight
        // neighbours (`World::set_blocked_at`, `docs/PATHFINDER.md` §5),
        // which costs `weight::GROUND` and pushes the route a tile off the
        // straight line it used to take. What is asserted is that it runs.
        let road: Vec<(i32, i32)> = (44..52)
            .flat_map(|y| (37..44).map(move |x| (x, y)))
            .filter(|&(x, y)| is_road(&sim, x, y))
            .collect();
        assert!(
            road.len() >= 4 && road.iter().any(|&(_, y)| y <= 46),
            "a road runs back towards the city: {road:?}"
        );
    }

    #[test]
    fn the_flag_fires_on_the_frame_its_object_number_picks_out() {
        let (mut sim, city, _, _) = town(Pos::new(40, 40));
        // `Build::activate` has already flagged the city through
        // `City::regen_roads`.
        assert!(sim.buildings[city].regen_roads, "activation flags the city");
        let o = i64::from(sim.buildings[city].index);
        for frame in 0..64 {
            let due = (frame + o).rem_euclid(ROTATION) == 0;
            sim.buildings[city].regen_roads = true;
            sim.regen_roads_due(city, frame);
            assert_eq!(
                sim.buildings[city].regen_roads, !due,
                "frame {frame}: object {o} fires when (frame + o) % 16 == 0"
            );
        }
    }

    /// `Build::process@0061edf0`'s `is_active` gate — `field_0x8 & 4`,
    /// which is `WallData::is_active@00472350` inlined — stands above the
    /// replan at the function's tail, so a **site** never replans however
    /// its flag was set, and the flag is not cleared either.
    ///
    /// East Indies 8193 is the frame that said so: `1/2015`, a half-built
    /// market, spent two hundred `calc_road_cost` draws the original never
    /// spends (`docs/ROADS.md` §1.1, item 334).
    #[test]
    fn a_site_does_not_replan_and_keeps_its_flag_until_it_finishes() {
        let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let b = sim
            .place_building(0, lib_ty, centre_of(Pos::new(40, 52)))
            .expect("the library places");
        sim.start_building(b);
        assert!(!sim.buildings[b].active, "still a site");
        let o = i64::from(sim.buildings[b].index);
        // Its own rotation slot, the frame `regen_roads_due` would fire on.
        let due = (ROTATION - o).rem_euclid(ROTATION);
        sim.buildings[b].regen_roads = true;
        let before = sim.rng.seed;
        sim.process_building_whole(b, due);
        assert_eq!(sim.rng.seed, before, "a site spends no road-cost draw");
        assert!(
            sim.buildings[b].regen_roads,
            "and the flag waits rather than being cleared"
        );
        // Finished, the same slot fires — so it is the gate that held it,
        // not the schedule.
        finish(&mut sim, b);
        sim.buildings[b].regen_roads = true;
        sim.process_building_whole(b, due);
        assert!(
            sim.rng.seed != before,
            "an active building replans on that same frame"
        );
        assert!(!sim.buildings[b].regen_roads, "and clears its flag");
    }

    #[test]
    fn the_ring_is_the_border_of_the_footprint_grown_by_one() {
        let (mut sim, city, _, _) = town(Pos::new(40, 40));
        sim.place_roads(city);
        // A 7×7 city at tile (40,40) has its corner at (37,37); the ring
        // runs from (36,36) to (36+7, 36+7) = (43,43), border only.
        for i in 36..=43 {
            assert!(is_road(&sim, i, 36), "top edge at {i}");
            assert!(is_road(&sim, i, 43), "bottom edge at {i}");
            assert!(is_road(&sim, 36, i), "left edge at {i}");
            assert!(is_road(&sim, 43, i), "right edge at {i}");
        }
        assert!(!is_road(&sim, 40, 40), "the footprint itself is not a road");
        assert!(!is_road(&sim, 35, 35), "nor anything outside the ring");
    }

    /// **The ring goes with the building** (item 3314, `docs/ROADS.md`
    /// §13): `mask_me`'s unmask ends in `place_roads(…, 0, REGEN_SIMPLE)`,
    /// so a closed Library's ring is handed back through `set_road_at(…,
    /// 0, 0, 0)` — and the road it planned to the centre, which only
    /// `param_5 != 0` reaches, stays.
    #[test]
    fn a_closed_building_takes_its_ring_road_with_it_and_leaves_its_road() {
        let (mut sim, _city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let b = library_at(&mut sim, lib_ty, Pos::new(40, 52));
        // A 3×3 library at (40,52) has its corner at (39,51): the ring runs
        // (38,50) to (41,53), border only.
        let ring: Vec<(i32, i32)> = (38..=41)
            .flat_map(|x| (50..=53).map(move |y| (x, y)))
            .filter(|&(x, y)| (x == 38 || x == 41 || y == 50 || y == 53) && is_road(&sim, x, y))
            .collect();
        let road: Vec<(i32, i32)> = (44..50)
            .flat_map(|y| (34..47).map(move |x| (x, y)))
            .filter(|&(x, y)| is_road(&sim, x, y))
            .collect();
        // The box's bottom row and right column are the footprint's own,
        // blocked here, so the ring is its top row and left column.
        assert_eq!(ring.len(), 7, "the ring is down: {ring:?}");
        assert!(!road.is_empty(), "and a road runs back towards the city");
        sim.close_building(b, false);
        let left: Vec<_> = ring.iter().filter(|&&(x, y)| is_road(&sim, x, y)).collect();
        assert!(left.is_empty(), "the ring went with it: {left:?} stand");
        let kept: Vec<_> = road
            .iter()
            .filter(|&&(x, y)| !is_road(&sim, x, y))
            .collect();
        assert!(
            kept.is_empty(),
            "and the road to the centre stays: {kept:?} went"
        );
    }

    #[test]
    fn laying_a_road_is_idempotent_and_takes_an_ocean_tile_to_land() {
        let (mut sim, _, _, _) = town(Pos::new(40, 40));
        let t = Pos::new(3, 3);
        sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
        sim.set_road_at(t);
        assert_eq!(sim.world.tile_mask(t), tile::SURFACE_ROAD);
        assert!(sim.world.cell_data(Cell::new(0, 0)).flags & cell::ROAD != 0);
        let before = sim.world.tile_mask(t);
        sim.set_road_at(t);
        assert_eq!(sim.world.tile_mask(t), before, "twice is once");
    }

    #[test]
    fn an_endpoint_is_the_city_s_tile_and_any_other_type_s_far_corner() {
        let (mut sim, city, city_ty, lib_ty) = town(Pos::new(40, 40));
        let lib = library_at(&mut sim, lib_ty, Pos::new(34, 40));
        assert_eq!(
            sim.road_end(city, city_ty),
            Pos::new(40, 40),
            "a city answers its own tile"
        );
        // A 3×3 library at (34,40) has its corner at (33,39); its endpoint
        // is `corner + 3 − 1` = (35,41).
        assert_eq!(sim.road_end(lib, lib_ty), Pos::new(35, 41));
    }

    #[test]
    fn a_road_is_planned_between_two_buildings_and_laid_where_it_is_not_blocked() {
        let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let lib = library_at(&mut sim, lib_ty, Pos::new(33, 40));
        let road = sim.find_road(lib, city);
        assert!(!road.is_empty(), "a road is found on open ground");
        // Every step of the chain is a cardinal move of one tile.
        let tiles: Vec<Pos> = road.iter().map(|p| p.tile()).collect();
        for w in tiles.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert_eq!(
                (a.x - b.x).abs() + (a.y - b.y).abs(),
                1,
                "the road walks cardinals only: {a:?} to {b:?}"
            );
        }
        // It runs between the two endpoints, exclusive of both.
        let s = sim.road_end(lib, lib_ty);
        assert!(!tiles.contains(&s) && !tiles.contains(&Pos::new(40, 40)));
    }

    /// §8.1: `param_6 < 0` gates the odd wheel indices out, and a caravan
    /// does not have it.
    #[test]
    fn a_caravan_expands_the_diagonals_and_a_building_does_not() {
        let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let lib = library_at(&mut sim, lib_ty, Pos::new(33, 33));
        let dirs = |sim: &mut Sim, caravan: bool| -> Vec<i32> {
            sim.trace_costs = true;
            sim.road_marks.clear();
            let mut st = sim.start_road(lib, city).expect("two endpoints");
            let _ = sim.step_road(&mut st, (lib, city), caravan, false);
            let mut d: Vec<i32> = sim.road_marks.iter().map(|m| m.dir).collect();
            d.sort_unstable();
            d.dedup();
            d
        };
        let building = dirs(&mut sim, false);
        assert!(
            building.iter().all(|d| d % 2 == 0),
            "a building's road is cardinal-only: {building:?}"
        );
        let caravan = dirs(&mut sim, true);
        assert!(
            caravan.iter().any(|d| d % 2 != 0),
            "a caravan's takes the diagonals too: {caravan:?}"
        );
    }

    /// §8.3: the budget parks the search rather than throwing it away, and
    /// the frame after carries on from it. Made to fail by returning
    /// [`RoadPlan::None`] instead of `Budget`.
    #[test]
    fn a_caravan_s_search_stops_at_the_budget_and_the_next_frame_resumes_it() {
        // A world wide enough that 3,200 nodes do not reach the goal.
        let mut w = World::new(60, 60);
        w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
        let mut sim = Sim::new(Tuning::RON, w, 2);
        for l in &mut sim.ledgers {
            l.bucket = [10_000; crate::economy::RESOURCES];
        }
        let ty = sim.add_build_type(BuildType {
            ident: Ident::Library,
            flags: flags::parse("a"),
            x_size: 1,
            y_size: 1,
            job_time: 1,
            hits: 100,
            ..BuildType::default()
        });
        let a = sim.init_build(0, ty, centre_of(Pos::new(8, 8)), false);
        let b = sim.init_build(0, ty, centre_of(Pos::new(200, 200)), false);
        sim.trace_costs = true;
        let mut st = sim.start_road(a, b).expect("two endpoints");
        let plan = sim.step_road(&mut st, (a, b), true, false);
        assert_eq!(plan, RoadPlan::Budget, "3,200 nodes do not span the map");
        let first = sim.road_marks.len();
        assert!(
            (WORK_CAP as usize..=WORK_CAP as usize + 8).contains(&first),
            "the budget trips one pop past 0xc80: {first}"
        );
        let seen: std::collections::BTreeSet<(i32, i32)> =
            sim.road_marks.iter().map(|m| m.from).collect();
        sim.road_marks.clear();
        let plan = sim.step_road(&mut st, (a, b), true, false);
        assert_eq!(plan, RoadPlan::Budget, "and the second frame is another");
        let then: std::collections::BTreeSet<(i32, i32)> =
            sim.road_marks.iter().map(|m| m.from).collect();
        assert!(
            sim.road_marks.len() >= WORK_CAP as usize,
            "the resumed frame gets the whole budget again, not the rest of one"
        );
        assert!(
            then.intersection(&seen).count() < then.len(),
            "and it expands nodes the first frame did not"
        );
    }

    /// **Between two regions, only the step off land pays the sea**
    /// (`calc_road_cost@00686300`, `docs/CARAVAN.md` §11, item 1115). With
    /// `pathfinder +0x8c` clear, `+0xa8` is added when the node's parent
    /// stands on land; a sea tile reached from the sea costs the base and
    /// the jitter. With it set (both ends in one region), every sea tile
    /// pays `+0xa4`. run413's packet holds East Indies' AI route to its
    /// second island node for node with this, and 171 of 1,437 nodes
    /// apart without it.
    #[test]
    fn a_sea_crossing_between_regions_pays_once_at_the_shore() {
        let (mut sim, ..) = town(Pos::new(40, 40));
        for x in 50..60 {
            sim.world
                .set_tile_mask(Pos::new(x, 20), tile::SURFACE_OCEAN);
        }
        let sea = centre_of(Pos::new(52, 20));
        let (off_sea, off_land) = (centre_of(Pos::new(51, 20)), centre_of(Pos::new(49, 20)));
        let price = |from: Pos, avoid_sea: bool| {
            let mut s = sim.clone();
            s.calc_road_cost(sea, from, 0, (0, 0), 2, avoid_sea).0
        };
        assert_eq!(
            price(off_land, false) - price(off_sea, false),
            weight::SEA_ENTER,
            "the shore step pays"
        );
        let jitter = price(off_sea, false) - weight::BASE;
        assert!(
            (0..0x14).contains(&jitter),
            "a sea step is the base and the jitter"
        );
        assert_eq!(
            price(off_sea, true),
            price(off_land, true),
            "one region: every sea tile pays"
        );
        assert_eq!(
            price(off_sea, true) - price(off_sea, false),
            weight::SEA_AVOIDED
        );
    }

    #[test]
    fn the_search_refuses_ocean_and_answers_nothing_across_it() {
        let (mut sim, city, _, lib_ty) = town(Pos::new(40, 40));
        sim.plan_roads = true;
        let lib = library_at(&mut sim, lib_ty, Pos::new(33, 40));
        for y in 0..80 {
            sim.world
                .set_tile_mask(Pos::new(37, y), tile::SURFACE_OCEAN);
        }
        assert!(
            sim.find_road(lib, city).is_empty(),
            "an unbroken sea wall leaves no road"
        );
    }
}
