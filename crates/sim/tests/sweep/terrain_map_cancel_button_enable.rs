//! `TerrainMap::cancel_button_enable@009b5ee0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainMap::cancel_button_enable@009b5ee0`, run by `tools/emu/sweep_obj.py` on its row `terrain_map_cancel_button_enable` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 3104 random bytes of object, 1 argument(s), callee 0x00a54110 (1 args) stubbed; the word at `m+0xc14` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_map_cancel_button_enable") else {
        return;
    };
    let obj = super::Obj {
        name: "cancel_button_enable",
        full: "TerrainMap::cancel_button_enable@009b5ee0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A null cancel button (the pointer at +0xc14) is a plain return; else the window's set_enabled
            // (stub 0, a tail jump) is handed the button and 1 for any nonzero argument, 0 for zero.
            if rd(m, 0xc14) == 0 {
                log.push("-".to_string());
            } else {
                log.push(format!("p0+0x0,{}", i32::from(a[0] != 0)));
            }
            0
        }
    });
}
