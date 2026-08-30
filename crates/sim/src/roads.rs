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
//! **The search is off by default** ([`crate::Sim::plan_roads`]): it expands
//! about six per cent fewer nodes than the original's, and a wrong count is
//! worse for the stream than no count at all. `docs/ROADS.md` §7 has the
//! measurement and the list of what the gap has been ruled out against.

use crate::ai_place::{MOVE_X, MOVE_Y};
use crate::build::{self, Ident, flags};
use crate::world::{Cell, Owner, Pos, UNITS_PER_TILE, cell, tile, vector_dist};
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
    /// SEAM: only the `REGEN_FORCE` call `Build::process` makes is modelled.
    /// `BuildType::mask_me`'s `REGEN_TOTAL` — which sets the flag rather than
    /// searching — and the `set == 0` teardown that *removes* a footprint's
    /// roads are both unmodelled; nothing in a traced game reaches them, and
    /// `City::regen_roads` is what actually flags a building here.
    pub fn place_roads(&mut self, b: usize) {
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        let (w, h) = (self.build_types[ty].x_size, self.build_types[ty].y_size);
        let corner = self.tile_corner(ty, self.buildings[b].pos);

        if build::is_city(&self.build_types, ty) {
            // A city lays its own ring and stops: it *is* the destination.
            self.road_ring(corner, w, h, 0);
            return;
        }
        if connects_to_roads(&self.build_types, ty) {
            self.road_ring(corner, w, h, 0);
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
            self.road_ring(corner, w, h, 1);
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
        // The original walks the stack down from the top, which is the
        // near-goal end; the order does not reach the world. A blocked tile
        // is on the path but is not laid.
        for p in road {
            let t = p.tile();
            if self.world.tile_mask(t) & tile::BLOCKED == 0 {
                self.set_road_at(t);
            }
        }
    }

    /// The footprint's ring. `pad` is the one asymmetry between
    /// `place_roads`' three ring loops: the city arm and the
    /// `connects_to_roads` arm walk `corner−1 ..= corner−1+size`, the third
    /// one column and row further.
    fn road_ring(&mut self, corner: Pos, w: i32, h: i32, pad: i32) {
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
                self.set_road_at(t);
            }
        }
    }

    /// `World::set_road_at@006b43b0` with `set != 0`: the tile's surface
    /// becomes road and the cell is marked as carrying one.
    pub(crate) fn set_road_at(&mut self, t: Pos) {
        let m = self.world.tile_mask(t);
        self.world
            .set_tile_mask(t, (m & !tile::SURFACE_OCEAN) | tile::SURFACE_ROAD);
        let c = Cell::new(t.x.div_euclid(4), t.y.div_euclid(4));
        let mut d = self.world.cell_data(c);
        d.flags |= cell::ROAD;
        self.world.set_cell_data(c, d);
    }

    // ------------------------------------------------------------------
    // §4 — `PathFinder::find_road@00688a40`
    // ------------------------------------------------------------------

    /// The endpoints, and the search between them. Returns the road's tiles
    /// as positions, empty when there is no road to lay.
    pub fn find_road(&mut self, from: usize, to: usize) -> Vec<Pos> {
        let (Some(fty), Some(tty)) = (self.buildings[from].ty, self.buildings[to].ty) else {
            return Vec::new();
        };
        let start = self.road_end(from, fty);
        let goal = self.road_end(to, tty);
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        let inside = |p: Pos| p.x >= 0 && p.y >= 0 && p.x < tw && p.y < th;
        if !inside(start) || !inside(goal) || start == goal {
            return Vec::new();
        }
        // `avoid_sea` here is "both ends are in the same region", which the
        // cost function reads only on an ocean tile — and `can_transport` is
        // zero for a building's road, so no ocean tile is ever valid.
        let avoid_sea = self.world.tregion(start).is_some()
            && self.world.tregion(start) == self.world.tregion(goal);
        let who = (self.buildings[from].owner, self.buildings[to].owner);
        self.astar_road(
            centre_of(start),
            centre_of(goal),
            who,
            (from, to),
            avoid_sea,
        )
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

    /// The search. Positions are in world units, the stride is one tile, and
    /// with no caravan the wheel turns **cardinals only** — the `param_6 < 0`
    /// arm, which is the only one a building takes.
    fn astar_road(
        &mut self,
        start: Pos,
        goal: Pos,
        who: (Player, Player),
        ends: (usize, usize),
        avoid_sea: bool,
    ) -> Vec<Pos> {
        let row = i64::from(self.world.width()) * 4;
        let metric_of = |p: Pos| {
            i64::from(p.x.div_euclid(UNITS_PER_TILE))
                + i64::from(p.y.div_euclid(UNITS_PER_TILE)) * row
        };

        let mut nodes: Vec<Node> = Vec::new();
        let mut open: BTreeMap<(i32, Reverse<u64>), u32> = BTreeMap::new();
        let mut open_by_metric: BTreeMap<i64, (u64, i32, u32)> = BTreeMap::new();
        let mut closed: BTreeMap<i64, u32> = BTreeMap::new();
        let mut seq: u64 = 0;

        // The root's estimate is the shared `get_estimate` at the tile
        // stride — `d × 60 / 0xc0` — where every later node's is
        // `d × 60 / 0x180`. The inconsistency is real (both were read off
        // the listing's two divide-by-magic sequences) and harmless: the
        // root is the only node in the open list when it is popped.
        let root_h = vector_dist(start.x - goal.x, start.y - goal.y) * 60 / 0xc0;
        nodes.push(Node {
            x: start.x,
            y: start.y,
            length: 0,
            value: root_h,
            metric: metric_of(start),
            // Unclamped and never negated: a search that starts on a road
            // does not get the road bonus on its first step.
            z_val: self.world.tile_z(start.tile()),
            parent: None,
        });
        open.insert((root_h, Reverse(seq)), 0);
        open_by_metric.insert(nodes[0].metric, (seq, root_h, 0));
        seq += 1;

        // The wheel's preference — a diagonal index, so `pref + 1` is the
        // cardinal facing the goal (`docs/PATHFINDER.md` §4.2). Only the
        // cardinals are expanded, so the order is `pref+1, +3, +5, +7`.
        let (dx0, dy0) = (start.x - goal.x, start.y - goal.y);
        let pref: i32 = if dy0.abs() < dx0.abs() {
            if goal.x < start.x { 7 } else { 3 }
        } else if goal.y < start.y {
            1
        } else {
            5
        };

        let mut traversed: i32 = 0;
        while let Some((&key, &cur_id)) = open.first_key_value() {
            open.remove(&key);
            let cur = nodes[cur_id as usize];
            // `first_open_node` tombstones the refs entry for the popped
            // node's metric; removing it is the same thing.
            if open_by_metric
                .get(&cur.metric)
                .is_some_and(|&(s, _, _)| s == key.1.0)
            {
                open_by_metric.remove(&cur.metric);
            }
            if (cur.x - goal.x).abs() + (cur.y - goal.y).abs() < 1 {
                return reconstruct(&nodes, cur_id);
            }
            if traversed >= WORK_CAP {
                // The budget: the road is simply not laid.
                return Vec::new();
            }

            for k in (pref + 1..).take(8) {
                let d = if k < 9 { k } else { k - 8 };
                if d % 2 != 0 {
                    continue;
                }
                let d = usize::try_from(d).expect("wheel index");
                let nx = cur.x + MOVE_X[d] * UNITS_PER_TILE;
                let ny = cur.y + MOVE_Y[d] * UNITS_PER_TILE;
                let metric = cur.metric + i64::from(MOVE_X[d]) + i64::from(MOVE_Y[d]) * row;
                let p = Pos::new(nx, ny);
                if !self.valid_roadcoord(p, ends) {
                    continue;
                }
                traversed += 1;
                let (mut cost, z_val) = self.calc_road_cost(p, cur.z_val, who, d, avoid_sea);
                if nx == goal.x && ny == goal.y {
                    cost /= 2;
                }
                let g = cur.length + cost;
                if closed.contains_key(&metric) {
                    continue;
                }
                if let Some(&(old_seq, old_value, old_id)) = open_by_metric.get(&metric) {
                    if nodes[old_id as usize].length <= g {
                        continue;
                    }
                    open.remove(&(old_value, Reverse(old_seq)));
                    open_by_metric.remove(&metric);
                }
                let h = vector_dist(nx - goal.x, ny - goal.y) * 60 / 0x180;
                let node = Node {
                    x: nx,
                    y: ny,
                    length: g,
                    value: g + h,
                    metric,
                    z_val,
                    parent: Some(cur_id),
                };
                let id = u32::try_from(nodes.len()).expect("road nodes");
                nodes.push(node);
                open.insert((node.value, Reverse(seq)), id);
                open_by_metric.insert(metric, (seq, node.value, id));
                seq += 1;
            }
            closed.insert(cur.metric, cur_id);
        }
        Vec::new()
    }

    /// `PathFinderData::valid_roadcoord@00688740`, with `can_transport`
    /// zero — which it always is for a building's road, so ocean is refused
    /// outright. The diagonal corner recursion is dead here for the same
    /// reason the diagonals are: only cardinals are expanded.
    fn valid_roadcoord(&self, p: Pos, ends: (usize, usize)) -> bool {
        let (tw, th) = (self.world.width() * 4, self.world.height() * 4);
        if p.x < 0 || p.y < 0 || p.x >= tw * UNITS_PER_TILE || p.y >= th * UNITS_PER_TILE {
            return false;
        }
        let t = p.tile();
        let m = self.world.tile_mask(t);
        if m & tile::SURFACE == tile::SURFACE_OCEAN {
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
    /// its count of nodes costed.
    fn calc_road_cost(
        &mut self,
        p: Pos,
        parent_z: i32,
        who: (Player, Player),
        dir: usize,
        avoid_sea: bool,
    ) -> (i32, i32) {
        let t = p.tile();
        self.mark(SITE_COST);
        // `Random::get(0, 0xffff) % 0x14` — the jitter that makes two roads
        // between the same two buildings different roads. `removable` is the
        // original's second accumulator: the part a road step gives back.
        let jitter = self.rng.roll() % 0x14;
        let mut removable = jitter;
        let mut total = jitter + weight::BASE;
        let m = self.world.tile_mask(t);
        let mut z_val = 0;

        if m & tile::SURFACE == tile::SURFACE_OCEAN {
            // SEAM: unreachable while `can_transport` is zero — every ocean
            // tile is refused by `valid_roadcoord` before this is called.
            total += if avoid_sea {
                weight::SEA_AVOIDED
            } else {
                weight::SEA_ENTER
            };
        } else {
            let c = Cell::new(t.x.div_euclid(4), t.y.div_euclid(4));
            let friendly = match self.world.owner(c) {
                Owner::None => {
                    total += weight::UNOWNED;
                    false
                }
                // SEAM: the alliance arm — a cell of an ally at peace with
                // both ends — is unread; no capture has an ally.
                Owner::Player(o) if o == who.0 => true,
                _ => {
                    total += weight::FOREIGN;
                    false
                }
            };
            if m & (tile::BAD_PATH | tile::BLOCKED) == tile::BAD_PATH
                && m & tile::SURFACE != tile::SURFACE_ROAD
            {
                total += weight::GROUND;
            }
            let d = self.world.cell_data(c);
            if d.flags & cell::ROCK != 0 {
                total += weight::ROUGH * 2;
                removable += weight::ROUGH * 2;
            }
            if m & tile::RIVER != 0 {
                total += weight::ROUGH;
                removable += weight::ROUGH;
            }
            z_val = self.world.tile_z(t).max(0);
            let climb = (z_val - parent_z.abs()).abs().min(weight::CLIMB);
            total += climb * 3;
            removable += climb * 3;

            let on_road = m & tile::SURFACE == tile::SURFACE_ROAD;
            let ground = if friendly {
                weight::GROUND >> 3
            } else {
                weight::GROUND * 4
            };
            if on_road {
                // The road bonus, and the sign that carries "I am on the
                // road" to the next node.
                if parent_z < 0 {
                    total -= removable;
                    if friendly {
                        total /= 2;
                    }
                } else {
                    total += ground;
                }
                z_val = -z_val;
            } else {
                total += ground;
            }
        }

        if !self.road_was_seen(t, who.0) {
            total *= 2;
        }
        if !dir.is_multiple_of(2) {
            // SEAM: a diagonal, which only a caravan's road expands.
            total = total * 7 / 5;
        }
        (total, z_val)
    }

    /// `WorldData::was_seen@006b53f0` at the tile's fog cell. SEAM: the
    /// ally-territory shortcut and the two `leader_flags` arms are unread —
    /// on every capture so far they and the fog bit agree, because the
    /// search never leaves the searcher's own territory.
    fn road_was_seen(&self, t: Pos, who: Player) -> bool {
        self.was_really_seen_fog(t.x >> 1, t.y >> 1, who)
    }
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
fn reconstruct(nodes: &[Node], end: u32) -> Vec<Pos> {
    let mut out = Vec::new();
    let mut cur = nodes[end as usize].parent;
    while let Some(id) = cur {
        let n = nodes[id as usize];
        if n.parent.is_none() {
            break;
        }
        out.push(Pos::new(n.x, n.y));
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
        let road: Vec<i32> = (44..52).filter(|&y| is_road(&sim, 40, y)).collect();
        assert!(
            road.len() >= 4,
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
