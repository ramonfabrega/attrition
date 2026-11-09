//! `UnitData::is_modern_infantry@00607b40` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_modern_infantry@00607b40`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_modern_infantry` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 28 random bytes of object, 0 argument(s), callee 0x006e1370 (1 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; the word at `m+0x18` points at 696 random bytes less `0` (placed); slot +0xec laid as 0x00661ac0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 3, 5, 6, 7, 9, -1] (90%); stub 2 answered from [0, 3, 5, 6, 7, 9, -1] (90%); the stubbed vtable laid at `p0+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_modern_infantry") else {
        return;
    };
    let obj = super::Obj {
        name: "is_modern_infantry",
        full: "UnitData::is_modern_infantry@00607b40",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let f = rd(&p[0], 0x2b4);
            if a[0] != 0 {
                i64::from(f & 0x100)
            } else if f & 0x100 == 0 {
                0
            } else {
                let age = if a[3] != 0 {
                    let v = rd(&p[0], 0x278);
                    if v == -1 { a[2] } else { v }
                } else {
                    a[1]
                };
                i64::from(age >= 6)
            }
        }
    });
}
