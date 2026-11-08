//! `UnitOut::issue_come_out@00607be0` under the emulator against the port (item 2644; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitOut::issue_come_out@00607be0`, run by `tools/emu/sweep_obj.py` on its row `unit_out_issue_come_out` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x00940a70 (0 args) stubbed; callee 0x0094bae0 (2 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 11 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_out_issue_come_out") else {
        return;
    };
    let obj = super::Obj {
        name: "issue_come_out",
        full: "UnitOut::issue_come_out@00607be0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            log.push("15269728".into());
            if a[0] == 0 {
                log.push("-".into());
                log.push("-".into());
            } else {
                let mut cmd = vec![0x31u8];
                cmd.extend_from_slice(&(m[9] as i32).to_le_bytes());
                cmd.extend_from_slice(&(i16::from_le_bytes([m[0xa], m[0xb]]) as i32).to_le_bytes());
                cmd.extend_from_slice(&m[0x30..0x32]);
                log.push("15269768,sp-0x14,11".into());
                log.push(cmd.iter().map(|b| format!("{b:02x}")).collect::<String>());
            };
            if a[0] == 0 { a[0] as i64 } else { a[1] as i64 }
        }
    });
}
