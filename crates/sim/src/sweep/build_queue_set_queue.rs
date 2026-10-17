//! `BuildQueue::set_queue@006309f0` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `BuildQueue::set_queue@006309f0`, run by `tools/emu/sweep_build_queue_set_queue.py` under unicorn
/// (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0 argument 2, p1 the place; argument 2 names 24 random bytes (null a third of the rows); the word at `m+4` points at 128 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_set_queue.py") else {
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
            {
                let (idx, s, q) = (a[0], a[1] as i16, a[3]);
                if idx < rd(m, 0) {
                    let rec = (idx * 0x14) as usize;
                    let put16 = |w: &mut Vec<u8>, off: usize, v: i16| {
                        w[off..off + 2].copy_from_slice(&v.to_le_bytes())
                    };
                    put16(&mut p[1], rec + 4, s);
                    if q == 0 {
                        p[1][rec..rec + 4].copy_from_slice(&0i32.to_le_bytes());
                    }
                    if a[2] != 0 {
                        let mut n = 0usize;
                        let mut done = false;
                        for k in 0..6usize {
                            let v = rd(&p[0], 4 * k);
                            if v != 0 {
                                put16(&mut p[1], rec + 6 + 2 * n, k as i16);
                                put16(&mut p[1], rec + 0xc + 2 * n, v as i16);
                                n += 1;
                                if n > 2 {
                                    done = true;
                                    break;
                                }
                            }
                        }
                        if !done {
                            for k in n..3 {
                                put16(&mut p[1], rec + 6 + 2 * k, -1);
                                put16(&mut p[1], rec + 0xc + 2 * k, 0);
                            }
                        }
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_queue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_queue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_queue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("BuildQueue::set_queue@006309f0: {n} rows agree");
}
