//! `Fractal::get_height@006aa870` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `Fractal::get_height@006aa870`, run by `tools/emu/sweep_fractal_get_height.py` under unicorn
/// (tools/emu): 120 random bytes of object, 2 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; the word at `m+0x10` points at 140 random bytes less `0` (placed); the word at `p0+0x10` points at 64 random bytes less `0` (placed); the word at `p0+0x2c` points at 64 random bytes less `0` (placed); the word at `p0+0x48` points at 64 random bytes less `0` (placed); the word at `p0+0x64` points at 64 random bytes less `0` (placed); the word at `p0+0x80` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_fractal_get_height.py") else {
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
            // Bilinear sample of the byte rows (record `i` = (int)(x_inc * (x + 0.5)) and its successor, each
            // a 0x1c-byte `Array` whose data pointer sits at +0x10; byte `j` = (int)(y_inc * (y + 0.5)) and
            // the next), summed in the executable's order, truncated and clamped to 0..255; then the
            // `parts` thresholds (+0x24, 16 ints, -1 ends) give the first index above the height, else the
            // flags (+0x20) bit 1 scales it by 100/256.
            let d =
                |off: usize| f64::from_le_bytes(m[off..off + 8].try_into().expect("eight bytes"));
            let (dx, dy) = (
                d(0x68) * (f64::from(a[0]) + 0.5),
                d(0x70) * (f64::from(a[1]) + 0.5),
            );
            let (i, j) = (dx as i32, dy as i32);
            let (fx, fy) = (dx - f64::from(i), dy - f64::from(j));
            let row = |r: usize| -> &Vec<u8> { &p[1 + r] };
            let (r0, r1, j) = (row(i as usize), row(i as usize + 1), j as usize);
            let (ax, ay) = (1.0 - fx, 1.0 - fy);
            let mut sum = ax * ay * f64::from(r0[j]);
            sum += 0.0;
            sum += ay * fx * f64::from(r1[j]);
            sum += ax * fy * f64::from(r0[j + 1]);
            sum += f64::from(r1[j + 1]) * (fy * fx);
            let h = (sum as i32).clamp(0, 255);
            let parts = |k: usize| rd(m, 0x24 + 4 * k);
            let ans = if parts(0) != -1 {
                let mut k = 0;
                while k < 16 && parts(k) != -1 && h >= parts(k) {
                    k += 1;
                }
                k as i32
            } else if rd(m, 0x20) & 2 != 0 {
                h * 100 >> 8
            } else {
                h
            };
            i64::from(ans & 0xff)
        };
        assert_eq!(got & 0xff, want_eax & 0xff, "get_height — al, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_height — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_height — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_height — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("Fractal::get_height@006aa870: {n} rows agree");
}
