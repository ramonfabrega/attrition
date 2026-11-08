//! `Doober::remove_hold_doobers@00846cd0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Doober::remove_hold_doobers@00846cd0`, run by `tools/emu/sweep_obj.py` on its row `doober_remove_hold_doobers` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 1 argument(s), `p` windows in order: p0 the place; 4 random bytes at 0x00ee121c; the word at `m+0x110` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("doober_remove_hold_doobers") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_hold_doobers",
        full: "Doober::remove_hold_doobers@00846cd0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // Index under the count at `+0xcc`: the hold-fade array's slot (the pointer at `+0x110`) set to
        // 0xffffff01; otherwise the error report (muted by `ignore_always`, cell 0xee121c) and the same OK.
        if a[0] < rd(m, 0xcc) {
            wr(&mut p[0], 4 * a[0] as usize, 0xffffff01u32 as i32);
        }
        0
    });
}
