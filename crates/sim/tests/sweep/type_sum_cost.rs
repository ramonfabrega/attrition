//! `Type::sum_cost@00668080` under the emulator against the port (item 2211; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Type::sum_cost@00668080`, run by `tools/emu/sweep_obj.py` on its row `type_sum_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 3 argument(s), callee 0x006e33a0 (2 args) stubbed; vtable slot +0x78 (7 args) stubbed; stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("type_sum_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "sum_cost",
        full: "Type::sum_cost@00668080",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        i64::from(if a[3] != 0 { a[4].wrapping_mul(6) } else { 0 })
    });
}
