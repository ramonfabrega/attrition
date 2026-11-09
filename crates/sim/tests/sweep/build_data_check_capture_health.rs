//! `BuildData::check_capture_health@0062d250` under the emulator against the port (item 2129; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::check_capture_health@0062d250`, run by `tools/emu/sweep_obj.py` on its row `build_data_check_capture_health` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x118 (0 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_check_capture_health") else {
        return;
    };
    let obj = super::Obj {
        name: "check_capture_health",
        full: "BuildData::check_capture_health@0062d250",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let hp = a[0];
            i64::from(if m[8] & 0x20 != 0 { hp > 5 } else { hp > 4 })
        }
    });
}
