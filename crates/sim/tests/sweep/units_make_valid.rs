//! `Units::make_valid@0061a960` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Units::make_valid@0061a960`, run by `tools/emu/sweep_obj.py` on its row `units_make_valid` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), 32 random bytes at 0x00c0aecc; vtable slot +0x0 (1 args) stubbed; the stubbed vtable laid at `c0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("units_make_valid") else {
        return;
    };
    let obj = super::Obj {
        name: "make_valid",
        full: "Units::make_valid@0061a960",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Leader 1's units list (the row 0x1c on from 0xc0aeb0: its grow callback through the word at +0, size at +4, count at +8, the step short at +0xc) is made valid for n+1: short of it, the callback is called with the larger of the shortfall and the step (the count when the step is negative); the size word is then the larger of it and n+1.
            let need = a[1] + 1;
            let count = rd(&c[0], 8);
            let s = i32::from(i16::from_le_bytes([c[0][0xc], c[0][0xd]]));
            let mut calls = Vec::new();
            if count < need {
                let bound = if s >= 0 { s } else { count };
                calls.push(format!("c0+0x0,{}", (need - count).max(bound)));
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            if rd(&c[0], 4) < need {
                wr(&mut c[0], 4, need);
            }
            0
        }
    });
}
