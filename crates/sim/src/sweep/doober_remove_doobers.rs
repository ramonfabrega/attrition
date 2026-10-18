//! `Doober::remove_doobers@00846250` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `Doober::remove_doobers@00846250`, run by `tools/emu/sweep_doober_remove_doobers.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; 48 random bytes at 0x00eafde0; 4 random bytes at 0x00eaff80; the word at `c0+0xc` points at 64 random bytes less `0` (placed); the word at `c0+0x28` points at 128 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_doober_remove_doobers.py") else {
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
            // The doober tables (`c0`: type count +0, type array +0xc -> p0, location count +0x1c, location
            // array +0x28 -> p1, 8 bytes a location: x then y, f32; `c1` the recalc flag): every location
            // within the octagonal distance a[2] of (a[0], a[1]) — the distance of (|dx|, |dy|) the shorter
            // leg squared over twice the longer, plus the longer (the longer leg plus half the shorter's
            // twice when the shorter is 60000 or more; 0 when the longer is 0) — is replaced by the last
            // location and the last type, both counts drop, the recalc flag is set and the scan stays on
            // the slot.
            let cvt = |w: i32| -> i32 {
                let f = f32::from_bits(w as u32).trunc();
                if f.is_nan() || !(-2147483648.0..2147483648.0).contains(&f) {
                    i32::MIN
                } else {
                    f as i32
                }
            };
            let abs = |v: i32| -> u32 { v.wrapping_abs() as u32 };
            let mut i = 0i32;
            while i < rd(&c[0], 0) {
                let at = i as usize * 8;
                let long_short = |a: u32, b: u32| -> u32 {
                    // a the longer leg (signed compare done by the caller), b the shorter
                    if a == 0 {
                        0
                    } else if b < 60000 {
                        (b.wrapping_mul(b) / a.wrapping_mul(2)).wrapping_add(a)
                    } else {
                        b.wrapping_add(a.wrapping_mul(2)) >> 1
                    }
                };
                let dx = abs(a[0].wrapping_sub(cvt(rd(&p[1], at))));
                let dy = abs(a[1].wrapping_sub(cvt(rd(&p[1], at + 4))));
                let dist = if (dy as i32) < (dx as i32) {
                    long_short(dx, dy)
                } else {
                    long_short(dy, dx)
                };
                if (dist as i32) < a[2] {
                    let n2 = rd(&c[0], 0x1c);
                    let last = (n2 - 1) as usize * 8;
                    let (lx, ly) = (rd(&p[1], last), rd(&p[1], last + 4));
                    wr(&mut p[1], at, lx);
                    wr(&mut p[1], at + 4, ly);
                    let nt = rd(&c[0], 0);
                    let t = rd(&p[0], (nt - 1) as usize * 4);
                    wr(&mut p[0], i as usize * 4, t);
                    wr(&mut c[0], 0, nt - 1);
                    wr(&mut c[0], 0x1c, n2 - 1);
                    wr(&mut c[1], 0, 1);
                    i -= 1;
                }
                i += 1;
            }
            want_eax
        };
        assert_eq!(got, want_eax, "remove_doobers — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "remove_doobers — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_doobers — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "remove_doobers — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Doober::remove_doobers@00846250: {n} rows agree");
}
