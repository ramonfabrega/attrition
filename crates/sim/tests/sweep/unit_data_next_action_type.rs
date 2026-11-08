//! `UnitData::next_action_type@006081f0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::next_action_type@006081f0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_next_action_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), callee 0x00608230 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; stub 0's answer points at 16 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_next_action_type") else {
        return;
    };
    let obj = super::Obj {
        name: "next_action_type",
        full: "UnitData::next_action_type@006081f0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `get_next_action` asked of `this`; its answer, non-null, takes the
            // tail `jmp` through its own slot +0x10 (item 2281).
            log.push("m+0x0".into());
            log.push(if a[0] != 0 {
                "p0+0x0".into()
            } else {
                "-".into()
            });
            if a[0] != 0 { i64::from(a[1]) } else { 0 }
        }
    });
}
