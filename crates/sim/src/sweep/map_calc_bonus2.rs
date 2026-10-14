//! `Map::calc_bonus2@00692e70` under the emulator against the port (item 2122; tools/sweep_gen.py, obj with windows).

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

/// `Map::calc_bonus2@00692e70`, run by `tools/emu/sweep_map_calc_bonus2.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061b8 points at 32 random bytes; the word at `c0+0x134` points at 28 random bytes less `(i32(c[0],0)*a[1]+a[0])*0x1c` (placed); the word at `c1+0x10` points at 8 random bytes less `i16(p[0],4)*0x88` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_calc_bonus2.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let r = i16::from_le_bytes([p[0][4], p[0][5]]);
                let f = if r == 0 { rd(m, 0x20) } else { rd(&p[1], 4) };
                if f == 0 {
                    0
                } else {
                    let (x, y, s) = (a[0], a[1], rd(&c[0], 0x7c));
                    i64::from(
                        (x % f).wrapping_mul(f).wrapping_add(y % f)
                            == (x / f)
                                .wrapping_mul(0xd)
                                .wrapping_add((y / f).wrapping_mul(0xb))
                                .wrapping_add(s)
                                .wrapping_rem(f.wrapping_mul(f)),
                    )
                }
            }
        };
        assert_eq!(got, want_eax, "calc_bonus2 — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "calc_bonus2 — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "calc_bonus2 — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "calc_bonus2 — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Map::calc_bonus2@00692e70: {n} rows agree");
}
