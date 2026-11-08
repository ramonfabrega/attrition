//! `Balance::lookup_absolute_name@00582910` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Balance::lookup_absolute_name@00582910`, run by `tools/emu/sweep_obj.py` on its row `balance_lookup_absolute_name` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0; argument 0 names 16 random bytes; callee 0x005829b0 (4 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("balance_lookup_absolute_name") else {
        return;
    };
    let obj = super::Obj {
        name: "lookup_absolute_name",
        full: "Balance::lookup_absolute_name@00582910",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Balance::lookup_absolute_name`: the absolute index picks one of five name tables
        // (0x160, 5, 2, 8 wide, the rest in the last) and `lookup_name` is asked the offset
        // with the table number as `ecx` and again as its last two arguments.
        let n = a[1];
        let (off, kind) = if n < 0x160 {
            (n, 0)
        } else if n - 0x160 < 5 {
            (n - 0x160, 1)
        } else if n - 0x165 < 2 {
            (n - 0x165, 2)
        } else if n - 0x167 < 8 {
            (n - 0x167, 3)
        } else {
            (n - 0x16f, 4)
        };
        log.push(format!("{kind},p0+0x0,{off},{kind},{kind}"));
        i64::from(a[0])
    });
}
