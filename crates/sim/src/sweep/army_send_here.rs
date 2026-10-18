//! `Army::send_here@006f98a0` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

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

/// `Army::send_here@006f98a0`, run by `tools/emu/sweep_army_send_here.py` under unicorn
/// (tools/emu): 160 random bytes of object, 3 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00e85f20 points at 8192 random bytes; callee 0x0092d130 (0 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x00a46a00 (0 args) stubbed; the word at `0xcae5fc` points at 64 random bytes less `0` (placed); stub 2 answers 8 values in call order, printed after the answers; stub 0 answers -2147483648..2147483647 every row; stub 2 answers -500..500 every row; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_send_here.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
            let wr = |w: &mut [u8], o: usize, v: i32| w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            // a[3] the find_angle stub's answer, a[6..14] the sin_table stub's run; windows: c0 the world (width, height),
            // c1 the groups (0x9d4 apart), p0 the div_3_table.
            let (mut x, mut y) = (a[0].max(0), a[1].max(0));
            if x >= rd(&c[0], 0).wrapping_mul(0x300) {
                x = rd(&c[0], 0).wrapping_mul(0x300) - 1;
            }
            if y >= rd(&c[0], 4).wrapping_mul(0x300) {
                y = rd(&c[0], 4).wrapping_mul(0x300) - 1;
            }
            let mut l0 = Vec::new();
            let mut l1 = Vec::new();
            let mut l2 = Vec::new();
            if x != rd(m, 0x38) && y != rd(m, 0x3c) {
                l0.push(format!("{},{}", x.wrapping_sub(rd(m, 0x38)), y.wrapping_sub(rd(m, 0x3c))));
                wr(m, 0x50, a[3]);
            }
            wr(m, 0x38, x);
            wr(m, 0x3c, y);
            wr(m, 0x48, rd(&p[0], 4 * (x >> 8) as usize));
            wr(m, 0x4c, rd(&p[0], 4 * (y >> 8) as usize));
            let (mut px, mut py) = (x, y);
            let mut k_sin = 0usize;
            let n = i16::from_le_bytes([m[0x96], m[0x97]]) as i32;
            // sin_table's arguments from a direction: the sign picks the amplitude, bit 30 folds the angle.
            let sin_args = |v: i32| {
                let amp = if v < 0 { -0x180 } else { 0x180 };
                let t = v & 0x7fff_ffff;
                (if t & 0x4000_0000 != 0 { 0x7fff_ffff - t } else { t }, amp)
            };
            for k in 0..n.max(0) {
                let g = rd(m, 0x54 + 4 * k as usize);
                if g < 0 || rd(&c[1], 0x9d4 * g as usize + 0xc) == 0 {
                    continue;
                }
                wr(&mut c[1], 0x9d4 * g as usize + 8, i32::from(i16::from_le_bytes([m[2], m[3]])));
                l1.push(format!(
                    "c1+{:#x},{},{},{},2,1,{},{},1,-1,-1,0",
                    0x9d4 * g as usize,
                    k,
                    px,
                    py,
                    rd(m, 0x50),
                    a[2]
                ));
                let ang = rd(m, 0x50) ^ i32::MIN;
                let (c0, e0) = sin_args(ang);
                l2.push(format!("{},{}", c0, e0));
                px = px.wrapping_add(a[6 + k_sin.min(7)]);
                k_sin += 1;
                let (c1, e1) = sin_args(ang.wrapping_add(0x4000_0000));
                l2.push(format!("{},{}", c1, e1));
                py = py.wrapping_sub(a[6 + k_sin.min(7)]);
                k_sin += 1;
            }
            for l in [l0, l1, l2] {
                log.push(if l.is_empty() { "-".to_string() } else { l.join("/") });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "send_here — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "send_here — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "send_here — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "send_here — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Army::send_here@006f98a0: {n} rows agree");
}
