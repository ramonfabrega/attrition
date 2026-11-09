//! `GroupOut::issue_set_transport@0070ad10` under the emulator against the port (item 2421; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupOut::issue_set_transport@0070ad10`, run by `tools/emu/sweep_obj.py` on its row `group_out_issue_set_transport` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c061e8 points at 2304 random bytes; callee 0x00940a70 (0 args) stubbed; callee 0x0094bb60 (1 args) stubbed; callee 0x0094bae0 (2 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 prints the 5 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_out_issue_set_transport") else {
        return;
    };
    let obj = super::Obj {
        name: "issue_set_transport",
        full: "GroupOut::issue_set_transport@0070ad10",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            if c[0][0x820] & 0x10 != 0 {
                log.push("-".into());
                log.push("-".into());
                log.push("-".into());
                log.push("-".into());
            } else {
                log.push("15269728".into());
                if a[1] == 0 {
                    log.push("-".into());
                    log.push("-".into());
                    log.push("-".into());
                } else {
                    log.push("15269768,m+0x0".into());
                    if a[2] == 0 {
                        log.push("-".into());
                        log.push("-".into());
                    } else {
                        log.push("15269768,sp-0x14,5".into());
                        log.push(format!(
                            "0e{}",
                            a[0].to_le_bytes()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        ));
                    }
                }
            }
            0
        }
    });
}
