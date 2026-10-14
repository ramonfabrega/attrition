//! `BuildTypeData::get_town@00639b90` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::get_town@00639b90`, run by `tools/emu/sweep_build_type_data_get_town.py` under unicorn
/// (tools/emu): 708 random bytes of object, 3 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061d4 points at 256 random bytes; callee 0x00658190 (3 args) stubbed; callee 0x0065b870 (5 args) stubbed; the word at `c1+0x138` points at 288 random bytes less `0` (placed); the word at `c2+0x10+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 1 answers -1..3 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_town.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                use crate::world::{Pos, tile};
                // A footprint `+0x234` by `+0x238` centred on tile `(a[0], a[1])`
                // (the port's `tile_corner_of`), every tile of it on the map
                // inside a city's radius (`p[0]` the masks, `c[1]` the world);
                // then `find_town_at` (`a[3]`) or `find_city_at` (`a[4]`) by the
                // flags `+0x2c0`, and the city's `+8` through `p[1]`'s row.
                let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
                let (w, h) = (rd(&c[1], 0x18), rd(&c[1], 0x1c));
                let centre = Pos::new(a[0] * 192 + 96, a[1] * 192 + 96);
                let corner = crate::place::tile_corner_of(xs, ys, centre);
                let mut clear = true;
                for x in corner.x..corner.x + xs {
                    for y in corner.y..corner.y + ys {
                        if x >= 0 && y >= 0 && x < w && y < h {
                            let t = ((w * y + x) * 2) as usize;
                            clear &=
                                u16::from_le_bytes([p[0][t], p[0][t + 1]]) & tile::CITY_RADIUS != 0;
                        }
                    }
                }
                let found = if !clear {
                    -1
                } else if crate::place::wants_town(rd(m, 0x2c0) as u32) {
                    a[3]
                } else {
                    a[4]
                };
                if found < 0 {
                    -1
                } else {
                    let rec = &p[2 + found as usize];
                    i64::from(i16::from_le_bytes([rec[8], rec[9]]))
                }
            }
        };
        assert_eq!(got, want_eax, "get_town — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_town — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_town — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "get_town — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::get_town@00639b90: {n} rows agree");
}
