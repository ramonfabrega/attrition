//! `BuildData::num_scholars@0062d430` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::num_scholars@0062d430`, run by `tools/emu/sweep_obj.py` on its row `build_data_num_scholars` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 2 argument(s), vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x0064dda0 (2 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_num_scholars") else {
        return;
    };
    let obj = super::Obj {
        name: "num_scholars",
        full: "BuildData::num_scholars@0062d430",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Slot +0xb8 the base `ObjectData::is` (the flag after the answers) → the type's own
            // slot +0x60 (stub 1, the record at +0x18) answers; else slot +0xb8 (stub 0) does,
            // each asked (0x1a4, 0). Zero → 0; else `count_inside(0x13, 0x34)` (stub 2).
            let based = a[5] != 0;
            log.push(if based {
                "-".into()
            } else {
                "m+0x0,420,0".into()
            });
            log.push(if based {
                "p0+0x0,420,0".into()
            } else {
                "-".into()
            });
            let r = if based { a[3] } else { a[2] };
            if r == 0 {
                log.push("-".into());
                0
            } else {
                log.push("m+0x0,19,52".into());
                i64::from(a[4])
            }
        }
    });
}
