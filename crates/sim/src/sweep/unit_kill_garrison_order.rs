//! `Unit::kill_garrison_order@005e2bd0` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::kill_garrison_order@005e2bd0`, run by `tools/emu/sweep_obj.py` on its row `unit_kill_garrison_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), the global cell 0x00c0618c points at 64 random bytes; 256 random bytes at 0x00e90000; 256 random bytes at 0x00e90200; 256 random bytes at 0x00e90400; 16 random bytes at 0x00e91000; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xa8 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x0`; stub 1 answers from its own `ecx`: the word at `+0x0`, or `ecx+0x0` when the word at `+0x0` is set; stub 2 answers from its own `ecx`: the word at `+0x40`, or `ecx+0x40` when the word at `+0x3c` is set; stub 3 answers from its own `ecx`: the word at `+0x8`; the stubbed vtable laid at `m+0x40` as well as at `this`; the stubbed vtable laid at `c1+0` as well as at `this`; the stubbed vtable laid at `c2+0` as well as at `this`; the stubbed vtable laid at `c3+0` as well as at `this`; the stubbed vtable laid at `c1+0x40` as well as at `this`; the stubbed vtable laid at `c2+0x40` as well as at `this`; the stubbed vtable laid at `c3+0x40` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_kill_garrison_order") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_garrison_order",
        full: "Unit::kill_garrison_order@005e2bd0",
        cells: 5,
        calls: true,
        eax: false,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // Node 0 is `this`, node k the unit window `c[k]` at
        // `UNITS + 0x200 (k - 1)`; `c[4]` is the object table
        // (`o_down` indexes it), `c[0]` the global cell.
        const UNITS: i32 = 0xe9_0000;
        fn win<'a>(m: &'a [u8], c: &'a [Vec<u8>], k: usize) -> &'a [u8] {
            if k == 0 { m } else { &c[k] }
        }
        let mut cx = (m, c, &mut *log);
        let tag = |k: usize| {
            if k == 0 {
                "m".to_string()
            } else {
                format!("c{k}")
            }
        };
        crate::orders::kill_garrison_order_steps(
            &mut cx,
            0usize,
            16,
            |(m, c, _), k| {
                let w = win(m, c, k);
                // `get_action` answers the order at +0x40 when +0x3c is
                // set; its type is the word at +0x48.
                rd(w, 0x3c) != 0 && rd(w, 0x48) == 0x1a
            },
            |(_, _, log), k| log.push(format!("s4:{}+0x0", tag(k))),
            |(_, _, log), k| log.push(format!("s5:{}+0x0,0", tag(k))),
            |(m, c, _), k| {
                let w = win(m, c, k);
                let od = i32::from(rd(w, 0x90) as i16);
                if od < 0 {
                    return None;
                }
                let addr = rd(&c[4], 4 * od as usize);
                let next = ((addr - UNITS) / 0x200) as usize + 1;
                (rd(win(m, c, next), 8) & 1 != 0).then_some(next)
            },
        );
        if log.is_empty() {
            log.push("-".into());
        }
        0
    });
}
