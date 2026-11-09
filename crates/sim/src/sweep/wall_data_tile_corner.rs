//! `WallData::tile_corner@00643440` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WallData::tile_corner@00643440`, run by `tools/emu/sweep_obj.py` on its row `wall_data_tile_corner` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 28 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x18` points at 572 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_data_tile_corner") else {
        return;
    };
    let obj = super::Obj {
        name: "tile_corner",
        full: "WallData::tile_corner@00643440",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (xs, ys) = (rd(&p[2], 0x234), rd(&p[2], 0x238));
            let c = crate::cities::place::tile_corner_of(
                xs,
                ys,
                crate::Pos::new(rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637),
            );
            wr(&mut p[0], 0, c.x);
            wr(&mut p[1], 0, c.y);
            0
        }
    });
}
