//! `CityData::get_building@00739480` under the emulator against the port (item 2863; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `CityData::get_building@00739480`, run by `tools/emu/sweep_obj.py` on its row `city_data_get_building` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 184 random bytes of object, 3 argument(s), vtable slot +0x3c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84+i8(m,0x5e)*0x1c` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `m+0x74` points at 20 random bytes less `0` (placed); the word at `p6+0` points at 768 random bytes less `0` (placed); the word at `p6+4` points at 768 random bytes less `0` (placed); the word at `p6+8` points at 768 random bytes less `0` (placed); the word at `p6+12` points at 768 random bytes less `0` (placed); the word at `p6+16` points at 768 random bytes less `0` (placed); the word at `p7+0x18` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 32 random bytes less `0` (placed); the word at `p10+0x18` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 0, 1] (90%); stub 4 answered from [0, 0, 1] (90%); the word at `p7+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p8+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p9+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p10+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p11+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers `p7+j`, the data of the object its `ecx` is in `p1+j` (j < 5), linked at `+0x74`; stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 4 answers 6 values in call order, printed after the answers; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("city_data_get_building") else {
        return;
    };
    let obj = super::Obj {
        name: "get_building",
        full: "CityData::get_building@00739480",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 800,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // As `is_disbanding`: the objects from `m+8` are `p[1 + o]`, their data `p[7 + o]`. A data record
            // counts when its flag byte (+8) has bit 0 and slots +0x20, +0x4c answer yes (`a[8..14]`, `a[14..20]`
            // in call order), and then the ask: a data record laid with `Build::vftable` (0xb42174) asks slot
            // +0x60 of the record at +0x18 (`p[12 + o]`, `a[26..32]`), any other slot +0xb8 (`a[20..26]`). The
            // first that answers yes is the index returned; the chain ending answers -1.
            let (mut n1, mut n2, mut n3, mut n4) = (0usize, 0usize, 0usize, 0usize);
            let next = |base: usize, n: &mut usize| {
                let v = a[base + (*n).min(5)];
                *n += 1;
                v
            };
            let mut o = i16::from_le_bytes([m[8], m[9]]);
            let mut found: i64 = -1;
            while o >= 0 {
                let k = o as usize;
                let link = i16::from_le_bytes([p[7 + k][0x74], p[7 + k][0x75]]);
                if p[7 + k][8] & 1 != 0 && next(8, &mut n1) != 0 && next(14, &mut n2) != 0 {
                    let yes = if rd(&p[7 + k], 0) as u32 == 0x00b4_2174 {
                        next(26, &mut n4) != 0
                    } else {
                        next(20, &mut n3) != 0
                    };
                    if yes {
                        found = i64::from(o);
                        break;
                    }
                }
                o = link;
            }
            found
        }
    });
}
