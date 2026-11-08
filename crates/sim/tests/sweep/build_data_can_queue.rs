//! `BuildData::can_queue@0062da00` under the emulator against the port (item 2170; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::can_queue@0062da00`, run by `tools/emu/sweep_obj.py` on its row `build_data_can_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), 4 random bytes at 0x00cab3a4; vtable slot +0x84 (4 args) stubbed; callee 0x0062da50 (1 args) stubbed; the word at `0xe85ddc` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p1+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_can_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "can_queue",
        full: "BuildData::can_queue@0062da00",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if a[1] == 0 {
            wr(&mut c[0], 0, 5);
            0
        } else {
            i64::from(a[2])
        }
    });
}
