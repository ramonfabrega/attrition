//! `Game::current_age@005946d0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::current_age@005946d0`, run by `tools/emu/sweep_obj.py` on its row `game_current_age` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the word at `p1+0` points at 4 random bytes less `0xdc` (placed); the word at `p3+0` points at 4 random bytes less `0xdc` (placed); the word at `p5+0` points at 4 random bytes less `0xdc` (placed); the word at `p7+0` points at 4 random bytes less `0xdc` (placed); the word at `p9+0` points at 4 random bytes less `0xdc` (placed); the word at `p11+0` points at 4 random bytes less `0xdc` (placed); the word at `p13+0` points at 4 random bytes less `0xdc` (placed); the word at `p15+0` points at 4 random bytes less `0xdc` (placed); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x1, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_current_age") else {
        return;
    };
    let obj = super::Obj {
        name: "current_age",
        full: "Game::current_age@005946d0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let mut best = 0i64;
            for k in 0..8 {
                if p[2 * k][0] & 1 != 0 {
                    best = best.max(i64::from(rd(&p[16 + k], 0) ^ 0x62766));
                }
            }
            best
        }
    });
}
