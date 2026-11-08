//! `Unit::do_think_order@005e5bf0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_think_order@005e5bf0`, run by `tools/emu/sweep_obj.py` on its row `unit_do_think_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 224 random bytes of object, 1 argument(s), callee 0x005e2cb0 (1 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x005f5760 (1 args) stubbed; the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 8 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); stub 1 answered from [0, 0, 3, 1, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_think_order") else {
        return;
    };
    let obj = super::Obj {
        name: "do_think_order",
        full: "Unit::do_think_order@005e5bf0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The list's head node `+0xdc` (`p[0]`, or null), its `+4` the
        // node `p[1]` whose `+8` is the order `p[2]` (type: vslot `+0x10`,
        // `a[2]`) and `+0xc` its byte; the list's cache `+0xd4`, `+0xcc`,
        // `+0xd0` is refreshed from them (the engine's iterator, held
        // here as bytes and not ported). The unit type `p[3]` (`+4`).
        // The log: `kill_current_order`'s `ecx,0`, `think_peasant`'s
        // `ecx,1`.
        let has = rd(m, 0xdc) != 0;
        if has {
            let (n1, ord) = (rd(&p[0], 4), rd(&p[1], 8));
            wr(m, 0xd4, n1);
            wr(m, 0xcc, ord);
            m[0xd0] = p[1][0xc];
        }
        let (mut killed, mut thought) = (false, false);
        crate::orders::think_order_steps(
            &mut (),
            |_| killed = true,
            |_| has.then_some(a[2]),
            |_| matches!(rd(&p[3], 4), 0x32..=0x35),
            |_| thought = true,
        );
        log.push(if killed { "m+0x0,0" } else { "-" }.into());
        log.push(if thought { "m+0x0,1" } else { "-" }.into());
        0
    });
}
