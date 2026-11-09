//! `RoadsOut::set_neighbor@00890c30` under the emulator against the port (item 2253; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `RoadsOut::set_neighbor@00890c30`, run by `tools/emu/sweep_obj.py` on its row `roads_out_set_neighbor` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 5 argument(s), callee 0x008744d0 (5 args) stubbed; stub 0's answer points at 8 random bytes less `0` (placed, null a third of the rows); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("roads_out_set_neighbor") else {
        return;
    };
    let obj = super::Obj {
        name: "set_neighbor",
        full: "RoadsOut::set_neighbor@00890c30",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            if a[5] == 0 {
                0
            } else {
                let f = rd(&p[0], 4);
                if a[2] & f != 0 {
                    0
                } else {
                    let g = f | a[2];
                    let g = if a[4] != -1 { !a[4] & g } else { g };
                    wr(&mut p[0], 4, g);
                    1
                }
            }
        }
    });
}
