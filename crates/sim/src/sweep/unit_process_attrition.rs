//! `Unit::process_attrition@005e11a0` under the emulator against the port (item 3400; tools/sweep_gen.py, obj with windows).

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

/// `Unit::process_attrition@005e11a0`, run by `tools/emu/sweep_unit_process_attrition.py` under unicorn
/// (tools/emu): 168 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 sparse table 0 (4 rows × 6 slice(s), row-major), p24..p31 sparse table 1 (4 rows × 2 slice(s), row-major), p32..p35 sparse table 2 (4 rows × 1 slice(s), row-major), p36..p41 the place(s), in flag order, p42..p43 the stubs' answer window(s); the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c061f0 points at 4096 random bytes; the global cell 0x00c06210 points at 768 random bytes; 4 random bytes at 0x00c06208; 4 random bytes at 0x00c8cd00; callee 0x0046cff0 (0 args) run, not hooked; callee 0x0046fa10 (0 args) stubbed; callee 0x006089b0 (1 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x0046d370 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; callee 0x006e1160 (1 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x00594670 (0 args) stubbed; callee 0x006da000 (0 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00592b00 (1 args) stubbed; callee 0x006e1250 (3 args) stubbed; callee 0x00608fd0 (1 args) stubbed; the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); the word at `p25+0` points at 64 random bytes less `0` (placed); the word at `p27+0` points at 64 random bytes less `0` (placed); the word at `p29+0` points at 64 random bytes less `0` (placed); the word at `p31+0` points at 64 random bytes less `0` (placed); slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 4 answered from [0, 0, 0, 0, 1] (90%); stub 5 answered from [0, 0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 0, 1] (90%); stub 7 answered from [0, 0, 0, 0, 1] (90%); stub 8 answered from [0, 0, 0, 0, 1] (90%); stub 9 answered from [0, 0, 0, 0, 1] (90%); stub 10 answered from [0, 0, 0, 0, 1] (90%); stub 11 answered from [0, 0, 0, 0, 1] (90%); stub 13 answered from [0, 0, 0, 0, 1] (90%); stub 14 answered from [0, 1, 1] (90%); stub 15 answered from [0, 1] (90%); stub 22 answered from [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 0, 128, 256, 1000, 3000] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; stub 12 answers the address of 64 random bytes, a `p` window after the places; stub 6 answers 2 values in call order, printed after the answers; stub 16 answers -1..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x8:0x4, +0x74:0x44, +0x1f4:0x4, +0x2b0:0x40, +0x7f8:0xc (`p` windows after the pointed buffers, before the places); 4 rows 0x18 apart at `@0xed64b0`, slices +0x4:0x4, +0x10:0x4 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 30000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_process_attrition.py") else {
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
        let log: Vec<String>;
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The port is inline: the crate's `attrition::process` is the model of this function's damage decision
                // (a NAMESAKE: it takes a pre-digested `Situation`, has no free points, messages or diplomacy stamps), not
                // this function. Windows: `m` the unit; `c[0]` the world (width +0, cells' array at +0x134), `c[1]` the
                // `div_3_table`, `c[2]` the game (`team_style` +0x24, `rush_rules` +0x32, `frame` +0x550, semaphore +0x822),
                // `c[3]` the constants, `c[4]` the console (`who` +0x298), `c[5]` the message window's word, `c[6]` the
                // string table's base. `p[6r..6r+6]` leader `r`'s slices (flags +0; target +8; +0x74 treaties and +0x94
                // contact flags; +0x1f4 warning stamp; +0x2b0 and +0x2d0 alliance-break stamps; +0x7f8 / +0x7fc / +0x800),
                // `p[24+2r]` its free-point count, `p[32+r]` the leaders' constant table's row, `p[36]` the type, `p[37]` the
                // world's cells, `p[38+r]` leader `r`'s free points, `p[42]`/`p[43]` the action and its slot's answer.
                // Answers `a`: the single answers of stubs (1 is_worker, 2 is_gathering, 4 is_merchant, 5 hero,
                // 7 special, 8 `is`, 9 caravan, 10 type's `+0x60`, 11 type's `+0x130`, 13 conquest, 14 is_enemy, 15
                // war_allowed, 16 get_target, 22 get_attrition), `a[23]`/`a[24]` slot +0xcc's two answers, `a[25..30]` the
                // base flags of slots +0xc4, +0xcc, +0xd4, +0xb8, +0xd0.
                const ENC: i32 = 0x63637;
                let mut calls: Vec<Vec<String>> = vec![Vec::new(); 14];
                let leader = m[9] as usize;
                let flags = rd(m, 0x68);
                wr(m, 0x68, flags & 0xffbf_ff7fu32 as i32);
                m[0x9e] = 0;
                m[0x9f] = 0;
                let (x, y) = (rd(m, 0x10) ^ ENC, rd(m, 0x14) ^ ENC);
                let d3 = |k: i32| rd(&c[1], 4 * (k >> 8) as usize);
                let t6 = |k: i32| rd(&c[1], 4 * (k >> 6) as usize);
                let cell = (d3(y).wrapping_mul(rd(&c[0], 0)).wrapping_add(d3(x)) as usize) * 28;
                let who = i32::from(p[37][cell + 0xf] as i8);
                let row = format!("p{}+0x0", 6 * leader);
                let (tx, ty) = (t6(x), t6(y));
                let count = rd(&p[24 + 2 * leader], 0);
                let mut free = false;
                if count > 0 {
                    for k in 0..count as usize {
                        let rec = &p[38 + leader];
                        let (rx, ry, rr) = (rd(rec, 16 * k + 4), rd(rec, 16 * k + 8), rd(rec, 16 * k + 12));
                        let d = crate::world::vector_dist((tx - rx).abs(), (ty - ry).abs());
                        if d <= rr {
                            free = true;
                            break;
                        }
                    }
                }
                let ctype = |p: &[Vec<u8>], o: usize| rd(&p[36], o);
                let mut seq_taken = 0usize;
                let ret: i64 = 'f: {
                    if free {
                        break 'f 0;
                    }
                    if who < 0 {
                        if rd(&p[6 * leader + 5], 8) == 0 {
                            break 'f 0;
                        }
                        if ctype(p, 0x218) != 1 {
                            let v = rd(&p[6 * leader + 5], 8) as i16;
                            m[0x9e..0xa0].copy_from_slice(&v.to_le_bytes());
                        }
                        // The leaders table before row 0 is the image's zeroes: no contact flags.
                        break 'f 0;
                    }
                    let w = who as usize;
                    if w == leader {
                        break 'f 0;
                    }
                    let wf = rd(&p[6 * w], 0);
                    if wf & 1 == 0 || wf & 2 == 0 {
                        break 'f 0;
                    }
                    let target = rd(&p[6 * leader + 1], 0);
                    if who == target {
                        break 'f 0;
                    }
                    if rd(&p[6 * leader + 2], 4 * w) == 2 && rd(&p[32 + w], 4 * target as usize) == 2 {
                        break 'f 0;
                    }
                    if rd(&p[6 * leader + 5], 4) != 0 || rd(&p[6 * w + 5], 0) != 0 {
                        break 'f 0;
                    }
                    let base_flag = |i: usize| a[25 + i] != 0;
                    let next_cc = |seq_taken: &mut usize| {
                        let v = a[23 + (*seq_taken).min(1)];
                        *seq_taken += 1;
                        v != 0
                    };
                    calls[0].push("m+0x0".into());
                    if a[1] != 0 {
                        calls[1].push("m+0x0,m+0x0".into());
                        if a[2] != 0 {
                            calls[2].push("m+0x0".into());
                            if p[43][0x25] != 0 {
                                break 'f 0;
                            }
                        }
                    } else {
                        calls[3].push("m+0x0".into());
                        let mut skip = a[4] != 0;
                        if !skip {
                            skip = if base_flag(0) { ctype(p, 0x2b8) & 0x20 != 0 } else { a[5] != 0 };
                        }
                        if !skip {
                            skip = if base_flag(1) { ctype(p, 0x2b8) & 0x40 != 0 } else { next_cc(&mut seq_taken) };
                        }
                        if !skip {
                            if ctype(p, 0x1e8) == 0 {
                                break 'f 0;
                            }
                            if if base_flag(2) { ctype(p, 0x2b8) & 0x10 != 0 } else { a[7] != 0 } {
                                break 'f 0;
                            }
                            if if base_flag(3) { a[10] != 0 } else { a[8] != 0 } {
                                break 'f 0;
                            }
                            if if base_flag(4) { a[11] != 0 } else { a[9] != 0 } {
                                break 'f 0;
                            }
                        }
                    }
                    if ctype(p, 0x218) == 1 {
                        break 'f 0;
                    }
                    if rd(&p[6 * leader + 5], 8) != 0 {
                        break 'f 0;
                    }
                    let sem = c[2][0x822] & 2 != 0;
                    let team = c[2][0x24];
                    if sem && team != 2 {
                        calls[4].push(format!("{row},9"));
                        if a[13] != 0 {
                            break 'f 0;
                        }
                    }
                    calls[5].push(format!("{row},{who}"));
                    let enemy = a[14] != 0;
                    let mut peace = !enemy;
                    if enemy && sem {
                        calls[6].push("c2+0x0".into());
                        peace = a[15] == 0;
                    }
                    let frame = rd(&c[2], 0x550);
                    let (delay, grace) = (rd(&c[3], 0xcf8), rd(&c[3], 0xcfc));
                    let msgbase = rd(&c[6], 0);
                    let msg_ecx = rd(&c[5], 0);
                    // One warning: the text, the window's line, the sound.
                    let say = |calls: &mut Vec<Vec<String>>, text: i32, shown: i32| {
                        calls[8].push(format!("15558448,{}", msgbase.wrapping_add(text)));
                        calls[9].push(format!(
                            "{msg_ecx},15558448,{x},{y},{shown},13161068,-1,{y},0,0,{y}"
                        ));
                        calls[10].push(format!("{msg_ecx},10"));
                    };
                    let meet = |calls: &mut Vec<Vec<String>>, p: &[Vec<u8>]| {
                        if rd(&p[6 * leader + 2], 0x20 + 4 * w) & 1 == 0 {
                            calls[12].push(format!("{row},{who},{x},{y}"));
                        }
                    };
                    if peace {
                        let recent = |t: i32| t != 0 && frame.wrapping_sub(t) < delay;
                        let l2b = rd(&p[6 * leader + 4], 4 * w);
                        let w2b = rd(&p[6 * w + 4], 4 * leader);
                        let l2d = rd(&p[6 * leader + 4], 0x20 + 4 * w);
                        let w2d = rd(&p[6 * w + 4], 0x20 + 4 * leader);
                        let (mut b4, mut b2, mut b5) = (false, false, false);
                        let diff = if recent(l2b) {
                            Some(frame.wrapping_sub(l2b))
                        } else if recent(w2b) {
                            Some(frame.wrapping_sub(w2b))
                        } else {
                            None
                        };
                        if let Some(d) = diff {
                            b4 = true;
                            b2 = d < grace;
                        }
                        if !b2 && (recent(l2d) || recent(w2d)) {
                            b5 = true;
                            b2 = true;
                        }
                        let stamp = rd(&p[6 * leader + 3], 0);
                        if stamp == 0 || frame.wrapping_sub(stamp) > 900 {
                            let mut b3 = false;
                            if rd(&c[4], 0x298) as usize == leader {
                                if c[2][0x32] != 0 {
                                    calls[6].push("c2+0x0".into());
                                    if a[15] == 0 {
                                        b3 = true;
                                        if stamp == 0 {
                                            calls[11].push("c2+0x0,c2+0x0".into());
                                            calls[9].push(format!(
                                                "{msg_ecx},15558448,{x},{y},1,13161068,-1,{y},0,0,{y}"
                                            ));
                                        }
                                    }
                                }
                                let text = if b2 {
                                    if b4 { 74000 } else { 0x12124 }
                                } else if b4 {
                                    0x12138
                                } else if b5 {
                                    0x1214c
                                } else if b3 {
                                    0x12160
                                } else {
                                    0x12174
                                };
                                say(&mut calls, text, 1);
                            }
                            let v = if b4 && b2 { (grace.wrapping_add(frame).wrapping_sub(900)).max(1) } else { frame };
                            wr(&mut p[6 * leader + 3], 0, v);
                        }
                        meet(&mut calls, p);
                        wr(m, 0x68, rd(m, 0x68) | 0x40_0080);
                        if b2 {
                            break 'f 0;
                        }
                        let v = rd(&c[3], 0xd34) as i16;
                        let v = if ctype(p, 0x218) == 2 { v / 2 } else { v };
                        m[0x9e..0xa0].copy_from_slice(&v.to_le_bytes());
                    } else if team == 2 {
                        calls[7].push(row.clone());
                        if a[16] != who {
                            let stamp = rd(&p[6 * leader + 3], 0);
                            if stamp == 0 || frame.wrapping_sub(stamp) > 900 {
                                if rd(&c[4], 0x298) as usize == leader {
                                    say(&mut calls, 0x12188, 1);
                                }
                                wr(&mut p[6 * leader + 3], 0, frame);
                            }
                            meet(&mut calls, p);
                            wr(m, 0x68, rd(m, 0x68) | 0x40_0080);
                            let v = rd(&c[3], 0xd38) as i16;
                            let v = if ctype(p, 0x218) == 2 { v / 2 } else { v };
                            m[0x9e..0xa0].copy_from_slice(&v.to_le_bytes());
                        }
                    }
                    // The tail: land units only.
                    if ctype(p, 0x218) != 0 {
                        break 'f 0;
                    }
                    if rd(m, 0x68) & 0x40_0000 == 0
                        && (if base_flag(1) { ctype(p, 0x2b8) & 0x40 != 0 } else { next_cc(&mut seq_taken) })
                    {
                        break 'f 0;
                    }
                    calls[13].push(format!("m+0x0,{who}"));
                    let att = a[22];
                    if att != 0 {
                        let v = rd(&c[3], 0xd3c).wrapping_mul(att);
                        let v7 = v.wrapping_add((v >> 31) & 0xff) >> 8;
                        let n = v7.max(1);
                        let cur = i16::from_le_bytes([m[0x9e], m[0x9f]]);
                        if cur == 0 || n < i32::from(cur) {
                            m[0x9e..0xa0].copy_from_slice(&(n as i16).to_le_bytes());
                        }
                        wr(m, 0x68, rd(m, 0x68) | 0x80);
                        meet(&mut calls, p);
                    }
                    0
                };
                log = calls
                    .iter()
                    .map(|c| if c.is_empty() { "-".to_string() } else { c.join("/") })
                    .collect();
                ret
            }

        };
        assert_eq!(got, want_eax, "process_attrition — eax, row `{line}`");
        assert_eq!(log, want_log, "process_attrition — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_attrition — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_attrition — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "process_attrition — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 30000, "only {n} rows");
    eprintln!("Unit::process_attrition@005e11a0: {n} rows agree");
}
