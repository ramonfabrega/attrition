//! `World::set_started_at@006b4570` under the emulator against the port (item 2187; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_started_at@006b4570`, run by `tools/emu/sweep_obj.py` on its row `world_set_started_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 316 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x138` points at 2 random bytes less `(i32(p[1],0)*i32(c[0],0x18)+i32(p[0],0))*2` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_started_at") else {
        return;
    };
    let obj = super::Obj {
        name: "set_started_at",
        full: "World::set_started_at@006b4570",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let w = u16::from_le_bytes([p[2][0], p[2][1]]);
            let w = if a[2] != 0 {
                w | crate::world::tile::PLACED
            } else {
                w & !crate::world::tile::PLACED
            };
            p[2][..2].copy_from_slice(&w.to_le_bytes());
            0
        }
    });
}
