//! `Unit::air_alt_speed@005ea360` under the emulator against the port (item 2513; hand-written).

/// `Unit::air_alt_speed@005ea360`, run by `tools/emu/sweep_unit_air_alt_speed.py` under unicorn: 0 unless the sign
/// bit of `ecx` agrees with `xmm1 < 0`, else `(int)(xmm1 * 25 / 20)` in `f32` (test code), `cvttss2si`'s
/// 0x80000000 for a NaN or an out-of-range value.
fn air_alt_speed(this: i32, speed: f32) -> i32 {
    if ((this as u32) >> 31 == 1) != (speed < 0.0) {
        return 0;
    }
    let v = (speed * 25.0) / 20.0;
    if v.is_nan() || v >= 2_147_483_648.0 || v < -2_147_483_648.0 {
        i32::MIN
    } else {
        v as i32
    }
}

#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_air_alt_speed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got = i64::from(air_alt_speed(a[0] as i32, f32::from_bits(a[1] as u32)));
        assert_eq!(got, want, "air_alt_speed — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::air_alt_speed@005ea360: {n} rows agree");
}
