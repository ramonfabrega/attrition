//! `Wall::mask_city@0063e310` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Wall::mask_city@0063e310`, run by `tools/emu/sweep_obj.py` on its row `wall_mask_city` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the global cell 0x00c06188 points at 316 random bytes; 64 random bytes at 0x00cae600; 24 random bytes at 0x00cab3b0; 24 random bytes at 0x00cae710; the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_mask_city") else {
        return;
    };
    let obj = super::Obj {
        name: "mask_city",
        full: "Wall::mask_city@0063e310",
        cells: 4,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (xs, ys) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let mut w = crate::World::new(xs / 4, ys / 4);
            let at = |x: i32, y: i32| ((y * xs + x) * 2) as usize;
            for y in 0..ys {
                for x in 0..xs {
                    let k = at(x, y);
                    w.set_tile_mask(
                        crate::Pos::new(x, y),
                        u16::from_le_bytes([p[0][k], p[0][k + 1]]),
                    );
                }
            }
            if m[8] & 0x20 != 0 {
                let n = rd(&c[1], 4 * a[3] as usize);
                let (ys, xs) = (&c[2], &c[3]);
                for (&dx, &dy) in xs.iter().zip(ys.iter()).take(n.max(0) as usize) {
                    let t = crate::Pos::new(a[0] + i32::from(dx as i8), a[1] + i32::from(dy as i8));
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
                    p[0][k..k + 2]
                        .copy_from_slice(&w.tile_mask(crate::Pos::new(x, y)).to_le_bytes());
                }
            }
            0
        }
    });
}
