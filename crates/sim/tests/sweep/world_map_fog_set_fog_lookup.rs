//! `WorldMapFog::set_fog_lookup@008d1300` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldMapFog::set_fog_lookup@008d1300`, run by `tools/emu/sweep_obj.py` on its row `world_map_fog_set_fog_lookup` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00e8a894 points at 64 random bytes; 8 random bytes at 0x00e8a79c; argument 0 names 4 random bytes; argument 1 names 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_map_fog_set_fog_lookup") else {
        return;
    };
    let obj = super::Obj {
        name: "set_fog_lookup",
        full: "WorldMapFog::set_fog_lookup@008d1300",
        cells: 2,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // set_fog_lookup: the cell (x, y) of a (xs+2) x (ys+2) byte table, which carries a one-cell border; a cell on the
            // edge of the map also writes the border cells beside it, the corners included.
            let _ = &m;
            let (xs, ys) = (rd(&c[1], 0), rd(&c[1], 4));
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let v = a[2] as u8;
            let stride = xs.wrapping_add(2);
            let t = &mut c[0];
            let mut put = |i: i32| t[i as usize] = v;
            put(y
                .wrapping_add(1)
                .wrapping_mul(stride)
                .wrapping_add(x)
                .wrapping_add(1));
            if y == 0 {
                put(x.wrapping_add(1));
                if x == 0 {
                    put(0);
                }
                if x == xs - 1 {
                    put(xs.wrapping_add(2));
                }
            }
            if y == ys - 1 {
                put(y
                    .wrapping_add(2)
                    .wrapping_mul(stride)
                    .wrapping_add(x)
                    .wrapping_add(1));
                if x == 0 {
                    put(y.wrapping_add(2).wrapping_mul(stride));
                }
                if x == xs - 1 {
                    put(y
                        .wrapping_add(2)
                        .wrapping_mul(stride)
                        .wrapping_add(x)
                        .wrapping_add(2));
                }
            }
            if x == 0 {
                put(y.wrapping_add(1).wrapping_mul(stride));
            }
            if x == xs - 1 {
                put(y
                    .wrapping_add(1)
                    .wrapping_mul(stride)
                    .wrapping_add(x)
                    .wrapping_add(2));
            }
            0
        }
    });
}
