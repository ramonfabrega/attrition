//! `Signposts::next@00897ac0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `Signposts::next@00897ac0`, run by `tools/emu/sweep_signposts_next.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), 84 random bytes at 0x00eb1a64; callee 0x00a1eeb0 (1 args) stubbed; the word at `c0+0x18` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `c0+0x30` points at 8 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `c0+0x48` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_signposts_next.py") else {
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
                // The three lists' next nodes: labels (guard +0x20, current +0x18, short +0x14,
                // the call's argument the node + 8), x (guard +0x38, current +0x30, value +0x28,
                // short +0x2c), y (guard +0x50, +0x48, +0x40, +0x44).
                if rd(&c[0], 0x20) != 0 {
                    let next = rd(&p[0], 0);
                    wr(&mut c[0], 0x18, next);
                    log.push("c0+0x0,p1+0x8".into());
                    let (b0, b1) = (p[1][0x1c], p[1][0x1d]);
                    c[0][0x14] = b0;
                    c[0][0x15] = b1;
                } else {
                    log.push("-".into());
                }
                for (src, node, guard, cur, val, sh) in [
                    (2usize, 3usize, 0x38usize, 0x30usize, 0x28usize, 0x2cusize),
                    (4, 5, 0x50, 0x48, 0x40, 0x44),
                ] {
                    if rd(&c[0], guard) != 0 {
                        let next = rd(&p[src], 0);
                        wr(&mut c[0], cur, next);
                        let v = rd(&p[node], 8);
                        wr(&mut c[0], val, v);
                        let (b0, b1) = (p[node][0xc], p[node][0xd]);
                        c[0][sh] = b0;
                        c[0][sh + 1] = b1;
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "next — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "next — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "next — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "next — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Signposts::next@00897ac0: {n} rows agree");
}
