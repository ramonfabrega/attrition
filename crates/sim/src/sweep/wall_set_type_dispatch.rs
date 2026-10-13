//! `Wall::set_type@004723b0` under the emulator against the port (item 2038; hand-written on tools/emu/hooks.py).

/// `Wall::set_type@004723b0`, run by `tools/emu/sweep_wall_set_type_dispatch.py` under unicorn (tools/emu): the
/// header's dispatcher. With its own slot `+0x84` the wall's own retype it runs that body inline — the stats off
/// before a `NORMAL` (0) retype, the base retype handed both arguments, the stats back after any kind but `LOAD`
/// (2), the three callees hooked; with the slot overridden, the slot is handed both arguments once. The row's
/// result is the call order, 1 off, 2 retype, 3 on, 4 the slot. `city.rs`'s `set_type` is the port's retype.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_set_type_dispatch.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (mode, ty, kind) = (a[0], a[1], a[2]);
        assert_eq!(a[5], 1, "every callee's ecx is this — row `{line}`");
        assert_eq!(
            &a[3..5],
            &[ty, kind],
            "both arguments handed on — row `{line}`"
        );
        let seq = if mode == 0 {
            4
        } else {
            match (kind == 0, kind != 2) {
                (true, true) => 123,
                (false, true) => 23,
                (false, false) => 2,
                (true, false) => unreachable!(),
            }
        };
        assert_eq!(seq, want, "set_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Wall::set_type@004723b0: {n} rows agree");
}
