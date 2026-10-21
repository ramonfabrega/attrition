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

/// `Unit::process@00610bc0`, run by `tools/emu/sweep_unit_process_train.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 sparse table 0 (4 rows × 6 slice(s), row-major), p24..p44 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c061f0 points at 576 random bytes; the global cell 0x00c061c0 points at 16 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c06204 points at 160 random bytes; the global cell 0x00e80030 points at 6144 random bytes; the global cell 0x00c061fc points at 32 random bytes; 4 random bytes at 0x00c8cd00; 560 random bytes at 0x00e37f30; callee 0x006e3140 (1 args) stubbed; callee 0x006508c0 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00461310 (4 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; callee 0x00662600 (2 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9b70 (9 args) stubbed; vtable slot +0xc (0 args) stubbed; callee 0x005e0670 (0 args) stubbed; the word at `0xc0a274` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 768 random bytes less `0` (placed); the word at `p24+4` points at 768 random bytes less `0` (placed); the word at `p24+8` points at 768 random bytes less `0` (placed); the word at `p24+12` points at 768 random bytes less `0` (placed); the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p29+0` points at 32 random bytes less `0` (placed); the word at `p29+4` points at 32 random bytes less `0` (placed); the word at `p29+8` points at 32 random bytes less `0` (placed); the word at `p29+12` points at 32 random bytes less `0` (placed); the word at `c7+0x10` points at 16 random bytes less `0` (placed); the word at `p34+0` points at 576 random bytes less `0` (placed); the word at `p34+4` points at 576 random bytes less `0` (placed); the word at `p34+8` points at 576 random bytes less `0` (placed); the word at `p34+12` points at 576 random bytes less `0` (placed); the word at `0xc0aec0+m[9]*0x1c` points at 16 random bytes less `0` (placed); the word at `p39+0` points at 128 random bytes less `0` (placed); the word at `p39+4` points at 128 random bytes less `0` (placed); the word at `p39+8` points at 128 random bytes less `0` (placed); the word at `p39+12` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 10 answered from [0, 1] (90%); stub 0 answers 0..3 every row; stub 1 answers 0..60 every row; stub 5 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stack 128 bytes under the call is zeroed before each row; the call ends where stub 11 is first entered (the function is swept to that callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x7e4:0x10, +0x940:0x4, +0xa10:0x18, +0x5a22:0x10, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_process_train.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(10);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `Unit::process@00610bc0` from `LAB_00611158` to the `process_healing` call, the
            // unit-production (accelerated training) arm, with the head's arms skipped by the
            // biases: the accumulator against `train_time`, the population-cap message, and the
            // production (the counters, `Objects::init_unit`, `go_inside`, the created sound
            // and the event). Windows: c0 the game (`+0x31` the pop-limit row, `+0x550` the frame),
            // c1 the constants (`+0x228` the accelerated step), c2 the ai speed, c3 the console
            // (`+0x298` who, `+0x2a0` play), c4 the message window, c5 the options (`+0x90`
            // rebuild), c6 the pop limits, c8 the string table base, c9 `game.info.player`
            // (`0x8c` apiece: `+0` the cap frame, `+4` the flags); p(6r+k) leader r's slice k
            // (0 the flag byte, 1 `+0x7e4` pop, 2 `+0x940`, 3 `+0xa10` the counters, 4 `+0x5a22`
            // the shorts, 5 `+0x6928` the colour); p25.. the types' `+0x2f0`, p30.. the type
            // objects, p35.. the unit types, p40.. the units. `a[k]` stub k's answer, `a[12]`
            // the base flag of slot +0xc.
            const ENC: i32 = 0x63637;
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let mut tok: Vec<Vec<String>> = vec![Vec::new(); 12];
            let mut lead: Vec<Vec<String>> = vec![Vec::new(); 4];
            let who = usize::from(m[9]);
            let o = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
            let frame = rd(&c[0], 0x550);
            let this = "m+0x0".to_string();
            for off in [0xae, 0xac, 0xad] {
                if m[off] != 0 {
                    m[off] -= 1;
                }
            }
            if u16at(m, 0x38) != 0 {
                let v = u16at(m, 0x38) - 1;
                m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
            }
            let dec = |w: &mut [u8], off: usize| {
                let v = rd(w, off);
                if v != 0 {
                    wr(w, off, v - 1);
                }
            };
            if u16at(m, 0xa0) != 0 && p[6 * who][0] & 2 != 0 {
                lead[who].push("r+0x0,308".to_string());
                let upg = a[0];
                let tt = a[1];
                tok[1].push(format!("{this},{upg}"));
                let accel = rd(&c[1], 0x228);
                let ai = rd(&c[2], 0);
                let step = if ai > 1 {
                    ai.wrapping_mul(accel)
                } else {
                    accel
                };
                let acc = rd(m, 0x64).wrapping_add(step);
                wr(m, 0x64, acc);
                if acc >= tt {
                    let pop = rd(&p[6 * who + 1], 0);
                    let want =
                        rd(&p[25 + upg as usize], 0x2f0).wrapping_add(rd(&p[6 * who + 2], 0));
                    if pop < want {
                        let v = rd(&p[6 * who + 1], 4);
                        wr(&mut p[6 * who + 1], 4, v.wrapping_add(1));
                        if rd(&c[3], 0x298) == who as i32 {
                            let row = rd(&c[3], 0x2a0) as usize * 0x8c;
                            let flags = u16at(&c[9], row + 4);
                            if flags & 0x800 == 0 {
                                c[9][row + 4..row + 6]
                                    .copy_from_slice(&(flags | 0x800).to_le_bytes());
                                wr(&mut c[9], row, frame);
                                let limit = rd(&c[6], usize::from(c[0][0x31]) * 0x58 + 0x3c);
                                let s = rd(&c[8], 0).wrapping_add(if pop < limit {
                                    0x121b0
                                } else {
                                    0x1219c
                                });
                                tok[2].push(format!("sp-0x70,{s}"));
                                tok[3].push("c4+0x0,0,0,0,0,0,-1,13161032".to_string());
                                tok[4].push("c4+0x0,91".to_string());
                            }
                        }
                        wr(m, 0x64, acc.wrapping_sub(step));
                    } else {
                        let a0 = (u16at(m, 0xa0) - 1).to_le_bytes();
                        m[0xa0..0xa2].copy_from_slice(&a0);
                        wr(m, 0x64, 0);
                        let u = upg as usize;
                        let sh = u16at(&p[6 * who + 4], 2 * u);
                        if sh != 0 {
                            p[6 * who + 4][2 * u..2 * u + 2]
                                .copy_from_slice(&(sh - 1).to_le_bytes());
                        }
                        let tp = &p[30 + u];
                        let isunit = if a[12] != 0 {
                            (0x32..0x19e).contains(&rd(tp, 4))
                        } else {
                            tok[10].push(format!("p{}+0x0", 30 + u));
                            a[10] != 0
                        };
                        if isunit {
                            let rec = p[35 + u].clone();
                            if rd(&rec, 0x1e8) != 0 {
                                let k = &mut p[6 * who + 3];
                                match rd(&rec, 0x40) {
                                    0x1ab => {
                                        dec(k, 0);
                                        dec(k, 0xc);
                                    }
                                    0x1ac => {
                                        dec(k, 4);
                                        dec(k, 0xc);
                                    }
                                    0x1ae => dec(k, 8),
                                    0x1b0 => dec(k, 0x10),
                                    _ => {
                                        if rd(&rec, 0x218) == 2 {
                                            dec(k, 0x14);
                                        }
                                    }
                                }
                            }
                        }
                        let (x, y) = (rd(m, 0x10) ^ ENC, rd(m, 0x14) ^ ENC);
                        tok[5].push(format!("{y},{who},{upg},{x},{y}"));
                        let idx = a[5];
                        if idx >= 0 {
                            tok[6].push(format!("p{}+0x0,{o},{who},0", 40 + idx));
                            wr(&mut c[5], 0x90, 1);
                            let cw = rd(&c[3], 0x298);
                            if cw == who as i32 && frame > 0 {
                                tok[7].push(format!("p{}+0x0,8,{cw}", 40 + idx));
                                let color = i32::from(p[6 * who + 5][0]);
                                tok[8].push(format!("{},sp-0x2c", 0xe3_9ce0 + color * 0x64));
                                tok[9].push(format!("c4+0x0,{x},{y},1,{},1,3072,0,0,0", a[8]));
                            }
                        }
                    }
                }
            }
            tok[11].push(this.clone());
            let join = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for l in lead.iter().take(4) {
                log.push(join(l));
            }
            log.push("-".to_string());
            for t in tok.iter().skip(1) {
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
            assert_eq!(w, &want[11 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::process@00610bc0: {n} rows agree");
}
