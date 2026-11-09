//! `CityData::ready_to_upgrade@00736480` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::ready_to_upgrade@00736480`, run by `tools/emu/sweep_obj.py` on its row `city_data_ready_to_upgrade` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), callee 0x006e33a0 (2 args) stubbed; callee 0x007383c0 (1 args) stubbed; the word at `0xc0ab84+i8(m,0x5e)*0x1c` points at 4 random bytes less `i16(m,8)*4` (placed); the word at `p0+0` points at 28 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_ready_to_upgrade") else {
        return;
    };
    let obj = super::Obj {
        name: "ready_to_upgrade",
        full: "CityData::ready_to_upgrade@00736480",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let id = rd(&p[2], 4);
            if id == 0x1a0 || id == 0x213 || a[0] == 0 {
                0
            } else {
                i64::from(a[1])
            }
        }
    });
}
