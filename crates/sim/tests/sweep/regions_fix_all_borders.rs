//! `Regions::fix_all_borders@0067f7d0` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Regions::fix_all_borders@0067f7d0`, run by `tools/emu/sweep_obj.py` on its row `regions_fix_all_borders` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061b8 points at 32 random bytes; the word at `c0+0x10` points at 8704 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("regions_fix_all_borders") else {
        return;
    };
    let obj = super::Obj {
        name: "fix_all_borders",
        full: "Regions::fix_all_borders@0067f7d0",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Every one of the 64 regions' rows (0x88 bytes, the list +0x10 of the record cell
            // 0xc061b8 names) has its word +0x2c cleared.
            for i in 0..0x40 {
                wr(&mut p[0], 0x2c + i * 0x88, 0);
            }
            0
        }
    });
}
