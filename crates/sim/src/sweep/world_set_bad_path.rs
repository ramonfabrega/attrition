//! `World::set_bad_path@006b4610` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `World::set_bad_path@006b4610`, run by `tools/emu/sweep_obj.py` on its row `world_set_bad_path` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 316 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x138` points at 2 random bytes less `(i32(p[1],0)*i32(m,0x18)+i32(p[0],0))*2` (placed); the word at `m+0x134` points at 4 random bytes less `((i32(p[1],0)>>2)*i32(m,0)+(i32(p[0],0)>>2))*0x1c+0x12` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_set_bad_path") else {
        return;
    };
    let obj = super::Obj {
        name: "set_bad_path",
        full: "World::set_bad_path@006b4610",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let (w, b) = crate::world::bad_path_step(
                u16::from_le_bytes([p[2][0], p[2][1]]),
                p[3][0],
                a[2] != 0,
            );
            p[2][..2].copy_from_slice(&w.to_le_bytes());
            p[3][0] = b;
            0
        }
    });
}
