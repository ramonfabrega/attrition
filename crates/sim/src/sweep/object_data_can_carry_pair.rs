//! `ObjectData::can_carry@006483c0` under the emulator against the port (item 3104; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::can_carry@006483c0`, run by `tools/emu/sweep_object_data_can_carry_pair.py` under unicorn
/// (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p10 the place(s), in flag order; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00609dc0 (1 args) stubbed; callee 0x0064dcd0 (0 args) stubbed; callee 0x00645330 (1 args) stubbed; callee 0x006454a0 (0 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 576 random bytes less `0` (placed); the word at `p3+0x18` points at 576 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 736 random bytes less `0` (placed); the word at `p8+0x18` points at 736 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1, 2, 3] (90%); stub 5 answered from [0, 1, 2] (90%); stub 6 answered from [0, 1, 2] (90%); stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 3 writes a value in [0, 1] through its argument 0, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_can_carry_pair.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the other object's index, 1 its player, 2..=8 the single answers of stubs 0..=6
                // (`home_base` 3, `cargo_inside` 4, `num_aircraft_here` 5, `num_aircraft_limit` 6), 9 what
                // `home_base` writes through its pointer, 10..=11 slot +0x18's run (the object, then `this`),
                // 12..=17 the own is-slot's run, 18..=23 the types' slot +0x60's run, 24 the based flag. Windows: p0 `this`'s type, p2/p3 the objects, p4/p5 their types, p7/p8 the units,
                // p9/p10 their records.
                let w4 = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
                let short = i16::from_le_bytes([m[10], m[11]]);
                let player = m[9] as i32;
                if (a[0] == i32::from(short) && a[1] == player) || a[1] != player {
                    0
                } else {
                    let idx = a[0] as usize;
                    let based = a[24] != 0;
                    let (mut t0, mut t1, mut t2) = (0usize, 0usize, 0usize);
                    let obj_w = format!("p{}", 2 + idx);
                    let ty_w = format!("p{}", 4 + idx);
                    let mut ask = |log: &mut Vec<String>, obj: bool, id: i32| -> bool {
                        let flag = 0;
                        if based {
                            let k = t2.min(5);
                            t2 += 1;
                            log.push(format!("s2:{}+0x0,{id},{flag}", if obj { &ty_w[..] } else { "p0" }));
                            a[18 + k] != 0
                        } else {
                            let k = t1.min(5);
                            t1 += 1;
                            log.push(format!("s1:{}+0x0,{id},{flag}", if obj { &obj_w[..] } else { "m" }));
                            a[12 + k] != 0
                        }
                    };
                    let mut s0 = |log: &mut Vec<String>, who: &str| -> bool {
                        let k = t0.min(1);
                        t0 += 1;
                        log.push(format!("s0:{who}+0x0"));
                        a[10 + k] != 0
                    };
                    let dom = w4(&p[4 + idx], 0x218);
                    let bit = w4(&p[4 + idx], 0x1e4) as u32 & 0x8000000;
                    let unit_of = |i: usize| format!("p{}", 7 + i);
                    if dom != 2 {
                        if dom == 1 || !s0(&mut log, &obj_w) || !s0(&mut log, "m") {
                            0
                        } else if ask(&mut log, false, 0x15f) {
                            0
                        } else {
                            log.push(format!("s4:{}+0x0", unit_of(short as usize)));
                            let cap_this = w4(&p[9 + short as usize], 0x2d4);
                            let cap_obj = w4(&p[9 + idx], 0x2d8);
                            i64::from(a[6].wrapping_add(cap_obj) < cap_this)
                        }
                    } else {
                        let mut pass = true;
                        if bit == 0 {
                            if ask(&mut log, true, 0x136) {
                                if !ask(&mut log, false, 0x1bf) {
                                    pass = false;
                                }
                            } else if ask(&mut log, false, 0x208) {
                                pass = false;
                            }
                        } else if ask(&mut log, false, 0x1bf) || ask(&mut log, false, 0x15f) {
                            pass = false;
                        }
                        if !pass {
                            0
                        } else {
                            let o = ask(&mut log, true, 0x134);
                            let t = ask(&mut log, false, 0x15f);
                            if (!o && t) || (o && !t) {
                                0
                            } else {
                                log.push(format!("s3:{}+0x0,sp-0x8", unit_of(idx)));
                                if a[5] == i32::from(short) && a[9] == player {
                                    1
                                } else {
                                    log.push("s5:m+0x0,0".into());
                                    log.push("s6:m+0x0".into());
                                    i64::from(a[7] < a[8])
                                }
                            }
                        }
                    }
                }
            }
        };
        if log.is_empty() {
            log.push("-".into());
        }
        assert_eq!(got, want_eax, "can_carry — eax, row `{line}`");
        assert_eq!(log, want_log, "can_carry — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_carry — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "can_carry — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "can_carry — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("ObjectData::can_carry@006483c0: {n} rows agree");
}
