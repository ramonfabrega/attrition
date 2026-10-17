//! `Map::land_dist@0069d970` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `Map::land_dist@0069d970`, run by `tools/emu/sweep_map_land_dist.py` under unicorn
/// (tools/emu): 8 random bytes of object, 3 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; 260 random bytes at 0x00cbe330; 1024 random bytes at 0x00cb7e90; 1024 random bytes at 0x00cbb0e0; the word at `c0+0x134` points at 4096 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_land_dist.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            'r: {
                let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
                let (x, y) = (a[0], a[1]);
                let rad = |i: i32| rd(&c[1], 4 * i as usize);
                let flags = |o: usize| u16::from_le_bytes([p[0][o], p[0][o + 1]]);
                let land = |o: usize| matches!(p[0][o + 2], 1 | 2);
                let s = ((ww * y + x) * 0x1c) as usize;
                if flags(s) & 0x100 != 0 || !land(s) {
                    break 'r 0;
                }
                for r in 1..=0x40 {
                    for i in rad(r - 1)..rad(r) {
                        let tx = c[2][i as usize] as i8 as i32 + x;
                        let ty = c[3][i as usize] as i8 as i32 + y;
                        if tx < 0 || ty < 0 || ww <= tx || wh <= ty {
                            if a[2] != 0 {
                                break 'r i64::from(r);
                            }
                        } else {
                            let o = ((ww * ty + tx) * 0x1c) as usize;
                            if flags(o) & 0x100 != 0 || !land(o) {
                                break 'r i64::from(r);
                            }
                        }
                    }
                }
                0x41
            }
        };
        assert_eq!(got, want_eax, "land_dist — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "land_dist — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "land_dist — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "land_dist — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Map::land_dist@0069d970: {n} rows agree");
}
