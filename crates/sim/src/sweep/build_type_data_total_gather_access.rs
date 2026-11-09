//! `BuildTypeData::total_gather_access@00636500` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::total_gather_access@00636500`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_total_gather_access` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), argument 0 names 20 random bytes; callee 0x006b4e50 (5 args) stubbed; the word at `p0+0x10` points at 80 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_total_gather_access") else {
        return;
    };
    let obj = super::Obj {
        name: "total_gather_access",
        full: "BuildTypeData::total_gather_access@00636500",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = rd(&p[0], 4);
            (0..n.max(0) as usize)
                .filter(|&i| {
                    let t = crate::Pos::new(rd(&p[1], 8 * i), rd(&p[1], 8 * i + 4));
                    a[4] != 0 && crate::economy::gather::beyond_access_ring(a[1], a[2], t)
                })
                .count() as i64
        }
    });
}
