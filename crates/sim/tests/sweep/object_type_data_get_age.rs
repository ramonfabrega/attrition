//! `ObjectTypeData::get_age@00661ac0` under the emulator against the port (item 2160; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectTypeData::get_age@00661ac0`, run by `tools/emu/sweep_obj.py` on its row `object_type_data_get_age` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 640 random bytes of object, 0 argument(s), vtable slot +0xe8 (0 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_type_data_get_age") else {
        return;
    };
    let obj = super::Obj {
        name: "get_age",
        full: "ObjectTypeData::get_age@00661ac0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let g = rd(&m[..], 0x278);
            i64::from(if g == -1 { a[0] } else { g })
        }
    });
}
