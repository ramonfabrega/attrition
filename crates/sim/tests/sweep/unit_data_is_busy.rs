//! `UnitData::is_busy@0060a370` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_busy@0060a370`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_busy` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 0's answer window; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; callee 0x0060a6f0 (0 args) stubbed; the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed, null a third of the rows); stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 0 answers the address of 48 random bytes, a `p` window after the places; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_busy") else {
        return;
    };
    let obj = super::Obj {
        name: "is_busy",
        full: "UnitData::is_busy@0060a370",
        cells: 0,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // With an order list (`+0xdc`) the cursor is set to its head's second node
        // (cached into +0xd4/+0xcc/+0xd0, node `p5` -> `p6`); then a current order
        // (`+0xcc`) whose type (the answer window `p8`'s +0x20 indexing the type table
        // `p1..p4`) answers slot +0x50 or +0x54 is busy; anything else is
        // `is_entering_or_exiting`'s answer (stub 3).
        if rd(m, 0xdc) != 0 {
            m[0xd4..0xd8].copy_from_slice(&p[5][4..8]);
            m[0xcc..0xd0].copy_from_slice(&p[6][8..12]);
            m[0xd0] = p[6][0xc];
        }
        if rd(m, 0xdc) != 0 && rd(m, 0xcc) != 0 && (a[1] != 0 || a[2] != 0) {
            1
        } else {
            i64::from(a[3])
        }
    });
}
