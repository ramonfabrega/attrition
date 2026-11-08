//! `BuildData::get_queue@0062d280` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::get_queue@0062d280`, run by `tools/emu/sweep_obj.py` on its row `build_data_get_queue` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 144 random bytes of object, 1 argument(s), `p` windows in order: p0..p13 the place(s), in flag order; 4 random bytes at 0x00c061e0; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x006db6c0 (0 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 144 random bytes less `0` (placed); the word at `p0+4` points at 144 random bytes less `0` (placed); the word at `p0+8` points at 144 random bytes less `0` (placed); the word at `p0+12` points at 144 random bytes less `0` (placed); the word at `p0+16` points at 144 random bytes less `0` (placed); the word at `p0+20` points at 144 random bytes less `0` (placed); the word at `p1+0x8c` points at 120 random bytes less `0` (placed); the word at `p2+0x8c` points at 120 random bytes less `0` (placed); the word at `p3+0x8c` points at 120 random bytes less `0` (placed); the word at `p4+0x8c` points at 120 random bytes less `0` (placed); the word at `p5+0x8c` points at 120 random bytes less `0` (placed); the word at `p6+0x8c` points at 120 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answers -1..5 every row; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_get_queue") else {
        return;
    };
    let obj = super::Obj {
        name: "get_queue",
        full: "BuildData::get_queue@0062d280",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `is(0x1b3)` is the type record's slot `+0x60` (`a[1]`) when the object's `+0xb8`
        // slot is `ObjectData::is` (`a[4] != 0`), else that slot (`a[2]`); a `true` with
        // `param >= queued` (`+0x82`) asks `get_first_library` (`a[3]`), a negative answer
        // falling back to the object's own index `+0xa`, and a different one pays `queued`
        // off the param. The queue entry is then `+0x4` of the 0x14-wide array `+0x8c`
        // of the object table `p[0]`'s record, bounded by that record's `+0x82` and `+0x88`.
        let own = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
        let queued = i32::from(m[0x82]);
        let mut param = a[0];
        let is = if a[4] != 0 { a[1] } else { a[2] };
        let mut lib = own;
        if is != 0 && param >= queued {
            lib = if a[3] < 0 { own } else { a[3] };
            if lib != own {
                param -= queued;
            }
        }
        let rec = &p[1 + lib as usize];
        if param < i32::from(rec[0x82]) && param < rd(rec, 0x88) {
            let arr = &p[7 + lib as usize];
            let at = param as usize * 0x14 + 4;
            i64::from(i16::from_le_bytes([arr[at], arr[at + 1]]))
        } else {
            -1
        }
    });
}
