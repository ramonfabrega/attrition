//! `BuildTypeData::get_stance_type@006396c0` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::get_stance_type@006396c0`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_get_stance_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), callee 0x00639de0 (0 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_get_stance_type") else {
        return;
    };
    let obj = super::Obj {
        name: "get_stance_type",
        full: "BuildTypeData::get_stance_type@006396c0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // Not a training building (`a[0]`) is `-1`; the city slot (`a[1]`) is 1 (worker), the fort slot
        // (`a[2]`) 2 (caster); the type word `m+4` 0x1ae/0x1af is 3 (packer), 0x1b4/0x1a4/0x1bf/0x208
        // is -1, any other 0 (item 2844). The slots are stubs here, never `is_city`/`is_fort`.
        let id = rd(m, 4);
        if a[0] == 0 {
            -1
        } else if a[1] != 0 {
            1
        } else if a[2] != 0 {
            2
        } else if id == 0x1ae || id == 0x1af {
            3
        } else if id == 0x1b4 || id == 0x1a4 || id == 0x1bf || id == 0x208 {
            -1
        } else {
            0
        }
    });
}
