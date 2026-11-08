//! `SpellType::unpay_cast_costs@00676ff0` under the emulator against the port (item 2271; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::unpay_cast_costs@00676ff0`, run by `tools/emu/sweep_obj.py` on its row `spell_type_unpay_cast_costs` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 472 random bytes of object, 2 argument(s), vtable slot +0xd8 (3 args) stubbed; the word at `0xc0aec0+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 4 random bytes less `0x96` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_unpay_cast_costs") else {
        return;
    };
    let obj = super::Obj {
        name: "unpay_cast_costs",
        full: "SpellType::unpay_cast_costs@00676ff0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let v = i32::from(rd(&p[1], 0) as i16);
            let k = v.min(rd(m, 0x1d0));
            let n = v.wrapping_sub(k) as u16;
            p[1][0..2].copy_from_slice(&n.to_le_bytes());
            0
        }
    });
}
