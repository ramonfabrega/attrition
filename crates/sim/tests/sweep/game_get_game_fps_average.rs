//! `Game::get_game_fps_average@0058da90` under the emulator against the port (item 2537; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::get_game_fps_average@0058da90`, run by `tools/emu/sweep_obj.py` on its row `game_get_game_fps_average` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 2352 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_get_game_fps_average") else {
        return;
    };
    let obj = super::Obj {
        name: "get_game_fps_average",
        full: "Game::get_game_fps_average@0058da90",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let v = rd(&c[0], 0x924);
            if v == 0 {
                i64::from(1000.0f32.to_bits() as i32)
            } else {
                let d = f64::from(v as u32) as f32;
                i64::from((30000.0f32 / d).to_bits() as i32)
            }
        }
    });
}
