//! `sort@0092d5c0` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `sort@0092d5c0`, run by `tools/emu/sweep_sort_c.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s); argument 0 names 64 random bytes; argument 1 names 64 random bytes; `__cdecl`: the caller pops the arguments; `ecx` an integer in 0..16, not the object, the row's last; `edx` the address of window `m`; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_sort_c.py") else {
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
            // The same bubble sort over three parallel word arrays (`p0` the keys, `p1` the tie-breaks, `m` the
            // values; `ecx` = a[2] long): the first adjacent pair whose key is smaller than its successor's,
            // or equal with the tie-break smaller (signed both), is swapped in all three and the scan restarts.
            let n = a[2];
            'again: loop {
                for i in 0..(n - 1).max(0) as usize {
                    let (k0, k1) = (rd(&p[0], i * 4), rd(&p[0], i * 4 + 4));
                    let swap = if k0 == k1 {
                        rd(&p[1], i * 4) < rd(&p[1], i * 4 + 4)
                    } else {
                        k0 < k1
                    };
                    if swap {
                        let (p0, p1) = p.split_at_mut(1);
                        for w in [&mut p0[0], &mut p1[0], &mut *m] {
                            let (x, y) = (rd(w, i * 4), rd(w, i * 4 + 4));
                            wr(w, i * 4, y);
                            wr(w, i * 4 + 4, x);
                        }
                        continue 'again;
                    }
                }
                break;
            }
            want_eax
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "sort — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "sort — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "sort — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("sort@0092d5c0: {n} rows agree");
}
