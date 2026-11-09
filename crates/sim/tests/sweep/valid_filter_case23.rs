//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_obj.py` on its row `valid_filter_case23` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("valid_filter_case23") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_filter",
        full: "Search::valid_filter@0067dbb0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // (`ebx` is the caller's: the harness holds it zero, so the slot index is 0.)
            // Arm 23: 1 when the +0x24 word of `objects[who][p0]` is zero.
            let who = a[4] as usize;
            i64::from(rd(&p[5 * who + 1], 0x24) == 0)
        }
    });
}
