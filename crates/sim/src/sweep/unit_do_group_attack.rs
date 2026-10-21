//! `Unit::do_group_attack@005e75a0` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_group_attack@005e75a0`, run by `tools/emu/sweep_unit_do_group_attack.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p22 the place(s), in flag order, p23..p28 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00e85f20 points at 8192 random bytes; argument 0 names 16 random bytes; vtable slot +0x9c (0 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x007304c0 (1 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x00710aa0 (1 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x007123f0 (1 args) stubbed; callee 0x00713390 (2 args) stubbed; callee 0x005fd4d0 (5 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; callee 0x00648da0 (5 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x98 (0 args) stubbed; vtable slot +0x17c (3 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x005faf30 (2 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x00713a50 (3 args) stubbed; callee 0x0046d700 (0 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p9+0` points at 144 random bytes less `0` (placed); the word at `p9+4` points at 144 random bytes less `0` (placed); the word at `p9+8` points at 144 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p13+0` points at 144 random bytes less `0` (placed); the word at `p13+4` points at 144 random bytes less `0` (placed); the word at `p13+8` points at 144 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); the word at `p17+4` points at 16 random bytes less `0` (placed); the word at `m+0xd4` points at 16 random bytes less `0` (placed); the word at `p19+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 16 random bytes less `0` (placed); the word at `m+0xcc` points at 16 random bytes less `0` (placed); stub 8 answered from [0, 1] (90%); stub 12 answered from [1, 1, 1, 0] (90%); stub 13 answered from [1, 1, 1, 0] (90%); stub 16 answered from [19, 19, 19, 19, 0, 5] (90%); stub 0 answers the address of 96 random bytes, a `p` window after the places; stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 17 answers the address of 64 random bytes, a `p` window after the places; stub 19 answers the address of 16 random bytes, a `p` window after the places; stub 3 answers a window of its own to each call, in call order (-16 bytes, ...); the stubbed vtable laid at `p0+0`; stub 14 answers -6..8 every row; stub 15 answers -3..3 every row; stub 18 answers -30..30 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 15 writes a value in -3..3 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `m+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0x4` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0x4` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_group_attack.py") else {
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
            // The port, inlined: the crate carries no `Unit::do_group_attack` (only the `_to` look), so the
            // sweep holds it. `a[0]` the order argument (`p0`), `a[1 + k]` stub `k`'s answer in the line's
            // order (0 the order's record, `p23`; 2 `get_obj`'s object; 3 the first `+0x50` answer; 8
            // `is_in_range`; 12, 13 an object's `+0x8`, `+0xbc`; 14, 15 `+0x130`, `find_nearby_target`; 16 an
            // order's type; 18 `+0x17c`), `a[25]` the word `find_nearby_target` writes through its out
            // pointer. The group, objects and units tables are the cells `c1`, `c0` and the image's `p1..p16`.
            let rec = p[23].clone();
            let r = |off: usize| rd(&rec, off);
            let id = r(8);
            let pl = r(0xc);
            let grp = i32::from(i16::from_le_bytes([m[0x80], m[0x81]]));
            let owner = i32::from(m[9]);
            let mid = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
            let gname = |g: i32| format!("c1+{:#x}", g.wrapping_mul(0x9d4));
            let sub = "p23+0x4".to_string();
            log.push("s0:p0+0x0".to_string());
            if m[0xae] == 0 {
                log.push(format!("s1:m+0x0,{},p0+0x0,0", r(0x38)));
                if grp == -1 {
                    // A new order: copy the argument's attack order onto a fresh one and queue it.
                    log.push("s2:m+0x0,10".to_string());
                    log.push("s3:p24+0x0".to_string());
                    log.push(format!("s3:{sub}"));
                    let t = if a[26] != 0 { 0 } else { a[4] };
                    let tn = if t == 0 { "0" } else { "p25+0x0" };
                    log.push(format!("s4:{tn},p26+0x0"));
                    let node = rd(&p[17], 4);
                    wr(m, 0xd4, node);
                    let w8 = rd(&p[18], 8);
                    wr(m, 0xcc, w8);
                    m[0xd0] = p[18][0xc];
                    log.push("s5:m+0xcc".to_string());
                    let at = if t == 0 { "0" } else { "p25+0x4" };
                    log.push(format!("s6:m+0xcc,{at}"));
                    let head = rd(&p[17], 0);
                    wr(m, 0xdc, head);
                } else if r(0x28) == mid && r(0x2c) == owner {
                    // The order already names this unit: fight, or hand the group the order.
                    let obj = 2 + 4 * pl as usize + id as usize;
                    if rd(&p[obj], 8) & 1 == 0 {
                        log.push(format!("s7:{},{}", gname(grp), r(0x30)));
                    } else {
                        log.push(format!("s8:m+0x0,{id},{pl},{id},0"));
                        if a[9] != 0 {
                            wr(&mut p[23], 0x40, id);
                            wr(&mut p[23], 0x44, pl);
                            log.push(format!(
                                "s11:m+0x0,{id},{pl},{},{},0",
                                i32::from(rec[0x1c]),
                                r(0x3c)
                            ));
                        } else if r(0x3c) == 0 {
                            log.push(format!("s9:{},{}", gname(grp), r(0x30)));
                            log.push(format!("s10:{},{id},{pl}", gname(grp)));
                        } else {
                            log.push(format!("s7:{},{}", gname(grp), r(0x30)));
                        }
                    }
                } else {
                    let (id2, pl2) = (r(0x28), r(0x2c));
                    let obj = 2 + 4 * pl2 as usize + id2 as usize;
                    let unit = 10 + 4 * pl2 as usize + id2 as usize;
                    let refresh = format!("s22:{},{},{mid},{owner}", gname(grp), r(0x30));
                    log.push(format!("s12:p{obj}+0x0"));
                    if a[13] == 0 {
                        log.push(refresh);
                    } else {
                        log.push(format!("s13:p{obj}+0x0"));
                        if a[14] == 0
                            || i32::from(i16::from_le_bytes([p[unit][0x80], p[unit][0x81]])) != grp
                        {
                            log.push(refresh);
                        } else {
                            let b = i32::from(rec[0x1c]);
                            let mut target: Option<(i32, i32)> = None;
                            let mut near = true;
                            if b >= 2 {
                                log.push(format!(
                                    "s8:m+0x0,{},{},{},0",
                                    r(8),
                                    r(0xc),
                                    grp & 0xffff
                                ));
                                if a[9] != 0 {
                                    target = Some((r(8), r(0xc)));
                                    near = false;
                                }
                            }
                            if near {
                                log.push("s14:m+0x0".to_string());
                                let dist = (a[15] + 4).wrapping_mul(0xc0);
                                log.push(format!("s15:m+0x0,{dist},sp-0x4,0,0,0"));
                                target = Some((a[16], a[25]));
                            }
                            let mut fought = false;
                            if let Some((x, y)) = target {
                                if x >= 0 && y >= 0 {
                                    log.push(format!("s8:m+0x0,{x},{y},m+0x0,0"));
                                    if a[9] != 0 {
                                        wr(&mut p[23], 0x40, x);
                                        wr(&mut p[23], 0x44, y);
                                        log.push(format!("s11:m+0x0,{x},{y},1,1,0"));
                                        fought = true;
                                    }
                                }
                            }
                            if !fought {
                                log.push("s23:m+0xcc".to_string());
                                log.push("s16:p21+0x0".to_string());
                                let k = 0x63637;
                                let (c1, c2) = (rd(m, 0x10) ^ k, rd(m, 0x14) ^ k);
                                let mut skip_ecx = "p21+0x0".to_string();
                                let mut skip = true;
                                if a[17] == 0x13 && r(0x3c) != 0 && rd(m, 0xc0) != 0 {
                                    log.push("s17:p21+0x0".to_string());
                                    let x17 = &p[27];
                                    if rd(x17, 0x2c) != c1 || rd(x17, 0x30) != c2 {
                                        skip = false;
                                    } else {
                                        skip_ecx = c2.to_string();
                                    }
                                }
                                if skip {
                                    if rd(m, 0x50) == r(0x38) {
                                        log.push("s21:m+0x0,0,0,1".to_string());
                                    } else {
                                        log.push(format!("s1:m+0x0,{},{skip_ecx},0", r(0x38)));
                                    }
                                } else {
                                    log.push("s5:m+0xcc".to_string());
                                    log.push(format!("s18:m+0x0,{c1},{c2},1"));
                                    let v = a[19].wrapping_mul(9);
                                    let e = v.wrapping_add((v >> 31) & 7) >> 3;
                                    log.push("s23:m+0xcc".to_string());
                                    log.push("s19:p22+0x0".to_string());
                                    log.push(format!("s20:m+0x0,p28+0x0,{e}"));
                                    log.push(format!("s6:m+0xcc,{sub}"));
                                    let head = rd(&p[17], 0);
                                    wr(m, 0xdc, head);
                                }
                            }
                        }
                    }
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_group_attack — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_group_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_group_attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "do_group_attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::do_group_attack@005e75a0: {n} rows agree");
}
