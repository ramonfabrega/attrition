use super::*;

fn next(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(0x0019_660d).wrapping_add(0x3c6e_f35f);
    *seed >> 4
}

/// Each operation against the host's `f64` on values the bilinear
/// sample makes: ratios, offset coordinates, fractions, complements,
/// bytes and their products and sums.
#[test]
fn every_operation_is_the_host_s_double() {
    let mut seed = 0x2468_ace1;
    for _ in 0..20_000 {
        let n = 1 + (next(&mut seed) % 4000) as i32;
        let d = 1 + (next(&mut seed) % 4000) as i32;
        let r = Double::ratio(n, d);
        let rh = f64::from(n) / f64::from(d);
        assert_eq!(r.bits(), rh.to_bits(), "{n} / {d}");
        let x = (next(&mut seed) % 2000) as i32;
        let xh = f64::from(x) + 0.5;
        let xd = Double::from_scaled(2 * i64::from(x) + 1, -1);
        assert_eq!(xd.bits(), xh.to_bits(), "{x} + 0.5");
        let p = r.mulsd(xd);
        let ph = rh * xh;
        assert_eq!(p.bits(), ph.to_bits(), "{rh} × {xh}");
        let i = p.trunc();
        assert_eq!(i, ph as i32, "trunc {ph}");
        let f = p.subsd(Double::from_i32(i));
        let fh = ph - f64::from(i);
        assert_eq!(f.bits(), fh.to_bits(), "{ph} − {i}");
        let o = Double::ONE.subsd(f);
        let oh = 1.0 - fh;
        assert_eq!(o.bits(), oh.to_bits(), "1 − {fh}");
        let b = Double::from_i32((next(&mut seed) % 256) as i32);
        let bh = f64::from(b.trunc());
        let t = o.mulsd(f).mulsd(b);
        let th = oh * fh * bh;
        assert_eq!(t.bits(), th.to_bits(), "{oh} × {fh} × {bh}");
        let s = t.addsd(p).addsd(Double::ZERO).addsd(r);
        let sh = th + ph + 0.0 + rh;
        assert_eq!(s.bits(), sh.to_bits(), "{th} + {ph} + {rh}");
        assert_eq!(s.trunc(), sh as i32, "trunc {sh}");
    }
}

/// A tie rounds to even, and a carry renormalises.
#[test]
fn a_tie_rounds_to_even() {
    // 2^53 + 1 is a tie between 2^53 and 2^53 + 2.
    let big = Double::from_scaled(1 << 53, 0);
    let one = Double::ONE;
    assert_eq!(
        big.addsd(one).bits(),
        (9_007_199_254_740_992.0f64 + 1.0).to_bits()
    );
    let odd = Double::from_scaled((1 << 53) + 2, 0);
    assert_eq!(
        odd.addsd(one).bits(),
        (9_007_199_254_740_994.0f64 + 1.0).to_bits()
    );
    assert_eq!(Double::ratio(1, 3).bits(), (1.0f64 / 3.0).to_bits());
}
