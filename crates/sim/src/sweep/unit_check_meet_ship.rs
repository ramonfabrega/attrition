//! `Unit::check_meet_ship@00604550` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

/// `Unit::check_meet_ship@00604550`, run by `tools/emu/sweep_unit_check_meet_ship.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p8 the place(s), in flag order, p9..p14 the stubs' answer window(s); the global cell 0x00c061f0 points at 256 random bytes; the global cell 0x00cae5fc points at 20480 random bytes; the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00c061b8 points at 64 random bytes; the global cell 0x00c0618c points at 256 random bytes; callee 0x006483c0 (2 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x00461420 (2 args) stubbed; callee 0x00681020 (3 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e4c80 (4 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x00616ed0 (9 args) stubbed; the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[0]` points at 256 random bytes less `0` (placed); the word at `c4+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p2+4*a[0]` points at 32 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `c2+0x134` points at 8192 random bytes less `0` (placed); the word at `c3+0x10` points at 1024 random bytes less `0` (placed); the word at `c4+0x40` points at 16 random bytes less `0` (placed); the word at `c4+0x44` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 8 answered from [0, 1, 1] (90%); stub 16 answered from [9, 9, 9, 8] (90%); stub 7 answers the address of -64 random bytes, a `p` window after the places; stub 9 answers the address of 64 random bytes, a `p` window after the places; stub 17 answers the address of 64 random bytes, a `p` window after the places; stub 18 answers the address of 64 random bytes, a `p` window after the places; stub 10 answers a window of its own to each call, in call order (256 bytes, ...); the stubbed vtable laid at `p9+0`; the word at `p11+0` laid as the address of `p7` every row; the word at `p12+0` laid as the address of `p8` every row; stub 1 answers 40 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 5 answers 81 values in call order, printed after the answers; stub 6 answers 81 values in call order, printed after the answers; stub 1 answers 0..1 every row; stub 2 answers 0..3 every row; stub 5 answers 0..1 every row; stub 6 answers 0..8 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 13 leaves `ecx` as a callee would, a value from [0, 5, 77], after the other numbers on the row; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_check_meet_ship.py") else {
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
                // `Unit::check_meet_ship(this, a[0] = slot, a[1] = player)`, ported inline (no crate method). `a[2 + k]` stub k's answer;
                // the seqs follow in flag order (`invalid_loc` a[22..62], `get_tregion` a[62..64], `valid` a[64..145],
                // `coast_here` a[145..226]), then the null flag (a[226]) and the `update_action` stub's `ecx` (a[227]).
                // Windows: p0 the units array, p1 the ship, p2 the objects array, p3 the target, p4 what `this+0xdc` points at,
                // p5 the world grid, p6 the regions, p7/p8 the two orders' vbtables, p9 `get_action`'s answer, p10 `get_obj`'s,
                // p11/p12 the two orders `slot 0x40` answers, p13 `update_order`'s answer, p14 `slot 0x3c`'s.
                const K: i32 = 0x63637;
                const MOVE_X: [i32; 81] = [
                    0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2,
                    2, -2, -3, -2, -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0, -1, -2, -3, -3, -3,
                    -3, -3, -3, -4, -3, -2, -1, 0, 1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 2, 1, 0,
                    -1, -2, -3, -4, -4, -4, -4, -4, -4, -4, -4,
                ];
                const MOVE_Y: [i32; 81] = [
                    0, -1, -1, -1, 0, 1, 1, 1, 0, -2, -2, -2, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2,
                    2, 2, -3, -3, -3, -3, -3, -3, -3, -2, -1, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 2, 1,
                    0, -1, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -3, -2, -1, 0, 1, 2, 3, 4, 4, 4,
                    4, 4, 4, 4, 4, 4, 3, 2, 1, 0, -1, -2, -3,
                ];
                // The x candidate's offsets (image 0xadd254) and the y candidate's (0xadd214), the far arm's four orthogonal steps.
                const ORTH_X: [i32; 4] = [0, 1, 0, -1];
                const ORTH_Y: [i32; 4] = [-1, 0, 1, 0];

                // `vector_dist(ecx = |dx|, edx = |dy|)`: the longer leg plus a quadratic share of the shorter, linear past 60000.
                fn vector_dist(ecx: i32, edx: i32) -> i32 {
                    let a = ecx.wrapping_abs() as u32;
                    let b = edx.wrapping_abs() as u32;
                    if (b as i32) < (a as i32) {
                        if a != 0 {
                            if 59999 < b {
                                return (b.wrapping_add(a.wrapping_mul(2)) >> 1) as i32;
                            }
                            return (b.wrapping_mul(b) / a.wrapping_mul(2)).wrapping_add(a) as i32;
                        }
                    } else if b != 0 {
                        if 59999 < a {
                            return (a.wrapping_add(b.wrapping_mul(2)) >> 1) as i32;
                        }
                        return (a.wrapping_mul(a) / b.wrapping_mul(2)).wrapping_add(b) as i32;
                    }
                    0
                }
                // What `find_angle` leaves in `ecx` on return (a register the caller never reads, but a stubbed callee after it
                // records): `dx` itself on the axes, else the straight-line correction term it computed last.
                fn find_angle_ecx(dx: i32, dy: i32) -> i32 {
                    let cy = dy.wrapping_neg();
                    if dx == 0 {
                        return 0;
                    }
                    if cy == 0 {
                        return dx;
                    }
                    let (a, b) = (dx.wrapping_abs(), cy.wrapping_abs());
                    let (hi, lo) = if b < a { (a, b) } else { (b, a) };
                    let t = lo.wrapping_mul(0x4000).wrapping_div(hi);
                    0x1333i32.wrapping_sub(t).wrapping_abs().wrapping_mul(0xb00) >> 14
                }
                // A fresh MOVE_TO order written into its window `r`; `vb` is the vbtable its first word names.
                fn order_fill(r: &mut [u8], vb: &[u8], x: i32, y: i32, ang: i32) {
                    wr(r, 4, x);
                    wr(r, 8, y);
                    wr(r, 0xc, ang);
                    wr(r, 0x10, 0);
                    wr(r, 0x2c, x);
                    wr(r, 0x30, y);
                    wr(r, 0x34, -1);
                    wr(r, 0x38, -1);
                    let d = 4 + rd(vb, 4) as usize;
                    r[d] &= 0xfe;
                    r[0x4c..0x4e].copy_from_slice(
                        &(x.wrapping_sub((x / 768).wrapping_mul(768)) as i16).to_le_bytes(),
                    );
                    r[0x4e..0x50].copy_from_slice(
                        &(y.wrapping_sub((y / 768).wrapping_mul(768)) as i16).to_le_bytes(),
                    );
                    wr(r, 0x44, -1);
                    wr(r, 0x48, -1);
                    wr(r, 0x18, 0);
                    wr(r, 0x1c, 0);
                    wr(r, 0x24, 0);
                    wr(r, 0x28, -1);
                    r[d] &= 0xfb;
                    r[d] &= 0xdf;
                }
                let hexw = |v: i32| format!("{:08x}", (v as u32).swap_bytes());
                let (slot_i, pl) = (a[0], a[1]);
                let slot = format!("p0+{:#x}", 4 * slot_i);
                let div3 = |c: &[Vec<u8>], i: i32| rd(&c[1], 4 * i as usize);
                let (mut n1, mut n5, mut n6) = (0usize, 0usize, 0usize);
                let (mut d2a, mut d2b, mut d5a, mut d5b): (
                    Vec<String>,
                    Vec<String>,
                    Vec<String>,
                    Vec<String>,
                ) = (vec![], vec![], vec![], vec![]);
                let this_short = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
                let this_byte = m[9] as i32;
                let ecxo = a[227];
                let mut eax = 1i64;
                'done: {
                    log.push(format!("s0:p1+0x0,{this_short},{this_byte}"));
                    if a[2] == 0 {
                        eax = 0;
                        break 'done;
                    }
                    let (tx, ty) = (rd(m, 0x10) ^ K, rd(m, 0x14) ^ K);
                    let (gx, gy) = (rd(&p[3], 0x10) ^ K, rd(&p[3], 0x14) ^ K);
                    let dist = vector_dist((tx - gx).wrapping_abs(), (ty - gy).wrapping_abs());
                    if rd(&c[0], 0x98) >= dist {
                        eax = 0;
                        break 'done;
                    }
                    // The spot the two meet at: (x0, y0) for the unit's order, (x1, y1) for the ship's.
                    let (x0, y0, x1, y1);
                    if dist >= 0x900 {
                        let (ax, ay) = (div3(c, tx >> 8), div3(c, ty >> 8));
                        let (mut bx, mut by) = (ax, ay);
                        for j in 0..4 {
                            let (cx, cy) = (ax + ORTH_X[j], ay + ORTH_Y[j]);
                            log.push(format!(
                                "s1:p1+0x0,{},{},0,0,0,0,0,{slot}",
                                cx * 4 + 2,
                                cy * 4 + 2
                            ));
                            let v = a[22 + n1.min(39)];
                            n1 += 1;
                            if v == 0 {
                                bx = cx;
                                by = cy;
                            }
                        }
                        x1 = bx.wrapping_mul(0x300).wrapping_add(0x180);
                        y1 = by.wrapping_mul(0x300).wrapping_add(0x180);
                        x0 = (gx + tx) / 2;
                        y0 = (gy + ty) / 2;
                    } else if dist <= 0xbf {
                        x0 = (gx + tx) / 2;
                        y0 = (gy + ty) / 2;
                        x1 = x0;
                        y1 = y0;
                    } else {
                        let (ux, uy) = (div3(c, tx >> 6), div3(c, ty >> 6));
                        d2a.push(hexw(ux));
                        d2b.push(hexw(uy));
                        log.push("s2:c2+0x0,sp-0x2c,sp-0x30".to_string());
                        let r1 = a[62];
                        let (vx, vy) = (div3(c, gx >> 6), div3(c, gy >> 6));
                        d2a.push(hexw(vx));
                        d2b.push(hexw(vy));
                        log.push("s2:c2+0x0,sp-0x2c,sp-0x30".to_string());
                        let r2 = a[63];
                        let reg = format!("p6+{:#x}", r1.wrapping_mul(0x88));
                        log.push(format!("s3:{reg},{r2}"));
                        if a[5] == 0 {
                            log.push("s4:m+0x0,0".to_string());
                            break 'done;
                        }
                        let (mx, my) = ((gx + tx) / 2, (gy + ty) / 2);
                        let (xt, yt) = (div3(c, mx >> 6), div3(c, my >> 6));
                        let offx = (mx >> 6).wrapping_mul(4);
                        log.push(format!("s1:m+0x0,{xt},{yt},0,0,0,0,0,{offx}"));
                        let inv1 = a[22 + n1.min(39)];
                        n1 += 1;
                        let mut inv2 = 0;
                        if inv1 != 0 {
                            log.push(format!("s1:p1+0x0,{xt},{yt},0,0,0,0,0,{slot}"));
                            inv2 = a[22 + n1.min(39)];
                            n1 += 1;
                        }
                        if inv1 == 0 || inv2 == 0 {
                            x0 = mx;
                            y0 = my;
                            x1 = mx;
                            y1 = my;
                        } else {
                            let (ax, ay) = (div3(c, mx >> 8), div3(c, my >> 8));
                            let width = rd(&c[2], 0);
                            let mut found = None;
                            'k: for k in 0..0x51usize {
                                let (wx, wy) = (ax + MOVE_X[k], ay + MOVE_Y[k]);
                                log.push("s5:c2+0x0,sp-0x24,sp-0x30".to_string());
                                d5a.push(hexw(wx));
                                d5b.push(hexw(wy));
                                let v = a[64 + n5.min(80)];
                                n5 += 1;
                                if v == 0 {
                                    continue;
                                }
                                let at = (width.wrapping_mul(wy).wrapping_add(wx)).wrapping_mul(28)
                                    as usize
                                    + 4;
                                let region = i16::from_le_bytes([p[5][at], p[5][at + 1]]) as i32;
                                if region != r1 {
                                    continue;
                                }
                                log.push(format!("s6:{reg},{r2},{wx},{wy}"));
                                let ch = a[145 + n6.min(80)];
                                n6 += 1;
                                if ch == 0 {
                                    continue;
                                }
                                let (mut lx, mut ly) = (wx.wrapping_mul(4), wy.wrapping_mul(4));
                                let (mut tvx, mut tvy) = (lx, ly);
                                if (ch.wrapping_sub(3) as u32) < 4 {
                                    lx = lx.wrapping_add(3);
                                    tvx = tvx.wrapping_add(4);
                                }
                                if 4 < ch {
                                    ly = ly.wrapping_add(3);
                                    tvy = tvy.wrapping_add(4);
                                }
                                if ch == 1 || 5 < ch {
                                    tvx = tvx.wrapping_sub(1);
                                }
                                if ch < 4 || ch == 8 {
                                    tvy = tvy.wrapping_sub(1);
                                }
                                let mut ok = false;
                                if ch & 1 == 0 {
                                    let mut sx = ((ch & 2) != 0) as i32;
                                    let mut sy = ((ch & 2) == 0) as i32;
                                    if 5 < ch {
                                        sx = -sx;
                                        sy = -sy;
                                    }
                                    for _ in 0..4 {
                                        log.push(format!("s1:m+0x0,{lx},{ly},0,0,0,0,0,{ly}"));
                                        let i1 = a[22 + n1.min(39)];
                                        n1 += 1;
                                        if i1 == 0 {
                                            log.push(format!(
                                                "s1:p1+0x0,{tvx},{tvy},0,0,0,0,0,{slot}"
                                            ));
                                            let i2 = a[22 + n1.min(39)];
                                            n1 += 1;
                                            if i2 == 0 {
                                                ok = true;
                                                break;
                                            }
                                        }
                                        tvy = tvy.wrapping_add(sy);
                                        lx = lx.wrapping_add(sx);
                                        ly = ly.wrapping_add(sy);
                                        tvx = tvx.wrapping_add(sx);
                                    }
                                } else {
                                    log.push(format!("s1:m+0x0,{lx},{ly},0,0,0,0,0,{ly}"));
                                    let i1 = a[22 + n1.min(39)];
                                    n1 += 1;
                                    if i1 == 0 {
                                        log.push(format!("s1:p1+0x0,{tvx},{tvy},0,0,0,0,0,{slot}"));
                                        let i2 = a[22 + n1.min(39)];
                                        n1 += 1;
                                        ok = i2 == 0;
                                    }
                                }
                                if ok {
                                    found = Some((
                                        lx.wrapping_mul(0xc0).wrapping_add(0x60),
                                        ly.wrapping_mul(0xc0).wrapping_add(0x60),
                                        tvx.wrapping_mul(0xc0).wrapping_add(0x60),
                                        tvy.wrapping_mul(0xc0).wrapping_add(0x60),
                                    ));
                                    break 'k;
                                }
                            }
                            match found {
                                Some((a0, b0, a1_, b1)) => {
                                    x0 = a0;
                                    y0 = b0;
                                    x1 = a1_;
                                    y1 = b1;
                                }
                                None => {
                                    log.push("s4:m+0x0,0".to_string());
                                    break 'done;
                                }
                            }
                        }
                    }
                    // The orders. The unit's first (every arm), then the ship's when the unit is the same player's and the ship has no
                    // action that answers its slot 0x10.
                    let mut arm_c = pl != this_byte;
                    if !arm_c {
                        log.push("s7:p1+0x0".to_string());
                        if a[9] != 0 {
                            log.push("s8:p9+0x0".to_string());
                            if a[10] != 0 {
                                arm_c = true;
                            }
                        }
                    }
                    let (dx, dy) = (x0.wrapping_sub(tx), y0.wrapping_sub(ty));
                    let ang = crate::movement::find_angle(dx, dy).0;
                    let (xt, yt) = (div3(c, x0 >> 4), div3(c, y0 >> 4));
                    let (rx, ry) = (
                        xt.wrapping_mul(48).wrapping_add(0x18),
                        yt.wrapping_mul(48).wrapping_add(0x18),
                    );
                    if arm_c {
                        log.push(format!("s9:{},1", find_angle_ecx(dx, dy)));
                    } else {
                        log.push("s9:c1+0x0,1".to_string());
                    }
                    log.push("s10:p10+0x0".to_string());
                    let (vb1, mut r1) = (p[7].clone(), std::mem::take(&mut p[11]));
                    order_fill(&mut r1, &vb1, rx, ry, ang);
                    p[11] = r1;
                    log.push(format!("s11:m+0xcc,p11+{:#x}", rd(&p[7], 4)));
                    log.push("s12:m+0x0".to_string());
                    let v = rd(&p[4], 0);
                    wr(m, 0xdc, v);
                    log.push("s13:m+0x0".to_string());
                    if arm_c {
                        if pl == this_byte {
                            log.push("s16:p1+0x0".to_string());
                            if a[18] == 9 {
                                log.push("s17:p1+0x0".to_string());
                                log.push("s18:p13+0x0".to_string());
                                if rd(&p[14], 8) == this_short && rd(&p[14], 0xc) == this_byte {
                                    log.push(format!("s19:p1+0x0,{x1},{y1},1,0,0,0,{slot},-1,-1"));
                                }
                            }
                        }
                        break 'done;
                    }
                    // The ship's own order.
                    let (sx, sy) = (rd(&p[1], 0x10) ^ K, rd(&p[1], 0x14) ^ K);
                    let ang2 =
                        crate::movement::find_angle(x1.wrapping_sub(sx), y1.wrapping_sub(sy)).0;
                    let (xt2, yt2) = (div3(c, x1 >> 4), div3(c, y1 >> 4));
                    let (rx2, ry2) = (
                        xt2.wrapping_mul(48).wrapping_add(0x18),
                        yt2.wrapping_mul(48).wrapping_add(0x18),
                    );
                    let f = rd(&p[1], 0x68) & 0xfbffffffu32 as i32;
                    wr(&mut p[1], 0x68, f);
                    wr(&mut p[1], 0xc0, 0);
                    log.push("s14:p1+0x0,0".to_string());
                    log.push("s12:p1+0x0".to_string());
                    log.push("s13:p1+0x0".to_string());
                    log.push(format!("s9:{ecxo},1"));
                    log.push("s10:p10+0x0".to_string());
                    let (vb2, mut r2) = (p[8].clone(), std::mem::take(&mut p[12]));
                    order_fill(&mut r2, &vb2, rx2, ry2, ang2);
                    p[12] = r2;
                    log.push(format!("s11:p1+0xcc,p12+{:#x}", rd(&p[8], 4)));
                    log.push("s13:p1+0x0".to_string());
                    log.push(format!("s15:p1+0x0,{this_short},{this_byte},{ecxo},0"));
                }
                let j = |v: &Vec<String>| {
                    if v.is_empty() {
                        "-".to_string()
                    } else {
                        v.join("/")
                    }
                };
                log.push(j(&d2a));
                log.push(j(&d2b));
                log.push(j(&d5a));
                log.push(j(&d5b));
                eax
            }
        };
        assert_eq!(got, want_eax, "check_meet_ship — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "check_meet_ship — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_meet_ship — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_meet_ship — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "check_meet_ship — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Unit::check_meet_ship@00604550: {n} rows agree");
}
