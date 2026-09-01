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
        let region = self.world.tregion(at.tile());
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
        let ra = self.world.tregion(from.tile());
        for (dx, dy) in &disc {
            let p = Pos::new(a.x + dx, a.y + dy);
            if ((p.x - b.x).abs() > size || (p.y - b.y).abs() > size) && self.coll_region_ok(ra, p)
            {
                self.coll.set(p.x, p.y, false);
            }
        }
        let rb = self.world.tregion(to.tile());
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
        if from == to {
            return true;
        }
        let on_map = self.units[u].on_map && self.units[u].alive();
        let cell_change = on_map && from.cell() != to.cell();
        if !self.shore_step(u, from, to, on_map, cell_change) {
            return false;
        }
        if cell_change {
            self.chain_remove(u);
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
        if move_guys {
            self.units[u].movement.body.pos = to;
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
    /// SEAM: `collide_here`'s two leading-edge fast paths — taken when the
    /// proposal is exactly one cell away on one axis — sweep a different
    /// set of cells and are not modelled; the disc is always walked.
    pub(crate) fn collide_here(&self, u: usize, at: Pos) -> Option<Pos> {
        let size = self.coll_size(u);
        if size == 0 {
            return None;
        }
        let mine = ucell(self.units[u].pos);
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

    /// `UnitData::is_here`: does this unit's block cover that unit cell?
    fn unit_is_here(&self, u: usize, cell: Pos) -> bool {
        let size = self.coll_size(u);
        if size == 0 {
            return false;
        }
        let c = ucell(self.units[u].pos);
        (cell.x - c.x).abs() <= size && (cell.y - c.y).abs() <= size
    }

    /// `UnitData::will_be_corner` / `is_corner`: `1, 3, 5, 7` for NW, NE,
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
    pub(crate) fn detect_quick(&self, u: usize, at: Pos) -> bool {
        if !self.detect_gates(u) {
            return false;
        }
        let c = ucell(at);
        c != ucell(self.units[u].pos) && self.collide_here(u, c).is_some()
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
                && let Some(cell) = self.collide_here(u, c)
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
                let theirs = Self::corner_of(self.coll_size(o), ucell(self.units[o].pos), cell);
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
    /// SEAM: only the same-player-attackers arm is modelled. The
    /// `TRADE_ROUTE`/`0xf` and `0xc` arms need action indices this crate
    /// does not carry, and the group arm needs `UnitData::group`, which it
    /// does not keep; no capture has entered any of the three
    /// (`docs/COLLISION.md` §9).
    fn soft_collision(&self, u: usize, o: usize, extra: i32) -> bool {
        let moving = |v: usize| self.current_order(v).is_some_and(Order::is_move);
        let acting = |v: usize| self.action_of(v).map(|a| self.units[v].orders[a].index());
        acting(u) == Some(index::ATTACK)
            && acting(o) == Some(index::ATTACK)
            && self.units[u].owner == self.units[o].owner
            && self.coll_size(u) == 1
            && self.coll_size(o) == 1
            && moving(u)
            && moving(o)
            && extra > 0x300
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
            return self.collide_here(u, ucell(at)).is_some();
        }
        self.chain_hit(u, at, |s, o| ucell(s.units[o].pos))
    }

    /// The same two queries with **no unit behind them** — the shape
    /// `UnitType::find_nearby_spot` asks when its `not_o`/`not_who` are
    /// `(-1, -1)`, which is `cast_transport`'s call for the water its barge
    /// is born on (`docs/TRANSPORT.md` §6). Nothing is excluded, and the
    /// block is the *type's*.
    ///
    /// Only the non-land branch is offered: the caller is a boat's type,
    /// and a land type with no unit behind it has no block of its own for
    /// `collide_here`'s "my own cells are mine" exemption to name.
    pub(crate) fn find_collision_for(&self, block_radius: i32, at: Pos) -> bool {
        let size = block_radius / UNIT_BLOCK_RADIUS;
        self.chain_hit_size(size, None, at, |s, o| ucell(s.units[o].pos))
            || self.chain_hit_size(size, None, at, |s, o| ucell(s.units[o].orders_pos))
    }

    /// `Objects::find_ordered_collision(x, y, o, who)@0065b440` — "is
    /// anything *walking to* this point". The same 3×3 chain walk for
    /// every domain, but against each other unit's `orders_x`/`orders_y`
    /// rather than where it stands, so a spot another unit has already
    /// been sent to is taken.
    ///
    /// SEAM: the original follows the chain walk with a second pass over
    /// **my own group's** member list, which catches a member ordered next
    /// to the candidate from anywhere on the map; this crate keeps no
    /// `UnitData::group` back-pointer, and every unit in every capture so
    /// far is ungrouped (`docs/COLLISION.md` §9).
    pub(crate) fn find_ordered_collision(&self, u: usize, at: Pos) -> bool {
        self.chain_hit(u, at, |s, o| ucell(s.units[o].orders_pos))
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
        let its_move = matches!(
            its_order,
            index::MOVE_TO | index::ATTACK_TO | index::EXPLORE_TO | index::FLEE_TO
        );

        // Step 4: the sidestep, only against a unit that is itself moving.
        if its_move
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
            let its_target_waits = self
                .collider_of(o)
                .is_some_and(|v| self.units[v].waiting_on);
            if i32::from(self.units[o].collide) < cap
                && i32::from(self.units[u].collide) < cap
                && !self.at_war_with(who, self.units[o].owner)
                && !on_me
                && !(self.units[o].waiting_on && its_target_waits)
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
            if self.detect_quick(u, p) {
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
            self.units[u].tolerance = 0;
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
                blocked = mask & UNWIND_REFUSES != 0 || self.collide_here(u, ucell(e.to)).is_some();
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
        let anti = self
            .action_of(u)
            .is_some_and(|a| self.units[u].orders[a].index() == index::ATTACK);
        let r = self.find_upath(u, anti);
        if r != 0 {
            // The new top is the waypoint; the line to it is unverified.
            if let Some(top) = self.units[u].path.last().copied() {
                self.units[u].tolerance = top.tolerance;
                if let Some(front) = self.units[u].orders.front_mut()
                    && let Some(m) = front.move_mut()
                {
                    m.has_waypoint = true;
                    m.waypoint = top.to;
                    m.last = None;
                }
            }
            self.units[u].line_ok = false;
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

    /// §4.1: `safe` — the cooldown a failed 48-grid search buys — turns the
    /// test off entirely while it runs, and `Unit::work` counts it down.
    #[test]
    fn the_safe_cooldown_suppresses_the_test_and_expires() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, _y) = pair(a, b);
        let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        assert!(sim.detect_quick(x, into), "blocked with `safe` clear");
        sim.units[x].safe = 3;
        assert!(!sim.detect_quick(x, into), "and clear with it set");
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
