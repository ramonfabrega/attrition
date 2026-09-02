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
const MANT: u32 = 0x007f_ffff;
/// The implicit bit of a normal significand.
const HIDDEN: u64 = 1 << 23;

impl Single {
    pub const ZERO: Single = Single(0);

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
        self.key() > other.key()
    }

    /// `ucomiss` with the equality test: `+0 == -0`.
    pub const fn eq_value(self, other: Single) -> bool {
        self.key() == other.key()
    }

    /// `cvttss2si` — truncation toward zero. Saturating rather than the
    /// original's `0x80000000` on overflow, which the banking cannot reach.
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
        let v = if v > i32::MAX as u64 {
            i32::MAX
        } else {
            v as i32
        };
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
        self.addss(other.neg())
    }

    /// `mulss`.
    pub fn mulss(self, other: Single) -> Single {
        let (m1, e1) = self.parts();
        let (m2, e2) = other.parts();
        let sign = self.negative() != other.negative();
        if m1 == 0 || m2 == 0 {
            return Single(if sign { SIGN } else { 0 });
        }
        round(sign, u128::from(m1) * u128::from(m2), e1 + e2, false)
    }

    /// `divss`. A zero divisor cannot arise here — the only divisor is a
    /// turn rate floored at `0x5b05b0` — and answers zero rather than an
    /// infinity.
    pub fn divss(self, other: Single) -> Single {
        let (m1, e1) = self.parts();
        let (m2, e2) = other.parts();
        let sign = self.negative() != other.negative();
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
mod tests {
    use super::*;

    /// A spread of bit patterns: the constants the banking uses, the edges
    /// of the subnormal range, and a deterministic scatter over the rest.
    fn samples() -> Vec<u32> {
        let mut v = vec![
            0x0000_0000,
            0x8000_0000,
            0x3f80_0000,
            0xbf80_0000,
            0x4000_0000,
            0x4120_0000,
            0x425c_0000,
            0xc25c_0000,
            0x3ea8_f5c3,
            0x3f00_0000,
            0x40a0_0000,
            0x0000_0001,
            0x007f_ffff,
            0x0080_0000,
            0x7f7f_ffff,
        ];
        let mut x: u64 = 0x2545_f491_4f6c_dd1d;
        for _ in 0..3000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let b = (x as u32) & 0x7fff_ffff;
            // Finite only: the module makes no claim about NaN or infinity.
            if b >> 23 != 0xff {
                v.push(b | ((x >> 40) as u32 & SIGN));
            }
        }
        v
    }

    /// The host's own single precision, as the oracle. `#[cfg(test)]` is the
    /// only place `no_float.rs` allows one — and that is the point: an
    /// oracle written in this module's arithmetic would prove nothing.
    fn host(s: Single) -> f32 {
        f32::from_bits(s.0)
    }

    #[test]
    fn the_four_operations_agree_with_the_host_bit_for_bit() {
        let v = samples();
        for (i, &a) in v.iter().enumerate() {
            for &b in v.iter().skip(i % 37).step_by(53) {
                let (x, y) = (Single(a), Single(b));
                for (name, got, want) in [
                    ("add", x.addss(y), host(x) + host(y)),
                    ("sub", x.subss(y), host(x) - host(y)),
                    ("mul", x.mulss(y), host(x) * host(y)),
                ] {
                    if want.is_finite() {
                        assert_eq!(got.0, want.to_bits(), "{name}({a:#010x}, {b:#010x})");
                    }
                }
                if !y.is_zero() {
                    let want = host(x) / host(y);
                    if want.is_finite() {
                        assert_eq!(x.divss(y).0, want.to_bits(), "div({a:#010x}, {b:#010x})");
                    }
                }
            }
        }
    }

    #[test]
    fn the_conversions_agree_with_the_host() {
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
        for _ in 0..20_000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let n = x as i32;
            assert_eq!(Single::from_i32(n).0, (n as f32).to_bits(), "from_i32({n})");
            let u = x as u32;
            assert_eq!(Single::from_u32(u).0, (u as f32).to_bits(), "from_u32({u})");
        }
        for &b in samples().iter() {
            let s = Single(b);
            let f = host(s);
            if f.abs() < 2.0e9 {
                assert_eq!(s.to_i32(), f as i32, "to_i32({b:#010x})");
            }
        }
    }

    #[test]
    fn the_order_is_the_hosts() {
        let v = samples();
        for (i, &a) in v.iter().enumerate() {
            for &b in v.iter().skip(i % 11).step_by(29) {
                let (x, y) = (Single(a), Single(b));
                assert_eq!(x.gt(y), host(x) > host(y), "{a:#010x} > {b:#010x}");
                assert_eq!(x.eq_value(y), host(x) == host(y), "{a:#010x} == {b:#010x}");
            }
        }
    }
}
