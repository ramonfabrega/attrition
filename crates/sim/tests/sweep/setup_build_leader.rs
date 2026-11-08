//! `Setup::build_leader@005abc70` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Setup::build_leader@005abc70`, run by `tools/emu/sweep_obj.py` on its row `setup_build_leader` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 3 argument(s), callee 0x006e3930 (3 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `@0xe3a390`, slices +0xc:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("setup_build_leader") else {
        return;
    };
    let obj = super::Obj {
        name: "build_leader",
        full: "Setup::build_leader@005abc70",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The leader's init (stub 0) on leaders row a0, handed a0, the row's +0xc word and a2;
            // the call split by the table's row, and the answer 0.
            let k = a[0] as usize;
            for row in 0..8 {
                log.push(if row == k {
                    format!("r+0x0,{},{},{}", a[0], rd(&p[k], 0), a[2])
                } else {
                    "-".into()
                });
            }
            log.push("-".into());
            0
        }
    });
}
