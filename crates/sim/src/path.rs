//! The pathfinder — `PathFinder::astar_path` and its three grid wrappers.
//! `docs/PATHFINDER.md`.
//!
//! One search at a time, as the original's single global has it. The search
//! is A\* in shape but not in temperament: the heuristic is 60 per grid step
//! (480 on the unit grid) against a base step cost of 32, so it is
//! deliberately greedy, and the open list breaks equal-f ties **LIFO** —
//! the newest node first — because the original's `Tree::ordered_insert`
//! sends equal keys left of their equals (`docs/PATHFINDER.md` §2.1). Both
//! are behaviour, not implementation detail: an admissible heuristic or an
//! oldest-first tie-break walks visibly different paths.
//!
//! What the world does not model yet enters as a named seam, each marked
//! `SEAM:` with the `docs/PATHFINDER.md` §5 term it stubs — fog (everything
//! is seen), the danger map (zero), diplomacy and rush-rule penalties
//! (none), terrain movement costs (zero — the harness's worlds are grass),
//! unit collision on the 48-grid (clear), transports (the unit cannot), and
//! the per-type collision size (one). Each returns the open-ground answer,
//! so on the flat worlds the harness builds the search is exact; the seams
//! are where the remaining layers plug in.

use std::collections::{BTreeMap, HashMap};

use crate::orders::{self, Body, MoveKind, PathData, Worker, path_flag};
use crate::world::{Owner, Pos, TILES_PER_CELL, Terrain, UNITS_PER_CELL, tile, vector_dist};
use crate::{Sim, movement};

/// The three grids' steps, in position units.
pub const STEP_WORLD: i32 = 0x300;
pub const STEP_TILE: i32 = 0xc0;
pub const STEP_UNIT: i32 = 0x30;

/// The compass tables (`move_x@00adcaf0` / `move_y@00adc400`, dumped from
/// the PE). Index 0 is unused; 1–8 are NW, N, NE, E, SE, S, SW, W — odd
/// diagonal, even cardinal.
const MOVE_X: [i32; 9] = [0, -1, 0, 1, 1, 1, 0, -1, -1];
const MOVE_Y: [i32; 9] = [0, -1, -1, -1, 0, 1, 1, 1, 0];

/// A step the cost function refuses.
const REFUSED: i32 = 0x7fff_ffff;

/// One search node — `PathNode`, minus the allocator.
#[derive(Clone, Copy, Debug)]
struct Node {
    x: i32,
    y: i32,
    /// g — accumulated cost.
    length: i32,
    /// h at this node — kept because the original keeps it (`PathNode`),
    /// read by nothing once `value` is computed.
    #[allow(dead_code)]
    estimate: i32,
    /// g + h — the open list's key.
    value: i32,
    /// Depth from the start, in steps.
    timeout: i32,
    /// The grid-cell index — the identity key for dedup.
    metric: i64,
    transport: bool,
    building: bool,
    parent: Option<u32>,
}

/// The per-search modes — the live fields of `PathFinderData`
/// (`docs/PATHFINDER.md` §2).
#[derive(Clone, Copy, Debug, Default)]
struct Modes {
    /// `anti_unit`: set for every unit-grid search; gates the node limit,
    /// the +5 probe cost on the tile grid, and flag 2 on waypoints.
    anti_unit: bool,
    /// Read by the unseen-cell and rough-terrain cost terms, all behind
    /// the fog and terrain-cost seams today.
    #[allow(dead_code)]
    army: bool,
    /// Read by the terrain-cost and corner-cutting terms, behind the same
    /// seams.
    #[allow(dead_code)]
    iroquois: bool,
    /// Read by the diplomacy term, behind its seam.
    #[allow(dead_code)]
    worker: bool,
    no_danger: bool,
    scouting: bool,
    /// `limit`: `500 / repaths²`, halved with `anti`; only read when
    /// `anti_unit`.
    limit: i32,
}

/// What `invalid_loc` answers (`docs/PATHFINDER.md` §6).
mod loc {
    pub const VALID: i32 = 0;
    pub const OFF_MAP: i32 = 1;
    pub const TERRAIN: i32 = 2;
    /// Shallows/river hazard for an armed ship — the code `valid_wcoord`
    /// forgives inside the goal cell.
    #[allow(dead_code)]
    pub const HAZARD: i32 = 3;
    pub const BUILDING: i32 = 4;
}

impl Sim {
    /// `UnitData::invalid_loc(t, ignore_buildings, fog_relax,
    /// enemy_builds_only, transport_a, transport_b)` — the world's refusal
    /// of a tile, for this unit. `t` is in tile coordinates.
    ///
    /// The domain is taken as land for every unit — the simulation has no
    /// ships or aircraft yet — and the cliff and per-cell hazard layers do
    /// not exist, so those refusals never fire. SEAM: fog_relax's
    /// flag-4-leader branch (stand in the unseen) is a no-op with no fog
    /// model; `transport_forced` (`unit_masks & 0x800000`) is never set, so
    /// water refuses every unit.
    #[allow(clippy::too_many_arguments)] // the original's five flags, kept by name
    pub(crate) fn invalid_loc(
        &self,
        u: usize,
        t: Pos,
        ignore_buildings: bool,
        _fog_relax: bool,
        enemy_builds_only: bool,
        transport_a: bool,
        transport_b: bool,
    ) -> i32 {
        // A transport-flagged path top relaxes the water test.
        let transport_a = transport_a
            || self.units[u]
                .path
                .last()
                .is_some_and(|p| p.flags & path_flag::TRANSPORT != 0);
        if !self.world.tile_in_bounds(t) {
            return loc::OFF_MAP;
        }
        let mask = self.world.tile_mask(t);
        let surface = mask & tile::SURFACE;
        let forest_walker = false; // SEAM: `unit_masks2 & 0x4000` (Iroquois).
        let transport_forced = false; // SEAM: `unit_masks & 0x800000`.
        // Land domain: forest, mountain, cliff, then water.
        if (surface == tile::SURFACE_FOREST && !forest_walker)
            || mask & tile::OBJECT == tile::OBJECT_MOUNTAIN
        {
            return loc::TERRAIN;
        }
        if surface == tile::SURFACE_OCEAN && !((transport_a || transport_b) && transport_forced) {
            return loc::TERRAIN;
        }
        // The building check: a blocked tile refuses, unless the unit is
        // itself standing on one (it may leave), or the caller asked to
        // ignore buildings.
        if !ignore_buildings
            && mask & tile::BLOCKED != 0
            && !(surface == tile::SURFACE_FOREST && forest_walker)
            && self.world.tile_mask(self.units[u].pos.tile()) & tile::BLOCKED == 0
        {
            // With `enemy_builds_only`, an armed unit passes its own side's
            // buildings. SEAM: building ownership at a tile is not indexed;
            // every blocked tile refuses. The original returns 0 here for
            // an armed unit over its own building.
            let _ = enemy_builds_only;
            return loc::BUILDING;
        }
        loc::VALID
    }

    /// `PathFinder::valid_wcoord` — the world grid's probe. `p` is a
    /// position; `timeout` the parent node's depth; `goal` the search goal.
    fn valid_wcoord(&self, u: usize, p: Pos, timeout: i32, goal: Pos) -> bool {
        if self.units[u].avoid == Some(p) {
            return false;
        }
        let r = self.invalid_loc(u, p.tile(), true, timeout > 1, false, true, false);
        if r == loc::HAZARD && p.cell() == goal.cell() {
            return true;
        }
        r == loc::VALID
    }

    /// `PathFinder::valid_tcoord` — the tile grid's probe (the pre-walk's
    /// form; the search inlines the same flags).
    fn valid_tcoord(&self, u: usize, p: Pos) -> bool {
        self.invalid_loc(u, p.tile(), false, true, true, true, false) == loc::VALID
    }

    /// `PathFinder::valid_ucoord` — the 48-grid's probe, memoised per
    /// search in the original's `validlist`. SEAM: `detect_unit_collision`
    /// — the half that sees other units — reports clear; the 48-grid is
    /// where collision avoidance would live.
    fn valid_ucoord(&self, u: usize, p: Pos, metric: i64, memo: &mut HashMap<i64, bool>) -> bool {
        let w = &self.world;
        if p.x < 0
            || p.y < 0
            || p.x >= w.width() * TILES_PER_CELL * 0xc0
            || p.y >= w.height() * TILES_PER_CELL * 0xc0
        {
            return false;
        }
        if let Some(&v) = memo.get(&metric) {
            return v;
        }
        let v = self.invalid_loc(u, p.tile(), false, true, false, true, false) == loc::VALID;
        memo.insert(metric, v);
        v
    }

    /// `UnitData::needs_transport`: 0 = no shoreline crossed, 1 = water to
    /// land, 2 = land to water. Tile coordinates.
    fn needs_transport(&self, from: Pos, to: Pos) -> i32 {
        if from == to {
            return 0;
        }
        let water = |t: Pos| self.world.tile_mask(t) & tile::SURFACE == tile::SURFACE_OCEAN;
        let (a, b) = (water(from), water(to));
        if a == b {
            0
        } else if a {
            1
        } else {
            2
        }
    }

    /// Whether a cell is ocean — `WorldData::is_ocean`, through the region
    /// layer.
    fn is_ocean_cell(&self, c: crate::world::Cell) -> bool {
        self.world
            .region_of(c)
            .is_some_and(|r| self.world.terrain(r) == Terrain::Sea)
    }

    /// `PathFinder::calc_cost` (`docs/PATHFINDER.md` §5): the cost of one
    /// step, or [`REFUSED`]. Returns `(cost, embarks)`.
    #[allow(clippy::too_many_arguments)]
    fn calc_cost(
        &self,
        u: usize,
        m: &Modes,
        from: Pos,
        to: Pos,
        dir: usize,
        step: i32,
        depth: i32,
        avoid_land: i32,
        avoid_sea: i32,
    ) -> (i32, bool) {
        let who = self.units[u].owner;
        let mut base = 0x100;
        let mut extra = 0;
        let mut embarks = false;

        if step != STEP_UNIT {
            // SEAM: `was_really_seen` — no fog model, every cell is seen,
            // so the unseen-cell branch (base 0x124, scout base 8) never
            // runs. A scout's preference for the unexplored is therefore
            // absent until fog exists.
            if step == STEP_WORLD && m.scouting {
                base = 0x400;
            }
            let to_cell = to.cell();
            if step == STEP_TILE {
                let mask = self.world.tile_mask(to.tile());
                base = if mask & tile::BAD_PATH != 0 {
                    0x400
                } else {
                    0x100
                };
                // SEAM: the danger map (read here unless `no_danger`) is
                // zero.
                let mut e = 0;
                if mask & tile::OBJECT == tile::OBJECT_BUILDING && mask & tile::BLOCKED != 0 {
                    e += 4000; // a gate tile
                }
                match self.world.owner(to_cell) {
                    Owner::Player(p) if p == who => e -= 4,
                    Owner::Player(p) if self.at_war[who as usize][p as usize] => e += 4,
                    _ => {}
                }
                extra = e.max(0);
            } else {
                // SEAM: the danger map is zero.
                let mut e = 0;
                if self.is_ocean_cell(to_cell) {
                    if avoid_sea != 0 {
                        e += 200;
                    }
                } else {
                    match self.world.owner(to_cell) {
                        Owner::Player(p) if p == who => e -= 4,
                        Owner::Player(p) if self.at_war[who as usize][p as usize] => e += 4,
                        _ => {}
                    }
                    if avoid_land != 0 {
                        e += 200;
                    }
                }
                // SEAM: terrain movement cost — the cell byte the original
                // reads (`+0x11`, `+0x13` for iroquois) has no layer here;
                // grass is 0, so the `+20×`, `+100000`, army-rough and
                // army-forest terms are dormant, and with cost 0 and no
                // iroquois flag the corner-cutting probes (§5.1) are
                // faithfully skipped.
                let tcost = 0;
                e += tcost * 20;
                extra = e.max(0);
            }
            // Fleeing triples the additive part.
            if self
                .current_order(u)
                .is_some_and(|o| o.flags & orders::flag::FLEEING != 0)
            {
                extra *= 3;
            }
            // SEAM: the no-rush timer (+500) and the team-style diplomacy
            // wall (+5000) read game rules the simulation does not carry.
            if step == STEP_WORLD {
                // River cells triple the base (`cell flags & 0x100`); the
                // cell-flag layer does not exist. SEAM.
            }
        }

        // The transport tail. SEAM: no unit can transport, so the embark
        // penalties and refusals are dormant; the water itself was priced
        // above.
        let crossing = self.needs_transport(from.tile(), to.tile());
        let can_transport = false;
        if crossing > 0 && can_transport && depth >= 2 {
            if avoid_sea == 2 {
                return (REFUSED, false);
            }
            if crossing != 1 {
                extra += if avoid_sea == 0 && avoid_land == 0 {
                    500
                } else {
                    2000
                };
            }
            // The unit-grid ×4 sits outside the embark guard: a disembark
            // skips the 500/2000 and still takes the shift (audit V21).
            if step == STEP_UNIT {
                extra *= 4;
            }
            embarks = true;
        } else if depth == 1 {
            let own_tile = self.units[u].pos.tile();
            if self.needs_transport(own_tile, to.tile()) > 0 && can_transport {
                if avoid_sea == 2 {
                    return (REFUSED, false);
                }
                if crossing != 1 {
                    extra += if avoid_sea == 0 && avoid_land == 0 {
                        250
                    } else {
                        1000
                    };
                }
                embarks = true;
            }
        }

        (base * 32 / 256 + extra + (dir as i32 & 1) * 8, embarks)
    }

    /// The heuristic — `PathFinderData::get_estimate`.
    fn estimate(dx: i32, dy: i32, step: i32) -> i32 {
        let d = vector_dist(dx, dy);
        if step == STEP_UNIT {
            d * 10
        } else {
            d * 60 / step
        }
    }

    /// The grid-cell index of a position — the `metric`.
    fn metric_of(&self, p: Pos, step: i32) -> i64 {
        let w = i64::from(self.world.width());
        match step {
            STEP_WORLD => i64::from(p.cell().x) + i64::from(p.cell().y) * w,
            STEP_TILE => {
                let t = p.tile();
                i64::from(t.x) + i64::from(t.y) * w * i64::from(TILES_PER_CELL)
            }
            _ => {
                let g = Pos::new(p.x.div_euclid(0x30), p.y.div_euclid(0x30));
                i64::from(g.x) + i64::from(g.y) * w * 16
            }
        }
    }

    /// `PathFinder::astar_path` (`docs/PATHFINDER.md` §4). Returns 1 on a
    /// path pushed, 0 on failure, −1 on a suspended unit-grid search.
    ///
    /// SEAM: suspension stashes nothing — `find_upath_restore` has no
    /// caller until collision recovery exists, so a suspended search is
    /// simply lost. The condition and return are the original's.
    fn astar_path(&mut self, u: usize, m: &Modes, step: i32, anti: i32) -> i32 {
        let stack_len = self.units[u].path.len();
        if stack_len < 2 {
            // The wrappers always push start and goal; anything else is a
            // caller bug, answered the way the engine answers an empty
            // list.
            return 0;
        }
        let start_e = self.units[u].path.pop().expect("start entry");
        let goal_e = self.units[u].path.pop().expect("goal entry");
        let (start, goal) = (start_e.to, goal_e.to);
        // The arrival tolerance is the *final* goal's — the entry now on
        // top — not the search-goal entry's.
        let tol = self.units[u].path.last().map_or(0, |p| p.tolerance);

        let work_cap = if step == STEP_UNIT { 500 } else { 50 } * 64;
        // SEAM: the unit-grid stride is `(type collision + 1) / 2`; no
        // collision size is loaded, so every unit searches at stride 1.
        let su: i32 = 1;
        let stride = su * step;
        let arrive = tol / 2 + stride;
        let width = i64::from(self.world.width());
        let row = match step {
            STEP_WORLD => width,
            STEP_TILE => width * i64::from(TILES_PER_CELL),
            _ => width * 16,
        };
        let dinc: i32 = if step == STEP_UNIT && anti != 0 { 2 } else { 1 };

        // avoid_land / avoid_sea from the start's terrain (§4.1).
        let same_region = self.world.tregion(start.tile()) == self.world.tregion(goal.tile());
        let (mut avoid_land, mut avoid_sea) = (0, 0);
        if same_region {
            let on_water = if step == STEP_WORLD {
                self.is_ocean_cell(start.cell())
            } else {
                self.world.tile_mask(start.tile()) & tile::SURFACE == tile::SURFACE_OCEAN
            };
            // SEAM: the amphibious exception (`unit_flags & 0x10` with
            // `unit_masks & 0x40000`) never fires.
            if on_water {
                avoid_land = 1;
            } else {
                avoid_sea = 1;
                if self
                    .action_of(u)
                    .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Attack(_)))
                {
                    avoid_sea = 2;
                }
            }
        } else if self.current_order(u).is_some_and(|o| o.flags & 0x20 != 0) {
            avoid_land = 1;
        }
        let no_danger = m.no_danger;
        let m = Modes { no_danger, ..*m };

        // The arena and the three keyed views.
        let mut nodes: Vec<Node> = Vec::new();
        // Min `value` first; equal values newest-first (LIFO), as the
        // original's BST leans (§2.1).
        let mut open: BTreeMap<(i32, std::cmp::Reverse<u64>), u32> = BTreeMap::new();
        let mut open_by_metric: HashMap<i64, (u64, i32, u32)> = HashMap::new();
        let mut closed: HashMap<i64, u32> = HashMap::new();
        let mut valid_memo: HashMap<i64, bool> = HashMap::new();
        let mut seq: u64 = 0;

        let root = Node {
            x: start.x,
            y: start.y,
            length: 0,
            estimate: Self::estimate(start.x - goal.x, start.y - goal.y, step),
            value: Self::estimate(start.x - goal.x, start.y - goal.y, step),
            timeout: 0,
            metric: self.metric_of(start, step),
            transport: false,
            building: false,
            parent: None,
        };
        nodes.push(root);
        open.insert((root.value, std::cmp::Reverse(seq)), 0);
        open_by_metric.insert(root.metric, (seq, root.value, 0));
        seq += 1;

        // The direction preference: the wheel starts one past this, at the
        // cardinal facing the goal.
        let (dx0, dy0) = (start.x - goal.x, start.y - goal.y);
        let pref: i32 = if dy0.abs() < dx0.abs() {
            if goal.x < start.x { 7 } else { 3 }
        } else if start.y <= goal.y {
            5
        } else {
            1
        };

        let mut probes: i32 = 0;
        let traversed: i32 = 0; // restored on resume; always 0 here.

        while let Some((&key, &cur_id)) = open.first_key_value() {
            open.remove(&key);
            let cur = nodes[cur_id as usize];
            // Tombstone the refs entry the way `first_open_node` does.
            if open_by_metric
                .get(&cur.metric)
                .is_some_and(|&(s, _, _)| s == key.1.0)
            {
                open_by_metric.remove(&cur.metric);
            }

            let manh = (cur.x - goal.x).abs() + (cur.y - goal.y).abs();
            let over_limit = m.anti_unit && m.limit < probes;
            if manh <= arrive || traversed + probes >= work_cap || over_limit {
                // Suspend (§4.3 step 3): a unit-grid search over its limit,
                // still short of the goal, without `anti`.
                if over_limit && manh > arrive && anti == 0 {
                    return -1;
                }
                let mut end_id = cur_id;
                let mut partial = false;
                if traversed + probes >= work_cap && anti == 0 {
                    match step {
                        STEP_TILE if m.anti_unit => return 0,
                        STEP_UNIT => {
                            // SEAM: the pause roll happens only when the
                            // order's target is a unit; move orders here
                            // never target one, so no draw. The `+0xb2 +=
                            // 30` retry cooldown is not modelled.
                            return 0;
                        }
                        STEP_WORLD => {
                            // Drain the open list for the node nearest the
                            // goal; the partial path is the answer.
                            let mut best = vector_dist(cur.x - goal.x, cur.y - goal.y);
                            while let Some((&k2, &n2)) = open.first_key_value() {
                                open.remove(&k2);
                                let cand = nodes[n2 as usize];
                                let d = vector_dist(cand.x - goal.x, cand.y - goal.y);
                                if d < best {
                                    best = d;
                                    end_id = n2;
                                }
                            }
                            partial = true;
                            // SEAM: the can-transport goal re-push
                            // (flags |= 4) is dormant.
                        }
                        _ => {}
                    }
                }
                return self.reconstruct(u, &m, &mut nodes, end_id, step, partial);
            }

            // Expansion: the wheel from `pref + 1`.
            let mut k = pref + 1;
            let mut c = 1;
            while c < 9 {
                let d = if k < 9 { k as usize } else { (k - 8) as usize };
                let nx = cur.x + MOVE_X[d] * stride;
                let ny = cur.y + MOVE_Y[d] * stride;
                let metric = cur.metric + i64::from(MOVE_X[d]) + i64::from(MOVE_Y[d]) * row;
                let p = Pos::new(nx, ny);

                let valid = match step {
                    STEP_WORLD => {
                        // The first two steps probe the cell corner (the
                        // target-unit offsets are zero here); later steps
                        // the centre.
                        let probe = if cur.timeout < 2 {
                            Pos::new(nx - 0x180, ny - 0x180)
                        } else {
                            p
                        };
                        self.valid_wcoord(u, probe, cur.timeout, goal)
                    }
                    STEP_TILE => {
                        self.invalid_loc(u, p.tile(), false, true, true, true, false) == loc::VALID
                    }
                    _ => {
                        // Big units re-check every sub-step on diagonals;
                        // with stride 1 the single probe is the whole
                        // check.
                        self.valid_ucoord(u, p, metric, &mut valid_memo)
                    }
                };
                if !valid {
                    k += dinc;
                    c += dinc;
                    continue;
                }
                probes += if m.anti_unit && step == STEP_TILE {
                    5
                } else {
                    1
                };

                let (mut cost, embarks) = self.calc_cost(
                    u,
                    &m,
                    Pos::new(cur.x, cur.y),
                    p,
                    d,
                    step,
                    cur.timeout + 1,
                    avoid_land,
                    avoid_sea,
                );
                if cost == REFUSED {
                    k += dinc;
                    c += dinc;
                    continue;
                }
                if nx == goal.x && ny == goal.y {
                    cost /= 2;
                }
                let g = cur.length + cost;
                if closed.contains_key(&metric) {
                    k += dinc;
                    c += dinc;
                    continue;
                }
                if let Some(&(old_seq, old_value, old_id)) = open_by_metric.get(&metric) {
                    if nodes[old_id as usize].length <= g {
                        k += dinc;
                        c += dinc;
                        continue;
                    }
                    open.remove(&(old_value, std::cmp::Reverse(old_seq)));
                    open_by_metric.remove(&metric);
                }
                let h = Self::estimate(nx - goal.x, ny - goal.y, step);
                let timeout = cur.timeout + 1;
                // The tile grid's depth-plus-distance cap.
                if step == STEP_TILE && h / 32 + timeout > 120 {
                    k += dinc;
                    c += dinc;
                    continue;
                }
                let node = Node {
                    x: nx,
                    y: ny,
                    length: g,
                    estimate: h,
                    value: g + h,
                    timeout,
                    metric,
                    transport: embarks,
                    building: false,
                    parent: Some(cur_id),
                };
                let id = nodes.len() as u32;
                nodes.push(node);
                open.insert((node.value, std::cmp::Reverse(seq)), id);
                open_by_metric.insert(metric, (seq, node.value, id));
                seq += 1;

                k += dinc;
                c += dinc;
            }
            closed.insert(cur.metric, cur_id);
        }

        // Open list exhausted: no path. SEAM: the unit-grid pause roll
        // (target-is-a-unit only) and the +30 cooldown are dormant.
        0
    }

    /// The walk back up the parent chain (`docs/PATHFINDER.md` §7).
    fn reconstruct(
        &mut self,
        u: usize,
        m: &Modes,
        nodes: &mut [Node],
        end_id: u32,
        step: i32,
        partial: bool,
    ) -> i32 {
        let end = nodes[end_id as usize];
        if end.parent.is_none() {
            return 1;
        }
        // The root: the arrival node, except a plain world-grid arrival
        // drops it — the goal itself is already on the stack below.
        let mut root_id = if step == STEP_WORLD && !partial && !end.transport {
            end.parent.expect("checked above")
        } else {
            end_id
        };
        // Tile grid: mark gate tiles (a passable building tile) with the
        // node's `building` flag and root the walk at the deepest one.
        if step == STEP_TILE {
            let mut id = Some(end_id);
            while let Some(i) = id {
                let n = nodes[i as usize];
                let mask = self.world.tile_mask(Pos::new(n.x, n.y).tile());
                if mask & tile::OBJECT == tile::OBJECT_BUILDING && mask & tile::BLOCKED != 0 {
                    nodes[i as usize].building = true;
                    root_id = i;
                }
                id = n.parent;
                if id.and_then(|p| nodes[p as usize].parent).is_none() {
                    break;
                }
            }
        }

        // SEAM: the target-is-a-unit offsets (`toff`) are zero — move
        // orders here have point goals.
        let mut id = Some(root_id);
        while let Some(i) = id {
            let n = nodes[i as usize];
            if n.parent.is_none() && step != STEP_UNIT {
                break;
            }
            let mut tolerance = match step {
                STEP_WORLD => 0x180,
                STEP_UNIT => 0,
                // `0` for a transporter outside anti mode; SEAM: no
                // transporters, so always 0x60.
                _ => 0x60,
            };
            let mut flags: u8 = if m.anti_unit || step == STEP_UNIT {
                path_flag::SIDESTEP
            } else {
                0
            };
            if n.building {
                flags |= 0x10;
            }
            if n.transport {
                flags |= path_flag::TRANSPORT;
                tolerance = 0;
            }
            let entry = PathData {
                to: Pos::new(n.x, n.y),
                tolerance,
                flags,
            };
            if self.units[u].path.last().map(|p| p.to) != Some(entry.to) {
                self.units[u].path.push(entry);
            }
            id = n.parent;
        }
        1
    }

    /// `PathFinder::find_wpath` — the world-cell planner
    /// (`docs/PATHFINDER.md` §3). Pops the goal, pre-walks it, runs the
    /// search, returns the stack length (0 no path, −1 off the map).
    pub(crate) fn find_wpath(&mut self, u: usize) -> i32 {
        let Some(goal_e) = self.units[u].path.pop() else {
            return 0;
        };
        let here = self.units[u].pos;
        let gc = goal_e.to.cell();
        if !self.world.contains(gc) {
            self.units[u].path.clear();
            return -1;
        }
        // Same cell, or a flyer (SEAM: no flyers): push back, done.
        if here.cell() == gc {
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        // The pull-back walk: step the goal toward the start until its
        // tile region matches the start's. On one-region maps this exits
        // immediately.
        let mut goal_e = goal_e;
        let mut goal = goal_e.to;
        loop {
            if self.world.tregion(goal.tile()) == self.world.tregion(here.tile()) {
                break;
            }
            let (dx, dy) = (here.x - goal.x, here.y - goal.y);
            let far = dx.abs() + dy.abs() >= 0x300;
            let mut s = if far { 0x180 } else { 0x30 };
            // The give-up exit: a remainder smaller than the step on both
            // axes takes the goal where it stands **without running A\***
            // (audit V17) — the same push-and-return as reaching the
            // start's cell. Only a region match continues to the search.
            if dx.abs() < s && dy.abs() < s {
                self.units[u].path.push(goal_e);
                return self.units[u].path.len() as i32;
            }
            let ang = movement::find_angle(dx, dy);
            if ang.0 < 0 {
                s = -s;
            }
            let sx = movement::sin_component(ang, s);
            let cy = movement::cos_component(ang, s);
            goal = Pos::new(goal.x + sx, goal.y - cy);
            goal_e.to = goal;
            if goal.cell() == here.cell() {
                self.units[u].path.push(goal_e);
                return self.units[u].path.len() as i32;
            }
        }
        self.units[u].path.push(goal_e);
        // The near test and the centre push use the (possibly pulled-back)
        // goal's cell.
        let gc = goal.cell();
        let hc = here.cell();
        if (gc.x - hc.x).abs() + (gc.y - hc.y).abs() < 3 {
            return self.units[u].path.len() as i32;
        }

        // `army`/`worker` are **AI-only**: `leaders.flags & 4` is
        // `is_human`, and a human's `find_wpath` jumps straight to the
        // search past the whole mode block (audit V14).
        let human = self.nation[self.units[u].owner as usize].human;
        let modes = Modes {
            scouting: self.current_order(u).is_some_and(
                |o| matches!(o.body, Body::Move(mo) if mo.kind == MoveKind::ExploreTo),
            ),
            army: !human && self.army_mode(u),
            worker: !human
                && self.units[u]
                    .ty
                    .is_some_and(|t| self.unit_types[t].worker != Worker::None),
            no_danger: self.no_danger_mode(u),
            iroquois: false, // SEAM: `unit_masks2 & 0x4000`.
            ..Modes::default()
        };
        let centre = |c: crate::world::Cell| {
            Pos::new(c.x * UNITS_PER_CELL + 0x180, c.y * UNITS_PER_CELL + 0x180)
        };
        self.units[u].path.push(PathData {
            to: centre(gc),
            tolerance: 0x180,
            flags: 0,
        });
        self.units[u].path.push(PathData {
            to: centre(here.cell()),
            tolerance: 0,
            flags: 0,
        });
        let r = self.astar_path(u, &modes, STEP_WORLD, 0);
        if r == 0 {
            let popped = self.units[u].path.pop();
            return -i32::from(popped.is_some_and(|p| p.flags & path_flag::FINAL != 0));
        }
        self.units[u].path.len() as i32
    }

    /// `PathFinder::find_tpath` — the tile-grid planner.
    pub(crate) fn find_tpath(&mut self, u: usize) -> i32 {
        let Some(mut goal_e) = self.units[u].path.pop() else {
            return 0;
        };
        let here = self.units[u].pos;
        let gt = goal_e.to.tile();
        if !self.world.tile_in_bounds(gt) {
            self.units[u].path.clear();
            return -1;
        }
        let ht = here.tile();
        if ht == gt {
            goal_e.tolerance = 0;
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        // The pull-back walk on tiles.
        let mut goal = goal_e.to;
        loop {
            if self.world.tregion(goal.tile()) == self.world.tregion(here.tile())
                && self.valid_tcoord(u, goal)
            {
                break;
            }
            let (dx, dy) = (here.x - goal.x, here.y - goal.y);
            if dx.abs() < 0x60 && dy.abs() < 0x60 {
                if goal_e.flags & path_flag::FINAL == 0 {
                    return 0;
                }
                break;
            }
            let ang = movement::find_angle(dx, dy);
            let s = if ang.0 < 0 { -0x30 } else { 0x30 };
            let sx = movement::sin_component(ang, s);
            let cy = movement::cos_component(ang, s);
            goal = Pos::new(goal.x + sx, goal.y - cy);
            goal_e.to = goal;
            if goal.tile() == ht {
                self.units[u].path.push(goal_e);
                return self.units[u].path.len() as i32;
            }
            if sx == 0 && cy == 0 {
                break;
            }
        }
        let gt = goal.tile();
        let td = (ht.x - gt.x).abs() + (ht.y - gt.y).abs();
        if td < 2 {
            goal_e.tolerance = 0;
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        let kept = goal_e.tolerance;
        self.units[u].path.push(goal_e);
        let centre_t = |t: Pos| Pos::new(t.x * 0xc0 + 0x60, t.y * 0xc0 + 0x60);
        self.units[u].path.push(PathData {
            to: centre_t(gt),
            tolerance: if kept == 0x180 { 0x180 } else { 0x60 },
            flags: 0,
        });
        self.units[u].path.push(PathData {
            to: centre_t(ht),
            tolerance: 0,
            flags: 0,
        });
        let modes = Modes {
            no_danger: self.no_danger_mode(u),
            ..Modes::default()
        };
        let r = self.astar_path(u, &modes, STEP_TILE, 0);
        if r < 1 {
            if self.units[u]
                .path
                .last()
                .is_some_and(|p| p.flags & path_flag::FINAL == 0)
            {
                self.units[u].path.pop();
            }
            return 0;
        }
        self.units[u].path.len() as i32
    }

    /// `PathFinder::find_upath` — the 48-grid planner, collision
    /// recovery's. `anti` halves the node limit and expands cardinals
    /// only. Public because its caller, `resolve_unit_collision`, is not
    /// modelled yet (`docs/ORDERS.md` §4.7); the mechanic is complete and
    /// tested, the wiring arrives with collision.
    pub fn find_upath(&mut self, u: usize, anti: bool) -> i32 {
        // SEAM: `repaths[who]` is 0 with no collision pressure model, so
        // the limit is the full 500 (250 with `anti`).
        let limit = if anti { 250 } else { 500 };
        let Some(mut goal_e) = self.units[u].path.pop() else {
            return 0;
        };
        let here = self.units[u].pos;
        let g48 = |p: Pos| Pos::new(p.x.div_euclid(0x30), p.y.div_euclid(0x30));
        let gg0 = g48(goal_e.to);
        if gg0.x < 0
            || gg0.y < 0
            || gg0.x >= self.world.width() * 16
            || gg0.y >= self.world.height() * 16
        {
            self.units[u].path.clear();
            return -1;
        }
        let hg = g48(here);
        if hg == g48(goal_e.to) {
            goal_e.tolerance = 0;
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        let mut goal = goal_e.to;
        let mut memo = HashMap::new();
        loop {
            let gg = g48(goal);
            let metric = i64::from(gg.x) + i64::from(gg.y) * i64::from(self.world.width()) * 16;
            if self.world.tregion(goal.tile()) == self.world.tregion(here.tile())
                && self.valid_ucoord(u, goal, metric, &mut memo)
            {
                break;
            }
            let (dx, dy) = (here.x - goal.x, here.y - goal.y);
            if dx.abs() < 0x18 && dy.abs() < 0x18 {
                if goal_e.flags & path_flag::FINAL == 0 {
                    return 0;
                }
                break;
            }
            let ang = movement::find_angle(dx, dy);
            let s = if ang.0 < 0 { -0x30 } else { 0x30 };
            let sx = movement::sin_component(ang, s);
            let cy = movement::cos_component(ang, s);
            goal = Pos::new(goal.x + sx, goal.y - cy);
            goal_e.to = goal;
            if g48(goal) == hg {
                self.units[u].path.push(goal_e);
                return self.units[u].path.len() as i32;
            }
            if sx == 0 && cy == 0 {
                break;
            }
        }
        let gg = g48(goal);
        let md = (hg.x - gg.x).abs() + (hg.y - gg.y).abs();
        if md < 2 {
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        self.units[u].path.push(goal_e);
        let centre48 = |g: Pos| Pos::new(g.x * 0x30 + 0x18, g.y * 0x30 + 0x18);
        self.units[u].path.push(PathData {
            to: centre48(gg),
            tolerance: 0,
            flags: 0,
        });
        self.units[u].path.push(PathData {
            to: centre48(hg),
            tolerance: 0,
            flags: 0,
        });
        let modes = Modes {
            anti_unit: true,
            limit,
            no_danger: self.no_danger_mode(u),
            ..Modes::default()
        };
        let r = self.astar_path(u, &modes, STEP_UNIT, i32::from(anti));
        if r < 1 {
            if r == 0 {
                if self.units[u]
                    .path
                    .last()
                    .is_some_and(|p| p.flags & path_flag::FINAL == 0)
                {
                    self.units[u].path.pop();
                }
                // Kill the order unless a pause was rolled (SEAM: no unit
                // targets, so no pause — a failed unit-grid plan kills).
                self.kill_current_order(u);
            }
            return r.min(0);
        }
        // Post-processing: drop a top equal to the unit's position, then
        // compact collinear side-steps.
        if self.units[u].path.len() > 3 {
            self.compact_upath(u);
        }
        self.units[u].path.len() as i32
    }

    /// `find_upath`'s success post-processing: the top-equals-position
    /// drop and the `flags & 2` collinear compaction.
    fn compact_upath(&mut self, u: usize) {
        let here = self.units[u].pos;
        let mut first = self.units[u].path.pop().expect("upath top");
        if first.to == here && first.flags & path_flag::FINAL == 0 {
            first = self.units[u].path.pop().expect("upath second");
        }
        let mut keep: Vec<PathData> = vec![first];
        if first.flags & path_flag::SIDESTEP != 0 && self.units[u].path.len() > 2 {
            // The reference point advances to the examined waypoint on a
            // keep and on a collinear drop, but not on a one-step-jump
            // drop — the decompile's comma-expression side effects,
            // followed exactly.
            let (mut ax, mut ay) = (first.to.x, first.to.y);
            let mut mid = self.units[u].path.pop().expect("upath mid");
            while let Some(&next) = self.units[u].path.last() {
                if mid.flags & path_flag::SIDESTEP == 0
                    || next.flags & path_flag::SIDESTEP == 0
                    || self.units[u].path.len() <= 1
                {
                    break;
                }
                let collinear =
                    ax - mid.to.x == mid.to.x - next.to.x && ay - mid.to.y == mid.to.y - next.to.y;
                let one_step = (ax - next.to.x).abs() == 0x30 && (ay - next.to.y).abs() == 0x30;
                if mid.flags & path_flag::TRANSPORT != 0 || (!collinear && !one_step) {
                    keep.push(mid);
                    ax = mid.to.x;
                    ay = mid.to.y;
                } else if collinear {
                    ax = mid.to.x;
                    ay = mid.to.y;
                }
                mid = self.units[u].path.pop().expect("upath chain");
            }
            self.units[u].path.push(mid);
        }
        while let Some(p) = keep.pop() {
            self.units[u].path.push(p);
        }
    }

    /// The `army` mode: a military unit, not a worker, not attacking, and
    /// (SEAM: the start cell's river flag has no layer) not on a river.
    fn army_mode(&self, u: usize) -> bool {
        let armed = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].combat.attack > 0);
        let worker = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].worker != Worker::None);
        let attacking = self.units[u].combat.target.is_some();
        armed && !worker && !attacking
    }

    /// The `no_danger` mode: the danger map is ignored when attacking.
    fn no_danger_mode(&self, u: usize) -> bool {
        self.action_of(u)
            .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Attack(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Cell;
    use crate::{Tuning, Unit, World};

    fn flat_sim(cells: i32) -> Sim {
        let mut world = World::new(cells, cells);
        world.fill_region(
            Terrain::Land,
            Cell::new(0, 0),
            Cell::new(cells - 1, cells - 1),
        );
        Sim::new(Tuning::RON, world, 2)
    }

    fn walker(sim: &mut Sim, pos: Pos) -> usize {
        let mut u = Unit::new(0, 1, pos, 10);
        u.movement.speed = 25;
        let i = sim.units.len();
        sim.units.push(u);
        i
    }

    fn push_goal(sim: &mut Sim, u: usize, to: Pos) {
        sim.units[u].path.push(PathData {
            to,
            tolerance: 0,
            flags: path_flag::FINAL,
        });
    }

    #[test]
    fn a_near_goal_leaves_only_the_goal_on_the_stack() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        // Two cells away: cell-Manhattan 2 < 3 — no A*.
        push_goal(&mut sim, u, Pos::new(0x180 + 2 * 0x300, 0x180));
        let r = sim.find_wpath(u);
        assert_eq!(r, 1);
        assert_eq!(sim.units[u].path.len(), 1);
        assert_eq!(sim.units[u].path[0].to, Pos::new(0x180 + 2 * 0x300, 0x180));
    }

    #[test]
    fn a_far_goal_gets_a_chain_of_cell_centres() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        // Six cells east.
        let goal = Pos::new(0x180 + 6 * 0x300, 0x180);
        push_goal(&mut sim, u, goal);
        let r = sim.find_wpath(u);
        assert!(r > 1, "expected a planned chain, got {r}");
        let path = &sim.units[u].path;
        // Bottom is the final goal; above it the goal-cell centre with
        // tolerance 0x180; the waypoints walk from the start side (top)
        // toward the goal.
        assert_eq!(path[0].to, goal);
        assert_eq!(path[0].flags & path_flag::FINAL, path_flag::FINAL);
        assert_eq!(path[1].tolerance, 0x180);
        // Every intermediate entry is a cell centre.
        for p in &path[1..] {
            assert_eq!((p.to.x - 0x180) % 0x300, 0, "{:?}", p.to);
            assert_eq!((p.to.y - 0x180) % 0x300, 0, "{:?}", p.to);
        }
        // The top entry is the first step away from the start cell, one
        // cell along the line — the walk is straight on open ground.
        let top = path.last().unwrap();
        assert_eq!(top.to, Pos::new(0x180 + 0x300, 0x180));
    }

    #[test]
    fn the_straight_chain_matches_the_axis_on_open_ground() {
        let mut sim = flat_sim(24);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let goal = Pos::new(0x180, 0x180 + 8 * 0x300);
        push_goal(&mut sim, u, goal);
        let r = sim.find_wpath(u);
        assert!(r > 1);
        for p in &sim.units[u].path[1..] {
            assert_eq!(p.to.x, 0x180, "the chain should hug the axis: {:?}", p.to);
        }
    }

    #[test]
    fn a_goal_off_the_map_returns_minus_one_and_empties_the_stack() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        push_goal(&mut sim, u, Pos::new(-0x300, 0x180));
        assert_eq!(sim.find_wpath(u), -1);
        assert!(sim.units[u].path.is_empty());
    }

    #[test]
    fn a_blocked_band_of_forest_is_routed_around_on_the_tile_grid() {
        let mut sim = flat_sim(12);
        // A forest wall across x-tile 8, y-tiles 0..20, with a gap at 21+.
        for ty in 0..20 {
            let t = Pos::new(8, ty);
            sim.world
                .set_tile_field(t, tile::SURFACE, tile::SURFACE_FOREST);
            sim.world.set_tile_bits(t, tile::BLOCKED);
        }
        let u = walker(&mut sim, Pos::new(4 * 0xc0 + 0x60, 4 * 0xc0 + 0x60));
        let goal = Pos::new(12 * 0xc0 + 0x60, 4 * 0xc0 + 0x60);
        push_goal(&mut sim, u, goal);
        let r = sim.find_tpath(u);
        assert!(r > 1, "expected a route, got {r}");
        // No waypoint sits on a forest tile.
        for p in &sim.units[u].path {
            let m = sim.world.tile_mask(p.to.tile());
            assert_ne!(
                m & tile::SURFACE,
                tile::SURFACE_FOREST,
                "waypoint in the trees: {:?}",
                p.to
            );
        }
    }

    #[test]
    fn the_unit_grid_plans_in_48_cells_and_compacts_the_line() {
        let mut sim = flat_sim(12);
        let u = walker(&mut sim, Pos::new(0x18, 0x18));
        // Ten 48-cells east: far enough for a chain, straight enough for
        // the collinear compaction to bite.
        let goal = Pos::new(0x18 + 10 * 0x30, 0x18);
        push_goal(&mut sim, u, goal);
        let r = sim.find_upath(u, false);
        assert!(r >= 1, "expected a unit-grid plan, got {r}");
        let path = &sim.units[u].path;
        assert_eq!(path[0].to, goal);
        // Every planned waypoint carries the side-step flag and zero
        // tolerance, and the collinear middle was compacted away.
        for p in &path[1..] {
            assert_eq!(p.flags & path_flag::SIDESTEP, path_flag::SIDESTEP);
            assert_eq!(p.tolerance, 0);
        }
        assert!(
            path.len() < 11,
            "collinear 48-centres should compact: {} entries",
            path.len()
        );
    }

    #[test]
    fn ties_are_broken_newest_first() {
        // Two equal-value keys: the later insertion must come out first.
        let mut open: BTreeMap<(i32, std::cmp::Reverse<u64>), u32> = BTreeMap::new();
        open.insert((100, std::cmp::Reverse(1)), 10);
        open.insert((100, std::cmp::Reverse(2)), 20);
        let (_, &first) = open.first_key_value().unwrap();
        assert_eq!(first, 20, "LIFO on equal f");
    }

    #[test]
    fn the_diagonal_costs_eight_more() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let m = Modes::default();
        let from = Pos::new(0x180 + 0x300, 0x180 + 0x300);
        let east = Pos::new(from.x + 0x300, from.y);
        let se = Pos::new(from.x + 0x300, from.y + 0x300);
        let (c_card, _) = sim.calc_cost(u, &m, from, east, 4, STEP_WORLD, 2, 0, 1);
        let (c_diag, _) = sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 2, 0, 1);
        assert_eq!(c_card, 32);
        assert_eq!(c_diag, 40);
    }

    #[test]
    fn enemy_territory_charges_and_the_own_discount_clamps_at_zero() {
        // The additive term is clamped at zero after the terrain add
        // (`docs/PATHFINDER.md` §5), so with no danger to offset, one's own
        // territory costs the same as unowned ground — the −4 only ever
        // cancels danger. Enemy territory's +4 survives the clamp.
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let m = Modes::default();
        let from = Pos::new(0x180, 0x180);
        let to = Pos::new(0x180 + 0x300, 0x180);
        sim.world
            .set_owner(to.cell(), Owner::Player(0), Owner::None);
        let (own, _) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1);
        assert_eq!(own, 32);
        sim.world
            .set_owner(to.cell(), Owner::Player(1), Owner::None);
        sim.at_war[0][1] = true;
        let (enemy, _) = sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1);
        assert_eq!(enemy, 36);
    }
}
