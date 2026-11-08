//! `Cliff::get_vert_offsets@008a4070` under the emulator against the port (item 2588; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Cliff::get_vert_offsets@008a4070`, run by `tools/emu/sweep_obj.py` on its row `cliff_get_vert_offsets` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 8 argument(s), argument 5 names 4 random bytes; argument 6 names 4 random bytes; argument 7 names 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("cliff_get_vert_offsets") else {
        return;
    };
    let obj = super::Obj {
        name: "get_vert_offsets",
        full: "Cliff::get_vert_offsets@008a4070",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            for w in p.iter_mut() {
                w[..4].fill(0);
            }
            0
        }
    });
}
