//! `UnitData::get_gpiece@00616d70` under the emulator against the port (item 2900; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::get_gpiece@00616d70`, run by `tools/emu/sweep_obj.py` on its row `unit_data_get_gpiece_vt` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x0090c030 (7 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0xf4` points at 4 random bytes less `0` (placed); the word at `p0+0` points at 144 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `0xc06214` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 32 random bytes less `0` (placed); the word at `p4+4` points at 32 random bytes less `0` (placed); the word at `p4+8` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 32 random bytes less `0` (placed); the word at `p8+4` points at 32 random bytes less `0` (placed); the word at `p8+8` points at 32 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); the word at `p7+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); the word at `p11+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_get_gpiece_vt") else {
        return;
    };
    let obj = super::Obj {
        name: "get_gpiece",
        full: "UnitData::get_gpiece@00616d70",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // A set word at this+0xe8 answers the i32 at +0x88 of the record behind the pointer behind this+0xf4 (`p1`). Otherwise
        // slot 0xb8 (or, when it is ObjectData::is, slot 0x60 of the record at this+0x18, `p2`) is asked (62, 0) -- the kept arm
        // is the flag after the answers -- and its whole `eax` != 0 is the fifth argument of `get_unit_gpiece`, handed the
        // graphic-pieces cell (`p3`), the type record's word +4 (the type behind the record `[player row][short at +0xa]`
        // of the table at 0xc0aec0), the player byte at +9, the short, 0, the flag, 0, 0; its answer is the answer.
        let _ = (&c, &a);
        if rd(m, 0xe8) != 0 {
            log.push("-".into());
            i64::from(rd(&p[1], 0x88))
        } else {
            let (tok, ans) = if a[3] != 0 {
                ("s2:p2+0x0,62,0", a[2])
            } else {
                ("s1:m+0x0,62,0", a[1])
            };
            log.push(tok.into());
            let player = usize::from(m[9]);
            let col = usize::from(u16::from_le_bytes([m[10], m[11]]));
            let ty = &p[12 + 3 * player + col];
            log.push(format!(
                "s0:p3+0x0,{},{},{},0,{},0,0",
                rd(ty, 4),
                player,
                col,
                i32::from(ans != 0)
            ));
            i64::from(a[0])
        }
    });
}
