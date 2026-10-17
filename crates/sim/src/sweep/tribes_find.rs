//! `Tribes::find@006ef2d0` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

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

/// `Tribes::find@006ef2d0`, run by `tools/emu/sweep_tribes_find.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1 the place; argument 0 names 24 random bytes; the word at `m+0x14` points at 36480 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tribes_find.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let u16at = |w: &[u8], o: usize| i32::from(u16::from_le_bytes([w[o], w[o + 1]]));
            let (s, r) = (&p[0], &p[1]);
            let (s_ptr, s_off, s_len, s_flags) = (rd(s, 0), u16at(s, 6), u16at(s, 8), s[10]);
            let mut found = -1i64;
            for k in 0..24usize {
                let b = k * 0x5f0;
                if s_len != u16at(r, b + 0x34) {
                    continue;
                }
                let r_flags = r[b + 0x36];
                let r_ptr = rd(r, b + 0x2c);
                let r_off = u16at(r, b + 0x32);
                let hit = if (r_flags & 2) == 0 || (s_flags & 2) == 0 {
                    if s_len == 0 {
                        true
                    } else if r_ptr == s_ptr && r_off == s_off {
                        true
                    } else if r_ptr == 0 || s_ptr == 0 {
                        // the pointers themselves compared: with bit 0 of the flags set the data
                        // pointer plus twice the offset, otherwise a null is 0 and a live one is
                        // dereferenced — the text's first word (0x401a9d68) for 0x401000
                        let side = |p: i32, off: i32, fl: u8| -> i32 {
                            if fl & 1 != 0 {
                                p.wrapping_add(off * 2)
                            } else if p == 0 {
                                0
                            } else {
                                0x401a_9d68_i32.wrapping_add(off * 2)
                            }
                        };
                        side(r_ptr, r_off, r_flags) == side(s_ptr, s_off, s_flags)
                    } else {
                        unreachable!(
                            "the _wcsicmp arm: the draw keeps non-null data pointers equal"
                        )
                    }
                } else {
                    rd(r, b + 0x3c) == rd(s, 16)
                };
                if hit {
                    found = k as i64;
                    break;
                }
            }
            found
        };
        assert_eq!(got, want_eax, "find — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Tribes::find@006ef2d0: {n} rows agree");
}
