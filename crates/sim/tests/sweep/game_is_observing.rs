//! `Game::is_observing@00594490` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Game::is_observing@00594490`, run by `tools/emu/sweep_obj.py` on its row `game_is_observing` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00e335c8 points at 64 random bytes; vtable slot +0x10 (0 args) stubbed; the word at `c0+0x28` points at 32 random bytes less `0` (placed, null a third of the rows); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("game_is_observing") else {
        return;
    };
    let obj = super::Obj {
        name: "is_observing",
        full: "Game::is_observing@00594490",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The net system's local player (+0x28): none answers 0; else its own
            // slot +0x10, entered by a tail jump, answers.
            if rd(&c[0], 0x28) == 0 {
                log.push("-".into());
                0
            } else {
                log.push("p0+0x0".into());
                i64::from(a[0])
            }
        }
    });
}
