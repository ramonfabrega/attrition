//! `ObjectsData::find_unit@0065ca80` under the emulator against the port (item 3339; tools/sweep_gen.py, obj with windows).

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

fn rd16u(m: &[u8], off: usize) -> i32 {
    i32::from(u16::from_le_bytes([m[off], m[off + 1]]))
}

/// An inline port: the crate has no general `ObjectsData::find_unit` (its callers specialise: `find_unit_with_radius`,
/// the spellcaster and army searches), so this reads the windows as the original does, calling the crate's own
/// `world::vector_dist` for the distance. Windows: m the `ObjectsData` (`unit_mark` at `+0x15c`, `find_dist`/`find_who`
/// at `+0x1fc`/`+0x200`); c0 `worldc` (width, height, the cell array at `+0x134`), c1 `div_3_table`, c2 `gamec`
/// (`total_units` at `+0x6d8`), c3 `circle_radius`, c4/c5 `circle_x`/`circle_y`; p16..p23 the leaders' flag words,
/// p40 + 3l + i the object `i` of leader `l`, p64 + 3l + i its unit record, p88 the world's cell array. `a[18..26]` are
/// `valid_search`'s answers, `a[26..34]` `valid_filter`'s (both filters share one sequence), `a[34..42]` `is_captain`'s,
/// each in call order with the last repeated; `a[42]` says the slot holds the image's own `is_captain` (bit 15 of the
/// word at `+0x8e`) rather than the stub.
fn find_unit_port(
    a: &[i32],
    m: &mut [u8],
    c: &[Vec<u8>],
    p: &[Vec<u8>],
    log: &mut Vec<String>,
) -> i64 {
    let (x, y, kind, who, r, flags) = (a[0], a[1], a[2], a[3], a[4], a[5]);
    let (f1, p8, p9, f2, p11, p12) = (a[6], a[7], a[8], a[9], a[10], a[11]);
    let (mut n_search, mut n_filter, mut n_cap) = (0usize, 0usize, 0usize);
    let keep = a[42] != 0;
    let div = |v: i32| rd(&c[1], 4 * (v >> 8) as usize);
    let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
    let cell = |i: i32| 28 * i as usize;
    let obj_of = |l: i32, i: i32| 40 + 3 * l as usize + i as usize;
    let mut valid_search = |l: i32, log: &mut Vec<String>| {
        log.push(format!("s2:{l},{kind},{who}"));
        let v = a[18 + n_search.min(7)];
        n_search += 1;
        v != 0
    };
    let mut valid_filter = |l: i32, i: i32, pa: i32, pb: i32, f: i32, log: &mut Vec<String>| {
        log.push(format!("s3:{l},{i},{pa},{pb},{}", f - 1));
        let v = a[26 + n_filter.min(7)];
        n_filter += 1;
        v != 0
    };
    let mut is_captain = |l: i32, i: i32, log: &mut Vec<String>| {
        let u = &p[64 + 3 * l as usize + i as usize];
        if keep {
            return rd16u(u, 0x8e) >> 15 != 0;
        }
        log.push(format!("s4:p{}+0x0", 64 + 3 * l + i));
        let v = a[34 + n_cap.min(7)];
        n_cap += 1;
        v != 0
    };
    wr(m, 0x200, who);
    wr(m, 0x1fc, 99_999_999);
    let mut result = -1i32;
    let cells = &p[88];
    let area = if flags & 0x200 != 0 {
        rd16(cells, cell(div(y) * w + div(x)) + 4)
    } else {
        -1
    };
    let captain = flags & 0x10000 != 0;
    let filt = |f: i32| f != 0 && ((f - 1) as u32) <= 0x16;
    // The measure shared by both arms: the unit's xor-ed position against the point.
    let measure = |o: &[u8]| {
        let dx = (rd(o, 0x10) ^ 0x63637).wrapping_sub(x);
        let dy = (rd(o, 0x14) ^ 0x63637).wrapping_sub(y);
        crate::world::vector_dist(dx, dy)
    };
    let iv9 = r.wrapping_add(0x2ff) / 0x300;
    if r < 0 || rd(&c[3], 4 * iv9 as usize) > rd(&c[2], 0x6d8) {
        for l in 0..8i32 {
            if rd(&p[16 + l as usize], 0) & 1 == 0 || !valid_search(l, log) {
                continue;
            }
            for i in 0..rd(m, 0x15c + 4 * l as usize) {
                let o = &p[obj_of(l, i)];
                let tok = format!("p{}+0x0", obj_of(l, i));
                log.push(format!("s0:{tok}"));
                let mut ok = rd(o, 0x20) != 0;
                if flags & 0x20000 == 0 && ok {
                    log.push(format!("s1:{tok}"));
                    ok = rd(o, 0x24) != 0;
                }
                if !ok
                    || (filt(f1) && !valid_filter(l, i, p8, p9, f1, log))
                    || (filt(f2) && !valid_filter(l, i, p11, p12, f2, log))
                {
                    continue;
                }
                if flags & 0x200 != 0 {
                    let at = cell(div(rd(o, 0x14) ^ 0x63637) * w + div(rd(o, 0x10) ^ 0x63637));
                    if area != rd16(cells, at + 4) {
                        continue;
                    }
                }
                if captain && !is_captain(l, i, log) {
                    continue;
                }
                let d = measure(o);
                if d <= rd(m, 0x1fc) && (r < 0 || d <= r) {
                    wr(m, 0x1fc, d);
                    wr(m, 0x200, l);
                    result = i;
                }
            }
        }
        return i64::from(result);
    }
    let (ix, iy) = (div(x), div(y));
    for k in 0..rd(&c[3], 4 * iv9 as usize) as usize {
        let cx = i32::from(c[4][k] as i8) + ix;
        let cy = i32::from(c[5][k] as i8) + iy;
        if cx < 0 || cy < 0 || cx >= w || cy >= h {
            continue;
        }
        let at = cell(w * cy + cx);
        let (mut idx, mut l) = (rd16(cells, at + 8), rd16(cells, at + 10));
        if idx < 0 || (flags & 0x200 != 0 && area != rd16(cells, at + 4)) {
            continue;
        }
        while idx >= 0 {
            let o = &p[obj_of(l, idx)];
            let (cur_i, cur_l) = (idx, l);
            idx = rd16(o, 0x2c);
            l = rd16(o, 0x2e);
            if cur_l >= 8 || (cur_l >= 0 && !valid_search(cur_l, log)) {
                continue;
            }
            let tok = format!("p{}+0x0", obj_of(cur_l, cur_i));
            log.push(format!("s0:{tok}"));
            if rd(o, 0x20) == 0 {
                continue;
            }
            log.push(format!("s1:{tok}"));
            if rd(o, 0x24) == 0
                || (filt(f1) && !valid_filter(cur_l, cur_i, p8, p9, f1, log))
                || (filt(f2) && !valid_filter(cur_l, cur_i, p11, p12, f2, log))
            {
                continue;
            }
            if captain && !is_captain(cur_l, cur_i, log) {
                continue;
            }
            let d = measure(o);
            if d <= rd(m, 0x1fc) && d <= r {
                wr(m, 0x1fc, d);
                wr(m, 0x200, cur_l);
                result = cur_i;
            }
        }
    }
    i64::from(result)
}

/// `ObjectsData::find_unit@0065ca80`, run by `tools/emu/sweep_objects_data_find_unit.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 13 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p15 sparse table 1 (8 rows × 1 slice(s), row-major), p16..p23 sparse table 2 (8 rows × 1 slice(s), row-major), p24..p88 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c061e8 points at 1760 random bytes; 40 random bytes at 0x00cbe330 (each word in 0..14); 16 random bytes at 0x00cb7e90 (each byte in -3..3); 16 random bytes at 0x00cbb0e0 (each byte in -3..3); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; vtable slot +0xe8 (0 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 64 random bytes less `0` (placed); the word at `p24+4` points at 64 random bytes less `0` (placed); the word at `p24+8` points at 64 random bytes less `0` (placed); the word at `p25+0` points at 64 random bytes less `0` (placed); the word at `p25+4` points at 64 random bytes less `0` (placed); the word at `p25+8` points at 64 random bytes less `0` (placed); the word at `p26+0` points at 64 random bytes less `0` (placed); the word at `p26+4` points at 64 random bytes less `0` (placed); the word at `p26+8` points at 64 random bytes less `0` (placed); the word at `p27+0` points at 64 random bytes less `0` (placed); the word at `p27+4` points at 64 random bytes less `0` (placed); the word at `p27+8` points at 64 random bytes less `0` (placed); the word at `p28+0` points at 64 random bytes less `0` (placed); the word at `p28+4` points at 64 random bytes less `0` (placed); the word at `p28+8` points at 64 random bytes less `0` (placed); the word at `p29+0` points at 64 random bytes less `0` (placed); the word at `p29+4` points at 64 random bytes less `0` (placed); the word at `p29+8` points at 64 random bytes less `0` (placed); the word at `p30+0` points at 64 random bytes less `0` (placed); the word at `p30+4` points at 64 random bytes less `0` (placed); the word at `p30+8` points at 64 random bytes less `0` (placed); the word at `p31+0` points at 64 random bytes less `0` (placed); the word at `p31+4` points at 64 random bytes less `0` (placed); the word at `p31+8` points at 64 random bytes less `0` (placed); the word at `p32+0` points at 160 random bytes less `0` (placed); the word at `p32+4` points at 260 random bytes less `0` (placed); the word at `p32+8` points at 160 random bytes less `0` (placed); the word at `p33+0` points at 160 random bytes less `0` (placed); the word at `p33+4` points at 260 random bytes less `0` (placed); the word at `p33+8` points at 160 random bytes less `0` (placed); the word at `p34+0` points at 160 random bytes less `0` (placed); the word at `p34+4` points at 260 random bytes less `0` (placed); the word at `p34+8` points at 160 random bytes less `0` (placed); the word at `p35+0` points at 160 random bytes less `0` (placed); the word at `p35+4` points at 260 random bytes less `0` (placed); the word at `p35+8` points at 160 random bytes less `0` (placed); the word at `p36+0` points at 160 random bytes less `0` (placed); the word at `p36+4` points at 260 random bytes less `0` (placed); the word at `p36+8` points at 160 random bytes less `0` (placed); the word at `p37+0` points at 160 random bytes less `0` (placed); the word at `p37+4` points at 260 random bytes less `0` (placed); the word at `p37+8` points at 160 random bytes less `0` (placed); the word at `p38+0` points at 160 random bytes less `0` (placed); the word at `p38+4` points at 260 random bytes less `0` (placed); the word at `p38+8` points at 160 random bytes less `0` (placed); the word at `p39+0` points at 160 random bytes less `0` (placed); the word at `p39+4` points at 260 random bytes less `0` (placed); the word at `p39+8` points at 160 random bytes less `0` (placed); the word at `c0+0x134` points at 2024 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; the stubbed vtable laid at `p40+0`; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 2 answers 0..3 every row; stub 3 answers 0..3 every row; stub 4 answers 0..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x20`; stub 1 answers from its own `ecx`: the word at `+0x24`; 8 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); figures p40..p42 link at +0x2c (2 bytes) above their own index or to -1; figures p43..p45 link at +0x2c (2 bytes) above their own index or to -1; figures p46..p48 link at +0x2c (2 bytes) above their own index or to -1; figures p49..p51 link at +0x2c (2 bytes) above their own index or to -1; figures p52..p54 link at +0x2c (2 bytes) above their own index or to -1; figures p55..p57 link at +0x2c (2 bytes) above their own index or to -1; figures p58..p60 link at +0x2c (2 bytes) above their own index or to -1; figures p61..p63 link at +0x2c (2 bytes) above their own index or to -1; figures p40..p42 hold a short at +0x2e drawn from 0..7, after the links; figures p43..p45 hold a short at +0x2e drawn from 0..7, after the links; figures p46..p48 hold a short at +0x2e drawn from 0..7, after the links; figures p49..p51 hold a short at +0x2e drawn from 0..7, after the links; figures p52..p54 hold a short at +0x2e drawn from 0..7, after the links; figures p55..p57 hold a short at +0x2e drawn from 0..7, after the links; figures p58..p60 hold a short at +0x2e drawn from 0..7, after the links; figures p61..p63 hold a short at +0x2e drawn from 0..7, after the links; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; the stubbed vtable laid at `p72+0` as well as at `this`; the stubbed vtable laid at `p73+0` as well as at `this`; the stubbed vtable laid at `p74+0` as well as at `this`; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p82+0` as well as at `this`; the stubbed vtable laid at `p83+0` as well as at `this`; the stubbed vtable laid at `p84+0` as well as at `this`; the stubbed vtable laid at `p85+0` as well as at `this`; the stubbed vtable laid at `p86+0` as well as at `this`; the stubbed vtable laid at `p87+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_unit.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let got = find_unit_port(a, m, c, p, &mut log);
            // A row that called no stub records `-`.
            if log.is_empty() {
                log.push("-".into());
            }
            got
        };
        assert_eq!(got, want_eax, "find_unit — eax, row `{line}`");
        assert_eq!(log, want_log, "find_unit — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_unit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_unit — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "find_unit — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("ObjectsData::find_unit@0065ca80: {n} rows agree");
}
