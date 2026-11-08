//! `Game::get_patch_version@00595260` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::get_patch_version@00595260`, run by `tools/emu/sweep_obj.py` on its row `game_get_patch_version` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), 4 random bytes at 0x00e7fe4c; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_get_patch_version") else {
        return;
    };
    let obj = super::Obj {
        name: "get_patch_version",
        full: "Game::get_patch_version@00595260",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `info.version` (`m+0xc`) compared byte by byte, low byte first, against the executable's
        // `xpack_rtm_version` (`c[0]`): 8 (`XPACK_RTM_VERSION`) when it is no greater, else 9 (item 2844).
        let v = rd(m, 0xc).to_le_bytes();
        let x = rd(&c[0], 0).to_le_bytes();
        if v <= x { 8 } else { 9 }
    });
}
