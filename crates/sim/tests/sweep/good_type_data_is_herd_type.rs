//! `GoodTypeData::is_herd_type@004761e0` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GoodTypeData::is_herd_type@004761e0`, run by `tools/emu/sweep_obj.py` on its row `good_type_data_is_herd_type` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), vtable slot +0x60 (2 args) stubbed; stub 0 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 0 answers 7 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("good_type_data_is_herd_type") else {
        return;
    };
    let obj = super::Obj {
        name: "is_herd_type",
        full: "GoodTypeData::is_herd_type@004761e0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Slot +0x60 (stub 0) asked `(good, 0)` for 0xe, 0x19, 0x17, 0x15, 6, 0x21, 0x1f in
            // turn; the first non-zero answers 1, none answers 0. The answers in call order are
            // a[1..8].
            let mut calls = Vec::new();
            let mut got = 0;
            for (i, g) in [0xe, 0x19, 0x17, 0x15, 6, 0x21, 0x1f]
                .into_iter()
                .enumerate()
            {
                calls.push(format!("m+0x0,{g},0"));
                if a[1 + i] != 0 {
                    got = 1;
                    break;
                }
            }
            log.push(calls.join("/"));
            got
        }
    });
}
