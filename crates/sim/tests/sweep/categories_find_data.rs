//! `Categories::find_data@006f1e30` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Categories::find_data@006f1e30`, run by `tools/emu/sweep_obj.py` on its row `categories_find_data` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), 4 random bytes at 0x00ee10e3; the word at `m+0x10` points at 360 random bytes less `0x3c` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("categories_find_data") else {
        return;
    };
    let obj = super::Obj {
        name: "find_data",
        full: "Categories::find_data@006f1e30",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The index of the first category (0x58 bytes apart, `p[0]`, the key at +0x3c) whose key is
        // the second argument (`a[1]`) among the `m+0x18` of them, else -1 (the error report is
        // skipped with `ignore_always`, `c[0]`'s low byte, set; item 2844).
        let n = rd(m, 0x18).max(0) as usize;
        (0..n)
            .find(|&i| rd(&p[0], 0x58 * i) == a[1])
            .map_or(-1, |i| i as i64)
    });
}
