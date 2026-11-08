//! `WonderData::get_value@0073c7a0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WonderData::get_value@0073c7a0`, run by `tools/emu/sweep_obj.py` on its row `wonder_data_get_value` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), vtable slot +0x14c (0 args) stubbed; vtable slot +0x118 (0 args) stubbed; the word at `0xc0ab84+i8(m,0xd)*0x1c` points at 4 random bytes less `i16(m,2)*4` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 4 random bytes less `0` (placed); slot +0x14c laid as 0x0046cf20 or its stub, a flag after the answers; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("wonder_data_get_value") else {
        return;
    };
    let obj = super::Obj {
        name: "get_value",
        full: "WonderData::get_value@0073c7a0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The object `objects[who][o]`; its slot +0x14c laid as
            // `ObjectData::get_wonder_value` hands the tail `jmp` to its type's
            // slot +0x118, else the slot itself is the tail (item 2281).
            if a[2] != 0 {
                log.push("-".into());
                log.push("p2+0x0".into());
                i64::from(a[1])
            } else {
                log.push("p1+0x0".into());
                log.push("-".into());
                i64::from(a[0])
            }
        }
    });
}
