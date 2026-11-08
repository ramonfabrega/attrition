//! `BuildData::can_queue@004d2030` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::can_queue@004d2030`, run by `tools/emu/sweep_obj.py` on its row `build_data_can_queue_4d2030` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 1 argument(s), 4 random bytes at 0x00cab3a4; vtable slot +0x84 (4 args) stubbed; callee 0x0062da50 (1 args) stubbed; the word at `0xe85ddc` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_can_queue_4d2030") else {
        return;
    };
    let obj = super::Obj {
        name: "can_queue",
        full: "BuildData::can_queue@004d2030",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Type a0 (the types list, the cell 0xe85ddc) asked through its slot
            // +0x84 (the owner byte +9, -1, the short +0xa, 1): a refusal stores
            // QUEUE_COST (5) in queue_fail and answers 0; else could_queue(a0).
            let s = i16::from_le_bytes([m[0xa], m[0xb]]);
            log.push(format!("p1+0x0,{},-1,{s},1", m[9]));
            if a[1] == 0 {
                wr(&mut c[0], 0, 5);
                log.push("-".into());
                0
            } else {
                log.push(format!("m+0x0,{}", a[0]));
                i64::from(a[2])
            }
        }
    });
}
