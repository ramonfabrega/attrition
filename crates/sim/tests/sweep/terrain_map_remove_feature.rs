//! `TerrainMap::remove_feature@009b39e0` under the emulator against the port (item 2818; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainMap::remove_feature@009b39e0`, run by `tools/emu/sweep_obj.py` on its row `terrain_map_remove_feature` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2512 random bytes of object, 2 argument(s), the global cell 0x00c06188 points at 4 random bytes; callee 0x009b37a0 (2 args) stubbed; callee 0x009b3590 (2 args) stubbed; callee 0x009b3340 (2 args) stubbed; the word at `m+0x9c8` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_map_remove_feature") else {
        return;
    };
    let obj = super::Obj {
        name: "remove_feature",
        full: "TerrainMap::remove_feature@009b39e0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The tile at world-width * y + x of the terrain map decides which remover runs: mountain (3, 6),
            // river (0x18) or cliff (4), each handed `this` and the two coordinates.
            let t = p[0][(rd(&c[0], 0) * a[1] + a[0]) as usize];
            let at = format!("m+0x0,{},{}", a[0], a[1]);
            let call = |hit: bool| if hit { at.clone() } else { "-".to_string() };
            log.push(call(t == 3 || t == 6));
            log.push(call(t == 0x18));
            log.push(call(t == 4));
            0
        }
    });
}
