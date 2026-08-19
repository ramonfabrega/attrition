//! Deterministic fixed-point arithmetic.
//!
//! The simulation must produce bit-identical results on every machine, every
//! run, forever. That is the basis of lockstep multiplayer *and* of diffing
//! our sim against a Rise of Nations recorded game — our only oracle. Floating
//! point cannot promise this across compilers, architectures, and optimisation
//! levels, so no gameplay value is ever an `f32` or `f64`.
//!
//! [`Fx`] is Q16.16: a signed 32-bit integer whose low 16 bits are the
//! fraction. Range is ±32768 with a resolution of 1/65536.
//!
//! # Determinism rules
//!
//! - **Rounding is truncation toward zero**, for both multiplication and
//!   division. Rust guarantees integer `/` truncates toward zero on every
//!   target, so this is portable. It is applied uniformly rather than mixing
//!   in a cheaper arithmetic shift, because one consistent rule is easier to
//!   reason about than two fast ones.
//! - **Overflow saturates.** Not wrapping, which would teleport a unit across
//!   the map; not panicking, which behaves differently between debug and
//!   release and would make the sim's behaviour depend on the build profile.
//!   Saturation is identical in both profiles. A pinned value is a visible
//!   bug; a wrapped one is an invisible desync.

#![no_std]

use core::fmt;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// Number of fractional bits.
pub const FRAC_BITS: u32 = 16;

/// The raw integer representation of `1.0`.
pub const ONE_RAW: i32 = 1 << FRAC_BITS;

/// A Q16.16 fixed-point number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fx(i32);

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(ONE_RAW);
    pub const HALF: Fx = Fx(ONE_RAW / 2);
    pub const MIN: Fx = Fx(i32::MIN);
    pub const MAX: Fx = Fx(i32::MAX);

    /// The smallest representable positive value, 1/65536.
    pub const EPSILON: Fx = Fx(1);

    /// Wraps a raw Q16.16 value. Use when decoding a value that is already in
    /// this representation; prefer [`Fx::from_int`] or [`Fx::ratio`] otherwise.
    #[inline]
    pub const fn from_raw(raw: i32) -> Fx {
        Fx(raw)
    }

    /// The underlying Q16.16 representation.
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Converts a whole number, saturating if it exceeds the ±32768 range.
    #[inline]
    pub const fn from_int(n: i32) -> Fx {
        match n.checked_mul(ONE_RAW) {
            Some(raw) => Fx(raw),
            None if n > 0 => Fx::MAX,
            None => Fx::MIN,
        }
    }

    /// Builds an exact ratio `num / den`.
    ///
    /// Rise of Nations stores many tuned values as scaled integers (unit costs
    /// are multiplied by ten, for instance), so exact ratios are the honest way
    /// to bring that data in without a float ever existing.
    ///
    /// # Panics
    ///
    /// If `den` is zero.
    #[inline]
    pub const fn ratio(num: i32, den: i32) -> Fx {
        assert!(den != 0, "Fx::ratio divides by zero");
        Fx(saturate(((num as i64) * (ONE_RAW as i64)) / (den as i64)))
    }

    /// Truncates toward zero to a whole number.
    #[inline]
    pub const fn to_int(self) -> i32 {
        self.0 / ONE_RAW
    }

    /// Rounds toward negative infinity to a whole number.
    #[inline]
    pub const fn floor_int(self) -> i32 {
        self.0 >> FRAC_BITS
    }

    /// The fractional part, always in `[0, 1)`.
    #[inline]
    pub const fn frac(self) -> Fx {
        Fx(self.0 & (ONE_RAW - 1))
    }

    #[inline]
    pub const fn abs(self) -> Fx {
        Fx(self.0.saturating_abs())
    }

    #[inline]
    pub const fn signum(self) -> i32 {
        self.0.signum()
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Square root, truncated toward zero. Negative inputs return [`Fx::ZERO`]
    /// rather than panicking, so a rounding artifact in a distance calculation
    /// cannot take down a running sim.
    #[inline]
    pub const fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        // sqrt(raw / 2^16) = sqrt(raw * 2^16) / 2^16
        Fx(saturate(((self.0 as i64) << FRAC_BITS).isqrt()))
    }

    /// Euclidean distance between two points, without ever touching a float.
    #[inline]
    pub const fn hypot(x: Fx, y: Fx) -> Fx {
        // Widened so that squaring a large coordinate cannot saturate before
        // the sum is taken.
        let xx = (x.0 as i64) * (x.0 as i64);
        let yy = (y.0 as i64) * (y.0 as i64);
        // (xx + yy) is in Q32.32; shifting back down to Q16.16 before the root
        // keeps the intermediate inside i64.
        let sum = xx.saturating_add(yy) >> FRAC_BITS;
        Fx(saturate((sum << FRAC_BITS).isqrt()))
    }

    /// Linear interpolation. `t` is clamped to `[0, 1]`.
    #[inline]
    pub fn lerp(self, other: Fx, t: Fx) -> Fx {
        let t = t.clamp(Fx::ZERO, Fx::ONE);
        self + (other - self) * t
    }
}

/// Clamps a widened intermediate back into `i32`, saturating at the bounds.
#[inline]
const fn saturate(v: i64) -> i32 {
    if v > i32::MAX as i64 {
        i32::MAX
    } else if v < i32::MIN as i64 {
        i32::MIN
    } else {
        v as i32
    }
}

impl Add for Fx {
    type Output = Fx;
    #[inline]
    fn add(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Fx {
    type Output = Fx;
    #[inline]
    fn sub(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_sub(rhs.0))
    }
}

impl Neg for Fx {
    type Output = Fx;
    #[inline]
    fn neg(self) -> Fx {
        Fx(self.0.saturating_neg())
    }
}

impl Mul for Fx {
    type Output = Fx;
    #[inline]
    fn mul(self, rhs: Fx) -> Fx {
        Fx(saturate(
            ((self.0 as i64) * (rhs.0 as i64)) / (ONE_RAW as i64),
        ))
    }
}

impl Div for Fx {
    type Output = Fx;
    /// # Panics
    ///
    /// If `rhs` is zero. A division by zero in the sim is a logic error, and
    /// silently producing a sentinel would let a desync propagate unnoticed.
    #[inline]
    fn div(self, rhs: Fx) -> Fx {
        assert!(!rhs.is_zero(), "Fx division by zero");
        Fx(saturate(((self.0 as i64) << FRAC_BITS) / (rhs.0 as i64)))
    }
}

impl AddAssign for Fx {
    #[inline]
    fn add_assign(&mut self, rhs: Fx) {
        *self = *self + rhs;
    }
}

impl SubAssign for Fx {
    #[inline]
    fn sub_assign(&mut self, rhs: Fx) {
        *self = *self - rhs;
    }
}

impl From<i32> for Fx {
    #[inline]
    fn from(n: i32) -> Fx {
        Fx::from_int(n)
    }
}

impl fmt::Display for Fx {
    /// Presentation only — never feed this back into the sim.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        // Negate in the wider type so that Fx::MIN formats correctly.
        let mag = (self.0 as i64).unsigned_abs();
        let whole = mag >> FRAC_BITS;
        // Four decimal places is finer than 1/65536, so this reads cleanly
        // without implying precision we do not have.
        let frac = ((mag & (ONE_RAW as u64 - 1)) * 10_000) >> FRAC_BITS;
        if neg {
            write!(f, "-")?;
        }
        write!(f, "{whole}.{frac:04}")
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fx({self} | raw {})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_numbers_round_trip() {
        for n in [-1000, -7, -1, 0, 1, 7, 1000, 32767] {
            assert_eq!(Fx::from_int(n).to_int(), n, "round trip failed for {n}");
        }
    }

    #[test]
    fn arithmetic_is_exact_for_representable_values() {
        let three = Fx::from_int(3);
        let four = Fx::from_int(4);
        assert_eq!(three + four, Fx::from_int(7));
        assert_eq!(three - four, Fx::from_int(-1));
        assert_eq!(three * four, Fx::from_int(12));
        assert_eq!((three / four) * four, three);
    }

    #[test]
    fn ratio_builds_exact_fractions() {
        assert_eq!(Fx::ratio(1, 2), Fx::HALF);
        assert_eq!(Fx::ratio(-1, 2), -Fx::HALF);
        assert_eq!(Fx::ratio(10, 5), Fx::from_int(2));
        // A RoN-shaped value: costs are stored multiplied by ten.
        assert_eq!(Fx::ratio(75, 10) * Fx::from_int(10), Fx::from_int(75));
    }

    #[test]
    fn truncation_is_toward_zero_in_both_directions() {
        // The asymmetry is deliberate and documented; this test pins it so a
        // future "optimisation" to a shift cannot silently change the sim.
        assert_eq!((Fx::ONE / Fx::from_int(3)).raw(), 21845);
        assert_eq!((-Fx::ONE / Fx::from_int(3)).raw(), -21845);
    }

    #[test]
    fn floor_and_truncate_differ_for_negatives() {
        let v = Fx::ratio(-3, 2); // -1.5
        assert_eq!(v.to_int(), -1);
        assert_eq!(v.floor_int(), -2);
    }

    #[test]
    fn frac_is_always_non_negative() {
        assert_eq!(Fx::ratio(3, 2).frac(), Fx::HALF);
        assert_eq!(Fx::ratio(-3, 2).frac(), Fx::HALF);
    }

    #[test]
    fn sqrt_of_perfect_squares_is_exact() {
        for (n, root) in [(0, 0), (1, 1), (4, 2), (9, 3), (16, 4), (144, 12)] {
            assert_eq!(Fx::from_int(n).sqrt(), Fx::from_int(root), "sqrt({n})");
        }
    }

    #[test]
    fn sqrt_of_negative_is_zero_not_a_panic() {
        assert_eq!(Fx::from_int(-9).sqrt(), Fx::ZERO);
    }

    #[test]
    fn hypot_matches_the_classic_triple() {
        assert_eq!(Fx::hypot(Fx::from_int(3), Fx::from_int(4)), Fx::from_int(5));
    }

    #[test]
    fn hypot_survives_large_coordinates() {
        // Squaring 3000 overflows i32 in Q16.16; the widened path must not.
        let d = Fx::hypot(Fx::from_int(3000), Fx::from_int(4000));
        assert_eq!(d.to_int(), 5000);
    }

    #[test]
    fn overflow_saturates_rather_than_wrapping() {
        assert_eq!(Fx::MAX + Fx::ONE, Fx::MAX);
        assert_eq!(Fx::MIN - Fx::ONE, Fx::MIN);
        assert_eq!(Fx::from_int(i32::MAX), Fx::MAX);
        assert_eq!(Fx::from_int(i32::MIN), Fx::MIN);
    }

    #[test]
    fn lerp_clamps_its_parameter() {
        let a = Fx::from_int(10);
        let b = Fx::from_int(20);
        assert_eq!(a.lerp(b, Fx::ZERO), a);
        assert_eq!(a.lerp(b, Fx::ONE), b);
        assert_eq!(a.lerp(b, Fx::HALF), Fx::from_int(15));
        assert_eq!(a.lerp(b, Fx::from_int(9)), b);
        assert_eq!(a.lerp(b, Fx::from_int(-9)), a);
    }

    #[test]
    fn ordering_follows_numeric_value() {
        assert!(Fx::from_int(-1) < Fx::ZERO);
        assert!(Fx::HALF < Fx::ONE);
        assert!(Fx::MAX > Fx::MIN);
    }
}
