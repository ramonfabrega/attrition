//! `UnitData::is_gathering_at@00608880` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_gathering_at@00608880`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_gathering_at` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 144 random bytes of object, 3 argument(s), vtable slot +0xbc (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; stub 2's answer points at 48 random bytes less `0` (placed, null a third of the rows); stub 4's answer points at 40 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, -1] (90%); stub 3 answered from [7, 7, 3] (90%); stub 0 answers 2 values in call order, printed after the answers; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_gathering_at") else {
        return;
    };
    let obj = super::Obj {
        name: "is_gathering_at",
        full: "UnitData::is_gathering_at@00608880",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` o, `a[1]` who, `a[2]` strict; `is_on_map` (vslot
        // `+0xbc`) answers `a[8]` then `a[9]`, or is inline (`a[10]`,
        // the base laid: `inside_up` `+0x82`'s sign bit); `get_inside`
        // answers `a[4]` and leaves its out-word -1 (a stub writes none);
        // `get_action` answers the order `p[0]` (`a[5]`, or null), whose
        // type (`+0x10`) is `a[6]` and whose target record (`+0xe4`) is
        // `p[1]`: `+8` o, `+0xc` who, `+0x27` arrived. The type `p[3]`.
        let ty = rd(&p[3], 4);
        let inline = a[10] != 0;
        let mut asked = 0usize;
        let on = |asked: &mut usize| {
            *asked += 1;
            if inline {
                u16::from_le_bytes([m[0x82], m[0x83]]) >> 15 != 0
            } else {
                a[8 + (*asked - 1).min(1)] != 0
            }
        };
        let cell = std::cell::RefCell::new(&mut asked);
        i64::from(crate::orders::gathering_at_steps(
            matches!(ty, 0x34 | 0x35),
            || on(&mut cell.borrow_mut()),
            || a[0] == a[4] && a[1] == -1,
            a[2] != 0,
            matches!(ty, 0x32..=0x35),
            || {
                (a[5] != 0 && a[6] == 7).then(|| {
                    (
                        rd(&p[1], 0xc) == a[1] && rd(&p[1], 8) == a[0],
                        i32::from(p[1][0x27]),
                    )
                })
            },
        ))
    });
}
