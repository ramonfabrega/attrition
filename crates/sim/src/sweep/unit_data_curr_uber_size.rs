//! `UnitData::curr_uber_size@0060a760` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::curr_uber_size@0060a760`, run by `tools/emu/sweep_obj.py` on its row `unit_data_curr_uber_size` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 148 random bytes of object, 1 argument(s), the word at `0xc0aec0+2*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 148 random bytes less `0` (placed); the word at `p0+4` points at 148 random bytes less `0` (placed); the word at `p0+8` points at 148 random bytes less `0` (placed); the word at `p0+12` points at 148 random bytes less `0` (placed); the word at `p0+16` points at 148 random bytes less `0` (placed); the word at `p0+20` points at 148 random bytes less `0` (placed); the word at `p0+24` points at 148 random bytes less `0` (placed); the word at `p0+28` points at 148 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_curr_uber_size") else {
        return;
    };
    let obj = super::Obj {
        name: "curr_uber_size",
        full: "UnitData::curr_uber_size@0060a760",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `this` is node 8 and its player's eight figures `p[1..9]`
            // (`who` held at 2); a figure's `o_up` is `+0x8e`, its `o_down`
            // `+0x90`, alive `+8 & 1`; every vtable is `Unit`'s, so
            // `is_captain` is `o_up`'s sign bit. `a[0]` nonzero skips the
            // climb, as the recursion's own call does.
            let fig = |i: usize| -> &[u8] { if i == 8 { &m[..] } else { &p[1 + i][..] } };
            let short = |i: usize, off: usize| i16::from_le_bytes([fig(i)[off], fig(i)[off + 1]]);
            let up = |i: usize| {
                let o = short(i, 0x8e);
                (a[0] == 0 && o >= 0).then_some(o as usize)
            };
            let down = |i: usize| {
                let o = short(i, 0x90);
                (o >= 0 && p[1 + o as usize][8] & 1 != 0).then_some(o as usize)
            };
            // The recursion's `param_1` holds only for its own call: past
            // the first figure the climb is skipped by construction.
            i64::from(crate::squad_size(8, up, down))
        }
    });
}
