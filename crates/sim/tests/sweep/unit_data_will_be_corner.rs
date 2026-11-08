//! `UnitData::will_be_corner@00609fa0` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::will_be_corner@00609fa0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_will_be_corner` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the word at `0xc0aec0+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); the word at `p0+0` points at 4 random bytes less `0x18` (placed); the word at `p1+0` points at 4 random bytes less `0x248` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_will_be_corner") else {
        return;
    };
    let obj = super::Obj {
        name: "will_be_corner",
        full: "UnitData::will_be_corner@00609fa0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The unit `units[owner][o]`'s type's `+0x248` against both deltas.
            let w = rd(&p[2], 0);
            let dx = a[0].wrapping_sub(a[2]);
            let dy = a[1].wrapping_sub(a[3]);
            let r = if dx.wrapping_abs() == w && dy.wrapping_abs() == w {
                if dx < 0 && dy < 0 {
                    1
                } else if dx > 0 && dy < 0 {
                    3
                } else if dx > 0 && dy > 0 {
                    5
                } else if dx < 0 && dy > 0 {
                    7
                } else {
                    0
                }
            } else {
                0
            };
            i64::from(r)
        }
    });
}
