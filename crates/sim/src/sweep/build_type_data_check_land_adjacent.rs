//! `BuildTypeData::check_land_adjacent@00638ed0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::check_land_adjacent@00638ed0`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_check_land_adjacent` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 576 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 320 random bytes; callee 0x006e1370 (1 args) stubbed; the word at `c0+0x134` points at 256 random bytes less `0` (placed); the word at `c0+0x138` points at 288 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_check_land_adjacent") else {
        return;
    };
    let obj = super::Obj {
        name: "check_land_adjacent",
        full: "BuildTypeData::check_land_adjacent@00638ed0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::world::Pos;
            // The ring one tile round the footprint `+0x234` by `+0x238`
            // from `(a[0], a[1])`, less its four corners; `who` is `a[2]`;
            // the world `c[0]`, its cell records `p[8]` and tile masks
            // `p[9]`, the leaders `p[0..8]`; `has_tribe_bonus(0x13)`'s
            // answer is `a[3]`.
            let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
            let (x0, y0, who) = (a[0], a[1], a[2]);
            let (cw, w, h) = (rd(&c[0], 0), rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let (x1, y1) = (x0 + xs, y0 + ys);
            let mut r = 2;
            'walk: for x in x0 - 1..=x1 {
                for y in y0 - 1..=y1 {
                    let inside = crate::cities::place::footprint_covers(
                        Pos::new(x0, y0),
                        xs,
                        ys,
                        Pos::new(x, y),
                    );
                    let corner = (x == x0 - 1 || x == x1) && (y == y0 - 1 || y == y1);
                    if inside || corner || x < 0 || y < 0 || x >= w || y >= h {
                        continue;
                    }
                    let t = ((w * y + x) * 2) as usize;
                    if !crate::cities::place::adjacent_land(u16::from_le_bytes([
                        p[9][t],
                        p[9][t + 1],
                    ])) {
                        continue;
                    }
                    r = 1;
                    if who < 0 {
                        r = 0;
                        break 'walk;
                    }
                    let o =
                        i32::from(p[8][(((y >> 2) * cw + (x >> 2)) * 0x1c + 0xf) as usize] as i8);
                    let me = rd(&p[who as usize], 0);
                    let friendly = o >= 0
                        && crate::cities::place::friendly_ground(
                            o,
                            me,
                            rd(&p[who as usize], (0x6c + 4 * o) as usize) == 2,
                            rd(&p[o as usize], (0x6c + 4 * me) as usize) == 2,
                        );
                    if friendly || (o == -1 && a[3] != 0) {
                        r = 0;
                        break 'walk;
                    }
                }
            }
            i64::from(r)
        }
    });
}
