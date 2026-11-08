//! `WorldData::get_down@004613d0` under the emulator against the port (item 2149; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::get_down@004613d0`, run by `tools/emu/sweep_obj.py` on its row `world_data_get_down` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 320 random bytes of object, 4 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; argument 2 names 4 random bytes; argument 3 names 4 random bytes; the word at `m+0x134` points at 28 random bytes less `(i32(m,0)*i32(p[1],0)+i32(p[0],0))*0x1c` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_get_down") else {
        return;
    };
    let obj = super::Obj {
        name: "get_down",
        full: "WorldData::get_down@004613d0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let d = u16::from_le_bytes([p[4][8], p[4][9]]);
            let w = i16::from_le_bytes([p[4][10], p[4][11]]);
            wr(&mut p[2], 0, i32::from(d as i16));
            wr(&mut p[3], 0, i32::from(w));
            i64::from((!d) >> 15)
        }
    });
}
