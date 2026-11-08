//! `BuildTypeData::check_enemy_adjacent@00638d70` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::check_enemy_adjacent@00638d70`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_check_enemy_adjacent` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 572 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the word at `c0+0x138` points at 288 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); 4 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_check_enemy_adjacent") else {
        return;
    };
    let obj = super::Obj {
        name: "check_enemy_adjacent",
        full: "BuildTypeData::check_enemy_adjacent@00638d70",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (xs, ys, w) = (rd(&c[0], 0x18), rd(&c[0], 0x1c), rd(&c[0], 0));
            let (p1, p2, p3) = (a[0], a[1], a[2] as usize);
            let x_hi = rd(m, 0x234).wrapping_add(p1);
            let y_hi = rd(m, 0x238).wrapping_add(p2);
            let ret = (|| -> i32 {
                let mut r = -1i32;
                let mut x = p1.wrapping_sub(1);
                while x <= x_hi {
                    let mut y = p2.wrapping_sub(1);
                    while y <= y_hi {
                        let outside = x < p1 || x_hi <= x || y < p2 || y_hi <= y;
                        let corner = (x != x_hi && x != p1.wrapping_sub(1))
                            || (y != y_hi && y != p2.wrapping_sub(1));
                        if outside
                            && corner
                            && x > -1
                            && y > -1
                            && x < xs
                            && y < ys
                            && (p[8][((xs * y + x) * 2) as usize] & 0x30) != 0x20
                        {
                            let cell = (((y >> 2) * w + (x >> 2)) * 0x1c) as usize;
                            r = i32::from(p[9][cell + 0xf] as i8);
                            if r < 0 {
                                return r;
                            }
                            let own = rd(&p[2 * p3], 0);
                            if r == own {
                                return r;
                            }
                            if rd(&p[2 * p3 + 1], 4 * r as usize) == 2
                                && rd(&p[2 * r as usize + 1], 4 * own as usize) == 2
                            {
                                return r;
                            }
                        }
                        y += 1;
                    }
                    x += 1;
                }
                r
            })();
            i64::from(ret)
        }
    });
}
