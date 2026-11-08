//! `Unit::do_gather@005fce20` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_gather@005fce20`, run by `tools/emu/sweep_obj.py` on its row `unit_do_gather` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 152 random bytes of object, 7 argument(s), callee 0x00609180 (10 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_gather") else {
        return;
    };
    let obj = super::Obj {
        name: "do_gather",
        full: "Unit::do_gather@005fce20",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let f = rd(m, 0x68);
            wr(m, 0x68, if a[4] != 0 { f | 0x20 } else { f & !0x20 });
            i64::from(a[7])
        }
    });
}
