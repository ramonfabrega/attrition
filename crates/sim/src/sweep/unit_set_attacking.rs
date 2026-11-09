//! `Unit::set_attacking@005ff5b0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::set_attacking@005ff5b0`, run by `tools/emu/sweep_obj.py` on its row `unit_set_attacking` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 72 random bytes of object, 1 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; vtable slot +0x164 (0 args) stubbed; callee 0x006b55c0 (3 args) stubbed; stub 1 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_set_attacking") else {
        return;
    };
    let obj = super::Obj {
        name: "set_attacking",
        full: "Unit::set_attacking@005ff5b0",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` the victim `who`, `a[2]` `is_seen`'s answer; `+8` the
        // flags, `+0x40` the seen-by bits. The leader tables the head
        // writes (`0xe3a424`, `+0x94`) are the port's SEAM, unwindowed.
        let (mut f, mut v) = (m[8], m[0x40]);
        crate::combat::fight::attacking_bits(&mut f, &mut v, a[0] as crate::Player, || a[2] != 0);
        m[8] = f;
        m[0x40] = v;
        0
    });
}
