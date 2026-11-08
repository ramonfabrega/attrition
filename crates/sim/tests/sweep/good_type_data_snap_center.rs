//! `GoodTypeData::snap_center@0066fcb0` under the emulator against the port (item 2020; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GoodTypeData::snap_center@0066fcb0`, run by `tools/emu/sweep_obj.py` on its row `good_type_data_snap_center` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 576 random bytes of object, 4 argument(s), argument 2 names 4 random bytes; argument 3 names 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("good_type_data_snap_center") else {
        return;
    };
    let obj = super::Obj {
        name: "snap_center",
        full: "GoodTypeData::snap_center@0066fcb0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let off = if m[0x234] & 1 == 0 { 0x60 } else { 0 };
            for (k, w) in p.iter_mut().enumerate() {
                let v = a[k].wrapping_mul(0x300).wrapping_add(0x120 + off);
                w[..4].copy_from_slice(&v.to_le_bytes());
            }
            0
        }
    });
}
