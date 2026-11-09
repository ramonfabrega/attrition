//! `BuildTypeData::non_friendly_territory@006389c0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::non_friendly_territory@006389c0`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_non_friendly_territory` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 576 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 320 random bytes; the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_non_friendly_territory") else {
        return;
    };
    let obj = super::Obj {
        name: "non_friendly_territory",
        full: "BuildTypeData::non_friendly_territory@006389c0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::world::tile;
            // The footprint `+0x234` by `+0x238` from `(a[0], a[1])`, the
            // world's `c[0]` (cells wide `+0`, tiles `+0x18` by `+0x1c`), its
            // cell records `p[8]` and tile masks `p[9]`, the leaders `p[0..8]`.
            let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
            let who = a[2] as usize;
            let (cw, w, h) = (rd(&c[0], 0), rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let me = rd(&p[who], 0);
            let mut r = 0;
            'walk: for x in a[0]..a[0] + xs {
                for y in a[1]..a[1] + ys {
                    if x < 0 || y < 0 || x >= w || y >= h {
                        continue;
                    }
                    let t = ((w * y + x) * 2) as usize;
                    let mask = u16::from_le_bytes([p[9][t], p[9][t + 1]]);
                    if mask & tile::SURFACE == tile::SURFACE_OCEAN {
                        continue;
                    }
                    let o =
                        i32::from(p[8][(((y >> 2) * cw + (x >> 2)) * 0x1c + 0xf) as usize] as i8);
                    let (to, from) = if o < 0 {
                        (-1, -1)
                    } else {
                        (
                            rd(&p[who], (0x6c + 4 * o) as usize),
                            rd(&p[o as usize], (0x6c + 4 * me) as usize),
                        )
                    };
                    let rank = crate::cities::place::ground_rank(
                        o,
                        me,
                        (to == 2, from == 2),
                        (to == 0, from == 0),
                    );
                    if rank == 3 {
                        r = 3;
                        break 'walk;
                    }
                    r = r.max(rank);
                }
            }
            i64::from(r)
        }
    });
}
