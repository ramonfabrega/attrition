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
//! (zero), the diplomacy penalty (none; the no-rush timer is built, item
//! 2423), transports (the unit cannot), and the per-type collision
//! *stride* (one; the size itself is loaded, `docs/COLLISION.md` §2). Unit collision on the 48-grid is no
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

use crate::Sim;
use crate::astar_view::{
    AstarView, MOVE_X, MOVE_Y, NoDangerView, Stash, SuspendView, Top, arrive, child_metric,
    child_pos, diag_probe, entry_steps, fresh_steps, suspend_steps, top_of_loop, wheel_dir,
    world_probe,
};
use crate::orders::{self, Body, PathData, path_flag};
use crate::world::{Owner, Pos, TILES_PER_CELL, cell, tile, vector_dist};

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
pub const STEP_WORLD: i32 = crate::ids::scale::UNITS_PER_CELL;
pub const STEP_TILE: i32 = crate::ids::scale::UNITS_PER_TILE;
pub const STEP_UNIT: i32 = crate::ids::scale::UNITS_PER_UCELL;

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
    pub const ON_BUILDING: i32 = 4;
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

/// What `astar_path@00683770:534-592` does with an exhausted work budget
/// (`traversed + probes >= 64 × the grid's cap`, and not an `anti` search).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapTail {
    /// The tile grid under `anti_unit`: give up, no path.
    Refuse,
    /// The unit grid: roll the retry, `+0xb2 += 30`, no path.
    Pause,
    /// **Every other grid** — the world's and the tile grid's without
    /// `anti_unit` — drains the open list for the node nearest the goal
    /// and ends on it (twenty-fourth pass, group 8; A4 row 31).
    Drain,
}

fn cap_tail(step: i32, anti_unit: bool) -> CapTail {
    match step {
        STEP_TILE if anti_unit => CapTail::Refuse,
        STEP_UNIT => CapTail::Pause,
        _ => CapTail::Drain,
    }
}

/// `PathFinder::valid_wcoord@00687da0`'s verdict on `invalid_loc`'s answer
/// `r`: valid, or a hazard in the goal's own cell. Item 2200's sweep holds
/// it against the original.
pub(crate) fn wcoord_verdict(r: i32, p: Pos, goal: Pos) -> bool {
    r == loc::VALID || (r == loc::HAZARD && p.cell() == goal.cell())
}

/// What `UnitData::invalid_loc@00607c30` reads of the unit, the tile and the
/// caller's flags (item 2400's sweep holds [`invalid_loc_steps`] to it).
pub(crate) struct InvalidLocIn {
    pub x: i32,
    pub y: i32,
    /// The world's width and height in tiles (`world+0x18`, `+0x1c`).
    pub width: i32,
    pub height: i32,
    /// The unit's leader (`this+9`).
    pub owner: i32,
    /// `leaders[owner].flags & 4`.
    pub human: bool,
    /// The path's last node carries the transport flag (`flags & 4`).
    pub path_top_transport: bool,
    /// The type's domain word (`type+0x218`): 0 land, 1 sea, 2 air.
    pub domain: i32,
    /// `type+0x1e8 != 0`.
    pub attack_nonzero: bool,
    /// `unit_masks2 & 0x4000`.
    pub forest_walker: bool,
    /// `unit_masks & 0x800000`.
    pub auto_transport: bool,
    pub ignore_buildings: bool,
    pub fog_relax: bool,
    pub enemy_builds_only: bool,
    pub transport_a: bool,
    pub transport_b: bool,
}

/// The callees and table reads of `invalid_loc`, each asked only when the
/// original asks it.
pub(crate) trait InvalidLocEnv {
    /// `WorldData::was_seen(fx, fy, owner)`.
    fn was_seen(&mut self, fx: i32, fy: i32) -> bool;
    /// The tile word at `(x, y)` (`world+0x138`).
    fn tile_mask(&mut self) -> u16;
    /// The cell word at `(x >> 2, y >> 2)` (`world+0x134`, stride `0x1c`).
    fn cell_flags(&mut self) -> u16;
    /// The tile word under the unit's own position.
    fn own_mask(&mut self) -> u16;
    /// `WorldData::is_cliff_at`.
    fn is_cliff(&mut self) -> bool;
    /// `UnitData::can_transport@0046f960`.
    fn can_transport(&mut self) -> bool;
    /// `is(AIRCRAFTCARRIER, 1)`.
    fn is_carrier(&mut self) -> bool;
    /// `WorldData::is_built_at`.
    fn is_built(&mut self) -> bool;
    /// `find_any_building_at`'s `find_who` (`objects+0x200`).
    fn building_who(&mut self) -> i32;
}

/// `UnitData::invalid_loc@00607c30` (the sim's [`Sim::invalid_loc`] is the
/// caller; the sweep is `sweep::unit_data_invalid_loc`).
pub(crate) fn invalid_loc_steps<E: InvalidLocEnv>(i: &InvalidLocIn, env: &mut E) -> i32 {
    // A transport-flagged path top relaxes the water test (`00607c49`).
    let ta = i.transport_a || i.path_top_transport;
    if i.x < 0 || i.y < 0 || i.x >= i.width || i.y >= i.height {
        return loc::OFF_MAP;
    }
    if i.fog_relax && i.human {
        let (fx, fy) = ((i.x >> 2) * 2, (i.y >> 2) * 2);
        let seen = [(0, 0), (0, 1), (1, 0), (1, 1)]
            .iter()
            .any(|&(dx, dy)| env.was_seen(fx + dx, fy + dy));
        if !seen {
            return loc::VALID;
        }
    }
    let fw = i.forest_walker;
    let relax = ta || i.transport_b;
    match i.domain {
        1 => {
            let m = env.tile_mask();
            if m & 0x30 == 0x20 {
                // The ocean side's hazard arm: a warship (`+0x1e8`, or a
                // carrier) over a `0x2400` tile (`00607df5`..`00607e57`).
                if (i.attack_nonzero || env.is_carrier()) && env.tile_mask() & 0x2400 != 0 {
                    return loc::HAZARD;
                }
            } else {
                if !relax || !env.can_transport() {
                    return loc::TERRAIN;
                }
                if env.cell_flags() & SEA_REFUSES != 0 {
                    return loc::TERRAIN;
                }
            }
        }
        0 => {
            let cf = env.cell_flags();
            if cf & SEA_REFUSES != 0 && (cf & 0x20 == 0 || !fw) && i.ignore_buildings && ta {
                return loc::TERRAIN;
            }
            let m = env.tile_mask();
            let surface = m & 0x30;
            if (surface == 0x30 || m & 3 == 2 || env.is_cliff()) && (surface != 0x30 || !fw) {
                return loc::TERRAIN;
            }
            if (!relax || !i.auto_transport) && surface == 0x20 {
                return loc::TERRAIN;
            }
        }
        2 => return loc::VALID,
        _ => {}
    }
    // The building check (`00607f50`..`00608017`).
    if !i.ignore_buildings {
        let m = env.tile_mask();
        if m & 0x4000 != 0 && (m & 0x30 != 0x30 || !fw) && env.own_mask() & 0x4000 == 0 {
            if i.attack_nonzero && i.enemy_builds_only && env.is_built() {
                return if env.building_who() == i.owner {
                    loc::ON_BUILDING
                } else {
                    loc::VALID
                };
            }
            return loc::ON_BUILDING;
        }
    }
    loc::VALID
}

struct SimLocEnv<'a> {
    sim: &'a Sim,
    u: usize,
    t: Pos,
}

impl InvalidLocEnv for SimLocEnv<'_> {
    fn was_seen(&mut self, fx: i32, fy: i32) -> bool {
        self.sim.was_seen_fog(fx, fy, self.sim.units[self.u].owner)
    }
    fn tile_mask(&mut self) -> u16 {
        self.sim.world.tile_mask(self.t)
    }
    fn cell_flags(&mut self) -> u16 {
        self.sim
            .world
            .cell_data(crate::world::World::cell_of_tile(self.t))
            .flags
    }
    fn own_mask(&mut self) -> u16 {
        self.sim.world.tile_mask(self.sim.units[self.u].pos.tile())
    }
    fn is_cliff(&mut self) -> bool {
        self.sim.world.tile_mask(self.t) & tile::OBJECT == tile::OBJECT_CLIFF
    }
    fn can_transport(&mut self) -> bool {
        self.sim.unit_can_transport(self.u)
    }
    fn is_carrier(&mut self) -> bool {
        self.sim.units[self.u]
            .ty
            .is_some_and(|ty| self.sim.is_aircraft_carrier(ty))
    }
    fn is_built(&mut self) -> bool {
        crate::world::is_built_at(self.sim.world.tile_mask(self.t))
    }
    fn building_who(&mut self) -> i32 {
        self.sim
            .find_any_building_at(self.t, self.sim.units[self.u].owner)
            .map_or(-1, |b| i32::from(self.sim.buildings[b].owner))
    }
}

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
        let unit = &self.units[u];
        let inp = InvalidLocIn {
            x: t.x,
            y: t.y,
            width: self.world.width() * TILES_PER_CELL,
            height: self.world.height() * TILES_PER_CELL,
            owner: i32::from(unit.owner),
            human: self
                .nation
                .get(unit.owner as usize)
                .is_some_and(|n| n.human),
            path_top_transport: unit
                .path
                .last()
                .is_some_and(|p| p.flags & path_flag::BOARDS_TRANSPORT != 0),
            domain: match self.unit_domain_of(u) {
                crate::attrition::Domain::Land => 0,
                crate::attrition::Domain::Sea => 1,
                crate::attrition::Domain::Air => 2,
            },
            attack_nonzero: self.profile(crate::combat::Obj::Unit(u)).attack != 0,
            // `unit_masks2 & 0x4000`, an Iroquois Scout's (item 3107).
            forest_walker: unit.unit_masks2 & crate::combat::umask2::FOREST_WALKER != 0,
            // The raw bit `unit_masks & 0x800000`, neither the veto nor the
            // type flag (`00607f19`).
            auto_transport: unit.auto_transport,
            ignore_buildings,
            fog_relax,
            enemy_builds_only,
            transport_a,
            transport_b,
        };
        let mut env = SimLocEnv { sim: self, u, t };
        invalid_loc_steps(&inp, &mut env)
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
    pub(crate) fn toff(&self, u: usize) -> Option<(i32, i32)> {
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
        wcoord_verdict(r, p, goal)
    }

    /// `PathFinder::valid_tcoord` — the tile grid's probe (the pre-walk's
    /// form; the search inlines the same flags).
    pub(crate) fn valid_tcoord(&self, u: usize, p: Pos) -> bool {
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
        if let Some(&v) = self.movement.path_memo.get(&metric) {
            return v;
        }
        let v = self.invalid_loc(u, p.tile(), false, true, false, true, false) == loc::VALID
            && !self.detect_quick(u, p, true)
            && !self.probe_refuses(u, p);
        self.movement.path_memo.insert(metric, v);
        v
    }

    /// `PathFinder::kill_lists@00687ae0`'s one effect this crate carries:
    /// the validity memo emptied. Every finder calls it right after its
    /// `astar_*` returns — `find_upath@00682f30:330`, `find_wpath@00688fc0:261`,
    /// `find_tpath@006897d0:182`, `find_road@00688a40:46` — and on none of
    /// their early returns (§24.5).
    pub(crate) fn kill_lists(&mut self) {
        self.movement.path_memo.clear();
        // The `blocklist` goes with it (`00687ae0`, the `+0x4c` loop).
        self.movement.coll_copies.get_mut().clear();
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
            // Fleeing triples the additive part. The predicate is
            // `UnitData::is_fleeing@0046efa0` (the current order's type is
            // `FLEE_TO`), not the order's `flags & 2`, which nothing writes
            // (`docs/ORDERS.md` §4; item 3246, `docs/GOLDEN.md` §134).
            if self.is_fleeing(u) {
                extra *= 3;
            }
            // **The no-rush timer** (`calc_cost@00684e50:411`-`420`, item
            // 2423, `docs/AI.md` §223): while the lobby's rush rules still
            // forbid war — [`Sim::war_allowed`] false, the same three
            // clauses — a cell another leader owns, and not an ally,
            // costs 500 more. After fleeing's triple and outside the
            // clamp, on the world-seen and tile branches only; an
            // unowned cell and the walker's own (`is_ally` answers 1 for
            // oneself) pay nothing.
            if !self.war_allowed()
                && let Owner::Player(p) = self.world.owner(to.cell())
                && !self.is_ally(who, p)
            {
                extra += 500;
            }
            // SEAM: the team-style diplomacy wall (+5000) reads game rules
            // the simulation does not carry.
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
            // From `PathFinder +0x58/+0x5c`: the wrapper's start argument,
            // which a group's plan makes something other than the unit's
            // own tile (group 7; A4 row 28).
            shore = self.needs_transport(self.movement.pf_start, to.tile());
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
        crate::astar_view::estimate(dx, dy, step)
    }

    /// `astar_path`'s `avoid_land`/`avoid_sea`, off the start's terrain
    /// (§4.1): [`crate::astar_view::avoid_steps`] over this unit. **The
    /// world grid does not ask `get_tregion`** — it reads `WData.region`
    /// (`+0x4`) of the two cells straight out of the array and compares the
    /// shorts (§4.1, "The same-region test is two functions"); only the tile
    /// and unit grids call `get_tregion`, and only they take the coastal
    /// refinement. Computer-controlled transport types can cross either
    /// terrain without the same-region preference (PATHFINDER §31).
    ///
    /// Not asked again on a resume: the pair is stashed and restored.
    #[cfg(test)]
    fn avoid_flags(&mut self, u: usize, start: Pos, goal: Pos, step: i32) -> (i32, i32) {
        crate::astar_view::avoid_steps(&mut SimAstar { sim: self, u }, step, start, goal)
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
        // **The entry** is `crate::astar_view::entry_steps` (item 3381,
        // `sweep::path_finder_astar_suspend`): the work cap, the stride
        // (`max(1, (collision size + 1) / 2)` steps on the unit grid, whose
        // larger units also check the intermediate diagonal positions,
        // `docs/PATHFINDER.md` §30, item 1431), the row width, the
        // direction increment, `toff` (zero when the current order is not a
        // move, which is exactly what the probe's `− 0x180` then means),
        // `no_danger` and `iroquois` — the last the unit's own
        // `unit_masks2 & 0x4000`, taken for every wrapper (§4.1; item 3117:
        // run918b's block 894, the Iroquois Scout `1/0`'s ten-leg explore
        // path, legs 6..8 one cell east in ours while it priced forest by
        // `blocked`).
        let e = entry_steps(&mut SimAstar { sim: self, u }, step, anti);
        let (work_cap, su, stride, toff, dinc) = (e.work_cap, e.su, e.stride, e.toff, e.dinc);
        let row = i64::from(e.row);
        let m = Modes {
            no_danger: e.no_danger,
            iroquois: m.iroquois || e.iroquois,
            ..*m
        };

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
            self.movement.path_memo = sus.valid_memo;
            *self.movement.coll_copies.get_mut() = sus.block_copies;
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
            // The two pops, the root node, the region tests and the wheel's
            // start are `crate::astar_view::fresh_steps`. The arrival
            // tolerance is the *final* goal's — the entry now on top — not
            // the search-goal entry's.
            let f = fresh_steps(&mut SimAstar { sim: self, u }, step);
            let start = f.start;
            goal = f.goal;
            tol = f.tol;
            (avoid_land, avoid_sea) = f.avoid;
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
                estimate: f.root_estimate,
                value: f.root_estimate,
                timeout: 0,
                metric: i64::from(f.root_metric),
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
            start_dist = f.start_dist;
            pref = f.pref;
            traversed = 0;
        }
        let arrive = arrive(tol, stride);
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
            let top = top_of_loop(
                manh,
                arrive,
                traversed,
                probes,
                work_cap,
                m.anti_unit,
                m.limit,
                anti,
            );
            if top != Top::Expand {
                // **Suspend** (§4.3 step 3): a unit-grid search over its
                // limit, still short of the goal, without `anti`. The node
                // it stopped on goes back on the open list — so the resumed
                // search expands it first and loses nothing — and the whole
                // state moves onto the unit. `find_upath` then returns −1
                // without touching the stack, which is left holding the
                // caller's final goal and nothing else.
                if top == Top::Suspend {
                    let mut sv = SimSuspend {
                        sim: self,
                        u,
                        nodes,
                        open,
                        open_by_metric,
                        closed,
                        seq,
                        key,
                        cur_id,
                        cur,
                    };
                    suspend_steps(
                        &mut sv,
                        &Stash {
                            tol,
                            pref,
                            goal,
                            start_dist,
                            traversed: traversed + probes,
                            avoid: (avoid_land, avoid_sea),
                        },
                    );
                    return -1;
                }
                let mut end_id = cur_id;
                let mut partial = false;
                if traversed + probes >= work_cap && anti == 0 {
                    match cap_tail(step, m.anti_unit) {
                        CapTail::Refuse => return 0,
                        CapTail::Pause => {
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
                        CapTail::Drain => {
                            // Drain the open list for the node nearest the
                            // goal; the partial path is the answer. **Every
                            // grid but the unit grid does** — the tile grid
                            // too, once its `anti_unit` arm above has gone
                            // (`astar_path@00683770:568-592`, twenty-fourth
                            // pass, group 8; A4 row 31).
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
                            // **The can-transport goal re-push**
                            // (`astar_path@00683770:586`-`613`, item 1948):
                            // when the node nearest the goal is still on the
                            // other side of the shore from it
                            // (`needs_transport`, tile coordinates) and the
                            // unit can take a transport, the stack's top —
                            // the goal the search was asked for — is popped
                            // and pushed back with `flags |= 4`, so the unit
                            // walks the partial path to it and casts there.
                            let best = nodes[end_id as usize];
                            if self.needs_transport(Pos::new(best.x, best.y).tile(), goal.tile())
                                != 0
                                && self.unit_can_transport(u)
                                && let Some(mut top) = self.units[u].path.pop()
                            {
                                top.flags |= path_flag::BOARDS_TRANSPORT;
                                self.units[u].path.push(top);
                            }
                        }
                    }
                }
                return self.reconstruct(u, &m, &mut nodes, end_id, step, partial);
            }

            // Expansion: the wheel from `pref + 1`.
            let mut k = pref + 1;
            let mut c = 1;
            while c < 9 {
                let d = wheel_dir(k);
                let p = child_pos(Pos::new(cur.x, cur.y), d, stride);
                let (nx, ny) = (p.x, p.y);
                let metric = child_metric(cur.metric, d, row);

                let valid = match step {
                    STEP_WORLD => {
                        // The first two steps probe `node + toff − 0x180`
                        // — the cell corner when the order is not a move,
                        // the order's own sub-cell offset when it is;
                        // later steps the node itself.
                        self.valid_wcoord(u, world_probe(cur.timeout, p, toff), cur.timeout, goal)
                    }
                    STEP_TILE => {
                        self.invalid_loc(u, p.tile(), false, true, true, true, false) == loc::VALID
                    }
                    _ => {
                        if su > 1 && d & 1 != 0 {
                            (1..=su).all(|n| {
                                // The original's diagonal memo uses position-unit
                                // offsets, unlike the endpoint's grid offsets
                                // (listing 00684350..00684371).
                                let (q, key) =
                                    diag_probe(Pos::new(cur.x, cur.y), cur.metric, d, step, row, n);
                                self.valid_ucoord(u, q, key)
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
                    self.tables.unit_types[t].cols.unit_flags
                        & crate::ai_load::uflags::FLAG_TRANSPORT
                        != 0
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
                flags |= path_flag::BOARDS_TRANSPORT;
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
        crate::wpath::find_wpath_steps(&mut crate::wpath::SimWpath::new(self, u, army_hint), here)
    }

    /// `find_wpath`'s search: `astar_path` on the world grid with the modes
    /// the planner set on the pathfinder (`pathfinder +0x94/+0x70/+0x78`); the
    /// caller runs `kill_lists` after reading the verdict (`68973d`-`68978e`).
    pub(crate) fn wpath_search(
        &mut self,
        u: usize,
        scouting: bool,
        army: bool,
        worker: bool,
    ) -> i32 {
        let modes = Modes {
            scouting,
            army,
            worker,
            no_danger: self.no_danger_mode(u),
            ..Modes::default()
        };
        self.astar_path(u, &modes, STEP_WORLD, 0, false)
    }

    /// `find_tpath`'s search: `astar_path` on the tile grid, then
    /// `kill_lists` (`689be0`–`689c47`).
    pub(crate) fn tpath_search(&mut self, u: usize) -> i32 {
        let modes = Modes {
            no_danger: self.no_danger_mode(u),
            ..Modes::default()
        };
        let r = self.astar_path(u, &modes, STEP_TILE, 0, false);
        self.kill_lists();
        r
    }

    /// `PathFinder::find_tpath` — the tile-grid planner
    /// (`find_tpath@006897d0`), as [`crate::tpath::find_tpath_steps`] over
    /// this unit (item 3337).
    pub(crate) fn find_tpath(&mut self, u: usize) -> i32 {
        let here = self.units[u].pos;
        crate::tpath::find_tpath_steps(&mut crate::tpath::SimTpath { sim: self, u }, here)
    }

    /// `PathFinder::find_upath` — the 48-grid planner, collision
    /// recovery's. `anti` halves the node limit and expands cardinals
    /// only. Its caller is `Sim::resolve_unit_collision`
    /// (`docs/COLLISION.md` §6 step 6).
    pub fn find_upath(&mut self, u: usize, anti: bool) -> i32 {
        let sq = self.movement.repaths[self.units[u].owner as usize]
            .pow(2)
            .max(1);
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
        let sq = self.movement.repaths[self.units[u].owner as usize]
            .pow(2)
            .max(1);
        self.upath(u, false, 300 / sq, true)
    }

    /// `PathFinder::find_upath@00682f30`, as
    /// [`crate::upath::find_upath_steps`] over this unit (item 3339).
    /// `resume` is the global `saving`.
    fn upath(&mut self, u: usize, anti: bool, limit: i32, resume: bool) -> i32 {
        let here = self.units[u].pos;
        let mut v = SimUpath {
            sim: self,
            u,
            limit,
            resume,
            keep: Vec::new(),
        };
        crate::upath::find_upath_steps(&mut v, here, i32::from(anti))
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
    /// the raw bit, as [`Sim::is_supply_unit`] is (item 3682 dropped its
    /// hero exclusion). A Supply Wagon plans as an
    /// army and pays `base << 5` for every `0x200` cell; golden chapter
    /// four's wagon `1/10` is the first `find_wpath` of one in the corpus.
    #[cfg(test)]
    fn army_mode(&self, u: usize) -> bool {
        let armed = self.units[u].ty.is_some_and(|t| {
            let ty = &self.tables.unit_types[t];
            ty.combat.attack > 0 || ty.cols.flag2(crate::ai_load::uflags2::SUPPLY_OR_HERO)
        });
        let worker = self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].worker != orders::Worker::None);
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
        crate::astar_view::no_danger_steps(&mut SimNoDanger { sim: self, u })
    }
}

/// The crate's [`Sim`] as `no_danger`'s view: unit `u`'s action, owner and
/// current order.
struct SimNoDanger<'a> {
    sim: &'a Sim,
    u: usize,
}

impl NoDangerView for SimNoDanger<'_> {
    fn action_type(&mut self) -> Option<i32> {
        self.sim
            .action_of(self.u)
            .map(|a| i32::from(self.sim.units[self.u].orders[a].index()))
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.sim.units[self.u].owner)
    }
    fn order_type(&mut self) -> i32 {
        i32::from(self.sim.order_type(self.u))
    }
}

/// The crate's [`Sim`] as `astar_path`'s view (item 3381,
/// [`crate::astar_view`]): unit `u`, its orders, the world and its path
/// stack. The pathfinder's own words (`+0x74`, `+0x7c`, `+0x88`, `+0x8c`,
/// `+0x90`) and the root node's trees have no field here — the loop keeps
/// them in locals and containers — so their setters are empty.
struct SimAstar<'a> {
    sim: &'a mut Sim,
    u: usize,
}

impl NoDangerView for SimAstar<'_> {
    fn action_type(&mut self) -> Option<i32> {
        SimNoDanger {
            sim: self.sim,
            u: self.u,
        }
        .action_type()
    }
    fn owner(&mut self) -> i32 {
        i32::from(self.sim.units[self.u].owner)
    }
    fn order_type(&mut self) -> i32 {
        i32::from(self.sim.order_type(self.u))
    }
}

impl AstarView for SimAstar<'_> {
    fn masks2(&mut self) -> i32 {
        self.sim.units[self.u].unit_masks2 as i32
    }
    fn collision(&mut self) -> i32 {
        self.sim.coll_size(self.u)
    }
    fn transport_type(&mut self) -> bool {
        self.sim.units[self.u].ty.is_some_and(|t| {
            self.sim.tables.unit_types[t].cols.unit_flags & crate::ai_load::uflags::FLAG_TRANSPORT
                != 0
        })
    }
    fn ai_bit(&mut self) -> bool {
        self.sim.unit_ai_bit(self.sim.units[self.u].owner)
    }
    fn list_present(&mut self) -> bool {
        !self.sim.units[self.u].orders.is_empty()
    }
    fn cursor_to_front(&mut self) {}
    fn has_order(&mut self) -> bool {
        self.sim.current_order(self.u).is_some()
    }
    fn order_is_move(&mut self) -> bool {
        self.sim
            .current_order(self.u)
            .is_some_and(orders::Order::is_move)
    }
    fn move_offset(&mut self) -> (i32, i32) {
        self.sim.toff(self.u).unwrap_or((0, 0))
    }
    fn order_present(&mut self) -> bool {
        self.sim.current_order(self.u).is_some()
    }
    fn order_flags(&mut self) -> i32 {
        self.sim
            .current_order(self.u)
            .map_or(0, |o| i32::from(o.flags))
    }
    fn world_width(&mut self) -> i32 {
        self.sim.world.width()
    }
    fn world_width_tiles(&mut self) -> i32 {
        self.sim.world.width() * TILES_PER_CELL
    }
    fn div3(&mut self, i: i32) -> i32 {
        i.div_euclid(3)
    }
    fn region_of_cell(&mut self, cx: i32, cy: i32) -> i32 {
        self.sim
            .world
            .region_of(crate::world::Cell::new(cx, cy))
            .map_or(-1, i32::from)
    }
    fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(tx, ty))
            .map_or(-1, i32::from)
    }
    fn is_ocean_cell(&mut self, cx: i32, cy: i32) -> bool {
        self.sim.world.is_ocean(crate::world::Cell::new(cx, cy))
    }
    fn tile_mask(&mut self, tx: i32, ty: i32) -> i32 {
        i32::from(self.sim.world.tile_mask(Pos::new(tx, ty)))
    }
    fn pop_path(&mut self) -> PathData {
        // The caller has seen two entries.
        self.sim.units[self.u].path.pop().expect("a path entry")
    }
    fn top_tolerance(&mut self) -> i32 {
        self.sim.units[self.u]
            .path
            .last()
            .map_or(0, |p| p.tolerance)
    }
    fn set_iroquois(&mut self, _word: i32) {}
    fn set_no_danger(&mut self, _on: bool) {}
    fn reset_valid_hit(&mut self) {}
    fn seed_open(&mut self, _start: Pos, _metric: i32, _estimate: i32) {}
    fn set_avoid(&mut self, _land: i32, _sea: i32) {}
}

/// The crate's [`Sim`] as the suspend's view: the search's containers, which
/// the hand-over moves onto the unit as a [`Search`].
struct SimSuspend<'a> {
    sim: &'a mut Sim,
    u: usize,
    nodes: Vec<Node>,
    open: BTreeMap<(i32, std::cmp::Reverse<u64>), u32>,
    open_by_metric: BTreeMap<i64, (u64, i32, u32)>,
    closed: BTreeMap<i64, u32>,
    seq: u64,
    key: (i32, std::cmp::Reverse<u64>),
    cur_id: u32,
    cur: Node,
}

impl SuspendView for SimSuspend<'_> {
    fn reinsert_current(&mut self) {
        self.open.insert(self.key, self.cur_id);
        self.open_by_metric
            .insert(self.cur.metric, (self.key.1.0, self.cur.value, self.cur_id));
    }
    fn hand_over(&mut self, st: &Stash) {
        // The stamp goes on the **unit**, not into the stash:
        // `astar_path@00683770:517` writes `UnitData +0x130`, which
        // outlives the containers it hands over beside it
        // (`docs/PATHFINDER.md` §18.6). Nothing but a unit's birth clears
        // it, so this is the one field of the hand-over that survives
        // `clear_partial_path`.
        self.sim.units[self.u].start_dist = st.start_dist;
        let search = Search {
            nodes: std::mem::take(&mut self.nodes),
            open: std::mem::take(&mut self.open),
            open_by_metric: std::mem::take(&mut self.open_by_metric),
            closed: std::mem::take(&mut self.closed),
            // The suspend hands the memo to the unit and pops an empty one
            // (`00683770:505`).
            valid_memo: std::mem::take(&mut self.sim.movement.path_memo),
            block_copies: std::mem::take(self.sim.movement.coll_copies.get_mut()),
            seq: self.seq,
            tol: st.tol,
            pref: st.pref,
            goal: st.goal,
            start_dist: st.start_dist,
            traversed: st.traversed,
            avoid_land: st.avoid.0,
            avoid_sea: st.avoid.1,
        };
        self.sim.units[self.u].search = Some(Box::new(search));
    }
}

/// The crate's [`Sim`] as `find_upath`'s view: unit `u`, the node `limit` the
/// caller's repath count set, `resume` the global `saving`, and the
/// function-static `keep` stack.
struct SimUpath<'a> {
    sim: &'a mut Sim,
    u: usize,
    limit: i32,
    resume: bool,
    keep: Vec<PathData>,
}

impl crate::upath::UpathView for SimUpath<'_> {
    fn begin(&mut self, here: Pos) {
        // `pathfinder +0x58/+0x5c`; the unit pointer and the zeroed
        // `+0x68/+0x6c/+0x94` have no field of their own here.
        self.sim.movement.pf_start = here.tile();
    }
    fn saving(&self) -> bool {
        self.resume
    }
    fn cell48(&self, c: i32) -> i32 {
        c.div_euclid(0x30)
    }
    fn world_dims(&self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn pop(&mut self) -> Option<PathData> {
        self.sim.units[self.u].path.pop()
    }
    fn peek(&mut self) -> PathData {
        *self.sim.units[self.u].path.last().expect("a peeked entry")
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
    fn domain_below_2(&self) -> bool {
        self.sim.unit_domain_of(self.u) != crate::attrition::Domain::Air
    }
    fn can_transport(&mut self) -> bool {
        self.sim.unit_can_transport(self.u)
    }
    fn tregion(&mut self, cx4: i32, cy4: i32) -> i32 {
        self.sim
            .world
            .tregion_alt(Pos::new(cx4, cy4))
            .map_or(-1, i32::from)
    }
    fn valid_ucoord(&mut self, p: Pos, metric: i32) -> bool {
        self.sim.valid_ucoord(self.u, p, i64::from(metric))
    }
    fn search(&mut self, anti: i32) -> i32 {
        let modes = Modes {
            anti_unit: true,
            limit: self.limit,
            no_danger: self.sim.no_danger_mode(self.u),
            ..Modes::default()
        };
        self.sim
            .astar_path(self.u, &modes, STEP_UNIT, anti, self.resume)
    }
    /// **The suspended flag, not a resume flag**: `astar_path` writes 1 on
    /// its suspend and 0 on every failure tail before it returns (`:535`,
    /// `:908`), `find_upath_restore` merely starts it at 1. So a resumed
    /// search that gives up pops its non-final top and kills the order
    /// unless the retry roll spared it, exactly as a fresh one does (item
    /// 1982: the AA missile `1/107` on 3180).
    fn suspended(&self, r: i32) -> bool {
        r < 0
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
    /// A current order that is a **move** (vslot `+0x14`, `call *0x14` at
    /// `006833c9`; the action bit is not read, item 673).
    fn order_is_move(&mut self) -> bool {
        self.sim
            .current_order(self.u)
            .is_some_and(crate::orders::Order::is_move)
    }
    /// The move data's `retry`, which the tail of `astar_path` has just
    /// written: a failed unit-grid search kills the order only when the roll
    /// did not happen (`Sim::roll_upath_retry`; item 329, Great Lakes 8187).
    fn move_retry(&mut self) -> i32 {
        self.sim.current_move(self.u).map_or(0, |m| m.retry)
    }
    fn kill_current_order(&mut self) {
        self.sim.kill_current_order(self.u);
    }
    fn kill_lists(&mut self) {
        self.sim.kill_lists();
    }
    fn unit_pos(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn keep_begin(&mut self) {
        self.keep.clear();
    }
    fn keep_push(&mut self, e: PathData) {
        self.keep.push(e);
    }
    fn keep_pop(&mut self) -> Option<PathData> {
        self.keep.pop()
    }
}

#[cfg(test)]
mod tests;
