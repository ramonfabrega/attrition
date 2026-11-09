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

/// `-0x44` at `601425`: `max(0x600, (max_range + 4) × 0xc0)`, the
/// distance from which a mandatory melee asker with no open slot walks to
/// the target's cell rather than giving up (`601868`).
fn melee_far(max_range: i32) -> i32 {
    ((max_range + 4) * 0xc0).max(0x600)
}

/// Quarter-tiles to a tile — the `>> 2` `find_open_slots` applies before
/// asking `invalid_loc` (`600f47`, `600f5e`).
const UCELLS_PER_UTILE: i32 = 4;
use crate::attrition::Domain;
use crate::combat::{self, Obj};
use crate::movement::{Angle, cos_component, sin_component};
use crate::orders::{Order, snapped};
use crate::world::{Pos, tile, vector_dist};

/// What [`Sim::chase_lead`] makes of a moving unit target.
enum Lead {
    /// Not led: the sweep runs round the target's own position.
    None,
    /// The asker outranges a target coming at it: it stays where it is.
    Stand(Pos),
    /// The target is running: the sweep runs round this point ahead of it,
    /// and its spot is taken whether or not it is in range.
    Ahead(Pos),
}

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

/// `Unit::find_melee_pos@006010b0`'s minimum (`6011c7`-`60121b`): the slot
/// nearest `from` by `vector_dist`, ties to the earlier one; `None` for no
/// slots.
pub fn nearest_slot(slots: &[Pos], from: Pos) -> Option<Pos> {
    let mut best = *slots.first()?;
    let mut best_d = 99_999_999;
    for s in slots {
        let d = vector_dist(s.x - from.x, s.y - from.y);
        if d < best_d {
            best_d = d;
            best = *s;
        }
    }
    Some(best)
}

/// The walk of `Unit::find_open_slots@00600e30`, free of the world: `mine`
/// and `theirs` are the two `coll_size`s, `hoplites` the asker's
/// `is(HOPLITES)`, `c` the target's unit cell, `dims` the world's size in
/// cells. `open(at, tile)` is the three predicates after the bounds
/// (`invalid_loc` with `param_3` clear, `find_collision`,
/// `find_ordered_collision`, in that order and short-circuited); `at` is
/// the candidate's quarter-tile centre, `tile` its tile.
pub fn find_open_slots_steps(
    mine: i32,
    theirs: i32,
    hoplites: bool,
    c: Pos,
    dims: (i32, i32),
    mut open: impl FnMut(Pos, Pos) -> bool,
) -> Vec<Pos> {
    /// `orthog_x@00add250[1..=4]` and `orthog_y@00add210[1..=4]`, the
    /// four sides in the order the walk takes them.
    const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    let step = mine * 2 + 1;
    let spread = if hoplites { 4 } else { 1 };
    let r = mine + theirs + spread;
    let (cx, cy) = (c.x, c.y);
    let (mut x, mut y) = (cx - r, cy + r);
    let (w, h) = (dims.0 * UCELLS_PER_CELL, dims.1 * UCELLS_PER_CELL);
    let mut side = 0usize;
    let mut out = Vec::new();
    loop {
        if x >= 0 && y >= 0 && x < w && y < h {
            let at = Pos::new(
                x * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
                y * UNITS_PER_UCELL + UNITS_PER_UCELL / 2,
            );
            let tl = Pos::new(x / UCELLS_PER_UTILE, y / UCELLS_PER_UTILE);
            if open(at, tl) {
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
    /// written; `None` its fallbacks, on which a caller walks at the
    /// target's own position — the ring walk's own failure is a `return 1`
    /// with exactly that point. The one `return 0` of the building half,
    /// the guard's refusal, is also `None`, and `fight` tells it apart with
    /// [`Sim::attack_pos_refused`] (item 1519).
    ///
    /// **The asker cannot ride while it searches** (`6014ef`–`601505`,
    /// `docs/COMBAT.md` §17.7, item 2845): a set `unit_masks & 0x800000`
    /// ([`crate::Unit::auto_transport`]) is cleared for the whole call and
    /// OR-ed back on every exit (`60160a`, `60193e`, `6027a8`, `602aa8`,
    /// `602b3f`, `602d65`, `602d77`). `invalid_loc`'s land arm admits
    /// water only with the bit, so the ring, the open slots and the sweep
    /// refuse a water spot a computer's unit could otherwise sail to. The
    /// peace pair's Rocket Artillery `1/63` walks its bombard ring past
    /// four water candidates on 6808 and draws eleven times, not fifteen.
    pub(crate) fn find_attack_pos(
        &mut self,
        u: usize,
        target: Obj,
        from: Pos,
        site: &'static str,
    ) -> Option<Pos> {
        let held = std::mem::replace(&mut self.units[u].auto_transport, false);
        let spot = self.find_attack_pos_grounded(u, target, from, site);
        self.units[u].auto_transport |= held;
        spot
    }

    /// [`Self::find_attack_pos`] with the asker's `0x800000` held clear.
    fn find_attack_pos_grounded(
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
        // **The bombard flag, `local_24`** (`601359`-`60139d`, item 1937):
        // `is_siege && unit_masks & 0x40000 && target->is_build` — a
        // computer's siege engine asking for a spot against a building.
        // It skips the first return below (already in range) and puts the
        // stand-off on the maximum-range arm.
        let bombard =
            p.siege && self.unit_ai_bit(self.units[u].owner) && matches!(target, Obj::Building(_));
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
                    return self
                        .find_melee_pos(u, t, self.units[u].pos)
                        .or_else(|| self.melee_far_spot(u, t));
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

        // `local_34 == 0xc` (`6015ea`): an asker whose activity is a
        // GUARD order is not moved at all — [`Sim::attack_pos_refused`].
        if self.attack_pos_refused(u, target) {
            return None;
        }

        self.ring_walk(u, target, from, stand, site)
    }

    /// **`find_attack_pos`'s guard refusal** (`Unit::find_attack_pos@
    /// 00601280`, `6015df`–`601625`, `docs/AI.md` §138): against a
    /// building (vslot `+0x1c`), an asker whose **activity** is a `GUARD`
    /// — `local_34`, read at `6013a2` through `UnitData::get_activity`,
    /// which passes over the moves and attacks in front of it
    /// ([`Sim::guard_activity`]) — takes no spot: the out-parameters get
    /// the caller's own point and the function returns **0**
    /// (`LAB_00601604`), with no draw.
    ///
    /// It is the one `return 0` of the building half a land asker
    /// reaches; the ring walk's own failure returns 1 with the target's
    /// position, which is what `None` means to every other caller. So
    /// `Unit::fight`'s chase asks this predicate to tell the two apart
    /// ([`Sim::chase_refused`]).
    ///
    /// This crate read the **head** order here until item 1519, and a
    /// guard's head mid-attack is the `ATTACK`: French East Indies' Elite
    /// Pikeman `1/115` on 17318 chased the Village `0/2000` from 1,948
    /// units off, where the original's dropped it and stood on its post.
    pub(crate) fn attack_pos_refused(&mut self, u: usize, target: Obj) -> bool {
        match target {
            Obj::Building(_) => self.guard_activity(u).is_some(),
            Obj::Unit(t) => self.melee_slots_refused(u, t) || self.chase_tail_refused(u, t),
        }
    }

    /// **`find_attack_pos`'s unit tail returns 0** (`602b9f`–`602c1e`, item
    /// 3520, `docs/AI.md` §304): a unit target the sweep found no spot for
    /// gets the target's own point in the out-parameters, and then **0**
    /// unless the current attack is mandatory (`local_38`, `[ebp-0x34]`,
    /// `602be5`) and the target stands at least [`melee_far`] off
    /// (`602c1e`). `Unit::fight`'s chase branches on the 0 (`5fe186`) to
    /// [`Sim::chase_refused`]. run1166's packet (logger frame 3920) walks
    /// the Russians' Cruiser `1/25`, holding a non-mandatory attack on the
    /// fleeing citizen `0/5`, through `601604`'s `return 0` and then
    /// `find_new_target`, which finds the building `0/2004`; this crate
    /// walked at the citizen's own point.
    ///
    /// SEAM: the mandatory far arm (`602c24`–`602dc2`): `find_nearby_spot`
    /// around the target and the domain gates after it (a sea asker, and a
    /// land asker against a sea target, return 0). It keeps this crate's
    /// walk at the target's point.
    fn chase_tail_refused(&mut self, u: usize, t: usize) -> bool {
        let mandatory = matches!(
            self.current_order(u).map(|o| &o.body),
            Some(crate::orders::Body::Attack(_))
        ) && self.units[u].combat.mandatory;
        let (me, tp) = (self.units[u].pos, self.units[t].pos);
        !mandatory
            || vector_dist(tp.x - me.x, tp.y - me.y) < melee_far(self.max_range_of(Obj::Unit(u)))
    }

    /// **`find_attack_pos`'s melee `return 0`** (`601803`–`601938`, item
    /// 3385, `docs/COMBAT.md` §19): a melee asker against a unit standing
    /// still whose [`Self::find_melee_pos`] finds no open slot gets the
    /// target's own point in the out-parameters and **0** — unless its
    /// current `ATTACK` is mandatory (`local_34`, the order's `+0x1c`,
    /// `601475`) and the target stands at least [`melee_far`] off, where
    /// it gets [`Self::melee_far_spot`] and 1. `Unit::fight`'s chase
    /// branches on the 0 (`5fe186`) to [`Sim::chase_refused`], so it is a
    /// refusal and not the fallback §19 first read it as. run1011d's
    /// packet (logger frame 13277) walks the Inca Inti Maceman `1/92`
    /// guarding its post through `find_melee_pos` to `601938` and then
    /// `find_new_target`, which re-finds the woodcutter `0/3` and kills
    /// the attack: a lone `GUARD` on block 13278.
    ///
    /// The `in range from where it stands` return and the moving
    /// target's arm come first, as in [`Self::find_attack_pos`]; this is
    /// asked only after it answered `None`.
    fn melee_slots_refused(&mut self, u: usize, t: usize) -> bool {
        let me = Obj::Unit(u);
        if self.profile(me).max_range != 0 || self.is_moving(t) {
            return false;
        }
        let target = Obj::Unit(t);
        let d = self.attack_dist(me, target);
        let range = self.max_range_of(me);
        if d > (range + 8) * 0xc0 {
            return false;
        }
        self.find_open_slots(u, t).is_empty() && self.melee_far_spot(u, t).is_none()
    }

    /// `601833`–`60190c`: the mandatory melee asker's spot when no slot
    /// is open — the target's 768-unit cell centre plus the asker's own
    /// offset within its cell, each axis `div_3_table[v >> 8] × 0x300 +
    /// 0x180 + v % 0x300 − 0x180`, through `WorldData::restrict` — when
    /// `vector_dist(target − asker)` reaches [`melee_far`]; `None` (the
    /// `return 0`) otherwise and for an asker whose current order is not
    /// a mandatory `ATTACK`.
    fn melee_far_spot(&mut self, u: usize, t: usize) -> Option<Pos> {
        let mandatory = matches!(
            self.current_order(u).map(|o| &o.body),
            Some(crate::orders::Body::Attack(_))
        ) && self.units[u].combat.mandatory;
        if !mandatory {
            return None;
        }
        let (me, tp) = (self.units[u].pos, self.units[t].pos);
        if vector_dist(tp.x - me.x, tp.y - me.y) < melee_far(self.max_range_of(Obj::Unit(u))) {
            return None;
        }
        let cell =
            |v: i32, mine: i32| (v >> 8).div_euclid(3) * 0x300 + 0x180 + mine % 0x300 - 0x180;
        Some(self.restrict(Pos::new(cell(tp.x, me.x), cell(tp.y, me.y))))
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
    /// **The flanking branch** (`602870`-`602a9c`, `local_40`) is
    /// [`Sim::chase_lead`]'s: a moving target is led, or the asker stands
    /// (item 3789, `docs/GOLDEN.md` §150.3).
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
        let centre = match self.chase_lead(u, target) {
            Lead::Stand(here) => return Some(here),
            Lead::Ahead(c) => Some(c),
            Lead::None => None,
        };
        let p = self.find_nearby_spot(u, centre.unwrap_or(t), min, max, step, angle, None)?;
        (far || centre.is_some() || self.is_in_range_at(me, p, target)).then_some(p)
    }

    /// **The flanking branch of `Unit::find_attack_pos@00601280`**
    /// (`6027d4`–`602abc`, item 3789, `docs/GOLDEN.md` §150.3): a moving
    /// unit target, asked by a player's ranged unit, is led.
    ///
    /// The bearing is `find_angle(target − asker)` from the asker's own
    /// position (`60281d`). The branch is entered only when the asker's
    /// `unit_masks & 0x40000` is clear ([`Sim::unit_ai_bit`]), its type's
    /// domain (`+0x218`) is air or the target's, the target's current
    /// order is a move (`update_order`, vslot `+0x14`), and the asker's
    /// type has a range (`+0x1fc`). Then, with `e = target.heading −
    /// bearing + 0x80000000`:
    ///
    /// - `flanking(e) == 1` (`6028fa`): the centre of the sweep is
    ///   `project(target, target.heading, ((dist / my_speed) × its_speed
    ///   × 3) / 2)` — `dist` the `vector_dist` between them, each speed the
    ///   vslot `+0x17c` at its own point with 0 — and, when
    ///   `WorldData::is_valid` takes it, the sweep's spot is taken outright
    ///   (`local_40`, `6029f7`); an invalid point leaves the target's.
    /// - `e < 0x2aaaaaaa` or `flanking(e) == 0` (`602a1a`): the target is
    ///   coming. An asker that outranges it (vslot `+0x130`) — or whose
    ///   `unit_masks2 & 0x20000` is set — and stands inside `(range + 4) ×
    ///   0xc0` of it **stays where it is** (`602a85`, `return 1`).
    /// - `flanking(e) == 2`: nothing; the sweep runs round the target.
    ///
    /// run1462's window: the human's Royal Balamob Slingers `0/20` chase
    /// the Greek Light Horse `1/16` on 860 and walk toward (20424, 12216),
    /// four cells past it, where this crate walked at (17112, 14760).
    fn chase_lead(&self, u: usize, target: Obj) -> Lead {
        let Obj::Unit(tu) = target else {
            return Lead::None;
        };
        let me = Obj::Unit(u);
        let p = self.profile(me);
        let tp = self.profile(target);
        if self.unit_ai_bit(self.units[u].owner)
            || !(matches!(p.domain, Domain::Air) || p.domain == tp.domain)
            || !self.is_moving(tu)
            || p.max_range == 0
        {
            return Lead::None;
        }
        let (a, t) = (self.units[u].pos, self.units[tu].pos);
        let bearing = crate::movement::find_angle(t.x - a.x, t.y - a.y);
        let heading = self.units[tu].movement.heading;
        let e = (heading.0 as u32)
            .wrapping_sub(bearing.0 as u32)
            .wrapping_add(0x8000_0000);
        let flank = if e < 0x2aaa_aaaa {
            0
        } else {
            combat::flanking(e)
        };
        match flank {
            1 => {
                let mine = self.get_speed_at(u, a, 0);
                let theirs = self.get_speed_at(tu, t, 0);
                if mine == 0 {
                    // `idivl` by zero faults in the original; a unit with
                    // a range and no speed does not chase.
                    return Lead::None;
                }
                let d = vector_dist(t.x - a.x, t.y - a.y) / mine * theirs * 3 / 2;
                let c = crate::army::step_along(t, heading, d);
                if self.in_world(c) {
                    Lead::Ahead(c)
                } else {
                    Lead::None
                }
            }
            0 => {
                // SEAM: `unit_masks2 & 0x20000`, the marines' trench bit,
                // also stays; no chaser here carries it.
                let range = self.max_range_of(me);
                if range > self.max_range_of(target)
                    && vector_dist(t.x - a.x, t.y - a.y) < (range + 4) * 0xc0
                {
                    Lead::Stand(a)
                } else {
                    Lead::None
                }
            }
            _ => Lead::None,
        }
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
            // `601178`-`6011a3`: the target's [`crate::Unit::full`] takes
            // `+5`, capped at `0x1e` — the "nobody can get at this one"
            // counter the search divides by (item 3385).
            let f = &mut self.units[t].full;
            *f = f.saturating_add(5).min(0x1e);
            return None;
        }
        nearest_slot(&slots, from)
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
        let me = Obj::Unit(u);
        let mine = self.coll_size(u);
        let hoplites = self.profile(me).hoplites;
        let theirs = self.coll_size(t);
        let c = crate::collide::ucell(self.units[t].pos);
        let dims = (self.world.width(), self.world.height());
        find_open_slots_steps(mine, theirs, hoplites, c, dims, |at, tl| {
            self.invalid_loc(u, tl, false, false, false, false, true) == 0
                && !self.find_collision(u, at)
                && !self.find_ordered_collision(u, at)
        })
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
mod tests;
