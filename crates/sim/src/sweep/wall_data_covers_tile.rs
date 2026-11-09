//! `WallData::covers_tile@006439b0` under the emulator against the port (item 2187; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WallData::covers_tile@006439b0`, run by `tools/emu/sweep_obj.py` on its row `wall_data_covers_tile` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 28 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the word at `m+0x18` points at 8 random bytes less `0x234` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wall_data_covers_tile") else {
        return;
    };
    let obj = super::Obj {
        name: "covers_tile",
        full: "WallData::covers_tile@006439b0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::world::Pos;
            let (xs, ys) = (rd(&p[0], 0), rd(&p[0], 4));
            let at = Pos::new(rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
            let corner = crate::cities::place::tile_corner_of(xs, ys, at);
            i64::from(crate::cities::place::footprint_covers(
                corner,
                xs,
                ys,
                Pos::new(a[0], a[1]),
            ))
        }
    });
}
