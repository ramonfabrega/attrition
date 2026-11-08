//! `Game::clock_init@00588530` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::clock_init@00588530`, run by `tools/emu/sweep_obj.py` on its row `game_clock_init` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2128 random bytes of object, 0 argument(s), the global cell 0x00c061c0 points at 4 random bytes; the import cell 0x00ac5460 holds a stub (0 args); stub 0 answered from [0, 1, 1000, 50000, 100000, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_clock_init") else {
        return;
    };
    let obj = super::Obj {
        name: "clock_init",
        full: "Game::clock_init@00588530",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `timeGetTime` (stub 0, through the import cell) is the graphic clock (+0x840); the tick and loop
            // counters (+0x844, +0x848) are cleared and the AI speed (the cell at 0xc061c0) is set to 1.
            wr(m, 0x840, a[0]);
            wr(m, 0x844, 0);
            wr(m, 0x848, 0);
            wr(&mut c[0], 0, 1);
            0
        }
    });
}
