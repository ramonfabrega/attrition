//! `BuildTypeData::get_garrison_limit@00633f50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildTypeData::get_garrison_limit@00633f50`, run by `tools/emu/sweep_obj.py` on its row `build_type_data_get_garrison_limit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 720 random bytes of object, 1 argument(s), the global cell 0x00c061e4 points at 3048 random bytes; vtable slot +0x60 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; callee 0x006db810 (1 args) stubbed; slot +0xfc laid as 0x00472ba0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_type_data_get_garrison_limit") else {
        return;
    };
    let obj = super::Obj {
        name: "get_garrison_limit",
        full: "BuildTypeData::get_garrison_limit@00633f50",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // The garrison cap `m+0x2c8`, plus, for a tower (slot +0x60's first answer `a[4]`, of its
        // sequence `a[4..6]`), the leader's fort-garrison level (`has_preq` of 0x2d9, 0x2d8, 0x2d7 in
        // turn, answers `a[6..12]` consumed in call order: 4, 3, else 1 or 2) less one times the
        // constants' `+0xbe0`; and likewise for a fort (the slot +0xfc answer `a[2]`, or slot +0x60's
        // second answer `a[5]` when slot +0xfc is laid as `is_fort`, flag `a[12]`) times `+0xbe4`
        // (item 2844).
        let mut pos = 6;
        let mut level = || {
            let mut next = || {
                pos += 1;
                a[pos - 1] != 0
            };
            if next() {
                4
            } else if next() {
                3
            } else {
                i32::from(next()) + 1
            }
        };
        let mut g = rd(m, 0x2c8);
        if a[4] != 0 {
            g = g.wrapping_add((level() - 1).wrapping_mul(rd(&c[0], 0xbe0)));
        }
        let fort = if a[12] != 0 { a[5] } else { a[2] };
        if fort != 0 {
            g = g.wrapping_add((level() - 1).wrapping_mul(rd(&c[0], 0xbe4)));
        }
        i64::from(g)
    });
}
