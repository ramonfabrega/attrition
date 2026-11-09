//! `BuildTypeData::count_water@00638ce0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::count_water@00638ce0`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_count_water` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 572 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_count_water") else {
        return;
    };
    let obj = super::Obj {
        name: "count_water",
        full: "BuildTypeData::count_water@00638ce0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (xs, ys) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let mut sim = crate::Sim::new(crate::Tuning::RON, crate::World::new(xs / 4, ys / 4), 2);
            for y in 0..ys {
                for x in 0..xs {
                    let k = ((y * xs + x) * 2) as usize;
                    sim.world.set_tile_mask(
                        crate::Pos::new(x, y),
                        u16::from_le_bytes([p[0][k], p[0][k + 1]]),
                    );
                }
            }
            let ty = sim.add_build_type(crate::cities::build::BuildType::default());
            sim.tables.build_types[ty].x_size = rd(m, 0x234);
            sim.tables.build_types[ty].y_size = rd(m, 0x238);
            i64::from(sim.count_water(ty, crate::Pos::new(a[0], a[1])))
        }
    });
}
