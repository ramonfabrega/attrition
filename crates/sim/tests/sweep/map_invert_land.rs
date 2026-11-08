//! `Map::invert_land@0069d880` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::invert_land@0069d880`, run by `tools/emu/sweep_obj.py` on its row `map_invert_land` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; callee 0x00680060 (0 args) stubbed; the word at `c0+0x134` points at 2048 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_invert_land") else {
        return;
    };
    let obj = super::Obj {
        name: "invert_land",
        full: "Map::invert_land@0069d880",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 300,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
            for y in 0..wh.max(0) {
                for x in 0..ww.max(0) {
                    let o = ((ww * y + x) * 0x1c) as usize;
                    let flags = u16::from_le_bytes([p[0][o], p[0][o + 1]]);
                    let land = flags & 0x100 == 0 && matches!(p[0][o + 2], 1 | 2);
                    p[0][o + 2..o + 4]
                        .copy_from_slice(&(if land { 0u16 } else { 2u16 }).to_le_bytes());
                    p[0][o + 4..o + 6].copy_from_slice(&0u16.to_le_bytes());
                }
            }
            0
        }
    });
}
