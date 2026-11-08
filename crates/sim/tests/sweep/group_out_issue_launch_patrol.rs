//! `GroupOut::issue_launch_patrol@0070adc0` under the emulator against the port (item 2421; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupOut::issue_launch_patrol@0070adc0`, run by `tools/emu/sweep_obj.py` on its row `group_out_issue_launch_patrol` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 6 argument(s), the global cell 0x00c061e8 points at 2304 random bytes; callee 0x00941860 (7 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_out_issue_launch_patrol") else {
        return;
    };
    let obj = super::Obj {
        name: "issue_launch_patrol",
        full: "GroupOut::issue_launch_patrol@0070adc0",
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
            } else {
                log.push(format!(
                    "m+0x0,m+0x0,{},{},{},{},{},{}",
                    a[0], a[1], a[2], a[3], a[4], a[5]
                ));
            }
            0
        }
    });
}
