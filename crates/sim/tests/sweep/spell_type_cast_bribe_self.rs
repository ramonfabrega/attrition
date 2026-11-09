//! `SpellType::cast_bribe@00672d00` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::cast_bribe@00672d00`, run by `tools/emu/sweep_obj.py` on its row `spell_type_cast_bribe_self` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 472 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; vtable slot +0xd8 (3 args) stubbed; the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+4*a[0]` points at 160 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_cast_bribe_self") else {
        return;
    };
    let obj = super::Obj {
        name: "cast_bribe",
        full: "SpellType::cast_bribe@00672d00",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The self-target arm (`param_2 == param_4`): the refund through slot 0xd8, then the unit's counter at +0x96
            // (a short) is lowered by the smaller of itself and `this+0x1d0`.
            let (p1, p2) = (a[0], a[1]);
            log.push(format!("s0:m+0x0,{p2},{p1},-1"));
            let sh = i16::from_le_bytes([p[1][0x96], p[1][0x97]]) as i32;
            let lim = rd(m, 0x1d0);
            let take = if lim < sh { lim } else { sh };
            let v = (sh as u16).wrapping_sub(take as u16);
            p[1][0x96..0x98].copy_from_slice(&v.to_le_bytes());
            0
        }
    });
}
