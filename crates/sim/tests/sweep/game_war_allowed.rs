//! `Game::war_allowed@00594670` under the emulator against the port (item 2187; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::war_allowed@00594670`, run by `tools/emu/sweep_obj.py` on its row `game_war_allowed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1364 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 52 random bytes; the global cell 0x00e80088 points at 1144 random bytes; callee 0x005946d0 (0 args) stubbed; stub 0 answered from [-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_war_allowed") else {
        return;
    };
    let obj = super::Obj {
        name: "war_allowed",
        full: "Game::war_allowed@00594670",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let rush = i32::from(c[0][0x32]);
            let allowed = rush == 0
                || a[0] >= rush
                || (rush >= 9
                    && rd(m, 0x550) >= rd(&c[1], rush as usize * 0x58 + 0x3c).wrapping_mul(900));
            i64::from(allowed)
        }
    });
}
