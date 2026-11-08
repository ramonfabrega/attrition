//! `WorldData::is_pass_land@006b56b0` under the emulator against the port (item 2160; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::is_pass_land@006b56b0`, run by `tools/emu/sweep_obj.py` on its row `world_data_is_pass_land` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 312 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b4830 (2 args) stubbed; the word at `c0+0x134` points at 4 random bytes less `(i32(p[1],0)*i32(c[0],0)+i32(p[0],0))*0x1c` (placed); stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_is_pass_land") else {
        return;
    };
    let obj = super::Obj {
        name: "is_pass_land",
        full: "WorldData::is_pass_land@006b56b0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(p[2][0] & 0x30 == 0 && a[2] == 0)
    });
}
