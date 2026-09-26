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
use crate::movement::{Angle, cos_component, find_angle, sin_component};
use crate::orders::{Body, MoveOrder, Order, PathData, QueuePos, index, path_flag};
use crate::world::{Cell, Pos, UNITS_PER_CELL, tile, vector_dist};

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
        let who = self.units[u].owner;
        if who >= 8 {
            return true;
        }
        let domain = self.units[u].kind.domain;
        let prof = self.profile(Obj::Unit(u));
        let (size, circles) = (prof.push_size, prof.push_circles.max(1));
        if size == 0 {
            return true;
        }
        let action = self.action_of(u).map(|a| self.units[u].orders[a].index());
        let transport = self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::TRANSPORT != 0
        });
        let found = self.find_push_candidates(u, at, size);
        if found.is_empty() {
            return true;
        }
        let r_mine = size / circles;
        let facing = self.units[u].movement.facing;
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
        let my_group = self.pool_group_of(u);
        let land = domain == crate::attrition::Domain::Land;
        for o in found {
            let other = &self.units[o];
            // `(local_54 == 0 || who < 8)`: a land pusher's candidates
            // include gaia's.
            if (!land && other.owner >= 8) || other.kind.domain != domain {
                continue;
            }
            let mate = other.owner == who && self.pool_group_of(o) == my_group && my_group != -1;
            if mate && mates && action != Some(10) {
                continue;
            }
            if self.units[u].collide_o == other.index
                && self.units[u].collide_who == other.owner as i8
                && self.units[u].collide_frame == self.frame
            {
                continue;
            }
            let op = self.profile(Obj::Unit(o));
            let (osize, ocircles) = (op.push_size, op.push_circles.max(1));
            if osize == 0 {
                continue;
            }
            let r_other = osize / ocircles;
            let (other0, other_step) = line(
                self.units[o].pos,
                self.units[o].movement.facing,
                r_other,
                ocircles,
            );
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
            if transport && self.profile(Obj::Unit(o)).attack != 0 {
                return false;
            }
            if !self.is_ally(who, self.units[o].owner) {
                return false;
            }
            // A land pusher's own refusals (`5fac8b`-`5faccc`): a packer
            // that is not packed or is unpacking, an entrenched unit
            // (`unit_masks & 0x2000000`), and a tank (the type's vslot
            // `+0x110`, `UnitTypeData::is_tank`).
            if land {
                let t = self.units[o].ty.map(|t| self.unit_types[t].cols);
                if t.is_some_and(|c| c.flag2(crate::ai_load::uflags2::PACKS))
                    && (!self.units[o].combat.packed || self.is_unpacking(o))
                {
                    return false;
                }
                if self.units[o].combat.entrenched
                    || t.is_some_and(|c| c.flag(crate::ai_load::uflags::TANK))
                {
                    return false;
                }
            }
            let push = overlap.min(0x30);
            let there = self.units[o].pos;
            let mut bearing = find_angle(there.x - at.x, there.y - at.y);
            let rel = (bearing.0 as u32).wrapping_sub(facing.0 as u32);
            if !self.is_moving(o) {
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
            if self.invalid_loc(o, to.tile(), false, false, false, false, false) != 0 {
                continue;
            }
            self.set_new_location(o, to, false);
            if mates && (ocircles == 1 || land) {
                self.units[o].collide_o = self.units[u].index;
                self.units[o].collide_who = who as i8;
                if self.units[o].orders.is_empty() {
                    // SEAM: `Guy::turn_angles` and `Guy::do_turn` on the
                    // pushed unit's guy 0 (`5faec8`, `5faedb`) are not
                    // modelled; the unit's own angle is.
                    self.unit_set_angle(o, bearing);
                }
            }
            self.units[o].collide_frame = self.frame;
        }
        true
    }

    /// `Objects::find_units(at, SEARCH_ALL, −1, range, 0x200,
    /// FILTER_NOT_ME)` as `detect_boat_collision` asks it (`5fa935`–
    /// `5fa953`): the circle walk over the object chains while
    /// `circle_radius[ring]` is within the live unit count, the object
    /// arrays with `vector_dist <= range` past it, and the `0x200` region
    /// gate either way (`docs/ORDERS.md` §5.10). The list path's
    /// tile-indexed region lookup is not reproduced, as in
    /// `build_crowd`.
    fn find_push_candidates(&self, u: usize, at: Pos, range: i32) -> Vec<usize> {
        let circle = crate::ai_place::circle();
        let ring = ((range.max(0) + 0x2ff) / 0x300).min(0x40) as usize;
        let live = self.units.iter().filter(|x| x.alive()).count();
        let region = self.world.region_of(at.cell());
        let mut out = Vec::new();
        if circle.radius[ring] <= live {
            let c0 = at.cell();
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) || self.world.region_of(c) != region {
                    continue;
                }
                let slot = (c.y as usize) * (self.world.width() as usize) + (c.x as usize);
                let mut next = self.chain_heads[slot];
                while let Some(o) = next {
                    next = self.units[o].down;
                    if o != u && self.units[o].alive() {
                        out.push(o);
                    }
                }
            }
        } else {
            for o in 0..self.units.len() {
                let p = self.units[o].pos;
                if o == u
                    || !self.units[o].alive()
                    || !self.units[o].on_map
                    || self.world.region_of(p.cell()) != region
                    || vector_dist(p.x - at.x, p.y - at.y) > range
                {
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
    /// disc. Idempotent.
    pub(crate) fn coll_add(&mut self, u: usize) {
        if !(self.units[u].alive() && self.units[u].on_map) {
            return;
        }
        let at = self.units[u].pos;
        self.coll_paint(u, at, true);
        self.units[u].coll_at = Some(at);
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
        let at = self.units[u].coll_at.unwrap_or(self.units[u].pos);
        self.coll_paint(u, at, true);
    }

    /// `Object::remove_from_world`'s half — the clear pass with nowhere to
    /// move to.
    pub(crate) fn coll_remove(&mut self, u: usize) {
        let at = self.units[u].coll_at.take().unwrap_or(self.units[u].pos);
        self.coll_paint(u, at, false);
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
            // `Guy::set_new_location(guy 0, pos, 1)` moves the disc now.
            self.coll_follow(u);
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
        let slots = self.probe_slots(at, size, nocoll);
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
                    return self.leading_edge(size, &slots, |k| {
                        Pos::new(at.x + dx * size, at.y - size + k)
                    });
                }
                if dy.abs() == 1 {
                    return self.leading_edge(size, &slots, |k| {
                        Pos::new(at.x - size + k, at.y + dy * size)
                    });
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
            if slots.get(self, p) {
                return Some(p);
            }
        }
        None
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

    /// The fast path's sweep: the `size + 1` cells of the leading edge its
    /// loop's own parity leaves, in order, the first occupied one winning.
    /// The disc's own-block exemption is not asked and does not have to be
    /// — the edge is a cell beyond the caller's own block on every step
    /// that reaches here.
    fn leading_edge(
        &self,
        size: i32,
        slots: &ProbeSlots,
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
            if slots.live(self, p) {
                if self.coll.get(p.x, p.y) {
                    return Some(p);
                }
                at += 1;
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
    /// The bookkeeping the original does on the way out — clearing
    /// `collide_o`/`collide_who`, ageing `collide`, clearing the wait flag
    /// — happens here too, because every path out passes through it.
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
    ///
    /// The soft one-shot is set **only when the walk ends without a hard
    /// hit**: `orl $0x100000, 0x68(%ebx)` at `006177f3` is reached by
    /// falling out of the nine-cell loop and nowhere else, and a hard hit
    /// returns 1 before it. A soft group-mate seen on the way to a hard
    /// collider leaves no half step — Great Lakes' word 11806, where
    /// `1/37` stepped through `1/64` on one (`docs/COLLISION.md` §12).
    fn name_collider(&mut self, u: usize, at: Pos, at_cell: Pos, cell: Pos) -> Option<usize> {
        let (hard, soft, _) = self.scan_colliders(u, at, at_cell, cell, &mut |_| {});
        if soft && hard.is_none() {
            self.units[u].half_step = true;
        }
        hard
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
    /// (~~`docs/GROUPS.md` §1: an army's members are its group~~ — an
    /// army's *or a pushed slot's*, which is [`Sim::group_of`] and was
    /// item 489's whole finding), and [`Self::same_group_soft`] is that
    /// arm.
    fn soft_collision(&self, u: usize, o: usize, extra: i32) -> bool {
        let moving = |v: usize| self.current_order(v).is_some_and(Order::is_move);
        let acting = |v: usize| self.action_of(v).map(|a| self.units[v].orders[a].index());
        // §4.3's second row (`00617546`-`0061757c`): its **action** is
        // `GUARD` and that order's target is me — an escort never blocks
        // its charge, whatever transit leg is at its head. golden chapter
        // eleven's wagon steps through its walking guard on run190's tick
        // 726, and run191's third take shows the original's scan reaching
        // `is_here` on the guard and never `is_corner` (item 696). The
        // first row, a caravan pair (`TRADE_ROUTE` both ways, both
        // moving), is the else-if before it and is still not carried here.
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
        let members = &g.list;
        if g.who != self.units[u].owner || !members.contains(&u) {
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
    use crate::orders::QueuePos;
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
    fn the_index_follows_the_figure_and_is_not_refcounted() {
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
        // The figure goes with it (`Guy::set_new_location(guy 0, pos, 1)`),
        // so the disc moves now.
        sim2.set_new_location(a2, ucell_centre(Pos::new(c.x - 4, c.y)), true);
        assert!(
            !sim2.coll.get(c.x + 1, c.y),
            "the mover cleared a cell its neighbour still occupies"
        );
        let _ = &sim;
    }

    /// §16: **the disc follows guy 0, not the unit.** `CollCheck::move_unit`
    /// has one caller, `Guy::set_new_location`, so a unit moved without
    /// its figures — a push, `Unit::set_new_location(x, y, 0, 0)` — keeps
    /// its bits on the old cell until its own `Guy::process` puts the
    /// figure on the new point. Golden chapter eleven's guard, pushed by
    /// its wagon on tick 732, is refused its own step on tick 733 by a
    /// bit its old disc still holds.
    #[test]
    fn a_pushed_unit_s_disc_waits_for_its_figure() {
        let (mut sim, a, _b) = pair(
            ucell_centre(Pos::new(20, 20)),
            ucell_centre(Pos::new(30, 30)),
        );
        let old = Pos::new(19, 20);
        let to = ucell_centre(Pos::new(23, 20));
        assert!(sim.coll.get(old.x, old.y), "the disc stands on its cell");
        sim.set_new_location(a, to, false);
        assert!(
            sim.coll.get(old.x, old.y) && !sim.coll.get(24, 20),
            "a push leaves the disc where the figure is"
        );
        assert_eq!(sim.units[a].coll_at, Some(ucell_centre(Pos::new(20, 20))));
        sim.units[a].movement.body.pos = to;
        sim.coll_follow(a);
        assert!(
            !sim.coll.get(old.x, old.y) && sim.coll.get(24, 20),
            "the figure's move takes the disc with it"
        );
        assert_eq!(sim.units[a].coll_at, Some(to));
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
    /// been colliding drops its loose near waypoints (tolerance 1 to
    /// `0x60`) and plans on the **tile** grid (`find_tpath`, waypoints at
    /// `tolerance 0x60`, no flags); one that has keeps them and plans on
    /// the **48** grid (`find_upath`, `flags 2`) — the finer one, and the
    /// only one that knows units are in the way. The plan laid here is
    /// loose (`0x30`) so the two arms part; a tolerance-0 plan survives
    /// both.
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
        let plan = |collide: i16, tolerance: i32| {
            let (mut sim, x, _y) = pair(start, Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18));
            sim.order_move(x, goal);
            sim.tick();
            // The 48-grid plan, and the unit sitting on the top of it.
            let here = sim.units[x].pos;
            sim.units[x].path.push(PathData {
                to: Pos::new(here.x - 0x30, here.y),
                tolerance,
                flags: path_flag::SIDESTEP,
            });
            sim.units[x].path.push(PathData {
                to: here,
                tolerance,
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

        let tiles = plan(0, 0x30);
        let units = plan(1, 0x30);
        assert!(
            tiles.iter().any(|p| p.tolerance == 0x60),
            "the tile grid's waypoints carry its half-tile tolerance: {tiles:?}"
        );
        assert!(
            !tiles.iter().any(|p| p.flags & path_flag::SIDESTEP != 0),
            "and the loose plan under it was dropped: {tiles:?}"
        );
        assert!(
            units.iter().any(|p| p.flags & path_flag::SIDESTEP != 0),
            "a unit that has been colliding keeps it and plans on the 48 \
             grid: {units:?}"
        );
        // **And a tolerance-0 plan is not loose** (item 669): the unwind
        // pops 1 to `0x60`, unsigned (`0x5f8ad2`), so the tile arm keeps
        // an exact point too. A fresh `find_upath` plan is the tolerance-0
        // kind, and until item 669 this test laid one and asserted the
        // tile arm dropped it — that was the misread `< 0x60`.
        let exact = plan(0, 0);
        assert!(
            exact
                .iter()
                .any(|p| p.flags & path_flag::SIDESTEP != 0 && p.tolerance == 0),
            "the tile arm keeps an exact point: {exact:?}"
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

    /// §6 step 3, **the enemy ladder**, arm B (item 445): a unit walking
    /// under an attack whose target is already in reach, bumped by a
    /// *different* enemy, drops the walk where it stands. The attack stays
    /// and so does the collider's name, and nothing is incremented: no
    /// `collide`, no `collide_frame`, no sidestep, no repath. That is
    /// golden chapter one's `0/8` on block 625 field for field
    /// (`docs/COMBAT.md` §48), and without the arm this crate sidestepped
    /// and walked on.
    ///
    /// With the target out of reach the arm does not fire and the unit
    /// falls through to steps 4–6, as the third, unmodelled arm leaves it.
    #[test]
    fn a_bump_from_another_enemy_drops_the_chase_when_the_target_is_in_reach() {
        let here = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let arrange = |target_at: Pos| {
            let (mut sim, x, t) = pair(here, target_at);
            sim.units[t].owner = 1;
            // The collider: a third unit, the target's player's, standing
            // east of `x`.
            let mut c = Unit::new(1, 7, Pos::new(here.x + 0x30, here.y), 40);
            c.ty = sim.units[t].ty;
            let c = sim.add_unit(c);
            sim.add_attack_order(x, Obj::Unit(t), QueuePos::New, false, false);
            sim.add_move_order(
                x,
                Pos::new(35 * 0x30 + 0x18, 20 * 0x30 + 0x18),
                crate::orders::MoveKind::MoveTo,
                QueuePos::First,
                false,
            );
            sim.units[x].collide_o = sim.units[c].index;
            sim.units[x].collide_who = 1;
            (sim, x)
        };

        // In reach: the move goes, the attack and the collider's name stay.
        let (mut sim, x) = arrange(Pos::new(here.x, here.y + 0x30));
        assert_eq!(sim.units[x].orders.len(), 2);
        sim.resolve_unit_collision(x);
        assert_eq!(sim.units[x].orders.len(), 1, "the chase is dropped");
        assert!(matches!(sim.units[x].orders[0].body, Body::Attack(_)));
        assert_eq!(
            (sim.units[x].collide_o, sim.units[x].collide_who),
            (7, 1),
            "arm B keeps the collider's name"
        );
        assert_eq!(
            (sim.units[x].collide, sim.units[x].collide_frame),
            (0, 0),
            "nothing is counted"
        );
        assert_eq!(sim.units[x].orders_pos, sim.units[x].pos);

        // Out of reach: the ladder passes it on to step 5, which counts
        // the collision (the collider holds no move, so no sidestep).
        let (mut sim, x) = arrange(Pos::new(here.x, here.y + 10 * 0x30));
        sim.resolve_unit_collision(x);
        assert_eq!(sim.units[x].orders.len(), 2, "the chase goes on");
        assert_eq!(sim.units[x].collide, 1, "and step 5 counts it");
    }

    /// Arm A of the same ladder: the collider **is** the target. A plain
    /// order forgets the collider and drops the walk; a group order is
    /// left alone with `collide = 1` (`005f9d30:162-174`). Reading only:
    /// no capture on disk reaches it.
    #[test]
    fn a_bump_from_the_target_itself_drops_the_chase_and_forgets_the_collider() {
        let here = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let (mut sim, x, t) = pair(here, Pos::new(here.x + 5 * 0x30, here.y));
        sim.units[t].owner = 1;
        sim.add_attack_order(x, Obj::Unit(t), QueuePos::New, false, false);
        sim.add_move_order(
            x,
            Pos::new(35 * 0x30 + 0x18, 20 * 0x30 + 0x18),
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        sim.units[x].collide_o = sim.units[t].index;
        sim.units[x].collide_who = 1;
        sim.resolve_unit_collision(x);
        assert_eq!(sim.units[x].orders.len(), 1);
        assert!(matches!(sim.units[x].orders[0].body, Body::Attack(_)));
        assert_eq!((sim.units[x].collide_o, sim.units[x].collide_who), (-1, -1));
    }

    /// **Arm C of the enemy ladder** (`005f9d30:189-252`, §14): a captain
    /// whose attack's target is out of range, bumped on its walk by an
    /// enemy that is not its target, drops the walk and the attack where
    /// it stands and takes what it can strike from there
    /// (`find_new_target(this, NULL, 1)`). run171's `1/6` on 658: chasing
    /// the General, bumped by `0/7`, it attacks `0/7` without a step or a
    /// count. At ten retargets on its leader's frame it queues the
    /// collider itself, unless it is siege; a follower takes its
    /// captain's target instead, when it can strike it from where it
    /// stands.
    ///
    /// Made to fail on purpose by taking the arm out: the captain keeps
    /// its walk and its attack on the far target and counts the collision.
    #[test]
    fn a_captain_bumped_by_another_enemy_strikes_what_it_can_reach() {
        let tile = 0xc0;
        let here = Pos::new(0x1000 + 24, 0x1000 + 24);
        let setup = |retargets: i32| {
            let mut sim = Sim::new(Tuning::RON, World::new(60, 60), 2);
            sim.world
                .fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(59, 59));
            sim.at_war[0][1] = true;
            sim.at_war[1][0] = true;
            let ty = sim.add_unit_type(UnitType {
                hits: 100,
                moves: 25,
                combat: crate::combat::Profile {
                    attack: 15,
                    max_range: 4,
                    uber_size: 1,
                    block_radius: 48,
                    big_radius: 48,
                    ..crate::combat::Profile::default()
                },
                ..UnitType::default()
            });
            let put = |sim: &mut Sim, who: Player, at: Pos| {
                let index = i16::try_from(sim.units.len()).unwrap();
                let mut u = Unit::new(who, index, at, 100);
                u.ty = Some(ty);
                u.on_map = true;
                u.kind = sim.unit_types[ty].kind;
                sim.add_unit(u)
            };
            let me = put(&mut sim, 0, here);
            sim.units[me].captain = true;
            let far = put(&mut sim, 1, Pos::new(here.x - 20 * tile, here.y));
            let bump = put(&mut sim, 1, Pos::new(here.x - tile, here.y));
            sim.add_attack_order(me, Obj::Unit(far), QueuePos::New, false, false);
            sim.add_move_order(
                me,
                Pos::new(here.x - 19 * tile, here.y),
                crate::orders::MoveKind::MoveTo,
                QueuePos::First,
                false,
            );
            sim.units[me].collide_o = sim.units[bump].index;
            sim.units[me].collide_who = 1;
            sim.retargets[0] = retargets;
            (sim, me, far, bump)
        };

        let (mut sim, me, _, bump) = setup(9);
        sim.resolve_unit_collision(me);
        assert_eq!(sim.units[me].orders.len(), 1, "the walk is dropped");
        assert!(matches!(sim.units[me].orders[0].body, Body::Attack(_)));
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(bump)));
        assert_eq!(sim.units[me].pos, here, "no snap");
        assert_eq!(sim.units[me].collide, 0, "no count");
        assert_eq!(
            sim.units[me].combat.stance,
            crate::combat::Stance::Aggressive,
            "the stance is put back"
        );

        // At ten the search is shut (`cmp [+0x9f4], 0xa; jge`), and a
        // captain that is not siege queues the collider itself in front.
        let (mut sim, me, _, bump) = setup(10);
        sim.resolve_unit_collision(me);
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(bump)));
        assert_eq!(sim.units[me].orders.len(), 1);
        assert_eq!(sim.units[me].collide, 0);
        // A siege captain at ten takes §6 as before.
        let (mut sim, me, far, _) = setup(10);
        let ty = sim.units[me].ty.unwrap();
        sim.unit_types[ty].combat.siege = true;
        sim.resolve_unit_collision(me);
        assert_eq!(sim.units[me].combat.target, Some(Obj::Unit(far)));
        assert_eq!(sim.units[me].collide, 1, "§6 step 5's count");

        // A follower takes its captain's target when it can strike it.
        let (mut sim, cap, _, bump) = setup(0);
        sim.clear_orders(cap);
        sim.add_attack_order(cap, Obj::Unit(bump), QueuePos::New, false, false);
        let index = i16::try_from(sim.units.len()).unwrap();
        let mut f = sim.units[cap].clone();
        f.index = index;
        f.captain = false;
        f.orders.clear();
        f.o_up = Some(cap);
        let f = sim.add_unit(f);
        let far = Obj::Unit(sim.units.len() - 2);
        sim.add_attack_order(f, far, QueuePos::New, false, false);
        sim.add_move_order(
            f,
            Pos::new(here.x - 19 * tile, here.y),
            crate::orders::MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
        sim.units[f].collide_o = sim.units[bump].index;
        sim.units[f].collide_who = 1;
        // The bump is the follower's collider but not its target, and
        // the captain's target is the bump: arm A does not fire, arm C's
        // follower branch does.
        sim.resolve_unit_collision(f);
        assert_eq!(sim.units[f].combat.target, Some(Obj::Unit(bump)));
        // `repath` takes the walk and `kill_current_order` the attack on
        // the far target, so the captain's is the only order left.
        assert_eq!(sim.units[f].orders.len(), 1);
        assert!(matches!(sim.units[f].orders[0].body, Body::Attack(_)));
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

    /// §4.3's group arm again, from the **other seat**: two members of a
    /// **pushed** group pass through each other too
    /// (`docs/COLLISION.md` §11).
    ///
    /// The original's test is `this->+0x80 == other->+0x80 != -1`
    /// (`detect_unit_collision@00617060:307`) — one `short`, the
    /// `Groups::list` slot, and it knows nothing about armies. A group
    /// here has one of two seats: an army's, or a
    /// [`crate::group::Pushed`] slot, and the sibling above only ever
    /// exercised the first. [`Sim::same_group_soft`] read
    /// [`Sim::army_of`] for a fortnight after item 465 gave this crate a
    /// real pool, so every member of a pushed group was **hard** to
    /// every other — which on Great Lakes is the whole raid, and was the
    /// headline's word at 10277.
    ///
    /// **Made to fail on purpose**: with `group_of` put back to
    /// `army_of`, the second assertion below collides, and Great Lakes'
    /// word returns from 10294 to 10277 with block 10278's seventeen
    /// rows of `1/40` and `1/41` back on it.
    #[test]
    fn two_members_of_a_pushed_group_pass_through_each_other() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let (mut sim, x, y) = pair(a, b);
        let into = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        assert!(
            sim.detect_unit_collision(x, into).is_some(),
            "two strangers block"
        );
        // The same two out of every army and into a pool slot, which is
        // where `do_group_move`'s probe leaves the AI's raiders.
        let mut g = crate::group::Group::stack(0);
        g.list = vec![x, y];
        assert!(sim.push_group(&mut g, false), "a pair takes a slot");
        assert_eq!(g.army, None, "and it is not an army's group");
        assert_eq!(g.pushed, Some(0), "it is the pool's");
        assert!(
            sim.detect_unit_collision(x, into).is_none(),
            "sharing a pushed group makes it a nudge, not a collision"
        );
        assert!(
            sim.units[x].half_step,
            "and a soft collision raises the one-shot the next step is \
             halved by"
        );
    }

    /// §4.3's one-shot is the **walk's**, not the soft candidate's: it is
    /// set only when the nine-cell sweep ends without a hard hit
    /// (`orl $0x100000, 0x68(%ebx)` at `006177f3`, reached by falling
    /// out of the loop; a hard hit returns 1 before it). A group-mate
    /// seen on the way to a stranger leaves no half step, in either
    /// chain order.
    ///
    /// Great Lakes' word 11806 was exactly this: `1/37`'s sweep met a
    /// soft group-mate and `1/64` hard on frame 11804, and this crate
    /// kept the bit, spent it on 11805 and stepped where the original's
    /// `1/37` waited (`docs/COLLISION.md` §12). **Made to fail on
    /// purpose**: with `name_collider` setting the bit on any soft
    /// candidate, the order that lists the group-mate first raises it.
    /// §13's distance, from the listing (`5fabf3`–`5fac63`): the longer
    /// leg plus the shorter's square over twice the longer, and past
    /// 60,000 on the shorter leg their mean with the longer counted twice.
    #[test]
    fn a_boat_measures_with_the_listing_s_distance() {
        assert_eq!(boat_dist(0, 0), 0);
        assert_eq!(boat_dist(3, -4), 4 + 9 / 8);
        assert_eq!(boat_dist(-100, 0), 100);
        assert_eq!(boat_dist(0, 50), 50);
        assert_eq!(boat_dist(100_000, 70_000), (70_000 + 200_000) / 2);
    }

    /// Two ships on open water, `push_size` 192 in two circles, the second
    /// standing idle across the first one's bow.
    fn fleet(a: Pos, b: Pos) -> (Sim, usize, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Sea, Cell::new(0, 0), Cell::new(39, 39));
        for tx in 0..40 {
            for ty in 0..40 {
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
            moves: 42,
            combat: crate::combat::Profile {
                block_radius: 192,
                big_radius: 192,
                uber_size: 1,
                domain: Domain::Sea,
                push_size: 192,
                push_circles: 2,
                ..crate::combat::Profile::default()
            },
            ..UnitType::default()
        });
        let make = |sim: &mut Sim, o: i16, at: Pos| {
            let mut u = Unit::new(0, o, at, 40);
            u.ty = Some(ty);
            u.kind.domain = Domain::Sea;
            let i = sim.add_unit(u);
            sim.units[i].movement.facing = crate::movement::Angle::EAST;
            i
        };
        let x = make(&mut sim, 0, a);
        let y = make(&mut sim, 1, b);
        (sim, x, y)
    }

    /// §13.2: **a ship takes `detect_unit_collision`'s second arm.** A
    /// quick probe without `nocoll` answers no collision before any scan,
    /// where a land unit on the same two points collides; `nocoll` —
    /// `valid_ucoord`'s — skips the arm and scans. Made to fail on
    /// purpose by answering `false` from `takes_boat_arm`.
    #[test]
    fn a_ship_s_quick_probe_never_scans() {
        let a = Pos::new(10 * 0x30 + 0x18, 10 * 0x30 + 0x18);
        let b = Pos::new(12 * 0x30 + 0x18, 10 * 0x30 + 0x18);
        let (sim, x, _) = pair(a, b);
        assert!(sim.detect_quick(x, b, false), "a land unit collides");
        let (sim, x, _) = fleet(a, b);
        assert!(sim.takes_boat_arm(x));
        assert!(!sim.detect_quick(x, b, false), "a ship does not scan");
        assert!(sim.detect_quick(x, b, true), "nocoll skips the arm");
    }

    /// §13.3: `detect_boat_collision` **passes a group-mate and pushes an
    /// idle stranger** of its own side, at least 45° off its facing and by
    /// at most 48; a foreign ship in the way answers 0 and leaves the step
    /// to the land scan. Made to fail on purpose by dropping the group-mate
    /// test, which pushes the mate too.
    #[test]
    fn a_ship_pushes_an_idle_stranger_and_passes_its_group_mate() {
        let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let b = Pos::new(a.x + 150, a.y);
        // The group-mate: nothing moves, and the push is handled.
        let (mut sim, x, y) = fleet(a, b);
        let slot = sim.init_army(0, None);
        sim.army_add_unit(0, slot, x);
        sim.army_add_unit(0, slot, y);
        assert!(
            sim.units[x].group_ptr.is_some() && sim.units[x].group_ptr == sim.units[y].group_ptr
        );
        assert!(sim.detect_boat_collision(x, a, true));
        assert_eq!(sim.units[y].pos, b, "a group-mate is not pushed");
        // The stranger: pushed, and stamped.
        let (mut sim, x, y) = fleet(a, b);
        sim.frame = 7;
        assert!(sim.detect_boat_collision(x, a, true));
        let moved = sim.units[y].pos;
        assert_ne!(moved, b, "the stranger is pushed");
        let (dx, dy) = (moved.x - b.x, moved.y - b.y);
        assert!(dx * dx + dy * dy <= 49 * 49, "by at most 48: ({dx}, {dy})");
        assert_eq!(sim.units[y].collide_frame, 7);
        // Due ahead and standing, it goes at least 45° off the bow: south
        // of east here, since the bearing is exactly the facing.
        assert!(moved.y > b.y, "turned off the bow: {moved:?}");
        // A foreign ship is not pushed and hands the step to the scan.
        let (mut sim, x, y) = fleet(a, b);
        sim.units[y].owner = 1;
        assert!(!sim.detect_boat_collision(x, a, true));
        assert_eq!(sim.units[y].pos, b);
    }

    /// A land convoy: a supply wagon of player 0 (`uflags2::SUPPLY_OR_HERO`,
    /// two push circles of 144) at `a`, and a one-circle land unit of the
    /// same player at `b`, standing; `other_flags` is its `unit_flags`.
    fn convoy(a: Pos, b: Pos, other_flags: u32) -> (Sim, usize, usize) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let profile = |push_size, push_circles| crate::combat::Profile {
            block_radius: 96,
            big_radius: 96,
            uber_size: 1,
            push_size,
            push_circles,
            ..crate::combat::Profile::default()
        };
        let wagon = sim.add_unit_type(UnitType {
            hits: 90,
            moves: 25,
            combat: profile(288, 2),
            cols: crate::ai_load::UnitCols {
                unit_flags2: crate::ai_load::uflags2::SUPPLY_OR_HERO,
                ..crate::ai_load::UnitCols::default()
            },
            ..UnitType::default()
        });
        let other = sim.add_unit_type(UnitType {
            hits: 65,
            moves: 33,
            combat: profile(72, 1),
            cols: crate::ai_load::UnitCols {
                unit_flags: other_flags,
                ..crate::ai_load::UnitCols::default()
            },
            ..UnitType::default()
        });
        let make = |sim: &mut Sim, o: i16, at: Pos, ty| {
            let mut u = Unit::new(0, o, at, 40);
            u.ty = Some(ty);
            let i = sim.add_unit(u);
            sim.units[i].movement.facing = crate::movement::Angle::EAST;
            i
        };
        let x = make(&mut sim, 7, a, wagon);
        let y = make(&mut sim, 6, b, other);
        (sim, x, y)
    }

    /// §13.3's **land half** (item 696): a supply wagon takes
    /// `detect_unit_collision`'s second arm as a ship does, and pushes a
    /// standing land unit of its own side aside, naming itself in the
    /// pushed unit's `collide_o`; a tank it will not shove, and hands the
    /// step to the land scan. run190's wagon pushes its guard (21, 1) on
    /// its first step. Made to fail on purpose by answering the arm for a
    /// ship alone, and by dropping the tank refusal.
    #[test]
    fn a_supply_wagon_pushes_a_standing_unit_and_not_a_tank() {
        let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let b = Pos::new(a.x + 150, a.y);
        let (mut sim, x, y) = convoy(a, b, 0);
        assert!(sim.takes_boat_arm(x), "a supply wagon takes the arm");
        assert!(!sim.takes_boat_arm(y), "a plain land unit does not");
        sim.frame = 721;
        assert!(sim.detect_boat_collision(x, a, true));
        let moved = sim.units[y].pos;
        assert_ne!(moved, b, "the standing unit is pushed");
        let (dx, dy) = (moved.x - b.x, moved.y - b.y);
        assert!(dx * dx + dy * dy <= 49 * 49, "by at most 48: ({dx}, {dy})");
        assert_eq!(sim.units[y].collide_frame, 721);
        assert_eq!(
            (sim.units[y].collide_o, sim.units[y].collide_who),
            (7, 0),
            "a land pusher names itself on the pushed unit"
        );
        let (mut sim, x, y) = convoy(a, b, crate::ai_load::uflags::TANK);
        assert!(!sim.detect_boat_collision(x, a, true), "a tank is refused");
        assert_eq!(sim.units[y].pos, b);
    }

    /// §4.3's **escort row** (item 696): a collider whose *action* is a
    /// `GUARD` on me is soft, whatever transit leg is at its head; the same
    /// guard on another unit is not. The original's scan on run190's tick
    /// 726 reaches `is_here` on the walking guard and never `is_corner`.
    /// Made to fail on purpose by deleting the row.
    #[test]
    fn an_escort_never_blocks_its_charge() {
        let a = Pos::new(20 * 0x30 + 0x18, 20 * 0x30 + 0x18);
        let b = Pos::new(a.x + 150, a.y);
        let (mut sim, wagon, guard) = convoy(a, b, 0);
        sim.add_guard_order(guard, wagon, 0, 372, QueuePos::New);
        sim.add_move_order(
            guard,
            Pos::new(b.x, b.y + 480),
            crate::orders::MoveKind::AttackTo,
            QueuePos::First,
            false,
        );
        assert!(sim.soft_collision(wagon, guard, 0), "an escort is soft");
        let (mut sim, wagon, guard) = convoy(a, b, 0);
        let stranger = sim.add_unit(Unit::new(0, 8, Pos::new(a.x, a.y + 960), 40));
        sim.add_guard_order(guard, stranger, 0, 372, QueuePos::New);
        assert!(
            !sim.soft_collision(wagon, guard, 0),
            "another's escort is not"
        );
    }

    #[test]
    fn a_hard_hit_leaves_no_half_step_whatever_soft_it_passed() {
        let a = Pos::new(30 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let b = Pos::new(28 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        for mate_first in [false, true] {
            let (mut sim, x, y) = pair(a, b);
            // A third unit on the same point, of the same type and owner
            // and in no group: a stranger, and so hard.
            let mut v = sim.units[y].clone();
            v.index = 2;
            let z = sim.add_unit(v);
            let slot = sim.init_army(0, None);
            sim.army_add_unit(0, slot, x);
            sim.army_add_unit(0, slot, y);
            // The chain's order is the insertion's; re-link the mate to
            // put it on the other side of the stranger.
            if mate_first {
                sim.chain_remove(y);
                sim.chain_add(y);
            }
            let hard = sim.detect_unit_collision(x, b);
            assert_eq!(
                hard,
                Some(z),
                "the stranger is hard (mate_first {mate_first})"
            );
            assert!(
                !sim.units[x].half_step,
                "a hard hit leaves no one-shot (mate_first {mate_first})"
            );
        }
        // And the soft mate alone still raises it.
        let (mut sim, x, y) = pair(a, b);
        let slot = sim.init_army(0, None);
        sim.army_add_unit(0, slot, x);
        sim.army_add_unit(0, slot, y);
        assert_eq!(sim.detect_unit_collision(x, b), None);
        assert!(sim.units[x].half_step, "the soft walk raises it");
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

    /// §17: **the give-up does not step.** A unit blocked for 26 frames
    /// (`collide >= 0x1a`) on a final waypoint within three reaches widens
    /// `tolerance` to twice the Manhattan distance still owed and jumps
    /// straight to the step's arrival tail (`005fb7b5`–`005fb7bb`,
    /// `jmp 005fb82c`), past `set_new_location`. The test, from where it
    /// stands, then takes the waypoint: run218's `1/23` on Great Lakes
    /// 16459, which this crate walked a step into The Despot.
    #[test]
    fn a_blocked_unit_that_gives_up_takes_its_waypoint_where_it_stands() {
        let b = Pos::new(27 * 0x30 + 0x18, 30 * 0x30 + 0x18);
        let a = Pos::new(b.x + 0x60, b.y);
        let (mut sim, x, y) = pair(a, b);
        sim.order_move(x, Pos::new(20 * 0x30 + 0x18, 30 * 0x30 + 0x18));
        sim.tick();
        let here = sim.units[x].pos;
        // One final leg, past the unit in front and inside three reaches
        // (`(48 + 48) * 3 = 288`), with a tolerance of its own.
        let to = Pos::new(b.x - 0x30, b.y);
        sim.units[x].path.clear();
        sim.units[x].path.push(PathData {
            to,
            tolerance: 0x30,
            flags: path_flag::FINAL,
        });
        sim.units[x].line_ok = true;
        if let Some(front) = sim.units[x].orders.front_mut()
            && let Some(m) = front.move_mut()
        {
            m.has_waypoint = true;
            m.waypoint = to;
        }
        sim.units[x].collide = 0x1a;
        sim.units[x].collide_frame = sim.frame;
        let manh = (to.x - here.x).abs() + (to.y - here.y).abs();
        assert!(manh < 3 * 96, "inside three reaches: {manh}");
        sim.tick();
        assert_eq!(
            (sim.units[x].collide_o, sim.units[x].collide_who),
            (
                sim.units[y].index,
                i8::try_from(sim.units[y].owner).unwrap()
            ),
            "the step was refused by the unit in front"
        );
        assert_eq!(sim.units[x].tolerance, manh * 2, "the widened tolerance");
        assert_eq!(sim.units[x].pos, here, "and no step was taken");
        assert!(
            sim.units[x].path.is_empty() && sim.units[x].orders.is_empty(),
            "the arrival test took the final leg where it stands: {:?} {:?}",
            sim.units[x].path,
            sim.units[x].orders
        );
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
    /// **The pathfinder's copy of a block outlives the probe that took
    /// it** (`CollCheck::fill_slots@006820e0`, `docs/PATHFINDER.md` §26).
    /// A `nocoll` probe reads a world cell's copy when the tree holds one
    /// and copies what it read live when it does not; only `kill_lists`
    /// empties the tree. So once a probe has copied a neighbour's block, a
    /// later `nocoll` probe still finds the neighbour where it stood,
    /// while a probe of the live blocks finds it gone.
    ///
    /// Written to fail first: with the live blocks alone the second
    /// `nocoll` probe finds nothing, which is what Great Lakes 12625's
    /// resumed search for `1/41` read on every cell it first probed.
    #[test]
    fn a_nocoll_probe_reads_the_block_the_last_one_copied() {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut sim = Sim::new(Tuning::RON, world, 2);
        let t = sim.add_unit_type(UnitType {
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
        let put = |sim: &mut Sim, o: i16, cell: Pos| {
            let mut u = Unit::new(0, o, ucell_centre(cell), 40);
            u.ty = Some(t);
            sim.add_unit(u)
        };
        let prober = put(&mut sim, 0, Pos::new(20, 20));
        let other = put(&mut sim, 1, Pos::new(24, 20));
        let at = Pos::new(23, 20);
        assert!(
            sim.collide_here(prober, at, true).is_some(),
            "the neighbour is there, and the probe copies its world cell"
        );
        assert!(sim.set_new_location(other, ucell_centre(Pos::new(30, 30)), true));
        assert_eq!(
            sim.collide_here(prober, at, false),
            None,
            "the live blocks have it gone"
        );
        assert!(
            sim.collide_here(prober, at, true).is_some(),
            "the copy still has it where it stood"
        );
        sim.kill_lists();
        assert_eq!(
            sim.collide_here(prober, at, true),
            None,
            "`kill_lists` empties the tree, and the next probe reads live"
        );
    }

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

    /// §4.2's fast path **does not advance past an empty slot** (item 539,
    /// `00682540:71`-`116`). An even pass tests its cell and advances the
    /// swept axis on a miss only when the cell's world cell holds a bit;
    /// a cell in an empty world cell is skipped where it stands, so the
    /// next cell tested is one step on, not two. Great Lakes' sim-frame
    /// 11304 is the shape, rebuilt at the origin: a `coll_size 1` walker
    /// stepping west from `(17, 16)` onto `(16, 16)` sweeps the column
    /// `x = 15` from `y = 15`, which is the last row of world cell
    /// `(0, 0)`, and a group-mate's block starts at `y = 17`, which is
    /// world cell `(0, 1)`.
    ///
    /// Written to fail first: with the fixed two-step stride the sweep
    /// tests `(15, 17)`, finds the neighbour, and the walker takes a half
    /// step that the original does not take.
    #[test]
    fn the_leading_edge_does_not_step_past_an_empty_world_cell() {
        let mut world = World::new(4, 4);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(3, 3));
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
        let put = |sim: &mut Sim, o: i16, cell: Pos| {
            let mut u = Unit::new(0, o, ucell_centre(cell), 40);
            u.ty = Some(ty);
            sim.add_unit(u)
        };
        let walker = put(&mut sim, 0, Pos::new(17, 16));
        // `(14..=16, 17..=19)`: world cells `(0, 1)` and `(1, 1)`.
        put(&mut sim, 1, Pos::new(15, 18));
        let at = Pos::new(16, 16);
        assert!(!sim.coll.any_in_cell(0, 0), "world cell (0, 0) is empty");
        assert_eq!(
            sim.collide_here(walker, at, false),
            None,
            "(15, 15)'s slot is empty, so the sweep tests (15, 16) and misses"
        );
        // The control: one bit anywhere in world cell `(0, 0)` makes the
        // slot live, and the stride is two again — `(15, 15)`, `(15, 17)`.
        put(&mut sim, 2, Pos::new(3, 3));
        assert!(sim.coll.any_in_cell(0, 0));
        assert_eq!(
            sim.collide_here(walker, at, false),
            Some(Pos::new(15, 17)),
            "a live slot advances past its miss"
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
