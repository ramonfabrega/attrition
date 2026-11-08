//! `Categories::find_key@006f20e0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Categories::find_key@006f20e0`, run by `tools/emu/sweep_obj.py` on its row `categories_find_key` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 the place; 4 random bytes at 0x00ee10e0; callee 0x00a1f140 (1 args) stubbed; the word at `m+0x10` points at 768 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("categories_find_key") else {
        return;
    };
    let obj = super::Obj {
        name: "find_key",
        full: "Categories::find_key@006f20e0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `m+0x18` the count, `m+0x10` the list (`p[0]`), 0x58 a record, its key at `+0x14`;
        // `a[0]` the key (the callee's `ecx`), `a[2..8]` the callee's answers in call order.
        // Nothing found reaches the error report, skipped by the `ignore_always` byte at `c0+2`.
        let n = rd(m, 0x18);
        let mut calls: Vec<String> = Vec::new();
        let mut got = -1;
        for i in 0..n.max(0) {
            calls.push(format!("{},p0+{:#x}", a[0], 0x14 + 0x58 * i));
            if a[2 + i as usize] as u8 != 0 {
                got = i;
                break;
            }
        }
        assert!(
            got >= 0 || c[0][2] != 0,
            "the error report arm is not drawn"
        );
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        i64::from(got)
    });
}
