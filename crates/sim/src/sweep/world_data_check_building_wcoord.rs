//! `WorldData::check_building_wcoord@006b26e0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::check_building_wcoord@006b26e0`, run by `tools/emu/sweep_obj.py` on its row `world_data_check_building_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 8 argument(s), the global cell 0x00c06188 points at 320 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b27f0 (5 args) stubbed; the word at `c0+0x134` points at 28 random bytes less `(i32(c[0],0)*i32(p[1],0)+i32(p[0],0))*0x1c` (placed); stub 0 answered from [0, 0, 1, 1, 2, 2, 3, 3, 4, 5, -1] (90%); stub 0 answers 49 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_check_building_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "check_building_wcoord",
        full: "WorldData::check_building_wcoord@006b26e0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let cell = &p[2];
            let o = cell[0xf] as i8;
            let owner = (o >= 0).then_some(o as u8);
            let flags = u16::from_le_bytes([cell[0], cell[1]]);
            if crate::ai::ai_place::wcoord_cell_refused(owner, a[2] as u8, flags, cell[0x11]) {
                0
            } else {
                // `space_at_corner` answers each corner in turn (`--seq`, a[9..58]).
                let mut k = 0usize;
                i64::from(crate::ai::ai_place::best_corner(
                    a[4],
                    a[5],
                    a[6],
                    |_, _| {
                        let v = a[9 + k.min(48)];
                        k += 1;
                        v
                    },
                ))
            }
        }
    });
}
