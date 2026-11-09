//! `ObjectData::is_cargo@00653600` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectData::is_cargo@00653600`, run by `tools/emu/sweep_obj.py` on its row `object_data_is_cargo` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0xbc (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x12c (0 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); slot +0x12c laid as 0x00661d80 or its stub, a flag after the answers; slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 5, -1] (90%); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_data_is_cargo") else {
        return;
    };
    let obj = super::Obj {
        name: "is_cargo",
        full: "ObjectData::is_cargo@00653600",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The answers of `this`'s slot `+0xbc` and the type's `+0xc` and
            // `+0x12c`, then whether `+0x12c` and `+0xc` were laid as the base ones.
            let (on_map, unit_ty, cargo) = (a[0], a[1], a[2]);
            let (cargo_base, unit_base) = (a[3] != 0, a[4] != 0);
            let land = rd(&p[0], 0x218) == 0;
            if on_map == 0 {
                0
            } else if !cargo_base {
                i64::from(cargo)
            } else if unit_base {
                i64::from(crate::ai::transport::cargo_type(rd(&p[0], 4), land))
            } else {
                i64::from(unit_ty != 0 && land)
            }
        }
    });
}
