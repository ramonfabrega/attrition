//! `Game::is_victory_timer@00594950` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::is_victory_timer@00594950`, run by `tools/emu/sweep_obj.py` on its row `game_is_victory_timer` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 64 random bytes; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x444:0x4, +0x44c:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_is_victory_timer") else {
        return;
    };
    let obj = super::Obj {
        name: "is_victory_timer",
        full: "Game::is_victory_timer@00594950",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Not while the game's info says victory (+0x38, a byte): a leader
            // whose low bits are both set (3) with +0x44c held answers 1; then the
            // same over +0x444. The leaders sit 0x6eec apart, three slices a row.
            let mut v = 0;
            if c[0][0x38] == 0 {
                for off in [2, 1] {
                    if (0..8).any(|k| p[3 * k][0] & 3 == 3 && rd(&p[3 * k + off], 0) != 0) {
                        v = 1;
                        break;
                    }
                }
            }
            v
        }
    });
}
