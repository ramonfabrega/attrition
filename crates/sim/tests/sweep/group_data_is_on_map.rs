//! `GroupData::is_on_map@0070c450` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::is_on_map@0070c450`, run by `tools/emu/sweep_obj.py` on its row `group_data_is_on_map` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2508 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 144 random bytes less `0` (placed); the word at `p5+4` points at 144 random bytes less `0` (placed); the word at `p5+8` points at 144 random bytes less `0` (placed); the word at `p5+12` points at 144 random bytes less `0` (placed); the stubbed vtable laid at `p6+0` half the rows; the stubbed vtable laid at `p7+0` half the rows; the stubbed vtable laid at `p8+0` half the rows; the stubbed vtable laid at `p9+0` half the rows; stub 0 answers 4 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_is_on_map") else {
        return;
    };
    let obj = super::Obj {
        name: "is_on_map",
        full: "GroupData::is_on_map@0070c450",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // Nothing in the list answers 0; `buildings` answers 1; else the first object whose slot +8
            // (stub 0, a[3..]) answers non-zero and whose unit record (ptrtab 1, p[6 + i]) is a captain
            // and on the map answers 1. A unit record whose vtable word is `UnitData::vftable` reads the
            // bit inline (`is_captain`: u16 +0x8e bit 15, `is_on_map`: u16 +0x82 bit 15); any other is the
            // stubbed vtable and its slots +0xe8 (stub 1, a[7..]) and +0xbc (stub 2, a[11..]) answer.
            let num = rd(m, 12);
            if num == 0 {
                0
            } else if m[0x49] != 0 {
                1
            } else {
                let (mut c0, mut c1, mut c2) = (0usize, 0usize, 0usize);
                let mut hit = 0i64;
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                            .expect("an index");
                    let r0 = a[3 + c0];
                    c0 += 1;
                    if r0 == 0 {
                        continue;
                    }
                    let u = &p[6 + i];
                    let real = rd(u, 0) == 0xb41b08;
                    let cap = if real {
                        u[0x8f] >> 7 != 0
                    } else {
                        c1 += 1;
                        a[7 + c1 - 1] != 0
                    };
                    if !cap {
                        continue;
                    }
                    let on = if real {
                        u[0x83] >> 7 != 0
                    } else {
                        c2 += 1;
                        a[11 + c2 - 1] != 0
                    };
                    if on {
                        hit = 1;
                        break;
                    }
                }
                hit
            }
        }
    });
}
