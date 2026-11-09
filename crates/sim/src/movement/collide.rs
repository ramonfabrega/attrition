//! Unit collision: the occupancy index, the probe, and what a blocked unit
//! does about it. `docs/COLLISION.md`.
//!
//! Two indices carry the whole mechanic. [`CollGrid`] is the original's
//! `CollBlock` bitmask — one bit per 48-unit cell, set for every cell a
//! unit's block covers — flattened from a per-world-cell allocation into
//! one bitset over the map. The object chain is `Unit::down` threaded
//! through a per-world-cell head, which is how `detect_unit_collision`
//! turns "something is there" into "that unit is there".
//!
//! The bits are deliberately **not refcounted**: `CollCheck::move_unit`
//! clears the cells a unit leaves without asking whether anybody else is
//! standing on them, so two overlapping blocks lose bits when one moves.
//! An index rebuilt from the unit list would answer differently, and for
//! `coll_size 1` — every unit in every capture so far — the blocks overlap
//! most of the time.

use crate::attrition::Domain;
use crate::combat::Obj;
use crate::detect_collision_view::{DetectArgs, DetectCollisionView, detect_unit_collision_steps};
use crate::movement::{Angle, cos_component, find_angle, sin_component};
use crate::orders::{Body, MoveOrder, Order, PathData, QueuePos, index, path_flag};
use crate::world::{Cell, Pos, UNITS_PER_CELL, World, tile, vector_dist};

/// The distance `Unit::detect_boat_collision` measures between two circle
/// centres (`5fabf3`–`5fac63`): the longer leg plus the shorter one's
/// square over twice the longer, in unsigned arithmetic — and, once the
/// shorter leg reaches 60,000, their mean with the longer counted twice.
/// Zero when the longer leg is.
fn boat_dist(dx: i32, dy: i32) -> i32 {
    let (a, b) = (dx.unsigned_abs(), dy.unsigned_abs());
    let (long, short) = if a > b { (a, b) } else { (b, a) };
    if long == 0 {
        0
    } else if short >= 60_000 {
        ((short + 2 * long) >> 1) as i32
    } else {
        (short * short / (2 * long) + long) as i32
    }
}
use crate::{Player, Sim};

/// What `CollCheck::fill_slots@006820e0` leaves a probe able to read
/// (`docs/COLLISION.md` §4.2): the world cells of the probe's 2×2 slots
/// whose `WData::region` is the probe centre's own `get_tregion`, or all of
/// them when that is none. A slot the gate refuses reads as empty.
struct ProbeSlots {
    /// The probe centre's `get_tregion`; `None` reads every world cell.
    region: Option<u16>,
    /// A `nocoll` probe's slots, each the block it reads: the pathfinder's
    /// copy when the tree held one, and otherwise the gated live block it
    /// has just copied ([`Sim::coll_copies`]). Empty for a probe that
    /// reads the live blocks.
    copied: Vec<((i32, i32), [u16; 16])>,
}

impl ProbeSlots {
    /// The slot holding the unit cell `p` gave the probe a block at all.
    fn readable(&self, sim: &Sim, p: Pos) -> bool {
        let c = Cell::new(
            p.x.div_euclid(UCELLS_PER_CELL),
            p.y.div_euclid(UCELLS_PER_CELL),
        );
        self.region
            .is_none_or(|r| sim.world.region_of(c) == Some(r))
    }

    /// The copy the probe reads for the world cell holding `p`, if it
    /// reads one.
    fn copy(&self, p: Pos) -> Option<&[u16; 16]> {
        let c = (
            p.x.div_euclid(UCELLS_PER_CELL),
            p.y.div_euclid(UCELLS_PER_CELL),
        );
        self.copied.iter().find(|(k, _)| *k == c).map(|(_, b)| b)
    }

    fn get(&self, sim: &Sim, p: Pos) -> bool {
        if let Some(b) = self.copy(p) {
            let (dx, dy) = (
                p.x.rem_euclid(UCELLS_PER_CELL),
                p.y.rem_euclid(UCELLS_PER_CELL),
            );
            return b[dy as usize] >> dx & 1 != 0;
        }
        self.readable(sim, p) && sim.coll.get(p.x, p.y)
    }

    /// `local_30[slot] == 0`: the slot has a block, the region gate let
    /// it through, and `BitMask<768>::empty` says it holds a bit.
    fn live(&self, sim: &Sim, p: Pos) -> bool {
        if let Some(b) = self.copy(p) {
            return b.iter().any(|&r| r != 0);
        }
        self.readable(sim, p)
            && sim.coll.any_in_cell(
                p.x.div_euclid(UCELLS_PER_CELL),
                p.y.div_euclid(UCELLS_PER_CELL),
            )
    }
}

/// A unit cell, `0x30` position units — the grid the index is keyed on.
pub const UNITS_PER_UCELL: i32 = 0x30;
/// Unit cells to a world cell, each way.
pub const UCELLS_PER_CELL: i32 = UNITS_PER_CELL / UNITS_PER_UCELL;
/// `Constants::unit_block_radius` — the scale `BLOCK_RADIUS` is loaded at.
const UNIT_BLOCK_RADIUS: i32 = 48;
/// The tile bits the path unwind refuses: blocked, or next to blocked.
const UNWIND_REFUSES: u16 = tile::BLOCKED | tile::BAD_PATH;

/// One unit the §4.3 scan reached, and what it made of it — a row of
/// [`SweepVerdict`], and the shape `UnitData::is_here@0060a0c0` and
/// `UnitData::is_corner@0060a040` are proxied as in a
/// `RON_COLLIDE_PROBE` build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The candidate's player.
    pub who: i32,
    /// Its object number.
    pub o: i32,
    /// `UnitData::is_here`: does its block cover the hit cell? The three
    /// fields below are asked only when it does, as the original asks
    /// them.
    pub is_here: bool,
    /// §4.3's exemption ladder said this is a nudge, not a collision.
    pub soft: bool,
    /// `UnitData::is_corner` on the hit cell — `Some(0)` when no figure
    /// of this unit puts the cell on a diagonal corner of its block, and
    /// **`None` when it was never asked**, because an earlier gate
    /// answered first.
    ///
    /// The two are not the same fact and the distinction is the one
    /// `docs/COLLISION.md` §9 turns on: `is_corner` is reached only when
    /// `will_be_corner` was non-zero *and* every soft arm declined, so
    /// its absence says as much as its answer.
    pub is_corner: Option<i32>,
}

/// What one sweep of one proposed point decided, step by step
/// (`docs/COLLISION.md` §9). [`Sim::sweep_verdict`] is what fills it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SweepVerdict {
    /// §4.1's gates let the test run at all, and the proposal is not the
    /// unit's own cell.
    pub gated: bool,
    /// The proposal's unit cell.
    pub at_cell: Pos,
    /// `CollCheck::collide_here`'s answer: the first cell of the sweep
    /// that is occupied, or `None`.
    pub hit: Option<Pos>,
    /// `UnitData::will_be_corner` measured on the hit cell against the
    /// **proposed** cell — the asking unit's half of the corner rule.
    pub will_be_corner: i32,
    /// Every unit the 3×3 walk reached, in the order it reached them.
    pub candidates: Vec<Candidate>,
    /// A soft collision was found, so the next step is halved.
    pub soft: bool,
    /// The hard collision, `(who, o)` — the pair the original would write
    /// into `collide_who`/`collide_o` and the snap arm would then clear.
    pub hard: Option<(i32, i32)>,
}

/// What [`Sim::sweep_watch`] collects: every sweep one named unit ran on
/// one named frame, recorded where the original's own bracket sits.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SweepWatch {
    /// The sim-frame to record on.
    pub frame: i64,
    /// The asking unit, `(who, o)`.
    pub unit: (i32, i32),
    /// Each proposed point and what the sweep made of it, in order.
    pub seen: Vec<(Pos, SweepVerdict)>,
}

impl SweepWatch {
    /// Watch one unit on one frame.
    pub fn new(frame: i64, who: i32, o: i32) -> SweepWatch {
        SweepWatch {
            frame,
            unit: (who, o),
            seen: Vec::new(),
        }
    }

    /// Each recorded sweep as `"(x,y) <verdict>"` — the line the run116
    /// comparison diffs against the trace's own bracket.
    pub fn rendered(&self) -> Vec<String> {
        self.seen
            .iter()
            .map(|(at, v)| format!("({},{}) {v}", at.x, at.y))
            .collect()
    }
}

impl std::fmt::Display for Candidate {
    /// `who/o=is_here[,soft][,corner N]` — the shape
    /// `RON_COLLIDE_PROBE`'s own `is_here`/`is_corner` pair prints, so a
    /// candidate walk can be compared as text against the trace's.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}={}", self.who, self.o, i32::from(self.is_here))?;
        if self.soft {
            write!(f, ",soft")?;
        }
        if let Some(n) = self.is_corner {
            write!(f, ",corner {n}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for SweepVerdict {
    /// One sweep as one line, in the order the original's own bracket
    /// prints it: the proposed cell, the probe's hit, `will_be_corner`,
    /// the candidate walk, and the verdict. It is what the run116
    /// comparison diffs — a rendered line against a rendered line — so
    /// every field of the verdict is read by the thing that checks it
    /// rather than by a ledger row saying nothing does.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.gated {
            return write!(f, "ungated");
        }
        write!(f, "at ({},{})", self.at_cell.x, self.at_cell.y)?;
        match self.hit {
            None => return write!(f, " clear"),
            Some(c) => write!(f, " hit ({},{}) will {}", c.x, c.y, self.will_be_corner)?,
        }
        for c in &self.candidates {
            write!(f, " {c}")?;
        }
        match self.hard {
            Some((who, o)) => write!(f, " hard {who}/{o}"),
            None if self.soft => write!(f, " soft"),
            None => write!(f, " pass"),
        }
    }
}

/// The one draw the whole mechanic spends: the stagger a unit gives itself
/// when the repath of §6 step 6 succeeded and the unit it collided with is
/// colliding with **it** — `Random::get(0, 0xffff) % 9 + 1` into the
/// order's `pause` (`docs/COLLISION.md` §6). Two units walking into each
/// other therefore do not both step off on the same frame.
pub const SITE_PAUSE: &str = "Unit::resolve_unit_collision+0xb52";

/// The unit cell a position falls in.
pub const fn ucell(p: Pos) -> Pos {
    Pos {
        x: p.x.div_euclid(UNITS_PER_UCELL),
        y: p.y.div_euclid(UNITS_PER_UCELL),
    }
}

/// The centre of a unit cell, in position units — where a collision snaps
/// a unit (`docs/COLLISION.md` §6 step 6).
pub const fn ucell_centre(c: Pos) -> Pos {
    Pos {
        x: c.x * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
        y: c.y * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
    }
}

/// `move_x` / `move_y` (`00adcaf0` / `00adc400`) and `radius[]`, as
/// `docs/COLLISION.md` §2.1 derives them: `radius[r] = (2r + 1)²`, and the
/// prefix of that length is the Chebyshev disc of radius `r`.
///
/// Ring `r` is walked clockwise from `(−r, −r)` — top row left to right,
/// right column top to bottom, bottom row right to left, left column
/// bottom to top — **except ring 2**, which is the same walk with its four
/// corners moved to the end in the order NW, NE, SE, SW. The quirk is in
/// the shipped table, verified entry for entry against the PE for `r ≤ 7`,
/// which covers every `BLOCK_RADIUS` in `unitrules.xml`.
pub fn spiral(r: i32) -> Vec<(i32, i32)> {
    let mut out = vec![(0, 0)];
    for ring in 1..=r {
        let mut cells = Vec::new();
        for x in -ring..=ring {
            cells.push((x, -ring));
        }
        for y in -ring + 1..=ring {
            cells.push((ring, y));
        }
        for x in (-ring..ring).rev() {
            cells.push((x, ring));
        }
        for y in (-ring + 1..ring).rev() {
            cells.push((-ring, y));
        }
        if ring == 2 {
            let corners = [(-2, -2), (2, -2), (2, 2), (-2, 2)];
            cells.retain(|c| !corners.contains(c));
            cells.extend_from_slice(&corners);
        }
        out.extend(cells);
    }
    out
}

/// `CollCheck::collide_here@00682540`'s body over what it reads, so a sweep
/// can run it on the original's own slot state: the caller's size and
/// proposed unit cell `at`, its own cell `mine` and whether it is on the
/// map, `nocoll`, and the two reads the slots give — `live(p)`, the
/// cell's slot has a block that is not empty (`local_30[slot] == 0`), and
/// `get(p)`, that block's bit for the cell.
pub fn collide_here_steps(
    size: i32,
    at: Pos,
    mine: Pos,
    on_map: bool,
    nocoll: bool,
    live: impl Fn(Pos) -> bool,
    get: impl Fn(Pos) -> bool,
) -> Option<Pos> {
    if size == 0 {
        return None;
    }
    // **The fast path, and it is not an optimisation** (§4.2, item 183).
    // With `nocoll` clear and the proposal exactly one cell away on one
    // axis, the original sweeps the **leading edge** — the row or column
    // the block is entering — and nothing else. That is a strict subset
    // of the parity-filtered disc, so it can name a *different* first
    // hit cell, and §4.3's corner rule is decided on the cell.
    if !nocoll {
        let (dx, dy) = (at.x - mine.x, at.y - mine.y);
        if dx == 0 || dy == 0 {
            if dx.abs() == 1 {
                return leading_edge(size, &live, &get, |k| {
                    Pos::new(at.x + dx * size, at.y - size + k)
                });
            }
            if dy.abs() == 1 {
                return leading_edge(size, &live, &get, |k| {
                    Pos::new(at.x - size + k, at.y + dy * size)
                });
            }
        }
    }
    for (dx, dy) in spiral(size) {
        if (dx + size) % 2 != 0 || (dy + size) % 2 != 0 {
            continue;
        }
        let p = Pos::new(at.x + dx, at.y + dy);
        // A cell inside my own block is mine to stand in.
        if on_map && (p.x - mine.x).abs() <= size && (p.y - mine.y).abs() <= size {
            continue;
        }
        if get(p) && live(p) {
            return Some(p);
        }
    }
    None
}

/// The fast path's sweep: the `size + 1` cells of the leading edge its
/// loop's own parity leaves, in order, the first occupied one winning.
/// The disc's own-block exemption is not asked and does not have to be
/// — the edge is a cell beyond the caller's own block on every step
/// that reaches here.
fn leading_edge(
    size: i32,
    live: &impl Fn(Pos) -> bool,
    get: &impl Fn(Pos) -> bool,
    cell: impl Fn(i32) -> Pos,
) -> Option<Pos> {
    // `00682540:71`-`116`, and `:126`-`174` for the other axis. The
    // loop runs `2·size + 1` times; an odd pass advances the swept
    // axis, and an even one tests the cell and advances it again on a
    // miss — **but only when the cell's slot is live**. A cell whose
    // world cell holds no bit (or which the region gate refused) is
    // skipped *without* the advance, so the next cell tested is one
    // step on, not two.
    let mut at = 0;
    for k in 0..=2 * size {
        if k % 2 == 1 {
            at += 1;
            continue;
        }
        let p = cell(at);
        if live(p) {
            if get(p) {
                return Some(p);
            }
            at += 1;
        }
    }
    None
}

/// `Objects::find_collision@0065b1b0`'s two decisions, over what they read:
/// with its fifth argument **clear** and a **land** caller (type `+0x218 ==
/// 0`) the answer is `CollCheck::collide_here`'s, returned as it stands
/// (`65b1c3`/`65b1e2`); otherwise a caller whose `coll_size` (type
/// `+0x248`) is zero answers 0 (`65b239`), and anything else answers the
/// nine-cell chain walk ([`chain_hit_steps`]).
pub(crate) fn find_collision_steps(
    forced: bool,
    land: bool,
    mine: i32,
    probe: impl FnOnce() -> i32,
    walk: impl FnOnce() -> bool,
) -> i32 {
    if !forced && land {
        return probe();
    }
    if mine == 0 {
        return 0;
    }
    i32::from(walk())
}

/// The second pass of `Objects::find_ordered_collision@0065b440`
/// (`65b680`-`65b7b3`), over closures (item 3261's sweep drives it): the
/// asker's own group — `group` is its owner and member list, `None` when the
/// asker has none — must belong to the asker's player and list the asker; then
/// every *other* member that `member` yields (`None` for one not alive and on
/// the map, or with no block) is tested with the unit-cell Chebyshev
/// predicate against its ordered spot, `<= mine + its block` on both axes.
/// A caller with no block of its own (`mine == 0`) answers false first.
pub(crate) fn ordered_group_steps(
    group: Option<(crate::Player, &[usize])>,
    u: usize,
    owner: crate::Player,
    mine: i32,
    c: (i32, i32),
    mut member: impl FnMut(usize) -> Option<(i32, (i32, i32))>,
) -> bool {
    let Some((who, members)) = group else {
        return false;
    };
    if who != owner || !members.contains(&u) || mine == 0 {
        return false;
    }
    members.iter().any(|&o| {
        if o == u {
            return false;
        }
        let Some((r, q)) = member(o) else {
            return false;
        };
        (c.0 - q.0).abs() <= mine + r && (c.1 - q.1).abs() <= mine + r
    })
}

/// The walk in `Objects::find_collision@0065b1b0` (`65b290`-`65b42d`), over
/// closures: for each of the nine world cells around the query, in the
/// shipped `move_x`/`move_y` order (`(0,0)` then the ring, [`spiral`]`(1)`),
/// walk the cell's chain from its head (`head(dx, dy)`, `None` for a cell
/// outside the world or an empty one) by `next`; `other(o)` is `None` for
/// an object the walk passes over (the caller itself, `who >= 8`, one that
/// is not alive and on the map, or has no block) and otherwise its
/// `coll_size` and unit cell. A hit is Chebyshev `<= mine + r` on both axes
/// (`65b3db`, `65b3fe`).
/// Whether `Object::add_to_world` and `remove_from_world` touch an owner's
/// objects at all: both test the owner byte `< 9` unsigned at their head.
pub(crate) fn threads(owner: crate::Player) -> bool {
    owner < 9
}

pub(crate) fn chain_hit_steps<O: Copy>(
    c: (i32, i32),
    mine: i32,
    mut head: impl FnMut(i32, i32) -> Option<O>,
    mut next: impl FnMut(O) -> Option<O>,
    mut other: impl FnMut(O) -> Option<(i32, (i32, i32))>,
) -> bool {
    for (dx, dy) in spiral(1) {
        let mut cur = head(dx, dy);
        while let Some(o) = cur {
            cur = next(o);
            let Some((r, q)) = other(o) else {
                continue;
            };
            if (c.0 - q.0).abs() <= mine + r && (c.1 - q.1).abs() <= mine + r {
                return true;
            }
        }
    }
    false
}

/// **`Guy::process@005e0230`'s sixty-fourth-frame gate** (`:57-62`, and the
/// flag clear at `:120` under the same test): the guy has stopped
/// (`avg_speed == 0`, `+0x84`) and `frame + (short)object index` is a
/// multiple of 64 - the signed remainder's zero, which is the Euclidean one
/// (item 3396).
pub const fn repaint_due(frame: i64, index: i16, avg_speed: i32) -> bool {
    avg_speed == 0 && (frame + index as i64).rem_euclid(64) == 0
}

/// What the repaint walk reads and writes ([`paint_walk`]): the figure's own
/// tile region, the region of a world cell, and one bit of the occupancy.
/// [`SimPaint`] is the simulation's; `sweep::guy_process` lays the emulated
/// original's block windows behind the same methods (item 3396).
pub trait PaintView {
    /// `WorldData::get_tregion` of the figure's own tile; `None` asks no
    /// region (the original's negative answer).
    fn region(&mut self) -> Option<u16>;
    /// The region of the world cell `cell`, `None` where it has none.
    fn cell_region(&mut self, cell: Pos) -> Option<u16>;
    /// Set (`on`) or clear the unit cell `p`'s bit, if it is on the map.
    fn mark(&mut self, p: Pos, on: bool);
}

/// **The disc walk** `Sim::coll_paint` and `Guy::process`'s repaint make:
/// every offset of [`spiral`]`(size)` around the unit cell `c`, marked when
/// its world cell's region is the figure's own, or the figure's has none.
pub fn paint_walk<V: PaintView>(v: &mut V, size: i32, c: Pos, on: bool) {
    let region = v.region();
    for (dx, dy) in spiral(size) {
        let p = Pos::new(c.x + dx, c.y + dy);
        let ok = match region {
            None => true,
            Some(r) => v
                .cell_region(Pos::new(
                    p.x.div_euclid(UCELLS_PER_CELL),
                    p.y.div_euclid(UCELLS_PER_CELL),
                ))
                .is_none_or(|cr| cr == r),
        };
        if ok {
            v.mark(p, on);
        }
    }
}

/// [`PaintView`] over the simulation's world and flat grid.
struct SimPaint<'a> {
    world: &'a World,
    coll: &'a mut CollGrid,
    region: Option<u16>,
}

impl PaintView for SimPaint<'_> {
    fn region(&mut self) -> Option<u16> {
        self.region
    }
    fn cell_region(&mut self, cell: Pos) -> Option<u16> {
        self.world.region_of(Cell::new(cell.x, cell.y))
    }
    fn mark(&mut self, p: Pos, on: bool) {
        self.coll.set(p.x, p.y, on);
    }
}

/// `CollBlock`, flattened: one bit per unit cell over the whole map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CollGrid {
    w: i32,
    h: i32,
    bits: Vec<u64>,
}

impl CollGrid {
    /// A grid over a world of `cw` × `ch` world cells.
    pub fn new(cw: i32, ch: i32) -> CollGrid {
        let (w, h) = (cw * UCELLS_PER_CELL, ch * UCELLS_PER_CELL);
        let n = (w as usize) * (h as usize);
        CollGrid {
            w,
            h,
            bits: vec![0; n.div_ceil(64)],
        }
    }

    const fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return None;
        }
        Some((y as usize) * (self.w as usize) + (x as usize))
    }

    /// Whether any unit cell of the world cell `(cx, cy)` is occupied —
    /// `BitMask<768>::empty@00479150`, negated. Off the map is empty.
    pub fn any_in_cell(&self, cx: i32, cy: i32) -> bool {
        (0..UCELLS_PER_CELL).any(|dy| {
            (0..UCELLS_PER_CELL)
                .any(|dx| self.get(cx * UCELLS_PER_CELL + dx, cy * UCELLS_PER_CELL + dy))
        })
    }

    /// Whether a unit cell is occupied.
    pub fn get(&self, x: i32, y: i32) -> bool {
        self.index(x, y)
            .is_some_and(|i| self.bits[i / 64] & (1u64 << (i % 64)) != 0)
    }

    pub fn set(&mut self, x: i32, y: i32, on: bool) {
        if let Some(i) = self.index(x, y) {
            if on {
                self.bits[i / 64] |= 1u64 << (i % 64);
            } else {
                self.bits[i / 64] &= !(1u64 << (i % 64));
            }
        }
    }

    /// The map's width in unit cells.
    pub const fn width(&self) -> i32 {
        self.w
    }
}

/// What [`detect_boat_collision_steps`] reads of the world and does to it:
/// the unit `u` that pushes, and `o`, a unit the push may move. [`SimBoat`]
/// is the simulation's; `sweep::unit_detect_boat_collision` lays the
/// emulated original's windows behind the same methods (item 3304).
pub trait BoatView {
    fn owner(&mut self, o: usize) -> Player;
    fn domain(&mut self, o: usize) -> Domain;
    /// The type's `push_size` and `push_circles` (`+0x2f8`, `+0x2fc`), raw.
    fn push_size(&mut self, o: usize) -> i32;
    fn push_circles(&mut self, o: usize) -> i32;
    /// `UnitData::get_action` and the order's slot `+0x10`.
    fn action_index(&mut self, u: usize) -> Option<i32>;
    fn is_transport(&mut self, u: usize) -> bool;
    /// `Objects::find_units(at, all players, range, 0x200, FILTER_NOT_ME)`,
    /// the pusher's raw circle count handed along.
    fn find_units(&mut self, u: usize, at: Pos, size: i32, circles: i32) -> Vec<usize>;
    /// The object's `+0xa8` slot, the unit it stands for; the simulation's
    /// list holds units already.
    fn resolve(&mut self, o: usize);
    fn facing(&mut self, o: usize) -> Angle;
    fn pos(&mut self, o: usize) -> Pos;
    fn group(&mut self, o: usize) -> i64;
    fn index(&mut self, o: usize) -> i16;
    fn collide_o(&mut self, o: usize) -> i16;
    fn collide_who(&mut self, o: usize) -> i8;
    fn collide_frame(&mut self, o: usize) -> i64;
    fn frame(&mut self) -> i64;
    /// The type's `attack` (`+0x1e8`) is set.
    fn attack(&mut self, o: usize) -> bool;
    fn is_ally(&mut self, a: Player, b: Player) -> bool;
    fn packs(&mut self, o: usize) -> bool;
    fn packed(&mut self, o: usize) -> bool;
    fn is_unpacking(&mut self, o: usize) -> bool;
    fn entrenched(&mut self, o: usize) -> bool;
    fn is_tank(&mut self, o: usize) -> bool;
    fn is_moving(&mut self, o: usize) -> bool;
    fn invalid_loc(&mut self, o: usize, tile: Pos) -> i32;
    fn set_new_location(&mut self, o: usize, to: Pos);
    /// `o`'s `collide_o` and `collide_who` name the pusher `by`.
    fn name_pusher(&mut self, o: usize, by: usize);
    fn no_orders(&mut self, o: usize) -> bool;
    /// `Unit::set_angle` and the guy-0 turn (`5faea6`-`5faedb`).
    fn turn_idle(&mut self, o: usize, bearing: Angle, by: usize);
    fn stamp_collide_frame(&mut self, o: usize);
}

/// [`Sim`] behind [`BoatView`].
struct SimBoat<'a>(&'a mut Sim);

impl BoatView for SimBoat<'_> {
    fn owner(&mut self, o: usize) -> Player {
        self.0.units[o].owner
    }
    fn domain(&mut self, o: usize) -> Domain {
        self.0.units[o].kind.domain
    }
    fn push_size(&mut self, o: usize) -> i32 {
        self.0.profile(Obj::Unit(o)).push_size
    }
    fn push_circles(&mut self, o: usize) -> i32 {
        self.0.profile(Obj::Unit(o)).push_circles
    }
    fn action_index(&mut self, u: usize) -> Option<i32> {
        self.0
            .action_of(u)
            .map(|a| i32::from(self.0.units[u].orders[a].index()))
    }
    fn is_transport(&mut self, u: usize) -> bool {
        self.0.units[u].ty.is_some_and(|t| {
            self.0.unit_types[t].cols.unit_flags & crate::ai_load::uflags::FLAG_TRANSPORT != 0
        })
    }
    fn find_units(&mut self, u: usize, at: Pos, size: i32, _circles: i32) -> Vec<usize> {
        self.0.find_push_candidates(u, at, size)
    }
    fn resolve(&mut self, _o: usize) {}
    fn facing(&mut self, o: usize) -> Angle {
        self.0.units[o].movement.facing
    }
    fn pos(&mut self, o: usize) -> Pos {
        self.0.units[o].pos
    }
    fn group(&mut self, o: usize) -> i64 {
        self.0.pool_group_of(o)
    }
    fn index(&mut self, o: usize) -> i16 {
        self.0.units[o].index
    }
    fn collide_o(&mut self, o: usize) -> i16 {
        self.0.units[o].collide_o
    }
    fn collide_who(&mut self, o: usize) -> i8 {
        self.0.units[o].collide_who
    }
    fn collide_frame(&mut self, o: usize) -> i64 {
        self.0.units[o].collide_frame
    }
    fn frame(&mut self) -> i64 {
        self.0.frame
    }
    fn attack(&mut self, o: usize) -> bool {
        self.0.profile(Obj::Unit(o)).attack != 0
    }
    fn is_ally(&mut self, a: Player, b: Player) -> bool {
        self.0.is_ally(a, b)
    }
    fn packs(&mut self, o: usize) -> bool {
        self.0.units[o]
            .ty
            .map(|t| self.0.unit_types[t].cols)
            .is_some_and(|c| c.flag2(crate::ai_load::uflags2::PACKS))
    }
    fn packed(&mut self, o: usize) -> bool {
        self.0.units[o].combat.packed
    }
    fn is_unpacking(&mut self, o: usize) -> bool {
        self.0.is_unpacking(o)
    }
    fn entrenched(&mut self, o: usize) -> bool {
        self.0.units[o].combat.entrenched
    }
    fn is_tank(&mut self, o: usize) -> bool {
        self.0.units[o]
            .ty
            .map(|t| self.0.unit_types[t].cols)
            .is_some_and(|c| c.flag(crate::ai_load::uflags::TANK))
    }
    fn is_moving(&mut self, o: usize) -> bool {
        self.0.is_moving(o)
    }
    fn invalid_loc(&mut self, o: usize, tile: Pos) -> i32 {
        self.0
            .invalid_loc(o, tile, false, false, false, false, false)
    }
    /// `set_new_location(pushed, x, y, 0, 0)` at `detect_boat_collision`'s
    /// push, and the ring reveal `005f8d20` throws inside it when the
    /// boat crosses a half-cell (`docs/VISION.md` §6, §11.5): a pushed
    /// boat sees as it drifts (item 3569, `docs/AI.md` §310).
    fn set_new_location(&mut self, o: usize, to: Pos) {
        let from = self.0.units[o].pos;
        self.0.set_new_location(o, to, false);
        self.0.moved_to(o, from, true);
    }
    fn name_pusher(&mut self, o: usize, by: usize) {
        self.0.units[o].collide_o = self.0.units[by].index;
        self.0.units[o].collide_who = self.0.units[by].owner as i8;
    }
    fn no_orders(&mut self, o: usize) -> bool {
        self.0.units[o].orders.is_empty()
    }
    fn turn_idle(&mut self, o: usize, bearing: Angle, _by: usize) {
        self.0.unit_set_angle(o, bearing);
        self.0.push_turn(o, bearing);
    }
    fn stamp_collide_frame(&mut self, o: usize) {
        self.0.units[o].collide_frame = self.0.frame;
    }
}

/// A place in a world cell's chain for [`SimDetect`]: a live unit, a wreck the
/// original still has on its chain (item 3233, `docs/GOLDEN.md` §131) — the
/// unit and its place in [`Sim::deaths`] — or the end of the chain.
#[derive(Clone, Copy, Debug)]
enum DetectNode {
    End,
    Live(usize),
    Wreck(usize, usize),
}

/// [`Sim`] behind [`DetectCollisionView`]: the unit `u` asking.
///
/// Where the original reads the objects table and a unit's vslots, this reads
/// the unit's own fields; **the node's `(o, who)` is the unit's `index` and
/// `owner`**. The wrecks that are still on the original's chain are walked
/// from the death list after each cell's live chain, because this crate
/// unlinks a unit at its death and keeps the disc beside it.
struct SimDetect<'a> {
    sim: &'a mut Sim,
    u: usize,
    /// The hit cell `collide_here` answered, which a wreck is tested against.
    hit: Pos,
    /// The world cell `get_down` last opened.
    cell: Cell,
    /// `set_move_x`'s word, held for `set_move_y`.
    move_x: i32,
}

impl<'a> SimDetect<'a> {
    fn new(sim: &'a mut Sim, u: usize) -> Self {
        Self {
            sim,
            u,
            hit: Pos::new(0, 0),
            cell: Cell { x: 0, y: 0 },
            move_x: 0,
        }
    }

    /// The unit a node names.
    fn unit_of(&self, n: DetectNode) -> usize {
        match n {
            DetectNode::Live(i) | DetectNode::Wreck(i, _) => i,
            DetectNode::End => unreachable!("the end of a chain names no unit"),
        }
    }

    /// The first wreck of the open cell from the death list's entry `from`
    /// that the original's chain would still hold and whose block covers the
    /// hit cell.
    fn wreck_from(&self, from: usize) -> DetectNode {
        let s = &*self.sim;
        for (di, d) in s.deaths.iter().enumerate().skip(from) {
            let Some(w) = s
                .units
                .iter()
                .rposition(|x| i32::from(x.owner) == d.who && i32::from(x.index) == d.o)
            else {
                continue;
            };
            if s.units[w].alive()
                || !s.units[w].on_map
                || s.units[w].coll_at.is_some()
                || s.units[w].pos.cell() != self.cell
                || !s.units[w].ty.is_some_and(|t| {
                    s.unit_types[t].cols.unit_flags & crate::ai_load::uflags::BLOCKS_WHILE_DEAD != 0
                })
                || !s.unit_is_here(w, self.hit)
            {
                continue;
            }
            return DetectNode::Wreck(w, di);
        }
        DetectNode::End
    }

    fn acting(&self, v: usize) -> Option<usize> {
        self.sim
            .action_of(v)
            .map(|a| self.sim.units[v].orders[a].index() as usize)
    }
}

impl DetectCollisionView for SimDetect<'_> {
    type Node = DetectNode;

    fn domain(&mut self) -> i32 {
        match self.sim.units[self.u].kind.domain {
            Domain::Land => 0,
            Domain::Sea => 1,
            Domain::Air => 2,
        }
    }
    fn type_is_siege(&mut self) -> bool {
        self.sim.is_siege_unit(self.u)
    }
    fn is_hero(&mut self) -> bool {
        self.sim.is_hero_unit(self.u)
    }
    fn is_supply(&mut self) -> bool {
        self.sim.is_supply_unit(self.u)
    }
    fn detect_boat_collision(&mut self, x: i32, y: i32) -> bool {
        self.sim.detect_boat_collision(self.u, Pos::new(x, y), true)
    }
    fn path_detour(&mut self) -> bool {
        self.sim.units[self.u]
            .path
            .last()
            .is_some_and(|t| t.flags & path_flag::DETOUR != 0)
    }
    fn is_safe(&mut self) -> bool {
        self.sim.units[self.u].safe != 0
    }
    fn own_cell(&mut self) -> (i32, i32) {
        let c = ucell(self.sim.units[self.u].pos);
        (c.x, c.y)
    }
    fn collide_here(&mut self, cell: (i32, i32), nocoll: bool) -> Option<(i32, i32)> {
        let hit = self
            .sim
            .collide_here(self.u, Pos::new(cell.0, cell.1), nocoll)?;
        self.hit = hit;
        Some((hit.x, hit.y))
    }
    fn will_be_corner(&mut self, hit: (i32, i32), cell: (i32, i32)) -> i32 {
        Sim::corner_of(
            self.sim.coll_size(self.u),
            Pos::new(cell.0, cell.1),
            Pos::new(hit.0, hit.1),
        )
    }
    fn action_type(&mut self) -> i32 {
        self.acting(self.u).map_or(-1, |i| i as i32)
    }
    fn attack_target_exists(&mut self) -> bool {
        let u = self.u;
        let Some(a) = self.sim.action_of(u) else {
            return false;
        };
        let Body::Attack(_) = self.sim.units[u].orders[a].body else {
            return false;
        };
        self.sim.units[u]
            .combat
            .target
            .is_some_and(|t| self.sim.obj_alive(t))
    }
    fn max_range(&mut self) -> i32 {
        self.sim.max_range_of(Obj::Unit(self.u))
    }
    fn attack_dist(&mut self, x: i32, y: i32) -> i32 {
        let u = self.u;
        let t = self.sim.units[u]
            .combat
            .target
            .expect("`attack_target_exists` held");
        let ext = crate::combat::extent(&self.sim.profile(t), matches!(t, Obj::Building(_)));
        let mine = crate::combat::extent(&self.sim.profile(Obj::Unit(u)), false);
        crate::combat::attack_dist(Pos::new(x, y), self.sim.pos_of(t), mine, ext, false)
    }
    fn world_size(&mut self) -> (i32, i32) {
        (self.sim.world.width(), self.sim.world.height())
    }
    fn get_down(&mut self, cell: (i32, i32)) -> Option<Self::Node> {
        self.cell = Cell {
            x: cell.0,
            y: cell.1,
        };
        let slot = (cell.1 as usize) * (self.sim.world.width() as usize) + (cell.0 as usize);
        Some(match self.sim.chain_heads[slot] {
            Some(h) => DetectNode::Live(h),
            None => self.wreck_from(0),
        })
    }
    fn node_id(&mut self, n: Self::Node) -> (i32, i32) {
        match n {
            DetectNode::End => (-1, -1),
            DetectNode::Live(i) | DetectNode::Wreck(i, _) => (
                i32::from(self.sim.units[i].index),
                i32::from(self.sim.units[i].owner),
            ),
        }
    }
    fn node_next(&mut self, n: Self::Node) -> Self::Node {
        match n {
            DetectNode::End => DetectNode::End,
            DetectNode::Live(i) => match self.sim.units[i].down {
                Some(d) => DetectNode::Live(d),
                None => self.wreck_from(0),
            },
            DetectNode::Wreck(_, di) => self.wreck_from(di + 1),
        }
    }
    fn node_is_me(&mut self, n: Self::Node) -> bool {
        matches!(n, DetectNode::Live(i) if i == self.u)
    }
    fn node_alive(&mut self, n: Self::Node) -> bool {
        match n {
            DetectNode::Live(i) => self.sim.units[i].alive(),
            _ => true,
        }
    }
    fn node_air(&mut self, n: Self::Node) -> bool {
        matches!(n, DetectNode::Live(i) if self.sim.is_air(i))
    }
    fn node_on_map(&mut self, n: Self::Node) -> bool {
        match n {
            DetectNode::Live(i) => self.sim.units[i].on_map,
            _ => true,
        }
    }
    fn node_is_here(&mut self, n: Self::Node, hit: (i32, i32)) -> bool {
        match n {
            DetectNode::Live(i) => self.sim.unit_is_here(i, Pos::new(hit.0, hit.1)),
            _ => true,
        }
    }
    fn node_action_type(&mut self, n: Self::Node) -> i32 {
        match n {
            DetectNode::Live(i) => self.acting(i).map_or(0, |a| a as i32),
            _ => 0,
        }
    }
    fn is_moving_me(&mut self) -> bool {
        self.sim.current_order(self.u).is_some_and(Order::is_move)
    }
    fn node_is_moving(&mut self, n: Self::Node) -> bool {
        matches!(n, DetectNode::Live(i) if self.sim.current_order(i).is_some_and(Order::is_move))
    }
    fn guard_targets_me(&mut self, n: Self::Node) -> bool {
        let DetectNode::Live(i) = n else {
            return false;
        };
        self.sim.action_of(i).is_some_and(
            |a| matches!(self.sim.units[i].orders[a].body, Body::Guard(g) if g.target == self.u),
        )
    }
    fn node_same_owner(&mut self, n: Self::Node) -> bool {
        let i = self.unit_of(n);
        self.sim.units[i].owner == self.sim.units[self.u].owner
    }
    fn my_block(&mut self) -> i32 {
        self.sim.coll_size(self.u)
    }
    fn node_block(&mut self, n: Self::Node) -> i32 {
        self.sim.coll_size(self.unit_of(n))
    }
    // **A wreck is read for the group arm too** (item 3824, `docs/GOLDEN.md`
    // §152.10): `detect_unit_collision:308`–`316` reads the candidate
    // object's `+0x80`, `+0x104` and `+0xdc`, and a death writes none of
    // the first two — no writer of `+0x80 = -1` is on a death path —
    // while `Unit::close`'s last statement, `close_orders`, empties the
    // third. So a dead group-mate is a nudge, not a wall: run1476's `2/22`
    // steps onto the dead Main Battle Tank `2/21`'s cell on 761.
    fn groups_shared(&mut self, n: Self::Node) -> bool {
        let (DetectNode::Live(i) | DetectNode::Wreck(i, _)) = n else {
            return false;
        };
        let (me, other) = (&self.sim.units[self.u], &self.sim.units[i]);
        // The original asks the one `short`; this crate keeps the owner beside it.
        me.owner == other.owner && me.group_ptr.is_some() && me.group_ptr == other.group_ptr
    }
    fn node_searching(&mut self, n: Self::Node) -> bool {
        matches!(n, DetectNode::Live(i) | DetectNode::Wreck(i, _) if self.sim.units[i].search.is_some())
    }
    fn node_front_order(&mut self, n: Self::Node) -> Option<i32> {
        let (DetectNode::Live(i) | DetectNode::Wreck(i, _)) = n else {
            return None;
        };
        let front = self.sim.units[i].orders.front()?;
        // The kinds the group arm lets through are `NONE`, `GUARD`, a
        // `CAST_SPELL` and the move family **of a move body**; any other
        // order answers a type no row of the arm tests.
        Some(match (front.index(), &front.body) {
            (index::NONE | index::GUARD, _) => i32::from(front.index()),
            (_, Body::Cast(_)) => i32::from(index::CAST_SPELL),
            (k, Body::Move(_)) if index::is_move_family(k) => i32::from(k),
            _ => -1,
        })
    }
    fn node_cast_soft(&mut self, n: Self::Node) -> bool {
        let DetectNode::Live(i) = n else {
            return false;
        };
        matches!(
            self.sim.units[i].orders.front().map(|f| &f.body),
            Some(Body::Cast(c)) if matches!(
                c.spell,
                crate::orders::spell::PACK
                    | crate::orders::spell::PACK_MACHINEGUN
                    | crate::orders::spell::PACK_MERCHANT
                    | crate::orders::spell::PACK_FISHERMEN
            )
        )
    }
    fn node_corner(&mut self, n: Self::Node, hit: (i32, i32)) -> i32 {
        self.sim.guy_corner(self.unit_of(n), Pos::new(hit.0, hit.1))
    }
    fn record_hit(&mut self, n: Self::Node) {
        let other = self.unit_of(n);
        let (index, owner) = (self.sim.units[other].index, self.sim.units[other].owner);
        let me = &mut self.sim.units[self.u];
        me.collide_o = index;
        me.collide_who = owner as i8;
        me.collide_guy = 0;
    }
    fn front_is_moving(&mut self) -> bool {
        self.sim.units[self.u]
            .orders
            .front_mut()
            .is_some_and(|f| f.move_mut().is_some())
    }
    fn set_move_x(&mut self, x: i32) {
        self.move_x = x;
    }
    fn set_move_y(&mut self, y: i32) {
        let at = Pos::new(self.move_x, y);
        if let Some(m) = self.sim.units[self.u]
            .orders
            .front_mut()
            .and_then(|f| f.move_mut())
        {
            m.coll = Some(at);
        }
    }
    fn set_half_step(&mut self) {
        self.sim.units[self.u].half_step = true;
    }
    fn clear_collider(&mut self) {
        self.sim.units[self.u].collide_o = -1;
        self.sim.units[self.u].collide_who = -1;
    }
    fn collide_frame(&mut self) -> i64 {
        self.sim.units[self.u].collide_frame
    }
    fn frame(&mut self) -> i64 {
        self.sim.frame
    }
    fn reset_collide_count(&mut self) {
        self.sim.units[self.u].collide = 0;
    }
    fn clear_wait_flag(&mut self) {
        self.sim.units[self.u].waiting_on = false;
    }
}

/// The body of [`Sim::detect_boat_collision`] over a [`BoatView`].
pub fn detect_boat_collision_steps<V: BoatView>(v: &mut V, u: usize, at: Pos, mates: bool) -> bool {
    let who = v.owner(u);
    if who >= 8 {
        return true;
    }
    let domain = v.domain(u);
    let action = v.action_index(u);
    let (size, raw_circles) = (v.push_size(u), v.push_circles(u));
    let circles = raw_circles.max(1);
    if size == 0 {
        return true;
    }
    let transport = v.is_transport(u);
    let found = v.find_units(u, at, size, raw_circles);
    if found.is_empty() {
        return true;
    }
    let r_mine = size / circles;
    let facing = v.facing(u);
    let line = |p: Pos, a: Angle, r: i32, n: i32| -> (Pos, (i32, i32)) {
        if n > 1 {
            let (dx, dy) = (sin_component(a, r), -cos_component(a, r));
            (
                Pos::new(p.x - (n - 1) * dx, p.y - (n - 1) * dy),
                (2 * dx, 2 * dy),
            )
        } else {
            (p, (0, 0))
        }
    };
    let (mine0, mine_step) = line(at, facing, r_mine, circles);
    let my_group = v.group(u);
    let land = domain == Domain::Land;
    for o in found {
        v.resolve(o);
        let oowner = v.owner(o);
        // `(local_54 == 0 || who < 8)`: a land pusher's candidates
        // include gaia's.
        if (!land && oowner >= 8) || v.domain(o) != domain {
            continue;
        }
        let mate = oowner == who && v.group(o) == my_group && my_group != -1;
        if mate && mates && action != Some(10) {
            continue;
        }
        if v.collide_o(u) == v.index(o)
            && v.collide_who(u) == oowner as i8
            && v.collide_frame(u) == v.frame()
        {
            continue;
        }
        let (osize, ocircles) = (v.push_size(o), v.push_circles(o).max(1));
        if osize == 0 {
            continue;
        }
        let r_other = osize / ocircles;
        let (opos, ofacing) = (v.pos(o), v.facing(o));
        let (other0, other_step) = line(opos, ofacing, r_other, ocircles);
        let mut nearest = 0x0fff_ffff;
        for i in 0..circles {
            let m = Pos::new(mine0.x + i * mine_step.0, mine0.y + i * mine_step.1);
            for j in 0..ocircles {
                let t = Pos::new(other0.x + j * other_step.0, other0.y + j * other_step.1);
                let d = boat_dist(m.x - t.x, m.y - t.y);
                if (i == 0 && j == 0) || d < nearest {
                    nearest = d;
                }
            }
        }
        let overlap = (r_other - nearest + r_mine) / 2;
        if overlap <= 0 {
            continue;
        }
        if transport && v.attack(o) {
            return false;
        }
        // `5fad01`-`5fad3f`: a stranger who is not a mutual ally
        // refuses the push only while it is a player's — `cmpb $8,
        // %dl; jb` — so gaia's animals are pushed aside.
        if oowner < 8 && !v.is_ally(who, oowner) {
            return false;
        }
        // A land pusher's own refusals (`5fac8b`-`5faccc`): a packer
        // that is not packed or is unpacking, an entrenched unit
        // (`unit_masks & 0x2000000`), and a tank (the type's vslot
        // `+0x110`, `UnitTypeData::is_tank`).
        if land {
            if v.packs(o) && (!v.packed(o) || v.is_unpacking(o)) {
                return false;
            }
            if v.entrenched(o) || v.is_tank(o) {
                return false;
            }
        }
        let push = overlap.min(0x30);
        let there = opos;
        let mut bearing = find_angle(there.x - at.x, there.y - at.y);
        let rel = (bearing.0 as u32).wrapping_sub(facing.0 as u32);
        if !v.is_moving(o) {
            if rel < 0x2000_0000 {
                bearing = Angle(facing.0.wrapping_add(0x2000_0000));
            } else if rel > 0xe000_0000 {
                bearing = Angle(facing.0.wrapping_sub(0x2000_0000));
            }
        } else if rel.wrapping_add(0xe000_0000) > 0xc000_0000 {
            return false;
        }
        let to = Pos::new(
            there.x + sin_component(bearing, push),
            there.y - cos_component(bearing, push),
        );
        if v.invalid_loc(o, to.tile()) != 0 {
            continue;
        }
        v.set_new_location(o, to);
        if mates && (ocircles == 1 || land) {
            v.name_pusher(o, u);
            if v.no_orders(o) {
                v.turn_idle(o, bearing, u);
            }
        }
        v.stamp_collide_frame(o);
    }
    true
}

impl Sim {
    // ------------------------------------------------------------------
    // §2 — the occupancy index
    // ------------------------------------------------------------------

    /// The type's `+0x248` — `BLOCK_RADIUS` before the `× 48`, which is
    /// the unit all of this mechanic's discs are measured in.
    pub(crate) fn coll_size(&self, u: usize) -> i32 {
        self.profile(Obj::Unit(u)).block_radius / UNIT_BLOCK_RADIUS
    }

    /// `ObjectType::domain == 2` — air, which neither occupies ground nor
    /// collides.
    ///
    /// ~~SEAM: no domain is loaded, so every unit is land.~~ The domain is
    /// loaded (`UnitKind::domain`, `rondata::load`), and the first air unit
    /// the simulation ever stands up is gaia's bird (`crate::gaia`), which
    /// must not paint the occupancy grid a citizen walks on.
    fn is_air(&self, u: usize) -> bool {
        self.units[u].kind.domain == crate::attrition::Domain::Air
    }

    /// Whether `detect_unit_collision@00617060` takes its **second arm**
    /// for this unit (§4.1 gate 3, §13). The original's test is sea-domain
    /// (`ptype +0x218 == 1`), or a type that answers
    /// `ObjectTypeData::is_siege` (`+0x10c` on the type's table,
    /// `unit_flags & 0x20000`), or a unit that answers `is_hero` (`+0xc4`)
    /// or `is_supply` (`+0xcc`) — `617094`–`6170e9` on the listing.
    ///
    /// The arm is taken only with `top_only` and `nocoll` both zero. Then
    /// a `quick` call, or one that does not ask for `boats`, returns 0 at
    /// `61782b` — a bare `xor eax, eax; ret`, **past** the exit
    /// bookkeeping, so it clears nothing and ages nothing — and one that
    /// asks for `boats` returns 0 there when
    /// [`Self::detect_boat_collision`] answers non-zero, and runs the land
    /// scan when it answers 0.
    ///
    /// ~~SEAM: **only the sea half is taken.**~~ Both halves are taken
    /// (item 696, golden chapter eleven): a siege engine, a hero or a
    /// supply wagon takes the arm too, and its push searches *land* units
    /// of every player, gaia included, and shoves them aside (§13.3, the
    /// land pusher's refusals). run190's wagon `0/7`, on its first step
    /// under a player's move, pushes the guard standing on its post (21,
    /// 1) and names itself in the guard's `collide_o` (run191's brackets).
    pub(crate) fn takes_boat_arm(&self, u: usize) -> bool {
        self.units[u].kind.domain == crate::attrition::Domain::Sea
            || self.is_siege_unit(u)
            || self.is_hero_unit(u)
            || self.is_supply_unit(u)
    }

    /// `Unit::detect_boat_collision@005fa8b0` for a sea unit (§13.3): does
    /// the unit about to stand on `at` push its way through? `true` is the
    /// original's 1, "handled", and [`Self::detect_unit_collision`]'s
    /// second arm then returns no collision; `false` hands the step to the
    /// land scan.
    ///
    /// The profile is `push_circles` circles of radius
    /// `push_size / push_circles`, strung along guy 0's facing and centred
    /// on the point (`project` with the facing and the radius, `5fa992`).
    /// The candidates are `Objects::find_units(at, all players, range
    /// push_size, 0x200, FILTER_NOT_ME)`; for each one of the same domain
    /// and a real player, not a group-mate when `mates` is asked for (the
    /// arm passes 1) unless the action is index 10, and not the unit this
    /// one already collided with this frame:
    ///
    /// - the overlap is `(r_other − d + r_mine) / 2`, `d` the nearest pair
    ///   of circle centres by the listing's own distance (`5fabf3`); none
    ///   is no push;
    /// - an attacker in the way of a transport, or a unit of a player who
    ///   is not a mutual ally, answers 0 outright;
    /// - a moving candidate within 45° of the pusher's facing answers 0;
    ///   one standing still is pushed at least 45° off it;
    /// - the push is `min(overlap, 48)` along that bearing, taken only if
    ///   `invalid_loc` allows the new tile, and the pushed unit's
    ///   `collide_frame` is stamped; with `mates` and one circle it also
    ///   records its pusher, and an idle one is turned to the bearing.
    pub(crate) fn detect_boat_collision(&mut self, u: usize, at: Pos, mates: bool) -> bool {
        detect_boat_collision_steps(&mut SimBoat(self), u, at, mates)
    }

    /// The pushed idle unit's guy 0 turned to the push's bearing
    /// (`5faeb1`–`5faedb`): `Guy::turn_angles(bearing, &out, 1, 1)` and
    /// `Guy::do_turn(bearing, out, 1, 1)`.
    ///
    /// `turn_angles@005d98c0` is [`crate::movement::turn_towards`] at the
    /// body's rate (`GuyData::turn_speed(1)`, the guy's own `last_speed`)
    /// **halved** by its fourth argument. `do_turn@005d97a0` then sets
    /// `guy_flags & 2` when `out` is not the angle it had — the bit the
    /// body's own turn respects, which this crate reads as the facing
    /// against [`crate::Movement::frame_facing`] — asks for the turn
    /// animation when the guy has one (its fifth argument), and writes
    /// `out` as guy 0's angle and every trackless crew figure's through
    /// `Guy::set_angle(out, 0)`. The peacock `8/2` on run511's block 9348,
    /// pushed by the Supply Wagon `1/86`, comes round 71° in the frame:
    /// its guy stands, so the rate is the instant one and half of it is
    /// still a quarter turn (`docs/COLLISION.md` §13.3).
    fn push_turn(&mut self, o: usize, bearing: Angle) {
        let m = self.units[o].movement;
        let turning = crate::movement::Turning {
            packed: self.units[o].combat.packed,
            ..m.turning
        };
        let rate = crate::movement::turn_speed(
            &self.tuning,
            &turning,
            m.body.last_speed,
            m.body.avg_speed,
            crate::movement::TurnMode::Body,
        ) >> 1;
        let was = m.facing;
        let (out, _) = crate::movement::turn_towards(was, bearing, rate);
        if out != was {
            let outer = self.phase_marks.last().map(|(label, _)| label.clone());
            self.mark(crate::anim::SITE_TURN_PUSHED);
            self.do_turn_anim(o, was, out, bearing);
            if let Some(outer) = outer {
                self.mark(&outer);
            }
        }
        self.units[o].movement.facing = out;
        let from = self.units[o].movement.body.pos;
        self.crew_des(o, from, out, false);
    }

    /// `Objects::find_units(at, SEARCH_ALL, −1, range, 0x200,
    /// FILTER_NOT_ME)` as `detect_boat_collision` asks it (`5fa935`–
    /// `5fa953`): the circle walk over the object chains while
    /// `circle_radius[ring]` is within the live unit count, the object
    /// arrays by leader and number with `vector_dist − R <= range` past
    /// it (R the candidate's radius, group 9), and the `0x200` region
    /// gate either way (`docs/COLLISION.md` §13.3). Region membership is
    /// tested on each candidate's tile, including the coastal refinement;
    /// a cell's primary region cannot stand in for all its occupants.
    fn find_push_candidates(&self, u: usize, at: Pos, range: i32) -> Vec<usize> {
        let circle = crate::ai_place::circle();
        let ring = ((range.max(0) + 0x2ff) / 0x300).min(0x40) as usize;
        let live = self.total_units();
        let region = self.world.tregion_alt(at.tile());
        let mut out = Vec::new();
        if circle.radius[ring] <= live {
            let c0 = at.cell();
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) {
                    continue;
                }
                let slot = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
                let mut next = self.chain_heads[slot];
                while let Some(o) = next {
                    next = self.units[o].down;
                    if o != u
                        && self.units[o].alive()
                        && self.world.tregion_alt(self.units[o].pos.tile()) == region
                    {
                        out.push(o);
                    }
                }
            }
        } else {
            // **The list branch** (`find_units@0065a620`, listing
            // `65aa72..65aaa6`; twenty-fourth pass, group 9; A2 row 11): by
            // leader, then object number, and a candidate is kept when
            // `vector_dist − R <= range` with R its `push_size` if it is a sea
            // unit and its `big_radius` otherwise. It runs only with fewer
            // than nine units alive.
            let mut order: Vec<usize> = (0..self.units.len()).collect();
            order.sort_by_key(|&o| (self.units[o].owner, self.units[o].index));
            for o in order {
                let p = self.units[o].pos;
                if o == u
                    || !self.units[o].alive()
                    || !self.units[o].on_map
                    || self.world.tregion_alt(p.tile()) != region
                {
                    continue;
                }
                if vector_dist(p.x - at.x, p.y - at.y) - self.find_units_reach(o) > range {
                    continue;
                }
                out.push(o);
            }
        }
        out
    }

    /// The region gate (`docs/COLLISION.md` §2): a cell is written only
    /// when its world cell's region matches the marking figure's own tile
    /// region, or the figure's tile has none.
    fn coll_region_ok(&self, region: Option<u16>, c: Pos) -> bool {
        let Some(r) = region else { return true };
        self.world
            .region_of(Cell::new(
                c.x.div_euclid(UCELLS_PER_CELL),
                c.y.div_euclid(UCELLS_PER_CELL),
            ))
            .is_none_or(|cr| cr == r)
    }

    /// `Object::add_to_world`'s marking half: set every cell of the unit's
    /// disc. Idempotent at one point — and **refused, in a debug build, at
    /// a second** (Loop 2132): a unit already painted at `coll_at` that is
    /// painted again at a moved `pos` leaves its first disc standing, and a
    /// test fixture that did so passed on the stale disc while the walk
    /// held it (item 2088). The move is `coll_move`; a repaint at a new
    /// point clears the old one first (`coll_remove`).
    pub(crate) fn coll_add(&mut self, u: usize) {
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let at = self.units[u].pos;
        debug_assert!(
            self.units[u].coll_at.is_none_or(|c| c == at),
            "coll_add on unit {u} at {:?} while its disc stands at {:?}: coll_remove first, or coll_move (Loop 2132)",
            at,
            self.units[u].coll_at
        );
        self.coll_paint(u, at, true);
        self.units[u].coll_at = Some(at);
    }

    /// **A unit coming out of something, on the occupancy index**
    /// (`docs/COLLISION.md` §25): `Unit::come_out@00617c10:514`'s
    /// `Object::remove_from_inside@006480f0` ends in `add_to_world`, which
    /// paints the disc at guy 0's **retained** point — where the figure
    /// stood when it went in, since nothing moves it while it is inside —
    /// and only then does `:518`'s `set_new_location(·, ·, 1, 1)` reach
    /// `Guy::set_new_location@005d86f0`'s `CollCheck::move_unit(old,
    /// spot)`. So the old footprint is painted and cleared again everywhere
    /// farther than `coll_size` from the spot — **taking with it any bit
    /// another unit had painted there since**, because the bits are not
    /// refcounted (§2) — and the spot's disc is set. East Indies' AI
    /// passenger `1/12`, put ashore on tick 1113 far from the point it
    /// boarded from on 1008, clears unit cell (890, 771) of Freighter
    /// `1/27`'s birth disc in run713's watch; this crate left it set.
    pub(crate) fn coll_come_out(&mut self, u: usize, from: Pos) {
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let to = self.units[u].pos;
        self.coll_paint(u, from, true);
        self.coll_move(u, from, to);
        self.units[u].coll_at = Some(to);
    }

    /// A reused unit slot retains guy 0's old point. `Unit::init` seats
    /// that figure with the new type's radius, so its old disc is cleared
    /// even though the previous occupant is closed (`docs/COLLISION.md` §23).
    pub(crate) fn coll_rebirth(&mut self, u: usize, from: Pos) {
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let to = self.units[u].pos;
        self.coll_move(u, from, to);
        self.units[u].coll_at = Some(to);
    }

    /// `Guy::set_new_location`'s `CollCheck::move_unit(from, to)`: the
    /// disc moves from where the occupancy last saw guy 0 to where guy 0
    /// now stands (`docs/COLLISION.md` §16). A unit that is not painted
    /// has nothing to move.
    pub(crate) fn coll_follow(&mut self, u: usize) {
        let Some(from) = self.units[u].coll_at else {
            return;
        };
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let to = self.units[u].movement.body.pos;
        self.coll_move(u, from, to);
        self.units[u].coll_at = Some(to);
    }

    /// **`Guy::process@005e0230`'s sixty-fourth frame** — the repaint that
    /// heals the holes the index's own design punches
    /// (`docs/COLLISION.md` §2.2).
    ///
    /// The bits are not refcounted: when a unit leaves a cell its
    /// [`Sim::coll_move`] clears every cell of its old disc that its new one
    /// does not cover, **including the cells another unit is still standing
    /// on**. Nothing in `move_unit` or `add_to_world` ever puts those back,
    /// so a standing unit's block would rot away one corner at a time as its
    /// neighbours walked past. This is what puts them back: after
    /// `Guy::move`, on the frames where `(game->frame + o) % 64 == 0`, a guy
    /// whose `avg_speed` is **zero** re-marks its whole disc —
    /// `radius[coll_size]`, the same walk `add_to_world` takes, set-only and
    /// with the same region gate.
    ///
    /// The gates are the original's, in its order: not air, `guy_num <
    /// squad_size`, `coll_size != 0`, the phase, and the standing test. The
    /// phase is per **object number**, so the map's units repaint on
    /// sixty-four different frames rather than all at once, and a hole lives
    /// for at most sixty-four frames.
    ///
    /// SEAM: `crates/sim` marks one figure a unit (§2), so this repaints guy
    /// 0's disc and no other's; `squad_size` is thereby always satisfied.
    pub(crate) fn coll_repaint(&mut self, u: usize) {
        // **And a boat that stepped ashore in its own work repaints too**
        // (item 1223, `docs/GOLDEN.md` §52): `Unit::process@00610bc0` runs
        // `Guy::process` over the dead boat's figure after the think that
        // killed it, and `Guy::process@005e0230`'s tail asks nothing of the
        // unit but its type — so on its sixty-fourth frame, standing, the
        // figure re-marks the disc `Object::close` has just cleared, and
        // nothing owns the bits after. [`Sim::process_unit`] reaches here
        // for a dead unit on that one arm alone.
        let dead_boat = !self.units[u].alive()
            && !self.units[u].on_map
            && self.units[u].kind.domain == crate::attrition::Domain::Sea;
        if !(self.units[u].alive() && self.units[u].on_map) && !dead_boat {
            return;
        }
        // `avg_speed` (`GuyData +0x84`) is `Guy::move`'s running quarter of
        // `last_speed`, so it reaches zero a few frames after the unit stops
        // and is non-zero for every frame of a walk.
        if !repaint_due(
            self.frame,
            self.units[u].index,
            self.units[u].movement.body.avg_speed,
        ) {
            return;
        }
        let at = self.units[u].coll_at.unwrap_or(self.units[u].pos);
        self.coll_paint(u, at, true);
    }

    /// **`Unit::set_type`'s crew loop, on the occupancy index**
    /// (`docs/AI.md` §120). `Unit::set_type@00612fa0`'s second kill loop
    /// walks the figures from the **old** type's `squad_size` to the
    /// unit's whole count and calls `Objects::kill_guy@00659410` on each,
    /// and `kill_guy`'s last arm clears `radius[coll_size]` around the
    /// *figure's own cell* — `CollBlock::set(…, 0)` over the same disc
    /// `add_to_world` set, by the unit's **old** type, since
    /// `set_type` swaps `+0x18` only after the loop. Nothing puts the
    /// cells back but the sixty-fourth-frame repaint, which marks the
    /// *new* disc: a Trebuchet (size 3, trackless crew standing on guy 0)
    /// that becomes a Bombard (size 2) leaves no ring of its old block
    /// and a hole in its new one until the next repaint.
    ///
    /// SEAM: `kill_guy`'s own gates before the clear — the figure's type
    /// is not the crash kind (`+0x14 == 8`), its domain is not air, and the
    /// object type's vslot `+0x120` answers 0 — are read as true;
    /// [`Self::coll_paint`] gates air. The region gate there is `coll_paint`'s,
    /// not `kill_guy`'s plain cell region; the two differ on a coastal cell.
    /// One figure of the crew loop: `Objects::kill_guy`'s clear around
    /// figure `i`'s own cell, called for each figure from the squad to the
    /// old total (`unit_set_type_view`) — and for every figure of a unit
    /// that dies, by `close_unit` (`Unit::close`'s two loops, item 3787,
    /// `docs/COLLISION.md` §2).
    pub(crate) fn coll_kill_guy(&mut self, u: usize, i: usize) {
        let body = self.units[u].movement.body.pos;
        let at = self.units[u].guys[i].follow.map_or(body, |f| f.body.pos);
        self.coll_paint(u, at, false);
    }

    /// `Object::remove_from_world`'s half — the clear pass with nowhere to
    /// move to.
    pub(crate) fn coll_remove(&mut self, u: usize) {
        let at = self.units[u].coll_at.take().unwrap_or(self.units[u].pos);
        self.coll_paint(u, at, false);
    }

    /// **A starting citizen's birth, on the occupancy index**
    /// (`docs/COLLISION.md` §20). `Setup::build_units@005aafc0` creates
    /// each of them with `Objects::init_unit` at its building's own point —
    /// `Unit::init@00612100:69` snaps it to the 48-grid, and its
    /// `set_new_location(·, ·, 1, 1)` at `:549` puts guy 0 there — and then
    /// `Unit::come_out@00617c10` steps it off the footprint with a second
    /// `set_new_location(·, ·, 1, 1)` (`:518`). That second move is
    /// `CollCheck::move_unit(birth, here)`, whose clear pass takes every
    /// cell of the birth disc the new disc does not cover — **including the
    /// cells of a citizen born there before it**, because the bits are not
    /// refcounted (§2). So the second citizen out of a woodcutter's camp
    /// leaves a hole in the first one's block, and the first frames of a
    /// game walk past it.
    ///
    /// Called after every unit is painted at stand-up, citizen by citizen
    /// in creation order: a cell is cleared unless a unit in `later` covers
    /// it, since that unit's own birth paints it again after this exit.
    pub fn birth_come_out(&mut self, u: usize, birth: Pos, later: &[usize]) {
        let size = self.coll_size(u);
        if size == 0 || self.is_air(u) || !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let here = self.units[u].coll_at.unwrap_or(self.units[u].pos);
        let (a, b) = (ucell(birth), ucell(here));
        let region = self.world.tregion_alt(birth.tile());
        for (dx, dy) in spiral(size) {
            let p = Pos::new(a.x + dx, a.y + dy);
            if (p.x - b.x).abs() <= size && (p.y - b.y).abs() <= size {
                continue;
            }
            if !self.coll_region_ok(region, p) {
                continue;
            }
            let covered = later.iter().any(|&v| {
                let r = self.coll_size(v);
                r != 0
                    && !self.is_air(v)
                    && self.units[v].alive()
                    && self.units[v].on_map
                    && self.units[v].coll_at.is_some_and(|at| {
                        let c = ucell(at);
                        (p.x - c.x).abs() <= r && (p.y - c.y).abs() <= r
                    })
            });
            if !covered {
                self.coll.set(p.x, p.y, false);
            }
        }
    }

    fn coll_paint(&mut self, u: usize, at: Pos, on: bool) {
        let size = self.coll_size(u);
        if size == 0 || self.is_air(u) {
            return;
        }
        let c = ucell(at);
        // **`get_tregion`, not `region_of`.** `WorldData::get_tregion@006b52e0`
        // answers a coastal cell's `region2` for an **ocean** tile of it and
        // its `region` otherwise, and §2's gate is the figure's own
        // `get_tregion` against the cell's plain `region`. For a boat lying
        // on the water half of a coastal cell the two differ, so the boat's
        // block marks **nothing** there — which is what lets a passenger it
        // puts ashore stand a hundred units away rather than four hundred
        // (`docs/TRANSPORT.md` §6.4). The plain form made a barge fill its
        // own cell, and only a land unit was ever near enough to notice.
        let region = self.world.tregion_alt(at.tile());
        paint_walk(
            &mut SimPaint {
                world: &self.world,
                coll: &mut self.coll,
                region,
            },
            size,
            c,
            on,
        );
    }

    /// `CollCheck::move_unit(from, to, size)`: clear the cells around
    /// `from` that are further than `size` from `to`, then set those around
    /// `to` that are further than `size` from `from`. Each pass uses its
    /// own end's tile region.
    pub(crate) fn coll_move(&mut self, u: usize, from: Pos, to: Pos) {
        let size = self.coll_size(u);
        if size == 0 || self.is_air(u) {
            return;
        }
        let (a, b) = (ucell(from), ucell(to));
        if a == b {
            return;
        }
        let disc = spiral(size);
        let ra = self.world.tregion_alt(from.tile());
        // **The one-block form** (`CollCheck::move_unit@00682ad0`, its
        // head, `docs/COLLISION.md` §2.4): when both discs lie inside one
        // world cell — `|Δ| + 2·size < 16` on both axes and the box's
        // corners share a cell — the original looks the block up once, by
        // the **old** tile's region, and runs both passes on it ungated:
        // a region that is not the cell's ends the call before either pass.
        // The two-ended form below is the general one.
        let n = UCELLS_PER_CELL;
        let (lo_x, lo_y) = (a.x.min(b.x) - size, a.y.min(b.y) - size);
        let (hi_x, hi_y) = (a.x.max(b.x) + size, a.y.max(b.y) + size);
        let one_cell = (a.x - b.x).abs() + 2 * size < n
            && (a.y - b.y).abs() + 2 * size < n
            && hi_x.div_euclid(n) == lo_x.div_euclid(n)
            && hi_y.div_euclid(n) == lo_y.div_euclid(n)
            && lo_x >= 0
            && lo_y >= 0
            && lo_x.div_euclid(n) < self.world.width()
            && lo_y.div_euclid(n) < self.world.height();
        if one_cell {
            if !self.coll_region_ok(ra, Pos::new(lo_x, lo_y)) {
                return;
            }
            for (dx, dy) in &disc {
                let p = Pos::new(a.x + dx, a.y + dy);
                if (p.x - b.x).abs() > size || (p.y - b.y).abs() > size {
                    self.coll.set(p.x, p.y, false);
                }
            }
            for (dx, dy) in &disc {
                let p = Pos::new(b.x + dx, b.y + dy);
                if (p.x - a.x).abs() > size || (p.y - a.y).abs() > size {
                    self.coll.set(p.x, p.y, true);
                }
            }
            return;
        }
        for (dx, dy) in &disc {
            let p = Pos::new(a.x + dx, a.y + dy);
            if ((p.x - b.x).abs() > size || (p.y - b.y).abs() > size) && self.coll_region_ok(ra, p)
            {
                self.coll.set(p.x, p.y, false);
            }
        }
        let rb = self.world.tregion_alt(to.tile());
        for (dx, dy) in &disc {
            let p = Pos::new(b.x + dx, b.y + dy);
            if ((p.x - a.x).abs() > size || (p.y - a.y).abs() > size) && self.coll_region_ok(rb, p)
            {
                self.coll.set(p.x, p.y, true);
            }
        }
    }

    // ------------------------------------------------------------------
    // §3 — the object chain
    // ------------------------------------------------------------------

    /// The world cell's slot in [`Sim::chain_heads`].
    fn chain_slot(&self, p: Pos) -> Option<usize> {
        let c = p.cell();
        if c.x < 0 || c.y < 0 || c.x >= self.world.width() || c.y >= self.world.height() {
            return None;
        }
        Some((c.y as usize) * (self.world.width() as usize) + (c.x as usize))
    }

    /// `Object::add_to_world`'s chain half: push onto the head of the
    /// world cell's list.
    ///
    /// **Owner 9 is never threaded** (`docs/COLLISION.md` §3.1, item
    /// 3709): `Object::add_to_world@0064d8c0` (at `0064d8e9`) and
    /// `Object::remove_from_world@00647970` (at `00647993`) both open on
    /// `cmp byte who, 9; jae` past the whole body, so a bird and a
    /// pasture's carry never head a cell — and a bird over a deployed boat
    /// must not hide the boat's claim from `think_fish`.
    pub(crate) fn chain_add(&mut self, u: usize) {
        if !threads(self.units[u].owner) {
            return;
        }
        let Some(s) = self.chain_slot(self.units[u].pos) else {
            return;
        };
        let head = self.chain_heads[s];
        self.units[u].up = None;
        self.units[u].down = head;
        if let Some(h) = head {
            self.units[h].up = Some(u);
        }
        self.chain_heads[s] = Some(u);
    }

    /// The world cell's object chain, head first — the order
    /// `Object::find_nearby_target` walks and the order the dump's
    /// `up`/`down`/`up_who`/`down_who` print
    /// (`docs/COMBAT.md` §12.2, §18.1).
    ///
    /// [`Self::chain_add`] pushes on the head, so the chain is newest
    /// first: chapter one's cell reads `1/8, 1/7, 1/6, 0/8, 0/7, 0/6`
    /// there and here alike. **Units only** — a building is an object on
    /// the original's chain too, and this crate has never threaded one,
    /// so a caller that wants both appends the buildings itself. That
    /// seam is stated where it is taken.
    pub(crate) fn cell_chain(&self, c: crate::Cell) -> Vec<usize> {
        if c.x < 0 || c.y < 0 || c.x >= self.world.width() || c.y >= self.world.height() {
            return Vec::new();
        }
        let s = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
        let mut out = Vec::new();
        let mut next = self.chain_heads[s];
        while let Some(u) = next {
            out.push(u);
            next = self.units[u].down;
        }
        out
    }

    /// `Object::remove_from_world`'s chain half, under the same owner gate
    /// as [`Self::chain_add`].
    pub(crate) fn chain_remove(&mut self, u: usize) {
        if !threads(self.units[u].owner) {
            return;
        }
        let (up, down) = (self.units[u].up, self.units[u].down);
        match up {
            Some(p) => self.units[p].down = down,
            None => {
                if let Some(s) = self.chain_slot(self.units[u].pos)
                    && self.chain_heads[s] == Some(u)
                {
                    self.chain_heads[s] = down;
                }
            }
        }
        if let Some(d) = down {
            self.units[d].up = up;
        }
        self.units[u].up = None;
        self.units[u].down = None;
    }

    // ------------------------------------------------------------------
    // The one place a unit's position changes — `Unit::set_new_location`
    // ------------------------------------------------------------------

    /// `Unit::set_new_location(x, y, move_guys, …)`: move the unit, and
    /// with it both indices. A world-cell change re-links the object chain;
    /// a unit-cell change moves the occupancy bits.
    ///
    /// `move_guys` is the original's `param_3`, and it is not decoration:
    /// with it set, `Guy::set_new_location` **teleports** the body onto the
    /// new point instead of leaving it to chase, so the follow phase reads
    /// a zero `last_speed` and the next frame's turn is instant
    /// (`docs/MOVEMENT.md`, "The body step"). `move_step` passes 0;
    /// `resolve_unit_collision` passes 1, which is why a unit that snaps to
    /// its cell centre can walk off in a new direction on the very next
    /// frame.
    ///
    /// The fog reveal that hangs off the same call stays with its caller
    /// ([`Sim::moved_to`], `docs/VISION.md` §6).
    /// **A payoff probe's hand**, not a mechanic: seat unit `u` on `to`
    /// through [`Sim::set_new_location`], so the collision grid and the
    /// figures follow. The diff harness uses it to put a unit where the
    /// original's dump has it and ask whether a later frame agrees.
    #[doc(hidden)]
    pub fn probe_relocate(&mut self, u: usize, to: Pos) -> bool {
        self.set_new_location(u, to, true)
    }

    pub(crate) fn set_new_location(&mut self, u: usize, to: Pos, move_guys: bool) -> bool {
        let from = self.units[u].pos;
        // **A move onto the point the unit already stands on is not a
        // no-op**, and the early return that used to sit here made it
        // one. `005f8d20` has no such return: an unchanged point makes
        // both of its cell tests false and falls through to
        // `LAB_005f9033`, which writes the coordinates back and then runs
        // the **guy half** — the crew loop included. The seating still
        // happens.
        //
        // That is the whole of item 210. `Unit::do_cast` re-seats a rare
        // collector on the first frame of its unpack, and Great Lakes'
        // Merchant `1/24` is already on its own unit-cell centre when it
        // does, so every part of this function but the last four lines is
        // a no-op for it — and those four lines are what put its crew
        // figure on its offset instead of leaving it to walk four more
        // frames and pay an arrival stand the original never pays.
        if from != to {
            let on_map = self.units[u].on_map && self.units[u].alive();
            let cell_change = on_map && from.cell() != to.cell();
            // **The forest cloak on a tile step** (`005f8d20:61-82`, item
            // 3958, `docs/AI.md` §402): a tile crossing inside one world
            // cell re-tests an Iroquois land unit's new tile, the same
            // test as [`Sim::process_cloak`]'s; a cell crossing does not.
            if on_map && !cell_change && from.tile() != to.tile() {
                self.step_cloak(u, to);
            }
            if !self.shore_step(u, from, to, on_map, cell_change) {
                return false;
            }
            if cell_change {
                self.chain_remove(u);
                // **The world-cell crossing heals the unit's own disc**
                // (§2.3, item 267). `Unit::set_new_location@005f8d20`
                // brackets the coordinate write with
                // `Object::remove_from_world` and `Object::add_to_world`
                // on a **world-cell** change, and both of those walk
                // `radius[coll_size]` around each figure's *current*
                // point — which is still the old one, since the guys do
                // not move until `Guy::process`. The clear and the set
                // cover the same cells under the same region gate, so
                // the pair is a **set** of the whole disc: every hole a
                // neighbour's `move_unit` punched in this unit's block
                // is filled before the block is moved.
                //
                // It is the second healer, and unlike
                // [`Self::coll_repaint`]'s sixty-fourth frame — which
                // asks `avg_speed == 0` and so only ever reaches a unit
                // standing still — this one fires for a unit **on the
                // march**, once every sixteen unit cells.
                //
                // Both walks are around the **figure's** point, which is
                // where [`Unit::coll_at`] keeps the disc; and the disc does
                // not move here at all (§16): `move_unit`'s one caller is
                // `Guy::set_new_location`, so the bits follow guy 0 when
                // it next moves — the same turn for a unit's own step, and
                // its next turn for a unit a later one pushed.
                if let Some(at) = self.units[u].coll_at {
                    self.coll_paint(u, at, true);
                }
            }
            self.units[u].pos = to;
            if cell_change {
                self.chain_add(u);
                // The goody box, `docs/GOODY.md` §2. The original's four
                // guards, in its own order after `add_to_world`: not an animal
                // (`SubObjectData::is_animal`, the same slot `+0x30` step 0 of
                // §6 reads), not a placement ghost (`unit_masks & 1`), a land
                // type (`type->domain == 0`), and the cell's `WData` first
                // `short` negative — bit `0x8000`, `GOODY`.
                if !self.units[u].is_gaia()
                    && !self.units[u].decoy
                    && self.units[u]
                        .ty
                        .is_none_or(|t| self.unit_types[t].combat.domain == Domain::Land)
                    && self.world.cell_data(to.cell()).flags & crate::world::cell::GOODY != 0
                {
                    self.explore_goody(u);
                }
            }
        }
        if move_guys {
            self.units[u].movement.body.pos = to;
            // `Guy::set_new_location(guy 0, pos, 1)`'s crew loop
            // (`005d86f0:83`–`127`): every crew figure takes guy 0's
            // `angle` as its `des_angle` and is snapped to it, so the
            // pair an order's `set_angle` left apart is closed
            // ([`crate::Movement::crew_des`]; item 3762). `Unit::fight`'s
            // strike reaches it before its swing; `do_attack_ground` does
            // not.
            self.units[u].movement.crew_des = None;
            // `Guy::set_new_location(guy 0, pos, 1)` moves the disc now.
            self.coll_follow(u);
            // Its helicopter climb, and `param_3`'s `last_z = z` after it
            // (`5d8859`). SEAM: the write for every other figure.
            if self.is_helicopter(u) {
                self.helicopter_climb(u, to);
                let af = &mut self.units[u].airframe;
                af.last_z = af.z;
            }
            // `Unit::set_new_location`'s `param_3` does not stop at guy 0.
            // It is handed on as `Guy::set_new_location(guy 0, pos, 1)`,
            // whose crew loop **puts** every tracked figure on its new
            // offset and snaps its facing onto guy 0's — `set_angle(crew,
            // des_angle, 1)` then `set_new_location(crew, des, 1)`. So a
            // caravan that is teleported does not leave its crew behind to
            // walk after it: run67's block 6572 has the merchant's figure
            // on (34464, 37595), the rotation of its (−48, −192) track
            // about the cell centre it was just snapped to, and its own
            // angle equal to the leader's to the digit.
            let facing = self.units[u].movement.facing;
            self.crew_des(u, to, facing, true);
        }
        true
    }

    /// The shore half of `Unit::set_new_location@005f8d20`, and the whole
    /// reason it returns an `int`: a step that crosses the waterline is not
    /// taken, it is **converted** (`docs/TRANSPORT.md` §6).
    ///
    /// It is gated first — the step has to change world cell, or change
    /// tile *out of a `HALFLAND` cell*, which is the shore itself; a walk
    /// inside one cell of dry land never asks. Then, for a unit that
    /// `can_transport` (§3.2): a **land** unit stepping onto ocean queues
    /// the transport spell `QUEUE_FIRST` and stays where it is, and a
    /// **sea** unit stepping off ocean ejects what it carries and dies if
    /// that leaves it empty. Both return 0 to the caller, which is what
    /// makes `move_step` end the unit's frame there.
    ///
    /// Answers `true` when the step may go on.
    ///
    /// SEAM: the Iroquois arm between the gate and the test —
    /// `has_tribe_bonus(0x12)` with a `SURFACE_FOREST` destination writing
    /// `unit_masks & 0x800` — is not modelled; nothing reads that bit here.
    fn shore_step(
        &mut self,
        u: usize,
        from: Pos,
        to: Pos,
        on_map: bool,
        cell_change: bool,
    ) -> bool {
        let tile_change = on_map && from.tile() != to.tile();
        if !cell_change
            && (!tile_change
                || self.world.cell_data(from.cell()).flags & crate::world::cell::HALFLAND == 0)
        {
            return true;
        }
        if !self.unit_can_transport(u) {
            return true;
        }
        let ocean = self.world.tile_mask(to.tile()) & crate::world::tile::SURFACE
            == crate::world::tile::SURFACE_OCEAN;
        match self.profile(crate::combat::Obj::Unit(u)).domain {
            crate::attrition::Domain::Land if ocean => {
                self.add_cast_order(u, crate::orders::spell::TRANSPORT);
                false
            }
            crate::attrition::Domain::Sea if !ocean => {
                self.disembark(u);
                false
            }
            _ => true,
        }
    }

    // ------------------------------------------------------------------
    // §4.2 — the probe
    // ------------------------------------------------------------------

    /// `CollCheck::collide_here`: the first cell of the disc around the
    /// unit cell `at` that is occupied, outside the caller's own block, and
    /// passes the parity filter. `None` when nothing is in the way.
    ///
    /// ~~SEAM: `collide_here`'s two leading-edge fast paths are not
    /// modelled.~~ They are, since item 183 — see the block below, and
    /// §4.2's "the fast path is not an optimisation". The disc is walked
    /// only when neither arm applies or `nocoll` is set.
    pub(crate) fn collide_here(&self, u: usize, at: Pos, nocoll: bool) -> Option<Pos> {
        let size = self.coll_size(u);
        if size == 0 {
            return None;
        }
        let slots = self.probe_slots(at, size, nocoll);
        collide_here_steps(
            size,
            at,
            ucell(self.units[u].pos),
            self.units[u].on_map,
            nocoll,
            |p| slots.live(self, p),
            |p| slots.get(self, p),
        )
    }

    /// `CollCheck::fill_slots@006820e0` — which world cells' bitmasks a
    /// probe centred on the unit cell `at` may read at all.
    ///
    /// `nocoll` reads the same gate: its probes go through the
    /// pathfinder's `+0x4c` tree of block copies, and a copy is taken from
    /// the gated slot. SEAM: a copy outlives the probe that took it, so a
    /// later `nocoll` probe from another region reads it ungated; this
    /// crate gates every probe on its own centre.
    fn probe_slots(&self, at: Pos, size: i32, nocoll: bool) -> ProbeSlots {
        // `get_tregion` of the probe centre's own tile: the world cell's
        // `region`, or its `region2` when the tile is the water half of a
        // coastal cell.
        let tile = Pos::new(at.x.div_euclid(4), at.y.div_euclid(4));
        let mut slots = ProbeSlots {
            region: self.world.tregion_alt(tile),
            copied: Vec::new(),
        };
        if !nocoll {
            return slots;
        }
        // **The copy tree** (`fill_slots:81`-`168`): the 2×2 world cells
        // the probe's box `at ± size` touches, the first always and the
        // others when the box crosses into them, and none off the map.
        // Each is read from the pathfinder's copy when it holds one; each
        // it does not hold is read live, through the region gate, and a
        // copy of what was read goes into the tree for the next probe.
        let (x0, y0) = (
            (at.x - size).div_euclid(UCELLS_PER_CELL),
            (at.y - size).div_euclid(UCELLS_PER_CELL),
        );
        let (x1, y1) = (
            (at.x + size).div_euclid(UCELLS_PER_CELL),
            (at.y + size).div_euclid(UCELLS_PER_CELL),
        );
        let mut corners = vec![(x0, y0)];
        if y1 != y0 {
            corners.push((x0, y1));
        }
        if x1 != x0 {
            corners.push((x1, y0));
            if y1 != y0 {
                corners.push((x1, y1));
            }
        }
        let mut copies = self.coll_copies.borrow_mut();
        for (cx, cy) in corners {
            if cx < 0 || cy < 0 || cx >= self.world.width() || cy >= self.world.height() {
                continue;
            }
            let block = *copies.entry((cx, cy)).or_insert_with(|| {
                let mut b = [0u16; 16];
                let base = Pos::new(cx * UCELLS_PER_CELL, cy * UCELLS_PER_CELL);
                if slots.readable(self, base) {
                    for (dy, row) in b.iter_mut().enumerate() {
                        for dx in 0..UCELLS_PER_CELL {
                            if self.coll.get(base.x + dx, base.y + dy as i32) {
                                *row |= 1 << dx;
                            }
                        }
                    }
                }
                b
            });
            slots.copied.push(((cx, cy), block));
        }
        slots
    }

    /// `UnitData::is_here`: does this unit's block cover that unit cell?
    fn unit_is_here(&self, u: usize, cell: Pos) -> bool {
        let size = self.coll_size(u);
        if size == 0 {
            return false;
        }
        let c = ucell(self.units[u].pos);
        (cell.x - c.x).abs() <= size && (cell.y - c.y).abs() <= size
    }

    /// `UnitData::is_corner@0060a040` — **and it is not the unit's own
    /// block** (§4.3, 2026-09-04). The function walks the unit's figures
    /// `0 .. guy_mark` and returns the first non-zero
    /// `GuyData::is_corner@005de270`, which measures the corner against
    /// that **figure's** own `GuyData::x/y` rather than the unit's
    /// `x_internal`/`y_internal`. The type's `coll_size` is the unit's for
    /// every figure, so only the centre moves.
    ///
    /// ~~The two answers come apart twice: for a **crew** figure, which
    /// stands on a track offset a whole cell or more from its leader; and~~
    /// **`guy_mark` is 1** ([`crate::anim::SQUAD_SIZE`]), so the walk is
    /// figure 0 and no crew figure is ever asked (item 1524, `docs/AI.md`
    /// §140: a Bombard decoy's second figure turned a hard collision into
    /// a slip-past on Great Sahara at Toughest's 15246). The two answers
    /// come apart for guy 0 itself on any frame its body has not caught up with the
    /// unit's point (`docs/ANIM.md` §4 step 1) — `is_here`, the test
    /// immediately before, reads the **unit's** position, so the original
    /// genuinely mixes the two.
    fn guy_corner(&self, o: usize, cell: Pos) -> i32 {
        let size = self.coll_size(o);
        // SEAM: a unit this crate has stood up **without figures** —
        // `Sim::add_unit` does not call `init_guys`, so a hand-built one
        // has an empty list — answers from its own cell, which is what
        // this rule read before the guys were consulted at all. A live
        // unit's `guy_mark` is never 0 in the original, so the fallback
        // stands in for a state the original does not have rather than
        // for one it does.
        if self.units[o].guys.is_empty() {
            return Self::corner_of(size, ucell(self.units[o].pos), cell);
        }
        let body = self.units[o].movement.body.pos;
        // **`0 .. guy_mark`, and `guy_mark` is 1** (item 1524,
        // [`crate::anim::SQUAD_SIZE`]): the loop is the squad's figures and
        // never the crew's. A Bombard decoy's four figures stand a cell off
        // their leader and the second one's NW/SE corner cancelled the
        // asker's own, so ours slipped past a blocker the original refused.
        self.units[o]
            .guys
            .iter()
            .take(crate::anim::SQUAD_SIZE)
            .map(|g| g.follow.map_or(body, |f| f.body.pos))
            .map(|p| Self::corner_of(size, ucell(p), cell))
            .find(|&c| c != 0)
            .unwrap_or(0)
    }

    /// `UnitData::will_be_corner`: `1, 3, 5, 7` for NW, NE,
    /// SE, SW when `cell` is exactly a diagonal corner of the block centred
    /// on `centre`, else 0.
    const fn corner_of(size: i32, centre: Pos, cell: Pos) -> i32 {
        let (dx, dy) = (cell.x - centre.x, cell.y - centre.y);
        if dx.abs() != size || dy.abs() != size {
            return 0;
        }
        match (dx < 0, dy < 0) {
            (true, true) => 1,
            (false, true) => 3,
            (false, false) => 5,
            (true, false) => 7,
        }
    }

    // ------------------------------------------------------------------
    // §4 — `Unit::detect_unit_collision`
    // ------------------------------------------------------------------

    /// §4.1's gates, minus the same-cell test each form applies itself.
    /// `false` means "do not test at all".
    fn detect_gates(&self, u: usize) -> bool {
        if self.is_air(u) || self.units[u].safe != 0 {
            return false;
        }
        // A `go_around_building` detour waypoint suppresses the test
        // (`docs/ORDERS.md` §4.1).
        !self.units[u]
            .path
            .last()
            .is_some_and(|t| t.flags & path_flag::DETOUR != 0)
    }

    /// The quick form: any occupied cell is a collision and nothing is
    /// recorded — `move_step`'s second call and all four of
    /// `resolve_unit_collision`'s.
    pub(crate) fn detect_quick(&self, u: usize, at: Pos, nocoll: bool) -> bool {
        if !self.detect_gates(u) {
            return false;
        }
        // The second arm's `quick != 0` return (§13.2): a ship never
        // collides on a quick probe unless the caller passes `nocoll` —
        // `valid_ucoord`'s, which skips the arm.
        if !nocoll && self.takes_boat_arm(u) {
            return false;
        }
        let c = ucell(at);
        c != ucell(self.units[u].pos) && self.collide_here(u, c, nocoll).is_some()
    }

    /// `do_move`'s blocker probe (`docs/COLLISION.md` §18):
    /// `detect_unit_collision(coll_x, coll_y, quick 1, boats 1, 0,
    /// nocoll 0, top_only 1)` at `005f7dab`. It is the quick form, so it
    /// names nobody, asks no corner and writes nothing: a hit returns 1
    /// before the ladder and a miss returns 0 before the bookkeeping, and
    /// `collide_o` keeps the blocker the repath named. `top_only` skips
    /// the second arm, so a ship, a hero, a supply wagon and a siege
    /// engine scan here like anyone else.
    pub(crate) fn blocker_still_there(&self, u: usize, at: Pos) -> bool {
        if !self.detect_gates(u) {
            return false;
        }
        let c = ucell(at);
        c != ucell(self.units[u].pos) && self.collide_here(u, c, false).is_some()
    }

    /// The full form: find the cell, name the unit, apply the exemption
    /// ladder and the corner rule, and record the result. `Some(other)` is
    /// a hard collision.
    ///
    /// **It is [`detect_unit_collision_steps`]** (item 3402,
    /// `Unit::detect_unit_collision@00617060` swept against the original) run
    /// over [`SimDetect`], with the arguments the callers leave it: `quick 0`,
    /// `nocoll 0`, and `top_only 1` — the second arm (§13.2) is the callers',
    /// [`Self::takes_boat_arm`]. The bookkeeping the original does on the way
    /// out — clearing `collide_o`/`collide_who`, ageing `collide`, clearing
    /// the wait flag — happens there too, because every path out passes
    /// through it.
    pub(crate) fn detect_unit_collision(&mut self, u: usize, at: Pos) -> Option<usize> {
        // The recorder, and it is read-only: `sweep_verdict` walks the
        // same scan this call is about to walk and changes nothing.
        let watching = self.sweep_watch.as_ref().is_some_and(|w| {
            w.frame == self.frame
                && w.unit
                    == (
                        i32::from(self.units[u].owner),
                        i32::from(self.units[u].index),
                    )
        });
        if watching {
            let v = self.sweep_verdict(u, at);
            if let Some(w) = self.sweep_watch.as_mut() {
                w.seen.push((at, v));
            }
        }
        // The old walk (`scan_colliders`, which `sweep_verdict` still runs)
        // is held to the steps on every call a test makes.
        #[cfg(test)]
        let (old, half_before) = (self.sweep_verdict(u, at), self.units[u].half_step);
        let mut view = SimDetect::new(self, u);
        let (_, hit) = detect_unit_collision_steps(
            &mut view,
            DetectArgs {
                x: at.x,
                y: at.y,
                quick: false,
                boats: true,
                nocoll: false,
                top_only: true,
            },
        );
        let hit = hit.map(|n| view.unit_of(n));
        #[cfg(test)]
        {
            let named = hit.map(|o| {
                (
                    i32::from(self.units[o].owner),
                    i32::from(self.units[o].index),
                )
            });
            assert_eq!(named, old.hard, "the steps name what the old walk named");
            assert_eq!(
                self.units[u].half_step,
                half_before || (old.soft && old.hard.is_none()),
                "the steps set the soft one-shot where the old walk did"
            );
        }
        hit
    }

    /// The scan itself, with a sink for **every candidate it reaches**.
    ///
    /// [`Self::name_collider`] passes an empty sink and keeps only the
    /// verdict; [`Self::sweep_verdict`] passes a recording one, so the
    /// step-by-step comparison against `RON_COLLIDE_PROBE`'s record
    /// (`docs/COLLISION.md` §9) runs the **same** walk the simulation
    /// runs rather than a copy of it. Returns `(hard, soft, will)`.
    fn scan_colliders(
        &self,
        u: usize,
        at: Pos,
        at_cell: Pos,
        cell: Pos,
        rec: &mut impl FnMut(Candidate),
    ) -> (Option<usize>, bool, i32) {
        let size = self.coll_size(u);
        let will = Self::corner_of(size, at_cell, cell);
        let extra = self.attack_overreach(u, at);
        let mut soft = false;
        for (dx, dy) in spiral(1) {
            let p = Pos::new(at.x + dx * UNITS_PER_CELL, at.y + dy * UNITS_PER_CELL);
            let Some(s) = self.chain_slot(p) else {
                continue;
            };
            let mut next = self.chain_heads[s];
            while let Some(o) = next {
                next = self.units[o].down;
                if o == u || self.is_air(o) || !self.units[o].alive() || !self.units[o].on_map {
                    continue;
                }
                let here = self.unit_is_here(o, cell);
                let mut c = Candidate {
                    who: self.units[o].owner as i32,
                    o: i32::from(self.units[o].index),
                    is_here: here,
                    soft: false,
                    is_corner: None,
                };
                if !here {
                    rec(c);
                    continue;
                }
                if self.soft_collision(u, o, extra) {
                    c.soft = true;
                    rec(c);
                    soft = true;
                    continue;
                }
                let theirs = self.guy_corner(o, cell);
                c.is_corner = Some(theirs);
                rec(c);
                if will == 0 || (will - theirs).abs() != 4 {
                    return (Some(o), soft, will);
                }
            }
            // **A wreck still on the original's chain** (item 3233,
            // `docs/GOLDEN.md` §131): a type with `blocks_while_dead`
            // keeps `remove_from_world` unrun until its death object's
            // cull, so the scan reaches it and names it a hard collider
            // (run933's Tank `1/6` on `0/10`'s `collide_o`). This crate
            // unlinks the unit at its death and keeps the disc beside it,
            // so the wrecks are walked from the death list.
            for d in &self.deaths {
                let Some(w) = self
                    .units
                    .iter()
                    .rposition(|x| i32::from(x.owner) == d.who && i32::from(x.index) == d.o)
                else {
                    continue;
                };
                if self.units[w].alive()
                    || !self.units[w].on_map
                    || self.units[w].coll_at.is_some()
                    || self.units[w].pos.cell() != p.cell()
                    || !self.units[w].ty.is_some_and(|t| {
                        self.unit_types[t].cols.unit_flags
                            & crate::ai_load::uflags::BLOCKS_WHILE_DEAD
                            != 0
                    })
                    || !self.unit_is_here(w, cell)
                {
                    continue;
                }
                // **A dead group-mate is soft** (item 3824, `docs/GOLDEN.md`
                // §152.10): the group arm reads the wreck's own `+0x80`,
                // which no death writes, and its emptied order list.
                if self.same_group_soft(u, w) {
                    rec(Candidate {
                        who: self.units[w].owner as i32,
                        o: i32::from(self.units[w].index),
                        is_here: true,
                        soft: true,
                        is_corner: None,
                    });
                    soft = true;
                    continue;
                }
                let theirs = self.guy_corner(w, cell);
                rec(Candidate {
                    who: self.units[w].owner as i32,
                    o: i32::from(self.units[w].index),
                    is_here: true,
                    soft: false,
                    is_corner: Some(theirs),
                });
                if will == 0 || (will - theirs).abs() != 4 {
                    return (Some(w), soft, will);
                }
            }
        }
        (None, soft, will)
    }

    /// What §4.2's probe and §4.3's scan make of one proposed point, step
    /// by step — the same four answers `RON_COLLIDE_PROBE` prints from
    /// inside the original (`docs/COLLISION.md` §9).
    ///
    /// It exists because the *outcome* of a sweep is one bit and the
    /// disagreement is never in the bit: run116 has the original refusing
    /// a point this crate stepped onto, and the four steps are what says
    /// which of them parted. Read-only, and it runs the simulation's own
    /// walk rather than a copy — a diagnostic that drifts from the code
    /// it diagnoses is worse than none.
    pub fn sweep_verdict(&self, u: usize, at: Pos) -> SweepVerdict {
        let at_cell = ucell(at);
        let gated = self.detect_gates(u) && at_cell != ucell(self.units[u].pos);
        let hit = gated
            .then(|| self.collide_here(u, at_cell, false))
            .flatten();
        let mut candidates = Vec::new();
        let (hard, soft, will) = match hit {
            None => (None, false, 0),
            Some(cell) => self.scan_colliders(u, at, at_cell, cell, &mut |c| candidates.push(c)),
        };
        SweepVerdict {
            gated,
            at_cell,
            hit,
            will_be_corner: will,
            candidates,
            soft,
            hard: hard.map(|o| (self.units[o].owner as i32, i32::from(self.units[o].index))),
        }
    }

    /// How far beyond its own range an attacker's target would still be
    /// from the proposed point — `max(0, attack_dist − range × 0xc0)`, 0
    /// when the action is not an attack.
    ///
    /// **And 0 when the target no longer stands**: the original asks the
    /// action's `TargetOrder` its vslot `+4`, `TargetOrder::target_exists@0072ff10`
    /// (the slot's live bit and its `uid`), before it measures
    /// (`detect_unit_collision@00617060:103-119`), and leaves `local_20` at
    /// 0 when it fails. The battery word 12836 (`docs/AI.md` §245): the
    /// Scout `0/0` dies on 12835, and on 12836 four of who=1's attackers
    /// marching on it collide **hard** with their group-mates in the
    /// original, where this crate still measured to the dead Scout, passed
    /// `0x300` and called them soft.
    fn attack_overreach(&self, u: usize, at: Pos) -> i32 {
        let Some(a) = self.action_of(u) else { return 0 };
        let Body::Attack(_) = self.units[u].orders[a].body else {
            return 0;
        };
        let Some(t) = self.units[u].combat.target else {
            return 0;
        };
        if !self.obj_alive(t) {
            return 0;
        }
        let range = self.max_range_of(Obj::Unit(u));
        let ext = crate::combat::extent(&self.profile(t), matches!(t, Obj::Building(_)));
        let mine = crate::combat::extent(&self.profile(Obj::Unit(u)), false);
        let d = crate::combat::attack_dist(at, self.pos_of(t), mine, ext, false);
        (d - range * 0xc0).max(0)
    }

    /// §4.3's table: the ways another unit in the way is a nudge rather
    /// than a collision.
    ///
    /// ~~SEAM: the `TRADE_ROUTE`/`0xf` and `0xc` arms need action indices
    /// this crate does not carry, and no capture has entered either
    /// (`docs/COLLISION.md` §9).~~ Both are carried: the `0xc` arm since
    /// item 696, the caravan pair since item 1127 (§19). ~~And the group arm needs
    /// `UnitData::group`, which it does not keep~~ — it keeps one now
    /// (~~`docs/GROUPS.md` §1: an army's members are its group~~ — an
    /// army's *or a pushed slot's*, which is [`Sim::group_of`] and was
    /// item 489's whole finding), and [`Self::same_group_soft`] is that
    /// arm.
    fn soft_collision(&self, u: usize, o: usize, extra: i32) -> bool {
        let moving = |v: usize| self.current_order(v).is_some_and(Order::is_move);
        let acting = |v: usize| self.action_of(v).map(|a| self.units[v].orders[a].index());
        // §4.3's first row (`docs/COLLISION.md` §19): **a caravan pair**.
        // My `action_type` is `TRADE_ROUTE` (`sete` into `-0x40(%ebp)` at
        // `617214`), its action's type is `0xf` (`6173f2`-`617402`), and
        // both are `is_moving` (`617408`-`6174dd`) — vfunc `+0xd8`,
        // `UnitData::is_moving@00610af0`, the front order's `+0x14`, which
        // for a `MoveOrder` is a thunk to `mov eax,1` (`0041e0e0`, folded; read off the PE)
        // and for `UnitOrder` `xor eax,eax`. No owner test. Two trade
        // caravans on their legs pass through each other: East Indies'
        // 6151, where `1/15` walks through `1/33` paused at its leg's end
        // and the original sets the one-shot half step on block 6152.
        if acting(u) == Some(index::TRADE_ROUTE)
            && acting(o) == Some(index::TRADE_ROUTE)
            && moving(u)
            && moving(o)
        {
            return true;
        }
        // §4.3's second row (`00617546`-`0061757c`): its **action** is
        // `GUARD` and that order's target is me — an escort never blocks
        // its charge, whatever transit leg is at its head. golden chapter
        // eleven's wagon steps through its walking guard on run190's tick
        // 726, and run191's third take shows the original's scan reaching
        // `is_here` on the guard and never `is_corner` (item 696). The
        // first row, a caravan pair, is the else-if before it, above.
        if let Some(a) = self.action_of(o)
            && let Body::Guard(g) = self.units[o].orders[a].body
            && g.target == u
        {
            return true;
        }
        let attackers = acting(u) == Some(index::ATTACK)
            && acting(o) == Some(index::ATTACK)
            && self.units[u].owner == self.units[o].owner
            && self.coll_size(u) == 1
            && self.coll_size(o) == 1
            && moving(u)
            && moving(o)
            && extra > 0x300;
        attackers || self.same_group_soft(u, o)
    }

    /// §4.3's **group** arm, and the reason a squad marching in formation
    /// does not stand blocked on its own leader.
    ///
    /// > we share a `group` (≠ −1), I am not attacking, it has no
    /// > suspended search (`+0x104 == 0`), and either it has no order or
    /// > its order is a spell in `{0x28b, 0x28d, 0x28f, 0x291}` or a
    /// > passable kind — `0` and `0xc` unconditionally, and
    /// > `1, 2, 3, 4, 0x12, 0x13, 0x15` needing its action ≠ `ATTACK`
    ///
    /// The split is the one `detect_unit_collision@00617060:366-369`
    /// writes, and it is **not** symmetric in the way an earlier draft of
    /// §4.3 had it: `(iVar7 == 0 || iVar7 == 0xc)` short-circuits the
    /// whole `&& (local_28 != 10)` that gates the other **seven**, so
    /// `GUARD` passes while its holder attacks and a plain `MOVE_TO` does
    /// not. `iVar7` is the collider's *order* type and `local_28` its
    /// *action*'s, so nothing here asks whether the order carries a
    /// group: `0x13`/`0x15` (`GROUP_MOVE`/`GROUP_ATTACK_TO`) sit in the
    /// gated seven in their own right, and [`crate::orders::GroupMove`]
    /// is not consulted (2026-09-05, the docs-versus-code pass's R7).
    /// ~~and are written as the plain `1`/`2` here~~ — item 237 gave
    /// [`crate::orders::Order::index`] the original's own `get_type()`,
    /// so this is [`index::is_move_family`], the set said once.
    ///
    /// ~~SEAM: `UnitData +0x104`, the suspended pathfinder search, which
    /// this crate does not keep — read as zero, which widens the arm.~~
    /// **It keeps one** — [`crate::Unit::search`] *is* `+0x104..0x148`,
    /// since `docs/PATHFINDER.md` §18 — and the seam outlived the field
    /// by a fortnight. Reading it as zero is the whole of Great Lakes'
    /// word 10161 (`docs/COLLISION.md` §9, item 456): `1/31`, a
    /// group-mate of the unit probing it, suspends a 48-grid search on
    /// that very frame, so the original's arm declines and the collision
    /// is **hard**; this crate called it soft and walked through. The
    /// clause is the collider's alone — a suspended search of *mine*
    /// does not make its holder passable.
    ///
    /// SEAM: `0x12`, `CHANGE_FORM`, is an order this crate does not have,
    /// so the gated set is six of the seven here.
    ///
    /// ~~An army's members are its group~~ — **they are one of its two
    /// seats**, and reading only that one was Great Lakes' word 10277
    /// (item 489, `docs/COLLISION.md` §11). The original's test is one
    /// `short`, `+0x80` against `+0x80` and against −1
    /// (`detect_unit_collision@00617060:307`), which is the
    /// `Groups::list` slot; a group here sits either in an army or in a
    /// [`crate::group::Pushed`] pool slot, and `push_group` takes its
    /// members **out** of the army when it installs one. So from item
    /// 465 — which gave the AI's raiders a real `GROUP_MOVE` in the pool
    /// — every member of a pushed group was hard to every other, and a
    /// squad walking home stood on itself. [`Sim::group_of`] is the
    /// resolver that item built and [`Sim::find_ordered_collision`] has
    /// used since item 470; this was the site that was never moved over
    /// to it, for a fortnight.
    fn same_group_soft(&self, u: usize, o: usize) -> bool {
        if self.units[u].owner != self.units[o].owner {
            return false;
        }
        // **The back-pointer, not the army slot.** `+0x80` names a
        // `Groups::list` slot, and since item 557 this crate carries it
        // ([`crate::Unit::group_ptr`], `docs/GROUPS.md` §23), so the test
        // is the original's: both name a group, and the same one. Reading
        // `army_of` answered `None` for every member of a pushed group,
        // which since item 465 is the whole of Great Lakes' raid.
        let a = self.units[u].group_ptr;
        if a.is_none() || a != self.units[o].group_ptr {
            return false;
        }
        if self.units[o].search.is_some() {
            return false;
        }
        let acting = |v: usize| self.action_of(v).map(|x| self.units[v].orders[x].index());
        if acting(u) == Some(index::ATTACK) {
            return false;
        }
        let Some(front) = self.units[o].orders.front() else {
            return true;
        };
        // The two that short-circuit, whatever the action is doing.
        if matches!(front.index(), index::NONE | index::GUARD) {
            return true;
        }
        let not_attacking = acting(o) != Some(index::ATTACK);
        match front.body {
            crate::orders::Body::Cast(c) => matches!(
                c.spell,
                crate::orders::spell::PACK
                    | crate::orders::spell::PACK_MACHINEGUN
                    | crate::orders::spell::PACK_MERCHANT
                    | crate::orders::spell::PACK_FISHERMEN
            ),
            crate::orders::Body::Move(_) => index::is_move_family(front.index()) && not_attacking,
            _ => false,
        }
    }

    // ------------------------------------------------------------------
    // §5.2 — the two queries `find_nearby_spot` asks
    // ------------------------------------------------------------------

    /// `Objects::find_collision(x, y, o, who, 0)@0065b1b0` — "is anything
    /// standing on this point" (`docs/COLLISION.md` §5.2).
    ///
    /// A **land** caller is the occupancy grid and nothing else: the
    /// original returns `CollCheck::collide_here` directly, the same probe
    /// a step takes, so the caller's own block is exempt and the parity
    /// filter applies. Sea and air callers walk the 3×3 world cells'
    /// object chains instead and compare *current* positions in unit
    /// cells, Chebyshev, against the sum of the two `coll_size`s.
    pub(crate) fn find_collision(&self, u: usize, at: Pos) -> bool {
        find_collision_steps(
            false,
            self.units[u].kind.domain == crate::attrition::Domain::Land,
            self.coll_size(u),
            || i32::from(self.collide_here(u, ucell(at), false).is_some()),
            || self.chain_hit(u, at, |s, o| ucell(s.units[o].pos)),
        ) != 0
    }

    /// `Objects::find_collision(x, y, o, who, 1)@0065b1b0` — the same
    /// query with its fifth argument **set**, which skips the land
    /// shortcut (`param_5 == 0 && type +0x218 == 0` → `collide_here`) for
    /// every domain: the nine world cells' chains, every other player's
    /// live on-map unit with a block, Chebyshev in unit cells against the
    /// two `coll_size`s at their **current** positions. `do_move@005f7b30`
    /// asks it at the chaser's own spot before it ends a chase
    /// (`5f7f9d` on a building, `5f7f11` on a unit).
    pub fn find_collision_forced(&self, u: usize, at: Pos) -> bool {
        find_collision_steps(
            true,
            false,
            self.coll_size(u),
            || 0,
            || self.chain_hit(u, at, |s, o| ucell(s.units[o].pos)),
        ) != 0
    }

    /// `ObjectsData::find_unit_with_radius(x, y, ·, -1, r_coll, ·,
    /// FILTER_NOT_ME, -1, -1)@00659890` — the query
    /// `UnitType::find_nearby_spot` asks when its `not_o`/`not_who` are
    /// `(-1, -1)`, which is `do_cast`'s and `cast_transport`'s call for the
    /// water a barge is born on (`docs/TRANSPORT.md` §6.1).
    ///
    /// **It is not the pairwise pair.** `find_nearby_spot@0061de70`'s
    /// `bVar17` — the flag that selects `Objects::find_collision` plus
    /// `find_ordered_collision` — is set only when the filter is
    /// `FILTER_NOT_ME`/`CAN_COLLIDE` **and both `not_o` and `not_who` are
    /// non-negative** (`0061deb0`). The `(-1, -1)` form fails it, so the
    /// sweep takes the general path instead: one radius query, and the
    /// *ordered* variant beside it is skipped outright because its guard is
    /// `not_who >= 0`.
    ///
    /// The predicate is a distance rather than a Chebyshev cell overlap:
    /// a spot is taken when some **player's** live, on-map unit answers
    /// `vector_dist(spot − it) <= its big_radius + r_coll`, where `r_coll`
    /// is the asking *type's* own block (`UnitType +0x240`). Gaia is
    /// invisible to it — the cell arm gates on `who < 8` and the whole-array
    /// arm loops the eight players — as it is to the pairwise pair.
    ///
    /// **Which units are asked is the disc's, not the predicate's**
    /// (`00659890`, the arm `-1 < r_coll && circle_radius[k] <=
    /// total_units`, `k = (r_coll + 0x2ff) / 0x300`): the original walks the
    /// chains of the `circle_radius[k]` 768-unit cells around the spot's
    /// own cell, and only when the game has fewer units than that does it
    /// walk every player's object array instead. ~~This took the second
    /// always, "because the predicate's reach never leaves the disc"~~: ring
    /// 0 is the spot's **own cell** alone, so a block-0 type's query
    /// (`r_coll` 0) never sees a unit across a cell line, however near.
    /// The Turks' Attack Sub `1/98` on East Indies' 3833 was re-placed by
    /// `Group::action_move_near`'s different-region arm round its leader,
    /// the Advanced Battleship `1/49` at (1747, 5674): the original took
    /// (1512, 5640), 237 from the battleship but one cell west of it, and
    /// this crate refused it (item 3782, `docs/AI.md` §378.2).
    ///
    /// `exempt` is `FILTER_NOT_ME`'s `not_o`, which the `(-1, -1)` form
    /// has none of and a squad placement (`docs/ARMY.md` §4.3) does.
    pub fn find_unit_with_radius(&self, r_coll: i32, at: Pos, exempt: Option<usize>) -> bool {
        let hit = |o: usize| {
            let u = &self.units[o];
            Some(o) != exempt
                && u.owner < 8
                && u.alive()
                && u.on_map
                && crate::world::vector_dist(at.x - u.pos.x, at.y - u.pos.y)
                    <= self.profile(Obj::Unit(o)).big_radius + r_coll
        };
        let circle = crate::ai_place::circle();
        let ring = ((r_coll + 0x2ff) / 0x300) as usize;
        if r_coll < 0 || ring >= circle.radius.len() || self.total_units() < circle.radius[ring] {
            return (0..self.units.len()).any(hit);
        }
        let c0 = at.cell();
        let width = self.world.width() as usize;
        (0..circle.radius[ring]).any(|i| {
            let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
            if !self.world.contains(c) {
                return false;
            }
            let mut next = self.chain_heads[(c.y as usize) * width + (c.x as usize)];
            while let Some(x) = next {
                next = self.units[x].down;
                if hit(x) {
                    return true;
                }
            }
            false
        })
    }

    /// `ObjectsData::find_unit_ordered_with_radius(x, y, who, r_coll, ·,
    /// filter, not_o, not_who)@00658ef0` — the *ordered* twin of
    /// [`Sim::find_unit_with_radius`], and the second half of the general
    /// path `find_nearby_spot` takes for a **squad** placement
    /// (`docs/ORDERS.md` §10).
    ///
    /// It walks one player's own array — `param_3` is the sweep's
    /// `not_who` — and looks only at units whose current order is one of
    /// the seven that walk somewhere: `MOVE_TO`, `ATTACK_TO`,
    /// `EXPLORE_TO`, `FLEE_TO`, `CHANGE_FORM`, `GROUP_MOVE`,
    /// `GROUP_ATTACK_TO` (`658fad`–`658fce`). The predicate is the same
    /// disc as the position query and against the same `big_radius`
    /// (`659002`), but measured to the unit's `orders_x/orders_y`
    /// (`658fdc`): a spot one of my own units is already walking to is
    /// taken.
    pub fn find_unit_ordered_with_radius(
        &self,
        r_coll: i32,
        at: Pos,
        who: crate::Player,
        exempt: Option<usize>,
    ) -> bool {
        (0..self.units.len()).any(|o| {
            let u = &self.units[o];
            if Some(o) == exempt || u.owner != who || !u.alive() {
                return false;
            }
            // The seven `MoveOrder` kinds, which `orders::index` already
            // names as the move family.
            let walking = self
                .current_order(o)
                .is_some_and(|od| crate::orders::index::is_move_family(od.index()));
            walking
                && crate::world::vector_dist(at.x - u.orders_pos.x, at.y - u.orders_pos.y)
                    <= self.profile(Obj::Unit(o)).big_radius + r_coll
        })
    }

    /// `Objects::find_ordered_collision(x, y, o, who)@0065b440` — "is
    /// anything *walking to* this point". The same 3×3 chain walk for
    /// every domain, but against each other unit's `orders_x`/`orders_y`
    /// rather than where it stands, so a spot another unit has already
    /// been sent to is taken.
    ///
    /// ~~SEAM: the original follows the chain walk with a second pass over
    /// **my own group's** member list, which catches a member ordered next
    /// to the candidate from anywhere on the map; this crate keeps no
    /// `UnitData::group` back-pointer, and every unit in every capture so
    /// far is ungrouped (`docs/COLLISION.md` §9).~~
    ///
    /// **The premise expired.** `docs/COMBAT.md` §17's probe pushes a real
    /// group (run19's block 8187 has the six on `group 65`), and the
    /// second pass is then the difference between a ring walk that lets
    /// six units pile onto one spot and one that spreads them — six
    /// members asking within one frame, each of them already a hundred
    /// tiles from the candidate and so invisible to the 3 × 3 chain walk.
    /// [`Sim::group_of`] is the resolver this crate lacked; until item
    /// 470 the pass read the **last slot pushed** instead, which is where
    /// it stood when the pool was a single slot, and chapter two's three
    /// hoplites were handed one slot between them while it looked at
    /// somebody else's members (`docs/COMBAT.md` §35.1).
    ///
    /// `65b4d4`-`65b58c`: the group is the asker's `+0x80`, it must belong
    /// to the asker's own player, the asker must be **in** its member
    /// list, and then every other member that is alive, on the map and
    /// has a block is tested with the same unit-cell Chebyshev predicate
    /// as the chain walk, against its `orders_x`/`orders_y`.
    pub(crate) fn find_ordered_collision(&self, u: usize, at: Pos) -> bool {
        if self.chain_hit(u, at, |s, o| ucell(s.units[o].orders_pos)) {
            return true;
        }
        // **The asker's own group, not the last one pushed** (`65b4d4`:
        // the group is `unit +0x80`). The single-slot pool had no way to
        // ask, and item 465's [`Sim::group_of`] is the resolver that
        // makes it answerable; item 470 is where it was measured, because
        // chapter two's three hoplites are a group of three and were
        // being handed one slot between them while this pass looked at
        // somebody else's members. `docs/COMBAT.md` §35.1.
        let Some(g) = self.group_of(u) else {
            return false;
        };
        ordered_group_steps(
            Some((g.who, &g.list)),
            u,
            self.units[u].owner,
            self.coll_size(u),
            {
                let c = ucell(at);
                (c.x, c.y)
            },
            |o| {
                if !self.units[o].alive() || !self.units[o].on_map {
                    return None;
                }
                let r = self.coll_size(o);
                let q = ucell(self.units[o].orders_pos);
                (r != 0).then_some((r, (q.x, q.y)))
            },
        )
    }

    /// The walk both share: the nine world cells around `at`, each cell's
    /// `down` chain, every other unit of a **player** (`who < 8`, so
    /// gaia's animals are invisible to it) that is alive, on the map and
    /// has a block; a hit is Chebyshev `<= my coll_size + its coll_size`
    /// in unit cells against whichever position `of` names.
    fn chain_hit(&self, u: usize, at: Pos, of: impl Fn(&Sim, usize) -> Pos) -> bool {
        self.chain_hit_size(self.coll_size(u), Some(u), at, of)
    }

    /// [`Sim::chain_hit`] with the block and the exemption given rather
    /// than read off a unit, so a *type* can ask it.
    fn chain_hit_size(
        &self,
        mine: i32,
        me: Option<usize>,
        at: Pos,
        of: impl Fn(&Sim, usize) -> Pos,
    ) -> bool {
        if mine == 0 {
            return false;
        }
        let c = ucell(at);
        chain_hit_steps(
            (c.x, c.y),
            mine,
            |dx, dy| {
                let p = Pos::new(at.x + dx * UNITS_PER_CELL, at.y + dy * UNITS_PER_CELL);
                self.chain_slot(p).and_then(|s| self.chain_heads[s])
            },
            |o| self.units[o].down,
            |o| {
                if Some(o) == me || self.units[o].owner >= 8 {
                    return None;
                }
                if !self.units[o].alive() || !self.units[o].on_map {
                    return None;
                }
                let r = self.coll_size(o);
                if r == 0 {
                    return None;
                }
                let q = of(self, o);
                Some((r, (q.x, q.y)))
            },
        )
    }

    // ------------------------------------------------------------------
    // §6 — `Unit::resolve_unit_collision`
    // ------------------------------------------------------------------

    /// **Arm C of the enemy ladder** (`005f9d30:189-252`,
    /// `docs/COLLISION.md` §14), reached when arms A and B have not
    /// returned: my action is an attack, the collider is another
    /// player's, it is not my target, and my target is not in range.
    /// Answers whether it took the collision.
    ///
    /// Read off the listing at `005f9ee0`–`005fa052`:
    ///
    /// - the action's attack is not `mandatory` (`+0x1c`), the current
    ///   order is not a group's (vslot `+0x2c`), and
    ///   `LeaderData::is_enemy(collide_who)`; otherwise nothing;
    /// - **a follower** (`is_captain`, `+0x8e >> 15`, clear): when its
    ///   captain's (`+0xe4`) action is an `ATTACK` whose target exists
    ///   and is in range from where I stand (`is_in_range`, margin off),
    ///   `repath`, `kill_current_order`, and `add_attack_order` on that
    ///   target, `QUEUE_FIRST`, not mandatory;
    /// - **a captain**, while `leaders[who].retargets < 10`
    ///   (`cmp [+0x9f4], 0xa; jge`): `find_new_target(this, NULL, 1)`,
    ///   which searches from where it stands ([`Sim::find_new_target`]).
    ///
    /// - **a captain at ten or more**: unless the type's vslot `+0x10c`
    ///   answers — `is_siege`, `unit_flags & 0x20000` (§13.2 names it) —
    ///   and when the collider is a valid target, `repath`,
    ///   `kill_current_order`, and the collider queued `QUEUE_FIRST`.
    ///
    /// Anything else falls through to the sidestep.
    fn enemy_ladder_arm_c(&mut self, u: usize) -> bool {
        if self.units[u].combat.mandatory
            || matches!(
                self.order_type(u),
                index::GROUP_MOVE | index::GROUP_ATTACK_TO
            )
        {
            return false;
        }
        let who = self.units[u].owner;
        let Ok(them) = u8::try_from(self.units[u].collide_who) else {
            return false;
        };
        if !self.is_enemy(who, them) {
            return false;
        }
        if !self.units[u].captain {
            let cap = self.squad_captain(u);
            if cap == u {
                return false;
            }
            let Some(k) = self.action_of(cap) else {
                return false;
            };
            if !matches!(self.units[cap].orders[k].body, Body::Attack(_)) {
                return false;
            }
            self.update_action(cap);
            let Some(t) = self.units[cap].combat.target else {
                return false;
            };
            if !self.active(t) || !self.is_in_range(Obj::Unit(u), t) {
                return false;
            }
            self.repath(u);
            self.kill_current_order(u);
            self.add_attack_order(u, t, QueuePos::First, false, false);
            return true;
        }
        if self.retargets[usize::from(who)] < 10 {
            self.find_new_target(u, true);
            return true;
        }
        if self.is_siege_unit(u) {
            return false;
        }
        let Some(c) = self.collider_of(u) else {
            return false;
        };
        if !self.valid_target(Obj::Unit(u), Obj::Unit(c)) {
            return false;
        }
        self.repath(u);
        self.kill_current_order(u);
        self.add_attack_order(u, Obj::Unit(c), QueuePos::First, false, false);
        true
    }

    /// What a blocked unit does.
    ///
    /// The original takes the refused point as an argument and then reads
    /// it back off the order's `coll_x`/`coll_y`, which
    /// [`Sim::detect_unit_collision`] has just written; this takes only the
    /// unit.
    pub(crate) fn resolve_unit_collision(&mut self, u: usize) {
        // Step 0, before everything: **an animal gives up**
        // (`docs/COLLISION.md` §6). `resolve_unit_collision@005f9d30`'s
        // first statement is a virtual on slot `+0x30`, which the map
        // folds onto a stub — `Buffer::is_pending_load` (`return 1`) in
        // `Animal::vftable`, `Window::get_button` (`return 0`) in
        // `Unit::vftable` — and the PDB's `LF_ONEMETHOD` list names it
        // **`SubObjectData::is_animal`**, vftable offset 48. When it
        // answers, the body is the `QUEUE_NEW` clear and nothing else:
        // the whole order list and the path go, and none of steps 1–6
        // runs. So a herd animal blocked by its herd-mate stops there for
        // good — no sidestep, no wait, no repath, no cell-centre snap.
        if self.units[u].is_gaia() {
            self.clear_orders(u);
            return;
        }

        let who = self.units[u].owner;
        let other = self.collider_of(u);
        // `005f9d30:96`: `update_action` runs on every path past step 0
        // (step 1, the suicide attacker, is not modelled), and its
        // `orders_x/y` and `dest_angle` writes are the original's own.
        let action = self.update_action(u);
        let attacking =
            action.is_some_and(|a| matches!(self.units[u].orders[a].body, Body::Attack(_)));

        // Step 3: **the enemy ladder** (`docs/COMBAT.md` §48). Taken when
        // the collider is another player's (`005f9d30:98`, `collide_who`
        // against the owner byte) and my action is an attack (vslot
        // `+0x18`, `is_attack`). Two of its three arms:
        //
        // - **A** (`:162-174`): the collider *is* my attack's target. A
        //   group order sets `collide = 1` and stops; anything else forgets
        //   the collider and kills the current order.
        // - **B** (`:176-187`): my attack's target exists and is in range
        //   from where I stand (the listing at `005f9eb2` pushes the
        //   `get_target_order`'s `ox`/`whom` and a zero, so the margin arm
        //   is off). The current order is killed, so the move under the
        //   attack is dropped where the unit stands and the attack fights
        //   next frame. `collide_o`/`collide_who` keep the collider.
        //
        // - **C** (`:189-252`, `docs/COLLISION.md` §14): the attack is
        //   not mandatory, the current order is not a group's, and the
        //   collider's player is an enemy. A follower takes its
        //   captain's attack target when it can strike it from here; a
        //   captain re-searches from where it stands while its leader's
        //   `retargets` is under ten, and past that takes the collider
        //   unless it is siege.
        if i16::from(self.units[u].collide_who) != i16::from(who) && attacking {
            let target = self.units[u].combat.target;
            if let Some(Obj::Unit(t)) = target
                && self.units[t].index == self.units[u].collide_o
                && i16::from(self.units[t].owner) == i16::from(self.units[u].collide_who)
            {
                // vslot `+0x2c`, `is_group`, on the current order.
                // SEAM: `GroupPatrolOrder` and `GroupAttackOrder` are group
                // classes this crate has no order for.
                if matches!(
                    self.order_type(u),
                    index::GROUP_MOVE | index::GROUP_ATTACK_TO
                ) {
                    self.units[u].collide = 1;
                    return;
                }
                self.units[u].collide_o = -1;
                self.units[u].collide_who = -1;
                self.kill_current_order(u);
                return;
            }
            if let Some(t) = target
                && self.is_in_range(Obj::Unit(u), t)
            {
                self.kill_current_order(u);
                return;
            }
            if self.enemy_ladder_arm_c(u) {
                return;
            }
        }

        // Step 2: standing inside my own **flat** gather target's footprint.
        //
        // The type virtual the original asks the target about is `+0x94`,
        // `BuildTypeData::is_flat` — `build_flags & 0x10000000`, named in
        // `docs/CITIES.md` §5 and read here since item 64. It is what
        // fences the kill to a farm: a citizen bumped while standing on
        // the field it works abandons the walk and re-picks a cell, while
        // one bumped on its way to a woodcutter's camp — whose footprint
        // it may also be standing on — falls through to the repath.
        //
        // **Not only against my own player's units** (item 445). The
        // original reaches this test for a foreign collider too, whenever
        // my action is not an attack (`005f9d30:158-160`, `is_attack == 0
        // → LAB_005fa057`). An attack is never a gather, so the collider's
        // owner is not part of the predicate at all.
        if let Some(a) = action
            && let Body::Gather(g) = self.units[u].orders[a].body
            && self
                .buildings
                .get(g.building)
                .is_some_and(|b| b.alive && b.active)
            && self.is_flat(g.building)
            && self.covers_tile(g.building, self.units[u].pos.tile())
        {
            self.kill_current_order(u);
            return;
        }

        let its_order = other.map_or(index::NONE, |o| self.order_type(o));
        // The **move family**, all seven (`005f9d30:261`, `:381`) — not
        // the four plain kinds. Until item 237 a grouped move answered
        // `MOVE_TO`/`ATTACK_TO` here and passed by accident; now it
        // answers `GROUP_MOVE`/`GROUP_ATTACK_TO` and passes by name.
        let its_move = index::is_move_family(its_order);

        // Step 4: the sidestep, only against a unit that is itself moving
        // — and only for a **land** unit. `005f9d30:265` reads
        // `ObjectType +0x218` (`domain`) off the asking unit and requires
        // it to be 0, so a boat blocked by a boat goes straight to the
        // wait and the repath and never pushes a `{cell centre, tol 0,
        // flags 2}` waypoint (the docs-versus-code wave's R2, taken
        // 2026-09-06 with item 236; §6 step 4 had it and the code did
        // not). No capture on disk collides two boats, so this is the
        // decompile's word and not a run's.
        if self.units[u].kind.domain == Domain::Land
            && its_move
            && self.units[u].collide_o >= 0
            && self.units[u]
                .path
                .last()
                .is_none_or(|t| t.flags & path_flag::SIDESTEP == 0)
            && let Some(mo) = self.current_move(u)
            && self.sidestep(u, mo)
        {
            return;
        }

        self.units[u].collide = self.units[u].collide.saturating_add(1);
        self.units[u].collide_frame = self.frame;

        // Step 5: wait for it, when the thing in the way is going
        // somewhere and neither of us has given up.
        //
        // **A transport cast is the eighth kind** (item 1410,
        // `005fa4f4`–`005fa559`): past the move family the listing tests
        // `order_type == CAST_SPELL` and the spell's `+0x20` against
        // `0x28a`, and a hit joins the same wait. A unit that stands
        // casting its boat's spell is something to wait for, not to walk
        // around — step 4's sidestep keeps the seven.
        let its_transport_cast = its_order == index::CAST_SPELL
            && other.is_some_and(|o| {
                matches!(
                    self.current_order(o).map(|x| x.body),
                    Some(Body::Cast(c)) if c.spell == crate::orders::spell::TRANSPORT
                )
            });
        if (its_move || its_transport_cast) && its_order != index::FLEE_TO {
            let cap = if self.repaths[who as usize] > 3 {
                0x80
            } else {
                0x20
            };
            let o = other.expect("a named order implies a named unit");
            let on_me = self.units[o].collide_o == self.units[u].index
                && self.units[o].collide_who == self.units[u].owner as i8;
            // `005fa5f0`: pass when the collider is not waiting, **or**
            // when its own `collide_o` is non-negative and the unit it
            // names is not waiting either. A collider that is waiting and
            // names nobody refuses the wait — the `-1 < sVar13` half, and
            // the crate used to grant it (2026-09-05, R8). Nothing here
            // reads my own flag.
            // The chain reads the latest occupant of the slot, even after
            // disembarkation closed it. A live-only lookup loses its wait
            // bit and wrongly permits waiting instead of recovery (§22).
            let its_chain_blocks = self.units[o].collide_o < 0
                || self
                    .units
                    .iter()
                    .rev()
                    .find(|v| {
                        i16::from(v.owner) == i16::from(self.units[o].collide_who)
                            && v.index == self.units[o].collide_o
                    })
                    .is_some_and(|v| v.waiting_on);
            //
            // **A gaia collider is always "an enemy"** (`005fa5f0`'s
            // `LeaderData::is_enemy@006ebaa0` on the collider's owner):
            // `diplos` is `int[8]`, so a gaia owner 8 reads `treaties[0]` of
            // *my* leader — 0 for the AI toward the human, as every capture
            // prints — and the gaia leader's own `diplos[who]` is never
            // written. A walker blocked by a walking animal repaths rather
            // than waits (item 1561, Great Sahara's 1247).
            let enemy_owner = self.units[o].is_gaia() || self.at_war_with(who, self.units[o].owner);
            if i32::from(self.units[o].collide) < cap
                && i32::from(self.units[u].collide) < cap
                && !enemy_owner
                && !on_me
                && !(self.units[o].waiting_on && its_chain_blocks)
            {
                self.units[u].waiting_on = true;
                return;
            }
        }

        self.collide_repath(u, other);
    }

    /// The unit named by `collide_o`/`collide_who`, if it is still alive.
    pub(crate) fn collider_of(&self, u: usize) -> Option<usize> {
        if self.units[u].collide_o < 0 || self.units[u].collide_who < 0 {
            return None;
        }
        self.unit_by_o(self.units[u].collide_who as Player, self.units[u].collide_o)
    }

    /// The current order's `MoveOrder`, copied.
    pub(crate) fn current_move(&self, u: usize) -> Option<MoveOrder> {
        match self.current_order(u)?.body {
            Body::Move(m) => Some(m),
            _ => None,
        }
    }

    /// §6 step 4. `true` when a perpendicular cell was free and pushed.
    fn sidestep(&mut self, u: usize, mo: MoveOrder) -> bool {
        let c = ucell(mo.coll.unwrap_or(self.units[u].pos));
        let mine = ucell(self.units[u].pos);
        let (dx, dy) = (c.x - mine.x, c.y - mine.y);
        let tries = if dx.abs() == dy.abs() {
            [Pos::new(c.x, c.y - dy), Pos::new(c.x - dx, c.y)]
        } else {
            [Pos::new(c.x - dy, c.y - dx), Pos::new(c.x + dy, c.y + dx)]
        };
        for t in tries {
            let p = ucell_centre(t);
            if self.invalid_loc(u, p.tile(), false, false, false, false, false) != 0 {
                continue;
            }
            if self.detect_quick(u, p, false) {
                continue;
            }
            self.units[u].path.push(PathData {
                to: p,
                tolerance: 0,
                flags: path_flag::SIDESTEP,
            });
            if let Some(front) = self.units[u].orders.front_mut()
                && let Some(m) = front.move_mut()
            {
                m.has_waypoint = true;
                m.waypoint = p;
            }
            // **`UnitData::tolerance` is deliberately not written here**
            // (§8.7). `LAB_005fa37a` in `resolve_unit_collision@005f9d30`
            // makes exactly three stores — the `Stack<PathData>::push` and
            // the order's `+0x2c`/`+0x30` — and the entry's own
            // `tolerance 0` reaches the unit only through `do_move`'s
            // `dest == 0` take, which this waypoint never goes through
            // because `dest` is already 1 by the time `move_step` runs.
            // So the sidestep is walked *under the current leg's*
            // tolerance, and `move_step`'s post-step Manhattan test
            // retires it on the first successful step. Zeroing it here
            // made this crate walk the remainder as a second step and cost
            // a frame per collision cycle — East Indies' word.
            return true;
        }
        false
    }

    /// §6 step 6: unwind the path stack to something worth walking, snap
    /// onto the cell centre, and plan on the 48-grid.
    fn collide_repath(&mut self, u: usize, other: Option<usize>) {
        if self.units[u].path.is_empty() {
            return;
        }
        let who = self.units[u].owner as usize;
        let repaths = self.repaths[who];
        let collide = i32::from(self.units[u].collide);
        let o = i32::from(self.units[u].index);
        let n = if repaths < 4 {
            collide
        } else {
            if (o + collide) & 3 != 0 {
                return;
            }
            collide.div_euclid(4)
        };
        if repaths >= 0x10 || (repaths >= 8 && (o + n) & 0xf != 0) {
            return;
        }
        self.repaths[who] = repaths + 1;
        // Pop until an entry is worth keeping, then push the last back.
        let mut kept = PathData {
            to: self.units[u].pos,
            tolerance: 0,
            flags: path_flag::FINAL,
        };
        while let Some(e) = self.units[u].path.pop() {
            kept = e;
            let mut blocked = false;
            if e.tolerance >= 0x60 || e.flags & path_flag::SIDESTEP == 0 {
                let mask = self.world.tile_mask(e.to.tile());
                blocked =
                    mask & UNWIND_REFUSES != 0 || self.collide_here(u, ucell(e.to), true).is_some();
            }
            let keep = e.flags & path_flag::FINAL != 0
                || (e.tolerance >= 0x60 && e.flags & path_flag::SIDESTEP == 0 && !blocked);
            if keep {
                break;
            }
        }
        self.units[u].path.push(kept);
        let from = self.units[u].pos;
        let snap = ucell_centre(ucell(from));
        self.set_new_location(u, snap, true);
        self.moved_to(u, from, true);
        // **`anti` is a conjunction, not my half of one** (the
        // docs-versus-code wave's R1, taken 2026-09-06 with item 236).
        // `005f9d30:473-480` tests `update_action(this)`'s order type
        // against `ATTACK` **and** `UnitData::action_type(the collider)`
        // against `ATTACK`; this crate asked only the first, so it planned
        // an anti-unit path against every collider an attacking unit met.
        // The second conjunct is read off the unit `resolve` was handed —
        // the same one the pause roll below re-reads — not off `collide_o`
        // again.
        let anti = self
            .action_of(u)
            .is_some_and(|a| self.units[u].orders[a].index() == index::ATTACK)
            && other.is_some_and(|o| {
                self.action_of(o)
                    .is_some_and(|a| self.units[o].orders[a].index() == index::ATTACK)
            });
        let r = self.find_upath(u, anti);
        // **`dest = 0`, on both arms** — `005f9d30`'s last two blocks are
        // the same store through `update_order()->get_move_order()`, and
        // the only thing the non-zero one adds is the pause roll. The
        // waypoint is *not* taken here: it is taken by `do_move`'s own
        // `dest == 0` block next frame, which re-reads the top, clears
        // `unit_masks & 8` and runs the leg's arrival test.
        //
        // Taking it here instead left run53's `1/7` standing on its own
        // waypoint with `dest` set, so the arrival test never ran and the
        // frame fell through to the grid roll the original does not spend
        // (item 204).
        if let Some(front) = self.units[u].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.has_waypoint = false;
        }
        if r != 0 {
            self.collide_pause(u, other);
        }
    }

    /// The tail of §6 step 6, and the mechanic's only draw.
    ///
    /// The original re-reads the unit it was handed at entry — not
    /// `collide_o` again — and rolls only when that unit names **me** back
    /// and is not already waiting on somebody: a stagger for a head-on
    /// pair, so the two do not step off together and collide again.
    fn collide_pause(&mut self, u: usize, other: Option<usize>) {
        let Some(o) = other else { return };
        if self.units[o].collide_o != self.units[u].index
            || self.units[o].collide_who != self.units[u].owner as i8
            || self.units[o].waiting_on
        {
            return;
        }
        self.mark(SITE_PAUSE);
        let pause = self.rng.roll() % 9 + 1;
        if let Some(front) = self.units[u].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.pause = pause;
        }
    }
}

#[cfg(test)]
mod tests;
