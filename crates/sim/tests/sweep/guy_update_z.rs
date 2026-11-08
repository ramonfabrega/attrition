//! `Guy::update_z@005d9950` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Guy::update_z@005d9950`, run by `tools/emu/sweep_obj.py` on its row `guy_update_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 176 random bytes of object, 0 argument(s), 128 random bytes at 0x00c0aec0; callee 0x00866560 (4 args) stubbed; the word at `c0+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 560 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..99 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_update_z") else {
        return;
    };
    let obj = super::Obj {
        name: "update_z",
        full: "Guy::update_z@005d9950",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        if rd(&p[2], 0x218) != 1 {
            let v = a[1];
            log.push(format!("12628208,sp-0xc,{},{},0", rd(m, 0xc), rd(m, 0x10)));
            wr(m, 0x14, v);
        } else {
            log.push("-".into());
            wr(m, 0x14, 0);
        }
        0i64
    });
}
