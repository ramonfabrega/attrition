//! `ObjectData::can_carry@00646c40` under the emulator against the port (item 3104; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::can_carry@00646c40`, run by `tools/emu/sweep_object_data_can_carry_domain.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0..p10 the place(s), in flag order; vtable slot +0x18 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 736 random bytes less `0` (placed); the word at `p3+0x18` points at 736 random bytes less `0` (placed); the word at `p5+0x18` points at 736 random bytes less `0` (placed); the word at `p6+0x18` points at 736 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_can_carry_domain.py") else {
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
                // `a`: 0 the domain (ground 0, air 2), 1..=3 the single answers (only slot +0x18's, `a[1]`,
                // is used), 4..=6 and 7..=9 the runs of the is-slot (own slot, then the type's slot +0x60),
                // 10 the based flag. The is-slot is asked in call order; the unit's record is
                // `p[7 + 2 * player + index]`, its word +0x2d4 the passenger capacity.
                let based = a[10] != 0;
                let mut turn = 0usize;
                let mut ask = |log: &mut Vec<String>, id: i32, flag: i32| -> bool {
                    let k = turn.min(2);
                    turn += 1;
                    if based {
                        log.push(format!("s2:p0+0x0,{id},{flag}"));
                        a[7 + k] != 0
                    } else {
                        log.push(format!("s1:m+0x0,{id},{flag}"));
                        a[4 + k] != 0
                    }
                };
                let player = m[9] as usize;
                let index = i16::from_le_bytes([m[10], m[11]]) as usize;
                let cap = {
                    let w = &p[7 + 2 * player + index];
                    i32::from_le_bytes(w[0x2d4..0x2d8].try_into().expect("four bytes"))
                };
                if a[0] != 2 {
                    let mut r = 0;
                    if a[0] == 0 {
                        log.push("s0:m+0x0".into());
                        if a[1] != 0 && cap != 0 && !ask(&mut log, 0x15f, 0) {
                            r = 1;
                        }
                    }
                    r
                } else if ask(&mut log, 0x1bf, 1)
                    || ask(&mut log, 0x15f, 1)
                    || ask(&mut log, 0x208, 1)
                {
                    1
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
    eprintln!("ObjectData::can_carry@00646c40: {n} rows agree");
}
