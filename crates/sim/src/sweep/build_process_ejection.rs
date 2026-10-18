//! `Build::process_ejection@006201e0` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

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

/// `Build::process_ejection@006201e0`, run by `tools/emu/sweep_build_process_ejection.py` under unicorn
/// (tools/emu): 112 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; callee 0x00617c10 (1 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `0xc0aef8` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 160 random bytes less `0` (placed); the word at `p0+4` points at 160 random bytes less `0` (placed); the word at `p0+8` points at 160 random bytes less `0` (placed); the word at `p1+0` points at 160 random bytes less `0` (placed); the word at `p1+4` points at 160 random bytes less `0` (placed); the word at `p1+8` points at 160 random bytes less `0` (placed); the word at `p2+0` points at 160 random bytes less `0` (placed); the word at `p2+4` points at 160 random bytes less `0` (placed); the word at `p2+8` points at 160 random bytes less `0` (placed); slot +0xe4 laid as 0x00610ab0 or its stub, a flag after the answers; stub 1 answers 0..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; figures p3..p5 link at +0x8e (2 bytes) above their own index or to -1; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_process_ejection.py") else {
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
                let rd16 = |w: &[u8], o: usize| i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32;
                // a[1] the slot +0xe4 stub's answer, a[2] the slot laid as `get_captain` itself; windows: p0..p2 a player's
                // unit array, p3 + 3 * player + index its units.
                let pl = m[0x3e] as i8 as usize;
                let at = rd16(m, 0x28);
                if at < 0 {
                    m[0x61] &= 0xbf;
                } else {
                    let u = 3 + 3 * pl + at as usize;
                    fn captain(p: &[Vec<u8>], u: usize) -> i32 {
                        let rd16 = |w: &[u8], o: usize| i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32;
                        let up = rd16(&p[u], 0x8e);
                        if up < 0 {
                            rd16(&p[u], 0xa)
                        } else {
                            captain(p, 3 + 3 * p[u][9] as usize + up as usize)
                        }
                    }
                    let who = if a[2] != 0 {
                        let up = rd16(&p[u], 0x8e);
                        if up < 0 {
                            rd16(&p[u], 0xa)
                        } else {
                            captain(p, 3 + 3 * p[u][9] as usize + up as usize)
                        }
                    } else {
                        a[1]
                    };
                    log.push(format!("p{}+0x0,1", 3 + 3 * pl + who as usize));
                    if m[8] & 1 == 0 {
                        let v = (rd16(m, 0x32) as u16).wrapping_add(1);
                        m[0x32..0x34].copy_from_slice(&v.to_le_bytes());
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        if log.is_empty() {
            log.push("-".to_string());
        }
        assert_eq!(log, want_log, "process_ejection — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_ejection — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_ejection — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_ejection — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Build::process_ejection@006201e0: {n} rows agree");
}
