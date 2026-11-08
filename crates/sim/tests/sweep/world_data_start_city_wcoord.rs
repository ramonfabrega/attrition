//! `WorldData::start_city_wcoord@006b30e0` under the emulator against the port (item 2149; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `WorldData::start_city_wcoord@006b30e0`, run by `tools/emu/sweep_obj.py` on its row `world_data_start_city_wcoord` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 32 random bytes; the global cell 0x00c06188 points at 256 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; the word at `c1+0xf8` points at 4 random bytes less `((i32(c[0],0)*i32(p[1],0)+i32(p[0],0)+2**31)%2**32-2**31)>>3` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("world_data_start_city_wcoord") else {
        return;
    };
    let obj = super::Obj {
        name: "start_city_wcoord",
        full: "WorldData::start_city_wcoord@006b30e0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let i = rd(&c[0], 0)
                .wrapping_mul(rd(&p[1], 0))
                .wrapping_add(rd(&p[0], 0));
            i64::from((p[2][0] >> (i & 7)) & 1)
        }
    });
}
