//! `UnitData::is_detected@0060a630` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_detected@0060a630`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_detected` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 24 random bytes of object, 1 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 360 random bytes; the word at `c1+0x164` points at 1 random bytes less `((i32(m,0x14)^0x63637)>>7)//3*i32(c[1],0xc)+((i32(m,0x10)^0x63637)>>7)//3` (placed); the word at `0xc061e0` points at 1 random bytes less `a[0]*0x6eec+0x6929` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_detected") else {
        return;
    };
    let obj = super::Obj {
        name: "is_detected",
        full: "UnitData::is_detected@0060a630",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[0] == i32::from(m[9]) {
                1
            } else {
                i64::from(p[1][0] & p[0][0] != 0)
            }
        }
    });
}
