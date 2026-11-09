//! `CliffsOut::get_mining_data@008a8590` under the emulator against the port (item 2271; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CliffsOut::get_mining_data@008a8590`, run by `tools/emu/sweep_obj.py` on its row `cliffs_out_get_mining_data` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 4 random bytes; the global cell 0x00c06218 points at 27316 random bytes; the global cell 0x00e88f00 points at 32 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c1+0x6ab0` points at 4 random bytes less `(i32(p[1],0)*i32(c[0],0)+i32(p[0],0))*4` (placed); the word at `p2+0` points at 4 random bytes less `0x6a` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("cliffs_out_get_mining_data") else {
        return;
    };
    let obj = super::Obj {
        name: "get_mining_data",
        full: "CliffsOut::get_mining_data@008a8590",
        cells: 3,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let s = rd(&p[3], 0) as i16;
            if s == -1 {
                0
            } else {
                i64::from(rd(&c[2], 4 * s as usize))
            }
        }
    });
}
