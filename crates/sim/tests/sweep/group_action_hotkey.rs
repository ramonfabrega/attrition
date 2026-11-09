//! `Group::action_hotkey@006fa7a0` under the emulator against the port (item 2644; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::action_hotkey@006fa7a0`, run by `tools/emu/sweep_obj.py` on its row `group_action_hotkey` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 1 argument(s), the global cell 0x00c0afe0 points at 10496 random bytes; callee 0x00715120 (0 args) stubbed; callee 0x007152f0 (1 args) stubbed; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_action_hotkey") else {
        return;
    };
    let obj = super::Obj {
        name: "action_hotkey",
        full: "Group::action_hotkey@006fa7a0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let o = a[0] as usize * 0x9fc;
            log.push(format!("c0+{:#x},m+0x0", o));
            log.push(format!("c0+{:#x},m+0x0,1", o));
            wr(&mut c[0], o + 0x9d8, 0);
            a[2] as i64
        }
    });
}
