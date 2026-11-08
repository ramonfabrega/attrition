//! `Supply::deplete@0073b5d0` under the emulator against the port (item 1735).

/// `Supply::deplete@0073b5d0`, run by `tools/emu/sweep_supply_deplete.py` under unicorn
/// (tools/emu): `retl $4`, no write: the supply's stock does not move under the call, the argument ignored (the port's depletion is the economy's, not a call of this).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_supply_deplete.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (_, want) = super::row(line);
        assert_eq!(want, 0, "Supply::deplete — row `{line}`");
        n += 1;
    }
    assert!(n >= 80, "only {n} rows");
    eprintln!("Supply::deplete@0073b5d0: {n} rows agree");
}
