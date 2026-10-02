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
use crate::world::{Owner, Pos, TILES_PER_CELL, UNITS_PER_CELL, cell, tile, vector_dist};
use crate::{Sim, movement};

/// **A payoff probe's hand**, not a mechanic (item 563,
/// `docs/PATHFINDER.md` §23): one unit cell that `valid_ucoord` refuses to
/// one unit's searches on one sim-frame, on top of whatever the index
/// says. The diff harness uses it to ask whether a single verdict is the
/// whole difference between two plans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefuseProbe {
    /// The sim-frame the refusal holds on.
    pub frame: i64,
    /// The searching unit, `(who, o)`.
    pub unit: (i32, i32),
    /// The unit cell (48-unit grid) refused.
    pub cell: Pos,
}

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

/// **A suspended 48-grid search** — `UnitData +0x104..0x148`
/// (`docs/PATHFINDER.md` §4.3 step 3, §18).
///
/// The original hands the five `PathFinder` containers to the unit and
/// takes fresh ones from the recyclers; here the containers *are* the
/// state, so the struct is the hand-over. Every field is one the original
/// stores by name: the PDB calls them `openlist`, `openlistrefs`,
/// `closedlist`, `validlist`, `blocklist`, then `tol` (`+0x128`), `offset`
/// (`+0x12c`, which is the direction *preference*, not an offset),
/// `start_dist` (`+0x130`), `valid_hit` (`+0x134`), `avoid_land`/
/// `avoid_sea` (`+0x138`/`+0x13c`), `endx`/`endy` (`+0x140`/`+0x144`) and
/// `traversed` (`+0x148`).
///
/// ~~`blocklist` has no counterpart here — the unit search never fills
/// it~~ `blocklist` (`+0x114`) is [`Search::block_copies`]: every
/// `valid_ucoord` probe is a `nocoll` one and fills it
/// (`docs/PATHFINDER.md` §26, item 678). `valid_hit` is a counter nothing
/// reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Search {
    nodes: Vec<Node>,
    open: BTreeMap<(i32, std::cmp::Reverse<u64>), u32>,
    open_by_metric: BTreeMap<i64, (u64, i32, u32)>,
    closed: BTreeMap<i64, u32>,
    valid_memo: BTreeMap<i64, bool>,
    /// `+0x114` — the `blocklist` the suspend handed over
    /// ([`Sim::coll_copies`]).
    block_copies: BTreeMap<(i32, i32), [u16; 16]>,
    seq: u64,
    /// `+0x128` — the *final* goal entry's tolerance, which is what
    /// `arrive` is built from and is not re-read from the stack on resume.
    tol: i32,
    /// `+0x12c` — the direction wheel's start.
    pref: i32,
    /// `+0x140`/`+0x144`.
    goal: Pos,
    /// `+0x130`. **The one dumped witness that a search suspended**: the
    /// start-to-goal Manhattan, and `astar_path`'s suspend block is its
    /// only writer in the whole executable.
    start_dist: i32,
    /// `+0x148` — `traversed + probes` at the moment of the suspend.
    traversed: i32,
    avoid_land: i32,
    avoid_sea: i32,
}

#[cfg(test)]
impl Search {
    /// An empty stash, for a test that needs a unit to be holding one.
    pub(crate) fn suspended_for_test() -> Self {
        Search {
            nodes: Vec::new(),
            open: BTreeMap::new(),
            open_by_metric: BTreeMap::new(),
            closed: BTreeMap::new(),
            valid_memo: BTreeMap::new(),
            block_copies: BTreeMap::new(),
            seq: 0,
            tol: 0,
            pref: 0,
            goal: Pos::new(0, 0),
            start_dist: 0,
            traversed: 0,
            avoid_land: 0,
            avoid_sea: 0,
        }
    }
}

/// One search node — `PathNode`, minus the allocator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub(crate) mod loc {
    pub const VALID: i32 = 0;
    pub const OFF_MAP: i32 = 1;
    pub const TERRAIN: i32 = 2;
    /// Shallows/river hazard for an armed ship — the code `valid_wcoord`
    /// forgives inside the goal cell.
    pub const HAZARD: i32 = 3;
    pub const BUILDING: i32 = 4;
}

/// The cell flags a boat's disembark tile may not carry —
/// `invalid_loc@00607c30`'s `WData.flags & 0x70` on the sea arm: mountain
/// (`0x10`), forest (`0x20`) and the unnamed `0x40` beside them. The land
/// arm reads the same three for a world probe (item 776).
const SEA_REFUSES: u16 = 0x70;

/// **The unit-grid search's retry roll, open-list-exhausted tail** —
/// `Random::get(game_random, 0, 0xffff)` at `00684e02`, so the trace's
/// return address is `00684e07` (`astar_path@00683770+0x1697`). `% 3 + 6`
/// goes into the current order's `MoveOrder::retry`, and the roll is spent
/// only when that order is a transit **and** its `attempts` is under 13
/// (`docs/PATHFINDER.md` §21).
pub const SITE_UPATH_RETRY: &str = "PathFinder::astar_path+0x1697";

/// **The same roll on the work-cap tail** — `006848c4`, return address
/// `006848c9` (`+0x1159`). It is a *different* site and a **different
/// gate**: the transit test alone, with no `attempts` ceiling
/// (`astar_path@00683770:552`-`563` against `:934`-`967`). Reading the two
/// tails as one is the error this pair exists to prevent.
pub const SITE_UPATH_RETRY_BUDGET: &str = "PathFinder::astar_path+0x1159";

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
    /// **`fog_relax` is the human's walk into the unknown** (item 676,
    /// `docs/GOLDEN.md` §17). With it set and the unit's leader human
    /// (`leaders & 4`, the test `find_wpath` reads as `Nation::human`), a
    /// tile is valid outright when its **cell's four fog half-cells** —
    /// `(2cx, 2cy)`, `(2cx, 2cy+1)`, `(2cx+1, 2cy)`, `(2cx+1, 2cy+1)` —
    /// are all unseen by the unit's owner, before the domain is read
    /// (`00607c9e`–`00607d4c`). `valid_wcoord` passes it past a node's
    /// second step, so a human's world plan runs straight through ground
    /// the player has never seen, and is refused only when the unit comes
    /// to see it. run180's chariot takes its plan through a sandy lake and
    /// re-plans round it on 693. `was_seen` is [`Sim::was_seen_fog`], the
    /// goody sweep's; with no fog grid installed it answers seen, and the
    /// arm is silent.
    ///
    /// SEAM: the cliff and per-cell hazard layers do not exist, so those
    /// refusals never fire; and the land arm's own cell test —
    /// `WData.flags & 0x70` under `ignore_buildings && transport_a` — is
    /// not modelled either.
    #[allow(clippy::too_many_arguments)] // the original's five flags, kept by name
    pub(crate) fn invalid_loc(
        &self,
        u: usize,
        t: Pos,
        ignore_buildings: bool,
        fog_relax: bool,
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
        let owner = self.units[u].owner;
        if fog_relax && self.nation.get(owner as usize).is_some_and(|n| n.human) {
            let (fx, fy) = (
                t.x.div_euclid(TILES_PER_CELL) * 2,
                t.y.div_euclid(TILES_PER_CELL) * 2,
            );
            let seen = [(0, 0), (0, 1), (1, 0), (1, 1)]
                .iter()
                .any(|&(dx, dy)| self.was_seen_fog(fx + dx, fy + dy, owner));
            if !seen {
                return loc::VALID;
            }
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
                // **The cell first** (`00607e6f`–`00607ead`, item 776): a
                // cell flagged mountain, forest or `0x40`
                // (`WData.flags & 0x70`) refuses when the caller passes
                // both `param_3` and `param_6` — `valid_wcoord`'s probe, so
                // the world search walks round a forest *cell* where the
                // tile test would only refuse its forest *tiles* — unless
                // it is forest and the unit a forest-walker.
                let cf = self
                    .world
                    .cell_data(crate::world::World::cell_of_tile(t))
                    .flags;
                if cf & SEA_REFUSES != 0
                    && (cf & crate::world::cell::FOREST == 0 || !forest_walker)
                    && ignore_buildings
                    && transport_a
                {
                    return loc::TERRAIN;
                }
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
            // The ocean side's hazard arm: a warship — `type +0x1e8`, the
            // `attack`, or `is(AIRCRAFTCARRIER, 1)` — on a `0x2400` tile is
            // 3, before any flag is read. A dock's margin is `BAD_PATH`
            // (`docs/ORDERS.md` §25), so this is what keeps a dock's own
            // warships off the water beside it.
            crate::attrition::Domain::Sea => {
                if ocean && mask & crate::orders::WARSHIP_REFUSES != 0 && self.is_warship(u) {
                    return loc::HAZARD;
                }
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
            // **`enemy_builds_only`: an armed unit passes a building that
            // is not its owner's** (`00607fb6`..`0060800a`, item 1209). With
            // the caller's `param_5`, a type whose base attack (`+0x1e8`)
            // is non-zero, over a tile `is_built_at`, asks
            // `find_any_building_at(t, who)` and refuses (4) only when the
            // building found is its own: another player's, and a tile
            // where nothing is found (`find_who` −1), are valid. The tile
            // grid passes it (`valid_tcoord`), so an armed walker's near
            // plan may cross another player's footprint at `calc_cost`'s
            // 4000 a tile, and those nodes carry `astar_path`'s building
            // flag into `resolve_block` ([`Sim::resolve_block`]). A
            // Citizen is armed here: its `attack` is 40.
            if enemy_builds_only
                && self.profile(crate::combat::Obj::Unit(u)).attack != 0
                && crate::world::is_built_at(mask)
            {
                let found = self
                    .find_any_building_at(t, owner)
                    .map(|b| self.buildings[b].owner);
                return if found == Some(owner) {
                    loc::BUILDING
                } else {
                    loc::VALID
                };
            }
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

    /// `PathFinder::valid_ucoord` — the 48-grid's probe, memoised in the
    /// pathfinder's `validlist`, [`Sim::path_memo`], which outlives the
    /// search that fills it (`docs/PATHFINDER.md` §24.5). Its second half is
    /// `detect_unit_collision` in its quick form, which is what makes the
    /// recovery path go **around** the units in the way rather than
    /// through them (`docs/COLLISION.md` §4.2).
    fn valid_ucoord(&mut self, u: usize, p: Pos, metric: i64) -> bool {
        let w = &self.world;
        if p.x < 0
            || p.y < 0
            || p.x >= w.width() * TILES_PER_CELL * 0xc0
            || p.y >= w.height() * TILES_PER_CELL * 0xc0
        {
            return false;
        }
        if let Some(&v) = self.path_memo.get(&metric) {
            return v;
        }
        let v = self.invalid_loc(u, p.tile(), false, true, false, true, false) == loc::VALID
            && !self.detect_quick(u, p, true)
            && !self.probe_refuses(u, p);
        self.path_memo.insert(metric, v);
        v
    }

    /// `PathFinder::kill_lists@00687ae0`'s one effect this crate carries:
    /// the validity memo emptied. Every finder calls it right after its
    /// `astar_*` returns — `find_upath@00682f30:330`, `find_wpath@00688fc0:261`,
    /// `find_tpath@006897d0:182`, `find_road@00688a40:46` — and on none of
    /// their early returns (§24.5).
    pub(crate) fn kill_lists(&mut self) {
        self.path_memo.clear();
        // The `blocklist` goes with it (`00687ae0`, the `+0x4c` loop).
        self.coll_copies.get_mut().clear();
    }

    /// Whether [`Sim::probe_refuse`] names this unit, this frame and the
    /// unit cell `p` lies in. False in every run but a test's.
    fn probe_refuses(&self, u: usize, p: Pos) -> bool {
        self.probe_refuse.as_ref().is_some_and(|r| {
            r.frame == self.frame
                && r.unit
                    == (
                        i32::from(self.units[u].owner),
                        i32::from(self.units[u].index),
                    )
                && crate::collide::ucell(p) == r.cell
        })
    }

    // `UnitData::needs_transport` is `Sim::needs_transport` in `transport.rs`
    // (`docs/TRANSPORT.md` §6).

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
                // The danger map again, and the tile grid takes it
                // **whole** — no shift (`006850a0`).
                let mut e = if m.no_danger {
                    0
                } else {
                    self.world.danger_half(who, to_cell)
                };
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
                // **The danger map, eighthed** (`00684fd2`): `danger[who]
                // [reg_xs × div3(y >> 9) + div3(x >> 9)] / 8`, truncated
                // toward zero, and it is the *first* term of `extra` — so
                // it is what the owner adjustment and the terrain cost are
                // added to, and what the `max(0)` at the end clamps.
                //
                // The sign is the point: around your own city the map is
                // negative, and this is 8 to 16 off every expensive step
                // there (`crate::danger`, `docs/DANGER.md` §5).
                let mut e = if m.no_danger {
                    0
                } else {
                    let d = self.world.danger_half(who, to_cell);
                    (d + ((d >> 31) & 7)) >> 3
                };
                if self.world.is_ocean(to_cell) {
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

    /// `astar_path`'s `avoid_land`/`avoid_sea`, off the start's terrain
    /// (§4.1). **The world grid does not ask `get_tregion`** — it reads
    /// `WData.region` (`+0x4`) of the two cells straight out of the array
    /// and compares the shorts (§4.1, "The same-region test is two
    /// functions"); only the tile and unit grids call `get_tregion`, and
    /// only they take the coastal refinement.
    ///
    /// Not asked again on a resume: the pair is stashed and restored.
    fn avoid_flags(&self, u: usize, start: Pos, goal: Pos, step: i32) -> (i32, i32) {
        let same_region = if step == STEP_WORLD {
            self.world.region_of(start.cell()) == self.world.region_of(goal.cell())
        } else {
            self.world.tregion_alt(start.tile()) == self.world.tregion_alt(goal.tile())
        };
        if same_region {
            let on_water = if step == STEP_WORLD {
                self.world.is_ocean(start.cell())
            } else {
                self.world.tile_mask(start.tile()) & tile::SURFACE == tile::SURFACE_OCEAN
            };
            // Computer-controlled transport types can cross either terrain
            // without the same-region preference (PATHFINDER §31).
            if self.ai_driven(self.units[u].owner)
                && self.units[u].ty.is_some_and(|t| {
                    self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::TRANSPORT != 0
                })
            {
                return (0, 0);
            }
            if on_water {
                return (1, 0);
            }
            let attacking = self
                .action_of(u)
                .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Attack(_)));
            return (0, if attacking { 2 } else { 1 });
        }
        if self.current_order(u).is_some_and(|o| o.flags & 0x20 != 0) {
            return (1, 0);
        }
        (0, 0)
    }

    /// `PathFinder::astar_path` (`docs/PATHFINDER.md` §4). Returns 1 on a
    /// path pushed, 0 on failure, −1 on a suspended unit-grid search.
    ///
    /// `resume` is the original's `PathFinder::saving` (`+0x84`): set, the
    /// whole prologue is skipped — **the stack is not popped** — and the
    /// state comes off the unit instead (§4.3 step 3). Everything the
    /// prologue derives from the *arguments* rather than from the stack is
    /// recomputed either way, exactly as `00683770` does: the work cap, the
    /// stride, the row width, the direction increment and `toff`.
    fn astar_path(&mut self, u: usize, m: &Modes, step: i32, anti: i32, resume: bool) -> i32 {
        let work_cap = if step == STEP_UNIT { 500 } else { 50 } * 64;
        // The recovery grid uses max(1, (collision size + 1) / 2).
        // Larger units also check the intermediate diagonal positions.
        // `docs/PATHFINDER.md` §30 (item 1431).
        let su: i32 = if step == STEP_UNIT {
            ((self.coll_size(u) + 1) / 2).max(1)
        } else {
            1
        };
        let stride = su * step;
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
        let no_danger = m.no_danger;
        let m = Modes { no_danger, ..*m };

        // **The resume.** `00683a2d` takes the five containers and the nine
        // scalars off the unit and jumps straight to the loop; the open
        // list is non-empty by construction, because the suspend re-inserted
        // the node it stopped on.
        #[allow(clippy::type_complexity)]
        let (mut nodes, mut open, mut open_by_metric, mut closed, mut seq): (
            Vec<Node>,
            BTreeMap<(i32, std::cmp::Reverse<u64>), u32>,
            BTreeMap<i64, (u64, i32, u32)>,
            BTreeMap<i64, u32>,
            u64,
        );
        let (tol, pref, goal, start_dist, traversed, avoid_land, avoid_sea);
        if resume {
            let Some(sus) = self.units[u].search.take() else {
                return 0;
            };
            let sus = *sus;
            nodes = sus.nodes;
            open = sus.open;
            open_by_metric = sus.open_by_metric;
            closed = sus.closed;
            // The resume closes the pathfinder's memo and takes the
            // unit's (`00683770:235`-`288`).
            self.path_memo = sus.valid_memo;
            *self.coll_copies.get_mut() = sus.block_copies;
            seq = sus.seq;
            tol = sus.tol;
            pref = sus.pref;
            goal = sus.goal;
            start_dist = sus.start_dist;
            traversed = sus.traversed;
            avoid_land = sus.avoid_land;
            avoid_sea = sus.avoid_sea;
        } else {
            let stack_len = self.units[u].path.len();
            if stack_len < 2 {
                // The wrappers always push start and goal; anything else is
                // a caller bug, answered the way the engine answers an empty
                // list.
                return 0;
            }
            let start_e = self.units[u].path.pop().expect("start entry");
            let goal_e = self.units[u].path.pop().expect("goal entry");
            let start;
            (start, goal) = (start_e.to, goal_e.to);
            // The arrival tolerance is the *final* goal's — the entry now on
            // top — not the search-goal entry's.
            tol = self.units[u].path.last().map_or(0, |p| p.tolerance);
            (avoid_land, avoid_sea) = self.avoid_flags(u, start, goal, step);
            nodes = Vec::new();
            // Min `value` first; equal values newest-first (LIFO), as the
            // original's BST leans (§2.1).
            open = BTreeMap::new();
            open_by_metric = BTreeMap::new();
            closed = BTreeMap::new();
            // **Not a fresh memo.** A fresh search starts on whatever the
            // pathfinder's holds: `astar_path@00683770`'s fresh arm resets
            // `valid_hit` (`+0x90`) and nothing else, so a goal pre-walk
            // that returned early without `kill_lists` hands its verdicts
            // on (§24.5).
            seq = 0;
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
            // The direction preference: the wheel starts one past this, at
            // the cardinal facing the goal.
            let (dx0, dy0) = (start.x - goal.x, start.y - goal.y);
            start_dist = dx0.abs() + dy0.abs();
            pref = if dy0.abs() < dx0.abs() {
                if goal.x < start.x { 7 } else { 3 }
            } else if start.y <= goal.y {
                5
            } else {
                1
            };
            traversed = 0;
        }
        let arrive = tol / 2 + stride;
        let mut probes: i32 = 0;

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
                // **Suspend** (§4.3 step 3): a unit-grid search over its
                // limit, still short of the goal, without `anti`. The node
                // it stopped on goes back on the open list — so the resumed
                // search expands it first and loses nothing — and the whole
                // state moves onto the unit. `find_upath` then returns −1
                // without touching the stack, which is left holding the
                // caller's final goal and nothing else.
                if over_limit && manh > arrive && anti == 0 {
                    open.insert(key, cur_id);
                    open_by_metric.insert(cur.metric, (key.1.0, cur.value, cur_id));
                    // The stamp goes on the **unit**, not into the stash:
                    // `astar_path@00683770:517` writes `UnitData +0x130`,
                    // which outlives the containers it hands over beside it
                    // (`docs/PATHFINDER.md` §18.6). Nothing but a unit's
                    // birth clears it, so this is the one field of the
                    // hand-over that survives `clear_partial_path`.
                    self.units[u].start_dist = start_dist;
                    self.units[u].search = Some(Box::new(Search {
                        nodes,
                        open,
                        open_by_metric,
                        closed,
                        // The suspend hands the memo to the unit and pops
                        // an empty one (`00683770:505`).
                        valid_memo: std::mem::take(&mut self.path_memo),
                        block_copies: std::mem::take(self.coll_copies.get_mut()),
                        seq,
                        tol,
                        pref,
                        goal,
                        start_dist,
                        traversed: traversed + probes,
                        avoid_land,
                        avoid_sea,
                    }));
                    return -1;
                }
                let mut end_id = cur_id;
                let mut partial = false;
                if traversed + probes >= work_cap && anti == 0 {
                    match step {
                        STEP_TILE if m.anti_unit => return 0,
                        STEP_UNIT => {
                            // **The work-cap tail**, `astar_path@00683770:
                            // 552`-`564`. The retry roll's only gate here is
                            // that the current order is a **transit** — the
                            // `attempts < 0xd` ceiling belongs to the
                            // *other* tail, below. Then `+0xb2 += 30`, which
                            // `detect_unit_collision` reads as "stop
                            // colliding for thirty frames"
                            // (`docs/COLLISION.md` §4.1).
                            //
                            // ~~SEAM: the pause roll happens only when the
                            // order's target is a unit~~ — that premise was
                            // never in the function and expired the moment
                            // a capture reached the tail (item 329).
                            self.roll_upath_retry(u, false);
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
                        if su > 1 && d & 1 != 0 {
                            (1..=su).all(|n| {
                                let dx = MOVE_X[d] * step * n;
                                let dy = MOVE_Y[d] * step * n;
                                // The original's diagonal memo uses position-unit
                                // offsets, unlike the endpoint's grid offsets
                                // (listing 00684350..00684371).
                                self.valid_ucoord(
                                    u,
                                    Pos::new(cur.x + dx, cur.y + dy),
                                    cur.metric + i64::from(dx) + i64::from(dy) * row,
                                )
                            })
                        } else {
                            self.valid_ucoord(u, p, metric)
                        }
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

        // **Open list exhausted: no path** — and, on the unit grid, the
        // tail that buys the unit a retry rather than killing its order
        // (`astar_path@00683770:919`-`971`, `docs/PATHFINDER.md` §21).
        // Run19's `1/28` on Great Lakes 8187 is the first capture to reach
        // it: the chase it was given on 8186 walks into `1/36`, the
        // 48-grid search finds nothing, and the roll here is what makes
        // `find_upath` spare the order (`Sim::upath_search`).
        if step == STEP_UNIT {
            self.roll_upath_retry(u, true);
            self.units[u].safe += 30;
        }
        0
    }

    /// The retry roll both unit-grid failure tails share
    /// (`docs/PATHFINDER.md` §21), and the one place their gates differ.
    ///
    /// `ceiling` is the open-list-exhausted tail's extra test — the move
    /// data's `attempts` under 13 (`astar_path@00683770:949`). The work-cap
    /// tail has only the move test, so it passes `false`: reading the
    /// two as one gate is the mistake this argument exists to make visible.
    ///
    /// **The move test is vslot `+0x14`, `is_move`, and nothing else**
    /// (`call *0x14` at `00684d5b` and on the work-cap tail; item 673,
    /// `docs/PATHFINDER.md` §21.6). Every move class answers it with the
    /// COMDAT fold of `return 1` (`StrafeOrder::is_air` at `+0x14` of
    /// `AttackToOrder`, `GroupAttackToOrder`, `GroupMoveOrder` and
    /// `MoveOrder`, `vtables.txt`), so the action bit is not read. Until
    /// item 673 this read [`crate::orders::Order::is_transit`], a move
    /// *without* the action bit, and an attack-move blocked on the unit
    /// grid lost its order where the original's buys a wait.
    ///
    /// The draw is spent **inside** the gates on both tails, so an order
    /// that is not a move — or one over the ceiling — costs the stream
    /// nothing.
    fn roll_upath_retry(&mut self, u: usize, ceiling: bool) {
        if !self
            .current_order(u)
            .is_some_and(crate::orders::Order::is_move)
        {
            return;
        }
        let Some(m) = self.current_move(u) else {
            return;
        };
        if ceiling && m.attempts >= 0xd {
            return;
        }
        self.mark(if ceiling {
            SITE_UPATH_RETRY
        } else {
            SITE_UPATH_RETRY_BUDGET
        });
        let r = self.rng.roll() % 3 + 6;
        if let Some(front) = self.units[u].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.retry = r;
        }
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
                flags |= path_flag::BLOCK;
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
        // Same cell, or a type that flies like a helicopter (`+0x2b4 &
        // 0x20`, `689110`, item 1048): push back, done.
        if here.cell() == gc || self.is_helicopter(u) {
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
        let boardless = !self.units[u].on_map || !self.unit_can_transport(u);
        // **A human's leader takes the other variant first** (`leaders &
        // 4`, `00689109`–`006892e4`; item 676, `docs/GOLDEN.md` §17): it
        // does not drag the goal back toward the start, it **pops**. While
        // the goal cell's centre tile is in another region from the start
        // cell's, or its centre half-cell (`2c + 1`) has never been seen,
        // the goal is dropped for the entry under it — stopping on a final
        // entry or an empty stack. So a human's unit whose next world
        // waypoint turns out to be water re-plans to the first waypoint
        // past it it may still stand on, or to its order's own goal, and
        // the search runs. Only a goal it **has** seen — or a sea unit —
        // then goes on to the AI's walk. run180's chariot does this on 693,
        // a cell short of the sand its first plan crossed unseen, and its
        // new plan goes round the lake to the destination.
        let human = self
            .nation
            .get(self.units[u].owner as usize)
            .is_some_and(|n| n.human);
        let sea = self.unit_domain_of(u) == crate::attrition::Domain::Sea;
        let mut ai_walk = true;
        if human {
            let owner = self.units[u].owner;
            let centre = |c: crate::world::Cell| {
                Pos::new(c.x * TILES_PER_CELL + 2, c.y * TILES_PER_CELL + 2)
            };
            let seen =
                |s: &Self, c: crate::world::Cell| s.was_seen_fog(2 * c.x + 1, 2 * c.y + 1, owner);
            if boardless {
                let start = self.world.tregion_alt(centre(here.cell()));
                loop {
                    let c = goal_e.to.cell();
                    if self.world.tregion_alt(centre(c)) == start && seen(self, c) {
                        break;
                    }
                    if goal_e.flags & crate::orders::path_flag::FINAL != 0
                        || self.units[u].path.is_empty()
                    {
                        break;
                    }
                    goal_e = self.units[u].path.pop().expect("a non-empty stack");
                    if !self.world.contains(goal_e.to.cell()) {
                        self.units[u].path.clear();
                        return -1;
                    }
                }
                goal = goal_e.to;
            }
            ai_walk = seen(self, goal.cell()) || sea;
        }
        let walks = ai_walk && self.unit_domain_of(u) != crate::attrition::Domain::Air && boardless;
        while walks && self.world.tregion_alt(goal.tile()) != self.world.tregion_alt(here.tile()) {
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
        // **Gaia is a leader and it is a computer one.** `self.nation` has
        // a row per *player*; the original's `leaders.list` has ten, and
        // run58's own dump prints owner 8 with `leader_flags 33554439` —
        // the same `0x2000007` the AI player carries, `& 4` set — against
        // the human's `0x800113`. So an owner with no row here is not
        // human, and a wandering animal takes the AI's mode block, which
        // is what it took in the original. Before this read panicked on
        // owner 8, 19,000 frames past East Indies' word.
        let human = self
            .nation
            .get(self.units[u].owner as usize)
            .is_some_and(|n| n.human);
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
        let r = self.astar_path(u, &modes, STEP_WORLD, 0, false);
        self.kill_lists();
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
        // The same tile, or a helicopter (`68990d`, item 1048).
        if ht == gt || self.is_helicopter(u) {
            goal_e.tolerance = 0;
            self.units[u].path.push(goal_e);
            return self.units[u].path.len() as i32;
        }
        // The pull-back walk on tiles.
        let mut goal = goal_e.to;
        loop {
            if self.world.tregion_alt(goal.tile()) == self.world.tregion_alt(here.tile())
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
        let r = self.astar_path(u, &modes, STEP_TILE, 0, false);
        self.kill_lists();
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
        let sq = self.repaths[self.units[u].owner as usize].pow(2).max(1);
        let limit = if anti { 500 / sq / 2 } else { 500 / sq };
        self.upath(u, anti, limit, false)
    }

    /// `PathFinder::find_upath_restore@00688f40` — the four-argument form
    /// that re-enters a suspended 48-grid search.
    ///
    /// It is `find_upath` with `saving = 1`, `anti = 0` and a **300**
    /// numerator instead of 500, and `saving` is what makes `find_upath`
    /// skip its whole pre-A\* block: no pop, no pull-back, no near test,
    /// no pushes. The stack is exactly as the suspend left it and the
    /// reconstruction pushes onto it.
    ///
    /// Its only caller is `do_move`'s suspended-search block
    /// (`docs/ORDERS.md` §4.4 step 2).
    pub fn find_upath_restore(&mut self, u: usize) -> i32 {
        let sq = self.repaths[self.units[u].owner as usize].pow(2).max(1);
        self.upath(u, false, 300 / sq, true)
    }

    /// `PathFinder::find_upath@00682f30`, the big form. `resume` is the
    /// global `saving`.
    fn upath(&mut self, u: usize, anti: bool, limit: i32, resume: bool) -> i32 {
        if resume {
            return self.upath_search(u, anti, limit, true);
        }
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
        // The pathfinder's memo, not a local one (§24.5): the pre-walk's
        // verdicts stay in it on every return below, none of which reaches
        // `kill_lists`, and a search — this one's or the next unit's —
        // reads them back.

        // **The pre-walk is gated, and the gate is not `find_wpath`'s**
        // (item 301). `00683095` tests the type's domain `+0x218 < 2` and
        // `UnitData::can_transport@0046f960` — a **conjunction with no
        // vfunc `+0x8` disjunct**, where `find_wpath@00688fc0:91` has
        // `domain < 2 && (vfunc+8 == 0 || !can_transport)`. So a land unit
        // whose side has a Dock — `unit_masks & 0x800000` without
        // `unit_masks2 & 0x2000`, which every citizen on an island map
        // carries — never pulls its 48-grid goal back at all: it goes
        // straight to the near test and the search.
        // **The pre-walk is gated, and the gate is not `find_wpath`'s**
        // (item 301, `docs/PATHFINDER.md` §18.1). `00683095` tests the
        // type's domain `+0x218 < 2` and `UnitData::can_transport@0046f960`
        // — a **conjunction with no vfunc `+0x8` disjunct**, where
        // `find_wpath@00688fc0:91` has `domain < 2 && (vfunc+8 == 0 ||
        // !can_transport)`. So a land unit whose side has a Dock —
        // `unit_masks & 0x800000` without `unit_masks2 & 0x2000`, which
        // every citizen on an island map carries — never pulls its 48-grid
        // goal back at all: it goes straight to the near test and the
        // search.
        let pulls_back =
            self.unit_domain_of(u) != crate::attrition::Domain::Air && !self.unit_can_transport(u);
        if pulls_back {
            loop {
                let gg = g48(goal);
                let metric = i64::from(gg.x) + i64::from(gg.y) * i64::from(self.world.width()) * 16;
                if self.world.tregion_alt(goal.tile()) == self.world.tregion_alt(here.tile())
                    && self.valid_ucoord(u, goal, metric)
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
        self.upath_search(u, anti, limit, false)
    }

    /// `find_upath`'s tail: `anti_unit` around the search, then the
    /// failure teardown and the success compaction. A **resume** skips the
    /// failure teardown entirely — `00683380`'s pop-and-kill block is
    /// inside the same `saving == 0` guard as the prologue, so a resumed
    /// search that gives up leaves the order alone.
    fn upath_search(&mut self, u: usize, anti: bool, limit: i32, resume: bool) -> i32 {
        let modes = Modes {
            anti_unit: true,
            limit,
            no_danger: self.no_danger_mode(u),
            ..Modes::default()
        };
        let r = self.astar_path(u, &modes, STEP_UNIT, i32::from(anti), resume);
        self.kill_lists();
        if r < 1 {
            if r == 0 && !resume {
                if self.units[u]
                    .path
                    .last()
                    .is_some_and(|p| p.flags & path_flag::FINAL == 0)
                {
                    self.units[u].path.pop();
                }
                // **The kill, and what spares it** — `find_upath@
                // 00682f30:187`-`195`: a current order that is a **move**
                // (vslot `+0x14`, `call *0x14` at `006833c9`; the action
                // bit is not read, item 673) whose move data carries a
                // non-zero `retry` is left alone, and the tail of
                // `astar_path` has just written that retry. So a failed
                // unit-grid search kills the order only when the roll did
                // not happen — an order that is not a move, or one over
                // the `attempts` ceiling ([`Sim::roll_upath_retry`]).
                //
                // Until item 329 the roll was a seam and the kill was
                // unconditional, which cost run19's `1/28` its whole chase
                // on Great Lakes 8187: the original stands it still for six
                // to eight frames with its 42-entry stack intact, and this
                // crate threw the stack away and went back to `fight`.
                let spared = self
                    .current_order(u)
                    .is_some_and(crate::orders::Order::is_move)
                    && self.current_move(u).is_some_and(|m| m.retry != 0);
                if !spared {
                    self.kill_current_order(u);
                }
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

    /// The `army` mode: a military unit, not a worker, **not attacking**, and
    /// not standing on a river cell (`docs/PATHFINDER.md` §3, §22, §29).
    ///
    /// **`is_attacking` is a term of it, and it is live for three order
    /// classes.** `UnitData::is_attacking@0060a5b0` tail-calls the current
    /// order's `+0x18` virtual, `is_attack`. §22 read that slot as a bare
    /// `return 0` in all seventeen order vtables it enumerated, and for
    /// those it is — but the census missed the classes whose `UnitOrder`
    /// vftable is a *secondary* one (`??_7AttackOrder@@6BUnitOrder@@@`
    /// `00b47628`, and `GroupAttackOrder`'s `00b491fc`). Their slot is the
    /// thunk `0047ef8e`, `sub ecx,[ecx-4]; jmp 0041e0e0`, and `0041e0e0`
    /// is `mov eax,1; ret`. The linker map names both ends:
    /// `?is_attack@AttackOrder@@UBEHXZ`, `@GroupAttackOrder@@` and
    /// `@StrafeOrder@@` at `0041e0e0`; `@UnitOrder@@`, `@AttackGroundOrder@@`
    /// and `@AirAttackGroundOrder@@` at `0041bff0`. So a unit whose current
    /// order is an `ATTACK` (or a `STRAFE`; `GroupAttackOrder` is not
    /// modelled) plans as a citizen. Reading the clause as "the unit has a
    /// combat target" was still wrong: an army under an `ATTACK_TO` walks as
    /// an army (§22). The case that shows it is a group's queued walk home
    /// planned while the leader's current order is the attack itself:
    /// Great Lakes 8186 and 17656 (§29).
    ///
    /// The river clause is the unit's **own** cell, not the search's
    /// start: the original reads `world.cells[unit.pos]` flags `& 0x100`
    /// ([`cell::HALFLAND`]) off the obfuscated position fields.
    ///
    /// **The first term has an `is_supply` arm** (item 569,
    /// `docs/PATHFINDER.md` §25): `type.attack == 0` still takes the mode
    /// when the unit's `is_supply` virtual answers yes (`00689680`–
    /// `006896a9`). Every unit vtable's `+0xcc` is the base
    /// `UnitData::is_supply@0046ce80`, `unit_flags2 & 0x40`, so the arm is
    /// the raw bit — without [`Sim::is_supply_unit`]'s hero exclusion,
    /// which no hero reaches here, being armed. A Supply Wagon plans as an
    /// army and pays `base << 5` for every `0x200` cell; golden chapter
    /// four's wagon `1/10` is the first `find_wpath` of one in the corpus.
    fn army_mode(&self, u: usize) -> bool {
        let armed = self.units[u].ty.is_some_and(|t| {
            let ty = &self.unit_types[t];
            ty.combat.attack > 0 || ty.cols.flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
        });
        let worker = self.units[u]
            .ty
            .is_some_and(|t| self.unit_types[t].worker != Worker::None);
        // `is_attacking`: the current order's `is_attack` (`006896b9`).
        let attacking = self
            .current_order(u)
            .is_some_and(|o| matches!(o.body, Body::Attack(_) | Body::Strafe(_)));
        let river = self.world.cell_data(self.units[u].pos.cell()).flags
            & crate::world::cell::HALFLAND
            != 0;
        armed && !worker && !attacking && !river
    }

    /// The `no_danger` mode (`PathFinderData +0x7c`), `astar_path@00683770:
    /// 119-127`: the danger map is ignored when the action is an attack
    /// (its `get_type` is `ATTACK`), when the owner is gaia (`who >= 8`),
    /// **or when the current order is `ATTACK_TO` or `GROUP_ATTACK_TO`**
    /// (`UnitData::order_type`). The last two arms were missing, so an
    /// army walking under an attack-to priced its own city's negative
    /// danger and cut through it (`docs/PATHFINDER.md` §28).
    fn no_danger_mode(&self, u: usize) -> bool {
        self.action_of(u)
            .is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Attack(_)))
            || self.units[u].owner >= 8
            || matches!(
                self.order_type(u),
                orders::index::ATTACK_TO | orders::index::GROUP_ATTACK_TO
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Cell, Terrain};
    use crate::{Tuning, Unit, World};

    /// `find_wpath`'s army mode takes an unarmed unit whose `is_supply`
    /// answers yes, `unit_flags2 & 0x40` (`docs/PATHFINDER.md` §25).
    /// Golden chapter four's Supply Wagon is the first in the corpus to
    /// plan a world path, and without the arm it cut through the `0x200`
    /// cells an army pays thirty-two times the base for.
    #[test]
    fn a_supply_wagon_plans_as_an_army_and_an_unarmed_plain_unit_does_not() {
        let mut sim = flat_sim(8);
        let wagon = sim.add_unit_type(crate::UnitType {
            cols: crate::ai_load::UnitCols {
                unit_flags2: crate::ai_load::uflags2::SUPPLY_OR_HERO,
                ..crate::ai_load::UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let plain = sim.add_unit_type(crate::UnitType::default());
        let u = walker(&mut sim, Pos::new(0x480, 0x480));
        sim.units[u].ty = Some(wagon);
        assert!(sim.army_mode(u), "attack 0, and still an army: is_supply");
        sim.units[u].ty = Some(plain);
        assert!(!sim.army_mode(u), "attack 0 and not supply: no army mode");
    }

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

    /// `astar_path@00683770:119-127`'s `no_danger`: an attack action, a
    /// gaia owner, **or a current order of `ATTACK_TO`/`GROUP_ATTACK_TO`**
    /// (`docs/PATHFINDER.md` §28). East Indies' army column on tick 17146
    /// walks under an `ATTACK_TO` and the original prices its world steps
    /// without who=1's danger; ours read it, and `1/58` cut north.
    #[test]
    fn an_attack_to_order_plans_without_the_danger_map() {
        for (kind, action, want) in [
            (MoveKind::AttackTo, true, true),
            (MoveKind::AttackTo, false, true),
            (MoveKind::MoveTo, true, false),
            (MoveKind::MoveTo, false, false),
        ] {
            let mut sim = flat_sim(12);
            let u = walker(&mut sim, Pos::new(0x180, 0x180));
            sim.add_move_order(
                u,
                Pos::new(0x180 + 6 * 0x300, 0x180),
                kind,
                crate::orders::QueuePos::New,
                action,
            );
            assert_eq!(
                sim.no_danger_mode(u),
                want,
                "{kind:?}, action {action}: no_danger"
            );
        }
        let mut sim = flat_sim(12);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        sim.units[u].owner = 8;
        assert!(sim.no_danger_mode(u), "gaia plans without danger");
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
    fn only_ai_transport_types_drop_same_region_terrain_preferences() {
        for water in [false, true] {
            for human in [false, true] {
                for transport in [false, true] {
                    let mut sim = flat_sim(4);
                    sim.nation[0].human = human;
                    if water {
                        sim.world
                            .fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(3, 3));
                    }
                    let start = Pos::new(384, 384);
                    let goal = Pos::new(1152, 384);
                    if water {
                        for p in [start, goal] {
                            sim.world
                                .set_tile_field(p.tile(), tile::SURFACE, tile::SURFACE_OCEAN);
                            let mut data = sim.world.cell_data(p.cell());
                            data.land = 1;
                            sim.world.set_cell_data(p.cell(), data);
                        }
                    }
                    let u = walker(&mut sim, start);
                    let ty = sim.add_unit_type(crate::UnitType::default());
                    sim.units[u].ty = Some(ty);
                    if transport {
                        sim.unit_types[ty].cols.unit_flags |= crate::ai_load::uflags::TRANSPORT;
                    }
                    let want = if !human && transport {
                        (0, 0)
                    } else if water {
                        (1, 0)
                    } else {
                        (0, 1)
                    };
                    for step in [STEP_WORLD, STEP_TILE, STEP_UNIT] {
                        assert_eq!(
                            sim.avoid_flags(u, start, goal, step),
                            want,
                            "water={water} human={human} transport={transport} step={step}"
                        );
                    }
                }
            }
        }
    }

    /// A large unit searches at its own stride and cannot jump a refused
    /// intermediate diagonal cell (PATHFINDER §30).
    #[test]
    fn large_recovery_steps_check_intermediate_diagonals() {
        for radius in [48, 144] {
            let mut sim = flat_sim(8);
            let start = Pos::new(1560, 1560);
            let u = walker(&mut sim, start);
            let ty = sim.add_unit_type(crate::UnitType {
                combat: crate::combat::Profile {
                    block_radius: radius,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            });
            sim.units[u].ty = Some(ty);
            sim.trace_costs = true;
            sim.probe_refuse = Some(RefuseProbe {
                frame: sim.frame,
                unit: (0, 1),
                cell: crate::collide::ucell(Pos::new(start.x + 48, start.y + 48)),
            });
            push_goal(&mut sim, u, Pos::new(start.x + 960, start.y + 960));
            assert!(sim.find_upath(u, false) > 0);
            let first: Vec<_> = sim
                .cost_marks
                .iter()
                .filter(|m| m.from == (start.x, start.y))
                .collect();
            assert!(!first.is_empty());
            let stride = if radius == 48 { 48 } else { 96 };
            assert!(
                first
                    .iter()
                    .all(|m| { (m.to.0 - start.x).abs().max((m.to.1 - start.y).abs()) == stride })
            );
            assert!(
                !first
                    .iter()
                    .any(|m| m.to == (start.x + stride, start.y + stride)),
                "the refused intermediate point blocks the diagonal"
            );
        }
    }

    /// **The suspend, and the resume that finishes what it started**
    /// (§4.3 step 3, §18, item 301). A 48-grid plan across a long stretch
    /// of open ground with the budget cut to a handful of probes stops
    /// short, hands its whole state to the unit, and returns −1 with the
    /// stack holding nothing but the caller's final goal. Resumed — over
    /// and over, exactly as `do_move`'s suspended-search block resumes it —
    /// it reaches the same goal it would have reached in one call.
    ///
    /// Made to fail on purpose twice: with the stash dropped instead of
    /// stored the resume answers 0 on the first call, and with the stop
    /// node **not** re-inserted into the open list the resumed plan is a
    /// different chain — 52 entries where the one call's compacts to 3,
    /// because the node the search stopped on is reached again through its
    /// neighbours instead of being expanded first.
    #[test]
    fn a_suspended_unit_search_is_resumed_where_it_stopped() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x18, 0x18));
        let goal = Pos::new(0x18 + 40 * 0x30, 0x18 + 40 * 0x30);

        // The whole plan, in one call, is the control.
        push_goal(&mut sim, u, goal);
        assert!(sim.find_upath(u, false) > 1, "the control plan");
        let whole = sim.units[u].path.clone();
        sim.units[u].path.clear();

        // Now the same plan on a budget of ten probes.
        push_goal(&mut sim, u, goal);
        assert_eq!(
            sim.upath(u, false, 10, false),
            -1,
            "a 48-grid plan over its limit and short of the goal suspends"
        );
        assert!(
            sim.units[u].search.is_some(),
            "the suspend hands the search to the unit"
        );
        assert_eq!(
            sim.units[u].path,
            vec![PathData {
                to: goal,
                tolerance: 0,
                flags: path_flag::FINAL
            }],
            "`astar_path` popped its own start and goal and pushed nothing: \
             the stack is the caller's final goal and nothing else"
        );
        let stashed = sim.units[u].search.as_ref().expect("the stash");
        assert_eq!(
            stashed.start_dist,
            (goal.x - 0x18).abs() + (goal.y - 0x18).abs(),
            "`UnitData::start_dist` is the start-to-goal Manhattan, and the \
             suspend is its only writer"
        );
        assert_eq!(stashed.goal, goal, "`endx`/`endy`");

        // Resume until it lands. Each resume is `find_upath_restore`'s own
        // call with the same small budget.
        let mut resumes = 0;
        let r = loop {
            resumes += 1;
            assert!(resumes < 400, "the resumed search never finished");
            let r = sim.upath(u, false, 10, true);
            if r != -1 {
                break r;
            }
        };
        assert!(resumes > 1, "one resume would not have tested anything");
        assert!(r > 1, "the resumed search lands a path, got {r}");
        assert!(
            sim.units[u].search.is_none(),
            "a finished search leaves no stash"
        );
        assert_eq!(
            sim.units[u].path, whole,
            "the resumed plan is the one call's plan, entry for entry"
        );

        // And `clear_partial_path` is what frees a stash that never
        // finishes — the original's `Unit::clear_partial_path@005e3920`.
        sim.units[u].path.clear();
        push_goal(&mut sim, u, goal);
        assert_eq!(sim.upath(u, false, 10, false), -1);
        sim.clear_partial_path(u);
        assert!(sim.units[u].search.is_none(), "the teardown frees it");
        assert_eq!(
            sim.upath(u, false, 10, true),
            0,
            "a resume with nothing stashed is a failure, not a panic"
        );
    }

    /// **A gatherer stuck on a suspended search gives its walk up**
    /// (`docs/COLLISION.md` §15, item 698). `do_move@005f7b30`'s suspended
    /// block, ahead of the blocker probe: when the action under the move is
    /// a `GATHER`, on the frame `(frame − collide_frame + 2) & 7 == 0`, and
    /// within `vector_dist < 0x120` of the move's own point, `avoid` takes
    /// that point, the gather order forgets its tile (`tx`/`ty`/`wait` −1,
    /// `goto_build` 1), and the move dies without a `collide` count.
    ///
    /// run178's `1/43` is the case: six frames after it met the standing
    /// `1/18`, 96 and 240 units short of its point, `vector_dist` 259.
    /// Each control changes one input: seven frames, 288 units short
    /// (`vector_dist` 304), and a plain walk with no gather under it. Made
    /// to fail on purpose with the arm taken out: the walk is kept.
    #[test]
    fn a_gatherer_on_a_suspended_search_gives_its_walk_up_near_its_point() {
        use crate::orders::{Body, MoveKind, QueuePos};
        let run = |elapsed: i64, short: i32, gathering: bool| {
            let mut sim = flat_sim(20);
            let at = Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18);
            let u = walker(&mut sim, at);
            let camp = sim.add_building(1, Pos::new(0x600, 0x600), 1);
            if gathering {
                sim.add_gather_order(u, camp, QueuePos::New, true);
                if let Some(Body::Gather(g)) = sim.units[u].orders.front_mut().map(|o| &mut o.body)
                {
                    g.tile = Some(Pos::new(30, 30));
                    g.wait = 408;
                    g.goto_build = false;
                }
            }
            let point = Pos::new(at.x + 96, at.y + short);
            sim.add_move_order(u, point, MoveKind::MoveTo, QueuePos::First, false);
            // The suspended 48-grid search, on a goal of its own.
            push_goal(&mut sim, u, Pos::new(at.x + 40 * 0x30, at.y + 40 * 0x30));
            assert_eq!(sim.upath(u, false, 10, false), -1, "the plan suspends");
            sim.units[u].collide = 6;
            sim.units[u].collide_frame = 1000;
            sim.work(u, 1000 + elapsed);
            let dest = match sim.units[u].orders.front().map(|o| o.body) {
                Some(Body::Move(m)) => Some(m.dest),
                _ => None,
            };
            (sim, u, dest)
        };

        let (sim, u, front) = run(6, 240, true);
        assert_eq!(front, None, "the walk is gone");
        let Some(Body::Gather(g)) = sim.units[u].orders.front().map(|o| o.body) else {
            panic!(
                "the gather order is the front order: {:?}",
                sim.units[u].orders
            );
        };
        assert_eq!(
            (g.tile, g.wait, g.goto_build),
            (None, -1, true),
            "the gather order forgets its tile"
        );
        assert_eq!(sim.units[u].collide, 6, "no count: the arm is above it");
        let (_, _, kept) = run(7, 240, true);
        assert_eq!(sim.units[u].avoid, kept, "`avoid` takes the move's point");

        for (elapsed, short, gathering, why) in [
            (7, 240, true, "seven frames"),
            (6, 288, true, "vector_dist 304"),
            (6, 240, false, "no gather under the walk"),
        ] {
            let (_, _, front) = run(elapsed, short, gathering);
            assert!(front.is_some(), "{why}: the walk is kept");
        }
    }

    /// **`kill_current_path` frees the stash, and an empty stack does
    /// not** (§18.4, item 304). `Unit::kill_current_path@005e31d0` pops
    /// the stack back through the current segment's final waypoint and
    /// then calls `clear_partial_path` — and the call sits *inside* the
    /// function's own `0 < length` guard, so a unit with nothing on its
    /// stack keeps whatever search it had.
    ///
    /// That guard is the whole difference between this and
    /// `kill_current_order`, which clears unconditionally, and it is why
    /// the assertion below is two-sided rather than one. The behaviour it
    /// buys is `Unit::ungroup_move_order@005fd140`'s: a follower dropped
    /// out of formation loses its suspended search on the same frame, and
    /// re-plans on the next one instead of standing in `do_move`'s
    /// suspended block for the rest of the capture.
    ///
    /// Made to fail on purpose both ways: without the `clear_partial_path`
    /// call the first assertion fails (and run76's `1/28` stops re-planning
    /// on 6861), and without the emptiness guard the second does.
    #[test]
    fn killing_the_current_path_frees_a_suspended_search() {
        let mut sim = flat_sim(20);
        let u = walker(&mut sim, Pos::new(0x18, 0x18));
        let goal = Pos::new(0x18 + 40 * 0x30, 0x18 + 40 * 0x30);

        push_goal(&mut sim, u, goal);
        assert_eq!(sim.upath(u, false, 10, false), -1, "the plan suspends");
        assert!(sim.units[u].search.is_some(), "the stash is on the unit");
        // The suspend leaves the caller's final goal on the stack, so the
        // pop runs and the clear with it.
        assert_eq!(sim.units[u].path.len(), 1);
        sim.kill_current_path(u);
        assert!(
            sim.units[u].path.is_empty() && sim.units[u].search.is_none(),
            "a non-empty stack: the segment goes and the stash goes with it"
        );

        // And the guard. Suspend a second search, empty the stack by hand,
        // and the teardown is a no-op on both.
        push_goal(&mut sim, u, goal);
        assert_eq!(sim.upath(u, false, 10, false), -1);
        sim.units[u].path.clear();
        sim.kill_current_path(u);
        assert!(
            sim.units[u].search.is_some(),
            "`0 < length` gates the clear: an empty stack frees nothing"
        );
    }

    /// **A walled-in attack-move buys the retry, and keeps its order**
    /// (item 673, `docs/PATHFINDER.md` §21.6). The gate on
    /// `astar_path`'s open-list-exhausted roll (`call *0x14` at
    /// `00684d5b`) and on `find_upath`'s reprieve (`006833c9`) is vslot
    /// `+0x14`, `is_move`, which every move class answers with `return 1`:
    /// the action bit is not read. So an `ATTACK_TO` with the action bit
    /// set fares exactly as a transit leg does: one roll, `retry` in
    /// `6..=8`, and the order stays. Against the old `is_transit` gate the
    /// action-bit arm lost its order here. That was Great Lakes' `1/27` on
    /// 12536, whose ungroup never came and so left its squad in formation.
    #[test]
    fn a_walled_in_attack_move_buys_the_retry_and_keeps_its_order() {
        for action in [true, false] {
            let mut sim = flat_sim(12);
            // A ring of blocked tiles two out from tile (5, 5).
            for d in -2..=2 {
                for t in [(5 + d, 3), (5 + d, 7), (3, 5 + d), (7, 5 + d)] {
                    let t = Pos::new(t.0, t.1);
                    sim.world
                        .set_tile_field(t, tile::SURFACE, tile::SURFACE_FOREST);
                    sim.world.set_tile_bits(t, tile::BLOCKED);
                }
            }
            let u = walker(&mut sim, Pos::new(5 * 0xc0 + 0x60, 5 * 0xc0 + 0x60));
            let goal = Pos::new(20 * 0xc0 + 0x60, 5 * 0xc0 + 0x60);
            sim.add_move_order(
                u,
                goal,
                MoveKind::AttackTo,
                crate::orders::QueuePos::New,
                action,
            );
            push_goal(&mut sim, u, goal);
            // A budget no ring this small can spend, so the open list is
            // exhausted rather than the search suspended.
            assert_eq!(
                sim.upath(u, false, 100_000, false),
                0,
                "the ring refuses the plan"
            );
            let m = sim
                .current_move(u)
                .unwrap_or_else(|| panic!("action {action}: the order is spared"));
            assert!(
                (6..=8).contains(&m.retry),
                "action {action}: `retry` is the roll, got {}",
                m.retry
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

    /// **The `army` mode's predicate, all four terms** (§22, §29).
    ///
    /// Made to fail on purpose: read `attacking` as "has a combat target"
    /// and the second case flips (it is the shape that cost Great Lakes
    /// 322 frames, §22.3); drop `!attacking` and the third flips (item
    /// 899: Great Lakes 8186's walk home and 17656's, §29); drop the
    /// `!river` term and the fifth flips; drop `!worker` and the sixth.
    /// The second and third have a value diff behind them, and the fifth
    /// and sixth rest on the listing alone.
    #[test]
    fn an_army_is_armed_unworked_and_off_the_river() {
        let mut sim = flat_sim(10);
        let archer = sim.add_unit_type(crate::UnitType {
            hits: 40,
            combat: crate::combat::Profile {
                attack: 5,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let citizen = sim.add_unit_type(crate::UnitType {
            hits: 40,
            worker: Worker::Citizen,
            combat: crate::combat::Profile {
                attack: 5,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(0x180, 0x180);
        let u = walker(&mut sim, at);
        sim.units[u].ty = Some(archer);
        assert!(sim.army_mode(u), "an armed non-worker on plain ground");

        // **`is_attacking` is not a term.** The original's clause calls
        // the current order's `+0x18` virtual, and that slot is a bare
        // `return 0` in every order vtable the executable ships (§22.1),
        // so a unit marching on a target is still an army — which is the
        // only kind of unit the mode exists for.
        sim.units[u].combat.target = Some(crate::combat::Obj::Unit(0));
        assert!(
            sim.army_mode(u),
            "a combat target is not what `is_attacking` tests, and nothing is"
        );
        sim.units[u].combat.target = None;

        // **`is_attacking` is the current order's `is_attack`** (§29):
        // `1` for an `AttackOrder` (and a `StrafeOrder`), `0` for every
        // move, an `ATTACK_TO` among them. Only the *current* order asks:
        // an attack queued behind a move leaves the unit an army.
        sim.add_move_order(
            u,
            Pos::new(0x480, 0x480),
            MoveKind::AttackTo,
            orders::QueuePos::New,
            true,
        );
        assert!(sim.army_mode(u), "an `ATTACK_TO` is a move, and an army");
        sim.add_attack_order(
            u,
            crate::combat::Obj::Unit(0),
            orders::QueuePos::Last,
            false,
            true,
        );
        assert!(
            sim.army_mode(u),
            "an attack queued behind the move is not the current order"
        );
        sim.add_attack_order(
            u,
            crate::combat::Obj::Unit(0),
            orders::QueuePos::New,
            false,
            true,
        );
        assert!(
            !sim.army_mode(u),
            "a unit whose current order is an `ATTACK` plans as a citizen"
        );
        sim.units[u].orders.clear();
        sim.units[u].combat.target = None;
        assert!(sim.army_mode(u), "and with no order it is an army again");

        // The river clause, on the unit's **own** cell.
        let mut d = sim.world.cell_data(at.cell());
        d.flags |= crate::world::cell::HALFLAND;
        sim.world.set_cell_data(at.cell(), d);
        assert!(!sim.army_mode(u), "a unit standing on a `0x100` cell");
        d.flags &= !crate::world::cell::HALFLAND;
        sim.world.set_cell_data(at.cell(), d);

        // And a worker is never an army, however well armed.
        sim.units[u].ty = Some(citizen);
        assert!(
            !sim.army_mode(u),
            "a worker with an attack is still a worker"
        );
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

    /// A lake of sea cells at `x` in `2..=4`, rows `2..=7`, on a ten-cell
    /// land map: region, `land` and every tile ocean, as run180's sand.
    fn lake_sim() -> Sim {
        let mut sim = flat_sim(10);
        let sea = sim.world.add_region(Terrain::Sea);
        for x in 2..=4 {
            for y in 2..=7 {
                let c = Cell::new(x, y);
                sim.world.set_region(c, sea);
                let mut d = sim.world.cell_data(c);
                d.land = 1;
                sim.world.set_cell_data(c, d);
                for ty in 0..TILES_PER_CELL {
                    for tx in 0..TILES_PER_CELL {
                        let t = Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty);
                        sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
                    }
                }
            }
        }
        // All dark: `seen2` a byte a half-cell, `2 × width` across.
        assert!(sim.world.set_fog(vec![0u8; 20 * 20]));
        sim
    }

    /// **`invalid_loc`'s fog arm** (item 676, `00607c9e`–`00607d4c`): with
    /// `fog_relax` set and a **human** leader, a tile whose cell's four
    /// fog half-cells are all unseen is valid before its terrain is read.
    /// Made to fail first on the seam it replaces, which refused the lake
    /// tile whatever the fog.
    #[test]
    fn a_human_s_probe_takes_an_unseen_cell_as_valid() {
        let mut sim = lake_sim();
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let t = Pos::new(3 * TILES_PER_CELL + 1, 4 * TILES_PER_CELL + 1);
        sim.nation[0].human = true;
        assert_eq!(
            sim.invalid_loc(u, t, true, true, false, true, false),
            loc::VALID
        );
        assert_eq!(
            sim.invalid_loc(u, t, true, false, false, true, false),
            loc::TERRAIN,
            "without fog_relax the lake refuses"
        );
        sim.nation[0].human = false;
        assert_eq!(
            sim.invalid_loc(u, t, true, true, false, true, false),
            loc::TERRAIN,
            "a computer leader's probe reads the terrain"
        );
        // One of the four half-cells seen is enough to read the terrain.
        sim.nation[0].human = true;
        let mut fog = vec![0u8; 20 * 20];
        fog[9 * 20 + 7] = 1; // (2·3 + 1, 2·4 + 1)
        assert!(sim.world.set_fog(fog));
        assert_eq!(
            sim.invalid_loc(u, t, true, true, false, true, false),
            loc::TERRAIN
        );
    }

    /// A walker of player 0 armed (`attack` 40, a Citizen's) or not, and a
    /// building of player 1 whose one footprint tile is blocked, `(10,
    /// 10)`. `add_building` leaves the type unset, so `covers_tile`
    /// answers the building's own tile alone.
    fn ring_tile(armed: bool) -> (Sim, usize, usize, Pos) {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let mut ty = crate::UnitType::default();
        ty.combat.domain = crate::attrition::Domain::Land;
        ty.combat.attack = if armed { 40 } else { 0 };
        let t = sim.add_unit_type(ty);
        sim.units[u].ty = Some(t);
        let tile_at = Pos::new(10, 10);
        sim.world
            .set_tile_mask(tile_at, tile::OBJECT_BUILDING | tile::BLOCKED);
        let b = sim.add_building(1, Pos::new(10 * 0xc0, 10 * 0xc0), 0);
        (sim, u, b, tile_at)
    }

    /// **`invalid_loc`'s armed arm** (item 1209, `00607fb6`..`0060800a`):
    /// under `param_5` a walker whose type is armed passes a blocked,
    /// built tile when the building found there is **not its own**, and
    /// when none is found; its own refuses, as does any building to an
    /// unarmed walker or without `param_5`. Made to fail first on the seam
    /// it replaces, which refused every blocked tile.
    #[test]
    fn an_armed_walker_passes_another_player_s_footprint_and_not_its_own() {
        let (mut sim, u, b, t) = ring_tile(true);
        let tile_search = |sim: &Sim| sim.invalid_loc(u, t, false, true, true, true, false);
        assert_eq!(tile_search(&sim), loc::VALID, "player 1's footprint");
        assert_eq!(
            sim.invalid_loc(u, t, false, true, false, true, false),
            loc::BUILDING,
            "without `param_5` every footprint refuses"
        );
        sim.buildings[b].owner = 0;
        assert_eq!(tile_search(&sim), loc::BUILDING, "its own refuses");
        sim.buildings[b].alive = false;
        assert_eq!(
            tile_search(&sim),
            loc::VALID,
            "nothing found: `find_who` −1 is not the walker's"
        );
        let (sim, u, _, t) = ring_tile(false);
        assert_eq!(
            sim.invalid_loc(u, t, false, true, true, true, false),
            loc::BUILDING,
            "an unarmed walker is refused whoever owns it"
        );
    }

    /// **`invalid_loc`'s cell arm** (item 776, `00607e6f`–`00607ead`): on
    /// the land domain, a tile whose **cell** carries mountain, forest or
    /// `0x40` (`WData.flags & 0x70`) refuses with 2 when the caller passes
    /// both `param_3` and `param_6` — which is `valid_wcoord`'s probe, and
    /// no other caller's unless a transport-flagged path top forces
    /// `param_6`. A forest-walker is let through a forest cell. The tile
    /// itself is plain ground here, so the tile arm below passes it: run226's
    /// `1/72` was planned through cell (56, 18), forest-flagged, where the
    /// original's world search refused it (`docs/PATHFINDER.md` §27).
    #[test]
    fn a_world_probe_refuses_a_forest_cell_on_plain_ground() {
        let mut sim = lake_sim();
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let c = Cell::new(7, 7);
        let mut d = sim.world.cell_data(c);
        d.flags |= crate::world::cell::FOREST;
        sim.world.set_cell_data(c, d);
        let t = Pos::new(7 * TILES_PER_CELL + 1, 7 * TILES_PER_CELL + 2);
        assert_eq!(sim.world.tile_mask(t) & tile::SURFACE, 0, "plain ground");
        assert_eq!(
            sim.invalid_loc(u, t, true, false, false, true, false),
            loc::TERRAIN,
            "`valid_wcoord`'s flags read the cell"
        );
        assert_eq!(
            sim.invalid_loc(u, t, false, false, false, true, false),
            loc::VALID,
            "without `param_3` the cell is not read"
        );
        assert_eq!(
            sim.invalid_loc(u, t, true, false, false, false, false),
            loc::VALID,
            "without `param_6` the cell is not read"
        );
        let mut d = sim.world.cell_data(c);
        d.flags = (d.flags & !crate::world::cell::FOREST) | crate::world::cell::MOUNTAIN;
        sim.world.set_cell_data(c, d);
        assert_eq!(
            sim.invalid_loc(u, t, true, false, false, true, false),
            loc::TERRAIN,
            "a mountain cell refuses the same way"
        );
    }

    /// **`find_wpath`'s human variant** (item 676, `00689109`–`006892e4`):
    /// a human's goal in another region, or unseen, is **popped** for the
    /// entry under it, down to the final one, and the search runs to it;
    /// a computer's is dragged back toward the start until its region
    /// matches, and returns without a search. run180's chariot, a cell
    /// short of the sand, is the first case. Made to fail first with the
    /// variant switched off, when the human's goal stopped on the shore.
    #[test]
    fn a_human_s_world_plan_pops_past_water_to_its_goal() {
        let run = |human: bool| {
            let mut sim = lake_sim();
            sim.nation[0].human = human;
            let u = walker(&mut sim, Pos::new(0x180, 4 * 0x300 + 0x180));
            let goal = Pos::new(8 * 0x300 + 0x180, 4 * 0x300 + 0x180);
            push_goal(&mut sim, u, goal);
            // A world waypoint past the lake, then one in it, on top.
            for x in [6, 3] {
                sim.units[u].path.push(PathData {
                    to: Pos::new(x * 0x300 + 0x180, 4 * 0x300 + 0x180),
                    tolerance: 0x180,
                    flags: 0,
                });
            }
            let r = sim.find_wpath(u);
            (r, sim.units[u].path.clone(), goal)
        };
        let (r, path, goal) = run(true);
        // Both waypoints popped, the goal kept, and a chain of cell centres
        // searched to it on top: more than the three entries it was given.
        assert!(r > 3 && path.len() > 3, "the human's search ran: {path:?}");
        assert_eq!(
            path.first().map(|p| p.to),
            Some(goal),
            "the human's plan ends on the order's own goal"
        );
        let (r, path, _) = run(false);
        let top = path.last().expect("a goal").to;
        assert_eq!(r, 3, "the computer's walk returns without a search");
        assert_eq!(top.cell().x, 1, "pulled back onto the shore: {top:?}");
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

    /// §16 — `calc_cost`'s water row is **`WorldData::is_ocean`**, the
    /// `WData` test, and not "the cell's region is a sea region". The two
    /// answers part on a `HALFLAND` cell, which is where a boat actually
    /// sails: the coastal cells of East Indies' channel were costing this
    /// crate the `avoid_land` 200 the original never charges.
    #[test]
    fn a_halfland_cell_in_a_sea_region_is_land_to_the_cost_function() {
        let mut sim = flat_sim(10);
        let u = walker(&mut sim, Pos::new(0x180, 0x180));
        let from = Pos::new(0x180, 0x180);
        let to = Pos::new(0x180 + 0x300, 0x180);
        let m = Modes::default();
        // A cell the region layer calls sea and `WData` calls land: ocean's
        // own `land` kind, and `HALFLAND` set.
        let sea = sim.world.add_region(Terrain::Sea);
        sim.world.set_region(to.cell(), sea);
        let mut d = sim.world.cell_data(to.cell());
        d.land = 1;
        d.flags |= crate::world::cell::HALFLAND;
        sim.world.set_cell_data(to.cell(), d);
        assert_eq!(sim.world.region_of(to.cell()), Some(sea));
        assert!(!sim.world.is_ocean(to.cell()), "HALFLAND is never ocean");
        // The halfland base is 0x300 × 32 / 256 = 96. `avoid_land` charges
        // its 200 on top; `avoid_sea` charges nothing, because the cell is
        // not ocean.
        assert_eq!(
            sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 1, 0).0,
            296
        );
        assert_eq!(sim.calc_cost(u, &m, from, to, 4, STEP_WORLD, 2, 0, 1).0, 96);
    }

    /// §16 — the world grid's same-region test reads `WData.region` raw.
    /// A boat standing on a `HALFLAND` cell is in the **land** region by
    /// that field and in the sea region by `get_tregion`, so the two rules
    /// disagree about whether it is crossing: `get_tregion` says "same
    /// region, and the start is not ocean" and hands the boat
    /// `avoid_sea = 1`, which prices every cell of its own sea at 232.
    #[test]
    fn a_boat_leaving_a_halfland_cell_is_crossing_regions_on_the_world_grid() {
        let mut sim = flat_sim(12);
        let sea = sim.world.add_region(Terrain::Sea);
        for x in 5..12 {
            for y in 0..12 {
                let c = Cell::new(x, y);
                sim.world.set_region(c, sea);
                let mut d = sim.world.cell_data(c);
                d.land = 1;
                sim.world.set_cell_data(c, d);
                for ty in 0..TILES_PER_CELL {
                    for tx in 0..TILES_PER_CELL {
                        let t = Pos::new(x * TILES_PER_CELL + tx, y * TILES_PER_CELL + ty);
                        sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
                    }
                }
            }
        }
        // The berth: a coastal cell whose `region` is the land it belongs
        // to and whose `region2` is the water beside it — and whose tiles
        // are water, so `get_tregion` answers the sea.
        let berth = Cell::new(4, 4);
        let mut d = sim.world.cell_data(berth);
        d.flags |= crate::world::cell::HALFLAND;
        d.region2 = Some(sea);
        sim.world.set_cell_data(berth, d);
        for ty in 0..TILES_PER_CELL {
            for tx in 0..TILES_PER_CELL {
                let t = Pos::new(berth.x * TILES_PER_CELL + tx, berth.y * TILES_PER_CELL + ty);
                sim.world.set_tile_mask(t, tile::SURFACE_OCEAN);
            }
        }
        assert_ne!(
            sim.world.region_of(berth),
            sim.world
                .tregion_alt(Pos::new(berth.x * TILES_PER_CELL, berth.y * TILES_PER_CELL)),
            "the berth is the cell the two rules disagree about"
        );

        let u = walker(
            &mut sim,
            Pos::new(berth.x * 0x300 + 0x180, berth.y * 0x300 + 0x180),
        );
        sim.units[u].kind.domain = crate::attrition::Domain::Sea;
        push_goal(&mut sim, u, Pos::new(10 * 0x300 + 0x180, 4 * 0x300 + 0x180));
        sim.trace_costs = true;
        assert!(sim.find_wpath(u) >= 1, "the boat found its way out");

        // Every step the search priced into open water cost the plain 32
        // (or 40 on a diagonal). With `avoid_sea` derived they would all
        // carry 200 more.
        let open: Vec<i32> = sim
            .cost_marks
            .iter()
            .filter(|c| {
                let cell = Pos::new(c.to.0, c.to.1).cell();
                cell.x >= 5 && sim.world.is_ocean(cell)
            })
            .map(|c| c.cost)
            .collect();
        assert!(!open.is_empty(), "the search priced some open water");
        assert!(
            open.iter().all(|&c| c == 32 || c == 40),
            "open water is free to a boat that is crossing regions: {open:?}"
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
