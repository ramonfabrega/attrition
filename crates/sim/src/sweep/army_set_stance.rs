//! `Army::set_stance@006f8750` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Army::set_stance@006f8750`, run by `tools/emu/sweep_obj.py` on its row `army_set_stance` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 1 argument(s), the global cell 0x00e85f20 points at 10064 random bytes; callee 0x0070d370 (0 args) stubbed; callee 0x0070d440 (1 args) stubbed; stub 0 answered from [0, 1, 2] (90%); stub 0 answers 16 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("army_set_stance") else {
        return;
    };
    let obj = super::Obj {
        name: "set_stance",
        full: "Army::set_stance@006f8750",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The group slots are `m+0x54` (the count the short `m+0x96`),
        // the groups table `c[0]` (stride `0x9d4`, `num` `+0xc`);
        // `get_stance_type` (stub 0) answers `a[3..]` in call order
        // (`--seq`), `STANCE_COMBAT` being 0, and `action_stance`
        // (stub 1) is recorded with its argument `a[0]`
        // (`--record-order`, item 2341).
        let mut turn = 0usize;
        let mut cx = (m, c, &mut *log);
        crate::army::set_stance_steps(
            &mut cx,
            |(m, _, _)| i16::from_le_bytes([m[0x96], m[0x97]]),
            |(m, _, _), i| rd(m, 0x54 + 4 * i),
            |(_, c, _), g| rd(&c[0], g as usize * 0x9d4 + 0xc),
            |(_, _, log), g| {
                log.push(format!("s0:c0+{:#x}", g as usize * 0x9d4));
                let v = a[3 + turn.min(15)];
                turn += 1;
                v == 0
            },
            |(_, _, log), g, s| log.push(format!("s1:c0+{:#x},{s}", g as usize * 0x9d4)),
            a[0],
        );
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
