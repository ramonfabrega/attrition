//! `Object::die@00647080` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Object::die@00647080`, run by `tools/emu/sweep_obj.py` on its row `object_die` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 3 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c0618c points at 320 random bytes; 4 random bytes at 0x00c0a888; vtable slot +0x150 (3 args) stubbed; the word at `c0+0x12c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `p0+16` points at 72 random bytes less `0` (placed); the word at `p0+20` points at 72 random bytes less `0` (placed); the word at `m+0x18` points at 528 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("object_die") else {
        return;
    };
    let obj = super::Obj {
        name: "die",
        full: "Object::die@00647080",
        cells: 2,
        calls: false,
        eax: false,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `Object::die@00647080` after its stubbed `close` (slot 0x150): c0 the objects record (array at +0x12c, count +0x120), p0 the array, p1..p6 its records, p7 the type (`this+0x18`), c1 the word `nuke_effect+0x108`.
        let n = rd(&c[0], 0x120);
        let ammo: Vec<crate::fight::DieAmmo> = (0..6usize)
            .take(usize::try_from(n.max(0)).unwrap_or(0))
            .map(|k| crate::fight::DieAmmo {
                flags: p[1 + k][4],
                who: rd(&p[1 + k], 0x3c),
                idx: rd(&p[1 + k], 0x40),
                total: rd(&p[1 + k], 0x28),
                cur: rd(&p[1 + k], 0x24),
            })
            .collect();
        let hold = u16::from_le_bytes([m[0x32], m[0x33]]);
        let sh = i16::from_le_bytes([m[0xa], m[0xb]]);
        let h = crate::fight::die_hold_steps(
            hold,
            m[9],
            sh,
            rd(&p[7], 0x1fc) != 0,
            rd(&c[1], 0),
            &ammo,
        );
        m[0x32..0x34].copy_from_slice(&h.to_le_bytes());
        0
    });
}
