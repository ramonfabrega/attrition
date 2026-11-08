//! `BuildData::is_unassimilated@0062d470` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::is_unassimilated@0062d470`, run by `tools/emu/sweep_obj.py` on its row `build_data_is_unassimilated` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 128 random bytes of object, 0 argument(s), the word at `0xc061d4` points at 4 random bytes less `0x10+u8(m,9)*0x1c` (placed); the word at `p0+0` points at 4 random bytes less `i16(m,0x72)*4` (placed); the word at `p1+0` points at 96 random bytes less `0` (placed); the word at `m+0x18` points at 4 random bytes less `0x2c0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_is_unassimilated") else {
        return;
    };
    let obj = super::Obj {
        name: "is_unassimilated",
        full: "BuildData::is_unassimilated@0062d470",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // no city, or the city's owner is the building's: assimilated
            if i16::from_le_bytes([m[0x72], m[0x73]]) < 0
                || i32::from(p[2][0x5f] as i8) == i32::from(m[9])
            {
                0
            } else if m[8] & 0x20 != 0 {
                1
            } else {
                i64::from(!((rd(&p[3], 0) as u32) >> 4) & 1)
            }
        }
    });
}
