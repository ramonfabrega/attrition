//! `Region::num_coasts@00680760` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Region::num_coasts@00680760`, run by `tools/emu/sweep_obj.py` on its row `region_num_coasts` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 108 random bytes of object, 1 argument(s), the global cell 0x00c061b8 points at 32 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("region_num_coasts") else {
        return;
    };
    let obj = super::Obj {
        name: "num_coasts",
        full: "Region::num_coasts@00680760",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let r = rd(m, 0x10);
            if r >= 0x40 {
                i64::from((0x41..=0x7e).contains(&r))
            } else {
                let b = u64::from_le_bytes(m[0x64..0x6c].try_into().expect("eight bytes"));
                i64::from(((b >> 1) & ((1u64 << 62) - 1)).count_ones())
            }
        }
    });
}
