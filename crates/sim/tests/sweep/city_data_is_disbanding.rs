//! `CityData::is_disbanding@007380a0` under the emulator against the port (item 2863; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::is_disbanding@007380a0`, run by `tools/emu/sweep_obj.py` on its row `city_data_is_disbanding` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 184 random bytes of object, 0 argument(s), vtable slot +0x3c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x190 (2 args) stubbed; the word at `0xc0ab84+i8(m,0x5e)*0x1c` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `m+0x74` points at 20 random bytes less `0` (placed); the word at `p6+0` points at 768 random bytes less `0` (placed); the word at `p6+4` points at 768 random bytes less `0` (placed); the word at `p6+8` points at 768 random bytes less `0` (placed); the word at `p6+12` points at 768 random bytes less `0` (placed); the word at `p6+16` points at 768 random bytes less `0` (placed); the word at `p7+0x18` points at 708 random bytes less `0` (placed); the word at `p8+0x18` points at 708 random bytes less `0` (placed); the word at `p9+0x18` points at 708 random bytes less `0` (placed); the word at `p10+0x18` points at 708 random bytes less `0` (placed); the word at `p11+0x18` points at 708 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 0 answers `p7+j`, the data of the object its `ecx` is in `p1+j` (j < 5), linked at `+0x74`; stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 3 answers 0..10 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_is_disbanding") else {
        return;
    };
    let obj = super::Obj {
        name: "is_disbanding",
        full: "CityData::is_disbanding@007380a0",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // The walk of the city's objects from the short at `m+8`: object `o` is `p[1 + o]`, its data record
            // `p[7 + o]` (what slot +0x3c answers), whose short at +0x74 is the next link. An object counts when
            // its flag byte (+8) has bit 0, slot +0x20 and +0x4c answer yes (`a[4..10]`, `a[10..16]` in call
            // order) and the data's pointed record (`p[12 + o]`) has bit 4 of +0x2c0 clear; it adds slot +0x190's
            // answer (`a[16..22]`).
            let (mut n1, mut n2, mut n3) = (0usize, 0usize, 0usize);
            let next = |base: usize, n: &mut usize| {
                let v = a[base + (*n).min(5)];
                *n += 1;
                v
            };
            let mut total: i32 = 0;
            let mut o = i16::from_le_bytes([m[8], m[9]]);
            while o >= 0 {
                let k = o as usize;
                let link = i16::from_le_bytes([p[7 + k][0x74], p[7 + k][0x75]]);
                if p[1 + k][8] & 1 != 0
                    && next(4, &mut n1) != 0
                    && next(10, &mut n2) != 0
                    && p[12 + k][0x2c0] & 0x10 == 0
                {
                    total = total.wrapping_add(next(16, &mut n3));
                }
                o = link;
            }
            i64::from(total)
        }
    });
}
