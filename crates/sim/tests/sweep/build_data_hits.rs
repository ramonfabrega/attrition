//! `BuildData::hits@0062e740` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::hits@0062e740`, run by `tools/emu/sweep_obj.py` on its row `build_data_hits` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 220 random bytes of object, 1 argument(s), vtable slot +0x4c (0 args) stubbed; callee 0x006508c0 (1 args) stubbed; the word at `m+0x8c` points at 8 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 1 answered from [0, 1, 7, 50, 100, 1000, -5] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_hits") else {
        return;
    };
    let obj = super::Obj {
        name: "hits",
        full: "BuildData::hits@0062e740",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The hit points `m+0x54` (`a[0] == 0`) or `m+0x20`, scaled (float, `cvttss2si`: out of range
        // and NaN are `i32::MIN`) by the build queue's progress when the building is active (slot +0x4c,
        // `a[1]`, or bit 4 of `m[8]` when laid as the base, flag `a[3]`), `queued` (`m[0x82]`), a
        // non-empty queue (`m+0x88`) and its head type (the short at `p0+4`) 0x29a or 0x286: the
        // queue's time `rd(p[0], 0)` against `train_time` (`a[2]`), at least 1 (item 2844).
        let h = if a[0] == 0 { rd(m, 0x54) } else { rd(m, 0x20) };
        let active = if a[3] != 0 { m[8] & 4 != 0 } else { a[1] != 0 };
        let ty = i32::from(i16::from_le_bytes([p[0][4], p[0][5]]));
        if active && m[0x82] != 0 && rd(m, 0x88) > 0 && (ty == 0x29a || ty == 0x286) {
            let t = a[2];
            let q = rd(&p[0], 0);
            let f = (t.wrapping_sub(q) as f32 * h as f32) / t as f32;
            let r = if !(-2147483648.0..2147483648.0).contains(&f) {
                i32::MIN
            } else {
                f as i32
            };
            i64::from(if r < 1 { 1 } else { r })
        } else {
            i64::from(h)
        }
    });
}
