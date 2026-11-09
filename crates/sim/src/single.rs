//! IEEE-754 binary32, done in integers — the arithmetic the original's
//! aircraft banking is written in, without a float ever existing here.
//!
//! `Unit::bank_aircraft@005e9520` is the one piece of gameplay the port has
//! reached that the original wrote in single-precision floating point, and
//! it is not decorative: the bank angle is an **accumulator**, stepped by at
//! most ten a frame and clamped near ±55, whose *sign* chooses the turn
//! direction and whose truncated magnitude scales the turn rate
//! (`Unit::air_turn_speed@005ea390`). A bird that banks a frame later flies
//! somewhere else for the rest of its life, so the arithmetic has to land on
//! the same bit pattern rather than merely close to it.
//!
//! `CLAUDE.md` forbids a float in the simulation, and the reason is exactly
//! why this module exists: a host float agrees with itself on this machine
//! and diverges on another compiler's spill or contraction. The original is
//! SSE — `movss`, `mulss`, `divss`, `addss`, `subss`, each correctly rounded
//! to nearest-even by the hardware — so the operations are reproducible
//! exactly, in integers, and this module does that: a [`Single`] is the 32
//! bits, and every operation is exact rational arithmetic followed by one
//! round-to-nearest-even.
//!
//! It is the same argument [`crate::combat`]'s `f32_sqrt` makes for a
//! projectile's flight time, one layer up: there a single expression, here a
//! small algebra. The tests check every operation against the host's own
//! single precision over random bit patterns — an oracle written in the
//! arithmetic this module refuses to use, which is the only way to know it
//! agrees.

/// A single-precision value, as its 32 bits.
///
/// `PartialEq` is bitwise, so `-0` and `+0` are different;
/// [`Single::eq_value`] is the comparison `ucomiss` makes, where they are
/// equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Single(pub u32);

const SIGN: u32 = 0x8000_0000;
const INF: u32 = 0x7f80_0000;
const QUIET: u32 = 0x0040_0000;
/// The "real indefinite" an invalid operation answers (`0/0`, `inf - inf`).
const DEFAULT_NAN: Single = Single(0xffc0_0000);
const MANT: u32 = 0x007f_ffff;
/// The implicit bit of a normal significand.
const HIDDEN: u64 = 1 << 23;

impl Single {
    pub const ZERO: Single = Single(0);
    /// `0x3f000000`, one half — `__real_3f000000@00b694c0`, the multiplier
    /// of `find_tcoord_z`'s mean and of the terraform's border blend.
    pub const HALF: Single = Single(0x3f00_0000);

    /// A literal, by its bits — the only way a constant enters this module,
    /// because writing one in decimal would be writing a float.
    pub const fn from_bits(bits: u32) -> Single {
        Single(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 & !SIGN == 0
    }

    /// `xorps` with the sign mask — the original's own negation.
    pub const fn neg(self) -> Single {
        Single(self.0 ^ SIGN)
    }

    /// `fabsf`, which the original calls out of line at `0x46ee30`.
    pub const fn abs(self) -> Single {
        Single(self.0 & !SIGN)
    }

    const fn negative(self) -> bool {
        self.0 & SIGN != 0
    }

    const fn is_nan(self) -> bool {
        self.0 & !SIGN > INF
    }

    const fn is_inf(self) -> bool {
        self.0 & !SIGN == INF
    }

    /// The SSE rule for a NaN operand (item 3272): the first NaN, quieted —
    /// the destination operand's when both are.
    const fn nan_operand(self, other: Single) -> Option<Single> {
        if self.is_nan() {
            Some(Single(self.0 | QUIET))
        } else if other.is_nan() {
            Some(Single(other.0 | QUIET))
        } else {
            None
        }
    }

    /// The value as `mant × 2^exp` with `mant` under 2^24 — exact, and zero
    /// for either zero. An infinity or a NaN cannot arise from the
    /// arithmetic this module is used for, and is decoded as if finite.
    const fn parts(self) -> (u64, i32) {
        let e = ((self.0 >> 23) & 0xff) as i32;
        let f = (self.0 & MANT) as u64;
        if e == 0 {
            (f, -149)
        } else {
            (f | HIDDEN, e - 150)
        }
    }

    /// The sign-magnitude order as an integer key: what `comiss` decides.
    const fn key(self) -> i64 {
        let m = (self.0 & !SIGN) as i64;
        if self.negative() { -m } else { m }
    }

    /// `comiss` with `ja` — strictly greater, both operands ordered.
    pub const fn gt(self, other: Single) -> bool {
        // Unordered (a NaN either side) is not greater: `ja` needs CF = ZF = 0.
        !self.is_nan() && !other.is_nan() && self.key() > other.key()
    }

    /// `ucomiss` with the equality test: `+0 == -0`.
    pub const fn eq_value(self, other: Single) -> bool {
        !self.is_nan() && !other.is_nan() && self.key() == other.key()
    }

    /// `cvttss2si` — truncation toward zero, and the instruction's own
    /// "integer indefinite" `0x80000000` for anything that does not fit:
    /// 2^31 and up in magnitude, an infinity, a NaN (item 3272: the sweep of
    /// `Unit::bank_aircraft` hands `air_turn_speed` such a bank; the crate
    /// saturated before).
    pub const fn to_i32(self) -> i32 {
        let (m, e) = self.parts();
        if m == 0 {
            return 0;
        }
        let v: u64 = if e >= 0 {
            if e > 40 { u64::MAX } else { m << e }
        } else if -e > 63 {
            0
        } else {
            m >> -e
        };
        if v >= 1 << 31 {
            return i32::MIN;
        }
        let v = v as i32;
        if self.negative() { -v } else { v }
    }

    /// `cvtdq2ps` — a signed int, rounded to nearest even.
    pub fn from_i32(n: i32) -> Single {
        round(n < 0, u128::from(n.unsigned_abs()), 0, false)
    }

    /// The original's unsigned conversion, which it spells `cvtdq2pd`, an
    /// `addsd` of 2^32 when the sign bit is set, and `cvtpd2ps`. Every value
    /// under 2^32 is exact as a double, so the pair is one rounding and this
    /// is it.
    pub fn from_u32(n: u32) -> Single {
        round(false, u128::from(n), 0, false)
    }

    /// `addss`.
    pub fn addss(self, other: Single) -> Single {
        if let Some(n) = self.nan_operand(other) {
            return n;
        }
        if self.is_inf() || other.is_inf() {
            return if self.is_inf() && other.is_inf() && self.negative() != other.negative() {
                DEFAULT_NAN
            } else if self.is_inf() {
                self
            } else {
                other
            };
        }
        let (m1, e1) = self.parts();
        let (m2, e2) = other.parts();
        if m1 == 0 {
            // `-0 + -0` is `-0`; every other zero sum is `+0`, and a zero
            // plus anything is that thing.
            return if m2 == 0 {
                Single(self.0 & other.0 & SIGN)
            } else {
                other
            };
        }
        if m2 == 0 {
            return self;
        }
        let e = e1.min(e2);
        let (d1, d2) = (e1 - e, e2 - e);
        // A significand is under 2^24 and the shift is capped so the product
        // stays inside a `u128`; past the cap the smaller operand lies more
        // than a hundred bits below the larger's last bit, and the sum
        // rounds to the larger exactly.
        if d1 > 100 {
            return self;
        }
        if d2 > 100 {
            return other;
        }
        let (a, b) = (u128::from(m1) << d1, u128::from(m2) << d2);
        let (mag, sign) = if self.negative() == other.negative() {
            (a + b, self.negative())
        } else if a >= b {
            (a - b, self.negative())
        } else {
            (b - a, other.negative())
        };
        if mag == 0 {
            // Exact cancellation is `+0` in round-to-nearest.
            return Single::ZERO;
        }
        round(sign, mag, e, false)
    }

    /// `subss`.
    pub fn subss(self, other: Single) -> Single {
        // A NaN subtrahend keeps its sign; only the number is negated.
        if let Some(n) = self.nan_operand(other) {
            return n;
        }
        self.addss(other.neg())
    }

    /// `mulss`.
    pub fn mulss(self, other: Single) -> Single {
        if let Some(n) = self.nan_operand(other) {
            return n;
        }
        let (m1, e1) = self.parts();
        let (m2, e2) = other.parts();
        let sign = self.negative() != other.negative();
        if self.is_inf() || other.is_inf() {
            return if self.is_zero() || other.is_zero() {
                DEFAULT_NAN
            } else {
                Single(INF | if sign { SIGN } else { 0 })
            };
        }
        if m1 == 0 || m2 == 0 {
            return Single(if sign { SIGN } else { 0 });
        }
        round(sign, u128::from(m1) * u128::from(m2), e1 + e2, false)
    }

    /// `divss`, the IEEE special cases included (item 3272: a helicopter's
    /// divisor is not floored, so a type with no turn speed divides by zero).
    pub fn divss(self, other: Single) -> Single {
        if let Some(n) = self.nan_operand(other) {
            return n;
        }
        let (m1, e1) = self.parts();
        let (m2, e2) = other.parts();
        let sign = self.negative() != other.negative();
        let inf = Single(INF | if sign { SIGN } else { 0 });
        if (self.is_inf() && other.is_inf()) || (self.is_zero() && other.is_zero()) {
            return DEFAULT_NAN;
        }
        if self.is_inf() || other.is_zero() {
            return inf;
        }
        if other.is_inf() {
            return Single(if sign { SIGN } else { 0 });
        }
        if m1 == 0 || m2 == 0 {
            return Single(if sign { SIGN } else { 0 });
        }
        // Sixty-four extra bits carry the 24 of the answer, a guard and a
        // sticky, whatever the operands' alignment.
        let num = u128::from(m1) << 64;
        let q = num / u128::from(m2);
        let exact = q * u128::from(m2) == num;
        round(sign, q, e1 - e2 - 64, !exact)
    }

    /// `sqrtf@0041e6f0`: `cvtss2sd`, the library's double `sqrt`, and
    /// `cvtsd2ss`. The double root is correctly rounded to 53 bits and
    /// then to 24, and 53 ≥ 2 × 24 + 2 makes that pair one correct rounding
    /// of the true root — which is what this returns. A negative operand
    /// cannot arise (the one caller roots a fall time and a squared
    /// distance) and answers zero rather than a NaN.
    pub fn sqrtss(self) -> Single {
        let (m, e) = self.parts();
        if m == 0 || self.negative() {
            return Single::ZERO;
        }
        // An even exponent, and eighty more bits under the significand so
        // the integer root carries the 24 of the answer, a guard and a
        // sticky.
        let (m, e) = if e % 2 != 0 {
            (u128::from(m) << 1, e - 1)
        } else {
            (u128::from(m), e)
        };
        let n = m << 80;
        let mut r = 1u128 << (128 - n.leading_zeros()).div_ceil(2);
        loop {
            let y = (r + n / r) / 2;
            if y >= r {
                break;
            }
            r = y;
        }
        round(false, r, (e - 80) / 2, r * r != n)
    }

    /// The single a six-decimal print names, and whether it is the only one
    /// — how the height grid enters this arithmetic (`docs/COMBAT.md`
    /// §46.2).
    ///
    /// `SimpleArray<float>::log_data` prints each of `master_land_heights`
    /// with `%f`, and this crate keeps the grid in those millionths. The
    /// answer is the single nearest `millionths / 10^6`, and the flag says
    /// whether **no other** single prints as the same six decimals. From 16
    /// up a single's spacing is wider than a millionth and the flag is
    /// always set; below it two to four singles can share a print, and the
    /// nearest is then a choice rather than a recovery — at most a few ulp,
    /// under 2e-6 of a world unit, off the original's.
    pub fn from_millionths(millionths: i64) -> (Single, bool) {
        let neg = millionths < 0;
        let d = u128::from(millionths.unsigned_abs());
        let num = d << 64;
        let q = num / 1_000_000;
        let s = round(neg, q, -64, q * 1_000_000 != num);
        let alike = |c: Single| c.prints_as_millionths(neg, d);
        let unique = !alike(Single(s.0.wrapping_add(1))) && !alike(Single(s.0.wrapping_sub(1)));
        (s, unique)
    }

    /// Whether a correctly rounded `%f` of this single could print as
    /// `millionths` — the start dump's own test of a height
    /// (`sim::map_heights`). From 16 up exactly one single does.
    pub fn prints_as(self, millionths: i64) -> bool {
        self.prints_as_millionths(millionths < 0, u128::from(millionths.unsigned_abs()))
    }

    /// Whether this value lies within half a millionth of `± d / 10^6`, so
    /// that a correctly rounded six-decimal print could show it as `d`. A
    /// tie counts, which can only ever mark a print ambiguous.
    fn prints_as_millionths(self, neg: bool, d: u128) -> bool {
        if (self.0 >> 23) & 0xff == 0xff {
            return false;
        }
        let (m, e) = self.parts();
        if m == 0 {
            return d == 0;
        }
        if self.negative() != neg && d != 0 {
            return false;
        }
        // |m × 2^e × 2·10^6 − 2d| ≤ 1, both sides scaled by 2^-e when e < 0.
        let m2 = u128::from(m) * 2_000_000;
        if e >= 0 {
            e < 64 && (m2 << e).abs_diff(2 * d) <= 1
        } else if -e > 80 {
            // Under 2^-56: a print of zero.
            d == 0
        } else {
            let scale = 1u128 << (-e);
            m2.abs_diff(2 * d * scale) <= scale
        }
    }
}

/// Rounds `mag × 2^exp` to a single, to nearest with ties to even, given
/// whether anything nonzero was already discarded below `mag`.
fn round(sign: bool, mag: u128, exp: i32, sticky: bool) -> Single {
    let bit = if sign { SIGN } else { 0 };
    if mag == 0 {
        return Single(bit);
    }
    let bits = 128 - mag.leading_zeros() as i32;
    // Either twenty-four significant bits, or as many as fit above the
    // subnormal floor of 2^-149 — whichever asks for more to be dropped.
    let drop = (bits - 24).max(-149 - exp);
    let (mut m, mut e) = if drop <= 0 {
        // Nothing to drop, so whatever `sticky` carries lies more than a bit
        // below the last one kept and rounds down.
        (mag << (-drop) as u32, exp + drop)
    } else if drop >= 128 {
        (0u128, exp + drop)
    } else {
        let dropped = mag & ((1u128 << drop) - 1);
        let half = 1u128 << (drop - 1);
        let m = mag >> drop;
        let up = dropped > half || (dropped == half && (sticky || m & 1 == 1));
        (m + u128::from(up), exp + drop)
    };
    if m >= 1 << 24 {
        m >>= 1;
        e += 1;
    }
    if m == 0 {
        return Single(bit);
    }
    let biased = e + 150;
    if m >= 1 << 23 {
        if biased > 254 {
            // An infinity, unreachable from the banking's arithmetic.
            return Single(bit | (0xff << 23));
        }
        Single(bit | ((biased as u32) << 23) | (m as u32 & MANT))
    } else {
        Single(bit | m as u32)
    }
}

#[cfg(test)]
mod tests;
