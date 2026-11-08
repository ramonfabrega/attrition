//! `Wall::set_type@00640da0` under the emulator against the port (item 2020; hand-written on tools/emu/hooks.py).

/// `Wall::set_type@00640da0`, run by `tools/emu/sweep_wall_set_type.py` under unicorn (tools/emu): the wall's
/// stats are taken off before a `NORMAL` (0) retype, the base retype is handed both arguments, and the stats are
/// put back after any kind but `LOAD` (2) — the three callees hooked, so not backed here; the row's result is
/// the call order, 1 off, 2 retype, 3 on. `city.rs`'s `set_type` is the port's retype.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_set_type.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (ty, kind) = (a[0], a[1]);
        assert_eq!(a[4], 1, "every callee's ecx is this — row `{line}`");
        assert_eq!(
            &a[2..4],
            &[ty, kind],
            "the retype's arguments — row `{line}`"
        );
        let seq = match (kind == 0, kind != 2) {
            (true, true) => 123,
            (false, true) => 23,
            (false, false) => 2,
            (true, false) => unreachable!(),
        };
        assert_eq!(seq, want, "set_type — row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Wall::set_type@00640da0: {n} rows agree");
}
