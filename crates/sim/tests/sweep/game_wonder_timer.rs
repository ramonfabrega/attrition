//! `Game::wonder_timer@005944f0` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::wonder_timer@005944f0`, run by `tools/emu/sweep_obj.py` on its row `game_wonder_timer` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 3340 random bytes; the global cell 0x00e7fca8 points at 328 random bytes; the global cell 0x00c06188 points at 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_wonder_timer") else {
        return;
    };
    let obj = super::Obj {
        name: "wonder_timer",
        full: "Game::wonder_timer@005944f0",
        cells: 3,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let t = rd(&c[0], 0xd08);
            if t < 1 {
                0
            } else {
                let s = rd(&c[1], 0x144);
                let v = (s / 2).wrapping_add(rd(&c[2], 0).wrapping_mul(t)) / s;
                i64::from(v.max(1))
            }
        }
    });
}
