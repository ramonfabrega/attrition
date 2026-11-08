//! `TerrainVis::invalidate_wcoord@008b2e80` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainVis::invalidate_wcoord@008b2e80`, run by `tools/emu/sweep_obj.py` on its row `terrain_vis_invalidate_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00c06218 points at 27328 random bytes; argument 0 names 16 random bytes; argument 1 names 16 random bytes; callee 0x008b13d0 (1 args) stubbed; the word at `c1+0x6ab0` points at 256 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_vis_invalidate_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "invalidate_wcoord",
        full: "TerrainVis::invalidate_wcoord@008b2e80",
        cells: 2,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The coord info at (*x, *y) of the terrain's table (row width the world cell's word, the pointers at terrain+0x6ab0) is handed to the one-coord overload, ecx the entry this.
            let idx = (rd(&p[1], 0) * rd(&c[0], 0) + rd(&p[0], 0)) as usize;
            log.push(format!("m+0x0,{}", rd(&p[2], 4 * idx)));
            0
        }
    });
}
