//! `BuildData::get_garrison_arrows@0062e3d0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::get_garrison_arrows@0062e3d0`, run by `tools/emu/sweep_obj.py` on its row `build_data_get_garrison_arrows` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x120 (0 args) stubbed; vtable slot +0x1c0 (0 args) stubbed; callee 0x0064dda0 (2 args) stubbed; vtable slot +0x1bc (0 args) stubbed; the word at `m+0x18` points at 720 random bytes less `0` (placed); slot +0x1bc laid as 0x0062e2f0 or its stub, a flag after the answers; stub 0 answers -20..80 every row; stub 1 answers 0..2 every row; stub 2 answers -5..20 every row; stub 3 answers -3..10 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_get_garrison_arrows") else {
        return;
    };
    let obj = super::Obj {
        name: "get_garrison_arrows",
        full: "BuildData::get_garrison_arrows@0062e3d0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `BuildData::get_garrison_arrows`: `n = (slot 0x120 + 5) / 10` shots per arrow (0 answers 0);
        // `ebx` is slot 0x1c0's answer; the arrows inside (`count_inside(0x1a, 0)`, `0x64dda0`) get
        // `n / 2` added when `ebx == 0`, are divided by `n`, and are capped at the type's most shots:
        // the word at `+0x2c4` of the type (`p[0]`) when slot 0x1bc is the base `most_shots`
        // (`a[4] == 1`, the stub never called), else slot 0x1bc's answer (called, `ecx` this).
        let (s0, s1, s2, s3, base) = (a[0], a[1], a[2], a[3], a[4] == 1);
        let n = (s0 + 5) / 10;
        if n == 0 {
            log.push("-".to_string());
            log.push("-".to_string());
            0i64
        } else {
            log.push("m+0x0,26,0".to_string());
            let mut inside = s2;
            if s1 == 0 {
                inside += n / 2;
            }
            let mut q = inside / n;
            let cap = if base {
                log.push("-".to_string());
                rd(&p[0], 0x2c4)
            } else {
                log.push("m+0x0".to_string());
                s3
            };
            if cap + s1 < q + s1 {
                q = cap;
            }
            i64::from(s1 + q)
        }
    });
}
