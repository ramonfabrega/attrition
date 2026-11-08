//! `draw_thick_diamond@008c51a0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `draw_thick_diamond@008c51a0`, run by `tools/emu/sweep_obj.py` on its row `draw_thick_diamond` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 6 argument(s), callee 0x008c6180 (6 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("draw_thick_diamond") else {
        return;
    };
    let obj = super::Obj {
        name: "draw_thick_diamond",
        full: "draw_thick_diamond@008c51a0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // draw_thick_rectangle (stub 0, cdecl) is handed the three Coords and the twelve-byte Color as six
            // words, and the caller's `ecx` in `edx`; its answer is the result.
            log.push(format!(
                "m+0x0,m+0x0,{},{},{},{},{},{}",
                a[0], a[1], a[2], a[3], a[4], a[5]
            ));
            i64::from(a[6])
        }
    });
}
