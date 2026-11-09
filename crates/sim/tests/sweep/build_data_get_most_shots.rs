//! `BuildData::get_most_shots@0062e300` under the emulator against the port (item 2160; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::get_most_shots@0062e300`, run by `tools/emu/sweep_obj.py` on its row `build_data_get_most_shots` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x1c0 (0 args) stubbed; vtable slot +0x1bc (0 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x2c4` (placed); slot +0x1bc laid as 0x0062e2f0 or its stub, a flag after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_get_most_shots") else {
        return;
    };
    let obj = super::Obj {
        name: "get_most_shots",
        full: "BuildData::get_most_shots@0062e300",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(a[0].wrapping_add(if a[2] == 1 { rd(&p[0], 0) } else { a[1] }))
    });
}
