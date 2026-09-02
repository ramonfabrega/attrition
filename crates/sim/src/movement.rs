//! Movement: how fast a unit moves, which way it faces, and how far one frame
//! carries it.
//!
//! Not how it decides where to go. Pathfinding, collision (`collide.rs`)
//! and formations are
//! separate mechanics; `docs/MOVEMENT.md` surveys them and this module does not
//! implement them.
//!
//! # Two positions
//!
//! The original keeps two positions per unit and moves them with different
//! code. The **unit's** position is what every other mechanic reads, and
//! [`move_step`] advances it at the unit's speed. The **body's** position is
//! the figure drawn for the unit, and [`body_follow`] puts it where the unit
//! now is — guy 0 never chases, it snaps, recording the length of the jump as
//! its `last_speed`. That bookkeeping, and `avg_speed` over it, is what the
//! unit's turn rate reads, which is why the body is modelled at all.
//!
//! # Two angles
//!
//! And two angles per unit, in different fields. `GuyData::angle` is the
//! **facing** — what the step is taken along, turned by at most the turn rate
//! each frame. `UnitData::angle` is the **heading** — the bearing to the
//! current destination, which `Unit::set_angle` writes outright at the top of
//! every step, and which the body turns toward while it stands still.
//!
//! # Units
//!
//! A speed is **position units per frame** — 1/768 of a cell, 1/192 of a tile.
//! A unit type's `MOVES` field is already that number: `UNIT_MOVE_SPEED`, whose
//! annotation reads `1/192 tile`, is the identity converter, because one 1/192
//! of a tile *is* one position unit.
//!
//! An angle is a **signed 32-bit binary angle**, so the full circle is 2^32 and
//! wrapping is free. North is zero and y increases southward. A type's
//! `TURN_SPEED` is degrees in the data and goes through [`degrees_to_angle`]
//! once, at load.
//!
//! # The one thing to be careful with
//!
//! [`quarter_lookup`]'s interpolation multiply **overflows a 32-bit register,
//! and the result depends on it.** At the axis angles the table's last entry
//! interpolates toward its first, and only the wrap brings the answer back to
//! exactly one. Widen that multiply and every unit ordered due north stops
//! moving. It is commented where it happens.

use crate::tuning::Tuning;
use crate::world::{Pos, UNITS_PER_TILE, vector_dist};

/// A signed 32-bit binary angle: the full circle is 2^32.
///
/// Wrapping is the point. Adding a quarter turn to a heading can never
/// overflow into nonsense, and the difference of two headings is their shortest
/// signed separation with no normalisation step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Angle(pub i32);

impl Angle {
    pub const NORTH: Angle = Angle(0);
    /// What `Unit::init@00612100` writes into `UnitData::angle` and
    /// `dest_angle` before anything has turned the unit: `0x55555555`, a
    /// third of a circle, and a literal in the source with nothing behind it.
    /// Every unit the original creates faces 120° until its first order.
    pub const INITIAL: Angle = Angle(0x5555_5555);
    pub const EAST: Angle = Angle(0x4000_0000);
    pub const SOUTH: Angle = Angle(i32::MIN);
    pub const WEST: Angle = Angle(-0x4000_0000);

    /// A quarter turn later. `cos` is `sin` of this.
    pub const fn quarter_turn(self) -> Angle {
        Angle(self.0.wrapping_add(Angle::EAST.0))
    }

    /// The signed shortest way round from `self` to `to`.
    pub const fn to(self, to: Angle) -> i32 {
        to.0.wrapping_sub(self.0)
    }
}

/// One degree, as the original's `degrees_to_angle` counts it: `0xb60b60`,
/// which is 2^32/360 truncated.
pub const ONE_DEGREE: i32 = 0x00b6_0b60;
/// Forty-five degrees — exactly an eighth of a turn.
const FORTY_FIVE: u32 = 0x2000_0000;
/// Eighty degrees, as the step compares it.
const EIGHTY: u32 = 0x38e3_8e3a;
/// A type turning slower than this — twenty degrees, as `degrees_to_angle`
/// makes it less one — gets the "slow" doubling of its turn-in-place ranges.
const SLOW_TURN_BELOW: i32 = 0x0e38_e38c;

/// Degrees to a binary angle, the way `UnitType::init` converts a type's
/// `TURN_SPEED` at load.
///
/// Not a multiply by 2^32/360. The original decomposes the angle into exact
/// pieces — quarter turns, then 45°, 30°, 15°, whole degrees — and adds
/// **three units for every complete five degrees in the remainder** to make up
/// what `0xb60b60` per degree loses to truncation. So `degrees_to_angle(45)`
/// is exactly an eighth of a turn, `degrees_to_angle(20)` is one more than the
/// constant the step compares it against, and the function is reproduced
/// rather than replaced because the step's thresholds were tuned against its
/// output.
pub const fn degrees_to_angle(degrees: i32) -> Angle {
    let (q90, r90) = (degrees / 90, degrees % 90);
    let (q45, r45) = (r90 / 45, r90 % 45);
    let (q30, r30) = (r45 / 30, r45 % 30);
    let (q15, r15) = (r30 / 15, r30 % 15);
    let q5 = r15 / 5;
    // Quarter turns wrap: 180° is `i32::MIN` and 360° is zero.
    let a = q90
        .wrapping_mul(2)
        .wrapping_add(q45)
        .wrapping_mul(FORTY_FIVE as i32)
        .wrapping_add(q30.wrapping_mul(0x1555_5555))
        .wrapping_add(q15.wrapping_mul(0x0aaa_aaaa))
        .wrapping_add(q5.wrapping_mul(3))
        .wrapping_add(r15.wrapping_mul(ONE_DEGREE));
    Angle(a)
}

/// The angle from the origin to `(dx, dy)`, with y increasing southward.
///
/// An integer arctangent with no table: a ratio of the shorter leg to the
/// longer, scaled by a straight-line correction, then placed in its octant.
///
/// It is exact on the axes by construction and worst at 45 degrees, where it
/// reads 0.94% high. That error is part of the game's behaviour — a true
/// `atan2` would change every heading and every position downstream of one.
pub const fn find_angle(dx: i32, dy: i32) -> Angle {
    // The original negates dy immediately: the stored y axis points south and
    // the angle is measured the other way.
    let cy = -dy;
    if dx == 0 {
        return if cy > 0 { Angle::NORTH } else { Angle::SOUTH };
    }
    if cy == 0 {
        return if dx > 0 { Angle::EAST } else { Angle::WEST };
    }

    let (a, b) = (dx.abs(), cy.abs());
    let (hi, lo) = if b < a { (a, b) } else { (b, a) };
    // True when |dx| <= |dy| — the y-dominant half of the octant pair.
    let y_dominant = a <= b;

    let t = (lo * 0x4000) / hi;
    let corrected = 0x2800 - (((0x1333 - t).abs() * 0xb00) >> 14);
    let m = (corrected * t) & !0x3fff;
    let q = m * 4;

    // Octant placement. Reads as a table because it is one: eight cases, by
    // the sign of each axis and by which one dominates. A half turn is
    // `i32::MIN`, and reaching it needs wrapping because `0x80000000` is not a
    // positive `i32`.
    const HALF: i32 = i32::MIN;
    const QUARTER: i32 = 0x4000_0000;
    Angle(match (dx > 0, cy > 0, y_dominant) {
        (false, false, true) => q.wrapping_add(HALF),
        (false, false, false) => -q - QUARTER,
        (false, true, true) => -q,
        (false, true, false) => q - QUARTER,
        (true, true, true) => q,
        (true, true, false) => QUARTER - q,
        (true, false, true) => (-q).wrapping_add(HALF),
        (true, false, false) => q + QUARTER,
    })
}

/// The original's quarter-wave sine table, as the integers it holds at runtime.
///
/// `trig_init` builds this at startup with
/// `sin(i * 1.570796327 / 255.0) * 65535.0`, truncated. Three things about that
/// are the original's and are reproduced rather than corrected:
///
/// - the `1.570796327` is a typed-in decimal, not the library's pi/2;
/// - the divisor is **255** while the index runs 0..=255, so the table reaches
///   1.0 one step early and runs slightly ahead of a true sine;
/// - the entries are truncated toward zero, not rounded.
///
/// It is pinned here as integers rather than recomputed. The original computes
/// it once from doubles, so the integers *are* the specification, and pinning
/// them keeps the simulation independent of any platform's `sin`.
pub const SIN_TABLE: [i32; 256] = [
    0, 403, 807, 1211, 1614, 2018, 2421, 2824, 3228, 3631, 4034, 4437, 4839, 5242, 5644, 6046,
    6448, 6850, 7251, 7652, 8053, 8453, 8854, 9253, 9653, 10052, 10451, 10849, 11247, 11644, 12042,
    12438, 12834, 13230, 13625, 14020, 14414, 14807, 15200, 15593, 15984, 16376, 16766, 17156,
    17545, 17934, 18322, 18709, 19096, 19482, 19867, 20251, 20634, 21017, 21399, 21780, 22161,
    22540, 22919, 23297, 23673, 24049, 24425, 24799, 25172, 25544, 25915, 26286, 26655, 27023,
    27391, 27757, 28122, 28486, 28849, 29211, 29572, 29931, 30290, 30647, 31004, 31359, 31713,
    32065, 32417, 32767, 33116, 33464, 33810, 34155, 34499, 34842, 35183, 35523, 35862, 36199,
    36535, 36869, 37202, 37534, 37864, 38193, 38520, 38846, 39170, 39493, 39815, 40134, 40453,
    40770, 41085, 41399, 41711, 42021, 42330, 42638, 42944, 43248, 43550, 43851, 44150, 44448,
    44743, 45038, 45330, 45621, 45910, 46197, 46482, 46766, 47048, 47328, 47606, 47883, 48158,
    48430, 48701, 48971, 49238, 49504, 49767, 50029, 50289, 50547, 50802, 51057, 51309, 51559,
    51807, 52053, 52298, 52540, 52780, 53018, 53255, 53489, 53721, 53951, 54180, 54406, 54630,
    54852, 55071, 55289, 55505, 55718, 55930, 56139, 56346, 56552, 56754, 56955, 57154, 57350,
    57545, 57737, 57927, 58114, 58300, 58483, 58664, 58843, 59019, 59194, 59366, 59536, 59703,
    59869, 60032, 60193, 60351, 60507, 60661, 60813, 60962, 61109, 61254, 61396, 61536, 61674,
    61809, 61942, 62073, 62201, 62327, 62451, 62572, 62691, 62807, 62921, 63033, 63142, 63249,
    63353, 63455, 63555, 63652, 63747, 63840, 63930, 64017, 64102, 64185, 64265, 64343, 64419,
    64492, 64562, 64630, 64696, 64759, 64820, 64878, 64934, 64987, 65038, 65086, 65132, 65175,
    65216, 65255, 65291, 65324, 65356, 65384, 65410, 65434, 65455, 65474, 65490, 65503, 65515,
    65523, 65530, 65533, 65535,
];

/// The table value for an angle already folded into the first quarter.
///
/// Returns a 16.16-style scale where 65536 is one, **not** 65535 — see below.
///
/// The interpolation multiply is a 32-bit `imul` in the original and it
/// **overflows on purpose**, or at least overflows and is relied upon. At the
/// very top of the quarter the index reaches 255 and the byte-wide increment
/// wraps the next entry round to `table[0]`, so the interpolation runs from
/// 65535 toward 0 — which read naively would collapse the sine to nothing at
/// exactly the axis angles, and freeze any unit ordered due north.
///
/// It does not, because `-65535 * 0x3fffff` does not fit in 32 bits. Truncated
/// to 32, it comes back as `+4,259,839`, which shifts down to 1 and lands the
/// result on 65536 — exactly one. The wrap cancels the wrap.
///
/// This is the single most delicate thing in the module, and it is why the
/// multiply below is a `wrapping_mul` and not a widening one.
///
/// The *caller* folds a second-quarter angle by mirroring it,
/// `0x7fffffff − a`, before looking it up here — as `sinx@0092d100` does and
/// as the compiler inlined at all 63 sim call sites of `sin_table@00a46a00`
/// (read from the listing, 2026-08-27; `docs/MOVEMENT.md`, "The sine
/// table"). `sin_table` itself has a bit-30 branch that returns
/// `0xffff − cur + delta` on the unmirrored index; it is real, and it is
/// reached only from `MapGrass::make_continents`. Do not model it here.
fn quarter_lookup(folded: i32) -> i32 {
    let idx = ((folded & 0x3fff_ffff) >> 22) as usize;
    let frac = folded & 0x003f_ffff;
    let cur = SIN_TABLE[idx];
    let next = SIN_TABLE[(idx + 1) & 0xff];
    cur + ((next - cur).wrapping_mul(frac) >> 22)
}

/// The component of a step of `distance` along `angle`, on the axis the
/// original calls sine — which is **x**, because north is zero.
///
/// Folding: a negative angle is the far half of the circle, and is handled by
/// negating the distance rather than by looking anything else up. The second
/// quarter is mirrored onto the first. So only a quarter wave is ever stored.
pub fn sin_component(angle: Angle, distance: i32) -> i32 {
    if distance == 0 {
        return 0;
    }
    let (mut a, mut d) = (angle.0, distance);
    if a < 0 {
        d = -d;
        a &= 0x7fff_ffff;
    }
    let folded = if a & 0x4000_0000 != 0 {
        0x7fff_ffff - a
    } else {
        a
    };
    let s = quarter_lookup(folded);
    // The same product three ways, with the 16-bit shift split differently so
    // that `s * distance` cannot overflow for a large distance. Which path runs
    // changes where the truncation falls, so it is behaviour, not an
    // optimisation. Movement only ever takes the first.
    if d < 0xffff {
        s.wrapping_mul(d) >> 16
    } else if d < 0x00ff_ffff {
        s.wrapping_mul(d >> 8) >> 8
    } else {
        s.wrapping_mul(d >> 16)
    }
}

/// The component on the other axis — **y**, and positive means *northward*, so
/// a caller subtracts it from a stored y.
///
/// Literally sine a quarter turn later; the original has two functions whose
/// bodies are identical apart from that addition.
pub fn cos_component(angle: Angle, distance: i32) -> i32 {
    sin_component(angle.quarter_turn(), distance)
}

/// Whether a step of `step` reaches the destination this frame.
///
/// A **Manhattan** test, not the octagonal `vector_dist` that territory and
/// supply measure with, and not a true distance. Both the unit and the body
/// snap onto their destinations by it, so a diagonal approach snaps from
/// further out than an axis approach of the same true distance.
pub const fn arrives(dx: i32, dy: i32, step: i32) -> bool {
    dx.abs() + dy.abs() <= step
}

/// The type-level facts a unit's turn rate and turn limits depend on.
///
/// Inputs, the same way `speed` is: they come from the unit type and the
/// unit's packed state, which the simulation does not model yet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Turning {
    /// The type's `TURN_SPEED`, already through [`degrees_to_angle`] — a
    /// binary angle, as `UnitType::init` stores it.
    pub type_turn_speed: i32,
    /// Whether the unit is packed (`unit_masks & 0x80000`), which doubles the
    /// rate.
    pub packed: bool,
    /// Whether the type turns instantly from a standstill — `guy_flags & 0x10`,
    /// which `Guy::init_real` sets for foot and mounted types.
    pub instant_from_stop: bool,
    /// Whether the type may keep moving while owing up to 80° at range rather
    /// than 45°: a non-land domain, or a vehicle.
    pub wide_limit: bool,
}

/// Which caller of `GuyData::turn_speed` is asking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnMode {
    /// Mode 0: the unit step. Divided by the body's average speed, floored at
    /// a degree a frame.
    Unit,
    /// Mode 1: the body. The base rate, always.
    Body,
}

/// The rate `GuyData::turn_speed` returns for a unit that turns instantly.
///
/// `0x80000000`, compared unsigned everywhere it is used, so it exceeds any
/// turn that can be owed and [`turn_towards`] snaps.
pub const INSTANT_TURN: u32 = 0x8000_0000;

/// How far a unit can turn in one frame, in binary angle units — the
/// original's `GuyData::turn_speed`, which both steps call.
///
/// The base is the type's angle with its low eight bits cleared (the `>> 8`
/// and the `UNIT_TURN_SPEED` of 256 cancel to that), doubled when packed. Then
/// three rules the data file's own comment on `TURN_SPEED` promises:
///
/// - a foot or mounted unit with `last_speed == 0` turns **instantly**;
/// - the body gets the base;
/// - the unit gets the base divided by `avg_speed / 4 + 1`, floored at one
///   degree a frame — so a unit at full stride turns slowly and one that has
///   just stopped turns at its full rate.
///
/// `last_speed` and `avg_speed` are the body's; see [`Body`].
pub const fn turn_speed(
    t: &Tuning,
    turning: &Turning,
    last_speed: i32,
    avg_speed: i32,
    mode: TurnMode,
) -> u32 {
    let mut base = ((turning.type_turn_speed as u32) >> 8).wrapping_mul(t.unit_turn_speed as u32);
    if turning.packed {
        base = base.wrapping_mul(t.unit_pack_turn_bonus as u32);
    }
    if last_speed == 0 && turning.instant_from_stop {
        return INSTANT_TURN;
    }
    match mode {
        TurnMode::Body => base,
        TurnMode::Unit => {
            // `UNIT_TURN_SPEED * 0xb60b` is `0xb60b00`: one degree, to the same
            // eight-bit-cleared precision as the base.
            let floor = (t.unit_turn_speed as u32).wrapping_mul(0xb60b);
            let moving = base / ((avg_speed / 4 + 1) as u32);
            if moving > floor { moving } else { floor }
        }
    }
}

/// A **crew** guy's turn rate: a flat quarter turn a frame.
///
/// `GuyData::turn_speed@005de340` is not one formula but two. Its whole
/// first half is fenced behind `guy_num < type->squad_size` — the type's
/// `TURN_SPEED`, the pack bonus, and with them everything [`turn_speed`]
/// computes. A guy past the squad falls to the `else`, and one with a
/// **track offset** returns `0x40000000` outright, ahead of the
/// instant-from-a-stop test and ahead of both modes.
///
/// Two consequences, and both are visible:
///
/// - a scout's dog comes round ninety degrees in a frame where the man
///   it follows manages twenty-seven — run54's frame 412 turns it from
///   `1681129472` to `-1540096000`, which is exactly a quarter turn;
/// - and it **never gives up a frame turning**. `Guy::move`'s tracked
///   branch abandons the step when `2 × rate < owed` and `owed` is
///   `|delta| − rate`, so the give-up wants `|delta| > 3 × rate` — 270°,
///   which no shortest turn can reach.
pub const CREW_TURN_SPEED: u32 = 0x4000_0000;

/// A heading below this much from its target counts as already facing, and
/// snaps. Three degrees.
pub const FACING_TOLERANCE: u32 = 0x0222_2220;

/// One frame of turning: the new heading, and how much turn is still owed.
///
/// Turning the short way round, by at most `rate`; within the tolerance, or
/// within one frame's turn, it snaps onto the target. The same arithmetic sits
/// inline in `Unit::move_step` and in `Guy::turn_towards`.
pub const fn turn_towards(from: Angle, to: Angle, rate: u32) -> (Angle, u32) {
    let delta = (to.0 as u32).wrapping_sub(from.0 as u32);
    // The magnitude of an anticlockwise turn is taken with a bitwise NOT rather
    // than a negation, so it comes out one short. Reproduced rather than
    // corrected: it is free, and the value is one the interface can show.
    let owed = if delta > 0x8000_0000 { !delta } else { delta };
    if owed < FACING_TOLERANCE || rate >= owed {
        return (to, 0);
    }
    let stepped = if delta < 0x8000_0001 {
        (from.0 as u32).wrapping_add(rate)
    } else {
        (from.0 as u32).wrapping_sub(rate)
    };
    (Angle(stepped as i32), owed - rate)
}

/// The lowest speed `UnitData::get_speed` will report.
///
/// Not decorative: a land unit in contact with its target and standing on slow
/// ground is halved twice, and this is what stops it from stopping altogether.
pub const SPEED_FLOOR: i32 = 3;

/// The effective speed of a unit in a group — the cap `UnitData::get_speed`
/// applies with flag 0, which is the **unit** step's call and never the body's.
///
/// A unit in a group with no overriding order is limited to the group's speed,
/// which is how a mixed army moves at the pace of its slowest member. A group
/// speed of zero means "not set" and does not cap anything.
pub const fn group_capped(speed: i32, group_speed: i32) -> i32 {
    let capped = if group_speed != 0 && group_speed < speed {
        group_speed
    } else {
        speed
    };
    if capped < SPEED_FLOOR {
        SPEED_FLOOR
    } else {
        capped
    }
}

/// What one frame did to the unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    /// Where it ended the frame. Unchanged if it spent the frame turning.
    pub pos: Pos,
    /// Which way it now faces — after this frame's turn, which is also the
    /// direction it stepped in.
    pub facing: Angle,
    /// The angle to the destination. `Unit::set_angle` records it as the
    /// body's desired facing, which is what the body turns to when idle.
    pub heading: Angle,
    /// Turn still owed after this frame.
    pub owed: u32,
    /// Whether it is now exactly on the destination.
    pub arrived: bool,
    /// Whether it got there by the **Manhattan snap** — `manh <= step`, the
    /// arm that writes the destination in outright — rather than by a
    /// partial step that happened to land on it.
    ///
    /// The two are different arms of `move_step@005faf30` and they end
    /// differently: the snap's arrival faces the order's angle when the move
    /// is the only order **or the action beneath is a gather**, the partial
    /// step's only when it is the only order (`docs/ORDERS.md` §4.5). A
    /// farmer walking to its next cell has a `GATHERORDER` beneath, so which
    /// arm it lands on decides whether it ends facing the order's angle or
    /// the bearing of its own last step.
    pub snapped: bool,
    /// Which of `move_step`'s two **turn-in-place** arms took the frame, if
    /// either — the arms that cover no ground and hand `Guy::do_turn` its
    /// animation override (`docs/ANIM.md` §4.8).
    pub turned_in_place: Option<TurnArm>,
}

/// The two arms of `Unit::move_step` that spend the frame turning, told
/// apart because the trace tells them apart: they are two `Guy::do_turn`
/// call sites and so two `ebp` chains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnArm {
    /// `move_step:148`, the call at `005fb2e1` — within a tile of the
    /// destination, or told to turn first by the waypoint, and owing any
    /// turn at all. **The near arm is the later address**: the compiler
    /// laid the source's first arm out second (`005fb24c`/`005fb256` both
    /// jump forward to it), so it returns to `move_step+0x3b6` and the far
    /// arm to `+0x389`.
    Near,
    /// `move_step:170`, the call at `005fb2b4` — further out and owing 45°
    /// or more (80° for a ship, aircraft or vehicle two tiles out).
    Far,
}

/// One frame of the unit's movement toward a destination — the original's
/// `Unit::move_step`, reached from `Unit::work → do_job → do_move`.
///
/// `step` is what `do_move` hands in: `get_speed` for the unit's square, times
/// the modern-infantry 5/4 where it applies. **No 11/8** — that belongs to the
/// body. `turn_rate` is [`turn_speed`] in [`TurnMode::Unit`].
///
/// The shape, none of which a fresh implementation would write:
///
/// 1. Turn first, by the rate, snapping within it.
/// 2. **Turn in place** — cover no ground — if still owing *any* turn within a
///    tile of the destination (two tiles for a type slower than 20°) **or when
///    the current waypoint carries `path_flag::TURN_FIRST`**, or if owing 45°
///    or more further out (80° for a ship, aircraft or vehicle two tiles out
///    or more).
/// 3. Otherwise **walk at half speed** while still owing 45° (22.5° for a slow
///    type).
/// 4. Snap onto the destination if the Manhattan distance is within the step.
/// 5. Otherwise take the trig components **along the new facing**, clamp each
///    axis to what remains — only when within two steps, which is also the
///    only time a clamp could fire — and propose that position.
///
/// The original also refuses a step outside the world, checks collision, and
/// pops its path stack on arrival. All three are the caller's — the bounds
/// check through `World::accepts`, the collision test and its recovery
/// through `Sim::detect_unit_collision` (`docs/COLLISION.md` §5) — which is
/// why this returns a proposal. Arrival is exact: `UnitData::tolerance` is
/// zero unless the unit is giving up on a collision.
///
/// `turn_first` is the current waypoint's `path_flag::TURN_FIRST` — the
/// `local_18 & 4` at `005fb1a5`, read off the **top** of the path stack
/// (`UnitData +0xb8`, length `+0xc0`). With the stack empty the original
/// reads slot 0 of it regardless, which is whatever the last path left
/// there; this crate passes `false`, since a unit with no path is walking
/// straight at its destination and the near-distance arm covers the last
/// tile anyway.
#[allow(clippy::too_many_arguments)]
pub fn move_step(
    from: Pos,
    facing: Angle,
    dest: Pos,
    step: i32,
    turning: &Turning,
    turn_rate: u32,
    turn_first: bool,
) -> Step {
    let (dx, dy) = (dest.x - from.x, dest.y - from.y);
    let heading = find_angle(dx, dy);
    let (facing, owed) = turn_towards(facing, heading, turn_rate);
    let manh = dx.abs() + dy.abs();
    let slow: i32 = if turning.type_turn_speed < SLOW_TURN_BELOW {
        2
    } else {
        1
    };

    let standing = Step {
        pos: from,
        facing,
        heading,
        owed,
        arrived: false,
        snapped: false,
        turned_in_place: None,
    };
    if manh < slow * UNITS_PER_TILE || turn_first {
        // Close in — or told to by the waypoint — any turn still owed
        // costs the frame.
        if owed != 0 {
            return Step {
                turned_in_place: Some(TurnArm::Near),
                ..standing
            };
        }
    } else {
        let limit = if turning.wide_limit && manh >= slow * 2 * UNITS_PER_TILE {
            EIGHTY
        } else {
            FORTY_FIVE
        };
        if owed >= limit {
            return Step {
                turned_in_place: Some(TurnArm::Far),
                ..standing
            };
        }
    }

    // Half a step while still turning hard. (The original also has a one-shot
    // half step here, `unit_masks & 0x100000`, which a *soft* collision sets
    // — `Unit::half_step`, written but not yet read: `docs/COLLISION.md` §7.)
    let step = if owed >= FORTY_FIVE / slow as u32 {
        step / 2
    } else {
        step
    };

    if arrives(dx, dy, step) {
        return Step {
            pos: dest,
            facing,
            heading,
            owed,
            arrived: true,
            snapped: true,
            turned_in_place: None,
        };
    }

    // Along the facing the unit actually has after turning, not the heading it
    // wants — a unit mid-turn walks where it is pointing.
    let mut sx = sin_component(facing, step);
    let mut cy = cos_component(facing, step);
    if manh < 2 * step {
        if sx.abs() > dx.abs() {
            sx = dx;
        }
        if cy.abs() > dy.abs() {
            cy = -dy;
        }
    }
    // y is subtracted: the cosine component points north and stored y runs
    // south.
    let pos = Pos::new(from.x + sx, from.y - cy);
    Step {
        pos,
        facing,
        heading,
        owed,
        arrived: pos == dest,
        snapped: false,
        turned_in_place: None,
    }
}

/// The body: the figure drawn for the unit, standing where the unit stands.
///
/// `GuyData::x / y`, `last_speed` and `avg_speed` for guy 0. The position is
/// presentation in the original too — nothing in the simulation reads it yet —
/// but the two counters are what the unit's turn rate divides by and what
/// makes a stopped unit turn instantly, so the body is modelled far enough to
/// keep them honest.
///
/// **Guy 0 does not chase; it snaps.** `Guy::move`'s first test is
/// `guy_num == 0 || (track_dx == 0 && track_dy == 0)`, and either way the body
/// is written straight onto its destination — the unit's own new position —
/// with `last_speed` set to `vector_dist` of the jump. So `last_speed` is the
/// **Euclidean length of the unit's own step**, and `avg_speed` the running
/// average of that. `docs/MOVEMENT.md`, "The body step".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Body {
    /// Where the body is.
    pub pos: Pos,
    /// How far it moved last frame; zero once it has caught up and stopped.
    pub last_speed: i32,
    /// A running average, three quarters old and one quarter new, each frame.
    pub avg_speed: i32,
}

impl Body {
    /// A body standing on its unit, stopped.
    pub const fn at(pos: Pos) -> Body {
        Body {
            pos,
            last_speed: 0,
            avg_speed: 0,
        }
    }
}

/// What one frame did to the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyStep {
    /// The body after the frame. Its `pos` is a proposal the caller checks
    /// against the world.
    pub body: Body,
    /// The shared facing after the body's own turn.
    pub facing: Angle,
}

/// One frame of the body — the original's `Guy::move` for guy 0, reached from
/// `Guy::process` after the unit has stepped.
///
/// `des` is the unit's position, which `Unit::set_new_location` wrote into
/// `des_x / des_y`; `heading` is `GuyData::des_angle`, which is whatever
/// `Unit::set_angle` last wrote; `turned_this_frame` is `guy_flags & 2`, the
/// bit `do_turn` sets and the idle body respects. `turn_rate` is
/// [`turn_speed`] in [`TurnMode::Body`].
///
/// Two cases, and **neither of them walks**:
///
/// - Already on the unit — which is every frame the unit did not move:
///   `last_speed` is zero, and if nothing turned the facing this frame it
///   turns toward `heading` at the body's own rate.
/// - Otherwise **snap onto the unit**, with `last_speed` set to
///   [`vector_dist`] of the jump. No turn: `Guy::move` takes this branch
///   before the turn, for `guy_num == 0` or a guy with no track offset.
///
/// Then, either way, `last_speed` folds into the average.
///
/// The consequence is the whole reason the body is modelled at all.
/// `last_speed` is the Euclidean length of the unit's own step, so
/// `avg_speed` settles on 33 for a diagonal step of 34 — which divides the
/// unit's turn rate by nine rather than the eleven an eleven-eighths chase
/// would have given. And a unit that spent its frame turning in place, or was
/// refused its step, has `last_speed` zero the *same* frame, which is what
/// arms the next frame's instant turn.
///
/// The walking branch — turn toward the unit, step
/// `floor(speed × 11 / 8)`, clamp each axis — belongs to a guy with a
/// non-zero `track_dx`/`track_dy`. It is read but not modelled: the
/// simulation has only guy 0. `docs/MOVEMENT.md`, "The body step".
pub fn body_follow(
    body: Body,
    facing: Angle,
    des: Pos,
    heading: Angle,
    turned_this_frame: bool,
    turn_rate: u32,
) -> BodyStep {
    let (dx, dy) = (des.x - body.pos.x, des.y - body.pos.y);
    let mut next = body;
    let mut facing = facing;
    if dx == 0 && dy == 0 {
        next.last_speed = 0;
        if facing != heading && !turned_this_frame {
            facing = turn_towards(facing, heading, turn_rate).0;
        }
    } else {
        next.last_speed = vector_dist(dx, dy);
        next.pos = des;
    }
    next.avg_speed = (next.avg_speed * 3 + next.last_speed) / 4;
    BodyStep { body: next, facing }
}

/// Where a crew guy is *told* to be — `Guy::set_new_location@005d86f0`'s
/// follower loop, which its leader runs every frame the leader's own body
/// moves.
///
/// `leader` is guy 0's new position and `facing` is guy 0's own `angle`
/// (`+0x18`), not the unit's heading. The track pair is rotated by it:
/// `track.0` sideways, `track.1` a quarter turn further round, each as one
/// [`sin_component`] per axis, and the axes are **not** the step's — the
/// listing at `005d88f3–005d894d` adds the first component to `y` and the
/// second to `x`, where a movement step does the opposite and negates.
///
/// `bound` is the world in position units, `(width, height) × 768`: the
/// original clamps the result into `0 ..= bound − 1` on each axis, and
/// does so only when at least one track component is non-zero, which is
/// the only case this function is called for.
///
/// Diff-backed: run56's start dump has the human scout's man at
/// `(5784, 8088)` facing `Angle::INITIAL` and its dog at `(5790, 7979)`,
/// which is this to the unit for the dog's `(-96, 48)`.
pub fn follower_des(leader: Pos, facing: Angle, track: (i32, i32), bound: Pos) -> Pos {
    let (dx, dy) = track;
    let mut x = leader.x;
    let mut y = leader.y;
    if dx != 0 {
        y += sin_component(facing, dx);
        x += sin_component(facing.quarter_turn(), dx);
    }
    if dy != 0 {
        y += sin_component(facing.quarter_turn(), dy);
        x += sin_component(facing.quarter_turn().quarter_turn(), dy);
    }
    Pos::new(x.clamp(0, bound.x - 1), y.clamp(0, bound.y - 1))
}

/// One frame of a crew guy's own body — `Guy::move@005d9240`'s **third**
/// branch, the one guy 0 never takes.
///
/// The facing always comes back, because the turn is taken before the
/// give-up test. The body comes back only when the frame was not spent
/// turning: the original *returns* from `Guy::move` there, past the
/// `last_speed`, `avg_speed` and `stopped` writes alike, so a guy that
/// gives up leaves all three as they stood. Where a body does come back
/// its `pos` is a proposal the caller checks against the world — the
/// original's `WorldData::is_valid` — and a refused step leaves the guy
/// where it was without rewriting anything else.
///
/// The arithmetic, in the original's order:
///
/// - the heading is `find_angle` of the remainder, and the turn toward it
///   is `Guy::turn_towards` at the body rate (`turn_speed` mode 1);
/// - `2 × rate < owed` gives the frame up, and the doubling is a 32-bit
///   multiply: an instant turn's `0x80000000` doubles to **zero**, which
///   is only ever compared against an `owed` the snap has already made
///   zero;
/// - the step is `floor(speed × 11 / 8)` — a literal, with no constant
///   behind it, and the one place the eleven-eighths is real. The
///   original scales it again by `GameAccess::ai_speed` where that is
///   above one, and `last_speed` keeps the **unscaled** figure; the game
///   speed is not modelled here, so the two are the same number;
/// - `last_speed` is that step, and then the Manhattan snap overwrites it
///   with `vector_dist` of the true remainder;
/// - otherwise each axis is clamped so the component cannot overshoot the
///   remainder on that axis.
pub fn follower_step(
    body: Body,
    facing: Angle,
    des: Pos,
    turn_rate: u32,
    speed: i32,
) -> (Angle, Option<Body>) {
    let (dx, dy) = (des.x - body.pos.x, des.y - body.pos.y);
    let heading = find_angle(dx, dy);
    let (facing, owed) = turn_towards(facing, heading, turn_rate);
    if turn_rate.wrapping_mul(2) < owed {
        return (facing, None);
    }
    let mut next = body;
    // `(speed * 11) >> 3` with the sign correction the compiler emits for
    // a signed divide by eight — a truncation toward zero, and the speed
    // is never negative.
    let step = (speed * 11) / 8;
    next.last_speed = step;
    if arrives(dx, dy, step) {
        next.last_speed = vector_dist(dx, dy);
        next.pos = des;
    } else {
        let mut sx = sin_component(facing, step);
        let mut cy = cos_component(facing, step);
        if sx.abs() > dx.abs() {
            sx = dx;
        }
        if cy.abs() > dy.abs() {
            cy = -dy;
        }
        next.pos = Pos::new(body.pos.x + sx, body.pos.y - cy);
    }
    next.avg_speed = (next.avg_speed * 3 + next.last_speed) / 4;
    (facing, Some(next))
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    /// A Citizen: 45°, on foot, so it turns instantly from a stop.
    const CITIZEN: Turning = Turning {
        type_turn_speed: degrees_to_angle(45).0,
        packed: false,
        instant_from_stop: true,
        wide_limit: false,
    };

    /// A slow type that does not turn instantly — a 10° siege piece.
    const SIEGE: Turning = Turning {
        type_turn_speed: degrees_to_angle(10).0,
        packed: false,
        instant_from_stop: false,
        wide_limit: false,
    };

    #[test]
    fn the_axes_are_exact_and_name_the_convention() {
        // North is zero and y increases southward. Everything else in this
        // module is oriented off these four.
        assert_eq!(find_angle(0, -100), Angle::NORTH);
        assert_eq!(find_angle(100, 0), Angle::EAST);
        assert_eq!(find_angle(0, 100), Angle::SOUTH);
        assert_eq!(find_angle(-100, 0), Angle::WEST);
        assert_eq!(Angle::NORTH.quarter_turn(), Angle::EAST);
        assert_eq!(Angle::SOUTH.quarter_turn(), Angle::WEST);
        // And a quarter turn from west wraps back to north.
        assert_eq!(Angle::WEST.quarter_turn(), Angle::NORTH);
    }

    #[test]
    fn degrees_decompose_into_exact_pieces() {
        // The quarter and eighth turns are exact by construction.
        assert_eq!(degrees_to_angle(90), Angle::EAST);
        assert_eq!(degrees_to_angle(45), Angle(0x2000_0000));
        assert_eq!(degrees_to_angle(180), Angle::SOUTH);
        assert_eq!(degrees_to_angle(270), Angle::WEST);
        assert_eq!(degrees_to_angle(360), Angle::NORTH);
        // One degree is the truncated constant.
        assert_eq!(degrees_to_angle(1), Angle(ONE_DEGREE));
        // Five degrees is five of those plus the three-unit correction; twenty
        // is fifteen plus five, and lands one above the step's "slow" line,
        // which is why a 20° type is not slow and a 19° one is.
        assert_eq!(degrees_to_angle(5), Angle(5 * ONE_DEGREE + 3));
        assert_eq!(degrees_to_angle(20), Angle(SLOW_TURN_BELOW + 1));
        assert!(degrees_to_angle(19).0 < SLOW_TURN_BELOW);
        // Non-multiples of the pieces still decompose: 75 = 45 + 30.
        assert_eq!(degrees_to_angle(75), Angle(0x2000_0000 + 0x1555_5555));
    }

    #[test]
    fn the_arctangent_overshoots_at_forty_five_degrees() {
        // The diagonal is where the polynomial is worst: 541,917,184 against a
        // true eighth-turn of 536,870,912, which is 0.94% high. Pinned because
        // a "better" approximation would move every unit in the game.
        assert_eq!(find_angle(100, -100), Angle(541_917_184));
        assert_eq!(0x2000_0000, 536_870_912);
    }

    #[test]
    fn every_angle_is_within_a_percent_of_the_true_bearing() {
        // The only floating point in this crate, and it is in a test: it bounds
        // the original's approximation rather than computing anything.
        let full = 4_294_967_296f64;
        let mut worst = 0f64;
        for dx in -40..=40 {
            for dy in -40..=40 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let got = f64::from(find_angle(dx, dy).0) / full;
                let want = f64::from(dx).atan2(f64::from(-dy)) / std::f64::consts::TAU;
                let mut err = (got - want).abs();
                if err > 0.5 {
                    err = 1.0 - err;
                }
                worst = worst.max(err);
            }
        }
        // Under a hundredth of a turn, everywhere.
        assert!(worst < 0.01, "worst error {worst} of a full turn");
    }

    #[test]
    fn the_sine_table_is_a_quarter_wave_that_saturates_one_step_early() {
        assert_eq!(SIN_TABLE[0], 0);
        assert_eq!(SIN_TABLE[255], 65535);
        assert!(SIN_TABLE.windows(2).all(|w| w[0] <= w[1]));
        // The divisor is 255 while the index runs over 256 steps, so the table
        // runs ahead of a true sine. At the midpoint it reads 46482 where a
        // true quarter-wave would read 46340.
        assert_eq!(SIN_TABLE[128], 46482);
    }

    #[test]
    fn arrival_is_a_manhattan_test_not_a_distance() {
        // Along an axis the two agree.
        assert!(arrives(25, 0, 25));
        assert!(!arrives(26, 0, 25));
        // Diagonally they do not: 18,18 is 25 away by any real measure and 36
        // by this one, so the unit does *not* snap — it is further out than an
        // axis approach of the same true distance.
        assert!(!arrives(18, 18, 25));
        assert!(arrives(12, 12, 25));
    }

    #[test]
    fn turning_takes_the_short_way_and_reports_what_is_owed() {
        let rate = 0x0800_0000;
        // A hair off counts as facing, and snaps onto the target.
        let (a, owed) = turn_towards(Angle::NORTH, Angle(0x0100_0000), rate);
        assert_eq!((a, owed), (Angle(0x0100_0000), 0));
        // Within one frame's turn, snap to the target.
        let (a, owed) = turn_towards(Angle::NORTH, Angle(0x0400_0000), rate);
        assert_eq!((a, owed), (Angle(0x0400_0000), 0));
        // Beyond it, turn by the rate and report the rest.
        let (a, owed) = turn_towards(Angle::NORTH, Angle::EAST, rate);
        assert_eq!(a, Angle(0x0800_0000));
        assert_eq!(owed, 0x4000_0000 - rate);
        // The short way round: north to west turns anticlockwise, not three
        // quarters clockwise.
        let (a, _) = turn_towards(Angle::NORTH, Angle::WEST, rate);
        assert_eq!(a, Angle(-0x0800_0000));
        // The instant rate snaps from anywhere, including a full reverse.
        let (a, owed) = turn_towards(Angle::NORTH, Angle::SOUTH, INSTANT_TURN);
        assert_eq!((a, owed), (Angle::SOUTH, 0));
    }

    #[test]
    fn the_turn_rate_is_the_types_angle_with_its_low_byte_cleared() {
        // UNIT_TURN_SPEED is 256, so `(angle >> 8) * 256` is the angle with
        // the low eight bits gone. 45° is already clean; 10° loses 198.
        assert_eq!(T.unit_turn_speed, 256);
        assert_eq!(turn_speed(&T, &CITIZEN, 25, 0, TurnMode::Body), 0x2000_0000);
        assert_eq!(
            turn_speed(&T, &SIEGE, 25, 0, TurnMode::Body),
            (degrees_to_angle(10).0 as u32) & !0xff
        );
        // Packing doubles it.
        let packed = Turning {
            packed: true,
            ..CITIZEN
        };
        assert_eq!(turn_speed(&T, &packed, 25, 0, TurnMode::Body), 0x4000_0000);
    }

    #[test]
    fn a_foot_unit_turns_instantly_from_a_stop_and_a_siege_piece_does_not() {
        // The data file's own comment: "Foot & Mounted units turn instantly
        // from a stopped position". `last_speed == 0` is "stopped".
        assert_eq!(turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit), INSTANT_TURN);
        assert_eq!(turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Body), INSTANT_TURN);
        // Moving, it is back on the rate.
        assert_ne!(
            turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit),
            INSTANT_TURN
        );
        // A type without the flag gets its base from a standstill.
        assert_eq!(
            turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit),
            (degrees_to_angle(10).0 as u32) & !0xff
        );
    }

    #[test]
    fn a_moving_unit_turns_slower_the_faster_it_goes_down_to_a_degree() {
        // Mode 0 divides by `avg_speed / 4 + 1`: a Citizen at full stride
        // turns at 45°/7, a little over six degrees a frame.
        assert_eq!(
            turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit),
            0x2000_0000 / 7
        );
        // The body does not slow down.
        assert_eq!(
            turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Body),
            0x2000_0000
        );
        // The floor is one degree a frame, to the same precision as the base:
        // a 5° type at a Citizen's pace would be 5°/7 and is held at 1°.
        let slow = Turning {
            type_turn_speed: degrees_to_angle(5).0,
            ..SIEGE
        };
        let floor = 256 * 0xb60b;
        assert_eq!(floor, (ONE_DEGREE as u32) & !0xff);
        assert_eq!(turn_speed(&T, &slow, 25, 25, TurnMode::Unit), floor);
    }

    #[test]
    fn the_axis_components_are_whole_steps_and_the_overflow_is_why() {
        // The four cardinals, which are exactly where the table's wrap-around
        // lands. If the interpolation multiply were widened these would all
        // come back as zero and every unit would stand still.
        let d = 1000;
        assert_eq!(sin_component(Angle::NORTH, d), 0);
        assert_eq!(cos_component(Angle::NORTH, d), d);
        assert_eq!(sin_component(Angle::EAST, d), d);
        assert_eq!(cos_component(Angle::EAST, d), 0);
        assert_eq!(sin_component(Angle::SOUTH, d), 0);
        assert_eq!(cos_component(Angle::SOUTH, d), -d);
        assert_eq!(sin_component(Angle::WEST, d), -d);
        assert_eq!(cos_component(Angle::WEST, d), 0);
        // And a zero step is zero regardless of heading.
        assert_eq!(sin_component(Angle::EAST, 0), 0);
    }

    #[test]
    fn the_components_stay_on_the_circle() {
        // Not a proof, a bound: every heading should put the two components
        // within a couple of percent of the step length. The metric here is a
        // true hypotenuse on purpose — this is a test asking "is the trig
        // sane", not simulation arithmetic.
        let d = 10_000f64;
        for k in 0i32..64 {
            let a = Angle(k.wrapping_mul(0x0400_0000));
            let (sx, cy) = (
                f64::from(sin_component(a, 10_000)),
                f64::from(cos_component(a, 10_000)),
            );
            let r = sx.hypot(cy);
            assert!((r - d).abs() / d < 0.02, "heading {k}: radius {r}");
        }
    }

    #[test]
    fn a_unit_walks_east_and_lands_exactly_on_its_destination() {
        // Twelve tiles east at a Citizen's speed. The unit's step is its speed,
        // 25 position units a frame — not 34, which is the body's — so this
        // takes a while and must not drift off the axis or overshoot at the
        // end.
        let start = Pos::new(0, 0);
        let dest = Pos::new(12 * 192, 0);
        let mut pos = start;
        let mut facing = Angle::EAST;
        let mut frames = 0;
        loop {
            let rate = turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Unit);
            let step = move_step(pos, facing, dest, 25, &CITIZEN, rate, false);
            pos = step.pos;
            facing = step.facing;
            frames += 1;
            if step.arrived {
                break;
            }
            assert!(frames < 1000, "never arrived");
        }
        assert_eq!(pos, dest);
        assert_eq!(pos.y, 0, "drifted off the axis");
        // 2304 units at 25 a frame is 92 full steps and a short one, and the
        // short one is the Manhattan snap onto the destination. (An earlier
        // draft of this test expected 68, at the body's 34.)
        assert_eq!(frames, 93);
    }

    #[test]
    fn a_foot_unit_reverses_and_walks_on_the_same_frame() {
        // Facing east, told to go west, stopped: the rate is instant, so it
        // faces west and takes a full step in one frame. (An earlier draft of
        // this module had every unit stand still for the turn; the original
        // reserves that for types without the flag, or already moving.)
        let rate = turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit);
        let s = move_step(
            Pos::new(0, 0),
            Angle::EAST,
            Pos::new(-10_000, 0),
            25,
            &CITIZEN,
            rate,
            false,
        );
        assert_eq!(s.facing, Angle::WEST);
        assert_eq!(s.owed, 0);
        assert_eq!(s.pos, Pos::new(-25, 0));
    }

    #[test]
    fn a_siege_piece_turns_in_place_and_then_walks_where_it_points() {
        // A 10° type, stopped, facing east and told to go north, with the
        // destination well over two tiles away. Its rate from a standstill is
        // its base, 10° less the low byte; it owes a quarter turn less that
        // each frame, and may not move while owing 45° or more. That is four
        // frames stood still; on the fifth it owes under 45° and walks — at
        // half speed, because it still owes more than 22.5°, and along the
        // facing it actually has, which is some forty degrees east of north.
        let dest = Pos::new(0, -10_000);
        let mut pos = Pos::new(0, 0);
        let mut facing = Angle::EAST;
        let mut stood = 0;
        let s = loop {
            let rate = turn_speed(&T, &SIEGE, 0, 0, TurnMode::Unit);
            let s = move_step(pos, facing, dest, 25, &SIEGE, rate, false);
            facing = s.facing;
            if s.pos != pos {
                break s;
            }
            pos = s.pos;
            stood += 1;
            assert!(stood < 40, "never got moving");
        };
        assert_eq!(stood, 4);
        assert!(s.owed < FORTY_FIVE && s.owed >= FORTY_FIVE / 2);
        // Half of 25 is 12, split between the two axes along the facing: it
        // walks north *and* east, because the trig uses the facing after the
        // turn and not the heading it wants.
        assert!(s.pos.y < 0, "moved the wrong way");
        assert!(s.pos.x > 0, "walked along the heading, not the facing");
        assert_eq!(s.pos.x.abs() + s.pos.y.abs(), 16, "not a half step");
    }

    #[test]
    fn close_to_the_destination_any_turn_owed_costs_the_frame() {
        // Within a tile — two for a slow type — the unit will not move while it
        // owes anything at all, even a turn it could otherwise walk through.
        let rate = turn_speed(&T, &SIEGE, 25, 25, TurnMode::Unit);
        // 300 away: inside two tiles for a 10° type, so owing a few degrees is
        // enough to stand.
        let near = move_step(
            Pos::new(0, 0),
            Angle(0x0400_0000),
            Pos::new(0, -300),
            25,
            &SIEGE,
            rate,
            false,
        );
        assert!(near.owed != 0 && near.owed < FORTY_FIVE);
        assert_eq!(near.pos, Pos::new(0, 0));
        // The same owed turn two tiles out is walked through.
        let far = move_step(
            Pos::new(0, 0),
            Angle(0x0400_0000),
            Pos::new(0, -1000),
            25,
            &SIEGE,
            rate,
            false,
        );
        assert_eq!(far.owed, near.owed);
        assert_ne!(far.pos, Pos::new(0, 0));

        // **And the waypoint's own `TURN_FIRST` puts the far case back in
        // the near case's arm** — `(manh < slow × 0xc0) || (path.flags & 4)`
        // at `005fb1a5`. The bit is `docs/PATHFINDER.md` §7's transport
        // marker and `shore_flagged`'s, and until 2026-09-01 this crate
        // wrote it and never read it: a citizen crossing the waterline
        // walked through the turn the original stands still for, and
        // reached its embark point a frame early (`docs/SYNC.md` §3.25).
        let told = move_step(
            Pos::new(0, 0),
            Angle(0x0400_0000),
            Pos::new(0, -1000),
            25,
            &SIEGE,
            rate,
            true,
        );
        assert_eq!(told.owed, near.owed);
        assert_eq!(told.pos, Pos::new(0, 0), "the flag costs the whole frame");
        // With nothing owed the flag costs nothing: it gates the stand, not
        // the step.
        let facing = move_step(
            Pos::new(0, 0),
            Angle::NORTH,
            Pos::new(0, -1000),
            25,
            &SIEGE,
            rate,
            true,
        );
        assert_eq!(facing.owed, 0);
        assert_ne!(facing.pos, Pos::new(0, 0));
    }

    #[test]
    fn the_clamps_land_the_unit_exactly_and_that_counts_as_arrival() {
        // A diagonal short hop: the true distance is 141 and the step is 149,
        // so a component would carry the unit past the destination on both
        // axes — but the Manhattan sum is 200, so the snap does not fire and
        // the clamps are what has to stop it. They are active because 200 is
        // within two steps.
        let dest = Pos::new(100, -100);
        assert!(!arrives(100, -100, 149));
        let rate = turn_speed(&T, &CITIZEN, 0, 0, TurnMode::Unit);
        let s = move_step(
            Pos::new(0, 0),
            Angle::NORTH,
            dest,
            149,
            &CITIZEN,
            rate,
            false,
        );
        // Both clamps fire, so it lands exactly on the destination — and with
        // a zero tolerance that is arrival, even though the snap never said so.
        assert_eq!(s.pos, dest);
        assert!(s.arrived);
        // And **which arm** it arrived on is the difference between facing
        // the order's angle and keeping this step's bearing when a gather
        // order sits beneath (`docs/ORDERS.md` §4.5). This one is the
        // partial step; a shorter hop of the same shape is the snap.
        assert!(!s.snapped, "the partial step, not the Manhattan snap");
        let near = Pos::new(3, -4);
        assert!(arrives(3, -4, 149));
        let s = move_step(
            Pos::new(0, 0),
            Angle::NORTH,
            near,
            149,
            &CITIZEN,
            rate,
            false,
        );
        assert_eq!(s.pos, near);
        assert!(s.arrived && s.snapped);
    }

    #[test]
    fn a_group_moves_at_its_slowest_and_never_below_the_floor() {
        assert_eq!(group_capped(25, 12), 12);
        assert_eq!(group_capped(12, 25), 12);
        // Zero means the group has no speed set, and caps nothing.
        assert_eq!(group_capped(25, 0), 25);
        // The floor bites after two halvings of an already slow unit.
        assert_eq!(group_capped(2, 0), SPEED_FLOOR);
        assert_eq!(group_capped(25, 1), SPEED_FLOOR);
    }

    #[test]
    fn the_body_lands_on_the_unit_and_records_the_euclidean_step() {
        // The unit stepped 25 east; the body is written straight onto it, and
        // `last_speed` is the distance of the jump — no chase, no 11/8, and
        // no turn on the way.
        let rate = turn_speed(&T, &CITIZEN, 25, 25, TurnMode::Body);
        let b = body_follow(
            Body::at(Pos::new(0, 0)),
            Angle::EAST,
            Pos::new(25, 0),
            Angle::EAST,
            false,
            rate,
        );
        assert_eq!(b.body.pos, Pos::new(25, 0));
        assert_eq!(b.body.last_speed, 25);
        assert_eq!(b.body.avg_speed, 6);
        assert_eq!(b.facing, Angle::EAST);
        // Standing on the unit — which is every frame the unit did not move —
        // the speed is zero and the average decays.
        let idle = body_follow(
            Body {
                pos: Pos::new(25, 0),
                last_speed: 25,
                avg_speed: 25,
            },
            Angle::EAST,
            Pos::new(25, 0),
            Angle::EAST,
            false,
            rate,
        );
        assert_eq!(idle.body.last_speed, 0);
        assert_eq!(idle.body.avg_speed, 18);
    }

    #[test]
    fn the_average_settles_below_the_step_and_that_is_what_divides_the_turn() {
        // run10's AI scout: `MOVES` 34, walking a diagonal, stepping (24, 24)
        // a frame. `vector_dist` calls that 36 — the octagonal measure, not
        // 33.9 — and the quarter-old-quarter-new average truncates its way to
        // **33**, not to 36.
        assert_eq!(vector_dist(24, 24), 36);
        let mut body = Body::at(Pos::new(0, 0));
        for i in 0..40 {
            let to = Pos::new(24 * (i + 1), 24 * (i + 1));
            body = body_follow(body, Angle::EAST, to, Angle::EAST, false, 0).body;
        }
        assert_eq!(body.last_speed, 36);
        assert_eq!(body.avg_speed, 33, "the fixpoint of (3a + 36) / 4");
        // And 33 is what the unit's turn rate is divided by: `33 / 4 + 1` is
        // nine. A Scout's 27 degrees over nine is the 0x222221c the original
        // turned by on run10's frame 62. An eleven-eighths chase would have
        // given `last_speed` 46, an average of 43, and a divisor of eleven.
        let scout = Turning {
            type_turn_speed: degrees_to_angle(27).0,
            ..CITIZEN
        };
        assert_eq!(
            turn_speed(&T, &scout, 36, body.avg_speed, TurnMode::Unit),
            0x0222_221c
        );
    }

    #[test]
    fn an_idle_body_turns_to_its_desired_angle_unless_the_unit_just_turned() {
        let rate = turn_speed(&T, &SIEGE, 0, 25, TurnMode::Body);
        let at = Body::at(Pos::new(0, 0));
        let b = body_follow(at, Angle::NORTH, Pos::new(0, 0), Angle::EAST, false, rate);
        assert_eq!(b.facing, Angle(rate as i32));
        // If the unit step already turned the shared facing this frame, the
        // body leaves it alone.
        let b = body_follow(at, Angle::NORTH, Pos::new(0, 0), Angle::EAST, true, rate);
        assert_eq!(b.facing, Angle::NORTH);
    }

    /// A scout's dog, in the three positions run56 prints for it.
    ///
    /// The pair is `(-96, 48)` — `unit_graphics.xml`'s `trackoffsetx=-20`
    /// and `trackoffsety=10` through the executable's `guy_scale` of 4.8
    /// (`rondata::artdata::piece_tracks`). Each row here is a `GUY` block
    /// the capture writes, so a rotation with its axes swapped, its signs
    /// flipped or its quarter turn dropped fails on one of them.
    #[test]
    fn a_crew_guy_stands_where_its_leader_s_facing_puts_it() {
        let bound = Pos::new(200 * 768, 200 * 768);
        let dog = (-96, 48);
        // The human scout at frame 0, facing `Unit::init`'s 120°.
        assert_eq!(
            follower_des(Pos::new(5784, 8088), Angle::INITIAL, dog, bound),
            Pos::new(5790, 7979)
        );
        // The AI's, on the frame its man arrives and on the frame after —
        // the same leader position, a different facing, and a destination
        // 150 units away because of it. That second point is where the dog
        // finally arrives, four frames later.
        let arrived = Pos::new(40416, 34272);
        assert_eq!(
            follower_des(arrived, Angle(-424673280), dog, bound),
            Pos::new(40365, 34367)
        );
        assert_eq!(
            follower_des(arrived, Angle(683016192), dog, bound),
            Pos::new(40322, 34217)
        );
        // And the clamp is the world's own edge, in position units: a
        // leader in the corner facing north puts its dog 96 to the west,
        // which is off the map, so the x is pinned at zero and the y —
        // 48 south of the corner, and on the map — is left alone.
        let corner = follower_des(Pos::new(0, 0), Angle::NORTH, dog, Pos::new(768, 768));
        assert_eq!(corner, Pos::new(0, 48));
        let far = follower_des(Pos::new(767, 767), Angle::SOUTH, dog, Pos::new(768, 768));
        assert_eq!(far, Pos::new(767, 719));
    }

    /// The crew's rate is a quarter turn, and the give-up can never fire.
    ///
    /// `Guy::move` abandons the step when `2 × rate < owed`, and
    /// [`turn_towards`] returns `|delta| − rate`; with the rate at
    /// [`CREW_TURN_SPEED`] that wants a shortest turn of more than 270°,
    /// which does not exist. So a tracked crew guy always steps.
    #[test]
    fn a_crew_guy_turns_a_quarter_at_a_time_and_never_gives_the_frame_up() {
        let body = Body::at(Pos::new(1000, 1000));
        // Facing north, told to walk due south: a half turn owed, of which
        // a quarter is taken this frame — and the step is taken anyway.
        let (facing, next) = follower_step(
            body,
            Angle::NORTH,
            Pos::new(1000, 2000),
            CREW_TURN_SPEED,
            32,
        );
        assert_eq!(facing, Angle::EAST);
        let next = next.expect("a crew guy never spends the whole frame turning");
        assert_eq!(next.last_speed, 44, "floor(32 * 11 / 8)");
        // Half a turn is the worst case there is, and the doubling that
        // guards the give-up is a 32-bit multiply — so the constant's own
        // arithmetic is what makes the branch dead.
        let (_, owed) = turn_towards(Angle::NORTH, Angle::SOUTH, CREW_TURN_SPEED);
        assert!(CREW_TURN_SPEED.wrapping_mul(2) >= owed);
    }

    /// The Manhattan snap, and the per-axis clamp that stands in for it
    /// when the step is short.
    #[test]
    fn a_crew_guy_snaps_inside_a_step_and_is_clamped_outside_one() {
        let body = Body::at(Pos::new(1000, 1000));
        // Twelve away with a step of 44: inside, so it lands exactly and
        // reports the octagonal distance rather than the step.
        let (_, next) = follower_step(body, Angle::EAST, Pos::new(1008, 1004), CREW_TURN_SPEED, 32);
        let next = next.unwrap();
        assert_eq!(next.pos, Pos::new(1008, 1004));
        assert_eq!(next.last_speed, vector_dist(8, 4));
        // Far away on x and one unit away on y: the y component is clamped
        // to the remainder so the guy cannot overshoot that axis.
        let (_, next) = follower_step(body, Angle::EAST, Pos::new(9000, 1001), CREW_TURN_SPEED, 32);
        let next = next.unwrap();
        assert_eq!(next.pos.y, 1001);
        assert_eq!(next.last_speed, 44);
    }
}
