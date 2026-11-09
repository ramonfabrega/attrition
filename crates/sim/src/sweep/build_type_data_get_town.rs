//! `BuildTypeData::get_town@00639b90` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::get_town@00639b90`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_get_town` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 708 random bytes of object, 3 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00c061d4 points at 256 random bytes; callee 0x00658190 (3 args) stubbed; callee 0x0065b870 (5 args) stubbed; the word at `c1+0x138` points at 288 random bytes less `0` (placed); the word at `c2+0x10+a[2]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 1 answers -1..3 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_get_town") else {
        return;
    };
    let obj = super::Obj {
        name: "get_town",
        full: "BuildTypeData::get_town@00639b90",
        cells: 3,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::world::{Pos, tile};
            // A footprint `+0x234` by `+0x238` centred on tile `(a[0], a[1])`
            // (the port's `tile_corner_of`), every tile of it on the map
            // inside a city's radius (`p[0]` the masks, `c[1]` the world);
            // then `find_town_at` (`a[3]`) or `find_city_at` (`a[4]`) by the
            // flags `+0x2c0`, and the city's `+8` through `p[1]`'s row.
            let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
            let (w, h) = (rd(&c[1], 0x18), rd(&c[1], 0x1c));
            let centre = Pos::new(a[0] * 192 + 96, a[1] * 192 + 96);
            let corner = crate::place::tile_corner_of(xs, ys, centre);
            let mut clear = true;
            for x in corner.x..corner.x + xs {
                for y in corner.y..corner.y + ys {
                    if x >= 0 && y >= 0 && x < w && y < h {
                        let t = ((w * y + x) * 2) as usize;
                        clear &=
                            u16::from_le_bytes([p[0][t], p[0][t + 1]]) & tile::CITY_RADIUS != 0;
                    }
                }
            }
            let found = if !clear {
                -1
            } else if crate::place::wants_town(rd(m, 0x2c0) as u32) {
                a[3]
            } else {
                a[4]
            };
            if found < 0 {
                -1
            } else {
                let rec = &p[2 + found as usize];
                i64::from(i16::from_le_bytes([rec[8], rec[9]]))
            }
        }
    });
}
