//! `UnbuiltFort::is_started@0073bb40` under the emulator against the port (item 2223; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnbuiltFort::is_started@0073bb40`, run by `tools/emu/sweep_obj.py` on its row `unbuilt_fort_is_started` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 8 random bytes of object, 0 argument(s), vtable slot +0xac (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; the word at `0xc0618c` points at 4 random bytes less `0x14+i8(m,2)*0x1c` (placed); the word at `p0+0` points at 4 random bytes less `i16(m,0)*4` (placed); the word at `p1+0` points at 12 random bytes less `0` (placed); stub 0 answers the address of 12 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; the stubbed vtable laid at `p3+0` half the rows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unbuilt_fort_is_started") else {
        return;
    };
    let obj = super::Obj {
        name: "is_started",
        full: "UnbuiltFort::is_started@0073bb40",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        if rd(&p[3], 0) == 0xb42174 {
            i64::from(p[3][8] & 2)
        } else {
            i64::from(a[1])
        }
    });
}
