//! `SubObjectData::is_spellcaster@006623e0` under the emulator against the port (item 2187; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SubObjectData::is_spellcaster@006623e0`, run by `tools/emu/sweep_obj.py` on its row `sub_object_data_is_spellcaster` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 28 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; callee 0x00639840 (1 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x2b8` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("sub_object_data_is_spellcaster") else {
        return;
    };
    let obj = super::Obj {
        name: "is_spellcaster",
        full: "SubObjectData::is_spellcaster@006623e0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if a[0] != 0 {
            i64::from(rd(&p[0], 0) & 2)
        } else if a[1] != 0 {
            i64::from(a[2])
        } else {
            0
        }
    });
}
