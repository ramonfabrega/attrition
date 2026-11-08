//! `Border::build_border@0087fe00` under the emulator against the port (item 2513; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Border::build_border@0087fe00`, run by `tools/emu/sweep_obj.py` on its row `border_build_border` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 224 random bytes of object, 0 argument(s), callee 0x00880c30 (0 args) stubbed; callee 0x00880960 (0 args) stubbed; callee 0x0087fbb0 (0 args) stubbed; callee 0x0087fd70 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("border_build_border") else {
        return;
    };
    let obj = super::Obj {
        name: "build_border",
        full: "Border::build_border@0087fe00",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            for s in 0..4 {
                log.push(format!("s{s}:m+0x0"));
            }
            m[0xc8] = 0;
            0
        }
    });
}
