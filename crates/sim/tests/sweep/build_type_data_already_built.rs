//! `BuildTypeData::already_built@0063ce10` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::already_built@0063ce10`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_already_built` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 1696 random bytes; vtable slot +0x1c (0 args) stubbed; slot +0x1c laid as 0x00470780 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_already_built") else {
        return;
    };
    let obj = super::Obj {
        name: "already_built",
        full: "BuildTypeData::already_built@0063ce10",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let id = rd(m, 4);
            let yes = if a[1] != 0 {
                (0x20e..=0x21e).contains(&id)
            } else {
                a[0] != 0
            };
            if yes {
                i64::from(rd(&c[0], (id * 4 - 0x220) as usize))
            } else {
                0
            }
        }
    });
}
