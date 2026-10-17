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
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1 the place; argument 0 names 24 random bytes; the import cell 0x00ac5640 holds a stub (2 args); the word at `m+0x14` points at 36480 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_tribes_find.py") else {
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
            let u16at = |w: &[u8], o: usize| i32::from(u16::from_le_bytes([w[o], w[o + 1]]));
            let (s, r) = (&p[0], &p[1]);
            let (s_ptr, s_off, s_len, s_flags) = (rd(s, 0), u16at(s, 6), u16at(s, 8), s[10]);
            let mut found = -1i64;
            let mut calls: Vec<String> = Vec::new();
            for k in 0..24usize {
                let b = k * 0x5f0;
                if s_len != u16at(r, b + 0x34) {
                    continue;
                }
                let r_flags = r[b + 0x36];
                let r_ptr = rd(r, b + 0x2c);
                let r_off = u16at(r, b + 0x32);
                let hit = if (r_flags & 2) == 0 || (s_flags & 2) == 0 {
                    // the data pointers compared with the offsets: a null is 0, a live one with bit 0 of the
                    // flags clear is dereferenced — the text's first word (0x401a9d68) for 0x401000 — and
                    // either way plus twice the offset; both live and differing reach `_wcsicmp`
                    let side = |p: i32, off: i32, fl: u8| -> i32 {
                        if fl & 1 != 0 {
                            p.wrapping_add(off * 2)
                        } else if p == 0 {
                            0
                        } else {
                            0x401a_9d68_i32.wrapping_add(off * 2)
                        }
                    };
                    if s_len == 0 || (r_ptr == s_ptr && r_off == s_off) {
                        true
                    } else if r_ptr == 0 || s_ptr == 0 {
                        side(r_ptr, r_off, r_flags) == side(s_ptr, s_off, s_flags)
                    } else {
                        let (x, y) = (side(r_ptr, r_off, r_flags), side(s_ptr, s_off, s_flags));
                        // the import's leftover `ecx` is the last offset loaded: the record's when its flags
                        // bit 0 is clear, else the string's, else the length both share
                        let ecx = if r_flags & 1 == 0 {
                            r_off
                        } else if s_flags & 1 == 0 {
                            s_off
                        } else {
                            s_len
                        };
                        calls.push(format!("{ecx},{x},{y}"));
                        a[1] == 0
                    }
                } else {
                    rd(r, b + 0x3c) == rd(s, 16)
                };
                if hit {
                    found = k as i64;
                    break;
                }
            }
            log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
            found
        };
        assert_eq!(got, want_eax, "find — eax, row `{line}`");
        assert_eq!(log, want_log, "find — the recorded calls, row `{line}`");
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
    assert!(n >= 800, "only {n} rows");
    eprintln!("Tribes::find@006ef2d0: {n} rows agree");
}
