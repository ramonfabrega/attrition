//! `ObjectsData::find_good_at@00659130` under the emulator against the port (item 2187; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `ObjectsData::find_good_at@00659130`, run by `tools/emu/sweep_obj.py` on its row `objects_data_find_good_at_tcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 4 argument(s), the global cell 0x00c061d0 points at 316 random bytes; callee 0x0065bec0 (5 args) stubbed; the word at `c0+0x138` points at 2 random bytes less `(i32(c[0],0x18)*a[1]+a[0])*2` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("objects_data_find_good_at_tcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "find_good_at",
        full: "ObjectsData::find_good_at@00659130",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let w = u16::from_le_bytes([p[0][0], p[0][1]]);
            if w & crate::world::tile::AS_BUILDING == 0 {
                -1
            } else {
                i64::from(a[4])
            }
        }
    });
}
