//! `Army::count@006f9120` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Army::count@006f9120`, run by `tools/emu/sweep_obj.py` on its row `army_count` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 160 random bytes of object, 2 argument(s), callee 0x00711720 (3 args) stubbed; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("army_count") else {
        return;
    };
    let obj = super::Obj {
        name: "count",
        full: "Army::count@006f9120",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let n = i32::from(i16::from_le_bytes([m[0x96], m[0x97]]));
            let mut s = 0i32;
            for i in 0..n.max(0) {
                if rd(m, 0x54 + 4 * i as usize) >= 0 {
                    s = s.wrapping_add(a[2]);
                }
            }
            i64::from(s)
        }
    });
}
