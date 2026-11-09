//! `Player::win@006ee290` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Player::win@006ee290`, run by `tools/emu/sweep_obj.py` on its row `player_win` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c06210 points at 768 random bytes; callee 0x006ee110 (0 args) stubbed; callee 0x0097f770 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("player_win") else {
        return;
    };
    let obj = super::Obj {
        name: "win",
        full: "Player::win@006ee290",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Flags (+0x30, a word) take 0x100, then `end_game` (stub 0); the console's player
            // (+0x2a0 of the record the cell 0xc06210 names) equal to this's (+0x36, a byte) and
            // flag 0x40 clear → the sound 0x81 (stub 1, its `ecx` the stub-left `this`).
            let f = u16::from_le_bytes([m[0x30], m[0x31]]) | 0x100;
            m[0x30..0x32].copy_from_slice(&f.to_le_bytes());
            log.push("m+0x0".into());
            if i32::from(m[0x36]) == rd(&c[0], 0x2a0) && f & 0x40 == 0 {
                log.push("m+0x0,129".into());
            } else {
                log.push("-".into());
            }
            0
        }
    });
}
