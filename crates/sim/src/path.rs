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
//! `SEAM:` with the `docs/PATHFINDER.md` §5 term it stubs — the danger map
//! (zero), diplomacy and rush-rule penalties (none), transports (the unit
//! cannot), and the per-type collision *stride* (one; the size itself is
//! loaded, `docs/COLLISION.md` §2). Unit collision on the 48-grid is no
//! longer one: `valid_ucoord` asks `detect_unit_collision`. Each remaining
//! stub returns the open-ground answer, so on the
//! flat worlds the harness builds the search is exact; the seams are where
//! the remaining layers plug in.
//!
//! Two of those seams closed on 2026-08-26 and they closed **together**:
//! the fog read and the terrain cost. Either alone leaves run20's scout
//! walking a route the original does not take — terrain alone is worse
//! than neither — and the pair reproduces the original's nine-entry chain
//! entry for entry. That is not a coincidence of one capture: a scout's
//! base is `8` on unseen ground against `0x400` on seen, so *whether a
//! cell is known* and *what it costs once known* are the two halves of one
//! number, and pricing one without the other prices nothing.
//!
//! The fog those terms read is the **frame-0 snapshot**, because nothing
//! reveals cells yet (`World::set_fog` has one caller). It is right at the
//! frames a capture is compared on and drifts from there — see
//! `docs/PATHFINDER.md` §12.

use std::collections::BTreeMap;

use crate::orders::{self, Body, MoveKind, PathData, Worker, path_flag};
use crate::world::{Owner, Pos, TILES_PER_CELL, Terrain, UNITS_PER_CELL, cell, tile, vector_dist};
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
pub const REFUSED: i32 = 0x7fff_ffff;

/// One `calc_cost` the search asked for, recorded while [`Sim::trace_costs`]
/// is set — this side's answer to a `CALL`/`RET` pair from the original's own
/// proxy (`tools/trace/tracer.c`, `rondata::trace::Call`).
///
/// The sequence is what makes it an oracle rather than a spot check: two
/// searches that agree on every step's price and still return different
/// routes part somewhere, and the first row where the argument *lists*
/// differ says the validity filter parted, while the first where the answers
/// differ says §5 did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CostMark {
    /// The unit whose search asked.
    pub unit: usize,
    pub from: (i32, i32),
    pub to: (i32, i32),
    /// The wheel index, 1–8, as the original's `dir`.
    pub dir: i32,
    /// [`STEP_WORLD`], [`STEP_TILE`] or [`STEP_UNIT`].
    pub step: i32,
    /// The node's depth from the start, which is the original's `depth`.
    pub depth: i32,
    /// What the function answered; [`REFUSED`] for a refusal.
    pub cost: i32,
}

/// A priced step's whole argument list — `(from.x, from.y, to.x, to.y, dir,
/// step, depth)`. Two sides that answer the same key differently disagree
/// about §5; two sides that never share a key disagree about the search.
pub type CostKey = (i32, i32, i32, i32, i32, i32, i32);

impl CostMark {
    /// This step's [`CostKey`].
    pub fn key(&self) -> CostKey {
        (
            self.from.0,
            self.from.1,
            self.to.0,
            self.to.1,
            self.dir,
            self.step,
            self.depth,
        )
    }
}

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
    /// Read by the unseen-cell and rough-terrain cost terms: an army pays
    /// `0x2480` to step into an unseen cell next to something blocked, and
    /// `+10000` to step onto rough ground it can see.
    army: bool,
    /// A forest-walker. Swaps the terrain cost's `WData.blocked` for the
    /// signed `WData.solid`, and exempts a fully forested tile from the
    /// corner-cutting probes.
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

/// The cell flags a boat's disembark tile may not carry —
/// `invalid_loc@00607c30`'s `WData.flags & 0x70` on the sea arm: mountain
/// (`0x10`), forest (`0x20`) and the unnamed `0x40` beside them.
const SEA_REFUSES: u16 = 0x70;

impl Sim {
    /// `UnitData::invalid_loc(t, ignore_buildings, fog_relax,
    /// enemy_builds_only, transport_a, transport_b)` — the world's refusal
    /// of a tile, for this unit. `t` is in tile coordinates.
    ///
    /// The original splits on the type's **domain** and this does too:
    /// land refuses forest, mountain, cliff and — unless the unit carries
    /// `unit_masks & 0x800000` and the caller asked — water; **sea**
    /// refuses everything that is *not* water, unless the same pair holds,
    /// which is the disembark; **air** refuses nothing.
    ///
    /// SEAM: fog_relax's flag-4-leader branch (stand in the unseen) is a
    /// no-op with no fog model; the cliff and per-cell hazard layers do not
    /// exist, so those refusals never fire; and the land arm's own cell
    /// test — `WData.flags & 0x70` under `ignore_buildings && transport_a`
    /// — is not modelled either.
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
        // ~~SEAM, deliberate: kept out of the water test until the boarding
        // path exists.~~ It exists (`docs/TRANSPORT.md` §6), and this is the
        // original's own predicate: the **raw bit**, not `can_transport` —
        // `invalid_loc@00607c30`'s land arm reads `unit_masks & 0x800000`
        // and neither the veto nor the type flag.
        let transport_forced = self.units[u].auto_transport;
        let ocean = surface == tile::SURFACE_OCEAN;
        match self.unit_domain_of(u) {
            // Air takes anything (`param_4 == 2` → `return 0`), and takes
            // it before the building test too.
            crate::attrition::Domain::Air => return loc::VALID,
            // Land: forest, mountain, cliff, then water. The original
            // tests the three together and only forest reaches the walker
            // exemption: `if (surface == 0x30 || (mask & 3) == 2 ||
            // is_cliff_at(t)) { if (surface != 0x30) return 2; if
            // (!forest_walker) return 2; }`.
            crate::attrition::Domain::Land => {
                if (surface == tile::SURFACE_FOREST && !forest_walker)
                    || mask & tile::OBJECT == tile::OBJECT_MOUNTAIN
                    || mask & tile::OBJECT == tile::OBJECT_CLIFF
                {
                    return loc::TERRAIN;
                }
                if ocean && !((transport_a || transport_b) && transport_forced) {
                    return loc::TERRAIN;
                }
            }
            // Sea, and it is the land arm's mirror: dry land refuses a boat
            // unless the caller asked for the shore *and* the boat
            // `can_transport` — the predicate here is the whole one, not
            // the raw bit — *and* the cell is not mountain, forest or
            // `0x40`. That is the **disembark**, and it is what walks a
            // barge onto the tile `set_new_location` then converts (§6).
            //
            // SEAM: the ocean side's hazard arm — `type +0x1e8` or
            // `is(AIRCRAFTCARRIER)` and then `mask & 0x2400` → 3 — reads a
            // type column this crate does not load.
            crate::attrition::Domain::Sea => {
                if !ocean {
                    if !(transport_a || transport_b) || !self.unit_can_transport(u) {
                        return loc::TERRAIN;
                    }
                    let c = crate::world::Cell::new(
                        t.x.div_euclid(crate::world::TILES_PER_CELL),
                        t.y.div_euclid(crate::world::TILES_PER_CELL),
                    );
                    if self.world.cell_data(c).flags & SEA_REFUSES != 0 {
                        return loc::TERRAIN;
                    }
                }
            }
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

    /// `toff` — the offset `astar_path` carries into the world probe (§6)
    /// and into every reconstructed waypoint (§7), `None` when it does not
    /// apply at all.
    ///
    /// `astar_path`'s prologue reads it through two virtuals of the unit's
    /// **current order**: slot `+0x14` is `UnitOrder::is_move` and slot
    /// `+0x40` is `UnitOrder::update_move_order` (`docs/ORDERS.md` §4.1's
    /// vtable). `is_move` is a folded constant — `1` on the move family,
    /// `0` on the base — and `update_move_order` hands back the order's own
    /// `MoveOrder` sub-object, so `toff` is **that order's** `off_x/off_y`
    /// (`MoveOrder +0x4c/+0x4e`), which
    /// `Unit::add_move_facing_order@005e55c0` writes as `dest % 0x300`.
    /// Nothing about it is conditional on a *target*: a plain move fills it
    /// too (`docs/PATHFINDER.md` §12, run20).
    fn toff(&self, u: usize) -> Option<(i32, i32)> {
        match self.current_order(u)?.body {
            Body::Move(mo) => Some((mo.dest.x % 0x300, mo.dest.y % 0x300)),
            _ => None,
        }
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
    /// search in the original's `validlist`. Its second half is
    /// `detect_unit_collision` in its quick form, which is what makes the
    /// recovery path go **around** the units in the way rather than
    /// through them (`docs/COLLISION.md` §4.2).
    fn valid_ucoord(&self, u: usize, p: Pos, metric: i64, memo: &mut BTreeMap<i64, bool>) -> bool {
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
        let v = self.invalid_loc(u, p.tile(), false, true, false, true, false) == loc::VALID
            && !self.detect_quick(u, p);
        memo.insert(metric, v);
        v
    }

    // `UnitData::needs_transport` is `Sim::needs_transport` in `transport.rs`
    // (`docs/TRANSPORT.md` §6).

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

        // `was_really_seen(div3[to >> 7], div3[to >> 7], who)` — the fog
        // read is at the **half-cell** the step lands in, not the cell's
        // `2c + 1` sample (`crate::scout`). World grid only.
        let seen = step != STEP_WORLD
            || self.was_really_seen_fog(to.x.div_euclid(0x180), to.y.div_euclid(0x180), who);
        if step == STEP_WORLD && !seen {
            // The fog branch: terrain, owner and danger are all unread —
            // the player cannot know them — so the only thing priced is
            // what the cell is *for*. An army pays through the nose to
            // enter an unseen cell next to something blocked; a scout pays
            // almost nothing to enter any unseen cell at all, which is the
            // whole of "exploration seeks the unexplored".
            base = 0x124;
            if m.army && self.world.cell_data(to.cell()).flags & cell::NEARBLOCK != 0 {
                base = 0x2480;
            }
            if m.scouting {
                base = 8;
            }
            extra = if m.scouting { 0 } else { 8 };
        } else if step != STEP_UNIT {
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
                    Owner::Player(p) if self.at_war_with(who, p) => e += 4,
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
                        Owner::Player(p) if self.at_war_with(who, p) => e += 4,
                        _ => {}
                    }
                    if avoid_land != 0 {
                        e += 200;
                    }
                }
                // Terrain movement cost: `WData.blocked` (`+0x11`), or the
                // signed `WData.solid` (`+0x13`) for a forest-walker. 13
                // and up is impassable in all but name — the `+100000`
                // makes any detour cheaper — and an army treats 5 as the
                // line where rough ground stops being worth crossing.
                let d = self.world.cell_data(to_cell);
                let tcost = if m.iroquois {
                    i32::from(d.solid)
                } else {
                    i32::from(d.blocked)
                };
                e += tcost * 20;
                if tcost >= 13 {
                    e += 100_000;
                }
                if m.army {
                    if tcost >= 5 {
                        e += 10_000;
                    }
                    if d.flags & cell::NEARBLOCK != 0 {
                        base <<= 5;
                    }
                }
                extra = e.max(0);
                // §5.1 — the corner-cutting probes, live now that a cell
                // can carry a cost at all.
                if depth < 10 && (tcost != 0 || m.iroquois) && self.cuts_a_corner(from, to, m) {
                    return (REFUSED, false);
                }
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
        }

        // The transport tail. `Sim::unit_can_transport(u)` is the predicate
        // (`docs/TRANSPORT.md` §3.2); it was held at false until the
        // boarding path existed, and §6 is that path, so the embark
        // penalties and refusals are live.
        let crossing = self.needs_transport(from.tile(), to.tile());
        let can_transport = self.unit_can_transport(u);
        // The value the shoreline test **last** answered with, which is
        // what the halfland multiplier below is gated on: at `depth == 1`
        // the second probe overwrites the first (`00685773`–`006858b9`).
        let mut shore = crossing;
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
            shore = self.needs_transport(own_tile, to.tile());
            if shore > 0 && can_transport {
                if avoid_sea == 2 {
                    return (REFUSED, false);
                }
                if shore != 1 {
                    extra += if avoid_sea == 0 && avoid_land == 0 {
                        250
                    } else {
                        1000
                    };
                }
                embarks = true;
            }
        }
        // A halfland cell — one whose second region is the sea — costs
        // three times the base to enter, and the multiplier is skipped
        // whenever the step crossed a shoreline at all, transporter or
        // not (audit V20).
        if step == STEP_WORLD
            && shore <= 0
            && self.world.cell_data(to.cell()).flags & cell::HALFLAND != 0
        {
            base *= 3;
        }

        (base * 32 / 256 + extra + (dir as i32 & 1) * 8, embarks)
    }

    /// `WorldData::is_blocked_at@00461340` — the tile's `BLOCKED` bit,
    /// except that a forest-walker (`mode ≠ 0`) is not stopped by a fully
    /// forested tile.
    fn is_blocked_at(&self, tx: i32, ty: i32, mode: bool) -> bool {
        let mask = self.world.tile_mask(Pos::new(tx, ty));
        mask & tile::BLOCKED != 0 && !(mode && mask & tile::SURFACE == tile::SURFACE_FOREST)
    }

    /// §5.1 — the corner-cutting refusal. `from`/`to` are the step's
    /// endpoints; every probe is on tiles around the **from** tile.
    ///
    /// A diagonal is refused when **either** of its two probes is blocked;
    /// a cardinal only when **all four** of its are — you cannot cut past a
    /// building's corner, but you may walk at a wall until it runs out.
    /// The direction is recovered from the step's own world-cell delta —
    /// `div_3_table[(to − from) >> 8]`, matched against `move_x`/`move_y`
    /// — and **not** taken from the caller's wheel index, which is why
    /// this does not take one. A big unit's stride is two cells, no
    /// `move_x` entry matches a delta of two, and the original therefore
    /// skips the whole block for it; passing `dir` would quietly invent a
    /// different behaviour the day big-unit strides go live.
    fn cuts_a_corner(&self, from: Pos, to: Pos, m: &Modes) -> bool {
        let cdx = (to.x - from.x).div_euclid(0x300);
        let cdy = (to.y - from.y).div_euclid(0x300);
        let Some(d) = (1..=8).find(|&d| MOVE_X[d] == cdx && MOVE_Y[d] == cdy) else {
            return false;
        };
        let (cx, cy) = (from.tile().x, from.tile().y);
        let probes: &[(i32, i32)] = match d {
            1 => &[(cx - 2, cy - 2), (cx - 3, cy - 3)],
            2 => &[
                (cx - 2, cy - 3),
                (cx - 1, cy - 3),
                (cx, cy - 3),
                (cx + 1, cy - 3),
            ],
            3 => &[(cx + 1, cy - 2), (cx + 2, cy - 3)],
            4 => &[
                (cx + 2, cy - 2),
                (cx + 2, cy - 1),
                (cx + 2, cy),
                (cx + 2, cy + 1),
            ],
            5 => &[(cx + 1, cy + 1), (cx + 2, cy + 2)],
            6 => &[
                (cx - 2, cy + 2),
                (cx - 1, cy + 2),
                (cx, cy + 2),
                (cx + 1, cy + 2),
            ],
            7 => &[(cx - 2, cy + 1), (cx - 3, cy + 2)],
            _ => &[
                (cx - 3, cy - 2),
                (cx - 3, cy - 1),
                (cx - 3, cy),
                (cx - 3, cy + 1),
            ],
        };
        let blocked = |&(x, y): &(i32, i32)| self.is_blocked_at(x, y, m.iroquois);
        if d & 1 == 1 {
            probes.iter().any(blocked)
        } else {
            probes.iter().all(blocked)
        }
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
    /// SEAM: suspension stashes nothing — `find_upath_restore` still has
    /// no caller, so a suspended search is simply lost. The condition and
    /// return are the original's.
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
        // SEAM: the unit-grid stride is `(type collision + 1) / 2`, which
        // is 1 for every `BLOCK_RADIUS 1` type — all of them in every
        // capture so far (`docs/COLLISION.md` §2).
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
        // The prologue's `toff`; zero when the current order is not a move,
        // which is exactly what the probe's `− 0x180` then means.
        let toff = self.toff(u).unwrap_or((0, 0));

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
        let mut open_by_metric: BTreeMap<i64, (u64, i32, u32)> = BTreeMap::new();
        let mut closed: BTreeMap<i64, u32> = BTreeMap::new();
        let mut valid_memo: BTreeMap<i64, bool> = BTreeMap::new();
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
                            // never target one, so no draw. The retry
                            // cooldown is: `+0xb2 += 30`, which
                            // `detect_unit_collision` reads as "stop
                            // colliding for thirty frames"
                            // (`docs/COLLISION.md` §4.1).
                            self.units[u].safe += 30;
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
                        // The first two steps probe `node + toff − 0x180`
                        // — the cell corner when the order is not a move,
                        // the order's own sub-cell offset when it is;
                        // later steps the node itself.
                        let probe = if cur.timeout < 2 {
                            Pos::new(nx + toff.0 - 0x180, ny + toff.1 - 0x180)
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
                if self.trace_costs {
                    self.cost_marks.push(CostMark {
                        unit: u,
                        from: (cur.x, cur.y),
                        to: (nx, ny),
                        dir: d as i32,
                        step,
                        depth: cur.timeout + 1,
                        cost,
                    });
                }
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

        // `toff` again, re-read here as the original re-reads it: the walk
        // offsets each emitted node only while the current order `is_move`,
        // and leaves it where it stands otherwise.
        let toff = self.toff(u);
        let mut id = Some(root_id);
        while let Some(i) = id {
            let n = nodes[i as usize];
            if n.parent.is_none() && step != STEP_UNIT {
                break;
            }
            // **The tile grid's tolerance is not a constant** (`00684bfc`):
            // a unit that can transport is given an *exact* waypoint, and
            // everything else is allowed to call a waypoint reached within
            // half a tile. The test is `anti_unit == 0` and either
            // `unit_masks & 0x800000` without `unit_masks2 & 0x2000` — the
            // auto-transport pair — or a type carrying `unit_flags & 0x10`,
            // the sea transport's own bit.
            //
            // It costs three frames a leg. East Indies' AI citizen `1/13`
            // is granted `0x800000` on frame 3580, the frame after its
            // Dock finishes (`docs/TRANSPORT.md` §3), and from there every
            // tile-grid waypoint it takes is one the original walks onto
            // and this crate cut the corner of — thirteen frames by the
            // time it reached its wood tile, and the sync word thirteen
            // frames early with it.
            let can_transport = (self.units[u].auto_transport && !self.units[u].never_transport)
                || self.units[u].ty.is_some_and(|t| {
                    self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::TRANSPORT != 0
                });
            let mut tolerance = match step {
                STEP_WORLD => 0x180,
                STEP_UNIT => 0,
                _ if !m.anti_unit && can_transport => 0,
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
            // World: `node + toff − 0x180`. Tile: `node + toff % 0xc0 −
            // 0x60`. The unit grid never offsets.
            let to = match (toff, step) {
                (Some((tx, ty)), STEP_WORLD) => Pos::new(n.x + tx - 0x180, n.y + ty - 0x180),
                (Some((tx, ty)), STEP_TILE) => {
                    Pos::new(n.x + tx % 0xc0 - 0x60, n.y + ty % 0xc0 - 0x60)
                }
                _ => Pos::new(n.x, n.y),
            };
            let entry = PathData {
                to,
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
        let here = self.units[u].pos;
        self.find_wpath_from(u, here, false)
    }

    /// [`Sim::find_wpath`] with the two arguments the group's own plan
    /// supplies (`docs/GROUPS.md` §6.7).
    ///
    /// The original's `PathFinder::find_wpath@00688fc0` takes the stack and
    /// the start point as arguments; the four-argument overload at
    /// `00688e10` is the one that reads the object's own position, and it is
    /// what every ordinary caller uses. `Group::action_move_near` calls the
    /// six-argument form with a **static** `grouppath` stack and a start
    /// that is the leader's top-of-stack, so this simulation installs that
    /// stack on the leader for the call and takes it back after.
    ///
    /// `army_hint` is `pathfinder +0x70` — `Group::action_move_near` sets it
    /// to 1 around the call for a group that belongs to an army
    /// (`706178`–`70619f`: `cmpl $0x0, 0x8(%eax)` on `GroupData::army`, then
    /// `movl $0x1, 0xe85eb0`). The flag is the **same** `army` mode
    /// `find_wpath` derives for an AI's own units at `00688fc0:242`, forced
    /// on from outside — which is the only way a *human*'s army ever gets
    /// it, since a human jumps the whole mode block (`leaders & 4` at
    /// `0068973d`).
    pub(crate) fn find_wpath_from(&mut self, u: usize, here: Pos, army_hint: bool) -> i32 {
        let Some(goal_e) = self.units[u].path.pop() else {
            return 0;
        };
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
        //
        // **A unit that can board skips it whole**, and that gate is the
        // whole of transport pathing: `00689375` runs the walk only for
        // `domain < 2 && (!is_on_map() || !can_transport())`, so a land
        // unit with `unit_masks & 0x800000` keeps the goal it was given and
        // the search is asked to cross the water. Without the gate the goal
        // is dragged back onto the unit's own island and the route ends at
        // the shore — which is what this crate did on the day `go_here`
        // first pointed a scout at another one (`docs/TRANSPORT.md` §7).
        //
        // SEAM: the walk's own break test has a second clause for a **sea**
        // unit — the matched region must also pass `invalid_loc(t, 0, 1, 1,
        // 1, 0)` — that this crate does not make.
        let mut goal_e = goal_e;
        let mut goal = goal_e.to;
        let walks = self.unit_domain_of(u) != crate::attrition::Domain::Air
            && (!self.units[u].on_map || !self.unit_can_transport(u));
        while walks && self.world.tregion(goal.tile()) != self.world.tregion(here.tile()) {
            let (dx, dy) = (here.x - goal.x, here.y - goal.y);
            let far = dx.abs() + dy.abs() >= 0x300;
            let s = if far { 0x180 } else { 0x30 };
            // The give-up exit: a remainder smaller than the step on both
            // axes takes the goal where it stands **without running A\***
            // (audit V17) — the same push-and-return as reaching the
            // start's cell. Only a region match continues to the search.
            if dx.abs() < s && dy.abs() < s {
                self.units[u].path.push(goal_e);
                return self.units[u].path.len() as i32;
            }
            let ang = movement::find_angle(dx, dy);
            // **No sign flip here.** The decompiler's `if (angle < 0) step =
            // -step` before the two `sin_table` calls is the *inlined* fold
            // that [`movement::sin_component`] already performs — and the
            // listing settles it: at `0x6894c3` the cosine's distance is
            // reloaded from the un-negated `s` and negated again only on the
            // sign of `angle + 0x40000000`. Doing it twice cancels the fold,
            // which sends the pull-back away from the start instead of
            // toward it; the walk then never converges. Every capture broke
            // out of this loop on its first region test, so nothing caught
            // it until a dock stood one cell further out
            // (`docs/PATHFINDER.md` §13).
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
            // `scouting = 1` iff the **type** is a scout — `role & 0x10`,
            // `is(SCOUT)` on land and `is(BARK)` at sea
            // (`UnitType::determine_roles@0061c320`) — **and** the order is
            // `EXPLORE_TO` (§3). The type half was missing, and it is not a
            // refinement: `scouting` prices seen ground at `0x400` against
            // unseen `8`, so any unit given an `EXPLORE_TO` walked toward
            // the fog. run10's AI citizen `1/1` is sent to its second
            // city's site under an `EXPLORETO` on frame 777 and took a
            // ten-cell detour west through unexplored ground where the
            // original walks seven cells south-east.
            //
            // SEAM: the clause `order.flags & 4 == 0 || unit_masks &
            // 0x40100`, which can only ever turn scouting *off* for a
            // scout whose explore order carries `ACTION`. Left out until
            // the two mask bits are modelled; every explore order in the
            // corpus has `flags 1`.
            scouting: self.units[u]
                .ty
                .is_some_and(|t| self.unit_types[t].cols.is(crate::ai_load::role::SCOUT))
                && self.current_order(u).is_some_and(
                    |o| matches!(o.body, Body::Move(mo) if mo.kind == MoveKind::ExploreTo),
                ),
            army: army_hint || (!human && self.army_mode(u)),
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
            // `0x60`, the same constant as the give-up test above, and no
            // caller-level fold — `mov ebx, 0x60` at `0x6899da`, negated to
            // `0xffffffa0` only inside the sine's own fold.
            let s = 0x60;
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
    /// only. Its caller is `Sim::resolve_unit_collision`
    /// (`docs/COLLISION.md` §6 step 6).
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
        let mut memo = BTreeMap::new();
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
            // `0x18` here — `mov edi, 0x18` at `0x68318b`.
            let s = 0x18;
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

    /// §7's `toff`: with a move order current, every reconstructed world
    /// node comes out at `cell*0x300 + off`, where `off` is the **order's
    /// own** `off_x/off_y` — the cell centre only when the offset happens
    /// to be `0x180`. The seam this replaced emitted the centre always, so
    /// pinning `toff` back to zero fails this on the first waypoint.
    #[test]
    fn a_move_orders_world_chain_carries_the_orders_own_sub_cell_offset() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        // A destination six cells east whose 48-grid snap puts it at
        // `+0x1f8` inside its cell — run20's own `off_x = 504`.
        let goal = Pos::new(6 * 0x300 + 0x1f8, 0x1f8);
        sim.add_move_order(u, goal, MoveKind::MoveTo, orders::QueuePos::New, false);
        let dest = match sim.current_order(u).expect("the move").body {
            Body::Move(mo) => mo.dest,
            _ => unreachable!(),
        };
        assert_eq!(
            (dest.x % 0x300, dest.y % 0x300),
            (504, 504),
            "the order's off"
        );
        assert_eq!(sim.toff(u), Some((504, 504)));

        push_goal(&mut sim, u, dest);
        let r = sim.find_wpath(u);
        assert!(r > 1, "expected a planned chain, got {r}");
        let path = &sim.units[u].path;
        assert_eq!(path[0].to, dest, "the goal is pushed as it stands");
        for p in &path[1..] {
            assert_eq!(p.to.x % 0x300, 504, "not on the order's offset: {:?}", p.to);
            assert_eq!(p.to.y % 0x300, 504, "not on the order's offset: {:?}", p.to);
        }
    }

    /// And the same walk with **no** move order current leaves the nodes on
    /// the cell centre: `toff` is `None`, and then nothing is added or
    /// subtracted at all.
    #[test]
    fn without_a_move_order_the_world_chain_stays_on_the_cell_centres() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        assert_eq!(sim.toff(u), None, "a unit with no order has no offset");
        push_goal(&mut sim, u, Pos::new(6 * 0x300 + 0x180, 0x180));
        assert!(sim.find_wpath(u) > 1);
        for p in &sim.units[u].path[1..] {
            assert_eq!(p.to.x % 0x300, 0x180, "{:?}", p.to);
            assert_eq!(p.to.y % 0x300, 0x180, "{:?}", p.to);
        }
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

    /// **The tile grid's tolerance is the unit's, not a constant**
    /// (`00684bfc`). A unit that may auto-transport is given the waypoint
    /// exactly; everything else may call it reached within `0x60`.
    ///
    /// The difference is three frames a leg at speed 25, and thirteen by
    /// the time East Indies' AI citizen `1/13` reaches its wood tile —
    /// which is what put the long capture's word at 4275 rather than
    /// 4313 (`docs/PATHFINDER.md` §7).
    #[test]
    fn a_unit_that_can_transport_gets_its_tile_waypoints_exactly() {
        let mut sim = flat_sim(12);
        let u = walker(&mut sim, Pos::new(4 * 0xc0 + 0x60, 4 * 0xc0 + 0x60));
        let goal = Pos::new(20 * 0xc0 + 0x60, 12 * 0xc0 + 0x60);
        push_goal(&mut sim, u, goal);
        assert!(sim.find_tpath(u) > 1, "expected a tile route");
        let plain: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
        assert!(
            !plain.is_empty() && plain.iter().all(|&t| t == 0x60),
            "a unit that cannot transport takes the half-tile tolerance: {plain:?}"
        );

        // The same walk with `unit_masks & 0x800000` set.
        sim.units[u].path.clear();
        sim.units[u].auto_transport = true;
        push_goal(&mut sim, u, goal);
        assert!(sim.find_tpath(u) > 1, "expected a tile route");
        let exact: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
        assert_eq!(exact.len(), plain.len(), "the same route, twice");
        assert!(
            exact.iter().all(|&t| t == 0),
            "and it walks onto each waypoint: {exact:?}"
        );

        // `unit_masks2 & 0x2000` is the scenario's veto and takes it back.
        sim.units[u].path.clear();
        sim.units[u].never_transport = true;
        push_goal(&mut sim, u, goal);
        assert!(sim.find_tpath(u) > 1, "expected a tile route");
        let vetoed: Vec<i32> = sim.units[u].path[1..].iter().map(|p| p.tolerance).collect();
        assert_eq!(vetoed, plain, "the veto puts the half-tile back");
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

    /// §5's terrain term: `+20 × WData.blocked`, the `+100000` at 13, and
    /// an army's `+10000` at 5. None of it fires on a flat world, which is
    /// why it sat behind a seam for so long; all of it fires the moment a
    /// cell record carries a cost.
    #[test]
    fn rough_ground_costs_twenty_a_point_and_thirteen_is_a_wall() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let from = Pos::new(0x180, 0x180);
        let to = Pos::new(0x180 + 0x300, 0x180);
        let cost = |sim: &Sim, m: &Modes| sim.calc_cost(u, m, from, to, 4, STEP_WORLD, 2, 0, 1).0;
        let plain = Modes::default();
        let army = Modes {
            army: true,
            ..Modes::default()
        };
        assert_eq!(cost(&sim, &plain), 32, "clean ground");

        let mut d = sim.world.cell_data(to.cell());
        d.blocked = 4;
        sim.world.set_cell_data(to.cell(), d);
        assert_eq!(cost(&sim, &plain), 32 + 80);
        assert_eq!(cost(&sim, &army), 32 + 80, "4 is under the army's line");

        d.blocked = 5;
        sim.world.set_cell_data(to.cell(), d);
        assert_eq!(cost(&sim, &plain), 32 + 100);
        assert_eq!(cost(&sim, &army), 32 + 100 + 10_000, "rough, to an army");

        d.blocked = 13;
        sim.world.set_cell_data(to.cell(), d);
        assert_eq!(cost(&sim, &plain), 32 + 260 + 100_000, "impassable");

        // The forest-walker reads the **signed** `solid` byte instead, and
        // a negative one is a discount the clamp then eats.
        d.solid = -1;
        sim.world.set_cell_data(to.cell(), d);
        let iroquois = Modes {
            iroquois: true,
            ..Modes::default()
        };
        assert_eq!(cost(&sim, &iroquois), 32, "solid −1, clamped at zero");
    }

    /// §5's fog branch, and the half of it that is the whole of scouting:
    /// on ground it cannot see, a scout's base is **8** against the `0x400`
    /// it pays for ground it can — a factor of 128 — so a scout's cheapest
    /// route is the one through the dark. Everyone else pays `0x124 + 8`
    /// for the unknown, slightly more than the `0x100` of the known.
    ///
    /// The read is at the step's **half-cell**, `to / 0x180`, not the
    /// cell's `2c + 1` sample: with the first half-cell of the destination
    /// dark and the second lit, a step landing in the first is priced as
    /// unseen. No capture on disk separates those two readings — every
    /// world node in every dump sits past the half-cell line — so this is
    /// the only thing holding the convention.
    #[test]
    fn the_unseen_is_cheap_to_a_scout_and_the_read_is_the_half_cell() {
        let mut sim = flat_sim(10);
        // A fog grid of all-dark. `seen2` is one byte a half-cell, a bit a
        // player, `2 × width` across.
        let fw = 20usize;
        let mut fog = vec![0u8; fw * fw];
        assert!(sim.world.set_fog(fog.clone()));
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        assert_eq!(sim.units[u].owner, 0);
        let from = Pos::new(0x180, 0x180);
        let scout = Modes {
            scouting: true,
            ..Modes::default()
        };
        let plain = Modes::default();
        let cost =
            |sim: &Sim, m: &Modes, to: Pos| sim.calc_cost(u, m, from, to, 4, STEP_WORLD, 2, 0, 1).0;

        // Dark everywhere: `base 8` for the scout, `0x124 + 8` for the rest.
        let to = Pos::new(0x180 + 0x300, 0x180);
        assert_eq!(cost(&sim, &scout, to), 1, "8 × 32 / 256");
        assert_eq!(cost(&sim, &plain, to), 0x124 * 32 / 256 + 8);

        // Light the destination cell's **second** half-cell only. `to` is
        // at `+0x180` inside its cell, which is the second half — so it
        // reads lit, and the scout pays the seen base.
        let c = to.cell();
        for fy in [2 * c.y, 2 * c.y + 1] {
            fog[fy as usize * fw + (2 * c.x + 1) as usize] = 1;
        }
        assert!(sim.world.set_fog(fog));
        assert_eq!(cost(&sim, &scout, to), 0x400 * 32 / 256, "seen: 128");
        assert_eq!(cost(&sim, &plain, to), 32);

        // A step landing in the same cell's **first** half-cell is still
        // dark. `2c + 1` would call it seen; `to / 0x180` does not.
        let first_half = Pos::new(c.x * 0x300 + 0x80, 0x80);
        assert_eq!(first_half.cell(), c);
        assert_eq!(cost(&sim, &scout, first_half), 1, "the near half is dark");
    }

    /// §5's halfland multiplier — `base ×= 3` on a cell whose `flags` carry
    /// `0x100`, and **skipped** whenever the step crossed a shoreline
    /// (audit V20). Nothing on disk exercises it: the routes the captures
    /// take either miss halfland cells or cross into them.
    #[test]
    fn a_halfland_cell_triples_the_base_unless_the_step_crossed_the_shore() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let from = Pos::new(0x180, 0x180);
        let to = Pos::new(0x180 + 0x300, 0x180);
        let m = Modes::default();
        assert_eq!(sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0, 32);

        let mut d = sim.world.cell_data(to.cell());
        d.flags |= crate::world::cell::HALFLAND;
        sim.world.set_cell_data(to.cell(), d);
        assert_eq!(
            sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0,
            96,
            "0x300 × 32 / 256"
        );
        // The multiplier is the *base*'s, so the diagonal's +8 sits outside
        // it, and the tile grid never sees it at all.
        let se = Pos::new(to.x, to.y + 0x300);
        let mut d2 = sim.world.cell_data(se.cell());
        d2.flags |= crate::world::cell::HALFLAND;
        sim.world.set_cell_data(se.cell(), d2);
        assert_eq!(
            sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 2, 0, 1).0,
            104
        );
    }

    /// §5.1's probe table: a diagonal is refused when **either** of its two
    /// probes is blocked, a cardinal only when **all four** of its are —
    /// and the whole block is skipped on ground that costs nothing, which
    /// is why a flat world never sees it.
    #[test]
    fn a_diagonal_is_refused_past_one_corner_and_a_cardinal_needs_all_four() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let from = Pos::new(3 * 0x300 + 0x180, 3 * 0x300 + 0x180);
        let (cx, cy) = (from.tile().x, from.tile().y);
        // Give the destination cells a cost, or §5.1 never runs.
        for (dx, dy) in [(1, 1), (1, 0)] {
            let c = crate::world::Cell::new(from.cell().x + dx, from.cell().y + dy);
            let mut d = sim.world.cell_data(c);
            d.blocked = 1;
            sim.world.set_cell_data(c, d);
        }
        let m = Modes::default();
        let se = Pos::new(from.x + 0x300, from.y + 0x300);
        let east = Pos::new(from.x + 0x300, from.y);
        let cost = |sim: &Sim, to: Pos, dir: usize| {
            sim.calc_cost(u, &m, from, to, dir, STEP_WORLD, 2, 0, 1).0
        };
        assert_ne!(cost(&sim, se, 5), REFUSED, "clear to begin with");

        // SE probes `(cx+1, cy+1)` then `(cx+2, cy+2)`; either is enough.
        sim.world
            .set_tile_bits(Pos::new(cx + 2, cy + 2), tile::BLOCKED);
        assert_eq!(cost(&sim, se, 5), REFUSED, "the far corner alone");
        sim.world
            .clear_tile_bits(Pos::new(cx + 2, cy + 2), tile::BLOCKED);
        sim.world
            .set_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);
        assert_eq!(cost(&sim, se, 5), REFUSED, "the near corner alone");
        sim.world
            .clear_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);

        // East probes the whole column `(cx+2, cy-2 ..= cy+1)` and needs
        // every one of them.
        let col = [
            Pos::new(cx + 2, cy - 2),
            Pos::new(cx + 2, cy - 1),
            Pos::new(cx + 2, cy),
            Pos::new(cx + 2, cy + 1),
        ];
        for t in &col[..3] {
            sim.world.set_tile_bits(*t, tile::BLOCKED);
        }
        assert_ne!(cost(&sim, east, 4), REFUSED, "three of four is a gap");
        sim.world.set_tile_bits(col[3], tile::BLOCKED);
        assert_eq!(cost(&sim, east, 4), REFUSED, "the edge is built over");

        // And a fully forested tile is not a wall to a forest-walker.
        for t in &col {
            sim.world.set_tile_bits(*t, tile::SURFACE_FOREST);
        }
        let iroquois = Modes {
            iroquois: true,
            ..Modes::default()
        };
        assert_ne!(
            sim.calc_cost(u, &iroquois, from, east, 4, STEP_WORLD, 2, 0, 1)
                .0,
            REFUSED,
            "the Iroquois walk through the trees"
        );
    }

    /// The depth gate: §5.1 runs only for the first nine steps of a search
    /// (`depth < 10`), so a corner ten steps out is not probed at all.
    #[test]
    fn the_corner_probes_stop_after_nine_steps() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let from = Pos::new(3 * 0x300 + 0x180, 3 * 0x300 + 0x180);
        let (cx, cy) = (from.tile().x, from.tile().y);
        let se = Pos::new(from.x + 0x300, from.y + 0x300);
        let mut d = sim.world.cell_data(se.cell());
        d.blocked = 1;
        sim.world.set_cell_data(se.cell(), d);
        sim.world
            .set_tile_bits(Pos::new(cx + 1, cy + 1), tile::BLOCKED);
        let m = Modes::default();
        assert_eq!(
            sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 9, 0, 1).0,
            REFUSED
        );
        assert_ne!(
            sim.calc_cost(u, &m, from, se, 5, STEP_WORLD, 10, 0, 1).0,
            REFUSED,
            "depth 10 is past the gate"
        );
    }

    /// **The original's own answers, under an emulator.** `tools/emu/callfn.py`
    /// enters `vector_dist@0046cff0` and `get_estimate@00688310` in the
    /// executable's image under unicorn — no game, no capture — over a seeded
    /// sweep, and prints `<name> <args…> -> <eax>`; every row is asserted
    /// against this crate here. The table is `$RON_EMU_TABLE`, or is generated
    /// from the install on the spot; a machine with neither says so.
    ///
    /// The sweep's first catch, 2026-09-01: `lo * lo` in `world::vector_dist`
    /// overflowed `i32` above `lo = 46340`, where the original squares in
    /// `unsigned` — 59999² is 3,599,880,001.
    #[test]
    fn the_emulated_original_agrees_on_every_row() {
        let Some(table) = emu_table() else { return };
        let mut rows = 0;
        for line in table.lines() {
            let (lhs, rhs) = line.split_once(" -> ").expect("a row");
            let f: Vec<&str> = lhs.split(' ').collect();
            let a: Vec<i32> = f[1..].iter().map(|s| s.parse().unwrap()).collect();
            let want: i32 = rhs.parse().unwrap();
            let got = match f[0] {
                "vector_dist" => crate::world::vector_dist(a[0], a[1]),
                "get_estimate" => Sim::estimate(a[0] - a[2], a[1] - a[3], a[4]),
                other => panic!("unknown function {other}"),
            };
            assert_eq!(got, want, "{line}");
            rows += 1;
        }
        assert!(rows > 3000, "{rows} rows");
    }

    /// The emulator's table: `$RON_EMU_TABLE` as written, else the sweep run
    /// against the install (`testenv::install_root`) through `uv`.
    fn emu_table() -> Option<String> {
        if let Ok(p) = std::env::var("RON_EMU_TABLE") {
            return Some(std::fs::read_to_string(p).expect("RON_EMU_TABLE"));
        }
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let Some(install) = crate::testenv::install_root() else {
            eprintln!("skipping: set RON_EMU_TABLE or RON_INSTALL");
            return None;
        };
        let out = std::process::Command::new("uv")
            .args(["run", &format!("{root}/tools/emu/callfn.py")])
            .arg(format!("{install}/riseofnations.exe"))
            .arg("sweep")
            .output();
        match out {
            Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).unwrap()),
            Ok(o) => panic!("callfn.py failed: {}", String::from_utf8_lossy(&o.stderr)),
            Err(e) => {
                eprintln!("skipping: uv not runnable ({e})");
                None
            }
        }
    }
}
