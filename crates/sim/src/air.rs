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
use crate::orders::{Body, CRUISING_ALT};
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

/// `Unit::do_spec_anim@005e5880`'s two draws for a helicopter's exit at an
/// Airbase (`5e59ea`, `5e5a0a`): the `x` offset, then the `y`.
pub const SITE_HELI_EXIT_X: &str = "Unit::do_spec_anim+0x16f";
pub const SITE_HELI_EXIT_Y: &str = "Unit::do_spec_anim+0x18f";

/// **A bomb's release** — `Guy::set_anim+0xf2f < Unit::set_anim+0x56 <
/// Unit::do_strafe+0x9d0`, the `CHAR_ATTACK2` a Bomber plays over its
/// target (`docs/ORDERS.md` §34.3). Chapter seventeen's word 805.
pub const SITE_STRAFE_BOMB: &str = "Unit::do_strafe+0x9d0";

/// `ObjectData::is(0x130)` — the Bomber line, which holds its
/// `cruising_alt` at 0x640 and throws no redraw.
pub(crate) const BOMBER: crate::tech::TypeId = 0x130;

/// `ObjectData::is(0x127)` — the line `do_strafe` lets release a quarter
/// turn off the nose where every other takes 15° (`0x5eb448`).
const WIDE_RELEASE: crate::tech::TypeId = 0x127;

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
pub(crate) const HALF_QUARTER: u32 = 0x2000_0000;

/// `FRAMES_BETWEEN_LAUNCHES`, the PE's `.data` at `0xc06248`: the frames a
/// hangar waits between two launches, and where its `launch_frames`
/// saturates (run223's Airbase reads 15 from its landing on).
pub const FRAMES_BETWEEN_LAUNCHES: i32 = 15;

/// `<MISSILEOFFSET x="-109" y="1" z="388"/>` under `EFFECTS` in
/// `effects_graphics.xml`, which `GraphicEvents::init@008e5390` reads into
/// `graphic_events +0xc8/+0xcc/+0xd0` before the first frame and
/// `Ammo::init` adds to a missile's point for its round's start (item
/// 1050). run371's `sz` 466 is the silo tile's 78 plus the third term.
/// `rondata`'s survey re-reads it off the install.
pub const MISSILE_OFFSET: (i32, i32, i32) = (-109, 1, 388);

/// A missile round's frames: `Spline::calc_nuke_spline@00913ad0` sets the
/// spline's `depth` to 120 (`+0x60 = 0x780003`, degree 3) on both its
/// arms, `generate_bspline` lays `depth + 1` points, and `Ammo::init`
/// takes `total_time` as the points less one (item 1050; run371's round
/// prints `length 121`, `depth 120`, `total_time 120`).
pub const MISSILE_FLIGHT: i32 = 120;

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
pub(crate) const fn owed(from: Angle, to: Angle) -> u32 {
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

/// The `AirOrder` base of a plane's front order — what `do_air_physics`,
/// `bank_aircraft` and `pitch_aircraft` read through `get_air_order`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AirBase {
    pub(crate) home: Option<usize>,
    pub(crate) cruising_alt: i32,
    pub(crate) sharp_turn: i32,
    pub(crate) returning: bool,
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
    pub(crate) fn air_line_is(&self, u: usize, t: crate::tech::TypeId) -> bool {
        self.units[u]
            .ty
            .and_then(|i| self.unit_types[i].tree)
            .is_some_and(|ut| {
                ut == t || (ut < self.tech_tree.types.len() && self.tech_tree.is(ut, t, false))
            })
    }

    /// A unit whose type strafes: `unit_flags & 0x400000`, flag `w`
    /// (the Fighter line; `docs/ORDERS.md` §39). A building is not.
    pub(crate) fn strafes(&self, o: crate::combat::Obj) -> bool {
        let crate::combat::Obj::Unit(u) = o else {
            return false;
        };
        self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::STRAFES != 0
        })
    }

    /// `ObjectData::is(0x130)`, the Bomber line.
    pub(crate) fn is_bomber(&self, u: usize) -> bool {
        self.air_line_is(u, BOMBER)
    }

    /// `ObjectData::is(0x127)`, `do_strafe`'s wide release.
    pub(crate) fn is_fighter_line(&self, u: usize) -> bool {
        self.air_line_is(u, WIDE_RELEASE)
    }

    /// `unit_flags & 0x20`, "flies like a helicopter" (`ai_load::uflags`).
    pub(crate) fn is_helicopter(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            self.unit_types[t].cols.unit_flags & crate::ai_load::uflags::HELICOPTER != 0
        })
    }

    /// **`Unit::work@0060d180`'s tail: two Helicopters set apart** (the
    /// listing `60dadd`..`60dc9f`, after `do_job`; item 1048,
    /// `docs/PRODUCTION.md` "The Helicopter's separation").
    ///
    /// A unit that flies like a helicopter (`+0x2b4 & 0x20`), alive
    /// (`+0x8 & 1`) and not inside (`inside_up < 0`), whose front order
    /// is none or not an air order (`UnitOrder` vslot `+0x30`, `is_air`:
    /// `mov eax, 1` on the air orders, 0 on the base) asks
    /// `ObjectsData::find_unit@0065ca80(x, y, SEARCH_ALL, −1, 0x180, 1,
    /// FILTER_TYPE, its type, 0, FILTER_NOT_ME, o, who)` for the nearest
    /// unit of its own type within `0x180`. Found, the pair are put 48
    /// apart along the bearing from it to this one: `project(find_angle(me
    /// − it), 0x30)`, this one by half the vector (`sar`, toward zero)
    /// and the other by the whole of it, each through `WorldData::restrict`
    /// and `set_new_location(·, ·, 1, 1)` — a teleport that snaps the
    /// figure's facing onto the unit's and puts guy 0 on it. run371's
    /// `0/11` and `0/12`, 2674..2676.
    pub(crate) fn helicopter_spread(&mut self, u: usize) {
        if !self.is_helicopter(u) {
            return;
        }
        let unit = &self.units[u];
        if !unit.alive() || !unit.on_map || unit.inside.is_some() {
            return;
        }
        if self.current_order(u).is_some_and(|o| {
            matches!(
                o.body,
                Body::Strafe(_) | Body::AirPatrol(_) | Body::AirAttackGround(_)
            )
        }) {
            return;
        }
        let Some(v) = self.nearest_of_own_type(u, 0x180) else {
            return;
        };
        let (me, it) = (self.units[u].pos, self.units[v].pos);
        let ang = find_angle(me.x - it.x, me.y - it.y);
        let (px, py) = (sin_component(ang, 0x30), -cos_component(ang, 0x30));
        let to = self.restrict_pos(Pos::new(me.x + px / 2, me.y + py / 2));
        self.spread_to(u, to);
        let to = self.restrict_pos(Pos::new(it.x - px, it.y - py));
        self.spread_to(v, to);
    }

    /// **`Guy::set_new_location@005d86f0`'s helicopter arm** (the listing
    /// `5d880b`..`5d884a`, item 1048): a figure of an air type that flies
    /// like a helicopter climbs toward 1000 over the ground at its new
    /// point, thirty a move at most —
    /// `z += clamp(find_data_z(x, y, 0) − z + 1000, −30, 30)`. Every move
    /// of the figure takes it: a teleport's
    /// and each step of `Guy::move` (`:190`). run371's `0/12` climbs 30 a
    /// frame to 727 over a lake bed at −273.
    pub(crate) fn helicopter_climb(&mut self, u: usize, at: Pos) {
        if !self.is_helicopter(u) || self.unit_domain_of(u) != crate::attrition::Domain::Air {
            return;
        }
        let ground = self.ground_z(at);
        let af = &mut self.units[u].airframe;
        af.z += (ground - af.z + 1000).clamp(-30, 30);
    }

    /// `set_new_location(·, ·, 1, 1)`: guy 0 onto the unit's own facing,
    /// then the teleport ([`Sim::set_new_location`]'s `move_guys`).
    fn spread_to(&mut self, u: usize, to: Pos) {
        let from = self.units[u].pos;
        let heading = self.units[u].movement.heading;
        self.units[u].movement.set_facing(heading);
        self.set_new_location(u, to, true);
        self.moved_to(u, from, false);
    }

    /// `ObjectsData::find_unit` as the separation calls it: every
    /// player's units (leaders below 8), alive and on the map, of the
    /// unit's own type by `ObjectData::is(type, 0)`, not the unit itself,
    /// within `range` by `vector_dist`, the last of a tie. The list walk
    /// (while `total_units` is under `circle_radius[ring]`) is each
    /// leader's units in `o` order; the circle walk the ring's cells in
    /// circle order, each cell's chain from its head.
    fn nearest_of_own_type(&self, u: usize, range: i32) -> Option<usize> {
        let at = self.units[u].pos;
        let mine = self.units[u].ty.and_then(|t| self.unit_types[t].tree)?;
        let fits = |c: usize| -> Option<i32> {
            let x = &self.units[c];
            if c == u || !x.alive() || !x.on_map || x.owner >= crate::world::PLAYER_SLOTS {
                return None;
            }
            let same =
                x.ty.and_then(|t| self.unit_types[t].tree)
                    .is_some_and(|t| self.tech_tree.is(t, mine, false));
            if !same {
                return None;
            }
            let d = vector_dist((x.pos.x - at.x).abs(), (x.pos.y - at.y).abs());
            (d <= range).then_some(d)
        };
        let mut best: Option<(i32, usize)> = None;
        let mut take = |c: usize| {
            if let Some(d) = fits(c)
                && best.is_none_or(|(bd, _)| d <= bd)
            {
                best = Some((d, c));
            }
        };
        let circle = crate::ai_place::circle();
        let ring = ((range + 0x2ff) / 0x300) as usize;
        let live = self.total_units();
        if live < circle.radius[ring] {
            let mut all: Vec<usize> = (0..self.units.len()).collect();
            all.sort_by_key(|&c| (self.units[c].owner, self.units[c].index));
            for c in all {
                take(c);
            }
        } else {
            let c0 = at.cell();
            let width = self.world.width() as usize;
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                if !self.world.contains(c) {
                    continue;
                }
                let mut next = self.chain_heads[(c.y as usize) * width + (c.x as usize)];
                while let Some(x) = next {
                    next = self.units[x].down;
                    take(x);
                }
            }
        }
        best.map(|(_, c)| c)
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
        // `+0xc`, the base's `z_internal`: `find_tcoord_z` of its tile as
        // `SubObject::init@00662300` and `set_new_location@00662680` ask
        // it, with the fourth argument 1 (`662369`, `66269b`), which
        // answers 0 for a height under 0 — item 1009's `0/2008`, whose
        // tile is −42 here and whose `z_internal` run362 prints 0.
        let tz = self.world.tile_z(b.pos.tile()).max(0);
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

    /// **`Unit::do_air_physics@005e86d0` for a plane** (`docs/ORDERS.md`
    /// §33, §34): the redraw, `check_fuel`'s approach home or the point
    /// the order hands it (`goal`, `param_2/param_3`), the landing test,
    /// the bank, the pitch and the step. The order is a `StrafeOrder` or
    /// an `AirPatrolOrder`; both carry the `AirOrder` base it reads.
    ///
    /// **A flight with a point** (`returning 0`, §34.6) differs from the
    /// flight home in five places: `check_fuel` returns at once and aims
    /// nothing (the altitude handed on is 0), there is no landing test,
    /// the keep-heading radius is the type's `min_range · 0xc0`, and the
    /// bank and the pitch take their non-returning arms.
    ///
    /// SEAMs: a helicopter (`unit_flags & 0x20`), which hovers rather
    /// than banks and is not flown here; a flying target's lead
    /// (`local_20`, the doubled bank); the tank (`check_fuel`'s empty-tank
    /// arm, §32 piece 4); a home that is not a live building, whose
    /// nearest-base search is not built; and a carrier as the home.
    pub(crate) fn plane_air_physics(&mut self, u: usize, goal: Option<Pos>, frame: i64) -> Flew {
        let Some(mut air) = self.current_air(u) else {
            return Flew::Done;
        };
        if self.is_helicopter(u) {
            return Flew::Done;
        }
        // **`check_fuel`'s first arm, the empty tank** (`docs/ORDERS.md`
        // §38.1): a type with a tank (`MANA` ≠ 0), not a missile, whose
        // `mana_left` is 0 turns for home — `returning` set on the order
        // it flies. The redraw below does not read it, so it is taken
        // here, ahead of the aim. run223's pair on 1212 and 1214, run265's
        // Fighter on 1178.
        if !air.returning
            && self.unit_mana(u) != 0
            && self.mana_left(u) == 0
            && !self
                .profile(crate::combat::Obj::Unit(u))
                .has(crate::combat::mask::MISSILE)
        {
            self.set_returning(u, true);
            air.returning = true;
        }
        let aim = if air.returning {
            let Some(home) = air.home.filter(|&b| self.buildings[b].alive) else {
                return Flew::Done;
            };
            Some((home, self.home_approach(u, home)))
        } else {
            None
        };
        let (point, aim_z) = match (aim, goal) {
            (Some((_, a)), _) => (a.at, a.z),
            (None, Some(g)) => (g, 0),
            (None, None) => return Flew::Done,
        };
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
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let (gx, gy) = (point.x.clamp(0, xs - 1), point.y.clamp(0, ys - 1));
        let clamped = (gx, gy) != (point.x, point.y);
        let at = self.units[u].pos;
        let (dx, dy) = (gx - at.x, gy - at.y);
        let mut speed = self.get_speed(u, 1);
        if self.ai_speed > 1 {
            speed *= self.ai_speed;
        }
        // **The landing test** (`+0x1ea`), a flight home's alone:
        // Manhattan inside a step and a half, the point not clamped. The
        // helicopter's altitude half is not reached.
        if air.returning && dx.abs() + dy.abs() < speed * 3 / 2 && !clamped {
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
        // Owing more than 45°, a plane keeps its heading within the
        // radius of the point: `0x300` for a flight home, and the type's
        // `min_range · 0xc0` beyond it for a flight with a point.
        let radius = if air.returning {
            0
        } else {
            self.profile(crate::combat::Obj::Unit(u)).min_range * 0xc0
        };
        if owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < radius + 0x300 {
            des = heading;
        }
        let turn = air.sharp_turn;
        if turn != 0 {
            des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
        }
        self.bank_plane(u, des, &mut speed, air.returning);
        self.pitch_plane(u, dx, dy, aim_z, &mut speed, air.returning);
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
        // `Unit::set_new_location@005f8d20`'s half-cell test and the
        // reveal behind it, `update_seen(param_3 == 0)`: `do_air_physics`
        // passes `param_3 = 0`, so the **ring** pass. A plane lights the
        // fog as it flies — which is how run223's pair first see the
        // Barracks their patrol then takes (`docs/ORDERS.md` §34.5).
        self.moved_to(u, at, true);
        if self.units[u].combat.recharging == 0 {
            self.set_anim(u, crate::anim::WALK, false, true);
        }
        Flew::On
    }

    /// **`Unit::do_air_patrol@005ea620` for a plane** (`docs/ORDERS.md`
    /// §34.5): the flight at the waypoint, then — for a patrol not going
    /// home — the arrival, the sixteen-frame search and the thirty-two
    /// frame look at the point's own tile.
    ///
    /// - **The arrival**: inside `0x240` of the (clamped) point after the
    ///   step, the waypoint steps on, or at the last one a patrol with an
    ///   order behind it is killed. Alone, it flies on.
    /// - **The search**, on `(o + frame) & 15 == 0`: a Bomber asks
    ///   [`Sim::find_new_bomber_target`] round the patrol's last point,
    ///   and a valid answer — at the last waypoint, or a flying one — is
    ///   pushed `QUEUE_FIRST` as a strafe with `mandatory 0` and no
    ///   action bit (run223's 777 and 778).
    /// - **The look**, on `(o + frame) & 31 == 0`: an enemy building
    ///   standing on the waypoint's own tile that this player has ever
    ///   seen is pushed the same way with `mandatory 1`.
    ///
    /// SEAM: the `semaphore & 2` fallback between the two searches (the
    /// unit-balance mode's), a
    /// `FIGHTERBOMBER`'s carrier-relative point, and a patrol going home.
    pub(crate) fn do_air_patrol(&mut self, u: usize, frame: i64) {
        let Some(crate::orders::Body::AirPatrol(mut p)) = self.current_order(u).map(|o| o.body)
        else {
            return;
        };
        // `5ea64b`..`5ea653`: a waypoint at or past the arrays' length starts over.
        if p.waypoint >= usize::from(p.len) {
            p.waypoint = 0;
            self.set_waypoint(u, 0);
        }
        let goal = (!p.returning).then_some(p.current());
        if self.plane_air_physics(u, goal, frame) == Flew::Done {
            return;
        }
        if p.returning {
            return;
        }
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let at = self.units[u].pos;
        let point = p.current();
        let (gx, gy) = (point.x.clamp(0, xs - 1), point.y.clamp(0, ys - 1));
        if vector_dist(at.x - gx, at.y - gy) < 0x240 {
            // **The waypoint steps on** (item 947): within `0x240` of
            // `points[waypoint]` a patrol with a point after it flies at the
            // next; at the last, a patrol with an order behind it is killed,
            // and one alone circles there.
            if p.waypoint + 1 < usize::from(p.len) {
                p.waypoint += 1;
                self.set_waypoint(u, p.waypoint);
            } else if self.units[u].orders.len() > 1 {
                self.kill_current_order(u);
                return;
            }
        }
        let phase = i64::from(self.units[u].index) + frame;
        if phase & 15 == 0 {
            let me = crate::combat::Obj::Unit(u);
            // The search is round the **last** point (`x_pos[length − 1]`),
            // and a target it finds is struck from the last leg (`waypoint
            // == length − 1`), or from any when it flies (`5ea8f4`: the
            // target's type `domain == 2`). A Bomber asks
            // `find_new_bomber_target`, anything else `find_new_air_target`
            // (item 1182: `is(BOMBER)` at `5ea86a`).
            let found = if self.is_bomber(u) {
                self.find_new_bomber_target(u, p.last())
            } else {
                self.find_new_air_target(u, p.last())
            };
            if let Some(t) = found.filter(|&t| self.valid_target(me, t)).filter(|&t| {
                p.waypoint + 1 == usize::from(p.len)
                    || matches!(self.profile(t).domain, crate::attrition::Domain::Air)
            }) {
                self.add_strafe_order(
                    u,
                    Some(t),
                    p.home,
                    false,
                    crate::orders::QueuePos::First,
                    false,
                );
                return;
            }
        }
        if phase & 31 == 0
            && let Some(b) = self.enemy_building_on_tile(u, p.current())
            && self.buildings[b].ever_seen & Self::who_bit(self.units[u].owner) != 0
        {
            let t = crate::combat::Obj::Building(b);
            self.add_strafe_order(
                u,
                Some(t),
                p.home,
                true,
                crate::orders::QueuePos::First,
                false,
            );
        }
    }

    /// **`UnitData::home_base@00609dc0`**: the building a unit is inside,
    /// or else its front air order's home (`get_air_order`'s `oxx/whose`)
    /// when that is alive and can carry it; `None` (−1) otherwise.
    ///
    /// SEAM: the arm for an order of type `0x19` (the tail's order when
    /// its count is 1); no plane here holds one.
    pub(crate) fn home_base(&self, u: usize) -> Option<usize> {
        if let Some(b) = self.units[u].inside {
            return Some(b);
        }
        let home = self.current_air(u)?.home?;
        let bd = &self.buildings[home];
        (bd.alive && bd.ty.is_some_and(|t| self.is_hangar(t))).then_some(home)
    }

    /// The front air order's `cruising_alt` and `sharp_turn`
    /// (`get_order()->is_air()`, then `update_order()->get_air_order()`'s
    /// `+0xc`/`+0x10`), for `Build::add_gather_point`'s hangar loop.
    pub(crate) fn air_order_heights(&self, u: usize) -> Option<(i32, i32)> {
        self.current_air(u).map(|a| (a.cruising_alt, a.sharp_turn))
    }

    /// `PatrolOrder::waypoint` (`+0x3c`) on the front patrol.
    pub(crate) fn set_waypoint(&mut self, u: usize, w: usize) {
        if let Some(crate::orders::Body::AirPatrol(p)) =
            self.units[u].orders.front_mut().map(|o| &mut o.body)
        {
            p.waypoint = w;
        }
    }

    /// `ObjectsData::find_building_at(tile, SEARCH_ENEMY, who, FILTER_ALL)`
    /// — the enemy building whose footprint holds the point's tile.
    fn enemy_building_on_tile(&self, u: usize, at: Pos) -> Option<usize> {
        let who = self.units[u].owner;
        let tile = at.tile();
        (0..self.buildings.len()).find(|&b| {
            let bd = &self.buildings[b];
            bd.alive
                && bd.owner != who
                && usize::from(bd.owner) < crate::world::PLAYER_SLOTS as usize
                && self.is_enemy(who, bd.owner)
                && self.covers_tile(b, tile)
        })
    }

    /// **`Unit::find_new_air_target(x, y, −1, −1)@005ebc70`** (item 1182,
    /// `docs/GOLDEN.md` §50), the arm a non-bomber's patrol search takes,
    /// off the listing (`5ebc70`..`5ebfd6`). `r` is
    /// `AIRCRAFT_RESPOND_RANGE · 0xc0`; best starts at −1 and a strictly
    /// greater `compare_target(t, 1, 0)` replaces it, in
    /// `Objects::find_units`' order:
    ///
    /// ```text
    /// a computer's plane (unit_masks & 0x40000) searches round itself
    /// find_units(me, SEARCH_ENEMY, r, FILTER_DOMAIN air): each valid
    ///   target within r of the plane; one found is the answer
    /// the plane more than AIRCRAFT_RESPOND_RANGE · 0x3c0 from the point: none
    /// find_units(point, SEARCH_ENEMY, r, FILTER_ALL): each valid target
    ///   within r of the point
    /// ```
    ///
    /// SEAM: the `param_3 ≥ 0` arm between them (a search round a current
    /// target), which the patrol never asks.
    pub(crate) fn find_new_air_target(&self, u: usize, at: Pos) -> Option<crate::combat::Obj> {
        let r = self.tuning.aircraft_respond_range * 0xc0;
        let me = self.units[u].pos;
        let who = self.units[u].owner;
        let at = if self.ai_driven(who) { me } else { at };
        let plane = crate::combat::Obj::Unit(u);
        let mut best: Option<(i32, crate::combat::Obj)> = None;
        let take = |s: &Self, o: usize, from: Pos, best: &mut Option<(i32, crate::combat::Obj)>| {
            let t = crate::combat::Obj::Unit(o);
            // Unit targets only: the capture arm is a building's.
            if !s.valid_target_const(plane, t) {
                return;
            }
            let p = s.units[o].pos;
            if vector_dist(p.x - from.x, p.y - from.y) > r {
                return;
            }
            let v = s.compare_target(plane, t, true, false);
            if v > best.map_or(-1, |(w, _)| w) {
                *best = Some((v, t));
            }
        };
        for o in self.find_units_round(me, r, who) {
            if matches!(
                self.profile(crate::combat::Obj::Unit(o)).domain,
                crate::attrition::Domain::Air
            ) {
                take(self, o, me, &mut best);
            }
        }
        if let Some((_, t)) = best {
            return Some(t);
        }
        let range = self.tuning.aircraft_respond_range;
        if vector_dist(at.x - me.x, at.y - me.y) > range * 0x3c0 {
            return None;
        }
        for o in self.find_units_round(at, r, who) {
            take(self, o, at, &mut best);
        }
        best.map(|(_, t)| t)
    }

    /// `Objects::find_units@0065a620(at, SEARCH_ENEMY, who, range, …)`'s
    /// walk: the cell circle to ring `(range + 0x2ff) / 0x300` round the
    /// point's cell, each cell's chain in order, when that ring holds no
    /// more points than there are units; the unit list otherwise. Units
    /// alive, on the map, and an enemy of `who`'s.
    fn find_units_round(&self, at: Pos, range: i32, who: crate::Player) -> Vec<usize> {
        let circle = crate::ai_place::circle();
        let ring = ((range.max(0) + 0x2ff) / 0x300).min(0x40) as usize;
        let live = self.total_units();
        let mut out = Vec::new();
        let keep = |o: usize, out: &mut Vec<usize>| {
            let x = &self.units[o];
            if x.alive() && x.on_map && self.is_enemy(x.owner, who) {
                out.push(o);
            }
        };
        if circle.radius[ring] <= live {
            let c0 = at.cell();
            for i in 0..circle.radius[ring] {
                let c = crate::world::Cell::new(c0.x + circle.x[i], c0.y + circle.y[i]);
                for o in self.cell_chain(c) {
                    keep(o, &mut out);
                }
            }
        } else {
            for o in 0..self.units.len() {
                keep(o, &mut out);
            }
        }
        out
    }

    /// **`Unit::find_new_bomber_target(x, y, −1, …)@005eb960`**
    /// (`docs/ORDERS.md` §34.5), the arm a patrol's search takes: nothing
    /// when the plane is more than `BOMBER_RESPOND_RANGE · 0x3c0` from the
    /// point; else, of the enemy buildings within `BOMBER_RESPOND_RANGE ·
    /// 0xc0` of it that `valid_target` passes, the best by
    /// `compare_target(t, 1, 0) / (dist / 0xc0 + 1)`.
    ///
    /// SEAM: the first arm (`param_3 ≥ 0`, a search round a current
    /// target at five times the circle), and the `unit_masks & 0x40000`
    /// arm that searches round the plane itself. The candidates are taken
    /// in building order where `Objects::find_builds` walks its rings, so
    /// a tie between two may part.
    pub(crate) fn find_new_bomber_target(
        &mut self,
        u: usize,
        at: Pos,
    ) -> Option<crate::combat::Obj> {
        let range = self.tuning.bomber_respond_range;
        let me = self.units[u].pos;
        if vector_dist(at.x - me.x, at.y - me.y) > range * 0x3c0 {
            return None;
        }
        let who = self.units[u].owner;
        let plane = crate::combat::Obj::Unit(u);
        let mut best: Option<(i32, crate::combat::Obj)> = None;
        for b in 0..self.buildings.len() {
            let (alive, owner, pos) = {
                let bd = &self.buildings[b];
                (bd.alive, bd.owner, bd.pos)
            };
            if !alive || owner == who || !self.is_enemy(who, owner) {
                continue;
            }
            let t = crate::combat::Obj::Building(b);
            if !self.valid_target(plane, t) {
                continue;
            }
            let d = vector_dist(pos.x - at.x, pos.y - at.y);
            if d > range * 0xc0 {
                continue;
            }
            let v = self.compare_target(plane, t, true, false) / (d / 0xc0 + 1);
            if best.is_none_or(|(w, _)| w < v) {
                best = Some((v, t));
            }
        }
        best.map(|(_, t)| t)
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
    /// **`Group::action_flight@006fb260`'s inside arm** — a player's
    /// strike on a plane standing in a base (`docs/ORDERS.md` §38.2,
    /// the listing `6fbbb0`–`6fbea0`; run265's 768). The plane's "inside"
    /// is `base`; the strike needs, in order, a target
    /// `Object::valid_target@00648ba0` passes, and the reach
    /// `vector_dist(base − target) ≤ mana · get_speed(x, y, 1)`, and then
    /// is `add_strafe_order(target, base, 1, QUEUE_NEW, 1)`: a strafe on
    /// the target, home the base, `returning 0`. The plane stays inside;
    /// [`Sim::do_launch`] puts it out.
    ///
    /// **A missile takes the same arm** (`6fbbb0`..`6fbea0`, item 1050):
    /// the valid target, the reach, and then `add_strafe_order`, whose
    /// head makes it an air attack on the target's point
    /// ([`Sim::add_air_attack_ground_order`]). run371's V2 `0/10` on 2672.
    ///
    /// **The shield refuses a missile** (item 1078, `6fbbd7`..`6fbbfb`):
    /// unless the target is the player's own, a target owner holding
    /// `MISSILE_DEFENSE_BONUS` ([`Sim::missile_defense_held`]) takes no
    /// missile's order. run390's V2c `0/15` pressed on who=1's Barracks on
    /// 3080, ten frames after `tech who=1 missile_shield on`: no order, and
    /// no launch.
    ///
    /// SEAM: the `NUCLEARMISSILE` arm (`can_nuke`, once a call), `Game::war_allowed`
    /// (always allowed here), `Object::valid_target`'s capture arm, and an
    /// air patrol's home standing for the "inside"; no capture reaches
    /// any of them.
    pub(crate) fn strike_from_inside(&mut self, u: usize, base: usize, target: crate::combat::Obj) {
        let me = crate::combat::Obj::Unit(u);
        if !self.valid_target(me, target) {
            return;
        }
        let whom = self.owner_of(target);
        if self.profile(me).has(crate::combat::mask::MISSILE)
            && whom != self.units[u].owner
            && self.missile_defense_held(whom)
        {
            return;
        }
        let (b, t) = (self.buildings[base].pos, self.pos_of(target));
        let reach = self.unit_mana(u) * self.get_speed(u, 1);
        if vector_dist(b.x - t.x, b.y - t.y) > reach {
            return;
        }
        self.add_strafe_order(
            u,
            Some(target),
            Some(base),
            true,
            crate::orders::QueuePos::New,
            true,
        );
    }

    /// `LeaderData::has_preq(MISSILE_DEFENSE_BONUS)` (item 1078): the
    /// player holds the bonus's one prerequisite, Missile Shield
    /// (`crate::tech::Roles::missile_defense_preq`). A tree without the
    /// role holds no shield.
    pub(crate) fn missile_defense_held(&self, who: crate::Player) -> bool {
        (who as usize) < self.tech.len()
            && self.tech_tree.roles.missile_defense_preq.is_some_and(|t| {
                self.tech_tree
                    .has_tech(&self.setup, &self.tech[who as usize], t)
            })
    }

    /// **`Group::action_buildmask@006fc9a0` on a group of buildings** —
    /// the player's repeat button (`Options::set_air_repeat@0071c740`)
    /// through `CommandPackage::process_buildmask@00947680`
    /// (`docs/GOLDEN.md` §32). **It toggles; it does not set**: the
    /// command's `set` is always 1 on the wire and the function never
    /// reads it (the listing, and 4232 ↔ 4104 under the emulator). Each
    /// member that is active and that `WallData::valid_buildmask@0063e2a0`
    /// admits — for `0x80`, `can_carry(AIR)`, [`Sim::is_hangar`] — gets the
    /// bit **set if it lacks it and every admitted member before it was
    /// set; otherwise cleared**, and so is every admitted member after a
    /// clear. run281's Airbase reads 4104 from 1442. Returns the members
    /// written.
    ///
    /// **And `0x40`, the infinite queue** (item 877, `docs/GOLDEN.md`
    /// §33): `valid_buildmask` admits it on a member that
    /// [`Sim::can_infinite`] passes — a training building with a train
    /// job queued — and the same toggle writes
    /// [`production::Queue::infinite`](crate::production::Queue::infinite).
    /// run285's Barracks reads 4160 from 902. The two carried bits are
    /// the word's `0x40` and `0x80`; a mask with neither writes nothing,
    /// and the message and sound the `0x40` toggle plays for the console's
    /// player are the interface's.
    pub fn action_buildmask(&mut self, buildings: &[usize], mask: i32) -> usize {
        let (queue, repeat) = (mask & 0x40 != 0, mask & 0x80 != 0);
        let mut all_set = true;
        let mut written = 0;
        for &b in buildings {
            let bd = &self.buildings[b];
            // `WallData::valid_buildmask@0063e2a0`: either bit's own test.
            let admitted = (queue && self.can_infinite(b))
                || (repeat && bd.ty.is_some_and(|t| self.is_hangar(t)));
            if !bd.alive || !admitted {
                continue;
            }
            // `(mask & build_masks) == 0`: none of the mask's bits held.
            let held = (queue && bd.queue.infinite) || (repeat && bd.repeat_air);
            let set = !held && all_set;
            let bd = &mut self.buildings[b];
            if queue {
                bd.queue.infinite = set;
            }
            if repeat {
                bd.repeat_air = set;
            }
            all_set = set;
            written += 1;
        }
        written
    }

    /// **`Object::do_launch@0064f3b0` for a building** — called from
    /// `Build::process@0061edf0` for a type with `build_masks & 8`, a
    /// hangar (`docs/ORDERS.md` §38.3). Only while something is inside:
    /// the counter is under `inside_down ≥ 0` with the rest (run265: an
    /// emptied base reads `launch_frames 0` from 778 on). `launch_frames`
    /// counts up, and once it was at [`FRAMES_BETWEEN_LAUNCHES`] it
    /// saturates there and the chain is walked:
    /// - a plane with no order, or with `mana_burn ≠ 0`, is passed over —
    ///   **the tank gates the launch** (run265: the strike laid on 767
    ///   waits to 778, the block the tank first reads 0);
    /// - one whose front order has the action bit, or whose base repeats
    ///   (`has_repeat_air() || flags & 4`), stays: a strike whose
    ///   target is invalid and whose point is off the world is killed; a
    ///   patrol's `returning` is cleared; it joins `launching`, and the
    ///   first of the call comes out ([`Sim::come_out`], whose tail is
    ///   the EXIT, [`Sim::exit_at_airbase`]) and `launch_frames` is 0;
    /// - one without is killed, and leaves `launching`.
    ///
    /// **A missile's launch is the silo's countdown** (item 1050): while
    /// the building's `recharging` (`BuildData +0x7a`) is not 0, the call
    /// is [`Sim::do_missile_launch`] and nothing else (`64f3e0`..`64f40f`),
    /// so `launch_frames` stands; and the launch of a missile sets
    /// `recharging` to the building type's `RECHARGE` (`+0x1f4`, 30 at a
    /// Missile Silo) where a plane comes out (`64f73b`..`64f7a8`), and
    /// zeroes `launch_frames` as a plane's does. run371's silo `0/2009`:
    /// `launch_frames` 15 → 0 and `recharging` 30 on 2672, the order's own
    /// frame.
    ///
    /// SEAM: a strafe home to another, full base turned `AirPatrolOrder`,
    /// and the chain's order, which is the garrison list's here (one plane
    /// in every capture).
    pub(crate) fn do_launch(&mut self, b: usize) {
        if self.buildings[b].garrison.is_empty() {
            return;
        }
        if self.buildings[b].recharging != 0 {
            self.do_missile_launch(b);
            return;
        }
        let was = self.buildings[b].launch_frames;
        self.buildings[b].launch_frames = was + 1;
        if was < FRAMES_BETWEEN_LAUNCHES {
            return;
        }
        self.buildings[b].launch_frames = FRAMES_BETWEEN_LAUNCHES;
        let mut launched = false;
        for u in self.buildings[b].garrison.clone() {
            if self.units[u].inside != Some(b) || !self.units[u].alive() {
                continue;
            }
            let Some(front) = self.units[u].orders.front().copied() else {
                continue;
            };
            if self.units[u].mana_burn != 0 {
                continue;
            }
            // `0064f4b0`: the base's vslot `0xf0` ([`Building::repeat_air`]
            // (crate::Building)) or the action bit keeps the order;
            // neither kills it. run281's `0/6` on 1585: its unflagged
            // patrol, kept by `land_plane` under the bit, is killed once
            // the bit is gone (`docs/GOLDEN.md` §32).
            if !self.buildings[b].repeat_air && front.flags & crate::orders::flag::ACTION == 0 {
                self.kill_current_order(u);
                self.buildings[b].launching.retain(|&x| x != u);
                continue;
            }
            match front.body {
                crate::orders::Body::Strafe(sf) => {
                    if let Some(t) = sf.target
                        && !self.buildings[b].launching.contains(&u)
                        && !self.valid_target(crate::combat::Obj::Unit(u), t)
                        && sf.at.is_none_or(|p| !self.in_world(p))
                    {
                        self.kill_current_order(u);
                        continue;
                    }
                }
                // `64f6a0`: the patrol's `waypoint` (`+0x3c`, the PDB's
                // `PatrolOrder` field list) is set 0 — not `returning`
                // (`+0x58`), which `land_plane` has cleared (item 947).
                crate::orders::Body::AirPatrol(_) => self.set_waypoint(u, 0),
                _ => {}
            }
            if !self.buildings[b].launching.contains(&u) {
                self.buildings[b].launching.push(u);
            }
            if !launched {
                let missile = self
                    .profile(crate::combat::Obj::Unit(u))
                    .has(crate::combat::mask::MISSILE);
                if missile {
                    self.buildings[b].recharging =
                        self.profile(crate::combat::Obj::Building(b)).recharge;
                }
                let out = !missile && self.come_out(u);
                self.buildings[b].launch_frames = 0;
                launched = true;
                // **The walk ends at a launch** (item 915, `docs/GOLDEN.md`
                // §38): the chain's next link is read from the launched
                // plane's own `inside_down` (`64f806`) after `come_out`,
                // and `Object::remove_from_inside` has stored −1 there. A
                // plane that did not come out keeps its link, and the walk
                // goes on.
                if out {
                    break;
                }
            }
        }
        self.computer_sortie(b);
    }

    /// **The computer's sortie** (`Object::do_launch@0064f3b0`, `64f81c`..
    /// `6508a7`, item 1182, `docs/GOLDEN.md` §50): past the chain walk, on
    /// every call that reached it, for a base whose owner is not human.
    ///
    /// ```text
    /// (frame + base id) % 32 == 0, and not leader_flags2 & 8
    /// a base that is(MISSILESILO): the silo's arm ([`Sim::silo_strike`])
    /// else, over the eight leaders L with leader_flags & 3 == 3 that the
    ///   owner is not at peace with, best = -1, strictly greater replaces:
    ///   L's cities with city_flags & 3 == 3 (alive and under attack):
    ///     skipped when the owner lacks STEALTHBOMBER and an AIRDEFENSE of
    ///     an enemy's stands within 0x900 (find_building, SEARCH_ENEMY);
    ///     (damage / 100 + 1000), doubled when L is_ally the owner,
    ///     / (vector_dist(city building - base) / 768 + 1)
    ///   and, when L is an enemy, its forts and its wonders (SEAM, below)
    /// a target: each unit of the base's chain whose type is of the air
    ///   domain is cleared (Unit::clear_orders), and when
    ///   vector_dist(target - base) < get_speed(it, 1) * mana(it)
    ///   (UnitData::get_speed@006086f0, the vslot 0x17c at its point) and,
    ///   for a friend's city, it is(BIPLANE), it takes
    ///   add_air_patrol_order(target, the base, action 0)
    /// ```
    ///
    /// SEAM: `leader_flags2 & 8`, the combat AI's scenario switch, which
    /// no staging sets; the forts' and wonders' loops (`64fa71`..`64fda4`), which this crate
    /// keeps no list for; the AIRDEFENSE search's radius, read as twelve
    /// tiles from `find_building`'s tile arithmetic and not run.
    fn computer_sortie(&mut self, b: usize) {
        let owner = self.buildings[b].owner;
        if self.nation.get(owner as usize).is_none_or(|n| n.human) {
            return;
        }
        let id = i64::from(self.buildings[b].index);
        if (self.frame + id) % 32 != 0 {
            return;
        }
        if self.building_ident(b) == crate::build::Ident::MissileSilo {
            self.silo_strike(b);
            return;
        }
        let base = self.buildings[b].pos;
        // A tree that does not carry the tech (the harness's small
        // fixtures) answers no.
        let stealth = STEALTHBOMBER < self.tech_tree.types.len()
            && self
                .tech
                .get(owner as usize)
                .is_some_and(|p| self.tech_tree.has_tech(&self.setup, p, STEALTHBOMBER));
        let mut best = -1;
        let mut target: Option<(crate::Player, usize)> = None;
        for l in 0..self.players.len() as crate::Player {
            if self.defeated[l as usize] || self.is_peace(owner, l) {
                continue;
            }
            for c in 0..self.cities.len() {
                let city = &self.cities[c];
                if city.owner != l || !city.alive || !city.no_heal {
                    continue;
                }
                let cb = city.building;
                if !stealth && self.enemy_air_defense_near(owner, self.buildings[cb].pos) {
                    continue;
                }
                let bd = &self.buildings[cb];
                let mut v = (bd.hits - bd.health) / 100 + 1000;
                if self.is_ally(l, owner) {
                    v *= 2;
                }
                let p = bd.pos;
                v /= vector_dist(p.x - base.x, p.y - base.y) / 768 + 1;
                if v > best {
                    best = v;
                    target = Some((l, cb));
                }
            }
        }
        let Some((l, t)) = target else {
            return;
        };
        let at = self.buildings[t].pos;
        let d = vector_dist(at.x - base.x, at.y - base.y);
        for u in self.buildings[b].garrison.clone() {
            if self.unit_domain_of(u) != crate::attrition::Domain::Air {
                continue;
            }
            self.clear_orders(u);
            if d >= self.get_speed(u, 1) * self.unit_mana(u) {
                continue;
            }
            if self.is_ally(l, owner) && !self.air_line_is(u, crate::airbase::BIPLANE) {
                continue;
            }
            self.add_air_patrol_order(u, at, Some(b), false);
        }
    }

    /// **The computer's silo strike** (`Object::do_launch@0064f3b0`'s
    /// silo arm, `64fdcd`..`6508a6`, item 1591, `docs/AI.md` §164): past
    /// the sortie's own gate, at a base that `is(MISSILESILO)`.
    ///
    /// ```text
    /// (frame + base id) % 128 == 0, and num_inside(1) != 0
    /// m = the chain's head (inside_down: the first to have entered)
    /// m is(NUCLEARMISSILE) → the nuke arm (an ICBM is one by its line)
    ///   (armageddon >= get_armageddon() - 2 → nothing: SEAM, below)
    /// reach = get_speed(m, at m, 1) * mana(m)       # the vslot 0x17c
    /// over the eight leaders L with leader_flags & 1, is_enemy(owner, L)
    ///   and not has_preq(L, MISSILE_DEFENSE_BONUS), best = -1:
    ///   L's cities with city_flags & 1 whose building's ever_seen != 0
    ///   (`64ff58`: any player's bit; the `1 << who` beside it is never
    ///   0) and no object of the owner's within 0x1800 of it
    ///   (ObjectsData::find(SEARCH_FRIENDLY, 0x1800, FILTER_ALL) < 0):
    ///     v = num_buildings(city) * (hits_left + 1000)
    ///     d = vector_dist(building - silo); skipped when d > reach
    ///     v /= d / 0x1200 + 1; strictly greater replaces
    /// a target: add_air_attack_ground_order(m, its point, home -1,
    ///   QUEUE_NEW, action 1)
    /// ```
    ///
    /// run710's ICBM `1/42` (`TypeIndex` 316) in the silo `1/2015`: on
    /// 1569 (`(1569 + 2015) % 128 == 0`) it takes `AIR_ATTACK_GROUND` at
    /// Napata's point (6240, 7008), the human's one city; the next
    /// `do_launch` counts the silo's `recharging` down from 30, it leaves
    /// on 1600 and its round lands on 1719.
    ///
    /// SEAM: the non-nuke arm (a V2 or Cruise Missile at the head:
    /// `hits_left ≥ 500`, `damage + 1000`, a draw `% 10` for a city whose
    /// `city_flags & 2` is clear, and a third list of targets), which no
    /// capture has reached (scan: `report.py <log> when Object::do_launch`
    /// over the 351 `rontrace-*.log` on 2026-10-07: no draw from
    /// `do_launch` in any — the arm's `% 10` roll is its only draw, so a
    /// V2 at a silo's head over an unflagged city was never met); the
    /// forts' and wonders' loops (`650150`..`650544`), for which this
    /// crate keeps no list; the Armageddon counter (`Game +0x6e0`), which
    /// no field here holds (`crate::nuke`), so the gate is read as open; and the friendly search's cell ring,
    /// walked here as every object of the owner's within the radius
    /// ([`Sim::enemy_object_within`]'s same reading).
    fn silo_strike(&mut self, b: usize) {
        let id = i64::from(self.buildings[b].index);
        if (self.frame + id) % 128 != 0 {
            return;
        }
        let Some(&m) = self.buildings[b].garrison.first() else {
            return;
        };
        if !self.air_line_is(m, crate::airbase::NUCLEARMISSILE) {
            return;
        }
        let owner = self.buildings[b].owner;
        let silo = self.buildings[b].pos;
        let reach = self.get_speed(m, 1) * self.unit_mana(m);
        let mut best = -1;
        let mut target = None;
        for l in 0..self.players.len().min(8) as crate::Player {
            if !self.is_enemy(owner, l) || self.missile_defense_held(l) {
                continue;
            }
            for c in 0..self.cities.len() {
                let city = &self.cities[c];
                if city.owner != l || !city.alive {
                    continue;
                }
                let cb = city.building;
                let at = self.buildings[cb].pos;
                if self.buildings[cb].ever_seen == 0 || self.own_object_within(owner, at, 0x1800) {
                    continue;
                }
                let d = vector_dist(at.x - silo.x, at.y - silo.y);
                if d > reach {
                    continue;
                }
                let v = self.num_buildings(c) * (self.buildings[cb].health.max(0) + 1000)
                    / (d / 0x1200 + 1);
                if v > best {
                    best = v;
                    target = Some(at);
                }
            }
        }
        if let Some(at) = target {
            self.add_air_attack_ground_order(m, at, None, crate::orders::QueuePos::New, true);
        }
    }

    /// `ObjectsData::find(x, y, SEARCH_FRIENDLY, who, range, ·, FILTER_ALL)
    /// >= 0`, as the silo strike asks it: a live unit on the map or a live
    /// building of `who`'s own (`SEARCH_FRIENDLY` is case 1 of
    /// `Search::valid_search@0067daa0`, the asker alone), within `range`
    /// of `at`.
    fn own_object_within(&self, who: crate::Player, at: Pos, range: i32) -> bool {
        let near = |p: Pos| vector_dist(p.x - at.x, p.y - at.y) <= range;
        self.units
            .iter()
            .any(|u| u.alive() && u.on_map && u.owner == who && near(u.pos))
            || self
                .buildings
                .iter()
                .any(|bd| bd.alive && bd.owner == who && near(bd.pos))
    }

    /// `LeaderData::is_peace@006e1200`: two players, a treaty each way,
    /// and not an alliance both ways.
    pub(crate) fn is_peace(&self, a: crate::Player, b: crate::Player) -> bool {
        a != b && !self.is_enemy(a, b) && !self.is_ally(a, b)
    }

    /// `ObjectsData::find_building(x, y, SEARCH_ENEMY, who, 0x900, 0,
    /// FILTER_TYPE, AIRDEFENSE, 0)@0065d260 >= 0`, as the sortie asks it:
    /// a live building of the AIRDEFENSE line whose owner is an enemy of
    /// `who`'s, within twelve tiles. SEAM: the radius's unit, read and not
    /// run (no staging has put one beside a city).
    fn enemy_air_defense_near(&self, who: crate::Player, at: Pos) -> bool {
        let a = at.tile();
        self.buildings.iter().enumerate().any(|(x, bd)| {
            bd.alive
                && self.is_enemy(bd.owner, who)
                && self.building_ident(x) == crate::build::Ident::AirDefense
                && {
                    let p = bd.pos.tile();
                    vector_dist(p.x - a.x, p.y - a.y) <= 0x900 / 0xc0
                }
        })
    }

    /// **`Build::do_missile_launch@00622670`** (item 1050): at a building
    /// that `is(MISSILESILO)`, while `launching` holds a missile, one off
    /// `recharging` a frame; at 0 the first of `launching` leaves it,
    /// comes out ([`Sim::come_out`], on the silo's own point) and is
    /// processed in the same call — `Unit::process` (vslot `0x9c`), whose
    /// order step is [`Sim::do_air_attack_ground`]: the flight's first step,
    /// the round and the missile's end, all inside the silo's own
    /// `Build::process`. An empty `launching` zeroes `recharging`. The
    /// strike's message and sound, and a nuke's `+0x40` and vslot `0x164`,
    /// write nothing this crate carries. run371: `recharging` 30 on 2672,
    /// 1 on 2701, and on 2702 `0/10` gone, `launching` empty and its round
    /// in flight — the trace's `Ammo::init+0xae8`/`+0xb25` on 2701, ahead
    /// of every draw of the frame's gaia.
    pub(crate) fn do_missile_launch(&mut self, b: usize) {
        if self.building_ident(b) != crate::build::Ident::MissileSilo {
            return;
        }
        if self.buildings[b].launching.is_empty() {
            self.buildings[b].recharging = 0;
            return;
        }
        self.buildings[b].recharging -= 1;
        if self.buildings[b].recharging != 0 {
            return;
        }
        let u = self.buildings[b].launching.remove(0);
        // The nuke arm (item 1091, `do_missile_launch`): a nuke on its
        // strike shows the silo to everyone, `+0x40` (`visible`) `0xff`,
        // then vslot `0x164`, `Wall::update_local_seen@0063ed50`, whose
        // seen bits nothing here reads.
        if self.air_line_is(u, crate::airbase::NUCLEARMISSILE)
            && matches!(
                self.current_order(u).map(|o| o.body),
                Some(crate::orders::Body::AirAttackGround(_))
            )
            && !self.nukes.shown.contains(&b)
        {
            self.nukes.shown.push(b);
        }
        self.come_out(u);
        let frame = self.frame;
        self.work(u, frame);
    }

    /// **`Unit::do_air_attack_ground@005ea420`** (item 1050), for the one
    /// unit that takes the order here, a missile:
    /// - `do_air_physics` flies the frame's step at the point
    ///   ([`Sim::plane_air_physics`]); a 0 return ends the call;
    /// - the unit's own `recharging` (`+0xae`) not 0, or the order's
    ///   `returning` (`+0x2c`), ends it;
    /// - `ObjectData::is_in_range@0064e4a0` of the point: a missile that is
    ///   not inside skips the whole test and is in range, so there is no
    ///   reach here;
    /// - the facing test is a non-missile's, and is passed over;
    /// - `set_attack(−1, −1)`, and a type neither strafing nor of the
    ///   Bomber line with an ammo piece fires at once, `fire_ammo(−1, −1)`
    ///   — [`Sim::missile_round`];
    /// - and a missile then dies ([`Sim::missile_dies`]).
    ///
    /// SEAM: every non-missile arm — the facing test's `is(0x127)`, the
    /// `CHAR_ATTACK2` release, the reload and the Bomber's mana cost. This
    /// crate lays the order on a missile alone
    /// ([`Sim::add_strafe_order`]'s head).
    pub(crate) fn do_air_attack_ground(&mut self, u: usize, frame: i64) {
        let Some(crate::orders::Body::AirAttackGround(g)) = self.current_order(u).map(|o| o.body)
        else {
            return;
        };
        // **A flock's bird** (`crate::flock`): its `do_air_physics` leaves
        // `recharging` at 1, so the `+0xae` test below returns on every
        // frame it flies.
        if self.is_flockbird(u) {
            self.flock_air_physics(u, g.at, frame);
            return;
        }
        if matches!(self.plane_air_physics(u, Some(g.at), frame), Flew::Done) {
            return;
        }
        if self.units[u].combat.recharging != 0 || g.returning {
            return;
        }
        let me = crate::combat::Obj::Unit(u);
        if !self.profile(me).has(crate::combat::mask::MISSILE) {
            return;
        }
        self.clear_attack(u);
        self.missile_round(u, g.at);
        self.missile_dies(u);
    }

    /// **`Ammo::init@0067bbf0`'s missile arm** (item 1050): the round of a
    /// shooter with the missile flag whose order is an air attack on the
    /// ground (`local_38`, the order's `AttackGroundOrder` base).
    /// - It leaves from the unit's point plus [`MISSILE_OFFSET`]
    ///   (`graphic_events +0xc8..+0xd0`, `67c1d5`..`67c263`), `sz` the
    ///   unit's `z_internal` plus its third term.
    /// - Its accuracy is `to_hit − attenuate · (dist / 192)` against the
    ///   plain distance to the point, and its scatter the land-unit
    ///   formula **doubled** for a missile that is not a nuke (`67c64b`) —
    ///   two draws, `Ammo::init+0xae8` and `+0xb25`, `point − s/2 + roll %
    ///   s` on each axis; `ez` is `find_data_z` at the point, never under 0.
    /// - Its flight is a spline (`traj` 2, `Spline::calc_nuke_spline`),
    ///   and its time the spline's points less one: `depth` is 120 on both
    ///   of `calc_nuke_spline`'s arms (`+0x60 = 0x780003`), so
    ///   [`MISSILE_FLIGHT`] frames whatever the distance.
    /// - `v1z` is every round's formula over `sz`, `ez` and the time.
    ///
    /// run371's V2 on 2701 (seed `0x14e73b8f`, rolls 62369 and 56984):
    /// accuracy 228 over 4,662, `s` 22, `ex` 13834 and `ey` 14969 off the
    /// point (13824, 14976), `sz` 466, `ez` 78, 120 frames, `v1z`
    /// 626.016663.
    ///
    /// SEAM: a nuke (`is(0x13b)`: no scatter, and the spline's other arm),
    /// the spline's own points, which the simulation never reads, and
    /// `Ammo::init`'s `balance.attacks` count.
    pub(crate) fn missile_round(&mut self, u: usize, at: Pos) {
        let me = crate::combat::Obj::Unit(u);
        let p = self.profile(me);
        let here = self.units[u].pos;
        let launch = Pos::new(here.x + MISSILE_OFFSET.0, here.y + MISSILE_OFFSET.1);
        let sz = self.world.tile_z(here.tile()).max(0) + MISSILE_OFFSET.2;
        let acc = crate::combat::accuracy(
            p.to_hit,
            p.attenuate,
            vector_dist(at.x - launch.x, at.y - launch.y),
        );
        // A nuke's scatter is 0 (`67c64b`: `is(0x13b)` zeroes it where a V2
        // doubles it), and `s − 1 < 1` takes neither draw (item 1091).
        let s = if self.air_line_is(u, crate::airbase::NUCLEARMISSILE) {
            0
        } else {
            crate::combat::scatter(&self.tuning, acc, true, true, false)
        };
        let mut landing = at;
        // `s − 1 < 1` (`local_28`) takes no draw.
        if s > 1 {
            self.mark(crate::fight::SITE_AMMO_GROUND_SCATTER_X);
            landing.x += self.rng.roll() % s - s / 2;
            self.mark(crate::fight::SITE_AMMO_GROUND_SCATTER_Y);
            landing.y += self.rng.roll() % s - s / 2;
        }
        let ez = self.ground_z(at).max(0);
        self.add_ammo(crate::combat::Projectile {
            shooter: me,
            owner: self.units[u].owner,
            target: None,
            launch,
            landing,
            cur_time: 0,
            total_time: MISSILE_FLIGHT,
            accuracy: acc,
            angle: find_angle(landing.x - launch.x, landing.y - launch.y),
            splash_area: p.splash_area,
            num_guys: 1,
            air: false,
            rolling: false,
            missed: false,
            harmless: false,
            sz,
            ez,
            v1z: crate::combat::arc_v1z(sz, ez, MISSILE_FLIGHT),
            slot: 0,
        });
    }

    /// **`Object::die(this, 0, −1, 0.0)`** for a missile that has fired
    /// (item 1050): `dtype` 0, so `Unit::close` takes no death draw and
    /// leaves no death object; the squad relink, the slot held while its
    /// round flies (`Object::die`'s tail, the same as a combat death's:
    /// `nuke_effect +0x108` + 1 + `total_time − cur_time` of its live
    /// ammo, 151 on run371's 2701 — 121 until item 1594 read the 30 in),
    /// the supply slot and both collision indices, and the object
    /// forgotten.
    ///
    /// **And the player's count of the type** (item 1078): `Unit::close`
    /// takes a unit that is no squad follower (`+0x8e < 0`) and whose type
    /// has population out of `num_units` — `Leader::track_unit_type(type,
    /// −1)` at `0060f3db` (the `pop == 0` arm admits `is(0x134)` and a
    /// governor-hero alone). A missile is never a follower, and its `POP`
    /// is one. Without it a dead V2 stayed counted, and the next V2's
    /// price and time were one step up the ramp: run390's second silo
    /// charged 100 and 100 on 2722, and this crate 120 and 120.
    pub(crate) fn missile_dies(&mut self, u: usize) {
        self.units[u].health = self.units[u].health.min(0);
        if let Some(ty) = self.units[u].ty
            && self.unit_types[ty].price.pop != 0
        {
            self.track_unit_type(self.units[u].owner, ty, -1);
        }
        self.relink_squad(u);
        let me = crate::combat::Obj::Unit(u);
        self.close_dead_orders(u);
        self.hold_dead_slot(u);
        self.close_supply(u);
        self.forget(me);
    }

    /// **`Unit::do_spec_anim@005e5880`'s EXIT at an `AIRBASE`** — the
    /// tail `Unit::come_out@00617c10` hands a unit leaving a building
    /// with a piece on a frame past 0 (`docs/GOLDEN.md` §30,
    /// `docs/ORDERS.md` §38.4; the listing `5e59cb`–`5e5ad7`). The plane
    /// is turned to 0 (`Unit::set_angle`), put on the base's point less
    /// `0xc0` (`set_new_location(·, ·, 1, 1)`), its figure on the ground
    /// there (`Guy::set_new_z(find_data_z, 1)`), its bank and pitch moved
    /// to `last_*` and zeroed; the order's kill takes it out of the
    /// base's `launching`; and then its `work` — the strike's first step,
    /// in the same call. run265's 778: the figure on (11424, 13920) at
    /// `last_z` 157, `last_bank` and `last_pitch` 0.0, the unit one step
    /// north. Nothing in the arm resets the figure's running
    /// `avg_speed`, which [`Sim::come_out`]'s placement rebuilds from rest:
    /// it is handed back (25 on 778, the value it stood inside with).
    ///
    /// **A helicopter's exit** (`unit_flags & 0x20`, `5e59d5`..`5e5a1c`,
    /// item 1019): two draws, `x` less `197 − rand % 11` and `y` plus
    /// `rand % 11 − 5`, in that order, and its figure 200 over the ground
    /// (`Guy::set_new_z(z + 200, 1)`, `5e5a91`).
    pub(crate) fn exit_at_airbase(&mut self, u: usize, host: usize, avg_speed: i32) {
        let at = self.buildings[host].pos;
        let heli = self.is_helicopter(u);
        let spot = if heli {
            self.mark(SITE_HELI_EXIT_X);
            let dx = self.rng.roll() % 11 - 0xc5;
            self.mark(SITE_HELI_EXIT_Y);
            let dy = self.rng.roll() % 11 - 5;
            Pos::new(at.x + dx, at.y + dy)
        } else {
            Pos::new(at.x - 0xc0, at.y)
        };
        {
            let m = &mut self.units[u].movement;
            m.heading = Angle(0);
            m.facing = Angle(0);
            m.frame_facing = Angle(0);
            m.des_angle = Angle(0);
        }
        let from = self.units[u].pos;
        self.set_new_location(u, spot, true);
        self.units[u].movement.body.avg_speed = avg_speed;
        self.moved_to(u, from, false);
        let z = self.ground_z(at);
        let af = &mut self.units[u].airframe;
        af.z = z;
        af.last_z = z;
        if heli {
            af.z += 200;
            af.last_z = af.z;
        }
        af.last_pitch = af.pitch;
        af.pitch = Single::ZERO;
        af.last_bank = af.bank;
        af.bank = Single::ZERO;
        // The kill's `unit_masks &= ~0x20000` is a caster's bit, which a
        // plane never holds.
        self.buildings[host].launching.retain(|&x| x != u);
        self.update_action(u);
        let frame = self.frame;
        self.work(u, frame);
    }

    fn land_plane(&mut self, u: usize) {
        let Some(home) = self.current_air(u).and_then(|a| a.home) else {
            return;
        };
        // **Every air order lands with `returning` cleared** (`+0x18` of
        // the `AirOrder`, `005e9950`), and only a strafe with no live
        // target goes: run265's Fighter comes home under its patrol and
        // is inside on 1385 with the patrol still on its stack.
        self.set_returning(u, false);
        let mut killed = false;
        if let Some(sf) = self.current_strafe(u)
            && sf.target.is_none()
        {
            if sf.mandatory {
                self.clear_orders(u);
            } else {
                self.kill_current_order(u);
            }
            killed = true;
        }
        // **The home's vslot `0xf0`** (`005e9a43`): `WallData::has_repeat_air
        // @00472410`, `build_masks & 0x80` (`docs/ORDERS.md` §40). Set, the
        // stack stays and, unless the strafe above went, the order it holds
        // loses its action bit (`update_order(this)->flags &= ~4`);
        // clear, the path and every order go. run265's Airbase reads 4232:
        // the Fighter is inside on 1385 and the Bomber on 1489 each with
        // its `AIRPATROLORDER`, flags 0.
        if self.buildings[home].repeat_air {
            if !killed && let Some(o) = self.units[u].orders.front_mut() {
                o.flags &= !crate::orders::flag::ACTION;
            }
        } else {
            self.units[u].path.clear();
            self.close_orders(u);
            self.clear_partial_path(u);
            self.update_action(u);
        }
        self.go_inside(u, home);
    }

    fn current_strafe(&self, u: usize) -> Option<crate::orders::StrafeOrder> {
        match self.current_order(u).map(|o| o.body) {
            Some(crate::orders::Body::Strafe(sf)) => Some(sf),
            _ => None,
        }
    }

    /// The front order's `AirOrder` base — `get_air_order` (vslot
    /// `+0x84`/`+0xfc`), which the three air classes answer and every
    /// other order answers 0.
    pub(crate) fn current_air(&self, u: usize) -> Option<AirBase> {
        match self.current_order(u).map(|o| o.body) {
            Some(crate::orders::Body::Strafe(sf)) => Some(AirBase {
                home: sf.home,
                cruising_alt: sf.cruising_alt,
                sharp_turn: sf.sharp_turn,
                returning: sf.returning,
            }),
            Some(crate::orders::Body::AirPatrol(p)) => Some(AirBase {
                home: p.home,
                cruising_alt: p.cruising_alt,
                sharp_turn: p.sharp_turn,
                returning: p.returning,
            }),
            Some(crate::orders::Body::AirAttackGround(g)) => Some(AirBase {
                home: g.home,
                cruising_alt: g.cruising_alt,
                sharp_turn: g.sharp_turn,
                returning: g.returning,
            }),
            _ => None,
        }
    }

    fn with_air(&mut self, u: usize, f: impl FnOnce(&mut i32, &mut i32)) {
        match self.units[u].orders.front_mut().map(|o| &mut o.body) {
            Some(crate::orders::Body::Strafe(sf)) => f(&mut sf.cruising_alt, &mut sf.sharp_turn),
            Some(crate::orders::Body::AirPatrol(p)) => f(&mut p.cruising_alt, &mut p.sharp_turn),
            Some(crate::orders::Body::AirAttackGround(g)) => {
                f(&mut g.cruising_alt, &mut g.sharp_turn)
            }
            _ => {}
        }
    }

    /// `WorldData::is_valid`: a point on the world.
    pub(crate) fn in_world(&self, p: Pos) -> bool {
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        p.x >= 0 && p.y >= 0 && p.x < xs && p.y < ys
    }

    /// The front air order's `returning` (`AirOrder +0x18`).
    pub(crate) fn set_returning(&mut self, u: usize, on: bool) {
        match self.units[u].orders.front_mut().map(|o| &mut o.body) {
            Some(crate::orders::Body::Strafe(sf)) => sf.returning = on,
            Some(crate::orders::Body::AirPatrol(p)) => p.returning = on,
            Some(crate::orders::Body::AirAttackGround(g)) => g.returning = on,
            _ => {}
        }
    }

    pub(crate) fn set_cruising_alt(&mut self, u: usize, alt: i32) {
        self.with_air(u, |c, _| *c = alt);
    }

    pub(crate) fn set_sharp_turn(&mut self, u: usize, t: i32) {
        self.with_air(u, |_, s| *s = t);
    }

    /// **`Unit::bank_aircraft(des, &speed, 0)` for a plane** —
    /// [`Sim::bank_aircraft`]'s shape, with the three arms the order's
    /// `returning` flag opens (`docs/ORDERS.md` §33.3): the bank it wants
    /// is **doubled** before the clamp, a **second turn** toward `des`
    /// follows the first (and takes it outright inside 3°), and a banked
    /// plane flies at **three quarters** of its speed.
    ///
    /// **Not returning** (`docs/ORDERS.md` §34.6), a player's plane wants
    /// **no bank at all within 199 of the ground** under it (`0x5e9700`:
    /// `|guy.z − find_tcoord_z| ≤ 199`), and none of the three arms: it
    /// levels its wings off the runway before it turns.
    ///
    /// **An animal** — a flock's bird (`crate::flock`) — takes two arms a
    /// plane never does (`005e9520`): with its bank settled and its
    /// `spell_time` not negative it skips the whole call every eighth
    /// frame (`frame & 7`, the game's own), and the ground test is an
    /// owner-under-eight's (`(byte)+0x9 > 7` goes straight to the clamp).
    pub(crate) fn bank_plane(&mut self, u: usize, des: Angle, speed: &mut i32, returning: bool) {
        let stored = self.units[u].airframe.bank;
        let mut roll = stored.neg();
        if self.units[u].is_gaia()
            && roll.is_zero()
            && self.units[u].spell_time >= 0
            && self.frame & 7 == 0
        {
            return;
        }
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
        if returning {
            // The returning arm: doubled, then the upper clamp.
            want = want.addss(want);
            if !F55.gt(want) {
                want = F55;
            }
        } else {
            let at = self.units[u].pos;
            let low = self.units[u].owner < 8
                && (self.units[u].airframe.z - self.world.tile_z(at.tile())).abs() <= 199;
            if low {
                want = Single::ZERO;
            } else if !F55.gt(want) {
                want = F55;
            }
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
        if returning {
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
        }
        // `(s·3 + (s·3 >> 31 & 3)) >> 2`: three quarters, toward zero —
        // returning only.
        if returning && !roll.eq_value(Single::ZERO) {
            *speed = *speed * 3 / 4;
        }
        let af = &mut self.units[u].airframe;
        af.last_bank = af.bank;
        af.bank = roll.neg();
    }

    /// **The altitude a plane with a point wants over the ground ahead**
    /// (`pitch_aircraft`'s non-returning arm, `0x5e9001`–`0x5e920b`;
    /// `docs/ORDERS.md` §39): its `cruising_alt`, unless it is a type that
    /// strafes (`unit_flags & 0x400000`, flag `w`) on a strike, which
    /// wants **half** of it (`cdq; sub; sar`, toward zero). The strike is
    /// the front order's `is_attack` (vslot `+0x18`: a `StrafeOrder` 1,
    /// an `AirPatrolOrder` 0), whose target stands
    /// (`TargetOrder::target_exists@0072ff10`), is no ally's, does not
    /// fly (`ObjectTypeData +0x218`, the domain, not 2), and lies within
    /// 90° of the heading (`fold(heading − find_angle(dx, dy)) ≤
    /// 0x40000000`, `0x5e9091`–`0x5e90ae`). run265's Fighter on 798:
    /// pitch 40 → 38 with `z` 677 under 800 + 191, where the whole 1600
    /// would have held it at 40.
    ///
    /// SEAM: a **flying** target's arm, which wants the target figure's
    /// own `z` less or plus `min(300, max(0, (0x600 − dist)·5))` by the
    /// order of the two objects' numbers (`0x5e9114`–`0x5e9206`); it is
    /// flown here on the whole `cruising_alt`.
    fn strike_altitude(&self, u: usize, dx: i32, dy: i32, cruise: i32) -> i32 {
        let strafes = self.strafes(crate::combat::Obj::Unit(u));
        let Some(t) = self
            .current_strafe(u)
            .filter(|_| strafes)
            .and_then(|sf| sf.target)
        else {
            return cruise;
        };
        if !self.obj_alive(t) {
            return cruise;
        }
        let whom = self.owner_of(t);
        if whom < crate::world::PLAYER_SLOTS && self.is_ally(self.units[u].owner, whom) {
            return cruise;
        }
        let heading = self.units[u].movement.heading;
        let off = fold((heading.0 as u32).wrapping_sub(find_angle(dx, dy).0 as u32));
        if off > 0x4000_0000 {
            return cruise;
        }
        if matches!(self.profile(t).domain, crate::attrition::Domain::Air) {
            return cruise;
        }
        cruise / 2
    }

    /// **`Unit::pitch_aircraft(dx, dy, z, &speed)@005e8de0` for a plane**
    /// (`docs/ORDERS.md` §33.4): the altitude it wants, the pitch that
    /// climbs or dives toward it two a frame, the step of altitude the
    /// pitch makes, and the two speed cuts.
    ///
    /// **Not returning** (§34.6): the altitude wanted is `cruising_alt`
    /// over the ground ahead — half of it for a strafing type on a strike
    /// ([`Sim::strike_altitude`], §39); there is no halving of the speed; the floor is the
    /// ground here plus 200; `extra` is 0; and the rate's divisor is
    /// `0x240 / speed` — the listing's `[ebp−0x18]` keeps the `0x240`
    /// `project` was handed, because only the returning arm overwrites it
    /// with the distance (`0x5e8fae`).
    ///
    /// SEAM: the `0x400000` type arm's flying target
    /// ([`Sim::strike_altitude`]).
    pub(crate) fn pitch_plane(
        &mut self,
        u: usize,
        dx: i32,
        dy: i32,
        aim_z: i32,
        speed: &mut i32,
        returning: bool,
    ) {
        // **An animal** — a flock's bird (`crate::flock`) — skips the
        // whole call with its pitch level and its `spell_time` not
        // negative on `(frame + 3) & 7 == 0` (`005e8de0`, the opening
        // test), and inside it takes three arms of its own: no cap at
        // `cruising_alt` over the ground ahead, a pitch rate of ±10 by the
        // height owed rather than the plane's glide, and no pitch speed
        // cut. The returning arm's halving inside `0x600` is not behind
        // the test and is taken.
        let animal = self.units[u].is_gaia();
        if animal
            && self.units[u].airframe.pitch.is_zero()
            && self.units[u].spell_time >= 0
            && (self.frame + 3) & 7 == 0
        {
            return;
        }
        let xs = self.world.width() * UNITS_PER_CELL;
        let ys = self.world.height() * UNITS_PER_CELL;
        let at = self.units[u].pos;
        let heading = self.units[u].movement.heading;
        // The ground `0x240` ahead, the point clamped into the world.
        let ax = (at.x + sin_component(heading, 0x240)).clamp(0, xs - 1);
        let ay = (at.y - cos_component(heading, 0x240)).clamp(0, ys - 1);
        let ahead = self.world.tile_z(Pos::new(ax, ay).tile());
        let cruise = self.current_air(u).map_or(CRUISING_ALT, |a| a.cruising_alt);
        // The returning arm (`air +0x18`). `local_2c`, the home carrier's
        // own speed test, is never set for a building.
        let (want, dist) = if returning {
            let dist = vector_dist(dx, dy);
            let want = (ahead + 100).max(aim_z + (dist / *speed) * 25);
            if dist < 0x600 {
                *speed /= 2;
            }
            (want, dist)
        } else {
            (self.strike_altitude(u, dx, dy, cruise) + ahead, 0x240)
        };
        let want = if animal {
            want
        } else {
            want.min(cruise + ahead)
        };
        let ground = self.world.tile_z(at.tile());
        let target = want.max(ground + if returning { 50 } else { 200 });
        let mut pitch = self.units[u].airframe.pitch;
        let extra = if returning && target < ground + 500 {
            F20_NEG
        } else {
            Single::ZERO
        };
        let z = self.units[u].airframe.z;
        let rate = if animal {
            match target - z {
                d if d >= 0xc9 => 10,
                d if d < -200 => -10,
                _ => 0,
            }
        } else {
            let n = (dist / *speed).max(1);
            ((target - z) / 25 * 20) / n
        };
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
        if mag.gt(F20) && !animal {
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

/// The launch line (item 836, `docs/ORDERS.md` §38): chapter seventeen's
/// cast on a world large enough to hold the base's point.
#[cfg(test)]
mod launch_tests {
    use crate::attrition::Domain;
    use crate::combat::{self, Obj};
    use crate::group::{Flight, Group};
    use crate::movement::Angle;
    use crate::orders::{Body, flag};
    use crate::world::Pos;
    use crate::{Player, Sim, Unit, UnitType};

    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(128, 128),
            2,
        );
        s.nation[0].human = true;
        s.nation[1].human = false;
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        s.frame = 766;
        s
    }

    /// who=0's Airbase at run223's point, and who=1's Barracks, finished
    /// and seen by who=0 (`ever_seen` 3 from 762).
    fn base_and_target(s: &mut Sim) -> (usize, usize) {
        let airbase = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::Airbase,
            ..crate::build::BuildType::default()
        });
        let base = s.add_building(0, Pos::new(11616, 13920), 0);
        s.buildings[base].ty = Some(airbase);
        let target = s.add_building(1, Pos::new(21120, 16512), 0);
        s.buildings[target].started = true;
        s.buildings[target].active = true;
        s.buildings[target].combat = Some(combat::Profile::default());
        s.buildings[target].health = 1200;
        s.buildings[target].ever_seen = 3;
        (base, target)
    }

    /// A Fighter with the `FIGHTER` row's tank, 400, standing inside `base`.
    fn fighter_inside(s: &mut Sim, base: usize, mana: i32) -> usize {
        let t = UnitType {
            hits: 100,
            moves: 75,
            mana,
            turn_speed: crate::movement::degrees_to_angle(10).0,
            kind: crate::attrition::UnitKind {
                domain: Domain::Air,
                ..crate::attrition::UnitKind::default()
            },
            combat: combat::Profile {
                attack: 15,
                uber_size: 1,
                max_range: 7 * 192,
                domain: Domain::Air,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        let ty = s.add_unit_type(t);
        let index = i16::try_from(s.units.len()).unwrap();
        let mut u = Unit::new(0, index, Pos::new(11424, 14005), 100);
        u.ty = Some(ty);
        u.on_map = true;
        let u = s.add_unit(u);
        s.units[u].kind = s.unit_types[ty].kind;
        s.units[u].movement.speed = 75;
        s.units[u].movement.turning = crate::turning_of(&s.unit_types[ty]);
        s.init_guys(u, Some(ty));
        s.go_inside(u, base);
        s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
        u
    }

    /// **A base on ground under 0 is aimed at from 0** (item 1009,
    /// `docs/GOLDEN.md` §43): the approach home reads the base's
    /// `z_internal`, which `SubObject::init` takes from `find_tcoord_z`
    /// with its fourth argument 1 — a negative height answers 0. run362's
    /// `0/2008` stands on a tile of −42 and prints `z_internal` 0; `0/7`'s
    /// descent to it parts on 2399 without the clamp.
    #[test]
    fn the_approach_home_reads_a_base_on_low_ground_as_height_zero() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.units[u].pos = Pos::new(8936, 16261);
        let t = s.buildings[base].pos.tile();
        s.world.set_tile_z(t, 0);
        let level = s.home_approach(u, base).z;
        s.world.set_tile_z(t, -42);
        assert_eq!(s.home_approach(u, base).z, level, "clamped at 0");
        s.world.set_tile_z(t, 157);
        assert_eq!(
            s.home_approach(u, base).z,
            level + 157,
            "above 0 it is read"
        );
    }

    fn group_of(who: Player, list: &[usize]) -> Group {
        Group {
            who,
            army: None,
            pushed: None,
            list: list.to_vec(),
        }
    }

    /// **The tank** (`Unit::process@00610bc0`'s air arm): one a frame on
    /// the map while any is left, to the type's `MANA` and no further;
    /// inside, `AIR_UNIT_MANA_RECHARGE` a frame, to 0. run265: 1 on 779,
    /// 400 on 1178; −2 a block inside. Made to fail with the refill at 1
    /// (the third assertion) and with the burn unbounded (the second).
    #[test]
    fn a_plane_burns_one_a_frame_on_the_map_and_refills_two_inside() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.units[u].mana_burn = 0;
        s.recover_mana(u, 779);
        assert_eq!(s.units[u].mana_burn, 1, "one a frame on the map");
        s.units[u].mana_burn = 399;
        s.recover_mana(u, 1178);
        s.recover_mana(u, 1179);
        assert_eq!(
            s.units[u].mana_burn, 400,
            "the tank's own size, and no more"
        );
        s.go_inside(u, base);
        s.units[u].mana_burn = 24;
        s.recover_mana(u, 767);
        assert_eq!(s.units[u].mana_burn, 22, "two a frame inside");
        s.units[u].mana_burn = 1;
        s.recover_mana(u, 768);
        assert_eq!(s.units[u].mana_burn, 0, "and never below 0");
    }

    /// **A strike from inside a base** (`Group::action_flight@006fb260`'s
    /// inside arm, run265's 768): one `StrafeOrder` on the target, home
    /// the base, `mandatory`, the action bit, `returning` 0 — and the
    /// plane still inside. Made to fail with the reach against `mana`
    /// alone (no order: 9857 > 400).
    #[test]
    fn a_strike_on_a_plane_inside_its_base_is_a_strafe_home_to_it() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        let o = s.units[u].orders.front().copied().expect("a strike");
        assert_eq!(o.flags & flag::ACTION, flag::ACTION);
        let Body::Strafe(sf) = o.body else {
            panic!("a strafe")
        };
        assert_eq!(sf.target, Some(Obj::Building(target)));
        assert_eq!(sf.home, Some(base));
        assert!(sf.mandatory && !sf.returning);
        assert_eq!(sf.at, Some(Pos::new(21120, 16512)));
        assert_eq!(s.units[u].inside, Some(base), "it waits in the base");
    }

    /// **Out of reach, no strike**: `vector_dist(base − target)` 9857
    /// against a tank of 100 at 75 a frame. Made to fail by dropping the
    /// reach test.
    #[test]
    fn a_strike_beyond_the_tank_s_reach_lays_nothing() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 100);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        assert!(s.units[u].orders.is_empty());
    }

    /// **`land_plane` under a repeating base** (`Unit::land_plane
    /// @005e9950`'s `005e9a43`, `docs/ORDERS.md` §40): with the home's
    /// `build_masks & 0x80` set the plane goes inside with its patrol on
    /// the stack and the action bit cleared — run265's Fighter on 1385 and
    /// Bomber on 1489 — and with it clear the stack is closed. Made to fail
    /// with the keep unconditional (the second arm keeps an order) and
    /// with the flag left alone (the first arm's bit).
    #[test]
    fn a_plane_landing_at_a_repeating_base_keeps_its_order_unflagged() {
        for repeat in [true, false] {
            let mut s = sim();
            let (base, _) = base_and_target(&mut s);
            s.buildings[base].repeat_air = repeat;
            let u = fighter_inside(&mut s, base, 400);
            s.come_out(u);
            s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
            s.set_returning(u, true);
            s.land_plane(u);
            assert_eq!(s.units[u].inside, Some(base), "{repeat}: inside");
            if repeat {
                assert_eq!(s.units[u].orders.len(), 1, "the patrol stays");
                let o = s.units[u].orders.front().copied().unwrap();
                assert_eq!(o.flags & flag::ACTION, 0, "its action bit cleared");
                let Body::AirPatrol(p) = o.body else {
                    panic!("a patrol")
                };
                assert!(!p.returning, "and `returning` cleared");
            } else {
                assert!(s.units[u].orders.is_empty(), "no repeat: closed");
            }
        }
    }

    /// **A computer's base sends its plane over a city under attack**
    /// (`Object::do_launch@0064f3b0`'s sortie, item 1182, `docs/GOLDEN.md`
    /// §50): on `(frame + id) % 32 == 0` a plane inside a computer's base
    /// is cleared and patrolled over an enemy city with `city_flags & 3 ==
    /// 3`; nothing for a human's base, off the cadence, or over a city not
    /// under attack. run437's Biplane on 778. Made to fail with the city's
    /// attack bit not read, and with the owner's human bit not read.
    #[test]
    fn a_computer_s_base_sends_its_plane_over_a_city_under_attack() {
        for (human, under_attack, on_cadence, sorties) in [
            (false, true, true, true),
            (true, true, true, false),
            (false, false, true, false),
            (false, true, false, false),
        ] {
            let mut s = sim();
            s.nation[0].human = human;
            let (base, target) = base_and_target(&mut s);
            let u = fighter_inside(&mut s, base, 400);
            s.units[u].orders.clear();
            let pos = s.buildings[target].pos;
            s.cities.push(crate::city::City {
                alive: true,
                owner: 1,
                race: Some(1),
                founder: 1,
                building: target,
                members: Vec::new(),
                reg: None,
                pos,
                capital: true,
                founding_capital: true,
                was_founding_capital: false,
                unassimilated: false,
                no_heal: under_attack,
                attacking: under_attack,
                ever_attacked: under_attack,
                alarm: false,
                no_muster: false,
                was_capital: 0,
                capture_stamp: 0,
                assimilation_timer: 0,
                attack_stamp: 0,
                reduce_stamp: 0,
                capture_strength: 0,
                pop: 1,
                has_citizen: false,
                source: None,
                trade_val: 0,
                traded_with: [0; 8],
            });
            let id = i64::from(s.buildings[base].index);
            s.frame = 32 * 30 - id + if on_cadence { 0 } else { 1 };
            s.do_launch(base);
            let patrol = s.units[u].orders.front().and_then(|o| match o.body {
                crate::orders::Body::AirPatrol(p) => Some(p.current()),
                _ => None,
            });
            if sorties {
                assert_eq!(patrol, Some(pos), "no sortie over the city");
            } else {
                assert_eq!(
                    patrol, None,
                    "human {human}, attacked {under_attack}: a sortie"
                );
            }
        }
    }

    /// **`do_launch` keeps an unflagged order only under a repeating
    /// base** (`Object::do_launch@0064f3b0`, `has_repeat_air() || flags &
    /// 4`; `docs/GOLDEN.md` §32): at a full tank the kept patrol is
    /// launched under the bit and killed without it, the plane staying
    /// inside. Made to fail with the kill unconditional (the first arm)
    /// and with the bit read as always set (the second).
    #[test]
    fn a_full_tank_launches_an_unflagged_patrol_only_under_the_bit() {
        for repeat in [true, false] {
            let mut s = sim();
            let (base, _) = base_and_target(&mut s);
            let u = fighter_inside(&mut s, base, 400);
            s.come_out(u);
            s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
            s.buildings[base].repeat_air = true;
            s.land_plane(u);
            s.buildings[base].repeat_air = repeat;
            s.units[u].mana_burn = 0;
            s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
            s.do_launch(base);
            if repeat {
                assert_eq!(s.units[u].inside, None, "launched");
                assert_eq!(s.units[u].orders.len(), 1, "on its patrol");
            } else {
                assert_eq!(s.units[u].inside, Some(base), "still inside");
                assert!(s.units[u].orders.is_empty(), "the patrol killed");
            }
        }
    }

    /// **The walk ends at a launch** (`Object::do_launch@0064f3b0`,
    /// `64f806`; `docs/GOLDEN.md` §38, item 915): two planes full under
    /// the bit, and one call launches the first and never reaches the
    /// second — the next link is the launched plane's own, which
    /// `remove_from_inside` has cleared. Under the emulator, with the
    /// link left, the second joins `launching`. Made to fail with the
    /// walk carried on after the launch.
    #[test]
    fn a_launch_ends_the_walk_along_the_base_s_chain() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        s.buildings[base].repeat_air = true;
        let mut planes = Vec::new();
        for _ in 0..2 {
            let u = fighter_inside(&mut s, base, 400);
            s.come_out(u);
            s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
            s.land_plane(u);
            s.units[u].mana_burn = 0;
            planes.push(u);
        }
        s.buildings[base].launch_frames = super::FRAMES_BETWEEN_LAUNCHES;
        s.do_launch(base);
        assert_eq!(s.units[planes[0]].inside, None, "the first launched");
        assert_eq!(s.units[planes[1]].inside, Some(base), "the second inside");
        assert!(
            !s.buildings[base].launching.contains(&planes[1]),
            "and never walked: not in `launching`"
        );
        assert_eq!(s.units[planes[1]].orders.len(), 1, "its patrol kept");
    }

    /// **An aircraft trained at an Airbase stays in it** (`Build::train
    /// @0062f9b0`'s `CARRY_AIR` arm, `62fac0`; `docs/GOLDEN.md` §38, item
    /// 915): no `come_out`, no order, in the base's chain; the same unit
    /// trained at a building without the mask comes out. Made to fail with
    /// the arm dropped (the plane on the map).
    #[test]
    fn an_aircraft_trained_at_an_airbase_stays_inside_with_no_order() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let first = fighter_inside(&mut s, base, 300);
        let ty = s.units[first].ty.unwrap();
        let trained = s.build_train(base, ty).unit;
        assert_eq!(s.units[trained].inside, Some(base), "inside the Airbase");
        assert!(s.units[trained].orders.is_empty(), "with no order");
        assert_eq!(
            s.buildings[base].garrison.last(),
            Some(&trained),
            "at the chain's tail"
        );
        s.units[trained].mana_burn = 0;
        s.do_launch(base);
        assert_eq!(s.units[trained].inside, Some(base), "passed over");
        let barracks = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::Barracks,
            ..crate::build::BuildType::default()
        });
        let b = s.add_building(0, Pos::new(5000, 5000), 0);
        s.buildings[b].ty = Some(barracks);
        let out = s.build_train(b, ty).unit;
        assert_eq!(s.units[out].inside, None, "a Barracks lets it out");
    }

    /// The front patrol of `u`, which the tests below expect.
    fn patrol_of(s: &Sim, u: usize) -> (crate::orders::AirPatrolOrder, u8) {
        match s.units[u].orders.front() {
            Some(o) => match o.body {
                Body::AirPatrol(p) => (p, o.flags),
                ref b => panic!("0/{}: not a patrol: {b:?}", s.units[u].index),
            },
            None => panic!("0/{}: no order", s.units[u].index),
        }
    }

    /// **An Airbase's gather point re-orders every plane homed there, on
    /// every press** (item 947, `Build::add_gather_point@00622e70`'s hangar
    /// loop; `docs/GOLDEN.md` §41). `QUEUE_NEW` sends the flying plane home
    /// and empties the one inside first, and then each takes a patrol over
    /// the list with the action bit; the flying one's heights are the strafe
    /// home's. `QUEUE_LAST` rebuilds each from the whole list and carries a
    /// patrol's `cruising_alt` and `sharp_turn`. Made to fail with the
    /// hangar loop dropped (the first press), with the heights not carried
    /// (the second), and with the later points not appended.
    #[test]
    fn an_airbase_s_gather_point_re_orders_every_plane_homed_there() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let inside = fighter_inside(&mut s, base, 400);
        let flying = fighter_inside(&mut s, base, 400);
        s.come_out(flying);
        s.add_air_patrol_order(inside, Pos::new(21120, 16512), Some(base), false);
        s.add_air_patrol_order(flying, Pos::new(21120, 16512), Some(base), false);
        let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
        let point = |pos| crate::rally::GatherPoint { pos, action: 0 };
        s.add_gather_point(base, point(p1), true);
        for u in [inside, flying] {
            let (p, flags) = patrol_of(&s, u);
            assert_eq!(p.live(), [p1], "0/{}: one patrol over P1", s.units[u].index);
            assert_eq!(flags & flag::ACTION, flag::ACTION, "the action bit");
            assert_eq!((p.cruising_alt, p.sharp_turn), (0x640, 0));
            assert_eq!(s.units[u].orders.len(), 1);
        }
        if let Some(Body::AirPatrol(p)) = s.units[flying].orders.front_mut().map(|o| &mut o.body) {
            p.cruising_alt = 1800;
            p.sharp_turn = -1;
        }
        s.add_gather_point(base, point(p2), false);
        let (p, _) = patrol_of(&s, flying);
        assert_eq!(p.live(), [p1, p2], "the list whole");
        assert_eq!(
            (p.waypoint, p.cruising_alt, p.sharp_turn),
            (0, 1800, -1),
            "carried"
        );
        let (p, _) = patrol_of(&s, inside);
        assert_eq!((p.live(), p.cruising_alt), (&[p1, p2][..], 0x640));
        assert_eq!(s.units[inside].inside, Some(base), "still inside");
    }

    /// **The Clear at an Airbase** (item 947, `Build::clear_gather@
    /// 00623180`'s hangar half): a homed plane on the map is sent home
    /// (`add_strafe_order(−1, −1, base, 0, QUEUE_NEW, 0)`), one inside loses
    /// its orders and leaves `launching`. Made to fail with the half
    /// dropped.
    #[test]
    fn the_clear_at_an_airbase_sends_a_flying_plane_home_and_empties_one_inside() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let inside = fighter_inside(&mut s, base, 400);
        let flying = fighter_inside(&mut s, base, 400);
        s.come_out(flying);
        // A plane on the map is homed by its air order's home.
        s.add_air_patrol_order(flying, Pos::new(21120, 16512), Some(base), false);
        let p1 = crate::rally::GatherPoint {
            pos: Pos::new(11520, 7680),
            action: 0,
        };
        s.add_gather_point(base, p1, true);
        s.buildings[base].launching.push(inside);
        s.clear_gather(base);
        assert!(s.buildings[base].gather.is_empty());
        assert!(s.units[inside].orders.is_empty(), "no order inside");
        assert!(
            !s.buildings[base].launching.contains(&inside),
            "out of launching"
        );
        let o = s.units[flying].orders.front().copied().expect("an order");
        let Body::Strafe(sf) = o.body else {
            panic!("a strafe home: {:?}", o.body)
        };
        assert_eq!(
            (sf.target, sf.home, sf.returning, sf.mandatory, o.flags),
            (None, Some(base), true, false, 0)
        );
        assert_eq!(s.units[flying].orders.len(), 1);
    }

    /// **A plane trained under an Airbase's list takes a patrol over it**
    /// (item 947, `Build::train@0062f9b0`'s `CARRY_AIR` arm, `62fadf`..
    /// `62fbfb`): the first point with the action bit, the rest appended,
    /// and it stays inside. Made to fail with the arm dropped.
    #[test]
    fn a_plane_trained_under_an_airbase_s_list_takes_a_patrol_over_it() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let first = fighter_inside(&mut s, base, 300);
        let ty = s.units[first].ty.unwrap();
        let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
        s.buildings[base].gather = vec![
            crate::rally::GatherPoint { pos: p1, action: 0 },
            crate::rally::GatherPoint { pos: p2, action: 0 },
        ];
        let trained = s.build_train(base, ty).unit;
        assert_eq!(s.units[trained].inside, Some(base), "inside");
        let (p, flags) = patrol_of(&s, trained);
        assert_eq!(p.live(), [p1, p2]);
        assert_eq!(
            (flags & flag::ACTION, p.home, p.waypoint),
            (flag::ACTION, Some(base), 0)
        );
    }

    /// **The patrol walks its points** (item 947, `Unit::do_air_patrol@
    /// 005ea620`): within `0x240` of `points[waypoint]` the waypoint steps
    /// on; alone at the last it stays there. Made to fail with the step
    /// dropped.
    #[test]
    fn a_patrol_steps_its_waypoint_on_at_each_point_and_stays_on_the_last() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        let (p1, p2) = (Pos::new(11520, 7680), Pos::new(5760, 5760));
        s.add_air_patrol_order(u, p1, Some(base), true);
        if let Some(Body::AirPatrol(p)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
            p.push(p2);
        }
        s.units[u].pos = p1;
        let frame = s.frame;
        s.do_air_patrol(u, frame);
        assert_eq!(patrol_of(&s, u).0.waypoint, 1, "on to P2");
        s.units[u].pos = p2;
        s.do_air_patrol(u, frame + 1);
        assert_eq!(patrol_of(&s, u).0.waypoint, 1, "the last kept");
        assert_eq!(s.units[u].orders.len(), 1, "alone, not killed");
    }

    /// **A launch sets the patrol's `waypoint` to 0** (item 947,
    /// `Object::do_launch@0064f3b0`'s store at `64f6a0`, `+0x3c`, the PDB's
    /// `PatrolOrder::waypoint`). Made to fail with the store dropped.
    #[test]
    fn a_launch_starts_a_patrol_at_its_first_point() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.add_air_patrol_order(u, Pos::new(11520, 7680), Some(base), true);
        if let Some(Body::AirPatrol(p)) = s.units[u].orders.front_mut().map(|o| &mut o.body) {
            p.push(Pos::new(5760, 5760));
            p.waypoint = 1;
        }
        s.units[u].mana_burn = 0;
        s.do_launch(base);
        assert_eq!(s.units[u].inside, None, "launched");
        assert_eq!(patrol_of(&s, u).0.waypoint, 0);
    }

    /// **The repeat button toggles off the first member's state**
    /// (`Group::action_buildmask@006fc9a0`, `docs/GOLDEN.md` §32): one base
    /// goes 1 → 0 → 1; of two, the first's state decides and a clear
    /// clears everything after it; a building that cannot carry aircraft
    /// and any mask but `0x80` are passed over. Made to fail with the bit
    /// set whatever it held (the toggle), and with `all_set` never cleared
    /// (the second pair).
    #[test]
    fn the_repeat_button_toggles_off_the_first_member() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        // `Build::init`'s `|= 0x88`, which `add_building` does not run.
        s.buildings[base].repeat_air = true;
        assert_eq!(s.action_buildmask(&[base], 0x80), 1);
        assert!(!s.buildings[base].repeat_air, "4232 -> 4104");
        assert_eq!(s.action_buildmask(&[base], 0x80), 1);
        assert!(s.buildings[base].repeat_air, "4104 -> 4232");
        assert_eq!(s.action_buildmask(&[base], 0x40), 0, "not carried");
        assert_eq!(s.action_buildmask(&[target], 0x80), 0, "no hangar");
        let second = s.add_building(0, Pos::new(13824, 13920), 0);
        s.buildings[second].ty = s.buildings[base].ty;
        for (a, b, want) in [
            (true, false, (false, false)),
            (false, true, (true, false)),
            (false, false, (true, true)),
            (true, true, (false, false)),
        ] {
            s.buildings[base].repeat_air = a;
            s.buildings[second].repeat_air = b;
            s.action_buildmask(&[base, second], 0x80);
            assert_eq!(
                (s.buildings[base].repeat_air, s.buildings[second].repeat_air),
                want,
                "from ({a}, {b})"
            );
        }
    }

    /// **A strafe with no target goes, and the bit spares what is behind
    /// it** (`005e9a43`'s `else if (!bVar2)`): under a repeating base the
    /// killed strafe's successor keeps its action bit. Made to fail with
    /// the flag cleared after the kill.
    #[test]
    fn a_killed_strafe_home_leaves_the_order_behind_it_flagged() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        s.buildings[base].repeat_air = true;
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        s.add_air_patrol_order(u, Pos::new(21120, 16512), Some(base), true);
        s.add_strafe_order(
            u,
            None,
            Some(base),
            false,
            crate::orders::QueuePos::First,
            true,
        );
        assert_eq!(s.units[u].orders.len(), 2);
        s.land_plane(u);
        assert_eq!(s.units[u].orders.len(), 1, "the strafe went");
        let o = s.units[u].orders.front().copied().unwrap();
        assert!(matches!(o.body, Body::AirPatrol(_)));
        assert_eq!(o.flags & flag::ACTION, flag::ACTION, "untouched");
    }

    /// **The tank gates the launch, and the launch is the EXIT's**
    /// (`Object::do_launch@0064f3b0`, `do_spec_anim@005e5880`; run265's
    /// 768–778). While `mana_burn` is above 0 the plane stays inside with
    /// its strike; on the call it reads 0 it comes out onto the base's
    /// point less `0xc0`, turned to 0, its figure on the ground with
    /// `last_bank`/`last_pitch` 0, and flies its first step north; the
    /// base's `launch_frames` is 0 and `launching` empty. Made to fail
    /// without the tank's test (out on the first call) and without the
    /// EXIT (the ring's spot, south of the base).
    #[test]
    fn a_strike_from_inside_waits_for_the_tank_and_leaves_on_the_exit() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.units[u].mana_burn = 22;
        s.units[u].airframe.pitch = crate::single::Single::from_i32(8);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        for f in 768..778 {
            s.frame = f;
            s.recover_mana(u, f);
            s.do_launch(base);
            assert_eq!(s.units[u].inside, Some(base), "{f}: the tank is not full");
            assert_eq!(
                s.buildings[base].launch_frames,
                super::FRAMES_BETWEEN_LAUNCHES
            );
        }
        s.frame = 778;
        s.recover_mana(u, 778);
        assert_eq!(s.units[u].mana_burn, 0);
        s.do_launch(base);
        assert_eq!(s.units[u].inside, None, "out on the tank's first 0");
        assert!(s.units[u].on_map);
        assert_eq!(s.buildings[base].launch_frames, 0);
        assert!(s.buildings[base].launching.is_empty());
        let at = s.units[u].pos;
        assert_eq!(at.x, 11424, "the base's point less 0xc0");
        assert!(at.y < 13920 && 13920 - at.y <= 75, "one step north: {at:?}");
        let af = s.units[u].airframe;
        assert_eq!(af.last_bank.bits(), 0);
        assert_eq!(af.last_pitch.bits(), 0, "the pitch moved and zeroed");
        assert!(matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::Strafe(_))
        ));
        let _ = Angle(0);
    }

    /// A Fighter out of `base` on a strike at `target`, its type flagged
    /// `w` when `strafes`, heading straight at the Barracks.
    fn striking_fighter(s: &mut Sim, strafes: bool) -> (usize, usize, i32, i32) {
        let (base, target) = base_and_target(s);
        let u = fighter_inside(s, base, 400);
        if strafes {
            let ty = s.units[u].ty.unwrap();
            s.unit_types[ty].cols.unit_flags |= crate::ai_load::uflags::STRAFES;
        }
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        s.come_out(u);
        let (at, to) = (s.units[u].pos, s.buildings[target].pos);
        let (dx, dy) = (to.x - at.x, to.y - at.y);
        s.units[u].movement.heading = crate::movement::find_angle(dx, dy);
        (u, target, dx, dy)
    }

    /// **A strafing type on a strike wants half its cruising altitude**
    /// (`pitch_aircraft`, `0x5e9001`–`0x5e920b`, `docs/ORDERS.md` §39),
    /// and every predicate of the arm gives the whole back: the flag, the
    /// target standing, the 90° off the nose. Made to fail with the flag
    /// test dropped (the second assertion) and with the angle's bound at
    /// 180° (the fourth).
    #[test]
    fn a_strafing_type_on_a_strike_wants_half_its_cruising_altitude() {
        let mut s = sim();
        let (u, target, dx, dy) = striking_fighter(&mut s, true);
        assert_eq!(s.strike_altitude(u, dx, dy, 1600), 800, "half, on a strike");
        let mut plain = sim();
        let (v, _, vdx, vdy) = striking_fighter(&mut plain, false);
        assert_eq!(
            plain.strike_altitude(v, vdx, vdy, 1600),
            1600,
            "no w, no half"
        );
        let h = s.units[u].movement.heading;
        s.units[u].movement.heading = Angle(h.0.wrapping_add(0x4000_0000));
        assert_eq!(s.strike_altitude(u, dx, dy, 1600), 800, "90° off is inside");
        s.units[u].movement.heading = Angle(h.0.wrapping_add(0x4000_0000 + 0x100_0000));
        assert_eq!(
            s.strike_altitude(u, dx, dy, 1600),
            1600,
            "past 90°, the whole"
        );
        s.units[u].movement.heading = h;
        s.buildings[target].alive = false;
        assert_eq!(
            s.strike_altitude(u, dx, dy, 1300),
            1300,
            "no target, the whole"
        );
    }

    /// **run265's 798**: the Fighter at pitch 40 and `z` 677 comes down
    /// two, where the whole 1600 over the ground held it at 40 (40.0 here
    /// against 38.0 there, before item 842). Flat ground at 0: the target
    /// is 800, `((800 − 677) / 25 · 20) / (0x240 / 75)` = 11, 29 under
    /// the pitch. Made to fail with `strike_altitude` returning the whole.
    #[test]
    fn run265_s_climb_comes_down_at_the_strafers_half() {
        let mut s = sim();
        let (u, _, dx, dy) = striking_fighter(&mut s, true);
        s.units[u].airframe.pitch = crate::single::Single::from_i32(40);
        s.units[u].airframe.z = 677;
        let mut speed = 75;
        s.pitch_plane(u, dx, dy, 0, &mut speed, false);
        assert_eq!(s.units[u].airframe.pitch.to_i32(), 38);
        assert_eq!(s.units[u].airframe.last_pitch.to_i32(), 40);
        let mut plain = sim();
        let (v, _, vdx, vdy) = striking_fighter(&mut plain, false);
        plain.units[v].airframe.pitch = crate::single::Single::from_i32(40);
        plain.units[v].airframe.z = 677;
        let mut speed = 75;
        plain.pitch_plane(v, vdx, vdy, 0, &mut speed, false);
        assert_eq!(plain.units[v].airframe.pitch.to_i32(), 40, "no w, it holds");
    }

    /// **A plane over a wood holds its fire** (`is_in_range@006486b0`'s
    /// world-cell test, `6486b0`..`64875b`, item 1200, `docs/GOLDEN.md`
    /// §50): the tile under the point asked from answers no when its
    /// surface is forest, and the test asks no domain. Chapter forty-one's
    /// Biplane on 839, over `TData` `0x7138`. A wood under the target
    /// alone does not. Made to fail with the test dropped.
    #[test]
    fn a_plane_over_a_wood_holds_its_fire() {
        use crate::world::tile;
        let mut s = sim();
        let (u, target, _, _) = striking_fighter(&mut s, true);
        let to = s.buildings[target].pos;
        s.units[u].pos = Pos::new(to.x - 600, to.y);
        let me = Obj::Unit(u);
        assert!(s.is_in_range(me, Obj::Building(target)), "in reach");
        s.world
            .set_tile_field(to.tile(), tile::SURFACE, tile::SURFACE_FOREST);
        assert!(
            s.is_in_range(me, Obj::Building(target)),
            "a wood under the target"
        );
        let under = s.units[u].pos.tile();
        s.world
            .set_tile_field(under, tile::SURFACE, tile::SURFACE_FOREST);
        assert!(
            !s.is_in_range(me, Obj::Building(target)),
            "a wood under the plane"
        );
    }

    /// **A dry strike with a patrol behind it dies in the tail** (`do_strafe`
    /// `5eb642`..`5eb673`, item 1200): the tank at `mana_burn` = `MANA`
    /// turns the strike for home (`check_fuel`'s arm, in the flight), and
    /// the tail kills a dry strike and one flying home when an order stands
    /// behind it, so the patrol flies alone — chapter forty-one's Biplane
    /// on 1108. Made to fail with the tail's early return for a strike
    /// flying home back and its dry test dropped.
    #[test]
    fn a_dry_strike_with_a_patrol_behind_it_dies_in_the_tail() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.come_out(u);
        let point = s.buildings[target].pos;
        s.add_air_patrol_order(u, point, Some(base), false);
        s.add_strafe_order(
            u,
            Some(Obj::Building(target)),
            Some(base),
            false,
            crate::orders::QueuePos::First,
            false,
        );
        assert_eq!(s.units[u].orders.len(), 2);
        assert!(matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::Strafe(_))
        ));
        s.units[u].mana_burn = 400;
        s.work(u, 767);
        assert_eq!(s.units[u].orders.len(), 1, "the strike is killed");
        assert!(matches!(
            s.units[u].orders.front().map(|o| o.body),
            Some(Body::AirPatrol(_))
        ));
    }

    /// **A strafer's round never rolls** (`Ammo::init`, `67c548`..`67c557`,
    /// item 1200): the unit-strafer arm jumps past `67c633`, where flag `4`
    /// and the `0x4b` over a land unit are set, so its round at a land
    /// unit lands where it lands — chapter forty-one's Biplane on 1529,
    /// short of `0/9`. A type without `w` rolls. Made to fail with the
    /// strafer test dropped from `rolling`.
    #[test]
    fn a_strafers_round_at_a_land_unit_does_not_roll() {
        for strafes in [true, false] {
            let mut s = sim();
            let (u, _, _, _) = striking_fighter(&mut s, strafes);
            let foot = s.add_unit_type(UnitType {
                hits: 100,
                combat: combat::Profile {
                    domain: Domain::Land,
                    ..combat::Profile::default()
                },
                ..UnitType::default()
            });
            let index = i16::try_from(s.units.len()).unwrap();
            let mut t = Unit::new(1, index, Pos::new(21120, 16800), 100);
            t.ty = Some(foot);
            t.on_map = true;
            t.kind = s.unit_types[foot].kind;
            let t = s.add_unit(t);
            let at = s.units[u].pos;
            s.fire_ammo_pub(
                Obj::Unit(u),
                Obj::Unit(t),
                Angle(0),
                1529,
                at,
                1231,
                0,
                false,
            );
            let p = s.projectiles.last().unwrap();
            assert_eq!(p.rolling, !strafes, "strafes {strafes}");
        }
    }

    /// **A strafer's round is exact** (`Ammo::init@0067bbf0`, `0x67c33a`):
    /// no scatter, so neither of the two draws a land shot spends. run265's
    /// 923: the original's Fighter plays its attack and spends nothing
    /// else. Made to fail with `exact` false at the call.
    #[test]
    fn a_strafers_round_spends_no_draw() {
        let mut s = sim();
        let (u, target, _, _) = striking_fighter(&mut s, true);
        let ty = s.units[u].ty.unwrap();
        s.unit_types[ty].combat.to_hit = 50;
        s.units[u].kind = s.unit_types[ty].kind;
        let before = s.rng;
        let at = s.units[u].pos;
        let n = s.projectiles.len();
        s.fire_ammo_pub(
            Obj::Unit(u),
            Obj::Building(target),
            Angle(0),
            923,
            at,
            870,
            0,
            false,
        );
        assert_eq!(s.projectiles.len(), n + 1, "a round");
        assert_eq!(s.rng, before, "and no draw");
    }

    /// **A strafer's landing walks with its gun** (`Ammo::init`,
    /// `0x67c9b2`–`0x67cb1a`, item 853): run265's first pair, released on
    /// `cur_time` 1 of 30 at heading 1340473344 on the Barracks at
    /// (21120, 16512), lands on (20816, 16440) from node 0 and
    /// (20853, 16351) from node 1 — `trunc((1/30 − 0.3f)·6·192)` = −307
    /// along the heading, then 48 to either side. Neither a figure out of
    /// its attack nor a type that does not strafe is walked. Made to fail
    /// with the node's sides swapped, and with the walk returning early.
    #[test]
    fn a_strafers_landing_walks_with_its_gun() {
        let fire = |s: &mut Sim, u: usize, target: usize, node: i8| {
            let at = s.units[u].pos;
            s.fire_ammo_pub(
                Obj::Unit(u),
                Obj::Building(target),
                Angle(0),
                923,
                at,
                873,
                node,
                false,
            );
            s.projectiles.last().unwrap().landing
        };
        let mut s = sim();
        let (u, target, _, _) = striking_fighter(&mut s, true);
        s.units[u].movement.heading = Angle(1_340_473_344);
        let g = &mut s.units[u].guys[0];
        (g.anim, g.cur_time, g.end_time) = (crate::anim::ATTACK2, 1, 30);
        assert_eq!(fire(&mut s, u, target, 0), Pos::new(20816, 16440), "node 0");
        assert_eq!(fire(&mut s, u, target, 1), Pos::new(20853, 16351), "node 1");
        let (me, it, at) = (
            Obj::Unit(u),
            Some(Obj::Building(target)),
            Pos::new(21120, 16512),
        );
        s.units[u].guys[0].cur_time = 30;
        assert_eq!(
            s.strafe_walk(me, it, at, 0),
            Pos::new(21847, 16865),
            "806 past, at the swing's end"
        );
        s.units[u].guys[0].anim = crate::anim::WALK;
        assert_eq!(s.strafe_walk(me, it, at, 0), at, "out of its attack");
        let mut plain = sim();
        let (v, target, _, _) = striking_fighter(&mut plain, false);
        plain.units[v].movement.heading = Angle(1_340_473_344);
        let g = &mut plain.units[v].guys[0];
        (g.anim, g.cur_time, g.end_time) = (crate::anim::ATTACK2, 1, 30);
        assert_eq!(
            plain.strafe_walk(Obj::Unit(v), Some(Obj::Building(target)), at, 0),
            at,
            "no w, no walk"
        );
    }

    /// **An empty base counts nothing** (run265: `launch_frames` 0 from
    /// 778 on): the whole of `do_launch` is under `inside_down ≥ 0`. Made
    /// to fail with the counter above the test.
    #[test]
    fn an_empty_hangar_s_launch_counter_stands() {
        let mut s = sim();
        let (base, _) = base_and_target(&mut s);
        for _ in 0..20 {
            s.do_launch(base);
        }
        assert_eq!(s.buildings[base].launch_frames, 0);
    }

    /// **The empty tank turns a strike for home** (`Unit::check_fuel@
    /// 005e9be0`'s first arm; run265's 1178, run223's 1212): `returning`
    /// set on the order flown. Made to fail without the arm.
    #[test]
    fn an_empty_tank_turns_a_strike_for_home() {
        let mut s = sim();
        let (base, target) = base_and_target(&mut s);
        let u = fighter_inside(&mut s, base, 400);
        s.group_action_flight(&group_of(0, &[u]), Obj::Building(target), Flight::Strike);
        s.come_out(u);
        s.units[u].mana_burn = 400;
        s.plane_air_physics(u, Some(Pos::new(21120, 16512)), 1178);
        let Some(Body::Strafe(sf)) = s.units[u].orders.front().map(|o| o.body) else {
            panic!("a strafe")
        };
        assert!(sf.returning);
    }
}

/// `Ammo::init@0067bbf0`'s air arm (`67bef8`..`67c16a`, item 1102,
/// `docs/COMBAT.md` §81): an `ANTI_AIR` shooter's one draw, at a fixed-wing
/// target flying low. The trace names a draw by its return address.
pub const SITE_FLAK_LOW: &str = "Ammo::init+0x432";
/// The same shooter's draw at a target flying high.
pub const SITE_FLAK_HIGH: &str = "Ammo::init+0x463";
/// Any other shooter's first draw at a low target, against the target's own
/// `FLY_LOW`.
pub const SITE_AIR_TARGET_LOW: &str = "Ammo::init+0x49f";
/// Its second, against its own `FLY_LOW`.
pub const SITE_AIR_SHOOTER_LOW: &str = "Ammo::init+0x4dc";
/// Any other shooter's first draw at a high target, against the target's
/// own `FLY_HIGH`.
pub const SITE_AIR_TARGET_HIGH: &str = "Ammo::init+0x50f";
/// Its second, against its own `FLY_HIGH`.
pub const SITE_AIR_SHOOTER_HIGH: &str = "Ammo::init+0x548";

/// `UnitData::is_flying_low@0060a140`'s reach: `vector_dist < 0x900`.
pub const LOW_REACH: i32 = 0x900;

/// `Unit::fight@005fd4d0`'s jam roll (`5fee89`): `GameAccess::rnd(100)`,
/// frameless, so its chain names `Unit::fight`'s own return into
/// `Unit::do_attack` (item 1102, `docs/COMBAT.md` §81).
pub const SITE_JAM_ROLL: &str = "GameAccess::rnd+0x20 < Unit::do_attack";

/// `Ammo::init_crash@0067b800`'s one draw from the game's stream (its
/// return, `67bb05`): the falling plane's `rolling = r % 7 − 3`.
pub const SITE_CRASH_ROLL: &str = "Ammo::init_crash+0x305";

/// `rules.xml`'s `unit_cats` index of `Air`, the `CAT` (`+0x14`) that
/// `Objects::kill_guy@00659410` compares with 8 at `659473`.
pub const CAT_AIR: i32 = 8;

/// `JAM_UNIT_RADAR_PROB`, rules.xml's `50%`: `Constants +0x21c`.
pub const JAM_UNIT_RADAR_PROB: i32 = 50;

/// `TypeIndex` `LOOKOUT`: an `ANTI_AIR` building (`OBJ_MASKS` `Z6`) that
/// `Build::do_attack@006228f0` fires itself, through `Object::fire_ammo`,
/// on its `recharging` countdown (`docs/COMBAT.md` §84).
pub const LOOKOUT: i32 = 0x209;
/// `TypeIndex` `OBSERVATIONPOST`, the Lookout's successor: fired by
/// `do_attack` as the Lookout is.
pub const OBSERVATIONPOST: i32 = 0x20a;
/// `TypeIndex` `AIRDEFENSE`, `RADAR` and `SAM`: the rest of the line, and
/// the three build pieces `GraphicPieces::verify_load@00906550` loads
/// through `init_unit_data` — the same-named `<UNIT>` of
/// `unit_graphics.xml` — rather than `init_build_data`, so their packet
/// has the unit's slots (§84.2).
pub const AIRDEFENSE: i32 = 0x20b;
/// `TypeIndex` `STEALTHBOMBER` (`0x132`): the tech that lets the
/// computer's sortie ([`Sim::computer_sortie`]) over an air defense.
pub const STEALTHBOMBER: crate::tech::TypeId = 0x132;
pub const RADAR: i32 = 0x20c;
pub const SAM: i32 = 0x20d;

/// **The packet an anti-air building's `Wall::inc_time` cycle reads**
/// (item 1112, `docs/COMBAT.md` §84): `get_game_frames(8)` and
/// `get_game_frames(0xc)` of its piece, the `<RELEASEEVENT>`s of slot
/// `0xc` (`CHAR_ATTACK2`) as `(frame, node, harmless)` in file order, and
/// where the round leaves from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WallCycle {
    /// `get_game_frames(8)`: `recharging` winds up to one under it.
    pub wind: i32,
    /// `get_game_frames(0xc)`: the swing, `recharging` −1 down to −it.
    pub swing: i32,
    /// Slot `0xc`'s release events, `rondata::artdata::PieceReleases`'
    /// shape.
    pub releases: Vec<(u32, i8, bool)>,
    /// The launch point's offset from the building's own `x/y/z`, as
    /// measured ([`WALL_LAUNCH`]); `None` leaves from the building itself.
    pub launch: Option<(i32, i32, i32)>,
}

/// **Where an anti-air building's round leaves from, measured** (item
/// 1112, `docs/COMBAT.md` §84.3). `execute_game_events` adds
/// `GraphicPieces::get_position`'s node vector to the package's `x/y/z`,
/// and `Wall::inc_time`'s package carries a fixed `angle` of
/// `0x20000000`, so the vector does not turn with the target: all twenty
/// of run404's Radar rounds leave from (22408, 16376, 235), the building at
/// (22272, 16512, 8) plus this row. The Air Defense Gun's and the SAM's
/// are not measured.
pub const WALL_LAUNCH: &[(i32, (i32, i32, i32))] = &[(RADAR, (136, -136, 227))];

impl Sim {
    /// **`Wall::inc_time@0063fb60` for one leader's buildings**, which
    /// `Objects::inc_time@0065db70` runs after that leader's units and
    /// before the next leader's ([`Sim::guys_inc_time`]), in object order
    /// from 2000. Only its anti-air arm changes the simulation
    /// ([`Sim::wall_anti_air`], `docs/COMBAT.md` §84).
    pub(crate) fn walls_inc_time(&mut self, who: u8) {
        let mut mine: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| self.buildings[b].owner == who && self.buildings[b].alive)
            .collect();
        mine.sort_by_key(|&b| self.buildings[b].index);
        for b in mine {
            self.wall_anti_air(b);
        }
    }

    /// The [`WallCycle`] of a building's type, `None` for one
    /// `Build::do_attack` fires itself.
    pub(crate) fn wall_cycle_of(&self, b: usize) -> Option<&WallCycle> {
        let bt = self.buildings[b].ty?;
        let id = self.build_types.get(bt)?.tree?;
        self.tech_tree.types.get(id)?.wall_cycle.as_ref()
    }

    /// **`Wall::inc_time`'s anti-air arm** (item 1112, `docs/COMBAT.md`
    /// §84.1), from the listing: past `is_active` (an unfinished building
    /// returns above it), `has_objmask(ANTI_AIR)` and the Lookout's and
    /// the Observation Post's own returns, on `BuildData::recharging`.
    ///
    /// - `near_o` (`+0x34`) negative, or the jam bit (`build_masks &
    ///   0x8000`): a negative count goes to `wind − 1`, and a positive one
    ///   counts down to 0 and holds.
    /// - otherwise, from 0 it counts **up** while under `wind − 1`; at the
    ///   top with no target (`attack_ox < 0`) it holds at `wind − 1`; with
    ///   one it goes to 0 and then −1, and down to `−swing`, where the next
    ///   frame is −1 again.
    ///
    /// While the count is negative the arm hands `execute_game_events` a
    /// package with `cur_anim 0xc`, `cur_time = −recharging` and
    /// `last_time` one under it, at the building's target, and a release
    /// event the clock crosses is a round (`Objects::add_ammo`), exactly as
    /// a unit's is — unless the jam bit is set.
    ///
    /// SEAM: the building's `near_o` is not held here; its sign is taken
    /// as the target's. run404 prints both on all 1,157 blocks of the
    /// Radar, and they agree in sign on every one. SEAM: no building here
    /// is ever jammed.
    fn wall_anti_air(&mut self, b: usize) {
        if !self.buildings[b].active {
            return;
        }
        let Some(cycle) = self.wall_cycle_of(b).cloned() else {
            return;
        };
        let target = self.buildings[b].target;
        let near = target.is_some();
        let top = cycle.wind - 1;
        let r = &mut self.buildings[b].recharging;
        if !near {
            if *r < 0 {
                *r = top;
            }
            if *r >= 1 {
                *r -= 1;
            }
        } else if *r >= 0 && *r < top {
            *r += 1;
        } else if target.is_none() {
            *r = top;
        } else {
            if *r >= 0 {
                *r = 0;
            }
            if *r <= -cycle.swing {
                *r = -1;
            } else {
                *r -= 1;
            }
        }
        let r = self.buildings[b].recharging;
        if r >= 0 {
            return;
        }
        // `execute_game_events` refuses a package whose `ox`/`whom` is
        // negative; SEAM, as for a unit's (`guy_release_events`): a target
        // that is no longer active is refused too.
        let Some(t) = target.filter(|&t| self.active(t)) else {
            return;
        };
        let cur = -r;
        let last = if cur == 0 { -1 } else { cur - 1 };
        for &(at, node, harmless) in &cycle.releases {
            let at = i32::try_from(at).unwrap_or(i32::MAX);
            if last < at && at <= cur {
                self.wall_round(b, t, cycle.launch, node, harmless);
            }
        }
    }

    /// One anti-air building's round: `execute_game_events+0x40d` →
    /// `Objects::add_ammo` → `Ammo::init` ([`Sim::fire_ammo_pub`]), from
    /// the building's own `x/y/z` plus the measured node vector
    /// ([`WALL_LAUNCH`]). The package's `num_guys` is the building's 0,
    /// which run404's rounds print.
    fn wall_round(
        &mut self,
        b: usize,
        t: crate::combat::Obj,
        launch: Option<(i32, i32, i32)>,
        node: i8,
        harmless: bool,
    ) {
        let me = crate::combat::Obj::Building(b);
        let at = self.buildings[b].pos;
        let z = self.world.tile_z(at.tile());
        let (dx, dy, dz) = launch.unwrap_or((0, 0, 0));
        let from = Pos::new(at.x + dx, at.y + dy);
        let to = self.pos_of(t);
        let angle = find_angle(to.x - from.x, to.y - from.y);
        let frame = self.frame;
        self.fire_ammo_pub(me, t, angle, frame, from, z + dz, node, harmless);
        for p in self.projectiles.iter_mut() {
            if p.shooter == me && p.cur_time == 0 {
                p.num_guys = 0;
            }
        }
    }

    /// A unit of the air domain that is neither a Helicopter nor a missile —
    /// `is_flying_low`'s and `Ammo::init`'s three type tests (`+0x218` 2,
    /// `+0x2b4 & 0x20` clear, `+0x1e4 & 0x8000000` clear).
    pub(crate) fn is_fixed_wing(&self, u: usize) -> bool {
        let p = self.profile(crate::combat::Obj::Unit(u));
        matches!(p.domain, crate::attrition::Domain::Air)
            && !self.is_helicopter(u)
            && !p.has(crate::combat::mask::MISSILE)
    }

    /// **`UnitData::is_flying_low@0060a140`** — a distance, not an altitude
    /// (item 1102, `docs/COMBAT.md` §81; the listing, `60a140`..`60a2ff`,
    /// run whole under `tools/emu/flak_arm.py`). A fixed-wing unit on the
    /// map is low when its front order is a `STRAFE` whose target is live
    /// and within [`LOW_REACH`] of it, or an `AIR_ATTACK_GROUND` whose point
    /// is; failing that, when its air order (`get_air_order`, vslot `0xfc`)
    /// has `returning` set and its live home is within the reach. No order
    /// is not low, and nor is any order that is not an air order.
    pub(crate) fn is_flying_low(&self, u: usize) -> bool {
        let unit = &self.units[u];
        if !self.is_fixed_wing(u) || !unit.alive() || unit.inside.is_some() {
            return false;
        }
        let here = unit.pos;
        let near = |p: Pos| vector_dist(p.x - here.x, p.y - here.y) < LOW_REACH;
        match self.current_order(u).map(|o| o.body) {
            None => return false,
            Some(Body::Strafe(sf)) => {
                if let Some(t) = sf.target
                    && self.active(t)
                    && near(self.pos_of(t))
                {
                    return true;
                }
            }
            Some(Body::AirAttackGround(g)) => {
                if near(g.at) {
                    return true;
                }
            }
            Some(_) => {}
        }
        self.current_air(u).is_some_and(|a| {
            a.returning
                && a.home
                    .is_some_and(|b| self.buildings[b].alive && near(self.buildings[b].pos))
        })
    }

    /// **An `ANTI_AIR` attacker rolls whether its radar is jammed**
    /// (item 1102, `docs/COMBAT.md` §81): `Unit::fight@005fd4d0`'s
    /// animation choice, past the ship's, the Patrol Boat's and the
    /// Immortals' arms (`5fee5a`..`5feec1`), asks the type's
    /// `has_objmask(0x80000000)`; an `ANTI_AIR` type draws
    /// `GameAccess::rnd(100)` and, when it is under
    /// [`JAM_UNIT_RADAR_PROB`] and `ObjectData::is_jammed@00653660` finds a
    /// jammer, plays animation 0 in place of `CHAR_ATTACK1`. run404's first
    /// Battery shot spends it on 742.
    ///
    /// SEAM: `is_jammed` is −1 unless the owner's leader carries
    /// `leader_flags & 0x40000` and an enemy's special stands over the
    /// unit; nothing in this crate sets either, so the roll is spent and
    /// never jams.
    pub(crate) fn radar_jams(&mut self, u: usize) -> bool {
        if !self
            .profile(crate::combat::Obj::Unit(u))
            .has(crate::combat::mask::ANTI_AIR)
        {
            return false;
        }
        self.mark(SITE_JAM_ROLL);
        let roll = self.rnd(100);
        roll < JAM_UNIT_RADAR_PROB && self.is_jammed(u)
    }

    /// **An `ANTI_AIR` unit shoots at an aircraft without turning**
    /// (item 1109, `docs/COMBAT.md` §83): `Unit::fight@005fd4d0`, past
    /// `Unit::set_attack` (`5fe7f4`), sets the "keep `this->angle`" answer
    /// to 1 when the **target's** type is of the air domain (`+0x218 == 2`,
    /// `5fe81f`) and the shooter `has_objmask(0x80000000)` (`5fe82a`..
    /// `5fe855`, a `cmovne`), so `5febb0` keeps the unit's angle whatever
    /// its pivots answered. run404's Battery re-attacks Bomber `0/7` on 773
    /// while its hull is still turning, keeps its heading, and swings on
    /// 776 when the turn ends.
    pub(crate) fn anti_air_keeps_angle(&self, u: usize, target: crate::combat::Obj) -> bool {
        use crate::combat::{Obj, mask};
        self.profile(Obj::Unit(u)).has(mask::ANTI_AIR)
            && matches!(self.profile(target).domain, crate::attrition::Domain::Air)
    }

    /// **`Objects::kill_guy@00659410`'s plane arm** (item 1102,
    /// `docs/COMBAT.md` §81): a guy whose type's `CAT` is Air and that is
    /// not a missile (`659473` → `659613`) takes a free `Ammo` and
    /// `Ammo::init_crash@0067b800`, never `add_death`. The crash draws once
    /// from the game's stream ([`SITE_CRASH_ROLL`]) and twice from a local
    /// `Random` seeded by the guy's point, which no trace of the game's
    /// stream counts. Returns whether the unit crashed, so its caller lays
    /// no death object. run404's two Bombers: `rolling` −1 and −2, from 58907
    /// and 56687.
    ///
    /// SEAM: the falling round itself — its landing (a float fall time
    /// and `sin_table` drift), its `rolling`, and `Ammo::check_hit` when it
    /// comes down — is not laid; run404's two land with no draw from the
    /// game's stream.
    pub(crate) fn crashes(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if self.unit_types[t].cols.cat != CAT_AIR
            || self
                .profile(crate::combat::Obj::Unit(u))
                .has(crate::combat::mask::MISSILE)
        {
            return false;
        }
        self.mark(SITE_CRASH_ROLL);
        let _rolling = self.rng.roll() % 7 - 3;
        true
    }

    /// `ObjectData::is_jammed@00653660`, as far as this crate reaches: no
    /// leader here carries `leader_flags & 0x40000`, so never.
    fn is_jammed(&self, _u: usize) -> bool {
        false
    }

    /// `UnitData::is_flying_high@0060a310`: a fixed-wing unit on the map that
    /// is not low.
    ///
    /// SEAM: its caller is `valid_target`'s air ladder (`docs/COMBAT.md`
    /// §61), which still reads every plane as high; run404's Infantry
    /// never weighed a low Bomber, so no capture parts on it.
    #[allow(dead_code)]
    pub(crate) fn is_flying_high(&self, u: usize) -> bool {
        let unit = &self.units[u];
        self.is_fixed_wing(u) && unit.alive() && unit.inside.is_none() && !self.is_flying_low(u)
    }

    /// **`Ammo::init@0067bbf0`'s air arm** (item 1102, `docs/COMBAT.md`
    /// §81): the round's miss (`flags |= 0x10`), or `None` when the arm is
    /// not entered. It is entered for a live fixed-wing target, unless the
    /// shooter is a unit on `ATTACK_GROUND` (0x17) or `AIR_ATTACK_GROUND`
    /// (0x18) (`67be7e`, `67bec4`). An `ANTI_AIR` shooter of the air domain
    /// never misses and draws nothing; any other `ANTI_AIR` shooter draws
    /// once and misses unless `r % 100` is under its own `FLY_LOW` (the
    /// target low) or `FLY_HIGH`; a shooter that is not `ANTI_AIR` draws
    /// against the target's own figure first and, past it, its own.
    pub(crate) fn air_round_misses(
        &mut self,
        shooter: crate::combat::Obj,
        target: crate::combat::Obj,
    ) -> Option<bool> {
        use crate::combat::{Obj, mask};
        let Obj::Unit(t) = target else {
            return None;
        };
        if !self.active(target) || !self.is_fixed_wing(t) {
            return None;
        }
        if let Obj::Unit(s) = shooter
            && matches!(self.current_order(s).map(|o| o.index()), Some(0x17 | 0x18))
        {
            return None;
        }
        let sp = self.profile(shooter);
        let tp = self.profile(target);
        let low = self.is_flying_low(t);
        if sp.has(mask::ANTI_AIR) {
            if matches!(sp.domain, crate::attrition::Domain::Air) {
                return Some(false);
            }
            self.mark(if low { SITE_FLAK_LOW } else { SITE_FLAK_HIGH });
            let r = self.rng.roll() % 100;
            return Some(r >= if low { sp.fly_low } else { sp.fly_high });
        }
        let (first, second, theirs, own) = if low {
            (
                SITE_AIR_TARGET_LOW,
                SITE_AIR_SHOOTER_LOW,
                tp.fly_low,
                sp.fly_low,
            )
        } else {
            (
                SITE_AIR_TARGET_HIGH,
                SITE_AIR_SHOOTER_HIGH,
                tp.fly_high,
                sp.fly_high,
            )
        };
        self.mark(first);
        if self.rng.roll() % 100 >= theirs {
            return Some(true);
        }
        self.mark(second);
        Some(self.rng.roll() % 100 >= own)
    }
}

#[cfg(test)]
mod flak_tests {
    use crate::attrition::Domain;
    use crate::combat::{self, Obj, mask};
    use crate::orders::{Body, Order, StrafeOrder};
    use crate::world::Pos;
    use crate::{Sim, Unit, UnitType};

    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(128, 128),
            2,
        );
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        s.rng = combat::Rng::new(12345);
        s
    }

    fn building(s: &mut Sim, who: crate::Player, at: Pos, prof: combat::Profile) -> usize {
        let b = s.add_building(who, at, 0);
        s.buildings[b].started = true;
        s.buildings[b].active = true;
        s.buildings[b].combat = Some(prof);
        s.buildings[b].health = 1200;
        b
    }

    fn unit(s: &mut Sim, who: crate::Player, at: Pos, prof: combat::Profile) -> usize {
        let ty = s.add_unit_type(UnitType {
            hits: 300,
            kind: crate::attrition::UnitKind {
                domain: prof.domain,
                ..crate::attrition::UnitKind::default()
            },
            combat: prof,
            ..UnitType::default()
        });
        let index = i16::try_from(s.units.len()).unwrap();
        let mut u = Unit::new(who, index, at, 300);
        u.ty = Some(ty);
        u.on_map = true;
        let u = s.add_unit(u);
        s.units[u].kind = s.unit_types[ty].kind;
        u
    }

    /// run223's Bomber: FLY_HIGH 0, FLY_LOW 10, of the air domain.
    fn bomber() -> combat::Profile {
        combat::Profile {
            domain: Domain::Air,
            fly_high: 0,
            fly_low: 10,
            ..combat::Profile::default()
        }
    }

    fn strafe(target: Option<Obj>, home: Option<usize>, returning: bool) -> Order {
        Order {
            flags: 0,
            body: Body::Strafe(StrafeOrder {
                target,
                mandatory: false,
                home,
                cruising_alt: crate::orders::CRUISING_ALT,
                sharp_turn: 0,
                returning,
                at: None,
            }),
        }
    }

    /// run404's Radar Air Defense: the tower columns of `harness_tests`'
    /// tower, `ANTI_AIR`, range 10, FLY 33/75, and its type's cycle — the
    /// Unpack's 20 frames, the Attack1's 10, the one release at frame 2 on
    /// node 0, and the measured launch vector.
    fn radar(s: &mut Sim, with_cycle: bool) -> usize {
        let b = building(
            s,
            1,
            Pos::new(22272, 16512),
            combat::Profile {
                obj_masks: mask::ANTI_AIR,
                attack: 80,
                recharge: 60,
                max_range: 10,
                to_hit: 400,
                proj_speed: 200,
                ammo_per_att: 1,
                base_arrows: 1,
                most_shots: 4,
                x_size: 2,
                y_size: 2,
                big_radius: 96,
                fly_high: 33,
                fly_low: 75,
                ..combat::Profile::default()
            },
        );
        if with_cycle {
            let mut d = crate::tech::TypeDef::building("Radar Air Defense");
            d.wall_cycle = Some(crate::air::WallCycle {
                wind: 20,
                swing: 10,
                releases: vec![(2, 0, false)],
                launch: Some((136, -136, 227)),
            });
            let id = s.tech_tree.add(d);
            let bt = s.add_build_type(crate::build::BuildType {
                tree: Some(id),
                ..crate::build::BuildType::default()
            });
            s.buildings[b].ty = Some(bt);
        }
        b
    }

    /// **An anti-air building does not fire from `do_attack`** (item 1112,
    /// the call site): `Build::do_attack`'s in-range arm returns before
    /// `Object::fire_ammo` for an `ANTI_AIR` building that is neither a
    /// Lookout nor an Observation Post, and its head leaves `recharging`
    /// alone. The same building without the cycle fires at once.
    #[test]
    fn an_anti_air_building_takes_its_target_and_does_not_fire_it() {
        for with_cycle in [true, false] {
            let mut s = sim();
            let r = radar(&mut s, with_cycle);
            let plane = unit(&mut s, 0, Pos::new(21600, 16400), bomber());
            s.buildings[r].recharging = 5;
            s.buildings[r].target = Some(Obj::Unit(plane));
            s.process_building_combat(r, 16);
            if with_cycle {
                assert_eq!(s.buildings[r].target, Some(Obj::Unit(plane)));
                assert!(s.projectiles.is_empty(), "no round from do_attack");
                assert_eq!(s.buildings[r].recharging, 5, "no countdown");
            } else {
                assert_eq!(s.buildings[r].recharging, 4, "the countdown");
                s.buildings[r].recharging = 0;
                s.process_building_combat(r, 16);
                assert_eq!(s.projectiles.len(), 1, "fired from do_attack");
            }
        }
    }

    /// **The Radar Air Defense's cycle** (item 1112, `docs/COMBAT.md`
    /// §84): run404's `1/2007` from its acquisition on trace frame 777 —
    /// `recharging` 1..19, then −1..−10 and round again, a round on each
    /// frame the swing's clock crosses 2 (trace frames 797, 807, …), from
    /// (22408, 16376, 235) with `num_guys` 0; with the target gone, −4
    /// becomes 18 (block 1021) and counts down to 0.
    #[test]
    fn an_anti_air_building_winds_up_and_fires_on_its_swing() {
        let mut s = sim();
        let r = radar(&mut s, true);
        let plane = unit(&mut s, 0, Pos::new(21600, 16400), bomber());
        s.walls_inc_time(1);
        assert_eq!(s.buildings[r].recharging, 0, "no target: held at 0");
        s.buildings[r].target = Some(Obj::Unit(plane));
        let mut seen = Vec::new();
        let mut rounds = Vec::new();
        for f in 0..33 {
            s.frame = f;
            let before = s.projectiles.len();
            s.walls_inc_time(1);
            seen.push(s.buildings[r].recharging);
            if s.projectiles.len() > before {
                rounds.push(f);
            }
        }
        assert_eq!(seen[..19], (1..=19).collect::<Vec<i32>>()[..]);
        assert_eq!(seen[19..29], (1..=10).map(|k| -k).collect::<Vec<i32>>()[..]);
        assert_eq!(seen[29..33], [-1, -2, -3, -4]);
        assert_eq!(rounds, [20, 30], "on the swing's frame 2");
        let p = s.projectiles[0];
        assert_eq!(p.launch, Pos::new(22408, 16376));
        assert_eq!(p.num_guys, 0);
        assert_eq!(p.shooter, Obj::Building(r));
        s.buildings[r].target = None;
        s.walls_inc_time(1);
        assert_eq!(s.buildings[r].recharging, 18, "−4 to 19, and down one");
        for _ in 0..30 {
            s.walls_inc_time(1);
        }
        assert_eq!(s.buildings[r].recharging, 0, "down to 0, and held");
        s.walls_inc_time(0);
        assert_eq!(s.buildings[r].recharging, 0, "another leader's pass");
    }

    /// **An anti-air unit keeps its angle at an aircraft, and only there**
    /// (item 1109): the Battery at a Bomber does; at a ground unit it does
    /// not, and a shooter without `ANTI_AIR` does not at a Bomber.
    #[test]
    fn an_anti_air_unit_keeps_its_angle_at_an_aircraft_only() {
        let mut s = sim();
        let aa = combat::Profile {
            obj_masks: mask::ANTI_AIR,
            max_range: 17,
            ..combat::Profile::default()
        };
        let battery = unit(&mut s, 1, Pos::new(21384, 17256), aa);
        let rifle = unit(
            &mut s,
            1,
            Pos::new(21384, 17456),
            combat::Profile::default(),
        );
        let plane = unit(&mut s, 0, Pos::new(22000, 16000), bomber());
        let foot = unit(
            &mut s,
            0,
            Pos::new(22000, 18000),
            combat::Profile::default(),
        );
        assert!(s.anti_air_keeps_angle(battery, Obj::Unit(plane)));
        assert!(!s.anti_air_keeps_angle(battery, Obj::Unit(foot)));
        assert!(!s.anti_air_keeps_angle(rifle, Obj::Unit(plane)));
    }

    /// **The strike keeps the heading** (item 1109, the call site): a
    /// Battery facing away from a Bomber and with no pivots fires without
    /// turning, where it turns to a ground target.
    #[test]
    fn an_anti_air_strike_at_a_plane_does_not_turn_the_unit() {
        let mut s = sim();
        let aa = combat::Profile {
            obj_masks: mask::ANTI_AIR,
            max_range: 17,
            ..combat::Profile::default()
        };
        let battery = unit(&mut s, 1, Pos::new(21384, 17256), aa);
        let plane = unit(&mut s, 0, Pos::new(19000, 17256), bomber());
        let foot = unit(
            &mut s,
            0,
            Pos::new(19000, 17256),
            combat::Profile::default(),
        );
        let east = crate::movement::Angle::EAST;
        s.units[battery].movement.heading = east;
        s.fight_pub(battery, Obj::Unit(plane), 0);
        assert_eq!(s.units[battery].movement.heading, east, "kept at a plane");
        s.fight_pub(battery, Obj::Unit(foot), 1);
        assert_ne!(s.units[battery].movement.heading, east, "turned to a man");
    }

    /// **`is_flying_low` is a distance to the strike** (item 1102): the
    /// emulated original's table — 1 at 2303 and 0 at 2304 on the strafe's
    /// arm and on a returning plane's home, 0 on the diagonal (1600, 1600),
    /// 0 with `returning` clear, and never inside or for a missile.
    #[test]
    fn a_plane_flies_low_within_0x900_of_its_strike_or_its_home_returning() {
        let mut s = sim();
        let t = building(
            &mut s,
            1,
            Pos::new(21120, 16512),
            combat::Profile::default(),
        );
        let home = building(
            &mut s,
            0,
            Pos::new(11616, 13920),
            combat::Profile::default(),
        );
        let b = unit(&mut s, 0, Pos::new(21120 - 2303, 16512), bomber());
        s.units[b]
            .orders
            .push_back(strafe(Some(Obj::Building(t)), Some(home), false));
        assert!(s.is_flying_low(b), "2303 off its target");
        assert!(!s.is_flying_high(b));
        s.units[b].pos = Pos::new(21120 - 2304, 16512);
        assert!(!s.is_flying_low(b), "2304 off its target");
        assert!(s.is_flying_high(b));
        s.units[b].pos = Pos::new(21120 - 1600, 16512 - 1600);
        assert!(
            !s.is_flying_low(b),
            "the diagonal (1600, 1600) is past the reach"
        );
        s.buildings[t].alive = false;
        s.units[b].pos = Pos::new(21120, 16512);
        assert!(!s.is_flying_low(b), "a dead target is not a strike");
        s.units[b].orders.clear();
        s.units[b].orders.push_back(strafe(None, Some(home), true));
        s.units[b].pos = Pos::new(11616 + 2303, 13920);
        assert!(s.is_flying_low(b), "returning, 2303 off its home");
        s.units[b].pos = Pos::new(11616 + 2304, 13920);
        assert!(!s.is_flying_low(b), "returning, 2304 off its home");
        s.units[b].orders.clear();
        s.units[b].orders.push_back(strafe(None, Some(home), false));
        s.units[b].pos = Pos::new(11616, 13920);
        assert!(!s.is_flying_low(b), "not returning");
        s.units[b].orders.clear();
        assert!(!s.is_flying_low(b), "no order");
    }

    /// **An `ANTI_AIR` shooter rolls once, against its own figure by the
    /// target's altitude** (item 1102): run under `tools/emu/flak_arm.py`,
    /// a Battery (50/90) hits a low Bomber on 89 and misses on 90, and a
    /// high one hits on 49 and misses on 50. Here the Radar Air Defense's
    /// 33/75 on this crate's stream, one draw a round.
    #[test]
    fn an_anti_air_round_draws_once_against_its_own_fly_figure() {
        let mut s = sim();
        let t = building(
            &mut s,
            1,
            Pos::new(21120, 16512),
            combat::Profile::default(),
        );
        let radar = building(
            &mut s,
            0,
            Pos::new(22272, 16512),
            combat::Profile {
                obj_masks: mask::ANTI_AIR,
                fly_high: 33,
                fly_low: 75,
                ..combat::Profile::default()
            },
        );
        let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
        s.units[b]
            .orders
            .push_back(strafe(Some(Obj::Building(t)), None, false));
        for (at, figure) in [(21120, 75), (21120 - 5000, 33)] {
            s.units[b].pos = Pos::new(at, 16512);
            let mut probe = s.rng;
            let r = probe.roll() % 100;
            let missed = s.air_round_misses(Obj::Building(radar), Obj::Unit(b));
            assert_eq!(missed, Some(r >= figure), "roll {r} against {figure}");
            assert_eq!(s.rng.seed, probe.seed, "one draw");
        }
        // An ANTI_AIR shooter of the air domain takes no roll.
        let fighter = unit(
            &mut s,
            0,
            Pos::new(20000, 16512),
            combat::Profile {
                obj_masks: mask::ANTI_AIR,
                domain: Domain::Air,
                fly_low: 25,
                ..combat::Profile::default()
            },
        );
        let seed = s.rng.seed;
        assert_eq!(
            s.air_round_misses(Obj::Unit(fighter), Obj::Unit(b)),
            Some(false)
        );
        assert_eq!(s.rng.seed, seed, "no draw");
        // A ground target takes no arm at all.
        assert_eq!(
            s.air_round_misses(Obj::Building(radar), Obj::Building(t)),
            None
        );
    }

    /// **An `ANTI_AIR` attacker spends the jam roll, and nothing else
    /// does** (item 1102): `Unit::fight`'s `GameAccess::rnd(100)` at
    /// `5fee89`, run404's 742. No jammer exists here, so it never jams.
    #[test]
    fn an_anti_air_attacker_rolls_its_radar_and_no_other_does() {
        let mut s = sim();
        let aa = unit(
            &mut s,
            1,
            Pos::new(22392, 16632),
            combat::Profile {
                obj_masks: mask::ANTI_AIR,
                ..combat::Profile::default()
            },
        );
        let inf = unit(
            &mut s,
            1,
            Pos::new(20000, 16632),
            combat::Profile::default(),
        );
        let seed = s.rng.seed;
        assert!(!s.radar_jams(inf));
        assert_eq!(s.rng.seed, seed, "no draw for a type without ANTI_AIR");
        let mut probe = s.rng;
        let _ = probe.roll();
        assert!(!s.radar_jams(aa), "no jammer, no jam");
        assert_eq!(s.rng.seed, probe.seed, "one draw");
    }

    /// **A plane shot down draws its crash's roll and lays no death
    /// object** (item 1102): `kill_guy`'s arm for `CAT` Air; a missile and
    /// any other category take `add_death`.
    #[test]
    fn a_plane_of_cat_air_crashes_and_a_missile_does_not() {
        let mut s = sim();
        let b = unit(&mut s, 0, Pos::new(21120, 16512), bomber());
        let ty = s.units[b].ty.unwrap();
        s.unit_types[ty].cols.cat = super::CAT_AIR;
        let mut probe = s.rng;
        let _ = probe.roll();
        assert!(s.crashes(b));
        assert_eq!(s.rng.seed, probe.seed, "one draw");
        let seed = s.rng.seed;
        let missile = unit(
            &mut s,
            0,
            Pos::new(21120, 16512),
            combat::Profile {
                obj_masks: mask::MISSILE,
                domain: Domain::Air,
                ..combat::Profile::default()
            },
        );
        let mty = s.units[missile].ty.unwrap();
        s.unit_types[mty].cols.cat = super::CAT_AIR;
        assert!(!s.crashes(missile), "a missile takes add_death");
        let foot = unit(
            &mut s,
            0,
            Pos::new(21120, 16512),
            combat::Profile::default(),
        );
        assert!(!s.crashes(foot), "Foot takes add_death");
        assert_eq!(s.rng.seed, seed, "no draw for either");
    }

    /// **A fired round at a plane carries the roll's miss** (item 1102):
    /// `Object::fire_ammo`'s round takes `Ammo::init`'s air arm, whose miss
    /// is the round's `0x10`, this crate's `harmless` — run404's Battery
    /// round of 753 (flags 18, a miss) against a hit.
    #[test]
    fn a_round_fired_at_a_plane_is_harmless_when_its_roll_misses() {
        let mut s = sim();
        let t = building(
            &mut s,
            1,
            Pos::new(21120, 16512),
            combat::Profile::default(),
        );
        let radar = building(
            &mut s,
            0,
            Pos::new(22272, 16512),
            combat::Profile {
                obj_masks: mask::ANTI_AIR,
                fly_high: 33,
                fly_low: 75,
                max_range: 10 * 192,
                ..combat::Profile::default()
            },
        );
        let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
        s.units[b]
            .orders
            .push_back(strafe(Some(Obj::Building(t)), None, false));
        let (mut hits, mut misses) = (0, 0);
        for seed in 1..60u32 {
            s.rng = combat::Rng::new(seed);
            let mut probe = s.rng;
            let missed = probe.roll() % 100 >= 75;
            s.projectiles.clear();
            s.fire_ammo_pub(
                Obj::Building(radar),
                Obj::Unit(b),
                crate::movement::Angle(0),
                800,
                Pos::new(22272, 16512),
                235,
                0,
                false,
            );
            assert_eq!(s.projectiles.len(), 1);
            assert_eq!(s.projectiles[0].harmless, missed, "seed {seed}");
            if missed { misses += 1 } else { hits += 1 }
        }
        assert!(
            hits > 0 && misses > 0,
            "both arms reached: {hits} hits, {misses} misses"
        );
    }

    /// **A shooter that is not `ANTI_AIR` rolls against the target's figure
    /// first** (item 1102): the emulator's Infantry (0/33) at a low Bomber
    /// — rolls 9 and 32 hit, 9 and 33 miss, a first roll of 10 misses with
    /// no second draw.
    #[test]
    fn any_other_shooter_rolls_the_target_s_figure_before_its_own() {
        let mut s = sim();
        let t = building(
            &mut s,
            1,
            Pos::new(21120, 16512),
            combat::Profile::default(),
        );
        let inf = unit(
            &mut s,
            0,
            Pos::new(20000, 16512),
            combat::Profile {
                fly_low: 33,
                ..combat::Profile::default()
            },
        );
        let b = unit(&mut s, 1, Pos::new(21120, 16512), bomber());
        s.units[b]
            .orders
            .push_back(strafe(Some(Obj::Building(t)), None, false));
        for seed in 1..400u32 {
            s.rng = combat::Rng::new(seed);
            let mut probe = s.rng;
            let r1 = probe.roll() % 100;
            let want = if r1 >= 10 {
                true
            } else {
                probe.roll() % 100 >= 33
            };
            assert_eq!(s.air_round_misses(Obj::Unit(inf), Obj::Unit(b)), Some(want));
            assert_eq!(
                s.rng.seed, probe.seed,
                "seed {seed}: one draw, or two past the first"
            );
        }
    }

    /// A Bomber that strikes: run404's `0/7`, with an attack so
    /// `get_damage` weighs something.
    fn striking_bomber() -> combat::Profile {
        combat::Profile {
            attack: 100,
            max_range: 3,
            min_range: 1,
            cost: 300,
            ..bomber()
        }
    }

    /// **An armed building's ×5 is a computer attacker's** (item 1131,
    /// `compare_target`'s `64f1a2`: `testb $0x4, leader_flags` of the
    /// attacker, `jne` past the weight). run404's human `0/7` weighs the
    /// Radar Air Defense at a fifth of what a computer's Bomber does, and
    /// that is what keeps its strafe on the Barracks on trace tick 818.
    #[test]
    fn a_human_s_bomber_weighs_an_armed_building_without_the_computer_s_five() {
        let weigh = |human: bool| {
            let mut s = sim();
            s.nation[0].human = human;
            let r = radar(&mut s, true);
            if let Some(p) = s.buildings[r].combat.as_mut() {
                p.cost = 250;
            }
            let plane = unit(&mut s, 0, Pos::new(22456, 16287), striking_bomber());
            s.compare_target(Obj::Unit(plane), Obj::Building(r), true, false)
        };
        let (human, computer) = (weigh(true), weigh(false));
        assert!(human > 15, "a weight above the floor: {human}");
        assert!(
            computer >= 4 * human,
            "the computer's ×5: {computer} against {human}"
        );
    }

    /// **No object stands below zero** (item 1131, `Unit::update_z`'s
    /// `pushl $0x1` at `00606598`): a plane over a tile whose height is
    /// negative presents `z` 0 to `get_damage`, so the river step (`T.z <
    /// 0`, ×2) never doubles a hit on it. run404's `0/7` over the river
    /// east of the Barracks took 32 from the Radar where ours dealt 64.
    #[test]
    fn a_plane_over_a_tile_below_zero_takes_no_river_doubling() {
        let hit = |z: i32| {
            let mut s = sim();
            let r = radar(&mut s, true);
            let at = Pos::new(22512, 16266);
            s.world.set_tile_z(at.tile(), z);
            let plane = unit(&mut s, 0, at, striking_bomber());
            s.do_damage(
                Obj::Building(r),
                Obj::Unit(plane),
                crate::movement::Angle(0),
                true,
                0x100,
                false,
                true,
                800,
            );
            300 - s.units[plane].health
        };
        assert!(hit(0) > 0, "the Radar hurts it");
        assert_eq!(hit(-6), hit(0), "z below zero is z 0");
        let mut w = crate::world::World::new(8, 8);
        let t = Pos::new(3, 3);
        w.set_tile_z(t, -6);
        assert_eq!(w.object_z(t), 0);
        w.set_tile_z(t, 17);
        assert_eq!(w.object_z(t), 17);
        w.set_tile_field(
            t,
            crate::world::tile::SURFACE,
            crate::world::tile::SURFACE_OCEAN,
        );
        assert_eq!(w.object_z(t), 0, "an ocean tile is 0");
    }

    /// **A building's hit records its own `o`** (item 1131,
    /// `ObjectData::get_captain@00472400`): run404's `0/7` reads `damage_o
    /// 2007` from the Radar's round landing on 828.
    #[test]
    fn a_building_s_hit_records_its_own_object_number() {
        let mut s = sim();
        let r = radar(&mut s, true);
        let plane = unit(&mut s, 0, Pos::new(22512, 16266), striking_bomber());
        s.do_damage(
            Obj::Building(r),
            Obj::Unit(plane),
            crate::movement::Angle(0),
            true,
            0x100,
            false,
            false,
            828,
        );
        assert_eq!(
            s.units[plane].combat.damage_o,
            i32::from(s.buildings[r].index)
        );
    }

    /// **An unordered building re-finds its target on every `do_attack`**
    /// (item 1131, `622a3f`): a moving target weighs a quarter, so the
    /// Radar leaves it for a still plane beside it on a frame that is not
    /// its phase. run404's `1/2007` turns from `0/7` to `0/6` on trace
    /// tick 836.
    #[test]
    fn an_unordered_building_re_finds_its_target_on_every_call() {
        let mut s = sim();
        let r = radar(&mut s, true);
        let a = unit(&mut s, 0, Pos::new(22900, 16000), striking_bomber());
        let b = unit(&mut s, 0, Pos::new(22900, 17000), striking_bomber());
        s.units[a].movement.dest = Some(Pos::new(30000, 16000));
        s.buildings[r].target = Some(Obj::Unit(a));
        let frame = 16;
        assert_ne!(
            (s.buildings[r].phase(frame) + 14) & 0x1f,
            0,
            "not the phase's frame"
        );
        s.process_building_combat(r, frame);
        assert_eq!(s.buildings[r].target, Some(Obj::Unit(b)));
    }

    /// **A building's target outlives its death** (item 1131): run404's
    /// Radar still holds `attack_ox 6` on block 1007, after `0/6` was shot
    /// down on 1006, so `Build::process` runs `do_attack` on tick 1007 and
    /// its `find_target` turns it to `0/7`.
    /// A stockade-shaped building: range 11, a ground unit 13 tiles away.
    fn stockade_and_scout(s: &mut Sim) -> (usize, usize) {
        let b = building(
            s,
            1,
            Pos::new(22272, 16512),
            combat::Profile {
                attack: 12,
                recharge: 30,
                max_range: 11,
                to_hit: 110,
                proj_speed: 200,
                ammo_per_att: 1,
                base_arrows: 1,
                most_shots: 2,
                x_size: 2,
                y_size: 2,
                ..combat::Profile::default()
            },
        );
        let sc = unit(
            s,
            0,
            Pos::new(22272 + 13 * 192 + 192 + 72, 16512),
            combat::Profile {
                domain: Domain::Land,
                ..combat::Profile::default()
            },
        );
        (b, sc)
    }

    /// **`get_building_range`'s British term** (item 1415, `docs/GOLDEN.md`
    /// §59): a British owner's Tower-line building reaches `BRITISH_TOWER_RANGE`
    /// further; anyone else's, or a non-tower's, does not.
    #[test]
    fn a_british_tower_line_building_reaches_two_tiles_further() {
        let mut s = sim();
        let (b, _) = stockade_and_scout(&mut s);
        let bt = s.add_build_type(crate::build::BuildType {
            ident: crate::build::Ident::Tower,
            ..crate::build::BuildType::default()
        });
        s.buildings[b].ty = Some(bt);
        let me = Obj::Building(b);
        assert_eq!(s.max_range_of(me), 11, "not British");
        s.nation[1].british = true;
        assert_eq!(s.max_range_of(me), 13, "British Tower line");
        let other = s.add_build_type(crate::build::BuildType::default());
        s.buildings[b].ty = Some(other);
        assert_eq!(s.max_range_of(me), 11, "British, not a tower");
    }

    /// **`Build::process`'s early trigger** (item 1415): a building with no
    /// target runs `do_attack` off its 32-frame phase when `near_o` is in
    /// range.
    #[test]
    fn a_building_with_a_sighting_in_range_attacks_off_its_phase() {
        let mut s = sim();
        let (b, sc) = stockade_and_scout(&mut s);
        s.units[sc].pos = Pos::new(22272 + 8 * 192, 16512);
        let frame = (5 - i64::from(s.buildings[b].index)).rem_euclid(32);
        s.process_building_combat(b, frame);
        assert_eq!(s.buildings[b].target, None, "no sighting, off phase");
        s.buildings[b].near = Some(Obj::Unit(sc));
        s.process_building_combat(b, frame);
        assert_eq!(
            s.buildings[b].target,
            Some(Obj::Unit(sc)),
            "sighting in range"
        );
    }

    /// **`Build::do_attack`'s `visible` write** (item 1419, `docs/GOLDEN.md`
    /// §59, `622b5b`..`622bd0`): a round fired sets the target owner's bit
    /// on the building, which is `BuildData::is_seen`'s first arm; the
    /// 32-frame phase with the latch down clears it.
    #[test]
    fn a_building_that_fires_shows_itself_to_the_player_it_shot() {
        let mut s = sim();
        let (b, sc) = stockade_and_scout(&mut s);
        s.units[sc].pos = Pos::new(22272 + 8 * 192, 16512);
        s.buildings[b].near = Some(Obj::Unit(sc));
        let frame = (5 - i64::from(s.buildings[b].index)).rem_euclid(32);
        assert_eq!(s.buildings[b].visible, 0);
        s.process_building_combat(b, frame);
        assert!(!s.projectiles.is_empty(), "it fired");
        assert_eq!(s.buildings[b].visible, 1, "the Scout's owner's bit");
        assert!(s.buildings[b].attacking);
        assert!(s.build_is_seen(b, 0), "seen by who it shot");
        assert!(!s.build_is_seen(b, 2), "and by nobody else");
        // The next phase call, with nothing to shoot: the latch holds the
        // bit and drops; the one after clears it.
        let phase0 = (32 - i64::from(s.buildings[b].index)).rem_euclid(32);
        s.buildings[b].recharging = 0;
        s.buildings[b].target = None;
        s.buildings[b].near = None;
        s.units[sc].pos = Pos::new(0, 0);
        s.process_building_combat(b, phase0);
        assert_eq!(s.buildings[b].visible, 1, "the latch held it");
        s.process_building_combat(b, phase0 + 32);
        assert_eq!(
            s.buildings[b].visible, 0,
            "cleared on the phase, latch down"
        );
    }

    #[test]
    fn a_building_keeps_a_dead_target_until_do_attack_replaces_it() {
        let mut s = sim();
        let r = radar(&mut s, true);
        let a = unit(&mut s, 0, Pos::new(22900, 16000), striking_bomber());
        let b = unit(&mut s, 0, Pos::new(22900, 17000), striking_bomber());
        s.buildings[r].target = Some(Obj::Unit(a));
        s.units[a].health = 0;
        s.forget(Obj::Unit(a));
        assert_eq!(s.buildings[r].target, Some(Obj::Unit(a)), "kept");
        s.process_building_combat(r, 16);
        assert_eq!(s.buildings[r].target, Some(Obj::Unit(b)), "replaced");
    }
}

/// **The Modern Infantry's step** (item 1109): `Unit::do_move`'s
/// `5f88b5`..`5f88cb`, `× 5 / 4` truncating on `get_speed`'s answer for a
/// type `is_modern_infantry` answers. It lives beside the air line because
/// chapter thirty-eight is where a Modern Infantry first walked in a
/// capture (run404's `1/6`, a first step of 42 on a speed of 34).
#[cfg(test)]
mod infantry_step_tests {
    use crate::world::Pos;
    use crate::{Sim, Unit, UnitType, movement};

    fn walker(age: i32, flags: u32) -> (Sim, usize) {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(128, 128),
            2,
        );
        let mut ty = UnitType {
            hits: 50,
            moves: 34,
            ..UnitType::default()
        };
        ty.cols.unit_flags = flags;
        ty.combat.age = age;
        let ty = s.add_unit_type(ty);
        let mut u = Unit::new(1, 0, Pos::new(0x60 * 20 + 0x30, 0x60 * 20 + 0x30), 50);
        u.ty = Some(ty);
        u.on_map = true;
        let u = s.add_unit(u);
        let m = &mut s.units[u].movement;
        m.speed = 34;
        m.turning = movement::Turning {
            type_turn_speed: movement::degrees_to_angle(45).0,
            packed: false,
            instant_from_stop: true,
            wide_limit: false,
        };
        m.set_facing(movement::Angle::EAST);
        (s, u)
    }

    fn first_step(age: i32, flags: u32) -> i32 {
        let (mut s, u) = walker(age, flags);
        let from = s.units[u].pos;
        s.order_move(u, Pos::new(from.x + 0x60 * 20, from.y));
        for _ in 0..8 {
            s.tick();
            if s.units[u].pos != from {
                break;
            }
        }
        s.units[u].pos.x - from.x
    }

    /// **The pack** (item 1113, `Unit::do_move`'s `5f82df`-`5f8390`): once
    /// every 128 frames, on its own phase `(o · 0x11 + frame) & 0x7f`, a
    /// walking Modern Infantry stops, plays `CHAR_PACK` and waits out the
    /// animation with `retry` its `end_time` and `attempts` −3. run404's
    /// `1/7` (o 7) packs on 777 and `1/6` (o 6) on 794, 128 apart from
    /// nothing but their numbers.
    #[test]
    fn a_walking_modern_infantry_packs_on_its_own_phase() {
        const PIECE: i32 = 7;
        let packs = |age: i32, flags: u32| {
            let (mut s, u) = walker(age, flags);
            s.units[u].index = 7;
            s.units[u].guys.push(crate::anim::Guy::fresh(PIECE));
            s.art
                .piece_lengths
                .entry(PIECE)
                .or_default()
                .insert(crate::anim::PACK, 22);
            let from = s.units[u].pos;
            s.order_move(u, Pos::new(from.x + 0x60 * 60, from.y));
            let mut seen = Vec::new();
            for _ in 0..300 {
                let f = s.frame;
                let at = s.units[u].pos;
                s.tick();
                let mo = s.units[u].orders.iter().find_map(|o| match &o.body {
                    crate::orders::Body::Move(m) => Some((m.retry, m.attempts)),
                    _ => None,
                });
                if s.units[u].guys[0].anim == crate::anim::PACK && mo == Some((22, -3)) {
                    seen.push((f, at == s.units[u].pos));
                }
            }
            seen
        };
        // o 7: `7 · 0x11 + 9 = 128`, so frames 9 and 137 of the walk (it
        // has arrived by 265), and the unit stands on each, `retry` 22
        // frames of `CHAR_PACK`.
        assert_eq!(
            packs(6, 0x100),
            [(9, true), (137, true)],
            "a Modern Infantry packs on its own phase"
        );
        assert!(packs(5, 0x100).is_empty(), "not past age 5: it walks on");
        assert!(packs(6, 0).is_empty(), "not the flag: it walks on");
    }

    #[test]
    fn a_modern_infantry_steps_five_quarters_of_its_speed() {
        // The step east is the trig component of the step, one short of it:
        // 42 walks 41 and 34 walks 33.
        assert_eq!(first_step(6, 0x100), 41, "34 x 5 / 4 = 42, truncated");
        assert_eq!(first_step(5, 0x100), 33, "not past age 5: 34");
        assert_eq!(first_step(6, 0), 33, "not the flag: 34");
    }
}
