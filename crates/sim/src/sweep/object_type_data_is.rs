//! `ObjectTypeData::is@0065f7d0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

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

/// `ObjectTypeData::is@0065f7d0`, run by `tools/emu/sweep_object_type_data_is.py` under unicorn
/// (tools/emu): 684 random bytes of object, 2 argument(s), vtable slot +0xf4 (2 args) stubbed; the word at `m+0x2a8` points at 64 random bytes less `0` (placed); the word at `m+0x28c` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_is.py") else {
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
                let x = a[0];
                if rd(m, 4) == x {
                    1
                } else if a[1] != 0 {
                    let n = rd(m, 0x29c);
                    if n == 0 {
                        i64::from(a[2])
                    } else {
                        i64::from((0..n.max(0) as usize).any(|k| {
                            i32::from(u16::from_le_bytes([p[0][2 * k], p[0][2 * k + 1]])) == x
                        }))
                    }
                } else if x < 0 {
                    0
                } else {
                    let n = rd(m, 0x280);
                    if n == 0 {
                        i64::from(a[2])
                    } else {
                        i64::from((0..n.max(0) as usize).any(|k| {
                            i32::from(u16::from_le_bytes([p[1][2 * k], p[1][2 * k + 1]])) == x
                        }))
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "is — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectTypeData::is@0065f7d0: {n} rows agree");
}
