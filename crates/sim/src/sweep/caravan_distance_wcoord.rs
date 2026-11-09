//! `Caravan::distance@0073d300` under the emulator against the port (item 2181; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Caravan::distance@0073d300`, run by `tools/emu/sweep_obj.py` on its row `caravan_distance_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), the global cell 0x00c06188 points at 4 random bytes; callee 0x0046cff0 (0 args) stubbed; stub 0 answered from [-3, -2, -1, -1, -2, 0, 1, 2, 24, 25, 49, 50, 79, 80, 99, 100, 159, 160, 199, 200, 319, 320, 399, 400, 2147483647] (90%); `__cdecl`: the caller pops the arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("caravan_distance_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "distance",
        full: "Caravan::distance@0073d300",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(crate::economy::caravan::distance_band(a[4], rd(&c[0], 0)))
    });
}
