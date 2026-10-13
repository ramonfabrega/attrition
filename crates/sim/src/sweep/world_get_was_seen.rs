//! `World::get_was_seen@006b1ba0` under the emulator against the port (item 1840).

/// `World::get_was_seen@006b1ba0`, run by `tools/emu/sweep_world_get_was_seen.py` under unicorn
/// (tools/emu): `__thiscall`, two stack pointers to `FCoord` words (x, then y); reads the `GameAccess::world` cell: the byte at `array[xs * y + x]` (`xs` at `+0xc`, array at `+0x160`). Rows `<xs> <x> <y> <val> <decoy>`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_get_was_seen.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let got: i64 = a[3];
        assert_eq!(got, want, "World::get_was_seen — row `{line}`");
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("World::get_was_seen@006b1ba0: {n} rows agree");
}
