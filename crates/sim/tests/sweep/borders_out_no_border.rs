//! `BordersOut::no_border@008831d0` under the emulator against the port (item 2253; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BordersOut::no_border@008831d0`, run by `tools/emu/sweep_obj.py` on its row `borders_out_no_border` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 312 random bytes; the word at `c0+0x134` points at 1792 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("borders_out_no_border") else {
        return;
    };
    let obj = super::Obj {
        name: "no_border",
        full: "BordersOut::no_border@008831d0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let i = rd(&c[0], 0).wrapping_mul(a[1]).wrapping_add(a[0]) as usize * 0x1c;
            let t = &p[0];
            if u16::from_le_bytes([t[i], t[i + 1]]) & 0x100 != 0 {
                0
            } else {
                let b = t[i + 2] as i8;
                i64::from(b == 1 || b == 2)
            }
        }
    });
}
