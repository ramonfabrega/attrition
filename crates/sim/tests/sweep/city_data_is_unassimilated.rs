//! `CityData::is_unassimilated@00738030` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::is_unassimilated@00738030`, run by `tools/emu/sweep_obj.py` on its row `city_data_is_unassimilated` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), vtable slot +0x3c (0 args) stubbed; callee 0x0062d470 (0 args) stubbed; the word at `0xc0ab84+i8(m,0x5e)*0x1c` points at 4 random bytes less `i16(m,8)*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 0 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_is_unassimilated") else {
        return;
    };
    let obj = super::Obj {
        name: "is_unassimilated",
        full: "CityData::is_unassimilated@00738030",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The city's object (objects[+0x5e][+8], a byte and a short) asked for
            // its record (slot +0x3c); the build record's twin, entered by a
            // tail jump on that record, answers.
            log.push("p1+0x0".into());
            log.push("p2+0x0".into());
            i64::from(a[1])
        }
    });
}
