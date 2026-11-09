//! A software double, for the one place map generation computes in SSE2
//! doubles: `Fractal::get_height@006aa870` (`docs/MAPGEN.md` §10, §11).
//!
//! Every operation there is an `addsd`, `subsd`, `mulsd` or a quotient of
//! two integers, each correctly rounded to 53 bits, nearest-even; so the
//! value is computed exactly in integers and rounded once per operation,
//! as DECISIONS 19 has it for singles ([`crate::single::Single`]).
//!
//! **The range is the caller's**: every value is zero or a positive normal
//! double, and a difference is taken only as `a − b` with `a ≥ b`. That is
//! what the bilinear sample holds — a coordinate times an increment, a
//! fraction in `[0, 1)`, its complement, a byte, and their products and
//! sums, all in `[0, 2^31)`. No sign, infinity, NaN or subnormal is
//! represented, and a call outside the range panics rather than guesses.

/// A zero or positive normal double: `m · 2^e`, with `m` in `[2^52, 2^53)`
/// unless it is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Double {
    m: u64,
    e: i32,
}

/// The significand's width, the implicit bit included.
const BITS: u32 = 53;

impl Double {
    pub const ZERO: Double = Double { m: 0, e: 0 };
    pub const ONE: Double = Double { m: 1 << 52, e: -52 };

    /// `mag · 2^exp`, rounded to 53 bits nearest-even, `sticky` saying
    /// whether anything nonzero was already discarded below `mag`.
    fn round(mag: u128, exp: i32, sticky: bool) -> Double {
        if mag == 0 {
            assert!(!sticky, "a value below the represented range");
            return Double::ZERO;
        }
        let bits = 128 - mag.leading_zeros();
        if bits <= BITS {
            assert!(!sticky, "a discarded remainder under a short value");
            let up = BITS - bits;
            return Double {
                m: (mag << up) as u64,
                e: exp - up as i32,
            };
        }
        let shift = bits - BITS;
        let mut keep = (mag >> shift) as u64;
        let rem = mag & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        let mut e = exp + shift as i32;
        if rem > half || (rem == half && (sticky || keep & 1 == 1)) {
            keep += 1;
            if keep == 1 << BITS {
                keep >>= 1;
                e += 1;
            }
        }
        Double { m: keep, e }
    }

    /// `cvtdq2pd` of a non-negative integer: exact.
    pub fn from_i32(n: i32) -> Double {
        assert!(n >= 0, "a negative integer");
        Double::round(n as u128, 0, false)
    }

    /// `n · 2^exp` for a non-negative `n`: exact (`x + 0.5` is `(2x + 1) ·
    /// 2^−1`).
    pub fn from_scaled(n: i64, exp: i32) -> Double {
        assert!(n >= 0, "a negative integer");
        Double::round(n as u128, exp, false)
    }

    /// `(double)n / (double)d` for positive integers below 2^31, one
    /// correctly rounded quotient.
    pub fn ratio(n: i32, d: i32) -> Double {
        assert!(n > 0 && d > 0, "a ratio of positive integers");
        // 64 bits of quotient over a 31-bit numerator leave 33 to spare.
        let k = 64;
        let num = (n as u128) << k;
        let (q, r) = (num / d as u128, num % d as u128);
        Double::round(q, -k, r != 0)
    }

    /// `addsd`.
    pub fn addsd(self, other: Double) -> Double {
        let (a, b) = if self.e >= other.e {
            (self, other)
        } else {
            (other, self)
        };
        if b.m == 0 {
            return a;
        }
        if a.m == 0 {
            return b;
        }
        let d = (a.e - b.e) as u32;
        // Past 70 places `b` is under a quarter of `a`'s last place below
        // it, so it moves nothing; and `a.m << 70` still fits.
        if d > 70 {
            return a;
        }
        Double::round(((a.m as u128) << d) + b.m as u128, b.e, false)
    }

    /// `subsd`, for `self ≥ other`.
    pub fn subsd(self, other: Double) -> Double {
        if other.m == 0 {
            return self;
        }
        assert!(
            self.m != 0 && self.e >= other.e,
            "a difference below zero, or of a smaller minuend"
        );
        let d = (self.e - other.e) as u32;
        if d > 70 {
            return self;
        }
        let a = (self.m as u128) << d;
        let b = other.m as u128;
        assert!(a >= b, "a difference below zero");
        Double::round(a - b, other.e, false)
    }

    /// `mulsd`.
    pub fn mulsd(self, other: Double) -> Double {
        if self.m == 0 || other.m == 0 {
            return Double::ZERO;
        }
        Double::round(self.m as u128 * other.m as u128, self.e + other.e, false)
    }

    /// `cvttsd2si`, toward zero, for a value under 2^31.
    pub fn trunc(self) -> i32 {
        if self.m == 0 {
            return 0;
        }
        let v = if self.e >= 0 {
            assert!(self.e <= 10, "a value past 2^63");
            (self.m as u128) << self.e
        } else if self.e > -64 {
            (self.m >> (-self.e) as u32) as u128
        } else {
            0
        };
        i32::try_from(v).expect("a value past 2^31")
    }

    /// The IEEE bits, for the oracle tests.
    #[cfg(test)]
    fn bits(self) -> u64 {
        if self.m == 0 {
            return 0;
        }
        let biased = (self.e + 52 + 1023) as u64;
        (biased << 52) | (self.m & ((1 << 52) - 1))
    }
}

#[cfg(test)]
#[allow(clippy::float_arithmetic)] // the oracle: the host's own `f64`
mod tests;
