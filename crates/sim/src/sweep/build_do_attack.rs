//! `Build::do_attack@006228f0` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `Build::do_attack@006228f0`, run by `tools/emu/sweep_build_do_attack.py` under unicorn
/// (tools/emu): 144 random bytes of object, 0 argument(s), `p` windows in order: p0..p42 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061ec points at 1376 random bytes; vtable slot +0x148 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x11c (1 args) stubbed; vtable slot +0x114 (0 args) stubbed; vtable slot +0xe0 (1 args) stubbed; vtable slot +0x1b8 (0 args) stubbed; vtable slot +0x164 (0 args) stubbed; vtable slot +0x134 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; callee 0x00622c80 (0 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x00648d70 (4 args) stubbed; callee 0x0064c8b0 (2 args) stubbed; callee 0x0097f340 (3 args) stubbed; the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 128 random bytes less `0` (placed); the word at `p5+4` points at 128 random bytes less `0` (placed); the word at `p5+8` points at 128 random bytes less `0` (placed); the word at `p5+12` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 128 random bytes less `0` (placed); the word at `p10+4` points at 128 random bytes less `0` (placed); the word at `p10+8` points at 128 random bytes less `0` (placed); the word at `p10+12` points at 128 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `p15+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 32 random bytes less `0` (placed); the word at `p20+4` points at 32 random bytes less `0` (placed); the word at `p20+8` points at 32 random bytes less `0` (placed); the word at `p20+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p25+0` points at 32 random bytes less `0` (placed); the word at `p25+4` points at 32 random bytes less `0` (placed); the word at `p25+8` points at 32 random bytes less `0` (placed); the word at `p25+12` points at 32 random bytes less `0` (placed); the word at `m+0x18` points at 512 random bytes less `0` (placed); the word at `p1+0x18` points at 720 random bytes less `0` (placed); the word at `p2+0x18` points at 720 random bytes less `0` (placed); the word at `p3+0x18` points at 720 random bytes less `0` (placed); the word at `p4+0x18` points at 720 random bytes less `0` (placed); the word at `p6+0x18` points at 720 random bytes less `0` (placed); the word at `p7+0x18` points at 720 random bytes less `0` (placed); the word at `p8+0x18` points at 720 random bytes less `0` (placed); the word at `p9+0x18` points at 720 random bytes less `0` (placed); the word at `p11+0x18` points at 720 random bytes less `0` (placed); the word at `p12+0x18` points at 720 random bytes less `0` (placed); the word at `p13+0x18` points at 720 random bytes less `0` (placed); the word at `p14+0x18` points at 720 random bytes less `0` (placed); stub 9 answered from [1, 1, 1, 0] (90%); stub 10 answered from [0, 0, 0, 0, 1] (90%); stub 12 answered from [1, 1, 1, 0] (90%); stub 13 answered from [14, 14, 3, 0, 1, 15] (90%); stub 14 answered from [1, 1, 1, 0] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; stub 3 answers -3..40 every row; stub 4 answers -2..5 every row; stub 5 answers -10..3 every row; stub 6 answers -2..5 every row; stub 8 answers -70000..70000 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 5 writes a value in 0..2 through its argument 0, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_do_attack.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // hand: port
                let ret: i64 = 'done: {
                    // Window map: m the building; c[1] the game (frame at +0x550); p[5w] the
                    // units-table array of player w and p[5w+1+ox] its units; p[30] the building's
                    // type; p[31+4w+ox] unit (w, ox)'s type. Stub answers: a[0] `+0x148`,
                    // a[3] `+0x11c`, a[4] `+0x114`, a[5] `+0xe0` (a[17] its out-parameter), a[6]
                    // `+0x1b8`, a[8] `+0x134`, a[9] `+0x18`, a[10] `+0xd8`, a[12] `valid_target`,
                    // a[13] `action_type`, a[14] `is_in_range`; the `--seq` runs a[18..20] (`+0x148`),
                    // a[20..24] (`+0x60`, the type's `is`), a[24..28] (`+0xb8`, this's `is`),
                    // a[28..30] (`+0x11c`).
                    let vf = rd(m, 0) as u32 == 0xb42174;
                    let seq = |base: usize, len: usize, k: usize| a[base + k.min(len - 1)];
                    let mut n_g = 0usize; // `+0x148` calls so far
                    let mut n_is = 0usize; // `is` calls so far
                    let is = |n_is: &mut usize| -> bool {
                        let v = if vf {
                            seq(20, 4, *n_is)
                        } else {
                            seq(24, 4, *n_is)
                        };
                        *n_is += 1;
                        v != 0
                    };
                    let frame = rd(&c[1], 0x550);
                    let o = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
                    let rech = u16::from_le_bytes([m[0x7a], m[0x7b]]);
                    // The head: a held-fire building (`+0x148` answers non-zero and neither `is`
                    // holds) skips the reload countdown.
                    let g = seq(18, 2, n_g) != 0;
                    n_g += 1;
                    let mut countdown = !g;
                    if g {
                        countdown = is(&mut n_is) || is(&mut n_is);
                    }
                    if countdown && rech != 0 {
                        m[0x7a..0x7c].copy_from_slice(&(rech - 1).to_le_bytes());
                        break 'done 0;
                    }
                    if frame.wrapping_add(o) & 0x1f == 0 && (m[8] as i8) >= 0 {
                        m[0x40] = 0;
                    }
                    m[8] &= 0x7f;
                    let n = if vf {
                        let max = a[28];
                        let d = a[29].wrapping_sub(rd(m, 0x24));
                        if d < 0 || max < 0 { 0 } else { d.min(max) }
                    } else {
                        a[4]
                    };
                    if n == 0 {
                        break 'done 0;
                    }
                    let flags = |m: &[u8]| u16::from_le_bytes([m[0x60], m[0x61]]);
                    let put_flags =
                        |m: &mut [u8], f: u16| m[0x60..0x62].copy_from_slice(&f.to_le_bytes());
                    if a[5] >= 0 {
                        // Jammed: the unit the slot names is marked, the building cries misfire.
                        let (w, ox) = (a[17] as usize, a[5] as usize);
                        let u = &mut p[5 * w + 1 + ox];
                        let v = rd(u, 0x6c) | 8;
                        wr(u, 0x6c, v);
                        log.push(format!(
                            "s16:{},110,{},{}",
                            rd(m, 0x14) ^ 0x63637,
                            rd(m, 0x10) ^ 0x63637,
                            rd(m, 0x14) ^ 0x63637
                        ));
                        put_flags(m, flags(m) | 0x8000);
                        break 'done 0;
                    }
                    put_flags(m, flags(m) & 0x7fff);
                    let arrows = a[6];
                    if arrows == 0 {
                        break 'done 0;
                    }
                    let find = |log: &mut Vec<String>| log.push("s11:m+0x0".to_string());
                    let clear = |m: &mut [u8]| {
                        let f = u16::from_le_bytes([m[0x60], m[0x61]]) & 0xfffb;
                        m[0x60..0x62].copy_from_slice(&f.to_le_bytes());
                        m[0x81] = 0xff;
                        m[0x7c..0x7e].copy_from_slice(&0xffffu16.to_le_bytes());
                    };
                    if flags(m) & 4 == 0 {
                        find(&mut log);
                    }
                    let who = m[0x81] as i8 as i32;
                    let ox = i16::from_le_bytes([m[0x7c], m[0x7d]]) as i32;
                    if ox < 0 || who < 0 {
                        clear(m);
                        break 'done 0;
                    }
                    log.push(format!("s12:m+0x0,{ox},{who},0"));
                    if a[12] == 0 {
                        find(&mut log);
                        break 'done 0;
                    }
                    let (w, x) = (who as usize, ox as usize);
                    let slot = format!("p{}+{:#x}", 5 * w, 4 * x);
                    let ecx: String;
                    if flags(m) & 4 != 0 {
                        ecx = "m+0x0".to_string();
                    } else {
                        let r = frame.wrapping_add(o).wrapping_add(14) & 0x8000001f_u32 as i32;
                        let r = if r < 0 { ((r - 1) | -32) + 1 } else { r };
                        if frame != 0 && r != 0 {
                            ecx = frame.to_string();
                        } else if a[9] == 0 || a[10] != 0 {
                            find(&mut log);
                            break 'done 0;
                        } else {
                            if rd(&p[31 + 4 * w + x], 0x2c8) & 0x10000 != 0 {
                                ecx = slot;
                            } else {
                                log.push(format!("s13:p{}+0x0", 5 * w + 1 + x));
                                if a[13] != 0xe {
                                    find(&mut log);
                                    break 'done 0;
                                }
                                ecx = format!("p{}+0x0", 5 * w + 1 + x);
                            }
                        }
                    }
                    log.push(format!("s14:m+0x0,{ox},{who},{ecx},0"));
                    if a[14] == 0 {
                        clear(m);
                        break 'done 0;
                    }
                    if seq(18, 2, n_g) != 0 {
                        let fire = is(&mut n_is) || is(&mut n_is);
                        if !fire {
                            break 'done 0;
                        }
                    }
                    log.push(format!("s15:m+0x0,{ox},{who}"));
                    m[8] |= 0x80;
                    let bit = 1i32.wrapping_shl(who as u32 & 31);
                    let b40 = m[0x40] as i8 as i32;
                    if bit & b40 == 0 {
                        m[0x40] = (b40 | bit) as u8;
                        // A group the building has not shouted for yet: slot `+0x164`.
                        if m[0x62] as i32 & (bit & 0xff) == 0 {
                            log.push("s7:m+0x0".to_string());
                        }
                    }
                    let num = if vf {
                        rd(&p[30], 0x1f4) as i16 as i32
                    } else {
                        a[8] as i16 as i32
                    };
                    let v = (num / arrows) as i16;
                    m[0x7a..0x7c].copy_from_slice(&v.to_le_bytes());
                    0
                };
                if log.is_empty() {
                    log.push("-".to_string());
                }
                ret
                // end hand
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "do_attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Build::do_attack@006228f0: {n} rows agree");
}
