//! `BuildData::can_make@00472430` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::can_make@00472430`, run by `tools/emu/sweep_obj.py` on its row `build_data_can_make` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 2 argument(s), vtable slot +0x198 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_can_make") else {
        return;
    };
    let obj = super::Obj {
        name: "can_make",
        full: "BuildData::can_make@00472430",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // A forwarder: a tail jump into this's slot +0x198 (stub 0) with both arguments.
            log.push(format!("m+0x0,{},{}", a[0], a[1]));
            i64::from(a[2])
        }
    });
}
