//! `UnitData::will_be_corner@00609fa0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::will_be_corner@00609fa0`, run by `tools/emu/sweep_unit_data_will_be_corner.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), the word at `0xc0aec0+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); the word at `p0+0` points at 4 random bytes less `0x18` (placed); the word at `p1+0` points at 4 random bytes less `0x248` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_will_be_corner.py") else {
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
                // The unit `units[owner][o]`'s type's `+0x248` against both deltas.
                let w = rd(&p[2], 0);
                let dx = a[0].wrapping_sub(a[2]);
                let dy = a[1].wrapping_sub(a[3]);
                let r = if dx.wrapping_abs() == w && dy.wrapping_abs() == w {
                    if dx < 0 && dy < 0 {
                        1
                    } else if dx > 0 && dy < 0 {
                        3
                    } else if dx > 0 && dy > 0 {
                        5
                    } else if dx < 0 && dy > 0 {
                        7
                    } else {
                        0
                    }
                } else {
                    0
                };
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "will_be_corner — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "will_be_corner — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "will_be_corner — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "will_be_corner — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::will_be_corner@00609fa0: {n} rows agree");
}
