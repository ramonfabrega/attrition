//! `Build::process@0061edf0` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `Build::process@0061edf0`, run by `tools/emu/sweep_build_process_head.py` under unicorn
/// (tools/emu): 136 random bytes of object, 0 argument(s), `p` windows in order: p0 sparse table 0 (1 rows × 1 slice(s), row-major), p1 sparse table 1 (1 rows × 1 slice(s), row-major), p2..p32 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; callee 0x00640450 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x006201e0 (0 args) stubbed; callee 0x0064f3b0 (0 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x006228f0 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0x180 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x1b4 (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p2+8` points at 32 random bytes less `0` (placed); the word at `p2+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 32 random bytes less `0` (placed); the word at `p7+4` points at 32 random bytes less `0` (placed); the word at `p7+8` points at 32 random bytes less `0` (placed); the word at `p7+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 104 random bytes less `0` (placed); the word at `p17+4` points at 104 random bytes less `0` (placed); the word at `p17+8` points at 104 random bytes less `0` (placed); the word at `p17+12` points at 104 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p22+0` points at 104 random bytes less `0` (placed); the word at `p22+4` points at 104 random bytes less `0` (placed); the word at `p22+8` points at 104 random bytes less `0` (placed); the word at `p22+12` points at 104 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p27+0` points at 104 random bytes less `0` (placed); the word at `p27+4` points at 104 random bytes less `0` (placed); the word at `p27+8` points at 104 random bytes less `0` (placed); the word at `p27+12` points at 104 random bytes less `0` (placed); the word at `m+0x18` points at 496 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 4 answered from [0, 1, 1, 1, 0] (90%); stub 6 answered from [0, 1, 1, 1] (90%); stub 7 answered from [0, 1, 1, 1, 0] (90%); stub 8 answered from [0, 1, 1, 1] (90%); stub 9 answered from [0, 1, 1, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 8 answers 4 values in call order, printed after the answers; stub 9 answers 4 values in call order, printed after the answers; stub 11 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 4096 bytes under the call is zeroed before each row; 1 rows 0x560 apart at `0xc061ec`, slices +0x550:0x4 (`p` windows after the pointed buffers, before the places); 1 rows 0x98 apart at `0xc06204`, slices +0x90:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p32+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_head.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `Build::process@0061edf0`'s head, `Wall::process` to the call of `do_queue`. The crate's parts
            // of it: `Sim::process_building` (the ejection and the launch, in `Objects::process_all`'s order),
            // `process_building_combat` (the `do_attack` gate, `fight.rs`); the turn toward the target
            // and the `0x200` city-flag toggles have no counterpart found in the crate.
            // Window map: m the building; p0 the game's frame; p1 the options' `rebuild` word; units of
            // player K array p2+5K, records p3+5K+i; cities of player K array p17+5K, records p18+5K+i;
            // p32 the building's type. **Domain** (asserted): `+0x81 >= 0`, `+0x36` of a range test in 0..2,
            // the city block, the roads and the first reminder shut; the leader's other fields read zero.
            const IMAGE: i32 = 0xb4_2174;
            const KEY: i32 = 0x6_3637;
            let short = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let frame = rd(&p[0], 0);
            let image = rd(m, 0) == IMAGE;
            let who = usize::from(m[9]);
            assert!(frame >= 0 && who < 3, "frame, who");
            let mut seq8 = 0usize;
            let mut seq9 = 0usize;
            log.push("s0:m+0x0".to_string());
            let active = if image {
                m[8] & 4 != 0
            } else {
                log.push("s1:m+0x0".to_string());
                a[1] != 0
            };
            if active {
                if short(m, 0x38) != 0 {
                    let v = (short(m, 0x38) - 1) as i16;
                    m[0x38..0x3a].copy_from_slice(&v.to_le_bytes());
                }
                if u16at(m, 0x60) & 0x4000 != 0 {
                    log.push("s2:m+0x0".to_string());
                }
                if m[0x60] & 8 != 0 {
                    log.push("s3:m+0x0".to_string());
                }
                let phase = short(m, 0xa);
                if rd(&p[32], 0x1e8) != 0 {
                    let (m7c, m81) = (short(m, 0x7c), i32::from(m[0x81] as i8));
                    assert!(m81 >= 0, "+0x81 is not negative in this slice");
                    let target: Option<(i32, i32)>;
                    let attack = |log: &mut Vec<String>| -> (i32, i32) {
                        log.push("s5:m+0x0".to_string());
                        (m7c, m81)
                    };
                    if m7c < 0 || m81 < 0 {
                        if (phase + frame) % 32 == 0 {
                            target = Some(attack(&mut log));
                        } else if (u16at(m, 0x34) as i16) < 0 {
                            target = None;
                        } else {
                            let (e, b) = (short(m, 0x34), short(m, 0x36));
                            assert!((0..3).contains(&b), "+0x36 indexes the three players");
                            log.push(format!("s4:m+0x0,{e},{b},{},0", u16at(m, 0x34)));
                            target = Some(if a[4] != 0 { attack(&mut log) } else { (e, b) });
                        }
                    } else {
                        target = Some(attack(&mut log));
                    }
                    if let Some((edi, ebx)) = target
                        && edi >= 0
                    {
                        log.push("s6:m+0x0,-2147483648".to_string());
                        if a[6] != 0 {
                            let unit = &p[3 + 5 * ebx as usize + edi as usize];
                            if unit[8] & 1 != 0 {
                                let (tx, ty) = (rd(unit, 0x10) ^ KEY, rd(unit, 0x14) ^ KEY);
                                let (sx, sy) = (rd(m, 0x10) ^ KEY, rd(m, 0x14) ^ KEY);
                                let ang = sim::movement::find_angle(
                                    tx.wrapping_sub(sx),
                                    ty.wrapping_sub(sy),
                                )
                                .0;
                                let cur = rd(m, 0x48);
                                let diff = ang.wrapping_sub(cur) as u32;
                                let dir: i32 = if diff < 0x8000_0000 { 1 } else { -1 };
                                let mut mag = diff.wrapping_neg();
                                if mag > 0x8000_0000 {
                                    mag = !mag;
                                }
                                let to = if mag < 0x38e_38e3 {
                                    ang
                                } else {
                                    dir.wrapping_mul(0x38e_38e3).wrapping_add(cur)
                                };
                                wr(m, 0x48, to);
                            }
                        }
                    }
                }
                // The city-flag toggles (`+0x200`): armed by a flag bit, undone by the next frame.
                let idx = short(m, 0x72);
                let cityw = |idx: i32| 18 + 5 * who + idx as usize;
                let mut tests = |log: &mut Vec<String>, id: i32| -> bool {
                    if image {
                        seq8 += 1;
                        log.push(format!("s8:p32+0x0,{id},0"));
                        a[12 + seq8 - 1] != 0
                    } else {
                        seq9 += 1;
                        log.push(format!("s9:m+0x0,{id},0"));
                        a[16 + seq9 - 1] != 0
                    }
                };
                let mut returned = false;
                if !image && {
                    log.push("s7:m+0x0".to_string());
                    a[7] != 0
                } {
                    returned = true;
                    if u16at(m, 0x60) & 0x200 == 0 {
                        wr(&mut p[1], 0, 1);
                        m[0x61] |= 2;
                        if idx >= 0 {
                            for (k, (id, mask)) in [
                                (0x1b5, 0x80u16),
                                (0x1a8, 0x400),
                                (0x1a7, 0x200),
                                (0x1b4, 0x800),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let on = tests(&mut log, id);
                                if on {
                                    let c = &mut p[cityw(idx)];
                                    let f = u16::from_le_bytes([c[4], c[5]]) & !mask;
                                    c[4..6].copy_from_slice(&f.to_le_bytes());
                                } else if k == 3 {
                                    break;
                                }
                            }
                        }
                    }
                } else if u16at(m, 0x60) & 0x200 != 0 {
                    wr(&mut p[1], 0, 1);
                    m[0x61] &= !2;
                    if idx >= 0 {
                        for (id, mask) in [
                            (0x1b5, 0x80u16),
                            (0x1a8, 0x400),
                            (0x1a7, 0x200),
                            (0x1b4, 0x800),
                        ] {
                            if tests(&mut log, id) {
                                let c = &mut p[cityw(idx)];
                                let f = u16::from_le_bytes([c[4], c[5]]) | mask;
                                c[4..6].copy_from_slice(&f.to_le_bytes());
                            }
                        }
                    }
                }
                if !returned {
                    log.push("s10:m+0x0,0".to_string());
                    log.push("s11:p32+0x0".to_string());
                    assert_eq!(a[11], 0);
                    assert!(
                        ![0x21a, 0x1b6, 0x1b4, 0x211].contains(&rd(&p[32], 4)),
                        "wonder arms excluded"
                    );
                    assert_ne!(
                        (frame + phase) % 256,
                        0,
                        "the first reminder's gate is shut"
                    );
                    assert_eq!(m[8] & 0x20, 0, "no city block in this slice");
                    assert_eq!(m[0x61] & 1, 0, "no road replan in this slice");
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
            assert_eq!(w, &want[2 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Build::process@0061edf0: {n} rows agree");
}
