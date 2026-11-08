//! `UnitData::is_gathering@006089b0` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_gathering@006089b0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_gathering` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), 128 random bytes at 0x00c0aec0; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `c0+i8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); stub 1 answered from [7, 7, 7, 7, 0, 12] (90%); stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (8 bytes, ...); the stubbed vtable laid at `p2+0`; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_gathering") else {
        return;
    };
    let obj = super::Obj {
        name: "is_gathering",
        full: "UnitData::is_gathering@006089b0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        let ty = rd(&p[0], 4);
        let mut r = 0;
        if (0x32..=0x35).contains(&ty) && a[2] == 7 {
            r = i32::from(rd(&p[4], 0xc) == i32::from(m[9]));
        }
        i64::from(r)
    });
}
