//! `World::compute_all_vals@006b3790` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::compute_all_vals@006b3790`, run by `tools/emu/sweep_obj.py` on its row `world_compute_all_vals` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; callee 0x00838ce0 (0 args) stubbed; callee 0x006b0350 (3 args) stubbed; the word at `c0+0x134` points at 512 random bytes less `0` (placed); stub 1 answered from [-3, 0, 5, 255, 256, 400] (90%); stub 1 answers 9 values in call order, printed after the answers; stub 1 writes a value in 0..255 through its argument 2, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_compute_all_vals") else {
        return;
    };
    let obj = super::Obj {
        name: "compute_all_vals",
        full: "World::compute_all_vals@006b3790",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // Every tile, x outer and y inner (the world cell's +0 by +4): `SplashScreen::refresh` (stubbed),
        // then `World::compute_val` (stubbed: the n-th call answers `a[3 + n]` and writes `a[2]` through
        // its out pointer); the answer clamped to 0..255 goes to the tile record's +0xc (0x1c-byte
        // rows at +0x134, row-major by the world's width), the out byte to +0xd.
        let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
        let mut n = 0;
        for x in 0..w.max(0) {
            for y in 0..h.max(0) {
                let v = a[3 + n];
                n += 1;
                let at = ((w * y + x) * 0x1c) as usize;
                p[0][at + 0xc] = v.clamp(0, 255) as u8;
                p[0][at + 0xd] = a[2] as u8;
            }
        }
        0
    });
}
