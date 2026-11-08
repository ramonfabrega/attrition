//! `UnitTypeData::can_entrench@0061d470` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitTypeData::can_entrench@0061d470`, run by `tools/emu/sweep_obj.py` on its row `unit_type_data_can_entrench` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 716 random bytes of object, 0 argument(s), vtable slot +0xf0 (1 args) stubbed; slot +0xf0 laid as 0x00470860 or its stub, a flag after the answers; stub 0 answered from [0, 1, 32, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_type_data_can_entrench") else {
        return;
    };
    let obj = super::Obj {
        name: "can_entrench",
        full: "UnitTypeData::can_entrench@0061d470",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let v = if a[1] != 0 { rd(m, 0x1e4) & 0x20 } else { a[0] };
            i64::from(v != 0 && rd(m, 0x1e8) != 0 && rd(m, 0x2c8) & 0x10000 != 0)
        }
    });
}
