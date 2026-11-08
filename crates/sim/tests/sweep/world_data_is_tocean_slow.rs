//! `WorldData::is_tocean_slow@006b2400` under the emulator against the port (item 2122; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::is_tocean_slow@006b2400`, run by `tools/emu/sweep_obj.py` on its row `world_data_is_tocean_slow` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 320 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c0+0x134` points at 4 random bytes less `((i32(p[1],0)>>2)*i32(c[0],0)+(i32(p[0],0)>>2))*0x1c` (placed); the word at `c0+0x138` points at 4 random bytes less `(i32(c[0],0x18)*i32(p[1],0)+i32(p[0],0))*2` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_is_tocean_slow") else {
        return;
    };
    let obj = super::Obj {
        name: "is_tocean_slow",
        full: "WorldData::is_tocean_slow@006b2400",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let f = u16::from_le_bytes([p[2][0], p[2][1]]);
            let l = p[2][2] as i8;
            if f & 0x100 == 0 && (l == 2 || l == 1) {
                1
            } else {
                i64::from(p[3][0] & 0x30 == 0x20)
            }
        }
    });
}
