//! `UnitData::is_corner@0060a040` under the emulator against the port (item 2621; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_corner@0060a040`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_corner` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 3 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x005de270 (2 args) stubbed; the word at `m+0xf4` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1, 1, 1] (90%); stub 2 answered from [0, 0, 0, 5] (90%); stub 2 answers 4 values in call order, the last from [0], printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_corner") else {
        return;
    };
    let obj = super::Obj {
        name: "is_corner",
        full: "UnitData::is_corner@0060a040",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let _ = (&c, &a[5]);
        let (s0, s1, seq) = (a[3], a[4], &a[6..10]);
        let mut r = 0;
        let mut cl: Vec<String> = Vec::new();
        if s0 != 0 && s1 != 0 {
            let n = i32::from(m[0xb5] as i8).max(0) as usize;
            let mut k = 0;
            for i in 0..n {
                let g = rd(&p[0], 4 * i);
                if g == 0 {
                    continue;
                }
                let ans = seq[k.min(3)];
                k += 1;
                cl.push(format!("{g},{},{}", a[0], a[1]));
                if ans != 0 {
                    r = ans;
                    break;
                }
            }
        }
        log.push(if cl.is_empty() {
            "-".into()
        } else {
            cl.join("/")
        });
        i64::from(r)
    });
}
