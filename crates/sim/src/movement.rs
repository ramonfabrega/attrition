//! Movement: how fast a unit moves, which way it faces, and how far one frame
//! carries it.
//!
//! Not how it decides where to go. Pathfinding, collision and formations are
//! separate mechanics; `docs/MOVEMENT.md` surveys them and this module does not
//! implement them.
//!
//! # Units
//!
//! A speed is **position units per frame** — 1/768 of a cell, 1/192 of a tile.
//! A unit type's `MOVES` field is already that number: `UNIT_MOVE_SPEED`, whose
//! annotation reads `1/192 tile`, is the identity converter, because one 1/192
//! of a tile *is* one position unit.
//!
//! An angle is a **signed 32-bit binary angle**, so the full circle is 2^32 and
//! wrapping is free. North is zero and y increases southward.
//!
//! # What is missing, and why
//!
//! The sine *lookup* is not here — only the table it reads. The original folds
//! an angle into the first quarter before indexing, and that fold as read makes
//! the northward component of a northward step come out as zero, which would
//! freeze a unit walking north. So the reading is wrong somewhere and the error
//! has not been found; everything depending on it is left out rather than
//! guessed. The table itself and [`find_angle`] are established and are here.
//!
//! See `docs/MOVEMENT.md`, "Open questions".

use crate::tuning::Tuning;

/// A signed 32-bit binary angle: the full circle is 2^32.
///
/// Wrapping is the point. Adding a quarter turn to a heading can never
/// overflow into nonsense, and the difference of two headings is their shortest
/// signed separation with no normalisation step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Angle(pub i32);

impl Angle {
    pub const NORTH: Angle = Angle(0);
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

/// Scale factor from a speed to the distance actually covered in one frame.
///
/// `Guy::move` multiplies by 11/8 — a literal, with no constant behind it. So
/// every speed quoted in the game's data is 27% slower than what a unit covers.
pub const fn step_distance(speed: i32) -> i32 {
    speed * 11 / 8
}

/// Whether a step of `step` reaches the destination this frame.
///
/// A **Manhattan** test, not the octagonal `vector_dist` that territory and
/// supply measure with, and not a true distance. A unit approaching diagonally
/// therefore snaps onto its destination from further out than one approaching
/// along an axis.
pub const fn arrives(dx: i32, dy: i32, step: i32) -> bool {
    dx.abs() + dy.abs() <= step
}

/// How far a figure can turn in one frame, in binary angle units.
///
/// `type_turn_speed` is the type's stored value, which is 8.8 fixed point —
/// hence the shift. `UNIT_TURN_SPEED` ships as 1 and `UNIT_PACK_TURN_BONUS` as
/// 2, so packing doubles the rate.
pub const fn turn_speed(t: &Tuning, type_turn_speed: u32, packed: bool) -> i32 {
    let base = (type_turn_speed >> 8) as i32 * t.unit_turn_speed;
    if packed {
        base * t.unit_pack_turn_bonus
    } else {
        base
    }
}

/// A heading below this much from its target counts as already facing.
/// About 1/120 of a full circle.
pub const FACING_TOLERANCE: i32 = 0x0222_2220;

/// One frame of turning: the new heading, and how much turn is still owed.
///
/// Turning the short way round, by at most `rate`. The owed amount is what the
/// movement gate tests — a figure owing more than twice its turn rate spends
/// the whole frame turning and covers no ground, which is why heavy units feel
/// sticky when reversed.
pub fn turn_towards(from: Angle, to: Angle, rate: i32) -> (Angle, i32) {
    let delta = from.to(to);
    let owed = delta.unsigned_abs();
    if owed < FACING_TOLERANCE as u32 {
        return (to, 0);
    }
    if (rate as u32) >= owed {
        return (to, 0);
    }
    let stepped = if delta > 0 {
        from.0.wrapping_add(rate)
    } else {
        from.0.wrapping_sub(rate)
    };
    (Angle(stepped), (owed - rate as u32) as i32)
}

/// Whether a figure owing `owed` turn may also move this frame.
pub const fn may_move_while_turning(owed: i32, rate: i32) -> bool {
    owed <= rate * 2
}

/// The lowest speed `UnitData::get_speed` will report.
///
/// Not decorative: an ordinary unit that is damaged and standing on slow ground
/// is halved twice, and this is what stops it from stopping altogether.
pub const SPEED_FLOOR: i32 = 3;

/// The effective speed of a unit in a group.
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

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

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
    fn a_step_is_eleven_eighths_of_the_quoted_speed() {
        // A Citizen's MOVES is 25, so it actually covers 34 position units a
        // frame — every speed in the game's data reads 27% low.
        assert_eq!(step_distance(25), 34);
        assert_eq!(step_distance(8), 11);
        // Truncating, not rounding.
        assert_eq!(step_distance(1), 1);
        assert_eq!(step_distance(0), 0);
    }

    #[test]
    fn arrival_is_a_manhattan_test_not_a_distance() {
        // Along an axis the two agree.
        assert!(arrives(34, 0, 34));
        assert!(!arrives(35, 0, 34));
        // Diagonally they do not: 24,24 is 34 away by any real measure and 48
        // by this one, so the unit does *not* snap — it is further out than an
        // axis approach of the same true distance.
        assert!(!arrives(24, 24, 34));
        assert!(arrives(17, 17, 34));
    }

    #[test]
    fn turning_takes_the_short_way_and_reports_what_is_owed() {
        let rate = 0x0800_0000;
        // A hair off counts as facing, without moving the heading.
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
    }

    #[test]
    fn a_unit_owing_more_than_two_turns_stands_still() {
        let rate = 0x0800_0000;
        assert!(may_move_while_turning(rate, rate));
        assert!(may_move_while_turning(rate * 2, rate));
        assert!(!may_move_while_turning(rate * 2 + 1, rate));
    }

    #[test]
    fn packing_doubles_the_turn_rate() {
        // The type's stored value is taken as an input: what its scale means is
        // not established. See docs/MOVEMENT.md.
        assert_eq!(turn_speed(&T, 16 << 8, false), 16);
        assert_eq!(turn_speed(&T, 16 << 8, true), 32);
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
}
