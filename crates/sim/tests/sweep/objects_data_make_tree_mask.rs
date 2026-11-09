//! `ObjectsData::make_tree_mask@00658880` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::make_tree_mask@00658880`, run by `tools/emu/sweep_obj.py` on its row `objects_data_make_tree_mask` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 2, p1 the place; the global cell 0x00c061d0 points at 1024 random bytes; argument 2 names 4 random bytes; the word at `c0+0x134` points at 4096 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_make_tree_mask") else {
        return;
    };
    let obj = super::Obj {
        name: "make_tree_mask",
        full: "ObjectsData::make_tree_mask@00658880",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
            let (x, y) = (a[0], a[1]);
            let tree = |tx: i32, ty: i32| -> bool {
                tx > -1
                    && ty > -1
                    && tx < ww
                    && ty < wh
                    && p[1][((ww * ty + tx) * 0x1c) as usize] & 0x20 != 0
            };
            let mut mask = 0;
            if tree(x - 1, y) {
                mask |= 1;
            }
            if tree(x, y - 1) {
                mask |= 2;
            }
            if tree(x + 1, y) {
                mask |= 4;
            }
            if tree(x, y + 1) {
                mask |= 8;
            }
            wr(&mut p[0], 0, mask);
            0
        }
    });
}
