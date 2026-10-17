//! `Cliffs::set_edge_flags@008a85e0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `Cliffs::set_edge_flags@008a85e0`, run by `tools/emu/sweep_cliffs_set_edge_flags.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), the global cell 0x00c06188 points at 320 random bytes; callee 0x006b4e50 (5 args) stubbed; the word at `m+0x50` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p1+0x2c` points at 48 random bytes less `0` (placed); the word at `p2+0x2c` points at 48 random bytes less `0` (placed); the word at `p3+0x2c` points at 48 random bytes less `0` (placed); the word at `c0+0x138` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 0 answers 9 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_cliffs_set_edge_flags.py") else {
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
            {
                // set_edge_flags: every point of every cliff that the world says has gather access (has_gather_access(&x, &y, -1, 0, 1)
                // on the world record) gets 0x8000 or'd into its tile's flag short (the array behind world+0x138, width at +0x18).
                let _ = &a[0];
                let hex = |v: i32| -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                };
                let (mut calls, mut xs, mut ys): (Vec<String>, Vec<String>, Vec<String>) =
                    (vec![], vec![], vec![]);
                for i in 0..rd(m, 0x44).max(0) as usize {
                    for j in 0..rd(&p[1 + i], 0x20).max(0) as usize {
                        let (x, y) = (rd(&p[4 + i], 8 * j), rd(&p[4 + i], 8 * j + 4));
                        calls.push("c0+0x0,sp-0x14,sp-0x10,-1,0,1".to_string());
                        xs.push(hex(x));
                        ys.push(hex(y));
                        if a[1 + calls.len().min(9) - 1] != 0 {
                            let w = rd(&c[0], 0x18);
                            let at = (w.wrapping_mul(y).wrapping_add(x) as usize) * 2;
                            let v = u16::from_le_bytes([p[7][at], p[7][at + 1]]) | 0x8000;
                            p[7][at..at + 2].copy_from_slice(&v.to_le_bytes());
                        }
                    }
                }
                if calls.is_empty() {
                    log.extend(["-".to_string(), "-".to_string(), "-".to_string()]);
                } else {
                    log.extend([calls.join("/"), xs.join("/"), ys.join("/")]);
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_edge_flags — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_edge_flags — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_edge_flags — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "set_edge_flags — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Cliffs::set_edge_flags@008a85e0: {n} rows agree");
}
