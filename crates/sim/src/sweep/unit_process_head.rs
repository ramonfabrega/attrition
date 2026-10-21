//! `Unit::process@00610bc0` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `Unit::process@00610bc0`, run by `tools/emu/sweep_unit_process_head.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (4 rows × 1 slice(s), row-major), p4..p7 sparse table 1 (4 rows × 1 slice(s), row-major), p8..p11 the place(s), in flag order; the global cell 0x00c061e4 points at 3296 random bytes; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c06188 points at 368 random bytes; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c061f0 points at 3104 random bytes; the global cell 0x00c06214 points at 16 random bytes; callee 0x006e0830 (0 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x005e1a10 (1 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; callee 0x00739ad0 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00909480 (2 args) stubbed; callee 0x008e1410 (7 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc8 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x6e12e0 (0 args) stubbed; vtable slot +0x168 (3 args) stubbed; callee 0x00609a50 (0 args) stubbed; callee 0x005e0670 (0 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `c2+0x164` points at 512 random bytes less `0` (placed); the word at `0xc0a130+i8(m,9)*0x1c` points at 16 random bytes less `i16(m,0x86)*4` (placed); the word at `0xc0a290+i8(m,9)*0x1c` points at 16 random bytes less `i16(m,0x86)*4` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xc8 laid as 0x0046ce70 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 3 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [0, 1] (90%); stub 0 answers -1..4 every row; stub 15 answers 0..3 every row; stub 17 answers 0..60 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 is `__cdecl`: the caller pops its arguments; the call ends where stub 18 is first entered (the function is swept to that callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0xc:0x4 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_process_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `Unit::process@00610bc0` from its entry to the `process_healing` call (the sweep stops
            // at that stub): the four counters, the decoy arm, the caster arm and the air arm. The
            // crate's pieces are scattered over `Sim` (`recover_mana`, `craft_rate`,
            // `process_spells`), so the port is inline over the window bytes. Windows: m the unit;
            // c0 the constants-const (`+0xcd0` decoy time), c1 `div_3_table`, c2 the world (`+0xc`
            // width, `+0x164` the seen3 cells), c3 the game (`+0x550` frame), c4 the constants
            // (`+0xa0` air recharge, `+0xb84` memnon, `+0xc00` heal rates), c5 the graphic pieces;
            // p0..p3 the leaders (`+0xc`), p4..p7 leadersc (`+0x6928`), p8 the type record, p9 the
            // seen3 cells, p10 the heroes' casters, p11 the specials'. `a[k]` is stub k's answer,
            // `a[19..25]` the base flags of slots +0xbc, +0xc4, +0xd4, +0xcc, +0xc8, +0xb8.
            const ENC: i32 = 0x63637;
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            let mut tok: Vec<Vec<String>> = vec![Vec::new(); 19];
            let mut lead: Vec<Vec<Vec<String>>> = vec![vec![Vec::new(); 4]; 19];
            let who = usize::from(m[9]);
            let o = i32::from(i16at(m, 0xa));
            let frame = rd(&c[3], 0x550);
            let ty = rd(&p[8], 0x2b8);
            let this = "m+0x0".to_string();
            // The decrements.
            for off in [0xae, 0xac, 0xad] {
                if m[off] != 0 {
                    m[off] -= 1;
                }
            }
            if u16at(m, 0x38) != 0 {
                let v = u16at(m, 0x38) - 1;
                m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
            }
            // `is(x, y)` through slot +0xb8: the type record's slot +0x60 on the base arm.
            let is_call = |tok: &mut Vec<Vec<String>>, x: i32, y: i32| -> bool {
                if a[24] != 0 {
                    tok[8].push(format!("p8+0x0,{x},{y}"));
                    a[8] != 0
                } else {
                    tok[7].push(format!("m+0x0,{x},{y}"));
                    a[7] != 0
                }
            };
            let masks = rd(m, 0x68);
            let mut finished = false;
            let wr16 =
                |m: &mut [u8], o: usize, v: i16| m[o..o + 2].copy_from_slice(&v.to_le_bytes());
            if masks & 1 != 0 {
                let burn = i16at(m, 0x96).wrapping_add(1);
                wr16(m, 0x96, burn);
                lead[0][who].push("r+0x0".to_string());
                let limit = a[0].wrapping_add(2).wrapping_mul(rd(&c[0], 0xcd0)) / 2;
                if i32::from(burn) >= limit {
                    tok[1].push(format!("{this},0,-1,0"));
                    finished = true;
                } else {
                    let on_map = if a[19] != 0 {
                        u32::from(u16at(m, 0x82) >> 15)
                    } else {
                        tok[2].push(this.clone());
                        a[2] as u32
                    };
                    if on_map != 0 {
                        let x = ((rd(m, 0x10) ^ ENC) >> 7) as usize;
                        let y = ((rd(m, 0x14) ^ ENC) >> 7) as usize;
                        let off = rd(&c[2], 0xc)
                            .wrapping_mul(rd(&c[1], 4 * y))
                            .wrapping_add(rd(&c[1], 4 * x))
                            as usize;
                        let cell = p[9][off];
                        let mask = !i32::from(p[4 + who][1]) as u8;
                        if cell & mask != 0 {
                            tok[3].push(format!("{this},{}", i32::from(frame % 7 == 0)));
                            if a[3] != 0 {
                                finished = true;
                            }
                        }
                    }
                }
            } else if ty & 2 != 0 {
                let caster = |w: &[u8]| i32::from_le_bytes([w[0], w[1], w[2], w[3]]);
                let hero = if a[20] != 0 {
                    ty & 0x20 != 0
                } else {
                    tok[4].push(this.clone());
                    a[4] != 0
                };
                let mut table = None;
                if hero {
                    table = Some(&p[10]);
                } else {
                    let special = if a[21] != 0 {
                        ty & 0x10 != 0
                    } else {
                        tok[5].push(this.clone());
                        a[5] != 0
                    };
                    if special {
                        table = Some(&p[11]);
                    }
                }
                if let Some(t) = table {
                    tok[6].push(format!("{},{o},{who},0", caster(t)));
                }
                if frame.wrapping_add(o) & 31 == 0
                    && (rd(m, 0x68) < 0 || m[0x6c] & 8 != 0)
                    && is_call(&mut tok, 0x4b, 0)
                {
                    let f = rd(m, 0x6c);
                    if f & 8 == 0 {
                        wr(m, 0x68, rd(m, 0x68) & 0x7fff_ffff);
                    } else {
                        wr(m, 0x68, rd(m, 0x68) | i32::MIN);
                        wr(m, 0x6c, f & !8);
                        tok[9].push(format!("c5+0x0,{who},{o}"));
                        let bits = ((rd(m, 0xc) ^ ENC) as f32).to_bits() as i32;
                        tok[10].push(format!("8,{bits},200,1000,0,0,-1,-1"));
                    }
                }
                if i16at(m, 0x96) != 0 {
                    let mut ok = true;
                    let supply = if a[22] != 0 {
                        ty & 0x40 != 0
                    } else {
                        tok[11].push(this.clone());
                        a[11] != 0
                    };
                    if supply {
                        let gov = if a[23] != 0 {
                            rd(&p[8], 0x2b4) & 0x400_0000 != 0
                        } else {
                            tok[12].push(this.clone());
                            a[12] != 0
                        };
                        ok = gov;
                    }
                    if ok && rd(m, 0x68) & 0x2a000 == 0 {
                        let mut r = 2i32;
                        lead[13][who].push("r+0x0,10".to_string());
                        if a[13] != 0 && is_call(&mut tok, 0x36, 1) {
                            r = 4;
                        }
                        lead[14][who].push("r+0x0,769".to_string());
                        if a[14] != 0 && (is_call(&mut tok, 0x36, 1) || is_call(&mut tok, 0x3a, 1))
                        {
                            r = r.wrapping_mul(2);
                        }
                        if is_call(&mut tok, 0x16c, 0) {
                            let v = rd(&c[4], 0xb84).wrapping_mul(r);
                            r = v.wrapping_add((v >> 31) & 0xff) >> 8;
                        }
                        let burn = i32::from(i16at(m, 0x96));
                        let k = (((frame & 1).wrapping_add(r)) / 2).min(burn);
                        wr16(m, 0x96, (burn - k) as i16);
                    }
                }
            } else if rd(&p[8], 0x218) == 2 {
                if i16at(m, 0x82) < 0 {
                    let burn = i16at(m, 0x96);
                    tok[17].push(this.clone());
                    if (a[17] - i32::from(burn)).max(0) != 0 {
                        wr16(m, 0x96, burn.wrapping_add(1));
                    }
                    wr16(m, 0x38, 0);
                } else {
                    let burn = i32::from(i16at(m, 0x96));
                    let rech = rd(&c[4], 0xa0);
                    if burn > rech {
                        wr16(m, 0x96, (burn as i16).wrapping_sub(rech as i16));
                    } else {
                        wr16(m, 0x96, 0);
                    }
                    lead[15][who].push("r+0x0".to_string());
                    let mut rate = rd(&c[4], 0xc00 + 4 * a[15] as usize);
                    if is_call(&mut tok, 0x130, 0) {
                        rate = (rate * 2 + 2) / 3;
                    }
                    if (o + frame) % rate == 0 {
                        tok[16].push(format!("{this},1,1,1"));
                        wr16(m, 0x38, 1);
                    }
                }
            }
            if !finished {
                tok[18].push(this.clone());
            }
            let join = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for k in 0..19 {
                if matches!(k, 0 | 13 | 14 | 15) {
                    for l in lead[k].iter().take(4) {
                        log.push(join(l));
                    }
                    log.push("-".to_string());
                } else {
                    log.push(join(&tok[k]));
                }
            }
            0i64
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::process@00610bc0: {n} rows agree");
}
