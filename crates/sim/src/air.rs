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
use crate::num::single::Single;
use crate::orders::{Body, CRUISING_ALT};
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

/// **A crash round's splash**, in tiles: `Ammo::init_crash@0067b800`
/// writes `+0x30 = 2` (`docs/AI.md` §230; run405's two crash rounds print
/// `splash_area 2`).
pub const CRASH_SPLASH_AREA: i32 = 2;

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
pub(crate) fn air_turn_speed(base: u32, roll: Single, sign: i32) -> u32 {
    air_turn_speed_as(base, roll, sign, false)
}

/// [`air_turn_speed`] with the type's helicopter bit (`unit_flags & 0x20`,
/// item 3272): the bank is read five times as large, and the rate is not
/// floored (the reversed-bank answer is the floor's own constant either way).
pub(crate) fn air_turn_speed_as(base: u32, roll: Single, sign: i32, heli: bool) -> u32 {
    let mut v = roll.to_i32().wrapping_neg();
    if heli {
        v = v.wrapping_mul(5);
    }
    if (v ^ sign) < 0 {
        return MIN_TURN;
    }
    let rate = (base / 0x37).wrapping_mul(v.unsigned_abs());
    if rate < MIN_TURN && !heli {
        MIN_TURN
    } else {
        rate
    }
}

/// `Unit::find_new_air_target`'s selection over answered callees (item 3250,
/// `docs/GOLDEN.md` §50): the walk of [`Sim::find_new_air_target`] with the
/// two `find_units` result lists, `valid_target` and `compare_target` handed
/// in, so a sweep can answer them from a row. `range` is
/// `AIRCRAFT_RESPOND_RANGE`; `near` the air-domain enemies round the plane,
/// `ring` the enemies round the point (both in `find_units`' order, each with
/// its position), `at` the point already replaced by the plane's own where
/// the plane is a computer's. Best starts at −1 and a strictly greater
/// `cmp` replaces it.
pub(crate) fn pick_air_target<T: Copy>(
    range: i32,
    me: Pos,
    at: Pos,
    near: impl FnOnce() -> Vec<(T, Pos)>,
    ring: impl FnOnce() -> Vec<(T, Pos)>,
    mut valid: impl FnMut(T) -> bool,
    mut cmp: impl FnMut(T) -> i32,
) -> Option<T> {
    let r = range * 0xc0;
    let mut best: Option<(i32, T)> = None;
    let mut take = |o: T, p: Pos, from: Pos, best: &mut Option<(i32, T)>| {
        if !valid(o) {
            return;
        }
        if vector_dist(p.x - from.x, p.y - from.y) > r {
            return;
        }
        let v = cmp(o);
        if v > best.map_or(-1, |(w, _)| w) {
            *best = Some((v, o));
        }
    };
    for (o, p) in near() {
        take(o, p, me, &mut best);
    }
    if let Some((_, t)) = best {
        return Some(t);
    }
    if vector_dist(at.x - me.x, at.y - me.y) > range * 0x3c0 {
        return None;
    }
    for (o, p) in ring() {
        take(o, p, at, &mut best);
    }
    best.map(|(_, t)| t)
}

/// What `Unit::do_air_physics` reads and does to a bird, as [`air_physics_steps`]
/// needs it: the unit's point and heading, the world's size, the type's
/// speed and range, the order's turn flag, and the effects the bird's path
/// has (the bank, the figure's snap, the coin, the move). [`SimAirPhysics`] is
/// the live [`Sim`]'s; `sweep::unit_do_air_physics` implements it over the
/// emulated original's windows.
pub trait AirPhysicsView {
    /// The world's width and height in units (`xs * 0x300`, `ys * 0x300`).
    fn world_units(&self) -> (i32, i32);
    /// The unit's point.
    fn pos(&self) -> Pos;
    /// `get_speed(x, y, 1)`, before the `ai_speed` multiply.
    fn get_speed(&mut self) -> i32;
    /// `*GameAccess::ai_speed`.
    fn ai_speed(&self) -> i32;
    /// The unit's heading (`+0x50`).
    fn heading(&self) -> Angle;
    /// The type's `min_range`.
    fn min_range(&self) -> i32;
    /// The order's turn flag (`order +0x10`).
    fn turn(&self) -> i32;
    fn set_turn(&mut self, t: i32);
    /// `Unit::bank_aircraft(des, &speed, 0)`.
    fn bank_aircraft(&mut self, des: Angle);
    /// `Guy::set_angle(heading, 1)`: the figure's snap onto the heading.
    fn snap_facing(&mut self, heading: Angle);
    /// The edge coin: `Random::get(0, 0xffff)`'s low bit.
    fn edge_coin(&mut self) -> bool;
    /// `Unit::set_new_location(x, y, 0, 1)`.
    fn set_new_location(&mut self, to: Pos);
}

/// `Unit::do_air_physics`, on the path a wild bird takes through it: the
/// body of [`Sim::do_air_physics`], over an [`AirPhysicsView`].
pub fn air_physics_steps<V: AirPhysicsView>(v: &mut V, goal: Pos) {
    let (xs, ys) = v.world_units();
    // The waypoint is clamped into the world before anything measures
    // against it — negatives to zero, and the far edge to one under.
    let gx = goal.x.clamp(0, xs - 1);
    let gy = goal.y.clamp(0, ys - 1);
    let at = v.pos();
    let (dx, dy) = (gx - at.x, gy - at.y);

    // `get_speed(x, y, 1)` — `AnimalData::get_speed`, which for an air
    // animal is `UnitData::speed` and nothing else (`orders.rs`). The
    // `ai_speed` multiply after it is `do_air_physics`'s own, not the
    // step's.
    let mut speed = v.get_speed();
    let ai = v.ai_speed();
    if ai > 1 {
        speed *= ai;
    }

    let heading = v.heading();
    let mut des = find_angle(dx, dy);
    // Owing more than forty-five degrees within a tile and a half of the
    // point, the bird keeps its heading and flies past rather than
    // wheeling on the spot. `min_range` is the *type's*, which is zero
    // for a bird, so the radius is the bare `0x300`.
    let min_range = v.min_range();
    if owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < min_range * 0xc0 + 0x300 {
        des = heading;
    }
    // Already turning away from an edge: the heading it wants is a
    // quarter turn off the one it has, which is what carries it back
    // over the map.
    let turn = v.turn();
    if turn != 0 {
        des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
    }

    v.bank_aircraft(des);

    // `Guy::set_angle(angle, 1)`: the figure snaps onto the unit's
    // heading rather than turning toward it, and the step is taken
    // along that.
    let heading = v.heading();
    v.snap_facing(heading);
    let nx = at.x + sin_component(heading, speed);
    let ny = at.y - cos_component(heading, speed);

    // `UnitData::invalid_loc` on an **air** type is the world's own
    // rectangle and nothing else: its domain arm (`type +0x218 == 2`)
    // returns valid before every terrain test, and the caller passes
    // zero for all five of the flags that would reach the rest. So the
    // only way a bird's step is refused is by leaving the map.
    if nx >= 0 && ny >= 0 && nx < xs && ny < ys {
        v.set_turn(0);
    } else if v.turn() == 0 {
        // `((rnd & 0x80000001) != 0) * 2 - 1`, and the draw's range is
        // `0, 0xffff`, so the sign bit can never be set and it is the
        // parity that decides.
        let t = if v.edge_coin() { 1 } else { -1 };
        v.set_turn(t);
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
    v.set_new_location(Pos::new(nx.clamp(0, xs - 1), ny.clamp(0, ys - 1)));
}

/// [`AirPhysicsView`] over the live [`Sim`]: unit `u` on `frame`.
struct SimAirPhysics<'a> {
    sim: &'a mut Sim,
    u: usize,
    frame: i64,
}

impl AirPhysicsView for SimAirPhysics<'_> {
    fn world_units(&self) -> (i32, i32) {
        (
            self.sim.world.width() * UNITS_PER_CELL,
            self.sim.world.height() * UNITS_PER_CELL,
        )
    }
    fn pos(&self) -> Pos {
        self.sim.units[self.u].pos
    }
    fn get_speed(&mut self) -> i32 {
        self.sim.get_speed(self.u, 1)
    }
    fn ai_speed(&self) -> i32 {
        self.sim.ai.ai_speed
    }
    fn heading(&self) -> Angle {
        self.sim.units[self.u].movement.heading
    }
    fn min_range(&self) -> i32 {
        self.sim.profile(crate::combat::Obj::Unit(self.u)).min_range
    }
    fn turn(&self) -> i32 {
        self.sim.flight(self.u).turn
    }
    fn set_turn(&mut self, t: i32) {
        let mut f = self.sim.flight(self.u);
        f.turn = t;
        self.sim.set_flight(self.u, f);
    }
    fn bank_aircraft(&mut self, des: Angle) {
        self.sim.bank_aircraft(self.u, des, self.frame);
    }
    fn snap_facing(&mut self, heading: Angle) {
        self.sim.units[self.u].movement.facing = heading;
    }
    fn edge_coin(&mut self) -> bool {
        self.sim.mark(SITE_AIR_TURN);
        self.sim.rng.roll() & 1 != 0
    }
    fn set_new_location(&mut self, to: Pos) {
        self.sim.set_new_location(self.u, to, false);
    }
}

impl Sim {
    /// A bird's [`Flight`], which starts at a zero bank and no turn.
    pub fn flight(&self, u: usize) -> Flight {
        self.gaia
            .gaia
            .bird_flight
            .iter()
            .find(|(b, _)| *b == u)
            .map_or(Flight::default(), |(_, f)| *f)
    }

    fn set_flight(&mut self, u: usize, f: Flight) {
        match self.gaia.gaia.bird_flight.iter_mut().find(|(b, _)| *b == u) {
            Some((_, slot)) => *slot = f,
            None => self.gaia.gaia.bird_flight.push((u, f)),
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
        air_physics_steps(
            &mut SimAirPhysics {
                sim: self,
                u,
                frame,
            },
            goal,
        );
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
            .wrapping_mul(self.tables.tuning.unit_turn_speed as u32);
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
            .and_then(|i| self.tables.unit_types[i].tree)
            .is_some_and(|ut| {
                ut == t
                    || (ut < self.tables.tech_tree.types.len()
                        && self.tables.tech_tree.is(ut, t, false))
            })
    }

    /// A unit whose type strafes: `unit_flags & 0x400000`, flag `w`
    /// (the Fighter line; `docs/ORDERS.md` §39). A building is not.
    pub(crate) fn strafes(&self, o: crate::combat::Obj) -> bool {
        let crate::combat::Obj::Unit(u) = o else {
            return false;
        };
        self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t].cols.unit_flags & crate::ai::ai_load::uflags::STRAFES != 0
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
            self.tables.unit_types[t].cols.unit_flags & crate::ai::ai_load::uflags::FLAG_HELICOPTER
                != 0
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
        if !self.is_helicopter(u)
            || self.unit_domain_of(u) != crate::territory::attrition::Domain::Air
        {
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
        let mine = self.units[u]
            .ty
            .and_then(|t| self.tables.unit_types[t].tree)?;
        let fits = |c: usize| -> Option<i32> {
            let x = &self.units[c];
            if c == u || !x.alive() || !x.on_map || x.owner >= crate::world::PLAYER_SLOTS {
                return None;
            }
            let same =
                x.ty.and_then(|t| self.tables.unit_types[t].tree)
                    .is_some_and(|t| self.tables.tech_tree.is(t, mine, false));
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
        let circle = crate::ai::ai_place::circle();
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
                let mut next = self.terrain.chain_heads[(c.y as usize) * width + (c.x as usize)];
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
    /// A helicopter (`unit_flags & 0x20`) is flown on its flight home
    /// alone (item 3990, `docs/GOLDEN.md` §183), and a home that is dead
    /// or will not carry the plane is re-homed or the plane dies
    /// (`rehome`, `out_of_fuel`). SEAMs: a helicopter's flight with a
    /// point; a flying target's lead (`local_20`, the doubled bank); and a
    /// carrier as the home.
    pub(crate) fn plane_air_physics(&mut self, u: usize, goal: Option<Pos>, frame: i64) -> Flew {
        let Some(mut air) = self.current_air(u) else {
            return Flew::Done;
        };
        // **A helicopter flies home alone** (item 3990): `do_garrison`'s
        // helicopter arm gives it a strafe with no target, which is
        // `returning` from its birth. SEAM: a helicopter's flight with a
        // point, which no adder here gives one.
        let heli = self.is_helicopter(u);
        if heli && !air.returning {
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
            let home = match air.home.filter(|&b| self.home_carries(b, u)) {
                Some(b) => Some(b),
                None => self.rehome(u),
            };
            match home {
                // `check_fuel:192`–`:236`: no approach for a helicopter; the
                // base's point less `0xc0`, at the base's height.
                Some(b) if heli => {
                    let p = self.buildings[b].pos;
                    Some(Approach {
                        at: Pos::new(p.x - 0xc0, p.y),
                        z: self.world.tile_z(p.tile()).max(0),
                    })
                }
                Some(b) => Some(self.home_approach(u, b)),
                // `check_fuel:120`–`:136`: no base. A helicopter aims at its
                // own point, at height 0; a plane dies.
                None if heli => Some(Approach {
                    at: self.units[u].pos,
                    z: 0,
                }),
                None => {
                    self.out_of_fuel(u);
                    return Flew::Done;
                }
            }
        } else {
            None
        };
        let (point, aim_z) = match (aim, goal) {
            (Some(a), _) => (a.at, a.z),
            (None, Some(g)) => (g, 0),
            (None, None) => return Flew::Done,
        };
        // **The redraw**, `+0x3b`–`+0xc4`: `(o + frame) & 7` with the
        // signed-modulo fix-up, which a non-negative sum never takes. A
        // Bomber holds `0x640` flat.
        if (i64::from(self.units[u].index) + frame) & 7 == 0 {
            // `+0x5e`: a Bomber, and a helicopter, hold `0x640` and draw
            // nothing.
            let alt = if self.is_bomber(u) || heli {
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
        if self.ai.ai_speed > 1 {
            speed *= self.ai.ai_speed;
        }
        // **The landing test** (`+0x1ea`), a flight home's alone:
        // Manhattan inside a step and a half, the point not clamped. The
        // helicopter's altitude half is not reached.
        // A helicopter's half (`+0x1ef`): its figure under `0x96` above
        // the height it aims at.
        if air.returning
            && dx.abs() + dy.abs() < speed * 3 / 2
            && !clamped
            && (!heli || self.units[u].airframe.z - aim_z < 0x96)
        {
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
        if !heli && owed(heading, des) > HALF_QUARTER && vector_dist(dx, dy) < radius + 0x300 {
            des = heading;
        }
        let turn = air.sharp_turn;
        if turn != 0 {
            des = Angle(heading.0.wrapping_add(turn.wrapping_shl(30)));
        }
        if heli {
            self.bank_plane_as(u, des, &mut speed, air.returning, false, true);
        } else {
            self.bank_plane(u, des, &mut speed, air.returning);
        }
        self.pitch_plane(u, dx, dy, aim_z, &mut speed, air.returning);
        let heading = self.units[u].movement.heading;
        self.units[u].movement.facing = heading;
        // **A helicopter hovers at its point** (`+0x56a`..`+0x5a0`): inside
        // `0x30` of a flight home (`0xc0` of a point) it takes no step; an
        // order behind this one is reached by killing it.
        if heli {
            let reach = if air.returning { 0x30 } else { 0xc0 };
            if vector_dist(dx, dy) < reach {
                if self.units[u].orders.len() > 1 {
                    self.kill_current_order(u);
                    return Flew::Done;
                }
                if self.units[u].combat.recharging == 0 {
                    self.set_anim(u, crate::units::anim::WALK, false, true);
                }
                return Flew::On;
            }
        }
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
            self.set_anim(u, crate::units::anim::WALK, false, true);
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
    /// `FIGHTERBOMBER`'s carrier-relative point.
    pub(crate) fn do_air_patrol(&mut self, u: usize, frame: i64) {
        if !matches!(
            self.current_order(u).map(|o| &o.body),
            Some(crate::orders::Body::AirPatrol(_))
        ) {
            return;
        }
        air_patrol_steps(&mut SimPatrol {
            sim: self,
            u,
            frame,
        });
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
        let me = self.units[u].pos;
        let who = self.units[u].owner;
        let at = if self.ai_driven(who) { me } else { at };
        let plane = crate::combat::Obj::Unit(u);
        let r = self.tables.tuning.aircraft_respond_range * 0xc0;
        pick_air_target(
            self.tables.tuning.aircraft_respond_range,
            me,
            at,
            || {
                self.find_units_round(me, r, who)
                    .into_iter()
                    .filter(|&o| {
                        matches!(
                            self.profile(crate::combat::Obj::Unit(o)).domain,
                            crate::territory::attrition::Domain::Air
                        )
                    })
                    .map(|o| (o, self.units[o].pos))
                    .collect()
            },
            || {
                self.find_units_round(at, r, who)
                    .into_iter()
                    .map(|o| (o, self.units[o].pos))
                    .collect()
            },
            // Unit targets only: the capture arm is a building's.
            |o| self.valid_target_const(plane, crate::combat::Obj::Unit(o)),
            |o| self.compare_target(plane, crate::combat::Obj::Unit(o), true, false),
        )
        .map(crate::combat::Obj::Unit)
    }

    /// `Objects::find_units@0065a620(at, SEARCH_ENEMY, who, range, …)`'s
    /// walk: the cell circle to ring `(range + 0x2ff) / 0x300` round the
    /// point's cell, each cell's chain in order, when that ring holds no
    /// more points than there are units; the unit list otherwise. Units
    /// alive, on the map, and an enemy of `who`'s.
    fn find_units_round(&self, at: Pos, range: i32, who: crate::Player) -> Vec<usize> {
        let circle = crate::ai::ai_place::circle();
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
        let range = self.tables.tuning.bomber_respond_range;
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
    /// is `base`, and the strike is `add_strafe_order(target, base, 1,
    /// QUEUE_NEW, 1)`: a strafe on the target, home the base, `returning
    /// 0`. The plane stays inside; [`Sim::do_launch`] puts it out. **A
    /// plane meets no gate** (item 3968, `docs/GOLDEN.md` §174): `6fbba8`'s
    /// `testl $0x8000000, 0x1e4` sends a type without the missile bit
    /// straight to `6fbe61`'s war test, so a target unseen or beyond the
    /// tank is struck — run1539's `0/6` on 742.
    ///
    /// **A missile takes the gates** (`6fbbb0`..`6fbea0`, item 1050):
    /// a target `Object::valid_target@00648ba0` passes, the shield, the
    /// reach `vector_dist(base − target) ≤ mana · get_speed(x, y, 1)`,
    /// and then `add_strafe_order`, whose head makes it an air attack on
    /// the target's point ([`Sim::add_air_attack_ground_order`]). run371's
    /// V2 `0/10` on 2672.
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
        // `6fbba8`: `testl $0x8000000, 0x1e4; je 6fbf6c` — a type without
        // the missile bit jumps past all three gates to `6fbe61`'s war
        // test (item 3968, T71-5).
        if self.profile(me).has(crate::combat::mask::MISSILE) {
            if !self.valid_target(me, target) {
                return;
            }
            let whom = self.owner_of(target);
            if whom != self.units[u].owner && self.missile_defense_held(whom) {
                return;
            }
            let (b, t) = (self.buildings[base].pos, self.pos_of(target));
            let reach = self.unit_mana(u) * self.get_speed(u, 1);
            if vector_dist(b.x - t.x, b.y - t.y) > reach {
                return;
            }
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
            && self
                .tables
                .tech_tree
                .roles
                .missile_defense_preq
                .is_some_and(|t| {
                    self.tables
                        .tech_tree
                        .has_tech(&self.tables.setup, &self.tech[who as usize], t)
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
    /// [`production::Queue::infinite`](crate::cities::production::Queue::infinite).
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
    } // twin of WallData::valid_buildmask

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

    /// **`Object::attempt_launch(x, y)@00643a10`** (item 3990,
    /// `docs/GOLDEN.md` §183): a blow on a base scrambles its fighter.
    /// `Object::do_damage@0064a480:194`–`:198` calls it on the struck
    /// object for every blow whose attacker is not a missile (`type
    /// +0x1e4 & 0x8000000`) and is alive (`+0x8 & 1`), after the struck
    /// unit's opportunity and before the scale, with the attacker's point.
    ///
    /// ```text
    /// launch_frames >= FRAMES_BETWEEN_LAUNCHES, inside_down >= 0,
    /// can_carry(AIR), launching empty, count_inside(COUNT_TYPE, BIPLANE):
    ///   walk the chain from inside_down:
    ///     an occupant with an order and a full tank (mana_burn 0): return
    ///     is(BIPLANE) and a full tank: the candidate (the last one wins)
    ///   the candidate: add_air_patrol_order(x, y, this, owner, action 1)
    /// ```
    ///
    /// The order is a patrol at the attacker with the action bit, so the
    /// base's next `do_launch` puts it out. SEAM: a carrier struck (a unit
    /// whose `inside_down` holds planes); this crate keeps no unit hangar.
    /// The chain's order is the garrison list's, as in [`Sim::do_launch`].
    pub(crate) fn attempt_launch(&mut self, b: usize, at: Pos) {
        let bd = &self.buildings[b];
        if bd.launch_frames < FRAMES_BETWEEN_LAUNCHES
            || bd.garrison.is_empty()
            || !bd.launching.is_empty()
            || !bd.ty.is_some_and(|t| self.is_hangar(t))
        {
            return;
        }
        let chain = bd.garrison.clone();
        if !chain
            .iter()
            .any(|&u| self.air_line_is(u, crate::air::airbase::BIPLANE))
        {
            return;
        }
        let mut pick = None;
        for u in chain {
            let full = self.units[u].mana_burn == 0;
            if full && !self.units[u].orders.is_empty() {
                return;
            }
            if full && self.air_line_is(u, crate::air::airbase::BIPLANE) {
                pick = Some(u);
            }
        }
        if let Some(u) = pick {
            self.add_air_patrol_order(u, at, Some(b), true);
        }
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
        if self
            .cities
            .nation
            .get(owner as usize)
            .is_none_or(|n| n.human)
        {
            return;
        }
        let id = i64::from(self.buildings[b].index);
        if (self.frame + id) % 32 != 0 {
            return;
        }
        if self.building_ident(b) == crate::cities::build::Ident::MissileSilo {
            self.silo_strike(b);
            return;
        }
        let base = self.buildings[b].pos;
        // A tree that does not carry the tech (the harness's small
        // fixtures) answers no.
        let stealth = STEALTHBOMBER < self.tables.tech_tree.types.len()
            && self.tech.get(owner as usize).is_some_and(|p| {
                self.tables
                    .tech_tree
                    .has_tech(&self.tables.setup, p, STEALTHBOMBER)
            });
        let mut best = -1;
        let mut target: Option<(crate::Player, usize)> = None;
        for l in 0..self.territory.players.len() as crate::Player {
            if self.leaders.defeated[l as usize] || self.is_peace(owner, l) {
                continue;
            }
            for c in 0..self.cities.cities.len() {
                let city = &self.cities.cities[c];
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
            if self.unit_domain_of(u) != crate::territory::attrition::Domain::Air {
                continue;
            }
            self.clear_orders(u);
            if d >= self.get_speed(u, 1) * self.unit_mana(u) {
                continue;
            }
            if self.is_ally(l, owner) && !self.air_line_is(u, crate::air::airbase::BIPLANE) {
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
    /// no field here holds (`crate::combat::nuke`), so the gate is read as open; and the friendly search's cell ring,
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
        if !self.air_line_is(m, crate::air::airbase::NUCLEARMISSILE) {
            return;
        }
        let owner = self.buildings[b].owner;
        let silo = self.buildings[b].pos;
        let reach = self.get_speed(m, 1) * self.unit_mana(m);
        let mut best = -1;
        let mut target = None;
        for l in 0..self.territory.players.len().min(8) as crate::Player {
            if !self.is_enemy(owner, l) || self.missile_defense_held(l) {
                continue;
            }
            for c in 0..self.cities.cities.len() {
                let city = &self.cities.cities[c];
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
                && self.building_ident(x) == crate::cities::build::Ident::AirDefense
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
        if self.building_ident(b) != crate::cities::build::Ident::MissileSilo {
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
        if self.air_line_is(u, crate::air::airbase::NUCLEARMISSILE)
            && matches!(
                self.current_order(u).map(|o| o.body),
                Some(crate::orders::Body::AirAttackGround(_))
            )
            && !self.combat.nukes.shown.contains(&b)
        {
            self.combat.nukes.shown.push(b);
        }
        self.come_out(u);
        let frame = self.frame;
        self.work(u, frame);
    }

    /// **`Unit::do_air_attack_ground@005ea420`** (items 1050, 3911):
    /// - `do_air_physics` flies the frame's step at the point
    ///   ([`Sim::plane_air_physics`]); a 0 return ends the call;
    /// - the unit's own `recharging` (`+0xae`) not 0, or the order's
    ///   `returning` (`+0x2c`), ends it;
    /// - `ObjectData::is_in_range@0064e4a0` of the point
    ///   ([`Sim::point_in_range`]): a missile that is not inside skips the
    ///   whole test and is in range;
    /// - a non-missile releases within 15° of its heading, 60° for the
    ///   `0x127` line (`5ea4a9`..`5ea4f2`), as `do_strafe` does;
    /// - `set_attack(−1, −1)`, and a type without `0x400000`, not of the
    ///   Bomber line, with an ammo piece fires at once, `fire_ammo(−1, −1)`
    ///   — for a missile [`Sim::missile_round`]; every other plays
    ///   `CHAR_ATTACK2` (`5ea56a`), run1505b's Bomber `0/7` on 846;
    /// - a non-missile reloads `recharge() + 1` (`5ea585`), and a missile
    ///   dies ([`Sim::missile_dies`]).
    ///
    /// SEAM: the immediate shot of a non-missile plane off the Bomber line
    /// with an ammo piece (no such plane takes the order here), and the
    /// Bomber's `BOMBING_MANA_COST` (0 in the shipped rules, and the tank
    /// is not carried — the strike's same seam).
    pub(crate) fn do_air_attack_ground(&mut self, u: usize, frame: i64) {
        let Some(crate::orders::Body::AirAttackGround(g)) = self.current_order(u).map(|o| o.body)
        else {
            return;
        };
        // **A flock's bird** (`crate::gaia::flock`): its `do_air_physics` leaves
        // `recharging` at 1, so the `+0xae` test below returns on every
        // frame it flies.
        if self.is_flockbird(u) {
            self.flock_air_physics(u, g.at, frame);
            return;
        }
        let flew = self.plane_air_physics(u, Some(g.at), frame);
        if matches!(flew, Flew::Done) {
            return;
        }
        if self.units[u].combat.recharging != 0 || g.returning {
            return;
        }
        let me = crate::combat::Obj::Unit(u);
        let missile = self.profile(me).has(crate::combat::mask::MISSILE);
        if !missile && !self.point_in_range(u, g.at) {
            return;
        }
        if !missile {
            let here = self.units[u].pos;
            let bearing = crate::movement::find_angle(g.at.x - here.x, g.at.y - here.y);
            let d = (self.units[u].movement.heading.0 as u32).wrapping_sub(bearing.0 as u32);
            let off = if d > 0x8000_0000 { !d } else { d };
            if off >= 0x0aaa_aaaa && (!self.is_fighter_line(u) || off > 0x2aaa_aaaa) {
                return;
            }
        }
        if !missile && !self.is_bomber(u) {
            // SEAM: a non-missile, non-Bomber plane's `fire_ammo` arm.
            return;
        }
        self.clear_attack(u);
        if missile {
            self.missile_round(u, g.at);
            self.missile_dies(u);
            return;
        }
        self.set_anim(u, crate::units::anim::ATTACK2, false, true);
        self.units[u].combat.recharging = self.reload_frames(u) + 1;
    }

    /// **`ObjectData::is_in_range@0064e4a0`, a unit's arm**, of a point:
    /// the unit's `vector_dist` to it less its own type's `big_radius`
    /// (`+0x244`, never under 0) — slot `+0x18` on `Unit`'s vtable is a
    /// folded `return 1` (`Buffer::is_pending_load@0041e0e0`), which takes
    /// that arm — within `0x66` for a type with no `max_range` (`+0x1fc`),
    /// else within `[min_range × 0xc0 − 6, max_range × 0xc0 + 6]`.
    /// run1505b's Bomber releases from 922 off the point on 846.
    pub(crate) fn point_in_range(&self, u: usize, at: Pos) -> bool {
        let me = crate::combat::Obj::Unit(u);
        let p = self.profile(me);
        let here = self.units[u].pos;
        let d = (vector_dist(at.x - here.x, at.y - here.y) - p.big_radius).max(0);
        if p.max_range == 0 {
            return d <= 0x66;
        }
        d >= p.min_range * 0xc0 - 6 && d <= self.max_range_of(me) * 0xc0 + 6
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
        let s = if self.air_line_is(u, crate::air::airbase::NUCLEARMISSILE) {
            0
        } else {
            crate::combat::scatter(&self.tables.tuning, acc, true, true, false)
        };
        let mut landing = at;
        // `s − 1 < 1` (`local_28`) takes no draw.
        if s > 1 {
            self.mark(crate::combat::fight::SITE_AMMO_GROUND_SCATTER_X);
            landing.x += self.rng.roll() % s - s / 2;
            self.mark(crate::combat::fight::SITE_AMMO_GROUND_SCATTER_Y);
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
            flame: false,
            punctures: false,
            spent: false,
            crash: None,
            spline: None,
            // `Object::fire_ammo`'s unit arm: the missile type's
            // `fire_proj`, its `<INSTANTHIT ammo>` (item 4175).
            gpiece: self.fire_proj(me),
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
            && self.tables.unit_types[ty].price.pop != 0
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

    /// `check_fuel@005e9be0:150`–`:156`'s test of the order's home: the
    /// object answers vslot `+0x4c` (alive) and `can_carry`s the plane.
    fn home_carries(&self, b: usize, u: usize) -> bool {
        self.buildings[b].alive && self.base_can_carry(b, u)
    }

    /// **`Unit::check_fuel@005e9be0`'s re-home** (`:60`–`:136`, item 3990,
    /// `docs/GOLDEN.md` §183): a returning plane whose home is gone or will
    /// not take it searches its owner's buildings in object order (from
    /// 2000) for one alive that `can_carry`s it, the nearest by
    /// `vector_dist` from the plane (a tie keeps the first), and writes it
    /// into the order's `oxx`/`whose`. SEAM: the second loop, over the
    /// owner's units (a carrier home), which this crate keeps no hangar for.
    fn rehome(&mut self, u: usize) -> Option<usize> {
        let who = self.units[u].owner;
        let at = self.units[u].pos;
        let mut bases: Vec<usize> = (0..self.buildings.len())
            .filter(|&b| self.buildings[b].owner == who && self.home_carries(b, u))
            .collect();
        bases.sort_by_key(|&b| self.buildings[b].index);
        let mut best: Option<(i32, usize)> = None;
        for b in bases {
            let p = self.buildings[b].pos;
            let d = vector_dist(p.x - at.x, p.y - at.y);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, b));
            }
        }
        let (_, b) = best?;
        match self.units[u].orders.front_mut().map(|o| &mut o.body) {
            Some(crate::orders::Body::Strafe(sf)) => sf.home = Some(b),
            Some(crate::orders::Body::AirPatrol(p)) => p.home = Some(b),
            Some(crate::orders::Body::AirAttackGround(g)) => g.home = Some(b),
            _ => {}
        }
        Some(b)
    }

    /// `check_fuel@005e9be0:120`–`:131`: no base takes a fixed-wing plane
    /// whose tank is empty, and it dies — vslot `+0x158`, `Unit::die(2, −1,
    /// 0)`. The message and the sound are the console's alone.
    fn out_of_fuel(&mut self, u: usize) {
        self.die_with(u, 2);
    }

    fn land_plane(&mut self, u: usize) {
        crate::air::land_plane_view::land_plane_steps(&mut SimLand { sim: self, u });
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
    /// **An animal** — a flock's bird (`crate::gaia::flock`) — takes two arms a
    /// plane never does (`005e9520`): with its bank settled and its
    /// `spell_time` not negative it skips the whole call every eighth
    /// frame (`frame & 7`, the game's own), and the ground test is an
    /// owner-under-eight's (`(byte)+0x9 > 7` goes straight to the clamp).
    pub(crate) fn bank_plane(&mut self, u: usize, des: Angle, speed: &mut i32, returning: bool) {
        let animal = self.units[u].is_gaia();
        self.bank_plane_as(u, des, speed, returning, animal, false);
    }

    /// [`Sim::bank_plane`] with the two answers it reads off the unit made
    /// arguments (item 3272, the sweep of `Unit::bank_aircraft@005e9520`):
    /// `animal` is the `is_animal` slot (`+0x30`), the owner byte's own
    /// `< 8` test staying the unit's; `heli` the type's helicopter bit,
    /// whose arms the plane's caller does not yet ask for (parked 3272a).
    ///
    /// **A helicopter** reads its bank five times over (`roll × 5` before the
    /// work, `÷ 5` before the store) when it is not an animal, takes no
    /// floor on the divisor, never doubles the wanted bank on a return (it
    /// goes the ground-clearance way instead), and [`air_turn_speed_as`]
    /// reads its bank times five too.
    pub fn bank_plane_as(
        &mut self,
        u: usize,
        des: Angle,
        speed: &mut i32,
        returning: bool,
        animal: bool,
        heli: bool,
    ) {
        let stored = self.units[u].airframe.bank;
        let mut roll = stored.neg();
        if animal {
            if roll.is_zero() && self.units[u].spell_time >= 0 && self.frame & 7 == 0 {
                return;
            }
        } else if heli {
            roll = roll.mulss(F5);
        }
        let heading = self.units[u].movement.heading;
        let d = (des.0 as u32).wrapping_sub(heading.0 as u32);
        let adelta = fold(d);
        let sign_in = if d > 0x8000_0000 { -1i32 } else { 1 };
        let raw = ((self.units[u].movement.turning.type_turn_speed as u32) >> 8)
            .wrapping_mul(self.tables.tuning.unit_turn_speed as u32);
        let divisor = if raw < MIN_TURN && !heli {
            MIN_TURN
        } else {
            raw
        };
        let mut want = Single::from_u32(adelta)
            .divss(Single::from_i32(divisor as i32))
            .mulss(F55)
            .mulss(F_33);
        if F5.gt(want.abs()) {
            want = want.mulss(F_HALF);
        }
        if returning && !heli {
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
            let rate = air_turn_speed_as(raw, stored, sign_out, heli) as i32;
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
                let rate = air_turn_speed_as(raw, stored, sign_out, heli) as i32;
                let h2 = Angle(h.0.wrapping_add(rate.wrapping_mul(sign_out)));
                if fold((h2.0 as u32).wrapping_sub(des.0 as u32)) < d2 {
                    self.units[u].movement.heading = h2;
                }
            }
        }
        // `(s·3 + (s·3 >> 31 & 3)) >> 2`: three quarters, toward zero —
        // returning only.
        if returning && !roll.eq_value(Single::ZERO) {
            *speed = speed.wrapping_mul(3) / 4;
        }
        if heli {
            roll = roll.divss(F5);
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
    /// **A flying target** (`0x5e9114`–`0x5e9206`, item 3991,
    /// `docs/GOLDEN.md` §184) wants the target figure's own `z`
    /// (`guys[0] +0x14`), plus or less `min(300, max(0, (0x600 −
    /// vector_dist(dx, dy))·5))`: plus when the target's object number is
    /// below the striker's, or, the numbers equal, its owner is; the caller
    /// caps it at `cruising_alt` over the ground ahead. The answer is over
    /// `ahead`, as the other two are.
    fn strike_altitude(&self, u: usize, dx: i32, dy: i32, cruise: i32, ahead: i32) -> i32 {
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
        if matches!(
            self.profile(t).domain,
            crate::territory::attrition::Domain::Air
        ) {
            let crate::combat::Obj::Unit(tu) = t else {
                return cruise;
            };
            let off = ((0x600 - vector_dist(dx, dy)) * 5).clamp(0, 300);
            let (mine, theirs) = (&self.units[u], &self.units[tu]);
            let up = if theirs.index == mine.index {
                (theirs.owner as i32) < (mine.owner as i32)
            } else {
                theirs.index < mine.index
            };
            let z = theirs.airframe.z + if up { off } else { -off };
            return z - ahead;
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
    /// The `0x400000` type arm's flying target is
    /// [`Sim::strike_altitude`]'s (item 3991).
    pub(crate) fn pitch_plane(
        &mut self,
        u: usize,
        dx: i32,
        dy: i32,
        aim_z: i32,
        speed: &mut i32,
        returning: bool,
    ) {
        // **An animal** — a flock's bird (`crate::gaia::flock`) — skips the
        // whole call with its pitch level and its `spell_time` not
        // negative on `(frame + 3) & 7 == 0` (`005e8de0`, the opening
        // test), and inside it takes three arms of its own: no cap at
        // `cruising_alt` over the ground ahead, a pitch rate of ±10 by the
        // height owed rather than the plane's glide, and no pitch speed
        // cut. The returning arm's halving inside `0x600` is not behind
        // the test and is taken.
        let animal = self.units[u].is_gaia();
        // **A helicopter** (`local_20`, `type +0x2b4 & 0x20`; item 3990):
        // no halving inside `0x600`, no `−20` toward the ground, a step of
        // 25 a frame by the pitch's sign that lands on the altitude wanted
        // inside 25, and its pitch stored 0.
        let heli = self.is_helicopter(u);
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
            if dist < 0x600 && !heli {
                *speed /= 2;
            }
            (want, dist)
        } else {
            (
                self.strike_altitude(u, dx, dy, cruise, ahead) + ahead,
                0x240,
            )
        };
        let want = if animal {
            want
        } else {
            want.min(cruise + ahead)
        };
        let ground = self.world.tile_z(at.tile());
        let target = want.max(ground + if returning { 50 } else { 200 });
        let mut pitch = self.units[u].airframe.pitch;
        let extra = if returning && !heli && target < ground + 500 {
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
        if heli && !sum.eq_value(Single::ZERO) {
            let step = if sum.gt(Single::ZERO) { 25 } else { -25 };
            let af = &mut self.units[u].airframe;
            af.last_z = af.z;
            af.z = if (af.z - target).abs() < 25 {
                target
            } else {
                af.z + step
            };
        } else if !sum.eq_value(Single::ZERO) {
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
        af.pitch = if heli { Single::ZERO } else { pitch };
    }
}

/// What `UnitData::is_flying_low@0060a140` makes of its front order's type
/// (`order_type@00616e80`, `60a18d`): `STRAFE` (0x10), `AIR_ATTACK_GROUND`
/// (0x18), `NONE` (0), or any other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LowFront {
    None,
    Strafe,
    AirAttackGround,
    Other,
}

/// **`UnitData::is_flying_low@0060a140`'s steps** (the listing,
/// `60a140`..`60a2ff`; item 2368's sweep holds them): a fixed-wing plane
/// ([`Sim::is_fixed_wing`]'s three type tests) asks whether it is on the map,
/// then its front order's type. `NONE` is not low. A `STRAFE` asks its target
/// (slot `0x100`: `None` when the slot's index or owner is negative or the
/// object is not active), an `AIR_ATTACK_GROUND` its point (slot `0xd4`), and
/// either is low when the point is within [`LOW_REACH`] (`vector_dist` of the
/// absolute differences, `< 0x900`). Failing that — and for any other order —
/// the air order (slot `0xfc`): `Some` home when it is `returning` with an
/// active home, low within the same reach. `cx` is the caller's context, the
/// closures' to read.
#[allow(clippy::too_many_arguments)]
pub(crate) fn flying_low_steps<C>(
    cx: &mut C,
    fixed_wing: bool,
    here: impl Fn(&mut C) -> Pos,
    on_map: impl FnOnce(&mut C) -> bool,
    front: impl FnOnce(&mut C) -> LowFront,
    strafe_target: impl FnOnce(&mut C) -> Option<Pos>,
    ground_point: impl FnOnce(&mut C) -> Pos,
    air_home: impl FnOnce(&mut C) -> Option<Pos>,
) -> bool {
    if !fixed_wing || !on_map(cx) {
        return false;
    }
    let near = |p: Pos, h: Pos| vector_dist(p.x - h.x, p.y - h.y) < LOW_REACH;
    let point = match front(cx) {
        LowFront::None => return false,
        LowFront::Strafe => strafe_target(cx),
        LowFront::AirAttackGround => Some(ground_point(cx)),
        LowFront::Other => None,
    };
    if let Some(p) = point
        && near(p, here(cx))
    {
        return true;
    }
    match air_home(cx) {
        Some(p) => near(p, here(cx)),
        None => false,
    }
}

/// `UnitData::is_flying_high@0060a310`'s order: a fixed-wing plane ([`Sim::is_fixed_wing`]'s three type tests)
/// asks whether it is on the map, and only on the map whether it flies low, answering not low; anything else is
/// not high and asks nothing (item 2090's sweep holds the asks).
pub(crate) fn flying_high_by(
    fixed_wing: bool,
    on_map: impl FnOnce() -> bool,
    low: impl FnOnce() -> bool,
) -> bool {
    fixed_wing && on_map() && !low()
}
/// [`crate::air::land_plane_view::LandPlaneView`] over the sim (item 3396): the
/// walk `Unit::land_plane@005e9950` makes, on a plane.
///
/// **Not modelled**: the `SpecialAnimOrder` the original adds, the home words it
/// writes into it and the landing sound are not modelled as such: the order
/// is run at once by the `work` at the function's tail (`docs/ORDERS.md`
/// §33.5), which is [`Sim::go_inside`], and a sound is the renderer's.
/// `is_animal` answers false: a flock's bird lands in `flock.rs`
/// (`flockbird_dies`) and the callers that hold one route it there.
struct SimLand<'a> {
    sim: &'a mut Sim,
    u: usize,
}

impl crate::air::land_plane_view::LandPlaneView for SimLand<'_> {
    type Home = usize;

    fn is_animal(&mut self) -> bool {
        false
    }
    fn die_slot_is_unit_die(&mut self) -> bool {
        true
    }
    fn object_die(&mut self) {}
    fn call_die_slot(&mut self) {}

    fn air_home(&mut self) -> Option<usize> {
        self.sim.current_air(self.u).and_then(|a| a.home)
    }
    fn clear_returning(&mut self) {
        self.sim.set_returning(self.u, false);
    }
    fn is_strafe(&mut self) -> bool {
        self.sim.current_strafe(self.u).is_some()
    }
    fn strafe_has_target(&mut self) -> bool {
        self.sim
            .current_strafe(self.u)
            .is_some_and(|sf| sf.target.is_some())
    }
    fn strafe_mandatory(&mut self) -> bool {
        self.sim
            .current_strafe(self.u)
            .is_some_and(|sf| sf.mandatory)
    }
    fn clear_orders(&mut self) {
        self.sim.clear_orders(self.u);
    }
    fn kill_current_order(&mut self) {
        self.sim.kill_current_order(self.u);
    }
    fn repeat_air(&mut self, home: usize) -> bool {
        self.sim.buildings[home].repeat_air
    }
    fn clear_order_action_bit(&mut self) {
        if let Some(o) = self.sim.units[self.u].orders.front_mut() {
            o.flags &= !crate::orders::flag::ACTION;
        }
    }
    fn clear_path_state(&mut self) {
        self.sim.units[self.u].path.clear();
    }
    fn close_orders(&mut self) {
        self.sim.close_orders(self.u);
    }
    fn clear_partial_path(&mut self) {
        self.sim.clear_partial_path(self.u);
    }
    fn update_action(&mut self) {
        self.sim.update_action(self.u);
    }
    fn is_helicopter(&mut self) -> bool {
        false
    }
    fn home_slot_34(&mut self, _home: usize) -> i32 {
        0
    }
    fn add_spec_anim_order(&mut self, _home: usize, _data1: i32, _data2: i32) {}
    fn set_special_home(&mut self, _home: usize) {}
    fn home_visible(&mut self, _home: usize) -> bool {
        false
    }
    fn play_landing_sound(&mut self, _home: usize) {}
    fn go_inside(&mut self, home: usize) {
        self.sim.go_inside(self.u, home);
    }
}

#[cfg(test)]
mod tests;

/// The launch line (item 836, `docs/ORDERS.md` §38): chapter seventeen's
/// cast on a world large enough to hold the base's point.
#[cfg(test)]
mod launch_tests;

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
    pub releases: Vec<(u32, i8, bool, i32)>,
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
        let id = self.tables.build_types.get(bt)?.tree?;
        self.tables.tech_tree.types.get(id)?.wall_cycle.as_ref()
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
        for &(at, node, harmless, piece) in &cycle.releases {
            let at = i32::try_from(at).unwrap_or(i32::MAX);
            if last < at && at <= cur {
                self.wall_round(b, t, cycle.launch, node, harmless, piece);
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
        piece: i32,
    ) {
        let me = crate::combat::Obj::Building(b);
        let at = self.buildings[b].pos;
        let z = self.world.tile_z(at.tile());
        let (dx, dy, dz) = launch.unwrap_or((0, 0, 0));
        let from = Pos::new(at.x + dx, at.y + dy);
        let to = self.pos_of(t);
        let angle = find_angle(to.x - from.x, to.y - from.y);
        let frame = self.frame;
        self.fire_ammo_pub(
            me,
            t,
            angle,
            frame,
            from,
            z + dz,
            node,
            harmless,
            false,
            piece,
        );
        for p in self.combat.projectiles.iter_mut() {
            if p.shooter == me && p.cur_time == 0 && !p.spent {
                p.num_guys = 0;
            }
        }
    }

    /// A unit of the air domain that is neither a Helicopter nor a missile —
    /// `is_flying_low`'s and `Ammo::init`'s three type tests (`+0x218` 2,
    /// `+0x2b4 & 0x20` clear, `+0x1e4 & 0x8000000` clear).
    pub(crate) fn is_fixed_wing(&self, u: usize) -> bool {
        let p = self.profile(crate::combat::Obj::Unit(u));
        matches!(p.domain, crate::territory::attrition::Domain::Air)
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
        let front = || self.current_order(u).map(|o| o.body);
        flying_low_steps(
            &mut (),
            self.is_fixed_wing(u),
            |_| unit.pos,
            |_| unit.alive() && unit.inside.is_none(),
            |_| match front() {
                None => LowFront::None,
                Some(Body::Strafe(_)) => LowFront::Strafe,
                Some(Body::AirAttackGround(_)) => LowFront::AirAttackGround,
                Some(_) => LowFront::Other,
            },
            |_| match front() {
                Some(Body::Strafe(sf)) => sf
                    .target
                    .filter(|&t| self.active(t))
                    .map(|t| self.pos_of(t)),
                _ => None,
            },
            |_| match front() {
                Some(Body::AirAttackGround(g)) => g.at,
                _ => unreachable!("asked only under an AIR_ATTACK_GROUND"),
            },
            |_| {
                self.current_air(u)
                    .filter(|a| a.returning)
                    .and_then(|a| a.home)
                    .filter(|&b| self.buildings[b].alive)
                    .map(|b| self.buildings[b].pos)
            },
        )
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
            && matches!(
                self.profile(target).domain,
                crate::territory::attrition::Domain::Air
            )
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
    /// **The falling round is laid** (item 2609, `docs/AI.md` §228): its
    /// `who`/`o` are the plane's, so it holds the dead plane's object
    /// number for its whole fall ([`Sim::hold_dead_slot`],
    /// [`Sim::hold_frames_tick`]), and it takes a slot in the ammo pool.
    /// [`Sim::crash_round`] is `init_crash` itself.
    ///
    /// **It strikes where it comes down** (item 2677, `docs/AI.md` §230):
    /// `init_crash` sets `splash_area` (`+0x30`) to 2 and clears flag
    /// `0x10`, so `Ammo::inc_time`'s tail runs `Ammo::do_damage` on it —
    /// the splash arm, no target, the dead plane as the shooter.
    pub(crate) fn crashes(&mut self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if self.tables.unit_types[t].cols.cat != CAT_AIR
            || self
                .profile(crate::combat::Obj::Unit(u))
                .has(crate::combat::mask::MISSILE)
        {
            return false;
        }
        self.crash_round(u, t);
        true
    }

    /// `Ammo::init_crash@0067b800`, from the listing (`67b909`–`67bb60`).
    ///
    /// - **The fall**: `sqrtf((float)(−2 × z) / GRAV_Z)` frames, `z` the
    ///   figure's altitude (`+0x14`).
    /// - **The landing**: the object's point plus the type's `moves`
    ///   (`+0x2c0` × `UNIT_MOVE_SPEED`) along the figure's angle, each axis
    ///   `sin_table` of it times the fall, truncated — `x` on the angle and
    ///   `y` on the angle less a quarter turn — and `WorldData::restrict`ed.
    /// - **`ez`**: `find_data_z` at the landing.
    /// - **`total_time`**: `sqrtf((float)(dx² + dy²)) / (float)moves`
    ///   truncated, from the **figure's** point to the landing, at least 1.
    /// - Then the one draw from the game's stream ([`SITE_CRASH_ROLL`]),
    ///   and the bank pair from a local `Random` seeded by `x + y + z`.
    ///
    /// run405 backs every input: `0/6` from (21949, 18537, 1783) takes
    /// `total_time` 18 and bank (23, 5), `0/7` from (22866, 16298, 1624)
    /// 17 and (0, 1).
    fn crash_round(&mut self, u: usize, t: usize) {
        let unit = &self.units[u];
        let figure = unit.movement.body.pos;
        let z = unit.airframe.z;
        let angle = unit.movement.facing;
        let object = unit.pos;
        let owner = unit.owner;
        let v = self.tables.unit_types[t]
            .moves
            .wrapping_mul(crate::combat::UNIT_MOVE_SPEED);
        let fall = Single::from_i32((-z).wrapping_mul(2))
            .divss(crate::combat::GRAV_Z)
            .sqrtss();
        let drift = |c: i32| Single::from_i32(c).mulss(fall).to_i32();
        let (dx, dy) = if v == 0 {
            (0, 0)
        } else {
            (
                drift(sin_component(angle, v)),
                drift(sin_component(Angle(angle.0.wrapping_sub(Angle::EAST.0)), v)),
            )
        };
        let landing = self.restrict_pos(Pos::new(object.x + dx, object.y + dy));
        let ez = self.ground_z(landing);
        let (ex, ey) = (landing.x - figure.x, landing.y - figure.y);
        let total_time = if v == 0 {
            // SEAM: a zero divisor is an infinity the original truncates to
            // `0x80000000`; no aircraft type has `moves` 0.
            1
        } else {
            Single::from_i32(ex.wrapping_mul(ex).wrapping_add(ey.wrapping_mul(ey)))
                .sqrtss()
                .divss(Single::from_i32(v))
                .to_i32()
                .max(1)
        };
        self.mark(SITE_CRASH_ROLL);
        let rolling = self.rng.roll() % 7 - 3;
        let mut local =
            crate::combat::Rng::new(figure.x.wrapping_add(figure.y).wrapping_add(z) as u32);
        let bank = (local.get(-0x1e, 0x1e), local.get(-0x1e, 0x1e));
        self.add_ammo(crate::combat::Projectile {
            shooter: crate::combat::Obj::Unit(u),
            owner,
            target: None,
            launch: figure,
            landing,
            cur_time: 0,
            total_time,
            accuracy: 0,
            angle,
            // `init_crash` writes `+0x30 = 2` (`docs/AI.md` §230): a crash
            // is a two-tile splash where it comes down.
            splash_area: CRASH_SPLASH_AREA,
            num_guys: 1,
            air: false,
            rolling: false,
            missed: false,
            harmless: false,
            sz: z,
            ez,
            v1z: Single::ZERO,
            slot: 0,
            flame: false,
            punctures: false,
            spent: false,
            crash: Some(crate::combat::Crash { bank, rolling }),
            spline: None,
            // `Ammo::init_crash@0067b800:44`: the plane's own vslot
            // `+0x34`, `UnitData::get_gpiece` — its guy 0's piece (item 4175).
            gpiece: self.units[u].guys.first().map_or(-1, |g| g.gpiece),
        });
    }

    /// `ObjectData::is_jammed@00653660`, as far as this crate reaches: no
    /// leader here carries `leader_flags & 0x40000`, so never.
    fn is_jammed(&self, _u: usize) -> bool {
        false
    }

    /// `UnitData::is_flying_high@0060a310`: a fixed-wing unit on the map that
    /// is not low. Its caller is `valid_target`'s air ladder
    /// (`docs/COMBAT.md` §61, item 4007).
    pub(crate) fn is_flying_high(&self, u: usize) -> bool {
        let unit = &self.units[u];
        flying_high_by(
            self.is_fixed_wing(u),
            || unit.alive() && unit.inside.is_none(),
            || self.is_flying_low(u),
        )
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
            if matches!(sp.domain, crate::territory::attrition::Domain::Air) {
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
mod flak_tests;

/// **The Modern Infantry's step** (item 1109): `Unit::do_move`'s
/// `5f88b5`..`5f88cb`, `× 5 / 4` truncating on `get_speed`'s answer for a
/// type `is_modern_infantry` answers. It lives beside the air line because
/// chapter thirty-eight is where a Modern Infantry first walked in a
/// capture (run404's `1/6`, a first step of 42 on a speed of 34).
/// What the home object's flag byte says (`home_flagged`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))] // a plane's home is a building: the sweep alone builds the others
pub(crate) enum Home {
    /// No home (`+0x44 < 0`).
    Absent,
    /// A home object whose flag byte is clear.
    Plain,
    /// A flagged one, at this position.
    At(i32, i32),
}

/// The register the look's `find_building_at` is handed in its last
/// argument slot, which the compiler leaves unset: whatever `ecx` held on
/// the path that reached the call. The simulation reads none of them; the
/// sweep records the call's arguments, so the steps name the paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stale {
    /// The unit, after a `this`-call (not a `FIGHTERBOMBER`).
    This,
    /// The home index (`+0x44`), negative.
    HomeIdx,
    /// The home object, its flag clear.
    HomeObject,
    /// The world, after `restrict`.
    World,
}

/// What `Unit::do_air_patrol@005ea620` asks of the world, in the order it
/// asks (item 3289): [`air_patrol_steps`] is the original's control flow and
/// the view is what it reads and calls. `Sim::do_air_patrol` runs it over
/// [`SimPatrol`]; `sweep/unit_do_air_patrol.rs` runs it over the emulated
/// original's windows and stubs.
pub(crate) trait PatrolView {
    /// An `(index, list)` pair of `ObjectsData`, opaque to the steps.
    type Target: Copy;
    /// `this->vtable[0x180](order, 0)`, the animal's think; first call.
    fn think(&mut self);
    /// `x_pos.length` (`+0x8`).
    fn len(&self) -> i32;
    /// `y_pos.length` (`+0x24`), which the last point's `y` is indexed by.
    fn ylen(&self) -> i32;
    /// `PatrolOrder::waypoint` (`+0x3c`).
    fn waypoint(&self) -> i32;
    fn set_waypoint(&mut self, w: i32);
    /// `AirOrder::returning` (`+0x58`), read live.
    fn returning(&self) -> bool;
    fn x_at(&self, i: i32) -> i32;
    fn y_at(&self, i: i32) -> i32;
    /// `this->vtable[0x30]`, `SubObjectData::is_animal`.
    fn is_animal(&mut self) -> bool;
    /// The home object's `vtable[8]` and, when it answers, its position
    /// (`+0x10/+0x14` less the key): the carrier-relative goal.
    fn carrier(&mut self) -> Option<(i32, i32)>;
    /// The home object's flag byte (`+8 & 1`) and, when set, its position:
    /// the carrier-relative search and look.
    fn home_flagged(&mut self) -> Home;
    /// `WorldData::restrict`.
    fn restrict(&mut self, x: i32, y: i32) -> (i32, i32);
    /// `Unit::do_air_physics(order, x, y)`; a patrol going home hands
    /// `None` (the original's -1, -1).
    fn air_physics(&mut self, goal: Option<(i32, i32)>) -> bool;
    /// `vector_dist(this - (x, y))`.
    fn dist_to(&self, x: i32, y: i32) -> i32;
    /// The unit's order count (`+0xd8`).
    fn orders(&self) -> i32;
    fn kill_current_order(&mut self);
    /// `short +0xa` plus the game frame.
    fn phase(&self) -> i32;
    /// `ObjectData::is(0x134)`.
    fn fighter_bomber(&mut self) -> bool;
    /// `ObjectData::is(0x130)`.
    fn bomber(&mut self) -> bool;
    /// `find_new_bomber_target(x, y, -1, p4, &out)`; `p4` the stale `ecx`
    /// the first call hands on (`None`) or the `y` the fallback passes.
    fn find_bomber(&mut self, x: i32, y: i32, p4: Option<i32>) -> Self::Target;
    /// `find_new_air_target(x, y, -1, -1, &out)`.
    fn find_air(&mut self, x: i32, y: i32) -> Self::Target;
    /// The unit's own position, `+0x10/+0x14` less the key.
    fn self_pos(&self) -> (i32, i32);
    /// `game->semaphore[1] & 2`.
    fn semaphore_fallback(&self) -> bool;
    fn valid_target(&mut self, t: Self::Target) -> bool;
    /// The target's type `domain == 2`.
    fn target_flies(&self, t: Self::Target) -> bool;
    /// `add_strafe_order(t, home, mandatory, QUEUE_FIRST, 0)`.
    fn strafe(&mut self, t: Self::Target, mandatory: bool);
    /// `ObjectsData::find_building_at(tile of (x, y), SEARCH_ENEMY, who,
    /// FILTER_ALL)`.
    fn find_building_at(&mut self, x: i32, y: i32, stale: Stale) -> Option<Self::Target>;
    /// The building's `ever_seen` (`+0x62`) holds this player's bit.
    fn target_ever_seen(&mut self, t: Self::Target) -> bool;
    /// `spell_time` (`+0x98`).
    fn caster_spell_time(&self) -> i32;
    fn set_spell_time(&mut self, v: i32);
}

/// **`Unit::do_air_patrol@005ea620`**, the control flow (item 3289; the
/// sweep `sweep/unit_do_air_patrol.rs` holds it to the original over the
/// emulated windows): the animal think and the waypoint's reset; the goal
/// of a patrol not going home, carrier-relative when its home is a unit;
/// `do_air_physics`; then the animal tail (`spell_time` 0 to 1), or, not
/// going home, the arrival and the sixteen-frame search, and for any
/// patrol the thirty-two-frame look.
pub(crate) fn air_patrol_steps<V: PatrolView>(v: &mut V) {
    v.think();
    if v.len() <= v.waypoint() {
        v.set_waypoint(0);
    }
    let mut goal = None;
    if !v.returning() {
        let w = v.waypoint();
        let (mut gx, mut gy) = (v.x_at(w), v.y_at(w));
        if !v.is_animal()
            && let Some((ox, oy)) = v.carrier()
        {
            (gx, gy) = v.restrict(gx + ox, gy + oy);
        }
        goal = Some((gx, gy));
    }
    if !v.air_physics(goal) {
        return;
    }
    if v.is_animal() {
        if v.caster_spell_time() == 0 {
            v.set_spell_time(1);
        }
        return;
    }
    // `+0x58` is read again after the flight: an empty tank sets it there. (The
    // original reads it a third time after the waypoint steps on; nothing writes
    // it between, and the sweep's mutations of that read all survived.)
    if !v.returning() {
        if let Some((gx, gy)) = goal
            && v.dist_to(gx, gy) < 0x240
        {
            if v.waypoint() < v.len() - 1 {
                let w = v.waypoint();
                v.set_waypoint(w + 1);
            } else if v.orders() > 1 {
                v.kill_current_order();
                return;
            }
        }
        if v.phase() & 15 == 0 {
            let (mut sx, mut sy) = (v.x_at(v.len() - 1), v.y_at(v.ylen() - 1));
            if v.fighter_bomber()
                && let Home::At(ox, oy) = v.home_flagged()
            {
                (sx, sy) = v.restrict(sx + ox, sy + oy);
            }
            let (px, py) = v.self_pos();
            let mut t;
            if v.bomber() {
                t = v.find_bomber(sx, sy, None);
                if !v.valid_target(t) && v.semaphore_fallback() {
                    t = v.find_air(px, py);
                }
            } else {
                t = v.find_air(sx, sy);
                if !v.valid_target(t) && v.semaphore_fallback() {
                    t = v.find_bomber(px, py, Some(py));
                }
            }
            if v.valid_target(t) && (v.waypoint() == v.len() - 1 || v.target_flies(t)) {
                v.strafe(t, false);
                return;
            }
        }
    }
    if v.phase() & 31 == 0 {
        let w = v.waypoint();
        let (mut lx, mut ly) = (v.x_at(w), v.y_at(w));
        let mut stale = Stale::This;
        if v.fighter_bomber() {
            match v.home_flagged() {
                Home::Absent => stale = Stale::HomeIdx,
                Home::Plain => stale = Stale::HomeObject,
                Home::At(ox, oy) => {
                    (lx, ly) = v.restrict(lx + ox, ly + oy);
                    stale = Stale::World;
                }
            }
        }
        if let Some(b) = v.find_building_at(lx, ly, stale)
            && v.target_ever_seen(b)
        {
            v.strafe(b, true);
        }
    }
}

/// [`PatrolView`] over the simulation: a plane's patrol.
///
/// SEAM: the animal arms (`is_animal`, the `+0x180` think and the
/// `spell_time` latch: a bird is `do_idle`'s, `crate::gaia`), the home as a
/// carrier unit (`carrier`, `home_flagged`: a hangar is a building, whose
/// `vtable[8]` answers false), and `game->semaphore & 2`.
struct SimPatrol<'a> {
    sim: &'a mut Sim,
    u: usize,
    frame: i64,
}

impl SimPatrol<'_> {
    fn patrol(&self) -> crate::orders::AirPatrolOrder {
        match self.sim.current_order(self.u).map(|o| o.body) {
            Some(Body::AirPatrol(p)) => p,
            _ => unreachable!("do_air_patrol runs on a patrol"),
        }
    }

    fn point(&self, i: i32) -> Pos {
        let p = self.patrol();
        p.points[usize::try_from(i).unwrap_or(0).min(p.points.len() - 1)]
    }
}

impl PatrolView for SimPatrol<'_> {
    type Target = Option<crate::combat::Obj>;

    fn think(&mut self) {}

    fn len(&self) -> i32 {
        i32::from(self.patrol().len)
    }

    fn ylen(&self) -> i32 {
        self.len()
    }

    fn waypoint(&self) -> i32 {
        self.patrol().waypoint as i32
    }

    fn set_waypoint(&mut self, w: i32) {
        self.sim.set_waypoint(self.u, w as usize);
    }

    fn returning(&self) -> bool {
        self.patrol().returning
    }

    fn x_at(&self, i: i32) -> i32 {
        self.point(i).x
    }

    fn y_at(&self, i: i32) -> i32 {
        self.point(i).y
    }

    fn is_animal(&mut self) -> bool {
        false
    }

    fn carrier(&mut self) -> Option<(i32, i32)> {
        None
    }

    fn home_flagged(&mut self) -> Home {
        Home::Absent
    }

    fn restrict(&mut self, x: i32, y: i32) -> (i32, i32) {
        let xs = self.sim.world.width() * UNITS_PER_CELL;
        let ys = self.sim.world.height() * UNITS_PER_CELL;
        (x.clamp(0, xs - 1), y.clamp(0, ys - 1))
    }

    fn air_physics(&mut self, goal: Option<(i32, i32)>) -> bool {
        let goal = goal.map(|(x, y)| Pos::new(x, y));
        self.sim.plane_air_physics(self.u, goal, self.frame) != Flew::Done
    }

    fn dist_to(&self, x: i32, y: i32) -> i32 {
        let at = self.sim.units[self.u].pos;
        vector_dist(at.x - x, at.y - y)
    }

    fn orders(&self) -> i32 {
        self.sim.units[self.u].orders.len() as i32
    }

    fn kill_current_order(&mut self) {
        self.sim.kill_current_order(self.u);
    }

    fn phase(&self) -> i32 {
        (i64::from(self.sim.units[self.u].index) + self.frame) as i32
    }

    fn fighter_bomber(&mut self) -> bool {
        false
    }

    fn bomber(&mut self) -> bool {
        self.sim.is_bomber(self.u)
    }

    fn find_bomber(&mut self, x: i32, y: i32, _p4: Option<i32>) -> Self::Target {
        self.sim.find_new_bomber_target(self.u, Pos::new(x, y))
    }

    fn find_air(&mut self, x: i32, y: i32) -> Self::Target {
        self.sim.find_new_air_target(self.u, Pos::new(x, y))
    }

    fn self_pos(&self) -> (i32, i32) {
        let at = self.sim.units[self.u].pos;
        (at.x, at.y)
    }

    fn semaphore_fallback(&self) -> bool {
        false
    }

    fn valid_target(&mut self, t: Self::Target) -> bool {
        t.is_some_and(|t| self.sim.valid_target(crate::combat::Obj::Unit(self.u), t))
    }

    fn target_flies(&self, t: Self::Target) -> bool {
        t.is_some_and(|t| {
            matches!(
                self.sim.profile(t).domain,
                crate::territory::attrition::Domain::Air
            )
        })
    }

    fn strafe(&mut self, t: Self::Target, mandatory: bool) {
        let home = self.patrol().home;
        self.sim.add_strafe_order(
            self.u,
            t,
            home,
            mandatory,
            crate::orders::QueuePos::First,
            false,
        );
    }

    fn find_building_at(&mut self, x: i32, y: i32, _stale: Stale) -> Option<Self::Target> {
        self.sim
            .enemy_building_on_tile(self.u, Pos::new(x, y))
            .map(|b| Some(crate::combat::Obj::Building(b)))
    }

    fn target_ever_seen(&mut self, t: Self::Target) -> bool {
        matches!(t, Some(crate::combat::Obj::Building(b))
            if self.sim.buildings[b].ever_seen & Sim::who_bit(self.sim.units[self.u].owner) != 0)
    }

    fn caster_spell_time(&self) -> i32 {
        i32::from(self.sim.units[self.u].spell_time)
    }

    fn set_spell_time(&mut self, v: i32) {
        self.sim.units[self.u].spell_time = v as _;
    }
}

#[cfg(test)]
mod infantry_step_tests;

// The files of `air/` (item 4012, `docs/STRUCTURE.md` §3).
pub mod airbase;
pub mod carrier_queue;
pub mod land_plane_view;
