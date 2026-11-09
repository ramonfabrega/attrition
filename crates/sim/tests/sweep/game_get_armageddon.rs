//! `Game::get_armageddon@00594020` under the emulator against the port (item 2241; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::get_armageddon@00594020`, run by `tools/emu/sweep_obj.py` on its row `game_get_armageddon` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 3360 random bytes; the global cell 0x00c061ec points at 1708 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_get_armageddon") else {
        return;
    };
    let obj = super::Obj {
        name: "get_armageddon",
        full: "Game::get_armageddon@00594020",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let v = rd(&c[0], 0xd1c)
                .wrapping_mul(rd(&c[1], 0x6a8))
                .wrapping_add(rd(&c[0], 0xd18).wrapping_mul(rd(&c[1], 0x6a0)))
                .wrapping_add(rd(&c[0], 0xd14));
            let k = c[1][0x2d];
            let mut v = match k {
                7 => v.wrapping_mul(3),
                5 | 6 | 11 | 12 => v.wrapping_mul(2),
                _ => v,
            };
            if k == 8 && v < 100 {
                v = 100;
            }
            i64::from(v)
        }
    });
}
