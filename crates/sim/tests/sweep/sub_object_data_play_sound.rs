//! `SubObjectData::play_sound@006625c0` under the emulator against the port (item 2621; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SubObjectData::play_sound@006625c0`, run by `tools/emu/sweep_obj.py` on its row `sub_object_data_play_sound` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), callee 0x006624f0 (4 args) stubbed; the word at `0xc0ab84+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 24 random bytes less `0` (placed); the word at `p0+4` points at 24 random bytes less `0` (placed); the word at `p0+8` points at 24 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); the word at `p4+4` points at 24 random bytes less `0` (placed); the word at `p4+8` points at 24 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 24 random bytes less `0` (placed); the word at `p8+4` points at 24 random bytes less `0` (placed); the word at `p8+8` points at 24 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("sub_object_data_play_sound") else {
        return;
    };
    let obj = super::Obj {
        name: "play_sound",
        full: "SubObjectData::play_sound@006625c0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let _ = (&c, &a[1]);
        let who = usize::from(m[9]);
        let o = usize::from(u16::from_le_bytes([m[0xa], m[0xb]]));
        let rec = &p[4 * who + 1 + o];
        let (x, y) = (rd(rec, 0x10) ^ 0x63637, rd(rec, 0x14) ^ 0x63637);
        log.push(format!("m+0x0,{},{x},{y},m+0x0", a[0]));
        i64::from(a[2])
    });
}
