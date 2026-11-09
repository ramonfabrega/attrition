//! `UnitData::get_order@0060b030` under the emulator against the port (item 2253; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::get_order@0060b030`, run by `tools/emu/sweep_obj.py` on its row `unit_data_get_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 224 random bytes of object, 0 argument(s), the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_get_order") else {
        return;
    };
    let obj = super::Obj {
        name: "get_order",
        full: "UnitData::get_order@0060b030",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if rd(m, 0xdc) == 0 {
                0
            } else {
                let nxt = rd(&p[0], 4);
                let v = rd(&p[1], 8);
                let b = p[1][0xc];
                wr(m, 0xd4, nxt);
                wr(m, 0xcc, v);
                m[0xd0] = b;
                i64::from(v)
            }
        }
    });
}
