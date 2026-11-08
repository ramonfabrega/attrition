//! `UnitData::is_cloaked@0060a6a0` under the emulator against the port (item 2160; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_cloaked@0060a6a0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_cloaked` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 112 random bytes of object, 0 argument(s), callee 0x0060b030 (0 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x2b4` (placed); stub 0 answered from [0, 1, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_cloaked") else {
        return;
    };
    let obj = super::Obj {
        name: "is_cloaked",
        full: "UnitData::is_cloaked@0060a6a0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let t = rd(&p[0], 0);
            if rd(&m[..], 0x68) & 0x800 != 0 || t & 0x4000 != 0 || rd(&m[..], 0x6c) & 0x8000 != 0 {
                1
            } else if t & 0x40000 == 0 {
                0
            } else {
                i64::from(a[0] == 0)
            }
        }
    });
}
