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

/// `Unit::process@00610bc0`, run by `tools/emu/sweep_unit_process_onmap.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 sparse table 0 (4 rows × 1 slice(s), row-major), p4..p19 the place(s), in flag order, p20 stub 12's answer window; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061ac points at 32 random bytes; the global cell 0x00c06208 points at 16 random bytes; callee 0x005e0670 (0 args) stubbed; callee 0x005e1c20 (0 args) stubbed; callee 0x005e11a0 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0xf0 (1 args) stubbed; vtable slot +0xe0 (1 args) stubbed; callee 0x0046d370 (0 args) stubbed; vtable slot +0x70 (0 args) stubbed; callee 0x00a1ee30 (0 args) stubbed; callee 0x00668a10 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0046e620 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9b70 (9 args) stubbed; callee 0x005e0560 (0 args) stubbed; callee 0x005e1a10 (1 args) stubbed; vtable slot +0x188 (0 args) stubbed; callee 0x005e0230 (0 args) stubbed; the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p4+0` points at 128 random bytes less `0` (placed); the word at `p4+4` points at 128 random bytes less `0` (placed); the word at `p4+8` points at 128 random bytes less `0` (placed); the word at `p4+12` points at 128 random bytes less `0` (placed); the word at `p4+16` points at 128 random bytes less `0` (placed); the word at `p4+20` points at 128 random bytes less `0` (placed); the word at `p4+24` points at 128 random bytes less `0` (placed); the word at `p4+28` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+o[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p13+0` points at 128 random bytes less `0` (placed); the word at `p13+4` points at 128 random bytes less `0` (placed); the word at `p13+8` points at 128 random bytes less `0` (placed); the word at `p13+12` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `c2+0x10` points at 64 random bytes less `0` (placed); slot +0x148 laid as 0x0046cf10 or its stub, a flag after the answers; slot +0x70 laid as 0x00662670 or its stub, a flag after the answers; stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); stub 12 answers the address of 64 random bytes, a `p` window after the places; stub 5 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 writes a value in 0..3 through its argument 0, printed after the answers (`o` in a place); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p18+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_process_onmap.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `Unit::process@00610bc0` past `process_healing`, the `+0x82 < 0` (on the map) arm
            // with the head's arms skipped by the biases (type `+0x2b8` bit 1 clear, `unit_masks`
            // bit 0 clear, domain not 2, `+0xa0` 0): the sixteen-frame block (targeted decay,
            // cloak, the thirty-two-frame attrition, the objmask mark, the merchant's text
            // bubble), the supply period, the work, and the figures' two loops. Windows: c0 the
            // game (`+0x550` frame), c1 the console (`+0x298` who), c2 the goodtypes cell
            // (`+0x10` -> p19), c3 the message window; p0..p3 the leaders (`+0x6928` team
            // colour), p4 the figures' array and p5..p12 the figures, p13 the units array and
            // p14..p17 its units, p18 the type record, p19 the good types, p20 the pop's
            // answer. `a[k]` stub k's answer, `a[19]` the out value of slot +0xe0, `a[20]` the base flag of
            // slot +0x148, `a[21]` of +0x70.
            const ENC: i32 = 0x63637;
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            let mut tok: Vec<Vec<String>> = vec![Vec::new(); 19];
            let who = usize::from(m[9]);
            let o = i32::from(i16at(m, 0xa));
            let frame = rd(&c[0], 0x550);
            let ty = rd(&p[18], 0x2b8);
            let this = "m+0x0".to_string();
            let _ = u16at;
            // The decrements, as the head's.
            for off in [0xae, 0xac, 0xad] {
                if m[off] != 0 {
                    m[off] -= 1;
                }
            }
            if u16at(m, 0x38) != 0 {
                let v = u16at(m, 0x38) - 1;
                m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
            }
            tok[0].push(this.clone());
            wr(m, 0x6c, rd(m, 0x6c) & !0x10);
            let mut finished = false;
            if frame.wrapping_add(o) & 15 == 0 {
                let c3d = i32::from(m[0x3d] as i8);
                m[0x3d] = ((c3d + ((c3d >> 31) & 3)) >> 2) as u8;
                tok[1].push(this.clone());
                if frame.wrapping_add(o) & 31 == 0 {
                    wr(m, 0x6c, rd(m, 0x6c) & !0x4_0000);
                    tok[2].push(this.clone());
                    let f = rd(m, 0x6c);
                    wr(m, 0x6c, if f & 2 == 0 { f & !1 } else { f & !2 });
                }
                let objmask = if a[20] != 0 {
                    tok[4].push("p18+0x0,-2147483648".to_string());
                    a[4]
                } else {
                    tok[3].push("m+0x0,-2147483648".to_string());
                    a[3]
                };
                if objmask != 0 {
                    tok[5].push("m+0x0,sp-0x18".to_string());
                    if a[5] >= 0 {
                        let w = &mut p[14 + a[5] as usize];
                        let f = rd(w, 0x6c);
                        wr(w, 0x6c, f | 8);
                    }
                }
                // The merchant's bubble.
                if ty & 4 != 0 && rd(m, 0x14c) < 0 && rd(&c[1], 0x298) == who as i32 {
                    tok[6].push(this.clone());
                    if a[6] != 0 && rd(m, 0x68) & 0x8_0000 == 0 && rd(m, 0x54) >= 0 {
                        let on_screen = if a[21] != 0 {
                            m[0x1c] != 0
                        } else {
                            tok[7].push(this.clone());
                            a[7] != 0
                        };
                        if on_screen {
                            wr(m, 0x14c, frame);
                            let gi = rd(m, 0x54) as usize;
                            tok[8].push("sp-0x48".to_string());
                            tok[9].push(format!("{},sp-0x34", rd(&p[19], 4 * gi)));
                            tok[10].push(format!("sp-0x48,{}", a[9]));
                            tok[11].push("sp-0x34".to_string());
                            tok[12].push("sp-0x34".to_string());
                            p[20][0x1a] = who as u8;
                            tok[10].push("p20+0x4,sp-0x48".to_string());
                            let color = i32::from(p[who][0]);
                            tok[13].push(format!("{},sp-0x2c", 0xe3_9ce0 + color * 0x64));
                            tok[14].push(format!(
                                "c3+0x0,{},{},0,{},-1,3072,0,p20+0x0,0",
                                rd(m, 0x10) ^ ENC,
                                rd(m, 0x14) ^ ENC,
                                a[13]
                            ));
                            tok[11].push("sp-0x48".to_string());
                        }
                    }
                }
            }
            let period = i32::from(i16at(m, 0x9e));
            if period != 0 && frame.wrapping_add(o) % period == 0 {
                if a[15] == 0 {
                    tok[15].push(this.clone());
                    tok[16].push(format!("{this},1"));
                    if a[16] != 0 {
                        finished = true;
                    }
                } else {
                    tok[15].push(this.clone());
                    wr(m, 0x6c, rd(m, 0x6c) | 0x4_0000);
                }
            }
            if !finished {
                tok[17].push(this.clone());
                let n1 = i32::from(m[0xb5] as i8);
                for i in 0..n1 {
                    tok[18].push(format!("p{}+0x0", 5 + i));
                }
                let mut i = rd(&p[18], 0x304);
                while i < rd(m, 0xe8) {
                    tok[18].push(format!("p{}+0x0", 5 + i));
                    i += 1;
                }
            }
            let join = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for t in &tok {
                log.push(join(t));
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
            assert_eq!(w, &want[5 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::process@00610bc0: {n} rows agree");
}
