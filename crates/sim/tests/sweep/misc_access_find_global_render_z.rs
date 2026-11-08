//! `MiscAccess::find_global_render_z@005837f0` under the emulator against the port (item 2513; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MiscAccess::find_global_render_z@005837f0`, run by `tools/emu/sweep_obj.py` on its row `misc_access_find_global_render_z` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c06218 points at 4 random bytes; callee 0x00866560 (4 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in -524288..524288, not the object, the row's last; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("misc_access_find_global_render_z") else {
        return;
    };
    let obj = super::Obj {
        name: "find_global_render_z",
        full: "MiscAccess::find_global_render_z@005837f0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push(format!("c0+0x0,{},{},{},1", a[3], a[0], a[1]));
            i64::from(a[3])
        }
    });
}
