//! `BuildData::get_shot@0062dd90` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::get_shot@0062dd90`, run by `tools/emu/sweep_obj.py` on its row `build_data_get_shot` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `0xc061e0` points at 4 random bytes less `0x6eb8+u8(m,9)*0x6eec` (placed); the word at `p1+0` points at 4 random bytes less `0xdc` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_get_shot") else {
        return;
    };
    let obj = super::Obj {
        name: "get_shot",
        full: "BuildData::get_shot@0062dd90",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        match rd(&p[0], 4) {
            0x1b9 | 0x1bd => 1,
            0x1ba | 0x20b | 0x20c => 2,
            0x1be => 4,
            0x209 | 0x20a => 0,
            0x20d => 5,
            _ => {
                let v = rd(&p[2], 0) ^ 0x62766;
                if v > 4 { 2 } else { i64::from(v > 2) }
            }
        }
    });
}
