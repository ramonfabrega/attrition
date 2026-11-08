//! `UnitData::is_here@0060a0c0` under the emulator against the port (item 2633; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_here@0060a0c0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_here` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 4096 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `m+0x18` points at 592 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_here") else {
        return;
    };
    let obj = super::Obj {
        name: "is_here",
        full: "UnitData::is_here@0060a0c0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        let t = rd(&p[2], 0x248);
        let d = |v: i32| rd(&c[0], (((v ^ 0x63637) >> 4) as usize) * 4);
        if t == 0 || t < (rd(&p[0], 0) - d(rd(m, 0x10))).wrapping_abs() {
            0
        } else {
            i64::from((rd(&p[1], 0) - d(rd(m, 0x14))).wrapping_abs() <= t)
        }
    });
}
