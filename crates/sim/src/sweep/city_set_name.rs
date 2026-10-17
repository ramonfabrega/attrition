//! `City::set_name@00735be0` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `City::set_name@00735be0`, run by `tools/emu/sweep_city_set_name.py` under unicorn
/// (tools/emu): 192 random bytes of object, 2 argument(s), callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1b6b0 (0 args) stubbed; the word at `m+0x90` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0xa4` points at 16 random bytes less `0` (placed, null a third of the rows); stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_set_name.py") else {
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
                let mut ops: Vec<String> = Vec::new();
                let mut hashes: Vec<String> = Vec::new();
                let mut next_hash = 4usize;
                let mut edx = "0".to_string();
                for (k, off) in [0x90usize, 0xa4].into_iter().enumerate() {
                    ops.push(format!("m+0x{off:x},{edx},{}", a[k]));
                    let ptr = rd(m, off);
                    edx = if ptr == 0 {
                        "0".into()
                    } else {
                        format!("p{k}+0x0")
                    };
                    if ptr == 0 {
                        wr(m, off + 0xc, 0);
                        wr(m, off + 0x10, 0);
                    } else if m[off + 0xa] & 2 == 0 {
                        let len = i32::from(u16::from_le_bytes([m[off + 6], m[off + 7]]));
                        edx = format!("m+0x{:x}", off + 0x10);
                        hashes.push(format!(
                            "{},{edx}",
                            if m[off + 0xa] & 1 != 0 {
                                format!("p{k}+0x{:x}", 2 * len)
                            } else {
                                rd(&p[k], 0).wrapping_add(2 * len).to_string()
                            }
                        ));
                        m[off + 0xa] |= 2;
                        wr(m, off + 0xc, a[next_hash]);
                        next_hash += 1;
                    }
                }
                log.push(ops.join("/"));
                log.push(if hashes.is_empty() {
                    "-".into()
                } else {
                    hashes.join("/")
                });
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "set_name — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_name — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_name — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_name — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("City::set_name@00735be0: {n} rows agree");
}
