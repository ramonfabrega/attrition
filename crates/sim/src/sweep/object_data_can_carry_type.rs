//! `ObjectData::can_carry@00645e00` under the emulator against the port (item 3104; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::can_carry@00645e00`, run by `tools/emu/sweep_object_data_can_carry_type.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; vtable slot +0xc (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x114 (0 args) stubbed; callee 0x00643d40 (2 args) stubbed; the word at `0xe85ddc` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `0xc0a274` points at 4 random bytes less `a[0]*4` (placed); the word at `p2+0` points at 560 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p6+0x18` points at 736 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 1] (90%); stub 2 answers 3 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_can_carry_type.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the type, 1..=6 the single answers of stubs 0..=5 (slot +0xc, +0x18, +0xb8 unused,
                // +0x60 unused, +0x114, `has_nuke`), 7..=9 the is-slot (own, stub 2) run, 10..=13 the type's
                // slot +0x60 (stub 3) run shared by the based `is` and the unit type's own asks, 14 the
                // slot-+0xb8-based flag, 15 the slot-+0xc-based flag (the range test `0x32..=0x19d` of
                // `p[1]`'s +4 instead of the stub). `p[3]` the unit type, `p[7]` the unit's record.
                let w4 = |w: &[u8], o: usize| {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                };
                let based = a[14] != 0;
                let mut own_turn = 0usize;
                let mut slot_turn = 0usize;
                // The is-slot, asked of `this`: based → the type's slot +0x60 (stub 3), else stub 2.
                let mut ask = |log: &mut Vec<String>, id: i32| -> bool {
                    if based {
                        let k = slot_turn.min(3);
                        slot_turn += 1;
                        log.push(format!("s3:p4+0x0,{id},0"));
                        a[10 + k] != 0
                    } else {
                        let k = own_turn.min(2);
                        own_turn += 1;
                        log.push(format!("s2:m+0x0,{id},0"));
                        a[7 + k] != 0
                    }
                };
                let is_unit = if a[15] != 0 {
                    let idx = w4(&p[1], 4);
                    (0x32..=0x19d).contains(&idx)
                } else {
                    log.push("s0:p1+0x0".into());
                    a[1] != 0
                };
                if !is_unit {
                    0
                } else if ask(&mut log, 0x208) {
                    log.push("s4:p3+0x0".into());
                    i64::from(a[5] != 0 && a[6] == 0)
                } else if !ask(&mut log, 0x1bf) {
                    if !ask(&mut log, 0x15f) {
                        let mut r = 0;
                        if w4(&p[3], 0x218) == 0 {
                            log.push("s1:m+0x0".into());
                            if a[2] != 0 && w4(&p[7], 0x2d4) != 0 {
                                r = 1;
                            }
                        }
                        r
                    } else {
                        // The unit type's own slot +0x60 (stub 3), asked (0x134, 0).
                        let k = slot_turn.min(3);
                        log.push("s3:p3+0x0,308,0".into());
                        i64::from(a[10 + k] != 0)
                    }
                } else if w4(&p[3], 0x218) == 2 && w4(&p[3], 0x1e4) as u32 & 0x8000000 == 0 {
                    let k = slot_turn.min(3);
                    log.push("s3:p3+0x0,308,0".into());
                    i64::from(a[10 + k] == 0)
                } else {
                    0
                }
            }
        };
        if log.is_empty() {
            log.push("-".into());
        }
        assert_eq!(got, want_eax, "can_carry — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "can_carry — the recorded calls, row `{line}`"
        );
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
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectData::can_carry@00645e00: {n} rows agree");
}
