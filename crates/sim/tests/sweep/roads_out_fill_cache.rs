//! `RoadsOut::fill_cache@00893b30` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `RoadsOut::fill_cache@00893b30`, run by `tools/emu/sweep_obj.py` on its row `roads_out_fill_cache` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 336 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 320 random bytes; the word at `c0+0x138` points at 128 random bytes less `0` (placed); 9 rows 0x4 apart at `@0xadc400`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 9 rows 0x4 apart at `@0xadcaf0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("roads_out_fill_cache") else {
        return;
    };
    let obj = super::Obj {
        name: "fill_cache",
        full: "RoadsOut::fill_cache@00893b30",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            for i in 0..9 {
                let y = rd(&p[i], 0).wrapping_add(a[1]);
                let x = rd(&p[9 + i], 0).wrapping_add(a[0]);
                let (d, e) = (0xfc + 4 * i, 0xfc + 4 * i + 0x2c);
                if x < 0 || y < 0 || x >= w || y >= h {
                    wr(m, d, 0);
                    wr(m, e, 0);
                } else {
                    let t = p[18][2 * (w * y + x) as usize];
                    let seen = t & 0x30 == 0x10;
                    wr(m, e, i32::from(seen));
                    wr(m, d, i32::from(t & 3 == 3 && !seen));
                }
            }
            0
        }
    });
}
