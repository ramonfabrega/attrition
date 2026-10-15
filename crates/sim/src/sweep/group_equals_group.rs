//! `Group::equals_group@00708000` under the emulator against the port (item 2731; tools/sweep_gen.py, obj with windows).

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

/// `Group::equals_group@00708000`, run by `tools/emu/sweep_group_equals_group.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 1 argument(s), argument 0 names 2272 random bytes; vtable slot +0x4 (0 args) stubbed; callee 0x00711540 (0 args) run, not hooked; callee 0x007081f0 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 3 answers -5..-1 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_equals_group.py") else {
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
                let _ = &c;
                // normalize (stub 1) is run, the original's own code: it drops the entries whose object is gone
                // (a negative index, or its object's flag bit clear), shifting the rest down, then `find_role`
                // (stub 2, hooked, writes nothing) and `find_leader` (stub 3, hooked, answers < 0), so the
                // speed words end 0. Its `field_0x4 >= 0` arm (the objects' vtable slots) is never drawn.
                let bits: Vec<bool> = (0..4).map(|s| p[2 + s][8] & 1 != 0).collect();
                let norm = |w: &mut [u8]| {
                    let mut n = rd(w, 0xc);
                    let mut i = n - 1;
                    while i >= 0 {
                        let iu = usize::try_from(i).expect("an index");
                        let s = i16::from_le_bytes([w[0x8cc + 2 * iu], w[0x8cd + 2 * iu]]);
                        if s < 0 || !bits[usize::try_from(s).expect("an object") % 4] {
                            if i < n - 1 {
                                let mut j = iu;
                                loop {
                                    for base in [0x4c, 0x24c, 0x44c, 0x64c] {
                                        let v = rd(w, base + 4 * (j + 1));
                                        wr(w, base + 4 * j, v);
                                    }
                                    w[0x84c + j] = w[0x84c + j + 1];
                                    w[0x8cc + 2 * j] = w[0x8cc + 2 * j + 2];
                                    w[0x8cd + 2 * j] = w[0x8cd + 2 * j + 2];
                                    j += 1;
                                    if i32::try_from(j).expect("an index") >= n - 1 {
                                        break;
                                    }
                                }
                            }
                            n -= 1;
                            wr(w, 0xc, n);
                        }
                        i -= 1;
                    }
                    wr(w, 0x40, 0);
                    wr(w, 0x3c, 0);
                };
                let mut calls: Vec<&str> = Vec::new();
                let r = if m[0x4a] != p[0][0x4a] {
                    0
                } else {
                    if rd(m, 4) != -1 {
                        calls.push("m+0x0");
                        norm(&mut *m);
                    }
                    if rd(&p[0], 4) != -1 {
                        calls.push("p0+0x0");
                        norm(&mut p[0]);
                    }
                    let g = &p[0];
                    if a[5] == a[6] && m[0x49] == g[0x49] {
                        let n = usize::try_from(rd(m, 0xc).max(0)).expect("a count");
                        i64::from((0..n).all(|i| {
                            m[0x8cc + 2 * i..0x8ce + 2 * i] == g[0x8cc + 2 * i..0x8ce + 2 * i]
                        }))
                    } else {
                        0
                    }
                };
                log.push(if calls.is_empty() {
                    "-".to_string()
                } else {
                    calls.join("/")
                });
                r
            }
        };
        assert_eq!(got, want_eax, "equals_group — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "equals_group — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "equals_group — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "equals_group — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "equals_group — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Group::equals_group@00708000: {n} rows agree");
}
