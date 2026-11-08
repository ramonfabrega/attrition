//! `UnitData::can_garrison@006099a0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::can_garrison@006099a0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_can_garrison` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 48 random bytes of object, 2 argument(s), vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x0061d7d0 (1 args) stubbed; the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p2+0` points at 48 random bytes less `0` (placed); the word at `p2+4` points at 48 random bytes less `0` (placed); the word at `p3+0` points at 48 random bytes less `0` (placed); the word at `p3+4` points at 48 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `4` (placed); the word at `p5+0x18` points at 8 random bytes less `4` (placed); the word at `p6+0x18` points at 8 random bytes less `4` (placed); the word at `p7+0x18` points at 8 random bytes less `4` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); the stubbed vtable laid at `p4+0`; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x1c apart at `@0xc0ab84`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_can_garrison") else {
        return;
    };
    let obj = super::Obj {
        name: "can_garrison",
        full: "UnitData::can_garrison@006099a0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The object a[0] of leader a[1]'s list (the table at 0xc0ab84, rows 0x1c
            // apart): `p[4 + 2 * a[1] + a[0]]`, its record `p[8 + ...]`.
            let k = (2 * a[1] + a[0]) as usize;
            if p[4 + k][8] & 1 != 0 && a[2] != 0 && a[3] != 0 {
                log.push(format!("p12+0x0,{}", rd(&p[8 + k], 0)));
                i64::from(a[4])
            } else {
                log.push("-".into());
                0
            }
        }
    });
}
