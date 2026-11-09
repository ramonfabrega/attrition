//! `ObjectTypeData::basic_type@006619e0` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectTypeData::basic_type@006619e0`, run by `tools/emu/sweep_obj.py` on its row `object_type_data_basic_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), vtable slot +0xe4 (0 args) stubbed; the word at `0xc0aaf4` points at 4 random bytes less `i32(m,0x3c)*4` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the stubbed vtable laid at `p1+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_type_data_basic_type") else {
        return;
    };
    let obj = super::Obj {
        name: "basic_type",
        full: "ObjectTypeData::basic_type@006619e0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if rd(m, 0x3c) < 0 {
            i64::from(rd(m, 4))
        } else {
            i64::from(a[0])
        }
    });
}
