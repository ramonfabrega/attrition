//! `Unit::find_attack_pos@00601280` — where to stand to attack something.
//!
//! `docs/COMBAT.md` §17 is the specification. This is the **building**
//! arm of it: the eight-sided ring walk around the target's footprint,
//! two arms in opposite directions, one draw per candidate that clears
//! the four rejections, lowest `vector_dist + draw % 0xc0` kept, and the
//! budget that caps a call at [`crate::fight::ATTACK_POS_CAP_RANGED`]
//! draws on the ranged arm and [`crate::fight::ATTACK_POS_CAP_NEAR`] on
//! the near one.
//!
//! **What is here and what is not.** The ring is entered when the target
//! is a *building* — `is_build`, the target's vtable `+0x1c`. A **unit**
//! target takes the other half of `00601280` entirely, a chase over the
//! target's own heading that ends in `UnitType::find_nearby_spot`; that
//! half is not modelled, and [`Sim::find_attack_pos`] answers `None` for
//! it so its callers keep the straight-line approach they had. The same
//! goes for `Unit::find_melee_pos@006010b0`, which only a melee asker
//! with a **unit** target reaches, and for the `project`-away arm of a
//! minimum-range asker standing inside its own dead zone. Each is named
//! where it is skipped.

use crate::collide::{UCELLS_PER_CELL, UNITS_PER_UCELL};

/// Quarter-tiles to a tile — the `>> 2` `find_open_slots` applies before
/// asking `invalid_loc` (`600f47`, `600f5e`).
const UCELLS_PER_UTILE: i32 = 4;
use crate::combat::Obj;
use crate::movement::{Angle, cos_component, sin_component};
use crate::orders::{Order, index, snapped};
use crate::world::{Pos, tile, vector_dist};

/// The initial budget, `[ebp-0x20]` at `601e20` — a hundred iterations
/// before a call gives up on its own.
const BUDGET: i32 = 100;

/// The iteration counter's start, `[ebp-0x24] = 4` at `601e2a`. The loop
/// tail tests `counter - 4 < budget` *after* the increment (`602782`), so
/// the four is what makes the first iteration's test read as zero.
const COUNTER_START: i32 = 4;

/// What an improvement adds to the counter to make the new budget on the
/// ranged arm (`60215d`: `add eax, 0xb`).
const BUDGET_SLACK_RANGED: i32 = 11;

/// One arm of the ring walk: which side it is on, how far along, and the
/// point it is walking from.
#[derive(Clone, Copy, Debug)]
struct Arm {
    /// 1..8, anticlockwise from the north-west corner. Odd is a corner
    /// and sweeps an arc; even is an edge and walks a straight line.
    side: i32,
    /// The arc step for a corner side (`[ebp+4*ebx-0x78]`).
    step: i32,
    /// The arc's base angle for a corner side (`[ebp+4*ebx-0x70]`).
    base_angle: i32,
    /// The point this arm is at (`[ebp+4*ebx-0x60]`, `-0x68`).
    at: Pos,
}

impl crate::Sim {
    /// `Unit::find_attack_pos(this, o, who, param_3, &x, &y, from, 1)`.
    ///
    /// `from` is the point the asker is approaching from — its own
    /// position for the chase (`Unit::fight+0xcb4`, through the
    /// seven-argument thunk `@00602e60`), the group leader's for
    /// `Group::action_attack+0x41a`. `site` is the label the draw is
    /// marked with, which is the only thing that tells the two chains
    /// apart in a trace comparison.
    ///
    /// `Some(p)` is the original's `return 1` with the out-parameters
    /// written; `None` its `return 0`, on which every caller falls back
    /// to the target's own position.
    pub(crate) fn find_attack_pos(
        &mut self,
        u: usize,
        target: Obj,
        from: Pos,
        site: &'static str,
    ) -> Option<Pos> {
        let me = Obj::Unit(u);
        let p = self.profile(me);
        // SEAM (`601304`): a unit inside a carrier tail-calls the
        // carrier's own `find_attack_pos`. Nothing here rides in one.
        //
        // SEAM (`60130d`-`601348`, `local_28`): the flag that forces the
        // stand-off onto the maximum-range arm is
        // `vtable+0x10c && (unit_masks & 0x40000) && target->is_build`,
        // and the middle term — a per-unit mask bit — is not modelled, so
        // it is taken as clear.
        let bombard = false;
        // `local_34`: the asker's own activity order, read at `6013a2`
        // through `get_activity(units[who][o])` — itself, not a captain.
        let activity = self.current_order(u).map(|o| o.index());
        // `local_38`: the head order's `mandatory` byte when the current
        // order is an ATTACK (`60142c`). It gates the `find_nearby_spot`
        // fallback at the very end, which is not modelled here.
        let range = self.max_range_of(me);
        // `local_2c`, set only by the far arm.
        let far: bool;

        // **The first return**: already in range from `from`.
        if self.units[u].on_map && !bombard && self.is_in_range_at(me, from, target) {
            return Some(from);
        }

        // **The stand-off, `local_18`** (§17.3). `attack_dist` is asked
        // from the unit's *own* position, not from `from` (`60138b`).
        let d = self.attack_dist(me, target);
        let target_is_unit = matches!(target, Obj::Unit(_));
        let tp = self.profile(target);
        let stand: i32;
        if !target_is_unit || d <= (range + 8) * 0xc0 {
            if p.max_range == 0 {
                // `role & 0x400` clear is exactly `max_range == 0`
                // (`UnitType::determine_roles@0061c320:58`-`60`), so a
                // melee asker stands a quarter-tile off the face — and
                // **only then**, if the target is a unit and is not
                // moving, the whole call is `find_melee_pos` and a
                // return (`6017a1`-`6017f4`, §17.6).
                stand = 0x30;
                if let Obj::Unit(t) = target
                    && !self.is_moving(t)
                {
                    return self.find_melee_pos(u, t, self.units[u].pos);
                }
            } else if d < p.min_range * 0xc0 - 6 && !bombard {
                // SEAM (`6014f1`): the asker is inside its own dead zone
                // and `project`s the approach point away from the target
                // before walking the ring. Nothing modelled here has a
                // minimum range, so the projection is skipped and only
                // the stand-off is taken.
                stand = p.min_range * 0xc0 + 0x90;
            } else if range * 0xc0 - 6 < d || bombard {
                let mut v = range * 0xc0 - 0x60;
                if range < 10 {
                    v += p.big_radius - 0x30;
                }
                if target_is_unit {
                    v -= tp.big_radius;
                }
                stand = v.max(0xc0);
            } else {
                // Neither arm: the stand-off is the measured distance.
                stand = d;
            }
            far = false;
        } else {
            // **The far arm** (`6015e0`-`6015fc`): a unit target further
            // than `(range + 8)` tiles. `local_2c` is set, and it is what
            // makes the spot below unconditional — a chase this far out
            // takes whatever the sweep offers without asking whether it
            // would be in range there.
            stand = (range + 2) * 0xc0;
            far = true;
        }

        let Obj::Building(_) = target else {
            return self.chase_spot(u, target, from, stand, far);
        };

        // `local_34 == 0xc` (`601616`): an asker whose activity is a
        // GUARD order is not moved at all.
        if activity == Some(index::GUARD) {
            return None;
        }

        self.ring_walk(u, target, from, stand, site)
    }

    /// **The unit half of `Unit::find_attack_pos@00601280`** — where a
    /// chaser stands to shoot a *unit*, which is a sweep around the
    /// target rather than a walk round a footprint (`docs/COMBAT.md`
    /// §32).
    ///
    /// `00601280` splits on the target's `is_build` (`+0x1c`) at `601604`.
    /// The building side is [`Self::ring_walk`]; this is the other, and
    /// until item 462 it was a `return None` that left every chaser
    /// walking at the target's own seat.
    ///
    /// Three steps and nothing else:
    ///
    /// 1. **The sweep's radius**, `60256a`-`602595`: `spot = target.
    ///    big_radius + my.big_radius + stand`. Over `0x240` it becomes a
    ///    *band* — `min = spot − 0xc0`, `max = spot`, `step = 0x60`, so
    ///    three rings — and at or under it a single ring at `spot` with
    ///    the defaults (`max = 0`, `step = 0`).
    /// 2. **The bearing**, `local_24`: `find_angle` of the approach
    ///    vector, so the sweep's first candidate is the point on the
    ///    target's own side of the asker. [`Sim::find_nearby_spot`] walks
    ///    `0, ±1, ±2, … ±7` sixteenths from it.
    /// 3. **The acceptance**, `6025dd`-`602620`: a spot the sweep found is
    ///    taken outright when the far arm asked (`local_2c`) or when the
    ///    flanking projection supplied the centre (`local_40`); otherwise
    ///    only if the asker would be **in range from it**. Anything else
    ///    falls through to the caller's fallback, which is the target's
    ///    own position.
    ///
    /// **Measured against run112 before it was written**: the three
    /// slingers' 622 destinations — `(1608, 8184)`, `(1560, 7848)` and
    /// `(1704, 8424)` — are all on the inner ring of a `stand` of 1008,
    /// at bearings `0`, `+1` and `−1` sixteenths from this angle
    /// (`docs/COMBAT.md` §32.2).
    ///
    /// **SEAM — the flanking branch** (`602870`-`602a9c`, `local_40`).
    /// When the target carries a **move** order the original first asks
    /// whether its back is turned: within 60° of running away it may
    /// return the asker's own position outright, and otherwise
    /// `flanking()` projects a point ahead of the target and sweeps from
    /// *there*. Neither is modelled; a moving target takes the centre a
    /// standing one would. Chapter two's target stands still, so the
    /// branch is unexercised by everything on disk, and
    /// `docs/COMBAT.md` §32.4 names the capture that would reach it.
    fn chase_spot(
        &mut self,
        u: usize,
        target: Obj,
        from: Pos,
        stand: i32,
        far: bool,
    ) -> Option<Pos> {
        let me = Obj::Unit(u);
        let t = self.pos_of(target);
        let spot = self.profile(target).big_radius + self.profile(me).big_radius + stand;
        let (min, max, step) = if spot > 0x240 {
            (spot - 0xc0, spot, 0x60)
        } else {
            (spot, 0, 0)
        };
        let angle = crate::movement::find_angle(from.x - t.x, from.y - t.y);
        let p = self.find_nearby_spot(u, t, min, max, step, angle, None)?;
        (far || self.is_in_range_at(me, p, target)).then_some(p)
    }

    /// `UnitData::is_moving@00610af0` — the **head** order's virtual
    /// `+0x14`, which is `Order::is_move`: true for the seven classes
    /// derived from `MoveOrder` and false for everything else
    /// (`vtables.txt`, where the slot COMDAT-folds onto the two return
    /// constants and `MoveOrder::is_move` is the surviving name).
    pub(crate) fn is_moving(&self, u: usize) -> bool {
        self.units[u].orders.front().is_some_and(Order::is_move)
    }

    /// `Unit::find_melee_pos@006010b0` — where a **melee** asker stands to
    /// hit a unit that is not moving (`docs/COMBAT.md` §19).
    ///
    /// The whole function is [`Self::find_open_slots`] and a minimum: the
    /// open slot nearest `from` by `vector_dist`, ties to the earlier one
    /// in ring order (`6011f5`'s `jle` keeps the incumbent). `None` is the
    /// original's `return 0`, on which `find_attack_pos` writes the
    /// target's own position into its out-parameters and returns 0 too —
    /// which is what every caller falls back to anyway.
    fn find_melee_pos(&mut self, u: usize, t: usize, from: Pos) -> Option<Pos> {
        let slots = self.find_open_slots(u, t);
        if slots.is_empty() {
            // SEAM (`601178`-`6011a3`): the target's `UnitData::full`
            // (`+0xac`) takes `+5`, capped at `0x1e` — the "nobody can
            // get at this one" counter. Nothing in this crate keeps
            // `full`, and nothing here reads it.
            return None;
        }
        let mut best = slots[0];
        let mut best_d = 99_999_999;
        for s in slots {
            let d = vector_dist(s.x - from.x, s.y - from.y);
            if d < best_d {
                best_d = d;
                best = s;
            }
        }
        Some(best)
    }

    /// `Unit::find_open_slots@00600e30` — the square ring of quarter-tile
    /// centres around a unit target that nothing occupies.
    ///
    /// The ring is axis-aligned and walked once anticlockwise from its
    /// **south-west** corner `(cx − r, cy + r)`, one direction per side
    /// out of `orthog_x@00add250` / `orthog_y@00add210` indices 1..4 —
    /// `(0, −1)`, `(1, 0)`, `(0, 1)`, `(−1, 0)` — a `step` of
    /// `2 × asker.coll_size + 1` quarter-tiles at a time, the corner
    /// clamped back onto the band each time a coordinate overshoots.
    ///
    /// `r` is `asker.coll_size + target.coll_size` plus **four** when the
    /// asker `is(HOPLITES)` and **one** when it does not (`600e94`'s
    /// devirtualised `ObjectData::is(0x84, 0)`, `600eaa`/`600ebc`).
    /// `coll_size` is the type's `+0x248`, `BLOCK_RADIUS` before its
    /// `× 48`.
    ///
    /// Each candidate is snapped to the quarter-tile centre and tested by
    /// four predicates in order — the map bounds, `UnitData::invalid_loc`
    /// (with `param_3` **clear**, unlike §17.4's ring), `find_collision`
    /// and `find_ordered_collision`. There is **no draw**: the whole
    /// function is deterministic, which is why a melee chase costs the
    /// frame nothing.
    fn find_open_slots(&mut self, u: usize, t: usize) -> Vec<Pos> {
        /// `orthog_x@00add250[1..=4]` and `orthog_y@00add210[1..=4]`, the
        /// four sides in the order the walk takes them.
        const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        let me = Obj::Unit(u);
        let mine = self.coll_size(u);
        let step = mine * 2 + 1;
        let spread = if self.profile(me).hoplites { 4 } else { 1 };
        let r = mine + self.coll_size(t) + spread;
        let c = crate::collide::ucell(self.units[t].pos);
        let (cx, cy) = (c.x, c.y);
        let (mut x, mut y) = (cx - r, cy + r);
        let (w, h) = (
            self.world.width() * UCELLS_PER_CELL,
            self.world.height() * UCELLS_PER_CELL,
        );
        let mut side = 0usize;
        let mut out = Vec::new();
        loop {
            if x >= 0 && y >= 0 && x < w && y < h {
                let at = Pos::new(
                    x * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
                    y * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
                );
                let tl = Pos::new(x / UCELLS_PER_UTILE, y / UCELLS_PER_UTILE);
                if self.invalid_loc(u, tl, false, false, false, false, true) == 0
                    && !self.find_collision(u, at)
                    && !self.find_ordered_collision(u, at)
                {
                    out.push(at);
                }
            }
            x += DIRS[side].0 * step;
            y += DIRS[side].1 * step;
            let over_x = (x - cx).abs() > r;
            if over_x {
                x = if x > cx { cx + r } else { cx - r };
            } else if (y - cy).abs() > r {
                y = if y > cy { cy + r } else { cy - r };
            } else {
                continue;
            }
            side += 1;
            if side >= DIRS.len() {
                return out;
            }
            x += DIRS[side].0 * step;
            y += DIRS[side].1 * step;
        }
    }

    /// §17.2 and §17.4: the ring around a building's footprint.
    fn ring_walk(
        &mut self,
        u: usize,
        target: Obj,
        from: Pos,
        stand: i32,
        site: &'static str,
    ) -> Option<Pos> {
        let me = Obj::Unit(u);
        let p = self.profile(me);
        let tp = self.profile(target);
        let t = self.pos_of(target);
        let (xs, ys) = (tp.x_size * 0x60, tp.y_size * 0x60);

        // **The starting side** — an octant of the vector from the
        // approach point to the target, measured past the footprint.
        let start = Self::start_side(
            from,
            t,
            (from.x - t.x).abs() - xs,
            (from.y - t.y).abs() - ys,
        );

        // **The stride and the arc's resolution**, from the asker's own
        // range and the two footprints (`601b50`-`601bb8`).
        let (stride, q) = if p.max_range == 0 {
            (0x20, 0x2000_0000)
        } else {
            let stride = if p.big_radius < 0x31 || tp.x_size < 3 || tp.y_size < 3 {
                if p.big_radius > 0x18 && tp.x_size > 1 && tp.y_size > 1 {
                    0x40
                } else {
                    0x20
                }
            } else {
                0xc0
            };
            let range = self.max_range_of(me).max(1);
            let q = ((0x4000_0000i64 / i64::from(range)) as u32 / (0xc0u32 / stride as u32)) as i32;
            (stride, q.min(0x2000_0000))
        };
        let k = 0x4000_0000i64 / i64::from(q.max(1));
        let steps_per_side = (k - 1) as i32;
        let angle_step = (0x4000_0000i64 / k.max(1)) as i32;

        // Both arms start on the same side, at the same point.
        let base = Self::start_base(start, t, xs, ys, stand);
        let arc_start = (steps_per_side / 2) + 1;
        let mut arms = [
            Arm {
                side: start,
                step: arc_start,
                base_angle: Self::corner_angle(start),
                at: base,
            },
            Arm {
                side: start,
                step: arc_start,
                base_angle: Self::corner_angle(start),
                at: base,
            },
        ];

        let mut best: Option<(i32, Pos)> = None;
        let mut budget = BUDGET;
        let mut counter = COUNTER_START;
        let mut arm = 0usize;
        loop {
            // The candidate: the arm's point, swept out by the stand-off
            // on a corner side.
            let a = arms[arm];
            let mut c = a.at;
            if a.side & 1 != 0 && stand != 0 {
                let ang = Angle(a.step.wrapping_mul(angle_step).wrapping_add(a.base_angle));
                c = Pos::new(
                    c.x + sin_component(ang, stand),
                    c.y - cos_component(ang, stand),
                );
            }
            let c = snapped(c);
            let tl = c.tile();
            let clear = self.invalid_loc(u, tl, true, false, false, false, true) == 0
                && self.world.tile_mask(tl) & tile::BLOCKED == 0
                && !self.find_collision(u, c)
                && !self.find_ordered_collision(u, c);
            if clear {
                self.mark(site);
                let draw = self.rng.roll() % 0xc0;
                let score = vector_dist(c.x - from.x, c.y - from.y) + draw;
                if best.is_none_or(|(b, _)| score < b) {
                    best = Some((score, c));
                    // **The cut** (`60215d`/`60216f`). A candidate that
                    // reaches the draw always beats a best-so-far of −1,
                    // so this lands on the first draw and the loop then
                    // runs its remaining fourteen (or three) iterations.
                    budget = if stand > 0x300 {
                        budget.min(counter + BUDGET_SLACK_RANGED)
                    } else {
                        budget.min(counter)
                    };
                }
            }
            // Flip to the other arm and step it on.
            arm ^= 1;
            Self::advance(
                &mut arms[arm],
                arm,
                t,
                xs,
                ys,
                stand,
                stride,
                steps_per_side,
            );
            counter += 1;
            if counter - COUNTER_START >= budget {
                break;
            }
        }
        best.map(|(_, p)| p)
    }

    /// §17.2's octant table (`601aa4`-`601b4a`): which of the eight
    /// sides the asker's own bearing puts it on. `ex`/`ey` are the
    /// absolute offsets from the target **less its footprint**, so
    /// `diag` asks whether the asker is clear of both faces rather than
    /// whether it is diagonal from the centre.
    fn start_side(from: Pos, t: Pos, ex: i32, ey: i32) -> i32 {
        let diag = ex >= 1 && ey >= 1;
        if ey < ex {
            if !diag {
                if from.x <= t.x { 8 } else { 4 }
            } else if t.x < from.x {
                if t.y < from.y { 5 } else { 3 }
            } else if t.y < from.y {
                7
            } else {
                1
            }
        } else if !diag {
            if t.y < from.y { 6 } else { 2 }
        } else if t.y < from.y {
            if from.x <= t.x { 7 } else { 5 }
        } else if t.x < from.x {
            3
        } else {
            1
        }
    }

    /// The base angle of a corner side — `0xc0000000`, `0`, `0x40000000`
    /// and `0x80000000` for 1, 3, 5 and 7 (`601c2a`-`601dba`). An even
    /// side has no arc and the value is never read.
    fn corner_angle(side: i32) -> i32 {
        match side {
            1 => 0xc000_0000u32 as i32,
            3 => 0,
            5 => 0x4000_0000,
            7 => 0x8000_0000u32 as i32,
            _ => 0,
        }
    }

    /// A side's starting point. A corner side's is the corner itself —
    /// the arc adds the stand-off. An edge side's is the face pushed out
    /// by the stand-off, entered from the end the arm walks *away* from:
    /// arm 0 walks the ring one way and arm 1 the other, so each enters
    /// an edge at the opposite corner (`6021e6`-`602441`).
    fn side_base(side: i32, t: Pos, xs: i32, ys: i32, stand: i32, arm: usize) -> Pos {
        let far = arm == 0;
        match side {
            1 => Pos::new(t.x - xs, t.y - ys),
            3 => Pos::new(t.x + xs, t.y - ys),
            5 => Pos::new(t.x + xs, t.y + ys),
            7 => Pos::new(t.x - xs, t.y + ys),
            2 => Pos::new(if far { t.x + xs } else { t.x - xs }, t.y - ys - stand),
            4 => Pos::new(t.x + xs + stand, if far { t.y + ys } else { t.y - ys }),
            6 => Pos::new(if far { t.x - xs } else { t.x + xs }, t.y + ys + stand),
            _ => Pos::new(t.x - xs - stand, if far { t.y - ys } else { t.y + ys }),
        }
    }

    /// **The starting point**, which both arms share (`601d88`-`601f54`,
    /// the switch on the start side at `601e3e`; item 1012,
    /// `docs/COMBAT.md` §66). A corner side starts at the corner, and an
    /// **edge** side at the face's **midpoint** pushed out by the
    /// stand-off: side 4's case (`601ea0`-`601ef4`) is `x = t.x + x_size
    /// × 0x60 + stand`, `y = t.y`, and both arms' slots take the pair at
    /// `601e04`-`601e1a`. Only an arm that *steps onto* an edge from a
    /// corner enters it at an end ([`Self::side_base`]). Great Lakes 4605's
    /// `1/17`, the one asker square on the city's east face, stands at
    /// (3912, 30840) in run356's block 4606, level with the centre.
    fn start_base(side: i32, t: Pos, xs: i32, ys: i32, stand: i32) -> Pos {
        match side {
            2 => Pos::new(t.x, t.y - ys - stand),
            4 => Pos::new(t.x + xs + stand, t.y),
            6 => Pos::new(t.x, t.y + ys + stand),
            8 => Pos::new(t.x - xs - stand, t.y),
            _ => Self::side_base(side, t, xs, ys, stand, 0),
        }
    }

    /// One step of one arm (`602183`-`602782`): along the arc on a corner
    /// side, along the face on an edge, and over to the next side when
    /// either runs past the footprint.
    #[allow(clippy::too_many_arguments)] // the ring's six invariants and the arm
    fn advance(
        a: &mut Arm,
        arm: usize,
        t: Pos,
        xs: i32,
        ys: i32,
        stand: i32,
        stride: i32,
        steps_per_side: i32,
    ) {
        let fwd = arm != 0;
        if a.side & 1 != 0 {
            a.step += if fwd { 1 } else { -1 };
            if a.step > steps_per_side || a.step < 1 {
                a.side += if fwd { 1 } else { -1 };
                a.side = wrap_side(a.side);
                a.at = Self::side_base(a.side, t, xs, ys, stand, arm);
            }
            return;
        }
        // An edge: one stride along the face, then the overshoot test
        // against the footprint's own half-width on that axis.
        let (over, limit) = match a.side {
            2 => {
                a.at.x += if fwd { stride } else { -stride };
                (t.x - a.at.x, xs)
            }
            4 => {
                a.at.y += if fwd { stride } else { -stride };
                (t.y - a.at.y, ys)
            }
            6 => {
                a.at.x += if fwd { -stride } else { stride };
                (t.x - a.at.x, xs)
            }
            _ => {
                a.at.y += if fwd { -stride } else { stride };
                (t.y - a.at.y, ys)
            }
        };
        if over.abs() <= limit {
            return;
        }
        a.side += if fwd { 1 } else { -1 };
        a.side = wrap_side(a.side);
        a.base_angle = Self::corner_angle(a.side);
        a.at = Self::side_base(a.side, t, xs, ys, stand, arm);
        a.step = if fwd { 1 } else { steps_per_side };
    }
}

/// 1..8, wrapping both ways — arm 0 only ever decrements and arm 1 only
/// increments, and the listing carries one test in each branch.
fn wrap_side(s: i32) -> i32 {
    if s < 1 {
        8
    } else if s > 8 {
        1
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The octant table of §17.2, read back as a function of where the
    /// asker stands. The sides run **anticlockwise from the north-west
    /// corner**: odd are corners, even are edge midpoints, and a target
    /// at the origin with a 4 × 4 footprint puts the asker's own compass
    /// bearing on the side facing it.
    ///
    /// Made to fail on purpose by swapping the `ey < ex` arms: the four
    /// diagonals keep their answers and all four axes move, which is why
    /// the axes are here as well as the corners.
    #[test]
    fn the_octant_table_faces_the_asker() {
        let t = Pos::new(10_000, 10_000);
        let (xs, ys) = (4, 4);
        let at = |dx: i32, dy: i32| {
            let from = Pos::new(t.x + dx, t.y + dy);
            let (ex, ey) = (
                (from.x - t.x).abs() - xs * 0x60,
                (from.y - t.y).abs() - ys * 0x60,
            );
            crate::Sim::start_side(from, t, ex, ey)
        };
        // The axes, well outside the footprint: due north is side 2,
        // east 4, south 6, west 8.
        assert_eq!(at(0, -5000), 2, "north");
        assert_eq!(at(5000, 0), 4, "east");
        assert_eq!(at(0, 5000), 6, "south");
        assert_eq!(at(-5000, 0), 8, "west");
        // The diagonals are the corners between them.
        assert_eq!(at(-5000, -5000), 1, "north-west");
        assert_eq!(at(5000, -5000), 3, "north-east");
        assert_eq!(at(5000, 5000), 5, "south-east");
        assert_eq!(at(-5000, 5000), 7, "south-west");
        // Great Lakes' own archers: far east and a little north of the
        // farm, which is the north-east corner.
        assert_eq!(at(34_392, -7_848), 3, "run19's `1/27`");
    }

    /// The two arms leave the same point in opposite directions, and an
    /// edge is entered from the end the arm walks away from (§17.2).
    ///
    /// Made to fail on purpose by giving both arms `arm = 0` in
    /// [`crate::Sim::side_base`]: the two arms then start the same edge
    /// at the same corner and walk into each other.
    #[test]
    fn the_two_arms_leave_the_same_edge_at_opposite_ends() {
        let t = Pos::new(10_000, 10_000);
        let (xs, ys) = (384, 384);
        let d = 0x30;
        // Side 2 is the north face. Arm 0 walks the ring anticlockwise
        // and so enters from the east end; arm 1 from the west.
        let a0 = crate::Sim::side_base(2, t, xs, ys, d, 0);
        let a1 = crate::Sim::side_base(2, t, xs, ys, d, 1);
        assert_eq!(a0, Pos::new(t.x + xs, t.y - ys - d));
        assert_eq!(a1, Pos::new(t.x - xs, t.y - ys - d));
        assert_eq!(a0.y, a1.y, "one face, one stand-off");
        // A corner's base point is the corner itself — the arc adds the
        // stand-off — and both arms share it.
        for side in [1, 3, 5, 7] {
            assert_eq!(
                crate::Sim::side_base(side, t, xs, ys, d, 0),
                crate::Sim::side_base(side, t, xs, ys, d, 1),
                "side {side} is a corner"
            );
        }
        assert_eq!(
            crate::Sim::side_base(1, t, xs, ys, d, 0),
            Pos::new(t.x - xs, t.y - ys)
        );
        assert_eq!(
            crate::Sim::side_base(5, t, xs, ys, d, 0),
            Pos::new(t.x + xs, t.y + ys)
        );
    }

    /// **An edge start is the face's midpoint** (item 1012,
    /// `docs/COMBAT.md` §66): the start-side switch at `601e3e` puts both
    /// arms on `(t.x + x_size × 0x60 + d, t.y)` for side 4
    /// (`601ea0`-`601ef4`), and likewise for 2, 6 and 8; only an arm
    /// stepping onto an edge from a corner enters it at an end. Great Lakes
    /// 4605's `1/17`, square on the city's east face, stands at (3912,
    /// 30840) in run356's block 4606, level with the centre.
    ///
    /// Made to fail by starting an edge at [`crate::Sim::side_base`]'s
    /// end: every edge assertion names the corner.
    #[test]
    fn an_edge_start_is_the_face_s_midpoint() {
        let t = Pos::new(10_000, 10_000);
        let (xs, ys) = (384, 384);
        let d = 0x30;
        let at = |side| crate::Sim::start_base(side, t, xs, ys, d);
        assert_eq!(at(2), Pos::new(t.x, t.y - ys - d), "north");
        assert_eq!(at(4), Pos::new(t.x + xs + d, t.y), "east");
        assert_eq!(at(6), Pos::new(t.x, t.y + ys + d), "south");
        assert_eq!(at(8), Pos::new(t.x - xs - d, t.y), "west");
        for side in [1, 3, 5, 7] {
            assert_eq!(
                at(side),
                crate::Sim::side_base(side, t, xs, ys, d, 0),
                "side {side} is a corner, and starts on it"
            );
        }
    }

    /// **The golden record's own melee chase** (§19): chapter one's
    /// `0/6`, ordered onto `1/6` at the end of frame 616, asks where to
    /// stand on 617 and the original's dump answers `(1080, 8280)` —
    /// `orders_x`/`orders_y` at block 618, with the five-node path stack
    /// beneath it.
    ///
    /// Every number here is the golden dump's: the six positions at block
    /// 617, and the three `orders_x`/`orders_y` that decide the ring's
    /// rejections — `0/7` seated on its own point, `1/7` walking to
    /// `(1320, 7800)` and `1/8` to `(1176, 8088)`, all three from the
    /// original's own block 617 (`docs/RUNS.md` run101–run105).
    ///
    /// **It is the rejections that make the answer, not the distance.**
    /// `(1080, 8280)` is the *farthest* of the ring's four corners from
    /// the asker; six nearer candidates beat it on `vector_dist` and every
    /// one of them is inside two unit cells of an ordered destination.
    /// Made to fail on purpose by seating `1/7` and `1/8` on their own
    /// points instead of their ordered ones — the call then answers
    /// `(1080, 7992)`, two rows up the same column.
    #[test]
    fn chapter_one_s_melee_chase_stands_where_the_golden_dump_puts_it() {
        use crate::combat::Profile;
        use crate::world::World;
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        // HOPLITES as the data loads it: `BLOCK_RADIUS 1` (so `+0x248` is
        // 1 and the ring's step is 3), melee, and in its own lineage.
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 120,
            combat: Profile {
                attack: 15,
                uber_size: 1,
                block_radius: 48,
                big_radius: 48,
                combat_role: true,
                hoplites: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let put = |sim: &mut crate::Sim, who: crate::Player, p: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(who, index, p, 120);
            u.ty = Some(ty);
            u.on_map = true;
            sim.add_unit(u)
        };
        // Block 617's `x_internal`/`y_internal`, in the dump's own order.
        let a6 = put(&mut sim, 0, Pos::new(888, 7800));
        let a7 = put(&mut sim, 0, Pos::new(1032, 7800));
        let a8 = put(&mut sim, 0, Pos::new(936, 7944));
        let b6 = put(&mut sim, 1, Pos::new(1368, 7992));
        let b7 = put(&mut sim, 1, Pos::new(1491, 7973));
        let b8 = put(&mut sim, 1, Pos::new(1388, 8131));
        // And block 617's `orders_x`/`orders_y`.
        sim.units[a6].orders_pos = Pos::new(888, 7800);
        sim.units[a7].orders_pos = Pos::new(1032, 7800);
        sim.units[a8].orders_pos = Pos::new(936, 7944);
        sim.units[b6].orders_pos = Pos::new(1368, 7992);
        sim.units[b7].orders_pos = Pos::new(1320, 7800);
        sim.units[b8].orders_pos = Pos::new(1176, 8088);
        let from = sim.units[a6].pos;
        assert_eq!(
            sim.find_attack_pos(a6, Obj::Unit(b6), from, crate::fight::SITE_ATTACK_POS_FIGHT),
            Some(Pos::new(1080, 8280)),
            "the golden dump's own `orders_x`/`orders_y` for `0/6` at block 618"
        );
    }

    /// **The golden record's own *ranged* chase** (§32.2): chapter two's
    /// slinger `0/9`, ordered onto the hoplite `1/8` at the end of frame
    /// 621, asks where to stand and run112's dump answers
    /// `(1608, 8184)` — `orders_x`/`orders_y` at block 622, with a
    /// six-node path stack beneath it.
    ///
    /// Every number is the dump's: `0/9` at `(888, 8376)` and `1/8` at
    /// `(2472, 7944)` are block 622's `x_internal`/`y_internal`, and the
    /// answer is that block's `MOVEORDER x`/`y`. The target carries no
    /// order at all there (`STACK<TYPE> length 0`), which is what keeps
    /// the call off §32.4's unmodelled flanking branch.
    ///
    /// The whole chain is pinned by one point: `stand` is 1008, the band
    /// is `[912, 1104]` by `0x60`, the bearing is `find_angle(from −
    /// target)` and the first candidate of the **inner** ring is the
    /// answer.
    ///
    /// Made to fail on purpose by sweeping from `spot` instead of
    /// `spot − 0xc0` — one ring out on the same bearing, and the call
    /// then answers `(1416, 8232)`.
    #[test]
    fn chapter_two_s_ranged_chase_stands_where_the_golden_dump_puts_it() {
        use crate::combat::Profile;
        use crate::world::World;
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.at_war[0][1] = true;
        sim.at_war[1][0] = true;
        let mut kind = |max_range: i32, hits: i32| {
            sim.add_unit_type(crate::UnitType {
                hits,
                combat: Profile {
                    attack: 15,
                    max_range,
                    uber_size: 1,
                    block_radius: 48,
                    big_radius: 48,
                    combat_role: true,
                    ..Profile::default()
                },
                ..crate::UnitType::default()
            })
        };
        let slinger = kind(6, 85);
        let hoplite = kind(0, 120);
        let put = |sim: &mut crate::Sim, who: crate::Player, ty: usize, p: Pos, hits: i32| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(who, index, p, hits);
            u.ty = Some(ty);
            u.on_map = true;
            let h = sim.add_unit(u);
            sim.units[h].orders_pos = p;
            h
        };
        let a9 = put(&mut sim, 0, slinger, Pos::new(888, 8376), 85);
        let b8 = put(&mut sim, 1, hoplite, Pos::new(2472, 7944), 120);
        // The stand-off the band is built on, stated so a change to §17.3
        // fails here rather than silently moving the ring.
        assert_eq!(
            sim.attack_dist(Obj::Unit(a9), Obj::Unit(b8)),
            1468,
            "block 622's own geometry"
        );
        let from = sim.units[a9].pos;
        assert_eq!(
            sim.find_attack_pos(a9, Obj::Unit(b8), from, crate::fight::SITE_ATTACK_POS_FIGHT),
            Some(Pos::new(1608, 8184)),
            "run112's own `orders_x`/`orders_y` for `0/9` at block 622"
        );
        // **The far arm takes the same road and a wider stand-off.** Past
        // `(range + 8) × 0xc0` the answer is unconditional — no `is_in_
        // range` test — so a chaser this far out still gets a ring point
        // and not the target's seat, which is what the whole of §32.2
        // exists to stop.
        let far = put(&mut sim, 0, slinger, Pos::new(888, 16_000), 85);
        let p = sim
            .find_attack_pos(
                far,
                Obj::Unit(b8),
                sim.units[far].pos,
                crate::fight::SITE_ATTACK_POS_FIGHT,
            )
            .expect("the far arm sweeps rather than falling through");
        assert_ne!(
            p, sim.units[b8].pos,
            "a chaser must never be sent to the target's own seat"
        );
        assert!(
            !sim.is_in_range_at(Obj::Unit(far), p, Obj::Unit(b8)),
            "the far arm's spot is accepted without the range test (`local_2c`)"
        );
    }

    /// The ring itself, read back as the seventeen quarter-tile centres
    /// `find_open_slots` walks — the south-west corner first, then north
    /// up the west side, east along the north, south down the east and
    /// west along the south, closing on the corner it started from.
    ///
    /// Made to fail on purpose by walking the ring the other way round —
    /// the wrap is not symmetric, and the reversed table answers thirteen
    /// slots along the south, east and north sides and never the west.
    #[test]
    fn the_open_slot_ring_walks_four_sides_and_closes() {
        use crate::combat::Profile;
        use crate::world::World;
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 120,
            combat: Profile {
                uber_size: 1,
                block_radius: 48,
                big_radius: 48,
                hoplites: true,
                ..Profile::default()
            },
            ..crate::UnitType::default()
        });
        let put = |sim: &mut crate::Sim, who: crate::Player, p: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(who, index, p, 120);
            u.ty = Some(ty);
            u.on_map = true;
            sim.add_unit(u)
        };
        // The asker stands far away so nothing it carries rejects a slot.
        let asker = put(&mut sim, 0, Pos::new(20_000, 20_000));
        let target = put(&mut sim, 1, Pos::new(1368, 7992));
        sim.units[asker].orders_pos = sim.units[asker].pos;
        sim.units[target].orders_pos = sim.units[target].pos;
        let got = sim.find_open_slots(asker, target);
        let cell = |x: i32, y: i32| Pos::new(x * 0x30 + 0x18, y * 0x30 + 0x18);
        // `1/6` sits in unit cell (28, 166); `r` is 1 + 1 + 4 and the step
        // is 2 × 1 + 1.
        let want: Vec<Pos> = [
            (22, 172),
            (22, 169),
            (22, 166),
            (22, 163),
            (22, 160),
            (25, 160),
            (28, 160),
            (31, 160),
            (34, 160),
            (34, 163),
            (34, 166),
            (34, 169),
            (34, 172),
            (31, 172),
            (28, 172),
            (25, 172),
            (22, 172),
        ]
        .iter()
        .map(|&(x, y)| cell(x, y))
        .collect();
        assert_eq!(got, want, "the ring, corner to corner and back");
    }

    /// `wrap_side` closes the ring both ways — arm 0 only decrements and
    /// arm 1 only increments, so each needs one of them.
    #[test]
    fn the_ring_closes_both_ways() {
        assert_eq!(wrap_side(0), 8);
        assert_eq!(wrap_side(9), 1);
        for s in 1..=8 {
            assert_eq!(wrap_side(s), s);
        }
    }
}
