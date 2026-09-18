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
use crate::orders::{Body, MoveOrder, Order, PathData, index, path_flag};
use crate::world::{Cell, Pos, UNITS_PER_CELL, tile};
use crate::{Player, Sim};

/// A unit cell, `0x30` position units — the grid the index is keyed on.
pub const UNITS_PER_UCELL: i32 = 0x30;
/// Unit cells to a world cell, each way.
pub const UCELLS_PER_CELL: i32 = UNITS_PER_CELL / UNITS_PER_UCELL;
/// `Constants::unit_block_radius` — the scale `BLOCK_RADIUS` is loaded at.
const UNIT_BLOCK_RADIUS: i32 = 48;
/// The tile bits the path unwind refuses: blocked, or next to blocked.
const UNWIND_REFUSES: u16 = tile::BLOCKED | tile::BAD_PATH;

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

    /// Whether a unit cell is occupied.
    pub fn get(&self, x: i32, y: i32) -> bool {
        self.index(x, y)
            .is_some_and(|i| self.bits[i / 64] & (1u64 << (i % 64)) != 0)
    }

    fn set(&mut self, x: i32, y: i32, on: bool) {
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
    /// disc. Idempotent.
    pub(crate) fn coll_add(&mut self, u: usize) {
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        self.coll_paint(u, self.units[u].pos, true);
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
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        // `avg_speed` (`GuyData +0x84`) is `Guy::move`'s running quarter of
        // `last_speed`, so it reaches zero a few frames after the unit stops
        // and is non-zero for every frame of a walk.
        if self.units[u].movement.body.avg_speed != 0 {
            return;
        }
        if (self.frame + i64::from(self.units[u].index)).rem_euclid(64) != 0 {
            return;
        }
        self.coll_paint(u, self.units[u].pos, true);
    }

    /// `Object::remove_from_world`'s half — the clear pass with nowhere to
    /// move to.
    pub(crate) fn coll_remove(&mut self, u: usize) {
        self.coll_paint(u, self.units[u].pos, false);
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
        for (dx, dy) in spiral(size) {
            let p = Pos::new(c.x + dx, c.y + dy);
            if self.coll_region_ok(region, p) {
                self.coll.set(p.x, p.y, on);
            }
        }
    }

    /// `CollCheck::move_unit(from, to, size)`: clear the cells around
    /// `from` that are further than `size` from `to`, then set those around
    /// `to` that are further than `size` from `from`. Each pass uses its
    /// own end's tile region.
    fn coll_move(&mut self, u: usize, from: Pos, to: Pos) {
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
    pub(crate) fn chain_add(&mut self, u: usize) {
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

    /// `Object::remove_from_world`'s chain half.
    pub(crate) fn chain_remove(&mut self, u: usize) {
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
                self.coll_add(u);
            }
            if on_map {
                self.coll_move(u, from, to);
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
        let mine = ucell(self.units[u].pos);
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
                    return self
                        .leading_edge(size, |k| Pos::new(at.x + dx * size, at.y - size + k));
                }
                if dy.abs() == 1 {
                    return self
                        .leading_edge(size, |k| Pos::new(at.x - size + k, at.y + dy * size));
                }
            }
        }
        let on_map = self.units[u].on_map;
        for (dx, dy) in spiral(size) {
            if (dx + size) % 2 != 0 || (dy + size) % 2 != 0 {
                continue;
            }
            let p = Pos::new(at.x + dx, at.y + dy);
            // A cell inside my own block is mine to stand in.
            if on_map && (p.x - mine.x).abs() <= size && (p.y - mine.y).abs() <= size {
                continue;
            }
            if self.coll.get(p.x, p.y) {
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
    fn leading_edge(&self, size: i32, cell: impl Fn(i32) -> Pos) -> Option<Pos> {
        (0..=size)
            .map(|k| cell(k * 2))
            .find(|p| self.coll.get(p.x, p.y))
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
    /// The two answers come apart twice: for a **crew** figure, which
    /// stands on a track offset a whole cell or more from its leader; and
    /// for guy 0 itself on any frame its body has not caught up with the
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
        self.units[o]
            .guys
            .iter()
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
        let c = ucell(at);
        c != ucell(self.units[u].pos) && self.collide_here(u, c, nocoll).is_some()
    }

    /// The full form: find the cell, name the unit, apply the exemption
    /// ladder and the corner rule, and record the result. `Some(other)` is
    /// a hard collision.
    ///
    /// The bookkeeping the original does on the way out — clearing
    /// `collide_o`/`collide_who`, ageing `collide`, clearing the wait flag
    /// — happens here too, because every path out passes through it.
    pub(crate) fn detect_unit_collision(&mut self, u: usize, at: Pos) -> Option<usize> {
        if self.detect_gates(u) {
            let c = ucell(at);
            if c != ucell(self.units[u].pos)
                && let Some(cell) = self.collide_here(u, c, false)
                && let Some(other) = self.name_collider(u, at, c, cell)
            {
                self.units[u].collide_o = self.units[other].index;
                self.units[u].collide_who = self.units[other].owner as i8;
                self.units[u].collide_guy = 0;
                if let Some(front) = self.units[u].orders.front_mut()
                    && let Some(m) = front.move_mut()
                {
                    m.coll = Some(at);
                }
                return Some(other);
            }
        }
        self.units[u].collide_o = -1;
        self.units[u].collide_who = -1;
        if self.units[u].collide_frame < self.frame - 5 {
            self.units[u].collide = 0;
        }
        self.units[u].waiting_on = false;
        None
    }

    /// §4.3: walk the 3×3 world cells around the proposal and their `down`
    /// chains for a unit whose block covers `cell`, and decide whether it
    /// is a hard collision. `None` means soft, or nothing that counts.
    fn name_collider(&mut self, u: usize, at: Pos, at_cell: Pos, cell: Pos) -> Option<usize> {
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
                if !self.unit_is_here(o, cell) {
                    continue;
                }
                if self.soft_collision(u, o, extra) {
                    soft = true;
                    continue;
                }
                let theirs = self.guy_corner(o, cell);
                if will == 0 || (will - theirs).abs() != 4 {
                    return Some(o);
                }
            }
        }
        if soft {
            self.units[u].half_step = true;
        }
        None
    }

    /// How far beyond its own range an attacker's target would still be
    /// from the proposed point — `max(0, attack_dist − range × 0xc0)`, 0
    /// when the action is not an attack.
    fn attack_overreach(&self, u: usize, at: Pos) -> i32 {
        let Some(a) = self.action_of(u) else { return 0 };
        let Body::Attack(_) = self.units[u].orders[a].body else {
            return 0;
        };
        let Some(t) = self.units[u].combat.target else {
            return 0;
        };
        let range = self.max_range_of(Obj::Unit(u));
        let ext = crate::combat::extent(&self.profile(t), matches!(t, Obj::Building(_)));
        let mine = crate::combat::extent(&self.profile(Obj::Unit(u)), false);
        let d = crate::combat::attack_dist(at, self.pos_of(t), mine, ext, false);
        (d - range * 0xc0).max(0)
    }

    /// §4.3's table: the ways another unit in the way is a nudge rather
    /// than a collision.
    ///
    /// SEAM: the `TRADE_ROUTE`/`0xf` and `0xc` arms need action indices
    /// this crate does not carry, and no capture has entered either
    /// (`docs/COLLISION.md` §9). ~~And the group arm needs
    /// `UnitData::group`, which it does not keep~~ — it keeps one now
    /// (`docs/GROUPS.md` §1: an army's members are its group), and
    /// [`Self::same_group_soft`] is that arm.
    fn soft_collision(&self, u: usize, o: usize, extra: i32) -> bool {
        let moving = |v: usize| self.current_order(v).is_some_and(Order::is_move);
        let acting = |v: usize| self.action_of(v).map(|a| self.units[v].orders[a].index());
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
    /// SEAM: `UnitData +0x104`, the suspended pathfinder search, which
    /// this crate does not keep — read as zero, which widens the arm.
    /// SEAM: `0x12`, `CHANGE_FORM`, is an order this crate does not have,
    /// so the gated set is six of the seven here.
    fn same_group_soft(&self, u: usize, o: usize) -> bool {
        if self.units[u].owner != self.units[o].owner {
            return false;
        }
        let (Some(a), Some(b)) = (self.army_of(u), self.army_of(o)) else {
            return false;
        };
        if a != b {
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
        if self.units[u].kind.domain == crate::attrition::Domain::Land {
            return self.collide_here(u, ucell(at), false).is_some();
        }
        self.chain_hit(u, at, |s, o| ucell(s.units[o].pos))
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
    /// The original picks between a disc of 768-unit blocks around the spot
    /// and a walk of every player's object array, on whether the disc holds
    /// fewer cells than the game has units; this takes the second, which
    /// answers the same because the predicate's reach — a block plus a
    /// block, under 400 units — never leaves the disc.
    ///
    /// `exempt` is `FILTER_NOT_ME`'s `not_o`, which the `(-1, -1)` form
    /// has none of and a squad placement (`docs/ARMY.md` §4.3) does.
    pub(crate) fn find_unit_with_radius(
        &self,
        r_coll: i32,
        at: Pos,
        exempt: Option<usize>,
    ) -> bool {
        (0..self.units.len()).any(|o| {
            let u = &self.units[o];
            Some(o) != exempt
                && u.owner < 8
                && u.alive()
                && u.on_map
                && crate::world::vector_dist(at.x - u.pos.x, at.y - u.pos.y)
                    <= self.profile(Obj::Unit(o)).big_radius + r_coll
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
    pub(crate) fn find_unit_ordered_with_radius(
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
    /// [`Sim::pushed_group`] is the pool slot this crate lacked.
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
        let Some((who, members)) = self.pushed_group.as_ref() else {
            return false;
        };
        if *who != self.units[u].owner || !members.contains(&u) {
            return false;
        }
        let mine = self.coll_size(u);
        if mine == 0 {
            return false;
        }
        let c = ucell(at);
        members.iter().any(|&o| {
            if o == u || !self.units[o].alive() || !self.units[o].on_map {
                return false;
            }
            let r = self.coll_size(o);
            if r == 0 {
                return false;
            }
            let q = ucell(self.units[o].orders_pos);
            (c.x - q.x).abs() <= mine + r && (c.y - q.y).abs() <= mine + r
        })
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
        for (dx, dy) in spiral(1) {
            let p = Pos::new(at.x + dx * UNITS_PER_CELL, at.y + dy * UNITS_PER_CELL);
            let Some(s) = self.chain_slot(p) else {
                continue;
            };
            let mut next = self.chain_heads[s];
            while let Some(o) = next {
                next = self.units[o].down;
                if Some(o) == me || self.units[o].owner >= 8 {
                    continue;
                }
                if !self.units[o].alive() || !self.units[o].on_map {
                    continue;
                }
                let r = self.coll_size(o);
                if r == 0 {
                    continue;
                }
                let q = of(self, o);
                if (c.x - q.x).abs() <= mine + r && (c.y - q.y).abs() <= mine + r {
                    return true;
                }
            }
        }
        false
    }

    // ------------------------------------------------------------------
    // §6 — `Unit::resolve_unit_collision`
    // ------------------------------------------------------------------

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

        // Step 2: standing inside my own **flat** gather target's footprint.
        //
        // The type virtual the original asks the target about is `+0x94`,
        // `BuildTypeData::is_flat` — `build_flags & 0x10000000`, named in
        // `docs/CITIES.md` §5 and read here since item 64. It is what
        // fences the kill to a farm: a citizen bumped while standing on
        // the field it works abandons the walk and re-picks a cell, while
        // one bumped on its way to a woodcutter's camp — whose footprint
        // it may also be standing on — falls through to the repath.
        if other.is_some_and(|o| self.units[o].owner == who)
            && let Some(a) = self.action_of(u)
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
        if its_move && its_order != index::FLEE_TO {
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
            let its_chain_blocks = self.units[o].collide_o < 0
                || self
                    .collider_of(o)
                    .is_some_and(|v| self.units[v].waiting_on);
            if i32::from(self.units[o].collide) < cap
                && i32::from(self.units[u].collide) < cap
                && !self.at_war_with(who, self.units[o].owner)
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
    fn collider_of(&self, u: usize) -> Option<usize> {
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
mod tests {
    use super::*;
    use crate::world::{Cell, Terrain, World};
    use crate::{Sim, Tuning, Unit, UnitType};

    /// A flat land world with two `BLOCK_RADIUS 1` walkers on it —
    /// `coll_size 1`, the only value any capture has.
    fn pair(a: Pos, b: Pos) -> (Sim, usize, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let ty = sim.add_unit_type(UnitType {
            hits: 40,
            moves: 25,
            combat: crate::combat::Profile {
                block_radius: 48,
                big_radius: 48,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        });
        let make = |sim: &mut Sim, o: i16, at: Pos| {
            let mut u = Unit::new(0, o, at, 40);
            u.ty = Some(ty);
            let i = sim.add_unit(u);
            sim.units[i].movement.speed = 25;
            sim.units[i].movement.turning = crate::movement::Turning {
                type_turn_speed: crate::movement::degrees_to_angle(45).0,
                packed: false,
                instant_from_stop: true,
                wide_limit: false,
            };
            i
        };
        let x = make(&mut sim, 0, a);
        let y = make(&mut sim, 1, b);
        (sim, x, y)
    }

    /// §2's region gate is `get_tregion` of the *figure's tile*, and for
    /// a boat lying on the water half of a coastal cell that is the cell's
    /// **`region2`** — which never equals the cell's own `region`, so the
    /// boat marks nothing there.
    ///
    /// This is the whole gate: with the plain `region_of` the two agree,
    /// every cell of the disc is written, and a `BLOCK_RADIUS 3` barge
    /// fills its own world cell — which is what pushed a disembarking
    /// passenger four hundred units inland (`docs/TRANSPORT.md` §6.4).
    /// Put `tregion` back in `coll_paint` and the second half fails.
    #[test]
    fn a_boat_on_a_coastal_cell_s_water_marks_no_collision_cells() {
        let mut world = World::new(8, 8);
        let land = world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
        // One coastal cell: land by `region`, with `region2` naming a sea
        // region and every tile of it surfaced as ocean — the shape a map
        // maker leaves where a shore cuts a cell.
        let sea = land + 1;
        let c = Cell::new(4, 4);
        let mut d = world.cell_data(c);
        d.flags |= crate::world::cell::HALFLAND;
        d.region2 = Some(sea);
        world.set_cell_data(c, d);
        for tx in 16..20 {
            for ty in 16..20 {
                world.set_tile_field(
                    Pos::new(tx, ty),
                    crate::world::tile::SURFACE,
                    crate::world::tile::SURFACE_OCEAN,
                );
            }
        }
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let ty = sim.add_unit_type(UnitType {
            hits: 40,
            combat: crate::combat::Profile {
                block_radius: 48,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        });
        let centre = Pos::new(4 * 768 + 384, 4 * 768 + 384);

        // A land unit on a dry cell marks its disc, which is the control:
        // the gate is not simply refusing everything.
        let dry = Pos::new(384, 384);
        let mut a = Unit::new(0, 0, dry, 40);
        a.ty = Some(ty);
        let a = sim.add_unit(a);
        sim.coll_add(a);
        assert!(
            sim.coll.get(ucell(dry).x, ucell(dry).y),
            "a land unit on its own region's cell is in the index"
        );

        // The boat, on the ocean tile of the coastal cell: `get_tregion` is
        // `region2` and the cell's `region` is the land one, so nothing.
        let mut b = Unit::new(0, 1, centre, 40);
        b.ty = Some(ty);
        let b = sim.add_unit(b);
        sim.coll_add(b);
        let uc = ucell(centre);
        for dy in -1..=1 {
            for dx in -1..=1 {
                assert!(
                    !sim.coll.get(uc.x + dx, uc.y + dy),
                    "the coastal cell's water marks nothing at ({dx}, {dy})"
                );
            }
        }
    }

    /// **A snap onto the point the unit already holds still seats the
    /// crew** — `005f8d20` has no early return for an unchanged position,
    /// and `crates/sim` had one until item 210 (`docs/MOVEMENT.md`, "Who
    /// writes it, and when").
    ///
    /// Written by making it fail: with the `if from == to { return true }`
    /// back in, the figure stays where it was dragged to.
    #[test]
    fn a_snap_to_the_point_already_held_still_seats_the_crew() {
        let (mut sim, a, _b) = pair(Pos::new(0x1800, 0x1800), Pos::new(0x4800, 0x4800));
        // One crew figure with a track offset, seated, and then dragged
        // away — a figure that is still walking after its leader.
        sim.units[a].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(2)];
        sim.art.tracks.insert(2, (-48, -192));
        sim.seat_guys(a);
        let seated = sim.units[a].guys[1].follow.expect("a tracked crew guy").des;
        let behind = Pos::new(seated.x - 300, seated.y - 300);
        sim.units[a].guys[1].follow.as_mut().unwrap().body.pos = behind;
        assert_ne!(behind, seated);

        let held = sim.units[a].pos;
        assert!(sim.set_new_location(a, held, true));
        let f = sim.units[a].guys[1].follow.expect("still tracked");
        assert_eq!(f.body.pos, seated, "the figure is put on its offset");
        assert_eq!(f.des, seated, "which is also where it is told to be");
        assert_eq!(
            f.facing, sim.units[a].movement.facing,
            "with its leader's facing written outright"
        );
        assert_eq!(sim.units[a].pos, held, "and the unit has not moved");
    }

    /// The index is written when a unit is added and moved when it walks —
    /// §2, and the reason a fresh scan of the unit list is not the same
    /// thing.
    #[test]
    fn the_index_follows_the_unit_and_is_not_refcounted() {
        let (sim, a, _b) = pair(Pos::new(0x1800, 0x1800), Pos::new(0x4800, 0x4800));
        let c = ucell(sim.units[a].pos);
        for (dx, dy) in spiral(1) {
            assert!(sim.coll.get(c.x + dx, c.y + dy), "the whole 3×3 block");
        }
        // Two cells over: the discs overlap on one column, and the mover
        // clears it on the way out even though the other unit is still
        // standing on it. That is the original's behaviour, kept.
        let d = Pos::new(c.x + 2, c.y);
        let (mut sim2, a2, _b2) = pair(ucell_centre(c), ucell_centre(d));
        assert!(sim2.coll.get(c.x + 1, c.y), "shared by both discs");
        sim2.set_new_location(a2, ucell_centre(Pos::new(c.x - 4, c.y)), false);
        assert!(
            !sim2.coll.get(c.x + 1, c.y),
            "the mover cleared a cell its neighbour still occupies"
        );
        let _ = &sim;
    }

    /// §4 again, and the half of it that is a **write into the order**:
    /// the probe records the point it refused in `coll_x`/`coll_y`, and
    /// `move_step`'s own stores must not put the stale pair back.
    ///
    /// The original steps on a pointer to the move order, so the write is
    /// simply there for everything downstream; this crate steps on a copy
    /// and has to take it back. The arm that shows the difference is the
    /// **blocked stand while a turn is still owed** (§5): it stores the
    /// move and returns without stepping, so a copy written back over the
    /// probe's is the whole of the bug. run10's `1/1` is the case — on the
    /// original's frame 792 it stands mid-turn with
    /// `coll_x/coll_y (40539, 18258)` and this crate held the pair it had
    /// refused fifteen frames earlier.
    #[test]
    fn a_blocked_stand_keeps_the_point_the_probe_refused() {
        // Facing north, ordered sixty degrees round to the east: one
        // frame's turn leaves fifteen degrees owed, which is under
        // `move_step`'s forty-five and so still takes the step — into the
        // unit standing on the next cell up and to the right.
        let a = Pos::new(30 * 0x30 + 40, 30 * 0x30 + 8);
        let b = ucell_centre(Pos::new(31, 29));
        let (mut sim, x, y) = pair(a, b);
        sim.units[x].movement.turning.instant_from_stop = false;
        sim.units[x].movement.facing = crate::movement::Angle(0);
        sim.order_move(x, Pos::new(a.x + 866, a.y - 500));
        sim.tick();

        assert_eq!(sim.units[x].pos, a, "it stood: the turn was still owed");
        assert_eq!(
            sim.units[x].collide_o, sim.units[y].index,
            "and it named the unit it would have walked into"
        );
        let coll = sim
            .current_move(x)
            .and_then(|m| m.coll)
            .expect("the refused point survives the blocked stand's own store");
        assert_eq!(coll, Pos::new(1497, 1431), "the point the step proposed");
        assert_ne!(ucell(coll), ucell(a), "and it is not the unit's own cell");
    }

    /// **§2.2 — the sixty-fourth frame puts back what the walker took.**
    ///
    /// The index is not refcounted, so a unit leaving a cell clears the
    /// bits another unit is still standing on: `A` stands at unit cell
    /// `(10, 10)` with a block of `9..=11` square, `B` steps from
    /// `(11, 11)` to `(12, 12)`, and its clear pass erases `(10, 10)`,
    /// `(10, 11)` and `(11, 10)` — three corners of a block whose owner
    /// never moved. `Guy::process@005e0230` is what puts them back, on the
    /// frames where `(frame + o) % 64 == 0`.
    ///
    /// Both halves are asserted, because the first is what makes the
    /// second a check rather than a tautology: **the hole is real** on the
    /// frame after the step, and **gone** by the repaint. Delete the call
    /// in `process_movement` and the second half fails; the run69 diff
    /// fails with it, which is where this was found (`docs/PATHFINDER.md`
    /// §17).
    #[test]
    fn the_sixty_fourth_frame_puts_back_the_block_a_walker_cleared() {
        let a = ucell_centre(Pos::new(10, 10));
        let b = ucell_centre(Pos::new(11, 11));
        let (mut sim, x, y) = pair(a, b);
        assert_eq!(sim.units[x].index, 0, "A's object number sets the phase");
        for c in [Pos::new(10, 10), Pos::new(10, 11), Pos::new(11, 10)] {
            assert!(sim.coll.get(c.x, c.y), "A's block starts whole at {c:?}");
        }

        // Off the phase, so the repaint cannot run before the hole is
        // looked at: A repaints on frames divisible by 64.
        while (sim.frame + i64::from(sim.units[x].index)).rem_euclid(64) == 0 {
            sim.tick();
        }
        sim.set_new_location(y, ucell_centre(Pos::new(12, 12)), true);
        let holes = [Pos::new(10, 10), Pos::new(10, 11), Pos::new(11, 10)];
        for c in holes {
            assert!(
                !sim.coll.get(c.x, c.y),
                "B's clear pass took {c:?} out of A's block — the index is not \
                 refcounted, and this is the hole the repaint exists for"
            );
        }

        // A stood still throughout, so its `avg_speed` is zero and the
        // next phase frame re-marks the whole disc.
        for _ in 0..64 {
            sim.tick();
        }
        for c in holes {
            assert!(
                sim.coll.get(c.x, c.y),
                "the sixty-fourth frame put {c:?} back"
            );
        }
    }

    /// **§2.3 — crossing a world cell puts back what the walker took**,
    /// and it is the only healer a unit on the march ever meets.
    ///
    /// `Unit::set_new_location` brackets the coordinate write with
    /// `Object::remove_from_world` and `Object::add_to_world` on a
    /// **world**-cell change, and the two walk the same disc around the
    /// figure's own — still unmoved — point, one clearing and one setting.
    /// So the pair sets the whole old block, and `move_unit` then carries a
    /// healed disc onto the new cell. §2.2's sixty-fourth frame cannot do
    /// this job: it asks `avg_speed == 0`.
    ///
    /// Both halves are asserted, and the second is what makes the first a
    /// check: the **same** puncture and the **same** one-cell step leave
    /// the hole standing when the step does not cross a world cell. Delete
    /// the `coll_add` in [`Sim::set_new_location`] and the first half
    /// fails; make it unconditional and the second does.
    #[test]
    fn a_world_cell_crossing_heals_the_block_a_neighbour_punctured() {
        // `A` at unit cell `(ax, 10)`, `B` at `(ax + 2, 11)`, whose block
        // covers `A`'s eastern column. `B` walks two cells south and its
        // clear pass takes the row `y = 10` with it — `(ax + 1, 10)` is
        // `A`'s, and `A` has not moved.
        let punch = |ax: i32| {
            let (mut sim, a, b) = pair(
                ucell_centre(Pos::new(ax, 10)),
                ucell_centre(Pos::new(ax + 2, 11)),
            );
            assert!(
                sim.coll.get(ax + 1, 10),
                "A's block starts whole at ({}, 10)",
                ax + 1
            );
            sim.set_new_location(b, ucell_centre(Pos::new(ax + 2, 13)), true);
            assert!(
                !sim.coll.get(ax + 1, 10),
                "B's clear pass did not punch ({}, 10) out of A's block",
                ax + 1
            );
            // One cell east. `move_unit` re-sets only the column the block
            // is entering, so nothing but the crossing can put this hole
            // back — it is `A`'s new **centre**.
            sim.set_new_location(a, ucell_centre(Pos::new(ax + 1, 10)), true);
            sim.coll.get(ax + 1, 10)
        };
        // 15 → 16 crosses: sixteen unit cells to a world cell, so the
        // object is re-seated and the disc repainted on the way.
        assert!(
            punch(15),
            "the world-cell crossing did not repaint A's own disc"
        );
        // 5 → 6 does not, and the same hole is still standing.
        assert!(
            !punch(5),
            "a step inside one world cell repainted the disc — the heal is \
             `Unit::set_new_location`'s `local_10`, not every move"
        );
    }

    /// §4: a unit walking into another one detects it, names it, and
    /// records the refused point.
    #[test]
    fn a_walker_detects_the_unit_in_front_and_names_it() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        let mut hit = None;
        for _ in 0..40 {
            sim.tick();
            if sim.units[x].collide_o >= 0 {
                hit = Some(sim.units[x].collide_o);
                break;
            }
        }
        assert_eq!(hit, Some(sim.units[y].index), "it names the unit in front");
        assert_eq!(sim.units[x].collide_who, 0);
        assert_eq!(sim.units[x].collide_guy, 0);
        assert!(sim.units[x].collide >= 1, "the counter moved");
        assert!(
            sim.current_move(x).and_then(|m| m.coll).is_some(),
            "`coll_x`/`coll_y` carry the refused point"
        );
    }

    /// §6 step 6: the recovery snaps the walker onto its own cell centre
    /// and re-plans on the 48-grid, and the plan goes **round** the unit in
    /// the way rather than through it — which is `valid_ucoord`'s half of
    /// the mechanic (`docs/PATHFINDER.md` §6).
    #[test]
    fn the_recovery_snaps_to_the_cell_centre_and_paths_around() {
        let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        sim.order_move(x, goal);
        let mut snapped = false;
        for _ in 0..60 {
            sim.tick();
            if sim.units[x].collide_frame != 0 {
                snapped = true;
                break;
            }
        }
        assert!(snapped, "the walk reached the unit in front");
        assert_eq!(
            sim.units[x].pos,
            ucell_centre(ucell(sim.units[x].pos)),
            "snapped onto its own cell centre"
        );
        assert!(
            sim.units[x].path.len() > 1,
            "the 48-grid plan is on the stack: {:?}",
            sim.units[x].path
        );
        assert!(
            sim.units[x]
                .path
                .iter()
                .any(|p| p.flags & path_flag::SIDESTEP != 0),
            "and its waypoints carry the unit grid's flag"
        );
        // It gets there, and it never walks over the unit standing still.
        let stood = sim.units[y].pos;
        for _ in 0..400 {
            sim.tick();
            assert!(
                (sim.units[x].pos.x - stood.x).abs() > 0x30
                    || (sim.units[x].pos.y - stood.y).abs() > 0x30,
                "walked through it at {:?}",
                sim.units[x].pos
            );
        }
        assert!(sim.units[y].pos == stood, "the other one never moved");
    }

    /// §6 step 0, the twin of the test above: **an animal takes none of
    /// it.** `resolve_unit_collision`'s first statement is
    /// `SubObjectData::is_animal` (vftable offset 48, from the PDB's
    /// `LF_ONEMETHOD` list — the map folds both overrides onto trivial
    /// stubs and cannot name it), and when it answers, the body is the
    /// `QUEUE_NEW` clear: the order list and the path go, and steps 1–6
    /// never run.
    ///
    /// The same walk, the same blocker, the same frame — and where a
    /// player's unit snaps onto its cell centre and paths around, gaia's
    /// stops dead where it stood, keeps no order, and never moves again.
    /// This is run39's `8/2`, whose herd-mate blocks it on frame 69:
    /// the original stands it at `(28856, 24197)` for the rest of the
    /// capture, and this crate walked it round to the goal
    /// (`docs/SYNC.md` §3.14).
    #[test]
    fn an_animal_drops_its_walk_where_it_stands_and_takes_no_step() {
        let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        // The walker is gaia's; everything else is the player's case.
        sim.units[x].owner = 8;
        sim.order_move(x, goal);
        // `collide_frame` is step 5's write and an animal never reaches
        // it; what names the blocker is `detect_unit_collision`, which
        // runs before `resolve_unit_collision` is called at all.
        let mut blocked = None;
        for _ in 0..60 {
            let before = sim.units[x].pos;
            sim.tick();
            if sim.units[x].collide_o >= 0 {
                blocked = Some(before);
                break;
            }
        }
        let stopped = blocked.expect("the walk reached the unit in front");
        assert_eq!(
            sim.units[x].pos, stopped,
            "an animal does not take §6 step 6's cell-centre snap"
        );
        assert_eq!(
            sim.units[x].collide_frame, 0,
            "and step 5's counter is never reached"
        );
        assert!(
            sim.units[x].orders.is_empty(),
            "the whole order list goes: {:?}",
            sim.units[x].orders
        );
        assert!(sim.units[x].path.is_empty(), "and so does the path");
        // And it stays there: no repath, no second attempt at the goal.
        for _ in 0..200 {
            sim.tick();
            assert_eq!(sim.units[x].pos, stopped, "it walked again");
        }
        assert_ne!(stopped, goal, "the goal was never reached");
        assert_eq!(sim.units[y].pos, b, "the blocker never moved");
    }

    /// **The other side of the same coin: an animal is a blocker like any
    /// other, and a crewed walker pays the stand twice** (2026-09-06,
    /// item 241).
    ///
    /// §4.3's chain walk skips itself, non-units and aircraft, and asks
    /// nothing about the owner —`detect_unit_collision@00617060:150-172`
    /// tests `alive`, vfunc `+0x18`, `domain != 2`, `is_on_map` and
    /// `is_here`, and no `who`. [`Sim::chain_hit`]'s `who < 8` fence is
    /// **not** on this path: a step probes [`Sim::collide_here`], the
    /// occupancy bitmask, which [`Sim::coll_paint`] writes for every unit
    /// with a block, gaia's included. So East Indies' `1/20` is blocked
    /// by gaia's animal `8/0` on run85's frame 7448 and the crate models
    /// that, whatever else it does with the frame
    /// (`docs/COLLISION.md` §8.2).
    ///
    /// And it pays it **once per figure**: `Unit::set_anim@00616f40` has
    /// two loops, the squad's `0 .. guy_mark` and the crew's `squad_size
    /// .. guy_num`, and run85's 7448 spends one draw from each —
    /// `Unit::set_anim+0x56` and `+0xb6`, both under
    /// [`crate::anim::SITE_BLOCKED`]. A Merchant is `CREW_SIZE 1`, so the
    /// frame owes two draws and not one.
    #[test]
    fn a_gaia_animal_blocks_a_player_s_walker_and_a_crew_pays_the_stand_twice() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        // The **blocker** is gaia's; the walker is a player's, which is
        // run85's 7448 the way round the animal test above is not.
        sim.units[y].owner = 8;
        for slot in 0..4 {
            sim.art.lengths.insert((1, slot), 40);
        }
        // Two figures, the Merchant's shape: `UBER_SIZE 1, CREW_SIZE 1`.
        sim.units[x].guys = vec![crate::anim::Guy::fresh(1), crate::anim::Guy::fresh(1)];
        sim.trace_phases = true;
        sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        let mut marks = None;
        for _ in 0..60 {
            sim.tick();
            if sim.units[x].collide_o >= 0 {
                marks = Some(sim.phase_marks.clone());
                break;
            }
        }
        let marks = marks.expect("gaia's animal blocked the player's walker");
        assert_eq!(
            (sim.units[x].collide_o, sim.units[x].collide_who),
            (sim.units[y].index, 8),
            "the blocker named is the animal, owner and all"
        );
        let at = marks
            .iter()
            .position(|(l, _)| l == crate::anim::SITE_BLOCKED)
            .expect("the blocked stand is marked on that frame");
        // How many draws the mark covers: walk the LCG from the word it
        // recorded to the next mark's (or the frame's end).
        let after = marks.get(at + 1).map_or(sim.rng.seed, |(_, w)| *w);
        let mut r = crate::combat::Rng::new(marks[at].1);
        let mut drew = 0;
        while r.seed != after && drew < 8 {
            r.get(0, 0xffff);
            drew += 1;
        }
        assert_eq!(
            (r.seed, drew),
            (after, 2),
            "one draw per figure — `Unit::set_anim`'s two loops, run85's \
             `Unit::set_anim+0x56` and `+0xb6` on 7448"
        );
    }

    /// **The snap arm has a collision block of its own, and it resolves
    /// nothing** (`docs/COLLISION.md` §5.4, item 360).
    ///
    /// `Unit::move_step@005faf30` splits on `param_2 < local_28` and
    /// probes on *each* side. The partial step's block is §5 — snap
    /// through, wait on an owed turn, `resolve_unit_collision`, widen the
    /// tolerance — and this crate spent it for both arms until Great
    /// Lakes 9134 said otherwise. The snap's, at `005fb3bd`, clears the
    /// collider the probe just named, consumes the waypoint where the
    /// unit stands, and joins the accepted step's tail.
    ///
    /// The shape here is 9134's without the capture: a walker one step
    /// short of its final waypoint, and a blocker standing on it. The
    /// blocker carries a move order of its own so that `do_move`'s own
    /// waypoint probe (§5.1) takes neither of its two arms — its first
    /// wants an action beneath the move, its second a collider that is
    /// **not** moving — and the leg reaches `move_step` intact, which is
    /// exactly how a formation hop reaches it.
    #[test]
    fn a_blocked_snap_stands_and_eats_its_waypoint_without_resolving() {
        // Neighbouring cell centres, 48 apart, and a walker fast enough to
        // cover that in one step — so the **first** proposal the walk
        // makes is already the snap onto its final waypoint, and §5's
        // partial arm is never reached.
        let a = Pos::new(31 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(32 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        for slot in 0..4 {
            sim.art.lengths.insert((1, slot), 40);
        }
        sim.trace_phases = true;
        sim.units[x].movement.speed = 60;
        // The blocker holds a move order it can never advance, so it is
        // "moving" to §5.1 — whose second arm wants a collider that is
        // *not* — and standing still to everyone else.
        sim.units[y].movement.speed = 0;
        sim.order_move(y, Pos::new(39 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        sim.order_move(x, b);
        let mut hit = None;
        for _ in 0..8 {
            let before = sim.units[x].pos;
            sim.tick();
            if sim
                .phase_marks
                .iter()
                .any(|(l, _)| l == crate::anim::SITE_SNAP_BLOCKED)
            {
                hit = Some(before);
                break;
            }
            assert!(
                !sim.phase_marks
                    .iter()
                    .any(|(l, _)| l == crate::anim::SITE_BLOCKED),
                "the partial step's block fired first: this walk never \
                 reaches the snap"
            );
        }
        let before = hit.expect("the walker snapped into the blocker");
        assert_eq!(
            sim.units[x].pos, before,
            "the snap arm takes no step: the unit stands where it was"
        );
        assert_eq!(
            (sim.units[x].collide_o, sim.units[x].collide_who),
            (-1, -1),
            "`field_0x8a = 0xffff` / `field_0xb3 = 0xff`: the arm clears \
             the collider the probe named"
        );
        // The waypoint is consumed where the unit stands, and this one was
        // the **final** leg, so the pop falls into `move_step`'s own tail:
        // `set_angle`, the pathed bit, `kill_current_order`. A *middle*
        // leg returns 1 instead and the walk resumes next frame from the
        // same place — which is what Great Lakes 9134 is, and what
        // `run53_s_24000_frames_put_the_ceiling_where_run33_did` pins
        // against the original rather than against this crate.
        assert!(
            sim.units[x].path.is_empty() && sim.units[x].orders.is_empty(),
            "the final waypoint is consumed and the order killed: path \
             {:?}, orders {:?}",
            sim.units[x].path,
            sim.units[x].orders
        );
        assert_eq!(
            sim.units[x].collide_frame, 0,
            "`resolve_unit_collision` never ran, so nothing dated the \
             collision — run97's 9135 carries a `collide_frame` nine \
             hundred frames stale for the same reason"
        );
        // `coll_x`/`coll_y` — the point the probe refused, stored into the
        // order — cannot be read here: this waypoint was the final one, so
        // the order it was stored into is gone by the time the tick ends.
        // `great_lakes_9134_is_the_snap_arm_s_blocked_stand` in
        // `rondata::diff` is where that field is pinned, against the
        // original's own `MOVEORDER` record on a middle leg.
    }

    /// **`collide` chooses the grid the re-plan runs on**
    /// (`docs/ORDERS.md` §4.4, `Unit::do_move@005f7b30`'s `field_0x88`
    /// test).
    ///
    /// Past the `% 5` roll the original branches: a unit that has *not*
    /// been colliding drops its loose near waypoints and plans on the
    /// **tile** grid (`find_tpath`, waypoints at `tolerance 0x60`, no
    /// flags); one that has keeps them and plans on the **48** grid
    /// (`find_upath`, `tolerance 0`, `flags 2`) — the finer one, and the
    /// only one that knows units are in the way.
    ///
    /// The state here is run53's `1/7` on frame 5502, hand-built: a unit
    /// standing **on** its own waypoint with the 48-grid plan
    /// `resolve_unit_collision` laid still under it. The straight-line
    /// check finds a top equal to its position, refuses, and the frame
    /// rolls. Taking the tile arm there popped that plan and walked the
    /// unit back into the collider it had just recovered from, frame after
    /// frame, to the end of the capture (item 204).
    #[test]
    fn collide_sends_the_re_plan_to_the_unit_grid_not_the_tile_grid() {
        let start = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let plan = |collide: i16| {
            let (mut sim, x, _y) = pair(start, Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18));
            sim.order_move(x, goal);
            sim.tick();
            // The 48-grid plan, and the unit sitting on the top of it.
            let here = sim.units[x].pos;
            sim.units[x].path.push(PathData {
                to: Pos::new(here.x - 0x30, here.y),
                tolerance: 0,
                flags: path_flag::SIDESTEP,
            });
            sim.units[x].path.push(PathData {
                to: here,
                tolerance: 0,
                flags: path_flag::SIDESTEP,
            });
            sim.units[x].line_ok = false;
            sim.units[x].collide = collide;
            if let Some(front) = sim.units[x].orders.front_mut()
                && let Some(m) = front.move_mut()
            {
                m.has_waypoint = true;
                m.waypoint = here;
            }
            sim.trace_phases = true;
            sim.phase_marks.clear();
            sim.tick();
            assert!(
                sim.phase_marks
                    .iter()
                    .any(|(l, _)| l == crate::orders::SITE_MOVE_GRID),
                "a unit standing on its own waypoint rolls, on both arms: {:?}",
                sim.phase_marks
            );
            sim.units[x].path.clone()
        };

        let tiles = plan(0);
        let units = plan(1);
        assert!(
            tiles.iter().any(|p| p.tolerance == 0x60),
            "the tile grid's waypoints carry its half-tile tolerance: {tiles:?}"
        );
        assert!(
            !tiles.iter().any(|p| p.flags & path_flag::SIDESTEP != 0),
            "and the loose plan under it was dropped: {tiles:?}"
        );
        assert!(
            units
                .iter()
                .any(|p| p.flags & path_flag::SIDESTEP != 0 && p.tolerance == 0),
            "a unit that has been colliding keeps it and plans on the 48 \
             grid: {units:?}"
        );
    }

    /// §6 step 6's last store is **`dest = 0`, and nothing else** — and
    /// what that buys is a frame the original does not spend.
    ///
    /// `005f9d30`'s two closing blocks both clear the order's `+0x10`
    /// through `update_order()->get_move_order()`; the only thing the
    /// successful one adds is the pause roll. So the recovery does **not**
    /// take the waypoint: `do_move`'s own `dest == 0` block does, on the
    /// next frame, and that block clears `unit_masks & 8` and runs the
    /// leg's arrival test.
    ///
    /// Taking it here instead is the difference between two frames. The
    /// top of a fresh `find_upath` plan is the unit's own snapped cell, so
    /// a unit handed that waypoint stands *on* it with `dest` set: the
    /// arrival test never runs, the straight-line check finds a top equal
    /// to its position, and the frame falls through to the grid roll
    /// (`SITE_MOVE_GRID`). run53's `1/7` spent that draw on frame 5502 and
    /// the original spent nothing (item 204).
    #[test]
    fn the_recovery_leaves_the_waypoint_for_do_move_to_take() {
        let a = Pos::new(30 * 0x30 + 0x20, 30 * 0x30 + 0x14);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, _y) = pair(a, b);
        sim.order_move(x, goal);
        sim.trace_phases = true;
        let mut snapped = false;
        for _ in 0..60 {
            sim.tick();
            if sim.units[x].collide_frame != 0 {
                snapped = true;
                break;
            }
        }
        assert!(snapped, "the walk reached the unit in front");
        assert!(
            sim.units[x].path.len() > 1,
            "the 48-grid plan is on the stack: {:?}",
            sim.units[x].path
        );
        let mo = sim.current_move(x).expect("still a move");
        assert!(
            !mo.has_waypoint,
            "the recovery clears `dest`; it does not take the top: {mo:?}"
        );

        // The next frame takes it — and the leg it takes is the snapped
        // cell the unit is already standing on, so the arrival test pops
        // it and the frame ends there. No grid roll.
        sim.phase_marks.clear();
        sim.tick();
        assert!(
            !sim.phase_marks
                .iter()
                .any(|(m, _)| m == crate::orders::SITE_MOVE_GRID),
            "the frame after the snap spends no grid draw: {:?}",
            sim.phase_marks
        );

        // And it gets where it was going rather than oscillating: the
        // pop-and-re-plan pair this used to make was a two-frame livelock
        // that ran to the end of run53's capture.
        let mut arrived = false;
        for _ in 0..400 {
            sim.tick();
            if sim.units[x].orders.is_empty() {
                arrived = true;
                break;
            }
        }
        assert!(
            arrived,
            "the walk finished; it stalled at {:?} with {:?}",
            sim.units[x].pos, sim.units[x].path
        );
    }

    /// §6's tail, the mechanic's **only** draw — and the two guards that
    /// silence it.
    ///
    /// A head-on pair: each names the other, so step 5's "wait for it"
    /// refuses on `on_me` and step 4's sidestep is out because the path
    /// top already carries the sidestep flag. What is left is the repath,
    /// and it ends in `Random::get(0, 0xffff) % 9 + 1` written into the
    /// order's `pause` — the stagger that keeps the two from stepping off
    /// on the same frame and colliding again. One-sided, or against a unit
    /// already waiting, nothing is rolled and the stream does not move.
    #[test]
    fn the_head_on_pair_staggers_itself_and_that_is_the_only_draw() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);

        // The three cases share the setup: `x` walking west into `y`,
        // `y` walking east into `x`, and a sidestep already on the top of
        // `x`'s stack.
        let arrange = |named_back: bool, waiting: bool| {
            let (mut sim, x, y) = pair(a, b);
            sim.order_move(x, goal);
            sim.order_move(y, Pos::new(35 * 0x30 + 0x18, 30 * 0x30 + 0x18));
            sim.tick();
            sim.units[x].path.push(PathData {
                to: Pos::new(29 * 0x30 + 0x18, 30 * 0x30 + 0x18),
                tolerance: 0,
                flags: path_flag::SIDESTEP,
            });
            sim.units[x].collide_o = sim.units[y].index;
            sim.units[x].collide_who = 0;
            let (o, who) = if named_back {
                (sim.units[x].index, 0)
            } else {
                (-1, -1)
            };
            sim.units[y].collide_o = o;
            sim.units[y].collide_who = who;
            sim.units[y].waiting_on = waiting;
            sim.trace_phases = true;
            sim.phase_marks.clear();
            (sim, x)
        };
        // The snap teleports the body, so the recovery re-anims and can
        // roll an idle variant of its own; what the site table cares about
        // is whether the *stagger* is one of the frame's draws.
        let staggered = |sim: &Sim| sim.phase_marks.iter().any(|(m, _)| m == SITE_PAUSE);

        let (mut sim, x) = arrange(true, false);
        sim.resolve_unit_collision(x);
        assert!(
            staggered(&sim),
            "the stagger is a draw: {:?}",
            sim.phase_marks
        );
        let pause = sim.current_move(x).expect("still a move").pause;
        assert!((1..=9).contains(&pause), "`% 9 + 1`, got {pause}");

        // It does not name me back: the repath still happens, and it is
        // silent.
        let (mut sim, x) = arrange(false, false);
        sim.resolve_unit_collision(x);
        assert!(
            !staggered(&sim),
            "one-sided, no draw: {:?}",
            sim.phase_marks
        );
        assert_eq!(sim.current_move(x).expect("still a move").pause, 0);

        // It names me back but is already waiting on somebody: also
        // silent, because it is not going to step off either.
        let (mut sim, x) = arrange(true, true);
        sim.resolve_unit_collision(x);
        assert!(!staggered(&sim), "already waiting: {:?}", sim.phase_marks);
        assert_eq!(sim.current_move(x).expect("still a move").pause, 0);
    }

    /// The throttle of §6 step 6 is a **rate**, not a lifetime count:
    /// `GameDaemon::process_all` halves every player's `repaths` at the top
    /// of each frame and snaps it to zero under three
    /// (`docs/PATHFINDER.md` §8).
    ///
    /// Without the decay the counter only ever climbs, and the fifth
    /// recovery a player ever makes puts it past 4 for the rest of the
    /// game — where `(o + collide) & 3` throws away three collisions in
    /// four. That is what run33's frame 571 was: `1/2` reached the repath
    /// with `repaths` stuck at 5 and was thrown away, so the stagger draw
    /// the original spends was never spent.
    #[test]
    fn the_repath_throttle_halves_every_frame_and_clears_under_three() {
        let (mut sim, _x, _y) = pair(
            Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18),
            Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18),
        );
        sim.repaths[0] = 12;
        sim.repaths[1] = 5;
        sim.tick();
        assert_eq!(
            (sim.repaths[0], sim.repaths[1]),
            (6, 0),
            "halved, and 2 < 3"
        );
        sim.tick();
        assert_eq!(sim.repaths[0], 3, "6 → 3, which survives");
        sim.tick();
        assert_eq!(sim.repaths[0], 0, "3 → 1, which does not");
    }

    /// §4.3's **group** arm: two members of one group pass through each
    /// other, and two units that merely share an owner do not.
    ///
    /// This is what keeps a marching squad from standing blocked on its
    /// own leader every time the formation swings (`docs/ORDERS.md` §8.3);
    /// it was a stated seam until this crate had a group — the army's — to
    /// ask about.
    #[test]
    fn two_members_of_one_group_pass_through_each_other() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        assert!(
            sim.detect_unit_collision(x, into).is_some(),
            "two strangers block"
        );
        // The same two in one army — and so in one group.
        let slot = sim.init_army(0, None);
        sim.army_add_unit(0, slot, x);
        sim.army_add_unit(0, slot, y);
        assert!(
            sim.detect_unit_collision(x, into).is_none(),
            "sharing a group makes it a nudge, not a collision"
        );
        // …unless I am attacking, which is the arm's own first test.
        sim.units[x].orders.push_front(crate::orders::Order {
            flags: crate::orders::flag::ACTION,
            body: crate::orders::Body::Attack(crate::orders::AttackOrder {
                defensive: false,
                def: None,
                in_range: false,
                ever_in_range: false,
                new_ord: true,
            }),
        });
        assert!(
            sim.detect_unit_collision(x, into).is_some(),
            "an attacker gets no exemption"
        );
    }

    /// §4.3: **the corner rule is decided on the blocker's *figures*, not
    /// on the blocker.** `UnitData::is_corner@0060a040` walks
    /// `0 .. guy_mark` and returns the first non-zero
    /// `GuyData::is_corner@005de270`, which measures against that figure's
    /// own `GuyData::x/y`. The test immediately before it —
    /// `UnitData::is_here` — reads the **unit's** `x_internal`, so the
    /// original genuinely mixes the two, and a crew figure standing on a
    /// track offset can turn a hard collision into a slip-past its leader
    /// alone never would.
    ///
    /// The blocker stands on unit cell `(28, 28)`, whose own corner
    /// against the hit cell `(28, 29)` is **0** — a face, not a diagonal —
    /// so the unit-centred reading is a hard collision. Its crew figure
    /// stands a cell west, on `(27, 28)`, for which the same hit cell is
    /// the **SE** corner: `|NW − SE| = |1 − 5| = 4`, the two opposite
    /// diagonals, and the two slip past.
    #[test]
    fn the_corner_rule_reads_the_blocker_s_figures_and_not_the_blocker() {
        let at_cell = |x: i32, y: i32| ucell_centre(Pos::new(x, y));
        let (mut sim, x, y) = pair(at_cell(30, 30), at_cell(28, 28));
        // Guy 0 stands on its unit, which is what leaves the corner at 0.
        sim.units[y].guys = vec![crate::anim::Guy::fresh(1)];
        sim.units[x].guys = vec![crate::anim::Guy::fresh(1)];
        sim.units[y].movement.body.pos = sim.units[y].pos;
        let into = at_cell(29, 30);
        assert_eq!(
            sim.detect_unit_collision(x, into),
            Some(y),
            "guy 0 alone: the hit cell is a face of the blocker, so it is hard"
        );
        // The crew figure, a cell west of its leader and cornered on the
        // same hit cell from the opposite side.
        let mut crew = crate::anim::Guy::fresh(2);
        crew.follow = Some(crate::anim::Follow {
            body: crate::movement::Body {
                pos: at_cell(27, 28),
                ..crate::movement::Body::default()
            },
            des: at_cell(27, 28),
            facing: crate::movement::Angle(0),
            des_angle: crate::movement::Angle(0),
            track: (-48, 0),
        });
        sim.units[y].guys.push(crew);
        assert_eq!(
            sim.detect_unit_collision(x, into),
            None,
            "with the crew figure, `is_corner` answers SE and the two slip past"
        );
    }

    /// §4.1: `safe` — the cooldown a failed 48-grid search buys — turns the
    /// test off entirely while it runs, and `Unit::work` counts it down.
    #[test]
    fn the_safe_cooldown_suppresses_the_test_and_expires() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, _y) = pair(a, b);
        let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        assert!(
            sim.detect_quick(x, into, false),
            "blocked with `safe` clear"
        );
        sim.units[x].safe = 3;
        assert!(!sim.detect_quick(x, into, false), "and clear with it set");
        sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        for _ in 0..3 {
            sim.tick();
        }
        assert_eq!(sim.units[x].safe, 0, "one a frame, in `Unit::work`");
    }

    /// §5: **the blocked stand.** A refused step asks for `CHAR_DEFAULT`
    /// before any of the three give-up tests, so a walker re-rolls its
    /// idle on the frame it is blocked — the draw run14 spends on frames
    /// 122, 184 and 256, [`crate::anim::SITE_BLOCKED`].
    #[test]
    fn a_refused_step_re_rolls_the_idle_before_the_give_up_tests() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, _y) = pair(a, b);
        // A guy with a piece whose idle has a length, so the roll has
        // somewhere to land.
        for slot in 0..4 {
            sim.art.lengths.insert((1, slot), 40);
        }
        sim.units[x].guys = vec![crate::anim::Guy::fresh(1)];
        sim.trace_phases = true;
        sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        let mut marks = None;
        for _ in 0..40 {
            sim.tick();
            if sim.units[x].collide_o >= 0 {
                marks = Some(sim.phase_marks.clone());
                break;
            }
        }
        let marks = marks.expect("a frame on which the step is refused");
        let at = marks
            .iter()
            .position(|(l, _)| l == crate::anim::SITE_BLOCKED)
            .expect("the blocked stand is marked on that frame");
        // And it *drew*: the word moves between this mark and the next.
        let after = marks.get(at + 1).map_or(sim.rng.seed, |(_, w)| *w);
        assert_ne!(marks[at].1, after, "the blocked stand spends a draw");
    }

    /// §6 step 2's fence, and what it is: the target type's `+0x94` is
    /// `BuildTypeData::is_flat` — `build_flags & 0x10000000`, the derived
    /// bit only the Farm, the Oil Well and the Oil Platform carry
    /// (`crate::build::init_final_flags`). So the step that abandons a walk
    /// because the unit is standing inside its own gather target's
    /// footprint fires for a **farmer on its field** and for nothing else.
    /// A woodcutter bumped while standing on its camp's footprint — which
    /// is run10's `1/6` on frame 206 — falls through to step 6 instead, and
    /// this crate read the virtual as `true` and killed its order for ever
    /// until item 64.
    #[test]
    fn only_a_flat_gather_target_abandons_the_walk_where_the_unit_stands() {
        for flat in [true, false] {
            // Near the west edge of its cell, so one step west lands in
            // the next cell along — the same-cell gate (§4.1 step 6) is
            // what a step that stays put would fall out of.
            let a = Pos::new(29 * 0x30 + 4, 30 * 0x30 + 0x14);
            let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
            let (mut sim, x, y) = pair(a, b);
            let bt = sim.add_build_type(crate::build::BuildType {
                ident: if flat {
                    crate::build::Ident::Farm
                } else {
                    crate::build::Ident::Woodcutter
                },
                x_size: 5,
                y_size: 5,
                flags: if flat { crate::build::flags::FLAT } else { 0 },
                ..crate::build::BuildType::default()
            });
            let camp = sim.add_building(0, a, 0);
            sim.buildings[camp].ty = Some(bt);
            assert!(
                sim.covers_tile(camp, sim.units[x].pos.tile()),
                "the walker stands inside the footprint either way"
            );
            assert_eq!(sim.is_flat(camp), flat, "and only one of them is flat");

            // A transit move under the gather, as `do_gather` queues one:
            // the `ACTION` flag belongs to the order beneath, which is the
            // one `action_of` has to find for step 2 to look at all.
            let goal = Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18);
            sim.add_move_order(
                x,
                goal,
                crate::orders::MoveKind::MoveTo,
                crate::orders::QueuePos::New,
                false,
            );
            sim.units[x].orders.push_back(Order {
                flags: crate::orders::flag::ACTION,
                body: Body::Gather(crate::orders::GatherOrder {
                    building: camp,
                    tile: None,
                    wait: 0,
                    goto_build: true,
                    dist_mod: 0,
                    been_there: false,
                }),
            });
            sim.units[x].path.push(PathData {
                to: goal,
                tolerance: 0,
                flags: path_flag::FINAL,
            });

            // The step that is refused: one speed west, into the block of
            // the unit standing still.
            let proposed = Pos::new(sim.units[x].pos.x - 25, sim.units[x].pos.y);
            assert_eq!(
                sim.detect_unit_collision(x, proposed),
                Some(y),
                "the unit in front is named"
            );
            sim.resolve_unit_collision(x);

            let moving = sim
                .current_order(x)
                .is_some_and(|o| matches!(o.body, Body::Move(_)));
            if flat {
                assert!(
                    !moving,
                    "the farmer on its own field abandons the walk: {:?}",
                    sim.units[x].orders
                );
            } else {
                assert!(
                    moving,
                    "the woodcutter on its camp keeps the walk: {:?}",
                    sim.units[x].orders
                );
                assert_eq!(
                    sim.units[x].pos,
                    ucell_centre(ucell(a)),
                    "and takes step 6's snap onto its own cell centre"
                );
                assert!(
                    sim.units[x].path.len() > 1,
                    "and step 6's plan on the 48-grid: {:?}",
                    sim.units[x].path
                );
            }
        }
    }

    /// §5.1's second arm: a **parked** unit on the waypoint widens the
    /// tolerance to three of its `big_radius` instead of killing the move,
    /// so the walker gives up short of it rather than colliding. The kill
    /// arm needs a `GATHER`/`ATTACK`/`BUILD_AT` action under the move and
    /// is tested end to end in `cities_tests`; this is the other branch,
    /// which no capture has entered.
    #[test]
    fn a_parked_unit_on_the_waypoint_widens_the_tolerance() {
        let a = Pos::new(34 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(24 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        // `big_radius` is `ObjectType +0x244` and `block_radius` `+0x240`;
        // they are the same number for every unit in every capture, so
        // give them different ones here and the test says which is read.
        sim.unit_types[0].combat.big_radius = 96;
        assert_eq!(sim.coll_size(y), 1, "still a one-cell block");
        // Straight onto the standing unit, ten cells away.
        sim.order_move(x, b);
        sim.tick();
        assert_eq!(
            sim.units[x].tolerance,
            3 * 96,
            "three of the parked unit's `big_radius`"
        );
        assert_eq!(
            sim.units[x].path.last().map(|p| p.tolerance),
            Some(3 * 96),
            "and the path top carries it"
        );
        for _ in 0..40 {
            sim.tick();
        }
        assert!(
            sim.units[x].orders.is_empty(),
            "the walk ended: {:?}",
            sim.units[x].orders
        );
        let gap = (sim.units[x].pos.x - sim.units[y].pos.x).abs();
        assert!(
            (0x30..=3 * 96).contains(&gap),
            "it stopped short of the parked unit rather than on it: {gap}"
        );
    }

    /// §2.1: the generated spiral is the Chebyshev disc, ring by ring, and
    /// its first nine entries are the compass the pathfinder uses.
    #[test]
    fn the_spiral_is_the_chebyshev_disc_and_ring_two_is_the_odd_one() {
        let s = spiral(7);
        assert_eq!(s.len(), 15 * 15, "radius[7] = (2·7+1)²");
        for r in 0..=7usize {
            let n = (2 * r + 1) * (2 * r + 1);
            let got: std::collections::BTreeSet<_> = s[..n].iter().copied().collect();
            let want: std::collections::BTreeSet<_> = (-(r as i32)..=r as i32)
                .flat_map(|x| (-(r as i32)..=r as i32).map(move |y| (x, y)))
                .collect();
            assert_eq!(got, want, "radius[{r}] is the disc");
        }
        assert_eq!(
            &s[..9],
            &[
                (0, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0)
            ],
            "the compass, as `docs/PATHFINDER.md` §4.2 has it"
        );
        assert_eq!(
            &s[21..25],
            &[(-2, -2), (2, -2), (2, 2), (-2, 2)],
            "ring 2 keeps its corners for last"
        );
    }

    /// §4.2's fast path, and **the reason it is not an optimisation**: it
    /// sweeps the leading edge alone, so it stops at a different first hit
    /// cell than the disc — and §4.3's corner rule is decided on the cell,
    /// so the two probes disagree about whether there is a collision at
    /// all. run66's sim-frame 6570 is the shape, rebuilt here at the
    /// origin: a `coll_size 2` walker stepping one cell north between two
    /// `coll_size 1` neighbours, one of them corner to corner with it and
    /// the other square on its leading edge.
    ///
    /// Written to fail first: with the whole disc the walker names the
    /// square neighbour and refuses the step, which is exactly what East
    /// Indies' word parted on.
    #[test]
    fn the_leading_edge_finds_the_corner_the_disc_walks_past() {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let ty = |sim: &mut Sim, block: i32| {
            sim.add_unit_type(UnitType {
                hits: 40,
                moves: 25,
                combat: crate::combat::Profile {
                    block_radius: block,
                    big_radius: block,
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..UnitType::default()
            })
        };
        let (wide, narrow) = (ty(&mut sim, 96), ty(&mut sim, 48));
        let put = |sim: &mut Sim, o: i16, cell: Pos, t| {
            let mut u = Unit::new(0, o, ucell_centre(cell), 40);
            u.ty = Some(t);
            sim.add_unit(u)
        };
        // The walker's own cell is `(20, 20)` and it proposes `(20, 19)`,
        // so its leading edge is the row `y = 17` at `x` 18, 20 and 22.
        let big = put(&mut sim, 0, Pos::new(20, 20), wide);
        // `(16..=18, 15..=17)` — the corner cell `(18, 17)` and no more.
        let corner = put(&mut sim, 1, Pos::new(17, 16), narrow);
        // `(20..=22, 15..=17)` — the edge's middle cell, and the one the
        // disc reaches first.
        let square = put(&mut sim, 2, Pos::new(21, 16), narrow);
        assert_eq!((sim.coll_size(big), sim.coll_size(corner)), (2, 1));
        assert_eq!(sim.coll_size(square), 1);

        let at = ucell_centre(Pos::new(20, 19));
        assert_eq!(
            sim.collide_here(big, ucell(at), false),
            Some(Pos::new(18, 17)),
            "the leading edge stops at the corner cell"
        );
        // The disc — which `valid_ucoord` and the stack unwind still ask —
        // takes the ring's edge cells before its corners.
        assert_eq!(
            sim.collide_here(big, ucell(at), true),
            Some(Pos::new(20, 17)),
            "the whole disc's first parity cell is the middle of the edge"
        );
        // And the corner rule turns the first into no collision at all:
        // `will_be_corner 1` against `is_corner 5` is the two opposite
        // diagonals, and they pass.
        assert_eq!(
            sim.detect_unit_collision(big, at),
            None,
            "corner to opposite corner: the two slip past"
        );
        assert_eq!(sim.units[big].collide_o, -1, "and nothing is recorded");
    }

    /// §4.2: for a `coll_size 1` unit the parity filter leaves exactly the
    /// four diagonals, which is what makes two 3×3 blocks detect each
    /// other at Chebyshev 2.
    #[test]
    fn the_parity_filter_leaves_the_four_diagonals() {
        let probed: Vec<_> = spiral(1)
            .into_iter()
            .filter(|(dx, dy)| (dx + 1) % 2 == 0 && (dy + 1) % 2 == 0)
            .collect();
        assert_eq!(probed, vec![(-1, -1), (1, -1), (1, 1), (-1, 1)]);
    }

    /// §4.3's corner rule, on the pair run10's frame 122 actually has:
    /// `1/6` proposing cell `(872, 355)` with the hit at `(871, 354)`,
    /// against `1/3` sitting on `(870, 354)`. `1` against `0` is not a
    /// difference of four, so it is a hard collision.
    #[test]
    fn run10_s_frame_122_pair_is_a_hard_collision() {
        let hit = Pos::new(871, 354);
        let will = Sim::corner_of(1, Pos::new(872, 355), hit);
        let theirs = Sim::corner_of(1, Pos::new(870, 354), hit);
        assert_eq!((will, theirs), (1, 0));
        assert!(will != 0 && (will - theirs).abs() != 4, "hard");
        // The opposite-diagonal pair that does slip past.
        assert_eq!(Sim::corner_of(1, Pos::new(870, 354), Pos::new(871, 355)), 5);
        assert_eq!(Sim::corner_of(1, Pos::new(872, 356), Pos::new(871, 355)), 1);
    }
}
