//! `Guy::set_new_z@005d86c0` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Guy::set_new_z@005d86c0`, run by `tools/emu/sweep_obj.py` on its row `guy_set_new_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 120 random bytes of object, 2 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_set_new_z") else {
        return;
    };
    let obj = super::Obj {
        name: "set_new_z",
        full: "Guy::set_new_z@005d86c0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[1] != 0 {
                wr(m, 0x70, a[0]);
                wr(m, 0x14, a[0]);
            } else {
                let z = rd(m, 0x14);
                wr(m, 0x70, z);
                wr(m, 0x14, a[0]);
            }
            0
        }
    });
}
