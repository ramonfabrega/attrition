//! `UnitOut::issue_unqueue@00607bc0` under the emulator against the port (item 2362; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitOut::issue_unqueue@00607bc0`, run by `tools/emu/sweep_obj.py` on its row `unit_out_issue_unqueue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 2112 random bytes; callee 0x00942ce0 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_out_issue_unqueue") else {
        return;
    };
    let obj = super::Obj {
        name: "issue_unqueue",
        full: "UnitOut::issue_unqueue@00607bc0",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The game's semaphore byte (+0x820 of the record cell 0xc061e8): bit 0x10 clear → the
            // command manager's unqueue (stub 0) is handed the unit twice, `ecx` left the unit.
            if c[0][0x820] & 0x10 == 0 {
                log.push("m+0x0,m+0x0,m+0x0".into());
            } else {
                log.push("-".into());
            }
            0
        }
    });
}
