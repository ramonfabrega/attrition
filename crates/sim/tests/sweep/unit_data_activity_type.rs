//! `UnitData::activity_type@00608210` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::activity_type@00608210`, run by `tools/emu/sweep_obj.py` on its row `unit_data_activity_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x00608370 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; stub 0's answer points at 32 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_activity_type") else {
        return;
    };
    let obj = super::Obj {
        name: "activity_type",
        full: "UnitData::activity_type@00608210",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `get_activity` (stub 0) answers the order; null answers NONE (0), else the order's slot
            // +0x10 (stub 1), entered by a tail jump, answers.
            log.push("m+0x0".into());
            if a[0] == 0 {
                log.push("-".into());
                0
            } else {
                log.push("p0+0x0".into());
                i64::from(a[1])
            }
        }
    });
}
