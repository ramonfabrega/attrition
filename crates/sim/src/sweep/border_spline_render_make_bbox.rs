//! `BorderSplineRender::make_bbox@008826f0` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `BorderSplineRender::make_bbox@008826f0`, run by `tools/emu/sweep_border_spline_render_make_bbox.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the word at `m+0x30` points at 96 random bytes less `0` (placed); xmm 1 an integer in 1..16, loaded as that over four (after the other numbers on the row); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_border_spline_render_make_bbox.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `bbox` (+0x18 x0, +0x1a x1, +0x1c y0, +0x1e y1, shorts) from the vertex list (+0x30, 12-byte
            // stride, x then y as f32) scaled by xmm1 (= a[0] / 4) and truncated (cvttss2si: out of range
            // or NaN is 0x80000000); the first vertex seeds the box, the rest widen it, then both minima
            // drop by one.
            let scale = a[0] as f32 / 4.0;
            let cvt = |f: f32| -> i32 {
                let t = f.trunc();
                if t.is_nan() || !(-2147483648.0..2147483648.0).contains(&t) {
                    i32::MIN
                } else {
                    t as i32
                }
            };
            let vf = |p: &[u8], i: usize, k: usize| f32::from_bits(rd(p, i * 12 + k * 4) as u32);
            let n = rd(m, 0x24);
            let sh = |m: &[u8], o: usize| i16::from_le_bytes([m[o], m[o + 1]]) as i32;
            let put = |m: &mut [u8], o: usize, v: i32| {
                m[o..o + 2].copy_from_slice(&(v as i16).to_le_bytes())
            };
            let x0 = cvt(vf(&p[0], 0, 0) * scale);
            let y0 = cvt(vf(&p[0], 0, 1) * scale);
            for o in [0x18, 0x1a] {
                put(m, o, x0);
            }
            for o in [0x1c, 0x1e] {
                put(m, o, y0);
            }
            if n > 1 {
                let (mut lx, mut ly) = (i32::from(x0 as u16), i32::from(y0 as u16));
                for i in 1..n as usize {
                    let (x, y) = (cvt(vf(&p[0], i, 0) * scale), cvt(vf(&p[0], i, 1) * scale));
                    if x < i32::from(lx as i16) {
                        lx = i32::from(x as u16);
                        put(m, 0x18, x);
                    } else if sh(m, 0x1a) < x {
                        put(m, 0x1a, x);
                    }
                    if y < i32::from(ly as i16) {
                        ly = i32::from(y as u16);
                        put(m, 0x1c, y);
                    } else if sh(m, 0x1e) < y {
                        put(m, 0x1e, y);
                    }
                }
            }
            for o in [0x18, 0x1c] {
                let v = sh(m, o) - 1;
                put(m, o, v);
            }
            want_eax
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "make_bbox — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "make_bbox — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "make_bbox — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("BorderSplineRender::make_bbox@008826f0: {n} rows agree");
}
