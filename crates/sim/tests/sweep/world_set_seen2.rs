//! `World::set_seen2@006b4bb0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_seen2@006b4bb0`, run by `tools/emu/sweep_obj.py` on its row `world_set_seen2` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p5 the place(s), in flag order; the global cell 0x00c06188 points at 1024 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x008d2760 (2 args) stubbed; the word at `c0+0x160` points at 64 random bytes less `0` (placed); the word at `c0+0x15c` points at 64 random bytes less `0` (placed); the word at `c0+0x168` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 512 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_seen2") else {
        return;
    };
    let obj = super::Obj {
        name: "set_seen2",
        full: "World::set_seen2@006b4bb0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 1500,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The fine grid's seen bits at `y * width + x` (world +0xc; the planes at +0x160 and +0x15c),
        // the half-resolution plane at +0x168 and the tile record's flag byte (+0x14 of 0x1c-byte
        // rows at +0x134) at `(y >> 1) * half + (x >> 1)`; `WorldMapFog::add_update` (stubbed) and a 1
        // when either fine plane changed. Argument 3 non-zero skips the +0x15c and +0x168 writes.
        let (half, wide) = (rd(&c[0], 0), rd(&c[0], 0xc));
        let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
        let b = a[2] as u8;
        let idx = (wide * y + x) as usize;
        let h = ((y >> 1) * half + (x >> 1)) as usize;
        let (old160, old15c) = (p[2][idx], p[3][idx]);
        p[2][idx] |= b;
        if a[3] == 0 {
            p[3][idx] |= b;
            p[4][h] |= b;
        }
        p[5][h * 0x1c + 0x14] |= b;
        i64::from(p[2][idx] != old160 || p[3][idx] != old15c)
    });
}
