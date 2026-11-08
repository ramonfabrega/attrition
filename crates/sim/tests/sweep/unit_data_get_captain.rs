//! `UnitData::get_captain@00610ab0` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::get_captain@00610ab0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_get_captain` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 144 random bytes of object, 0 argument(s), vtable slot +0xe4 (0 args) stubbed; the word at `0xc0aec0+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0x8e)*4` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the stubbed vtable laid at `p1+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_get_captain") else {
        return;
    };
    let obj = super::Obj {
        name: "get_captain",
        full: "UnitData::get_captain@00610ab0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if i16::from_le_bytes([m[0x8e], m[0x8f]]) < 0 {
            i64::from(i16::from_le_bytes([m[0xa], m[0xb]]))
        } else {
            i64::from(a[0])
        }
    });
}
