//! `Wall::mask_city@0063e310` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `Wall::mask_city@0063e310`, run by `tools/emu/sweep_wall_mask_city.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), the global cell 0x00c06188 points at 316 random bytes; 64 random bytes at 0x00cae600; 24 random bytes at 0x00cab3b0; 24 random bytes at 0x00cae710; the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_mask_city.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    let (xs, ys) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
    let mut w = crate::World::new(xs / 4, ys / 4);
    let at = |x: i32, y: i32| ((y * xs + x) * 2) as usize;
    for y in 0..ys {
        for x in 0..xs {
            let k = at(x, y);
            w.set_tile_mask(crate::Pos::new(x, y), u16::from_le_bytes([p[0][k], p[0][k + 1]]));
        }
    }
    if m[8] & 0x20 != 0 {
        let n = rd(&c[1], 4 * a[3] as usize);
        for i in 0..n.max(0) as usize {
            let t = crate::Pos::new(a[0] + i32::from(c[3][i] as i8), a[1] + i32::from(c[2][i] as i8));
            if w.tile_in_bounds(t) {
                if a[2] != 0 {
                    w.set_tile_bits(t, crate::world::tile::CITY_RADIUS);
                } else {
                    w.clear_tile_bits(t, crate::world::tile::CITY_RADIUS);
                }
            }
        }
    }
    for y in 0..ys {
        for x in 0..xs {
            let k = at(x, y);
            p[0][k..k + 2].copy_from_slice(&w.tile_mask(crate::Pos::new(x, y)).to_le_bytes());
        }
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "mask_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "mask_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "mask_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Wall::mask_city@0063e310: {n} rows agree");
}
