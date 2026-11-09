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
fn the_square_root_agrees_with_the_host_bit_for_bit() {
    for a in samples() {
        let x = Single(a & !SIGN);
        let want = f64::from(host(x)).sqrt() as f32;
        assert_eq!(x.sqrtss().0, want.to_bits(), "sqrt {:#x}", x.0);
    }
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

/// [`Single::from_millionths`] names the printed single wherever the
/// print is unique, says so exactly when it is, and is never more than
/// a few ulp off where it is not. The print is the host's own `{:.6}`,
/// which is correctly rounded, as `%f` is.
#[test]
fn a_six_decimal_print_names_its_single_where_it_can() {
    let mut unique_seen = 0;
    let mut shared_seen = 0;
    for b in samples() {
        let x = host(Single(b));
        if !(1.0..=1.0e6).contains(&x.abs()) {
            continue;
        }
        let printed = format!("{x:.6}");
        let (neg, digits) = match printed.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, printed.as_str()),
        };
        let (w, f) = digits.split_once('.').unwrap();
        let mag = w.parse::<i64>().unwrap() * 1_000_000 + f.parse::<i64>().unwrap();
        let (got, unique) = Single::from_millionths(if neg { -mag } else { mag });
        if x.abs() >= 16.0 {
            assert!(unique, "{x} is the only single that prints as {printed}");
        }
        // Every single within reach that prints alike, by the host.
        let alike: Vec<u32> = (b.saturating_sub(24)..=b.saturating_add(24))
            .filter(|&c| format!("{:.6}", host(Single(c))) == printed)
            .collect();
        assert_eq!(unique, alike.len() == 1, "{printed}: {alike:x?}");
        if unique {
            unique_seen += 1;
            assert_eq!(got.0, b, "{printed} names {x}");
        } else {
            shared_seen += 1;
            assert!(
                alike.contains(&got.0),
                "{printed}: {:#x} not in {alike:x?}",
                got.0
            );
        }
    }
    assert!(
        unique_seen > 150 && shared_seen > 20,
        "{unique_seen} / {shared_seen}"
    );
}
