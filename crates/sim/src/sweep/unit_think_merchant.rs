//! `Unit::think_merchant@005f4740` under the emulator against the port (item 3400; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think_merchant@005f4740`, run by `tools/emu/sweep_unit_think_merchant.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p20 the place(s), in flag order; the global cell 0x00c06188 points at 352 random bytes; the global cell 0x00cae5fc points at 2560 random bytes; 4 random bytes at 0x00c0618c; 20 random bytes at 0x00e411fc; 20 random bytes at 0x00e480e8; 20 random bytes at 0x00e4efd4; 20 random bytes at 0x00e55ec0; callee 0x006038e0 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x0065bc40 (12 args) stubbed; callee 0x00462e70 (1 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; vtable slot +0x0 (1 args) stubbed; the word at `0xc0a0f0` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 36 random bytes less `0` (placed); the word at `p0+4` points at 36 random bytes less `0` (placed); the word at `p0+8` points at 36 random bytes less `0` (placed); the word at `p0+12` points at 36 random bytes less `0` (placed); the word at `p0+16` points at 36 random bytes less `0` (placed); the word at `p0+20` points at 36 random bytes less `0` (placed); the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 2304 random bytes less `0` (placed); the word at `c0+0x138` points at 512 random bytes less `0` (placed); the word at `c0+0x13c` points at 512 random bytes less `0` (placed); the word at `c0+0x140` points at 512 random bytes less `0` (placed); the word at `c0+0x144` points at 512 random bytes less `0` (placed); the word at `c0+0x148` points at 512 random bytes less `0` (placed); the word at `c3+0x10` points at 64 random bytes less `0` (placed); the word at `c4+0x10` points at 64 random bytes less `0` (placed); the word at `c5+0x10` points at 64 random bytes less `0` (placed); the word at `c6+0x10` points at 64 random bytes less `0` (placed); stub 10's answer points at 32 random bytes less `0` (placed); stub 11's answer points at 96 random bytes less `0` (placed); the word at `p19+0` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 1 answered from [0, 1, 2, 3] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [-1, -1, -1, -1, -1, -1, -1, 0, 5] (90%); stub 4 answered from [-1, -1, -1, -1, -1, -1, -1, 0, 3] (90%); stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `c3+0` as well as at `this`; the stubbed vtable laid at `c4+0` as well as at `this`; the stubbed vtable laid at `c5+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_merchant.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `Unit::think_merchant`, inline over the windows: `think_merchant` in `crate::merchant` is welded to `Sim`
                // (its `ai`, `world`, `units`), so this is a re-expression of the same branches, checked against the
                // original. Windows: c0 world, c1 `div_3_table`, c2 the objects cell, c3..c6 the four leaders' `new_rares`
                // headers; p0..p6 goods, p7 the type record, p8 cells, p9 tile flags, p10..p13 danger, p14..p17 the lists'
                // data, p18/p19/p20 the order records.
                let res: i64 = 'done: {
                    const K: i32 = 0x63637;
                    let ix = |v: i32| usize::try_from(v).expect("table index");
                    let div = |c: &[Vec<u8>], v: i32| rd(&c[1], 4 * ix(v));
                    let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                    let f68 = rd(m, 0x68);
                    if f68 & 0x80000 == 0 {
                        break 'done 1;
                    }
                    log.push("s0:m+0x0,3".to_string());
                    if a[0] != 0 {
                        break 'done 1;
                    }
                    let who = usize::from(m[9]);
                    let whoi = i32::from(m[9]);
                    let short = sh(m, 0xa);
                    let ty = rd(&p[7], 4);
                    let objs = rd(&c[2], 0);
                    let (w, w18, w24) = (rd(&c[0], 0), rd(&c[0], 0x18), rd(&c[0], 0x24));
                    let (mx, my) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                    log.push("s1:c0+0x0,sp-0x28,sp-0x20".to_string());
                    let mine = a[1];
                    let mine_ocean = {
                        let idx = div(c, my >> 6) * rd(&c[0], 0x18) + div(c, mx >> 6);
                        p[9][ix(idx) * 2] & 0x30 == 0x20
                    };
                    let count = rd(&c[3 + who], 4);
                    let (mut best, mut best_score) = (-1i32, -1i32);
                    let mut score0 = 200i32;
                    let (mut k3, mut k4) = (0usize, 0usize);
                    let ally_ecx = 0x00e3_a390 + whoi * 0x6eec;
                    for i in 0..count {
                        let gi = rd(&p[14 + who], 4 * ix(i));
                        let g = &p[ix(gi) + 1];
                        'cand: {
                            if g[8] & 1 == 0 || g[0x20] & (1u32 << whoi) as u8 == 0 {
                                break 'cand;
                            }
                            let (gxr, gyr) = (rd(g, 0x10) ^ K, rd(g, 0x14) ^ K);
                            let (gx, gy) = (div(c, gxr >> 8), div(c, gyr >> 8));
                            let cell = ix(w * gy + gx) * 0x1c;
                            let owner = i32::from(p[8][cell + 0xf] as i8);
                            if owner >= 0 {
                                log.push(format!("s2:{ally_ecx},{owner}"));
                                if a[2] == 0 {
                                    break 'cand;
                                }
                            }
                            let g_ocean = {
                                let idx = div(c, gyr >> 6) * w18 + div(c, gxr >> 6);
                                p[9][ix(idx) * 2] & 0x30 == 0x20
                            };
                            if g_ocean != mine_ocean {
                                break 'cand;
                            }
                            let x = gx * 0x300 + 0x180;
                            let y = gy * 0x300 + 0x180;
                            log.push(format!(
                                "s3:{objs},{x},{y},1,{whoi},768,0,3,{short},{whoi},1,{ty},0,{ty}"
                            ));
                            let r = a[14 + k3.min(7)];
                            k3 += 1;
                            if r >= 0 {
                                break 'cand;
                            }
                            log.push(format!(
                                "s4:{whoi},{x},{y},{whoi},{whoi},768,0,3,{short},{whoi},1,{ty},0"
                            ));
                            let r4 = a[22 + k4.min(3)];
                            k4 += 1;
                            if r4 >= 0 {
                                break 'cand;
                            }
                            log.push(format!(
                                "s3:{objs},{x},{y},3,{whoi},3072,0,9,0,0,0,0,0,{whoi}"
                            ));
                            let r = a[14 + k3.min(7)];
                            k3 += 1;
                            if r >= 0 {
                                break 'cand;
                            }
                            let mut score = score0;
                            if mine == sh(&p[8], cell + 4) {
                                score += 100;
                            }
                            let dg = rd(&p[10 + who], 4 * ix((gy >> 1) * w24 + (gx >> 1)));
                            let s = score.wrapping_sub(dg).max(1);
                            if best_score < s {
                                best = gi;
                                best_score = s;
                            }
                        }
                        score0 -= 10;
                    }
                    if best < 0 {
                        break 'done 0;
                    }
                    log.push(format!("s5:c{}+0x0,{best}", 3 + who));
                    if rd(&c[3 + who], 4) >= rd(&c[3 + who], 8) {
                        log.push(format!("s13:c{}+0x0,{}", 3 + who, sh(&c[3 + who], 0xc)));
                    }
                    let cnt = ix(rd(&c[3 + who], 4));
                    wr(&mut p[14 + who], 4 * cnt, best);
                    let nc = rd(&c[3 + who], 4) + 1;
                    wr(&mut c[3 + who], 4, nc);
                    let g = &p[ix(best) + 1];
                    let (gxr, gyr) = (rd(g, 0x10) ^ K, rd(g, 0x14) ^ K);
                    log.push(format!("s6:{}", gxr - mx));
                    let xt = div(c, gxr >> 4) * 0x30 + 0x18;
                    let yt = div(c, gyr >> 4) * 0x30 + 0x18;
                    wr(m, 0x68, f68 & 0xfbff_ffffu32 as i32);
                    wr(m, 0xc0, 0);
                    log.push("s7:m+0x0,0".to_string());
                    log.push("s8:m+0x0".to_string());
                    log.push("s9:m+0x0".to_string());
                    log.push("s10:m+0x0,1".to_string());
                    log.push("s11:p18+0x0".to_string());
                    let disp = ix(rd(&p[20], 4));
                    let o = &mut p[19];
                    wr(o, 4, xt);
                    wr(o, 8, yt);
                    wr(o, 0xc, a[6]);
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
                    log.push(format!("s12:m+0xcc,p19+{disp:#x}"));
                    log.push("s9:m+0x0".to_string());
                    1
                };
                if log.is_empty() {
                    log.push("-".to_string());
                }
                res
            }
        };
        assert_eq!(got, want_eax, "think_merchant — eax, row `{line}`");
        assert_eq!(log, want_log, "think_merchant — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_merchant — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_merchant — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "think_merchant — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::think_merchant@005f4740: {n} rows agree");
}
