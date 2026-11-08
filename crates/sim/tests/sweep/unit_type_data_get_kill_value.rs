//! `UnitTypeData::get_kill_value@0061d3d0` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitTypeData::get_kill_value@0061d3d0`, run by `tools/emu/sweep_obj.py` on its row `unit_type_data_get_kill_value` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 632 random bytes of object, 1 argument(s), vtable slot +0x7c (1 args) stubbed; the word at `0xc061e0` points at 808 random bytes less `(a[0]*0x3776+i32(m,4))*2+0x56fe` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_type_data_get_kill_value") else {
        return;
    };
    let obj = super::Obj {
        name: "get_kill_value",
        full: "UnitTypeData::get_kill_value@0061d3d0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[0] < 0 {
                i64::from(a[1])
            } else {
                let k = i32::from(u16::from_le_bytes([p[0][0], p[0][1]]))
                    + i32::from(u16::from_le_bytes([p[0][0x324], p[0][0x325]]));
                i64::from(
                    rd(m, 0x274)
                        .wrapping_add(rd(m, 0x270))
                        .wrapping_mul(k)
                        .wrapping_add(a[1]),
                )
            }
        }
    });
}
