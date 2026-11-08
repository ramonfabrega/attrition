//! `Unit::add_think_order@005e3df0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_think_order@005e3df0`, run by `tools/emu/sweep_obj.py` on its row `unit_add_think_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), callee 0x00730ac0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 0 answers the address of 192 random bytes, a `p` window after the places; stub 1 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_think_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_think_order",
        full: "Unit::add_think_order@005e3df0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The made order is `p[2]` (stub 1's answer, through `p[1]`'s
        // vtable, `p[1]` being stub 0's), carried as its flags byte
        // `+4`; the link, `clear_partial_path` and `update_action`
        // are recorded, in call order (`--record-order`, item 2341); the list's head node `+0xdc` is `p[0]`, and
        // the rotation `head = head->next` (`+0`) is the engine's
        // iterator, held here as bytes (the port's is `new_head`).
        let mut cx = (m, p, &mut *log);
        crate::orders::add_think_steps(
            &mut cx,
            |(_, p, log)| {
                log.push(format!("s0:m+0x0,{}", crate::orders::index::THINK));
                log.push("s1:p1+0x0".into());
                p[2][4]
            },
            |f| f,
            |(_, p, log), f| {
                p[2][4] = f;
                log.push("s2:m+0xcc,p2+0x0".into());
            },
            |(_, _, log)| log.push("s3:m+0x0".into()),
            |(m, p, _)| {
                let next = rd(&p[0], 0);
                wr(m, 0xdc, next);
            },
            |(_, _, log)| log.push("s4:m+0x0".into()),
        );
        0
    });
}
