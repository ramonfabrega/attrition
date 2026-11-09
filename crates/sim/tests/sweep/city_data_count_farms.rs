//! `CityData::count_farms@007368c0` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::count_farms@007368c0`, run by `tools/emu/sweep_obj.py` on its row `city_data_count_farms` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 1 argument(s), `p` windows in order: p0..p12 the place(s), in flag order; the global cell 0x00c0a914 points at 768 random bytes; 4 random bytes at 0x00c0a908; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 120 random bytes less `0` (placed); the word at `p0+4` points at 120 random bytes less `0` (placed); the word at `p0+8` points at 120 random bytes less `0` (placed); the word at `p0+12` points at 120 random bytes less `0` (placed); the word at `p0+16` points at 120 random bytes less `0` (placed); the word at `p0+20` points at 120 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; figures p1..p6 link at +0x74 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_count_farms") else {
        return;
    };
    let obj = super::Obj {
        name: "count_farms",
        full: "CityData::count_farms@007368c0",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The city's chain head `+8`, its objects table `p[0]` of six `Build`s
        // `p[1..7]` (vtable `0xb42174`), each's next `+0x74`, `flags` `+8`; a live
        // member asks its type record's `is` (`+0x60`, answering `a[2..8]` in call
        // order) for 0x1a1, and one that answers counts when its farm `+0x78` is
        // out of the farms table (`c[1]` the count, `c[0]` the records, 0xc0 a
        // record, `+0xbd & 1` the farm's flag) or the farm's flag is clear.
        let mut at = i16::from_le_bytes([m[8], m[9]]);
        let (mut k, mut n) = (0usize, 0i64);
        while at >= 0 {
            let b = &p[1 + at as usize];
            at = i16::from_le_bytes([b[0x74], b[0x75]]);
            if b[8] & 1 == 0 {
                continue;
            }
            k += 1;
            if a[2 + (k - 1).min(5)] == 0 {
                continue;
            }
            let idx = i16::from_le_bytes([b[0x78], b[0x79]]);
            if idx < 0 {
                continue;
            }
            let idx = idx as usize;
            let counted = (idx as i32) < rd(&c[1], 0);
            if !counted || c[0][idx * 0xc0 + 0xbd] & 1 == 0 {
                n += 1;
            }
        }
        n
    });
}
