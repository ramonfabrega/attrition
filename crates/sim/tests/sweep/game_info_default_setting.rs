//! `GameInfo::default_setting@005d7200` under the emulator against the port (item 2644; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GameInfo::default_setting@005d7200`, run by `tools/emu/sweep_obj.py` on its row `game_info_default_setting` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), 32 random bytes at 0x00c06188; `ecx` an integer in 0..29, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_info_default_setting") else {
        return;
    };
    let obj = super::Obj {
        name: "default_setting",
        full: "GameInfo::default_setting@005d7200",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        [
            1, 14, 2, 1, 0, 2, 0, 0, 2, 1, 1, 3, 1, 5, 0, 0, 0, 1, 7, 1, 0, 5, 4, 8, 1, 3, 2, 0, 0,
            0,
        ][a[a.len() - 1] as usize] as i64
    });
}
