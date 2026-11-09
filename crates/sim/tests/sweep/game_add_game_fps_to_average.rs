//! `Game::add_game_fps_to_average@0058dae0` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::add_game_fps_to_average@0058dae0`, run by `tools/emu/sweep_obj.py` on its row `game_add_game_fps_to_average` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2344 random bytes of object, 1 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_add_game_fps_to_average") else {
        return;
    };
    let obj = super::Obj {
        name: "add_game_fps_to_average",
        full: "Game::add_game_fps_to_average@0058dae0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let pos = rd(m, 0x920);
            let v = a[0];
            let o = 0x8a8 + 4 * pos as usize;
            let old = rd(m, o);
            wr(m, 0x924, rd(m, 0x924).wrapping_add(v.wrapping_sub(old)));
            wr(m, o, v);
            wr(m, 0x920, if pos + 1 == 0x1e { 0 } else { pos + 1 });
            0
        }
    });
}
