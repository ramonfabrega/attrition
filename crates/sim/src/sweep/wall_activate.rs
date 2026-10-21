//! `Wall::activate@0063e4b0` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `Wall::activate@0063e4b0`, run by `tools/emu/sweep_wall_activate.py` under unicorn
/// (tools/emu): 112 random bytes of object, 3 argument(s), `p` windows in order: p0..p11 sparse table 0 (3 rows × 4 slice(s), row-major), p12..p36 the place(s), in flag order, p37 stub 5's answer window; the global cell 0x00c06204 points at 148 random bytes; the global cell 0x00c061f4 points at 20 random bytes; the global cell 0x00c0620c points at 560 random bytes; the global cell 0x00c061fc points at 20 random bytes; the global cell 0x00c0618c points at 112 random bytes; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c061ec points at 1364 random bytes; the global cell 0x00c06208 points at 16 random bytes; vtable slot +0x50 (0 args) stubbed; vtable slot +0x1a4 (1 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x174 (1 args) stubbed; callee 0x00636600 (2 args) stubbed; callee 0x00643270 (0 args) stubbed; callee 0x00662600 (2 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9b70 (10 args) stubbed; callee 0x00643440 (2 args) stubbed; callee 0x00846a30 (5 args) stubbed; the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `c3+0x10` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 544 random bytes less `0` (placed); the word at `p17+4` points at 544 random bytes less `0` (placed); the word at `p17+8` points at 544 random bytes less `0` (placed); the word at `p17+12` points at 544 random bytes less `0` (placed); the word at `c4+0x14` points at 8 random bytes less `0` (placed); the word at `p22+0` points at 8 random bytes less `0` (placed); the word at `p22+4` points at 8 random bytes less `0` (placed); the word at `c4+0x30` points at 8 random bytes less `0` (placed); the word at `p25+0` points at 8 random bytes less `0` (placed); the word at `p25+4` points at 8 random bytes less `0` (placed); the word at `c4+0x4c` points at 8 random bytes less `0` (placed); the word at `p28+0` points at 8 random bytes less `0` (placed); the word at `p28+4` points at 8 random bytes less `0` (placed); the word at `c1+0x10` points at 20 random bytes less `-4` (placed); the word at `p31+4` points at 8 random bytes less `0` (placed); the word at `p31+8` points at 8 random bytes less `0` (placed); the word at `p31+12` points at 8 random bytes less `0` (placed); the word at `p31+16` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 712 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [0, 1] (90%); stub 5 answers the address of 128 random bytes, a `p` window after the places; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 13 writes a value in 0..200 through its argument 0, printed after the answers (`o` in a place); stub 13 writes a value in 0..200 through its argument 1, printed after the answers (`o` in a place); 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0xa10:0x18, +0x5a22:0x8, +0x6928:0x2 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_activate.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // No crate method is `Wall::activate`: `Sim::activate` is `Build::activate` (it calls
            // `crate::activate::activate_head_steps` before its own tail, and its economy bits are the
            // crate's own state, not this function's windows). Ported inline over the windows.
            // Windows: m the building; c0 the options (`+0x90` rebuild), c1 the build types cell (`+0x10`
            // the array), c2 the scene (`+0x22c` recalc_builds), c3 the unit types cell, c4 the objects cell
            // (`+0x14 + 0x1c*who` a leader's object array), c5 the console (`+0x298` who), c6 the game
            // (`+0x550` frame), c7 the message window; p[4*who..4*who+4] leader `who`'s slices (flags;
            // `+0xa10` the unit-class counters; `+0x5a22` per-type counts; `+0x6928` colour and ally
            // mask); p12 the type array and p13..16 its records; p17, p18..21 the unit types; p22.. the
            // objects (three leaders' array and two objects each); p31 the build type array (entry -1 at
            // its first word) and p32..35 its records; p36 the type record; p37 the answer of slot `+0xac`.
            // Stubs: s0 `+0x50`, s1 `+0x1a4`, s2 `+0xb8`, s3 type `+0x60`, s4 `+0x20`, s5 `+0xac`,
            // s6 type-data `+0xc`, s7 `+0x174`, s8 `is_on_screen`, s9 `increment_stats`, s10
            // `play_sound_nonlocal`, s11 `get_neon`, s12 `add_event`, s13 `tile_corner`, s14
            // `remove_cover_doober`. Answers `a[3 + k]`; `a[18]`/`a[19]` the corner; `a[20..23]` the
            // three base flags (`+0x50`, `+0xb8`, `+0xc`; 1 = the base function is laid).
            let ans = |k: usize| a[3 + k];
            let (param1, param2) = (a[0], a[1]);
            let who = m[9] as usize;
            let idx = i32::from(i16::from_le_bytes([m[10], m[11]]));
            let x = rd(m, 0x10) ^ 0x63637;
            let y = rd(m, 0x14) ^ 0x63637;
            let ty = rd(&p[36], 4);
            let lead = 4 * who;
            let this = "m+0x0";
            wr(&mut c[0], 0x90, 1);
            let on_screen_ecx = if ty >= 0 {
                format!("p{}+0x0", 32 + ty)
            } else {
                rd(&p[31], 0).to_string()
            };
            log.push(format!("s8:{on_screen_ecx},{x},{y}"));
            if ans(8) != 0 {
                c[2][0x22c] = 1;
            }
            m[0x63] |= p[lead + 3][1];
            let started = if a[20] == 1 {
                m[8] & 2 != 0
            } else {
                log.push(format!("s0:{this}"));
                ans(0) != 0
            };
            if !started {
                log.push(format!("s1:{this},{param2}"));
            }
            m[8] |= 4;
            let f = u16::from_le_bytes([m[0x60], m[0x61]]) | 0x1000;
            m[0x60..0x62].copy_from_slice(&f.to_le_bytes());
            wr(m, 0x48, 0);
            wr(m, 0x4c, 0);
            let wonder = if a[21] == 1 {
                log.push("s3:p36+0x0,523,0".to_string());
                ans(3) != 0
            } else {
                log.push(format!("s2:{this},523,0"));
                ans(2) != 0
            };
            if wonder {
                wr(m, 0x48, 0x4000_0000);
            }
            let mut count = p[36][0x2c0] & 0x10 != 0;
            if !count {
                log.push(format!("s4:{this}"));
                if ans(4) != 0 {
                    let obj = 22 + 3 * who + 1 + idx as usize;
                    log.push(format!("s5:p{obj}+0x0"));
                    count = i16::from_le_bytes([p[37][0x72], p[37][0x73]]) >= 0;
                }
            }
            if count {
                log.push(format!("s9:{this}"));
            }
            if ty >= 0 {
                let off = 2 * ty as usize;
                let n = u16::from_le_bytes([p[lead + 2][off], p[lead + 2][off + 1]]);
                if n != 0 {
                    p[lead + 2][off..off + 2].copy_from_slice(&(n - 1).to_le_bytes());
                }
                let rec = 13 + ty as usize;
                let unit_type = if a[22] == 1 {
                    (0x32..0x19e).contains(&rd(&p[rec], 4))
                } else {
                    log.push(format!("s6:p{rec}+0x0"));
                    ans(6) != 0
                };
                if unit_type {
                    let ut = &p[18 + ty as usize];
                    let (class, count_class, extra) = (rd(ut, 0x40), rd(ut, 0x1e8), rd(ut, 0x218));
                    if count_class != 0 {
                        // (counter offsets from +0xa10, and whether +0xa1c follows)
                        let (first, also) = match class {
                            0x1ab => (Some(0x0), true),
                            0x1ac => (Some(0x4), true),
                            0x1ae => (Some(0x8), false),
                            0x1b0 => (Some(0x10), false),
                            _ if extra == 2 => (Some(0x14), false),
                            _ => (None, false),
                        };
                        let mut dec = |o: usize| {
                            let v = rd(&p[lead + 1], o);
                            if v != 0 {
                                wr(&mut p[lead + 1], o, v - 1);
                            }
                        };
                        if let Some(o) = first {
                            dec(o);
                        }
                        if also {
                            dec(0xc);
                        }
                    }
                }
            }
            let lf = rd(&p[lead], 0) | 0x2000000 | 0x8000000;
            wr(&mut p[lead], 0, lf);
            log.push(format!("s4:{this}"));
            if ans(4) == 0 {
                log.push(format!("s7:{this},0"));
            }
            if rd(&c[5], 0x298) == who as i32 && rd(&c[6], 0x550) > 0 && param1 == 0 {
                log.push(format!("s10:{this},8,{who}"));
                let colour = i32::from(p[lead + 3][0]);
                // `team_colors.list + 100 * colour` can fall inside the leaders table's slices.
                let ea = 0xe3_9ce0 + colour * 100;
                let mut name = ea.to_string();
                for row in 0..3 {
                    for (k, (off, len)) in [(0, 4), (0xa10, 0x18), (0x5a22, 8), (0x6928, 2)]
                        .into_iter()
                        .enumerate()
                    {
                        let base = 0xe3_a390 + 0x6eec * row + off;
                        if (base..base + len).contains(&ea) {
                            name = format!("p{}+0x{:x}", 4 * row as usize + k, ea - base);
                        }
                    }
                }
                log.push(format!("s11:{name},sp-0x20"));
                log.push(format!("s12:c7+0x0,{x},{y},1,{},1,3072,0,0,0,0", ans(11)));
            }
            log.push(format!("s13:{this},sp-0xc,sp-0x8"));
            let (corner_x, corner_y) = (a[18], a[19]);
            log.push(format!(
                "s14:15400352,{ty},{corner_x},{corner_y},{who},{idx}"
            ));
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "activate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "activate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "activate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "activate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Wall::activate@0063e4b0: {n} rows agree");
}
