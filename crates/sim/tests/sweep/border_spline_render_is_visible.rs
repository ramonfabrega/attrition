//! `BorderSplineRender::is_visible@008826c0` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BorderSplineRender::is_visible@008826c0`, run by `tools/emu/sweep_obj.py` on its row `border_spline_render_is_visible` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 0 argument(s), callee 0x008642e0 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("border_spline_render_is_visible") else {
        return;
    };
    let obj = super::Obj {
        name: "is_visible",
        full: "BorderSplineRender::is_visible@008826c0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `BorderSplineRender::is_visible()`: the bounding box is four shorts at `this+0x18..0x1e`
        // (`x0, x1, y0, y1`, sign-extended); it answers `TerrainOut::is_region_visible` (`0x8642e0`, a
        // `__cdecl` stub taking `ecx = x0` and the extents `x1 - x0`, `y1 - y0`), whose answer is `a[0]`.
        let s = |o: usize| i16::from_le_bytes([m[o], m[o + 1]]) as i32;
        let (x0, x1, y0, y1) = (s(0x18), s(0x1a), s(0x1c), s(0x1e));
        log.push(format!("{x0},{},{}", x1 - x0, y1 - y0));
        i64::from(a[0])
    });
}
