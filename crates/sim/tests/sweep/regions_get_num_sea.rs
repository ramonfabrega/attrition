//! `Regions::get_num_sea@0067eed0` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Regions::get_num_sea@0067eed0`, run by `tools/emu/sweep_obj.py` on its row `regions_get_num_sea` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c061b8 points at 64 random bytes; the word at `c0+0x10` points at 17408 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("regions_get_num_sea") else {
        return;
    };
    let obj = super::Obj {
        name: "get_num_sea",
        full: "Regions::get_num_sea@0067eed0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(
            (64..127usize)
                .filter(|k| rd(&p[0], k * 0x88 + 0x14) != 0)
                .count() as i32,
        )
    });
}
