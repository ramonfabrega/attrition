//! `Guy::turn_angles@005d98c0` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

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

/// `Guy::turn_angles@005d98c0`, run by `tools/emu/sweep_guy_turn_angles.py` under unicorn
/// (tools/emu): 32 random bytes of object, 4 argument(s), argument 1 names 4 random bytes; callee 0x005de340 (1 args) stubbed; stub 0 answered from [0, 1, 16777216, 35791392, 67108864, 268435456, 1073741824, 2147483647, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_turn_angles.py") else {
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
                let cur = rd(m, 0x18) as u32;
                let d = (a[0] as u32).wrapping_sub(cur);
                let e = if d > 0x8000_0000 { !d } else { d };
                if e < 0x222_2220 {
                    wr(&mut p[0], 0, a[0]);
                    0
                } else {
                    let mut s = a[4] as u32;
                    if a[3] != 0 {
                        s >>= 1;
                    }
                    if e <= s {
                        wr(&mut p[0], 0, a[0]);
                        0
                    } else {
                        let to = if d <= 0x8000_0000 {
                            cur.wrapping_add(s)
                        } else {
                            cur.wrapping_sub(s)
                        };
                        wr(&mut p[0], 0, to as i32);
                        i64::from((e - s) as i32)
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "turn_angles — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "turn_angles — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "turn_angles — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "turn_angles — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Guy::turn_angles@005d98c0: {n} rows agree");
}
