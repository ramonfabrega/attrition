//! `WallOut::make_options@00642470` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WallOut::make_options@00642470`, run by `tools/emu/sweep_obj.py` on its row `wall_out_make_options` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c06204 points at 128 random bytes; callee 0x0072c290 (6 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_out_make_options") else {
        return;
    };
    let obj = super::Obj {
        name: "make_options",
        full: "WallOut::make_options@00642470",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `WallOut::make_options()`: `Options::add` (`0x72c290`, `ecx` the options record, `c[0]`, mode at
        // `+0x74`). Mode 0 (`STANDARD_OPTIONS`) adds `OPTION_DISBAND` (0xf, 5, 0, -1, 0, 0); the stub does
        // not move the mode, so then a mode of 0x13 or more adds `OPTION_BACK` (0x42, 4, 3, ...), any other
        // nonzero mode `OPTION_CANCEL` (0x43, 4, 3, ...), and mode 0 nothing more. The mode compares signed.
        let mode = rd(&c[0], 0x74);
        if mode == 0 {
            log.push("c0+0x0,15,5,0,-1,0,0".to_string());
        } else if mode >= 0x13 {
            log.push("c0+0x0,66,4,3,-1,0,0".to_string());
        } else {
            log.push("c0+0x0,67,4,3,-1,0,0".to_string());
        }
        0i64
    });
}
