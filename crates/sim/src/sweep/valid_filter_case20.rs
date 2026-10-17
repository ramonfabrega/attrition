//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

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

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_valid_filter_case20.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), vtable slot +0x18 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; callee 0x006099a0 (2 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x00646d50 (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed); the word at `p5+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 112 random bytes less `0` (placed); the word at `p10+4` points at 112 random bytes less `0` (placed); the word at `p10+8` points at 112 random bytes less `0` (placed); the word at `p10+12` points at 112 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 112 random bytes less `0` (placed); the word at `p15+4` points at 112 random bytes less `0` (placed); the word at `p15+8` points at 112 random bytes less `0` (placed); the word at `p15+12` points at 112 random bytes less `0` (placed); the word at `p11+0x18` points at 768 random bytes less `0` (placed); the word at `p12+0x18` points at 768 random bytes less `0` (placed); the word at `p13+0x18` points at 768 random bytes less `0` (placed); the word at `p14+0x18` points at 768 random bytes less `0` (placed); the word at `p16+0x18` points at 768 random bytes less `0` (placed); the word at `p17+0x18` points at 768 random bytes less `0` (placed); the word at `p18+0x18` points at 768 random bytes less `0` (placed); the word at `p19+0x18` points at 768 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 4 answered from [0, 1, -3, 50] (90%); stub 5 answered from [-5, 0, 5, 100] (90%); stub 3 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_valid_filter_case20.py") else {
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
                // Arm 20: slot +0x18 of `objects[p2][p1]`, slot +0x20 of `objects[who][p0]` and `0x6099a0`
                // (stub 2, on the type table's `[p2][p1]`, handed `(p0, who)`) all non-zero; then the sum of
                // the type record's +0x2f0 word (0 when the object has bit 0 of +0x68) and `0x646d50` (stub 4,
                // on the object slot +0x3c answered, the placed p28) against slot +0x178 of that object, signed.
                let who = a[10] as usize;
                let (i0, i1, i2) = (a[0] as usize, a[1] as usize, a[2] as usize);
                let wb = format!("p{}+0x0", 5 * who + 1 + i0);
                let t2 = 11 + 5 * i2 + i1;
                log.push(format!("p{}+0x0", 5 * i2 + 1 + i1));
                let dash = |log: &mut Vec<String>, n: usize| {
                    for _ in 0..n {
                        log.push("-".into());
                    }
                };
                if a[4] == 0 {
                    dash(&mut log, 5);
                    0
                } else {
                    log.push(wb.clone());
                    if a[5] == 0 {
                        dash(&mut log, 4);
                        0
                    } else {
                        log.push(format!("p{t2}+0x0,{i0},{who}"));
                        if a[6] == 0 {
                            dash(&mut log, 3);
                            0
                        } else {
                            log.push(format!("{wb}/{wb}"));
                            log.push("p28+0x0".into());
                            log.push("p28+0x0".into());
                            let base = if rd(&p[t2], 0x68) & 1 != 0 {
                                0
                            } else {
                                rd(&p[20 + 4 * i2 + i1], 0x2f0)
                            };
                            i64::from(base.wrapping_add(a[8]) <= a[9])
                        }
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "valid_filter — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "valid_filter — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_filter — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_filter — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_filter — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Search::valid_filter@0067dbb0: {n} rows agree");
}
