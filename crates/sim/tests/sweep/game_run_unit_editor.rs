//! `Game::run_unit_editor@005860a0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::run_unit_editor@005860a0`, run by `tools/emu/sweep_obj.py` on its row `game_run_unit_editor` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 64 random bytes; callee 0x00594b80 (1 args) stubbed; callee 0x005950b0 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_run_unit_editor") else {
        return;
    };
    let obj = super::Obj {
        name: "run_unit_editor",
        full: "Game::run_unit_editor@005860a0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The game object (the global cell at 0xc061ec) is closed with 1 (stub 0), then cleared (stub 1).
            log.push("c0+0x0,1".to_string());
            log.push("c0+0x0".to_string());
            0
        }
    });
}
