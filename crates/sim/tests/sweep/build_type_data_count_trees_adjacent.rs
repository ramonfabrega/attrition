//! `BuildTypeData::count_trees_adjacent@00639090` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::count_trees_adjacent@00639090`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_count_trees_adjacent` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 572 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_count_trees_adjacent") else {
        return;
    };
    let obj = super::Obj {
        name: "count_trees_adjacent",
        full: "BuildTypeData::count_trees_adjacent@00639090",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (xs, ys) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let (p1, p2, p3) = (a[0], a[1], a[2]);
            let (x0, y0) = (rd(m, 0x234).wrapping_add(p1), rd(m, 0x238).wrapping_add(p2));
            let (lo_x, lo_y) = (p1.wrapping_sub(p3), p2.wrapping_sub(p3));
            let mut n = 0i32;
            let mut x = lo_x;
            while x < x0.wrapping_add(p3) {
                let mut y = lo_y;
                while y < p3.wrapping_add(y0) {
                    let outside = x < p1 || x0 <= x || y < p2 || y0 <= y;
                    let corner = (x != p3.wrapping_sub(1).wrapping_add(x0) && x != lo_x)
                        || (y != y0.wrapping_add(p3).wrapping_sub(1) && y != lo_y);
                    if outside && corner && x > -1 && y > -1 && x < xs && y < ys {
                        let k = ((xs * y + x) * 2) as usize;
                        let t = p[0][k];
                        if (t & 0x30) == 0x30 {
                            n += 1;
                        }
                    }
                    y += 1;
                }
                x += 1;
            }
            i64::from(n)
        }
    });
}
