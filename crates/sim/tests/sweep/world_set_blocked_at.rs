//! `World::set_blocked_at@006b4900` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_blocked_at@006b4900`, run by `tools/emu/sweep_obj.py` on its row `world_set_blocked_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 316 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b43b0 (5 args) stubbed; the word at `m+0x138` points at 128 random bytes less `0` (placed); the word at `m+0x134` points at 112 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_blocked_at") else {
        return;
    };
    let obj = super::Obj {
        name: "set_blocked_at",
        full: "World::set_blocked_at@006b4900",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // An 8 x 8-tile world (2 x 2 cells): the tiles' masks from p[2], the
            // cells' `blocked`, `bad`, `solid` bytes (`+0x11`..`+0x13`) from p[3].
            let mut w = crate::World::new(2, 2);
            for y in 0..8 {
                for x in 0..8 {
                    let k = ((y * 8 + x) * 2) as usize;
                    w.set_tile_mask(
                        crate::Pos::new(x, y),
                        u16::from_le_bytes([p[2][k], p[2][k + 1]]),
                    );
                }
            }
            for i in 0..4usize {
                let c = crate::world::Cell::new((i % 2) as i32, (i / 2) as i32);
                let mut d = w.cell_data(c);
                d.blocked = p[3][i * 0x1c + 0x11];
                d.bad = p[3][i * 0x1c + 0x12];
                d.solid = p[3][i * 0x1c + 0x13] as i8;
                w.set_cell_data(c, d);
            }
            w.set_blocked_at(crate::Pos::new(rd(&p[0], 0), rd(&p[1], 0)), a[2] != 0);
            for y in 0..8 {
                for x in 0..8 {
                    let k = ((y * 8 + x) * 2) as usize;
                    p[2][k..k + 2]
                        .copy_from_slice(&w.tile_mask(crate::Pos::new(x, y)).to_le_bytes());
                }
            }
            for i in 0..4usize {
                let d = w.cell_data(crate::world::Cell::new((i % 2) as i32, (i / 2) as i32));
                p[3][i * 0x1c + 0x11] = d.blocked;
                p[3][i * 0x1c + 0x12] = d.bad;
                p[3][i * 0x1c + 0x13] = d.solid as u8;
            }
            0
        }
    });
}
