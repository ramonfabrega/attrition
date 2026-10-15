//! `ObjectData::is_in_range@0064e4a0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::is_in_range@0064e4a0`, run by `tools/emu/sweep_object_data_is_in_range.py` under unicorn
/// (tools/emu): 64 random bytes of object, 4 argument(s), callee 0x00651a80 (1 args) stubbed; callee 0x0046cff0 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x12c (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 48 random bytes less `0` (placed); the word at `p3+4` points at 48 random bytes less `0` (placed); the word at `p4+0` points at 48 random bytes less `0` (placed); the word at `p4+4` points at 48 random bytes less `0` (placed); the word at `p5+0x18` points at 4 random bytes less `0x244` (placed); the word at `p6+0x18` points at 4 random bytes less `0x244` (placed); the word at `p7+0x18` points at 4 random bytes less `0x244` (placed); the word at `p8+0x18` points at 4 random bytes less `0x244` (placed); stub 0 answered from [-1, 0, 5] (90%); stub 1 answered from [0, 3, 60, 100, 102, 103, 150, 300, 500] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1, 2, 3] (90%); stub 5 answered from [0, 1, 2, 3] (90%); stub 2 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_in_range.py") else {
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
                let ty = &p[2];
                i64::from('blk: {
                    if rd(ty, 0x1e4) & 0x8000000 != 0 {
                        log.push("s0:m+0x0,0".into());
                        if a[4] < 0 {
                            break 'blk true;
                        }
                    }
                    let dx = a[0].wrapping_sub(a[2]).wrapping_abs();
                    log.push("s2:m+0x0".into());
                    log.push(format!("s1:{dx}"));
                    let dist = a[5];
                    log.push("s3:m+0x0".into());
                    let d = if a[7] != 0 {
                        let owner = ((rd(m, 8) >> 8) & 0xff) as usize;
                        let idx = i16::from_le_bytes([m[0xa], m[0xb]]) as usize;
                        let val = rd(&p[9 + 2 * owner + idx], 0);
                        if val < dist { dist - val } else { 0 }
                    } else {
                        log.push("s2:m+0x0".into());
                        if a[11] != 0 {
                            let e = rd(ty, 0x238).wrapping_add(rd(ty, 0x234)).wrapping_mul(0x30);
                            if dist > e { dist.wrapping_sub(e) } else { 0 }
                        } else {
                            dist
                        }
                    };
                    if rd(ty, 0x1fc) == 0 {
                        d <= 0x66
                    } else {
                        log.push("s4:m+0x0".into());
                        if d < a[8].wrapping_mul(0xc0).wrapping_sub(6) {
                            break 'blk false;
                        }
                        log.push("s5:m+0x0".into());
                        d <= a[9].wrapping_mul(0xc0).wrapping_add(6)
                    }
                })
            }
        };
        assert_eq!(got, want_eax, "is_in_range — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_in_range — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_in_range — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_in_range — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_in_range — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::is_in_range@0064e4a0: {n} rows agree");
}
