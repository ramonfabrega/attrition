//! `TerrainVis::invalidate_fog_wcoord@008b1e60` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainVis::invalidate_fog_wcoord@008b1e60`, run by `tools/emu/sweep_obj.py` on its row `terrain_vis_invalidate_fog_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00c06218 points at 27328 random bytes; argument 0 names 16 random bytes; argument 1 names 16 random bytes; the word at `c1+0x6ab0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 116 random bytes less `0` (placed); the word at `p2+4` points at 116 random bytes less `0` (placed); the word at `p2+8` points at 116 random bytes less `0` (placed); the word at `p2+0xc` points at 116 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_vis_invalidate_fog_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "invalidate_fog_wcoord",
        full: "TerrainVis::invalidate_fog_wcoord@008b1e60",
        cells: 2,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The coord info at (*x, *y) (row width the world cell's word): unless its +0x70 is the terrain-vis last_moved_frame (this+0xfc) or -2 it is set to -1, else to -2.
            let idx = (rd(&p[1], 0) * rd(&c[0], 0) + rd(&p[0], 0)) as usize;
            let v = rd(&p[3 + idx], 0x70);
            wr(
                &mut p[3 + idx],
                0x70,
                if v != rd(m, 0xfc) && v != -2 { -1 } else { -2 },
            );
            0
        }
    });
}
