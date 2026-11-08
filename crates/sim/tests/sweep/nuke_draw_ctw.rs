//! `Nuke::draw_ctw@0092b9f0` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Nuke::draw_ctw@0092b9f0`, run by `tools/emu/sweep_obj.py` on its row `nuke_draw_ctw` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), 4 random bytes at 0x00c0a8a0; callee 0x00926410 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("nuke_draw_ctw") else {
        return;
    };
    let obj = super::Obj {
        name: "draw_ctw",
        full: "Nuke::draw_ctw@0092b9f0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The third float's bits are stored at 0xc0a8a0 (c[0]); NukeOut::draw (stub 0) is handed the object
            // and the first two floats; the result is 0.
            wr(&mut c[0], 0, a[2]);
            log.push(format!("m+0x0,{},{}", a[0], a[1]));
            0
        }
    });
}
