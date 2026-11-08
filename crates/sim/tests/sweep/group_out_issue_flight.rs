//! `GroupOut::issue_flight@00708b10` under the emulator against the port (item 2421; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupOut::issue_flight@00708b10`, run by `tools/emu/sweep_obj.py` on its row `group_out_issue_flight` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 6 argument(s), the global cell 0x00c061e8 points at 2304 random bytes; callee 0x00941d40 (7 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_out_issue_flight") else {
        return;
    };
    let obj = super::Obj {
        name: "issue_flight",
        full: "GroupOut::issue_flight@00708b10",
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
