//! The bird's flight — `Unit::do_air_physics@005e86d0` and the two helpers
//! that decide where it points, `Unit::bank_aircraft@005e9520` and
//! `Unit::air_turn_speed@005ea390`.
//!
//! `docs/SYNC.md` §3.9 has the whole of it. In one paragraph: a wild bird
//! carries a single air-patrol order from birth, `Animal::think_bird` walks
//! that order's one waypoint about the map, and this is the half that flies
//! the bird at it. Nothing dumps owner 9, so the flight has exactly one
//! observable — the draw at `Unit::do_air_physics+0x639`, a coin thrown when
//! the projected step leaves the world and the order is not already turning
//! away from an edge. That draw is the whole reason this module exists: it
//! is what parted Great Lakes' word at 1802 and East Indies' at 5437.
//!
//! **A bird takes a narrow path through a wide function.** `do_air_physics`
//! is written for the game's aircraft, and almost every branch in it asks a
//! question a bird answers the same way every frame. What is left, and what
//! this module implements, is: the heading toward the patrol point, the bank
//! that turns toward that heading at a rate the bank angle itself scales,
//! the step along the heading, and the edge coin. The arms it does *not*
//! take are named at their sites below, because the next reader's first
//! question will be whether they were read at all.

use crate::Sim;
use crate::movement::{Angle, cos_component, find_angle, sin_component};
use crate::orders::CRUISING_ALT;
use crate::single::Single;
use crate::world::{Pos, UNITS_PER_CELL, vector_dist};

/// The coin the bird throws when its next step would leave the world:
/// `Random::get(0, 0xffff)`, whose low bit alone picks the way it turns.
pub const SITE_AIR_TURN: &str = "Unit::do_air_physics+0x639";

/// **A non-bomber plane's altitude redraw** — `Random::get(0, 0xffff)` at
/// `Unit::do_air_physics+0xba` (`0x5e878a`) on every frame `(o + frame) &
/// 7 == 0` of a flight, and `cruising_alt = (r % 7 + 13) · 100`
/// (`docs/ORDERS.md` §33.1). Chapter seventeen's first word, 642.
pub const SITE_AIR_ALT: &str = "Unit::do_air_physics+0xba";

/// `ObjectData::is(0x130)` — the Bomber line, which holds its
/// `cruising_alt` at 0x640 and throws no redraw.
const BOMBER: crate::tech::TypeId = 0x130;

/// Half a degree a frame — `0x5b05b0`, the floor under every turn rate a
/// non-hovering type can be given. `GuyData::turn_speed` has the same one
/// (`crate::movement`), spelled `UNIT_TURN_SPEED * 0xb60b` there and as a
/// literal here, which is what the original does too.
pub const MIN_TURN: u32 = 0x005b_05b0;

/// Five degrees, as `degrees_to_angle` makes them — the gap inside which a
/// bank that has settled snaps the heading onto the one it wants. A literal
/// in `bank_aircraft`, and not the bird's own `TURN_SPEED` of five degrees,
/// which it coincidentally equals.
const SNAP_WITHIN: u32 = 0x038e_38e3;

/// Forty-five degrees: past this much owed, a bird close to its patrol point
/// keeps the heading it has rather than swinging round.
const HALF_QUARTER: u32 = 0x2000_0000;

// The banking's constants, by their bits — `single::Single` exists so that
// none of them is a float.
const F2: Single = Single::from_bits(0x4000_0000);
const F5: Single = Single::from_bits(0x40a0_0000);
const F10: Single = Single::from_bits(0x4120_0000);
const F55: Single = Single::from_bits(0x425c_0000);
const F55_NEG: Single = Single::from_bits(0xc25c_0000);
const F_HALF: Single = Single::from_bits(0x3f00_0000);
/// `0.33`, and the nearest single to it — the original's own literal, which
/// is not a third.
const F_33: Single = Single::from_bits(0x3ea8_f5c3);
// `pitch_aircraft`'s, by their addresses in the listing: `0xb69514` 1,
// `0xb69798` −1, `0xb69670` 40, `0xb697b8` −40, `0xb6964c` 20,
// `0xb697ac` −20, `0xb69658` 25 (`docs/ORDERS.md` §33.4).
const F1: Single = Single::from_bits(0x3f80_0000);
const F1_NEG: Single = Single::from_bits(0xbf80_0000);
const F20: Single = Single::from_bits(0x41a0_0000);
const F20_NEG: Single = Single::from_bits(0xc1a0_0000);
const F25: Single = Single::from_bits(0x41c8_0000);
const F40: Single = Single::from_bits(0x4220_0000);
const F40_NEG: Single = Single::from_bits(0xc220_0000);

/// Three degrees (`0x2222220`): inside it, a returning plane's second turn
/// takes the heading it wants outright (`bank_aircraft`, `LAB_005e97b9`).
const RETURN_SNAP: u32 = 0x0222_2220;

/// **A plane's figure in the air** — the five fields of guy 0 that
/// `Unit::bank_aircraft@005e9520` and `Unit::pitch_aircraft@005e8de0`
/// write on every frame of a flight, and that the dump prints on the
/// `GUY` record (`docs/ORDERS.md` §33.3).
///
/// `bank` and `pitch` are singles, stored by their bits so the simulation
/// holds no float; the altitude `z` is an integer the pitch steps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Airframe {
    /// `GuyData +0x44`, the bank, in the guy's sign convention — the
    /// negation of the one `bank_aircraft` works in, as a bird's
    /// [`Flight::roll`] is.
    pub bank: Single,
    /// `+0x48`, the bank the frame before.
    pub last_bank: Single,
    /// `+0x4c`, the pitch: stepped by two a frame inside ±40.
    pub pitch: Single,
    /// `+0x50`, the pitch the frame before.
    pub last_pitch: Single,
    /// `+0x14`, the figure's altitude — the `GUY` record's `z`.
    pub z: i32,
    /// `+0x70`, the altitude before the last step that moved it.
    pub last_z: i32,
}

/// A bird's flight state: the two fields of the original's that this crate
/// has to keep, because both are read a frame later.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flight {
    /// `GuyData +0x44`, the bank angle — a single-precision accumulator
    /// stepped by at most ten a frame. `bank_aircraft` works with its
    /// negation throughout and stores the negation back, so this field is
    /// the guy's own sign convention and not the working one.
    pub roll: Single,
    /// The air order's `+0x10`: which way the bird is turning away from an
    /// edge, `+1` or `-1`, and `0` when it is not. Set by the coin, cleared
    /// by the first step that lands inside the world.
    pub turn: i32,
}

/// The shortest way round from `from` to `to`, as a magnitude — the
/// original's `d = to − from; if (0x80000000 < d) d = ~d`, an unsigned
/// compare and a bitwise not rather than a negation, so the answer for
/// exactly half a turn is `0x80000000` and not `0x7fffffff`.
const fn owed(from: Angle, to: Angle) -> u32 {
    let d = (to.0 as u32).wrapping_sub(from.0 as u32);
    if d > 0x8000_0000 { !d } else { d }
}

/// `Unit::air_turn_speed(sign, 0)` — how far the heading may move this
/// frame, which is the bank angle's own magnitude scaled onto the type's
/// rate.
///
/// `base` is the **unfloored** `(TURN_SPEED >> 8) × UNIT_TURN_SPEED`: the
/// floor is applied after the scaling here and before it in
/// [`Sim::bank_aircraft`]'s divisor, which is the same number for every
/// shipped type but not the same expression.
///
/// `roll` is the guy's *stored* bank, read before this frame's has been
/// written — so a frame that reverses the bank turns at the floor rather
/// than at the old rate, which is the `(u ^ sign) < 0` arm.
fn air_turn_speed(base: u32, roll: Single, sign: i32) -> u32 {
    let v = roll.to_i32().wrapping_neg();
    if (v ^ sign) < 0 {
        return MIN_TURN;
    }
    let rate = (base / 0x37).wrapping_mul(v.unsigned_abs());
    if rate < MIN_TURN { MIN_TURN } else { rate }
}

impl Sim {
    /// A bird's [`Flight`], which starts at a zero bank and no turn.
    pub fn flight(&self, u: usize) -> Flight {
        self.gaia
            .bird_flight
            .iter()
            .find(|(b, _)| *b == u)
            .map_or(Flight::default(), |(_, f)| *f)
    }

    fn set_flight(&mut self, u: usize, f: Flight) {
        match self.gaia.bird_flight.iter_mut().find(|(b, _)| *b == u) {
            Some((_, slot)) => *slot = f,
            None => self.gaia.bird_flight.push((u, f)),
        }
    }

    /// `Unit::do_air_physics`, on the path a wild bird takes through it.
    ///
    /// `goal` is the air order's one waypoint — the patrol point
    /// `Animal::think_bird` has just moved. The arms a bird does not take,
    /// in the original's own order:
    ///
    /// - **the altitude re-roll** at `+0x3b`, every eighth frame. Its draw
    ///   is behind `is_animal() == 0 || type == GULLBIRD`, which a wild bird
    ///   fails, so the else arm gives it a flat `0x640 + 200` and spends
    ///   nothing. Altitude is not modelled at all: `pitch_aircraft` moves it
    ///   and reads it, and every use of it downstream — the bank's
    ///   ground-clearance test, the speed cut — is behind a test an owner-9
    ///   bird already fails.
    /// - **`check_fuel`**, `land_plane` and the whole `0x193` landing
    ///   approach: all three are behind the same `is_animal` slot or behind
    ///   the order's landing flag, which is zero for a patrol.
    /// - **the target-ahead flag**, the `local_20` that would let the bank
    ///   turn at twice the rate: it needs the order to carry a target
    ///   object, and `think_bird` clears the patrol order's on every frame
    ///   the counter is not negative.
    /// - **`pitch_aircraft`'s speed cut**, which is behind `is_animal() == 0`.
    ///
    /// What is left is here, and the one draw in it is the edge coin.
    pub fn do_air_physics(&mut self, u: usize, goal: Pos, frame: i64) {
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        // The waypoint is clamped into the world before anything measures
        // against it — negatives to zero, and the far edge to one under.
        let gx = goal.x.clamp(0, xs - 1);
        let gy = goal.y.clamp(0, ys - 1);
        let at = self.units[u].pos;
        let (dx, dy) = (gx - at.x, gy - at.y);

        // `get_speed(x, y, 1)` — `AnimalData::get_speed`, which for an air
        // animal is `UnitData::speed` and nothing else (`orders.rs`). The
        // `ai_speed` multiply after it is `do_air_physics`'s own, not the
        // step's.
        let mut speed = self.get_speed(u, 1);
        if self.ai_speed > 1 {
            speed *= self.ai_speed;
        }

        let heading = self.units[u].movement.heading;
        let mut des = find_angle(dx, dy);
        // Owing more than forty-five degrees within a tile and a half of the
        // point, the bird keeps its heading and flies past rather than
        // wheeling on the spot. `min_range` is the *type's*, which is zero
        // for a bird, so the radius is the bare `0x300`.
        let min_range = self.profile(crate::combat::Obj::Unit(u)).min_range;
        if owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < min_range * 0xc0 + 0x300 {
            des = heading;
        }
        // Already turning away from an edge: the heading it wants is a
        // quarter turn off the one it has, which is what carries it back
        // over the map.
        let turn = self.flight(u).turn;
        if turn != 0 {
            des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
        }

        self.bank_aircraft(u, des, frame);

        // `Guy::set_angle(angle, 1)`: the figure snaps onto the unit's
        // heading rather than turning toward it, and the step is taken
        // along that.
        let heading = self.units[u].movement.heading;
        self.units[u].movement.facing = heading;
        let nx = at.x + sin_component(heading, speed);
        let ny = at.y - cos_component(heading, speed);

        // `UnitData::invalid_loc` on an **air** type is the world's own
        // rectangle and nothing else: its domain arm (`type +0x218 == 2`)
        // returns valid before every terrain test, and the caller passes
        // zero for all five of the flags that would reach the rest. So the
        // only way a bird's step is refused is by leaving the map.
        if nx >= 0 && ny >= 0 && nx < xs && ny < ys {
            let mut f = self.flight(u);
            f.turn = 0;
            self.set_flight(u, f);
        } else if self.flight(u).turn == 0 {
            self.mark(SITE_AIR_TURN);
            // `((rnd & 0x80000001) != 0) * 2 - 1`, and the draw's range is
            // `0, 0xffff`, so the sign bit can never be set and it is the
            // parity that decides.
            let t = if self.rng.roll() & 1 != 0 { 1 } else { -1 };
            let mut f = self.flight(u);
            f.turn = t;
            self.set_flight(u, f);
        }
        // `WorldData::restrict`, which runs only on the refused step — the
        // accepted one is already inside.
        //
        // **`param_3` is zero** (`005e86d0:237`, `set_new_location(x, y,
        // 0, 1)`), so the figure is *told* where to be and not put there:
        // `Unit::set_new_location@005f8d20:158` writes guy 0's `des` from
        // the unit's new point whatever `param_3` says, and only a
        // non-zero one teleports it (`:161`). The figure catches up in
        // `Guy::move`, one step later in the same frame — which is what
        // makes `des == pos` there mean *the bird did not move this
        // frame*, and that is the whole of its arrival stand
        // (`anim.rs`'s `guys_follow`, `docs/SYNC.md` §3.9).
        let to = Pos::new(nx.clamp(0, xs - 1), ny.clamp(0, ys - 1));
        self.set_new_location(u, to, false);
    }

    /// `Unit::bank_aircraft(des, &speed, 0)` — the bank angle, and the
    /// heading it turns.
    ///
    /// The shape, once a bird's answers are substituted:
    ///
    /// ```text
    /// roll = −guy.roll
    /// a settled bank on an eighth frame does nothing at all       # the early return
    /// want = ±min(55, |des − heading| / rate × 55 × 0.33)         # halved under five
    /// step = want − roll
    ///     |step| ≥ 2 → roll moves toward want by min(|step|, 10), inside ±55
    ///     |step| < 2 and |roll| < 2 → roll = 0
    /// roll > 0 → heading += air_turn_speed(+1)
    /// roll < 0 → heading −= air_turn_speed(−1)
    /// roll = 0 → heading = des, when less than five degrees is owed
    /// guy.roll = −roll
    /// ```
    ///
    /// **The bank is the state and the heading is downstream of it.** That
    /// is why this is worth a module: a bird cannot turn until it has banked
    /// into the turn, and it cannot stop turning until the bank has come
    /// back through zero, so the heading lags the patrol point by a dozen
    /// frames and overshoots it. A bird flown by pointing it at its
    /// waypoint would never reach an edge at all.
    ///
    /// The `speed` argument is not threaded through: both places
    /// `bank_aircraft` cuts it are behind the order's landing flag, which a
    /// patrol never sets.
    fn bank_aircraft(&mut self, u: usize, des: Angle, frame: i64) {
        let stored = self.flight(u).roll;
        let mut roll = stored.neg();
        // The animal arm of the opening test: a bird whose bank has settled
        // and whose counter is not negative skips the whole function every
        // eighth frame. `frame & 7`, the game's own, and not the unit's
        // phase — the altitude re-roll above is the one that adds the index.
        if roll.is_zero() && self.units[u].spell_time >= 0 && frame & 7 == 0 {
            return;
        }
        let heading = self.units[u].movement.heading;
        let d = (des.0 as u32).wrapping_sub(heading.0 as u32);
        let adelta = if d > 0x8000_0000 { !d } else { d };
        // The way round, when the caller has not already decided one.
        let sign_in = if d > 0x8000_0000 { -1i32 } else { 1 };

        let raw = ((self.units[u].movement.turning.type_turn_speed as u32) >> 8)
            .wrapping_mul(self.tuning.unit_turn_speed as u32);
        let divisor = if raw < MIN_TURN { MIN_TURN } else { raw };

        // `(double)(unsigned)adelta / (float)divisor × 55 × 0.33` — three
        // roundings, and the `unsigned` matters: half a turn owed is
        // `0x80000000`, which as a signed int would be negative.
        let mut want = Single::from_u32(adelta)
            .divss(Single::from_i32(divisor as i32))
            .mulss(F55)
            .mulss(F_33);
        if F5.gt(want.abs()) {
            want = want.mulss(F_HALF);
        }
        // The doubling arm needs the order's landing flag; the
        // ground-clearance arm needs an owner under eight. A bird is owner
        // nine, so it is the bare clamp.
        if !F55.gt(want) {
            want = F55;
        }

        let step = Single::from_i32(sign_in).mulss(want).subss(roll);
        let mag = step.abs();
        // `settled` is the original's `LAB_005e9806`: the bank is inside a
        // couple of degrees of where it wants to be and stops moving.
        let mut settled = false;
        if F2.gt(mag) {
            if F2.gt(roll.abs()) {
                roll = Single::ZERO;
                settled = true;
            }
        } else if step.gt(Single::ZERO) {
            if F55.gt(roll) {
                roll = roll.addss(if F10.gt(mag) { mag } else { F10 });
            }
        } else if roll.gt(F55_NEG) {
            roll = roll.subss(if F10.gt(mag) { mag } else { F10 });
        }

        let mut sign_out = 0;
        if !settled {
            if roll.gt(Single::ZERO) {
                sign_out = 1;
            } else if Single::ZERO.gt(roll) {
                sign_out = -1;
            } else {
                settled = true;
            }
        }
        let mut snapped = false;
        if settled {
            sign_out = 0;
            // Level, and within five degrees: take the heading outright.
            if adelta < SNAP_WITHIN {
                self.units[u].movement.heading = des;
                snapped = true;
            }
        }
        if !snapped {
            let rate = air_turn_speed(raw, stored, sign_out) as i32;
            let by = rate.wrapping_mul(sign_out);
            let h = self.units[u].movement.heading;
            self.units[u].movement.heading = Angle(h.0.wrapping_add(by));
        }
        // The two arms between here and the store — a second turn toward
        // `des` and the three-quarter speed cut — both need the order's
        // landing flag, so a patrol reaches neither.
        let mut f = self.flight(u);
        f.roll = roll.neg();
        self.set_flight(u, f);
    }
}

/// Folded the original's way — [`owed`] as a free function on raw words,
/// for the three places the plane's arm compares a heading with a heading.
const fn fold(d: u32) -> u32 {
    if d > 0x8000_0000 { !d } else { d }
}

/// What `Unit::check_fuel@005e9be0` hands `do_air_physics` for a plane on
/// its way home: the point it flies at this frame (`param_2`/`param_3`)
/// and the altitude it aims for there (`param_4`, `local_c` after it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Approach {
    at: Pos,
    z: i32,
}

/// How a plane's frame of flight ended — `do_air_physics`' return, which
/// `do_strafe` branches on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flew {
    /// It stepped: 1, and `do_strafe` carries on.
    On,
    /// `land_plane` ran, or the flight could not be flown here: 0, and
    /// `do_strafe` returns.
    Done,
}

impl Sim {
    /// `ObjectData::is(t, 0)` on a unit's type — the lineage test.
    fn air_line_is(&self, u: usize, t: crate::tech::TypeId) -> bool {
        self.units[u]
            .ty
            .and_then(|i| self.unit_types[i].tree)
            .is_some_and(|ut| {
                ut == t || (ut < self.tech_tree.types.len() && self.tech_tree.is(ut, t, false))
            })
    }

    /// `ObjectData::is(0x130)`, the Bomber line.
    pub(crate) fn is_bomber(&self, u: usize) -> bool {
        self.air_line_is(u, BOMBER)
    }

    /// `unit_flags & 0x20`, "flies like a helicopter" (`ai_load::uflags`).
    pub(crate) fn is_helicopter(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::HELICOPTER != 0
        })
    }

    /// **`Unit::check_fuel@005e9be0`, a plane going home to a building**
    /// (`docs/ORDERS.md` §33.2): the point this frame's flight aims at.
    ///
    /// The landing point is the base's own less `0xc0` in `x`. The
    /// **approach** is north-bound (`0`) unless the base stands in the
    /// map's southern quarter (`y > height · 0x240`), then south-bound.
    /// Within 30° of the approach the plane chases a point half its
    /// distance back along the approach from the landing point; further
    /// round it chases one `0xc00` behind it plus `0x300`; inside `0xc0`,
    /// or more than 120° round, it flies at the point itself. The
    /// altitude is the base's ground plus half the push.
    fn home_approach(&self, u: usize, home: usize) -> Approach {
        let b = &self.buildings[home];
        let (tx, ty) = (b.pos.x - 0xc0, b.pos.y);
        // `+0xc`, the base's `z_internal`: `find_tcoord_z` of its tile.
        let tz = self.world.tile_z(b.pos.tile());
        let approach: u32 = if b.pos.y <= self.world.height() * 0x240 {
            0
        } else {
            0x8000_0000
        };
        let at = self.units[u].pos;
        let (dx, dy) = (tx - at.x, ty - at.y);
        let d = fold(approach.wrapping_sub(find_angle(dx, dy).0 as u32));
        let back = Angle(approach.wrapping_add(0x8000_0000) as i32);
        let (mut px, mut py) = (tx, ty);
        let r = if d <= 0x1555_5555 {
            vector_dist(dx, dy) / 2
        } else {
            px = tx + sin_component(back, 0xc00);
            py = ty - cos_component(back, 0xc00);
            vector_dist(px - at.x, py - at.y) + 0x300
        };
        if r > 0xc0 && d < 0x5555_5555 {
            // `project` from the landing point itself — `*px`/`*py` in
            // memory are still `tx`/`ty` at `0x5ea2cf`, whichever arm ran.
            let r = r.min(0xc00);
            px = tx + sin_component(back, r);
            py = ty - cos_component(back, r);
        }
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let (cx, cy) = (px.clamp(0, xs - 1), py.clamp(0, ys - 1));
        Approach {
            at: Pos::new(px, py),
            z: tz + vector_dist(tx - cx, ty - cy) / 2,
        }
    }

    /// **`Unit::do_air_physics@005e86d0` for a plane on a `StrafeOrder`
    /// home** (`docs/ORDERS.md` §33): the redraw, `check_fuel`'s
    /// approach, the landing test, the bank, the pitch and the step.
    ///
    /// SEAMs, none reached by a flight home: a helicopter (`unit_flags &
    /// 0x20`), which hovers rather than banks and is not flown here; a
    /// flight with a target (`returning 0`), whose target-ahead doubling
    /// and fuel test are not built; a home that is not a live building,
    /// whose nearest-base search is not built; and a carrier as the home.
    pub(crate) fn plane_air_physics(&mut self, u: usize, frame: i64) -> Flew {
        let Some(sf) = self.current_strafe(u) else {
            return Flew::Done;
        };
        let Some(home) = sf.home.filter(|&b| self.buildings[b].alive) else {
            return Flew::Done;
        };
        if !sf.returning || self.is_helicopter(u) {
            return Flew::Done;
        }
        // **The redraw**, `+0x3b`–`+0xc4`: `(o + frame) & 7` with the
        // signed-modulo fix-up, which a non-negative sum never takes. A
        // Bomber holds `0x640` flat.
        if (i64::from(self.units[u].index) + frame) & 7 == 0 {
            let alt = if self.is_bomber(u) {
                CRUISING_ALT
            } else {
                self.mark(SITE_AIR_ALT);
                (self.rng.roll() % 7 + 13) * 100
            };
            self.set_cruising_alt(u, alt);
        }
        // `field_0xc0 = 0`: the path stack is emptied before anything
        // can return, and gets this frame's one point below.
        self.units[u].path.clear();
        let aim = self.home_approach(u, home);
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let (gx, gy) = (aim.at.x.clamp(0, xs - 1), aim.at.y.clamp(0, ys - 1));
        let clamped = (gx, gy) != (aim.at.x, aim.at.y);
        let at = self.units[u].pos;
        let (dx, dy) = (gx - at.x, gy - at.y);
        let mut speed = self.get_speed(u, 1);
        if self.ai_speed > 1 {
            speed *= self.ai_speed;
        }
        // **The landing test** (`+0x1ea`): Manhattan inside a step and a
        // half, the point not clamped. The helicopter's altitude half is
        // not reached.
        if dx.abs() + dy.abs() < speed * 3 / 2 && !clamped {
            self.land_plane(u);
            return Flew::Done;
        }
        self.units[u].path.push(crate::orders::PathData {
            to: Pos::new(gx, gy),
            tolerance: 0,
            flags: 0,
        });
        let heading = self.units[u].movement.heading;
        let mut des = find_angle(dx, dy);
        // Owing more than 45°, a returning plane keeps its heading only
        // within `0x300` of the point: its radius is `0`, not the type's
        // `min_range`.
        if owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < 0x300 {
            des = heading;
        }
        let turn = self.current_strafe(u).map_or(0, |s| s.sharp_turn);
        if turn != 0 {
            des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
        }
        self.bank_plane(u, des, &mut speed);
        self.pitch_plane(u, dx, dy, aim.z, &mut speed);
        let heading = self.units[u].movement.heading;
        self.units[u].movement.facing = heading;
        let nx = at.x + sin_component(heading, speed);
        let ny = at.y - cos_component(heading, speed);
        if nx >= 0 && ny >= 0 && nx < xs && ny < ys {
            self.set_sharp_turn(u, 0);
        } else if turn == 0 {
            self.mark(SITE_AIR_TURN);
            let t = if self.rng.roll() & 1 != 0 { 1 } else { -1 };
            self.set_sharp_turn(u, t);
        }
        let to = Pos::new(nx.clamp(0, xs - 1), ny.clamp(0, ys - 1));
        self.set_new_location(u, to, false);
        if self.units[u].combat.recharging == 0 {
            self.set_anim(u, crate::anim::WALK, false, true);
        }
        Flew::On
    }

    /// **`Unit::land_plane@005e9950`, into a building** (`docs/ORDERS.md`
    /// §33.5). The strafe home has no target, so it goes — `clear_orders`
    /// for a `mandatory` one, `kill_current_order` otherwise — the path
    /// and the orders are closed for a base that is not a carrier, and the
    /// `SpecialAnimOrder` it adds is run at once by the `work` call at its
    /// tail: `do_spec_anim`'s landing arm is `go_inside` and a kill, all
    /// inside the frame, which is why no dump ever prints one. The plane
    /// keeps its point, its altitude and its bank; the pitch and the bank
    /// are the figure's and nothing on this arm clears them.
    ///
    /// SEAM: a strafe with a live target keeps its order here (the
    /// original re-reads it after the landing), and a carrier home.
    fn land_plane(&mut self, u: usize) {
        let Some(sf) = self.current_strafe(u) else {
            return;
        };
        let Some(home) = sf.home else {
            return;
        };
        if sf.target.is_none() {
            if sf.mandatory {
                self.clear_orders(u);
            } else {
                self.kill_current_order(u);
            }
        }
        self.units[u].path.clear();
        self.close_orders(u);
        self.clear_partial_path(u);
        self.update_action(u);
        self.go_inside(u, home);
    }

    fn current_strafe(&self, u: usize) -> Option<crate::orders::StrafeOrder> {
        match self.current_order(u).map(|o| o.body) {
            Some(crate::orders::Body::Strafe(sf)) => Some(sf),
            _ => None,
        }
    }

    fn with_strafe(&mut self, u: usize, f: impl FnOnce(&mut crate::orders::StrafeOrder)) {
        if let Some(crate::orders::Order {
            body: crate::orders::Body::Strafe(sf),
            ..
        }) = self.units[u].orders.front_mut()
        {
            f(sf);
        }
    }

    fn set_cruising_alt(&mut self, u: usize, alt: i32) {
        self.with_strafe(u, |sf| sf.cruising_alt = alt);
    }

    fn set_sharp_turn(&mut self, u: usize, t: i32) {
        self.with_strafe(u, |sf| sf.sharp_turn = t);
    }

    /// **`Unit::bank_aircraft(des, &speed, 0)` for a plane going home** —
    /// [`Sim::bank_aircraft`]'s shape, with the three arms the order's
    /// `returning` flag opens (`docs/ORDERS.md` §33.3): the bank it wants
    /// is **doubled** before the clamp, a **second turn** toward `des`
    /// follows the first (and takes it outright inside 3°), and a banked
    /// plane flies at **three quarters** of its speed.
    fn bank_plane(&mut self, u: usize, des: Angle, speed: &mut i32) {
        let stored = self.units[u].airframe.bank;
        let mut roll = stored.neg();
        let heading = self.units[u].movement.heading;
        let d = (des.0 as u32).wrapping_sub(heading.0 as u32);
        let adelta = fold(d);
        let sign_in = if d > 0x8000_0000 { -1i32 } else { 1 };
        let raw = ((self.units[u].movement.turning.type_turn_speed as u32) >> 8)
            .wrapping_mul(self.tuning.unit_turn_speed as u32);
        let divisor = if raw < MIN_TURN { MIN_TURN } else { raw };
        let mut want = Single::from_u32(adelta)
            .divss(Single::from_i32(divisor as i32))
            .mulss(F55)
            .mulss(F_33);
        if F5.gt(want.abs()) {
            want = want.mulss(F_HALF);
        }
        // The returning arm: doubled, then the upper clamp.
        want = want.addss(want);
        if !F55.gt(want) {
            want = F55;
        }
        let step = Single::from_i32(sign_in).mulss(want).subss(roll);
        let mag = step.abs();
        let mut settled = false;
        if F2.gt(mag) {
            if F2.gt(roll.abs()) {
                roll = Single::ZERO;
                settled = true;
            }
        } else if step.gt(Single::ZERO) {
            if F55.gt(roll) {
                roll = roll.addss(if F10.gt(mag) { mag } else { F10 });
            }
        } else if roll.gt(F55_NEG) {
            roll = roll.subss(if F10.gt(mag) { mag } else { F10 });
        }
        let mut sign_out = 0;
        if !settled {
            if roll.gt(Single::ZERO) {
                sign_out = 1;
            } else if Single::ZERO.gt(roll) {
                sign_out = -1;
            } else {
                settled = true;
            }
        }
        let mut snapped = false;
        if settled {
            sign_out = 0;
            if adelta < SNAP_WITHIN {
                self.units[u].movement.heading = des;
                snapped = true;
            }
        }
        if !snapped {
            let rate = air_turn_speed(raw, stored, sign_out) as i32;
            let h = self.units[u].movement.heading;
            self.units[u].movement.heading = Angle(h.0.wrapping_add(rate.wrapping_mul(sign_out)));
        }
        // `LAB_005e97b9`: the second turn, which only a returning order
        // takes.
        let h = self.units[u].movement.heading;
        let d2 = fold((h.0 as u32).wrapping_sub(des.0 as u32));
        if d2 < RETURN_SNAP {
            self.units[u].movement.heading = des;
        } else {
            let rate = air_turn_speed(raw, stored, sign_out) as i32;
            let h2 = Angle(h.0.wrapping_add(rate.wrapping_mul(sign_out)));
            if fold((h2.0 as u32).wrapping_sub(des.0 as u32)) < d2 {
                self.units[u].movement.heading = h2;
            }
        }
        // `(s·3 + (s·3 >> 31 & 3)) >> 2`: three quarters, toward zero.
        if !roll.eq_value(Single::ZERO) {
            *speed = *speed * 3 / 4;
        }
        let af = &mut self.units[u].airframe;
        af.last_bank = af.bank;
        af.bank = roll.neg();
    }

    /// **`Unit::pitch_aircraft(dx, dy, z, &speed)@005e8de0` for a plane
    /// going home** (`docs/ORDERS.md` §33.4): the altitude it wants, the
    /// pitch that climbs or dives toward it two a frame, the step of
    /// altitude the pitch makes, and the two speed cuts.
    fn pitch_plane(&mut self, u: usize, dx: i32, dy: i32, aim_z: i32, speed: &mut i32) {
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let at = self.units[u].pos;
        let heading = self.units[u].movement.heading;
        // The ground `0x240` ahead, the point clamped into the world.
        let ax = (at.x + sin_component(heading, 0x240)).clamp(0, xs - 1);
        let ay = (at.y - cos_component(heading, 0x240)).clamp(0, ys - 1);
        let ahead = self.world.tile_z(Pos::new(ax, ay).tile());
        let cruise = self
            .current_strafe(u)
            .map_or(CRUISING_ALT, |s| s.cruising_alt);
        // The returning arm (`air +0x18`). `local_2c`, the home carrier's
        // own speed test, is never set for a building.
        let dist = vector_dist(dx, dy);
        let want = (ahead + 100).max(aim_z + (dist / *speed) * 25);
        if dist < 0x600 {
            *speed /= 2;
        }
        let want = want.min(cruise + ahead);
        let ground = self.world.tile_z(at.tile());
        let target = want.max(ground + 50);
        let mut pitch = self.units[u].airframe.pitch;
        let extra = if target < ground + 500 {
            F20_NEG
        } else {
            Single::ZERO
        };
        let z = self.units[u].airframe.z;
        let n = (dist / *speed).max(1);
        let rate = ((target - z) / 25 * 20) / n;
        let owe = Single::from_i32(rate).subss(extra.addss(pitch));
        let mut dir = 0;
        if owe.gt(F1) {
            dir = 1;
            if F40.gt(pitch) {
                pitch = pitch.addss(F2);
            }
        } else if F1_NEG.gt(owe) {
            dir = -1;
            if pitch.gt(F40_NEG) {
                pitch = pitch.subss(F2);
            }
        }
        if F1.gt(extra.addss(pitch).abs()) && dir == 0 {
            pitch = extra.neg();
        }
        let sum = extra.addss(pitch);
        if !sum.eq_value(Single::ZERO) {
            let step = sum.mulss(F25).divss(F20).to_i32();
            let af = &mut self.units[u].airframe;
            af.last_z = af.z;
            af.z += step;
        }
        let mag = pitch.abs();
        if mag.gt(F20) {
            let half = Single::from_i32(*speed / 2);
            *speed = F40.subss(mag).mulss(half).divss(F20).addss(half).to_i32();
        }
        let af = &mut self.units[u].airframe;
        af.last_pitch = af.pitch;
        af.pitch = pitch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_owed_angle_is_a_bitwise_not_and_not_a_negation() {
        // Exactly half a turn: the original's `~d` gives `0x7fffffff` for
        // `0x80000000`... except that the compare is `<`, so half a turn is
        // *not* folded and stays `0x80000000`.
        assert_eq!(owed(Angle(0), Angle(i32::MIN)), 0x8000_0000);
        assert_eq!(owed(Angle(0), Angle(0x4000_0000)), 0x4000_0000);
        // One past half a turn folds, and to one less than half rather than
        // to the exact mirror — `~0x80000001` is `0x7ffffffe`.
        assert_eq!(owed(Angle(0), Angle(i32::MIN + 1)), 0x7fff_fffe);
    }

    #[test]
    fn the_turn_rate_is_the_banks_magnitude_over_fifty_five() {
        // The bird's own: `TURN_SPEED 5` through `degrees_to_angle`, with
        // its low byte cleared.
        let raw = ((crate::movement::degrees_to_angle(5).0 as u32) >> 8) * 256;
        assert_eq!(raw, 0x038e_3800);
        // A level bird turns at the floor.
        assert_eq!(air_turn_speed(raw, Single::ZERO, 1), MIN_TURN);
        // Banked hard the wrong way for the sign asked, it also turns at the
        // floor — the frame a bank reverses is a frame at half a degree.
        assert_eq!(air_turn_speed(raw, F55, 1), MIN_TURN);
        // Banked fully, it turns at the type's whole rate.
        assert_eq!(air_turn_speed(raw, F55.neg(), 1), (raw / 0x37) * 55);
        // And a shallow bank is still floored: five degrees of bank buys
        // less than half a degree of turn.
        assert!(air_turn_speed(raw, F5.neg(), 1) == MIN_TURN);
    }

    /// A bird on the shipped `WILDBIRD` numbers, put where the test wants
    /// it and pointed where the test wants it pointed.
    fn bird(at: crate::world::Pos, heading: i32) -> (crate::Sim, usize) {
        let mut s = crate::Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(60, 60),
            2,
        );
        s.rng = crate::combat::Rng::new(7);
        let ty = s.add_unit_type(crate::UnitType {
            hits: 1,
            moves: 35,
            turn_speed: crate::movement::degrees_to_angle(5).0,
            kind: crate::attrition::UnitKind {
                domain: crate::attrition::Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            ..crate::UnitType::default()
        });
        let mut u = crate::Unit::new(crate::gaia::BIRD_OWNER, 0, at, 1);
        u.ty = Some(ty);
        u.kind = s.unit_types[ty].kind;
        u.type_index = crate::anim::BIRD_TYPE;
        u.movement.speed = 35;
        u.movement.turning = crate::turning_of(&s.unit_types[ty]);
        u.movement.heading = Angle(heading);
        let b = s.add_unit(u);
        // The figure, because `Guy::move` runs for a bird like anybody
        // else and the arrival stand is its own draw.
        s.init_guys(b, Some(ty));
        s.gaia.bird_goals.push((b, at));
        (s, b)
    }

    /// **The figure lags its unit by a step, and that is what the arrival
    /// stand reads.** `Unit::do_air_physics` ends in `set_new_location(x,
    /// y, 0, 1)` — `param_3` zero (`005e86d0:237`) — so
    /// `Unit::set_new_location@005f8d20:158` writes guy 0's `des` to the
    /// unit's new point and `:161`'s teleport is skipped: the figure is
    /// still on last frame's point when `Guy::process` runs, and takes the
    /// **moving** arm. A bird pinned on the boundary, whose step
    /// `WorldData::restrict` hands back unchanged, is at its `des` instead
    /// and takes the arrival one.
    ///
    /// Made to fail on purpose both ways: with `move_guys` back at `true`
    /// the figure never lags and the flying frame pays a stand it should
    /// not; with `guys_follow`'s old air-gaia return in place the pinned
    /// frame pays none.
    #[test]
    fn a_bird_s_figure_lags_its_unit_and_stands_only_when_the_step_is_refused() {
        // Pointed due west, sixteen units from the edge, with the patrol
        // point behind it so the heading stands.
        let at = crate::world::Pos::new(16, 20_000);
        let (mut s, b) = bird(at, Angle::WEST.0);
        let goal = crate::world::Pos::new(20_000, 20_000);
        s.gaia.bird_goals[0].1 = goal;
        s.art.lengths.insert((-1, crate::anim::WALK), 31);
        s.art.lengths.insert((-1, crate::anim::JOG), 23);
        s.art.lengths.insert((-1, crate::anim::DEFAULT), 3);
        s.units[b].guys[0].anim = crate::anim::WALK;
        s.units[b].guys[0].end_time = 31;
        s.units[b].guys[0].stopped = true;
        s.trace_phases = true;

        // Frame one: the step is refused and `restrict` puts the bird on
        // the boundary, which is a move — the figure lags and the moving
        // arm runs, no stand.
        s.do_air_physics(b, goal, 1);
        assert_eq!(s.units[b].pos.x, 0, "restrict puts it on the boundary");
        assert_ne!(
            s.units[b].movement.body.pos, s.units[b].pos,
            "the figure has not been teleported with the unit"
        );
        let before = s.rng.seed;
        s.process_movement(b);
        assert_eq!(s.rng.seed, before, "a bird that moved pays no stand");
        assert!(!s.units[b].guys[0].stopped, "and is marked unstopped");
        assert_eq!(s.units[b].movement.body.pos, s.units[b].pos, "it caught up");

        // Frame two: still pointed off the map and already on the
        // boundary, so `restrict` hands back the point it stands on. The
        // figure is on its `des`, the first visit only marks it stopped.
        s.do_air_physics(b, goal, 2);
        assert_eq!(s.units[b].pos.x, 0, "still pinned");
        let before = s.rng.seed;
        s.process_movement(b);
        assert_eq!(s.rng.seed, before, "the first standing frame only stops it");
        assert!(s.units[b].guys[0].stopped);

        // Frame three: stopped, on `CHAR_WALK`, and at its destination —
        // the stand, one draw, and the slot goes to `CHAR_DEFAULT`, which
        // is what makes the *next* frame's `set_anim(CHAR_WALK)` throw the
        // wing-beat coin instead of taking gaia's early return.
        s.do_air_physics(b, goal, 3);
        let before = s.rng.seed;
        s.phase_marks.clear();
        s.process_movement(b);
        assert_ne!(s.rng.seed, before, "the arrival stand's draw");
        assert_eq!(s.units[b].guys[0].anim, crate::anim::DEFAULT);
        assert!(
            s.phase_marks
                .iter()
                .any(|(l, _)| l == crate::anim::SITE_ARRIVE),
            "and it is marked as the arrival stand: {:?}",
            s.phase_marks
        );
    }

    /// **The edge coin, end to end.** A bird pointed at the western edge
    /// spends `Unit::do_air_physics+0x639` exactly once: on the frame its
    /// step first leaves the world. `WorldData::restrict` puts it on the
    /// boundary, `AirOrder::sharp_turn` takes the coin's `±1`, and while
    /// that field stands no further coin is thrown however long the bird
    /// stays out — which is what makes run53's coins cluster a hundred
    /// frames apart rather than fire every frame.
    #[test]
    fn a_bird_pushed_off_the_map_spends_one_coin_and_then_circles() {
        // Sixteen position units from the western edge, pointed due west,
        // with the patrol point behind it so the heading stands.
        let at = crate::world::Pos::new(16, 20_000);
        let (mut s, b) = bird(at, Angle::WEST.0);
        let goal = crate::world::Pos::new(20_000, 20_000);
        s.gaia.bird_goals[0].1 = goal;
        s.trace_phases = true;

        let before = s.rng.seed;
        s.do_air_physics(b, goal, 1);
        assert_eq!(s.units[b].pos.x, 0, "restrict puts it on the boundary");
        assert_ne!(s.rng.seed, before, "the coin is a draw");
        let turn = s.flight(b).turn;
        assert!(turn == 1 || turn == -1, "sharp_turn takes the coin's sign");

        // Held out for another twenty frames: the field stands, and the
        // stream does not move again.
        let held = s.rng.seed;
        for f in 2..22 {
            s.do_air_physics(b, goal, f);
        }
        assert_eq!(s.rng.seed, held, "no second coin while sharp_turn stands");
        assert_eq!(s.flight(b).turn, turn, "and the field is not re-rolled");
    }

    /// **The bank is what turns the bird, and it ramps.** Ten a frame to
    /// fifty-five, and the heading moves by `air_turn_speed` of the bank
    /// the *previous* frame left — so the first frame of a turn is at the
    /// half-degree floor however much is owed.
    #[test]
    fn the_bank_ramps_by_ten_and_the_heading_lags_it_by_a_frame() {
        let at = crate::world::Pos::new(20_000, 20_000);
        let (mut s, b) = bird(at, Angle::NORTH.0);
        // Half a turn owed, and far enough that the keep-heading arm does
        // not fire.
        let goal = crate::world::Pos::new(20_000, 30_000);
        s.gaia.bird_goals[0].1 = goal;

        let raw = ((crate::movement::degrees_to_angle(5).0 as u32) >> 8) * 256;
        let mut heading = s.units[b].movement.heading.0;
        let want: [u32; 6] = [
            0xc120_0000,
            0xc1a0_0000,
            0xc1f0_0000,
            0xc220_0000,
            0xc248_0000,
            0xc25c_0000,
        ];
        for (i, bits) in want.iter().enumerate() {
            // The rate this frame is the bank the last one left.
            let bank = s.flight(b).roll;
            s.do_air_physics(b, goal, i as i64 + 1);
            assert_eq!(
                s.flight(b).roll.bits(),
                *bits,
                "frame {i}: the bank steps by ten toward fifty-five"
            );
            let step = air_turn_speed(raw, bank, 1) as i32;
            assert_eq!(
                s.units[b].movement.heading.0,
                heading.wrapping_add(step),
                "frame {i}: the heading moves by the *previous* bank's rate"
            );
            heading = s.units[b].movement.heading.0;
        }
        // The first of those steps is the floor, and the last is the
        // type's whole rate.
        assert_eq!(air_turn_speed(raw, Single::ZERO, 1), MIN_TURN);
        assert_eq!(air_turn_speed(raw, F55.neg(), 1), (raw / 0x37) * 55);
    }

    /// **The keep-heading arm**: owing more than forty-five degrees inside
    /// a tile and a half of the patrol point, a bird flies straight past it
    /// rather than wheeling on the spot. It is what makes the orbit an
    /// orbit — without it a bird would circle its point at the turn
    /// radius and never reach an edge at all.
    #[test]
    fn a_bird_over_its_patrol_point_keeps_the_heading_it_has() {
        let at = crate::world::Pos::new(20_000, 20_000);
        let (mut s, b) = bird(at, Angle::NORTH.0);
        // The point a hundred units behind it: half a turn owed, well
        // inside `0x300`.
        let goal = crate::world::Pos::new(20_000, 20_100);
        s.gaia.bird_goals[0].1 = goal;
        for f in 1..10 {
            s.do_air_physics(b, goal, f);
        }
        assert_eq!(
            s.units[b].movement.heading,
            Angle::NORTH,
            "inside 0x300 with more than 45 degrees owed, the heading stands"
        );
        assert_eq!(s.units[b].pos.x, 20_000, "and the bird flies straight");
        assert!(s.units[b].pos.y < 19_700, "north, at its own speed");
    }
}
