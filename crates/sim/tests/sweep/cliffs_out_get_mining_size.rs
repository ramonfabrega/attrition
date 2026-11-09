//! `CliffsOut::get_mining_size@008a6da0` under the emulator against the port (item 2170; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CliffsOut::get_mining_size@008a6da0`, run by `tools/emu/sweep_obj.py` on its row `cliffs_out_get_mining_size` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 4 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `0xc06218` points at 4 random bytes less `0x6ab0` (placed); the word at `p2+0` points at 4 random bytes less `(i32(p[1],0)*i32(c[0],0)+i32(p[0],0))*4` (placed); the word at `p3+0` points at 4 random bytes less `0x6a` (placed); the word at `0xe88ee4` points at 4 random bytes less `i16(p[4],0)*4` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("cliffs_out_get_mining_size") else {
        return;
    };
    let obj = super::Obj {
        name: "get_mining_size",
        full: "CliffsOut::get_mining_size@008a6da0",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let s = i16::from_le_bytes([p[4][0], p[4][1]]);
            if s == -1 { 0 } else { i64::from(rd(&p[5], 0)) }
        }
    });
}
