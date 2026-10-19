//! `Unit::think_fish@005f4c60` under the emulator against the port (item 3282; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think_fish@005f4c60`, run by `tools/emu/sweep_unit_think_fish.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p19 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; 1156 random bytes at 0x00adcaf0 (each word in -4..4); 1156 random bytes at 0x00adc400 (each word in -4..4); callee 0x00609180 (10 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0065bec0 (5 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p0+4` points at 224 random bytes less `0` (placed); the word at `p0+8` points at 224 random bytes less `0` (placed); the word at `p0+12` points at 224 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); the word at `p5+4` points at 224 random bytes less `0` (placed); the word at `p5+8` points at 224 random bytes less `0` (placed); the word at `p5+12` points at 224 random bytes less `0` (placed); the word at `0xc0aef8` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 224 random bytes less `0` (placed); the word at `p10+4` points at 224 random bytes less `0` (placed); the word at `p10+8` points at 224 random bytes less `0` (placed); the word at `p10+12` points at 224 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `c1+0x134` points at 2304 random bytes less `0` (placed); stub 9's answer points at 32 random bytes less `0` (placed); stub 10's answer points at 96 random bytes less `0` (placed); the word at `p18+0` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 1, 2, 3, -1, 65535] (90%); stub 2 answered from [-1, -1, -1, 0, 5] (90%); stub 2 answers 300 values in call order, printed after the answers; stub 3 answers 300 values in call order, printed after the answers; stub 3 answers 0..65535 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 writes a value in [0, 0, 1, 5] through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p17+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_fish.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `Unit::think_fish`: the head (cadence, `calc_gather`, the `0x20` bit), the 289-cell walk (`crate::fish::best_cell`
                // over `crate::fish::cell_base`), the own-cell cast, and the tail that builds a MOVE_TO order. The tail has no
                // crate method (`add_move_order` is welded to the Sim), so the order object's bytes are written inline.
                let res: i64 = 'done: {
                    let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                    let ix = |v: i32| usize::try_from(v).expect("table index");
                    let div = |c: &[Vec<u8>], v: i32| rd(&c[2], 4 * ix(v));
                    let ty_bit = p[15][0x2b8] & 4 != 0;
                    let mut f68 = rd(m, 0x68);
                    let packed = f68 & 0x80000 != 0;
                    let me = i16::from_le_bytes([m[0xa], m[0xb]]);
                    if ty_bit && !packed {
                        if !crate::fish::recheck_due(me, i64::from(rd(&c[0], 0x550))) {
                            break 'done 0;
                        }
                        log.push(
                            "s0:m+0x0,m+0x54,sp-0x18,m+0x94,sp-0x40,0,0,1,1,-1,-1".to_string(),
                        );
                        if a[12] == 0 {
                            f68 &= !0x20;
                        } else {
                            f68 |= 0x20;
                        }
                        wr(m, 0x68, f68);
                        if a[0] != 0 {
                            break 'done 0;
                        }
                    }
                    let (w, h) = (rd(&c[1], 0), rd(&c[1], 4));
                    let here = (
                        div(c, (rd(m, 0x10) ^ 0x63637) >> 8),
                        div(c, (rd(m, 0x14) ^ 0x63637) >> 8),
                    );
                    let moves: Vec<(i32, i32)> = (0..0x121)
                        .map(|i| (rd(&c[3], 4 * i), rd(&c[4], 4 * i)))
                        .collect();
                    let who = i32::from(m[9]);
                    let (mut kg, mut kr) = (0usize, 0usize);
                    let best = crate::fish::best_cell(here, &moves, |_, (x, y)| {
                        if x < 0 || y < 0 || x >= w || y >= h {
                            return None;
                        }
                        let off = ix(w * y + x) * 0x1c;
                        let cell = &p[16][off..off + 0x1c];
                        let flags = u16::from_le_bytes([cell[0], cell[1]]);
                        let base = crate::fish::cell_base(
                            flags,
                            cell[2] as i8,
                            sh(cell, 4) == a[1],
                            || {
                                log.push(format!("s2:p16+{off:#x},{x},{y},{who},0,0"));
                                kg += 1;
                                a[13 + (kg - 1).min(299)] >= 0
                            },
                            || Some((sh(cell, 8) as i16, sh(cell, 10) as i16)),
                            me,
                        )?;
                        kr += 1;
                        Some((base, a[313 + (kr - 1).min(299)]))
                    });
                    let (dx, dy) = moves[best];
                    if dx == 0 && dy == 0 {
                        if ty_bit && packed {
                            log.push("s4:m+0x0,-1,-1,-1,-1,652,0,0".to_string());
                            break 'done 1;
                        }
                        break 'done 0;
                    }
                    let (tx, ty) = (here.0 + dx, here.1 + dy);
                    let lu = 5 * usize::from(m[9])
                        + 1
                        + usize::from(u16::from_le_bytes([m[0xa], m[0xb]]));
                    let f = rd(&p[lu], 0x68) & 0xfbff_ffffu32 as i32;
                    wr(&mut p[lu], 0x68, f);
                    wr(&mut p[lu], 0xc0, 0);
                    let u = format!("p{lu}+0x0");
                    log.push(format!("s6:{u},0"));
                    log.push(format!("s7:{u}"));
                    log.push(format!("s8:{u}"));
                    log.push(format!("s9:{u},1"));
                    let disp = ix(rd(&p[19], 4));
                    let xt = div(c, (tx * 0x300 + 0x180) >> 4) * 0x30 + 0x18;
                    let yt = div(c, (ty * 0x300 + 0x180) >> 4) * 0x30 + 0x18;
                    let o = &mut p[18];
                    wr(o, 4, xt);
                    wr(o, 8, yt);
                    wr(o, 0xc, a[5]);
                    wr(o, 0x10, 0);
                    wr(o, 0x2c, xt);
                    wr(o, 0x30, yt);
                    wr(o, 0x34, -1);
                    wr(o, 0x38, -1);
                    o[disp + 4] &= 0xda;
                    o[0x4c..0x4e].copy_from_slice(&((xt % 0x300) as i16).to_le_bytes());
                    o[0x4e..0x50].copy_from_slice(&((yt % 0x300) as i16).to_le_bytes());
                    wr(o, 0x44, -1);
                    wr(o, 0x48, -1);
                    wr(o, 0x18, 0);
                    wr(o, 0x1c, 0);
                    wr(o, 0x24, 0);
                    wr(o, 0x28, -1);
                    log.push(format!("s11:p{lu}+0xcc,p18+{disp:#x}"));
                    log.push(format!("s8:{u}"));
                    1
                };
                if log.is_empty() {
                    log.push("-".to_string());
                }
                res
            }
        };
        assert_eq!(got, want_eax, "think_fish — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "think_fish — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_fish — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_fish — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "think_fish — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Unit::think_fish@005f4c60: {n} rows agree");
}
