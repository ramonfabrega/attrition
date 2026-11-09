//! `BuildData::find_in_queue@0062de70` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::find_in_queue@0062de70`, run by `tools/emu/sweep_obj.py` on its row `build_data_find_in_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 2 argument(s), the global cell 0x00e85ddc points at 16 random bytes; vtable slot +0x60 (2 args) stubbed; the word at `m+0x8c` points at 80 random bytes less `0` (placed); the word at `c0+0` points at 16 random bytes less `0` (placed); the word at `c0+4` points at 16 random bytes less `0` (placed); the word at `c0+8` points at 16 random bytes less `0` (placed); the word at `c0+0xc` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 5] (90%); the stubbed vtable laid at `p1+0`; stub 0 answers 4 values in call order, the last from [5], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_find_in_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "find_in_queue",
        full: "BuildData::find_in_queue@0062de70",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Of the m[0x82] queued records (0x14 apiece at *(this+0x8c), the short at +4 a type index into the types list; the queue is held at least that long), the first whose type answers its slot 0x60 (a1, 1) with non-zero; -1 when none (the stub'"'"'s answers a call in call order, the last repeated).
            let mut calls = Vec::new();
            let mut r = -1;
            for i in 0..m[0x82] as usize {
                let at = 0x14 * i + 4;
                let ty = i16::from_le_bytes([p[0][at], p[0][at + 1]]);
                calls.push(format!("p{}+0x0,{},1", 1 + ty, a[1]));
                if a[2 + 1 + i.min(3)] != 0 {
                    r = i as i32;
                    break;
                }
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            i64::from(r)
        }
    });
}
