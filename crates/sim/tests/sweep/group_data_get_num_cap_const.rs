//! `GroupData::get_num_cap_const@0070f960` under the emulator against the port (item 2768; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GroupData::get_num_cap_const@0070f960`, run by `tools/emu/sweep_obj.py` on its row `group_data_get_num_cap_const` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2508 random bytes of object, 0 argument(s), vtable slot +0xe8 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 20 values in call order, printed after the answers; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_data_get_num_cap_const") else {
        return;
    };
    let obj = super::Obj {
        name: "get_num_cap_const",
        full: "GroupData::get_num_cap_const@0070f960",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `GroupData::list` (i16 at 0x8cc) names an object through `objects[who]`'s pointer array
            // (p[0]); an object counts when its flag byte +8 has bit 0 set and its vtable slot +0xe8
            // (stub 0, answers a[1..]) answers non-zero.
            let num = rd(m, 12);
            if num <= 0 {
                0
            } else {
                let mut n = num;
                let mut calls = 0usize;
                for i in (0..usize::try_from(num).expect("a count")).rev() {
                    let idx =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cd + 2 * i]]))
                            .expect("an index");
                    let o = &p[1 + idx];
                    if o[8] & 1 == 0 {
                        n -= 1;
                    } else {
                        let r = a[1 + calls];
                        calls += 1;
                        if r == 0 {
                            n -= 1;
                        }
                    }
                }
                i64::from(n)
            }
        }
    });
}
