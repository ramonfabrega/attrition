//! `Guy::copy_data@005d84b0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Guy::copy_data@005d84b0`, run by `tools/emu/sweep_obj.py` on its row `guy_copy_data` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 192 random bytes of object, 1 argument(s), argument 0 names 188 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_copy_data") else {
        return;
    };
    let obj = super::Obj {
        name: "copy_data",
        full: "Guy::copy_data@005d84b0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // copy_data: the byte at +0x9d and +0xa2, then the words +0xc..+0x14, +0x18, +0x20..+0x3c, +0x40 (a float, copied as bits)
            let s = &p[0];
            m[0x9d] = s[0x9d];
            m[0xa2] = s[0xa2];
            for (d, o) in [(0xc, 0xc), (0x10, 0x10), (0x14, 0x14), (0x18, 0x18)] {
                m[d..d + 4].copy_from_slice(&s[o..o + 4]);
            }
            m[0x20..0x44].copy_from_slice(&s[0x20..0x44]);
            0
        }
    });
}
