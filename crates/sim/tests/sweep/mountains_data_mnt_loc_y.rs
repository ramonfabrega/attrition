//! `MountainsData::mnt_loc_y@0089d080` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `MountainsData::mnt_loc_y@0089d080`, run by `tools/emu/sweep_obj.py` on its row `mountains_data_mnt_loc_y` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 512 random bytes of object, 2 argument(s), `p` windows in order: p0 argument 0, p1 the place; 2 random bytes at 0x00ee1244; argument 0 names 4 random bytes; the word at `m+0x94` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("mountains_data_mnt_loc_y") else {
        return;
    };
    let obj = super::Obj {
        name: "mnt_loc_y",
        full: "MountainsData::mnt_loc_y@0089d080",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The out `WCoord` (argument 0) gets the array's word at the index (the bounds report is muted by
        // `ignore_always`, so an index past the count still reads); the answer is the out pointer itself.
        let v = rd(&p[1], 4 * a[1] as usize);
        wr(&mut p[0], 0, v);
        i64::from(a[0])
    });
}
