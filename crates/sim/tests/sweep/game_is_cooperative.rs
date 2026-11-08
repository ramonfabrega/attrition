//! `Game::is_cooperative@00594810` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::is_cooperative@00594810`, run by `tools/emu/sweep_obj.py` on its row `game_is_cooperative` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 1104 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_is_cooperative") else {
        return;
    };
    let obj = super::Obj {
        name: "is_cooperative",
        full: "Game::is_cooperative@00594810",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        (|| {
            // `team_style` 8..10 is cooperative; 11 and 12 ask whether two of the
            // eight seats in play share a team.
            let g = &c[0];
            match g[0x24] {
                8..=10 => 1,
                11 | 12 => {
                    let mut seen = [false; 8];
                    for k in 0..8 {
                        let at = 0x77 + k * 0x8c;
                        if g[at - 3] & 1 != 0 {
                            let t = usize::from(g[at]);
                            if seen[t] {
                                return 1;
                            }
                            seen[t] = true;
                        }
                    }
                    0
                }
                _ => 0,
            }
        })()
    });
}
