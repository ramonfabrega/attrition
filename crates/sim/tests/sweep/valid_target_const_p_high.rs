//! `ObjectData::valid_target_const@006472c0` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::valid_target_const@006472c0`, run by `tools/emu/sweep_obj.py` on its row `valid_target_const_p_high` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: none; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("valid_target_const_p_high") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_target_const",
        full: "ObjectData::valid_target_const@006472c0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 300,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Every row names an index or a player the first test refuses: the
            // original answers 0 before it reads a table.
            assert!(
                !crate::combat::fight::valid_target_const_args(a[0], a[1]),
                "the port lets the arguments through"
            );
            0
        }
    });
}
