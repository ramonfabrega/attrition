//! `Player::is_connection@006edb90` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Player::is_connection@006edb90`, run by `tools/emu/sweep_obj.py` on its row `player_is_connection` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 0 argument(s), 16 random bytes at 0x00eae400; 16 random bytes at 0x00eae41c; the word at `c0+0xc` points at 24 random bytes less `0` (placed); the word at `c1+0xc` points at 72 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("player_is_connection") else {
        return;
    };
    let obj = super::Obj {
        name: "is_connection",
        full: "Player::is_connection@006edb90",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Connected: a net player at +0x7c, or the play byte (+0x36) among the
            // pending players (a count and a list of words at 0xeae400, +0xc) or the
            // dropped ones (0xeae41c, 12-byte records whose first word is the play).
            let play = i32::from(m[0x36]);
            if rd(m, 0x7c) != 0
                || (0..rd(&c[0], 0).max(0) as usize).any(|i| rd(&p[0], 4 * i) == play)
                || (0..rd(&c[1], 0).max(0) as usize).any(|i| rd(&p[1], 12 * i) == play)
            {
                1
            } else {
                0
            }
        }
    });
}
