//! `ObjectsData::find@0065c6b0` under the emulator against the port (item 3337; tools/sweep_gen.py, obj with windows).

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

fn rd16(m: &[u8], off: usize) -> i32 {
    i32::from(i16::from_le_bytes([m[off], m[off + 1]]))
}

/// `vector_dist@0046cff0` (the octagonal estimate the array arm spells inline) of two absolute deltas.
fn vdist(dx: i32, dy: i32) -> i32 {
    let (hi, lo) = if dy < dx {
        (dx as u32, dy as u32)
    } else {
        (dy as u32, dx as u32)
    };
    (if hi == 0 {
        0
    } else if lo < 60_000 {
        (lo * lo) / (hi * 2) + hi
    } else {
        (lo + hi * 2) >> 1
    }) as i32
}

/// An inline port: the crate has no `ObjectsData::find` (only the spellcaster's view-trait stub), so this reads the
/// windows as the original does. Windows: p0..p7 the leaders' flag bytes, p(8 + 5s) the table s's pointer array and
/// p(9 + 5s + i) its object i, p48..p50 the three count arrays, p51 the world's cell array; c0 `objects`, c1 `worldc`,
/// c2 `div_3_table`, c3 `circle_radius`, c4/c5 `circle_x`/`circle_y`, c6 `obj_base`. `a[15..31]` and `a[31..47]` are
/// `valid_search`'s and `valid_filter`'s answers, in call order, the last repeated.
fn find_port(a: &[i32], c: &mut [Vec<u8>], p: &[Vec<u8>], log: &mut Vec<String>) -> i64 {
    let (x, y, p3, p4, r, filt, p8, p9, early) =
        (a[0], a[1], a[2], a[3], a[4], a[6], a[7], a[8], a[12] != 0);
    let (mut n_search, mut n_filter) = (0usize, 0usize);
    let mut valid_search = |s: i32, log: &mut Vec<String>| {
        log.push(format!("s0:{s},{p3},{p4}"));
        let v = a[15 + n_search.min(15)];
        n_search += 1;
        v != 0
    };
    let mut valid_filter = |s: i32, i: i32, log: &mut Vec<String>| {
        log.push(format!("s1:{s},{i},{p8},{p9},{}", filt - 1));
        let v = a[31 + n_filter.min(15)];
        n_filter += 1;
        v != 0
    };
    let obj = |s: i32, i: i32| &p[(9 + 5 * s + i) as usize];
    wr(&mut c[0], 0x200, p4);
    wr(&mut c[0], 0x1fc, 99_999_999);
    let mut result = -1;
    let iv6 = r.wrapping_add(0x2ff) / 0x300;
    let filtered = filt != 0 && (filt - 1) as u32 <= 0x16;
    if r > -1 && iv6 < 10 {
        let ix = rd(&c[2], 4 * (x >> 8) as usize);
        let iy = rd(&c[2], 4 * (y >> 8) as usize);
        let (w, h) = (rd(&c[1], 0), rd(&c[1], 4));
        for k in 0..rd(&c[3], 4 * iv6 as usize) as usize {
            let cx = i32::from(c[4][k] as i8) + ix;
            let cy = i32::from(c[5][k] as i8) + iy;
            if cx < 0 || cy < 0 || cx >= w || cy >= h {
                continue;
            }
            let at = 28 * (w * cy + cx) as usize;
            let mut idx = rd16(&p[51], at + 8);
            let mut s = rd16(&p[51], at + 10);
            while idx >= 0 {
                let o = obj(s, idx);
                let (cur_i, cur_s) = (idx, s);
                idx = rd16(o, 0x2c);
                s = rd16(o, 0x2e);
                if cur_s < 8
                    && o[8] & 1 != 0
                    && (cur_s < 0 || valid_search(cur_s, log))
                    && (!filtered || valid_filter(cur_s, cur_i, log))
                {
                    let dx = ((rd(o, 0x10) ^ 0x63637).wrapping_sub(x)).abs();
                    let dy = ((rd(o, 0x14) ^ 0x63637).wrapping_sub(y)).abs();
                    let d = vdist(dx, dy);
                    if d <= rd(&c[0], 0x1fc) && d <= r {
                        wr(&mut c[0], 0x1fc, d);
                        wr(&mut c[0], 0x200, cur_s);
                        result = cur_i;
                        if early {
                            return i64::from(cur_i);
                        }
                    }
                }
            }
        }
        return i64::from(result);
    }
    for l in 0..8i32 {
        if p[l as usize][0] & 1 == 0 || !valid_search(l, log) {
            continue;
        }
        for k in 0..3usize {
            let count = rd(&p[48 + k], 4 * l as usize);
            let mut i = rd(&c[6], 4 * k);
            while i < count {
                let o = obj(l, i);
                if o[8] & 1 != 0 && (!filtered || valid_filter(l, i, log)) {
                    let dx = ((rd(o, 0x10) ^ 0x63637).wrapping_sub(x)).abs();
                    let dy = ((rd(o, 0x14) ^ 0x63637).wrapping_sub(y)).abs();
                    let d = vdist(dx, dy);
                    if d <= rd(&c[0], 0x1fc) && (r < 0 || d <= r) {
                        wr(&mut c[0], 0x1fc, d);
                        wr(&mut c[0], 0x200, l);
                        result = i;
                        if early {
                            return i64::from(i);
                        }
                    }
                }
                i += 1;
            }
        }
    }
    i64::from(result)
}

/// `ObjectsData::find@0065c6b0`, run by `tools/emu/sweep_objects_data_find.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 13 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p51 the place(s), in flag order; the global cell 0x00c0618c points at 520 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 40 random bytes at 0x00cbe330 (each word in 0..14); 16 random bytes at 0x00cb7e90 (each byte in -3..3); 16 random bytes at 0x00cbb0e0 (each byte in -3..3); 12 random bytes at 0x00cab2fc (each word in 0..3); callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; the word at `0xc0ab84+0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 48 random bytes less `0` (placed); the word at `p8+4` points at 48 random bytes less `0` (placed); the word at `p8+8` points at 48 random bytes less `0` (placed); the word at `p8+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+28` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 48 random bytes less `0` (placed); the word at `p13+4` points at 48 random bytes less `0` (placed); the word at `p13+8` points at 48 random bytes less `0` (placed); the word at `p13+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+56` points at 16 random bytes less `0` (placed); the word at `p18+0` points at 48 random bytes less `0` (placed); the word at `p18+4` points at 48 random bytes less `0` (placed); the word at `p18+8` points at 48 random bytes less `0` (placed); the word at `p18+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+84` points at 16 random bytes less `0` (placed); the word at `p23+0` points at 48 random bytes less `0` (placed); the word at `p23+4` points at 48 random bytes less `0` (placed); the word at `p23+8` points at 48 random bytes less `0` (placed); the word at `p23+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+112` points at 16 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `p28+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+140` points at 16 random bytes less `0` (placed); the word at `p33+0` points at 48 random bytes less `0` (placed); the word at `p33+4` points at 48 random bytes less `0` (placed); the word at `p33+8` points at 48 random bytes less `0` (placed); the word at `p33+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+168` points at 16 random bytes less `0` (placed); the word at `p38+0` points at 48 random bytes less `0` (placed); the word at `p38+4` points at 48 random bytes less `0` (placed); the word at `p38+8` points at 48 random bytes less `0` (placed); the word at `p38+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+196` points at 16 random bytes less `0` (placed); the word at `p43+0` points at 48 random bytes less `0` (placed); the word at `p43+4` points at 48 random bytes less `0` (placed); the word at `p43+8` points at 48 random bytes less `0` (placed); the word at `p43+12` points at 48 random bytes less `0` (placed); the word at `0xc0ad58+0` points at 32 random bytes less `0` (placed); the word at `0xc0ad58+4` points at 32 random bytes less `0` (placed); the word at `0xc0ad58+8` points at 32 random bytes less `0` (placed); the word at `c1+0x134` points at 1024 random bytes less `0` (placed); stub 0 answers 16 values in call order, printed after the answers; stub 1 answers 16 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); figures p9..p12 link at +0x2c (2 bytes) above their own index or to -1; figures p14..p17 link at +0x2c (2 bytes) above their own index or to -1; figures p19..p22 link at +0x2c (2 bytes) above their own index or to -1; figures p24..p27 link at +0x2c (2 bytes) above their own index or to -1; figures p29..p32 link at +0x2c (2 bytes) above their own index or to -1; figures p34..p37 link at +0x2c (2 bytes) above their own index or to -1; figures p39..p42 link at +0x2c (2 bytes) above their own index or to -1; figures p44..p47 link at +0x2c (2 bytes) above their own index or to -1; figures p9..p12 hold a short at +0x2e drawn from 0..7, after the links; figures p14..p17 hold a short at +0x2e drawn from 0..7, after the links; figures p19..p22 hold a short at +0x2e drawn from 0..7, after the links; figures p24..p27 hold a short at +0x2e drawn from 0..7, after the links; figures p29..p32 hold a short at +0x2e drawn from 0..7, after the links; figures p34..p37 hold a short at +0x2e drawn from 0..7, after the links; figures p39..p42 hold a short at +0x2e drawn from 0..7, after the links; figures p44..p47 hold a short at +0x2e drawn from 0..7, after the links; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let got = find_port(a, c, p, &mut log);
            // A row that called neither stub records `-`.
            if log.is_empty() {
                log.push("-".into());
            }
            got
        };
        assert_eq!(got, want_eax, "find — eax, row `{line}`");
        assert_eq!(log, want_log, "find — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "find — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("ObjectsData::find@0065c6b0: {n} rows agree");
}
