//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_obj.py` on its row `valid_filter_case19` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the global cell 0x00c0ab10 points at 2624 random bytes; callee 0x006763c0 (4 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [-1, 0, 1, 5] (90%); stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("valid_filter_case19") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_filter",
        full: "Search::valid_filter@0067dbb0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Arm 19: `0x6763c0` (stub 0, on the entry p1 of the array the cell at 0xc0ab10 points at, handed
            // `(p2, p0, who, 0)`) non-zero, then p1 - 0x275 picks through the byte table at 0x67e5e8: 0 and 12
            // are arm 23's (the +0x24 word zero), 1 is arm 24's (1), 10 is arm 25's (slot +0x1c, +0x3c, +0x4c
            // and +0x178 of `objects[who][p0]`, here with `ebx` = p0), the rest arm 26's (0).
            const BYTES: [u8; 13] = [0, 3, 1, 3, 3, 3, 3, 3, 3, 3, 2, 3, 0];
            let who = a[9] as usize;
            let (i0, i1, i2) = (a[0] as usize, a[1], a[2]);
            let w = format!("p{}+0x0", 5 * who + 1 + i0);
            let ecx = rd(&c[0], 4 * i1 as usize);
            log.push(format!("{ecx},{i2},{i0},{who},0"));
            let dash = |log: &mut Vec<String>, n: usize| {
                for _ in 0..n {
                    log.push("-".into());
                }
            };
            let k = i1 - 0x275;
            if a[4] == 0 || !(0..=12).contains(&k) {
                dash(&mut *log, 4);
                0
            } else {
                match BYTES[k as usize] {
                    0 => {
                        dash(&mut *log, 4);
                        i64::from(rd(&p[5 * who + 1 + i0], 0x24) == 0)
                    }
                    1 => {
                        dash(&mut *log, 4);
                        1
                    }
                    3 => {
                        dash(&mut *log, 4);
                        0
                    }
                    _ => {
                        log.push(w.clone());
                        if a[5] == 0 {
                            dash(&mut *log, 3);
                            0
                        } else if a[7] == 0 {
                            log.push(w);
                            log.push("p10+0x0".into());
                            log.push("-".into());
                            0
                        } else {
                            log.push(format!("{w}/{w}"));
                            log.push("p10+0x0".into());
                            log.push("p10+0x0".into());
                            i64::from(a[8] > 0)
                        }
                    }
                }
            }
        }
    });
}
