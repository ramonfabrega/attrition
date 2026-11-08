//! `WorldData::is_detected_by_enemy@006b50f0` under the emulator against the port (item 2149; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::is_detected_by_enemy@006b50f0`, run by `tools/emu/sweep_obj.py` on its row `world_data_is_detected_by_enemy` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 368 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x164` points at 4 random bytes less `i32(m,0xc)*i32(p[1],0)+i32(p[0],0)` (placed); the word at `0xc061e0` points at 4 random bytes less `a[2]*0x6eec+0x6929` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_is_detected_by_enemy") else {
        return;
    };
    let obj = super::Obj {
        name: "is_detected_by_enemy",
        full: "WorldData::is_detected_by_enemy@006b50f0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(p[2][0] & !p[3][0])
    });
}
