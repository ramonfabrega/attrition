//! `BuildTypeData::is_dock_tile@00636700` under the emulator against the port (item 3261; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::is_dock_tile@00636700`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_is_dock_tile` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_is_dock_tile") else {
        return;
    };
    let obj = super::Obj {
        name: "is_dock_tile",
        full: "BuildTypeData::is_dock_tile@00636700",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The world `c[0]` (cells wide and high at +0 and +4; the tile
        // dimensions at +0x18/+0x1c are held to four times them), its
        // cell records `p[0]` (0x1c bytes: flags at +0, land at +2) and
        // tile masks `p[1]`; the cell is `(a[0], a[1])`, `a[2]` unread.
        // The crate's own rule, over this data.
        let (cw, ch) = (rd(&c[0], 0), rd(&c[0], 4));
        let tw = rd(&c[0], 0x18);
        let cells = &p[0];
        let tiles = &p[1];
        let r = crate::ai::transport::dock_tile_in(
            cw,
            ch,
            |cell| {
                let o = ((cell.y * cw + cell.x) * 0x1c) as usize;
                (
                    u16::from_le_bytes([cells[o], cells[o + 1]]),
                    cells[o + 2] as i8,
                )
            },
            |t| {
                let o = ((t.y * tw + t.x) * 2) as usize;
                u16::from_le_bytes([tiles[o], tiles[o + 1]])
            },
            crate::world::Cell::new(a[0], a[1]),
        );
        i64::from(r)
    });
}
