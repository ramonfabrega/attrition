//! `GroupData::get_num_const@0070d000` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::get_num_const@0070d000`, run by `tools/emu/sweep_obj.py` on its row `group_data_get_num_const` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2272 random bytes of object, 0 argument(s), the word at `0xc0ab84+u8(m,0x4a)*0x1c` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 12 random bytes less `0` (placed); the word at `p0+4` points at 12 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_get_num_const") else {
        return;
    };
    let obj = super::Obj {
        name: "get_num_const",
        full: "GroupData::get_num_const@0070d000",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(m, 0xc);
            let mut k = n.max(0);
            for i in (0..n.max(0)).rev() {
                let o = 0x8cc + 2 * i as usize;
                let idx = i16::from_le_bytes([m[o], m[o + 1]]) as usize;
                if p[1 + idx][8] & 1 == 0 {
                    k -= 1;
                }
            }
            i64::from(k)
        }
    });
}
