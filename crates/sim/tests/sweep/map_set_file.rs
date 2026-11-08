//! `Map::set_file@006a06a0` under the emulator against the port (item 2644; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Map::set_file@006a06a0`, run by `tools/emu/sweep_obj.py` on its row `map_set_file` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 288 random bytes of object, 1 argument(s), the global cell 0x00c06378 points at 32 random bytes; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("map_set_file") else {
        return;
    };
    let obj = super::Obj {
        name: "set_file",
        full: "Map::set_file@006a06a0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push(format!("m+0x114,{}", rd(&c[0], 0x10).wrapping_add(0x170fc)));
            log.push(format!("m+0x114,{}", a[0]));
            0
        }
    });
}
