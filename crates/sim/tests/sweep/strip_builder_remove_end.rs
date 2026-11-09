//! `StripBuilder::remove_end@008820a0` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `StripBuilder::remove_end@008820a0`, run by `tools/emu/sweep_obj.py` on its row `strip_builder_remove_end` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), argument 0 names 16 random bytes; argument 1 names 20 random bytes; the word at `p1+0x10` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("strip_builder_remove_end") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_end",
        full: "StripBuilder::remove_end@008820a0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let i = usize::from(u16::from_le_bytes([p[0][0xc], p[0][0xd]]));
            if i != 0 && i as i32 <= rd(&p[1], 4) {
                wr(&mut p[2], 4 * i - 4, 0);
            }
            p[0][0xc] = 0;
            p[0][0xd] = 0;
            0
        }
    });
}
