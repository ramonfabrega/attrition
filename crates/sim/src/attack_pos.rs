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

use crate::combat::Obj;
use crate::movement::{Angle, cos_component, sin_component};
use crate::orders::{index, snapped};
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
                // melee asker stands a quarter-tile off the face. The
                // `find_melee_pos` arm beyond it needs a **unit** target
                // that is not moving, so a building never reaches it.
                stand = 0x30;
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
        } else {
            // The far arm. It needs a **unit** target, and a unit target
            // never reaches the ring, so this only feeds the half of the
            // function that is not modelled.
            return None;
        }

        let Obj::Building(_) = target else {
            // SEAM: a unit target takes `00601280`'s other half — the
            // flanking chase and `UnitType::find_nearby_spot` — which is
            // not modelled. The caller keeps its straight-line approach.
            return None;
        };

        // `local_34 == 0xc` (`601616`): an asker whose activity is a
        // GUARD order is not moved at all.
        if activity == Some(index::GUARD) {
            return None;
        }

        self.ring_walk(u, target, from, stand, site)
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
        let base = Self::side_base(start, t, xs, ys, stand, 0);
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
