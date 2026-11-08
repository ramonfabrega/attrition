//! `World::set_road_at@006b43b0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_road_at@006b43b0`, run by `tools/emu/sweep_obj.py` on its row `world_set_road_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 316 random bytes of object, 5 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x008955d0 (3 args) stubbed; callee 0x008954d0 (4 args) stubbed; the word at `m+0x138` points at 128 random bytes less `0` (placed); the word at `m+0x134` points at 112 random bytes less `0` (placed); the global cell 0x00c06188 holds `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_road_at") else {
        return;
    };
    let obj = super::Obj {
        name: "set_road_at",
        full: "World::set_road_at@006b43b0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // An 8 x 8-tile world (2 x 2 cells), `this` and `GameAccess::world`
            // one object: the tiles' masks from p[2], each cell's flags word (`+0`)
            // from p[3]. The callees are stubs, so the port is entered with
            // `p5 = 1`: the mesh's half (`road_cleared`, `road_added`) is theirs.
            let mut sim = crate::Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 2);
            let tile = |k: usize| crate::Pos::new((k % 8) as i32, (k / 8) as i32);
            let cell = |i: usize| crate::world::Cell::new((i % 2) as i32, (i / 2) as i32);
            for k in 0..64 {
                sim.world
                    .set_tile_mask(tile(k), u16::from_le_bytes([p[2][2 * k], p[2][2 * k + 1]]));
            }
            for i in 0..4 {
                let mut d = sim.world.cell_data(cell(i));
                d.flags = u16::from_le_bytes([p[3][i * 0x1c], p[3][i * 0x1c + 1]]);
                sim.world.set_cell_data(cell(i), d);
            }
            sim.world_set_road_at(
                crate::Pos::new(rd(&p[0], 0), rd(&p[1], 0)),
                a[2] != 0,
                a[3],
                1,
            );
            for k in 0..64 {
                p[2][2 * k..2 * k + 2].copy_from_slice(&sim.world.tile_mask(tile(k)).to_le_bytes());
            }
            for i in 0..4 {
                p[3][i * 0x1c..i * 0x1c + 2]
                    .copy_from_slice(&sim.world.cell_data(cell(i)).flags.to_le_bytes());
            }
            0
        }
    });
}
