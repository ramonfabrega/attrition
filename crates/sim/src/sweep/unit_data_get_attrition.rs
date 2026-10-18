//! `UnitData::get_attrition@00608fd0` under the emulator against the port (item 3078; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `UnitData::get_attrition@00608fd0`, run by `tools/emu/sweep_unit_data_get_attrition.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0..p3 the place(s), in flag order; the global cell 0x00c061e0 points at 85444 random bytes; the global cell 0x00c061e4 points at 3392 random bytes; vtable slot +0x10c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x0046fa40 (0 args) stubbed; the word at `m+0x18` points at 560 random bytes less `0` (placed); the word at `c0+0x6eb8` points at 224 random bytes less `0` (placed); the word at `c0+0xdda4` points at 224 random bytes less `0` (placed); the word at `c0+0x14c90` points at 224 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_get_attrition.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the leader index, 1..=5 the stubs' answers (the object's `siege` slot +0x10c, the object's
                // slot +0x60 and `this`'s slot +0xb8, both `is(0x42, 0)`, `has_preq(0x2fe)`, `is_idle`), 6 the flag
                // that `this`'s slot +0xb8 is `ObjectData::is` itself. `c[0]` the leaders (0x6eec a record), `c[1]` the
                // constants, `p[0]` the object, `p[1 + i]` leader i's second record (its age at +0xdc, xored).
                // The one float, `anti_att * 2^-8` on the scale, is the exact product: the f32's own mantissa and
                // exponent, no float in the arithmetic.
                (|| -> i64 {
                    let ld = |i: usize, o: usize| rd(&c[0], i * 0x6eec + o);
                    let k = |o: usize| rd(&c[1], o);
                    let idx = a[0] as usize;
                    let strength = ld(idx, 0x7f0);
                    if strength == 0 || k(0xd3c) == 0 {
                        return 0;
                    }
                    let mine = usize::from(m[9]);
                    let mut v: i32 = 0x100;
                    if a[1] != 0 {
                        if k(0x1e8) > 99 {
                            return 0;
                        }
                        v = 0x6400 / (100 - k(0x1e8));
                    }
                    let militia = if a[6] != 0 { a[2] } else { a[3] } != 0;
                    if militia {
                        v = (v * 100) / (k(0x1ec) + 100);
                    } else {
                        let bits = ld(mine, 0x7f4) as u32;
                        let (e, mant) = (
                            (bits >> 23) & 0xff,
                            i128::from((bits & 0x7f_ffff) | 0x80_0000),
                        );
                        let prod = if bits & 0x7fff_ffff == 0 {
                            0
                        } else {
                            i128::from(v) * mant
                        };
                        // value = mant * 2^(e - 150), then the scale 2^-8, then the truncation.
                        let sh = 150 - i32::from(e as u8) + 8;
                        v = if prod == 0 {
                            0
                        } else if sh >= 0 {
                            (prod >> sh) as i32
                        } else {
                            (prod << -sh) as i32
                        };
                        if a[4] != 0 && a[5] != 0 {
                            return 0;
                        }
                    }
                    let ty = rd(&p[0], 4);
                    if ty == 0x3d || ty == 0x3e || ty == 400 {
                        v /= 2;
                    }
                    if rd(&p[0], 0x218) == 2 {
                        v /= 2;
                    }
                    let age = |i: usize| rd(&p[1 + i], 0xdc) ^ 0x62766;
                    let diff = age(idx) - age(mine);
                    let mut s = strength;
                    if diff >= 0 {
                        s = (k(0x1f0)
                            .wrapping_mul(diff)
                            .wrapping_add(100)
                            .wrapping_mul(strength)
                            .wrapping_add(99))
                            / 100;
                    }
                    i64::from(v / s)
                })()
            }
        };
        assert_eq!(got, want_eax, "get_attrition — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_attrition — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_attrition — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "get_attrition — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("UnitData::get_attrition@00608fd0: {n} rows agree");
}
