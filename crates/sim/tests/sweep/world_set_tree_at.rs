//! `World::set_tree_at@006b2060` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_tree_at@006b2060`, run by `tools/emu/sweep_obj.py` on its row `world_set_tree_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 316 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x134` points at 448 random bytes less `0` (placed); the word at `m+0x138` points at 512 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_tree_at") else {
        return;
    };
    let obj = super::Obj {
        name: "set_tree_at",
        full: "World::set_tree_at@006b2060",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let x = rd(&p[0], 0);
            let y = rd(&p[1], 0);
            let flag = a[2];
            let fw = rd(m, 0x18);
            let cw = rd(m, 0);
            let fi = ((fw.wrapping_mul(y).wrapping_add(x)) as usize).wrapping_mul(2);
            let ci = (((((y >> 2).wrapping_mul(cw)).wrapping_add(x >> 2)) as usize)
                .wrapping_mul(0x1c))
            .wrapping_add(0x13);
            let fine = u16::from_le_bytes([p[3][fi], p[3][fi + 1]]);
            let bits = fine & 0x30;
            let out = if flag != 0 {
                if bits != 0x30 {
                    p[2][ci] = p[2][ci].wrapping_sub(1);
                }
                fine | 0x38
            } else {
                let mut v = fine;
                if bits == 0x30 {
                    v &= 0xffcf;
                    p[2][ci] = p[2][ci].wrapping_add(1);
                }
                v & 0xfff7
            };
            p[3][fi..fi + 2].copy_from_slice(&out.to_le_bytes());
            0
        }
    });
}
