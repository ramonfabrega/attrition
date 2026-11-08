//! `ObjectsData::find_town@00658e60` under the emulator against the port (item 2160; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::find_town@00658e60`, run by `tools/emu/sweep_obj.py` on its row `objects_data_find_town` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00c061d4 points at 256 random bytes; callee 0x00658190 (3 args) stubbed; the word at `c0+0x10+a[2]*0x1c` points at 4 random bytes less `s[0]*4` (placed); the word at `p0+0` points at 4 random bytes less `8` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_find_town") else {
        return;
    };
    let obj = super::Obj {
        name: "find_town",
        full: "ObjectsData::find_town@00658e60",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if a[3] < 0 {
            -1
        } else {
            i64::from(i16::from_le_bytes([p[1][0], p[1][1]]))
        }
    });
}
